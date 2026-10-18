//! Shared current-source acquisition and mechanical source-backing checks. No oracle.
use crate::context::Context;
use serde_json::{json,Value};
use std::collections::BTreeMap;
pub type Sources=BTreeMap<String,Value>;
pub fn array(v:&Value)->Vec<Value>{v.as_array().cloned().unwrap_or_default()}
pub fn normalized(v:&str)->String{v.chars().filter(|c|!c.is_whitespace()&&!c.is_ascii_punctuation()).flat_map(char::to_lowercase).collect()}
pub fn catalog(sources:&Sources)->Vec<Value>{sources.values().map(|s|json!({"id":s["id"],"title":s["title"],"revision":s["revision"],"conversation":s["conversation"],"kind":s["kind"],"outline":s["outline"],"line_count":s["line_count"],"revoked":s["revoked"],"delivered_lines":s["delivered_lines"]})).collect()}
pub fn bounds(s:&Value)->(u64,u64){(s["delivered_lines"][0].as_u64().unwrap_or(1),s["delivered_lines"][1].as_u64().unwrap_or(s["line_count"].as_u64().unwrap_or(0)))}
pub fn valid_span(span:&Value,sources:&Sources)->bool{let Some(s)=span["source_id"].as_str().and_then(|id|sources.get(id))else{return false;};let(Some(lo),Some(hi))=(span["start_line"].as_u64(),span["end_line"].as_u64())else{return false;};let(a,b)=bounds(s);s["revoked"].as_bool()!=Some(true)&&s["revision"]==span["revision"]&&lo>=a&&lo<=hi&&hi<=b}
pub fn contained(span:&Value,p:&Value)->bool{span["source_id"]==p["source_id"]&&span["revision"]==p["revision"]&&p["start_line"].as_u64().unwrap_or(u64::MAX)<=span["start_line"].as_u64().unwrap_or(0)&&p["end_line"].as_u64().unwrap_or(0)>=span["end_line"].as_u64().unwrap_or(u64::MAX)}
pub fn quote_in_span(span:&Value,p:&Value,quote:&str)->bool{
 if !contained(span,p){return false;}let Some(text)=p["text"].as_str()else{return false;};
 if p["kind"]=="ACTUALLY_DELIVERED_ANSWER"{return text.contains(quote);}
 let base=p["start_line"].as_u64().unwrap_or(1);let lo=span["start_line"].as_u64().unwrap_or(0);let hi=span["end_line"].as_u64().unwrap_or(0);
 if lo<base||hi<lo{return false;}text.lines().skip((lo-base)as usize).take((hi-lo+1)as usize).collect::<Vec<_>>().join("\n").contains(quote)
}
pub fn read(ctx:&Context,s:&Value,lo:u64,hi:u64)->Result<Value,String>{
 if s["revoked"].as_bool()==Some(true){return Err("DENIED_REVOKED".into());}let(a,b)=bounds(s);let lo=lo.max(a);let hi=hi.min(b);if lo==0||lo>hi{return Err("DENIED_EMPTY_RANGE".into());}
 let raw=ctx.source(s)?;let lines:Vec<&str>=raw["text"].as_str().ok_or("source text missing")?.lines().collect();if hi as usize>lines.len(){return Err("SOURCE_RANGE_UNAVAILABLE".into());}
 let p=json!({"source_id":s["id"],"revision":s["revision"],"start_line":lo,"end_line":hi,"text":lines[lo as usize-1..hi as usize].join("\n"),"unit":"one_based_line_inclusive","actual_delivery_only":!s["delivered_lines"].is_null()});ctx.event("EVIDENCE_READ",&p);Ok(p)
}
pub fn read_selected(ctx:&Context,reads:&[Value],sources:&Sources,rolling:&[Value])->(Vec<Value>,Vec<Value>){let mut passages=Vec::new();let mut denied=Vec::new();
 for r in reads.iter().take(8){let Some(s)=r["source_id"].as_str().and_then(|id|sources.get(id))else{denied.push(json!({"request":r,"reason":"NO_MATCH"}));continue;};let(a,b)=bounds(s);let lo=r["start_line"].as_u64().unwrap_or(a).max(a);let hi=r["end_line"].as_u64().unwrap_or(b).min(b);let wanted=json!({"source_id":s["id"],"revision":s["revision"],"start_line":lo,"end_line":hi});
  if let Some(p)=rolling.iter().find(|p|p["text"].is_string()&&valid_span(p,sources)&&contained(&wanted,p)){ctx.event("ROLLING_EVIDENCE_REUSE",p);passages.push(p.clone());continue;}
  match read(ctx,s,lo,hi){Ok(p)=>passages.push(p),Err(reason)=>denied.push(json!({"request":r,"reason":reason}))}
 }(passages,denied)
}
pub fn targets_valid(targets:&Value,sources:&Sources)->bool{let t=array(targets);!t.is_empty()&&t.len()<=8&&t.iter().all(|v|v.as_str().and_then(|id|sources.get(id)).map(|s|s["revoked"].as_bool()!=Some(true)).unwrap_or(false))}
pub fn completion_errors(answer:&Value,utterance:&str,expression:&str,sources:&Sources,available:&[Value])->Vec<String>{
 let mut errors=Vec::new();let text=answer["text"].as_str().unwrap_or("");let facts=array(&answer["facts"]);let targets=array(&answer["targets"]);
 if text.trim().is_empty()||text.chars().count()>3000||normalized(text)==normalized(utterance)||(!expression.trim().is_empty()&&normalized(text)==normalized(expression)){errors.push("empty_oversized_or_request_echo".into());}if !targets_valid(&answer["targets"],sources){errors.push("invalid_or_missing_targets".into());}if facts.is_empty()||facts.len()>12{errors.push("missing_or_oversized_facts".into());}
 for f in &facts{let statement=f["statement"].as_str().unwrap_or("");let quote=f["quote"].as_str().unwrap_or("");if statement.trim().is_empty()||quote.trim().is_empty()||!valid_span(f,sources)||!available.iter().any(|p|quote_in_span(f,p,quote)){errors.push("fact_claim_or_supplied_current_span_quote_missing".into());}}
 for target in targets{if !facts.iter().any(|f|f["source_id"]==target){errors.push("target_has_no_grounded_fact".into());}if !array(&answer["source_revisions"]).iter().any(|p|p["source_id"]==target&&target.as_str().and_then(|id|sources.get(id)).map(|s|s["revision"]==p["revision"]).unwrap_or(false)){errors.push("missing_current_source_revision".into());}}
 errors
}
#[cfg(test)]mod tests{
 use super::*;
 fn setup()->(Sources,Value,Vec<Value>){let mut s=Sources::new();s.insert("source".into(),json!({"id":"source","revision":2,"line_count":3,"delivered_lines":[1,2]}));let p=json!({"source_id":"source","revision":2,"start_line":1,"end_line":2,"text":"Current amount is 500. Warranty is 2 years."});let a=json!({"targets":["source"],"text":"Current amount is 500.","source_revisions":[{"source_id":"source","revision":2}],"facts":[{"source_id":"source","revision":2,"start_line":1,"end_line":1,"quote":"amount is 500","statement":"Current amount is 500."}]});(s,a,vec![p])}
 #[test]fn valid_two_character_quote_accepted(){let(s,mut a,mut p)=setup();p[0]["text"]=json!("Idle power is 8W.");a["text"]=json!("Observed idle power is 8W.");a["facts"][0]["statement"]=json!("Idle power is 8W.");a["facts"][0]["quote"]=json!("8W");assert!(completion_errors(&a,"power?","the source",&s,&p).is_empty());}
 #[test]fn empty_quote_rejected(){let(s,mut a,p)=setup();a["facts"][0]["quote"]=json!("  ");assert!(!completion_errors(&a,"amount?","the source",&s,&p).is_empty());}
 #[test]fn supplied_current_fact_accepted(){let(s,a,p)=setup();assert!(completion_errors(&a,"amount?","the source",&s,&p).is_empty());}
 #[test]fn metadata_without_body_rejected(){let(s,a,_)=setup();assert!(!completion_errors(&a,"amount?","the source",&s,&[]).is_empty());}
 #[test]fn version_declaration_not_content(){let(s,mut a,p)=setup();a["facts"]=json!([]);assert!(!completion_errors(&a,"amount?","the source",&s,&p).is_empty());}
 #[test]fn revoked_stale_undelivered_rejected(){let(mut s,a,p)=setup();s.get_mut("source").unwrap()["revoked"]=json!(true);assert!(!completion_errors(&a,"amount?","the source",&s,&p).is_empty());let(s,mut a,p)=setup();a["facts"][0]["revision"]=json!(1);assert!(!completion_errors(&a,"amount?","the source",&s,&p).is_empty());a["facts"][0]["revision"]=json!(2);a["facts"][0]["end_line"]=json!(3);assert!(!completion_errors(&a,"amount?","the source",&s,&p).is_empty());}
 #[test]fn paraphrased_display_is_not_forced_to_duplicate_claim(){let(s,mut a,p)=setup();a["text"]=json!("The current amount totals five hundred.");assert!(completion_errors(&a,"amount?","the source",&s,&p).is_empty());}
 #[test]fn expression_only_echo_rejected(){let(s,mut a,p)=setup();a["text"]=json!("the source");assert!(!completion_errors(&a,"Please summarize the source.","the source",&s,&p).is_empty());}
 #[test]fn request_echo_rejected(){let(s,mut a,p)=setup();a["text"]=json!("What is the amount?");assert!(!completion_errors(&a,"What is the amount?","the source",&s,&p).is_empty());}
 #[test]fn quote_must_be_in_cited_lines_not_just_same_document(){let(s,mut a,mut p)=setup();p[0]["text"]=json!("Current amount is 500.\nWarranty is 2 years.");a["facts"][0]["quote"]=json!("Warranty is 2 years.");assert!(!completion_errors(&a,"amount?","the source",&s,&p).is_empty());}
}
