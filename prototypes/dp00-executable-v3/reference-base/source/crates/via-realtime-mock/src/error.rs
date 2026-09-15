//! Everything the mock refuses.
//!
//! # Why there is no `message(locale)` here
//!
//! Every other VIA error type splits a developer sentence ([`core::fmt::Display`])
//! from a person's sentence (`message(locale)`), because both audiences exist.
//! For this one only the first audience does: a [`MockError`] means a *test
//! fixture* is broken — an unreadable script file, a WAV this crate cannot
//! decode, a script server that has already stopped. None of those can reach a
//! user, because the mock provider never runs outside a test or a `via chat`
//! rehearsal.
//!
//! The two sentences that *are* a person's — what a mock provider says when it
//! is unconfigured and what it says when the connect budget runs out — go
//! through [`via_i18n`] like every other provider's, in [`crate::provider`].

/// Anything the mock realtime provider or its script server refuses.
///
/// Not `#[non_exhaustive]`, matching every other VIA error enum: a crate above
/// this one should be made to recompile when a new refusal appears rather than
/// fold it into a wildcard arm that already existed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MockError {
    /// A script fixture could not be read off disk.
    #[error("mock realtime script fixture {path} could not be read: {detail}")]
    FixtureUnreadable {
        /// The path as it was given.
        path: String,
        /// The I/O failure's own text.
        detail: String,
    },

    /// A script fixture is not a script.
    #[error("mock realtime script fixture {path} is not a valid script: {detail}")]
    FixtureInvalid {
        /// The path as it was given.
        path: String,
        /// The deserializer's own text, which names the offending field.
        detail: String,
    },

    /// A script given as JSON text is not a script.
    #[error("the mock realtime script is not valid: {detail}")]
    ScriptInvalid {
        /// The deserializer's own text.
        detail: String,
    },

    /// Canned audio could not be built or decoded.
    #[error("mock realtime canned audio is unusable: {detail}")]
    Audio {
        /// [`via_audio::AudioError`]'s own text.
        detail: String,
    },

    /// Base64 that is not base64.
    ///
    /// Reachable only from [`Transcript::input_audio_pcm16`], which decodes what
    /// the *session under test* wrote — so it fires when the caller under test
    /// put something that is not base64 PCM16 on the wire, which is exactly the
    /// bug worth reporting rather than swallowing.
    ///
    /// [`Transcript::input_audio_pcm16`]: crate::Transcript::input_audio_pcm16
    #[error("mock realtime audio frame is not base64: {detail}")]
    NotBase64 {
        /// The decoder's own text.
        detail: String,
    },

    /// The session never wrote the frame a test was waiting for.
    ///
    /// Answered rather than hung: a mock that waits forever turns a regression
    /// into a CI timeout with no message on it.
    #[error("the session never wrote a `{kind}` frame")]
    FrameNotWritten {
        /// The frame type that was waited for.
        kind: String,
    },

    /// The script server has stopped, so it can neither be queried nor driven.
    ///
    /// It stops when the session it was serving closed *and* every
    /// [`MockHandle`](crate::MockHandle) was dropped.
    #[error("the mock realtime script server has stopped")]
    ServerStopped,
}

impl MockError {
    /// A stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::FixtureUnreadable { .. } => "VIA_MOCK_FIXTURE_UNREADABLE",
            Self::FixtureInvalid { .. } | Self::ScriptInvalid { .. } => "VIA_MOCK_SCRIPT_INVALID",
            Self::Audio { .. } => "VIA_MOCK_AUDIO",
            Self::NotBase64 { .. } => "VIA_MOCK_NOT_BASE64",
            Self::FrameNotWritten { .. } => "VIA_MOCK_FRAME_NOT_WRITTEN",
            Self::ServerStopped => "VIA_MOCK_SERVER_STOPPED",
        }
    }
}

impl From<via_audio::AudioError> for MockError {
    /// Flattens the source chain into the detail.
    ///
    /// `AudioError::Wav`'s own `Display` is the four words `wav i/o failed`;
    /// everything that says *which* file and *why* lives on its source. A
    /// fixture that fails to load has to say so in one line, so the chain is
    /// joined here rather than lost.
    fn from(error: via_audio::AudioError) -> Self {
        let mut detail = error.to_string();
        let mut source = std::error::Error::source(&error);
        while let Some(cause) = source {
            detail.push_str(": ");
            detail.push_str(&cause.to_string());
            source = cause.source();
        }
        Self::Audio { detail }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn every_variant_has_its_own_code_except_the_two_that_share_one() {
        let codes = [
            MockError::FixtureUnreadable {
                path: "a".into(),
                detail: "b".into(),
            }
            .code(),
            MockError::FixtureInvalid {
                path: "a".into(),
                detail: "b".into(),
            }
            .code(),
            MockError::ScriptInvalid { detail: "b".into() }.code(),
            MockError::Audio { detail: "b".into() }.code(),
            MockError::NotBase64 { detail: "b".into() }.code(),
            MockError::FrameNotWritten { kind: "a.b".into() }.code(),
            MockError::ServerStopped.code(),
        ];
        // The two script-shaped failures deliberately share a code: they are the
        // same fault reported from two doors.
        assert_eq!(codes[1], codes[2]);
        let mut unique: Vec<&str> = codes.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), codes.len() - 1);
        assert!(unique.iter().all(|code| code.starts_with("VIA_MOCK_")));
    }

    #[test]
    fn an_audio_failure_carries_the_audio_crates_own_text() {
        let error = MockError::from(via_audio::AudioError::ZeroSampleRate);
        assert_eq!(
            error,
            MockError::Audio {
                detail: via_audio::AudioError::ZeroSampleRate.to_string(),
            }
        );
        assert_eq!(error.code(), "VIA_MOCK_AUDIO");
    }
}
