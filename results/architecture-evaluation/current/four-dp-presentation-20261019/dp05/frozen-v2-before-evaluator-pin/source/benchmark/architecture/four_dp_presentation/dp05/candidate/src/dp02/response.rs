//! Bounded detailed response creation and source/shape checks; no evaluator oracle is imported.
use serde_json::{json,Value};
use crate::context::Context;
use super::state::{self,Shared,Request};
fn source_claims(v:&Value,sources:&Value)->Result<(),String>{
    if let Some(claims)=v["source_claims"].as_array(){for c in claims{
        let src=sources.as_array().and_then(|a|a.iter().find(|s|s["id"]==c["source_id"] && s["revision"]==c["revision"])).ok_or_else(||format!("source claim ID/revision unavailable: source_id={} revision={}; only input.sources IDs are valid, user events are not source IDs",c["source_id"],c["revision"]))?;
        let quote=c["quote"].as_str().filter(|s|!s.is_empty()).ok_or("source quote missing")?;
        if !src["text"].as_str().unwrap_or("").contains(quote){return Err(format!("quote is not an exact contiguous substring: source_id={} revision={} quote={}; do not use ellipses or join separated text",c["source_id"],c["revision"],c["quote"]))}
    }}Ok(())
}
pub fn validate(v:&Value,r:&Request,sources:&Value)->Result<(),String>{
    let answer=v["answer"].as_str().filter(|s|!s.trim().is_empty()).ok_or("answer null/empty")?;
    if answer.trim()==r.raw.trim(){return Err("raw request echo is not an answer".into())}
    if v["applied_revision"].as_u64()!=Some(r.revision){return Err("response revision is stale/missing".into())}
    let sections=v["sections"].as_array().filter(|a|!a.is_empty()).ok_or("detailed sections missing")?;
    for s in sections{if s["id"].as_str().is_none() || s["text"].as_str().filter(|s|!s.trim().is_empty()).is_none(){return Err("response section malformed".into())}}
    let adopted_sources=r.adoption["source_revisions"].as_object().or_else(||r.correction.as_ref().and_then(|c|c["adopted_intent"]["source_revisions"].as_object()));
    if adopted_sources.is_some_and(|s|!s.is_empty()) && !v["source_claims"].as_array().is_some_and(|a|!a.is_empty()){return Err("source-backed response requires actual supplied quote claims".into())}
    source_claims(v,sources)
}
pub fn detailed(ctx:&mut Context,input:&Value,request:&Request,state:&Shared)->Result<Value,String>{
    let mut reason=Value::Null;let mut previous=Value::Null;
    for attempt in 0..=1{
        let p=json!({"instruction":"Complete this VIA user goal from its actual original request, adopted intent and latest revision/correction. Preserve all non-corrected conditions; obey the latest scope. Return JSON {answer:nonempty detailed string,sections:[{id:string,text:nonempty string}],applied_revision:number,source_claims:[{source_id,revision,quote:exact supplied substring}]}. Cover the adopted goal, not an unrelated pending goal. Include the requested comparison values/tradeoffs or protocol explanation in the answer itself. Never return null, echo the instruction, or replace the answer with a plan. source_claims.quote must be a single EXACT CONTIGUOUS substring from the supplied sources text: never insert ellipses, summarize a quote, join separated passages, or cite a user event as a source ID. Use separate claims for separate actual passages. For an answer to an Agent question, identify the resolved recipient/choice for actual forwarding; do not claim it was already sent.","input":{"original_request":request.raw,"goal_id":request.goal_id,"current_revision":request.revision,"current_scope":request.scope,"adopted_intent":request.adoption,"later_correction":request.correction,"sources":input["sources"],"actual_state":state::snapshot(state),"repair_reason":reason,"previous_rejected_proposal":previous},"max_tokens":2600});
        match ctx.call(if attempt==0{"dp02_complete_response"}else{"dp02_repair_response"},&p){Ok(v)=>match validate(&v,request,&input["sources"]){Ok(())=>return Ok(v),Err(e)=>{reason=json!(e);previous=v.clone();ctx.event("SEMANTIC_STAGE_REJECTED",&json!({"stage":"response","attempt":attempt,"reason":e,"proposal":v,"goal_id":request.goal_id}));}},Err(e)=>{reason=json!(e);ctx.event("SEMANTIC_STAGE_REJECTED",&json!({"stage":"response","attempt":attempt,"reason":e,"goal_id":request.goal_id}));}}
    }Err(format!("response recovery exhausted: {}",reason))
}
pub fn fast_answer(intent:&Value,r:&Request,sources:&Value)->Option<Value>{
    let text=intent["complete_answer"].as_str().filter(|s|!s.trim().is_empty())?;
    let v=json!({"answer":text,"sections":[{"id":"main","text":text}],"applied_revision":r.revision,"source_claims":intent["source_claims"].as_array().cloned().unwrap_or_default(),"path":"VALID_COMPLETE_ANSWER_FROM_ADOPTION"});
    validate(&v,r,sources).ok().map(|_|v)
}
#[cfg(test)]mod tests{use super::*;fn r()->Request{Request{goal_id:"r".into(),stimulus_id:"r".into(),raw:"compare A and B".into(),revision:2,scope:"cost".into(),adoption:json!({}),correction:None,delivered:None}}#[test]fn null_is_failed_attempt(){assert!(validate(&json!({"answer":null,"applied_revision":2,"sections":[]}),&r(),&json!([])).is_err());}#[test]fn stale_result_rejected(){assert!(validate(&json!({"answer":"A costs120 B75","sections":[{"id":"cost","text":"A120 B75"}],"applied_revision":1}),&r(),&json!([])).is_err());}#[test]fn wrong_nonempty_answer_retains_terminal_shape(){assert!(validate(&json!({"answer":"A costs999 B998","sections":[{"id":"cost","text":"A999 B998"}],"applied_revision":2}),&r(),&json!([])).is_ok());}}
