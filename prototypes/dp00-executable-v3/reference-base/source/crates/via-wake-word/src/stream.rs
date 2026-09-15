//! Host capture in, detections out.
//!
//! Ported from upstream `server/src/voice/wake-word/sherpa-detector.mjs:66-81`,
//! which is nine lines because upstream's host always captured at exactly the
//! rate the feature extractor wanted. VIA's does not, and this is the module
//! that makes the difference invisible to the detector.
//!
//! # What sits between the microphone and the engine
//!
//! ```text
//! PCM16LE bytes ─► FrameBuffer ─► downmix ─► Resampler ─► f32 ─► WakeWordDetector
//!                  (torn frame    (stereo    (48/24 →     (÷32768)
//!                   carried)       → mono)    16 kHz)
//! ```
//!
//! Every stage is `via-audio`'s. Nothing about a sample, a rate or a frame is
//! decided here.
//!
//! # Three things this gets right that a naive version does not
//!
//! **A torn PCM frame is carried, not dropped.** A socket splits a PCM stream
//! at arbitrary byte offsets, so a chunk can end mid-sample.
//! [`FrameBuffer::append_pcm16le`] carries the odd byte into the next call.
//! Dropping it — which is what upstream's stateless base64 decoder does, and
//! correctly, because it decodes one self-contained payload at a time — would
//! shift every subsequent sample by one byte and turn the rest of the chunk
//! into noise.
//!
//! **Downsampling is band-limited, not decimation.** Taking every third sample
//! of a 48 kHz stream to reach 16 kHz folds everything between 8 and 24 kHz
//! back on top of the speech: a fan, sibilance, a laptop's own switching noise.
//! It does not sound like a glitch; it sounds like the wake word not working.
//! `via-audio`'s resampler exists for exactly this, and its module docs argue
//! the case at length.
//!
//! **An empty chunk never reaches the detector.** An FFT resampler accumulates
//! until it has a whole transform, so early calls legitimately produce nothing.
//! Upstream returns early on `!samples.length`
//! (`sherpa-detector.mjs:70`) and so does this — before the call, not inside
//! it, so an engine cannot be handed a zero-length buffer at all.

use via_audio::{
    ChannelCount, FrameBuffer, Resampler, SampleRate, downmix_to_mono_pcm16, pcm16_to_f32_slice,
};

use crate::detect::{Detection, WakeWordDetector};
use crate::error::Result;
use crate::keyword::KeywordSet;

/// A detector with the host's audio format in front of it.
#[derive(Debug)]
pub struct WakeWordStream<D> {
    detector: D,
    capture_rate: SampleRate,
    channels: ChannelCount,
    buffer: FrameBuffer,
    resampler: Option<Resampler>,
    keywords: KeywordSet,
    delivered_samples: u64,
}

impl<D: WakeWordDetector> WakeWordStream<D> {
    /// A stream feeding `detector` from a mono capture at `capture_rate`.
    ///
    /// # Errors
    ///
    /// [`WakeWordError::Audio`](crate::WakeWordError::Audio) if the rate pair
    /// is one the resampler cannot build.
    pub fn new(detector: D, capture_rate: SampleRate, keywords: KeywordSet) -> Result<Self> {
        Self::with_channels(detector, capture_rate, ChannelCount::MONO, keywords)
    }

    /// A stream feeding `detector` from an interleaved capture.
    ///
    /// The downmix happens before the resample: it is cheaper on the wider
    /// signal, and it means the resampler only ever runs one channel, which is
    /// the only channel count the feature extractor has any use for.
    ///
    /// # Errors
    ///
    /// [`WakeWordError::Audio`](crate::WakeWordError::Audio) if the rate pair
    /// is one the resampler cannot build.
    pub fn with_channels(
        detector: D,
        capture_rate: SampleRate,
        channels: ChannelCount,
        keywords: KeywordSet,
    ) -> Result<Self> {
        let target = detector.sample_rate();
        let resampler = if capture_rate == target {
            None
        } else {
            Some(Resampler::new(capture_rate, target, ChannelCount::MONO)?)
        };
        Ok(Self {
            detector,
            capture_rate,
            channels,
            buffer: FrameBuffer::new(capture_rate, channels),
            resampler,
            keywords,
            delivered_samples: 0,
        })
    }

    /// The rate the host captures at.
    #[must_use]
    pub fn capture_rate(&self) -> SampleRate {
        self.capture_rate
    }

    /// The interleaved channel count the host captures at.
    #[must_use]
    pub fn channels(&self) -> ChannelCount {
        self.channels
    }

    /// The rate the detector is fed at.
    #[must_use]
    pub fn detector_rate(&self) -> SampleRate {
        self.detector.sample_rate()
    }

    /// Whether a resampler sits in the path.
    #[must_use]
    pub fn is_resampling(&self) -> bool {
        self.resampler.is_some()
    }

    /// Samples handed to the detector so far, at the detector's rate.
    #[must_use]
    pub fn delivered_samples(&self) -> u64 {
        self.delivered_samples
    }

    /// The keyword table detections are attributed against.
    #[must_use]
    pub fn keywords(&self) -> &KeywordSet {
        &self.keywords
    }

    /// The detector, for a caller that wants to inspect it.
    #[must_use]
    pub fn detector(&self) -> &D {
        &self.detector
    }

    /// Take the detector back.
    #[must_use]
    pub fn into_detector(self) -> D {
        self.detector
    }

    /// Feed one chunk of interleaved little-endian PCM16 from the host.
    ///
    /// # Errors
    ///
    /// [`WakeWordError::Audio`](crate::WakeWordError::Audio) from the downmix
    /// or the resampler.
    pub fn accept_pcm16le(&mut self, bytes: &[u8]) -> Result<Option<Detection>> {
        self.buffer.append_pcm16le(bytes);
        self.drain()
    }

    /// Feed one chunk of interleaved `i16` samples from the host.
    ///
    /// # Errors
    ///
    /// [`WakeWordError::Audio`](crate::WakeWordError::Audio) from the downmix
    /// or the resampler.
    pub fn accept_samples(&mut self, samples: &[i16]) -> Result<Option<Detection>> {
        self.buffer.append(samples);
        self.drain()
    }

    /// Forget the decoder's state and everything buffered in front of it.
    ///
    /// The resampler is deliberately **not** rebuilt: its state is a filter
    /// history, not a decision, and discarding it would put a discontinuity
    /// into the signal at exactly the moment the session is trying to hear
    /// again.
    pub fn reset(&mut self) {
        self.buffer.reset();
        self.detector.reset();
    }

    /// Move everything committed through the chain and offer it to the
    /// detector.
    fn drain(&mut self) -> Result<Option<Detection>> {
        self.buffer.commit();
        let interleaved = self.buffer.take_committed();
        if interleaved.is_empty() {
            return Ok(None);
        }

        let mono = downmix_to_mono_pcm16(&interleaved, self.channels)?;
        let at_detector_rate = match self.resampler.as_mut() {
            Some(resampler) => resampler.process_interleaved_pcm16(&mono)?,
            None => mono,
        };
        if at_detector_rate.is_empty() {
            // The resampler has not accumulated a whole chunk yet. The
            // detector contract says an empty slice is a no-op; not making the
            // call at all is the same answer with one less chance to get it
            // wrong.
            return Ok(None);
        }

        self.delivered_samples += at_detector_rate.len() as u64;
        let samples = pcm16_to_f32_slice(&at_detector_rate);
        Ok(self.detector.accept(&samples).map(|detection| {
            let locale = self.keywords.locale_of(&detection.keyword);
            Detection {
                locale,
                ..detection
            }
        }))
    }
}
