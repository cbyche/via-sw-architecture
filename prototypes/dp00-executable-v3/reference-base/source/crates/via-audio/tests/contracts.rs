//! External contract assertions for `via-audio`.
//!
//! Three entries in `docs/reference/contracts.json` land in this crate. Each
//! test below names the contract, quotes its `exactValue`, and asserts it
//! against this crate's behaviour rather than against a restatement of it.
//!
//! | Contract `name` | Kind | Upstream file |
//! | --- | --- | --- |
//! | `pcm16 -> f32 conversion` | `default-value` | `server/src/voice/wake-word/sherpa-detector.mjs:9-17` |
//! | `PCM16 to float conversion` | `json-field` | `server/test/wake-word-detector.test.mjs:5-17` |
//! | `audio sample rates` | `default-value` | `server/src/voice/providers/dashscope.mjs:47-48`, `server/src/voice/providers/s2s.mjs:11-12,34-35`, `tui/native/portaudio-voice-io.py:24-25,133,146` |

use via_audio::{ChannelCount, PCM16_SCALE, SampleRate, pcm16_to_f32, pcm16le_to_f32};

/// Contract `pcm16 -> f32 conversion`:
///
/// > `samples[i] = readInt16LE(i*2) / 32768; input is base64 PCM16LE; sample
/// > count = floor(bytes/2)`
///
/// > Divisor 32768 (not 32767) must match or detection thresholds shift.
#[test]
fn pcm16_to_f32_divides_by_32768_not_32767() {
    assert_eq!(PCM16_SCALE, 32_768.0);
    assert_ne!(PCM16_SCALE, 32_767.0);

    // The literal operation, spelled out: v / 32768 for every representable v.
    for raw in [i16::MIN, -16_384, -1, 0, 1, 16_384, i16::MAX] {
        assert_eq!(
            pcm16_to_f32(raw),
            f32::from(raw) / 32_768.0,
            "sample {raw} did not divide by 32768"
        );
    }

    // Dividing by 32767 would put -32768 outside the normalised range, which is
    // the concrete reason upstream picked 32768.
    assert_eq!(pcm16_to_f32(i16::MIN), -1.0);
    assert!(f32::from(i16::MIN) / 32_767.0 < -1.0);
}

/// Contract `pcm16 -> f32 conversion`, the length half:
///
/// > `sample count = floor(bytes/2)`
#[test]
fn sample_count_is_floor_of_bytes_over_two() {
    for bytes in 0..16_usize {
        let input = vec![0_u8; bytes];
        assert_eq!(
            pcm16le_to_f32(&input).len(),
            bytes / 2,
            "{bytes} bytes did not decode to floor({bytes}/2) samples"
        );
    }
}

/// Contract `PCM16 to float conversion`:
///
/// > `divisor is 32768 (not 32767): -32768→-1, -16384→-0.5, 0→0,
/// > 32767→32767/32768; an odd trailing byte is dropped`
///
/// Upstream asserts exactly this in `server/test/wake-word-detector.test.mjs`,
/// over a four-sample little-endian buffer. This reproduces that buffer byte
/// for byte.
#[test]
fn upstream_wake_word_detector_vector_reproduces_exactly() {
    // `pcm.writeInt16LE(-32768, 0); …(-16384, 2); …(0, 4); …(32767, 6)`
    let pcm: [u8; 8] = [0x00, 0x80, 0x00, 0xC0, 0x00, 0x00, 0xFF, 0x7F];
    assert_eq!(
        pcm16le_to_f32(&pcm),
        vec![-1.0, -0.5, 0.0, 32_767.0 / 32_768.0]
    );
}

/// Contract `PCM16 to float conversion`, the trailing-byte half:
///
/// > `an odd trailing byte is dropped`
///
/// Upstream: `pcm16Base64ToFloat32(Buffer.from([1, 0, 2]))` yields
/// `[1 / 32768]`.
#[test]
fn an_odd_trailing_byte_is_dropped_by_the_stateless_decoder() {
    assert_eq!(pcm16le_to_f32(&[1, 0, 2]), vec![1.0 / 32_768.0]);
}

/// Contract `audio sample rates`:
///
/// > `inputSampleRate = 16000 (dashscope and s2s); outputSampleRate = 24000
/// > (dashscope and s2s).`
///
/// Also `built-in realtime providers`, which pins the same two numbers on the
/// provider descriptors, and the Layer-1 upstream's
/// `REALTIME_SAMPLE_RATE_HZ = 24_000` (upstream `tiniffi/src/live_ffi.rs:60`).
#[test]
fn realtime_input_and_output_rates() {
    assert_eq!(SampleRate::HZ_16000.hz(), 16_000);
    assert_eq!(SampleRate::HZ_24000.hz(), 24_000);
}

/// Contract `audio sample rates`, the block-size half:
///
/// > `Capture chunk = capture_rate/50 (20 ms, min 160 frames); playback block =
/// > playback_rate/50 (min 240 frames).`
///
/// From `tui/native/portaudio-voice-io.py:133,146` —
/// `blocksize=max(160, capture_rate // 50)` and
/// `blocksize=max(240, playback_rate // 50)`.
#[test]
fn capture_and_playback_block_sizes() {
    // The rates the product actually opens streams at.
    assert_eq!(SampleRate::HZ_16000.capture_block_frames(), 320);
    assert_eq!(SampleRate::HZ_24000.capture_block_frames(), 480);
    assert_eq!(SampleRate::HZ_48000.capture_block_frames(), 960);

    assert_eq!(SampleRate::HZ_16000.playback_block_frames(), 320);
    assert_eq!(SampleRate::HZ_24000.playback_block_frames(), 480);
    assert_eq!(SampleRate::HZ_48000.playback_block_frames(), 960);

    // The `max(…)` floors, which only bind below 8 kHz and 12 kHz. Reproduced
    // rather than dropped: they are part of the upstream expression.
    let low = SampleRate::new(4_000).expect("non-zero");
    assert_eq!(low.hz() / 50, 80);
    assert_eq!(low.capture_block_frames(), 160);
    assert_eq!(low.playback_block_frames(), 240);

    // Exactly at each floor's crossover.
    let eight = SampleRate::new(8_000).expect("non-zero");
    assert_eq!(eight.capture_block_frames(), 160);
    let twelve = SampleRate::new(12_000).expect("non-zero");
    assert_eq!(twelve.playback_block_frames(), 240);
}

/// Contract `audio sample rates`, the "20 ms" gloss: a block is a fiftieth of a
/// second at every rate where the floors do not bind.
#[test]
fn a_block_is_twenty_milliseconds() {
    assert_eq!(via_audio::BLOCK_MILLIS, 20);
    for rate in [
        SampleRate::HZ_16000,
        SampleRate::HZ_24000,
        SampleRate::HZ_48000,
    ] {
        let frames = rate.capture_block_frames() as u64;
        assert_eq!(rate.frames_to_millis(frames), via_audio::BLOCK_MILLIS);
        assert_eq!(rate.millis_to_frames(via_audio::BLOCK_MILLIS), frames);
    }
}

/// Contract `sherpa KWS detection config`, the one field of it that is this
/// crate's business:
///
/// > `featConfig {samplingRate:16000, featureDim:80}`
///
/// The wake-word feature extractor runs at the realtime input rate, so audio
/// reaching it needs no further conversion. Asserted here so that a change to
/// [`SampleRate::HZ_16000`] cannot silently desynchronise the two.
#[test]
fn wake_word_features_run_at_the_realtime_input_rate() {
    assert_eq!(SampleRate::HZ_16000.hz(), 16_000);
}

/// Not a catalogued contract, but the invariant the catalogued ones rest on:
/// the conversion this crate performs must be reversible, or a fixture written
/// by one part of VIA and read by another drifts.
#[test]
fn the_documented_convention_holds_across_the_public_api() {
    let originals: Vec<i16> = (i16::MIN..=i16::MAX).step_by(7).collect();
    let bytes = via_audio::i16_to_pcm16le(&originals);
    let floats = pcm16le_to_f32(&bytes);
    let back = via_audio::f32_to_pcm16_slice(&floats);
    assert_eq!(back, originals);

    // And the same values survive a mono→stereo→mono trip.
    let stereo = via_audio::upmix_from_mono(&originals, ChannelCount::STEREO);
    let mono = via_audio::downmix_to_mono_pcm16(&stereo, ChannelCount::STEREO)
        .expect("stereo is frame aligned");
    assert_eq!(mono, originals);
}
