//! Where the pipeline's model files are, and what happens when one is not
//! there.
//!
//! # The bug this module exists to make impossible
//!
//! `docs/architecture.md` §7, *"One bug not to port"*: upstream answers an
//! unknown realtime model id with an all-capabilities-false profile, and
//! `dashscope.mjs:84-87` then gates `session.turn_detection` on
//! `transportCapabilities.audioInput` — so an unknown id **opens a session that
//! hears nothing**. A local model id is by definition not in the DashScope
//! table, so on this path that fallback would be reached on the happy path.
//!
//! `via-catalog` already fixed the first half: [`local_realtime_model_profile`]
//! carries real flags, `audio_input` included, and there is no `unknown` family
//! to fall into. This module is the second half. A model file that is not on
//! disk is [`LocalError::ModelFileMissing`] **naming the path**, raised from
//! [`preflight`] before a socket, a task or a session exists — never a session
//! that connects and then hears nothing.
//!
//! # The layout
//!
//! One root, four stages, conventional names:
//!
//! ```text
//! <root>/silero_vad.onnx                 vad
//! <root>/asr/{encoder,decoder,joiner}.onnx
//! <root>/asr/tokens.txt                  asr
//! <root>/reasoning/<anything>.gguf       reasoning
//! <root>/tts/model.onnx
//! <root>/tts/tokens.txt
//! <root>/tts/voices.bin
//! <root>/tts/espeak-ng-data/             tts
//! ```
//!
//! Every path is a field, so a deployment that keeps its GGUF somewhere else
//! sets that one field and inherits the rest. [`WeightsSet::resolve`] is the
//! convention; the struct is the contract.
//!
//! **The root is not read from the environment here.** `via-core` owns
//! environment reading and ships with no local-model variable of its own, and
//! this crate does not edit it — so the root is derived from
//! [`InstallPaths`](via_core::InstallPaths), which is the same place
//! `via-wake-word` gets `models/wake-word` from, and a caller may override it.
//! `docs/deviations/phase-8-via-realtime-local.md` records the choice.
//!
//! [`local_realtime_model_profile`]: via_catalog::local_realtime_model_profile
//! [`preflight`]: via_realtime::RealtimeProvider::preflight

use std::path::{Path, PathBuf};

use crate::error::{LocalError, Result};
use crate::stages::Stage;

/// The pipeline's model root, relative to the configuration directory.
///
/// Beside `via-core`'s `models/wake-word`, and for the same reason: model
/// artifacts are large, replaceable and not state, so they live under one
/// `models/` parent rather than beside `tasks.json`.
pub const LOCAL_MODEL_DIRECTORY: &str = "models/local-omni";

/// The Silero VAD graph.
pub const VAD_FILE: &str = "silero_vad.onnx";

/// The streaming-ASR subdirectory.
pub const ASR_DIRECTORY: &str = "asr";

/// The streaming transducer's three graphs and its token inventory, in the
/// order a sherpa transducer configuration names them.
///
/// The same four members `via-wake-word`'s keyword model has, because it is the
/// same model architecture — a streaming zipformer transducer — with a
/// different head.
pub const ASR_FILES: [&str; 4] = ["encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt"];

/// The reasoning subdirectory.
pub const REASONING_DIRECTORY: &str = "reasoning";

/// The reasoning model's file name under [`REASONING_DIRECTORY`].
///
/// A GGUF. The *name* is conventional and the *stem* is the model id this
/// provider publishes, which is why a deployment that wants a legible id in
/// `/api/health` renames the file rather than setting a variable.
pub const REASONING_FILE: &str = "model.gguf";

/// The GGUF extension, checked when a directory is scanned for a model.
pub const GGUF_EXTENSION: &str = "gguf";

/// The speech-synthesis subdirectory.
pub const TTS_DIRECTORY: &str = "tts";

/// The synthesis graph.
pub const TTS_MODEL_FILE: &str = "model.onnx";

/// The synthesis token inventory.
pub const TTS_TOKENS_FILE: &str = "tokens.txt";

/// The voice pack.
///
/// Kokoro ships every voice in one file; a Piper install has one graph per
/// voice and no pack, which is why this member is [`Option`] rather than a
/// fifth required path.
pub const TTS_VOICES_FILE: &str = "voices.bin";

/// The phonemizer data directory.
///
/// A directory, not a file — the one member of the set that has to be checked
/// with `is_dir`, and the reason [`RequiredPath`] carries a kind at all.
pub const TTS_DATA_DIRECTORY: &str = "espeak-ng-data";

/// One path the pipeline needs, and which stage needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredPath {
    /// The stage that would load it.
    pub stage: Stage,
    /// Where it is expected.
    pub path: PathBuf,
    /// Whether a directory is expected rather than a file.
    pub is_directory: bool,
}

impl RequiredPath {
    /// A required file.
    #[must_use]
    pub fn file(stage: Stage, path: impl Into<PathBuf>) -> Self {
        Self {
            stage,
            path: path.into(),
            is_directory: false,
        }
    }

    /// A required directory.
    #[must_use]
    pub fn directory(stage: Stage, path: impl Into<PathBuf>) -> Self {
        Self {
            stage,
            path: path.into(),
            is_directory: true,
        }
    }

    /// Whether it is there, and of the right kind.
    ///
    /// The kind check is not pedantry: `espeak-ng-data` extracted as a tarball
    /// beside its own directory leaves a *file* by that name, and a phonemizer
    /// pointed at it fails inside the C library rather than at configuration
    /// time.
    #[must_use]
    pub fn is_present(&self) -> bool {
        if self.is_directory {
            self.path.is_dir()
        } else {
            self.path.is_file()
        }
    }
}

/// The streaming recognizer's four files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsrWeights {
    /// The transducer encoder.
    pub encoder: PathBuf,
    /// The transducer decoder.
    pub decoder: PathBuf,
    /// The transducer joiner.
    pub joiner: PathBuf,
    /// The token inventory.
    pub tokens: PathBuf,
}

impl AsrWeights {
    /// The conventional layout under `directory`.
    #[must_use]
    pub fn resolve(directory: &Path) -> Self {
        Self {
            encoder: directory.join(ASR_FILES[0]),
            decoder: directory.join(ASR_FILES[1]),
            joiner: directory.join(ASR_FILES[2]),
            tokens: directory.join(ASR_FILES[3]),
        }
    }

    /// The four paths, in [`ASR_FILES`] order.
    #[must_use]
    pub fn all(&self) -> [&Path; 4] {
        [&self.encoder, &self.decoder, &self.joiner, &self.tokens]
    }
}

/// The synthesizer's files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsWeights {
    /// The synthesis graph.
    pub model: PathBuf,
    /// The token inventory.
    pub tokens: PathBuf,
    /// The voice pack, when the engine has one.
    pub voices: Option<PathBuf>,
    /// The phonemizer data directory, when the engine needs one.
    pub data_directory: Option<PathBuf>,
}

impl TtsWeights {
    /// The conventional Kokoro layout under `directory`.
    #[must_use]
    pub fn resolve(directory: &Path) -> Self {
        Self {
            model: directory.join(TTS_MODEL_FILE),
            tokens: directory.join(TTS_TOKENS_FILE),
            voices: Some(directory.join(TTS_VOICES_FILE)),
            data_directory: Some(directory.join(TTS_DATA_DIRECTORY)),
        }
    }
}

/// Every model file the pipeline loads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightsSet {
    /// The directory the conventional layout was resolved against.
    pub root: PathBuf,
    /// The Silero VAD graph.
    pub voice_activity: PathBuf,
    /// The streaming recognizer.
    pub transcriber: AsrWeights,
    /// The reasoning GGUF.
    pub reasoning: PathBuf,
    /// The synthesizer.
    pub speaker: TtsWeights,
}

impl WeightsSet {
    /// The conventional layout under `root`.
    #[must_use]
    pub fn resolve(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            voice_activity: root.join(VAD_FILE),
            transcriber: AsrWeights::resolve(&root.join(ASR_DIRECTORY)),
            reasoning: root.join(REASONING_DIRECTORY).join(REASONING_FILE),
            speaker: TtsWeights::resolve(&root.join(TTS_DIRECTORY)),
            root,
        }
    }

    /// The same layout, with the reasoning model taken from whatever `.gguf`
    /// is in `<root>/reasoning`.
    ///
    /// A GGUF's file name is its provenance — `Qwen3-8B-Q4_K_M.gguf` — and an
    /// operator who downloaded one is not going to rename it to `model.gguf`.
    /// So the conventional name is what [`resolve`](Self::resolve) *expects* and
    /// this is what a Gateway *finds*: exactly one `.gguf` is adopted, and two
    /// leave the conventional name in place rather than picking one at random,
    /// because picking would make the answer depend on directory order.
    #[must_use]
    pub fn discover(root: impl Into<PathBuf>) -> Self {
        let mut set = Self::resolve(root);
        if set.reasoning.is_file() {
            return set;
        }
        let mut found: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(set.root.join(REASONING_DIRECTORY)) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case(GGUF_EXTENSION))
                {
                    found.push(path);
                }
            }
        }
        // Sorted so a directory listing's order cannot change the answer, then
        // adopted only when it is unambiguous.
        found.sort();
        if let [only] = found.as_slice() {
            set.reasoning.clone_from(only);
        }
        set
    }

    /// Every path the pipeline needs, in pipeline order.
    #[must_use]
    pub fn required(&self) -> Vec<RequiredPath> {
        let mut required = vec![RequiredPath::file(Stage::Vad, self.voice_activity.clone())];
        required.extend(
            self.transcriber
                .all()
                .into_iter()
                .map(|path| RequiredPath::file(Stage::Asr, path.to_path_buf())),
        );
        required.push(RequiredPath::file(Stage::Reasoning, self.reasoning.clone()));
        required.push(RequiredPath::file(Stage::Tts, self.speaker.model.clone()));
        required.push(RequiredPath::file(Stage::Tts, self.speaker.tokens.clone()));
        if let Some(voices) = &self.speaker.voices {
            required.push(RequiredPath::file(Stage::Tts, voices.clone()));
        }
        if let Some(data) = &self.speaker.data_directory {
            required.push(RequiredPath::directory(Stage::Tts, data.clone()));
        }
        required
    }

    /// Every required path that is not there, in pipeline order.
    #[must_use]
    pub fn missing(&self) -> Vec<RequiredPath> {
        self.required()
            .into_iter()
            .filter(|required| !required.is_present())
            .collect()
    }

    /// Whether every required path is there.
    #[must_use]
    pub fn is_installed(&self) -> bool {
        self.missing().is_empty()
    }

    /// The model id this provider publishes: the reasoning file's stem.
    ///
    /// `None` when the stem is empty or not valid UTF-8, which is the only
    /// state in which this provider has no model id at all — and
    /// [`RealtimeProvider::model`](via_realtime::RealtimeProvider::model) is
    /// `Option` precisely so that state does not have to be invented around.
    #[must_use]
    pub fn model_id(&self) -> Option<String> {
        self.reasoning
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::trim)
            .filter(|stem| !stem.is_empty())
            .map(str::to_owned)
    }

    /// Refuse a pipeline whose weights are not on disk.
    ///
    /// # Errors
    ///
    /// [`LocalError::PipelineNotInstalled`] naming the root when **nothing** is
    /// there, and [`LocalError::ModelFileMissing`] naming the first absent path
    /// otherwise. The two are different problems: none present is an install
    /// that never happened, one absent is an install that broke, and telling an
    /// operator to reinstall when they have seven of eight files wastes a
    /// download.
    pub fn verify(&self) -> Result<()> {
        let missing = self.missing();
        let Some(first) = missing.first() else {
            return Ok(());
        };
        if missing.len() == self.required().len() {
            return Err(LocalError::PipelineNotInstalled {
                root: self.root.clone(),
            });
        }
        Err(LocalError::model_file_missing(first.stage, &first.path))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(path, b"x").expect("write fixture");
    }

    fn install(root: &Path) -> WeightsSet {
        let set = WeightsSet::resolve(root);
        for required in set.required() {
            if required.is_directory {
                std::fs::create_dir_all(&required.path).expect("create directory");
            } else {
                touch(&required.path);
            }
        }
        set
    }

    #[test]
    fn the_conventional_layout_is_one_root_and_four_stages() {
        let set = WeightsSet::resolve("/models/local-omni");
        assert_eq!(
            set.voice_activity,
            PathBuf::from("/models/local-omni/silero_vad.onnx")
        );
        assert_eq!(
            set.transcriber.encoder,
            PathBuf::from("/models/local-omni/asr/encoder.onnx")
        );
        assert_eq!(
            set.reasoning,
            PathBuf::from("/models/local-omni/reasoning/model.gguf")
        );
        assert_eq!(
            set.speaker.model,
            PathBuf::from("/models/local-omni/tts/model.onnx")
        );
        assert_eq!(
            set.speaker.data_directory,
            Some(PathBuf::from("/models/local-omni/tts/espeak-ng-data"))
        );
    }

    #[test]
    fn every_stage_is_represented_in_the_required_set_exactly_once_per_file() {
        let set = WeightsSet::resolve("/models/local-omni");
        let required = set.required();
        assert_eq!(required.len(), 10, "{required:#?}");
        for stage in Stage::ALL {
            assert!(
                required.iter().any(|entry| entry.stage == stage),
                "{stage} has no required path"
            );
        }
        // Only one of them is a directory, and it is the phonemizer's.
        let directories: Vec<&RequiredPath> =
            required.iter().filter(|entry| entry.is_directory).collect();
        assert_eq!(directories.len(), 1);
        assert!(directories[0].path.ends_with(TTS_DATA_DIRECTORY));
    }

    #[test]
    fn a_complete_install_verifies() {
        let root = tempfile::tempdir().expect("tempdir");
        let set = install(root.path());
        assert!(set.is_installed());
        assert_eq!(set.missing(), Vec::new());
        assert_eq!(set.verify(), Ok(()));
    }

    #[test]
    fn an_empty_root_is_an_install_that_never_happened() {
        let root = tempfile::tempdir().expect("tempdir");
        let set = WeightsSet::resolve(root.path());
        assert_eq!(
            set.verify(),
            Err(LocalError::PipelineNotInstalled {
                root: root.path().to_path_buf()
            })
        );
    }

    #[test]
    fn one_absent_file_names_that_file_and_its_stage() {
        let root = tempfile::tempdir().expect("tempdir");
        let set = install(root.path());
        std::fs::remove_file(&set.transcriber.joiner).expect("remove joiner");
        assert_eq!(
            set.verify(),
            Err(LocalError::model_file_missing(
                Stage::Asr,
                &set.transcriber.joiner
            ))
        );
    }

    #[test]
    fn the_first_absent_path_is_reported_in_pipeline_order() {
        let root = tempfile::tempdir().expect("tempdir");
        let set = install(root.path());
        std::fs::remove_file(&set.speaker.model).expect("remove tts");
        std::fs::remove_file(&set.voice_activity).expect("remove vad");
        // Both are gone; the VAD is first because it is first in the pipeline.
        assert_eq!(
            set.verify(),
            Err(LocalError::model_file_missing(
                Stage::Vad,
                &set.voice_activity
            ))
        );
    }

    #[test]
    fn a_data_directory_that_is_a_file_is_still_missing() {
        let root = tempfile::tempdir().expect("tempdir");
        let set = install(root.path());
        let data = set
            .speaker
            .data_directory
            .clone()
            .expect("the conventional layout has one");
        std::fs::remove_dir_all(&data).expect("remove directory");
        touch(&data);
        assert!(data.is_file());
        assert_eq!(
            set.verify(),
            Err(LocalError::model_file_missing(Stage::Tts, &data))
        );
    }

    #[test]
    fn the_model_id_is_the_reasoning_files_stem() {
        let set = WeightsSet {
            reasoning: PathBuf::from("/models/local-omni/reasoning/Qwen3-8B-Q4_K_M.gguf"),
            ..WeightsSet::resolve("/models/local-omni")
        };
        assert_eq!(set.model_id().as_deref(), Some("Qwen3-8B-Q4_K_M"));
    }

    #[test]
    fn a_reasoning_path_with_no_stem_has_no_model_id() {
        for path in ["", "/", "/models/.."] {
            let set = WeightsSet {
                reasoning: PathBuf::from(path),
                ..WeightsSet::resolve("/models/local-omni")
            };
            assert_eq!(set.model_id(), None, "{path}");
        }
    }

    #[test]
    fn discovery_adopts_the_one_gguf_and_refuses_to_choose_between_two() {
        let root = tempfile::tempdir().expect("tempdir");
        let reasoning = root.path().join(REASONING_DIRECTORY);
        std::fs::create_dir_all(&reasoning).expect("create reasoning");

        // Nothing there: the conventional name stands.
        assert_eq!(
            WeightsSet::discover(root.path()).reasoning,
            reasoning.join(REASONING_FILE)
        );

        touch(&reasoning.join("Qwen3-8B-Q4_K_M.gguf"));
        assert_eq!(
            WeightsSet::discover(root.path()).reasoning,
            reasoning.join("Qwen3-8B-Q4_K_M.gguf")
        );

        // Two candidates: neither is adopted, so the answer cannot depend on
        // the order the directory happened to be read in.
        touch(&reasoning.join("Qwen3-4B-Q8_0.gguf"));
        assert_eq!(
            WeightsSet::discover(root.path()).reasoning,
            reasoning.join(REASONING_FILE)
        );
    }

    #[test]
    fn discovery_prefers_the_conventional_name_when_it_exists() {
        let root = tempfile::tempdir().expect("tempdir");
        let reasoning = root.path().join(REASONING_DIRECTORY);
        touch(&reasoning.join(REASONING_FILE));
        touch(&reasoning.join("Other-Model.gguf"));
        assert_eq!(
            WeightsSet::discover(root.path()).reasoning,
            reasoning.join(REASONING_FILE)
        );
    }

    #[test]
    fn discovery_ignores_a_directory_wearing_a_gguf_name() {
        let root = tempfile::tempdir().expect("tempdir");
        let reasoning = root.path().join(REASONING_DIRECTORY);
        std::fs::create_dir_all(reasoning.join("Not-A-Model.gguf")).expect("create directory");
        assert_eq!(
            WeightsSet::discover(root.path()).reasoning,
            reasoning.join(REASONING_FILE)
        );
    }

    #[test]
    fn discovery_on_a_root_that_does_not_exist_is_the_conventional_layout() {
        let set = WeightsSet::discover("/nonexistent-root-for-a-test");
        assert_eq!(
            set.reasoning,
            PathBuf::from("/nonexistent-root-for-a-test/reasoning/model.gguf")
        );
        assert!(!set.is_installed());
    }
}
