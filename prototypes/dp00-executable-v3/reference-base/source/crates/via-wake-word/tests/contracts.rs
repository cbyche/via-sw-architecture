//! Every value in this crate that `docs/reference/contracts.json` pins.
//!
//! Ten catalogue rows land here. Each test names its row, quotes the part of
//! the `exactValue` it is asserting, and **parses** that part out of the
//! catalogue rather than retyping it, so a contract that is edited or renamed
//! fails here instead of silently agreeing with a stale copy.
//!
//! | Contract `name` | Kind | Status |
//! | --- | --- | --- |
//! | `WAKE_WORD_MODEL_NAME` | `file-path` | asserted |
//! | `WAKE_WORD_MODEL_URL` | `http-route` | asserted |
//! | `WAKE_WORD_MODEL_SHA256` | `default-value` | asserted |
//! | `WAKE_WORD_MODEL_FILES` | `file-path` | asserted |
//! | `generated keywords.txt content` | `file-path` | asserted |
//! | `sherpa KWS detection config` | `default-value` | asserted (the sample rate is read from `via-audio`) |
//! | `pcm16 -> f32 conversion` | `default-value` | asserted end to end through the stream |
//! | `PCM16 to float conversion` | `json-field` | asserted end to end through the stream |
//! | `wakeWord phrase` | `default-value` | **divergent** — recorded in `docs/architecture.md` §16 and `docs/rebrand.md` rows 147-149 |
//! | `wake word / sleep env vars` | `env-var` | partial — `via-core` owns the variables, this asserts the install layout under the directory they name |
//!
//! # The upstream phrase appears here only as a parsed fixture
//!
//! `generated keywords.txt content` carries the upstream product's own wake
//! phrase. `docs/rebrand.md` marks it RENAME and `docs/architecture.md` §16
//! makes VIA's phrase a configuration value that has not been chosen. So it is
//! **read out of the catalogue at run time** and never written into this
//! source file — which is also the honest way to test a format: the round trip
//! below proves [`WakePhrase::keyword_line`] reproduces the catalogued line
//! byte for byte, without anyone typing the line twice.

mod common;

use common::contract_of_kind;
use pretty_assertions::assert_eq;
use via_audio::SampleRate;
use via_core::Config;
use via_i18n::Locale;
use via_wake_word::{
    DEBUG, DetectionConfig, DisabledReason, FEATURE_DIM, KEYWORDS_SCORE, KEYWORDS_THRESHOLD,
    KeywordSet, MAX_ACTIVE_PATHS, MODELING_UNIT_CJKCHAR, ModelArtifact, ModelPaths, NUM_THREADS,
    NUM_TRAILING_BLANKS, PROVIDER_CPU, ScriptedDetector, WAKE_WORD_MODEL_ARCHIVE_SUFFIX,
    WAKE_WORD_MODEL_FILES, WAKE_WORD_MODEL_NAME, WAKE_WORD_MODEL_RELEASE_BASE,
    WAKE_WORD_MODEL_SHA256, WakePhrase, WakeWordDetector, WakeWordSettings, WakeWordStream,
};

// ── the artifact ────────────────────────────────────────────────────────────

/// Contract `WAKE_WORD_MODEL_NAME`:
///
/// > `sherpa-onnx-kws-zipformer-zh-en-3M-2025-12-20`
///
/// > Upstream model id; also the installed directory name under
/// > `wakeWordModelDirectory`.
///
/// Both halves are asserted: the constant, and the *directory name*, which is
/// the half a reader would otherwise have to take on trust.
#[test]
fn model_name_is_the_catalogued_id_and_the_directory_name() {
    let contract = contract_of_kind("file-path", "WAKE_WORD_MODEL_NAME");
    assert_eq!(WAKE_WORD_MODEL_NAME, contract.exact_value);
    assert_eq!(ModelArtifact::ZH_EN_3M.id.as_ref(), contract.exact_value);

    contract.assert_why_mentions("installed directory name");
    let manager = via_wake_word::ModelManager::new(
        "/models",
        std::sync::Arc::new(via_wake_word::ScriptedFetch::new()),
    );
    assert_eq!(
        manager.install_directory(&ModelArtifact::ZH_EN_3M),
        std::path::Path::new("/models").join(&contract.exact_value)
    );
}

/// Contract `WAKE_WORD_MODEL_URL`:
///
/// > `https://github.com/k2-fsa/sherpa-onnx/releases/download/kws-models/sherpa-onnx-kws-zipformer-zh-en-3M-2025-12-20.tar.bz2`
///
/// Upstream builds it by interpolating the id
/// (`` `${RELEASE}/${WAKE_WORD_MODEL_NAME}.tar.bz2` ``) and so does
/// [`ModelArtifact::url`]. Asserting the *built* string rather than a constant
/// is what makes the interpolation part of the contract: a base and an id that
/// were each right but joined wrongly would pass a constant comparison.
#[test]
fn model_url_is_the_base_the_id_and_the_suffix() {
    let contract = contract_of_kind("http-route", "WAKE_WORD_MODEL_URL");
    assert_eq!(ModelArtifact::ZH_EN_3M.url(), contract.exact_value);

    // The three pieces, so a future artifact built from other pieces is
    // assembled the same way.
    let expected = format!(
        "{WAKE_WORD_MODEL_RELEASE_BASE}/{WAKE_WORD_MODEL_NAME}{WAKE_WORD_MODEL_ARCHIVE_SUFFIX}"
    );
    assert_eq!(expected, contract.exact_value);
    assert!(
        contract
            .exact_value
            .starts_with(WAKE_WORD_MODEL_RELEASE_BASE),
        "the release base is no longer the URL's prefix"
    );
    assert_eq!(
        ModelArtifact::ZH_EN_3M.archive_name(),
        format!("{WAKE_WORD_MODEL_NAME}{WAKE_WORD_MODEL_ARCHIVE_SUFFIX}")
    );
}

/// Contract `WAKE_WORD_MODEL_SHA256`:
///
/// > `68447f4fbc67e70eee3a93961f36e81e98f47aef73ce7e7ca00885c6cd3616a6`
///
/// > Supply-chain integrity check; mismatch throws `唤醒词模型校验失败`.
///
/// The rationale carries the user-facing message, so it is a contract too. It
/// is compared against `via-i18n`'s `zh` column, which is where every VIA
/// sentence lives.
#[test]
fn model_digest_is_pinned_and_its_failure_message_is_catalogued() {
    let contract = contract_of_kind("default-value", "WAKE_WORD_MODEL_SHA256");
    assert_eq!(WAKE_WORD_MODEL_SHA256, contract.exact_value);
    assert_eq!(
        ModelArtifact::ZH_EN_3M.sha256.as_ref(),
        contract.exact_value
    );

    // A digest is 32 bytes of lowercase hex, and nothing else.
    assert_eq!(contract.exact_value.len(), 64);
    assert!(
        contract
            .exact_value
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "the pinned digest is not lowercase hex"
    );

    let message = contract
        .why_quoted()
        .into_iter()
        .next()
        .expect("the rationale quotes the failure message");
    let error = via_wake_word::WakeWordError::Checksum {
        expected: contract.exact_value.clone(),
        actual: "0".repeat(64),
    };
    assert_eq!(error.localized(Locale::Zh), message);
}

/// Contract `WAKE_WORD_MODEL_FILES`:
///
/// > `{encoder:'…int8.onnx', decoder:'….onnx', joiner:'…int8.onnx',
/// > tokens:'tokens.txt', keywords:'keywords.txt'}`
///
/// > only encoder/decoder/joiner/tokens are extracted, keywords.txt is
/// > generated.
///
/// The second sentence is the one worth a test: the split between the four
/// archive members and the one generated file is what makes a changed phrase
/// take effect against an already-installed model.
#[test]
fn model_files_are_the_catalogued_names_and_only_four_come_from_the_archive() {
    let contract = contract_of_kind("file-path", "WAKE_WORD_MODEL_FILES");
    let files = &WAKE_WORD_MODEL_FILES;

    assert_eq!(files.encoder.as_ref(), contract.quoted_after("encoder:"));
    assert_eq!(files.decoder.as_ref(), contract.quoted_after("decoder:"));
    assert_eq!(files.joiner.as_ref(), contract.quoted_after("joiner:"));
    assert_eq!(files.tokens.as_ref(), contract.quoted_after("tokens:"));
    assert_eq!(files.keywords.as_ref(), contract.quoted_after("keywords:"));

    contract.assert_why_mentions("only encoder/decoder/joiner/tokens are extracted");
    contract.assert_why_mentions("keywords.txt is generated");
    assert_eq!(
        files.archived().to_vec(),
        vec![
            files.encoder.as_ref(),
            files.decoder.as_ref(),
            files.joiner.as_ref(),
            files.tokens.as_ref(),
        ]
    );
    assert!(!files.is_archived(files.keywords.as_ref()));
    assert_eq!(files.required().len(), 5);

    // The two int8 members and the one that is not: a real property of this
    // artifact that a typo in any of the three names would break.
    assert!(files.encoder.ends_with(".int8.onnx"));
    assert!(files.joiner.ends_with(".int8.onnx"));
    assert!(files.decoder.ends_with(".onnx") && !files.decoder.ends_with(".int8.onnx"));
}

// ── the keyword file ────────────────────────────────────────────────────────

/// Contract `generated keywords.txt content`:
///
/// > `n ǐ h ǎo q iān w èn @<phrase>\n`
///
/// > Pinyin-token encoding of the wake phrase consumed by sherpa-onnx (tokens
/// > before `@`, display label after). Written with mode 0o600.
///
/// The phrase in that line is the upstream product's, not VIA's, so it is read
/// from the catalogue and split on the separator rather than typed here. The
/// round trip proves [`WakePhrase::keyword_line`] rebuilds the catalogued line
/// exactly — trailing newline, single space before `@`, and all.
#[test]
fn keyword_line_reproduces_the_catalogued_format_byte_for_byte() {
    let contract = contract_of_kind("file-path", "generated keywords.txt content");
    let line = contract.unescaped();

    assert!(line.ends_with('\n'), "the catalogued line has no newline");
    let (tokens, text) = line
        .trim_end_matches('\n')
        .split_once(" @")
        .expect("the catalogued line separates tokens from the display text with ` @`");

    let phrase = WakePhrase::new(text, tokens).expect("the upstream fixture is a valid phrase");
    assert_eq!(phrase.keyword_line(), line);
    assert_eq!(phrase.tokens(), tokens);
    assert_eq!(phrase.text(), text);

    contract.assert_why_mentions("tokens before '@'");
    contract.assert_why_mentions("0o600");
    assert_eq!(via_store::FILE_MODE, 0o600);
}

/// Contract `generated keywords.txt content`, the set half.
///
/// One phrase is one line. The upstream fixture is used again — parsed, not
/// typed — because a keyword file with a CJK display column is the case where
/// a byte/char confusion in `keywordsBufSize` would actually bite.
#[test]
fn keywords_buf_size_is_bytes_not_characters() {
    let contract = contract_of_kind("file-path", "generated keywords.txt content");
    let line = contract.unescaped();
    let (tokens, text) = line
        .trim_end_matches('\n')
        .split_once(" @")
        .expect("the catalogued line is well formed");

    let keywords = KeywordSet::new().with(
        Locale::Zh,
        via_wake_word::Keyword::zh_en(text, tokens).expect("a valid fixture"),
    );
    let file = keywords.keywords_file(WAKE_WORD_MODEL_NAME);
    assert_eq!(file, line);

    let config = DetectionConfig::new(
        ModelPaths::resolve(std::path::Path::new("/models"), &ModelArtifact::ZH_EN_3M),
        file.clone(),
    );
    assert_eq!(config.keywords_buf_size(), file.len());
    assert!(
        config.keywords_buf_size() > file.chars().count(),
        "the fixture's display column is multi-byte, so a character count would be smaller — \
         that is the confusion this contract exists to prevent"
    );
}

// ── the detection configuration ─────────────────────────────────────────────

/// Contract `sherpa KWS detection config`:
///
/// > `featConfig {samplingRate:16000, featureDim:80}; modelConfig
/// > {transducer{encoder,decoder,joiner}, tokens, numThreads:1,
/// > provider:'cpu', debug:0, modelingUnit:'cjkchar'}; maxActivePaths:4;
/// > numTrailingBlanks:1; keywordsScore:1.0; keywordsThreshold:0.25;
/// > keywords:''; keywordsBuf:<file text>; keywordsBufSize:<byteLength>`
///
/// Nine fields belong to this crate. The tenth — `samplingRate` — belongs to
/// `via-audio`, which already asserts it against this same row;
/// `docs/deviations/phase-0.md` records the split. So the assertion here is
/// that [`DetectionConfig::sample_rate`] **is** `SampleRate::HZ_16000`, which
/// is a different and stronger claim than "it is 16000": it cannot drift from
/// the rate the resampler converts to.
#[test]
fn detection_config_is_the_catalogued_configuration() {
    let contract = contract_of_kind("default-value", "sherpa KWS detection config");
    let config = DetectionConfig::new(
        ModelPaths::resolve(std::path::Path::new("/models"), &ModelArtifact::ZH_EN_3M),
        "tokens @phrase\n",
    );

    // The half `via-audio` owns — read, never retyped.
    assert_eq!(config.sample_rate, SampleRate::HZ_16000);
    assert_eq!(
        u64::from(config.sample_rate.hz()),
        contract.number_after("samplingRate:")
    );

    // The nine this crate owns.
    assert_eq!(
        u64::from(config.feature_dim.unsigned_abs()),
        contract.number_after("featureDim:")
    );
    assert_eq!(
        u64::from(config.num_threads.unsigned_abs()),
        contract.number_after("numThreads:")
    );
    assert_eq!(config.provider, contract.quoted_after("provider:"));
    assert_eq!(u64::from(config.debug), contract.number_after("debug:"));
    assert_eq!(config.modeling_unit, contract.quoted_after("modelingUnit:"));
    assert_eq!(
        u64::from(config.max_active_paths.unsigned_abs()),
        contract.number_after("maxActivePaths:")
    );
    assert_eq!(
        u64::from(config.num_trailing_blanks.unsigned_abs()),
        contract.number_after("numTrailingBlanks:")
    );
    assert_eq!(
        f64::from(config.keywords_score),
        contract.decimal_after("keywordsScore:")
    );
    assert_eq!(
        f64::from(config.keywords_threshold),
        contract.decimal_after("keywordsThreshold:")
    );

    // The same nine as free constants, so a caller assembling a configuration
    // by hand cannot pick a different set.
    assert_eq!(config.feature_dim, FEATURE_DIM);
    assert_eq!(config.num_threads, NUM_THREADS);
    assert_eq!(config.provider, PROVIDER_CPU);
    assert_eq!(config.debug, DEBUG);
    assert_eq!(config.modeling_unit, MODELING_UNIT_CJKCHAR);
    assert_eq!(config.max_active_paths, MAX_ACTIVE_PATHS);
    assert_eq!(config.num_trailing_blanks, NUM_TRAILING_BLANKS);
    assert_eq!(config.keywords_score, KEYWORDS_SCORE);
    assert_eq!(config.keywords_threshold, KEYWORDS_THRESHOLD);

    // `keywords: ''` — keywords come from the buffer, not a path.
    assert_eq!(contract.quoted_after("keywords:"), "");
    assert_eq!(config.keywords_file(), None);
    assert_eq!(config.keywords_buf, "tokens @phrase\n");
}

/// Contract `sherpa KWS detection config`, the rationale.
///
/// > Detection sensitivity is a product-visible tradeoff (0.25 documented as
/// > the office-environment compromise…).
///
/// Pinned separately because it is the one number in the row that a reader
/// might reasonably think is a tuning knob.
#[test]
fn the_detection_threshold_is_a_product_decision() {
    let contract = contract_of_kind("default-value", "sherpa KWS detection config");
    contract.assert_why_mentions("product-visible tradeoff");
    contract.assert_why_mentions("office-environment compromise");
    assert_eq!(
        f64::from(KEYWORDS_THRESHOLD),
        contract.decimal_after("keywordsThreshold:")
    );
}

// ── PCM, end to end through the stream ──────────────────────────────────────

/// Contracts `pcm16 -> f32 conversion` and `PCM16 to float conversion`:
///
/// > `samples[i] = readInt16LE(i*2) / 32768`
/// > `divisor is 32768 (not 32767): -32768→-1, -16384→-0.5, 0→0,
/// > 32767→32767/32768`
///
/// `via-audio` owns the conversion and asserts it directly. What is asserted
/// here is the thing only this crate can prove: that the bytes a host hands
/// [`WakeWordStream::accept_pcm16le`] reach the detector as exactly those
/// values, with no scaling, offset or clamp introduced on the way through the
/// frame buffer.
#[test]
fn host_pcm16_reaches_the_detector_divided_by_32768() {
    let contract = contract_of_kind("json-field", "PCM16 to float conversion");
    let divisor = contract.number_after("divisor is ") as f32;
    assert_eq!(divisor, 32_768.0);
    assert_ne!(divisor, 32_767.0);

    // The upstream test vector, byte for byte:
    // `writeInt16LE(-32768, 0); …(-16384, 2); …(0, 4); …(32767, 6)`.
    let pcm: [u8; 8] = [0x00, 0x80, 0x00, 0xC0, 0x00, 0x00, 0xFF, 0x7F];
    let mut stream = WakeWordStream::new(
        ScriptedDetector::silent(),
        SampleRate::HZ_16000,
        KeywordSet::new(),
    )
    .expect("16 kHz in, 16 kHz out needs no resampler");
    assert!(!stream.is_resampling());
    assert_eq!(stream.accept_pcm16le(&pcm).expect("accepted"), None);

    let expected: Vec<f32> = [i16::MIN, -16_384, 0, i16::MAX]
        .iter()
        .map(|raw| f32::from(*raw) / divisor)
        .collect();
    assert_eq!(stream.detector().last_chunk(), expected.as_slice());
    assert_eq!(stream.detector().last_chunk()[0], -1.0);
    assert_eq!(stream.detector().last_chunk()[1], -0.5);
}

/// Contract `pcm16 -> f32 conversion`, the length half:
///
/// > `sample count = floor(bytes/2)` … `an odd trailing byte is dropped`
///
/// **This crate deliberately does not drop it.** Upstream decodes one
/// self-contained base64 payload per call, where the odd byte is genuinely
/// junk; a streaming front end is handed arbitrary socket-sized chunks, where
/// dropping it shifts every following sample by one byte. `via-audio`'s
/// `FrameBuffer` carries it, and this asserts the carry rather than the drop —
/// with the drop asserted alongside, so the deviation cannot be mistaken for
/// an accident.
#[test]
fn a_torn_pcm_frame_is_carried_into_the_next_chunk_not_dropped() {
    let contract = contract_of_kind("default-value", "pcm16 -> f32 conversion");
    contract.assert_mentions("floor(bytes/2)");
    let divisor = f32::from(i16::MAX) + 1.0;

    // The stateless decoder upstream uses: three bytes, one sample.
    assert_eq!(via_audio::pcm16le_to_f32(&[1, 0, 2]).len(), 1);

    let mut stream = WakeWordStream::new(
        ScriptedDetector::silent(),
        SampleRate::HZ_16000,
        KeywordSet::new(),
    )
    .expect("no resampler");
    stream.accept_pcm16le(&[1, 0, 2]).expect("accepted");
    assert_eq!(stream.detector().last_chunk(), &[1.0 / divisor]);

    // The carried `2` joins the next byte instead of vanishing.
    stream.accept_pcm16le(&[3]).expect("accepted");
    assert_eq!(
        stream.detector().last_chunk(),
        &[f32::from(i16::from_le_bytes([2, 3])) / divisor]
    );
    assert_eq!(stream.detector().samples(), 2);
}

// ── the phrase, which VIA does not inherit ──────────────────────────────────

/// Contract `wakeWord phrase`:
///
/// > `<upstream phrase>` (hard-coded, no env override)
///
/// **Divergent**, recorded in `docs/architecture.md` §16 (*"The phrase is a
/// configuration value, never a literal"*) and `docs/rebrand.md` rows 147-149,
/// which mark the upstream phrase RENAME and note that changing it requires a
/// regenerated keyword model.
///
/// The test asserts both sides, so the divergence cannot drift into an
/// accident: upstream hard-codes with no override, VIA reads `VIA_WAKE_WORD`
/// and defaults to nothing, and the upstream phrase is not what any VIA locale
/// resolves to.
#[test]
fn via_does_not_inherit_the_upstream_wake_phrase() {
    let contract = contract_of_kind("default-value", "wakeWord phrase");
    contract.assert_mentions("hard-coded, no env override");
    let upstream_phrase = contract
        .exact_value
        .split(" (")
        .next()
        .expect("the catalogued value starts with the phrase")
        .to_owned();
    assert!(!upstream_phrase.is_empty());

    // VIA's own default is nothing at all.
    let config = Config::default();
    assert_eq!(config.wake_word, "");
    assert!(!config.wake_word_enabled);

    let settings = WakeWordSettings::from_config(&config, KeywordSet::new());
    for locale in Locale::ALL {
        assert_eq!(settings.phrase(*locale), None);
        assert_ne!(settings.phrase(*locale), Some(upstream_phrase.as_str()));
        assert_eq!(settings.asleep_message(*locale), None);
    }

    // Even switched on, an unconfigured phrase is a clean disabled — not a
    // fallback to somebody else's.
    let enabled = Config {
        wake_word_enabled: true,
        ..Config::default()
    };
    assert_eq!(
        WakeWordSettings::from_config(&enabled, KeywordSet::new()).resolve(Locale::En),
        via_wake_word::Resolution::Disabled(DisabledReason::NoPhraseConfigured)
    );
}

// ── where the model lives ───────────────────────────────────────────────────

/// Contract `wake word / sleep env vars`, the model-directory half:
///
/// > `QWEN_AUDIO_WAKE_WORD_MODEL_DIR (default <configDir>/models/wake-word)`
///
/// Partial: `via-core` owns the variable, its VIA name and its default, and
/// asserts them. What this asserts is the layout *inside* that directory — one
/// subdirectory per artifact id — which is the half a model manager owns.
#[test]
fn models_install_one_directory_per_artifact_under_the_configured_root() {
    let contract = contract_of_kind("env-var", "wake word / sleep env vars");
    contract.assert_mentions("models/wake-word");
    assert_eq!(
        via_core::paths::WAKE_WORD_MODEL_DIRECTORY,
        "models/wake-word"
    );

    let root = std::path::Path::new("/home/example/.config/via/models/wake-word");
    let manager = via_wake_word::ModelManager::new(
        root,
        std::sync::Arc::new(via_wake_word::ScriptedFetch::new()),
    );
    assert_eq!(
        manager.install_directory(&ModelArtifact::ZH_EN_3M),
        root.join(WAKE_WORD_MODEL_NAME)
    );

    // Two artifacts never share a directory, which is what lets a second
    // locale's model install beside the first instead of over it.
    let other = ModelArtifact {
        id: std::borrow::Cow::Borrowed("some-other-kws-model"),
        ..ModelArtifact::ZH_EN_3M
    };
    assert_ne!(
        manager.install_directory(&ModelArtifact::ZH_EN_3M),
        manager.install_directory(&other)
    );
}

/// The detector contract's default rate, stated once so an implementation that
/// forgets to override it lands on the catalogued value rather than on zero.
#[test]
fn the_detector_default_rate_is_the_feature_extractor_rate() {
    assert_eq!(
        ScriptedDetector::silent().sample_rate(),
        SampleRate::HZ_16000
    );
    assert_eq!(
        DetectionConfig::new(
            ModelPaths::resolve(std::path::Path::new("/m"), &ModelArtifact::ZH_EN_3M),
            "t @p\n"
        )
        .sample_rate,
        SampleRate::HZ_16000
    );
}
