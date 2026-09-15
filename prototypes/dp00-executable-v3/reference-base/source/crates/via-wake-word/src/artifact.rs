//! The downloadable keyword model: its id, its release URL, its digest and the
//! five file names it is made of.
//!
//! Ported from upstream `server/src/voice/wake-word/model-manager.mjs:17-33`.
//!
//! # Why this is a type and not five constants
//!
//! `docs/architecture.md` §16 makes wake word **per-locale**: *"three locales
//! may mean three models."* The keyword artifact is therefore something a
//! locale resolves *to*, not a global. One artifact ships today — the
//! zh-en model upstream pins — and it covers both `zh` and `en`; a Korean
//! phrase needs a Korean keyword model, and when one is chosen it is a second
//! [`ModelArtifact`] beside this one, installed into its own directory under
//! the same root, with no change to anything that consumes them.
//!
//! # The id and the URL are third-party names
//!
//! `sherpa-onnx-kws-zipformer-zh-en-3M-2025-12-20` and the `k2-fsa/sherpa-onnx`
//! release URL belong to the k2-fsa project. `docs/rebrand.md`'s KEEP table
//! carries both, for the plain reason that renaming either one breaks the
//! download and the digest that verifies it.

use std::borrow::Cow;

/// The catalogued model id.
///
/// **External contract** — `model-manager.mjs:17`, catalogued as
/// `WAKE_WORD_MODEL_NAME`. It is also the directory name the model installs
/// into, so it is a path component as well as a URL component.
pub const WAKE_WORD_MODEL_NAME: &str = "sherpa-onnx-kws-zipformer-zh-en-3M-2025-12-20";

/// The release directory the archive is published in.
///
/// **External contract** — the prefix of `WAKE_WORD_MODEL_URL`
/// (`model-manager.mjs:18`). Split from the id because the URL is built the
/// way upstream builds it, by interpolating the id, rather than written out a
/// second time and left to drift.
pub const WAKE_WORD_MODEL_RELEASE_BASE: &str =
    "https://github.com/k2-fsa/sherpa-onnx/releases/download/kws-models";

/// The archive extension.
///
/// **External contract** — the suffix of `WAKE_WORD_MODEL_URL`. It is also a
/// statement about the format: bzip2-compressed tar, which is what
/// [`crate::install`] knows how to open.
pub const WAKE_WORD_MODEL_ARCHIVE_SUFFIX: &str = ".tar.bz2";

/// The pinned SHA-256 of the archive, lowercase hex.
///
/// **External contract** — `model-manager.mjs:19`, catalogued as
/// `WAKE_WORD_MODEL_SHA256`. A mismatch is
/// [`WakeWordError::Checksum`](crate::WakeWordError::Checksum) and nothing is
/// installed.
pub const WAKE_WORD_MODEL_SHA256: &str =
    "68447f4fbc67e70eee3a93961f36e81e98f47aef73ce7e7ca00885c6cd3616a6";

/// The five file names a keyword model is made of.
///
/// **External contract** — `model-manager.mjs:21-27`, catalogued as
/// `WAKE_WORD_MODEL_FILES`. Four come out of the archive; `keywords` is
/// generated from the configured phrases and is the one file that is *not*
/// covered by the digest, because its content is a product decision rather
/// than a downloaded artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFiles {
    /// The transducer encoder.
    pub encoder: Cow<'static, str>,
    /// The transducer decoder.
    pub decoder: Cow<'static, str>,
    /// The transducer joiner.
    pub joiner: Cow<'static, str>,
    /// The token inventory.
    pub tokens: Cow<'static, str>,
    /// The generated keyword file.
    pub keywords: Cow<'static, str>,
}

impl ModelFiles {
    /// The four names that are extracted from the archive, in artifact order.
    ///
    /// **External contract** — `model-manager.mjs:29-34` (`ARCHIVE_FILES`).
    /// `keywords` is deliberately absent: upstream generates it after
    /// extraction and so does VIA.
    #[must_use]
    pub fn archived(&self) -> [&str; 4] {
        [&self.encoder, &self.decoder, &self.joiner, &self.tokens]
    }

    /// All five names — what a complete install contains.
    ///
    /// **External contract** — `model-manager.mjs:35` (`REQUIRED_FILES`).
    #[must_use]
    pub fn required(&self) -> [&str; 5] {
        [
            &self.encoder,
            &self.decoder,
            &self.joiner,
            &self.tokens,
            &self.keywords,
        ]
    }

    /// Whether `name` is one of the four members worth extracting.
    #[must_use]
    pub fn is_archived(&self, name: &str) -> bool {
        self.archived().contains(&name)
    }
}

/// The catalogued file names.
///
/// **External contract** — `model-manager.mjs:21-27`.
pub const WAKE_WORD_MODEL_FILES: ModelFiles = ModelFiles {
    encoder: Cow::Borrowed("encoder-epoch-13-avg-2-chunk-8-left-64.int8.onnx"),
    decoder: Cow::Borrowed("decoder-epoch-13-avg-2-chunk-8-left-64.onnx"),
    joiner: Cow::Borrowed("joiner-epoch-13-avg-2-chunk-8-left-64.int8.onnx"),
    tokens: Cow::Borrowed("tokens.txt"),
    keywords: Cow::Borrowed("keywords.txt"),
};

/// One downloadable, checksummed keyword model.
///
/// `docs/architecture.md` §16: *"the keyword model is a downloadable
/// checksummed artifact resolved per phrase"*. This is that artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelArtifact {
    /// The model id — the last path segment of the URL, and the directory the
    /// model installs into.
    pub id: Cow<'static, str>,
    /// The release directory the archive is published in, without a trailing
    /// slash.
    pub release_base: Cow<'static, str>,
    /// The archive extension, including the leading dot.
    pub archive_suffix: Cow<'static, str>,
    /// The pinned SHA-256 of the archive, lowercase hex.
    pub sha256: Cow<'static, str>,
    /// The five file names.
    pub files: ModelFiles,
}

impl ModelArtifact {
    /// The one artifact VIA catalogues today: upstream's zh-en keyword model.
    ///
    /// It carries both Chinese and English keywords, so a `zh` phrase and an
    /// `en` phrase can resolve to this same artifact and share one install.
    pub const ZH_EN_3M: Self = Self {
        id: Cow::Borrowed(WAKE_WORD_MODEL_NAME),
        release_base: Cow::Borrowed(WAKE_WORD_MODEL_RELEASE_BASE),
        archive_suffix: Cow::Borrowed(WAKE_WORD_MODEL_ARCHIVE_SUFFIX),
        sha256: Cow::Borrowed(WAKE_WORD_MODEL_SHA256),
        files: WAKE_WORD_MODEL_FILES,
    };

    /// The archive's download URL.
    ///
    /// **External contract** — `model-manager.mjs:18`, catalogued as
    /// `WAKE_WORD_MODEL_URL`. Built by interpolation exactly as upstream builds
    /// it, so the id cannot be changed in one place and forgotten in the other.
    #[must_use]
    pub fn url(&self) -> String {
        format!("{}/{}{}", self.release_base, self.id, self.archive_suffix)
    }

    /// The archive's file name, `<id><suffix>`.
    #[must_use]
    pub fn archive_name(&self) -> String {
        format!("{}{}", self.id, self.archive_suffix)
    }
}

impl Default for ModelArtifact {
    fn default() -> Self {
        Self::ZH_EN_3M
    }
}
