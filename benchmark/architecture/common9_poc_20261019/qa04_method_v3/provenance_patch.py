"""C05 delivered-only provenance generator; independent frozen copies only."""
from pathlib import Path
from extensions import once, C05_HELPERS


RUNTIME_HELPER = r'''
fn qa04_handle_provenance(engine:&Arc<Engine>,ctx:&Context,event:&Value)->Result<(),String>{
    let(result,before,after,sources_before,sources_after)={
        let effect=ctx.effect_guard()?;let source_guard=engine.sources.guard();let state=engine.state.lock().unwrap();
        let before=qa04_readonly_state(&state);let sources_before=engine.sources.catalog();
        let result=qa04_query_provenance(&state.delivered,&engine.sources,event);
        let after=qa04_readonly_state(&state);let sources_after=engine.sources.catalog();
        ctx.check_effect(&effect)?;assert_eq!(before,after);assert_eq!(sources_before,sources_after);
        drop(state);drop(source_guard);drop(effect);(result,before,after,sources_before,sources_after)
    };
    ctx.event("QA04_PROVENANCE_READONLY_STATE",&json!({"event_id":event["id"],"before":before,"after":after,"sources_before":sources_before,"sources_after":sources_after}));
    let display=json!({"stimulus_id":event["id"],"phase":"QA04_PROVENANCE","content_type":"PROVENANCE_PANEL","text":result.to_string(),"provenance_panel":result});
    let receipt=ctx.local_operation("publication_host_receipt_wait","VIA",||ctx.request_publication_checked(&display,||{
        let _source=engine.sources.guard();let state=engine.state.lock().unwrap();
        if qa04_query_provenance(&state.delivered,&engine.sources,event)!=result{return Err("PROVENANCE_PANEL_CHANGED_BEFORE_DISPLAY".into());}Ok(())
    }))?;
    ctx.check_active()?;
    let mut actual=result;actual["publication_id"]=receipt["publication_id"].clone();actual["actual_host_observed_ns"]=receipt["observed_ns"].clone();
    ctx.event("QA04_PROVENANCE_PANEL_DISPLAY",&actual);Ok(())
}
'''

RUNTIME_BRANCH = r'''
if event["kind"]=="qa04_provenance_query"{
    drop(_input_effect);let e=engine.clone();let c=ctx.without_boundary();let query=event.clone();
    engine.background.lock().unwrap().push(thread::spawn(move||{
        if let Err(error)=qa04_handle_provenance(&e,&c,&query){c.event("QA04_PROVENANCE_QUERY_REJECTED",&json!({"event_id":query["id"],"error":error}));}
        c.event("QA04_FEATURE_RESPONSE_FINAL",&json!({"event_id":query["id"],"kind":"provenance_query"}));
    }));continue;
}
'''


def apply(crate: Path, configuration=None):
    path=crate/'src/runtime.rs';text=path.read_text()
    anchor='let _adoption_effect=ctx.effect_guard()?;'
    text=once(text,anchor,'plan["qa04_adopted_geometry"]=qa04_actual_geometry(&plan,&grounded);'+anchor)
    anchor='"bindings":plan["bindings"],"targets":plan["targets"]'
    text=once(text,anchor,'"adopted_geometry_provenance":plan["qa04_adopted_geometry"],'+anchor)
    anchor='if let Err(error)=engine.capture(&ctx,event,&_input_effect)'
    text=once(text,anchor,RUNTIME_BRANCH+'\n'+anchor)
    path.write_text(text+'\n'+C05_HELPERS+'\n'+RUNTIME_HELPER)
