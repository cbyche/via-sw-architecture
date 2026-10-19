"""Finite feature generators for independent copies of the final frozen source.

PREPARATION ONLY until FINAL-REBASE.json records the exact reviewed source.
"""
from pathlib import Path

def once(text,old,new):
 # Exact Rust tokens; whitespace/formatting only is ignored. No semantic/fuzzy rebase.
 from module_inventory import tokens
 spans=tokens(text,with_spans=True);needle=tokens(old);values=[v for v,_,_ in spans]
 starts=[i for i in range(len(values)-len(needle)+1)if values[i:i+len(needle)]==needle]
 if len(starts)!=1:raise RuntimeError('Feature anchor tokens changed; inspect formal baseline: '+old[:90])
 at=starts[0];a=spans[at][1];b=spans[at+len(needle)-1][2]
 return text[:a]+new+text[b:]

C05_HELPERS = r'''
fn qa04_actual_geometry(plan:&Value,grounded:&Value)->Value{
 let mut adopted=array(&plan["targets"]);for b in array(&plan["bindings"]){adopted.extend(array(&b["targets"]));}
 let mut evidence=Vec::new();for b in array(&grounded["bindings"]){for extent in array(&b["extents"]){let target=extent["canonical_target"].clone();if !target.is_string(){continue}if adopted.iter().any(|a|*a==target||a["canonical_target"]==target){evidence.push(extent);}}}json!(evidence)
}
fn qa04_readonly_state(state:&State)->Value{json!({"history":state.history,"captured":state.captured,"pending":state.pending,"requests":state.requests,"delivered":state.delivered,"tasks":state.tasks,"ingress":state.ingress,"counter":state.counter,"active":state.active,"ended":state.ended,"failures":state.failures})}
fn qa04_query_provenance(delivered:&[Value],sources:&SourceRegistry,event:&Value)->Value{
 let target=event["target_event_id"].as_str().unwrap_or("");let revision=event["response_revision"].as_u64();
 let actual=delivered.iter().rev().find(|r|r["phase"]!="HANDLING"&&[r["stimulus_id"].as_str(),r["event_id"].as_str(),r["id"].as_str()].contains(&Some(target))&&revision.is_none_or(|v|r["revision"]==v)&&r["text"].as_str().is_some_and(|t|!t.trim().is_empty()));
 let Some(actual)=actual else{return json!({"status":"NO_MATCH","target_event_id":target})};
 let mut spans=Vec::new();for span in array(&actual["source_spans"]){let id=span["source_id"].as_str().unwrap_or("");let revision=span["revision"].as_u64().unwrap_or(0);let current=sources.current(id);let status=if !crate::evidence::permitted(sources,&span,false){"NOT_AUTHORIZED_NOW"}else if current.is_some_and(|s|s["revision"]==revision){"CURRENT_AUTHORIZED"}else{"HISTORICAL_AUTHORIZED"};let mut span=span;span["current_use_status"]=json!(status);spans.push(span);}
 json!({"status":"DELIVERED","target_event_id":target,"response_revision":actual["revision"],"phase":actual["phase"],"source_spans":spans,"adopted_geometry":actual["adopted_geometry_provenance"],"delivery_source_id":actual["delivery_source_id"],"origin_provenance_available":!array(&actual["source_spans"]).is_empty(),"past_delivery_not_claimed_current":true})
}
#[cfg(test)]mod qa04_c05_tests{use super::*;
 #[test]fn query_only_actual_delivery_no_mutation_and_historical_status(){let sources=SourceRegistry::new();sources.register(json!({"id":"s","revision":1,"text":"actual original"})).unwrap();let mut actual=vec![json!({"stimulus_id":"shown","revision":1,"phase":"DIRECT","text":"Actually displayed body","source_spans":[{"source_id":"s","revision":1,"start_line":1,"end_line":1}],"adopted_geometry_provenance":[{"canonical_target":"s:v1:P1","actual_source_sha256":"observed-sha"}]})];let before=json!(actual);let event=json!({"target_event_id":"shown","response_revision":1});let p=qa04_query_provenance(&actual,&sources,&event);assert_eq!(p["status"],"DELIVERED");assert_eq!(p["source_spans"][0]["current_use_status"],"CURRENT_AUTHORIZED");assert_eq!(p,qa04_query_provenance(&actual,&sources,&event));assert_eq!(json!(actual),before);sources.register(json!({"id":"s","revision":2,"text":"updated original"})).unwrap();assert_eq!(qa04_query_provenance(&actual,&sources,&event)["source_spans"][0]["current_use_status"],"HISTORICAL_AUTHORIZED");actual.push(json!({"stimulus_id":"handling-only","revision":1,"phase":"HANDLING","text":"Concrete intention"}));for key in ["unshown-proposal","rejected","failed","handling-only"]{assert_eq!(qa04_query_provenance(&actual,&sources,&json!({"target_event_id":key}))["status"],"NO_MATCH");}}
 #[test]fn only_adopted_geometry_is_exposed(){let p=json!({"targets":["DOC:v1:P1"],"bindings":[]});let g=json!({"bindings":[{"extents":[{"canonical_target":"DOC:v1:P1","bbox":[1,2,3,4]},{"canonical_target":"DOC:v1:P2","bbox":[5,6,7,8]}]}]});let kept=qa04_actual_geometry(&p,&g);assert_eq!(kept.as_array().unwrap().len(),1);assert_eq!(kept[0]["canonical_target"],"DOC:v1:P1");}
}
'''

C06_SOURCE_HELPERS = r'''
fn qa04_subtract_ranges(ranges:&Value,a:u64,b:u64)->Value{let mut out=Vec::new();for r in ranges.as_array().into_iter().flatten(){let(Some(lo),Some(hi))=(r[0].as_u64(),r[1].as_u64())else{continue};if hi<a||lo>b{out.push(json!([lo,hi]));continue}if lo<a{out.push(json!([lo,a-1]))}if hi>b{out.push(json!([b+1,hi]))}}json!(out)}
#[cfg(test)]mod qa04_c06_source_tests{use super::*;
 #[test]fn old_versions_cannot_bypass_withdrawal_and_raw_fragments_survive(){let s=SourceRegistry::new();for v in [1,2]{s.register(json!({"id":"s","revision":v,"text":"left public\nWITHDRAWN PRIVATE FACT\nright public","segments":{"all":[1,3]}})).unwrap();}s.withdraw_range(&json!({"source_id":"s","revision":2,"start_line":2,"end_line":2,"access_epoch":2})).unwrap();for v in [1,2]{assert!(!s.allowed("s",v,2,2,false));assert!(!s.allowed("s",v,1,3,true));assert!(s.allowed("s",v,1,1,true));assert!(s.allowed("s",v,3,3,false));}assert_eq!(s.read("s",2,1,1,false).unwrap().text,"left public");assert_eq!(s.read("s",2,3,3,false).unwrap().text,"right public");s.register(json!({"id":"s","revision":2,"text":"left public\nWITHDRAWN PRIVATE FACT\nright public","segments":{"all":[1,3]}})).unwrap();s.permission(&json!({"source_id":"s","read_segments":["all"],"external_segments":["all"]})).unwrap();assert!(!s.allowed("s",2,2,2,false));assert_eq!(s.current("s").unwrap()["allowed_segments"],json!([[1,1],[3,3]]));}
}
'''

C04_HELPERS = r'''
fn qa04_sections(text:&str)->Value{let mut sections=Vec::new();let mut name="body".to_owned();let mut lines=Vec::new();for line in text.lines(){if let Some(heading)=line.strip_prefix("## "){if !lines.is_empty(){sections.push(json!({"name":name,"text":lines.join("\n")}));lines.clear();}name=heading.trim().to_owned();if name.is_empty(){name="body".into();}}else{lines.push(line.to_owned());}}if !lines.is_empty()||sections.is_empty(){sections.push(json!({"name":name,"text":lines.join("\n")}));}json!(sections)}
fn qa04_render_sections(sections:&Value)->String{let v=array(sections);if v.len()==1&&v[0]["name"]=="body"{return v[0]["text"].as_str().unwrap_or("").into()}v.iter().map(|s|format!("## {}\n{}",s["name"].as_str().unwrap_or("body"),s["text"].as_str().unwrap_or(""))).collect::<Vec<_>>().join("\n")}
fn qa04_section_edit(delivered:&mut[Value],event:&Value)->Result<Value,String>{let target=event["target_event_id"].as_str().ok_or("section target missing")?;let name=event["section"].as_str().ok_or("section name missing")?;let replacement=event["text"].as_str().filter(|t|!t.trim().is_empty()).ok_or("section replacement empty")?;let expected=event["expected_section_revision"].as_u64().ok_or("section revision missing")?;let current=delivered.iter_mut().rev().find(|r|r["phase"]=="AGENT_RESULT"&&[r["stimulus_id"].as_str(),r["event_id"].as_str(),r["id"].as_str()].contains(&Some(target))).ok_or("section target not actually displayed Agent result")?;if current["section_revision"].as_u64().unwrap_or(1)!=expected{return Err("STALE_SECTION_REVISION".into())}if event["task_id"].is_string()&&current["task_id"]!=event["task_id"]{return Err("SECTION_TASK_MISMATCH".into())}let before=current.clone();let mut sections=if current["sections"].is_array(){current["sections"].clone()}else{qa04_sections(current["text"].as_str().unwrap_or(""))};let matches=sections.as_array().unwrap().iter().filter(|s|s["name"]==name).count();if matches!=1{return Err("SECTION_NAME_NOT_UNIQUE_OR_ABSENT".into())}for s in sections.as_array_mut().unwrap(){if s["name"]==name{s["text"]=json!(replacement);}}let text=qa04_render_sections(&sections);if text==current["text"].as_str().unwrap_or(""){return Err("DUPLICATE_SECTION_UPDATE".into())}current["sections"]=sections;current["text"]=json!(text);current["section_revision"]=json!(expected+1);current["section_edit_event_id"]=event["id"].clone();Ok(json!({"before":before,"after":current,"section":name}))}
fn qa04_task_result(response:&Value)->Value{json!({"delivery_event_id":response["stimulus_id"],"delivery_source_id":response["delivery_source_id"],"goal_revision":response["revision"],"section_revision":response["section_revision"],"text":response["text"],"sections":response["sections"]})}
fn qa04_patch_task_result(tasks:&mut[Value],edit:&Value)->Result<(),String>{let response=&edit["after"];let id=response["task_id"].as_str().ok_or("Agent result Task absent")?;let task=tasks.iter_mut().find(|t|t["task_id"]==id).ok_or("Agent result Task unavailable")?;if task["result_response"]["delivery_event_id"]!=response["stimulus_id"]{return Err("TASK_ACTUAL_RESULT_ASSOCIATION_MISMATCH".into())}task["result_response"]=qa04_task_result(response);Ok(())}
#[cfg(test)]mod qa04_c04_tests{use super::*;
 #[test]fn section_patch_preserves_other_text_task_goal_and_sources(){let mut delivered=vec![json!({"stimulus_id":"shown","goal_id":"g","revision":3,"task_id":"TA","phase":"AGENT_RESULT","text":"## Summary\nOld summary\n## Details\nKeep exact details","section_revision":1,"source_spans":[{"source_id":"s","revision":1,"start_line":1,"end_line":1}]})];let e=json!({"id":"edit","target_event_id":"shown","task_id":"TA","section":"Summary","text":"New summary","expected_section_revision":1});let r=qa04_section_edit(&mut delivered,&e).unwrap();assert_eq!(r["after"]["text"],"## Summary\nNew summary\n## Details\nKeep exact details");for k in ["goal_id","revision","task_id","source_spans"]{assert_eq!(r["before"][k],r["after"][k]);}assert!(qa04_section_edit(&mut delivered,&e).is_err());let mut wrong=e.clone();wrong["expected_section_revision"]=json!(2);wrong["task_id"]=json!("TB");assert!(qa04_section_edit(&mut delivered,&wrong).is_err());wrong["task_id"]=json!("TA");wrong["target_event_id"]=json!("unshown");assert!(qa04_section_edit(&mut delivered,&wrong).is_err());}
 #[test]fn task_result_section_only_changes(){let old=json!({"task_id":"TA","state":"COMPLETED","title":"keep title","result_response":{"delivery_event_id":"shown","text":"old"},"artifact":{"uri":"original-file"}});let other=json!({"task_id":"TB","result_response":{"text":"keep other original"}});let mut tasks=vec![old.clone(),other.clone()];let after=json!({"task_id":"TA","stimulus_id":"shown","text":"new section body","section_revision":2,"sections":[{"name":"Summary","text":"new"},{"name":"Details","text":"keep"}]});qa04_patch_task_result(&mut tasks,&json!({"after":after})).unwrap();assert_eq!(tasks[1],other);for k in ["task_id","state","title","artifact"]{assert_eq!(tasks[0][k],old[k]);}assert_eq!(tasks[0]["result_response"]["text"],"new section body");}
 #[test]fn only_displayed_sections_and_unique_names_are_editable(){let mut d=vec![json!({"stimulus_id":"h","phase":"HANDLING","text":"Concrete action"})];assert!(qa04_section_edit(&mut d,&json!({"target_event_id":"h","section":"body","text":"x","expected_section_revision":1})).is_err());let text="## one\na\n## two\nb";assert_eq!(qa04_render_sections(&qa04_sections(text)),text);}
}
'''

def apply_c04(crate:Path,configuration=None):
 from section_patch import apply
 return apply(crate,configuration)

def apply_c05(crate:Path,configuration=None):
 from provenance_patch import apply
 return apply(crate,configuration)

def apply_c06(crate:Path,configuration=None):
 from range_patch import apply
 return apply(crate,configuration)

