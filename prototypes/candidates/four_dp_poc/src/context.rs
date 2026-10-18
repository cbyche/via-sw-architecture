use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::{mpsc, Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::time::Instant;

struct Inner {
    pending: Mutex<HashMap<u64,mpsc::Sender<Value>>>,
    stdout: Mutex<io::Stdout>,
    sequence: AtomicU64,
    origin: Instant,
}
#[derive(Clone)]
pub struct Context { inner: Arc<Inner> }
impl Context {
    pub fn new() -> Self {
        let ctx = Self { inner: Arc::new(Inner {pending:Mutex::new(HashMap::new()),stdout:Mutex::new(io::stdout()),sequence:AtomicU64::new(1),origin:Instant::now()})};
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
    fn send(&self,value:&Value) {
        let mut out=self.inner.stdout.lock().unwrap();
        writeln!(out,"{}",value).unwrap(); out.flush().unwrap();
    }
    pub fn event(&self,kind:&str,data:&Value) {
        self.send(&json!({"type":"event","kind":kind,"elapsed_ns":self.inner.origin.elapsed().as_nanos() as u64,"data":data}));
    }
    pub fn call(&mut self,stage:&str,payload:&Value)->Result<Value,String> {
        let id=self.inner.sequence.fetch_add(1,Ordering::SeqCst);
        let (tx,rx)=mpsc::channel(); self.inner.pending.lock().unwrap().insert(id,tx);
        self.send(&crate::model_wire::encode(id,stage,payload));
        let response=rx.recv().map_err(|e|e.to_string())?;
        crate::model_wire::decode(&response)
    }
    pub fn present(&self,value:&Value) { self.event("DISPLAY_PROXY",value); }
    pub fn source(&self,source:&Value)->Result<Value,String> {
        let result=crate::source_contract::read(source)?;
        self.event("SOURCE_READ",&json!({"id":source["id"],"revision":source["revision"],"bytes":result.to_string().len()}));
        Ok(result)
    }
}
