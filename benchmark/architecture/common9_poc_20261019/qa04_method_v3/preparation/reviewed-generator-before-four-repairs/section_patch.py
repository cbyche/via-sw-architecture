"""C04 transaction generator. Preparation only until exact final source rebase.

Only independent copied candidates may be passed to apply(). No provider calls.
"""
from pathlib import Path
from extensions import once, C04_HELPERS


SOURCE_TRANSACTION = r'''
    /// Caller holds the registry source guard and the owning runtime state lock.
    /// All fallible preparation and replacement-map construction precede check.
    /// The apply callback contains only precomputed, bounded state assignments.
    pub fn qa04_display_transaction<C>(
        &self, _source_guard: &std::sync::MutexGuard<'_, ()>, expected_revision: u64,
        mut source: Value, check: impl FnOnce() -> Result<C, String>, apply: impl FnOnce(),
    ) -> Result<C, String> {
        let id=source["id"].as_str().ok_or("SECTION_SOURCE_ID_MISSING")?.to_owned();
        let revision=source["revision"].as_u64().ok_or("SECTION_SOURCE_REVISION_MISSING")?;
        if revision!=expected_revision.checked_add(1).ok_or("SECTION_SOURCE_REVISION_OVERFLOW")? {return Err("SECTION_SOURCE_REVISION_NOT_NEXT".into());}
        if source.get("path").is_some(){return Err("SECTION_SOURCE_MUST_BE_PREPARED_TEXT".into());}
        let lines=source["text"].as_str().filter(|s|!s.is_empty()).ok_or("SECTION_SOURCE_TEXT_NOT_PREPARED")?.lines().count();
        source["line_count"]=json!(lines);
        source["allowed_segments"]=if source["local_allowed"]==false{json!([])}else{json!([[1,lines]])};
        source["external_segments"]=if source["external_allowed"]==false{json!([])}else{source["allowed_segments"].clone()};
        let mut registry=self.inner.lock().unwrap();
        if registry.current.get(&id).copied()!=Some(expected_revision){return Err("SECTION_SOURCE_REVISION_CHANGED".into());}
        let mut versions=registry.versions.clone();let mut current=registry.current.clone();
        versions.insert((id.clone(),revision),source);current.insert(id,revision);
        let receipt=check()?;
        registry.versions=versions;registry.current=current;apply();
        Ok(receipt)
    }
'''


RUNTIME_TRANSACTION = r'''
fn qa04_section_plan(state:&State,sources:&SourceRegistry,event:&Value)->Result<Value,String>{
    let target=event["target_event_id"].as_str().ok_or("SECTION_TARGET_MISSING")?;
    let index=state.delivered.iter().rposition(|r|r["phase"]=="AGENT_RESULT"&&r["stimulus_id"]==target).ok_or("SECTION_TARGET_NOT_ACTUAL_AGENT_RESULT")?;
    let mut one=vec![state.delivered[index].clone()];let mut edit=qa04_section_edit(&mut one,event)?;
    let after=edit["after"].clone();let task_id=after["task_id"].as_str().ok_or("SECTION_TASK_MISSING")?;
    let task_index=state.tasks.iter().position(|t|t["task_id"]==task_id).ok_or("SECTION_TASK_UNAVAILABLE")?;
    let existing=&state.tasks[task_index]["result_response"];
    for (field,key) in [("delivery_event_id","stimulus_id"),("delivery_source_id","delivery_source_id"),("goal_revision","revision")] {
        if existing[field]!=edit["before"][key]{return Err("SECTION_TASK_RESULT_IDENTITY_CHANGED".into());}
    }
    if existing["section_revision"].as_u64().unwrap_or(1)!=edit["before"]["section_revision"].as_u64().unwrap_or(1)||existing["text"]!=edit["before"]["text"]{return Err("SECTION_TASK_RESULT_REVISION_CHANGED".into());}
    for span in array(&after["source_spans"]){if !crate::evidence::permitted(sources,&span,false){return Err("SECTION_SOURCE_PERMISSION_CHANGED".into());}}
    let id=after["delivery_source_id"].as_str().ok_or("SECTION_DELIVERY_SOURCE_MISSING")?;
    let prior=sources.current(id).ok_or("SECTION_DISPLAY_SOURCE_UNAVAILABLE")?;
    let revision=prior["revision"].as_u64().ok_or("SECTION_DISPLAY_SOURCE_REVISION_MISSING")?;
    let lines=prior["line_count"].as_u64().unwrap_or(0)as usize;
    if !sources.allowed(id,revision,1,lines,false){return Err("SECTION_DISPLAY_SOURCE_NOT_AUTHORIZED".into());}
    let external=sources.allowed(id,revision,1,lines,true)&&array(&after["source_spans"]).iter().all(|p|crate::evidence::permitted(sources,p,true));
    let next=revision.checked_add(1).ok_or("SECTION_SOURCE_REVISION_OVERFLOW")?;
    let prepared=json!({"id":id,"revision":next,"kind":"delivered","text":after["text"],"provenance":after["source_spans"],"local_allowed":true,"external_allowed":external,"extents":{"delivered":[1,after["text"].as_str().ok_or("SECTION_TEXT_MISSING")?.lines().count()]}});
    edit["delivery_index"]=json!(index);edit["task_index"]=json!(task_index);edit["source_base_revision"]=json!(revision);edit["prepared_source"]=prepared;
    Ok(edit)
}
fn qa04_same_section_basis(a:&Value,b:&Value)->bool{
    a["before"]==b["before"]&&a["source_base_revision"]==b["source_base_revision"]&&a["prepared_source"]==b["prepared_source"]
}
fn qa04_finish_section(engine:&Arc<Engine>,ctx:&Context,event:&Value,admission:&Value,receipt:&Value)->Result<Value,String>{
    let publication=receipt["publication_id"].as_str().ok_or("SECTION_RECEIPT_ID_MISSING")?;
    let effect=ctx.effect_guard()?;let source_guard=engine.sources.guard();let mut state=engine.state.lock().unwrap();
    if state.qa04_section_history.iter().any(|h|h["publication_id"]==publication||h["section_edit_event_id"]==event["id"]){return Err("SECTION_RECEIPT_OR_EDIT_ALREADY_COMMITTED".into());}
    let live=qa04_section_plan(&state,&engine.sources,event);
    let reason=match &live {Err(e)=>Some(e.clone()),Ok(now)if !qa04_same_section_basis(admission,now)=>Some("SECTION_TARGET_OR_SOURCE_CHANGED_AFTER_DISPLAY".into()),_=>None};
    let mut history=admission["after"].clone();history["phase"]=json!("QA04_SECTION_DISPLAY");history["publication_id"]=json!(publication);history["actual_host_observed_ns"]=receipt["observed_ns"].clone();history["section_edit_event_id"]=event["id"].clone();history["applied"]=json!(reason.is_none());history["rejection_reason"]=json!(reason);history["historical_display_only"]=json!(reason.is_some());
    // Clone LIVE history while its owning lock is held, never a pre-ACK snapshot.
    let mut history_after=state.qa04_section_history.clone();history_after.push(history.clone());
    let tasks_before=state.tasks.clone();let requests_before=state.requests.clone();
    let counter=state.counter+1;
    if let Some(reason)=reason {
        let result=json!({"applied":false,"event_id":event["id"],"error":reason,"history":history,"historical_display_recorded":true,"task_state_before":tasks_before,"task_state_after":state.tasks,"request_state_before":requests_before,"request_state_after":state.requests});
        let commit=ctx.commit_effect(&effect,"QA04_SECTION_HISTORICAL_DISPLAY",&json!({"publication_id":publication,"stimulus_id":event["id"],"actual_host_observed_ns":receipt["observed_ns"],"applied":false,"reason":reason}))?;
        state.qa04_section_history=history_after;state.counter=counter;
        drop(state);drop(source_guard);drop(effect);drop(commit);return Ok(result);
    }
    let edit=live?;let di=edit["delivery_index"].as_u64().ok_or("SECTION_DELIVERY_INDEX_MISSING")?as usize;let ti=edit["task_index"].as_u64().ok_or("SECTION_TASK_INDEX_MISSING")?as usize;
    let mut canonical=edit["after"].clone();canonical["publication_id"]=json!(publication);canonical["actual_host_observed_ns"]=receipt["observed_ns"].clone();
    let task_result=qa04_task_result(&canonical);let mut tasks_after=tasks_before.clone();tasks_after[ti]["result_response"]=task_result.clone();
    let result=json!({"applied":true,"event_id":event["id"],"before":edit["before"],"after":canonical,"history":history,"historical_display_recorded":true,"registered_display_revision":edit["prepared_source"]["revision"],"task_state_before":tasks_before,"task_state_after":tasks_after,"request_state_before":requests_before,"request_state_after":state.requests});
    let audit=json!({"publication_id":publication,"stimulus_id":event["id"],"actual_host_observed_ns":receipt["observed_ns"],"target_event_id":event["target_event_id"],"section_revision":canonical["section_revision"],"source_revision":edit["prepared_source"]["revision"],"applied":true});
    let commit=engine.sources.qa04_display_transaction(&source_guard,edit["source_base_revision"].as_u64().ok_or("SECTION_BASE_REVISION_MISSING")?,edit["prepared_source"].clone(),||ctx.commit_effect(&effect,"QA04_SECTION_CANONICAL_TRANSACTION",&audit),||{
        state.delivered[di]=canonical;state.tasks[ti]["result_response"]=task_result;state.qa04_section_history=history_after;state.counter=counter;
    })?;
    drop(state);drop(source_guard);drop(effect);drop(commit);Ok(result)
}
fn qa04_handle_section(engine:&Arc<Engine>,ctx:&Context,event:&Value)->Result<Value,String>{
    let edit_id=event["id"].as_str().filter(|s|!s.is_empty()).ok_or("SECTION_EDIT_ID_MISSING")?.to_owned();
    let admission={
        let effect=ctx.effect_guard()?;let source_guard=engine.sources.guard();let mut state=engine.state.lock().unwrap();
        if state.qa04_section_seen.contains(&edit_id){return Err("SECTION_EDIT_REPLAY".into());}
        let plan=qa04_section_plan(&state,&engine.sources,event)?;let reserved=edit_id.clone();
        let commit=ctx.commit_effect(&effect,"QA04_SECTION_ATTEMPT",&json!({"event_id":edit_id,"target_event_id":event["target_event_id"]}))?;
        state.qa04_section_seen.insert(reserved);drop(state);drop(source_guard);drop(effect);drop(commit);plan
    };
    let mut proposed=admission["after"].clone();proposed["phase"]=json!("QA04_SECTION_DISPLAY");proposed["section_edit_event_id"]=event["id"].clone();
    let receipt=ctx.local_operation("publication_host_receipt_wait","VIA",||ctx.request_publication_checked(&proposed,||{
        let _source=engine.sources.guard();let state=engine.state.lock().unwrap();let live=qa04_section_plan(&state,&engine.sources,event)?;
        if !qa04_same_section_basis(&admission,&live){return Err("SECTION_TARGET_OR_SOURCE_CHANGED_BEFORE_DISPLAY".into());}Ok(())
    }))?;
    let result=qa04_finish_section(engine,ctx,event,&admission,&receipt)?;
    if result["applied"]==true{engine.activity(ctx,json!({"kind":"RESPONSE_SECTION_REVISED","id":result["after"]["delivery_source_id"]}));}
    Ok(result)
}
'''


RUNTIME_BRANCH = r'''
if event["kind"]=="qa04_section_update"{
    // Main ingress must stay able to process actual source/authority changes.
    // The worker acquires its own fresh gates and owns no locks during receipt wait.
    drop(_input_effect);let e=engine.clone();let c=ctx.without_boundary();let edit=event.clone();
    engine.background.lock().unwrap().push(thread::spawn(move||{
        match qa04_handle_section(&e,&c,&edit){
            Ok(result)=>c.event(if result["applied"]==true{"QA04_SECTION_UPDATE_APPLIED"}else{"QA04_SECTION_UPDATE_REJECTED"},&result),
            Err(error)=>c.event("QA04_SECTION_UPDATE_REJECTED",&json!({"event_id":edit["id"],"error":error,"applied":false})),
        }
        c.event("QA04_FEATURE_RESPONSE_FINAL",&json!({"event_id":edit["id"],"kind":"section_update"}));
    }));continue;
}
'''


def apply(crate: Path, configuration=None):
    source_file = crate/'src/sources.rs'
    source = source_file.read_text()
    source = once(source, 'pub fn current(&self,id:&str)', SOURCE_TRANSACTION+'\npub fn current(&self,id:&str)')
    source_file.write_text(source)
    runtime_file = crate/'src/runtime.rs'
    text = runtime_file.read_text()
    text = once(text, 'delivered: Vec<Value>,', 'delivered: Vec<Value>,qa04_section_history:Vec<Value>,qa04_section_seen:BTreeSet<String>,')
    text = once(text, '"text":text,"phase":phase,', '"text":text,"phase":phase,"sections":qa04_sections(text),"section_revision":1,')
    text = once(text, '"delivery_source_id":event["delivery_source_id"],', '"delivery_source_id":if phase=="AGENT_RESULT"{event["result_source"]["id"].clone()}else{event["delivery_source_id"].clone()},')
    # Precompute the extra Task metadata before the baseline final pure check.
    anchor='let _commit=ctx.commit_effect(&effect,"PUBLICATION_DELIVERY_STATE",&json!({"stimulus_id":event["id"],"goal_id":gid,"revision":revision,"publication_id":publication,"actual_host_observed_ns":receipt["observed_ns"],"complete":actual_complete,"historical_display_recorded":true,"completion_ineligible_reason":completion_error}))?;'
    text = once(text, anchor, 'let qa04_task_index=if phase=="AGENT_RESULT"&&completion_error.is_none(){state.tasks.iter().position(|t|t["task_id"]==output["task_id"])}else{None};let qa04_result=qa04_task_result(&output);'+anchor)
    text = once(text, 'state.delivered.push(output.clone());', 'state.delivered.push(output.clone());if let Some(i)=qa04_task_index{state.tasks[i]["result_response"]=qa04_result;}')
    text = once(text, 'if let Err(error)=engine.capture(&ctx,event,&_input_effect)', RUNTIME_BRANCH+'\nif let Err(error)=engine.capture(&ctx,event,&_input_effect)')
    text = once(text, '"configuration":engine.config.id,"requests":s.requests,', '"configuration":engine.config.id,"qa04_section_history":s.qa04_section_history,"qa04_section_seen":s.qa04_section_seen,"requests":s.requests,')
    runtime_file.write_text(text+'\n'+C04_HELPERS+'\n'+RUNTIME_TRANSACTION)
