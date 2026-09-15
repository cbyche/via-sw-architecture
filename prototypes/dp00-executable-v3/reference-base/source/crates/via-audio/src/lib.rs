//! Audio primitives for VIA.
//!
//! A leaf crate with no upstream to port: qwen-audio-agent does its sample
//! conversion inline in the wake-word detector and leaves resampling to the
//! host, and ARGO assumes the Android side hands it 24 kHz PCM16 already. VIA
//! has five realtime providers, a local pipeline and a replay-based mock, and
//! every one of them needs the same four things. Getting any of them subtly
//! wrong — an off-by-one in interleaving, a clipped sample on the `i16`
//! boundary, an aliased downsample — does not produce a crash or a stack trace.
//! It produces a model that mishears things. So they live here, once, tested
//! against their boundaries.
//!
//! # What is in here
//!
//! | Module | What |
//! | --- | --- |
//! | [`sample`] | PCM16 ↔ `f32`, little-endian byte codecs, interleave / deinterleave, down- and up-mix |
//! | [`rate`] | [`SampleRate`], [`ChannelCount`], and exact `frames ↔ Duration` arithmetic |
//! | [`clock`] | [`PlaybackCursor`] — the Injection Gate's drain predicate and `audio_end_ms` |
//! | [`buffer`] | [`FrameBuffer`] — append / commit / take, the shape every realtime input buffer has |
//! | [`resample`] | 48 ↔ 24 ↔ 16 kHz, band-limited, with the interleaved ↔ planar adapters (feature `resample`) |
//! | [`wav`] | Fixtures and the `mock` provider's canned audio (feature `wav`) |
//!
//! # Conventions this crate fixes
//!
//! **PCM16 divides by 32768, and clamps coming back.** `-32768` maps to
//! `-1.0` exactly; `+1.0` saturates to `32767`. See [`sample`] for the full
//! statement and why the other choice was not available. This is an external
//! contract, from `server/src/voice/wake-word/sherpa-detector.mjs:9-17`.
//!
//! **Interleaved on the wire, planar in the resampler.** A realtime socket
//! carries `L R L R …` little-endian `i16`; `rubato` wants one `Vec` per
//! channel. Neither is going to change, so the adapters are in [`sample`] and
//! [`resample`] uses them rather than each provider writing its own.
//!
//! **A torn frame is an error, not a truncation.** An interleaved buffer whose
//! length is not a multiple of the channel count means channel phase was lost
//! upstream; every frame after it has its channels swapped. The one place a
//! partial sample is tolerated is [`FrameBuffer::append_pcm16le`], which
//! carries the odd byte into the next chunk instead of dropping it.
//!
//! **Duration is integer arithmetic.** No `f64` seconds anywhere: the two
//! consumers compare the result against a wire value, and half a millisecond of
//! float drift is a clipped sentence.
//!
//! # Scope
//!
//! No device I/O, no codecs, no base64 — those are transport concerns and
//! belong with the transport. The only `std::io` in the crate is behind the
//! `wav` feature.
//!
//! # Example
//!
//! Host capture at 48 kHz stereo, on its way to a 16 kHz mono ASR:
//!
//! ```
//! use via_audio::{ChannelCount, FrameBuffer, SampleRate, downmix_to_mono_pcm16};
//! # fn main() -> Result<(), via_audio::AudioError> {
//! let mut capture = FrameBuffer::new(SampleRate::HZ_48000, ChannelCount::STEREO);
//! capture.append_pcm16le(&[0u8; 3_840]); // 960 stereo frames = 20 ms
//! capture.commit();
//!
//! let interleaved = capture.take_committed();
//! let mono = downmix_to_mono_pcm16(&interleaved, ChannelCount::STEREO)?;
//! # #[cfg(feature = "resample")] {
//! let asr = via_audio::resample_mono_pcm16(
//!     &mono,
//!     SampleRate::HZ_48000,
//!     SampleRate::HZ_16000,
//! )?;
//! assert_eq!(asr.len(), 320); // 20 ms at 16 kHz
//! # }
//! # Ok(())
//! # }
//! ```

pub mod buffer;
pub mod clock;
pub mod error;
pub mod rate;
pub mod sample;

#[cfg(feature = "resample")]
pub mod resample;

#[cfg(feature = "wav")]
pub mod wav;

pub use buffer::FrameBuffer;
pub use clock::PlaybackCursor;
pub use error::{AudioError, Result};
pub use rate::{BLOCK_MILLIS, ChannelCount, SampleRate};
pub use sample::{
    PCM16_BYTES_PER_SAMPLE, PCM16_ROUND_TRIP_EPSILON, PCM16_SCALE, deinterleave,
    deinterleave_pcm16, downmix_to_mono, downmix_to_mono_pcm16, f32_to_pcm16, f32_to_pcm16_slice,
    f32_to_pcm16le, i16_to_pcm16le, interleave, interleave_pcm16, pcm16_to_f32, pcm16_to_f32_slice,
    pcm16le_to_f32, pcm16le_to_i16, planar_frames, upmix_from_mono,
};

#[cfg(feature = "resample")]
pub use resample::{
    Resampler, resample_interleaved, resample_interleaved_pcm16, resample_mono_pcm16,
};

#[cfg(feature = "wav")]
pub use wav::{WavAudio, read_wav, read_wav_file, write_wav, write_wav_file};
