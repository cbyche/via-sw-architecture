//! Common handoff source authority and receiver evidence backing; no expected answer data.
use crate::context::Context;use serde_json::{json,Value};use std::collections::BTreeMap;
pub type Sources=BTreeMap<String,Value>;
pub fn array(v:&Value)->Vec<Value>{v.as_array().cloned().unwrap_or_default()}
pub fn bounds(s:&Value)->(u64,u64){(s["allowed_lines"][0].as_u64().unwrap_or(1),s["allowed_lines"][1].as_u64().unwrap_or(s["line_count"].as_u64().unwrap_or(0)))}
pub fn catalog(s:&Sources)->Vec<Value>{s.values().map(|v|json!({"id":v["id"],"title":v["title"],"revision":v["revision"],"outline":v["outline"],"line_count":v["line_count"],"revoked":v["revoked"],"allowed_lines":v["allowed_lines"]})).collect()}
pub fn valid_span(p:&Value,sources:&Sources)->bool{let Some(s)=p["source_id"].as_str().and_then(|id|sources.get(id))else{return false;};let(Some(lo),Some(hi))=(p["start_line"].as_u64(),p["end_line"].as_u64())else{return false;};let(a,b)=bounds(s);s["revoked"].as_bool()!=Some(true)&&s["revision"]==p["revision"]&&lo>=a&&lo<=hi&&hi<=b}
pub fn contained(span:&Value,p:&Value)->bool{span["source_id"]==p["source_id"]&&span["revision"]==p["revision"]&&p["start_line"].as_u64().unwrap_or(u64::MAX)<=span["start_line"].as_u64().unwrap_or(0)&&p["end_line"].as_u64().unwrap_or(0)>=span["end_line"].as_u64().unwrap_or(u64::MAX)}
pub fn quote_in_span(span:&Value,p:&Value,quote:&str)->bool{
 if !contained(span,p){return false;}let Some(text)=p["text"].as_str()else{return false;};
 if p["kind"]=="ACTUALLY_DELIVERED_FACT"{return text.contains(quote);}
 let base=p["start_line"].as_u64().unwrap_or(1);let lo=span["start_line"].as_u64().unwrap_or(0);let hi=span["end_line"].as_u64().unwrap_or(0);
 if lo<base||hi<lo{return false;}text.lines().skip((lo-base)as usize).take((hi-lo+1)as usize).collect::<Vec<_>>().join("\n").contains(quote)
}
pub fn read(ctx:&Context,s:&Value,lo:u64,hi:u64,consumer:&str)->Result<Value,String>{
 let(a,b)=bounds(s);if s["revoked"].as_bool()==Some(true)||lo<a||hi>b||lo==0||lo>hi{return Err("DENIED_CURRENT_ACCESS_OR_RANGE".into());}
 let raw=ctx.source(s)?;let lines:Vec<&str>=raw["text"].as_str().ok_or("source text")?.lines().collect();if hi as usize>lines.len(){return Err("SOURCE_RANGE_UNAVAILABLE".into());}
 let p=json!({"source_id":s["id"],"revision":s["revision"],"start_line":lo,"end_line":hi,"unit":"one_based_line_inclusive","text":lines[lo as usize-1..hi as usize].join("\n"),"consumer":consumer});ctx.event("HANDOFF_SOURCE_READ",&p);Ok(p)
}
pub fn selected_reads(ctx:&Context,requests:&Value,sources:&Sources,consumer:&str)->(Vec<Value>,Vec<Value>){let mut passages=Vec::new();let mut denied=Vec::new();for r in array(requests).into_iter().take(8){let sid=r["source_id"].as_str().unwrap_or("");let result=if let Some(s)=sources.get(sid){read(ctx,s,r["start_line"].as_u64().unwrap_or(0),r["end_line"].as_u64().unwrap_or(0),consumer)}else{Err("NO_MATCH".into())};match result{Ok(p)=>passages.push(p),Err(reason)=>denied.push(json!({"request":r,"reason":reason}))}}(passages,denied)}
pub fn valid_block(b:&Value,s:&Sources)->bool{let spans=array(&b["source_spans"]);!spans.is_empty()&&spans.iter().all(|p|valid_span(p,s))&&array(&b["supporting_passages"]).iter().all(|p|valid_span(p,s))}
pub fn capsule_evidence(c:&Value)->Vec<Value>{let mut p=array(&c["passages"]);for b in array(&c["record_blocks"]){p.extend(array(&b["supporting_passages"]));}p}
pub fn capsule_valid(c:&Value,s:&Sources)->bool{if c.is_null(){return true;}let refs=array(&c["references"]);let p=array(&c["passages"]);let b=array(&c["record_blocks"]);(!refs.is_empty()||!p.is_empty()||!b.is_empty())&&refs.iter().chain(p.iter()).all(|p|valid_span(p,s))&&b.iter().all(|b|valid_block(b,s))}
pub fn result_errors(r:&Value,utterance:&str,s:&Sources,available:&[Value],required:&Value)->Vec<String>{
 let text=r["text"].as_str().unwrap_or("");let norm=|v:&str|v.split_whitespace().collect::<String>().to_lowercase();let facts=array(&r["facts"]);let used=array(&r["used_source_ids"]);let spans=array(&r["source_spans"]);let mut e=Vec::new();
 if text.trim().is_empty()||text.chars().count()>3500||norm(text)==norm(utterance){e.push("empty_oversized_or_request_echo".into());}if facts.is_empty()||facts.len()>14||used.is_empty()||spans.is_empty(){e.push("missing_structured_facts_or_provenance".into());}
 for f in &facts{let statement=f["statement"].as_str().unwrap_or("");let quote=f["quote"].as_str().unwrap_or("");if statement.trim().is_empty()||quote.trim().is_empty()||!valid_span(f,s)||!available.iter().any(|p|quote_in_span(f,p,quote)){e.push("fact_claim_or_actual_current_evidence_missing".into());}}
 if !used.iter().chain(array(required).iter()).all(|id|facts.iter().any(|f|f["source_id"]==*id)){e.push("declared_or_required_source_without_actual_fact".into());}
 if !spans.iter().all(|p|valid_span(p,s)&&available.iter().any(|given|contained(p,given))){e.push("result_span_not_actually_supplied_or_current".into());}e
}
#[cfg(test)]mod tests{use super::*;
 fn setup()->(Sources,Value,Value){let mut s=Sources::new();s.insert("s".into(),json!({"id":"s","revision":2,"line_count":5,"allowed_lines":[1,3]}));let p=json!({"source_id":"s","revision":2,"start_line":1,"end_line":2,"text":"Amount is 750. Shipping is pending."});let r=json!({"text":"Amount is 750.","used_source_ids":["s"],"source_spans":[{"source_id":"s","revision":2,"start_line":1,"end_line":1}],"facts":[{"statement":"Amount is 750.","quote":"Amount is 750.","source_id":"s","revision":2,"start_line":1,"end_line":1}]});(s,p,r)}
 #[test]fn valid_refs_do_not_require_raw_via_read(){let(s,_,_)=setup();assert!(capsule_valid(&json!({"references":[{"source_id":"s","revision":2,"start_line":1,"end_line":2}]}),&s));}
 #[test]fn manifest_only_cannot_ground_completion(){let(s,_,r)=setup();assert!(!result_errors(&r,"make draft",&s,&[],&json!(["s"])).is_empty());}
 #[test]fn valid_two_character_quote_accepted(){let(s,mut p,mut r)=setup();p["text"]=json!("Idle power is 8W.");r["text"]=json!("Observed idle power is 8W.");r["facts"][0]["statement"]=json!("Idle power is 8W.");r["facts"][0]["quote"]=json!("8W");assert!(result_errors(&r,"draft",&s,&[p],&json!(["s"])).is_empty());}
 #[test]fn empty_quote_rejected(){let(s,p,mut r)=setup();r["facts"][0]["quote"]=json!("");assert!(!result_errors(&r,"draft",&s,&[p],&json!(["s"])).is_empty());}
 #[test]fn actual_passages_allow_completion(){let(s,p,r)=setup();assert!(result_errors(&r,"make draft",&s,&[p],&json!(["s"])).is_empty());}
 #[test]fn revoked_and_stale_capsules_rejected(){let(mut s,p,_)=setup();s.get_mut("s").unwrap()["revoked"]=json!(true);assert!(!capsule_valid(&json!({"passages":[p]}),&s));let(s,mut p,_)=setup();p["revision"]=json!(1);assert!(!capsule_valid(&json!({"references":[p]}),&s));}
 #[test]fn paraphrased_display_does_not_duplicate_claim(){let(s,p,mut r)=setup();r["text"]=json!("The amount totals seven hundred fifty.");assert!(result_errors(&r,"make draft",&s,&[p],&json!(["s"])).is_empty());}
 #[test]fn quote_must_be_actually_supplied(){let(s,p,mut r)=setup();r["facts"][0]["quote"]=json!("fabricated original");assert!(!result_errors(&r,"make draft",&s,&[p],&json!(["s"])).is_empty());}
 #[test]fn quote_must_be_inside_cited_lines(){let(s,mut p,mut r)=setup();p["text"]=json!("Amount is 750.\nShipping is pending.");r["facts"][0]["quote"]=json!("Shipping is pending.");assert!(!result_errors(&r,"draft",&s,&[p],&json!(["s"])).is_empty());}
}
