//! Semantic intent adoption from actual user input and current public/retained state.
use serde_json::{json,Value};
use crate::context::Context;
use super::state::{self,Shared};
const INSTRUCTION:&str="Interpret ONLY input.event, the CURRENT user input in VIA. Other captured or adopted inputs are context for association, not additional intents to attribute to this event. Every returned intent must be expressed by the current event text; never treat a later captured reply as part of this event. Return JSON {intents:[{relation:'NEW'|'FOLLOWUP'|'CONTINUE'|'CORRECTION'|'ANSWER',target_request_id:string|null,question_id:string|null,task_id:string|null,scope:string,targets:[string],handling_text:string,source_revisions:{source_id:revision},complete_answer:string|null,source_claims:[{source_id,revision,quote:exact supplied substring}]}]}. One utterance may contain multiple intents: preserving an earlier job and separately asking a new question are separate intents. Do not collapse them to one label. Captured-but-uninterpreted input is not an already adopted request. A correction changes only its stated scope and targets the actual earlier request; preserve remaining conditions. An answer associates with the actually displayed question and its Task, even when another request is active. handling_text must concretely name the adopted objects/topic and intended action; resolve pronouns, never simply echo unresolved 'it/that'. This stage first adopts the objects, relation and handling intention. complete_answer is a useful fast path ONLY when the requested answer naturally completes in one or two sentences; for a detailed multi-part comparison or analysis return null and let the real detailed-response stage continue. Never truncate the requested answer to force the fast path. An instruction to continue is not a new independent goal. A correction acknowledgement is not completion of the revised original goal. For a complete_answer using source facts, include actual quote claims for its source_revisions. Do not invent source facts or claim external execution.";
pub fn validate(value:&Value,history:&Value,sources:&Value)->Result<(),String>{
    let intents=value["intents"].as_array().filter(|a|!a.is_empty()).ok_or("intents must be a nonempty array")?;
    for i in intents{
        let relation=i["relation"].as_str().ok_or("relation missing")?;
        if !["NEW","FOLLOWUP","CONTINUE","CORRECTION","ANSWER"].contains(&relation){return Err("unknown relation".into())}
        for f in ["scope","handling_text"]{if i[f].as_str().filter(|s|!s.trim().is_empty()).is_none(){return Err(format!("{} missing",f))}}
        if !i["targets"].is_array() || !i["source_revisions"].is_object(){return Err("targets/source_revisions malformed".into())}
        for (id,revision) in i["source_revisions"].as_object().unwrap(){
            if !sources.as_array().is_some_and(|a|a.iter().any(|s|s["id"]==id.as_str() && s["revision"]==*revision && s["revoked"]!=true)){return Err("adopted source ID/revision is not actually supplied/current".into())}
        }
        if ["FOLLOWUP","CONTINUE","CORRECTION"].contains(&relation){
            let id=i["target_request_id"].as_str().ok_or("target request missing")?;
            let known=history["requests"].as_array().is_some_and(|a|a.iter().any(|r|r["request_id"]==id)) || history["captured_inputs"].as_array().is_some_and(|a|a.iter().any(|r|r["id"]==id));
            if !known{return Err("target request not in actual capture/adoption state".into())}
        }
        if relation=="ANSWER"{
            let q=history["actually_displayed_questions"].as_array().and_then(|a|a.iter().find(|q|q["id"]==i["question_id"] && q["task_id"]==i["task_id"]));
            if q.is_none(){return Err("answer has no matching displayed question/task".into())}
        }
    }Ok(())
}
pub fn interpret(ctx:&mut Context,input:&Value,event:&Value,state:&Shared,retained:Option<&Value>)->Result<Value,String>{
    let mut history=state::snapshot(state);
    // Interpretation view stops at the actual current captured input. Ingress/publication state
    // still retains every captured input, including later pending corrections/answers.
    if let Some(captured)=history["captured_inputs"].as_array_mut(){if let Some(index)=captured.iter().position(|e|e["id"]==event["id"]){captured.truncate(index+1);}}
    let mut error=Value::Null;
    for attempt in 0..=1{
        let p=json!({"instruction":INSTRUCTION,"input":{"event":event,"sources":input["sources"],"conversation":history,"retained_conversation_step":retained,"repair_reason":error},"max_tokens":2000});
        match ctx.call(if attempt==0{"dp02_adopt_intents"}else{"dp02_repair_intents"},&p){
            Ok(mut v)=>match validate(&v,&history,&input["sources"]){Ok(())=>{v["event_id"]=event["id"].clone();v["revision"]=event["revision"].clone();ctx.event("INTENT_PROPOSAL_VALIDATED",&v);return Ok(v)},Err(e)=>{error=json!(e);ctx.event("SEMANTIC_STAGE_REJECTED",&json!({"event_id":event["id"],"stage":"intent","attempt":attempt,"reason":e,"proposal":v}));}},
            Err(e)=>{error=json!(e);ctx.event("SEMANTIC_STAGE_REJECTED",&json!({"event_id":event["id"],"stage":"intent","attempt":attempt,"reason":e}));}
        }
    }Err(format!("intent recovery exhausted: {}",error))
}
#[cfg(test)]mod tests{use super::*;#[test]fn null_intents_rejected(){assert!(validate(&json!({"intents":null}),&json!({}),&json!([])).is_err());}#[test]fn answer_requires_actual_task_question(){assert!(validate(&json!({"intents":[{"relation":"ANSWER","scope":"s","handling_text":"send Kim","targets":[],"source_revisions":{},"question_id":"q","task_id":"wrong"}]}),&json!({"actually_displayed_questions":[{"id":"q","task_id":"right"}]}),&json!([])).is_err());}#[test]fn multiple_intents_permitted(){let i=json!({"relation":"NEW","scope":"s","handling_text":"Explain TCP and UDP","targets":["TCP","UDP"],"source_revisions":{}});assert!(validate(&json!({"intents":[i.clone(),i]}),&json!({}),&json!([])).is_ok());}}
