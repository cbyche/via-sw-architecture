use serde_json::{Value, json};
use std::collections::VecDeque;
use std::io;
use std::time::Instant;
use via_dp00_executable_v3::serve;

fn main() -> io::Result<()> {
    serve(|request| {
        let start = Instant::now();
        let depth_ms = request
            .get("playback_buffer_depth_ms")
            .and_then(Value::as_u64)
            .unwrap_or(40);
        let mut frames: VecDeque<u64> =
            (0..depth_ms.div_ceil(10)).map(|index| index * 10).collect();
        let consumed_at_onset = frames.pop_front();
        std::thread::yield_now();
        let cleared_frames = frames.len();
        frames.clear();
        json!({
            "component":"via-local-playback",
            "pid":std::process::id(),
            "frame_ms":10,
            "cleared_frames":cleared_frames,
            "remaining_frames":frames.len(),
            "last_frame_end_offset_ms":consumed_at_onset.map(|_| 10).unwrap_or(0),
            "measured_cancel_ns":start.elapsed().as_nanos(),
            "events":["playback-active","speech-onset","barge-in-detected","cancel-request","ring-clear","last-frame"]
        })
    })
}
