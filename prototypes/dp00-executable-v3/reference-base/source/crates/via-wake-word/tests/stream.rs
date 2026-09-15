//! The PCM front end: what the host captures, and what the detector receives.
//!
//! Upstream captures at exactly the extractor's rate and passes the samples
//! straight through, so none of this exists there. These tests are about the
//! difference: a host at 48 kHz stereo has to reach a 16 kHz mono feature
//! extractor with the right samples, the right count, and no chunk boundary
//! artefacts.

use pretty_assertions::assert_eq;
use rstest::rstest;
use via_audio::{ChannelCount, SampleRate};
use via_i18n::Locale;
use via_wake_word::{
    Detection, Keyword, KeywordSet, ScriptStep, ScriptedDetector, WakeWordDetector, WakeWordStream,
};

/// Two configured keywords. Note the labels: the engine reports `hey_via`,
/// never `hey via`, because the keyword file's display column cannot contain a
/// space (see [`via_wake_word::keyword`]).
fn keywords() -> KeywordSet {
    KeywordSet::new()
        .with(
            Locale::En,
            Keyword::zh_en("hey via", "HH EY1 V IY1 AH0").expect("valid"),
        )
        .with(
            Locale::Ko,
            Keyword::zh_en("안녕 비아", "a n n y e o n g b i a").expect("valid"),
        )
}

/// `frames` frames of interleaved silence at `channels`, as PCM16LE bytes.
fn silence(frames: usize, channels: ChannelCount) -> Vec<u8> {
    vec![0u8; frames * channels.get() * 2]
}

// ── rate handling ───────────────────────────────────────────────────────────

#[test]
fn a_host_at_the_extractor_rate_needs_no_resampler() {
    let stream = WakeWordStream::new(
        ScriptedDetector::silent(),
        SampleRate::HZ_16000,
        KeywordSet::new(),
    )
    .expect("built");
    assert!(!stream.is_resampling());
    assert_eq!(stream.capture_rate(), SampleRate::HZ_16000);
    assert_eq!(stream.detector_rate(), SampleRate::HZ_16000);
    assert_eq!(stream.channels(), ChannelCount::MONO);
}

#[rstest]
#[case::forty_eight(SampleRate::HZ_48000)]
#[case::twenty_four(SampleRate::HZ_24000)]
fn a_host_at_another_rate_is_resampled_to_the_extractor_rate(#[case] capture: SampleRate) {
    let mut stream =
        WakeWordStream::new(ScriptedDetector::silent(), capture, KeywordSet::new()).expect("built");
    assert!(stream.is_resampling());
    assert_eq!(stream.detector_rate(), SampleRate::HZ_16000);

    // One second of capture. The resampler compensates its own group delay and
    // emits whole chunks, so the delivered count lands close to — never past —
    // one second at the extractor rate.
    let frames = capture.hz() as usize;
    stream
        .accept_pcm16le(&silence(frames, ChannelCount::MONO))
        .expect("accepted");

    let expected = u64::from(SampleRate::HZ_16000.hz());
    let delivered = stream.delivered_samples();
    assert!(
        delivered <= expected,
        "{capture:?}: delivered {delivered} samples for one second at 16 kHz"
    );
    assert!(
        delivered * 10 >= expected * 9,
        "{capture:?}: delivered only {delivered} of about {expected} samples"
    );
}

#[test]
fn the_resampler_is_built_from_the_detectors_rate_not_from_an_assumption() {
    // A detector that wants 24 kHz gets 24 kHz. Nothing in the stream hard-codes
    // the extractor rate; it asks.
    let detector = ScriptedDetector::silent().at_rate(SampleRate::HZ_24000);
    let stream =
        WakeWordStream::new(detector, SampleRate::HZ_48000, KeywordSet::new()).expect("built");
    assert_eq!(stream.detector_rate(), SampleRate::HZ_24000);
    assert!(stream.is_resampling());

    // And a detector that wants what the host produces gets no resampler at all.
    let detector = ScriptedDetector::silent().at_rate(SampleRate::HZ_48000);
    let stream =
        WakeWordStream::new(detector, SampleRate::HZ_48000, KeywordSet::new()).expect("built");
    assert!(!stream.is_resampling());
}

#[test]
fn a_resampler_that_has_not_filled_a_chunk_hands_the_detector_nothing() {
    // The failure mode this prevents: calling an engine with a zero-length
    // buffer every 20 ms until the FFT has enough input.
    let mut stream = WakeWordStream::new(
        ScriptedDetector::detecting("hey_via"),
        SampleRate::HZ_48000,
        keywords(),
    )
    .expect("built");

    // Half of the resampler's chunk: it has to buffer and produce nothing.
    let half = via_audio::Resampler::new(
        SampleRate::HZ_48000,
        SampleRate::HZ_16000,
        ChannelCount::MONO,
    )
    .expect("built")
    .chunk_frames()
        / 2;
    assert_eq!(
        stream
            .accept_pcm16le(&silence(half, ChannelCount::MONO))
            .expect("accepted"),
        None
    );
    assert_eq!(stream.detector().empty_chunks(), 0);
    assert_eq!(stream.detector().chunks(), 0, "the engine was not called");
    assert_eq!(
        stream.detector().remaining(),
        1,
        "the script was not consumed"
    );

    // …and the test is not vacuous: completing the chunk does reach the engine.
    assert!(
        stream
            .accept_pcm16le(&silence(half * 3, ChannelCount::MONO))
            .expect("accepted")
            .is_some()
    );
    assert_eq!(stream.detector().empty_chunks(), 0);
}

// ── channels ────────────────────────────────────────────────────────────────

#[test]
fn stereo_capture_is_downmixed_before_it_reaches_the_detector() {
    let mut stream = WakeWordStream::with_channels(
        ScriptedDetector::silent(),
        SampleRate::HZ_16000,
        ChannelCount::STEREO,
        KeywordSet::new(),
    )
    .expect("built");

    // Two frames: (L=+1000, R=-1000) and (L=+2000, R=+4000).
    let interleaved: [i16; 4] = [1_000, -1_000, 2_000, 4_000];
    stream.accept_samples(&interleaved).expect("accepted");

    assert_eq!(
        stream.detector().samples(),
        2,
        "two frames became two samples"
    );
    let received = stream.detector().last_chunk();
    let divisor = f32::from(i16::MAX) + 1.0;
    assert_eq!(received[0], 0.0);
    assert_eq!(received[1], 3_000.0 / divisor);
}

#[test]
fn a_torn_stereo_frame_never_reaches_the_downmix() {
    // A partial frame would make the downmix reject the buffer as
    // channel-desynchronised. The frame buffer holds it back until it is whole,
    // so the error is unreachable through this path rather than merely handled.
    let mut stream = WakeWordStream::with_channels(
        ScriptedDetector::silent(),
        SampleRate::HZ_16000,
        ChannelCount::STEREO,
        KeywordSet::new(),
    )
    .expect("built");

    // Three bytes: one and a half samples, and not even one whole frame.
    assert_eq!(stream.accept_pcm16le(&[1, 0, 2]).expect("accepted"), None);
    assert_eq!(stream.detector().chunks(), 0);

    // Five more bytes complete two frames and leave nothing torn.
    stream.accept_pcm16le(&[0, 3, 0, 4, 0]).expect("accepted");
    assert_eq!(stream.detector().samples(), 2);
}

#[test]
fn stereo_at_48_kilohertz_reaches_a_16_kilohertz_mono_detector() {
    // The whole chain at once: the shape a real desktop capture has.
    let mut stream = WakeWordStream::with_channels(
        ScriptedDetector::detecting("hey_via"),
        SampleRate::HZ_48000,
        ChannelCount::STEREO,
        keywords(),
    )
    .expect("built");
    assert!(stream.is_resampling());

    let mut detection = None;
    // 20 ms blocks, as a host hands them over.
    for _ in 0..16 {
        if let Some(event) = stream
            .accept_pcm16le(&silence(960, ChannelCount::STEREO))
            .expect("accepted")
        {
            detection = Some(event);
            break;
        }
    }
    let detection = detection.expect("the script fires once a chunk is available");
    assert_eq!(detection.keyword, "hey_via");
    assert_eq!(detection.locale, Some(Locale::En));
}

// ── detections ──────────────────────────────────────────────────────────────

#[test]
fn a_detection_is_attributed_to_the_locale_that_configured_it() {
    let mut stream = WakeWordStream::new(
        ScriptedDetector::detecting("안녕_비아"),
        SampleRate::HZ_16000,
        keywords(),
    )
    .expect("built");
    let detection = stream
        .accept_samples(&[0, 0, 0, 0])
        .expect("accepted")
        .expect("the script fires");
    assert_eq!(detection.keyword, "안녕_비아");
    assert_eq!(detection.locale, Some(Locale::Ko));
}

#[test]
fn a_detection_of_something_nobody_configured_carries_no_locale() {
    // The engine reports whatever keyword file it was loaded with. If that ever
    // disagrees with the table — a stale keywords.txt, a hand-edited file — the
    // detection still arrives, unattributed, rather than being swallowed.
    let mut stream = WakeWordStream::new(
        ScriptedDetector::detecting("something else entirely"),
        SampleRate::HZ_16000,
        keywords(),
    )
    .expect("built");
    let detection = stream
        .accept_samples(&[0, 0])
        .expect("accepted")
        .expect("the script fires");
    assert_eq!(detection.keyword, "something else entirely");
    assert_eq!(detection.locale, None);
}

#[test]
fn a_detector_that_reports_a_locale_itself_is_overridden_by_the_table() {
    // The table is the configuration; a detector's own guess is not.
    let mut stream = WakeWordStream::new(
        ScriptedDetector::new([ScriptStep::Detect(
            Detection::new("hey_via").with_locale(Locale::Zh),
        )]),
        SampleRate::HZ_16000,
        keywords(),
    )
    .expect("built");
    let detection = stream
        .accept_samples(&[0, 0])
        .expect("accepted")
        .expect("fires");
    assert_eq!(detection.locale, Some(Locale::En));
}

#[test]
fn only_the_matching_chunk_reports_a_detection() {
    let mut stream = WakeWordStream::new(
        ScriptedDetector::after(2, Detection::new("hey_via")),
        SampleRate::HZ_16000,
        keywords(),
    )
    .expect("built");

    assert_eq!(stream.accept_samples(&[0]).expect("accepted"), None);
    assert_eq!(stream.accept_samples(&[0]).expect("accepted"), None);
    assert!(stream.accept_samples(&[0]).expect("accepted").is_some());
    assert_eq!(stream.accept_samples(&[0]).expect("accepted"), None);
    assert_eq!(stream.detector().chunks(), 4);
}

// ── reset ───────────────────────────────────────────────────────────────────

#[test]
fn reset_clears_the_buffer_and_the_decoder() {
    let mut stream = WakeWordStream::new(
        ScriptedDetector::silent(),
        SampleRate::HZ_16000,
        KeywordSet::new(),
    )
    .expect("built");

    // A torn frame in flight: one whole sample decoded, one byte carried.
    stream.accept_pcm16le(&[1, 0, 2]).expect("accepted");
    assert_eq!(stream.detector().samples(), 1);

    stream.reset();
    assert_eq!(stream.detector().resets(), 1);

    // The carried `2` is gone. Without the reset these four bytes would decode
    // as `[0x0002, 0x0000]` and leave a byte carried again; with it they are two
    // clean zero samples.
    stream.accept_pcm16le(&[0, 0, 0, 0]).expect("accepted");
    assert_eq!(stream.detector().last_chunk(), &[0.0, 0.0]);
}

#[test]
fn reset_keeps_the_resampler_and_its_filter_history() {
    let mut stream = WakeWordStream::with_channels(
        ScriptedDetector::silent(),
        SampleRate::HZ_48000,
        ChannelCount::STEREO,
        KeywordSet::new(),
    )
    .expect("built");
    assert!(stream.is_resampling());

    stream
        .accept_pcm16le(&silence(960, ChannelCount::STEREO))
        .expect("accepted");
    stream.reset();

    // The converter was not rebuilt: discarding its filter history would put a
    // discontinuity into the signal at exactly the moment the session is
    // trying to hear again.
    assert!(stream.is_resampling());
    assert_eq!(stream.capture_rate(), SampleRate::HZ_48000);
    assert_eq!(stream.detector_rate(), SampleRate::HZ_16000);
}

#[test]
fn the_detector_can_be_taken_back() {
    let mut stream = WakeWordStream::new(
        ScriptedDetector::silent(),
        SampleRate::HZ_16000,
        KeywordSet::new(),
    )
    .expect("built");
    stream.accept_samples(&[0, 0, 0]).expect("accepted");
    assert_eq!(stream.keywords().len(), 0);

    let detector = stream.into_detector();
    assert_eq!(detector.samples(), 3);
}

// ── the detector contract ───────────────────────────────────────────────────

#[test]
fn an_empty_chunk_consumes_no_script_and_cannot_fire() {
    // Asserted against the double directly, because it is the contract every
    // implementation owes — including the engine, which upstream guards the
    // same way at `sherpa-detector.mjs:70`.
    let mut detector = ScriptedDetector::detecting("hey_via");
    assert_eq!(detector.accept(&[]), None);
    assert_eq!(detector.empty_chunks(), 1);
    assert_eq!(detector.chunks(), 0);
    assert_eq!(detector.remaining(), 1);

    assert!(detector.accept(&[0.0]).is_some());
    assert_eq!(detector.remaining(), 0);
}

#[test]
fn a_match_resets_the_stream_so_one_utterance_cannot_fire_twice() {
    let mut detector =
        ScriptedDetector::new([ScriptStep::detect("hey via"), ScriptStep::detect("hey via")]);
    assert_eq!(detector.resets(), 0);
    assert!(detector.accept(&[0.0]).is_some());
    assert_eq!(detector.resets(), 1, "the match reset the stream");
    assert!(detector.accept(&[0.0]).is_some());
    assert_eq!(detector.resets(), 2);

    detector.reset();
    assert_eq!(detector.resets(), 3);
}

#[test]
fn an_exhausted_script_is_silence_not_a_repeat() {
    let mut detector = ScriptedDetector::detecting("hey_via");
    assert!(detector.accept(&[0.0]).is_some());
    for _ in 0..5 {
        assert_eq!(detector.accept(&[0.0]), None);
    }
    assert_eq!(detector.chunks(), 6);
    assert_eq!(detector.remaining(), 0);
}

#[test]
fn the_double_records_exactly_what_it_was_handed() {
    let mut detector = ScriptedDetector::silent();
    detector.accept(&[0.25, -0.5]);
    assert_eq!(detector.last_chunk(), &[0.25, -0.5]);
    detector.accept(&[1.0]);
    assert_eq!(detector.last_chunk(), &[1.0], "only the last chunk is kept");
    assert_eq!(detector.samples(), 3, "but every sample is counted");
}
