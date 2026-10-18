use super::{producers, geometry};
use crate::context::Context;
use serde_json::{json, Value};
const BINDER:&str="You are the Multimodal Binding Resolver. Combine Speech Reference Producer expressions and Screen Target Producer candidates with prior accepted bindings. You do NOT have pixels and may NOT create new targets or extents. Output final targets only from candidate target_id values. Preserve source revisions and use expression timing/cue relations to distinguish historical pointing from later page navigation. If candidates lack a required target/extent, return need_expansion {producer:'screen',scope:string,expression_ids:[...]}; the screen owner will read its original pixels. You may reason semantically about existing candidates. Return ONLY compact JSON {adoptions:[{expression_id,targets:[existing candidate IDs],status:'ready'|'ambiguous'|'need_evidence',evidence:string}],need_expansion:false|{producer,scope,expression_ids}}. Preserve speech count exactly; never adopt six targets when three were requested. Evidence max100 characters, no reasoning_summary or chain-of-thought. Screen geometry_issues are actual consistency failures, not harmless hints. For every expression of the current speech revision produce a result. Do not convert navigation into designation merely by temporal proximity. In a correction retain or replace bindings according to speech meaning; never blindly keep old targets.";
fn complete(v: &Value) -> bool {
    v["complete"].as_bool() == Some(true) && v["unknown"].as_array().is_some_and(|a| a.is_empty())
}
fn code_bind(screen: &Value, speech: &Value) -> Option<Value> {
    if screen["geometry_issues"].as_array().is_none_or(|x| !x.is_empty())
        || !complete(&screen["coverage"])
        || !complete(&speech["coverage"])
        || screen["coverage"]["competing"]
            .as_array()
            .is_none_or(|a| !a.is_empty())
    {
        return None;
    }
    let refs = speech["expressions"].as_array()?;
    let candidates = screen["candidates"].as_array()?;
    let relations = screen["cue_relations"].as_array()?;
    let mut out = Vec::new();
    for r in refs {
        if r["unresolved"].as_array().is_none_or(|a| !a.is_empty()) {
            return None;
        }
        let targets = if r["relation"] == "explicit_ids" {
            r["explicit_target_ids"].as_array()?.clone()
        } else if r["relation"] == "designated_by_cue" || r["relation"] == "within_region" {
            let cues = r["cue_ids"].as_array()?;
            if cues.len() != 1 {
                return None;
            }
            let matching: Vec<&Value> = relations
                .iter()
                .filter(|g| {
                    g["cue_id"] == cues[0]
                        && g["complete"].as_bool() == Some(true)
                        && g["unique"].as_bool() == Some(true)
                        && g["navigation_alternative"].as_bool() == Some(false)
                })
                .collect();
            if matching.len() != 1 {
                return None;
            }
            matching[0]["target_ids"].as_array()?.clone()
        } else {
            return None;
        };
        if targets.is_empty()
            || targets
                .iter()
                .any(|t| !candidates.iter().any(|c| c["target_id"] == *t))
        {
            return None;
        }
        if let Some(count) = r["count"].as_u64() {
            if targets.len() != count as usize {
                return None;
            }
        }
        out.push(json!({"expression_id":r["expression_id"],"targets":targets,"status":"ready","evidence":"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"}));
    }
    Some(json!({"adoptions":out,"need_expansion":false,"resolver":"typed_code"}))
}
pub fn resolve(
    obs: &Value,
    prior: &Value,
    mut screen: Value,
    speech: Value,
    ctx: &mut Context,
) -> Result<Value, String> {
    let mut proposal = propose(prior,&screen,&speech,ctx)?;
    let mut issues=geometry::final_issues(obs,&proposal,Some(&screen),Some(&speech));
    if proposal["need_expansion"].is_object() || !issues.is_empty() {
        let request=json!({"binding_request":proposal["need_expansion"],"consistency_errors":issues,"repair_feedback":geometry::repair_feedback(&issues),"speech":speech,"previous_screen":screen,"producer_peer_revision":speech["revision"]});
        ctx.event("PRODUCER_EXPANSION_REQUEST",&request);
        if screen["producer_generation"].as_u64().unwrap_or(1) < 2 {
            screen=producers::screen(obs,&request,2,ctx)?;
            proposal=propose(prior,&screen,&speech,ctx)?;
            issues=geometry::final_issues(obs,&proposal,Some(&screen),Some(&speech));
        }
        if proposal["need_expansion"].is_object() {issues.push("EXPANSION_BUDGET_EXHAUSTED".into());}
        if !issues.is_empty() {
            ctx.event("GROUNDING_CONSISTENCY_REJECTED",&json!({"owner":"candidate_binding","issues":issues,"proposal":proposal}));
            proposal=geometry::unresolved(obs,&issues);
        }
    }
    proposal["intent"]=speech["intent"].clone();
    // Materialize final extents by exact candidate reference, not by target-oracle matching.
    if let Some(adoptions)=proposal["adoptions"].as_array_mut() {
        for a in adoptions {
            if let Some(semantic)=speech["expressions"].as_array().and_then(|xs|xs.iter().find(|x|x["expression_id"]==a["expression_id"])) {
                for field in ["count","cue_ids","relation"] {a[field]=semantic[field].clone();}
            }
            let ids=a["targets"].as_array().cloned().unwrap_or_default();
            a["extents"]=json!(screen["candidates"].as_array().into_iter().flatten().filter(|c|ids.contains(&c["target_id"])).collect::<Vec<_>>());
        }
    }
    // The binder cannot bypass candidate ownership by hallucinating raw-derived target IDs.
    let ids: Vec<Value> = screen["candidates"]
        .as_array()
        .map(|v| v.iter().map(|c| c["target_id"].clone()).collect())
        .unwrap_or_default();
    if let Some(adoptions) = proposal["adoptions"].as_array_mut() {
        for a in adoptions {
            if a["targets"]
                .as_array()
                .is_some_and(|ts| ts.iter().any(|t| !ids.contains(t)))
            {
                ctx.event("BINDER_INVALID_TARGET_REJECTED", a);
                a["targets"] = json!([]);
                a["status"] = json!("need_evidence");
            }
        }
    }
    proposal["candidate_read_set"] = json!({"screen_generation":screen["producer_generation"],"screen_source_snapshot":screen["source_snapshot"],"speech_revision":speech["revision"]});
    proposal["screen_state"] = screen;
    proposal["speech_state"] = speech;
    Ok(proposal)
}
fn propose(prior:&Value,screen:&Value,speech:&Value,ctx:&mut Context)->Result<Value,String>{
    if let Some(p)=code_bind(screen,speech){ctx.event("TYPED_CODE_BIND",&p);Ok(p)}else{model_bind(prior,screen,speech,ctx)}
}
fn model_bind(
    prior: &Value,
    screen: &Value,
    speech: &Value,
    ctx: &mut Context,
) -> Result<Value, String> {
    ctx.call("dp04.candidate_binding",&json!({"instruction":format!("{} {}",BINDER,geometry::RELATION_CONTRACT),"input":{"screen":screen,"speech":speech,"prior":prior},"max_tokens":2000}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_binding_requires_completeness_and_unique_non_navigation_relation() {
        let mut screen = json!({"geometry_issues":[],"coverage":{"complete":true,"unknown":[],"competing":[]},"candidates":[{"target_id":"visible-item"}],"cue_relations":[{"cue_id":"p","target_ids":["visible-item"],"complete":true,"unique":true,"navigation_alternative":false}]});
        let speech = json!({"coverage":{"complete":true,"unknown":[]},"expressions":[{"expression_id":"e","unresolved":[],"relation":"designated_by_cue","cue_ids":["p"],"count":1}]});
        assert!(code_bind(&screen, &speech).is_some());
        screen["cue_relations"][0]["navigation_alternative"] = json!(true);
        assert!(code_bind(&screen, &speech).is_none());
        screen["cue_relations"][0]["navigation_alternative"] = json!(false);
        screen["coverage"]["unknown"] = json!(["uncovered"]);
        assert!(code_bind(&screen, &speech).is_none());
    }
}
