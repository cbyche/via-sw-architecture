//! The real engine — `--features sherpa` only.
//!
//! Ported from upstream `server/src/voice/wake-word/sherpa-detector.mjs:20-81`.
//!
//! This is the half of the crate that is **not** in the default build. Turning
//! the feature on pulls `sherpa-onnx`, whose `-sys` crate fetches prebuilt
//! native libraries at build time and links them statically. `docs/adr/0001`
//! records why that is opt-in: VIA's native dependencies are exactly what keeps
//! it out of ARGO's workspace, and a native toolchain in every CI lane for a
//! feature most builds never touch is that same decision made badly at a
//! smaller scale.
//!
//! Everything above this module — the installer, the keyword table, the
//! configuration, the streaming front end, the [`WakeWordDetector`] contract —
//! is engine-independent and compiles without it.
//!
//! # What is different from the JavaScript
//!
//! **`keywords_buf_size` is not passed.** Upstream must pass it because the
//! Node binding takes a length beside the pointer; the Rust binding derives it
//! from the buffer's own length. [`DetectionConfig::keywords_buf_size`] still
//! exists so the catalogued value is asserted rather than assumed.
//!
//! **`keywords: ''` is `keywords_file: None`.** The same statement — *load
//! keywords from the buffer, not from a path* — in the two bindings' spellings.
//!
//! **`debug: 0` is `debug: false`.** The C API's `int`, typed.
//!
//! **A missing model file is reported before the engine is asked.** The C API
//! answers a null spotter and no reason, so "the joiner is missing" and "the
//! ONNX graph is corrupt" arrive identically. Checking the four paths first
//! turns the common one into
//! [`WakeWordError::Incomplete`](crate::WakeWordError::Incomplete), which names
//! the file.
//!
//! **The keyword file is validated before the engine is constructed.** This one
//! is not a nicety. `sherpa-onnx` does not report an unencodable token — it logs
//! and ends the process. `open` therefore reads the model's own `tokens.txt`
//! and answers
//! [`WakeWordError::UnknownTokens`](crate::WakeWordError::UnknownTokens)
//! itself. See [`crate::tokens`] for the parser rule it reproduces.

use std::fmt;

use sherpa_onnx::{KeywordSpotter, KeywordSpotterConfig, OnlineStream};
use via_audio::SampleRate;

use crate::detect::{Detection, DetectionConfig, WakeWordDetector};
use crate::error::{KeywordError, Result, WakeWordError};
use crate::tokens::TokenInventory;

/// A keyword spotter and the one stream it decodes.
///
/// One stream per detector, as upstream does: the spotter holds the model and
/// is shareable, the stream holds the decoder state for one microphone.
pub struct SherpaDetector {
    spotter: KeywordSpotter,
    stream: OnlineStream,
    sample_rate: SampleRate,
    rate_hz: i32,
}

impl fmt::Debug for SherpaDetector {
    /// Hand-written because neither `KeywordSpotter` nor `OnlineStream` is
    /// `Debug`: both are raw pointers into the C library.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SherpaDetector")
            .field("sample_rate", &self.sample_rate.hz())
            .finish_non_exhaustive()
    }
}

impl SherpaDetector {
    /// Open the engine described by `config`.
    ///
    /// # Errors
    ///
    /// - [`KeywordError::Empty`] when the keyword buffer is blank. A spotter
    ///   with no keywords loads, runs, and never matches anything.
    /// - [`WakeWordError::Incomplete`] when one of the four model files is not
    ///   on disk.
    /// - [`WakeWordError::UnknownTokens`] when the model's `tokens.txt` cannot
    ///   encode the keyword file. The library would end the process instead.
    /// - [`WakeWordError::Engine`] when the library refuses the configuration —
    ///   which is all it tells anyone.
    pub fn open(config: &DetectionConfig) -> Result<Self> {
        if config.keywords_buf.trim().is_empty() {
            return Err(KeywordError::Empty { field: "keywords" }.into());
        }

        let missing: Vec<String> = config
            .model
            .all()
            .iter()
            .filter(|path| !path.is_file())
            .map(|path| path.display().to_string())
            .collect();
        if !missing.is_empty() {
            return Err(WakeWordError::Incomplete { missing });
        }

        // The guard that has to run before `KeywordSpotter::create`, not after:
        // a token the model cannot encode does not produce a null spotter, it
        // ends the process. See `crate::tokens`.
        TokenInventory::read(&config.model.tokens)?.validate(&config.keywords_buf)?;

        let rate_hz =
            i32::try_from(config.sample_rate.hz()).map_err(|_| WakeWordError::Engine {
                detail: format!(
                    "the feature extractor rate {} does not fit the engine's i32",
                    config.sample_rate.hz()
                ),
            })?;

        let mut spotter_config = KeywordSpotterConfig::default();
        spotter_config.feat_config.sample_rate = rate_hz;
        spotter_config.feat_config.feature_dim = config.feature_dim;
        spotter_config.model_config.transducer.encoder = Some(path_string(&config.model.encoder)?);
        spotter_config.model_config.transducer.decoder = Some(path_string(&config.model.decoder)?);
        spotter_config.model_config.transducer.joiner = Some(path_string(&config.model.joiner)?);
        spotter_config.model_config.tokens = Some(path_string(&config.model.tokens)?);
        spotter_config.model_config.num_threads = config.num_threads;
        spotter_config.model_config.provider = Some(config.provider.clone());
        spotter_config.model_config.debug = config.debug;
        spotter_config.model_config.modeling_unit = Some(config.modeling_unit.clone());
        spotter_config.max_active_paths = config.max_active_paths;
        spotter_config.num_trailing_blanks = config.num_trailing_blanks;
        spotter_config.keywords_score = config.keywords_score;
        spotter_config.keywords_threshold = config.keywords_threshold;
        // `keywords: ''` upstream: keywords come from the buffer, not a path.
        spotter_config.keywords_file = None;
        spotter_config.keywords_buf = Some(config.keywords_buf.clone());

        let spotter =
            KeywordSpotter::create(&spotter_config).ok_or_else(|| WakeWordError::Engine {
                detail: "sherpa-onnx returned no keyword spotter for this configuration".to_owned(),
            })?;
        let stream = spotter.create_stream();

        Ok(Self {
            spotter,
            stream,
            sample_rate: config.sample_rate,
            rate_hz,
        })
    }
}

impl WakeWordDetector for SherpaDetector {
    fn accept(&mut self, samples: &[f32]) -> Option<Detection> {
        // `sherpa-detector.mjs:70`: an empty chunk is not decoded.
        if samples.is_empty() {
            return None;
        }
        self.stream.accept_waveform(self.rate_hz, samples);
        while self.spotter.is_ready(&self.stream) {
            self.spotter.decode(&self.stream);
        }
        let result = self.spotter.get_result(&self.stream)?;
        if result.keyword.is_empty() {
            return None;
        }
        // `sherpa-detector.mjs:76`: a match resets the stream, so one utterance
        // cannot fire twice.
        self.spotter.reset(&self.stream);
        Some(Detection {
            keyword: result.keyword,
            tokens: result.tokens,
            start_time_seconds: result.start_time,
            locale: None,
        })
    }

    fn reset(&mut self) {
        self.spotter.reset(&self.stream);
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }
}

/// A model path as the `String` the C API wants.
fn path_string(path: &std::path::Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| WakeWordError::Engine {
            detail: format!("the model path {} is not valid UTF-8", path.display()),
        })
}
