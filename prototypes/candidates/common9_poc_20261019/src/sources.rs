//! Versioned originals and independently granted local/external range authority.
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
#[derive(Clone, Debug)]
pub struct Passage {
    pub source_id: String,
    pub revision: u64,
    pub start_line: usize,
    pub end_line: usize,
    pub text: String,
}
impl Passage {
    pub fn to_value(&self) -> Value {
        json!({"source_id":self.source_id,"revision":self.revision,"start_line":self.start_line,"end_line":self.end_line,"text":self.text})
    }
}
#[derive(Default)]
struct Registry {
    versions: BTreeMap<(String, u64), Value>,
    current: BTreeMap<String, u64>,
}
#[derive(Clone, Default)]
pub struct SourceRegistry {
    inner: Arc<Mutex<Registry>>,
    gate: Arc<Mutex<()>>,
}
impl SourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn guard(&self) -> std::sync::MutexGuard<'_, ()> {
        self.gate.lock().unwrap()
    }
    pub fn register(&self, source: Value) -> Result<(), String> {
        self.register_checked(source, || Ok(()))
    }
    pub fn register_checked<C>(&self, source: Value, commit_check: impl FnOnce() -> Result<C, String>) -> Result<(), String> {
        let gate = self.guard();
        self.register_guarded_checked(source, &gate, commit_check)
    }
    pub fn register_guarded_checked<C>(&self, mut source: Value, _gate: &std::sync::MutexGuard<'_, ()>, commit_check: impl FnOnce() -> Result<C, String>) -> Result<(), String> {
        let id = source["id"].as_str().ok_or("source id missing")?.to_owned();
        let revision = source["revision"]
            .as_u64()
            .ok_or("source revision missing")?;
        if let Some(path) = source["path"].as_str() {
            source["text"] = json!(std::fs::read_to_string(path).map_err(|e| e.to_string())?);
        }
        let text = source["text"].as_str().ok_or("original text missing")?;
        source["line_count"] = json!(text.lines().count());
        if source.get("allowed_segments").is_none() {
            let n = source["line_count"].as_u64().unwrap();
            source["allowed_segments"] = if source["local_allowed"] == false {
                json!([])
            } else {
                json!([[1, n]])
            };
        }
        if source.get("external_segments").is_none() {
            source["external_segments"] = if source["external_allowed"] == false {
                json!([])
            } else {
                source["allowed_segments"].clone()
            };
        }
        let mut s = self.inner.lock().unwrap();
        if let Some(oldrev) = s.current.get(&id) {
            if let Some(old) = s.versions.get(&(id.clone(), *oldrev)) {
                source["allowed_segments"] = old["allowed_segments"].clone();
                source["external_segments"] = old["external_segments"].clone();
            }
        }
        let _commit = commit_check()?;
        s.current.insert(id.clone(), revision);
        s.versions.insert((id, revision), source);
        Ok(())
    }
    pub fn current(&self, id: &str) -> Option<Value> {
        let s = self.inner.lock().unwrap();
        s.current
            .get(id)
            .and_then(|r| s.versions.get(&(id.to_owned(), *r)))
            .cloned()
    }
    pub fn version(&self, id: &str, revision: u64) -> Option<Value> {
        self.inner
            .lock()
            .unwrap()
            .versions
            .get(&(id.to_owned(), revision))
            .cloned()
    }
    pub fn catalog(&self) -> Value {
        let s = self.inner.lock().unwrap();
        json!(s.versions.values().map(|v|{let mut x=v.clone();x.as_object_mut().unwrap().remove("text");x.as_object_mut().unwrap().remove("path");if let Some(provenance)=x["provenance"].as_array(){x["provenance"]=json!(provenance.iter().map(|p|json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"]})).collect::<Vec<_>>());}x["is_current"]=json!(s.current.get(v["id"].as_str().unwrap())==v["revision"].as_u64().as_ref());let id=v["id"].as_str().unwrap();let revision=v["revision"].as_u64().unwrap();let mut references=Vec::new();if x["is_current"]==true{references.push(id.to_owned());}if let Some(extents)=v["extents"].as_object(){for name in extents.keys(){references.push(format!("{id}:v{revision}:{name}"));}}x["known_source_reference_strings"]=json!(references);x["reference_strings_are_permission_grants"]=json!(false);x["local_allowed"]=json!(x["allowed_segments"].as_array().is_some_and(|a|!a.is_empty()));x["external_allowed"]=json!(x["external_segments"].as_array().is_some_and(|a|!a.is_empty()));x}).collect::<Vec<_>>())
    }
    pub fn read(
        &self,
        id: &str,
        revision: u64,
        start_line: usize,
        end_line: usize,
        external: bool,
    ) -> Result<Passage, String> {
        if !self.allowed(id, revision, start_line, end_line, external) {
            return Err("RANGE_OR_PROVENANCE_NOT_AUTHORIZED".into());
        }
        let src = self
            .version(id, revision)
            .ok_or("source/version unavailable")?;
        if src["revoked"] == true {
            return Err("SOURCE_REVOKED".into());
        }
        let key = if external {
            "external_segments"
        } else {
            "allowed_segments"
        };
        let ranges = src[key].as_array().ok_or("permission ranges missing")?;
        if start_line == 0
            || end_line < start_line
            || !(start_line..=end_line).all(|line| {
                ranges.iter().any(|r| {
                    r[0].as_u64().is_some_and(|a| line >= a as usize)
                        && r[1].as_u64().is_some_and(|b| line <= b as usize)
                })
            })
        {
            return Err("RANGE_NOT_AUTHORIZED".into());
        }
        let lines = src["text"]
            .as_str()
            .ok_or("source text absent")?
            .lines()
            .collect::<Vec<_>>();
        if end_line > lines.len() {
            return Err("range beyond original".into());
        }
        Ok(Passage {
            source_id: id.into(),
            revision,
            start_line,
            end_line,
            text: lines[start_line - 1..end_line].join("\n"),
        })
    }
    pub fn validate_permission_batch(&self, operations: &Value) -> Result<(), String> {
        let s = self.inner.lock().unwrap();
        Self::permission_updates(&s, operations).map(|_| ())
    }
    fn permission_updates(
        s: &Registry,
        operations: &Value,
    ) -> Result<Vec<((String, u64), String, Value)>, String> {
        let operations = operations
            .as_array()
            .ok_or("permission_operations must be an array")?;
        let mut updates = Vec::new();
        for operation in operations {
            let id = operation["source_id"]
                .as_str()
                .ok_or("permission source_id must be a string")?;
            if !s.current.contains_key(id) {
                return Err(format!("permission source unavailable: {id}"));
            }
            if operation.get("read_segments").is_none()
                && operation.get("external_segments").is_none()
            {
                return Err("permission operation has no declared scope field".into());
            }
            for ((sid, revision), source) in &s.versions {
                if sid != id {
                    continue;
                }
                for (key, field) in [
                    ("read_segments", "allowed_segments"),
                    ("external_segments", "external_segments"),
                ] {
                    if let Some(value) = operation.get(key) {
                        let names = value.as_array().ok_or_else(|| {
                            format!("{key} must be an array of declared segment names")
                        })?;
                        let mut ranges = Vec::new();
                        for name in names {
                            let name = name.as_str().ok_or("segment name must be a string")?;
                            let range = source["segments"].get(name).ok_or_else(|| {
                                format!("segment not declared by source: {id} v{revision} {name}")
                            })?;
                            ranges.push(range.clone());
                        }
                        updates.push(((sid.clone(), *revision), field.into(), json!(ranges)));
                    }
                }
            }
        }
        Ok(updates)
    }
    pub fn permission_batch_checked(
        &self,
        operations: &Value,
        check: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        self.permission_batch_checked_at(operations, check, || Ok(()))
    }
    pub fn permission_batch_checked_at<C>(
        &self,
        operations: &Value,
        check: impl FnOnce() -> Result<(), String>,
        commit_check: impl FnOnce() -> Result<C, String>,
    ) -> Result<(), String> {
        let _gate = self.guard();
        // The full proposal is rechecked under the same source gate as the commit.
        // Build every update before mutating anything; failure cannot grant a prefix.
        check()?;
        let mut s = self.inner.lock().unwrap();
        let updates = Self::permission_updates(&s, operations)?;
        let _commit = commit_check()?;
        for (key, field, ranges) in updates {
            s.versions.get_mut(&key).unwrap()[field] = ranges;
        }
        Ok(())
    }
    pub fn permission(&self, operation: &Value) -> Result<(), String> {
        self.permission_batch_checked(&json!([operation]), || Ok(()))
    }
    pub fn allowed(
        &self,
        id: &str,
        revision: u64,
        start_line: usize,
        end_line: usize,
        external: bool,
    ) -> bool {
        let Some(src) = self.version(id, revision) else {
            return false;
        };
        if src["revoked"] == true
            || start_line == 0
            || end_line < start_line
            || end_line > src["line_count"].as_u64().unwrap_or(0) as usize
        {
            return false;
        }
        if let Some(provenance) = src["provenance"].as_array() {
            if !provenance.iter().all(|p| {
                p["source_id"].as_str().is_some_and(|pid| {
                    pid != id
                        && self.allowed(
                            pid,
                            p["revision"].as_u64().unwrap_or(0),
                            p["start_line"].as_u64().unwrap_or(0) as usize,
                            p["end_line"].as_u64().unwrap_or(0) as usize,
                            external,
                        )
                })
            }) {
                return false;
            }
        }
        let key = if external {
            "external_segments"
        } else {
            "allowed_segments"
        };
        src[key].as_array().is_some_and(|ranges| {
            (start_line..=end_line).all(|line| {
                ranges.iter().any(|r| {
                    r[0].as_u64().is_some_and(|a| line >= a as usize)
                        && r[1].as_u64().is_some_and(|b| line <= b as usize)
                })
            })
        })
    }
    pub fn valid_passage(&self, p: &Value, external: bool) -> bool {
        let Some(id) = p["source_id"].as_str() else {
            return false;
        };
        let Some(revision) = p["revision"].as_u64() else {
            return false;
        };
        let Some(a) = p["start_line"].as_u64() else {
            return false;
        };
        let Some(b) = p["end_line"].as_u64() else {
            return false;
        };
        self.read(id, revision, a as usize, b as usize, external)
            .is_ok_and(|actual| {
                p.get("text")
                    .and_then(Value::as_str)
                    .is_none_or(|text| actual.text == text)
            })
    }
}

#[cfg(test)]
mod derived_authority_tests {
    use super::*;
    #[test]
    fn actual_derived_artifact_read_and_export_follow_original_authority() {
        let sources = SourceRegistry::new();
        sources.register(json!({"id":"DOC","revision":1,"text":"original restricted fact","segments":{"body":[1,1]}})).unwrap();
        sources
            .register(json!({"id":"SAFE","revision":1,"text":"unaffected permitted original"}))
            .unwrap();
        let actual = sources.read("DOC", 1, 1, 1, false).unwrap().to_value();
        sources.register(json!({"id":"RESPONSE","revision":1,"text":"derived delivered explanation","provenance":[actual]})).unwrap();
        let response = sources.read("RESPONSE", 1, 1, 1, true).unwrap().to_value();
        sources.register(json!({"id":"TASK:result","revision":1,"text":"actual receiver artifact body","provenance":[response]})).unwrap();
        assert!(sources.read("TASK:result", 1, 1, 1, false).is_ok());
        assert!(sources.allowed("TASK:result", 1, 1, 1, true));
        sources
            .permission(&json!({"source_id":"DOC","read_segments":[],"external_segments":[]}))
            .unwrap();
        assert!(sources.read("TASK:result", 1, 1, 1, false).is_err());
        assert!(sources.read("TASK:result", 1, 1, 1, true).is_err());
        assert!(!sources.allowed("TASK:result", 1, 1, 1, false));
        assert!(!sources.allowed("TASK:result", 1, 1, 1, true));
        assert!(sources.read("SAFE", 1, 1, 1, false).is_ok());
        assert!(sources.read("SAFE", 1, 1, 1, true).is_ok());
    }
}

#[cfg(test)]
mod permission_batch_tests {
    use super::*;
    #[test]
    fn full_batch_is_pure_until_all_operations_validate() {
        let s = SourceRegistry::new();
        s.register(json!({"id":"PRIVATE","revision":1,"text":"private body","segments":{"body":[1,1]},"local_allowed":false,"external_allowed":false})).unwrap();
        let batch = json!([{"source_id":"PRIVATE","read_segments":["body"]},{"source_id":"PRIVATE","external_segments":["1-1"]}]);
        let before = s.catalog();
        assert!(s.validate_permission_batch(&batch).is_err());
        assert_eq!(s.catalog(), before);
        assert!(s.permission_batch_checked(&batch, || Ok(())).is_err());
        assert_eq!(s.catalog(), before);
        assert!(!s.allowed("PRIVATE", 1, 1, 1, false));
    }
    #[test]
    fn proposal_revalidated_inside_commit_gate_before_any_authority_change() {
        let s = SourceRegistry::new();
        s.register(json!({"id":"PRIVATE","revision":1,"text":"private body","segments":{"body":[1,1]},"local_allowed":false})).unwrap();
        let before = s.catalog();
        let batch = json!([{"source_id":"PRIVATE","read_segments":["body"]}]);
        assert!(s.validate_permission_batch(&batch).is_ok());
        assert!(s
            .permission_batch_checked(&batch, || Err("actual proposal became invalid".into()))
            .is_err());
        assert_eq!(s.catalog(), before);
        s.permission_batch_checked(&batch, || Ok(())).unwrap();
        assert!(s.allowed("PRIVATE", 1, 1, 1, false));
    }
}

#[cfg(test)]
mod deadline_registration_tests {
    use super::*;
    use crate::{context::Context, lifecycle::host_monotonic_ns};
    #[test]
    fn register_rechecks_after_actual_registry_lock_wait() {
        let sources=SourceRegistry::new();let held=sources.inner.lock().unwrap();
        let ctx=Context::new();let deadline=host_monotonic_ns()+500_000_000;ctx.test_deadline(deadline);
        let registry=sources.clone();let c=ctx.clone();let(tx,rx)=std::sync::mpsc::channel();
        let worker=std::thread::spawn(move||{let effect=c.effect_guard().unwrap();tx.send(()).unwrap();registry.register_checked(json!({"id":"TOO-LATE","revision":1,"text":"must not be registered"}),||c.commit_effect(&effect,"SOURCE_REGISTER",&json!({"source_id":"TOO-LATE"})))});
        rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        let wait=deadline.saturating_sub(host_monotonic_ns())+1_000_000;std::thread::sleep(std::time::Duration::from_nanos(wait));drop(held);
        assert!(worker.join().unwrap().unwrap_err().contains("OBSERVATION_DEADLINE"));
        assert!(sources.current("TOO-LATE").is_none());
    }
    #[test]
    fn syntax_catalog_preserves_history_colons_and_denial() {
        let s=SourceRegistry::new();s.register(json!({"id":"TASK:artifact","revision":1,"text":"one","extents":{"body":[1,1]},"local_allowed":false,"external_allowed":false})).unwrap();
        s.register(json!({"id":"TASK:artifact","revision":2,"text":"two","extents":{"body":[1,1]}})).unwrap();
        let c=s.catalog();let rows=c.as_array().unwrap();
        let old=rows.iter().find(|r|r["revision"]==1).unwrap();let current=rows.iter().find(|r|r["revision"]==2).unwrap();
        assert_eq!(old["known_source_reference_strings"],json!(["TASK:artifact:v1:body"]));
        assert_eq!(current["known_source_reference_strings"],json!(["TASK:artifact","TASK:artifact:v2:body"]));
        assert_eq!(current["reference_strings_are_permission_grants"],false);
        assert!(!s.allowed("TASK:artifact",1,1,1,false));assert!(!s.allowed("TASK:artifact",2,1,1,true));
    }
}
