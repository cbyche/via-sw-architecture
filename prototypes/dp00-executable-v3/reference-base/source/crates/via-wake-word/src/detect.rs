//! The detector seam and the ten numbers that configure it.
//!
//! Ported from upstream `server/src/voice/wake-word/sherpa-detector.mjs:26-50`.
//!
//! # The eleven fields, and who owns which
//!
//! The catalogued `sherpa KWS detection config` contract pins eleven values:
//!
//! ```text
//! featConfig  { samplingRate: 16000, featureDim: 80 }
//! modelConfig { transducer{encoder,decoder,joiner}, tokens,
//!               numThreads: 1, provider: 'cpu', debug: 0,
//!               modelingUnit: 'cjkchar' }
//! maxActivePaths: 4   numTrailingBlanks: 1
//! keywordsScore: 1.0  keywordsThreshold: 0.25
//! keywords: ''        keywordsBuf: <file text>  keywordsBufSize: <byteLength>
//! ```
//!
//! **`samplingRate` is not one of them.** `via-audio` already owns 16 000 as
//! [`SampleRate::HZ_16000`] and its own `tests/contracts.rs` asserts it against
//! the same catalogue row; `docs/deviations/phase-0.md` records the split. So
//! [`DetectionConfig::sample_rate`] *reads* that constant and this crate
//! asserts the other nine. Retyping `16000` here would have created two places
//! for the feature extractor's rate to live, which is exactly the class of bug
//! `via-audio` exists to prevent — a rate mismatch between the resampler and
//! the extractor does not crash, it just quietly stops hearing the phrase.
//!
//! # `keywordsThreshold: 0.25` is a product decision
//!
//! The catalogue says why: *"Detection sensitivity is a product-visible
//! tradeoff (0.25 documented as the office-environment compromise)."* Lower it
//! and the Gateway wakes at the wrong moment in an open-plan office; raise it
//! and it does not wake at all. It is pinned, and it is pinned here, once.

use std::fmt;
use std::path::{Path, PathBuf};

use via_audio::SampleRate;
use via_i18n::Locale;

use crate::artifact::ModelArtifact;

/// Mel-filterbank dimension the model was trained with.
///
/// **External contract** — `sherpa-detector.mjs:26` (`featureDim: 80`).
pub const FEATURE_DIM: i32 = 80;

/// Decoder threads.
///
/// **External contract** — `sherpa-detector.mjs:35` (`numThreads: 1`). One,
/// because the detector runs continuously beside a realtime session and a
/// second thread buys nothing on a 3 M-parameter model.
pub const NUM_THREADS: i32 = 1;

/// ONNX execution provider.
///
/// **External contract** — `sherpa-detector.mjs:36` (`provider: 'cpu'`).
pub const PROVIDER_CPU: &str = "cpu";

/// Engine debug logging.
///
/// **External contract** — `sherpa-detector.mjs:37` (`debug: 0`). The Rust
/// binding types this as a `bool` where the C API and the JS binding type it as
/// an `int`; `0` and `false` are the same value.
pub const DEBUG: bool = false;

/// The unit the token inventory is spelled in.
///
/// **External contract** — `sherpa-detector.mjs:38` (`modelingUnit:
/// 'cjkchar'`). It is also why a keyword's token line is configuration: with
/// `cjkchar`, tokens are tone-marked pinyin syllables, which cannot be derived
/// from the display text without the model's own inventory.
pub const MODELING_UNIT_CJKCHAR: &str = "cjkchar";

/// Beam width.
///
/// **External contract** — `sherpa-detector.mjs:40` (`maxActivePaths: 4`).
pub const MAX_ACTIVE_PATHS: i32 = 4;

/// Blank frames required after a keyword before it is emitted.
///
/// **External contract** — `sherpa-detector.mjs:41` (`numTrailingBlanks: 1`).
pub const NUM_TRAILING_BLANKS: i32 = 1;

/// Per-keyword boost applied during decoding.
///
/// **External contract** — `sherpa-detector.mjs:42` (`keywordsScore: 1.0`).
pub const KEYWORDS_SCORE: f32 = 1.0;

/// Detection threshold.
///
/// **External contract** — `sherpa-detector.mjs:43` (`keywordsThreshold:
/// 0.25`), documented upstream as the office-environment compromise. A
/// product-visible tradeoff, not a tuning knob.
pub const KEYWORDS_THRESHOLD: f32 = 0.25;

/// The four model files, resolved to absolute paths inside one install.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPaths {
    /// The transducer encoder.
    pub encoder: PathBuf,
    /// The transducer decoder.
    pub decoder: PathBuf,
    /// The transducer joiner.
    pub joiner: PathBuf,
    /// The token inventory.
    pub tokens: PathBuf,
}

impl ModelPaths {
    /// Resolve `artifact`'s four archive members inside `directory`.
    #[must_use]
    pub fn resolve(directory: &Path, artifact: &ModelArtifact) -> Self {
        Self {
            encoder: directory.join(artifact.files.encoder.as_ref()),
            decoder: directory.join(artifact.files.decoder.as_ref()),
            joiner: directory.join(artifact.files.joiner.as_ref()),
            tokens: directory.join(artifact.files.tokens.as_ref()),
        }
    }

    /// The four paths, in artifact order.
    #[must_use]
    pub fn all(&self) -> [&Path; 4] {
        [&self.encoder, &self.decoder, &self.joiner, &self.tokens]
    }
}

/// Everything the keyword spotter is constructed from.
///
/// The integer and float widths are `i32` / `f32` on purpose: these values
/// cross an FFI boundary into the sherpa C API unchanged, and a wider Rust type
/// here would put a narrowing cast between the catalogued value and the engine
/// that reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct DetectionConfig {
    /// Feature-extractor rate. Read from
    /// [`SampleRate::HZ_16000`][via_audio::SampleRate::HZ_16000], never
    /// retyped — see the module docs.
    pub sample_rate: SampleRate,
    /// Mel-filterbank dimension. [`FEATURE_DIM`].
    pub feature_dim: i32,
    /// The four model files.
    pub model: ModelPaths,
    /// Decoder threads. [`NUM_THREADS`].
    pub num_threads: i32,
    /// ONNX execution provider. [`PROVIDER_CPU`].
    pub provider: String,
    /// Engine debug logging. [`DEBUG`].
    pub debug: bool,
    /// Token modelling unit. [`MODELING_UNIT_CJKCHAR`].
    pub modeling_unit: String,
    /// Beam width. [`MAX_ACTIVE_PATHS`].
    pub max_active_paths: i32,
    /// Trailing blanks before emission. [`NUM_TRAILING_BLANKS`].
    pub num_trailing_blanks: i32,
    /// Per-keyword boost. [`KEYWORDS_SCORE`].
    pub keywords_score: f32,
    /// Detection threshold. [`KEYWORDS_THRESHOLD`].
    pub keywords_threshold: f32,
    /// The keyword file's *content*, not its path.
    ///
    /// **External contract** — `sherpa-detector.mjs:44-49`: upstream passes
    /// `keywords: ''` and supplies `keywordsBuf` / `keywordsBufSize` instead,
    /// with the comment *"Supplying the verified local file as a buffer keeps
    /// keyword loading independent of the WASM virtual filesystem."* VIA has no
    /// WASM filesystem, but the buffer is still the right seam for a different
    /// reason: the phrase is configuration, so the keyword text can change
    /// between two runs against one installed model, and a buffer cannot go
    /// stale the way a path can.
    pub keywords_buf: String,
}

impl DetectionConfig {
    /// The catalogued configuration for one install and one keyword file.
    #[must_use]
    pub fn new(model: ModelPaths, keywords_buf: impl Into<String>) -> Self {
        Self {
            sample_rate: SampleRate::HZ_16000,
            feature_dim: FEATURE_DIM,
            model,
            num_threads: NUM_THREADS,
            provider: PROVIDER_CPU.to_owned(),
            debug: DEBUG,
            modeling_unit: MODELING_UNIT_CJKCHAR.to_owned(),
            max_active_paths: MAX_ACTIVE_PATHS,
            num_trailing_blanks: NUM_TRAILING_BLANKS,
            keywords_score: KEYWORDS_SCORE,
            keywords_threshold: KEYWORDS_THRESHOLD,
            keywords_buf: keywords_buf.into(),
        }
    }

    /// The byte length upstream passes as `keywordsBufSize`.
    ///
    /// **External contract** — `sherpa-detector.mjs:49`
    /// (`Buffer.byteLength(keywords)`). Bytes, not characters: a keyword file
    /// holding a CJK phrase has roughly three times as many bytes as `char`s,
    /// and passing the character count truncates the buffer mid-keyword. The
    /// Rust binding derives the same number from the buffer itself, so this
    /// method exists to *assert* the value rather than to pass it.
    #[must_use]
    pub fn keywords_buf_size(&self) -> usize {
        self.keywords_buf.len()
    }

    /// The path field upstream sets to the empty string.
    ///
    /// **External contract** — `sherpa-detector.mjs:47` (`keywords: ''`). The
    /// Rust binding spells the same thing `keywords_file: None`, and this is
    /// the one place that equivalence is written down.
    #[must_use]
    pub fn keywords_file(&self) -> Option<&Path> {
        None
    }
}

/// One wake-word match.
#[derive(Clone, Debug, PartialEq)]
pub struct Detection {
    /// The display text of the keyword that matched — the right-hand column of
    /// the keyword file line, which is the configured phrase.
    pub keyword: String,
    /// The tokens the engine decoded, its own spelling of the match.
    pub tokens: String,
    /// Where the match started, in seconds from the beginning of the stream.
    pub start_time_seconds: f32,
    /// Which configured locale's phrase this was.
    ///
    /// A [`WakeWordDetector`] on its own does not know — it sees a keyword
    /// file, not a locale table — so an implementation leaves this `None` and
    /// [`WakeWordStream`](crate::WakeWordStream) fills it in from the
    /// [`KeywordSet`](crate::KeywordSet) it was built with.
    pub locale: Option<Locale>,
}

impl Detection {
    /// A detection of `keyword` with no token detail and no start time.
    #[must_use]
    pub fn new(keyword: impl Into<String>) -> Self {
        Self {
            keyword: keyword.into(),
            tokens: String::new(),
            start_time_seconds: 0.0,
            locale: None,
        }
    }

    /// The same detection, attributed to `locale`.
    #[must_use]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = Some(locale);
        self
    }
}

/// A streaming wake-word detector.
///
/// One method that matters. Feed it mono `f32` samples at
/// [`WakeWordDetector::sample_rate`]; it answers `Some` exactly once per match
/// and `None` the rest of the time.
///
/// # The contract implementations must keep
///
/// - **An empty slice is a no-op.** Upstream returns early on
///   `!samples.length` (`sherpa-detector.mjs:70`), and a resampler that has not
///   yet accumulated a whole chunk hands out empty slices routinely.
/// - **A match resets the stream.** Upstream calls `reset` immediately after
///   reading a non-empty keyword (`sherpa-detector.mjs:76`), so the same
///   utterance cannot fire twice and the decoder starts the next wake from a
///   clean state.
///
/// Both are asserted against [`ScriptedDetector`](crate::ScriptedDetector) as
/// well as against the engine, because a test double that gets them wrong tests
/// nothing.
///
/// `Sync` as well as `Send`: a connection holds its stream behind `&self`
/// across an `.await`, which needs the whole chain — including a boxed
/// trait object — to be shareable across threads, not merely movable to one.
pub trait WakeWordDetector: Send + Sync + fmt::Debug {
    /// Feed one chunk of mono samples.
    fn accept(&mut self, samples: &[f32]) -> Option<Detection>;

    /// Forget everything decoded so far.
    fn reset(&mut self);

    /// The rate `accept` expects. Always [`SampleRate::HZ_16000`] for the
    /// catalogued configuration; the method exists so
    /// [`WakeWordStream`](crate::WakeWordStream) can build the right resampler
    /// instead of assuming.
    fn sample_rate(&self) -> SampleRate {
        SampleRate::HZ_16000
    }
}

/// A boxed detector is a detector.
///
/// Without this, [`WakeWordStream`](crate::WakeWordStream) could hold a
/// concrete `D: WakeWordDetector` but never a *dynamically chosen* one — and a
/// caller composing "the real engine when `--features sherpa` is on, a
/// scripted double in a test, nothing in between" needs exactly that: one
/// connection-level type that does not name which engine built it. See
/// `via-voice`'s wake-word module, the production seam this exists for.
impl WakeWordDetector for Box<dyn WakeWordDetector> {
    fn accept(&mut self, samples: &[f32]) -> Option<Detection> {
        (**self).accept(samples)
    }

    fn reset(&mut self) {
        (**self).reset();
    }

    fn sample_rate(&self) -> SampleRate {
        (**self).sample_rate()
    }
}
