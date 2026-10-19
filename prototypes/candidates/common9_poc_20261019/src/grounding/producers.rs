use super::wire;
use crate::{api::GroundingInput, context::Context};
use serde_json::{json, Value};
const SCREEN:&str="Screen Target Producer. You do not receive speech and you do not bind expressions. From actual PNGs, fallible dedicated detector/OCR/caption output, actual native selected ranges and raw cues, produce whole visible source items. OCR glyph boxes are NOT whole table/paragraph extents; infer required item extents from original pixels. Return JSON {candidates:[{item_id,frame_id,label,coordinate_space:'normalized_0_1000',bbox_2d:[nx1,ny1,nx2,ny2]}],coverage:{complete:bool,unknowns:[string]}}. Keep competing/unknown visual items and all actual observed cue targets. IDs must be read from visible labels or actual native item metadata. Maximum20 items; labels max100characters. Expansion rereads the same original sources and owns all missing/incorrect candidate extents. No final speech bindings or invented targets.";
const SPEECH:&str="Speech Reference Producer. You have speech, raw cue event IDs/timing/types and ACTUALLY adopted prior state, without pixels or screen candidates. Extract final reference noun phrases and their relationship/cue IDs. Within-utterance self-correction discards the superseded phrase; later partial correction retains requested old scope. 'first' refers within the previously preserved set using observed document order; JSON serialization order is not new evidence. Return compactJSON {references:[{expression:string,relation:'native_selection'|'point_inside'|'prior_reference'|'other',cue_ids:[event_id],count:integer|null,meaning:string}],intent:string,unknowns:[string]}. Use exact short noun phrases without trailing Korean case particles, e.g.'이 표들'; no evaluator IDs or final target IDs. Do not infer screen labels. Preserve an explicit count; use null when speech does not fix one. Intent states the actual user's action and correction/retention scope, not your parsing activity.";
pub fn native_sufficient(input: &GroundingInput) -> bool {
    !wire::native_candidates(input).is_empty()
        && !input.cues.iter().any(|c| {
            c["type"] == "pointer_move"
                || c["type"] == "pointer_path"
                || c["type"] == "pointer_drag"
        })
}
pub fn screen(
    ctx: &mut Context,
    input: &GroundingInput,
    expansion: &Value,
) -> Result<Value, String> {
    if native_sufficient(input) {
        let mut candidates = wire::native_candidates(input);
        for c in &mut candidates {
            wire::normalize_extent(input, c, true)?;
        }
        let state = json!({"candidates":candidates,"coverage":{"complete":false,"unknowns":["Nonselected scene objects unavailable to selection-only native API"]},"native_selection_only":true});
        ctx.event("GROUNDING_NATIVE_CANDIDATES", &state);
        return Ok(state);
    }
    let helper = ctx.call(
        "grounding.local_screen_parser",
        &json!({"input":{"frames":input.frames},"max_tokens":0}),
    )?;
    if helper["parser_error"].is_string() {
        return Err(format!(
            "Dedicated parser failed: {}",
            helper["parser_error"]
        ));
    }
    let mut previous = Value::Null;
    let mut errors = Vec::<String>::new();
    for attempt in 0..2 {
        let payload = json!({"instruction":format!("{} {}",SCREEN,wire::CONTRACT),"input":{"frames":input.frames,"cues":input.cues,"dedicated_parser":helper,"expansion_request":expansion,"previous_attempt":previous,"contract_errors":errors},"images":input.frames.iter().filter_map(|f|f["image_path"].as_str()).collect::<Vec<_>>(),"max_tokens":2600});
        let mut v = ctx.call(
            if attempt == 0 {
                "grounding.screen_producer"
            } else {
                "grounding.screen_output_repair"
            },
            &payload,
        )?;
        let result = (|| -> Result<(), String> {
            let cs = v["candidates"]
                .as_array_mut()
                .ok_or("Screen candidates array")?;
            if cs.len() > 20 {
                return Err("Screen candidate cap".into());
            }
            for c in cs {
                wire::normalize_extent(input, c, false)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                let cs = v["candidates"].as_array_mut().unwrap();
                for mut n in wire::native_candidates(input) {
                    wire::normalize_extent(input, &mut n, true)?;
                    cs.retain(|c| c["canonical_target"] != n["canonical_target"]);
                    cs.push(n);
                }
                ctx.event("GROUNDING_SCREEN_CANDIDATES", &v);
                return Ok(v);
            }
            Err(e) => {
                errors = vec![e];
                previous = v;
                ctx.event(
                    "GROUNDING_OWNER_REJUDGMENT",
                    &json!({"owner":"screen","attempt":attempt,"errors":errors}),
                );
            }
        }
    }
    Err(format!("Screen output contract failed: {:?}", errors))
}
pub fn speech(ctx: &mut Context, input: &GroundingInput) -> Result<Value, String> {
    let cues = input
        .cues
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if let Some(o) = c.as_object_mut() {
                o.remove("item_ids");
            }
            c
        })
        .collect::<Vec<_>>();
    let mut previous = Value::Null;
    for attempt in 0..2 {
        let v=ctx.call(if attempt==0{"grounding.speech_producer"}else{"grounding.speech_output_repair"},&json!({"instruction":SPEECH,"input":{"transcript":input.transcript,"cues":cues,"prior":input.prior,"previous_attempt":previous},"max_tokens":1400}))?;
        if v["references"].as_array().is_some_and(|xs| {
            !xs.is_empty()
                && xs.iter().all(|x| {
                    x["expression"]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty())
                        && x["cue_ids"].is_array()
                        && (x["count"].is_null() || x["count"].as_u64().is_some())
                })
        }) {
            return Ok(v);
        }
        previous = v;
    }
    Err("Speech reference output missing typed references/cues/count".into())
}
