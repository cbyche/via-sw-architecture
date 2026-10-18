use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::{mpsc, Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::time::Instant;

struct Inner {
    pending: Mutex<HashMap<u64,mpsc::Sender<Value>>>,
    captures: Mutex<HashMap<String,u64>>,
    goal_starts: Mutex<HashMap<String,u64>>,
    stdout: Mutex<io::Stdout>,
    sequence: AtomicU64,
    origin: Instant,
}
#[derive(Default)]
struct Scope { goal_ids:Vec<String>, parents:Mutex<Vec<String>> }
impl Clone for Scope{fn clone(&self)->Self{Self{goal_ids:self.goal_ids.clone(),parents:Mutex::new(self.parents.lock().unwrap().clone())}}}
#[derive(Clone)]
pub struct Context { inner: Arc<Inner>, scope:Scope, session:usize, sharing:bool }
impl Context {
    pub fn new() -> Self {
        let ctx = Self { inner: Arc::new(Inner {pending:Mutex::new(HashMap::new()),captures:Mutex::new(HashMap::new()),goal_starts:Mutex::new(HashMap::new()),stdout:Mutex::new(io::stdout()),sequence:AtomicU64::new(1),origin:Instant::now()}),scope:Scope::default(),session:0,sharing:false};
        let reader = ctx.clone();
        std::thread::spawn(move || {
            for line in io::stdin().lock().lines().map_while(Result::ok) {
                if let Ok(value)=serde_json::from_str::<Value>(&line) {
                    if let Some(id)=value["call_id"].as_u64() {
                        if let Some(tx)=reader.inner.pending.lock().unwrap().remove(&id) { let _=tx.send(value); }
                    }
                }
            }
        });
        ctx
    }
    pub fn immutable_sharing(&self)->bool{self.sharing}
    pub fn for_session(&self,session:usize,sharing:bool)->Self{Self{inner:self.inner.clone(),scope:Scope::default(),session,sharing}}
    pub fn now_ns(&self)->u64 {self.inner.origin.elapsed().as_nanos() as u64}
    fn next(&self)->u64 {self.inner.sequence.fetch_add(1,Ordering::SeqCst)}
    fn send(&self,value:&Value) {
        let mut out=self.inner.stdout.lock().unwrap();
        write!(out,"{{\"session_id\":{},\"allocator_live_bytes\":{},\"allocator_peak_bytes\":{}",self.session,crate::allocation::live(),crate::allocation::peak()).unwrap();
        for (key,v) in value.as_object().unwrap(){write!(out,",").unwrap();serde_json::to_writer(&mut *out,key).unwrap();write!(out,":").unwrap();serde_json::to_writer(&mut *out,v).unwrap();}
        writeln!(out,"}}").unwrap();out.flush().unwrap();
    }
    pub fn event(&self,kind:&str,data:&Value) {
        self.send(&json!({"type":"event","event_id":format!("event-{}",self.next()),"kind":kind,"elapsed_ns":self.now_ns(),"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"data":data}));
    }
    pub fn scoped(&self,goal_ids:&[String],parents:&[String])->Self {
        Self{inner:self.inner.clone(),scope:Scope{goal_ids:goal_ids.to_vec(),parents:Mutex::new(parents.to_vec())},session:self.session,sharing:self.sharing}
    }
    fn parents(&self)->Vec<String>{self.scope.parents.lock().unwrap().clone()}
    pub fn last_node(&self)->Option<String>{self.parents().last().cloned()}
    pub fn join(&self,parents:&[String]){*self.scope.parents.lock().unwrap()=parents.to_vec();self.event("CAUSAL_JOIN",&json!({"parents":parents}));}
    pub fn call(&mut self,stage:&str,payload:&Value)->Result<Value,String> {
        let id=self.next();let node=format!("operation-{}",id);
        let role=payload["role"].as_str().unwrap_or("VIA");
        let trace=json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role});
        self.event("OPERATION_START",&trace);
        let mut wire_payload=payload.clone();wire_payload["_trace"]=trace.clone();
        let (tx,rx)=mpsc::channel(); self.inner.pending.lock().unwrap().insert(id,tx);
        self.logical_buffer(&format!("model_request_{}",id),wire_payload.to_string().len(),"ALLOCATE");
        self.send(&crate::model_wire::encode(id,stage,&wire_payload));
        let response=rx.recv().map_err(|e|e.to_string())?;
        self.logical_buffer(&format!("model_request_{}",id),0,"RELEASE");
        let decoded=crate::model_wire::decode(&response);
        self.event("OPERATION_END",&json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role,"status":if decoded.is_ok(){"RESPONSE_RECEIVED"}else{"FAILED"},"response_logical_bytes":response.to_string().len()}));
        *self.scope.parents.lock().unwrap()=vec![node];
        decoded
    }
    pub fn call_with_sources(&mut self,stage:&str,payload:&Value,sources:&Value)->Result<Value,String>{
        if !self.sharing{return self.call(stage,payload)}
        let id=self.next();let node=format!("operation-{}",id);let role=payload["role"].as_str().unwrap_or("VIA");
        let trace=json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role});
        self.event("OPERATION_START",&trace);let (tx,rx)=mpsc::channel();self.inner.pending.lock().unwrap().insert(id,tx);
        self.logical_buffer(&format!("model_request_{}",id),crate::model_wire::borrowed_payload_len(payload,&trace,sources),"ALLOCATE");
        {let mut out=self.inner.stdout.lock().unwrap();write!(out,"{{\"session_id\":{},\"allocator_live_bytes\":{},\"allocator_peak_bytes\":{},",self.session,crate::allocation::live(),crate::allocation::peak()).unwrap();crate::model_wire::write_borrowed_fields(&mut *out,id,stage,payload,&trace,sources).unwrap();writeln!(out,"}}").unwrap();out.flush().unwrap();}
        let response=rx.recv().map_err(|e|e.to_string())?;self.logical_buffer(&format!("model_request_{}",id),0,"RELEASE");
        let decoded=crate::model_wire::decode(&response);self.event("OPERATION_END",&json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role,"status":if decoded.is_ok(){"RESPONSE_RECEIVED"}else{"FAILED"},"response_logical_bytes":response.to_string().len()}));
        *self.scope.parents.lock().unwrap()=vec![node];decoded
    }
    pub fn present(&self,value:&Value) { self.event("DISPLAY_PROXY",value); }
    pub fn capture(&self,stimulus_id:&str,data:&Value){
        let now=self.now_ns();let at=*self.inner.captures.lock().unwrap().entry(format!("{}:{}",self.session,stimulus_id)).or_insert(now);
        let mut d=data.clone();d["stimulus_id"]=json!(stimulus_id);d["input_elapsed_ns"]=json!(at);
        self.event("INPUT_CAPTURED",&d);self.event("STIMULUS",&d);
    }
    pub fn goal_start(&self,goal_id:&str,data:&Value){
        let stimulus=data["stimulus_id"].as_str().unwrap_or(goal_id);
        let capture_key=format!("{}:{}",self.session,stimulus);let goal_key=format!("{}:{}",self.session,goal_id);
        if !self.inner.captures.lock().unwrap().contains_key(&capture_key){self.capture(stimulus,data);}
        let at=self.inner.captures.lock().unwrap().get(&capture_key).copied().unwrap_or(self.now_ns());
        let mut starts=self.inner.goal_starts.lock().unwrap();let earliest=starts.entry(goal_key).or_insert(at);*earliest=(*earliest).min(at);let at=*earliest;drop(starts);
        let mut d=data.clone();d["goal_id"]=json!(goal_id);d["input_elapsed_ns"]=json!(at);
        self.event("GOAL_START",&d);
    }
    pub fn progress(&self,goal_id:&str,stimulus_id:&str,adoption:&Value)->Result<(),String>{
        validate_progress(adoption)?;
        let mut d=adoption.clone();d["goal_id"]=json!(goal_id);d["stimulus_id"]=json!(stimulus_id);d["content_type"]=json!("ADOPTED_HANDLING");
        self.event("ADOPTION_COMMITTED",&d);self.present(&d);Ok(())
    }
    pub fn complete(&self,goal_id:&str,result:&Value)->Result<(),String>{
        validate_completion(result)?;
        let mut d=result.clone();d["goal_id"]=json!(goal_id);d["content_type"]=json!("DETAILED_RESULT");
        self.present(&d);self.event("GOAL_COMPLETE",&d);Ok(())
    }
    pub fn fail(&self,goal_id:&str,reason:&Value){
        let mut d=reason.clone();if !d.is_object(){d=json!({"reason":reason});}d["goal_id"]=json!(goal_id);self.event("GOAL_FAILED",&d);
    }
    pub fn logical_buffer(&self,name:&str,bytes:usize,action:&str){
        self.event("LOGICAL_BUFFER",&json!({"name":name,"bytes":bytes,"action":action,"unit":"serialized_UTF8_bytes","not_rss":true}));
    }
    pub fn source(&self,source:&Value)->Result<Value,String> {
        let node=format!("source-{}",self.next());
        self.event("LOCAL_OPERATION_START",&json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":"source_read"}));
        let result=crate::source_contract::read(source);
        self.event("SOURCE_READ",&json!({"id":source["id"],"revision":source["revision"],"bytes":result.as_ref().map(|v|v.to_string().len()).unwrap_or(0),"status":if result.is_ok(){"OK"}else{"FAILED"}}));
        self.event("LOCAL_OPERATION_END",&json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":"source_read"}));
        *self.scope.parents.lock().unwrap()=vec![node];
        result
    }
}
fn normalized(s:&str)->String{s.split_whitespace().collect::<String>().to_lowercase()}
pub fn validate_progress(v:&Value)->Result<(),String>{
    let text=v["text"].as_str().filter(|s|!s.trim().is_empty()).ok_or("progress text missing")?;
    if v["resolved"]!=true{return Err("progress is not an actual resolved adoption".into())}
    if v["adoption_id"].as_str().filter(|s|!s.is_empty()).is_none(){return Err("adoption_id missing".into())}
    if !v["targets"].is_array() || !v["source_revisions"].is_object(){return Err("progress targets/source revisions malformed".into())}
    for field in ["scope","handling"]{if v.get(field).is_none_or(|x|x.is_null() || x.as_str()==Some("")){return Err(format!("progress {} missing",field))}}
    if let Some(raw)=v["raw_utterance"].as_str(){if normalized(text)==normalized(raw){return Err("progress is an unprocessed utterance echo".into())}}
    if ["ok","okay","processing","working","네","처리중입니다"].contains(&normalized(text).as_str()){return Err("generic acknowledgement is not substantive".into())}
    Ok(())
}
pub fn validate_completion(v:&Value)->Result<(),String>{
    if v["text"].as_str().filter(|s|!s.trim().is_empty()).is_none(){return Err("completion text missing/null".into())}
    if v["content_contract_checked"]!=true{return Err("completion host contract not checked".into())}
    if let Some(raw)=v["raw_utterance"].as_str(){if normalized(v["text"].as_str().unwrap())==normalized(raw){return Err("completion is an unprocessed utterance echo".into())}}
    Ok(())
}
#[cfg(test)]mod tests{use super::*;#[test]fn empty_completion_rejected(){assert!(validate_completion(&json!({"text":null,"content_contract_checked":true})).is_err());}#[test]fn echo_progress_rejected(){assert!(validate_progress(&json!({"text":"Read this","raw_utterance":"Read this","resolved":true,"adoption_id":"a","targets":[],"scope":"s","handling":"h","source_revisions":{}})).is_err());}#[test]fn concrete_progress_accepted(){assert!(validate_progress(&json!({"text":"I will compare Plan A and Plan B using D1 revision 1.","resolved":true,"adoption_id":"a","targets":["D1"],"scope":"cost and recovery","handling":"compare","source_revisions":{"D1":1}})).is_ok());}}
