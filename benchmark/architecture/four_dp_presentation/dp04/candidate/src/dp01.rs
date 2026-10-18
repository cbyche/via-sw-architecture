//! Evidence lifecycle: conversation view A, request bundle B, published relationships C.
use crate::context::Context;
use serde_json::{json,Value};
#[path="dp01_evidence.rs"] mod evidence;
#[path="dp01_producer.rs"] mod producer;
#[path="dp01_request.rs"] mod request;
use evidence::{array,Sources};
pub fn run(input:&Value,ctx:&mut Context)->Result<Value,String>{
 let variant=input["variant"].as_str().unwrap_or("A");let mut sources=Sources::new();
 let mut rolling=Vec::new();let mut recent=Vec::new();let mut published=Value::Null;let mut publication_revision=0;
 let mut conversation=String::new();let mut adoptions=Vec::new();let mut outputs=Vec::new();
 for event in input["events"].as_array().ok_or("events missing")?{match event["kind"].as_str().unwrap_or(""){
  "source"=>{let s=event["source"].clone();let sid=s["id"].as_str().ok_or("source id")?.to_owned();
   let changed=sources.get(&sid).map(|old|old["revision"]!=s["revision"]||old["revoked"]!=s["revoked"]).unwrap_or(false);
   sources.insert(sid.clone(),s);ctx.event("SOURCE_ACTIVITY",event);
   if changed{rolling.clear();published=Value::Null;recent.retain(|r:&Value|!array(&r["source_revisions"]).iter().any(|p|p["source_id"].as_str()==Some(&sid)));ctx.event("DERIVED_STATE_INVALIDATED",&json!({"source_id":sid,"scope":"whole_view_or_publication"}));}
   if variant=="C"{publication_revision+=1;let mut prep=ctx.scoped(&[],&[]);prep.event("PREPARATION_START",&json!({"preparation_id":format!("dp01-publication-{}",publication_revision),"future_query_available":false}));published=producer::produce(&mut prep,&sources,event,publication_revision)?;prep.event("PREPARATION_COMPLETE",&json!({"preparation_id":format!("dp01-publication-{}",publication_revision)}));}
  }
  "request"=>{let conv=event["conversation"].as_str().unwrap_or("default");
   if conversation!=conv{rolling.clear();recent.clear();conversation=conv.to_owned();ctx.event("CONVERSATION_BOUNDARY",&json!({"conversation":conv,"working_view_reset":true}));}
   let r=request::execute(ctx,event,variant,&sources,&rolling,&recent,&published)?;
   adoptions.extend(array(&r["adoptions"]));outputs.push(r["output"].clone());
   if r["completed"].as_bool()==Some(true){
    if variant=="A"{for p in array(&r["read_passages"]){if !rolling.contains(&p){rolling.push(p);}}
     while rolling.iter().map(|p|p.to_string().len()).sum::<usize>()>14000&&rolling.len()>1{rolling.remove(0);ctx.event("WORKING_VIEW_BUDGET_EVICTION",&json!({"policy":"oldest_passage","budget_bytes":14000}));}
     ctx.event("WORKING_VIEW_COMMIT",&json!({"conversation":conv,"view":rolling}));}
    recent.push(r["delivered_answer"].clone());if recent.len()>3{recent.remove(0);}
   }
  }
  _=>return Err(format!("unknown dp01 event {}",event["kind"])),
 }ctx.logical_buffer("dp01_sources",sources.values().map(|v|v.to_string().len()).sum(),"STATE");ctx.logical_buffer("dp01_working_view",serde_json::to_vec(&rolling).unwrap().len(),"STATE");ctx.logical_buffer("dp01_published",published.to_string().len(),"STATE");ctx.logical_buffer("dp01_recent",serde_json::to_vec(&recent).unwrap().len(),"STATE");}
 Ok(json!({"adoptions":adoptions,"outputs":outputs,"handoffs":[],"production_state":published,"working_view":rolling,"fixture_class":"CONSTRUCTED_SYNTHETIC_POC"}))
}
