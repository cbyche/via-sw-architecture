use super::producers;
use crate::{api::GroundingInput, context::Context};
use serde_json::{json, Value};
const INSTRUCTION:&str="Multimodal Binding Resolver. Combine current speech reference meanings with producer-owned visual candidates and prior ACTUALLY adopted bindings. You have NO pixels and MUST NOT create targets or extents. Final target IDs must be existing candidate canonical_target values. Respect within-utterance self-correction and observed document ordering within the prior preserved set for 'first'. Keep currently retained paragraph action while replacing only the spoken preserved-table scope. Return JSON {bindings:[{expression:exact producer expression,targets:[existing canonical_target],status:'ready'|'need_evidence',relation:producer_relation,cue_ids:producer_cue_ids,count:producer_count}],need_expansion:false|{reason:string},intent:string,unknowns:[string]}. If a needed source item or geometry is missing/ambiguous, request screen-owner expansion instead of guessing. Intent describes the user's actual handling scope. No chain-of-thought or user response.";
fn typed(input: &GroundingInput, screen: &Value, speech: &Value) -> Option<Value> {
    let refs = speech["references"].as_array()?;
    let cs = screen["candidates"].as_array()?;
    let mut out = Vec::new();
    for r in refs {
        let ids = r["cue_ids"].as_array()?;
        if ids.len() != 1 {
            return None;
        }
        let cue = input.cues.iter().find(|c| c["event_id"] == ids[0])?;
        let chosen = if r["relation"] == "native_selection" && cue["type"] == "native_selection" {
            cs.iter()
                .filter(|c| {
                    c["origin"] == "ACTUAL_NATIVE_SELECTION"
                        && c["frame_id"] == cue["frame_id"]
                        && cue["item_ids"]
                            .as_array()
                            .is_some_and(|items| items.contains(&c["item_id"]))
                })
                .collect::<Vec<_>>()
        } else if r["relation"] == "point_inside"
            && cue["type"] == "pointer_move"
            && screen["coverage"]["complete"] == true
            && screen["coverage"]["unknowns"]
                .as_array()
                .is_some_and(|a| a.is_empty())
        {
            let x = cue["x"].as_f64()?;
            let y = cue["y"].as_f64()?;
            let hits = cs
                .iter()
                .filter(|c| {
                    let Some(b) = c["bbox"].as_array() else {
                        return false;
                    };
                    c["frame_id"] == cue["frame_id"]
                        && b.len() == 4
                        && b[0].as_f64().unwrap_or(f64::INFINITY) <= x
                        && x <= b[2].as_f64().unwrap_or(f64::NEG_INFINITY)
                        && b[1].as_f64().unwrap_or(f64::INFINITY) <= y
                        && y <= b[3].as_f64().unwrap_or(f64::NEG_INFINITY)
                })
                .collect::<Vec<_>>();
            if hits.len() != 1 {
                return None;
            }
            hits
        } else {
            return None;
        };
        if chosen.is_empty()
            || r["count"]
                .as_u64()
                .is_some_and(|n| n as usize != chosen.len())
        {
            return None;
        }
        out.push(json!({"expression":r["expression"],"targets":chosen.iter().map(|c|c["canonical_target"].clone()).collect::<Vec<_>>(),"extents":chosen,"status":"ready","relation":r["relation"],"cue_ids":r["cue_ids"],"count":r["count"]}));
    }
    Some(
        json!({"bindings":out,"intent":speech["intent"],"unknowns":[],"resolver":"TYPED_OBSERVED_CUE_RELATION","need_expansion":false}),
    )
}
fn propose(
    ctx: &mut Context,
    input: &GroundingInput,
    screen: &Value,
    speech: &Value,
) -> Result<Value, String> {
    if let Some(v) = typed(input, screen, speech) {
        ctx.event("GROUNDING_TYPED_BIND", &v);
        return Ok(v);
    }
    ctx.call("grounding.candidate_binding",&json!({"instruction":INSTRUCTION,"input":{"screen":screen,"speech":speech,"cues":input.cues,"prior":input.prior},"max_tokens":1500}))
}
fn materialize(
    input: &GroundingInput,
    result: &mut Value,
    screen: &Value,
    speech: &Value,
) -> Result<(), String> {
    let cs = screen["candidates"]
        .as_array()
        .ok_or("Candidate array absent")?;
    let refs = speech["references"]
        .as_array()
        .ok_or("Speech references absent")?;
    let bs = result["bindings"]
        .as_array_mut()
        .ok_or("Binding array absent")?;
    for r in refs {
        let found = bs
            .iter()
            .filter(|b| b["expression"] == r["expression"])
            .count();
        if found != 1 {
            return Err("Every produced speech expression exactly once".into());
        }
    }
    if bs.len() != refs.len() {
        return Err("Binding introduced expression absent from speech owner".into());
    }
    for b in bs {
        let r = refs
            .iter()
            .find(|r| r["expression"] == b["expression"])
            .ok_or("Unowned expression")?;
        for field in ["count", "cue_ids", "relation"] {
            b[field] = r[field].clone();
        }
        let ts = b["targets"].as_array().ok_or("Targets array absent")?;
        let mut extents = Vec::new();
        for t in ts {
            let matches = cs
                .iter()
                .filter(|c| c["canonical_target"] == *t)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err("Binder may only use exactly one producer candidate per target".into());
            }
            extents.push(matches[0].clone());
        }
        b["extents"] = json!(extents);
    }
    // Candidate extents are already canonical original pixels. Validate all geometry,
    // converting back to normalized wire is neither needed nor an inference step.
    for b in result["bindings"].as_array().unwrap() {
        if b["status"] != "ready" {
            return Err("Unresolved candidate binding".into());
        }
        let ts = b["targets"].as_array().unwrap();
        if ts.is_empty() {
            return Err("Ready binding empty".into());
        }
        if b["count"].as_u64().is_some_and(|n| n as usize != ts.len()) {
            return Err("Final speech count mismatch".into());
        }
        if b["relation"] == "point_inside" {
            let ids = b["cue_ids"].as_array().ok_or("Point cue IDs")?;
            if ids.len() != 1 {
                return Err("Point needs unique cue".into());
            }
            let c = input
                .cues
                .iter()
                .find(|c| c["event_id"] == ids[0] && c["type"] == "pointer_move")
                .ok_or("Raw designation cue absent")?;
            let x = c["x"].as_f64().ok_or("cue x")?;
            let y = c["y"].as_f64().ok_or("cue y")?;
            if !b["extents"].as_array().unwrap().iter().any(|e| {
                let a = e["bbox"].as_array().unwrap();
                e["frame_id"] == c["frame_id"]
                    && a[0].as_f64().unwrap() <= x
                    && x <= a[2].as_f64().unwrap()
                    && a[1].as_f64().unwrap() <= y
                    && y <= a[3].as_f64().unwrap()
            }) {
                return Err("Raw designation point outside producer-owned item".into());
            }
        }
    }
    Ok(())
}
pub fn resolve(
    ctx: &mut Context,
    input: &GroundingInput,
    mut screen: Value,
    speech: Value,
) -> Result<Value, String> {
    let mut proposal = Value::Null;
    let mut errors = Vec::new();
    for attempt in 0..2 {
        if attempt > 0 {
            ctx.event(
                "GROUNDING_SCREEN_EXPANSION",
                &json!({"errors":errors,"previous_proposal":proposal}),
            );
            screen = producers::screen(
                ctx,
                input,
                &json!({"errors":errors,"binding_request":proposal,"speech":speech}),
            )?;
        }
        proposal = propose(ctx, input, &screen, &speech)?;
        if proposal["need_expansion"].is_object() {
            errors = vec!["Binder requested original-source expansion".to_string()];
            continue;
        }
        match materialize(input, &mut proposal, &screen, &speech) {
            Ok(()) => {
                proposal["intent"] = speech["intent"].clone();
                proposal["input_revision"] = json!(input.revision);
                proposal["mechanism"] = json!("DEDICATED_VISUAL_PRODUCTION_THEN_CANDIDATE_BINDING");
                proposal["candidate_read_set"] = json!({"source_frames":input.frames.iter().map(|f|json!({"source_id":f["source_id"],"source_revision":f["source_revision"],"sha256":f["sha256"]})).collect::<Vec<_>>()});
                return Ok(proposal);
            }
            Err(e) => errors = vec![e],
        }
    }
    Err(format!(
        "Candidate binding failed after owner expansion: {:?}",
        errors
    ))
}
