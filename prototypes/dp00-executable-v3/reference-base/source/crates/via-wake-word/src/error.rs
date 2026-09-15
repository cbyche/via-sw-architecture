//! What can go wrong installing or running a wake word.
//!
//! # Two audiences, two strings
//!
//! [`WakeWordError`]'s `Display` is a **diagnostic**: English, specific, and
//! aimed at whoever is reading a log or a test failure. It names the path, the
//! digest, the missing file.
//!
//! [`WakeWordError::localized`] is the **sentence a person reads**, and it
//! comes from `via-i18n` like every other one in VIA. Three of them are
//! catalogued upstream messages —
//! `realtime.wake_word_model_download_failed`,
//! `realtime.wake_word_model_checksum_failed` and
//! `realtime.wake_word_model_incomplete` — and the rest fold into
//! `realtime.wake_word_detection_stopped`, whose `{detail}` slot is where the
//! diagnostic goes.
//!
//! The split is deliberate. A checksum failure must say *"wake-word model
//! verification failed"* to the user and *"expected …, got …"* to the operator,
//! and neither string is a good substitute for the other.

use std::path::{Path, PathBuf};

use via_i18n::{Locale, format as i18n_format, keys};

/// The result type every fallible entry point in this crate returns.
pub type Result<T> = core::result::Result<T, WakeWordError>;

/// A wake-word install or detection failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WakeWordError {
    /// The release download did not answer with a body and a 2xx status.
    ///
    /// **External contract** — `server/src/voice/wake-word/model-manager.mjs:52`
    /// (`if (!response.ok || !response.body) throw …`). Both halves are this
    /// one variant, because upstream does not distinguish them either; a 200
    /// with no body is reported as its status.
    #[error("wake-word model download failed: HTTP {status}")]
    Download {
        /// The status the fetch reported.
        status: u16,
    },

    /// The request could not be made, or its body could not be read.
    ///
    /// Distinct from [`Self::Download`], which is an *answer* VIA did not like.
    /// This one is "there was no answer" — DNS, TLS, a reset connection, a
    /// truncated body. Upstream has no counterpart because `fetch` throws and
    /// the rejection propagates untyped.
    #[error("could not download the wake-word model from {url}: {detail}")]
    Fetch {
        /// The URL that was attempted.
        url: String,
        /// What the client reported.
        detail: String,
    },

    /// The downloaded archive did not hash to the pinned digest.
    ///
    /// **External contract** — `model-manager.mjs:88-90`. The supply-chain
    /// check: nothing derived from these bytes reaches the install directory.
    #[error("wake-word model checksum mismatch: expected {expected}, got {actual}")]
    Checksum {
        /// The digest the artifact pins.
        expected: String,
        /// The digest the downloaded bytes actually have.
        actual: String,
    },

    /// The staged install is missing one of the files the model needs.
    ///
    /// **External contract** — `model-manager.mjs:97-99`. Raised *before* the
    /// staging directory is promoted, so a half-extracted archive can never
    /// become the installed model.
    #[error("the wake-word model is missing required files: {}", .missing.join(", "))]
    Incomplete {
        /// The file names that were expected and absent, in artifact order.
        missing: Vec<String>,
    },

    /// The keyword file names tokens the model's inventory does not have.
    ///
    /// **The most dangerous failure in this crate**, and the reason it is
    /// checked rather than left to the engine: `sherpa-onnx` does not *return*
    /// an error for an unencodable keyword. `EncodeBase` logs *"Cannot find ID
    /// for token …"*, `InitKeywords` logs *"Encode keywords failed."*, and the
    /// C++ library ends the process. A Gateway whose operator mistyped a token
    /// line would exit, not degrade. See [`crate::tokens`].
    #[error("the model's token inventory has no entry for: {}", .unknown.join(", "))]
    UnknownTokens {
        /// Every word the model cannot encode, in order, without repeats.
        unknown: Vec<String>,
    },

    /// The keyword file has a `:` or `#` marker carrying no number.
    ///
    /// The second way a keyword file ends the process. `EncodeBase` reaches
    /// `std::stof(word.substr(1))` with no `try`, so `:later` is not a rejected
    /// score — it is an uncaught `std::invalid_argument`. See [`crate::tokens`].
    #[error("the keyword file has a marker with no number in it: {}", .malformed.join(", "))]
    MalformedMarker {
        /// Every marker-shaped word whose payload is not a number, in order,
        /// without repeats.
        malformed: Vec<String>,
    },

    /// The archive could not be decompressed, walked, or read.
    #[error("the wake-word model archive could not be read: {detail}")]
    Archive {
        /// What the decoder or the tar walker said.
        detail: String,
    },

    /// A filesystem operation failed.
    #[error("could not {operation} {}: {detail}", .path.display())]
    Io {
        /// The verb, so the message reads as a sentence: `create`, `write`,
        /// `read`, `remove`, `rename`.
        operation: &'static str,
        /// The path it was attempted on.
        path: PathBuf,
        /// The operating system's description.
        detail: String,
    },

    /// A keyword line, phrase or token string is not usable.
    #[error(transparent)]
    Keyword(#[from] KeywordError),

    /// The audio front end rejected a rate, a channel count or a frame.
    #[error("wake-word audio front end: {0}")]
    Audio(#[from] via_audio::AudioError),

    /// The keyword-spotting engine could not be constructed from an otherwise
    /// complete install.
    ///
    /// Only reachable with `--features sherpa`; without it there is no engine
    /// to fail. The detail is the engine's own, which for `sherpa-onnx` means
    /// "it returned a null spotter" and nothing more — the C API does not
    /// report why.
    #[error("the wake-word engine could not be opened: {detail}")]
    Engine {
        /// What the engine reported, or the closest thing to it.
        detail: String,
    },

    /// An install task was cancelled or panicked before it answered.
    #[error("the wake-word install task did not finish: {detail}")]
    InstallAbandoned {
        /// The join failure's description.
        detail: String,
    },
}

impl WakeWordError {
    /// Build an [`WakeWordError::Io`] from a `std::io::Error`.
    pub(crate) fn io(
        operation: &'static str,
        path: impl AsRef<Path>,
        error: &std::io::Error,
    ) -> Self {
        Self::Io {
            operation,
            path: path.as_ref().to_path_buf(),
            detail: error.to_string(),
        }
    }

    /// The sentence a person reads, in `locale`.
    ///
    /// Three variants have their own catalogued key; everything else is a
    /// `{detail}` inside `realtime.wake_word_detection_stopped`, because from
    /// the outside "the archive was corrupt" and "the disk was full" are the
    /// same event: detection is not going to happen, and here is why.
    #[must_use]
    pub fn localized(&self, locale: Locale) -> String {
        match self {
            Self::Download { status } => i18n_format(
                locale,
                keys::REALTIME_WAKE_WORD_MODEL_DOWNLOAD_FAILED,
                &[("status", &status.to_string())],
            ),
            Self::Checksum { .. } => {
                i18n_format(locale, keys::REALTIME_WAKE_WORD_MODEL_CHECKSUM_FAILED, &[])
            }
            Self::Incomplete { .. } => {
                i18n_format(locale, keys::REALTIME_WAKE_WORD_MODEL_INCOMPLETE, &[])
            }
            other => i18n_format(
                locale,
                keys::REALTIME_WAKE_WORD_DETECTION_STOPPED,
                &[("detail", &other.to_string())],
            ),
        }
    }
}

/// A keyword, phrase or token string that cannot be used.
///
/// Separate from [`WakeWordError`] because these are all *configuration*
/// faults, reported the moment the value is constructed rather than when a
/// microphone is opened. `docs/architecture.md` §16 makes the phrase a
/// configuration value; this is the type that says a configuration value is
/// wrong.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum KeywordError {
    /// The phrase, or its token line, is empty once trimmed.
    ///
    /// An empty phrase is not an error at the *settings* level — it is how a
    /// deployment says "no wake word" (see
    /// [`DisabledReason::NoPhraseConfigured`](crate::DisabledReason::NoPhraseConfigured)).
    /// It is an error here, where a `Keyword` is being built: a keyword with
    /// nothing to match is a silent no-op, and silent no-ops are what this
    /// crate's tests exist to prevent.
    #[error("a wake-word {field} cannot be empty")]
    Empty {
        /// `phrase` or `tokens`.
        field: &'static str,
    },

    /// The phrase or its token line contains a line break.
    ///
    /// The keyword file is one keyword per line, so an embedded newline
    /// registers a second keyword nobody configured.
    #[error("a wake-word {field} cannot contain a line break")]
    LineBreak {
        /// `phrase` or `tokens`.
        field: &'static str,
    },

    /// The phrase or its token line contains `@`.
    ///
    /// `@` is the keyword file's own separator between the token line and the
    /// display text (`tokens @display`), so a value containing one is parsed
    /// at the wrong place.
    #[error("a wake-word {field} cannot contain `@`, which separates tokens from the display text")]
    Separator {
        /// `phrase` or `tokens`.
        field: &'static str,
    },

    /// A word in the token column begins with a keyword-file marker.
    ///
    /// `sherpa-onnx` reads a word beginning with `:` as a per-keyword boost
    /// score and one beginning with `#` as a per-keyword threshold, parsing the
    /// rest with `std::stof`. A token that happens to start with either is
    /// therefore not a token at all — it is a malformed number, in a C++
    /// parser, inside a library that ends the process when keyword encoding
    /// fails.
    #[error(
        "a wake-word {field} word cannot begin with `{marker}`, which the keyword file reads as a per-keyword score or threshold"
    )]
    Marker {
        /// `phrase` or `tokens`.
        field: &'static str,
        /// The character it began with.
        marker: char,
    },
}
