use super::geometry;
use crate::context::Context;
use serde_json::{json, Value};
const INSTRUCTION: &str = "Joint screen/speech grounding. Read actual PNGs, full transcript, original event coordinates/time and prior accepted binding. The overlaid cross/path and event IDs visualize raw observations; they are NOT inferred targets. Original coordinates are frame pixels with supplied width/height. Identify exact visible item/sample IDs for each expression. For corrections preserve the explicitly retained binding, revise replaced bindings, recheck the complete set. Historic pointing remains on its original frame after navigation. A loop may designate plotted samples. You own both target extents and final binding. Do not perform the business task. The intent MUST state the USER requested action and topic/scope (such as comparing performance or averaging selected samples), not your internal bind/identify/grounding operation. Ground the action in the actual full transcript; name adopted targets in the adoptions, not as a substitute for the user action. Return ONLY compact JSON: {intent:string,adoptions:[{expression_id:string,targets:[visible_id],status:'ready'|'ambiguous'|'need_evidence',count:integer|null,relation:'point_inside'|'within_region'|'relative_position'|'retain_prior'|'other',cue_ids:[event_id],extents:[{target_id,frame_id,label:visible_header_max60chars,bbox:[x1,y1,x2,y2]}],evidence:string}],need_evidence:false|{scope:string}}. Each expression once, count is the number required by speech when explicit; IDs must be visible, bbox in original frame pixels and within frame bounds. A point-inside coordinate must fall in chosen bbox. For loop sample sets, each sample bbox center must be in the raw polygon. Request source reread or mark unresolved on contradiction. Evidence max100 characters, intent max120 characters; no chain-of-thought or extra reasoning fields. Any auxiliary parser data is fallible derived interpretation, never truth; verify original pixels/events.";
pub fn ground(obs: &Value, prior: &Value, ctx: &mut Context) -> Result<Value, String> { ground_impl(obs,prior,&Value::Null,ctx,false) }
pub fn ground_with_parser(obs: &Value, prior: &Value, screen: &Value, ctx: &mut Context) -> Result<Value, String> { ground_impl(obs,prior,screen,ctx,false) }
pub fn ground_local(obs:&Value,prior:&Value,ctx:&mut Context)->Result<Value,String>{ground_impl(obs,prior,&Value::Null,ctx,true)}
fn ground_impl(obs: &Value, prior: &Value, screen: &Value, ctx: &mut Context, local: bool) -> Result<Value, String> {
    let input=json!({"observations":obs["revision"],"prior":prior,"auxiliary_screen_parser":screen});
    let mut previous=Value::Null; let mut issues=Vec::new();
    for attempt in 0..2 {
        let mut payload=json!({"instruction":format!("{} {}",INSTRUCTION,geometry::RELATION_CONTRACT),"input":{"original":input,"previous_attempt":previous,"consistency_errors":issues,"repair_feedback":geometry::repair_feedback(&issues),"remaining_source_rejudgments":1-attempt},"images":obs["images"],"max_tokens":2600});
        let repair_ids=if local && attempt==1 {affected(obs,&previous)} else {Vec::new()};
        if local && attempt==1 {
            payload["coordinate_view_request"]=obs["revision"].clone();
            payload["input"]["local_repair_policy"]=json!({"instruction":"You remain the joint final binding owner. Inspect original pixels/events and coordinate views. Return adoptions ONLY for repair_expression_ids using the same compact schema. Host preserves unaffected proposal entries then revalidates the COMPLETE set. These are not yet accepted semantics or truth. If another binding/intent also needs rejudgment, set full_rejudgment:true and return ALL expressions within this same recovery. No additional attempt is granted. Use ORIGINAL frame pixel bboxes unless explicitly using view_pixels with view_id. Do not clamp a bbox, drop a target merely to pass geometry, or invent evidence. Keep uncertainty explicit.","repair_expression_ids":repair_ids,"previous_proposal_is_not_truth":true});
            ctx.event("LOCAL_OWNER_REPAIR_REQUEST",&json!({"issues":issues,"bounded_attempt":1,"final_writer":"joint","no_oracle":true}));
        }
        let stage=if attempt==0 {"dp04.joint_grounding"} else {"dp04.joint_source_reread"};
        match ctx.call(stage,&payload) {
            Ok(mut result)=>{
                if local && attempt==1 && result["full_rejudgment"]!=true {
                    result=merge(&previous,&result,&repair_ids);
                    ctx.event("LOCAL_OWNER_PATCH_MERGED",&json!({"affected_expression_ids":repair_ids,"full_revalidation":true,"final_writer":"joint"}));
                }
                issues=geometry::final_issues(obs,&result,None,None);
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

fn affected(obs:&Value,previous:&Value)->Vec<Value>{
    let expressions=obs["revision"]["expressions"].as_array().cloned().unwrap_or_default();
    let mut ids=Vec::new();
    for expression in &expressions {
        let mut one=obs.clone();one["revision"]["expressions"]=json!([expression]);
        if !geometry::final_issues(&one,previous,None,None).is_empty(){ids.push(expression["expression_id"].clone());}
    }
    if ids.is_empty(){ids=expressions.iter().map(|e|e["expression_id"].clone()).collect();}
    ids
}
fn merge(previous:&Value,patch:&Value,ids:&[Value])->Value{
    let mut result=patch.clone();
    let mut adoptions=previous["adoptions"].as_array().cloned().unwrap_or_default().into_iter().filter(|a|!ids.contains(&a["expression_id"])).collect::<Vec<_>>();
    adoptions.extend(patch["adoptions"].as_array().into_iter().flatten().filter(|a|ids.contains(&a["expression_id"])).cloned());
    result["adoptions"]=json!(adoptions);
    if result["intent"].is_null(){result["intent"]=previous["intent"].clone();}
    result
}
#[cfg(test)]mod local_tests{
    use super::*;
    #[test]fn patch_preserves_unaffected_and_does_not_fill_missing(){
        let old=json!({"intent":"compare","adoptions":[{"expression_id":"a","targets":["x"]},{"expression_id":"b","targets":["y"]}]});
        let patch=json!({"adoptions":[{"expression_id":"a","targets":["z"]},{"expression_id":"b","targets":["invented"]}]});
        let merged=merge(&old,&patch,&[json!("a")]);
        assert_eq!(merged["adoptions"][0]["targets"],json!(["y"]));
        assert_eq!(merged["adoptions"][1]["targets"],json!(["z"]));
        assert_eq!(merge(&old,&json!({"adoptions":[]}),&[json!("a")])["adoptions"].as_array().unwrap().len(),1);
    }
}
