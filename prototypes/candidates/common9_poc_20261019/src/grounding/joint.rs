use super::wire;
use crate::{api::GroundingInput, context::Context};
use serde_json::{json, Value};
const INSTRUCTION:&str="Joint screen/speech grounder. Read actual PNGs, current raw speech and raw pointer/native-selection observations together with prior ACTUALLY adopted state. Return only final reference phrases, removing within-utterance self-corrected references. For a later partial correction, preserve the specified prior item and revise only the replaced scope. 'first' refers within the previously preserved set, using its observed document order; JSON serialization order is not new semantic evidence. Pointer time alone is not intent. Native selection exports only actually selected object ranges; it is not a complete scene object map. Return compact JSON {bindings:[{expression:string,targets:[visible_item_id],extents:[{item_id,frame_id,coordinate_space:'normalized_0_1000',bbox_2d:[nx1,ny1,nx2,ny2]}],status:'ready'|'need_evidence',relation:'native_selection'|'point_inside'|'prior_reference'|'other',cue_ids:[raw_event_id],count:integer|null}],intent:string,unknowns:[string]}. expression is the exact short referential noun phrase without trailing Korean case particles, e.g. '이 표들' rather than a full sentence. Do not invent evaluator binding IDs. Intent describes the USER's actual requested action and retained/corrected scope, not your internal detection job. Every ready target needs its own whole-item extent. If claiming point_inside, attach the actual designation cue. Do not perform the business task or emit any user response.";
pub fn ground(ctx: &mut Context, input: &GroundingInput) -> Result<Value, String> {
    let mut prior_attempt = Value::Null;
    let mut errors = Vec::<String>::new();
    for attempt in 0..2 {
        let payload = json!({"instruction":format!("{} {}",INSTRUCTION,wire::CONTRACT),"input":{"transcript":input.transcript,"frames":input.frames,"cues":input.cues,"prior":input.prior,"evidence":input.evidence,"previous_attempt":prior_attempt,"contract_errors":errors,"input_revision":input.revision},"images":input.frames.iter().filter_map(|f|f["image_path"].as_str()).collect::<Vec<_>>(),"max_tokens":2200});
        match ctx.call(
            if attempt == 0 {
                "grounding.joint"
            } else {
                "grounding.joint_source_rejudgment"
            },
            &payload,
        ) {
            Ok(mut v) => match wire::normalize_bindings(input, &mut v, false) {
                Ok(()) => {
                    v["mechanism"] = json!("JOINT_ORIGINAL_EVIDENCE");
                    v["input_revision"] = json!(input.revision);
                    return Ok(v);
                }
                Err(e) => {
                    prior_attempt = v;
                    errors = vec![e];
                }
            },
            Err(e) => errors = vec![e],
        }
        ctx.event(
            "GROUNDING_OWNER_REJUDGMENT",
            &json!({"owner":"joint","attempt":attempt,"errors":errors}),
        );
    }
    Err(format!(
        "Joint grounding contract failed after bounded reread: {:?}",
        errors
    ))
}
