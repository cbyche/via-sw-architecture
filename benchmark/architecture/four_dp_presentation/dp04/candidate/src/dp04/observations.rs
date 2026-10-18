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
        let node=format!("dp04-source-r{}-{}",revision["revision"],frame["frame_id"]);
        ctx.event("LOCAL_OPERATION_START",&json!({"node_id":node,"stage":"dp04_source_pixels","parent_ids":ctx.last_node().into_iter().collect::<Vec<_>>() }));
        let path = frame["image_path"].as_str().ok_or("image_path missing")?;
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            return Err("Frame is not PNG".into());
        }
        let identity = json!({"frame_id":frame["frame_id"],"source_id":frame["source_id"],"source_revision":frame["source_revision"],"sha256":frame["sha256"]});
        snapshot.push(identity.clone());
        images.push(json!(path));
        if let Some(overlay) = frame["overlay_image_path"].as_str() {
            let data = fs::read(overlay).map_err(|e| e.to_string())?;
            if !data.starts_with(b"\x89PNG\r\n\x1a\n") { return Err("Overlay is not PNG".into()); }
            images.push(json!(overlay));
            ctx.event("RAW_EVENT_OVERLAY_READ", &json!({"frame_id":frame["frame_id"],"path":overlay,"bytes":data.len(),"encoding":frame["overlay_encoding"],"no_target_inference":true}));
            ctx.logical_buffer("dp04_overlay_png",data.len(),"ALLOCATE");
            ctx.logical_buffer("dp04_overlay_png",0,"RELEASE");
        }
        ctx.event(
            "SOURCE_READ",
            &json!({"kind":"PNG_PIXELS","identity":identity,"bytes":bytes.len(),"path":path}),
        );
        ctx.logical_buffer("dp04_source_png",bytes.len(),"ALLOCATE");
        ctx.logical_buffer("dp04_source_png",0,"RELEASE");
        ctx.event("LOCAL_OPERATION_END",&json!({"node_id":node,"stage":"dp04_source_pixels"}));
        ctx.join(&[node]);
    }
    ctx.logical_buffer("dp04_observations",revision.to_string().len(),"ALLOCATE");
    Ok(json!({"revision":revision,"images":images,"source_snapshot":snapshot}))
}
