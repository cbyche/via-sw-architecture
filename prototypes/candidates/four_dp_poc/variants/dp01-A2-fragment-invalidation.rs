//! DP01: rolling context, request-local composition, or independently published relations.
use crate::context::Context;
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn id(v:&Value)->String { v["id"].as_str().unwrap_or("").to_string() }
fn catalog(sources:&BTreeMap<String,Value>)->Vec<Value> {
    sources.values().map(|s|json!({"id":s["id"],"title":s["title"],"revision":s["revision"],"conversation":s["conversation"],"kind":s["kind"],"outline":s["outline"],"revoked":s["revoked"],"delivered_lines":s["delivered_lines"]})).collect()
}
fn read(ctx:&Context,s:&Value,start:usize,end:usize)->Result<Value,String> {
    if s["revoked"].as_bool()==Some(true) { return Err(format!("DENIED {}",s["id"])); }
    let raw=ctx.source(s)?;
    let text=raw["text"].as_str().ok_or("source text absent")?;
    let lines:Vec<&str>=text.lines().collect();
    let allowed_start=s["delivered_lines"][0].as_u64().unwrap_or(1) as usize;
    let allowed_end=s["delivered_lines"][1].as_u64().unwrap_or(lines.len() as u64) as usize;
    let lo=start.max(1).max(allowed_start).min(lines.len()+1);
    let hi=end.min(allowed_end).min(lines.len());
    let passage=if lo<=hi {lines[lo-1..hi].join("\n")}else{String::new()};
    let value=json!({"source_id":s["id"],"revision":s["revision"],"start_line":lo,"end_line":hi,"unit":"one_based_line_inclusive","text":passage,"actual_delivery_only":!s["delivered_lines"].is_null()});
    ctx.event("EVIDENCE_READ",&value); Ok(value)
}
fn array(v:&Value)->Vec<Value>{v.as_array().cloned().unwrap_or_default()}
fn valid_span(span:&Value,sources:&BTreeMap<String,Value>)->bool{
    let Some(source)=span["source_id"].as_str().and_then(|sid|sources.get(sid)) else{return false;};
    let (Some(lo),Some(hi))=(span["start_line"].as_u64(),span["end_line"].as_u64()) else{return false;};
    source["revoked"].as_bool()!=Some(true) && source["revision"]==span["revision"] && lo>=source["delivered_lines"][0].as_u64().unwrap_or(1) && lo<=hi && hi<=source["delivered_lines"][1].as_u64().unwrap_or(source["line_count"].as_u64().unwrap_or(0))
}

pub fn run(input:&Value,ctx:&mut Context)->Result<Value,String>{
    let variant=input["variant"].as_str().unwrap_or("A");
    let mut sources:BTreeMap<String,Value>=BTreeMap::new();
    let mut rolling:Vec<Value>=Vec::new();
    let mut published=Value::Null;
    let mut publication_revision=0;
    let mut conversation=String::new();
    let mut recent:Vec<Value>=Vec::new();
    let mut adoptions=Vec::new(); let mut outputs=Vec::new();
    for event in input["events"].as_array().ok_or("events missing")? {
        match event["kind"].as_str().unwrap_or("") {
            "source"=>{
                let s=event["source"].clone(); let sid=id(&s);
                let changed=sources.get(&sid).map(|old|old["revision"]!=s["revision"] || old["revoked"]!=s["revoked"]).unwrap_or(false);
                sources.insert(sid.clone(),s);
                ctx.event("SOURCE_ACTIVITY",event);
                if changed {
                    // Whole-view rebuild is the basic design; partial patches remain a tactic.
                    let before=rolling.len();
                    let fragment_tactic=variant=="A" && input["tactic"].as_str()==Some("A2");
                    if fragment_tactic {
                        rolling.retain(|span|valid_span(span,&sources));
                        ctx.event("A2_FRAGMENT_INVALIDATION",&json!({"changed_source_id":sid,"before_fragments":before,"retained_fragments":rolling.len(),"removed_fragments":before-rolling.len(),"retained_source_spans":rolling}));
                    }else{rolling.clear();}
                    published=Value::Null;
                    recent.retain(|entry|!array(&entry["source_revisions"]).iter().any(|span|span["source_id"].as_str()==Some(&sid)));
                    ctx.event("DERIVED_STATE_INVALIDATED",&json!({"source_id":sid,"reason":"revision_or_access_change","scope":if fragment_tactic{"invalid_source_dependent_working_fragments"}else{"whole_working_view_or_publication"}}));
                }
                if variant=="C" {
                    let mut evidence=Vec::new();
                    for source in sources.values().filter(|s|s["revoked"].as_bool()!=Some(true)) {
                        evidence.push(read(ctx,source,1,usize::MAX)?);
                    }
                    let candidate=ctx.call("dp01_publish_relations",&json!({"instruction":"You are an independent evidence producer. Process only the observed source activity, never predict future requests. Publish source-backed relationships and explanation items from the sources. Distinguish actually delivered response lines from prepared undisplayed content. Return JSON {records:[{record_id:string,subject:string,predicate:string,object:string,explanation:string,source_spans:[{source_id:string,revision:number,start_line:number,end_line:number}]}],coverage:{source_ids:[string],gaps:[string]}}. Include facts and relations useful across conversations; mark uncertainty, do not invent selected targets or future tasks. Every semantic assertion needs source spans. Limit 10 compact records.","input":{"source_activity":event,"catalog":catalog(&sources),"observed_evidence":evidence},"max_tokens":1800}))?;
                    let mut records=Vec::new();let mut rejected=Vec::new();
                    for record in array(&candidate["records"]){let spans=array(&record["source_spans"]);if !spans.is_empty() && spans.iter().all(|span|valid_span(span,&sources)){records.push(record);}else{ctx.event("PROVENANCE_REJECTED",&json!({"stage":"relation_publication","record":record}));rejected.push(record["record_id"].clone());}}
                    publication_revision+=1;
                    published=json!({"publication_revision":publication_revision,"source_vector":sources.values().map(|s|json!({"id":s["id"],"revision":s["revision"],"revoked":s["revoked"]})).collect::<Vec<_>>(),"records":records,"coverage":candidate["coverage"],"rejected_record_ids":rejected,"state":"PUBLISHED"});
                    ctx.event("RELATION_PUBLICATION",&published);
                }
            }
            "request"=>{
                let conv=event["conversation"].as_str().unwrap_or("default");
                if conversation!=conv { rolling.clear();recent.clear();conversation=conv.to_string();ctx.event("CONVERSATION_BOUNDARY",&json!({"conversation":conv,"working_view_reset":true})); }
                let sid=event["stimulus_id"].as_str().unwrap_or("");let goal=event["goal_id"].as_str().unwrap_or("");
                ctx.event("STIMULUS",&json!({"stimulus_id":sid,"goal_id":goal,"input":event}));
                let plan=ctx.call("dp01_select_evidence_or_answer",&json!({"instruction":"Interpret this CURRENT utterance using the ACTUAL valid evidence supplied. If retained passages, source-backed published records, or an actually delivered recent answer fully suffice, answer now without an extra call. Metadata/outline alone is NEVER evidence for facts. Return JSON {mode:'ANSWER'|'READ',answer:{targets:[source ID strings],text:string,conditions:[string],source_revisions:[{source_id:string,revision:number}],uncertainties:[string]}|null,reads:[{source_id:string,start_line:number,end_line:number}],record_ids:[string],reason:string}. ANSWER requires the exact set of original source IDs referred to by the specified expression and their current source revisions. Resolve section references to the original source ID. Only actually delivered response lines count as explained, never prepared undisplayed content. If actual evidence is missing, choose READ and bounded inclusive one-based source passages; choose useful published record IDs too if applicable. Later detail or corrected facts may require a read even if an earlier answer concerns the same source. Do not invent source IDs or facts, or infer unseen values from titles. Recent answers are common to all designs and may suffice in any design.","input":{"utterance":event["utterance"],"expression":event["expression"],"catalog":catalog(&sources),"recent_conversation":recent,"retained_working_view":if variant=="A"{json!(rolling)}else{Value::Null},"published_relations":if variant=="C"{published.clone()}else{Value::Null}},"max_tokens":1100}))?;
                ctx.event("EVIDENCE_QUERY",&plan);
                let mut available=Vec::new();
                if variant=="A" {available.extend(rolling.iter().map(|v|v["source_id"].clone()));}
                if variant=="C" {for record in array(&published["records"]){available.extend(array(&record["source_spans"]).into_iter().map(|s|s["source_id"].clone()));}}
                for entry in &recent {available.extend(array(&entry["source_revisions"]).into_iter().map(|s|s["source_id"].clone()));}
                let proposed=&plan["answer"];let targets=array(&proposed["targets"]);let provenance=array(&proposed["source_revisions"]);
                let shortcut=plan["mode"].as_str()==Some("ANSWER") && !targets.is_empty() && proposed["text"].is_string() && targets.iter().all(|target|available.contains(target) && provenance.iter().any(|p|p["source_id"]==*target && p["source_id"].as_str().and_then(|s|sources.get(s)).map(|s|s["revision"]==p["revision"] && s["revoked"].as_bool()!=Some(true)).unwrap_or(false)));
                ctx.event("EVIDENCE_BRANCH",&json!({"branch":if shortcut{"ANSWER_FROM_AVAILABLE_EVIDENCE"}else{"BOUNDED_SOURCE_COMPLETION"},"model_requested_mode":plan["mode"],"available_source_ids":available,"provenance_verified":shortcut}));
                let mut bundle=Vec::new();let mut unavailable=Vec::new();
                for req in if shortcut{Vec::new()}else{array(&plan["reads"])} {
                    let source_id=req["source_id"].as_str().unwrap_or("");
                    if let Some(source)=sources.get(source_id) {
                        let lo=req["start_line"].as_u64().unwrap_or(1) as usize;
                        let hi=req["end_line"].as_u64().unwrap_or(100) as usize;
                        let cached=if variant=="A" {rolling.iter().find(|v|v["source_id"]==source["id"] && v["revision"]==source["revision"] && v["start_line"].as_u64().unwrap_or(1)<=lo as u64 && v["end_line"].as_u64().unwrap_or(0)>=hi as u64).cloned()} else {None};
                        if let Some(value)=cached {ctx.event("ROLLING_EVIDENCE_REUSE",&value);bundle.push(value);}else {
                            match read(ctx,source,lo,hi){Ok(v)=>bundle.push(v),Err(e)=>unavailable.push(json!({"source_id":source_id,"status":e}))}
                        }
                    } else {unavailable.push(json!({"source_id":source_id,"status":"NO_MATCH"}));}
                }
                let selected_records:Vec<Value>=array(&published["records"]).into_iter().filter(|r|array(&plan["record_ids"]).contains(&r["record_id"])).collect();
                let answer=if shortcut{proposed.clone()}else{ctx.call("dp01_interpret_and_answer",&json!({"instruction":"Interpret the user's referent expression and answer only from supplied evidence. Return JSON {targets:[source ID strings],text:string,conditions:[string],source_revisions:[{source_id:string,revision:number}],uncertainties:[string]}. targets is the exact set of original source IDs referred to by the specified expression, excluding unrelated sources. For a section of one source target that original source ID. Use current source revisions and actual delivery only: prepared but undisplayed response text was never explained to the user. A correction supersedes old source facts only within its scope. Published semantic records are derived interpretations and may need original evidence; do not assume they are independent source facts. Say evidence missing if needed. Do not describe target metadata as successful reading.","input":{"utterance":event["utterance"],"expression":event["expression"],"catalog":catalog(&sources),"recent_conversation":recent,"read_bundle":bundle,"retained_working_view":if variant=="A"{json!(rolling)}else{Value::Null},"published_records":selected_records,"unavailable":unavailable},"max_tokens":1000}))?};
                let adoption=json!({"expression_id":event["expression_id"],"revision":event["revision"],"targets":answer["targets"],"source_revisions":answer["source_revisions"],"conditions":answer["conditions"],"first_adoption":true});
                ctx.event("SEMANTIC_ADOPTION",&adoption);adoptions.push(adoption);
                let output=json!({"goal_id":goal,"stimulus_id":sid,"text":answer["text"],"details":answer});
                ctx.present(&output);ctx.event("GOAL_COMPLETE",&output);outputs.push(output);
                if variant=="A" {
                    for b in bundle {if !rolling.contains(&b){rolling.push(b);}}
                    if rolling.iter().map(|v|v.to_string().len()).sum::<usize>()>14000 {
                        let compressed=ctx.call("dp01_compress_working_view",&json!({"instruction":"Compress the rolling evidence for future turns in this same conversation. Preserve source ID, revision and exact source span provenance, exceptions and actual delivery scope. Return JSON {items:[{source_id:string,revision:number,start_line:number,end_line:number,text:string}]}. Do not add facts.","input":rolling,"max_tokens":1500}))?;
                        rolling=array(&compressed["items"]).into_iter().filter(|span|valid_span(span,&sources)).collect();ctx.event("WORKING_VIEW_COMPRESSED",&json!({"view":rolling}));
                    }
                    ctx.event("WORKING_VIEW_COMMIT",&json!({"conversation":conversation,"view":rolling}));
                }
                recent.push(json!({"utterance":event["utterance"],"adopted_targets":answer["targets"],"text":answer["text"],"source_revisions":answer["source_revisions"]}));
                if recent.len()>3 {recent.remove(0);}
            }
            _=>return Err(format!("unknown dp01 event {}",event["kind"])),
        }
    }
    Ok(json!({"adoptions":adoptions,"outputs":outputs,"handoffs":[],"production_state":published,"working_view":rolling,"fixture_class":"CONSTRUCTED_SYNTHETIC_POC"}))
}
