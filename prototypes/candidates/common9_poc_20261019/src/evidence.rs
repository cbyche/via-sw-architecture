//! DP1 evidence lifetime inside the common capture/goal lifecycle. No adoption or UI owner.
use crate::{api::EvidenceResult, context::Context, sources::SourceRegistry};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

pub fn array(v: &Value) -> Vec<Value> {
    v.as_array().cloned().unwrap_or_default()
}
pub fn span(v: &Value) -> Option<(&str, u64, usize, usize)> {
    Some((
        v["source_id"].as_str()?,
        v["revision"].as_u64()?,
        v["start_line"].as_u64()? as usize,
        v["end_line"].as_u64()? as usize,
    ))
}
pub fn permitted(sources: &SourceRegistry, p: &Value, external: bool) -> bool {
    span(p).is_some_and(|(id, r, a, b)| sources.allowed(id, r, a, b, external))
}
pub fn contained(w: &Value, p: &Value) -> bool {
    w["source_id"] == p["source_id"]
        && w["revision"] == p["revision"]
        && w["start_line"].as_u64().unwrap_or(0) >= p["start_line"].as_u64().unwrap_or(u64::MAX)
        && w["end_line"].as_u64().unwrap_or(u64::MAX) <= p["end_line"].as_u64().unwrap_or(0)
}
pub fn slice_passage(w: &Value, p: &Value) -> Option<Value> {
    if !contained(w, p) {
        return None;
    }
    let (_, _, a, b) = span(w)?;
    let (_, _, base, _) = span(p)?;
    let text = p["text"]
        .as_str()?
        .lines()
        .skip(a - base)
        .take(b - a + 1)
        .collect::<Vec<_>>()
        .join("\n");
    Some(
        json!({"source_id":w["source_id"],"revision":w["revision"],"start_line":a,"end_line":b,"text":text,"acquisition_node":p["acquisition_node"],"producer_node":p["producer_node"]}),
    )
}
pub fn read_ranges(
    ctx: &Context,
    sources: &SourceRegistry,
    ranges: &Value,
    external: bool,
    consumer: &str,
) -> (Vec<Value>, Vec<Value>) {
    let mut out = Vec::new();
    let mut denials = Vec::new();
    for r in array(ranges).into_iter().take(12) {
        let result = span(&r)
            .ok_or_else(|| "MALFORMED_SOURCE_RANGE".to_owned())
            .and_then(|(id, v, a, b)| {
                ctx.local_operation("source_range_read", "VIA", || {
                    sources.read(id, v, a, b, external)
                })
            });
        match result {
            Ok(p) => {
                let mut p = p.to_value();
                p["acquisition_node"] = json!(ctx.last_node());
                ctx.event(
                    "SOURCE_RANGE_READ",
                    &json!({"consumer":consumer,"external":external,"passage":p}),
                );
                out.push(p)
            }
            Err(error) => {
                let d = json!({"request":r,"error":error,"consumer":consumer,"external":external});
                ctx.event("SOURCE_RANGE_DENIED", &d);
                denials.push(d)
            }
        }
    }
    (out, denials)
}
pub fn source_ranges(sources: &SourceRegistry, external: bool) -> Vec<Value> {
    let mut ranges = Vec::new();
    for s in array(&sources.catalog()) {
        if s["is_current"] == false {
            continue;
        }
        for r in array(
            &s[if external {
                "external_segments"
            } else {
                "allowed_segments"
            }],
        ) {
            ranges.push(json!({"source_id":s["id"],"revision":s["revision"],"start_line":r[0],"end_line":r[1]}));
        }
    }
    ranges
}
/// Query-independent production budget; every authorized locator remains in the catalog.
/// Iterate all source ranges; no source-count or line-count sampling cap.
pub fn production_snapshot(
    ctx: &Context,
    sources: &SourceRegistry,
    consumer: &str,
) -> (Vec<Value>, Vec<Value>) {
    let mut output = Vec::new();
    let mut gaps = Vec::new();
    let mut bytes = 0usize;
    for r in source_ranges(sources, false) {
        let (p, d) = read_ranges(ctx, sources, &json!([r.clone()]), false, consumer);
        gaps.extend(d);
        for p in p {
            let (_, _, a, b) = span(&p).unwrap();
            let lines = p["text"].as_str().unwrap_or("").lines().collect::<Vec<_>>();
            let mut kept = Vec::new();
            for line in &lines {
                let add = line.len() + usize::from(!kept.is_empty());
                if bytes + add > 65536 {
                    break;
                }
                bytes += add;
                kept.push(*line);
            }
            if !kept.is_empty() {
                let end = a + kept.len() - 1;
                output.push(json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":a,"end_line":end,"text":kept.join("\n")}));
                if end < b {
                    gaps.push(json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":end+1,"end_line":b,"reason":"PRODUCTION_65536_BYTE_INPUT_BUDGET"}));
                }
            } else {
                gaps.push(json!({"range":r,"reason":"PRODUCTION_65536_BYTE_INPUT_BUDGET"}));
            }
        }
    }
    ctx.event("PRODUCTION_INPUT_COVERAGE",&json!({"consumer":consumer,"text_bytes":bytes,"budget_bytes":65536,"supplied_spans":output.iter().map(|p|json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"]})).collect::<Vec<_>>(),"gaps":gaps,"query_independent":true}));
    (output, gaps)
}
pub fn dedup_passages(passages: &mut Vec<Value>) {
    let mut seen = Vec::new();
    passages.retain(|p| {
        let k = json!([
            p["source_id"],
            p["revision"],
            p["start_line"],
            p["end_line"],
            p["text"]
        ]);
        if seen.contains(&k) {
            false
        } else {
            seen.push(k);
            true
        }
    })
}
fn records_valid(records: &Value, sources: &SourceRegistry, external: bool) -> Vec<Value> {
    array(records)
        .into_iter()
        .filter(|r| {
            !array(&r["source_spans"]).is_empty()
                && array(&r["source_spans"])
                    .iter()
                    .all(|p| permitted(sources, p, external))
                && array(&r["supporting_passages"])
                    .iter()
                    .all(|p| permitted(sources, p, external))
        })
        .collect()
}
pub fn record_support(records: &Value, sources: &SourceRegistry, external: bool) -> Vec<Value> {
    let mut p = Vec::new();
    for r in records_valid(records, sources, external) {
        p.extend(array(&r["supporting_passages"]));
    }
    dedup_passages(&mut p);
    p
}
pub fn support_records(
    proposal: &Value,
    evidence: &[Value],
    sources: &SourceRegistry,
    field: &str,
) -> Vec<Value> {
    let mut output = Vec::new();
    for mut record in array(&proposal[field]).into_iter().take(12) {
        let spans = array(&record["source_spans"]);
        if spans.is_empty()
            || !spans
                .iter()
                .all(|r| permitted(sources, r, false) && evidence.iter().any(|p| contained(r, p)))
        {
            continue;
        }
        let mut support = Vec::new();
        for r in &spans {
            if let Some(p) = evidence.iter().find(|p| contained(r, p)) {
                if let Some(p) = slice_passage(r, p) {
                    support.push(p);
                }
            }
        }
        record["supporting_passages"] = json!(support);
        output.push(record);
    }
    output
}
/// Original access changes also govern new use of source-dependent prior response text.
pub fn authorized_public_state(sources: &SourceRegistry, public_state: &Value) -> Value {
    let mut safe = public_state.clone();
    safe["delivered"] = json!(array(&public_state["delivered"])
        .into_iter()
        .filter(|r| array(&r["source_spans"])
            .iter()
            .all(|p| permitted(sources, p, false)))
        .collect::<Vec<_>>());
    for field in ["history", "captured_inputs"] {
        safe[field] = json!(array(&public_state[field])
            .into_iter()
            .map(|mut item| {
                if !array(&item["source_spans"])
                    .iter()
                    .all(|p| permitted(sources, p, false))
                {
                    item["text"] = json!("[source authority withdrawn]");
                    item["authority_redacted"] = json!(true);
                }
                item
            })
            .collect::<Vec<_>>());
    }
    let mut denied_tasks = Vec::new();
    safe["requests"]=json!(array(&public_state["requests"]).into_iter().map(|mut request|{
  let adoption=&request["adoption"];let claims=array(&adoption["source_claims"]);
  let cited=adoption["source_revisions"].as_object().is_some_and(|v|!v.is_empty());
  if (cited&&claims.is_empty())||!claims.iter().all(|p|permitted(sources,p,false)){
   if let Some(task)=request["task_id"].as_str(){denied_tasks.push(task.to_owned());}
   request["adoption"]=json!({"authority_redacted":true,"relation":adoption["relation"],"route":adoption["route"],"target_goal_id":adoption["target_goal_id"],"target_event_id":adoption["target_event_id"],"task_id":adoption["task_id"],"targets":adoption["targets"],"bindings":adoption["bindings"],"source_revisions":adoption["source_revisions"]});
   request.as_object_mut().unwrap().remove("question");
  }request
 }).collect::<Vec<_>>());
    safe["tasks"] = json!(array(&public_state["tasks"])
        .into_iter()
        .map(|mut task| {
            if task["task_id"]
                .as_str()
                .is_some_and(|id| denied_tasks.iter().any(|t| t == id))
                || !array(&task["source_spans"])
                    .iter()
                    .all(|p| permitted(sources, p, false))
            {
                task.as_object_mut().unwrap().remove("title");
                task.as_object_mut().unwrap().remove("questions");
                task.as_object_mut().unwrap().remove("last_result");
                task.as_object_mut().unwrap().remove("last_progress");
            }
            task
        })
        .collect::<Vec<_>>());
    safe
}
#[derive(Default)]
struct State {
    working: Vec<Value>,
    records: Vec<Value>,
    recent: Vec<Value>,
    latest_activity: u64,
    source_versions: BTreeMap<String, u64>,
}
#[derive(Clone)]
pub struct EvidenceStore {
    variant: String,
    state: Arc<Mutex<State>>,
}
impl EvidenceStore {
    pub fn new(variant: &str) -> Self {
        Self {
            variant: variant.into(),
            state: Arc::new(Mutex::new(State::default())),
        }
    }
    pub fn observe(
        &self,
        ctx: &mut Context,
        sources: &SourceRegistry,
        event: &Value,
        public_state: &Value,
    ) -> Result<(), String> {
        let safe_public = authorized_public_state(sources, public_state);
        let public_state = &safe_public;
        let seq = event["runtime_activity_seq"]
            .as_u64()
            .or_else(|| event["_activity_seq"].as_u64())
            .unwrap_or(public_state["revision_counter"].as_u64().unwrap_or(0));
        {
            let _effect = ctx.effect_guard()?;
            let mut state = self.state.lock().unwrap();
            let working=state.working.iter().filter(|p|permitted(sources,p,false)&&span(p).is_some_and(|(id,r,_,_)|sources.current(id).is_some_and(|source|source["revision"]==r))).cloned().collect();
            let records=records_valid(&json!(state.records),sources,false);
            let recent=array(&public_state["delivered"]).into_iter().rev().take(3).collect();
            let latest=state.latest_activity.max(seq);
            let _commit=ctx.commit_effect(&_effect,"PRODUCER_STATE",&json!({"owner":"evidence","variant":self.variant}))?;
            state.latest_activity=latest;state.working=working;state.records=records;state.recent=recent;
        }
        if self.variant != "C" {
            return Ok(());
        }
        ctx.event(
            "PREPARATION_START",
            &json!({"owner":"DP1_C","activity_seq":seq,"future_request_available":false}),
        );
        let (bounded, gaps) = production_snapshot(ctx, sources, "EVIDENCE_RELATION_PRODUCER");
        if bounded.is_empty() {
            ctx.event("PREPARATION_COMPLETE",&json!({"owner":"DP1_C","activity_seq":seq,"records":0,"reason":"NO_AUTHORIZED_PUBLIC_EVIDENCE"}));
            return Ok(());
        }
        let proposal=ctx.call("evidence_publish_relations",&json!({"instruction":"Publish reusable relations from observed source evidence and actual delivered history ONLY. No future request is known. Return JSON {records:[{record_id:string,interpretation:string,source_spans:[{source_id:string,revision:number,start_line:number,end_line:number}]}],gaps:[string]}. At most12 compact records. Derived interpretations are not independent source authority. Preserve conditional scope, delivered versus unshown content, uncertainty and exact original spans. Do not invent a future purpose or read absent content.","input":{"observed_activity":{"kind":event["kind"],"event_id":event["id"],"source_id":event["source_id"],"activity_seq":seq},"catalog":sources.catalog(),"observed_evidence":bounded,"actual_delivered":public_state["delivered"]},"max_tokens":1500}))?;
        let mut records = support_records(&proposal, &bounded, sources, "records");
        for record in &mut records {
            record["producer_node"] = json!(ctx.last_node());
            if let Some(passages) = record["supporting_passages"].as_array_mut() {
                for p in passages {
                    p["producer_node"] = json!(ctx.last_node());
                }
            }
        }
        let _effect=ctx.effect_guard()?;let mut state=self.state.lock().unwrap();
        let records=records_valid(&json!(records),sources,false);
        let versions=sources.catalog().as_array().unwrap_or(&Vec::new()).iter().filter_map(|source|Some((source["id"].as_str()?.to_owned(),source["revision"].as_u64()?))).collect();
        let discarded=seq<state.latest_activity;let latest=state.latest_activity;
        if !discarded {
            let _commit=ctx.commit_effect(&_effect,"PRODUCER_STATE",&json!({"owner":"evidence","variant":self.variant}))?;
            state.records=records;state.source_versions=versions;
        }
        let committed=state.records.clone();drop(state);drop(_effect);
        if discarded {ctx.event("PREPARATION_DISCARDED",&json!({"owner":"DP1_C","activity_seq":seq,"latest_activity":latest,"reason":"SUPERSEDED_ACTIVITY"}));}
        else {ctx.event("EVIDENCE_RELATIONS_COMMITTED",&json!({"activity_seq":seq,"records":committed,"unread_gaps":gaps}));}
        ctx.logical_buffer("common_evidence_records",serde_json::to_vec(&committed).unwrap().len(),"STATE");
        ctx.event("PREPARATION_COMPLETE",&json!({"owner":"DP1_C","activity_seq":seq}));
        Ok(())
    }
    pub fn absorb(&self, ctx: &Context, passages: &[Value]) -> Result<(), String> {
        if self.variant == "A" {
            let _effect = ctx.effect_guard()?;
            let mut state = self.state.lock().unwrap();
            let mut working=state.working.clone();working.extend(passages.iter().cloned());dedup_passages(&mut working);
            let _commit=ctx.commit_effect(&_effect,"PRODUCER_STATE",&json!({"owner":"evidence","variant":self.variant}))?;
            state.working=working;
        }
        Ok(())
    }
    pub fn resolve(
        &self,
        ctx: &mut Context,
        sources: &SourceRegistry,
        event: &Value,
        public_state: &Value,
    ) -> Result<EvidenceResult, String> {
        let safe_public = authorized_public_state(sources, public_state);
        let public_state = &safe_public;
        let (mut available,records,recent)={
            let effect=ctx.effect_guard()?;let mut state=self.state.lock().unwrap();
            let working=state.working.iter().filter(|p|permitted(sources,p,false)).cloned().collect::<Vec<_>>();
            let records=records_valid(&json!(state.records),sources,false);
            let recent=state.recent.iter().filter(|r|array(&r["source_spans"]).iter().all(|p|permitted(sources,p,false))).cloned().collect::<Vec<_>>();
            let mut available=if self.variant=="A"{working.clone()}else if self.variant=="C"{record_support(&json!(records),sources,false)}else{Vec::new()};
            available.extend(array(&public_state["handoff_prefix_originals"]));dedup_passages(&mut available);
            let result=(available,records.clone(),recent.clone());
            let _commit=ctx.commit_effect(&effect,"PRODUCER_STATE",&json!({"owner":"evidence","variant":self.variant}))?;
            state.working=working;state.records=records;state.recent=recent;result
        };
        // The same actual recent displays are available to every architecture. Unshown proposals are absent.
        let delivered = array(&public_state["delivered"]);
        let actual_recent = if delivered.is_empty() {
            recent
        } else {
            delivered.into_iter().rev().take(3).collect()
        };
        let mut dependencies = ctx.last_node().into_iter().collect::<Vec<_>>();
        for p in &available {
            for key in ["acquisition_node", "producer_node"] {
                if let Some(node) = p[key].as_str() {
                    dependencies.push(node.into());
                }
            }
        }
        if self.variant == "C" {
            for r in &records {
                if let Some(node) = r["producer_node"].as_str() {
                    dependencies.push(node.into());
                }
            }
        }
        dependencies.sort();
        dependencies.dedup();
        if !dependencies.is_empty() {
            ctx.join(&dependencies);
        }
        let proposal=ctx.call("evidence_select_ranges",&json!({"instruction":"Select only original source ranges needed to interpret and answer the actual current input. This is evidence acquisition, not final user-intent adoption. Return JSON {reads:[{source_id:string,revision:number,start_line:number,end_line:number}],bindings:[{expression:string,targets:[canonical source references]}],unresolved:[string]}. Every range must use actual catalog IDs/revisions and permitted bounds. Available original passages can be reused without another original read; return needed ranges even when already supplied so the host can select them. Actual delivered response text is history, not invented source facts. Preserve historical versus current revision distinction. If no source facts are needed (e.g. a self-contained general knowledge question), reads may be empty. Do not manufacture resolved targets for ambiguous inputs or assume denied content.","input":{"event":event,"catalog":sources.catalog(),"available_originals":available,"published_relations":if self.variant=="C"{json!(records)}else{Value::Null},"actual_recent_delivered":actual_recent,"public_state":public_state},"max_tokens":1200}))?;
        let mut selected = Vec::new();
        let mut missing = Vec::new();
        for r in array(&proposal["reads"]).into_iter().take(12) {
            if !permitted(sources, &r, false) {
                missing.push(json!({"request":r,"reason":"DENIED_OR_UNKNOWN_RANGE"}));
                continue;
            }
            if let Some(p) = available.iter().find(|p| contained(&r, p)) {
                if let Some(p) = slice_passage(&r, p) {
                    ctx.event(
                        "EVIDENCE_ORIGINAL_REUSED",
                        &json!({"variant":self.variant,"passage":p}),
                    );
                    selected.push(p);
                }
            } else {
                let (p, d) = read_ranges(
                    ctx,
                    sources,
                    &json!([r]),
                    false,
                    "REQUEST_EVIDENCE_ACQUISITION",
                );
                selected.extend(p);
                missing.extend(d);
            }
        }
        dedup_passages(&mut selected);
        if self.variant=="A" {
            let effect=ctx.effect_guard()?;let mut state=self.state.lock().unwrap();
            let mut working=state.working.clone();working.extend(selected.clone());dedup_passages(&mut working);let mut evicted=Vec::new();
            while working.iter().map(|p|p.to_string().len()).sum::<usize>()>14000 {
                if working.is_empty(){break;}let removed=working.remove(0);evicted.push(json!({"source_id":removed["source_id"],"budget_bytes":14000}));
            }
            let bytes=serde_json::to_vec(&working).unwrap().len();
            let commit=ctx.commit_effect(&effect,"PRODUCER_STATE",&json!({"owner":"evidence","variant":self.variant}))?;
            state.working=working;drop(state);drop(commit);drop(effect);
            for event in evicted {ctx.event("EVIDENCE_VIEW_EVICTED",&event);}
            ctx.logical_buffer("common_evidence_working_view",bytes,"STATE");
        }
        available.clear();
        ctx.event("EVIDENCE_REQUEST_READY",&json!({"variant":self.variant,"selected_original_spans":selected.iter().map(|p|json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"]})).collect::<Vec<_>>(),"denials":missing}));
        Ok(EvidenceResult {
            context: json!({"actual_recent_delivered":actual_recent,"published_relations":if self.variant=="C"{json!(records)}else{Value::Null},"acquisition_unknowns":proposal["unresolved"],"denials":missing}),
            passages: selected,
            bindings: array(&proposal["bindings"]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn registry() -> SourceRegistry {
        let s = SourceRegistry::new();
        s.register(json!({"id":"source","revision":1,"title":"Observed original","text":"left public\nprivate middle\nright public","segments":{"left":[1,1],"middle":[2,2],"right":[3,3]}})).unwrap();
        s
    }
    #[test]
    fn exact_slice_preserves_original_line_extent() {
        let p = json!({"source_id":"source","revision":1,"start_line":1,"end_line":3,"text":"left public\nprivate middle\nright public"});
        let w = json!({"source_id":"source","revision":1,"start_line":3,"end_line":3});
        let q = slice_passage(&w, &p).unwrap();
        assert_eq!(q["text"], "right public");
        assert_eq!(q["start_line"], 3);
        assert!(slice_passage(
            &json!({"source_id":"other","revision":1,"start_line":1,"end_line":1}),
            &p
        )
        .is_none());
    }
    #[test]
    fn published_record_cannot_add_unread_source_content() {
        let sources = registry();
        let actual = vec![sources.read("source", 1, 1, 1, false).unwrap().to_value()];
        let bad = json!({"records":[{"record_id":"bad","source_spans":[{"source_id":"source","revision":1,"start_line":2,"end_line":2}]}]});
        assert!(support_records(&bad, &actual, &sources, "records").is_empty());
        let good = json!({"records":[{"record_id":"good","interpretation":"derived statement","source_spans":[{"source_id":"source","revision":1,"start_line":1,"end_line":1}]}]});
        let r = support_records(&good, &actual, &sources, "records");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0]["supporting_passages"][0]["text"], "left public");
    }
    #[test]
    fn source_authority_change_blocks_old_record_and_keeps_unaffected() {
        let sources = registry();
        let passages = vec![
            sources.read("source", 1, 1, 1, false).unwrap().to_value(),
            sources.read("source", 1, 2, 2, false).unwrap().to_value(),
        ];
        let proposal = json!({"records":[{"record_id":"left","source_spans":[passages[0]]},{"record_id":"middle","source_spans":[passages[1]]}]});
        let records = support_records(&proposal, &passages, &sources, "records");
        sources.permission(&json!({"source_id":"source","read_segments":["left","right"],"external_segments":["left"]})).unwrap();
        let kept = records_valid(&json!(records), &sources, false);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0]["record_id"], "left");
        assert!(!permitted(&sources, &passages[1], false));
        assert!(sources.allowed("source", 1, 3, 3, false));
        assert!(!sources.allowed("source", 1, 3, 3, true));
    }
    #[test]
    fn revoked_pending_adoption_and_task_text_never_reenter_public_context() {
        let sources = registry();
        let public = json!({"requests":[{"goal_id":"g","task_id":"t","revision":2,"raw_text":"summarize the authorized document","adoption":{"answer":"private middle","purpose":"private middle","handling_text":"private middle","relation":"NEW","route":"DIRECT","source_revisions":{"source":1},"source_claims":[{"source_id":"source","revision":1,"start_line":2,"end_line":2,"quote":"private middle"}]}}],"tasks":[{"task_id":"t","title":"private middle","state":"RUNNING"}],"delivered":[{"text":"private middle","source_spans":[{"source_id":"source","revision":1,"start_line":2,"end_line":2}]}]});
        sources
            .permission(&json!({"source_id":"source","read_segments":[]}))
            .unwrap();
        let safe = authorized_public_state(&sources, &public);
        assert!(!safe.to_string().contains("private middle"));
        assert_eq!(safe["requests"][0]["goal_id"], "g");
        assert_eq!(safe["requests"][0]["revision"], 2);
        assert_eq!(safe["tasks"][0]["state"], "RUNNING");
    }
    #[test]
    fn recent_fallback_rechecks_authority_after_observation() {
        let sources = registry();
        let store = EvidenceStore::new("A");
        store.state.lock().unwrap().recent = vec![
            json!({"text":"private middle","source_spans":[{"source_id":"source","revision":1,"start_line":2,"end_line":2}]}),
        ];
        sources
            .permission(&json!({"source_id":"source","read_segments":[]}))
            .unwrap();
        let mut state = store.state.lock().unwrap();
        state.recent.retain(|r| {
            array(&r["source_spans"])
                .iter()
                .all(|p| permitted(&sources, p, false))
        });
        assert!(state.recent.is_empty());
    }
}
