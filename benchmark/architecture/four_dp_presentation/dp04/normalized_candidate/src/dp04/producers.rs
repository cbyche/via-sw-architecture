use super::geometry;
use crate::context::Context;
use serde_json::{json, Value};
const SCREEN:&str="Screen Target Producer and Qwen vision parser, not final binder. Parse original PNGs plus raw-event overlay PNGs and actual event coordinates WITHOUT speech/intended answers. Overlay cross/path and event IDs are raw coordinates, not inferred targets. Raw event input stays in original frame pixels; output extents use normalized0..1000 axes. Return ONLY compact JSON {candidates:[{target_id,frame_id,kind,label,coordinate_space:'qwen_normalized_0_1000',bbox_2d:[nx1,ny1,nx2,ny2]}],cue_relations:[{cue_id,target_ids,relation:'point_inside'|'within_region',complete:bool,unique:bool,navigation_alternative:bool}],coverage:{complete:bool,unknown:[strings],competing:[strings]},source_refs:[frame_ids]}. Use visible IDs, short visible header labels max60characters, accurate normalized integer bbox_2d values in0..1000. Point-inside claims must actually contain the coordinate. A loop geometric sample set contains bbox centers within its polygon; inspect actual plotted samples. Preserve unknown/competing candidates/navigation alternatives; geometry alone is not designation. All relevant named alternatives, max30candidates. Expansion asks YOU to reread same pixels and repair missing/invalid extents. Never produce expression bindings. No prose or chain-of-thought.";
const SPEECH:&str="Speech Reference Producer. Interpret full transcript, expression spans, actual event timestamps and prior binding without pixels. Return ONLY compact JSON {intent:string,expressions:[{expression_id,kind,count:integer|null,relation:'designated_by_cue'|'within_region'|'explicit_ids'|'relative_position'|'correction'|'retain_prior'|'other',cue_ids:[event_id],explicit_target_ids:[IDs literally stated in speech],unresolved:[strings],meaning:string}],coverage:{complete:bool,unknown:[strings]}}. Every expression exactly once. Intent MUST describe the user requested action/topic/scope from the full transcript, never your internal reference detection or binding operation. Preserve any explicit speech count for mandatory final checking; singular designated item has count1. Connect cues using wording and time interval, not time alone; distinguish later navigation. For partial correction preserve retained/replaced scope. Intent max120characters, meaning max100characters; no final targets, invented visible IDs or chain-of-thought.";
pub fn screen(
    obs: &Value,
    expansion: &Value,
    generation: u64,
    ctx: &mut Context,
) -> Result<Value, String> {
    let r = &obs["revision"];
    let input = json!({"revision":r["revision"],"frames":r["frames"],"events":r["events"],"native_selection":r["native_selection"],"capture_gaps":r["capture_gaps"],"expansion_request":expansion,"producer_generation":generation});
    let payload=json!({"instruction":format!("{} {}",SCREEN,format!("{} {}",geometry::RELATION_CONTRACT,super::coordinate_wire::CONTRACT)),"input":input,"images":obs["images"],"max_tokens":3000});
    let mut actual_generation=generation;
    let initial=ctx.call(if generation==1 {"dp04.screen_producer"} else {"dp04.screen_source_expansion"},&payload);
    let mut result=match initial {
        Ok(r)=>r,
        Err(error) if generation==1 => {
            ctx.event("GROUNDING_OUTPUT_REJECTED",&json!({"owner":"screen","error":error,"attempt":0}));
            actual_generation=2;
            let mut repair=payload.clone();repair["input"]["output_contract_error"]=json!(error);
            ctx.call("dp04.screen_output_repair",&repair)?
        },
        Err(error)=>return Err(error)
    };
    let wire=super::coordinate_wire::normalize(obs,&mut result,true);
    let wire_error=wire.as_ref().err().cloned();
    result["coordinate_wire_valid"]=json!(wire_error.is_none());
    match wire {Ok(mappings)=>ctx.event("COORDINATES_NORMALIZED",&json!({"owner":"screen","mappings":mappings})),Err(error)=>ctx.event("NORMALIZED_WIRE_REJECTED",&json!({"owner":"screen","error":error}))};
    result["revision"] = r["revision"].clone();
    result["producer_generation"] = json!(actual_generation);
    result["source_snapshot"] = obs["source_snapshot"].clone();
    let mut geometry_issues=geometry::screen_issues(obs,&result);if let Some(error)=wire_error{geometry_issues.push(format!("NORMALIZED_WIRE:{}",error));} result["geometry_issues"]=json!(geometry_issues);
    ctx.logical_buffer(&format!("dp04_screen_candidates_r{}",r["revision"]),result.to_string().len(),"ALLOCATE_OR_REPLACE");
    ctx.event(
        "SCREEN_CANDIDATES_PUBLISHED",
        &json!({"revision":r["revision"],"generation":actual_generation,"state":result}),
    );
    Ok(result)
}
pub fn speech(obs: &Value, prior: &Value, ctx: &mut Context) -> Result<Value, String> {
    let r = &obs["revision"];
    let base=json!({"revision":r["revision"],"transcript":r["transcript"],"expressions":r["expressions"],"events":r["events"],"prior":prior});
    let mut result=Value::Null;
    let mut errors=Vec::new();
    for attempt in 0..2 {
        match ctx.call(if attempt==0 {"dp04.speech_producer"} else {"dp04.speech_output_repair"},&json!({"instruction":format!("{} {}",SPEECH,geometry::RELATION_CONTRACT),"input":{"original":base,"contract_errors":errors,"previous_attempt":result},"max_tokens":1400})) {
            Ok(value)=>{result=value;errors=speech_issues(r,&result);},
            Err(error)=>errors=vec![error],
        }
        if errors.is_empty(){break;}
        ctx.event("GROUNDING_OUTPUT_REJECTED",&json!({"owner":"speech","attempt":attempt,"errors":errors}));
    }
    if !errors.is_empty(){return Err(format!("speech output contract failed: {:?}",errors));}
    result["revision"] = r["revision"].clone();
    ctx.logical_buffer(&format!("dp04_speech_candidates_r{}",r["revision"]),result.to_string().len(),"ALLOCATE_OR_REPLACE");
    ctx.event(
        "SPEECH_CANDIDATES_PUBLISHED",
        &json!({"revision":r["revision"],"state":result}),
    );
    Ok(result)
}

fn speech_issues(revision:&Value,state:&Value)->Vec<String>{
    let mut errors=Vec::new();
    if state["intent"].as_str().is_none_or(|s|s.trim().is_empty() || s.chars().count()>120){errors.push("COMPACT_INTENT_REQUIRED".into());}
    let Some(expressions)=state["expressions"].as_array() else{return vec!["EXPRESSIONS_REQUIRED".into()];};
    for e in revision["expressions"].as_array().into_iter().flatten(){
        let found=expressions.iter().filter(|x|x["expression_id"]==e["expression_id"]).collect::<Vec<_>>();
        if found.len()!=1{errors.push("SPEECH_EXPRESSION_MISSING_OR_DUPLICATE".into());continue;}
        let x=found[0];
        if x.get("count").is_none() || (!x["count"].is_null() && x["count"].as_u64().is_none()){errors.push("COUNT_INTEGER_OR_EXPLICIT_NULL_REQUIRED".into());}
        if !x["cue_ids"].is_array() || !x["unresolved"].is_array(){errors.push("CUE_AND_UNRESOLVED_ARRAYS_REQUIRED".into());}
        if !["designated_by_cue","within_region","explicit_ids","relative_position","correction","retain_prior","other"].contains(&x["relation"].as_str().unwrap_or("")){errors.push("UNKNOWN_SPEECH_RELATION".into());}
    }
    errors
}
