//! The real VAD, streaming ASR and TTS — `--features sherpa` only.
//!
//! This is the half of the crate that is **not** in the default build. Turning
//! the feature on pulls `sherpa-onnx`, whose `-sys` crate fetches prebuilt
//! native libraries at build time and links them statically.
//! `docs/adr/0001-placement.md` records why that is opt-in: VIA's native
//! dependencies are exactly what keeps it out of ARGO's workspace, and a native
//! toolchain in every CI lane for a feature most builds never touch is that
//! same decision made badly at a smaller scale. `via-wake-word` draws the line
//! in the same place, for the same reason.
//!
//! Everything above this module — the session machine, the four stage traits,
//! both providers, the sentence splitter, the configuration surface — is
//! engine-independent and compiles without it.
//!
//! # What each trait becomes
//!
//! | Trait | Engine | The one thing that is not a straight mapping |
//! | --- | --- | --- |
//! | [`VoiceActivity`] | `VoiceActivityDetector` (Silero) | the library reports a **level**; the trait wants **edges** |
//! | [`Transcriber`] | `OnlineRecognizer` + `OnlineStream` | the library returns the whole hypothesis every time; the wire wants a running transcript |
//! | [`Speaker`] | `OfflineTts` | the library is blocking and callback-driven; the trait wants a droppable stream |
//!
//! **Edges, not levels.** `SherpaOnnxVoiceActivityDetectorDetected` answers *"is
//! speech being detected right now"*, so it is `true` for every block of an
//! utterance. [`SpeechEvent::Started`] must fire once — the machine keys
//! barge-in on it, and a detector that reported `Started` on every voiced block
//! would cancel the same response over and over. [`SherpaVoiceActivity`] holds
//! one `bool` and differentiates.
//!
//! **A running transcript, not a repeated one.** `get_result` returns the
//! current hypothesis in full on every call, so publishing it verbatim would
//! send an identical transcription delta for every 20 ms of silence inside an
//! utterance. The recognizer remembers what it last published and answers an
//! empty update when nothing changed, which is what the machine already skips.
//!
//! **A dropped stream stops synthesis.** `generate_with_config` is blocking and
//! pushes chunks through a callback whose `bool` return means *keep going*.
//! Dropping the [`SpeechStream`] closes the channel, the next `send` fails, the
//! callback answers `false`, and the C++ side unwinds on its own thread — which
//! is exactly the contract [`crate::stages`] states: a dropped stream must not
//! block, and must be inert afterwards.
//!
//! # Not verified against real weights in this repository
//!
//! The API surface below is written against `sherpa-onnx` 1.13's own sources.
//! No model files are checked in, and no lane here builds the native library, so
//! **these three stages have not been run against real weights in this
//! repository.** The seam that has is [`crate::scripted`], which is what the
//! machine is tested through.

use std::sync::Arc;

use futures::stream;
use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig,
    OfflineTtsModelConfig, OnlineModelConfig, OnlineRecognizer, OnlineRecognizerConfig,
    OnlineStream, OnlineTransducerModelConfig, SileroVadModelConfig, VadModelConfig,
    VoiceActivityDetector,
};
use via_audio::SampleRate;

use crate::error::{LocalError, Result};
use crate::stages::{
    PIPELINE_INPUT_RATE, Responder, Speaker, SpeechEvent, SpeechStream, Stage, StageError, Stages,
    Transcriber, TranscriptUpdate, VoiceActivity,
};
use crate::weights::WeightsSet;

/// ONNX execution provider.
///
/// `cpu`, the same value `via-wake-word`'s detector uses. A local pipeline that
/// silently fell back to a GPU provider it could not initialise would report the
/// failure from inside the C++ library.
pub const PROVIDER_CPU: &str = "cpu";

/// Decoder threads for the recognizer and the detector.
///
/// One. Both run continuously beside a realtime session, and a second thread
/// buys nothing on models this size while it competes with the reasoning stage
/// for cores.
pub const NUM_THREADS: i32 = 1;

/// Silero's detection threshold.
///
/// The library's own documented default. Lower it and the pipeline barges in on
/// a cough; raise it and it does not hear a quiet speaker.
pub const VAD_THRESHOLD: f32 = 0.5;

/// Trailing silence, in seconds, before Silero calls an utterance finished.
pub const VAD_MIN_SILENCE_SECONDS: f32 = 0.5;

/// Leading speech, in seconds, before Silero calls an utterance started.
pub const VAD_MIN_SPEECH_SECONDS: f32 = 0.25;

/// Samples per Silero window at 16 kHz.
///
/// Fixed by the model rather than chosen: Silero v5 is trained on 512-sample
/// windows at 16 kHz, and a different value is a different graph.
pub const VAD_WINDOW_SAMPLES: i32 = 512;

/// The longest utterance Silero will hold before forcing an end, in seconds.
pub const VAD_MAX_SPEECH_SECONDS: f32 = 20.0;

/// Seconds of audio the detector buffers.
pub const VAD_BUFFER_SECONDS: f32 = 30.0;

/// Decoding method for the streaming recognizer.
pub const DECODING_METHOD: &str = "greedy_search";

/// The speaker id used when the configured voice does not name one.
pub const DEFAULT_SPEAKER_ID: i32 = 0;

/// Sentences the synthesizer batches per call.
///
/// One, because [`crate::sentence`] has already cut the generation into
/// utterances and the machine hands them over one at a time. Letting the
/// library re-batch would put the whole turn back into a single blocking call,
/// which is the latency this crate's sentence splitter exists to remove.
///
/// It is set explicitly because `OfflineTtsConfig` derives `Default`, so an
/// unset value is `0` rather than the C library's own default.
pub const MAX_SENTENCES_PER_CALL: i32 = 1;

/// Silence appended between sentences, as a fraction of the library's own unit.
///
/// `GenerationConfig`'s hand-written default, restated here because
/// `OfflineTtsConfig`'s is derived and would otherwise be `0.0`.
pub const SILENCE_SCALE: f32 = 0.2;

fn path_string(path: &std::path::Path, stage: Stage) -> Result<String> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        StageError::new(
            stage,
            format!("the path {} is not valid UTF-8", path.display()),
        )
        .into()
    })
}

fn rate_hz(rate: SampleRate, stage: Stage) -> Result<i32> {
    i32::try_from(rate.hz()).map_err(|_| {
        LocalError::from(StageError::new(
            stage,
            format!("the rate {} Hz does not fit the engine's i32", rate.hz()),
        ))
    })
}

// ── voice activity ──────────────────────────────────────────────────────────

/// Silero VAD behind [`VoiceActivity`].
pub struct SherpaVoiceActivity {
    detector: VoiceActivityDetector,
    speaking: bool,
    rate: SampleRate,
}

impl core::fmt::Debug for SherpaVoiceActivity {
    /// Hand-written because `VoiceActivityDetector` is a raw pointer into the C
    /// library and is not `Debug`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SherpaVoiceActivity")
            .field("rate", &self.rate.hz())
            .field("speaking", &self.speaking)
            .finish_non_exhaustive()
    }
}

impl SherpaVoiceActivity {
    /// Open Silero from `weights`.
    ///
    /// # Errors
    ///
    /// [`LocalError::ModelFileMissing`] when the graph is not on disk — checked
    /// first, because the C API answers a null detector with no reason at all,
    /// so "the file is missing" and "the graph is corrupt" would arrive
    /// identically. [`LocalError::Stage`] when the library refuses the
    /// configuration.
    pub fn open(weights: &WeightsSet) -> Result<Self> {
        if !weights.voice_activity.is_file() {
            return Err(LocalError::model_file_missing(
                Stage::Vad,
                &weights.voice_activity,
            ));
        }
        let rate = PIPELINE_INPUT_RATE;
        let config = VadModelConfig {
            silero_vad: SileroVadModelConfig {
                model: Some(path_string(&weights.voice_activity, Stage::Vad)?),
                threshold: VAD_THRESHOLD,
                min_silence_duration: VAD_MIN_SILENCE_SECONDS,
                min_speech_duration: VAD_MIN_SPEECH_SECONDS,
                window_size: VAD_WINDOW_SAMPLES,
                max_speech_duration: VAD_MAX_SPEECH_SECONDS,
            },
            sample_rate: rate_hz(rate, Stage::Vad)?,
            num_threads: NUM_THREADS,
            provider: Some(PROVIDER_CPU.to_owned()),
            debug: false,
            ..VadModelConfig::default()
        };
        let detector =
            VoiceActivityDetector::create(&config, VAD_BUFFER_SECONDS).ok_or_else(|| {
                LocalError::from(StageError::new(
                    Stage::Vad,
                    "sherpa-onnx refused the Silero configuration",
                ))
            })?;
        Ok(Self {
            detector,
            speaking: false,
            rate,
        })
    }
}

impl VoiceActivity for SherpaVoiceActivity {
    fn accept(&mut self, samples: &[f32]) -> core::result::Result<SpeechEvent, StageError> {
        if samples.is_empty() {
            return Ok(SpeechEvent::Silence);
        }
        self.detector.accept_waveform(samples);
        // The library reports a level; the trait wants edges. See the module
        // docs — this `bool` is the whole of the difference.
        let detected = self.detector.detected();
        let event = match (self.speaking, detected) {
            (false, true) => SpeechEvent::Started,
            (true, true) => SpeechEvent::Speaking,
            (true, false) => SpeechEvent::Ended,
            (false, false) => SpeechEvent::Silence,
        };
        self.speaking = detected;
        if event == SpeechEvent::Ended {
            // The completed segment is the machine's only through the
            // recognizer, which was fed the same blocks; holding it would grow
            // the detector's queue for the life of the session.
            self.detector.clear();
        }
        Ok(event)
    }

    fn reset(&mut self) {
        self.detector.reset();
        self.detector.clear();
        self.speaking = false;
    }

    fn sample_rate(&self) -> SampleRate {
        self.rate
    }
}

// ── streaming recognition ───────────────────────────────────────────────────

/// A streaming zipformer transducer behind [`Transcriber`].
pub struct SherpaTranscriber {
    recognizer: OnlineRecognizer,
    stream: OnlineStream,
    published: String,
    rate: SampleRate,
    rate_hz: i32,
}

impl core::fmt::Debug for SherpaTranscriber {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SherpaTranscriber")
            .field("rate", &self.rate.hz())
            .finish_non_exhaustive()
    }
}

impl SherpaTranscriber {
    /// Open the recognizer from `weights`.
    ///
    /// # Errors
    ///
    /// [`LocalError::ModelFileMissing`] naming the first of the four files that
    /// is absent, then [`LocalError::Stage`] when the library refuses.
    pub fn open(weights: &WeightsSet) -> Result<Self> {
        for path in weights.transcriber.all() {
            if !path.is_file() {
                return Err(LocalError::model_file_missing(Stage::Asr, path));
            }
        }
        let rate = PIPELINE_INPUT_RATE;
        let rate_hz = rate_hz(rate, Stage::Asr)?;
        let mut config = OnlineRecognizerConfig {
            model_config: OnlineModelConfig {
                transducer: OnlineTransducerModelConfig {
                    encoder: Some(path_string(&weights.transcriber.encoder, Stage::Asr)?),
                    decoder: Some(path_string(&weights.transcriber.decoder, Stage::Asr)?),
                    joiner: Some(path_string(&weights.transcriber.joiner, Stage::Asr)?),
                },
                tokens: Some(path_string(&weights.transcriber.tokens, Stage::Asr)?),
                num_threads: NUM_THREADS,
                provider: Some(PROVIDER_CPU.to_owned()),
                debug: false,
                ..OnlineModelConfig::default()
            },
            decoding_method: Some(DECODING_METHOD.to_owned()),
            // Endpointing is deliberately **off**: the pipeline's turn detection
            // is the VAD, and a recognizer that ended turns on its own timer
            // would disagree with it — two endpoint detectors on one microphone
            // is one more than the session can act on.
            enable_endpoint: false,
            ..OnlineRecognizerConfig::default()
        };
        config.feat_config.sample_rate = rate_hz;

        let recognizer = OnlineRecognizer::create(&config).ok_or_else(|| {
            LocalError::from(StageError::new(
                Stage::Asr,
                "sherpa-onnx refused the streaming recognizer configuration",
            ))
        })?;
        let stream = recognizer.create_stream();
        Ok(Self {
            recognizer,
            stream,
            published: String::new(),
            rate,
            rate_hz,
        })
    }

    fn decode(&mut self) {
        while self.recognizer.is_ready(&self.stream) {
            self.recognizer.decode(&self.stream);
        }
    }

    fn hypothesis(&self) -> String {
        self.recognizer
            .get_result(&self.stream)
            .map(|result| result.text)
            .unwrap_or_default()
    }
}

impl Transcriber for SherpaTranscriber {
    fn accept(&mut self, samples: &[f32]) -> core::result::Result<TranscriptUpdate, StageError> {
        if samples.is_empty() {
            return Ok(TranscriptUpdate::default());
        }
        self.stream.accept_waveform(self.rate_hz, samples);
        self.decode();
        let hypothesis = self.hypothesis();
        if hypothesis == self.published {
            // Nothing new. Publishing the same hypothesis again would send an
            // identical transcription delta to every client for every 20 ms of
            // silence inside an utterance.
            return Ok(TranscriptUpdate::default());
        }
        // The recognizer commits and revises in one string, so the whole
        // hypothesis is the committed prefix and there is no separate
        // uncommitted tail to put in `stash`.
        self.published.clone_from(&hypothesis);
        Ok(TranscriptUpdate::committed(hypothesis))
    }

    fn finish(&mut self) -> core::result::Result<String, StageError> {
        self.stream.input_finished();
        self.decode();
        let transcript = self.hypothesis();
        self.reset();
        Ok(transcript)
    }

    fn reset(&mut self) {
        // A fresh stream rather than `recognizer.reset`: it clears the decoder
        // state *and* the endpointing state in one step, and the old stream's
        // destructor is the C library's own.
        self.stream = self.recognizer.create_stream();
        self.published.clear();
    }

    fn sample_rate(&self) -> SampleRate {
        self.rate
    }
}

// ── synthesis ───────────────────────────────────────────────────────────────

/// Kokoro (or any `sherpa-onnx` offline TTS) behind [`Speaker`].
///
/// # Voices are speaker **ids** here
///
/// Kokoro ships every voice in one `voices.bin` and selects between them with an
/// integer `sid`, while VIA's configuration surface — and `via-catalog`'s
/// [`DEFAULT_LOCAL_REALTIME_VOICE`](via_catalog::realtime_model::DEFAULT_LOCAL_REALTIME_VOICE) —
/// carries a **name**. [`SherpaSpeaker::with_voice`] is the table between them,
/// and a voice that is neither a known name nor a number falls back to
/// [`DEFAULT_SPEAKER_ID`] rather than failing the utterance: a session that
/// spoke in the wrong voice is recoverable, and one that refused to speak is
/// not.
pub struct SherpaSpeaker {
    engine: Arc<OfflineTts>,
    voices: Vec<(String, i32)>,
    rate: SampleRate,
}

impl core::fmt::Debug for SherpaSpeaker {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SherpaSpeaker")
            .field("rate", &self.rate.hz())
            .field("voices", &self.voices.len())
            .finish_non_exhaustive()
    }
}

impl SherpaSpeaker {
    /// Open the synthesizer from `weights`.
    ///
    /// # Errors
    ///
    /// [`LocalError::ModelFileMissing`] naming the first absent file, then
    /// [`LocalError::Stage`] when the library refuses the configuration.
    pub fn open(weights: &WeightsSet) -> Result<Self> {
        for path in [&weights.speaker.model, &weights.speaker.tokens] {
            if !path.is_file() {
                return Err(LocalError::model_file_missing(Stage::Tts, path));
            }
        }
        let mut kokoro = OfflineTtsKokoroModelConfig {
            model: Some(path_string(&weights.speaker.model, Stage::Tts)?),
            tokens: Some(path_string(&weights.speaker.tokens, Stage::Tts)?),
            ..OfflineTtsKokoroModelConfig::default()
        };
        if let Some(voices) = &weights.speaker.voices {
            if !voices.is_file() {
                return Err(LocalError::model_file_missing(Stage::Tts, voices));
            }
            kokoro.voices = Some(path_string(voices, Stage::Tts)?);
        }
        if let Some(data) = &weights.speaker.data_directory {
            if !data.is_dir() {
                return Err(LocalError::model_file_missing(Stage::Tts, data));
            }
            kokoro.data_dir = Some(path_string(data, Stage::Tts)?);
        }

        let config = OfflineTtsConfig {
            model: OfflineTtsModelConfig {
                kokoro,
                num_threads: NUM_THREADS,
                provider: Some(PROVIDER_CPU.to_owned()),
                debug: false,
                ..OfflineTtsModelConfig::default()
            },
            max_num_sentences: MAX_SENTENCES_PER_CALL,
            silence_scale: SILENCE_SCALE,
            ..OfflineTtsConfig::default()
        };
        let engine = OfflineTts::create(&config).ok_or_else(|| {
            LocalError::from(StageError::new(
                Stage::Tts,
                "sherpa-onnx refused the synthesis configuration",
            ))
        })?;
        // The engine's own rate, not an assumption: a voice pack that
        // synthesizes at something other than 24 kHz would otherwise be played
        // at the wrong speed, and that is the kind of failure nobody reports as
        // a bug because it sounds like a bad model.
        let hz = u32::try_from(engine.sample_rate()).unwrap_or(0);
        let rate = SampleRate::new(hz).map_err(|_| {
            LocalError::from(StageError::new(
                Stage::Tts,
                format!("the synthesizer reported an unusable rate of {hz} Hz"),
            ))
        })?;
        Ok(Self {
            engine: Arc::new(engine),
            voices: Vec::new(),
            rate,
        })
    }

    /// Map a voice name to a speaker id.
    #[must_use]
    pub fn with_voice(mut self, name: impl Into<String>, id: i32) -> Self {
        self.voices.push((name.into(), id));
        self
    }

    /// The speaker id a voice name resolves to.
    ///
    /// A configured table first, then a bare number, then
    /// [`DEFAULT_SPEAKER_ID`].
    #[must_use]
    pub fn speaker_id(&self, voice: &str) -> i32 {
        let voice = voice.trim();
        if let Some((_, id)) = self
            .voices
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(voice))
        {
            return *id;
        }
        voice.parse().unwrap_or(DEFAULT_SPEAKER_ID)
    }
}

#[async_trait::async_trait]
impl Speaker for SherpaSpeaker {
    async fn speak(
        &self,
        text: &str,
        voice: &str,
    ) -> core::result::Result<SpeechStream, StageError> {
        let engine = Arc::clone(&self.engine);
        let sid = self.speaker_id(voice);
        let text = text.to_owned();
        let (sender, receiver) =
            tokio::sync::mpsc::unbounded_channel::<core::result::Result<Vec<i16>, StageError>>();

        // Blocking, on a blocking thread. The callback's `bool` return is the
        // cancellation: when the machine drops the stream, `receiver` is gone,
        // `send` fails, and the C++ side stops generating. Nothing here blocks
        // on drop.
        let failures = sender.clone();
        tokio::task::spawn_blocking(move || {
            let progress = move |samples: &[f32], _progress: f32| {
                sender
                    .send(Ok(via_audio::f32_to_pcm16_slice(samples)))
                    .is_ok()
            };
            if engine
                .generate_with_config(
                    &text,
                    &GenerationConfig {
                        sid,
                        ..GenerationConfig::default()
                    },
                    Some(progress),
                )
                .is_none()
            {
                let _ = failures.send(Err(StageError::new(
                    Stage::Tts,
                    "sherpa-onnx produced no audio for the utterance",
                )));
            }
        });

        Ok(Box::pin(stream::unfold(
            receiver,
            |mut receiver| async move { receiver.recv().await.map(|chunk| (chunk, receiver)) },
        )))
    }

    fn sample_rate(&self) -> SampleRate {
        self.rate
    }
}

/// The three sherpa stages, built from one [`WeightsSet`], plus the caller's
/// reasoning stage.
///
/// The reasoning stage is a parameter rather than a fourth thing built here
/// because the two features are independent: `--features sherpa` alone is a
/// perfectly sensible build for an embedder that already owns a language model,
/// and `--features llama` supplies
/// [`LlamaResponder`](crate::llama::LlamaResponder) for one that does not.
///
/// # Errors
///
/// The first stage that refuses, naming its file or its configuration.
pub fn sherpa_stages(weights: &WeightsSet, responder: Arc<dyn Responder>) -> Result<Stages> {
    Ok(Stages {
        voice_activity: Box::new(SherpaVoiceActivity::open(weights)?),
        transcriber: Box::new(SherpaTranscriber::open(weights)?),
        responder,
        speaker: Arc::new(SherpaSpeaker::open(weights)?),
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn a_missing_graph_is_named_before_the_library_is_asked() {
        // The check that has to come first: the C API answers a null detector
        // with no reason, so "the file is missing" and "the graph is corrupt"
        // would otherwise arrive identically.
        let weights = WeightsSet::resolve("/nonexistent-model-root");
        assert_eq!(
            SherpaVoiceActivity::open(&weights).err(),
            Some(LocalError::model_file_missing(
                Stage::Vad,
                &weights.voice_activity
            ))
        );
        assert_eq!(
            SherpaTranscriber::open(&weights).err(),
            Some(LocalError::model_file_missing(
                Stage::Asr,
                &weights.transcriber.encoder
            ))
        );
        assert_eq!(
            SherpaSpeaker::open(&weights).err(),
            Some(LocalError::model_file_missing(
                Stage::Tts,
                &weights.speaker.model
            ))
        );
    }

    #[test]
    fn the_window_size_is_the_one_silero_was_trained_on() {
        assert_eq!(VAD_WINDOW_SAMPLES, 512);
        assert_eq!(PIPELINE_INPUT_RATE, SampleRate::HZ_16000);
    }
}
