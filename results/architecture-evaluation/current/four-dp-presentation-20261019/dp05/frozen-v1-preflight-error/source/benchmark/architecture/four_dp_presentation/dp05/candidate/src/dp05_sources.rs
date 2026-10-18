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
/// Only call with the host `selected_reads` return, never model/capsule/record text.
/// Reading an original is acquisition even when later composition or delegation fails.
fn merge_via_original_reads(common:&mut Vec<Value>,actual_reads:&[Value],sources:&Sources)->Value{
 let before=common.len();common.retain(|p|valid_span(p,sources));let mut acquired=Vec::new();let mut rejected=Vec::new();
 for p in actual_reads{
  let host_consumer=matches!(p["consumer"].as_str(),Some("VIA_COMPOSER"|"COMMON_ACTUALLY_ACQUIRED"));
  let expected_lines=p["start_line"].as_u64().and_then(|lo|p["end_line"].as_u64().and_then(|hi|hi.checked_sub(lo).and_then(|n|n.checked_add(1))));
  let raw_shape=p["unit"]=="one_based_line_inclusive"&&p.get("kind").is_none()&&p["text"].as_str().map(|text|Some(text.split('\n').count()as u64)==expected_lines).unwrap_or(false);
  if !host_consumer||!raw_shape||!valid_span(p,sources){rejected.push(json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"],"reason":"NOT_CURRENT_HOST_VIA_ORIGINAL_READ"}));continue;}
  // Exact observed-range dedup only; do not invent merged/unread lines or semantic records.
  common.retain(|old|!(old["source_id"]==p["source_id"]&&old["revision"]==p["revision"]&&old["start_line"]==p["start_line"]&&old["end_line"]==p["end_line"]));
  common.push(p.clone());acquired.push(json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"]}));
 }
 json!({"before_passages":before,"after_passages":common.len(),"actual_via_read_spans":acquired,"rejected":rejected,"scope":"current_authorized_originals_only","semantic_record_or_capsule_cache":false})
}
pub fn retain_via_original_reads(ctx:&Context,common:&mut Vec<Value>,actual_reads:&[Value],sources:&Sources){
 let report=merge_via_original_reads(common,actual_reads,sources);ctx.event("COMMON_ORIGINAL_EVIDENCE_RETAINED",&report);
 ctx.logical_buffer("dp05_common_acquired",serde_json::to_vec(common).unwrap().len(),"STATE");
}
/// Temporary prefix view of actual VIA-owned originals; no derived explanation or new cache.
pub fn prefix_original_evidence(common:&[Value],record:&Value,sources:&Sources)->Vec<Value>{
 let mut candidates=common.to_vec();for block in array(&record["blocks"]).into_iter().filter(|b|valid_block(b,sources)){candidates.extend(array(&block["supporting_passages"]));}
 let mut output:Vec<Value>=Vec::new();for p in candidates{
  if !valid_span(&p,sources){continue;}let Some(text)=p["text"].as_str()else{continue;};let lo=p["start_line"].as_u64().unwrap();let hi=p["end_line"].as_u64().unwrap();if text.split('\n').count()as u64!=hi-lo+1{continue;}
  if output.iter().any(|old|old["source_id"]==p["source_id"]&&old["revision"]==p["revision"]&&old["start_line"]==p["start_line"]&&old["end_line"]==p["end_line"]){continue;}
  output.push(json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":lo,"end_line":hi,"unit":"one_based_line_inclusive","text":text}));
 }output
}
pub fn valid_block(b:&Value,s:&Sources)->bool{let spans=array(&b["source_spans"]);!spans.is_empty()&&spans.iter().all(|p|valid_span(p,s))&&array(&b["supporting_passages"]).iter().all(|p|valid_span(p,s))}
pub fn capsule_evidence(c:&Value)->Vec<Value>{let mut p=array(&c["passages"]);for b in array(&c["record_blocks"]){p.extend(array(&b["supporting_passages"]));}p}
pub fn capsule_valid(c:&Value,s:&Sources)->bool{if c.is_null(){return true;}let refs=array(&c["references"]);let p=array(&c["passages"]);let b=array(&c["record_blocks"]);(!refs.is_empty()||!p.is_empty()||!b.is_empty())&&refs.iter().chain(p.iter()).all(|p|valid_span(p,s))&&b.iter().all(|b|valid_block(b,s))}
/// Aggregate result spans may cover a gapless union of actual supplied original ranges.
/// Existing span schemas use one-based inclusive lines when unit is omitted.
fn supplied_union_covers(span:&Value,sources:&Sources,available:&[Value])->bool{
 let unit=|p:&Value|p.get("unit").is_none()||p["unit"]=="one_based_line_inclusive";
 if !unit(span)||!valid_span(span,sources){return false;}
 let lo=span["start_line"].as_u64().unwrap();let hi=span["end_line"].as_u64().unwrap();
 let mut pieces:Vec<(u64,u64)>=available.iter().filter(|p|unit(p)&&p["source_id"]==span["source_id"]&&p["revision"]==span["revision"]&&valid_span(p,sources)).map(|p|(p["start_line"].as_u64().unwrap(),p["end_line"].as_u64().unwrap())).collect();pieces.sort_unstable();
 let mut cursor=lo;for(a,b)in pieces{if b<cursor{continue;}if a>cursor{return false;}if b>=hi{return true;}cursor=b+1;}false
}
pub fn result_errors(r:&Value,utterance:&str,s:&Sources,available:&[Value],required:&Value)->Vec<String>{
 let text=r["text"].as_str().unwrap_or("");let norm=|v:&str|v.split_whitespace().collect::<String>().to_lowercase();let facts=array(&r["facts"]);let used=array(&r["used_source_ids"]);let spans=array(&r["source_spans"]);let mut e=Vec::new();
 if text.trim().is_empty()||text.chars().count()>3500||norm(text)==norm(utterance){e.push("empty_oversized_or_request_echo".into());}if facts.is_empty()||facts.len()>14||used.is_empty()||spans.is_empty(){e.push("missing_structured_facts_or_provenance".into());}
 for f in &facts{let statement=f["statement"].as_str().unwrap_or("");let quote=f["quote"].as_str().unwrap_or("");if statement.trim().is_empty()||quote.trim().is_empty()||!valid_span(f,s)||!available.iter().any(|p|quote_in_span(f,p,quote)){e.push("fact_claim_or_actual_current_evidence_missing".into());}}
 if !used.iter().chain(array(required).iter()).all(|id|facts.iter().any(|f|f["source_id"]==*id)){e.push("declared_or_required_source_without_actual_fact".into());}
 if !spans.iter().all(|p|supplied_union_covers(p,s,available)){e.push("result_span_not_actually_supplied_or_current".into());}e
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

#[cfg(test)]mod retention_tests{use super::*;
 fn authority()->Sources{let mut s=Sources::new();s.insert("original".into(),json!({"id":"original","revision":2,"allowed_lines":[1,4]}));s}
 fn actual()->Value{json!({"source_id":"original","revision":2,"start_line":1,"end_line":2,"unit":"one_based_line_inclusive","consumer":"VIA_COMPOSER","text":"Observed first line.\nObserved second line."})}
 #[test]fn actual_original_retained_without_semantic_or_delivery_commit(){let mut state=Vec::new();let p=actual();let report=merge_via_original_reads(&mut state,&[p.clone()],&authority());assert_eq!(state,vec![p]);assert_eq!(report["after_passages"],1);assert_eq!(report["semantic_record_or_capsule_cache"],false);}
 #[test]fn exact_observed_range_deduplicated(){let mut state=vec![actual()];merge_via_original_reads(&mut state,&[actual(),actual()],&authority());assert_eq!(state.len(),1);}
 #[test]fn stale_and_revoked_originals_removed(){let mut state=vec![actual()];let mut s=authority();s.get_mut("original").unwrap()["revision"]=json!(3);merge_via_original_reads(&mut state,&[actual()],&s);assert!(state.is_empty());let mut s=authority();s.get_mut("original").unwrap()["revoked"]=json!(true);state.push(actual());merge_via_original_reads(&mut state,&[],&s);assert!(state.is_empty());}
 #[test]fn reduced_allowed_range_blocks_retained_original(){let mut state=vec![actual()];let mut s=authority();s.get_mut("original").unwrap()["allowed_lines"]=json!([2,4]);merge_via_original_reads(&mut state,&[],&s);assert!(state.is_empty());}
 #[test]fn receiver_record_and_model_derived_text_not_promoted(){for consumer in ["RECEIVER","VIA_RECORD_PRODUCER","MODEL"]{let mut p=actual();p["consumer"]=json!(consumer);let mut state=Vec::new();merge_via_original_reads(&mut state,&[p],&authority());assert!(state.is_empty());}let mut p=actual();p["kind"]=json!("ACTUALLY_DELIVERED_FACT");let mut state=Vec::new();merge_via_original_reads(&mut state,&[p],&authority());assert!(state.is_empty());}
 #[test]fn range_text_shape_mismatch_not_promoted(){let mut p=actual();p["end_line"]=json!(3);let mut state=Vec::new();merge_via_original_reads(&mut state,&[p],&authority());assert!(state.is_empty());}
 #[test]fn explicit_common_acquisition_uses_same_rule(){let mut p=actual();p["consumer"]=json!("COMMON_ACTUALLY_ACQUIRED");let mut state=Vec::new();merge_via_original_reads(&mut state,&[p.clone()],&authority());assert_eq!(state,vec![p]);}
 #[test]fn prefix_includes_owned_record_raw_support_but_no_derived_text(){let p=actual();let record=json!({"blocks":[{"explanation":"UNSEEN DERIVED SUMMARY","source_spans":[p],"supporting_passages":[p]}],"conditions":["UNSEEN DERIVED CONDITION"]});let common=Vec::new();let view=prefix_original_evidence(&common,&record,&authority());assert_eq!(view.len(),1);assert_eq!(view[0]["text"],actual()["text"]);assert!(!serde_json::to_string(&view).unwrap().contains("UNSEEN DERIVED"));assert!(common.is_empty());}
 #[test]fn prefix_deduplicates_common_and_record_original_without_persistence(){let p=actual();let common=vec![p.clone()];let record=json!({"blocks":[{"source_spans":[p],"supporting_passages":[p]}]});assert_eq!(prefix_original_evidence(&common,&record,&authority()).len(),1);assert_eq!(common,vec![actual()]);}
 #[test]fn prefix_excludes_stale_or_revoked_record_support(){let mut p=actual();p["revision"]=json!(1);let record=json!({"blocks":[{"source_spans":[p],"supporting_passages":[p]}]});assert!(prefix_original_evidence(&[],&record,&authority()).is_empty());let p=actual();let record=json!({"blocks":[{"source_spans":[p],"supporting_passages":[p]}]});let mut s=authority();s.get_mut("original").unwrap()["revoked"]=json!(true);assert!(prefix_original_evidence(&[actual()],&record,&s).is_empty());}
}

#[cfg(test)]mod aggregate_union_tests{use super::*;
 fn authority()->Sources{let mut s=Sources::new();s.insert("s".into(),json!({"id":"s","revision":1,"allowed_lines":[1,8]}));s}
 fn span(lo:u64,hi:u64)->Value{json!({"source_id":"s","revision":1,"start_line":lo,"end_line":hi})}
 #[test]fn adjacent_actual_ranges_cover_aggregate(){assert!(supplied_union_covers(&span(2,4),&authority(),&[span(2,3),span(4,4)]));}
 #[test]fn overlapping_unordered_ranges_cover_aggregate(){assert!(supplied_union_covers(&span(2,5),&authority(),&[span(4,5),span(2,4)]));}
 #[test]fn gap_is_not_supplied_coverage(){assert!(!supplied_union_covers(&span(2,4),&authority(),&[span(2,2),span(4,4)]));}
 #[test]fn source_revision_or_unit_mix_cannot_bridge_gap(){for field in ["source_id","revision","unit"]{let mut bad=span(4,4);bad[field]=match field{"source_id"=>json!("other"),"revision"=>json!(2),_=>json!("byte_offset")};assert!(!supplied_union_covers(&span(2,4),&authority(),&[span(2,3),bad]));}}
 #[test]fn revoked_source_coverage_rejected(){let mut s=authority();s.get_mut("s").unwrap()["revoked"]=json!(true);assert!(!supplied_union_covers(&span(2,4),&s,&[span(2,3),span(4,4)]));}
 #[test]fn outside_current_permission_rejected(){let mut s=authority();s.get_mut("s").unwrap()["allowed_lines"]=json!([1,3]);assert!(!supplied_union_covers(&span(2,4),&s,&[span(2,3),span(4,4)]));}
 #[test]fn aggregate_wrong_unit_rejected(){for value in [json!("byte_offset"),json!(17),json!(true),Value::Null]{let mut p=span(2,4);p["unit"]=value.clone();assert!(!supplied_union_covers(&p,&authority(),&[span(2,4)]));let mut bad=span(2,4);bad["unit"]=value;assert!(!supplied_union_covers(&span(2,4),&authority(),&[bad]));}}
 #[test]fn actual_fact_quotes_still_required(){let s=authority();let mut p=span(2,3);p["text"]=json!("first actual line\nsecond actual line");let mut q=span(4,4);q["text"]=json!("third actual line");let mut fact=span(2,2);fact["statement"]=json!("A factual paraphrase.");fact["quote"]=json!("first actual line");let mut result=json!({"text":"A bounded original summary.","used_source_ids":["s"],"source_spans":[span(2,4)],"facts":[fact]});assert!(result_errors(&result,"request",&s,&[p.clone(),q.clone()],&json!(["s"])).is_empty());result["facts"][0]["quote"]=json!("fabricated quote");assert!(result_errors(&result,"request",&s,&[p,q],&json!(["s"])).iter().any(|e|e=="fact_claim_or_actual_current_evidence_missing"));}
}
