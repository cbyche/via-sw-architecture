use crate::api::GroundingInput;
use serde_json::{json, Value};
pub const CONTRACT:&str="All MODEL-produced item extents must use coordinate_space:'normalized_0_1000', bbox_2d:[nx1,ny1,nx2,ny2], four INTEGER axes0..1000 on the ORIGINAL frame. Do not emit bbox or guess pixels. Raw native bbox and pointer coordinates are original pixels. Infer whole item extents, not just OCR glyph boxes. Visible item IDs such as T1/P1 belong to actual source frames; source_id and source_revision are supplied frame metadata. The host converts ties-even without clamping, checks bounds and any claimed pointer geometry, and canonicalizes actual extent identity as source_id:vSOURCE_REVISION:item_id. Parser output is fallible derived data, never independent evidence or a final target oracle.";
pub fn canonical(frame: &Value, item: &str) -> Result<String, String> {
    if item.trim().is_empty() || item.contains(':') {
        return Err("Visible item_id must be nonempty without canonical prefix".into());
    }
    Ok(format!(
        "{}:v{}:{}",
        frame["source_id"].as_str().ok_or("Source ID missing")?,
        frame["source_revision"]
            .as_u64()
            .ok_or("Source revision missing")?,
        item
    ))
}
pub fn frame<'a>(input: &'a GroundingInput, id: &Value) -> Result<&'a Value, String> {
    let fs = input
        .frames
        .iter()
        .filter(|f| f["frame_id"] == *id)
        .collect::<Vec<_>>();
    if fs.len() != 1 {
        return Err("Extent frame unknown/duplicated".into());
    }
    Ok(fs[0])
}
fn bounds(input: &GroundingInput, extent: &Value) -> Result<(f64, f64), String> {
    let f = frame(input, &extent["frame_id"])?;
    let w = f["width"].as_f64().ok_or("Frame width")?;
    let h = f["height"].as_f64().ok_or("Frame height")?;
    if w <= 0. || h <= 0. {
        return Err("Invalid frame dimensions".into());
    }
    Ok((w, h))
}
pub fn normalize_extent(
    input: &GroundingInput,
    extent: &mut Value,
    allow_native: bool,
) -> Result<(), String> {
    let f = frame(input, &extent["frame_id"])?;
    let (w, h) = bounds(input, extent)?;
    let item = extent["item_id"].as_str().ok_or("Extent item_id missing")?;
    let id = canonical(f, item)?;
    let bbox = if allow_native && extent["origin"] == "ACTUAL_NATIVE_SELECTION" {
        let actual = f["native_selection"]["items"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|n| {
                n["item_id"] == extent["item_id"]
                    && n["bbox"] == extent["bbox"]
                    && n["source_revision"] == f["source_revision"]
                    && n["source_id"] == f["source_id"]
            })
            .ok_or("Native selection extent not actually observed")?;
        actual["bbox"]
            .as_array()
            .ok_or("Native bbox missing")?
            .iter()
            .map(|v| v.as_f64().ok_or("Native bbox numeric"))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        if extent["coordinate_space"] != "normalized_0_1000" || extent.get("bbox").is_some() {
            return Err("Explicit normalized model wire without duplicate pixels required".into());
        }
        let b = extent["bbox_2d"].as_array().ok_or("bbox_2d array")?;
        if b.len() != 4 {
            return Err("bbox_2d length".into());
        }
        let raw = b
            .iter()
            .map(|v| {
                let x = v.as_f64().ok_or("bbox numeric")?;
                if !x.is_finite() || x.fract() != 0. || x < 0. || x > 1000. {
                    return Err("Normalized integer axis outside0..1000");
                }
                Ok(x)
            })
            .collect::<Result<Vec<f64>, _>>()?;
        vec![
            (raw[0] / 1000. * w).round_ties_even(),
            (raw[1] / 1000. * h).round_ties_even(),
            (raw[2] / 1000. * w).round_ties_even(),
            (raw[3] / 1000. * h).round_ties_even(),
        ]
    };
    if bbox.len() != 4
        || bbox.iter().any(|x| !x.is_finite())
        || bbox[0] < 0.
        || bbox[1] < 0.
        || bbox[0] >= bbox[2]
        || bbox[1] >= bbox[3]
        || bbox[2] > w
        || bbox[3] > h
    {
        return Err("Invalid original-frame bbox; no clamping".into());
    }
    extent["bbox"] = json!(bbox);
    extent["canonical_target"] = json!(id);
    extent["source_id"] = f["source_id"].clone();
    extent["source_revision"] = f["source_revision"].clone();
    extent["source_sha256"] = f["sha256"].clone();
    extent["coordinate_space"] = json!("frame_pixels");
    Ok(())
}
pub fn native_candidates(input: &GroundingInput) -> Vec<Value> {
    input.frames.iter().flat_map(|f|f["native_selection"]["items"].as_array().into_iter().flatten().map(|n|json!({"item_id":n["item_id"],"frame_id":f["frame_id"],"source_id":f["source_id"],"source_revision":f["source_revision"],"bbox":n["bbox"],"coordinate_space":"frame_pixels","origin":"ACTUAL_NATIVE_SELECTION","label":n["item_id"]}))).collect()
}
pub fn normalize_bindings(
    input: &GroundingInput,
    result: &mut Value,
    allow_native: bool,
) -> Result<(), String> {
    let mut next = result.clone();
    let bindings = next["bindings"]
        .as_array_mut()
        .ok_or("bindings array missing")?;
    let mut expressions = Vec::new();
    for b in bindings {
        let phrase = b["expression"]
            .as_str()
            .ok_or("binding expression missing")?
            .trim()
            .to_owned();
        if phrase.is_empty() || expressions.contains(&phrase) {
            return Err("Empty or duplicate expression".into());
        }
        expressions.push(phrase);
        if b["status"] != "ready" {
            b["targets"] = json!([]);
            continue;
        }
        let mut extents = b["extents"]
            .as_array()
            .ok_or("ready binding extents missing")?
            .clone();
        if extents.is_empty() {
            return Err("Ready binding needs real source extents".into());
        }
        for e in extents.iter_mut() {
            normalize_extent(input, e, allow_native)?;
        }
        let raw = b["targets"].as_array().ok_or("targets array missing")?;
        let mut canonical_targets = Vec::new();
        for t in raw {
            let matches = extents
                .iter()
                .filter(|e| e["item_id"] == *t || e["canonical_target"] == *t)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err("Target must name exactly one produced extent".into());
            }
            let id = matches[0]["canonical_target"].clone();
            if canonical_targets.contains(&id) {
                return Err("Duplicate target".into());
            }
            canonical_targets.push(id);
        }
        if canonical_targets.is_empty() || canonical_targets.len() != extents.len() {
            return Err("All and only adopted targets must have extents".into());
        }
        if let Some(n) = b["count"].as_u64() {
            if n as usize != canonical_targets.len() {
                return Err("Speech count mismatch".into());
            }
        }
        if b["relation"] == "native_selection" {
            let ids = b["cue_ids"].as_array().ok_or("Native cue IDs missing")?;
            if ids.len() != 1 {
                return Err("Native selection needs one actual cue".into());
            }
            let cue = input
                .cues
                .iter()
                .find(|c| c["event_id"] == ids[0] && c["type"] == "native_selection")
                .ok_or("Actual native cue absent")?;
            let f = frame(input, &cue["frame_id"])?;
            let observed = cue["item_ids"]
                .as_array()
                .ok_or("Native selected items missing")?
                .iter()
                .map(|i| canonical(f, i.as_str().unwrap_or("")))
                .collect::<Result<Vec<_>, _>>()?;
            if observed.len() != canonical_targets.len()
                || !canonical_targets
                    .iter()
                    .all(|t| t.as_str().is_some_and(|s| observed.iter().any(|x| x == s)))
            {
                return Err(
                    "Claimed complete native selection differs from actual selected objects".into(),
                );
            }
        }
        if b["relation"] == "point_inside" {
            let ids = b["cue_ids"].as_array().ok_or("Point cue IDs missing")?;
            if ids.len() != 1 {
                return Err("One claimed designation point required".into());
            }
            let c = input
                .cues
                .iter()
                .find(|c| c["event_id"] == ids[0] && c["type"] == "pointer_move")
                .ok_or("Claimed raw point absent")?;
            let x = c["x"].as_f64().ok_or("point x")?;
            let y = c["y"].as_f64().ok_or("point y")?;
            if !extents.iter().any(|e| {
                let a = e["bbox"].as_array().unwrap();
                e["frame_id"] == c["frame_id"]
                    && a[0].as_f64().unwrap() <= x
                    && x <= a[2].as_f64().unwrap()
                    && a[1].as_f64().unwrap() <= y
                    && y <= a[3].as_f64().unwrap()
            }) {
                return Err("Claimed pointer outside adopted item extent".into());
            }
        }
        b["targets"] = json!(canonical_targets);
        b["extents"] = json!(extents);
    }
    *result = next;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> GroundingInput {
        GroundingInput {
            event_id: "unit".into(),
            revision: 2,
            transcript: "test".into(),
            frames: vec![
                json!({"frame_id":"f","source_id":"DOC","source_revision":1,"width":2880,"height":1800,"native_selection":{"items":[{"item_id":"X1","source_id":"DOC","source_revision":1,"bbox":[100,100,300,300]}]}}),
            ],
            cues: vec![],
            prior: Value::Null,
            evidence: Value::Null,
        }
    }
    fn extent() -> Value {
        json!({"item_id":"X1","frame_id":"f","coordinate_space":"normalized_0_1000","bbox_2d":[0,0,1000,1000]})
    }
    #[test]
    fn canonical_source_revision_is_not_request_revision() {
        let mut e = extent();
        normalize_extent(&input(), &mut e, false).unwrap();
        assert_eq!(e["canonical_target"], "DOC:v1:X1");
        assert_eq!(e["bbox"], json!([0., 0., 2880., 1800.]));
    }
    #[test]
    fn malformed_geometry_never_clamped() {
        for b in [
            json!([-1, 0, 1000, 1000]),
            json!([0, 0, 1001, 1000]),
            json!([0, 0, 1.5, 1000]),
            json!([400, 400, 300, 800]),
        ] {
            let mut e = extent();
            e["bbox_2d"] = b;
            assert!(normalize_extent(&input(), &mut e, false).is_err());
        }
    }
    #[test]
    fn native_boxes_require_actual_selected_object() {
        let mut e = json!({"item_id":"X1","frame_id":"f","bbox":[100,100,300,300],"origin":"ACTUAL_NATIVE_SELECTION"});
        normalize_extent(&input(), &mut e, true).unwrap();
        let mut forged = e.clone();
        forged["bbox"] = json!([90, 90, 300, 300]);
        assert!(normalize_extent(&input(), &mut forged, true).is_err());
    }
    #[test]
    fn failed_binding_normalization_is_transactional() {
        let mut r = json!({"bindings":[{"expression":"this","targets":["X1"],"status":"ready","extents":[extent()],"count":2}]});
        let before = r.clone();
        assert!(normalize_bindings(&input(), &mut r, false).is_err());
        assert_eq!(r, before);
    }
}
