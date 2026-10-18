//! C2 exact receiver-serving cache; selection remains the actual receiver model's.
use super::sources::*;use crate::context::Context;use serde_json::{json,Value};use std::collections::BTreeMap;
#[derive(Default)]pub struct Cache{pub enabled:bool,entries:BTreeMap<String,Value>,pub hits:u64,pub misses:u64,pub denied:u64}
impl Cache{
 fn key(s:&Value,lo:u64,hi:u64,epoch:&Value,recipient:&Value)->String{json!([s["id"],s["revision"],lo,hi,epoch,recipient]).to_string()}
 pub fn clear(&mut self){self.entries.clear();}
 pub fn serve(&mut self,ctx:&Context,requests:&Value,sources:&Sources,epoch:&Value,recipient:&Value)->(Vec<Value>,Vec<Value>){
  if !self.enabled{return selected_reads(ctx,requests,sources,"RECEIVER");}
  let mut out=Vec::new();let mut denied=Vec::new();for request in array(requests).into_iter().take(8){let start=std::time::Instant::now();let sid=request["source_id"].as_str().unwrap_or("");let lo=request["start_line"].as_u64().unwrap_or(0);let hi=request["end_line"].as_u64().unwrap_or(0);
   let result=if let Some(s)=sources.get(sid){let wanted=json!({"source_id":s["id"],"revision":s["revision"],"start_line":lo,"end_line":hi});
    if lo==0||!valid_span(&wanted,sources){self.denied+=1;Err("DENIED_CURRENT_ACCESS_OR_RANGE".into())}else{let key=Self::key(s,lo,hi,epoch,recipient);if let Some(p)=self.entries.get(&key){self.hits+=1;ctx.event("C2_EXACT_READ_HIT",&json!({"request":request,"access_epoch":epoch,"recipient":recipient,"serving_ns":start.elapsed().as_nanos()as u64}));Ok(p.clone())}else{self.misses+=1;let result=read(ctx,s,lo,hi,"RECEIVER");if let Ok(ref p)=result{self.entries.insert(key,p.clone());}result}}
   }else{self.denied+=1;Err("NO_MATCH".into())};match result{Ok(p)=>out.push(p),Err(reason)=>denied.push(json!({"request":request,"reason":reason}))};ctx.event("C2_READ_SERVICE",&json!({"request":request,"serving_ns":start.elapsed().as_nanos()as u64}));
  }ctx.logical_buffer("dp05_C2_exact_cache",self.entries.values().map(|p|p.to_string().len()).sum(),"STATE");(out,denied)
 }
 pub fn stats(&self)->Value{json!({"hits":self.hits,"misses":self.misses,"denied":self.denied,"entries":self.entries.len(),"retained_bytes":self.entries.values().map(|p|p.to_string().len()).sum::<usize>()})}
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn exact_key_separates_all_authority_fields(){let s=json!({"id":"s","revision":1});let key=Cache::key(&s,1,2,&json!(1),&json!("r"));for(lo,hi,epoch,recipient)in[(2,2,json!(1),json!("r")),(1,3,json!(1),json!("r")),(1,2,json!(2),json!("r")),(1,2,json!(1),json!("other"))]{assert_ne!(key,Cache::key(&s,lo,hi,&epoch,&recipient));}assert_ne!(key,Cache::key(&json!({"id":"s","revision":2}),1,2,&json!(1),&json!("r")));}
}
