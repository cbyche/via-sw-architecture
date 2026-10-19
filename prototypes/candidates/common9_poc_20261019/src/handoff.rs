//! DP4 handoff preparation and capable receiver. Common runtime owns intent, ingress and publication.
use crate::{
    api::HandoffRequest,
    context::Context,
    evidence::{
        array, authorized_public_state, contained, dedup_passages, permitted, production_snapshot,
        read_ranges, record_support, slice_passage, span, support_records,
    },
    sources::SourceRegistry,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
#[derive(Default)]
struct State {
    acquired: Vec<Value>,
    records: Vec<Value>,
    latest_activity: u64,
}
#[derive(Clone)]
pub struct HandoffStore {
    variant: String,
    state: Arc<Mutex<State>>,
}
fn export_manifest(sources: &SourceRegistry) -> Value {
    json!(array(&sources.catalog())
        .into_iter()
        .filter(|s| s["external_allowed"] == true)
        .map(|mut s| {
            s["allowed_segments"] = s["external_segments"].clone();
            s
        })
        .collect::<Vec<_>>())
}
fn record_valid(record: &Value, sources: &SourceRegistry, external: bool) -> bool {
    let spans = array(&record["source_spans"]);
    !spans.is_empty()
        && spans.iter().all(|r| permitted(sources, r, external))
        && array(&record["supporting_passages"])
            .iter()
            .all(|r| permitted(sources, r, external))
}
fn capsule_support(capsule: &Value, sources: &SourceRegistry) -> Vec<Value> {
    let mut p = array(&capsule["passages"]);
    p.retain(|p| permitted(sources, p, true));
    p.extend(record_support(&capsule["records"], sources, true));
    dedup_passages(&mut p);
    p
}
fn capsule_valid(capsule: &Value, sources: &SourceRegistry) -> bool {
    array(&capsule["references"])
        .iter()
        .chain(array(&capsule["passages"]).iter())
        .all(|p| permitted(sources, p, true))
        && array(&capsule["records"])
            .iter()
            .all(|r| record_valid(r, sources, true))
}
fn select_capsule(
    plan: &Value,
    available: &[Value],
    records: &[Value],
    sources: &SourceRegistry,
) -> Value {
    let mut passages = Vec::new();
    for s in array(&plan["selected_spans"]) {
        if !permitted(sources, &s, true) {
            continue;
        }
        if let Some(p) = available.iter().find(|p| contained(&s, p)) {
            if let Some(p) = slice_passage(&s, p) {
                passages.push(p);
            }
        }
    }
    let selected_records = records
        .iter()
        .filter(|r| {
            array(&plan["selected_record_ids"]).contains(&r["record_id"])
                && record_valid(r, sources, true)
        })
        .cloned()
        .collect::<Vec<_>>();
    json!({"summary":plan["summary"],"references":array(&plan["references"]).into_iter().filter(|r|permitted(sources,r,true)).collect::<Vec<_>>(),"passages":passages,"records":selected_records,"unknowns":plan["unknowns"]})
}
fn fact_errors(
    result: &Value,
    sources: &SourceRegistry,
    supplied: &[Value],
    required: &[Value],
) -> Vec<String> {
    let mut errors = Vec::new();
    let text = result["text"].as_str().unwrap_or("");
    if text.trim().is_empty() || text.chars().count() > 5000 {
        errors.push("EMPTY_OR_OVERSIZED_RESULT".into());
    }
    let facts = array(&result["facts"]);
    for fact in &facts {
        let quote = fact["quote"].as_str().unwrap_or("");
        let supported = !quote.is_empty()
            && permitted(sources, fact, true)
            && supplied.iter().any(|p| {
                slice_passage(fact, p)
                    .is_some_and(|q| q["text"].as_str().is_some_and(|t| t.contains(quote)))
            });
        if !supported || fact["statement"].as_str().unwrap_or("").trim().is_empty() {
            errors.push("FACT_NOT_ACTUALLY_SUPPLIED_AT_AUTHORIZED_SPAN".into());
        }
    }
    for sid in required {
        if !facts.iter().any(|f| f["source_id"] == *sid) {
            errors.push(format!("REQUIRED_SOURCE_FACT_MISSING {}", sid));
        }
    }
    if !array(&result["actions_performed"]).is_empty() {
        errors.push("SHIM_MUST_NOT_CLAIM_REAL_EXTERNAL_ACTION".into());
    }
    errors
}
impl HandoffStore {
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
            let acquired=state.acquired.iter().filter(|p|permitted(sources,p,false)).cloned().collect();
            let records=state.records.iter().filter(|p|record_valid(p,sources,false)).cloned().collect();let latest=state.latest_activity.max(seq);
            let _commit=ctx.commit_effect(&_effect,"PRODUCER_STATE",&json!({"owner":"handoff","variant":self.variant}))?;
            state.latest_activity=latest;state.acquired=acquired;state.records=records;
        }
        if self.variant != "B" {
            return Ok(());
        }
        ctx.event(
            "PREPARATION_START",
            &json!({"owner":"DP4_B","activity_seq":seq,"future_delegation_available":false}),
        );
        let (evidence, gaps) = production_snapshot(ctx, sources, "HANDOFF_WORK_RECORD_PRODUCER");
        ctx.event(
            "HANDOFF_RECORD_COVERAGE",
            &json!({"activity_seq":seq,"gaps":gaps}),
        );
        if evidence.is_empty() {
            ctx.event(
                "PREPARATION_COMPLETE",
                &json!({"owner":"DP4_B","activity_seq":seq,"records":0}),
            );
            return Ok(());
        }
        let proposal=ctx.call("handoff_publish_work_record",&json!({"instruction":"Produce a purpose-neutral reusable public work record from actually observed original evidence and actual public task/response state only. No future handoff is known. Return JSON {records:[{record_id:string,interpretation:string,source_spans:[{source_id:string,revision:number,start_line:number,end_line:number}]}],unknowns:[string]}. At most12 compact blocks. Preserve qualifications, conditional rules and unknowns. Each claim cites actual exact original spans. This is derived interpretation, not independent authority or a future domain plan. Do not equate prepared unshown text with actual delivered response.","input":{"observed_activity":{"kind":event["kind"],"event_id":event["id"],"source_id":event["source_id"],"activity_seq":seq},"actual_public_state":public_state,"actual_evidence":evidence,"catalog":sources.catalog()},"max_tokens":1500}))?;
        let mut records = support_records(&proposal, &evidence, sources, "records");
        for record in &mut records {
            record["producer_node"] = json!(ctx.last_node());
            if let Some(passages) = record["supporting_passages"].as_array_mut() {
                for p in passages {
                    p["producer_node"] = json!(ctx.last_node());
                }
            }
        }
        let effect=ctx.effect_guard()?;let mut state=self.state.lock().unwrap();
        let records=records.into_iter().filter(|record|record_valid(record,sources,false)).collect();
        let discarded=seq<state.latest_activity;let latest=state.latest_activity;
        if !discarded {let _commit=ctx.commit_effect(&effect,"PRODUCER_STATE",&json!({"owner":"handoff","variant":self.variant}))?;state.records=records;}
        let committed=state.records.clone();drop(state);drop(effect);
        if discarded {ctx.event("PREPARATION_DISCARDED",&json!({"owner":"DP4_B","activity_seq":seq,"latest_activity":latest}));}
        else {ctx.event("HANDOFF_RECORD_COMMITTED",&json!({"activity_seq":seq,"records":committed}));}
        ctx.logical_buffer("common_handoff_work_record",serde_json::to_vec(&committed).unwrap().len(),"STATE");
        ctx.event("PREPARATION_COMPLETE",&json!({"owner":"DP4_B","activity_seq":seq}));
        Ok(())
    }
    /// Actual locally owned originals can contribute to the common first semantic input.
    /// Derived record prose and future purpose are excluded; no new reads or preparation wait.
    pub fn prefix_originals(&self, sources: &SourceRegistry) -> Vec<Value> {
        let state = self.state.lock().unwrap();
        let mut p = state
            .acquired
            .iter()
            .filter(|p| permitted(sources, p, false))
            .cloned()
            .collect::<Vec<_>>();
        if self.variant == "B" {
            p.extend(record_support(&json!(state.records), sources, false));
        }
        dedup_passages(&mut p);
        p
    }
    pub fn build(
        &self,
        ctx: &mut Context,
        sources: &SourceRegistry,
        request: &HandoffRequest,
    ) -> Result<Value, String> {
        let adoption = &request.adoption;
        for f in ["purpose", "completion_condition"] {
            if adoption[f].as_str().unwrap_or("").trim().is_empty() {
                return Err(format!("HANDOFF_ADOPTION_{}_MISSING", f));
            }
        }
        let mut required = Vec::new();
        if let Some(versions) = adoption["source_revisions"].as_object() {
            for (id, rev) in versions {
                let source = sources
                    .version(id, rev.as_u64().ok_or("invalid source revision")?)
                    .ok_or("adopted source version unavailable")?;
                if array(&source["external_segments"]).is_empty() {
                    return Err(format!("EXTERNAL_SOURCE_PERMISSION_REQUIRED {}", id));
                }
                required.push(json!(id));
            }
        }
        let effect=ctx.effect_guard()?;let mut acquired=self.state.lock().unwrap();
        let mut originals=acquired.acquired.iter().filter(|p|permitted(sources,p,false)).cloned().collect::<Vec<_>>();
        originals.extend(request.evidence.passages.iter().filter(|p|permitted(sources,p,false)&&sources.valid_passage(p,false)).cloned());dedup_passages(&mut originals);
        let available=originals.iter().filter(|p|permitted(sources,p,true)).cloned().collect::<Vec<_>>();
        let records=if self.variant=="B" {acquired.records.iter().filter(|r|record_valid(r,sources,true)).cloned().collect::<Vec<_>>()}else{Vec::new()};
        let commit=ctx.commit_effect(&effect,"PRODUCER_STATE",&json!({"owner":"handoff_acquired","variant":self.variant}))?;
        acquired.acquired=originals;drop(acquired);drop(commit);drop(effect);
        let mut dependencies = ctx.last_node().into_iter().collect::<Vec<_>>();
        for p in &available {
            for key in ["acquisition_node", "producer_node"] {
                if let Some(node) = p[key].as_str() {
                    dependencies.push(node.into());
                }
            }
        }
        for r in &records {
            if let Some(node) = r["producer_node"].as_str() {
                dependencies.push(node.into());
            }
        }
        dependencies.sort();
        dependencies.dedup();
        if !dependencies.is_empty() {
            ctx.join(&dependencies);
        }
        let mandatory = json!({"goal_id":request.goal_id,"event_id":request.event_id,"request_revision":request.revision,"raw_user_text":request.raw_text,"purpose":adoption["purpose"],"completion_condition":adoption["completion_condition"],"recipient":adoption["recipient"],"artifact_type":adoption["artifact_type"],"task_id":adoption["task_id"],"targets":adoption["targets"],"source_revisions":adoption["source_revisions"],"required_source_ids":required,"constraints":adoption["constraints"],"exclusions":adoption["exclusions"],"allowed_actions":adoption["allowed_actions"],"completed_actions":adoption["completed_actions"],"unknown_actions":adoption["unknown_actions"],"grounding":request.grounding});
        let manifest = export_manifest(sources);
        let capsule = if self.variant == "C" {
            ctx.event(
                "HANDOFF_BUILD_BRANCH",
                &json!({"variant":"C","branch":"MANDATORY_AND_RECEIVER_READS"}),
            );
            Value::Null
        } else {
            let payload = json!({"instruction":"Select a sufficient purpose-specific handoff from actual acquired originals, current work-record blocks or authorized original references. Return JSON {mode:'DIRECT'|'READ',summary:string,references:[{source_id:string,revision:number,start_line:number,end_line:number}],selected_spans:[{source_id:string,revision:number,start_line:number,end_line:number}],selected_record_ids:[string],reads:[{source_id:string,revision:number,start_line:number,end_line:number}],unknowns:[string]}. DIRECT may select sufficient acquired originals or valid record blocks without rereading. Adequate references are also legal: every receiver supports bounded source reads. If VIA needs missing semantic detail to select the purpose-specific handoff, READ bounded originals. No new domain plan. Metadata is not source facts. Mandatory current constraints/permissions override derived prior interpretations. Do not broaden actions or access.","input":{"mandatory":mandatory,"manifest":manifest,"actually_acquired_originals":available,"work_records":records},"max_tokens":1400});
            let mut plan = ctx.call("handoff_select_details", &payload)?;
            let mut selected = select_capsule(&plan, &available, &records, sources);
            let adequate = |c: &Value| {
                !array(&c["references"]).is_empty()
                    || !array(&c["passages"]).is_empty()
                    || !array(&c["records"]).is_empty()
                    || required.is_empty()
            };
            if plan["mode"] != "READ" && adequate(&selected) && capsule_valid(&selected, sources) {
                ctx.event("HANDOFF_BUILD_BRANCH",&json!({"variant":self.variant,"branch":"DIRECT_ACQUIRED_OR_RECORD_OR_REFERENCES","record_ids":plan["selected_record_ids"]}));
                selected
            } else {
                if array(&plan["reads"]).is_empty() {
                    let mut repair = payload.clone();
                    repair["input"]["previous_rejected_plan"] = plan;
                    repair["input"]["repair_reason"]=json!("No sufficient direct handoff and no actual read plan. Select valid refs/raw/record or reads; do not invent facts.");
                    plan = ctx.call("handoff_repair_selection", &repair)?;
                    selected = select_capsule(&plan, &available, &records, sources);
                }
                if plan["mode"] != "READ"
                    && adequate(&selected)
                    && capsule_valid(&selected, sources)
                {
                    selected
                } else {
                    let (passages, denials) =
                        read_ranges(ctx, sources, &plan["reads"], true, "VIA_HANDOFF_COMPOSER");
                    if passages.is_empty() {
                        return Err(format!("HANDOFF_NO_AUTHORIZED_DETAILS {}", json!(denials)));
                    }
                    let effect=ctx.effect_guard()?;let mut state=self.state.lock().unwrap();
                    let mut acquired=state.acquired.clone();acquired.extend(passages.clone());dedup_passages(&mut acquired);
                    let commit=ctx.commit_effect(&effect,"PRODUCER_STATE",&json!({"owner":"handoff_acquired","variant":self.variant}))?;
                    state.acquired=acquired;drop(state);drop(commit);drop(effect);
                    let composed=ctx.call("handoff_compose_capsule",&json!({"instruction":"Compose a short purpose-specific explanation using only supplied actual evidence and mandatory constraints. Return JSON {summary:string,unknowns:[string]}. Do not invent source contents, reinterpret access grants or produce a new domain plan. Original evidence will be attached unchanged.","input":{"mandatory":mandatory,"actual_evidence":passages,"denials":denials},"max_tokens":1000}))?;
                    ctx.event(
                        "HANDOFF_BUILD_BRANCH",
                        &json!({"variant":self.variant,"branch":"READ_AND_COMPOSE"}),
                    );
                    json!({"summary":composed["summary"],"passages":passages,"records":[],"references":[],"unknowns":composed["unknowns"]})
                }
            }
        };
        let contract = json!({"mandatory":mandatory,"manifest":manifest,"capsule":capsule,"variant":self.variant,"receiver_profile":"CAPABLE_BOUNDED_ORIGINAL_READS_SAME_FOR_ALL"});
        if !capsule_valid(&contract["capsule"], sources) {
            return Err("HANDOFF_PERMISSION_CHANGED_BEFORE_CONTRACT".into());
        }
        ctx.logical_buffer(
            "common_handoff_acquired_originals",
            serde_json::to_vec(&self.state.lock().unwrap().acquired)
                .unwrap()
                .len(),
            "STATE",
        );
        ctx.event("HANDOFF_CONTRACT_BUILT", &contract);
        Ok(contract)
    }
    pub fn receive(
        &self,
        ctx: &mut Context,
        sources: &SourceRegistry,
        contract: &Value,
    ) -> Result<Value, String> {
        if !capsule_valid(&contract["capsule"], sources) {
            return Err("RECEIVER_CONTRACT_ACCESS_INVALIDATED".into());
        }
        let mut supplied = capsule_support(&contract["capsule"], sources);
        let required = array(&contract["mandatory"]["required_source_ids"]);
        let result_schema="{text:string,facts:[{source_id:string,revision:number,start_line:number,end_line:number,statement:string,quote:string}],used_source_ids:[string],source_spans:[{source_id:string,revision:number,start_line:number,end_line:number}],conditions_respected:[string],actions_performed:[],unknowns:[string]}";
        let payload = json!({"role":"RECEIVER","instruction":format!("Act as the same capable bounded receiver for every architecture. Honor actual mandatory purpose, constraints, exclusions, permissions and unknown/completed actions. Return JSON {{mode:'COMPLETE'|'READ',result:{}|null,reads:[{{source_id:string,revision:number,start_line:number,end_line:number}}]}}. COMPLETE is legal only when actual supplied originals/source-backed blocks suffice. Metadata, references and unsupported capsule prose are not source facts. Each source fact requires an exact quote from actual supplied original text at its cited lines and current authorized version. Otherwise choose bounded original READ. Preserve conditional rules; do not turn a tie-break into an unconditional preference. Produce only a small draft/verification artifact, with actions_performed empty. Real external source outcomes arrive separately; do not claim actual sending or tool execution. If the task needs no source facts, ordinary knowledge may be used without fictitious provenance.",result_schema),"input":{"contract":contract,"actual_supplied_originals":supplied},"max_tokens":2200});
        let mut choice = ctx.call("receiver_select_or_complete", &payload)?;
        let mut result = choice["result"].clone();
        let direct = choice["mode"] == "COMPLETE"
            && fact_errors(&result, sources, &supplied, &required).is_empty();
        ctx.event("RECEIVER_BRANCH",&json!({"variant":self.variant,"branch":if direct{"DIRECT_FROM_ACTUAL_SUPPORT"}else{"BOUNDED_ORIGINAL_READ"}}));
        if !direct {
            if array(&choice["reads"]).is_empty() {
                let mut repair = payload.clone();
                repair["input"]["rejected_proposal"] = choice;
                repair["input"]["reason"]=json!("Completion lacks actually supplied source support and selected no reads. Select actual authorized source ranges now.");
                choice = ctx.call("receiver_repair_read_selection", &repair)?;
            }
            ctx.event(
                "SERVING_START",
                &json!({"goal_id":contract["mandatory"]["goal_id"]}),
            );
            let (p, d) = read_ranges(ctx, sources, &choice["reads"], true, "DOWNSTREAM_RECEIVER");
            ctx.event(
                "SERVING_END",
                &json!({"goal_id":contract["mandatory"]["goal_id"]}),
            );
            supplied.extend(p);
            dedup_passages(&mut supplied);
            ctx.event(
                "RECEIVER_READ_RECEIPT",
                &json!({"served":supplied,"denied":d,"selected_reads":choice["reads"]}),
            );
            if supplied.is_empty() && !required.is_empty() {
                return Ok(
                    json!({"accepted":false,"contract":contract,"errors":["RECEIVER_NO_ACTUAL_EVIDENCE"]}),
                );
            }
            result=ctx.call("receiver_complete",&json!({"role":"RECEIVER","instruction":format!("Produce the bounded draft or handoff verification required by mandatory contract from actual supplied originals only. Return JSON {}. Every source fact quote must be exact at cited lines, with at least one actual fact for every required source. Preserve exclusions/conditions and conditional rules. Do not report real external actions; actions_performed is empty. No guessed facts or restatement of the request as completion.",result_schema),"input":{"mandatory":contract["mandatory"],"actual_evidence":supplied,"denials":d},"max_tokens":2200}))?;
        }
        let mut errors = fact_errors(&result, sources, &supplied, &required);
        if !errors.is_empty() {
            ctx.event(
                "RECEIVER_RESULT_REJECTED",
                &json!({"result":result,"errors":errors}),
            );
            result=ctx.call("receiver_repair_result",&json!({"role":"RECEIVER","instruction":format!("Repair this bounded receiver result using only actual supplied evidence/current mandatory conditions. Return JSON {}. Each required source needs a supported actual fact and exact original quote at cited lines. Preserve uncertainty, exclusions and conditional scope. actions_performed must be empty. If impossible return text:null and unknowns rather than fabricate.",result_schema),"input":{"mandatory":contract["mandatory"],"actual_evidence":supplied,"rejected":result,"errors":errors},"max_tokens":2200}))?;
            errors = fact_errors(&result, sources, &supplied, &required);
        }
        // Runtime atomically rechecks the current request revision and source permissions at publication.
        Ok(
            json!({"accepted":errors.is_empty(),"contract":contract,"receiver_result":result,"errors":errors,"actually_supplied_source_spans":supplied.iter().map(|p|json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"]})).collect::<Vec<_>>() }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn registry() -> SourceRegistry {
        let s = SourceRegistry::new();
        s.register(json!({"id":"source","revision":1,"text":"actual first\nactual second","segments":{"first":[1,1],"second":[2,2]}})).unwrap();
        s
    }
    #[test]
    fn direct_acquired_passage_needs_no_new_original_read() {
        let s = registry();
        let raw = s.read("source", 1, 1, 2, false).unwrap().to_value();
        let selected = json!({"source_id":"source","revision":1,"start_line":2,"end_line":2});
        let capsule = select_capsule(&json!({"selected_spans":[selected]}), &[raw], &[], &s);
        assert_eq!(capsule["passages"][0]["text"], "actual second");
        assert!(capsule_valid(&capsule, &s));
    }
    #[test]
    fn shared_source_rights_rechecked_for_old_direct_capsule() {
        let s = registry();
        let raw = s.read("source", 1, 1, 2, false).unwrap().to_value();
        let c = json!({"passages":[raw]});
        assert!(capsule_valid(&c, &s));
        s.permission(&json!({"source_id":"source","external_segments":["first"]}))
            .unwrap();
        assert!(!capsule_valid(&c, &s));
        assert!(s.allowed("source", 1, 1, 1, true));
    }
    #[test]
    fn metadata_or_quote_outside_actual_cited_line_cannot_complete_receiver() {
        let s = registry();
        let p = s.read("source", 1, 1, 2, true).unwrap().to_value();
        let mut r = json!({"text":"A draft containing the actual first fact.","facts":[{"source_id":"source","revision":1,"start_line":1,"end_line":1,"quote":"actual second","statement":"A paraphrase."}],"actions_performed":[]});
        assert!(!fact_errors(&r, &s, &[p.clone()], &[json!("source")]).is_empty());
        r["facts"][0]["quote"] = json!("actual first");
        assert!(fact_errors(&r, &s, &[p], &[json!("source")]).is_empty());
        assert!(!fact_errors(&r, &s, &[], &[json!("source")]).is_empty());
        r["actions_performed"] = json!(["email_sent"]);
        assert!(!fact_errors(&r, &s, &[], &[json!("source")]).is_empty());
    }
}
