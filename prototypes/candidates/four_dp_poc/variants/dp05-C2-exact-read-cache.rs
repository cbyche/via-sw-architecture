//! DP05: delegation-time capsule, continuously produced work record, receiver-selected reads.
use crate::context::Context;
use serde_json::{json,Value};
use std::collections::BTreeMap;
fn array(v:&Value)->Vec<Value>{v.as_array().cloned().unwrap_or_default()}
fn valid_span(span:&Value,sources:&BTreeMap<String,Value>)->bool{
    let Some(sid)=span["source_id"].as_str() else{return false;};
    let Some(source)=sources.get(sid) else{return false;};
    let (Some(lo),Some(hi))=(span["start_line"].as_u64(),span["end_line"].as_u64()) else{return false;};
    source["revoked"].as_bool()!=Some(true) && source["revision"]==span["revision"] && lo>=source["allowed_lines"][0].as_u64().unwrap_or(1) && lo<=hi && hi<=source["allowed_lines"][1].as_u64().unwrap_or(0)
}
fn valid_block(block:&Value,sources:&BTreeMap<String,Value>)->bool{
    let spans=array(&block["source_spans"]);!spans.is_empty() && spans.iter().all(|s|valid_span(s,sources))
}
fn capsule_valid(v:&Value,sources:&BTreeMap<String,Value>)->bool{
    if v.is_null(){return true;}
    let passages=array(&v["passages"]);let blocks=array(&v["record_blocks"]);
    (!passages.is_empty() || !blocks.is_empty()) && passages.iter().all(|p|valid_span(p,sources)) && blocks.iter().all(|b|valid_block(b,sources))
}
fn capsule_spans(capsule:&Value)->Vec<Value>{
    let mut spans=array(&capsule["passages"]);
    for block in array(&capsule["record_blocks"]){spans.extend(array(&block["source_spans"]));}
    spans
}
fn receiver_completion_valid(result:&Value,capsule:&Value,sources:&BTreeMap<String,Value>)->bool{
    let available=capsule_spans(capsule);let cited=array(&result["source_spans"]);let used=array(&result["used_source_ids"]);
    result["text"].as_str().map(|t|!t.trim().is_empty()).unwrap_or(false)
        && !used.is_empty() && !cited.is_empty() && !array(&result["facts"]).is_empty()
        && used.iter().all(|sid|cited.iter().any(|span|span["source_id"]==*sid))
        && cited.iter().all(|span|valid_span(span,sources) && available.iter().any(|given|
            given["source_id"]==span["source_id"] && given["revision"]==span["revision"]
            && given["start_line"].as_u64().unwrap_or(u64::MAX)<=span["start_line"].as_u64().unwrap_or(0)
            && given["end_line"].as_u64().unwrap_or(0)>=span["end_line"].as_u64().unwrap_or(u64::MAX)))
}
fn catalog(sources:&BTreeMap<String,Value>)->Vec<Value>{sources.values().map(|s|json!({"id":s["id"],"title":s["title"],"revision":s["revision"],"outline":s["outline"],"line_count":s["line_count"],"revoked":s["revoked"],"allowed_lines":s["allowed_lines"]})).collect()}
fn read(ctx:&Context,s:&Value,start:usize,end:usize,consumer:&str)->Result<Value,String>{
    if s["revoked"].as_bool()==Some(true){return Err(format!("DENIED_REVOKED {}",s["id"]));}
    let raw=ctx.source(s)?;let lines:Vec<&str>=raw["text"].as_str().ok_or("missing source text")?.lines().collect();
    let lower=s["allowed_lines"][0].as_u64().unwrap_or(1) as usize;let upper=s["allowed_lines"][1].as_u64().unwrap_or(lines.len() as u64) as usize;
    if start<lower || end>upper || start>end || end>lines.len(){return Err(format!("DENIED_RANGE {} [{},{}]",s["id"],start,end));}
    let value=json!({"source_id":s["id"],"revision":s["revision"],"start_line":start,"end_line":end,"unit":"one_based_line_inclusive","text":lines[start-1..end].join("\n"),"consumer":consumer});ctx.event("HANDOFF_SOURCE_READ",&value);Ok(value)
}
fn selected_reads(ctx:&Context,requests:&Value,sources:&BTreeMap<String,Value>,consumer:&str)->(Vec<Value>,Vec<Value>){
    let mut passages=Vec::new();let mut denied=Vec::new();
    for r in array(requests){let sid=r["source_id"].as_str().unwrap_or("");if let Some(s)=sources.get(sid){
        let lo=r["start_line"].as_u64().unwrap_or(1) as usize;let hi=r["end_line"].as_u64().unwrap_or(1) as usize;
        match read(ctx,s,lo,hi,consumer){Ok(v)=>passages.push(v),Err(e)=>denied.push(json!({"request":r,"reason":e}))}
    }else{denied.push(json!({"request":r,"reason":"NO_MATCH"}));}}
    (passages,denied)
}
#[derive(Default)]
struct ReceiverReadCache { entries:BTreeMap<String,Value>,hits:u64,misses:u64,disk_reads:u64,denied:u64,evictions:u64,serving_ns:u64 }
fn cache_key(source:&Value,start:usize,end:usize,epoch:&Value,recipient:&Value)->String{
    json!([source["id"],source["revision"],start,end,epoch,recipient]).to_string()
}
fn current_read_allowed(source:&Value,start:usize,end:usize)->bool{
    source["revoked"].as_bool()!=Some(true) && start>=source["allowed_lines"][0].as_u64().unwrap_or(1) as usize
        && start<=end && end<=source["allowed_lines"][1].as_u64().unwrap_or(0) as usize
        && end<=source["line_count"].as_u64().unwrap_or(0) as usize
}
impl ReceiverReadCache {
    fn evict_source(&mut self,sid:&str,ctx:&Context,reason:&str){
        let before=self.entries.len();self.entries.retain(|_,v|v["source_id"].as_str()!=Some(sid));let removed=before-self.entries.len();self.evictions+=removed as u64;
        ctx.event("C2_CACHE_INVALIDATED",&json!({"source_id":sid,"reason":reason,"removed_entries":removed}));
    }
    fn evict_epoch(&mut self,ctx:&Context){let removed=self.entries.len();self.entries.clear();self.evictions+=removed as u64;ctx.event("C2_CACHE_INVALIDATED",&json!({"reason":"access_epoch_changed","removed_entries":removed}));}
    fn serve(&mut self,ctx:&Context,source:&Value,start:usize,end:usize,epoch:&Value,recipient:&Value)->Result<Value,String>{
        let begin=std::time::Instant::now();
        // A hit never grants authority: current source/range/access are checked before cache lookup.
        if !current_read_allowed(source,start,end){self.denied+=1;let elapsed=begin.elapsed().as_nanos() as u64;self.serving_ns+=elapsed;ctx.event("C2_READ_SERVICE",&json!({"source_id":source["id"],"revision":source["revision"],"start_line":start,"end_line":end,"access_epoch":epoch,"cache_hit":false,"status":"DENIED_CURRENT_ACCESS","disk_reads":0,"serving_ns":elapsed}));return Err("DENIED_CURRENT_ACCESS".into());}
        let key=cache_key(source,start,end,epoch,recipient);
        let cached=self.entries.get(&key).filter(|v|v["revision"]==source["revision"]).cloned();
        let hit=cached.is_some();
        let result=if let Some(v)=cached{self.hits+=1;Ok(v)}else{self.misses+=1;let value=read(ctx,source,start,end,"RECEIVER");if let Ok(ref v)=value{self.disk_reads+=1;self.entries.insert(key,v.clone());}value};
        let elapsed=begin.elapsed().as_nanos() as u64;self.serving_ns+=elapsed;
        ctx.event("C2_READ_SERVICE",&json!({"source_id":source["id"],"revision":source["revision"],"start_line":start,"end_line":end,"access_epoch":epoch,"recipient":recipient,"cache_hit":hit,"status":if result.is_ok(){"OK"}else{"READ_FAILED"},"disk_reads":if !hit && result.is_ok(){1}else{0},"serving_ns":elapsed}));
        result
    }
    fn stats(&self)->Value{json!({"cache_hits":self.hits,"cache_misses":self.misses,"successful_disk_reads":self.disk_reads,"denied_reads":self.denied,"evicted_entries":self.evictions,"resident_entries":self.entries.len(),"serving_ns":self.serving_ns,"scope":"receiver exact range serving; no model call or payload change"})}
}
fn cached_receiver_reads(ctx:&Context,requests:&Value,sources:&BTreeMap<String,Value>,cache:&mut ReceiverReadCache,epoch:&Value,recipient:&Value)->(Vec<Value>,Vec<Value>){
    let mut passages=Vec::new();let mut denied=Vec::new();
    for request in array(requests){let sid=request["source_id"].as_str().unwrap_or("");if let Some(source)=sources.get(sid){let lo=request["start_line"].as_u64().unwrap_or(1) as usize;let hi=request["end_line"].as_u64().unwrap_or(1) as usize;match cache.serve(ctx,source,lo,hi,epoch,recipient){Ok(v)=>passages.push(v),Err(reason)=>denied.push(json!({"request":request,"reason":reason}))}}else{denied.push(json!({"request":request,"reason":"NO_MATCH"}));}}
    (passages,denied)
}
fn produce(ctx:&mut Context,sources:&BTreeMap<String,Value>,state:&Value,previous:&Value,trigger:&Value,revision:u64)->Result<Value,String>{
    let mut evidence=Vec::new();for s in sources.values().filter(|s|s["revoked"].as_bool()!=Some(true)){
        evidence.push(read(ctx,s,s["allowed_lines"][0].as_u64().unwrap_or(1) as usize,s["allowed_lines"][1].as_u64().unwrap_or(s["line_count"].as_u64().unwrap_or(1)) as usize,"VIA_RECORD_PRODUCER")?);
    }
    let response=ctx.call("dp05_produce_work_record",&json!({"instruction":"Maintain a source-backed, purpose-neutral handoff work record from observed public activities and adopted state. No future delegation purpose is known. Produce reusable explanation of what each public result established, how results qualify or replace each other, conditions/exclusions and unresolved scope. Copying Task IDs alone is insufficient. Do not invent domain plans or future requests. Rebuild all dependent explanation on corrections. Return JSON {blocks:[{block_id:string,explanation:string,source_spans:[{source_id:string,revision:number,start_line:number,end_line:number}]}],conditions:[string],unknowns:[string],coverage:[string]}. Every explanation needs original source spans. Preserve current revisions and access revocations; old state is not authority.","input":{"observed_trigger":trigger,"current_adopted_state":state,"public_evidence":evidence,"previous_record":previous},"max_tokens":1800}))?;
    let mut valid=Vec::new();let mut rejected=Vec::new();
    for block in array(&response["blocks"]){if valid_block(&block,sources){valid.push(block);}else{ctx.event("PROVENANCE_REJECTED",&json!({"stage":"work_record_publication","block":block,"reason":"missing_or_stale_or_revoked_or_out_of_range_source_span"}));rejected.push(block);}}
    let record=json!({"record_revision":revision,"status":"PUBLISHED","source_vector":catalog(sources),"blocks":valid,"conditions":response["conditions"],"unknowns":response["unknowns"],"coverage":response["coverage"],"rejected_block_count":rejected.len()});ctx.event("WORK_RECORD_PUBLICATION",&record);Ok(record)
}

pub fn run(input:&Value,ctx:&mut Context)->Result<Value,String>{
    let variant=input["variant"].as_str().unwrap_or("A");let cache_enabled=variant=="C" && input["tactic"].as_str()==Some("C2");let mut read_cache=ReceiverReadCache::default();let mut sources:BTreeMap<String,Value>=BTreeMap::new();
    let mut state=json!({});let mut record=Value::Null;let mut record_revision=0;let mut outputs=Vec::new();let mut handoffs=Vec::new();let mut receipts=Vec::new();
    for event in input["events"].as_array().ok_or("events missing")?{
        match event["kind"].as_str().unwrap_or(""){
            "source"|"adopt"|"revoke"=>{
                match event["kind"].as_str().unwrap(){
                    "source"=>{let s=event["source"].clone();let sid=s["id"].as_str().ok_or("source id")?.to_string();
                        if sources.get(&sid).map(|old|old["revision"]!=s["revision"] || old["revoked"]!=s["revoked"]).unwrap_or(false){if cache_enabled{read_cache.evict_source(&sid,ctx,"source_revision_or_access_changed");}record=Value::Null;ctx.event("DERIVED_STATE_INVALIDATED",&json!({"source_id":sid,"reason":"source_revision_or_access_change","before_next_model_input":true}));}
                        sources.insert(sid,s);}
                    "adopt"=>{if cache_enabled && state["access_epoch"]!=event["state"]["access_epoch"]{read_cache.evict_epoch(ctx);}state=event["state"].clone();}
                    "revoke"=>{let sid=event["source_id"].as_str().ok_or("revoked source id")?;if cache_enabled{read_cache.evict_source(sid,ctx,"revoked");}if let Some(s)=sources.get_mut(sid){s["revoked"]=json!(true);}record=Value::Null;ctx.event("LEASE_REVOKED",event);ctx.event("DERIVED_STATE_INVALIDATED",&json!({"source_id":sid,"reason":"revoked","before_next_model_input":true}));}
                    _=>unreachable!(),
                }
                ctx.event("PUBLIC_ACTIVITY",event);
                if variant=="B" {record_revision+=1;record=produce(ctx,&sources,&state,&record,event,record_revision)?;}
            }
            "observe"=>{
                let out=json!({"goal_id":event["goal_id"],"stimulus_id":event["stimulus_id"],"text":event["text"]});
                ctx.event("STIMULUS",&json!({"stimulus_id":event["stimulus_id"],"goal_id":event["goal_id"]}));ctx.present(&out);ctx.event("GOAL_COMPLETE",&out);outputs.push(out);
            }
            "delegate"=>{
                let goal=event["goal_id"].as_str().unwrap_or("");let stimulus=event["stimulus_id"].as_str().unwrap_or("");
                ctx.event("STIMULUS",&json!({"stimulus_id":stimulus,"goal_id":goal,"input":event}));
                let intention=json!({"goal_id":goal,"stimulus_id":stimulus,"text":format!("현재 채택한 조건으로 {} 작업을 준비합니다.",event["purpose"].as_str().unwrap_or("요청한")),"phase":"HANDLING_INTENTION"});ctx.present(&intention);
                let available:Vec<Value>=catalog(&sources).into_iter().filter(|s|s["revoked"].as_bool()!=Some(true)).collect();
                let mut mandatory=json!({"delegation_id":event["delegation_id"],"delegation_revision":event["delegation_revision"],"request_revision":state["request_revision"],"task_id":state["task_id"],"goal_id":goal,"purpose":event["purpose"],"completion_condition":event["completion_condition"],"constraints":state["constraints"],"exclusions":state["exclusions"],"allowed_actions":state["allowed_actions"],"already_completed_actions":state["already_completed_actions"],"unknown_actions":state["unknown_actions"],"required_source_ids":state["required_source_ids"],"condition_revision":state["condition_revision"],"recipient":event["recipient"],"access_epoch":state["access_epoch"],"revoked_sources":sources.values().filter(|s|s["revoked"].as_bool()==Some(true)).map(|s|s["id"].clone()).collect::<Vec<_>>()});
                let mut missing=Vec::new();for required in array(&state["required_source_ids"]){if let Some(sid)=required.as_str(){if !sources.get(sid).map(|s|s["revoked"].as_bool()!=Some(true)).unwrap_or(false){missing.push(required);}}}
                mandatory["disposition"]=json!(if missing.is_empty(){"AUTHORIZED"}else{"BLOCKED_REQUIRED_SOURCE"});
                mandatory["missing_required_sources"]=json!(missing);
                let mut capsule=Value::Null;
                if missing.is_empty() && variant!="C" {
                    let plan=ctx.call(if variant=="B"{"dp05_extract_record"}else{"dp05_select_handoff_details"},&json!({"instruction":"Select details needed for the CURRENT adopted handoff purpose. Metadata/outline is not source content. Return JSON {reads:[{source_id:string,start_line:number,end_line:number}],summary:string,selected_record_blocks:[string]}. For A, request only necessary original source ranges and do not summarize unseen facts. For B, extract actual source-backed work-record blocks; request original spans only for missing/uncertain detail. Artifact references and receiver read tools are legal in both designs. Mandatory conditions are supplied separately and cannot be altered. Do not add domain plans or broaden actions.","input":{"mandatory":mandatory,"manifest":available,"work_record":if variant=="B"{record.clone()}else{Value::Null},"basic_variant":variant},"max_tokens":900}))?;
                    let (passages,denials)=selected_reads(ctx,&plan["reads"],&sources,"VIA_COMPOSER");
                    let chosen_blocks:Vec<Value>=array(&record["blocks"]).into_iter().filter(|b|array(&plan["selected_record_blocks"]).contains(&b["block_id"])).collect();
                    if variant=="A" || !passages.is_empty(){
                        capsule=ctx.call("dp05_compose_capsule",&json!({"instruction":"Compose a compact purpose-specific handoff explanation using only actual passages and selected valid record blocks. Preserve caveats and source spans. Mandatory contract is authoritative and separate. Return JSON {summary:string,passages:[{source_id:string,revision:number,start_line:number,end_line:number,explanation:string}],unknowns:[string]}. References may carry large documents; do not copy the whole artifact or guess missing facts.","input":{"mandatory":mandatory,"actual_passages":passages,"selected_record_blocks":chosen_blocks,"denials":denials},"max_tokens":1100}))?;
                    }else{capsule=json!({"summary":plan["summary"],"record_blocks":chosen_blocks,"unknowns":denials});}
                }
                let invalid_capsule=!capsule_valid(&capsule,&sources);
                if invalid_capsule{ctx.event("PROVENANCE_REJECTED",&json!({"stage":"handoff_admission","capsule":capsule,"reason":"missing_or_stale_or_revoked_or_out_of_range_source_span"}));capsule=Value::Null;mandatory["disposition"]=json!("BLOCKED_INVALID_CAPSULE_PROVENANCE");}
                let contract=json!({"mandatory":mandatory,"manifest":available,"capsule":capsule,"detail_selection_authority":if variant=="C"{"RECEIVER"}else{"VIA_WITH_RECEIVER_FOLLOWUP_READS"},"source_vector":sources.values().map(|s|json!({"source_id":s["id"],"revision":s["revision"],"revoked":s["revoked"]})).collect::<Vec<_>>()});
                ctx.event("HANDOFF_ADMISSION",&contract);handoffs.push(contract.clone());
                if !missing.is_empty() || invalid_capsule{let out=json!({"goal_id":goal,"stimulus_id":stimulus,"text":"필수 자료의 현재 접근 권한이나 생성 설명의 출처를 검증할 수 없어 위임을 보류합니다.","phase":"BLOCKED"});ctx.present(&out);ctx.event("GOAL_COMPLETE",&out);outputs.push(out);continue;}
                ctx.event("AGENT_INGRESS",&json!({"goal_id":goal,"stimulus_id":stimulus,"delegation_id":event["delegation_id"]}));
                // Identical real model receiver and available read tool for A/B/C.
                let mut selection=ctx.call("dp05_receiver_select_or_complete",&json!({"role":"RECEIVER","instruction":"You are the minimal downstream receiver. Prepare the small bounded draft requested by this authorized contract. You may complete NOW from actual valid source-backed capsule passages or record blocks if they fully suffice; otherwise select original source reads. Manifest titles/outlines alone are NEVER source facts. The mandatory contract supplies authoritative goals/conditions, not the missing public report contents. Return JSON {mode:'COMPLETE'|'READ',result:{text:string,used_source_ids:[string],source_spans:[{source_id:string,revision:number,start_line:number,end_line:number}],facts:[string],constraints_respected:[string],actions_performed:[string],uncertainties:[string]}|null,reads:[{source_id:string,start_line:number,end_line:number}],reason:string}. COMPLETE must cite only exact source-backed content actually present in the capsule, with source spans contained in its supplied spans; do not cite a manifest entry as if read. If facts are absent or inadequate, choose READ with bounded inclusive one-based ranges within allowed_lines. A/B/C have identical completion and read capabilities. Do not perform actual external actions or repeat unknown actions.","input":contract,"max_tokens":1100}))?;
                let complete=selection["mode"].as_str()==Some("COMPLETE") && receiver_completion_valid(&selection["result"],&contract["capsule"],&sources);
                ctx.event("RECEIVER_BRANCH",&json!({"delegation_id":event["delegation_id"],"model_mode":selection["mode"],"branch":if complete{"COMPLETE_FROM_SUPPLIED_CAPSULE"}else{"READ_ORIGINAL_SOURCES"},"completion_provenance_verified":complete,"actually_supplied_source_spans":capsule_spans(&contract["capsule"])}));
                let receiver=if complete {
                    let receipt=json!({"delegation_id":event["delegation_id"],"selected_reads":[],"served":[],"denied":[],"completion_from_supplied_capsule":true});
                    ctx.event("RECEIVER_READ_RECEIPT",&receipt);receipts.push(receipt);
                    selection["result"].clone()
                }else{
                    if selection["mode"].as_str()==Some("COMPLETE"){
                        ctx.event("RECEIVER_COMPLETION_REJECTED",&json!({"delegation_id":event["delegation_id"],"reason":"source_facts_or_valid_actually_supplied_provenance_missing","proposed_result":selection["result"]}));
                        selection=ctx.call("dp05_receiver_request_valid_reads",&json!({"role":"RECEIVER","instruction":"The proposed completion lacked actual supplied evidence or valid provenance and was not accepted. Select original source ranges needed for this bounded draft. Return JSON {reads:[{source_id:string,start_line:number,end_line:number}],reason:string}. Metadata is not content. Read only current authorized source ranges; do not guess unseen facts.","input":contract,"max_tokens":700}))?;
                    }
                    ctx.event("RECEIVER_READ_SELECTION",&selection);
                    let (passages,denials)=if cache_enabled{cached_receiver_reads(ctx,&selection["reads"],&sources,&mut read_cache,&state["access_epoch"],&event["recipient"])}else{selected_reads(ctx,&selection["reads"],&sources,"RECEIVER")};
                    let receipt=json!({"delegation_id":event["delegation_id"],"selected_reads":selection["reads"],"served":passages,"denied":denials,"completion_from_supplied_capsule":false});ctx.event("RECEIVER_READ_RECEIPT",&receipt);receipts.push(receipt);
                    ctx.call("dp05_receiver_result",&json!({"role":"RECEIVER","instruction":"Return the requested bounded demonstration result from the authorized handoff and actually supplied source passages. Return JSON {text:string,used_source_ids:[string],source_spans:[{source_id:string,revision:number,start_line:number,end_line:number}],facts:[string],constraints_respected:[string],actions_performed:[string],uncertainties:[string]}. Produce only a small draft or structured check, not a full product. Honor all current conditions and exclusions; do not use withdrawn sources or obsolete revisions. No actual external action is performed. Do not invent source content; report missing material. You may rely on source-backed capsule explanation as well as actual read passages; cite their supplied source spans. Manifest metadata alone is not evidence.","input":{"contract":contract,"tool_passages":passages,"tool_denials":denials},"max_tokens":1100}))?
                };
                ctx.event("AGENT_RESULT_AVAILABLE",&json!({"goal_id":goal,"result":receiver}));
                let out=json!({"goal_id":goal,"stimulus_id":stimulus,"text":receiver["text"],"phase":"RESULT","receiver":receiver});ctx.present(&out);ctx.event("GOAL_COMPLETE",&out);outputs.push(out);
            }
            _=>return Err(format!("unknown dp05 event {}",event["kind"])),
        }
    }
    if cache_enabled{ctx.event("C2_CACHE_STATS",&read_cache.stats());}
    Ok(json!({"adoptions":[],"outputs":outputs,"handoffs":handoffs,"receiver_receipts":receipts,"production_state":record,"cache_stats":if cache_enabled{read_cache.stats()}else{Value::Null},"fixture_class":"CONSTRUCTED_SYNTHETIC_POC"}))
}
