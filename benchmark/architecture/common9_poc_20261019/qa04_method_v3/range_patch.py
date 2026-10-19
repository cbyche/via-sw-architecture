"""C06 guarded range-withdrawal generator; independent frozen copies only."""
from pathlib import Path
from extensions import once, C06_SOURCE_HELPERS


SOURCE_METHODS = r'''
    pub fn withdraw_range(&self,operation:&Value)->Result<(),String>{
        let owner=self.guard();self.qa04_withdraw_range_checked(&owner,operation,||Ok(()),||{}).map(|_|())
    }
    /// Caller owns the source guard and any affected runtime state lock.
    pub fn qa04_withdraw_range_checked<C>(&self,_owner:&std::sync::MutexGuard<'_,()>,operation:&Value,check:impl FnOnce()->Result<C,String>,apply:impl FnOnce())->Result<C,String>{
        let id=operation["source_id"].as_str().ok_or("RANGE_SOURCE_MISSING")?;
        let a=operation["start_line"].as_u64().ok_or("RANGE_START_MISSING")?;
        let b=operation["end_line"].as_u64().ok_or("RANGE_END_MISSING")?;
        if a==0||b<a{return Err("RANGE_INVALID".into());}
        let mut registry=self.inner.lock().unwrap();
        let revision=*registry.current.get(id).ok_or("RANGE_SOURCE_UNAVAILABLE")?;
        if operation["revision"].as_u64()!=Some(revision){return Err("RANGE_SOURCE_REVISION_STALE".into());}
        let n=registry.versions[&(id.to_owned(),revision)]["line_count"].as_u64().unwrap_or(0);
        if b>n{return Err("RANGE_BEYOND_CURRENT_SOURCE".into());}
        let mut versions=registry.versions.clone();
        for((sid,_),source)in versions.iter_mut(){
            if sid!=id{continue;}
            for key in ["allowed_segments","external_segments"]{source[key]=qa04_subtract_ranges(&source[key],a,b);}
            let mut withdrawn=source["withdrawn_ranges"].as_array().cloned().unwrap_or_default();
            withdrawn.push(json!([a,b]));source["withdrawn_ranges"]=json!(withdrawn);source["range_access_epoch"]=operation["access_epoch"].clone();
        }
        // No locks, I/O or fallible preparation between this check and batch.
        let receipt=check()?;registry.versions=versions;apply();Ok(receipt)
    }
'''

RUNTIME_BRANCH = r'''
if event["kind"]=="qa04_withdraw_range"{
    let result=(||->Result<(),String>{
        let source_guard=engine.sources.guard();let mut state=engine.state.lock().unwrap();let counter=state.counter+1;
        let audit=json!({"event_id":event["id"],"source_id":event["source_id"],"revision":event["revision"],"start_line":event["start_line"],"end_line":event["end_line"]});
        let commit=engine.sources.qa04_withdraw_range_checked(&source_guard,event,||ctx.commit_effect(&_input_effect,"QA04_SOURCE_RANGE_WITHDRAWAL",&audit),||{state.counter=counter;})?;
        drop(state);drop(source_guard);drop(commit);Ok(())
    })();
    drop(_input_effect);
    match result{
        Ok(())=>{engine.activity(&ctx,json!({"kind":"SOURCE_RANGE_WITHDRAWAL","id":event["source_id"]}));ctx.event("QA04_RANGE_WITHDRAWAL_APPLIED",event);},
        Err(error)=>ctx.event("QA04_RANGE_WITHDRAWAL_REJECTED",&json!({"event_id":event["id"],"error":error})),
    }
    ctx.event("QA04_FEATURE_RESPONSE_FINAL",&json!({"event_id":event["id"],"kind":"withdraw_range"}));continue;
}
'''


def apply(crate: Path, configuration=None):
    path=crate/'src/sources.rs';text=path.read_text()
    text=once(text,'pub fn current(&self,id:&str)',SOURCE_METHODS+'\npub fn current(&self,id:&str)')
    anchor='source["external_segments"]=old["external_segments"].clone();'
    text=once(text,anchor,anchor+'if old.get("withdrawn_ranges").is_some(){source["withdrawn_ranges"]=old["withdrawn_ranges"].clone();}')
    anchor='let Some(src)=self.version(id,revision)else{return false;};'
    text=once(text,anchor,anchor+'if src["withdrawn_ranges"].as_array().is_some_and(|ranges|ranges.iter().any(|r|r[0].as_u64().is_some_and(|a|a<=end_line as u64)&&r[1].as_u64().is_some_and(|b|b>=start_line as u64))){return false;}')
    anchor='updates.push(((sid.clone(), *revision), field.into(), json!(ranges)));'
    text=once(text,anchor,'let mut permitted_ranges=json!(ranges);for withdrawn in source["withdrawn_ranges"].as_array().cloned().unwrap_or_default(){permitted_ranges=qa04_subtract_ranges(&permitted_ranges,withdrawn[0].as_u64().unwrap_or(0),withdrawn[1].as_u64().unwrap_or(0));}updates.push(((sid.clone(), *revision), field.into(), permitted_ranges));')
    path.write_text(text+'\n'+C06_SOURCE_HELPERS)
    path=crate/'src/runtime.rs';text=path.read_text()
    anchor='if let Err(error)=engine.capture(&ctx,event,&_input_effect)'
    text=once(text,anchor,RUNTIME_BRANCH+'\n'+anchor);path.write_text(text)
