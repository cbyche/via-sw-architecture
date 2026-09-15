//! The crate's single error type.

/// Convenience alias for results produced by this crate.
pub type Result<T> = core::result::Result<T, AudioError>;

/// Everything `via-audio` can fail at.
///
/// Deliberately small: the conversion layer is total (every `i16` has an `f32`
/// image and vice versa), so the only failures are *shape* failures — a caller
/// handing over samples that do not describe whole frames, or an unusable rate
/// / channel count — plus whatever the two optional backends report.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AudioError {
    /// A sample rate of zero was supplied. Rates are `NonZeroU32` internally so
    /// that every `frames ↔ Duration` division is infallible.
    #[error("sample rate must be greater than zero")]
    ZeroSampleRate,

    /// A channel count of zero was supplied.
    #[error("channel count must be greater than zero")]
    ZeroChannels,

    /// A planar buffer did not have the channel count the operation expected.
    #[error("expected {expected} channels, got {actual}")]
    ChannelCountMismatch {
        /// Channels the operation was configured for.
        expected: usize,
        /// Channels the caller actually supplied.
        actual: usize,
    },

    /// A planar buffer's channels had differing lengths. Planar audio is a
    /// rectangle; a ragged one is always a bug at the call site.
    #[error(
        "planar channel lengths differ: channel 0 has {first} frames, channel {channel} has {actual}"
    )]
    ChannelLengthMismatch {
        /// Frame count of channel 0, taken as the reference.
        first: usize,
        /// Index of the first channel that disagreed.
        channel: usize,
        /// Frame count of that channel.
        actual: usize,
    },

    /// An interleaved buffer's length was not a whole multiple of the channel
    /// count, so the last frame is torn.
    ///
    /// This is *not* silently truncated: a torn frame means the caller lost
    /// channel phase somewhere upstream, and every subsequent frame in that
    /// stream has its channels swapped. Failing loudly here is the whole reason
    /// interleaving lives in one place.
    #[error("{samples} samples is not a whole number of {channels}-channel frames")]
    NotFrameAligned {
        /// Number of interleaved samples supplied.
        samples: usize,
        /// Channel count they were meant to describe.
        channels: usize,
    },

    /// The resampler could not be constructed for the requested rate pair.
    #[cfg(feature = "resample")]
    #[error("could not construct a resampler")]
    ResamplerConstruction(#[from] rubato::ResamplerConstructionError),

    /// The resampler rejected a chunk.
    #[cfg(feature = "resample")]
    #[error("resampling failed")]
    Resample(#[from] rubato::ResampleError),

    /// The WAV backend failed to read or write.
    #[cfg(feature = "wav")]
    #[error("wav i/o failed")]
    Wav(#[from] hound::Error),

    /// A WAV file used a sample format this crate does not decode.
    ///
    /// `hound` decodes 8- and 16-bit integer, 24- and 32-bit integer, and
    /// 32-bit float. Anything else — 64-bit float, ADPCM, a bit depth with no
    /// matching byte width — lands here rather than being guessed at.
    #[cfg(feature = "wav")]
    #[error("unsupported WAV sample format: {bits}-bit {format}")]
    UnsupportedWavFormat {
        /// Bits per sample declared by the file.
        bits: u16,
        /// `"integer"` or `"float"`.
        format: &'static str,
    },
}
