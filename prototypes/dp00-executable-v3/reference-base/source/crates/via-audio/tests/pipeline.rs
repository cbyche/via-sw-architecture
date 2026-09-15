//! The two paths a realtime session actually walks, exercised end to end.
//!
//! The unit tests check each primitive against its own boundary. These check
//! that the primitives compose — which is where off-by-ones between them show
//! up, and where a provider author will be reading for an example.

#![cfg(all(feature = "resample", feature = "wav"))]

use std::io::Cursor;
use std::time::Duration;

use via_audio::{
    ChannelCount, FrameBuffer, PlaybackCursor, Resampler, SampleRate, WavAudio,
    downmix_to_mono_pcm16, read_wav, resample_mono_pcm16, write_wav,
};

/// 48 kHz stereo host capture arriving in ragged byte chunks, coming out as the
/// 16 kHz mono PCM16 an ASR path wants.
#[test]
fn capture_path_48k_stereo_to_16k_mono() {
    const FRAMES: usize = 48_000; // one second

    // A 300 Hz tone on the left, silence on the right, so a channel mix-up is
    // visible rather than merely suspected.
    let mut interleaved = Vec::with_capacity(FRAMES * 2);
    for n in 0..FRAMES {
        let phase = std::f32::consts::TAU * 300.0 * n as f32 / 48_000.0;
        interleaved.push(via_audio::f32_to_pcm16(phase.sin() * 0.5));
        interleaved.push(0);
    }
    let wire = via_audio::i16_to_pcm16le(&interleaved);

    // The host hands over 1000-byte reads: not a whole frame count, not a whole
    // sample count, and never aligned with a 20 ms block.
    let mut capture = FrameBuffer::new(SampleRate::HZ_48000, ChannelCount::STEREO);
    let mut resampler = Resampler::new(
        SampleRate::HZ_48000,
        SampleRate::HZ_16000,
        ChannelCount::MONO,
    )
    .expect("rates");

    let mut asr_input: Vec<i16> = Vec::new();
    for chunk in wire.chunks(1_000) {
        capture.append_pcm16le(chunk);
        // Commit on the 20 ms cadence the device would use.
        if capture.staged_frames() >= SampleRate::HZ_48000.capture_block_frames() {
            capture.commit();
            let block = capture.take_committed();
            let mono = downmix_to_mono_pcm16(&block, ChannelCount::STEREO).expect("aligned");
            asr_input.extend(
                resampler
                    .process_interleaved_pcm16(&mono)
                    .expect("resample"),
            );
        }
    }
    // Flush what is left: the last partial block, then the resampler's tail.
    capture.commit();
    let block = capture.take_committed();
    let mono = downmix_to_mono_pcm16(&block, ChannelCount::STEREO).expect("aligned");
    asr_input.extend(
        resampler
            .process_interleaved_pcm16(&mono)
            .expect("resample"),
    );
    asr_input.extend(resampler.finish_pcm16().expect("flush"));

    // Nothing was dropped, added, or reordered: one second in, one second out.
    assert_eq!(asr_input.len(), 16_000);
    assert_eq!(capture.overrun_frames(), 0);
    assert!(capture.is_empty());
    assert_eq!(
        SampleRate::HZ_16000.frames_to_duration(asr_input.len() as u64),
        Duration::from_secs(1)
    );

    // Amplitude accounting, stated rather than eyeballed: the tone went in at
    // 0.5 on the left channel, the mono downmix averaged it against a silent
    // right channel to 0.25, and the resampler is expected to preserve that —
    // so the rms is 0.25 / sqrt(2).
    let interior = &asr_input[320..asr_input.len() - 320];
    let level = rms_pcm16(interior);
    let expected = 0.25 / std::f32::consts::SQRT_2;
    assert!(
        (level - expected).abs() < 0.005,
        "300 Hz tone came through at rms {level}, expected {expected}"
    );
}

/// The playback side: 24 kHz model speech drains through the Injection Gate,
/// then a barge-in truncates it.
#[test]
fn playback_path_tracks_drain_and_truncation() {
    let rate = SampleRate::HZ_24000;
    let mut speaker = PlaybackCursor::new(rate);

    // Six 20 ms blocks of model speech arrive and are queued.
    let block = rate.playback_block_frames() as u64;
    assert_eq!(block, 480);
    for _ in 0..6 {
        speaker.enqueue(block);
    }
    assert!(speaker.is_draining(), "the gate must hold: audio is queued");
    assert_eq!(speaker.pending_ms(), 120);

    // Two blocks play out.
    speaker.advance(block * 2);
    assert!(speaker.is_draining());
    assert_eq!(speaker.audio_end_ms(), 40);
    assert_eq!(speaker.pending_ms(), 80);

    // Barge-in. The model is told it was cut off 40 ms in, and the four queued
    // blocks are discarded rather than counted as heard.
    let truncate_at = speaker.audio_end_ms();
    assert_eq!(truncate_at, 40);
    assert_eq!(speaker.clear(), block * 4);
    assert!(
        !speaker.is_draining(),
        "the gate must open once the queue is discarded"
    );

    // A fresh response drains fully, and the gate opens.
    speaker.enqueue(block);
    speaker.advance(block);
    assert!(!speaker.is_draining());
    assert_eq!(speaker.pending_duration(), Duration::ZERO);
}

/// A canned WAV fixture — the `mock` provider's shape — read, converted to the
/// realtime output rate, and re-encoded.
#[test]
fn mock_provider_fixture_round_trip() {
    let source = WavAudio::new(
        SampleRate::HZ_48000,
        ChannelCount::MONO,
        (0..48_000)
            .map(|n| {
                let phase = std::f32::consts::TAU * 440.0 * n as f32 / 48_000.0;
                via_audio::f32_to_pcm16(phase.sin() * 0.8)
            })
            .collect(),
    )
    .expect("mono");

    let mut encoded = Cursor::new(Vec::new());
    write_wav(&mut encoded, &source).expect("write");
    let decoded = read_wav(Cursor::new(encoded.into_inner())).expect("read");
    assert_eq!(decoded, source);
    assert_eq!(decoded.duration(), Duration::from_secs(1));

    // Convert to the 24 kHz the realtime wire carries, and check the bytes a
    // provider would actually send.
    let at_24k = resample_mono_pcm16(decoded.samples(), decoded.rate(), SampleRate::HZ_24000)
        .expect("resample");
    assert_eq!(at_24k.len(), 24_000);

    let wire = via_audio::i16_to_pcm16le(&at_24k);
    assert_eq!(wire.len(), 48_000);
    assert_eq!(via_audio::pcm16le_to_i16(&wire), at_24k);

    // And it is still a 440 Hz tone at the level it went in at.
    let level = rms_pcm16(&at_24k[240..at_24k.len() - 240]);
    assert!(
        (level - 0.8 / 2.0_f32.sqrt()).abs() < 0.02,
        "tone came out at rms {level}"
    );
}

/// A session that never commits must not grow without bound, and the audio it
/// keeps must still be usable.
#[test]
fn an_uncommitted_capture_stays_bounded_and_correct() {
    let rate = SampleRate::HZ_16000;
    // A 500 ms pre-roll ring.
    let capacity = rate.millis_to_frames(500) as usize;
    let mut capture = FrameBuffer::with_capacity_frames(rate, ChannelCount::MONO, capacity);

    // Ten seconds of audio, never committed.
    for block in 0..500 {
        let samples: Vec<i16> = (0..320).map(|n| (block * 320 + n) as i16).collect();
        capture.append(&samples);
        assert!(capture.staged_frames() <= capacity);
    }
    assert_eq!(capture.staged_frames(), capacity);
    assert_eq!(capture.staged_duration(), Duration::from_millis(500));
    assert!(capture.overrun_frames() > 0);

    // What survived is the most recent 500 ms, contiguous and in order.
    capture.commit();
    let kept = capture.take_committed();
    assert_eq!(kept.len(), capacity);
    let last = (500 * 320 - 1) as i16;
    assert_eq!(kept[kept.len() - 1], last);
    for pair in kept.windows(2) {
        assert_eq!(pair[1], pair[0].wrapping_add(1), "the ring lost continuity");
    }
}

fn rms_pcm16(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples
        .iter()
        .map(|s| {
            let x = via_audio::pcm16_to_f32(*s);
            x * x
        })
        .sum();
    (sum / samples.len() as f32).sqrt()
}
