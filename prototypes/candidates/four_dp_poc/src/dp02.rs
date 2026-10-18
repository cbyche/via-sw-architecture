use serde_json::{json,Value};
use crate::context::Context;
use std::time::Duration;

fn interpret(ctx:&mut Context, input:&Value,event:&Value,history:&Value)->Result<Value,String>{
    let mut value=ctx.call("interpret_execution",&json!({"instruction":"You interpret VIA user inputs from source evidence and actual conversation state. The current event may also appear in the capture registry; that duplicate is not an earlier interpreted event. Infer the relation from genuinely earlier inputs and actual delivery state. Return JSON {relation:'NEW'|'FOLLOWUP'|'CORRECTION'|'ANSWER',target_request_id:string|null,question_id:string|null,task_id:string|null,answer:string}. Preserve all unresolved goals. A correction changes only the specified scope. An answer must be associated with the actually displayed Agent question, not whichever other task is active. Give a detailed concise factual answer. Do not invent source facts.","input":{"event":event,"sources":input["sources"],"conversation":history},"max_tokens":1600}))?;
    value["event_id"]=event["id"].clone();value["revision"]=event["revision"].clone();Ok(value)
}
fn stimulus(ctx:&Context,e:&Value){ctx.event("STIMULUS",&json!({"stimulus_id":e["id"],"goal_id":e["id"],"text":e["text"]}));}
fn display(ctx:&Context,e:&Value,v:&Value){let p=json!({"stimulus_id":e["id"],"goal_id":e["id"],"text":v["answer"],"decision":v});ctx.present(&p);ctx.event("GOAL_COMPLETE",&p);}
pub fn run(input:&Value,ctx:&mut Context)->Result<Value,String>{
    let events=input["events"].as_array().ok_or("events missing")?;
    let first=&events[0];stimulus(ctx,first);
    let history=json!({"active_requests":[{"id":first["id"],"text":first["text"],"revision":1,"interpretation_state":"CAPTURED_UNINTERPRETED"}],"delivered":[]});
    let mut outputs=Vec::new();let mut associations=Vec::new();
    if input["overlap"].as_bool()!=Some(true){
        let v=interpret(ctx,input,first,&history)?;display(ctx,first,&v);outputs.push(json!({"goal_id":first["id"],"text":v["answer"]}));associations.push(v.clone());
        let second=&events[1];stimulus(ctx,second);let h=json!({"active_requests":history["active_requests"],"delivered":[v]});
        let v2=interpret(ctx,input,second,&h)?;display(ctx,second,&v2);outputs.push(json!({"goal_id":second["id"],"text":v2["answer"]}));associations.push(v2);
    }else{
        let mut worker=ctx.clone();let i=input.clone();let f=first.clone();let h=history.clone();
        ctx.event("REQUEST_EXECUTION_START",&json!({"request_id":first["id"],"variant":input["variant"]}));
        let mut handle=Some(std::thread::spawn(move||{let r=interpret(&mut worker,&i,&f,&h);worker.event("REQUEST_RESULT_READY",&json!({"request_id":f["id"],"publication_requires_pending_relation_resolution":true}));r}));
        // Authored input arrives during the real model call. No model/service delay is simulated.
        std::thread::sleep(Duration::from_millis(input["arrival_ms"].as_u64().unwrap_or(120)));
        let second=&events[1];stimulus(ctx,second);
        ctx.event("PUBLICATION_GATE",&json!({"request_id":first["id"],"reason":"new_input_relation_unresolved; prevent stale adoption"}));
        if let Some(q)=input.get("agent_question"){
            ctx.event("STIMULUS",&json!({"stimulus_id":q["id"],"goal_id":q["id"],"external_event":"AGENT_QUESTION"}));
            let p=json!({"stimulus_id":q["id"],"goal_id":q["id"],"text":q["text"],"question_id":q["id"],"task_id":q["task_id"]});ctx.present(&p);ctx.event("GOAL_COMPLETE",&p);
            let answer=&events[2];ctx.event("USER_WAIT_START",&json!({"question_id":q["id"]}));std::thread::sleep(Duration::from_millis(30));ctx.event("USER_WAIT_END",&json!({"question_id":q["id"]}));stimulus(ctx,answer);
        }
        let a=input["variant"].as_str()==Some("A");
        let mut first_result=None;
        if a{first_result=Some(handle.take().unwrap().join().map_err(|_|"worker panic")??);ctx.event("STEP_BOUNDARY",&json!({"request_id":first["id"]}));}
        let common_or_retained=if a {json!({"active_requests":history["active_requests"],"retained_pending_interpretation":first_result,"actually_delivered":[]})}else{history.clone()};
        ctx.event("INTERPRETATION_CONTEXT_SELECTED",&json!({"variant":input["variant"],"kind":if a {"retained conversation step evidence + draft"}else{"common adopted state; R1 draft remains request-owned"}}));
        let second_result=interpret(ctx,input,second,&common_or_retained)?;
        display(ctx,second,&second_result);outputs.push(json!({"goal_id":second["id"],"text":second_result["answer"]}));associations.push(second_result.clone());


        let corrected=second_result["relation"].as_str()==Some("CORRECTION");
        if corrected {
        if let Some(q)=input.get("agent_question"){let answer=&events[2];
            let v=interpret(ctx,input,answer,&json!({"active_requests":history["active_requests"],"actually_displayed_question":q,"delivered":outputs}))?;
            display(ctx,answer,&v);outputs.push(json!({"goal_id":answer["id"],"text":v["answer"]}));associations.push(v);
        }
        }
        let original=if let Some(v)=first_result{v}else{handle.take().unwrap().join().map_err(|_|"worker panic")??};
        let original_association=original.clone();
        let final_result=if corrected{
            ctx.event("STALE_RESULT_RETAINED_NOT_ADOPTED",&json!({"request_id":first["id"],"prior_result":original}));
            let revised=json!({"id":first["id"],"text":format!("Original request: {}. Apply this later correction and preserve remaining conditions: {}",first["text"],second["text"]),"revision":2});
            interpret(ctx,input,&revised,&json!({"active_requests":history["active_requests"],"correction":second_result}))?
        }else{original};
        display(ctx,first,&final_result);outputs.push(json!({"goal_id":first["id"],"text":final_result["answer"]}));associations.insert(0,original_association);
        if !corrected {
        if let Some(q)=input.get("agent_question"){let answer=&events[2];
            let v=interpret(ctx,input,answer,&json!({"active_requests":history["active_requests"],"actually_displayed_question":q,"delivered":outputs}))?;
            display(ctx,answer,&v);outputs.push(json!({"goal_id":answer["id"],"text":v["answer"]}));associations.push(v);
        }
        }


    }
    Ok(json!({"outputs":outputs,"associations":associations,"adoptions":[],"variant":input["variant"]}))
}
