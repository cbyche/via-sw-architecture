//! One ingress, authority gate, interpretation scheduler, response and goal lifecycle.
use crate::{
    api::{Configuration, EvidenceResult, GroundingInput, HandoffRequest},
    context::Context,
    evidence::{array, EvidenceStore},
    grounding,
    handoff::HandoffStore,
    sources::SourceRegistry,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Duration,
};
#[derive(Default)]
struct State {
    history: Vec<Value>,
    captured: Vec<Value>,
    pending: BTreeSet<String>,
    requests: BTreeMap<String, Value>,
    delivered: Vec<Value>,
    tasks: Vec<Value>,
    ingress: BTreeMap<String, Value>,
    counter: u64,
    active: usize,
    ended: bool,
    failures: Vec<Value>,
}
struct Engine {
    config: Configuration,
    sources: SourceRegistry,
    evidence: EvidenceStore,
    handoff: HandoffStore,
    state: Mutex<State>,
    changed: Condvar,
    queue: Mutex<VecDeque<Value>>,
    background: Mutex<Vec<thread::JoinHandle<()>>>,
}
impl Engine {
    fn public(&self) -> Value {
        let s = self.state.lock().unwrap();
        let history = s.history.iter().rev().take(10).cloned().collect::<Vec<_>>();
        let delivered = s
            .delivered
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        crate::evidence::authorized_public_state(
            &self.sources,
            &json!({"history":history,"captured_inputs":s.captured,"requests":s.requests.values().collect::<Vec<_>>(),"delivered":delivered,"tasks":s.tasks,"permissions":self.sources.catalog(),"revision_counter":s.counter,"handoff_prefix_originals":self.handoff.prefix_originals(&self.sources)}),
        )
    }
    fn activity(self: &Arc<Self>, ctx: &Context, mut event: Value) {
        let public = self.public();
        event["runtime_activity_seq"] = public["revision_counter"].clone();
        let engine = self.clone();
        let mut c = ctx.without_boundary().scoped(&[], &[]);
        self.background.lock().unwrap().push(thread::spawn(move || {
            if let Err(error) = engine
                .evidence
                .observe(&mut c, &engine.sources, &event, &public)
            {
                c.event(
                    "PREPARATION_FAILED",
                    &json!({"owner":"evidence","error":error}),
                );
            }
            if let Err(error) = engine
                .handoff
                .observe(&mut c, &engine.sources, &event, &public)
            {
                c.event(
                    "PREPARATION_FAILED",
                    &json!({"owner":"handoff","error":error}),
                );
            }
        }));
    }
    fn capture(&self, ctx: &Context, event: &Value, effect: &crate::lifecycle::EffectState) -> Result<(), String> {
        let id = event["id"].as_str().unwrap_or("");
        ctx.capture(id,&json!({"text":event["text"],"input_class":event["kind"],"task_id":event["task_id"],"event_class":event["event_class"],"source_monotonic_ns":event["source_monotonic_ns"],"input_begin_monotonic_ns":event["input_begin_monotonic_ns"],"source_node":event["source_node"],"source_parent":event["source_parent"]}));
        let mut s = self.state.lock().unwrap();
        let _commit = ctx.commit_effect(effect, "INPUT_CAPTURE", &json!({"stimulus_id":id,"input_class":event["kind"]}))?;
        s.counter += 1;
        let spans = event["caused_by"]
            .as_str()
            .and_then(|cause| s.ingress.get(cause))
            .map(|i| array(&i["receiver_receipt"]["actually_supplied_source_spans"]))
            .unwrap_or_default();
        s.captured
            .push(json!({"id":id,"text":event["text"],"kind":event["kind"],"source_spans":spans}));
        s.history.push(
            json!({"event_id":id,"text":event["text"],"kind":event["kind"],"source_spans":spans}),
        );
        if event["kind"] == "user" {
            s.pending.insert(id.into());
            s.active += 1;
        }
        Ok(())
    }
    fn fail(&self, ctx: &Context, event: &Value, reason: String) {
        let mut s = self.state.lock().unwrap();
        s.pending.remove(event["id"].as_str().unwrap_or(""));
        s.failures
            .push(json!({"event_id":event["id"],"reason":reason}));
        ctx.event(
            "WORKFLOW_FAILED",
            &json!({"event_id":event["id"],"reason":reason}),
        );
        self.changed.notify_all();
    }
    fn finish_worker(&self) {
        let mut s = self.state.lock().unwrap();
        s.active = s.active.saturating_sub(1);
        self.changed.notify_all();
    }
    fn boundary(self: &Arc<Self>, ctx: &Context, parents: Vec<String>) -> Vec<String> {
        let mut nodes = Vec::new();
        loop {
            let event = self.queue.lock().unwrap().pop_front();
            let Some(event) = event else { break };
            ctx.event(
                "SCHEDULER_A_BOUNDARY_ACCEPT",
                &json!({"event_id":event["id"],"semantic_relation_known":false}),
            );
            let mut incoming_parents = parents.clone();
            if let Some(source) = event["source_node"].as_str() {
                incoming_parents.push(source.into());
            }
            let mut c = ctx.scoped(
                &[format!("input:{}", event["id"].as_str().unwrap_or(""))],
                &incoming_parents,
            );
            self.attach_boundary(&mut c);
            self.workflow(&mut c, event);
            nodes.extend(c.last_node());
        }
        nodes
    }
    fn attach_boundary(self: &Arc<Self>, ctx: &mut Context) {
        if self.config.execution == "A" {
            let engine = self.clone();
            let c = ctx.without_boundary();
            ctx.set_boundary(Arc::new(move |parents| engine.boundary(&c, parents)));
        }
    }
    fn checked_text(value: &Value) -> Result<String, String> {
        value
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .ok_or("empty response text".into())
    }
    fn meaning(
        &self,
        ctx: &mut Context,
        event: &Value,
        evidence: &EvidenceResult,
        grounded: &Value,
        repair: &Value,
    ) -> Result<Value, String> {
        if !evidence
            .passages
            .iter()
            .all(|p| crate::evidence::permitted(&self.sources, p, false))
        {
            return Err("MEANING_INPUT_SOURCE_AUTHORITY_CHANGED".into());
        }
        ctx.call("meaning_interpret_and_respond",&json!({"instruction":"Interpret ONLY the actual current user input in VIA. Return JSON {relation:'NEW'|'FOLLOWUP'|'CORRECTION'|'ANSWER'|'CONTROL',target_goal_id:string|null,target_event_id:string|null,task_id:string|null,route:'DIRECT'|'DELEGATE'|'WAIT'|'CLARIFY'|'CONTROL',purpose:string,handling_text:string,answer:string|null,targets:[canonical source references],bindings:[{expression:exact current spoken substring,targets:[canonical source references]}],source_revisions:{actual_catalog_source_ID:number},source_claims:[{source_id,revision,start_line,end_line,quote:exact supplied substring}],constraints:[string],exclusions:[string],allowed_actions:[string],completed_actions:[],unknown_actions:[],permission_operations:[{source_id:string,read_segments:[declared segment names],external_segments:[declared segment names]}],control_operations:[{task_id,question_id:string|null,action:string,value:string,disposition:'APPROVE'|'DENY'|'UPDATE'}],recipient:string|null,artifact_type:string|null,completion_condition:string}. Ordinary reading of an already authorized source changes no permission: permission_operations MUST be []. Emit a permission operation only when the current user actually gives or withdraws scope consent, never to describe a read. Names are keys of that actual source catalog segments object, not allowed_segments/external_segments numeric ranges. When a source declares no segment names, no named grant is legal; do not invent all or turn line numbers into names. source_revisions keys must be exact actual catalog source IDs (never the placeholder source_id), with integer versions that actually exist. For source targets/binding targets, copy a literal string from an actual source_catalog row known_source_reference_strings. These are mechanically enumerated syntax choices, not relevance judgments or permission grants. A bare exact current source ID names its whole original; a named extent has exact sourceID:vINTEGER:declared_extent spelling. Historical named extents are listed on their actual version rows. Do not generate a numeric-range/version suffix or guess a reference string. quote must reproduce a supplied substring after JSON decoding, including actual newline characters rather than literal backslash+n; copy only a shorter exact substring if sufficient. Schema/support rejudgment shares a total limit of three meaning calls with initial interpretation and required post-consent rereading. After actual permission application permission_operations must be empty. A validation error describes an observed invalid proposal, not permission to change the user meaning or invent facts. Use actual captured and delivered history to determine association; event IDs are identifiers, never semantic instructions. Independent follow-up questions are new goals; a correction/consent/answer continues its actual earlier goal. Clarify genuinely ambiguous short replies; do not approve the latest question by default. First handling_text must identify the resolved topic/objects and concrete action, not a generic acknowledgement or unresolved pronoun echo. DIRECT must include the actual complete concise answer; metadata is not source facts. Facts must use actually supplied originals with exact quote claims. Grounding/evidence bindings are proposals: explicitly adopt valid final phrase target sets, retaining within-utterance correction and noncorrected conditions. Actual delivered and Task artifact sources use their exact registered IDs/reference strings from the same catalog. target_goal_id, target_event_id, task_id and control question_id use their own actual public-state identifier namespaces; source-reference strings are not those identifiers. Do not invent IDs or independent goal/Task meanings. DELEGATE is external work such as modifying a document or creating a file; preserve restrictions and current source versions. Access permission, external sharing consent and Task action approval are separate: do not send locally-only content. A request for private content must WAIT for scope consent; after body-only consent select only the body segment and compose after real read. A later external-sharing refusal completes that declined goal without ingress. Do not claim actions completed before their actual source confirms. CONTROL uses existing Task/question IDs and explicit dispositions. If actual supplied evidence is insufficient, answer=null and request bounded read/rejudgment; never fabricate a source-backed answer.","input":{"event":event,"public_state":self.public(),"source_catalog":self.sources.catalog(),"evidence":evidence.context,"actually_supplied_originals":evidence.passages,"evidence_binding_proposals":evidence.bindings,"grounding":grounded,"repair_reason":repair},"max_tokens":1900}))
    }
    fn validate_proposal(
        &self,
        event: &Value,
        plan: &Value,
        evidence: &EvidenceResult,
        permissions_applied: bool,
    ) -> Result<(), String> {
        let state = self.state.lock().unwrap();
        validate_proposal(
            plan,
            event,
            &self.sources,
            evidence,
            &state,
            permissions_applied,
        )
    }
    fn resolve_goal(&self, ctx: &Context, event: &Value, plan: &mut Value, effect: &crate::lifecycle::EffectState) -> Result<(String, u64, Value), String> {
        let eid = event["id"].as_str().ok_or("event ID absent")?;
        let relation = plan["relation"].as_str().unwrap_or("").to_owned();
        let mut s = self.state.lock().unwrap();
        let existing = if ["CORRECTION", "ANSWER", "CONTROL"].contains(&relation.as_str()) {
            plan["target_goal_id"].as_str().map(String::from).or_else(|| plan["target_event_id"].as_str().map(|e|format!("goal:{}",e)))
        } else { None };
        let gid = existing.unwrap_or_else(||format!("goal:{}",eid));
        // Build the entire adoption in local values. No state prefix is mutated
        // before the cutoff check under the actual owning state lock.
        let mut request = if let Some(request)=s.requests.get(&gid) { request.clone() }
        else if relation=="CORRECTION" && plan["target_event_id"].as_str().is_some() {
            let target=plan["target_event_id"].as_str().unwrap();
            let origin=s.captured.iter().rev().find(|e|e["id"]==target&&e["text"].is_string()).ok_or("correction target not captured")?;
            json!({"goal_id":gid,"origin_event_id":target,"raw_text":origin["text"],"revision":1,"status":"CAPTURED"})
        } else if existing_target_required(&relation)&&plan["target_goal_id"].is_string() { return Err("target goal not actually adopted".into()); }
        else { json!({"goal_id":gid,"origin_event_id":eid,"raw_text":event["text"],"revision":1,"status":"ADOPTED"}) };
        if relation=="NEW"&&request["revision"].as_u64().unwrap_or(1)>1&&request["last_event_id"]!=eid { return Err("ORIGINAL_WORKFLOW_SUPERSEDED_BY_ADOPTED_CORRECTION".into()); }
        let revision=request["revision"].as_u64().unwrap_or(1)+if relation=="CORRECTION"{1}else{0};
        let mut new_task=None;
        if plan["route"]=="DELEGATE" {
            let task=request["task_id"].as_str().map(String::from).unwrap_or_else(||format!("TASK-{}",s.counter));
            plan["task_id"]=json!(task);request["task_id"]=json!(task);
            if !s.tasks.iter().any(|t|t["task_id"]==task) { new_task=Some(json!({"task_id":task,"title":plan["purpose"],"state":"ADOPTED","questions":[],"allowed_actions":plan["allowed_actions"]})); }
        }
        request["revision"]=json!(revision);request["adoption"]=plan.clone();request["last_event_id"]=json!(eid);request["status"]=json!(plan["route"].as_str().unwrap_or("UNRESOLVED"));
        let origin=request["origin_event_id"].clone();
        let _commit = ctx.commit_effect(effect,"SEMANTIC_ADOPTION",&json!({"stimulus_id":eid,"goal_id":gid,"revision":revision}))?;
        s.requests.insert(gid.clone(),request);
        if let Some(task)=new_task {s.tasks.push(task);}
        s.pending.remove(eid);self.changed.notify_all();
        Ok((gid,revision,origin))
    }
    fn await_pending(&self, ctx: &Context, event: &Value) -> Result<(), String> {
        let mut started = false;
        loop {
            ctx.check_active()?;
            let s = self.state.lock().unwrap();
            if !later_pending(&s, event) {
                if started {
                    ctx.event("POLICY_WAIT_END",&json!({"stimulus_id":event["id"],"reason":"later captured input interpreted"}));
                }
                return Ok(());
            }
            if !started {
                ctx.event(
                    "POLICY_WAIT_START",
                    &json!({"stimulus_id":event["id"],"reason":"unresolved later captured input"}),
                );
                started = true;
            }
            let (s, _) = self
                .changed
                .wait_timeout(s, Duration::from_millis(10))
                .unwrap();
            drop(s);
            ctx.drain_boundary();
        }
    }
    fn publication(
        self: &Arc<Self>, ctx: &Context, event: &Value, gid: &str, revision: u64,
        text: &str, phase: &str, plan: &Value, evidence: &EvidenceResult, complete: bool,
    ) -> Result<String, String> {
        self.await_pending(ctx,event)?;
        let check=|| {
            let _source=self.sources.guard();let state=self.state.lock().unwrap();
            if later_pending(&state,event){return Err("NEW_INPUT_AT_PUBLICATION_REQUIRES_REJUDGMENT".into());}
            let request=state.requests.get(gid).ok_or("publication goal absent")?;
            if request["revision"]!=revision{return Err("STALE_GOAL_REVISION_NOT_PUBLISHED".into());}
            if !evidence.passages.iter().all(|p|self.sources.valid_passage(p,evidence.context["enforce_external"]==true)){return Err("PUBLICATION_SOURCE_PERMISSION_CHANGED".into());}
            Ok::<(),String>(())
        };
        let origin=self.state.lock().unwrap().requests.get(gid).ok_or("publication goal absent")?["origin_event_id"].clone();
        let mut output=json!({"goal_id":gid,"origin_event_id":origin,"stimulus_id":event["id"],"revision":revision,"text":text,"phase":phase,"task_id":event["task_id"].as_str().map(String::from).or_else(||plan["task_id"].as_str().map(String::from)),"event_class":event["event_class"],"bindings":plan["bindings"],"targets":plan["targets"],"source_revisions":plan["source_revisions"],"source_spans":evidence.passages.iter().map(|p|json!({"source_id":p["source_id"],"revision":p["revision"],"start_line":p["start_line"],"end_line":p["end_line"]})).collect::<Vec<_>>(),"content_contract_checked":true,"checks":["atomic_goal_revision","current_source_range_authority","actual_host_display_receipt"],"delivery_source_id":event["delivery_source_id"],"content_type":if phase=="HANDLING"{"ADOPTED_HANDLING"}else{"DETAILED_RESULT"}});
        let receipt=ctx.local_operation("publication_host_receipt_wait","VIA",||ctx.request_publication_checked(&output,check))?;
        let publication=receipt["publication_id"].as_str().ok_or("publication receipt ID absent")?.to_owned();
        output["publication_id"]=json!(publication);output["actual_host_observed_ns"]=receipt["observed_ns"].clone();
        let completion_error;
        let actual_complete;
        {
            let effect=ctx.effect_guard()?;let _source=self.sources.guard();let mut state=self.state.lock().unwrap();
            completion_error=if later_pending(&state,event){Some("NEW_INPUT_AFTER_HOST_DISPLAY_RECEIPT".to_owned())}
                else if state.requests.get(gid).is_none_or(|request|request["revision"]!=revision){Some("DISPLAY_OBSERVED_BUT_STALE_GOAL_NOT_COMPLETED".to_owned())}
                else if !evidence.passages.iter().all(|p|self.sources.valid_passage(p,evidence.context["enforce_external"]==true)){Some("DISPLAY_OBSERVED_BUT_SOURCE_PERMISSION_CHANGED".to_owned())}
                else{None};
            actual_complete=complete&&completion_error.is_none();
            output["current_completion_eligible"]=json!(completion_error.is_none());output["completion_ineligible_reason"]=json!(completion_error);output["goal_complete"]=json!(actual_complete);
            output["delivery_history_semantics"]=json!("ACTUAL_HOST_OBSERVED_HISTORICAL_FACT_AUTHORITY_RECHECK_REQUIRED_FOR_REUSE");
            let _commit=ctx.commit_effect(&effect,"PUBLICATION_DELIVERY_STATE",&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"publication_id":publication,"actual_host_observed_ns":receipt["observed_ns"],"complete":actual_complete,"historical_display_recorded":true,"completion_ineligible_reason":completion_error}))?;
            state.delivered.push(output.clone());state.counter+=1;
            if actual_complete {state.requests.get_mut(gid).unwrap()["status"]=json!("HOST_COMPLETE");}
        }
        if actual_complete {ctx.event("GOAL_COMPLETE",&output);}
        if phase!="HANDLING" {ctx.event("RESPONSE_FINAL",&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"phase":phase,"node_id":ctx.last_node(),"publication_id":publication,"current_completion_eligible":completion_error.is_none()}));}
        if let Some(error)=completion_error{return Err(error);}
        Ok(publication)
    }
    fn workflow(self: &Arc<Self>, ctx: &mut Context, event: Value) {
        let result = self.workflow_inner(ctx, &event);
        if let Err(reason) = result {
            self.fail(ctx, &event, reason);
        }
        self.finish_worker();
    }
    fn workflow_inner(self: &Arc<Self>, ctx: &mut Context, event: &Value) -> Result<(), String> {
        let mut evidence = self
            .evidence
            .resolve(ctx, &self.sources, event, &self.public())?;
        let frames = array(&event["frames"]);
        let prior = json!({"delivered":self.public()["delivered"]});
        let grounded = grounding::ground(
            ctx,
            &self.config.grounding,
            &GroundingInput {
                event_id: event["id"].as_str().unwrap_or("").into(),
                revision: 1,
                transcript: event["text"].as_str().unwrap_or("").into(),
                frames,
                cues: array(&event["cues"]),
                prior,
                evidence: json!({"passages":evidence.passages,"context":evidence.context}),
            },
        )?;
        for binding in array(&grounded["bindings"]) {
            for target in array(&binding["targets"]) {
                if let Some(reference) = target.as_str() {
                    if let Some((id, rev, a, b)) = target_range(&self.sources, reference) {
                        if !evidence.passages.iter().any(|p| {
                            p["source_id"] == id
                                && p["revision"] == rev
                                && p["start_line"].as_u64().unwrap_or(u64::MAX) <= a as u64
                                && p["end_line"].as_u64().unwrap_or(0) >= b as u64
                        }) {
                            let p = ctx.local_operation(
                                "grounded_original_acquisition",
                                "VIA",
                                || self.sources.read(&id, rev, a, b, false),
                            )?;
                            let mut acquired = p.to_value();
                            acquired["acquisition_node"] = json!(ctx.last_node());
                            evidence.passages.push(acquired);
                        }
                    }
                }
            }
        }
        self.evidence.absorb(ctx, &evidence.passages)?;
        let mut repair = Value::Null;
        let mut permissions_applied = false;
        let mut attempt = 0usize;
        let mut plan = loop {
            attempt += 1;
            let proposal = self.meaning(ctx, event, &evidence, &grounded, &repair)?;
            let check = self.validate_proposal(event, &proposal, &evidence, permissions_applied);
            if let Err(error) = check {
                ctx.event("MEANING_PROPOSAL_REJECTED",&json!({"stimulus_id":event["id"],"attempt":attempt,"total_limit":3,"observed_error":error,"proposal":proposal,"permissions_already_applied":permissions_applied}));
                if attempt >= 3 {
                    return Err(format!("MEANING_TOTAL_LIMIT_EXHAUSTED: {error}"));
                }
                repair = json!({"observed_validation_error":error,"meaning_calls_used":attempt,"meaning_calls_total_limit":3});
                continue;
            }
            let ops = array(&proposal["permission_operations"]);
            if !ops.is_empty() {
                // Recheck the complete proposal inside the authority gate, then
                // validate all registry updates before committing the batch.
                let result = ctx.effect_guard().and_then(|_effect| self
                    .sources
                    .permission_batch_checked_at(&proposal["permission_operations"], || {
                        self.validate_proposal(event, &proposal, &evidence, permissions_applied)
                    }, || ctx.commit_effect(&_effect, "PERMISSION_BATCH", &json!({"stimulus_id":event["id"],"operations":proposal["permission_operations"]}))));
                if let Err(error) = result {
                    ctx.event("MEANING_PROPOSAL_REJECTED",&json!({"stimulus_id":event["id"],"attempt":attempt,"total_limit":3,"observed_error":error,"proposal":proposal,"permissions_already_applied":permissions_applied}));
                    if attempt >= 3 {
                        return Err(format!("MEANING_TOTAL_LIMIT_EXHAUSTED: {error}"));
                    }
                    repair = json!({"observed_validation_error":error,"meaning_calls_used":attempt,"meaning_calls_total_limit":3});
                    continue;
                }
                permissions_applied = true;
                for op in ops {
                    ctx.event("PERMISSION_OPERATION_APPLIED", &op);
                }
                if attempt >= 3 {
                    return Err("MEANING_TOTAL_LIMIT_EXHAUSTED_AFTER_VALID_PERMISSION_BATCH: authority mutation retained; no complete answer adopted".into());
                }
                evidence = self
                    .evidence
                    .resolve(ctx, &self.sources, event, &self.public())?;
                repair = json!({"observed_state":"Valid permission batch actually applied once; authorized originals now actually reread. Return a proposal with empty permission_operations using this current evidence. Prior valid authority fact is retained even if this workflow later fails.","meaning_calls_used":attempt,"meaning_calls_total_limit":3});
                continue;
            }
            ctx.event("MEANING_PROPOSAL_ACCEPTED",&json!({"stimulus_id":event["id"],"attempt":attempt,"total_limit":3,"permissions_already_applied":permissions_applied,"proposal":proposal}));
            break proposal;
        };
        let _adoption_effect = ctx.effect_guard()?;
        let (gid, revision, origin) = self.resolve_goal(ctx, event, &mut plan, &_adoption_effect)?;
        ctx.goal_start(
            &gid,
            &json!({"stimulus_id":origin,"revision":revision,"raw_text":event["text"]}),
        );
        ctx.event("SEMANTIC_ADOPTION",&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"plan":plan,"grounding":grounded}));
        drop(_adoption_effect);
        let text = Self::checked_text(&plan["handling_text"])?;
        if plan["route"] != "DIRECT" {
            let targets = array(&plan["targets"])
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ");
            let concrete = if targets.is_empty() {
                text.clone()
            } else {
                format!("{}: {}", targets, text)
            };
            self.publication(
                ctx, event, &gid, revision, &concrete, "HANDLING", &plan, &evidence, false,
            )?;
        }
        if plan["route"] == "DELEGATE" {
            let _effect = ctx.effect_guard()?;
            ctx.event("RESPONSE_FINAL",&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"phase":"ADOPTED_DELEGATION_INTENTION","node_id":ctx.last_node()}));
        }
        match plan["route"].as_str().unwrap_or("") {
            "DIRECT" => {
                let answer = Self::checked_text(&plan["answer"])?;
                let publication_id=self.publication(
                    ctx, event, &gid, revision, &answer, "DIRECT", &plan, &evidence, true,
                )?;
                self.register_delivery(ctx, event, &gid, revision, &answer, &plan, &evidence, &publication_id)?;
            }
            "CLARIFY" | "WAIT" => {
                let answer = plan["answer"].as_str().unwrap_or(&text);
                self.publication(
                    ctx,
                    event,
                    &gid,
                    revision,
                    answer,
                    "WAITING_USER",
                    &plan,
                    &evidence,
                    false,
                )?;
                let _effect = ctx.effect_guard()?;
                let _source_guard = self.sources.guard();
                let mut s = self.state.lock().unwrap();
                if s.requests.get(&gid).is_none_or(|r|r["revision"]!=revision) || later_pending(&s,event) { return Err("STALE_QUESTION_STATE_NOT_RECORDED".into()); }
                if !evidence.passages.iter().all(|p|self.sources.valid_passage(p,evidence.context["enforce_external"]==true)) { return Err("QUESTION_STATE_AUTHORITY_CHANGED_AFTER_DISPLAY".into()); }
                let _commit = ctx.commit_effect(&_effect, "QUESTION_STATE", &json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision}))?;
                s.requests.get_mut(&gid).unwrap()["question"] = json!(answer);
            }
            "CONTROL" => {
                let ops = array(&plan["control_operations"]);
                if ops.is_empty() {
                    let answer = plan["answer"].as_str().unwrap_or(&text);
                    self.publication(
                        ctx, event, &gid, revision, answer, "DECLINED", &plan, &evidence, true,
                    )?;
                } else {
                    let _effect = ctx.effect_guard()?;
                    let _gate = self.sources.guard();
                    let mut s = self.state.lock().unwrap();
                    if s.requests[&gid]["revision"] != revision {
                        return Err("STALE_CONTROL_NOT_SENT".into());
                    }
                    for op in &ops {
                        if !s.tasks.iter().any(|t| t["task_id"] == op["task_id"]) {
                            return Err("CONTROL_UNKNOWN_ACTUAL_TASK".into());
                        }
                        if op["question_id"].is_string()
                            && !s.tasks.iter().any(|t| {
                                t["task_id"] == op["task_id"]
                                    && array(&t["questions"]).iter().any(|q| {
                                        q["question_id"] == op["question_id"]
                                            && q["task_id"] == op["task_id"]
                                    })
                            })
                        {
                            return Err("CONTROL_QUESTION_NOT_DISPLAYED".into());
                        }
                    }
                    let record = json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"control_operations":ops,"kind":"CONTROL","node_id":ctx.last_node()});
                    let _commit = ctx.commit_effect(&_effect, "CONTROL_INGRESS", &json!({"stimulus_id":event["id"]}))?;
                    s.ingress
                        .insert(event["id"].as_str().unwrap().into(), record.clone());
                    ctx.event(
                        "CONTROL_INGRESS",
                        &crate::agent_wire::encode_request(&record),
                    );
                    ctx.event("RESPONSE_FINAL",&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"phase":"CONTROL_ADMITTED","node_id":ctx.last_node()}));
                    ctx.event("RECEIVER_RESULT_AVAILABLE",&crate::agent_wire::encode_receipt(&json!({"stimulus_id":event["id"],"accepted":true,"control_operations":ops,"goal_id":gid,"revision":revision})));
                }
            }
            "DELEGATE" => {
                let request = HandoffRequest {
                    goal_id: gid.clone(),
                    event_id: event["id"].as_str().unwrap().into(),
                    raw_text: event["text"].as_str().unwrap_or("").into(),
                    revision,
                    adoption: plan.clone(),
                    evidence: evidence.clone(),
                    grounding: grounded,
                    public_state: self.public(),
                };
                let contract = self.handoff.build(ctx, &self.sources, &request)?;
                self.await_pending(ctx, event)?;
                {
                    let _effect = ctx.effect_guard()?;
                    let _gate = self.sources.guard();
                    let mut s = self.state.lock().unwrap();
                    if s.requests[&gid]["revision"] != revision || later_pending(&s, event) {
                        ctx.event(
                            "STALE_HANDOFF_DISCARDED",
                            &json!({"goal_id":gid,"revision":revision}),
                        );
                        return Ok(());
                    }
                    for target in array(&plan["targets"]) {
                        if let Some(reference) = target.as_str() {
                            if let Some((id, rev, a, b)) = target_range(&self.sources, reference) {
                                if !self.sources.allowed(&id, rev, a, b, true) {
                                    return Err("HANDOFF_TARGET_EXPORT_PERMISSION_CHANGED".into());
                                }
                            }
                        }
                    }
                    let record = json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"kind":"DELEGATE","contract":contract,"node_id":ctx.last_node()});
                    let _commit = ctx.commit_effect(&_effect, "AGENT_INGRESS", &json!({"stimulus_id":event["id"]}))?;
                    s.ingress
                        .insert(event["id"].as_str().unwrap().into(), record.clone());
                    ctx.event("AGENT_INGRESS", &crate::agent_wire::encode_request(&record));
                }
                let engine = self.clone();
                let event = event.clone();
                let mut c = ctx.without_boundary().scoped(
                    &[gid.clone()],
                    &ctx.last_node().into_iter().collect::<Vec<_>>(),
                );
                self.background.lock().unwrap().push(thread::spawn(move||{let wire=crate::agent_wire::encode_request(&contract);let decoded=crate::agent_wire::decode_request(&wire);let received=decoded.and_then(|contract|engine.handoff.receive(&mut c,&engine.sources,&contract));match received{Ok(value)=>{if let Ok(effect)=c.effect_guard(){let mut state=engine.state.lock().unwrap();if let Ok(_commit)=c.commit_effect(&effect,"RECEIVER_RECEIPT_STATE",&json!({"stimulus_id":event["id"]})){if let Some(i)=state.ingress.get_mut(event["id"].as_str().unwrap()){i["receiver_receipt"]=value.clone();}}}c.event("RECEIVER_RESULT_AVAILABLE",&crate::agent_wire::encode_receipt(&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"accepted":value["accepted"],"actual_result":value,"node_id":c.last_node()})))},Err(error)=>c.event("RECEIVER_RESULT_AVAILABLE",&crate::agent_wire::encode_receipt(&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"accepted":false,"error":error})))}}));
            }
            _ => return Err("UNKNOWN_MEANING_ROUTE".into()),
        }
        Ok(())
    }
    fn register_delivery(
        self: &Arc<Self>,
        ctx: &Context,
        event: &Value,
        gid: &str,
        revision: u64,
        text: &str,
        plan: &Value,
        evidence: &EvidenceResult,
        publication_id: &str,
    ) -> Result<(), String> {
        let _effect = ctx.effect_guard()?;
        let source_guard = self.sources.guard();
        let state = self.state.lock().unwrap();
        require_current_publication_state(&state,event,gid,revision)?;
        if !evidence.passages.iter().all(|p|self.sources.valid_passage(p,evidence.context["enforce_external"]==true)) { return Err("DELIVERED_SOURCE_AUTHORITY_CHANGED".into()); }
        let id = event["delivery_source_id"]
            .as_str()
            .unwrap_or(event["id"].as_str().unwrap());
        let external = evidence
            .passages
            .iter()
            .all(|p| crate::evidence::permitted(&self.sources, p, true));
        if self.sources.current(id).is_some_and(|source|source["revision"].as_u64().unwrap_or(0)>revision) { return Err("DELIVERED_SOURCE_POINTER_ROLLBACK_REJECTED".into()); }
        self.sources.register_guarded_checked(json!({"id":id,"revision":revision,"title":format!("실제 표시한 {}",id),"kind":"delivered","text":text,"local_allowed":true,"external_allowed":external,"provenance":evidence.passages,"extents":{"delivered":[1,text.lines().count()]}}), &source_guard, || ctx.commit_effect(&_effect, "SOURCE_REGISTER", &json!({"stimulus_id":event["id"],"source_id":id,"publication_id":publication_id})))?;
        drop(state);drop(source_guard);drop(_effect);
        self.activity(ctx,json!({"kind":"RESPONSE_DELIVERED","id":id,"goal_id":gid,"source_revisions":plan["source_revisions"]}));
        Ok(())
    }
    fn agent(self: &Arc<Self>, ctx: &Context, event: &Value) -> Result<(), String> {
        let class = event["event_class"].as_str().unwrap_or("RESULT");
        let text = Self::checked_text(&event["text"])?;
        let cause = event["caused_by"].as_str();
        let ingress = cause.and_then(|id| self.state.lock().unwrap().ingress.get(id).cloned());
        if let Some(i) = ingress {
            let gid = i["goal_id"].as_str().unwrap();
            let revision = i["revision"].as_u64().unwrap();
            let plan = self.state.lock().unwrap().requests[gid]["adoption"].clone();
            let spans = array(&i["receiver_receipt"]["actually_supplied_source_spans"]);
            let evidence = EvidenceResult {
                context: json!({"enforce_external":true}),
                passages: spans,
                bindings: Vec::new(),
            };
            let mut parents = event["source_node"]
                .as_str()
                .map(|n| vec![n.into()])
                .unwrap_or_default();
            if let Some(node) = i["node_id"].as_str() {
                parents.push(node.into());
            }
            if let Some(node) = event["receiver_node"].as_str() {
                parents.push(node.into());
            }
            let c = ctx.scoped(&[gid.into()], &parents);
            let publication_id=self.publication(
                &c,
                event,
                gid,
                revision,
                &text,
                "AGENT_RESULT",
                &plan,
                &evidence,
                true,
            )?;
            {
                let _effect = ctx.effect_guard()?;
                let _source_guard = self.sources.guard();
                let mut state = self.state.lock().unwrap();
                require_current_result_state(&state,event,&i,gid,revision)?;
                if !evidence.passages.iter().all(|p|self.sources.valid_passage(p,true)) { return Err("AGENT_RESULT_AUTHORITY_CHANGED_AFTER_DISPLAY".into()); }
                let _commit = ctx.commit_effect(&_effect, "TASK_CHANGE", &json!({"stimulus_id":event["id"],"class":class,"publication_id":publication_id,"goal_id":gid,"revision":revision,"task_id":event["task_id"]}))?;
                if let Some(task) = state
                    .tasks
                    .iter_mut()
                    .find(|task| task["task_id"] == event["task_id"])
                {
                    task["state"] = json!("COMPLETED");
                    task["last_result"] = json!(text);
                    task["source_spans"] = json!(evidence.passages);
                    task["result_source_id"] = event["result_source"]["id"].clone();
                }
            }
            if let Some(source) = event.get("result_source") {
                let _effect = ctx.effect_guard()?;
                let source_guard = self.sources.guard();
                let state = self.state.lock().unwrap();
                require_current_result_state(&state,event,&i,gid,revision)?;
                if !evidence.passages.iter().all(|p|self.sources.valid_passage(p,true)) { return Err("AGENT_ARTIFACT_AUTHORITY_CHANGED_AFTER_DISPLAY".into()); }
                if self.sources.current(source["id"].as_str().ok_or("artifact source ID missing")?).is_some_and(|current|current["revision"].as_u64().unwrap_or(0)>source["revision"].as_u64().unwrap_or(0)) { return Err("AGENT_ARTIFACT_POINTER_ROLLBACK_REJECTED".into()); }
                let mut registered = source.clone();
                registered["provenance"] = json!(evidence.passages);
                registered["local_allowed"] = json!(evidence
                    .passages
                    .iter()
                    .all(|p| crate::evidence::permitted(&self.sources, p, false)));
                registered["external_allowed"] = json!(evidence
                    .passages
                    .iter()
                    .all(|p| crate::evidence::permitted(&self.sources, p, true)));
                self.sources.register_guarded_checked(registered, &source_guard, ||ctx.commit_effect(&_effect, "SOURCE_REGISTER", &json!({"stimulus_id":event["id"],"source_id":source["id"],"publication_id":publication_id})))?;
                drop(state);drop(source_guard);drop(_effect);
                self.activity(ctx, json!({"kind":"AGENT_RESULT_SOURCE","id":source["id"]}));
            }
        } else {
            let output=json!({"stimulus_id":event["id"],"text":text,"task_id":event["task_id"],"event_class":event["event_class"],"content_type":"TASK_QUESTION","source_monotonic_ns":event["source_monotonic_ns"],"phase":"ACTUAL_AGENT_FACT"});
            let receipt=ctx.local_operation("publication_host_receipt_wait","VIA",||ctx.request_publication(&output))?;
            let publication_id=receipt["publication_id"].clone();
            {
                let effect=ctx.effect_guard()?;let mut state=self.state.lock().unwrap();
                let _commit=ctx.commit_effect(&effect,"PUBLICATION_DELIVERY_STATE",&json!({"stimulus_id":event["id"],"class":class,"phase":"ACTUAL_AGENT_FACT","task_id":event["task_id"],"publication_id":publication_id,"actual_host_observed_ns":receipt["observed_ns"],"task_fact_state_recorded":true}))?;
                if class=="QUESTION" {
                    for question in array(&event["questions"]) {
                        if let Some(task)=state.tasks.iter_mut().find(|task|task["task_id"]==question["task_id"]) {
                            let mut questions=array(&task["questions"]);questions.push(question);task["questions"]=json!(questions);
                        }
                    }
                }
                if let Some(task)=state.tasks.iter_mut().find(|task|task["task_id"]==event["task_id"]) {
                    if class=="PROGRESS"{task["state"]=json!("RUNNING");task["last_progress"]=json!(text);}
                    else if class=="RESULT"{task["state"]=json!("COMPLETED");task["last_result"]=json!(text);}
                }
            }
            ctx.event("RESPONSE_FINAL",&json!({"stimulus_id":event["id"],"phase":"ACTUAL_AGENT_FACT","publication_id":publication_id}));
        }
        Ok(())
    }
}
fn require_current_publication_state(state:&State,event:&Value,gid:&str,revision:u64)->Result<(),String>{
    if state.requests.get(gid).is_none_or(|r|r["revision"]!=revision)||later_pending(state,event){return Err("STALE_POST_PUBLICATION_STATE_REJECTED".into());}
    Ok(())
}
fn require_current_result_state(state:&State,event:&Value,ingress:&Value,gid:&str,revision:u64)->Result<(),String>{
    require_current_publication_state(state,event,gid,revision)?;
    let task=&event["task_id"];
    let request=&state.requests[gid];
    let associated=if ingress["kind"]=="DELEGATE"{request["task_id"]==*task&&ingress["contract"]["mandatory"]["task_id"]==*task}
        else{array(&request["adoption"]["control_operations"]).iter().any(|op|op["task_id"]==*task)&&array(&ingress["control_operations"]).iter().any(|op|op["task_id"]==*task)};
    if !task.is_string()||!associated||!state.tasks.iter().any(|t|t["task_id"]==*task){return Err("STALE_OR_DIFFERENT_TASK_RESULT_REJECTED".into());}
    Ok(())
}
fn later_pending(s: &State, event: &Value) -> bool {
    if event["kind"] == "agent" {
        return !s.pending.is_empty();
    }
    let current = s
        .captured
        .iter()
        .position(|e| e["id"] == event["id"])
        .unwrap_or(usize::MAX);
    s.pending.iter().any(|id| {
        s.captured
            .iter()
            .position(|e| e["id"] == id.as_str())
            .is_some_and(|i| i > current)
    })
}
fn validate_proposal(
    plan: &Value,
    event: &Value,
    sources: &SourceRegistry,
    evidence: &EvidenceResult,
    state: &State,
    permissions_applied: bool,
) -> Result<(), String> {
    if !plan.is_object() {
        return Err("meaning proposal must be an object".into());
    }
    if !evidence
        .passages
        .iter()
        .all(|p| crate::evidence::permitted(sources, p, false))
    {
        return Err("PROPOSAL_SUPPLIED_SOURCE_AUTHORITY_CHANGED".into());
    }
    let relation = plan["relation"]
        .as_str()
        .ok_or("relation must be a string")?;
    if !["NEW", "FOLLOWUP", "CORRECTION", "ANSWER", "CONTROL"].contains(&relation) {
        return Err("unknown relation enum".into());
    }
    let route = plan["route"].as_str().ok_or("route must be a string")?;
    if !["DIRECT", "DELEGATE", "WAIT", "CLARIFY", "CONTROL"].contains(&route) {
        return Err("unknown route enum".into());
    }
    for key in ["purpose", "handling_text"] {
        if plan[key].as_str().is_none_or(|v| v.trim().is_empty()) {
            return Err(format!("{key} must be nonempty text"));
        }
    }
    for key in [
        "target_goal_id",
        "target_event_id",
        "task_id",
        "answer",
        "recipient",
        "artifact_type",
        "completion_condition",
    ] {
        if !plan[key].is_null() && !plan[key].is_string() {
            return Err(format!("{key} must be text or null"));
        }
    }
    for key in [
        "constraints",
        "exclusions",
        "allowed_actions",
        "completed_actions",
        "unknown_actions",
    ] {
        let values = plan[key]
            .as_array()
            .ok_or_else(|| format!("{key} must be an array"))?;
        if values.iter().any(|v| !v.is_string()) {
            return Err(format!("{key} entries must be strings"));
        }
    }
    let revisions = plan["source_revisions"]
        .as_object()
        .ok_or("source_revisions must be actual-ID keyed object")?;
    for (id, revision) in revisions {
        let revision = revision
            .as_u64()
            .ok_or("source_revisions values must be integer revisions")?;
        if sources.version(id, revision).is_none() {
            return Err(format!(
                "source_revisions key/version not present in actual catalog: {id} v{revision}"
            ));
        }
    }
    let claims = plan["source_claims"]
        .as_array()
        .ok_or("source_claims must be an array")?;
    for claim in claims {
        let id = claim["source_id"]
            .as_str()
            .ok_or("claim source_id must be a string")?;
        let revision = claim["revision"]
            .as_u64()
            .ok_or("claim revision must be an integer")?;
        let a = claim["start_line"]
            .as_u64()
            .ok_or("claim start_line must be an integer")?;
        let b = claim["end_line"]
            .as_u64()
            .ok_or("claim end_line must be an integer")?;
        let quote = claim["quote"]
            .as_str()
            .filter(|v| !v.is_empty())
            .ok_or("claim quote must be nonempty text")?;
        let source = sources
            .version(id, revision)
            .ok_or("claim source/version not present in actual catalog")?;
        if a == 0 || b < a || b > source["line_count"].as_u64().unwrap_or(0) {
            return Err("claim range beyond actual source".into());
        }
        if revisions.get(id) != Some(&json!(revision)) {
            return Err("claim source/version missing from source_revisions map".into());
        }
        if !evidence.passages.iter().any(|p| {
            crate::evidence::slice_passage(claim, p).is_some_and(|actual| {
                actual["text"]
                    .as_str()
                    .is_some_and(|text| text.contains(quote))
            })
        }) {
            return Err("claim quote is not an exact substring of actually supplied span after JSON decoding".into());
        }
    }
    let check_ref = |reference: &Value| -> Result<(), String> {
        let reference = reference
            .as_str()
            .ok_or("canonical reference must be text")?;
        if target_range(sources, reference).is_none() {
            return Err(format!(
                "canonical reference does not name actual source/declared extent: {reference}"
            ));
        }
        Ok(())
    };
    for reference in plan["targets"]
        .as_array()
        .ok_or("targets must be an array")?
    {
        check_ref(reference)?
    }
    for binding in plan["bindings"]
        .as_array()
        .ok_or("bindings must be an array")?
    {
        if binding["expression"].as_str().is_none_or(|v| v.is_empty()) {
            return Err("binding expression must be nonempty text".into());
        }
        for reference in binding["targets"]
            .as_array()
            .ok_or("binding targets must be an array")?
        {
            check_ref(reference)?
        }
    }
    if let Some(id) = plan["target_goal_id"].as_str() {
        if !state.requests.contains_key(id) {
            return Err("target_goal_id was not actually adopted".into());
        }
    }
    if let Some(id) = plan["target_event_id"].as_str() {
        if id != event["id"].as_str().unwrap_or("") && !state.captured.iter().any(|e| e["id"] == id)
        {
            return Err("target_event_id was not actually captured".into());
        }
    }
    if let Some(id) = plan["task_id"].as_str() {
        if !state.tasks.iter().any(|t| t["task_id"] == id) {
            return Err("task_id not present in actual Task state".into());
        }
    }
    let controls = plan["control_operations"]
        .as_array()
        .ok_or("control_operations must be an array")?;
    for op in controls {
        let task = op["task_id"]
            .as_str()
            .ok_or("control task_id must be a string")?;
        let task = state
            .tasks
            .iter()
            .find(|t| t["task_id"] == task)
            .ok_or("CONTROL_UNKNOWN_ACTUAL_TASK")?;
        if !op["question_id"].is_null() {
            let q = op["question_id"]
                .as_str()
                .ok_or("control question_id must be text or null")?;
            if !array(&task["questions"])
                .iter()
                .any(|v| v["question_id"] == q && v["task_id"] == op["task_id"])
            {
                return Err("CONTROL_QUESTION_NOT_DISPLAYED".into());
            }
        }
        if !op["action"].is_string() || !op["value"].is_string() {
            return Err("control action/value must be text".into());
        }
        if !["APPROVE", "DENY", "UPDATE"].contains(&op["disposition"].as_str().unwrap_or("")) {
            return Err("control disposition invalid".into());
        }
    }
    let permissions = plan["permission_operations"]
        .as_array()
        .ok_or("permission_operations must be an array")?;
    if permissions_applied && !permissions.is_empty() {
        return Err("permission batch already applied; subsequent proposal must use empty permission_operations".into());
    }
    sources.validate_permission_batch(&plan["permission_operations"])?;
    if route == "DIRECT" && permissions.is_empty() && !direct_supported(plan, evidence) {
        return Err("DIRECT_ACTUAL_SUPPORT_VALIDATION_FAILED".into());
    }
    Ok(())
}
fn target_range(sources: &SourceRegistry, reference: &str) -> Option<(String, u64, usize, usize)> {
    if let Some((id, rest)) = reference.rsplit_once(":v") {
        let (version, extent) = rest.split_once(':')?;
        let revision = version.parse().ok()?;
        let source = sources.version(id, revision)?;
        let range = &source["extents"][extent];
        Some((
            id.into(),
            revision,
            range[0].as_u64()? as usize,
            range[1].as_u64()? as usize,
        ))
    } else {
        let source = sources.current(reference)?;
        Some((
            reference.into(),
            source["revision"].as_u64()?,
            1,
            source["line_count"].as_u64()? as usize,
        ))
    }
}
fn direct_supported(plan: &Value, evidence: &EvidenceResult) -> bool {
    if plan["answer"].as_str().is_none_or(|s| s.trim().is_empty()) {
        return false;
    }
    let sources = plan["source_revisions"].as_object();
    if sources.is_none_or(|s| s.is_empty()) {
        return true;
    }
    let claims = array(&plan["source_claims"]);
    sources.unwrap().iter().all(|(id, revision)| {
        claims.iter().any(|c| {
            c["source_id"] == id.as_str()
                && c["revision"] == *revision
                && c["quote"].as_str().is_some_and(|q| {
                    !q.is_empty()
                        && evidence.passages.iter().any(|p| {
                            crate::evidence::slice_passage(c, p).is_some_and(|actual| {
                                actual["text"].as_str().is_some_and(|text| text.contains(q))
                            })
                        })
                })
        })
    })
}
fn existing_target_required(relation: &str) -> bool {
    ["CORRECTION", "ANSWER", "CONTROL"].contains(&relation)
}
pub fn run(input: Value, ctx: Context) -> Result<Value, String> {
    let config = Configuration::from_value(&input["configuration"])?;
    let sources = SourceRegistry::new();
    for source in array(&input["initial"]["sources"]) {
        sources.register(source)?;
    }
    let state = State {
        history: array(&input["initial"]["raw_history"]),
        delivered: array(&input["initial"]["historical_delivered"]),
        tasks: array(&input["initial"]["tasks"]),
        ..Default::default()
    };
    let engine = Arc::new(Engine {
        evidence: EvidenceStore::new(&config.evidence),
        handoff: HandoffStore::new(&config.handoff),
        config,
        sources,
        state: Mutex::new(state),
        changed: Condvar::new(),
        queue: Mutex::new(VecDeque::new()),
        background: Mutex::new(Vec::new()),
    });
    ctx.event("EPISODE_READY",&json!({"episode":input["id"],"configuration":engine.config.id,"initial_history_turns":array(&input["initial"]["raw_history"]).len(),"completed_tasks":array(&input["initial"]["tasks"]).iter().filter(|t|t["state"]=="COMPLETED").count(),"other_task_slots":array(&input["initial"]["tasks"]).iter().filter(|t|t["state"]!="COMPLETED").count()}));
    engine.activity(
        &ctx,
        json!({"kind":"INITIAL_SOURCE_SNAPSHOT","id":"initial","runtime_activity_seq":0}),
    );
    let aengine = engine.clone();
    let actx = ctx.clone();
    let scheduler = if engine.config.execution == "A" {
        Some(thread::spawn(move || loop {
            let event = aengine.queue.lock().unwrap().pop_front();
            if let Some(event) = event {
                let mut c = actx.scoped(
                    &[format!("input:{}", event["id"].as_str().unwrap_or(""))],
                    &event["source_node"]
                        .as_str()
                        .map(|n| vec![n.into()])
                        .unwrap_or_default(),
                );
                aengine.attach_boundary(&mut c);
                aengine.workflow(&mut c, event);
            } else {
                let s = aengine.state.lock().unwrap();
                if s.ended && s.active == 0 {
                    break;
                }
                drop(s);
                thread::sleep(Duration::from_millis(2));
            }
        }))
    } else {
        None
    };
    loop {
        match ctx.recv_input(Duration::from_millis(50)) {
            Ok(message) => {
                if message["type"] == "end_input" || message["type"] == "cancel_work" {
                    engine.state.lock().unwrap().ended = true;
                    engine.changed.notify_all();
                    break;
                }
                let _input_effect = match ctx.effect_guard() {
                    Ok(guard) => guard,
                    Err(error) => { ctx.event("INPUT_REJECTED_AFTER_CUTOFF", &json!({"message":message,"error":error})); continue; }
                };
                let event = &message["event"];
                if event["kind"] == "input_begin" {
                    let mut s = engine.state.lock().unwrap();
                    let _commit=match ctx.commit_effect(&_input_effect, "INPUT_BEGIN", &json!({"event_id":event["id"]})) {Ok(commit)=>commit,Err(error)=>{ctx.event("INPUT_REJECTED_AFTER_CUTOFF",&json!({"event":event,"error":error}));continue;}};
                    s.pending.insert(event["id"].as_str().unwrap_or("").into());
                    s.captured
                        .push(json!({"id":event["id"],"kind":"user_begin"}));
                    ctx.event("AUTHORED_INPUT_BEGIN", event);
                    continue;
                }
                if event["kind"] == "source" {
                    if let Err(error)=engine.sources.register_checked(event["source"].clone(), ||ctx.commit_effect(&_input_effect, "SOURCE_REGISTER", &json!({"source_id":event["source"]["id"]}))) {if error.starts_with("OBSERVATION_DEADLINE"){ctx.event("INPUT_REJECTED_AFTER_CUTOFF",&json!({"event":event,"error":error}));continue;}return Err(error);}
                    let mut state=engine.state.lock().unwrap();
                    let _commit=match ctx.commit_effect(&_input_effect, "SOURCE_ACTIVITY_STATE", &json!({"source_id":event["source"]["id"]})) {Ok(commit)=>commit,Err(error)=>{ctx.event("INPUT_REJECTED_AFTER_CUTOFF",&json!({"event":event,"error":error,"source_registered_before_cutoff":true}));continue;}};
                    state.counter += 1;drop(state);
                    engine.activity(
                        &ctx,
                        json!({"kind":"SOURCE_UPDATE","id":event["source"]["id"]}),
                    );
                    continue;
                }
                if let Err(error)=engine.capture(&ctx, event, &_input_effect) {ctx.event("INPUT_REJECTED_AFTER_CUTOFF",&json!({"event":event,"error":error}));continue;}
                if event["kind"] == "agent" {
                    let e = engine.clone();
                    let c = ctx.without_boundary();
                    let event = event.clone();
                    engine
                        .background
                        .lock()
                        .unwrap()
                        .push(thread::spawn(move || {
                            if let Err(error) = e.agent(&c, &event) {
                                e.fail(&c, &event, error);
                            }
                        }));
                    continue;
                }
                if event["kind"] == "user" {
                    if engine.config.execution == "A" {
                        engine.queue.lock().unwrap().push_back(event.clone());
                    } else {
                        let e = engine.clone();
                        let event = event.clone();
                        let mut c = ctx.scoped(
                            &[format!("input:{}", event["id"].as_str().unwrap_or(""))],
                            &event["source_node"]
                                .as_str()
                                .map(|n| vec![n.into()])
                                .unwrap_or_default(),
                        );
                        engine
                            .background
                            .lock()
                            .unwrap()
                            .push(thread::spawn(move || e.workflow(&mut c, event)));
                    }
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => return Err("input channel disconnected".into()),
        }
    }
    if let Some(worker) = scheduler {
        worker.join().map_err(|_| "A scheduler panicked")?;
    }
    loop {
        let handles = std::mem::take(&mut *engine.background.lock().unwrap());
        if handles.is_empty() {
            break;
        }
        for handle in handles {
            let _ = handle.join();
        }
    }
    let s = engine.state.lock().unwrap();
    Ok(
        json!({"configuration":engine.config.id,"requests":s.requests,"delivered":s.delivered,"ingress":s.ingress,"failures":s.failures,"pending_inputs":s.pending,"active":s.active}),
    )
}

#[cfg(test)]
mod proposal_protocol_tests {
    use super::*;
    fn fixture() -> (SourceRegistry, EvidenceResult, State, Value, Value) {
        let sources = SourceRegistry::new();
        sources.register(json!({"id":"DOC:actual","revision":1,"text":"left exact\nright exact","extents":{"body":[1,2]},"segments":{"body":[1,2]}})).unwrap();
        let mut evidence = EvidenceResult::default();
        evidence.passages.push(
            sources
                .read("DOC:actual", 1, 1, 2, false)
                .unwrap()
                .to_value(),
        );
        let plan = json!({"relation":"NEW","target_goal_id":null,"target_event_id":null,"task_id":null,"route":"DIRECT","purpose":"read actual source","handling_text":"Read the actual document.","answer":"left exact; right exact","targets":["DOC:actual:v1:body"],"bindings":[],"source_revisions":{"DOC:actual":1},"source_claims":[{"source_id":"DOC:actual","revision":1,"start_line":1,"end_line":2,"quote":"left exact\nright exact"}],"constraints":[],"exclusions":[],"allowed_actions":[],"completed_actions":[],"unknown_actions":[],"permission_operations":[],"control_operations":[],"recipient":null,"artifact_type":null,"completion_condition":null});
        (
            sources,
            evidence,
            State::default(),
            json!({"id":"input","text":"read actual source"}),
            plan,
        )
    }
    #[test]
    fn valid_colon_source_and_supplied_historical_revision_remain_valid() {
        let (s, e, state, event, p) = fixture();
        s.register(json!({"id":"DOC:actual","revision":2,"text":"changed source","segments":{"body":[1,1]}})).unwrap();
        assert!(validate_proposal(&p, &event, &s, &e, &state, false).is_ok());
    }
    #[test]
    fn raw_invalid_variants_fail_without_rewriting_or_mutation() {
        let (s, e, state, event, p) = fixture();
        let mut variants = Vec::new();
        let mut v = p.clone();
        v["permission_operations"] = json!([{"source_id":"DOC:actual","read_segments":["1-2"]}]);
        variants.push((v, "segment not declared by source"));
        let mut v = p.clone();
        v["source_revisions"] = json!({"source_id":1});
        variants.push((v, "not present in actual catalog"));
        let mut v = p.clone();
        v["source_claims"][0]["source_id"] = json!(1);
        variants.push((v, "claim source_id must be a string"));
        let mut v = p.clone();
        v["source_claims"][0]["quote"] = json!("left exact\\nright exact");
        variants.push((v, "not an exact substring"));
        let before = s.catalog();
        for (v, message) in variants {
            let raw = v.clone();
            let error = validate_proposal(&v, &event, &s, &e, &state, false).unwrap_err();
            assert!(error.contains(message), "{error}");
            assert_eq!(v, raw);
            assert_eq!(s.catalog(), before);
        }
    }
    #[test]
    fn permissions_may_be_empty_after_real_grant_but_not_applied_twice() {
        let (s, e, state, event, mut p) = fixture();
        p["permission_operations"] = json!([{"source_id":"DOC:actual","read_segments":["body"]}]);
        assert!(validate_proposal(&p, &event, &s, &e, &state, false).is_ok());
        assert!(validate_proposal(&p, &event, &s, &e, &state, true)
            .unwrap_err()
            .contains("already applied"));
    }
    #[test]
    fn withdrawn_actual_support_blocks_otherwise_valid_elsewhere_grant_at_commit() {
        let (s, e, state, event, mut p) = fixture();
        s.register(json!({"id":"OTHER","revision":1,"text":"other private body","segments":{"body":[1,1]},"local_allowed":false,"external_allowed":false})).unwrap();
        p["permission_operations"] = json!([{"source_id":"OTHER","read_segments":["body"]}]);
        assert!(validate_proposal(&p, &event, &s, &e, &state, false).is_ok());
        s.permission(&json!({"source_id":"DOC:actual","read_segments":[],"external_segments":[]}))
            .unwrap();
        let before = s.catalog();
        let result = s.permission_batch_checked(&p["permission_operations"], || {
            validate_proposal(&p, &event, &s, &e, &state, false)
        });
        assert!(result.unwrap_err().contains("SOURCE_AUTHORITY_CHANGED"));
        assert_eq!(s.catalog(), before);
        assert!(!s.allowed("OTHER", 1, 1, 1, false));
    }
    #[test]
    fn invalid_extent_not_reinterpreted_as_line_range() {
        let (s, e, state, event, mut p) = fixture();
        p["targets"] = json!(["DOC:actual:1:1-2"]);
        assert!(validate_proposal(&p, &event, &s, &e, &state, false)
            .unwrap_err()
            .contains("canonical reference"));
    }
}

#[cfg(test)]
mod deadline_adoption_tests {
    use super::*;
    fn engine() -> Arc<Engine> {
        let config=Configuration::from_value(&json!({"id":"AAAA","evidence":"A","execution":"A","grounding":"A","handoff":"A"})).unwrap();
        Arc::new(Engine{config,sources:SourceRegistry::new(),evidence:EvidenceStore::new("A"),handoff:HandoffStore::new("A"),state:Mutex::new(State::default()),changed:Condvar::new(),queue:Mutex::new(VecDeque::new()),background:Mutex::new(Vec::new())})
    }
    #[test]
    fn post_publication_result_rechecks_current_revision_and_task_association() {
        let mut s=State::default();s.requests.insert("g".into(),json!({"revision":1,"task_id":"T"}));s.tasks.push(json!({"task_id":"T","state":"RUNNING"}));
        let event=json!({"id":"actual-result","kind":"agent","task_id":"T"});let ingress=json!({"kind":"DELEGATE","contract":{"mandatory":{"task_id":"T"}}});
        assert!(require_current_result_state(&s,&event,&ingress,"g",1).is_ok());
        s.requests.get_mut("g").unwrap()["revision"]=json!(2);
        assert!(require_current_result_state(&s,&event,&ingress,"g",1).unwrap_err().contains("STALE_POST_PUBLICATION"));
        s.requests.get_mut("g").unwrap()["revision"]=json!(1);s.requests.get_mut("g").unwrap()["task_id"]=json!("new-task");
        assert!(require_current_result_state(&s,&event,&ingress,"g",1).unwrap_err().contains("DIFFERENT_TASK"));
        assert_eq!(s.tasks[0]["state"],"RUNNING");
    }
    #[test]
    fn delivered_source_registration_cannot_roll_back_after_newer_revision() {
        let e=engine();e.state.lock().unwrap().requests.insert("g".into(),json!({"revision":1}));
        e.sources.register(json!({"id":"D","revision":2,"text":"newer actually displayed body"})).unwrap();
        let ctx=Context::new();let event=json!({"id":"old","kind":"user","delivery_source_id":"D"});
        let result=e.register_delivery(&ctx,&event,"g",1,"old body",&json!({}),&EvidenceResult::default(),"old-publication");
        assert!(result.unwrap_err().contains("POINTER_ROLLBACK"));assert_eq!(e.sources.current("D").unwrap()["revision"],2);
        e.state.lock().unwrap().requests.get_mut("g").unwrap()["revision"]=json!(2);
        assert!(e.register_delivery(&ctx,&event,"g",1,"old body",&json!({}),&EvidenceResult::default(),"old-publication").unwrap_err().contains("STALE_POST_PUBLICATION"));
        assert_eq!(e.sources.current("D").unwrap()["text"],"newer actually displayed body");
    }
    #[test]
    fn adoption_batch_rechecks_after_actual_state_lock_wait() {
        let config=Configuration::from_value(&json!({"id":"AAAA","evidence":"A","execution":"A","grounding":"A","handoff":"A"})).unwrap();
        let engine=Arc::new(Engine{config,sources:SourceRegistry::new(),evidence:EvidenceStore::new("A"),handoff:HandoffStore::new("A"),state:Mutex::new(State::default()),changed:Condvar::new(),queue:Mutex::new(VecDeque::new()),background:Mutex::new(Vec::new())});
        let held=engine.state.lock().unwrap();let ctx=Context::new();let deadline=crate::lifecycle::host_monotonic_ns()+500_000_000;ctx.test_deadline(deadline);
        let e=engine.clone();let c=ctx.clone();let(tx,rx)=std::sync::mpsc::channel();
        let worker=thread::spawn(move||{let effect=c.effect_guard().unwrap();tx.send(()).unwrap();let mut plan=json!({"relation":"NEW","route":"DELEGATE","purpose":"actual test purpose","allowed_actions":[]});e.resolve_goal(&c,&json!({"id":"held-input","text":"actual test input"}),&mut plan,&effect)});
        rx.recv_timeout(Duration::from_secs(2)).unwrap();let wait=deadline.saturating_sub(crate::lifecycle::host_monotonic_ns())+1_000_000;thread::sleep(Duration::from_nanos(wait));drop(held);
        assert!(worker.join().unwrap().unwrap_err().contains("OBSERVATION_DEADLINE"));let state=engine.state.lock().unwrap();assert!(state.requests.is_empty());assert!(state.tasks.is_empty());
    }
}
