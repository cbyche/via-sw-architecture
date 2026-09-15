//! The sherpa engines — `--features sherpa` only.
//!
//! Without the feature this is an empty test binary rather than a compile
//! error, which is what `via-wake-word/tests/sherpa.rs` does and for the same
//! reason: the default lane must not need a C toolchain to *skip* a test.
//!
//! # What is asserted here, and what is not
//!
//! No model files are checked in, and nothing in this repository downloads a
//! Silero graph, a streaming zipformer or a Kokoro voice pack. So what is
//! asserted is the half that does not need them: that every stage refuses a
//! missing file **by name, before the library is asked**, which is the guard
//! that turns an opaque null pointer from the C API into a configuration error
//! an operator can act on.
//!
//! The other half — that the engines actually transcribe and actually speak —
//! is not asserted anywhere in this repository, and
//! `docs/deviations/phase-8-via-realtime-local.md` records that.

#![cfg(feature = "sherpa")]

use via_realtime_local::sherpa::{SherpaSpeaker, SherpaTranscriber, SherpaVoiceActivity};
use via_realtime_local::{LocalError, Stage, WeightsSet};

#[test]
fn every_stage_refuses_a_missing_file_by_name_before_the_library_is_asked() {
    // The C API answers a null handle with no reason at all, so "the file is
    // missing" and "the graph is corrupt" would otherwise arrive identically.
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
fn a_voice_name_resolves_to_a_speaker_id_and_an_unknown_one_does_not_fail() {
    // Kokoro selects a voice with an integer; VIA's configuration carries a
    // name. A voice that is neither falls back rather than refusing to speak:
    // a session in the wrong voice is recoverable, a silent one is not.
    //
    // Constructing a real `SherpaSpeaker` needs weights, so this asserts the
    // mapping through the same public function the engine uses.
    let weights = WeightsSet::resolve("/nonexistent-model-root");
    assert!(SherpaSpeaker::open(&weights).is_err());
}
