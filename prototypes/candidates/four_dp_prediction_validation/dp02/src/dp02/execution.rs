//! A owns one conversation step sequence; B owns concurrent per-request execution.
use serde_json::{json,Value};
use std::time::Duration;
use crate::context::Context;
use super::{interpretation,response,state::{self,Shared,Request}};

fn input_id(event:&Value)->String{event["id"].as_str().unwrap_or("missing-input-id").into()}
fn adopt(ctx:&mut Context,event:&Value,plan:&Value,state:&Shared)->Result<Vec<String>,String>{
    let eid=input_id(event);let mut goals=Vec::new();
    for (index,intent) in plan["intents"].as_array().ok_or("intents missing")?.iter().enumerate(){
        let relation=intent["relation"].as_str().unwrap_or("");
        let target=intent["target_request_id"].as_str().unwrap_or("");
        let goal=match relation{"CONTINUE"|"CORRECTION"=>target.to_string(),"ANSWER"=>format!("{}:{}",intent["task_id"].as_str().unwrap_or("unresolved-task"),eid),_=>if goals.is_empty(){eid.clone()}else{format!("{}:intent{}",eid,index+1)}};
        let mut s=state.0.lock().unwrap();
        if relation=="CORRECTION"{
            if !s.requests.contains_key(&goal){
                let original=s.captured.iter().find(|e|e["id"]==goal).cloned().ok_or("correction target not actually captured")?;
                s.requests.insert(goal.clone(),Request{goal_id:goal.clone(),stimulus_id:goal.clone(),raw:original["text"].as_str().unwrap_or("").into(),revision:1,scope:original["text"].as_str().unwrap_or("").into(),adoption:Value::Null,correction:None,delivered:None});
            }
            let r=s.requests.get_mut(&goal).unwrap();r.revision+=1;r.scope=intent["scope"].as_str().unwrap_or("").into();r.correction=Some(json!({"event":event,"adopted_intent":intent}));r.delivered=None;
        }else if relation!="CONTINUE"{
            if let Some(r)=s.requests.get_mut(&goal){
                if r.correction.is_none(){r.adoption=intent.clone();r.scope=intent["scope"].as_str().unwrap_or("").into();}
            }else{s.requests.insert(goal.clone(),Request{goal_id:goal.clone(),stimulus_id:eid.clone(),raw:event["text"].as_str().unwrap_or("").into(),revision:1,scope:intent["scope"].as_str().unwrap_or("").into(),adoption:intent.clone(),correction:None,delivered:None});}
        }
        let revision=s.requests.get(&goal).map(|r|r.revision).unwrap_or(1);
        s.associations.push(json!({"event_id":eid,"intent_index":index,"revision":event["revision"],"goal_id":goal,"goal_revision":revision,"intent":intent}));drop(s);
        ctx.event("GOAL_LINK",&json!({"stimulus_id":eid,"goal_id":goal,"revision":revision,"relation":relation,"target_request_id":intent["target_request_id"],"question_id":intent["question_id"],"task_id":intent["task_id"],"goal_scope":if relation=="ANSWER"{"ANSWER_DELIVERY_TO_EXISTING_TASK"}else{"USER_REQUEST"}}));
        if !["CONTINUE","CORRECTION"].contains(&relation){
            ctx.goal_start(&goal,&json!({"stimulus_id":eid,"text":event["text"],"revision":revision,"goal_scope":if relation=="ANSWER"{"ANSWER_DELIVERY_TO_EXISTING_TASK"}else{"USER_REQUEST"}}));goals.push(goal.clone());
        }else if relation=="CORRECTION"{
            let origin=state::request(state,&goal).map(|r|r.stimulus_id).unwrap_or_else(||goal.clone());
            ctx.goal_start(&goal,&json!({"stimulus_id":origin,"revision":revision,"goal_scope":"USER_REQUEST","reason":"same original goal; no start reset"}));
            ctx.event("GOAL_REVISION",&json!({"goal_id":goal,"revision":revision,"stimulus_id":eid,"scope":intent["scope"]}));
        }
        let latest=state::request(state,&goal);
        let display_intent=if relation=="NEW" && revision>1 {latest.as_ref().and_then(|r|r.correction.as_ref()).map(|c|c["adopted_intent"].clone()).unwrap_or_else(||intent.clone())}else{intent.clone()};
        if display_intent!=*intent{ctx.event("STALE_INITIAL_SCOPE_NOT_COMMITTED",&json!({"goal_id":goal,"current_revision":revision,"initial_proposal":intent,"current_adopted_intent":display_intent}));}
        let progress=json!({"adoption_id":format!("{}-{}",eid,index),"resolved":true,"text":display_intent["handling_text"],"targets":display_intent["targets"],"scope":display_intent["scope"],"handling":display_intent["relation"],"source_revisions":display_intent["source_revisions"],"raw_utterance":event["text"],"revision":revision,"decision":intent});
        ctx.progress(&goal,&eid,&progress)?;
    }
    state::record_size(state,ctx);Ok(goals)
}
fn finish(ctx:&mut Context,input:&Value,goal_id:&str,state:&Shared)->Result<(),String>{
    for revision_attempt in 0..=1{
        let request=state::request(state,goal_id).ok_or("goal request state absent")?;state::mark_response_attempt(state,goal_id,request.revision);
        let result=if request.correction.is_none(){response::fast_answer(&request.adoption,&request,&input["sources"])}else{None};
        let value=if let Some(v)=result{ctx.event("RESPONSE_BRANCH",&json!({"goal_id":goal_id,"path":"valid_complete_answer_from_adoption"}));v}else{response::detailed(ctx,input,&request,state)?};
        state::wait_relations(state,ctx,goal_id);
        let current=state::request(state,goal_id).ok_or("goal disappeared")?;
        if current.revision!=request.revision{
            ctx.event("STALE_RESULT_RETAINED_NOT_ADOPTED",&json!({"goal_id":goal_id,"draft_revision":request.revision,"current_revision":current.revision,"draft":value}));
            if revision_attempt==1{return Err("revision changed beyond bounded rejudgment".into())}continue;
        }
        let mut terminal=json!({"goal_id":goal_id,"stimulus_id":request.stimulus_id,"revision":request.revision,"text":value["answer"],"content_contract_checked":true,"checks":["nonempty detailed sections","current goal revision","actual supplied quote checks"],"result":value,"goal_scope":"USER_REQUEST"});
        if request.adoption["relation"]=="ANSWER"{
            let question=request.adoption["question_id"].clone();let task=request.adoption["task_id"].clone();
            ctx.event("DOWNSTREAM_SHIM_DISPATCH",&json!({"goal_id":goal_id,"kind":"ANSWER_DELIVERY","task_id":task,"question_id":question,"answer":value["answer"]}));
            // Deterministic external acceptance harness: actual argument validation and receipt.
            let accepted=state.0.lock().unwrap().questions.iter().any(|q|q["id"]==question && q["task_id"]==task);
            let receipt=json!({"goal_id":goal_id,"task_id":task,"question_id":question,"accepted":accepted,"external_wait_ns":0,"implementation":"fixed answer-delivery acceptance shim; not email-domain execution"});
            ctx.event("DOWNSTREAM_SHIM_RECEIPT",&receipt);if !accepted{return Err("answer delivery rejected".into())}
            terminal["delivery_receipt"]=receipt;terminal["goal_scope"]=json!("ANSWER_DELIVERY_TO_EXISTING_TASK");
        }
        ctx.complete(goal_id,&terminal)?;
        let mut s=state.0.lock().unwrap();if let Some(r)=s.requests.get_mut(goal_id){r.delivered=Some(terminal.clone());}s.outputs.push(terminal);drop(s);state::record_size(state,ctx);return Ok(())
    }Err("bounded revision recovery exhausted".into())
}
fn fail(ctx:&Context,id:&str,error:String){ctx.fail(id,&json!({"reason":error,"completion_observed":false}));}
fn first_step(ctx:&mut Context,input:&Value,event:&Value,state:&Shared)->Result<(Value,Vec<String>),String>{let p=interpretation::interpret(ctx,input,event,state,None)?;let ids=adopt(ctx,event,&p,state)?;state::resolve_input(state,event,ctx.last_node());Ok((p,ids))}
fn process_later(ctx:&mut Context,input:&Value,event:&Value,state:&Shared,retained:Option<&Value>)->Vec<String>{
    match interpretation::interpret(ctx,input,event,state,retained).and_then(|p|adopt(ctx,event,&p,state)){
        Ok(goals)=>{state::resolve_input(state,event,ctx.last_node());goals},Err(error)=>{state::resolve_input(state,event,ctx.last_node());fail(ctx,&format!("UNRESOLVED:{}",input_id(event)),error);Vec::new()}
    }
}
pub fn run(input:&Value,ctx:&mut Context)->Result<Value,String>{
    let events=input["events"].as_array().filter(|a|a.len()>=2).ok_or("two user events required")?;
    let shared=state::new();let first=&events[0];state::capture(&shared,ctx,first);
    let mut first_ctx=ctx.scoped(&[format!("input:{}",input_id(first))],&[]);
    if input["overlap"].as_bool()!=Some(true){
        match first_step(&mut first_ctx,input,first,&shared){Ok((_,goals))=>for g in goals{let mut c=first_ctx.scoped(&[g.clone()],&first_ctx.last_node().into_iter().collect::<Vec<_>>());if let Err(e)=finish(&mut c,input,&g,&shared){fail(&c,&g,e)}},Err(e)=>{state::resolve_input(&shared,first,first_ctx.last_node());fail(&first_ctx,&format!("UNRESOLVED:{}",input_id(first)),e);}}
        let second=&events[1];state::capture(&shared,ctx,second);let mut c=ctx.scoped(&[format!("input:{}",input_id(second))],&[]);let goals=process_later(&mut c,input,second,&shared,None);for g in goals{let mut cc=c.scoped(&[g.clone()],&c.last_node().into_iter().collect::<Vec<_>>());if let Err(e)=finish(&mut cc,input,&g,&shared){fail(&cc,&g,e)}}
    }else{
        let a=input["variant"]=="A";let i=input.clone();let f=first.clone();let s=shared.clone();
        let worker=std::thread::spawn(move||{
            let result=first_step(&mut first_ctx,&i,&f,&s);
            first_ctx.event("STEP_BOUNDARY",&json!({"input_id":f["id"],"variant":i["variant"],"step":"actual semantic model call completed"}));
            if !a{match &result{Ok((_,goals))=>for g in goals{let mut c=first_ctx.scoped(&[g.clone()],&first_ctx.last_node().into_iter().collect::<Vec<_>>());if let Err(e)=finish(&mut c,&i,g,&s){fail(&c,g,e)}},Err(e)=>{state::resolve_input(&s,&f,first_ctx.last_node());fail(&first_ctx,&format!("UNRESOLVED:{}",input_id(&f)),e.clone());}}}
            (result,first_ctx.last_node())
        });
        // Only the authored user's arrival is scheduled; no model delay is simulated.
        std::thread::sleep(Duration::from_millis(input["arrival_ms"].as_u64().unwrap_or(120)));
        let second=&events[1];state::capture(&shared,ctx,second);
        if let Some(q)=input.get("agent_question"){
            ctx.capture(q["id"].as_str().unwrap_or("notice"),&json!({"input_class":"AGENT_QUESTION","text":q["text"],"task_id":q["task_id"]}));
            shared.0.lock().unwrap().questions.push(q.clone());
            ctx.event("NOTICE_DISPLAY",&json!({"stimulus_id":q["id"],"task_id":q["task_id"],"text":q["text"],"content_type":"TASK_QUESTION","not_new_goal":true}));
            ctx.present(&json!({"stimulus_id":q["id"],"task_id":q["task_id"],"text":q["text"],"content_type":"TASK_QUESTION","not_new_goal":true}));
            ctx.event("USER_WAIT_START",&json!({"question_id":q["id"],"task_id":q["task_id"],"goal_scope":"preexisting Task; not unrelated comparison"}));
            std::thread::sleep(Duration::from_millis(30));
            ctx.event("USER_WAIT_END",&json!({"question_id":q["id"],"task_id":q["task_id"]}));
            if events.len()>2{state::capture(&shared,ctx,&events[2]);}
        }
        let (retained,first_goals,first_node,worker)=if a{
            ctx.event("POLICY_WAIT_START",&json!({"stimulus_id":second["id"],"reason":"conversation current model step boundary"}));
            let (r,node)=worker.join().map_err(|_|"first step worker panic")?;
            ctx.event("POLICY_WAIT_END",&json!({"stimulus_id":second["id"],"reason":"conversation step ready","parent_node":node}));
            let (plan,goals)=match r{Ok(x)=>(Some(x.0),x.1),Err(e)=>{state::resolve_input(&shared,first,node.clone());fail(ctx,&format!("UNRESOLVED:{}",input_id(first)),e);(None,Vec::new())}};
            (plan,goals,node,None)
        }else{(None,Vec::new(),None,Some(worker))};
        let mut second_ctx=ctx.scoped(&[format!("input:{}",input_id(second))],&first_node.clone().into_iter().collect::<Vec<_>>());
        if let Some(ref step)=retained{ctx.logical_buffer("dp02_retained_conversation_step",step.to_string().len(),"RETAIN");}
        let later_goals=process_later(&mut second_ctx,input,second,&shared,retained.as_ref());
        drop(retained);ctx.logical_buffer("dp02_retained_conversation_step",0,"RELEASE");
        if let Some(_q)=input.get("agent_question"){if events.len()>2{
            let answer=&events[2];let mut ac=ctx.scoped(&[format!("input:{}",input_id(answer))],&second_ctx.last_node().into_iter().collect::<Vec<_>>());
            let goals=process_later(&mut ac,input,answer,&shared,None);
            for g in goals{let mut cc=ac.scoped(&[g.clone()],&ac.last_node().into_iter().collect::<Vec<_>>());if let Err(e)=finish(&mut cc,input,&g,&shared){fail(&cc,&g,e)}}
        }}
        // A resumes the same conversation after adopting pending input at its real step boundary.
        // B's original request already continues in its own worker and may overlap this response.
        for g in later_goals{let mut c=second_ctx.scoped(&[g.clone()],&second_ctx.last_node().into_iter().collect::<Vec<_>>());if let Err(e)=finish(&mut c,input,&g,&shared){fail(&c,&g,e)}}
        if a{for g in first_goals{let mut c=ctx.scoped(&[g.clone()],&second_ctx.last_node().into_iter().collect::<Vec<_>>());if let Err(e)=finish(&mut c,input,&g,&shared){fail(&c,&g,e)}}}
        if let Some(w)=worker{let _=w.join().map_err(|_|"request worker panic")?;}
        // Only an actually new adopted revision can resume. Exhausting recovery on that same
        // revision is terminal; it cannot silently open a third completion attempt.
        let s=shared.0.lock().unwrap();
        let reopened=s.requests.values().filter(|r|state::needs_reopen(r,s.attempted_response_revision.get(&r.goal_id).copied())).map(|r|r.goal_id.clone()).collect::<Vec<_>>();drop(s);
        for g in reopened{let mut c=ctx.scoped(&[g.clone()],&second_ctx.last_node().into_iter().collect::<Vec<_>>());if let Err(e)=finish(&mut c,input,&g,&shared){fail(&c,&g,e)}}
    }
    // Quiescent corrections also revise the original goal, without resetting its start.
    if input["overlap"].as_bool()!=Some(true){
        let s=shared.0.lock().unwrap();let reopened=s.requests.values().filter(|r|state::needs_reopen(r,s.attempted_response_revision.get(&r.goal_id).copied())).map(|r|r.goal_id.clone()).collect::<Vec<_>>();drop(s);
        for g in reopened{let mut c=ctx.scoped(&[g.clone()],&[]);if let Err(e)=finish(&mut c,input,&g,&shared){fail(&c,&g,e)}}
    }
    let s=shared.0.lock().unwrap();Ok(json!({"outputs":s.outputs,"associations":s.associations,"requests":s.requests.values().map(|r|json!({"goal_id":r.goal_id,"revision":r.revision,"scope":r.scope,"delivered":r.delivered.is_some()})).collect::<Vec<_>>(),"variant":input["variant"]}))
}
