"""Future-feature patches; source copies only, never the active baseline.

Final anchors are checked against the formal frozen source. Failed rebase attempts stay visible.
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

C05_HELPERS=r'''
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

def apply_c05(crate:Path,configuration=None):
 p=crate/'src/runtime.rs';s=p.read_text()
 # Final actual adopted geometry is preserved only after final semantic rejudgment.
 anchor='let(gid,revision)=self.resolve_goal(event,&plan)?;'
 s=once(s,anchor,'plan["qa04_adopted_geometry"]=qa04_actual_geometry(&plan,&grounded);'+anchor)
 anchor='"bindings":plan["bindings"],"targets":plan["targets"]'
 s=once(s,anchor,'"adopted_geometry_provenance":plan["qa04_adopted_geometry"],'+anchor)
 anchor='engine.capture(&ctx,event);'
 branch='''if event["kind"]=="qa04_provenance_query"{let _gate=engine.sources.guard();let s=engine.state.lock().unwrap();let before=qa04_readonly_state(&s);let sources_before=engine.sources.catalog();let result=qa04_query_provenance(&s.delivered,&engine.sources,event);let after=qa04_readonly_state(&s);let sources_after=engine.sources.catalog();assert_eq!(before,after);assert_eq!(sources_before,sources_after);drop(s);ctx.event("QA04_PROVENANCE_READONLY_STATE",&json!({"event_id":event["id"],"before":before,"after":after,"sources_before":sources_before,"sources_after":sources_after}));ctx.event("QA04_PROVENANCE_PANEL_DISPLAY",&result);ctx.event("QA04_FEATURE_RESPONSE_FINAL",&json!({"event_id":event["id"],"kind":"provenance_query"}));continue}'''
 s=once(s,anchor,branch+anchor);s+='\n'+C05_HELPERS;p.write_text(s)

C06_SOURCE_METHODS=r'''
 pub fn withdraw_range(&self,operation:&Value)->Result<(),String>{let _gate=self.guard();let id=operation["source_id"].as_str().ok_or("range source missing")?;let a=operation["start_line"].as_u64().ok_or("range start missing")?;let b=operation["end_line"].as_u64().ok_or("range end missing")?;if a==0||b<a{return Err("invalid withdrawal range".into())}let mut s=self.inner.lock().unwrap();let current=*s.current.get(id).ok_or("withdraw source unavailable")?;if operation["revision"].as_u64()!=Some(current){return Err("withdraw authority source revision stale".into())}let n=s.versions[&(id.to_owned(),current)]["line_count"].as_u64().unwrap_or(0);if b>n{return Err("withdraw range beyond current source".into())}for((sid,_),source)in s.versions.iter_mut(){if sid!=id{continue}for key in ["allowed_segments","external_segments"]{source[key]=qa04_subtract_ranges(&source[key],a,b);}let mut withdrawn=source["withdrawn_ranges"].as_array().cloned().unwrap_or_default();withdrawn.push(json!([a,b]));source["withdrawn_ranges"]=json!(withdrawn);source["range_access_epoch"]=operation["access_epoch"].clone();}Ok(())}

'''
C06_SOURCE_HELPERS=r'''
fn qa04_subtract_ranges(ranges:&Value,a:u64,b:u64)->Value{let mut out=Vec::new();for r in ranges.as_array().into_iter().flatten(){let(Some(lo),Some(hi))=(r[0].as_u64(),r[1].as_u64())else{continue};if hi<a||lo>b{out.push(json!([lo,hi]));continue}if lo<a{out.push(json!([lo,a-1]))}if hi>b{out.push(json!([b+1,hi]))}}json!(out)}
#[cfg(test)]mod qa04_c06_source_tests{use super::*;
 #[test]fn old_versions_cannot_bypass_withdrawal_and_raw_fragments_survive(){let s=SourceRegistry::new();for v in [1,2]{s.register(json!({"id":"s","revision":v,"text":"left public\nWITHDRAWN PRIVATE FACT\nright public","segments":{"all":[1,3]}})).unwrap();}s.withdraw_range(&json!({"source_id":"s","revision":2,"start_line":2,"end_line":2,"access_epoch":2})).unwrap();for v in [1,2]{assert!(!s.allowed("s",v,2,2,false));assert!(!s.allowed("s",v,1,3,true));assert!(s.allowed("s",v,1,1,true));assert!(s.allowed("s",v,3,3,false));}assert_eq!(s.read("s",2,1,1,false).unwrap().text,"left public");assert_eq!(s.read("s",2,3,3,false).unwrap().text,"right public");s.register(json!({"id":"s","revision":2,"text":"left public\nWITHDRAWN PRIVATE FACT\nright public","segments":{"all":[1,3]}})).unwrap();s.permission(&json!({"source_id":"s","read_segments":["all"],"external_segments":["all"]})).unwrap();assert!(!s.allowed("s",2,2,2,false));assert_eq!(s.current("s").unwrap()["allowed_segments"],json!([[1,1],[3,3]]));}
}
'''

def apply_c06(crate:Path,configuration=None):
 p=crate/'src/sources.rs';s=p.read_text();s=once(s,' pub fn current(&self,id:&str)',C06_SOURCE_METHODS+'\n pub fn current(&self,id:&str)')
 anchor='source["external_segments"]=old["external_segments"].clone();'
 s=once(s,anchor,anchor+'if old.get("withdrawn_ranges").is_some(){source["withdrawn_ranges"]=old["withdrawn_ranges"].clone();}')
 anchor='let Some(src)=self.version(id,revision)else{return false;};'
 guard='if src["withdrawn_ranges"].as_array().is_some_and(|ranges|ranges.iter().any(|r|r[0].as_u64().is_some_and(|a|a<=end_line as u64)&&r[1].as_u64().is_some_and(|b|b>=start_line as u64))){return false}'
 s=once(s,anchor,anchor+guard)
 anchor='source[field]=json!(ranges);'
 s=once(s,anchor,anchor+'for withdrawn in source["withdrawn_ranges"].as_array().cloned().unwrap_or_default(){source[field]=qa04_subtract_ranges(&source[field],withdrawn[0].as_u64().unwrap_or(0),withdrawn[1].as_u64().unwrap_or(0));}')
 s+='\n'+C06_SOURCE_HELPERS;p.write_text(s)
 p=crate/'src/runtime.rs';s=p.read_text();anchor='engine.capture(&ctx,event);'
 branch='''if event["kind"]=="qa04_withdraw_range"{engine.sources.withdraw_range(event)?;engine.state.lock().unwrap().counter+=1;engine.activity(&ctx,json!({"kind":"SOURCE_RANGE_WITHDRAWAL","id":event["source_id"]}));ctx.event("QA04_RANGE_WITHDRAWAL_APPLIED",event);ctx.event("QA04_FEATURE_RESPONSE_FINAL",&json!({"event_id":event["id"],"kind":"withdraw_range"}));continue}'''
 s=once(s,anchor,branch+anchor);p.write_text(s)

C04_HELPERS=r'''
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
 p=crate/'src/sources.rs';source=p.read_text();anchor='pub fn register(&self,mut source:Value)->Result<(),String>{let _gate=self.guard();'
 source=once(source,anchor,'pub fn register(&self,source:Value)->Result<(),String>{let _gate=self.guard();self.register_locked(source,false)}\n pub fn register_locked(&self,mut source:Value,display_edit:bool)->Result<(),String>{')
 source=once(source,'if let Some(oldrev)=s.current.get(&id){','if !display_edit{if let Some(oldrev)=s.current.get(&id){')
 source=once(source,'source["external_segments"]=old["external_segments"].clone();}}s.current','source["external_segments"]=old["external_segments"].clone();}}}s.current');p.write_text(source)
 p=crate/'src/runtime.rs';s=p.read_text()
 # Section state is attached to actual publication; models/proposals cannot create delivery history.
 anchor='"text":text,"phase":phase,'
 s=once(s,anchor,anchor+'"sections":qa04_sections(text),"section_revision":1,')
 anchor='s.delivered.push(output.clone());'
 s=once(s,anchor,anchor+'if phase=="AGENT_RESULT"{if let Some(task)=s.tasks.iter_mut().find(|t|t["task_id"]==output["task_id"]){task["result_response"]=qa04_task_result(&output);}}')
 anchor='ctx.local_operation("response_publication","VIA",||'
 branch='if phase=="AGENT_RESULT"{let id=output["delivery_source_id"].as_str().unwrap_or(event["id"].as_str().unwrap_or("")).to_owned();output["delivery_source_id"]=json!(id);self.sources.register_locked(json!({"id":id,"revision":revision,"kind":"delivered","text":text,"local_allowed":true,"external_allowed":evidence.passages.iter().all(|p|crate::evidence::permitted(&self.sources,p,true)),"provenance":evidence.passages,"extents":{"delivered":[1,text.lines().count()]}}),false)?;}'
 s=once(s,anchor,branch+anchor)
 anchor='engine.capture(&ctx,event);'
 branch=r'''if event["kind"]=="qa04_section_update"{let result=(||->Result<Value,String>{let _gate=engine.sources.guard();let mut state=engine.state.lock().unwrap();let tasks_before=state.tasks.clone();let requests_before=state.requests.clone();let mut proposed=state.delivered.clone();let mut edit=qa04_section_edit(&mut proposed,event)?;let mut tasks_after=tasks_before.clone();qa04_patch_task_result(&mut tasks_after,&edit)?;for span in array(&edit["after"]["source_spans"]){if !crate::evidence::permitted(&engine.sources,&span,false){return Err("SECTION_SOURCE_PERMISSION_CHANGED".into())}}let after=&edit["after"];let id=after["delivery_source_id"].as_str().or_else(||after["stimulus_id"].as_str()).ok_or("section delivery ID missing")?;let prior=engine.sources.current(id).ok_or("section displayed source unavailable")?;let prior_revision=prior["revision"].as_u64().unwrap_or(0);let prior_lines=prior["line_count"].as_u64().unwrap_or(0)as usize;if !engine.sources.allowed(id,prior_revision,1,prior_lines,false){return Err("SECTION_DISPLAY_ACCESS_REVOKED".into())}let may_export=engine.sources.allowed(id,prior_revision,1,prior_lines,true);let source_revision=prior_revision+1;engine.sources.register_locked(json!({"id":id,"revision":source_revision,"kind":"delivered","text":after["text"],"provenance":after["source_spans"],"local_allowed":true,"external_allowed":may_export&&array(&after["source_spans"]).iter().all(|p|crate::evidence::permitted(&engine.sources,p,true)),"extents":{"delivered":[1,after["text"].as_str().unwrap_or("").lines().count()]}}),true)?;state.delivered=proposed;state.tasks=tasks_after;state.counter+=1;assert_eq!(state.requests,requests_before);edit["task_state_before"]=json!(tasks_before);edit["task_state_after"]=json!(state.tasks);edit["request_state_before"]=json!(requests_before);edit["request_state_after"]=json!(state.requests);edit["registered_display_revision"]=json!(source_revision);ctx.event("QA04_SECTION_DISPLAY_PROXY",&edit["after"]);Ok(edit)})();match result{Ok(edit)=>{engine.activity(&ctx,json!({"kind":"RESPONSE_SECTION_REVISED","id":edit["after"]["delivery_source_id"]}));ctx.event("QA04_SECTION_UPDATE_APPLIED",&edit)},Err(error)=>ctx.event("QA04_SECTION_UPDATE_REJECTED",&json!({"event_id":event["id"],"error":error}))}ctx.event("QA04_FEATURE_RESPONSE_FINAL",&json!({"event_id":event["id"],"kind":"section_update"}));continue}'''
 s=once(s,anchor,branch+anchor);s+='\n'+C04_HELPERS;p.write_text(s)
