use crate::context::Context;
use serde_json::{json, Value};
use std::fs;

/// Reads only the authorized preserved observation window; never the SVG authoring source.
pub fn read(revision: &Value, ctx: &Context) -> Result<Value, String> {
    let mut normalized_revision = revision.clone();
    normalized_revision["frames"] = json!(revision["frames"]
        .as_array()
        .ok_or("frames missing")?
        .iter()
        .map(crate::source_contract::normalize)
        .collect::<Vec<_>>());
    let revision = &normalized_revision;
    let mut images = Vec::new();
    let mut snapshot = Vec::new();
    for frame in revision["frames"].as_array().ok_or("frames missing")? {
        let path = frame["image_path"].as_str().ok_or("image_path missing")?;
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err("Frame is not PNG".into());
        }
        let identity = json!({"frame_id":frame["frame_id"],"source_id":frame["source_id"],"source_revision":frame["source_revision"],"sha256":frame["sha256"]});
        snapshot.push(identity.clone());
        images.push(json!(path));
        ctx.event(
            "SOURCE_READ",
            &json!({"kind":"PNG_PIXELS","identity":identity,"bytes":bytes.len(),"path":path}),
        );
    }
    Ok(json!({"revision":revision,"images":images,"source_snapshot":snapshot}))
}
