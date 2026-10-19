use super::geometry;
use crate::context::Context;
use serde_json::{json, Value};
const INSTRUCTION: &str = "Joint screen/speech grounding. Read actual PNGs, full transcript, original event coordinates/time and prior accepted binding. The overlaid cross/path and event IDs visualize raw observations; they are NOT inferred targets. Original coordinates are frame pixels with supplied width/height. Identify exact visible item/sample IDs for each expression. For corrections preserve the explicitly retained binding, revise replaced bindings, recheck the complete set. Historic pointing remains on its original frame after navigation. A loop may designate plotted samples. You own both target extents and final binding. Do not perform the business task. The intent MUST state the USER requested action and topic/scope (such as comparing performance or averaging selected samples), not your internal bind/identify/grounding operation. Ground the action in the actual full transcript; name adopted targets in the adoptions, not as a substitute for the user action. Return ONLY compact JSON: {intent:string,adoptions:[{expression_id:string,targets:[visible_id],status:'ready'|'ambiguous'|'need_evidence',count:integer|null,relation:'point_inside'|'within_region'|'relative_position'|'retain_prior'|'other',cue_ids:[event_id],extents:[{target_id,frame_id,label:visible_header_max60chars,coordinate_space:'qwen_normalized_0_1000',bbox_2d:[nx1,ny1,nx2,ny2]}],evidence:string}],need_evidence:false|{scope:string}}. Each expression once, count is the number required by speech when explicit; IDs must be visible, bbox_2d in normalized 0..1000 original-frame axes. After the shared normalized-to-pixel conversion, a raw point must fall in the chosen bbox. For loop sample sets, each sample bbox center must be in the raw polygon. Request source reread or mark unresolved on contradiction. Evidence max100 characters, intent max120 characters; no chain-of-thought or extra reasoning fields. Any auxiliary parser data is fallible derived interpretation, never truth; verify original pixels/events.";
pub fn ground(obs: &Value, prior: &Value, ctx: &mut Context) -> Result<Value, String> { ground_impl(obs,prior,&Value::Null,ctx) }
pub fn ground_with_parser(obs: &Value, prior: &Value, screen: &Value, ctx: &mut Context) -> Result<Value, String> { ground_impl(obs,prior,screen,ctx) }
fn ground_impl(obs: &Value, prior: &Value, screen: &Value, ctx: &mut Context) -> Result<Value, String> {
    let input=json!({"observations":obs["revision"],"prior":prior,"auxiliary_screen_parser":screen});
    let mut previous=Value::Null; let mut issues=Vec::new();
    for attempt in 0..2 {
        let payload=json!({"instruction":format!("{} {}",INSTRUCTION,format!("{} {}",geometry::RELATION_CONTRACT,super::coordinate_wire::CONTRACT)),"input":{"original":input,"previous_attempt":previous,"consistency_errors":issues,"repair_feedback":geometry::repair_feedback(&issues),"remaining_source_rejudgments":1-attempt},"images":obs["images"],"max_tokens":2600});
        let stage=if attempt==0 {"dp04.joint_grounding"} else {"dp04.joint_source_reread"};
        match ctx.call(stage,&payload) {
            Ok(mut result)=>{
                let wire=super::coordinate_wire::normalize(obs,&mut result,false);
                issues=geometry::final_issues(obs,&result,None,None);
                match wire {Ok(mappings)=>ctx.event("COORDINATES_NORMALIZED",&json!({"owner":"joint","mappings":mappings})),Err(error)=>{issues.push(format!("NORMALIZED_WIRE:{}",error));ctx.event("NORMALIZED_WIRE_REJECTED",&json!({"owner":"joint","error":error}));}}
                if result["intent"].as_str().is_none_or(|s|s.trim().is_empty() || s.chars().count()>120) {issues.push("COMPACT_INTENT_REQUIRED".into());}
                if result["need_evidence"].is_object(){issues.push("MODEL_REQUESTED_SOURCE_REREAD".into());}
                if issues.is_empty(){result["validated_by"]=json!("actual_frame_bounds_count_and_claimed_event_geometry");return Ok(result);}
                ctx.event("GROUNDING_CONSISTENCY_REJECTED",&json!({"owner":"joint","attempt":attempt,"issues":issues,"proposal":result})); previous=result;
            },
            Err(error)=>{issues=vec![format!("MODEL_OUTPUT_CONTRACT:{}",error)];ctx.event("GROUNDING_OUTPUT_REJECTED",&json!({"owner":"joint","attempt":attempt,"error":error}));}
        }
        if attempt==0 {ctx.event("BOUNDED_SOURCE_REREAD",&json!({"owner":"joint","issues":issues,"revision":obs["revision"]["revision"]}));}
    }
    Ok(geometry::unresolved(obs,&issues))
}
