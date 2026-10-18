use serde_json::{json, Value};
pub struct RevisionGate {
    revision: u64,
    snapshot: Value,
}
impl RevisionGate {
    pub fn new() -> Self {
        Self {
            revision: 0,
            snapshot: Value::Null,
        }
    }
    pub fn begin(&mut self, revision: u64, source: &Value) {
        self.revision = revision;
        self.snapshot=json!(source["frames"].as_array().unwrap_or(&Vec::new()).iter().map(|f|json!({"frame_id":f["frame_id"],"source_id":f["source_id"],"source_revision":f["source_revision"],"sha256":f["sha256"]})).collect::<Vec<_>>());
    }
    pub fn accept(&self, proposal: &Value) -> Result<(), String> {
        if proposal["revision"].as_u64() != Some(self.revision) {
            return Err("STALE_INPUT_REVISION".into());
        }
        if proposal["source_snapshot"] != self.snapshot {
            return Err("SOURCE_SNAPSHOT_MISMATCH".into());
        }
        Ok(())
    }
}
pub fn normalize_adoptions(proposal: &Value, input: &Value, rev: u64) -> Vec<Value> {
    input["expressions"].as_array().unwrap().iter().map(|e|{
        let a=proposal["adoptions"].as_array().and_then(|xs|xs.iter().find(|a|a["expression_id"]==e["expression_id"]));
        let targets=a.filter(|a|a["status"].as_str()==Some("ready")).and_then(|a|a["targets"].as_array()).cloned().unwrap_or_default();
        let mut strings:Vec<String>=targets.iter().filter_map(|v|v.as_str().map(String::from)).collect(); strings.sort(); strings.dedup();
        json!({"expression_id":e["expression_id"],"revision":rev,"targets":strings,"status":a.map(|a|a["status"].clone()).unwrap_or(json!("missing")),"evidence":a.map(|a|a["evidence"].clone()).unwrap_or(Value::Null)})
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_previously_valid_proposal_after_revision_registration() {
        let source =
            json!({"frames":[{"frame_id":"f","source_id":"s","source_revision":1,"sha256":"h"}]});
        let snapshot = json!([{"frame_id":"f","source_id":"s","source_revision":1,"sha256":"h"}]);
        let old = json!({"revision":1,"source_snapshot":snapshot});
        let mut gate = RevisionGate::new();
        gate.begin(1, &source);
        assert!(gate.accept(&old).is_ok());
        gate.begin(2, &source);
        assert_eq!(gate.accept(&old), Err("STALE_INPUT_REVISION".into()));
    }
    #[test]
    fn unresolved_proposal_is_not_counted_as_adopted_targets() {
        let proposal = json!({"adoptions":[{"expression_id":"e","targets":["plausible"],"status":"ambiguous"}]});
        let input = json!({"expressions":[{"expression_id":"e"}]});
        let normalized = normalize_adoptions(&proposal, &input, 1);
        assert_eq!(normalized[0]["targets"], json!([]));
        assert_eq!(proposal["adoptions"][0]["targets"], json!(["plausible"]));
    }
    #[test]
    fn source_snapshot_mismatch_is_not_published() {
        let mut gate = RevisionGate::new();
        gate.begin(1, &json!({"frames":[]}));
        assert_eq!(
            gate.accept(&json!({"revision":1,"source_snapshot":[{"sha256":"different"}]})),
            Err("SOURCE_SNAPSHOT_MISMATCH".into())
        );
    }
}
