//! The engine — compiled only with `--features sherpa`.
//!
//! Without the feature this file is an empty test binary, which is the point:
//! the default `cargo test -p via-wake-word` stays green on a machine with no
//! native toolchain and no model on disk.
//!
//! # What can be tested without 33 MB of weights, and what cannot
//!
//! Everything below runs with **no model files at all**, because every one of
//! them is about the guards in front of the engine — the cases where
//! `sherpa-onnx` would otherwise answer a null pointer and no reason.
//!
//! A real detection needs the real model. [`live_install_opens_the_engine`] is
//! that test: it is `#[ignore]`d and additionally gated on
//! `VIA_WAKE_WORD_LIVE_MODEL=1`, so it runs only when somebody asks for it by
//! name. It downloads the catalogued artifact from the k2-fsa release page,
//! verifies the pinned digest against the real bytes, installs it, and opens
//! the engine. That makes it the one test that can prove
//! `WAKE_WORD_MODEL_SHA256` and `WAKE_WORD_MODEL_FILES` describe the artifact
//! that actually exists today — and the reason it is not in CI is that CI
//! should not depend on a third party's release page staying up.
//!
//! ```text
//! cargo test -p via-wake-word --features sherpa,http -- --ignored --nocapture
//! ```

#![cfg(feature = "sherpa")]

use std::path::Path;

use pretty_assertions::assert_eq;
use tempfile::TempDir;
use via_audio::SampleRate;
use via_wake_word::{
    DetectionConfig, KeywordError, ModelArtifact, ModelPaths, SherpaDetector, WakeWordDetector,
    WakeWordError,
};

const KEYWORDS: &str = "w a k e u p @wake_up\n";

fn config_in(directory: &Path, keywords: &str) -> DetectionConfig {
    DetectionConfig::new(
        ModelPaths::resolve(directory, &ModelArtifact::ZH_EN_3M),
        keywords,
    )
}

#[test]
fn an_engine_with_no_keywords_is_refused() {
    // A spotter with an empty keyword list loads, runs, consumes CPU forever
    // and never matches anything. That is the failure this guard exists for.
    let directory = TempDir::new().expect("a temp directory");
    for blank in ["", "   ", "\n"] {
        let error = SherpaDetector::open(&config_in(directory.path(), blank))
            .expect_err("no keywords is not a usable engine");
        assert!(
            matches!(
                error,
                WakeWordError::Keyword(KeywordError::Empty { field: "keywords" })
            ),
            "expected an empty-keywords error, got {error:?}"
        );
    }
}

#[test]
fn a_missing_model_names_the_files_rather_than_the_library() {
    let directory = TempDir::new().expect("a temp directory");
    let error = SherpaDetector::open(&config_in(directory.path(), KEYWORDS))
        .expect_err("there is no model in an empty directory");

    match &error {
        WakeWordError::Incomplete { missing } => {
            assert_eq!(missing.len(), 4, "all four model files are absent");
            for name in ModelArtifact::ZH_EN_3M.files.archived() {
                assert!(
                    missing.iter().any(|path| path.ends_with(name)),
                    "{name} is not named in {missing:?}"
                );
            }
        }
        other => panic!("expected an incomplete install, got {other:?}"),
    }
}

#[test]
fn a_partially_present_model_names_only_what_is_missing() {
    let directory = TempDir::new().expect("a temp directory");
    let files = &ModelArtifact::ZH_EN_3M.files;
    // Everything but the joiner. The bytes are nonsense; the guard runs before
    // the library is asked to read them.
    for name in [
        files.encoder.as_ref(),
        files.decoder.as_ref(),
        files.tokens.as_ref(),
    ] {
        std::fs::write(directory.path().join(name), b"not a real model").expect("written");
    }

    let error = SherpaDetector::open(&config_in(directory.path(), KEYWORDS))
        .expect_err("the joiner is missing");
    match &error {
        WakeWordError::Incomplete { missing } => {
            assert_eq!(missing.len(), 1);
            assert!(missing[0].ends_with(files.joiner.as_ref()));
        }
        other => panic!("expected an incomplete install, got {other:?}"),
    }
}

#[test]
fn a_directory_wearing_a_model_files_name_is_not_a_model_file() {
    let directory = TempDir::new().expect("a temp directory");
    let files = &ModelArtifact::ZH_EN_3M.files;
    for name in files.archived() {
        std::fs::create_dir(directory.path().join(name)).expect("created");
    }
    let error = SherpaDetector::open(&config_in(directory.path(), KEYWORDS))
        .expect_err("a directory is not a model file");
    assert!(
        matches!(error, WakeWordError::Incomplete { .. }),
        "expected an incomplete install, got {error:?}"
    );
}

#[test]
fn the_configuration_the_engine_is_opened_with_is_the_catalogued_one() {
    // The engine cannot be opened here, but the configuration it would be
    // opened with is the same value `tests/contracts.rs` asserts field by
    // field. This is the seam between the two.
    let directory = TempDir::new().expect("a temp directory");
    let config = config_in(directory.path(), KEYWORDS);
    assert_eq!(config.sample_rate, SampleRate::HZ_16000);
    assert_eq!(config.keywords_buf_size(), KEYWORDS.len());
    assert_eq!(config.keywords_file(), None);
    assert_eq!(
        config.model.encoder,
        directory
            .path()
            .join(ModelArtifact::ZH_EN_3M.files.encoder.as_ref())
    );
}

/// The one test that touches the network, and only when asked twice.
///
/// `#[ignore]` keeps it out of `cargo test`; the environment variable keeps it
/// out of `cargo test -- --ignored` unless it is what somebody meant.
///
/// It is the only test that can prove four things at once, and each of them is
/// a claim this crate makes about an artifact it does not control:
///
/// 1. `WAKE_WORD_MODEL_URL` still resolves, and `WAKE_WORD_MODEL_SHA256` still
///    matches the bytes it serves — the download is verified inside `ensure`,
///    so reaching the next line at all is the assertion.
/// 2. `WAKE_WORD_MODEL_FILES` names four members that are really in the archive
///    — the install would be `Incomplete` otherwise.
/// 3. The catalogued detection configuration really opens a spotter.
/// 4. A wake word said out loud is really detected, through the whole VIA
///    front end — `WakeWordStream` into `SherpaDetector` — rather than by
///    handing samples straight to the engine.
///
/// The audio and the keyword both come from the model's own `test_wavs/`: the
/// vendor's `LIGHT_UP` keyword, and the recording whose transcript contains
/// *"…WOULD LIGHT UP HERE AND THERE…"*. Neither is a VIA phrase — VIA has not
/// chosen one — and using the vendor's own pair is what makes a positive
/// detection meaningful rather than a threshold accident.
#[cfg(feature = "http")]
#[tokio::test]
#[ignore = "downloads ~33 MB from the k2-fsa release page; set VIA_WAKE_WORD_LIVE_MODEL=1"]
async fn live_install_detects_the_vendors_own_keyword() {
    use std::io::Read;
    use std::sync::Arc;

    use via_wake_word::{
        HttpModelFetch, KeywordSet, ModelFetch, ModelManager, ScriptedFetch, WakeWordStream,
    };

    if std::env::var("VIA_WAKE_WORD_LIVE_MODEL").as_deref() != Ok("1") {
        eprintln!("skipped: set VIA_WAKE_WORD_LIVE_MODEL=1 to run the live install");
        return;
    }

    let artifact = ModelArtifact::ZH_EN_3M;

    // Fetch once; the installer is then fed the same bytes so the test does not
    // pull 33 MB twice.
    let archive = HttpModelFetch::new()
        .fetch(&artifact.url())
        .await
        .expect("the release page answers")
        .body
        .expect("the answer carries the archive");

    let root = TempDir::new().expect("a temp root");
    let manager = ModelManager::new(
        root.path(),
        Arc::new(ScriptedFetch::serving(archive.clone())),
    );

    // The vendor's own keyword line, from `test_wavs/keywords.txt`. The label
    // is `LIGHT_UP` — one word — which is the convention this crate's
    // `WakePhrase::label` reproduces.
    let keywords = "L AY1 T AH1 P @LIGHT_UP\n";

    // Claims 1 and 2: the digest is verified and the four members are found
    // inside `ensure`, so an error here is one of them failing.
    let install = manager
        .ensure(&artifact, keywords)
        .await
        .expect("the catalogued artifact installs");
    assert!(manager.is_installed(&artifact));

    // Claim 3.
    let detector = SherpaDetector::open(&install.detection_config(keywords))
        .expect("the installed model opens");
    assert_eq!(detector.sample_rate(), SampleRate::HZ_16000);

    // Claim 4, through the whole front end rather than straight into the
    // engine. The fixture's own format decides whether a resampler and a
    // downmix sit in the path; both are printed so the run says what it
    // exercised.
    let wav = read_member(&archive, "en_0.wav");
    let audio = via_audio::read_wav(std::io::Cursor::new(wav)).expect("the fixture is a WAV");
    let mut stream =
        WakeWordStream::with_channels(detector, audio.rate(), audio.channels(), KeywordSet::new())
            .expect("the front end builds for the fixture's format");

    eprintln!(
        "fixture format: {} Hz, {} channel(s), resampling: {}",
        stream.capture_rate().hz(),
        stream.channels().get(),
        stream.is_resampling()
    );

    let mut detection = None;
    // 20 ms at a time, the way a host hands audio over.
    let block = audio.rate().capture_block_frames() * audio.channels().get();
    for chunk in audio.samples().chunks(block) {
        if let Some(event) = stream.accept_samples(chunk).expect("accepted") {
            detection = Some(event);
            break;
        }
    }
    let detection = detection.expect("the vendor's keyword is in the vendor's recording");
    assert_eq!(detection.keyword, "LIGHT_UP");
    eprintln!(
        "live detection ok: {} at {:.2}s",
        detection.keyword, detection.start_time_seconds
    );

    // Claim 4 again, this time with the front end doing real work: the same
    // recording upsampled to what a desktop capture device actually produces.
    // The detection has to survive a 3:1 band-limited resample and a stereo
    // downmix, which no fixture-driven test can prove against a real model.
    let upsampled = via_audio::resample_mono_pcm16(
        audio.samples(),
        audio.rate(),
        via_audio::SampleRate::HZ_48000,
    )
    .expect("16 kHz upsamples to 48 kHz");
    let stereo = via_audio::upmix_from_mono(&upsampled, via_audio::ChannelCount::STEREO);

    let mut host = WakeWordStream::with_channels(
        SherpaDetector::open(&install.detection_config(keywords)).expect("a second engine"),
        via_audio::SampleRate::HZ_48000,
        via_audio::ChannelCount::STEREO,
        KeywordSet::new(),
    )
    .expect("the front end builds for 48 kHz stereo");
    assert!(host.is_resampling());

    let block = via_audio::SampleRate::HZ_48000.capture_block_frames()
        * via_audio::ChannelCount::STEREO.get();
    let mut resampled_detection = None;
    for chunk in stereo.chunks(block) {
        if let Some(event) = host.accept_samples(chunk).expect("accepted") {
            resampled_detection = Some(event);
            break;
        }
    }
    assert_eq!(
        resampled_detection
            .expect("the keyword survives 48 kHz stereo capture")
            .keyword,
        "LIGHT_UP"
    );
    eprintln!("live detection ok through 48 kHz stereo capture");

    // And the guard that keeps a bad token line from ending the process: the
    // library would log and exit here, so reaching the assertion is the point.
    let mut config = install.detection_config("L AY1 NOT_A_TOKEN @X\n");
    config.keywords_buf = "L AY1 NOT_A_TOKEN @X\n".to_owned();
    let error = SherpaDetector::open(&config).expect_err("NOT_A_TOKEN is not in tokens.txt");
    assert!(
        matches!(error, WakeWordError::UnknownTokens { .. }),
        "expected unknown tokens, got {error:?}"
    );

    /// Pull one member out of the release archive by basename.
    fn read_member(archive: &[u8], name: &str) -> Vec<u8> {
        let mut tar = tar::Archive::new(bzip2::read::BzDecoder::new(archive));
        for entry in tar.entries().expect("the archive opens") {
            let mut entry = entry.expect("an entry");
            let path = entry.path().expect("a path").into_owned();
            if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes).expect("the member reads");
                return bytes;
            }
        }
        panic!("{name} is not in the release archive")
    }
}
