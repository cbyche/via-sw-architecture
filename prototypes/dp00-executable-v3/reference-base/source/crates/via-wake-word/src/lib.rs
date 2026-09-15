//! VIA's on-device wake word.
//!
//! Ported from upstream `qwen-audio-agent` v1.11.0
//! `server/src/voice/wake-word/{model-manager,sherpa-detector}.mjs`.
//!
//! # The crate has two halves, and only one of them is in the default build
//!
//! | Half | Feature | What is in it |
//! | --- | --- | --- |
//! | **The pipeline** | *default* | [`ModelManager`] · [`ModelArtifact`] · [`KeywordSet`] · [`TokenInventory`] · [`WakeWordSettings`] · [`DetectionConfig`] · [`WakeWordDetector`] · [`WakeWordStream`] · [`ScriptedDetector`] · [`ScriptedFetch`] |
//! | **The engine** | `sherpa` | [`SherpaDetector`] — the real `sherpa-onnx` keyword spotter |
//! | The production fetcher | `http` | [`HttpModelFetch`] — one `reqwest` GET |
//!
//! `cargo build -p via-wake-word` needs no C toolchain, no prebuilt native
//! library, and no network. `cargo test -p via-wake-word` runs the whole
//! pipeline — download, digest, extraction, install, keyword resolution,
//! resampling, detection — against [`ScriptedFetch`] and [`ScriptedDetector`],
//! with no bytes leaving the machine.
//!
//! **Why the line is drawn there.** `docs/adr/0001-placement.md` records that
//! VIA's native dependencies — `sherpa-onnx`, `llama-cpp-2`, `cpal` — are
//! precisely what keeps VIA out of ARGO's workspace, because ARGO requires
//! every crate to cross-compile clean to four targets. Putting a native
//! toolchain into every VIA build lane, for a feature most builds never
//! exercise, is that same decision made badly at a smaller scale. So the seam
//! is a trait, the trait is in the default build, and the engine is behind
//! `--features sherpa`.
//!
//! # The phrase is configuration, never a literal
//!
//! `docs/architecture.md` §16: *"The phrase is a configuration value, never a
//! literal; the keyword model is a downloadable checksummed artifact resolved
//! per phrase; and wake word is per-locale, so three locales may mean three
//! models."*
//!
//! **VIA has not chosen its wake phrase.** There is no default in this crate —
//! no constant, no `Default`, no fallback. A deployment that configures nothing
//! resolves to [`DisabledReason::NoPhraseConfigured`], which is a state its
//! caller handles rather than an error it recovers from. Upstream's phrase
//! belongs to the upstream product (`docs/rebrand.md` rows 147-149) and appears
//! in this repository only as a labelled test fixture.
//!
//! The per-locale part is [`KeywordSet`]: a table keyed by
//! [`Locale`](via_i18n::Locale), so at most [`MAX_KEYWORD_MODELS`] entries and
//! no way to express a fourth. Two locales may share one
//! [`ModelArtifact`] — `en` and `zh` do, on the catalogued zh-en model — and
//! then they share one download and one install directory.
//!
//! # One guard that is not upstream's
//!
//! `sherpa-onnx` does not *return* an error for a keyword its model cannot
//! encode. It logs *"Cannot find ID for token …"*, logs *"Encode keywords
//! failed."*, and ends the process. Upstream can live with that: its token line
//! is a hard-coded literal that was correct when it was written. VIA's is
//! configuration, so a mistyped token would take the Gateway down instead of
//! disabling a feature.
//!
//! [`TokenInventory`] therefore reads the model's own `tokens.txt`, applies the
//! same rule `EncodeBase` applies, and answers
//! [`WakeWordError::UnknownTokens`] — at **install**, so a keyword file that
//! would kill the Gateway is never written next to a model, and again at
//! **open**. It catches the sibling hazard too: `EncodeBase` reaches
//! `std::stof` on a `:score` or `#threshold` payload with no `try`, so a marker
//! carrying no number is an uncaught C++ exception rather than a rejected
//! value ([`WakeWordError::MalformedMarker`]). It is also why [`WakePhrase`] has a *label* distinct from its
//! text: the keyword file's display column is one whitespace-free word, and a
//! multi-word phrase written straight into it makes its second word an
//! unencodable token.
//!
//! # What it is built on rather than restating
//!
//! | Owned by | What |
//! | --- | --- |
//! | `via-audio` | [`SampleRate::HZ_16000`](via_audio::SampleRate::HZ_16000), the resampler, the frame buffer, every PCM16 ↔ `f32` conversion |
//! | `via-core` | `Config` — the enable flag, the phrase, the install root, the locale |
//! | `via-i18n` | every sentence a person reads, and `Locale` |
//! | `via-store` | the crash-atomic file replace and `FILE_MODE` |
//!
//! In particular the feature extractor's 16 000 Hz is **read** from
//! `via-audio`, not retyped: `via-audio`'s own `tests/contracts.rs` asserts it
//! against the same `sherpa KWS detection config` catalogue row, and
//! `docs/deviations/phase-0.md` records the split. The other nine fields of
//! that contract are asserted here. A rate that disagreed between the resampler
//! and the extractor would not crash anything — it would just stop hearing the
//! phrase.
//!
//! # Example
//!
//! Install a model, build a stream, feed it 48 kHz stereo capture:
//!
//! ```
//! use via_audio::{ChannelCount, SampleRate};
//! use via_i18n::Locale;
//! use via_wake_word::{
//!     Keyword, KeywordSet, ScriptedDetector, WakeWordSettings, WakeWordStream,
//! };
//!
//! # fn main() -> Result<(), via_wake_word::WakeWordError> {
//! // The phrase and its token line are configuration. This one is a fixture:
//! // ARPAbet with stress digits, which is how the catalogued zh-en model
//! // spells English.
//! let keywords = KeywordSet::new().with(
//!     Locale::En,
//!     Keyword::zh_en("hey via", "HH EY1 V IY1 AH0")?,
//! );
//!
//! let settings = WakeWordSettings::new(true, "/var/lib/via/models/wake-word", keywords.clone());
//! assert_eq!(settings.phrase(Locale::En), Some("hey via"));
//! assert_eq!(
//!     settings.asleep_message(Locale::En).as_deref(),
//!     Some("Asleep — say “hey via” to wake me."),
//! );
//!
//! // A detector that will match on its first non-empty chunk. The engine
//! // reports the *label* — the keyword file's display column, which cannot
//! // contain a space.
//! let detector = ScriptedDetector::detecting("hey_via");
//! let mut stream = WakeWordStream::with_channels(
//!     detector,
//!     SampleRate::HZ_48000,
//!     ChannelCount::STEREO,
//!     keywords,
//! )?;
//!
//! // 20 ms of 48 kHz stereo silence: 960 frames × 2 channels × 2 bytes.
//! let mut detected = None;
//! for _ in 0..8 {
//!     if let Some(event) = stream.accept_pcm16le(&[0u8; 3_840])? {
//!         detected = Some(event);
//!         break;
//!     }
//! }
//! let detected = detected.expect("the script fires once the resampler has a chunk");
//! assert_eq!(detected.keyword, "hey_via");
//! assert_eq!(detected.locale, Some(Locale::En));
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod artifact;
pub mod detect;
pub mod error;
pub mod install;
pub mod keyword;
pub mod scripted;
pub mod settings;
pub mod stream;
pub mod tokens;

#[cfg(feature = "sherpa")]
pub mod sherpa;

pub use artifact::{
    ModelArtifact, ModelFiles, WAKE_WORD_MODEL_ARCHIVE_SUFFIX, WAKE_WORD_MODEL_FILES,
    WAKE_WORD_MODEL_NAME, WAKE_WORD_MODEL_RELEASE_BASE, WAKE_WORD_MODEL_SHA256,
};
pub use detect::{
    DEBUG, Detection, DetectionConfig, FEATURE_DIM, KEYWORDS_SCORE, KEYWORDS_THRESHOLD,
    MAX_ACTIVE_PATHS, MODELING_UNIT_CJKCHAR, ModelPaths, NUM_THREADS, NUM_TRAILING_BLANKS,
    PROVIDER_CPU, WakeWordDetector,
};
pub use error::{KeywordError, Result, WakeWordError};
pub use install::{FetchResponse, MODEL_DIR_MODE, ModelFetch, ModelInstall, ModelManager};
pub use keyword::{
    KEYWORD_DISPLAY_SEPARATOR, Keyword, KeywordSet, LABEL_WORD_SEPARATOR, MAX_KEYWORD_MODELS,
    WakePhrase,
};
pub use scripted::{ScriptStep, ScriptedDetector, ScriptedFetch};
pub use settings::{DisabledReason, Resolution, WakeWordSettings};
pub use stream::WakeWordStream;
pub use tokens::{KEYWORD_MARKERS, TokenInventory};

#[cfg(feature = "http")]
pub use install::HttpModelFetch;

#[cfg(feature = "sherpa")]
pub use sherpa::SherpaDetector;
