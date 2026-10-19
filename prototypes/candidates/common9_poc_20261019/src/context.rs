use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc, Mutex,
};
use std::time::Instant;

struct Inner {
    effects: crate::lifecycle::EffectGate,
    pending: Mutex<HashMap<u64, mpsc::Sender<Value>>>,
    publication_pending: Mutex<HashMap<String, (String, mpsc::Sender<Value>)>>,
    captures: Mutex<HashMap<String, u64>>,
    goal_starts: Mutex<HashMap<String, u64>>,
    stdout: Mutex<io::Stdout>,
    sequence: AtomicU64,
    origin: Instant,
    ingress: Mutex<mpsc::Receiver<Value>>,
    ingress_sender: mpsc::Sender<Value>,
}

#[cfg(test)]
mod effect_io_tests {
    use super::*;
    #[test]
    fn blocked_stdout_cannot_delay_state_write_past_admission_check() {
        let ctx=Context::new();let output=ctx.inner.stdout.lock().unwrap();
        let deadline=crate::lifecycle::host_monotonic_ns()+500_000_000;ctx.test_deadline(deadline);
        let state=Arc::new(Mutex::new(false));let held=state.lock().unwrap();let c=ctx.clone();let changed=state.clone();
        let(ready_tx,ready_rx)=mpsc::channel();let(done_tx,done_rx)=mpsc::channel();
        let worker=std::thread::spawn(move||{
            let effect=c.effect_guard().unwrap();ready_tx.send(()).unwrap();let mut value=changed.lock().unwrap();
            let result=(||{let _commit=c.commit_effect(&effect,"TEST_BOUNDED_ASSIGNMENT",&json!({}))?;*value=true;Ok::<(),String>(())})();
            done_tx.send(result).unwrap();
        });
        ready_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        std::thread::sleep(std::time::Duration::from_nanos(deadline.saturating_sub(crate::lifecycle::host_monotonic_ns())+1_000_000));drop(held);
        // This must finish while stdout is STILL blocked: no logging can occur
        // between the post-owning-lock clock check and the rejected mutation.
        let result=done_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();assert!(result.unwrap_err().contains("OBSERVATION_DEADLINE"));
        assert!(!*state.lock().unwrap());drop(output);worker.join().unwrap();
    }
    #[test]
    fn cancellation_signal_reaches_already_acquired_effect_guard() {
        let ctx=Context::new();let effect=ctx.effect_guard().unwrap();
        ctx.inner.effects.cancel();
        assert!(ctx.commit_effect(&effect,"TEST_MUST_NOT_COMMIT",&json!({})).is_err());
    }
}
#[derive(Default)]
struct Scope {
    goal_ids: Vec<String>,
    parents: Mutex<Vec<String>>,
}
impl Clone for Scope {
    fn clone(&self) -> Self {
        Self {
            goal_ids: self.goal_ids.clone(),
            parents: Mutex::new(self.parents.lock().unwrap().clone()),
        }
    }
}
#[derive(Clone)]
pub struct Context {
    inner: Arc<Inner>,
    scope: Scope,
    boundary: Option<Arc<dyn Fn(Vec<String>) -> Vec<String> + Send + Sync>>,
}
/// Pure admission fact. Audit I/O occurs on drop, after the caller's bounded
/// in-memory mutation, never between the final clock check and that mutation.
pub struct EffectCommit {
    ctx: Context,
    kind: String,
    data: Value,
    at: u64,
    deadline: Option<u64>,
}
impl Drop for EffectCommit {
    fn drop(&mut self) {
        self.ctx.event("STATE_EFFECT_COMMIT", &json!({"effect_kind":self.kind,"linearization_ns":self.at,"deadline_ns":self.deadline,"data":self.data,"audit_emission":"AFTER_BOUNDED_STATE_MUTATION"}));
    }
}
impl Context {
    pub fn new() -> Self {
        let (ingress_sender, ingress) = mpsc::channel();
        let ctx = Self {
            inner: Arc::new(Inner {
                effects: crate::lifecycle::EffectGate::default(),
                pending: Mutex::new(HashMap::new()),
                publication_pending: Mutex::new(HashMap::new()),
                captures: Mutex::new(HashMap::new()),
                goal_starts: Mutex::new(HashMap::new()),
                stdout: Mutex::new(io::stdout()),
                sequence: AtomicU64::new(1),
                origin: Instant::now(),
                ingress: Mutex::new(ingress),
                ingress_sender,
            }),
            scope: Scope::default(),
            boundary: None,
        };
        let reader = ctx.clone();
        std::thread::spawn(move || {
            for line in io::stdin().lock().lines().map_while(Result::ok) {
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    if value["type"] == "publication_receipt" {
                        let publication=value["publication_id"].as_str().unwrap_or("");
                        let mut pending=reader.inner.publication_pending.lock().unwrap();
                        let valid=pending.get(publication).is_some_and(|(event,_)|value["display_event_id"]==event.as_str()&&value["observed_ns"].as_u64().is_some()&&value["accepted"].is_boolean());
                        if valid { let (_,tx)=pending.remove(publication).unwrap();let _=tx.send(value); }
                        else {drop(pending);reader.event("PUBLICATION_RECEIPT_REJECTED",&json!({"receipt":value,"reason":"UNKNOWN_DUPLICATE_OR_MALFORMED_PUBLICATION_RECEIPT"}));}
                        continue;
                    }
                    if value["type"] == "clock_probe" {
                        reader.event("HOST_CLOCK_PROBE_ACK", &json!({"probe_id":value["host_probe_id"],"host_monotonic_ns":crate::lifecycle::host_monotonic_ns(),"clock":"mach_absolute_time/timebase"}));
                        continue;
                    }
                    if value["type"] == "observation_deadline" {
                        let result = value["deadline_ns"].as_u64().ok_or("deadline absent".to_owned()).and_then(|deadline| reader.inner.effects.update(deadline));
                        reader.event("OBSERVATION_DEADLINE_ACK", &json!({"control":value,"host_monotonic_ns":crate::lifecycle::host_monotonic_ns(),"update_admitted_ns":result.as_ref().ok().copied(),"accepted":result.is_ok(),"error":result.err()}));
                        continue;
                    }
                    if value["type"] == "cancel_work" {
                        let at = reader.inner.effects.cancel();
                        reader.event("WORK_CANCELLATION_ACK", &json!({"control":value,"host_monotonic_ns":at}));
                        let _ = reader.inner.ingress_sender.send(value);
                        continue;
                    }
                    if value["type"] == "input_event" || value["type"] == "end_input" {
                        let _ = reader.inner.ingress_sender.send(value);
                        continue;
                    }
                    if let Some(id) = value["call_id"].as_u64() {
                        if let Some(tx) = reader.inner.pending.lock().unwrap().remove(&id) {
                            let _ = tx.send(value);
                        }
                    }
                }
            }
        });
        ctx
    }
    pub fn set_boundary(&mut self, hook: Arc<dyn Fn(Vec<String>) -> Vec<String> + Send + Sync>) {
        self.boundary = Some(hook);
    }
    pub fn effect_guard(&self) -> Result<std::sync::MutexGuard<'_, crate::lifecycle::EffectState>, String> { self.inner.effects.guard() }
    pub fn check_active(&self) -> Result<(), String> { self.effect_guard().map(|_| ()) }
    #[cfg(test)]
    pub fn test_deadline(&self, deadline: u64) { self.inner.effects.update(deadline).unwrap(); }
    pub fn check_effect(&self, guard: &crate::lifecycle::EffectState) -> Result<(), String> { guard.check_at(crate::lifecycle::host_monotonic_ns()) }
    pub fn commit_effect(&self, guard: &crate::lifecycle::EffectState, kind: &str, data: &Value) -> Result<EffectCommit, String> {
        // Clone all potentially locking/allocating audit data before the final
        // check. Keeping the returned receipt bound until after mutation is
        // required; source registry methods retain it through their inserts.
        let ctx=self.without_boundary();let kind=kind.to_owned();let data=data.clone();
        let at = crate::lifecycle::host_monotonic_ns();
        guard.check_at(at)?;
        Ok(EffectCommit{ctx,kind,data,at,deadline:guard.deadline_ns})
    }
    pub fn request_publication(&self, value: &Value) -> Result<Value, String> {
        self.request_publication_checked(value, || Ok(()))
    }
    pub fn request_publication_checked(&self, value: &Value, check: impl FnOnce() -> Result<(),String>) -> Result<Value, String> {
        let publication=format!("publication-{}",self.next());let event_id=format!("event-{}",self.next());
        let mut data=value.clone();data["publication_id"]=json!(publication);
        let prepared=crate::lifecycle::host_monotonic_ns();
        let event=json!({"type":"event","event_id":event_id,"kind":"DISPLAY_PROXY","elapsed_ns":self.now_ns(),"host_monotonic_ns":prepared,"prepared_host_monotonic_ns":prepared,"host_clock_semantics":"EVENT_ASSEMBLY_BEFORE_STDOUT_LOCK_NOT_DISPATCH_OR_DELIVERY","goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"data":data});
        let serialized=event.to_string();let(tx,rx)=mpsc::channel();
        self.inner.publication_pending.lock().unwrap().insert(publication.clone(),(event_id.clone(),tx));
        let sent=(||{
            let effect=self.effect_guard()?;
            // Runtime authority/revision checks execute under the shared effect
            // gate; all authorized mutations use that gate until this dispatch.
            check()?;
            let mut out=self.inner.stdout.lock().unwrap();
            let admitted=crate::lifecycle::host_monotonic_ns();effect.check_at(admitted)?;
            writeln!(out,"{}",serialized).map_err(|e|e.to_string())?;out.flush().map_err(|e|e.to_string())?;
            drop(out);drop(effect);
            self.event("PUBLICATION_DISPATCH_ADMITTED",&json!({"publication_id":publication,"display_event_id":event_id,"linearization_ns":admitted,"semantics":"HOST_CALLBACK_IPC_CALL_INITIATION_NOT_HOST_DELIVERY"}));
            Ok::<u64,String>(admitted)
        })();
        let admitted=match sent {Ok(at)=>at,Err(error)=>{self.inner.publication_pending.lock().unwrap().remove(&publication);return Err(error);}};
        // No effect/source/state/stdout lock is held during receipt waiting.
        let receipt=rx.recv_timeout(std::time::Duration::from_secs(5));
        let receipt=match receipt {Ok(value)=>value,Err(error)=>{self.inner.publication_pending.lock().unwrap().remove(&publication);self.event("PUBLICATION_RECEIPT_TIMEOUT",&json!({"publication_id":publication,"display_event_id":event_id,"error":error.to_string()}));return Err("PUBLICATION_HOST_RECEIPT_NOT_OBSERVED".into());}};
        self.event("PUBLICATION_HOST_RECEIPT",&receipt);
        let observed=receipt["observed_ns"].as_u64().ok_or("publication receipt clock absent")?;
        if observed<admitted || observed>crate::lifecycle::host_monotonic_ns() || receipt["observation_deadline_ns"].as_u64().is_some_and(|cutoff|observed>cutoff) {
            return Err("PUBLICATION_HOST_RECEIPT_CLOCK_OR_CUTOFF_INVALID".into());
        }
        if receipt["accepted"]!=true {return Err("PUBLICATION_HOST_RECEIPT_OUTSIDE_OBSERVATION_WINDOW".into());}
        Ok(receipt)
    }
    pub fn without_boundary(&self) -> Self {
        let mut c = self.clone();
        c.boundary = None;
        c
    }
    pub fn drain_boundary(&self) {
        if let Some(hook) = self.boundary.clone() {
            let mut parents = self.parents();
            parents.extend(hook(self.parents()));
            if parents.len() > 1 {
                self.join(&parents);
            }
        }
    }
    pub fn local_operation<T>(&self, stage: &str, role: &str, action: impl FnOnce() -> T) -> T {
        let node = format!("local-{}", self.next());
        self.event("LOCAL_OPERATION_START",&json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role}));
        let result = action();
        self.event("LOCAL_OPERATION_END",&json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role}));
        *self.scope.parents.lock().unwrap() = vec![node];
        result
    }
    pub fn recv_input(
        &self,
        timeout: std::time::Duration,
    ) -> Result<Value, mpsc::RecvTimeoutError> {
        self.inner.ingress.lock().unwrap().recv_timeout(timeout)
    }
    pub fn now_ns(&self) -> u64 {
        self.inner.origin.elapsed().as_nanos() as u64
    }
    fn next(&self) -> u64 {
        self.inner.sequence.fetch_add(1, Ordering::SeqCst)
    }
    fn send(&self, value: &Value) {
        let mut out = self.inner.stdout.lock().unwrap();
        writeln!(out, "{}", value).unwrap();
        out.flush().unwrap();
    }
    pub fn event(&self, kind: &str, data: &Value) {
        self.send(&json!({"type":"event","event_id":format!("event-{}",self.next()),"kind":kind,"elapsed_ns":self.now_ns(),"host_monotonic_ns":crate::lifecycle::host_monotonic_ns(),"host_clock_semantics":"EVENT_ASSEMBLY_BEFORE_STDOUT_LOCK_NOT_DISPATCH_OR_DELIVERY","goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"data":data}));
    }
    pub fn scoped(&self, goal_ids: &[String], parents: &[String]) -> Self {
        Self {
            inner: self.inner.clone(),
            scope: Scope {
                goal_ids: goal_ids.to_vec(),
                parents: Mutex::new(parents.to_vec()),
            },
            boundary: self.boundary.clone(),
        }
    }
    fn parents(&self) -> Vec<String> {
        self.scope.parents.lock().unwrap().clone()
    }
    pub fn last_node(&self) -> Option<String> {
        self.parents().last().cloned()
    }
    pub fn join(&self, parents: &[String]) {
        *self.scope.parents.lock().unwrap() = parents.to_vec();
        self.event("CAUSAL_JOIN", &json!({"parents":parents}));
    }
    pub fn call(&mut self, stage: &str, payload: &Value) -> Result<Value, String> {
        self.check_active()?;
        let id = self.next();
        let node = format!("operation-{}", id);
        let role = payload["role"].as_str().unwrap_or("VIA");
        let trace = json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role});
        self.event("OPERATION_START", &trace);
        let mut wire_payload = payload.clone();
        wire_payload["_trace"] = trace.clone();
        let (tx, rx) = mpsc::channel();
        self.inner.pending.lock().unwrap().insert(id, tx);
        self.logical_buffer(
            &format!("model_request_{}", id),
            wire_payload.to_string().len(),
            "ALLOCATE",
        );
        {
            match self.effect_guard() {
                Ok(_effect) => self.send(&crate::model_wire::encode(id, stage, &wire_payload)),
                Err(error) => {
                    self.inner.pending.lock().unwrap().remove(&id);
                    self.logical_buffer(&format!("model_request_{}", id), 0, "RELEASE");
                    self.event("OPERATION_END", &json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role,"status":"CANCELLED_BEFORE_HOST_DISPATCH"}));
                    return Err(error);
                }
            }
        }
        let response = rx.recv().map_err(|e| e.to_string())?;
        self.logical_buffer(&format!("model_request_{}", id), 0, "RELEASE");
        let decoded = crate::model_wire::decode(&response);
        self.event("OPERATION_END",&json!({"node_id":node,"goal_ids":self.scope.goal_ids,"parent_ids":self.parents(),"stage":stage,"role":role,"status":if decoded.is_ok(){"RESPONSE_RECEIVED"}else{"FAILED"},"response_logical_bytes":response.to_string().len()}));
        *self.scope.parents.lock().unwrap() = vec![node];
        self.check_active()?;
        if role != "RECEIVER" {
            if let Some(hook) = self.boundary.clone() {
                let mut parents = self.parents();
                parents.extend(hook(self.parents()));
                if parents.len() > 1 {
                    self.join(&parents);
                }
            }
        }
        decoded
    }
    pub fn present(&self, value: &Value) {
        self.event("DISPLAY_PROXY", value);
    }
    pub fn capture(&self, stimulus_id: &str, data: &Value) {
        let now = self.now_ns();
        let at = *self
            .inner
            .captures
            .lock()
            .unwrap()
            .entry(stimulus_id.to_string())
            .or_insert(now);
        let mut d = data.clone();
        d["stimulus_id"] = json!(stimulus_id);
        d["input_elapsed_ns"] = json!(at);
        self.event("INPUT_CAPTURED", &d);
        self.event("STIMULUS", &d);
    }
    pub fn goal_start(&self, goal_id: &str, data: &Value) {
        let stimulus = data["stimulus_id"].as_str().unwrap_or(goal_id);
        if !self.inner.captures.lock().unwrap().contains_key(stimulus) {
            self.capture(stimulus, data);
        }
        let at = self
            .inner
            .captures
            .lock()
            .unwrap()
            .get(stimulus)
            .copied()
            .unwrap_or(self.now_ns());
        let mut starts = self.inner.goal_starts.lock().unwrap();
        let earliest = starts.entry(goal_id.to_string()).or_insert(at);
        *earliest = (*earliest).min(at);
        let at = *earliest;
        drop(starts);
        let mut d = data.clone();
        d["goal_id"] = json!(goal_id);
        d["input_elapsed_ns"] = json!(at);
        self.event("GOAL_START", &d);
    }
    pub fn progress(
        &self,
        goal_id: &str,
        stimulus_id: &str,
        adoption: &Value,
    ) -> Result<(), String> {
        validate_progress(adoption)?;
        let mut d = adoption.clone();
        d["goal_id"] = json!(goal_id);
        d["stimulus_id"] = json!(stimulus_id);
        d["content_type"] = json!("ADOPTED_HANDLING");
        self.event("ADOPTION_COMMITTED", &d);
        self.present(&d);
        Ok(())
    }
    pub fn complete(&self, goal_id: &str, result: &Value) -> Result<(), String> {
        validate_completion(result)?;
        let mut d = result.clone();
        d["goal_id"] = json!(goal_id);
        d["content_type"] = json!("DETAILED_RESULT");
        self.present(&d);
        self.event("GOAL_COMPLETE", &d);
        Ok(())
    }
    pub fn fail(&self, goal_id: &str, reason: &Value) {
        let mut d = reason.clone();
        if !d.is_object() {
            d = json!({"reason":reason});
        }
        d["goal_id"] = json!(goal_id);
        self.event("GOAL_FAILED", &d);
    }
    pub fn logical_buffer(&self, name: &str, bytes: usize, action: &str) {
        self.event("LOGICAL_BUFFER",&json!({"name":name,"bytes":bytes,"action":action,"unit":"serialized_UTF8_bytes","not_rss":true}));
    }
}
fn normalized(s: &str) -> String {
    s.split_whitespace().collect::<String>().to_lowercase()
}
pub fn validate_progress(v: &Value) -> Result<(), String> {
    let text = v["text"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or("progress text missing")?;
    if v["resolved"] != true {
        return Err("progress is not an actual resolved adoption".into());
    }
    if v["adoption_id"]
        .as_str()
        .filter(|s| !s.is_empty())
        .is_none()
    {
        return Err("adoption_id missing".into());
    }
    if !v["targets"].is_array() || !v["source_revisions"].is_object() {
        return Err("progress targets/source revisions malformed".into());
    }
    for field in ["scope", "handling"] {
        if v.get(field)
            .is_none_or(|x| x.is_null() || x.as_str() == Some(""))
        {
            return Err(format!("progress {} missing", field));
        }
    }
    if let Some(raw) = v["raw_utterance"].as_str() {
        if normalized(text) == normalized(raw) {
            return Err("progress is an unprocessed utterance echo".into());
        }
    }
    if ["ok", "okay", "processing", "working", "네", "처리중입니다"]
        .contains(&normalized(text).as_str())
    {
        return Err("generic acknowledgement is not substantive".into());
    }
    Ok(())
}
pub fn validate_completion(v: &Value) -> Result<(), String> {
    if v["text"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .is_none()
    {
        return Err("completion text missing/null".into());
    }
    if v["content_contract_checked"] != true {
        return Err("completion host contract not checked".into());
    }
    if let Some(raw) = v["raw_utterance"].as_str() {
        if normalized(v["text"].as_str().unwrap()) == normalized(raw) {
            return Err("completion is an unprocessed utterance echo".into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_completion_rejected() {
        assert!(
            validate_completion(&json!({"text":null,"content_contract_checked":true})).is_err()
        );
    }
    #[test]
    fn echo_progress_rejected() {
        assert!(validate_progress(&json!({"text":"Read this","raw_utterance":"Read this","resolved":true,"adoption_id":"a","targets":[],"scope":"s","handling":"h","source_revisions":{}})).is_err());
    }
    #[test]
    fn concrete_progress_accepted() {
        assert!(validate_progress(&json!({"text":"I will compare Plan A and Plan B using D1 revision 1.","resolved":true,"adoption_id":"a","targets":["D1"],"scope":"cost and recovery","handling":"compare","source_revisions":{"D1":1}})).is_ok());
    }
}
