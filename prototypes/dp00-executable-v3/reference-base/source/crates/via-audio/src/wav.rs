//! WAV read and write, for test fixtures and the `mock` realtime provider.
//!
//! Behind the `wav` feature, because it is the only part of the crate that
//! touches `std::io` and the only one that pulls a dependency a wire-only build
//! has no use for.
//!
//! Everything normalises to interleaved PCM16 on the way in and out. A fixture
//! recorded at 32-bit float is still a fixture; making every caller branch on
//! the file's bit depth would guarantee that some caller does it wrong.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;
use std::time::Duration;

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

use crate::error::{AudioError, Result};
use crate::rate::{ChannelCount, SampleRate};
use crate::sample::{f32_to_pcm16, i16_to_pcm16le};

/// Interleaved PCM16 audio with its rate and channel count attached.
///
/// The three fields travel together because two of them are meaningless
/// without the third: a `Vec<i16>` on its own cannot say how long it is or
/// which samples belong to the same instant.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WavAudio {
    rate: SampleRate,
    channels: ChannelCount,
    samples: Vec<i16>,
}

impl WavAudio {
    /// Wrap interleaved samples.
    ///
    /// # Errors
    ///
    /// [`AudioError::NotFrameAligned`] if `samples.len()` is not a multiple of
    /// `channels` — a WAV whose data chunk ends mid-frame is corrupt, and
    /// rounding it off would silently swap the channels of everything a caller
    /// appends afterwards.
    pub fn new(rate: SampleRate, channels: ChannelCount, samples: Vec<i16>) -> Result<Self> {
        channels.frames(samples.len())?;
        Ok(Self {
            rate,
            channels,
            samples,
        })
    }

    /// The sample rate.
    #[must_use]
    pub fn rate(&self) -> SampleRate {
        self.rate
    }

    /// The interleaved channel count.
    #[must_use]
    pub fn channels(&self) -> ChannelCount {
        self.channels
    }

    /// The interleaved samples.
    #[must_use]
    pub fn samples(&self) -> &[i16] {
        &self.samples
    }

    /// Consume `self` and yield the interleaved samples.
    #[must_use]
    pub fn into_samples(self) -> Vec<i16> {
        self.samples
    }

    /// Frame count.
    #[must_use]
    pub fn frames(&self) -> usize {
        self.channels.frames_floor(self.samples.len())
    }

    /// Playing time, exact at the file's own rate.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.rate.frames_to_duration(self.frames() as u64)
    }

    /// The samples as little-endian PCM16 bytes — what a realtime socket wants.
    #[must_use]
    pub fn to_pcm16le(&self) -> Vec<u8> {
        i16_to_pcm16le(&self.samples)
    }
}

/// Read a WAV stream as interleaved PCM16.
///
/// Accepts 8-, 16-, 24- and 32-bit integer and 32-bit float files, scaling each
/// to the full `i16` range. Anything else is
/// [`AudioError::UnsupportedWavFormat`] rather than a guess.
///
/// # Errors
///
/// [`AudioError::Wav`] on a malformed file, [`AudioError::UnsupportedWavFormat`]
/// on a bit depth this crate does not decode, or [`AudioError::ZeroSampleRate`]
/// / [`AudioError::ZeroChannels`] on a header that declares either as zero.
pub fn read_wav<R: Read>(reader: R) -> Result<WavAudio> {
    let mut wav = WavReader::new(reader)?;
    let spec = wav.spec();
    let rate = SampleRate::new(spec.sample_rate)?;
    let channels = ChannelCount::new(spec.channels as usize)?;

    let samples: Vec<i16> = match (spec.sample_format, spec.bits_per_sample) {
        // hound decodes 8-bit as a signed value in -128..=127; shifting left by
        // 8 puts it back on the full i16 scale instead of leaving the fixture
        // 48 dB quieter than it sounds.
        (SampleFormat::Int, 8) => wav
            .samples::<i16>()
            .map(|s| s.map(|v| v << 8))
            .collect::<hound::Result<_>>()?,
        (SampleFormat::Int, 16) => wav.samples::<i16>().collect::<hound::Result<_>>()?,
        (SampleFormat::Int, bits @ (24 | 32)) => {
            let shift = bits - 16;
            wav.samples::<i32>()
                .map(|s| s.map(|v| (v >> shift) as i16))
                .collect::<hound::Result<_>>()?
        }
        (SampleFormat::Float, 32) => wav
            .samples::<f32>()
            .map(|s| s.map(f32_to_pcm16))
            .collect::<hound::Result<_>>()?,
        (format, bits) => {
            return Err(AudioError::UnsupportedWavFormat {
                bits,
                format: match format {
                    SampleFormat::Int => "integer",
                    SampleFormat::Float => "float",
                },
            });
        }
    };

    WavAudio::new(rate, channels, samples)
}

/// Read a WAV file from disk as interleaved PCM16.
///
/// # Errors
///
/// As [`read_wav`], plus [`AudioError::Wav`] if the file cannot be opened.
pub fn read_wav_file<P: AsRef<Path>>(path: P) -> Result<WavAudio> {
    let file = File::open(path).map_err(hound::Error::IoError)?;
    read_wav(BufReader::new(file))
}

/// Write interleaved PCM16 as a 16-bit WAV stream.
///
/// # Errors
///
/// [`AudioError::Wav`] if the writer fails or the header cannot be finalised.
pub fn write_wav<W: Write + Seek>(writer: W, audio: &WavAudio) -> Result<()> {
    let spec = WavSpec {
        // `WavSpec::channels` is a u16; a channel count that does not fit is
        // not a WAV file anyone can write.
        channels: u16::try_from(audio.channels.get()).map_err(|_| {
            AudioError::ChannelCountMismatch {
                expected: usize::from(u16::MAX),
                actual: audio.channels.get(),
            }
        })?,
        sample_rate: audio.rate.hz(),
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut wav = WavWriter::new(writer, spec)?;
    for sample in &audio.samples {
        wav.write_sample(*sample)?;
    }
    // `finalize` rewrites the RIFF and data chunk lengths. Dropping the writer
    // instead would leave a file whose header claims zero samples.
    wav.finalize()?;
    Ok(())
}

/// Write interleaved PCM16 to a 16-bit WAV file on disk.
///
/// # Errors
///
/// As [`write_wav`], plus [`AudioError::Wav`] if the file cannot be created.
pub fn write_wav_file<P: AsRef<Path>>(path: P, audio: &WavAudio) -> Result<()> {
    let file = File::create(path).map_err(hound::Error::IoError)?;
    write_wav(BufWriter::new(file), audio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> WavAudio {
        let samples: Vec<i16> = (0..960)
            .map(|n| ((n * 97) % 65_536 - 32_768) as i16)
            .collect();
        WavAudio::new(SampleRate::HZ_24000, ChannelCount::MONO, samples).expect("mono")
    }

    fn write_to_vec(audio: &WavAudio) -> Vec<u8> {
        let mut buffer = Cursor::new(Vec::new());
        write_wav(&mut buffer, audio).expect("write");
        buffer.into_inner()
    }

    #[test]
    fn round_trips_through_memory() {
        let audio = fixture();
        let bytes = write_to_vec(&audio);
        let read = read_wav(Cursor::new(bytes)).expect("read");
        assert_eq!(read, audio);
        assert_eq!(read.rate(), SampleRate::HZ_24000);
        assert_eq!(read.channels(), ChannelCount::MONO);
        assert_eq!(read.frames(), 960);
        assert_eq!(read.duration(), Duration::from_millis(40));
    }

    #[test]
    fn round_trips_through_a_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("fixture.wav");
        let audio = fixture();
        write_wav_file(&path, &audio).expect("write");
        let read = read_wav_file(&path).expect("read");
        assert_eq!(read, audio);
    }

    #[test]
    fn the_header_is_finalized_not_left_claiming_zero_samples() {
        let bytes = write_to_vec(&fixture());
        // Canonical 44-byte header plus 960 mono 16-bit samples.
        assert_eq!(bytes.len(), 44 + 960 * 2);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        // The RIFF size field must count the whole file minus the 8-byte
        // "RIFF<size>" prefix.
        let riff_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(riff_size as usize, bytes.len() - 8);
    }

    #[test]
    fn stereo_keeps_its_frame_count() {
        let audio = WavAudio::new(
            SampleRate::HZ_48000,
            ChannelCount::STEREO,
            vec![1, -1, 2, -2, 3, -3],
        )
        .expect("stereo");
        assert_eq!(audio.frames(), 3);
        let read = read_wav(Cursor::new(write_to_vec(&audio))).expect("read");
        assert_eq!(read, audio);
        assert_eq!(read.samples(), &[1, -1, 2, -2, 3, -3]);
    }

    #[test]
    fn a_torn_frame_is_refused() {
        assert!(matches!(
            WavAudio::new(SampleRate::HZ_48000, ChannelCount::STEREO, vec![1, 2, 3]),
            Err(AudioError::NotFrameAligned {
                samples: 3,
                channels: 2
            })
        ));
    }

    #[test]
    fn a_float_wav_is_normalised_to_pcm16() {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 24_000,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut buffer, spec).expect("writer");
            for sample in [-1.0_f32, -0.5, 0.0, 32_767.0 / 32_768.0, 1.0] {
                writer.write_sample(sample).expect("sample");
            }
            writer.finalize().expect("finalize");
        }
        let read = read_wav(Cursor::new(buffer.into_inner())).expect("read");
        // The same boundary mapping the rest of the crate uses, including the
        // saturation of +1.0.
        assert_eq!(read.samples(), &[i16::MIN, -16_384, 0, i16::MAX, i16::MAX]);
    }

    #[test]
    fn a_24_bit_wav_is_scaled_down_not_wrapped() {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 24,
            sample_format: SampleFormat::Int,
        };
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut buffer, spec).expect("writer");
            for sample in [-8_388_608_i32, 0, 8_388_607] {
                writer.write_sample(sample).expect("sample");
            }
            writer.finalize().expect("finalize");
        }
        let read = read_wav(Cursor::new(buffer.into_inner())).expect("read");
        assert_eq!(read.samples(), &[i16::MIN, 0, i16::MAX]);
        assert_eq!(read.rate(), SampleRate::HZ_16000);
    }

    #[test]
    fn pcm16le_bytes_match_the_data_chunk() {
        let audio = fixture();
        let bytes = write_to_vec(&audio);
        assert_eq!(audio.to_pcm16le(), bytes[44..]);
    }
}
