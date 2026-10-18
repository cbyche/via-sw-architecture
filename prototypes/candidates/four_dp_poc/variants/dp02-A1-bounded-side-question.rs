use serde_json::{json,Value};
use crate::context::Context;
use std::time::Duration;

fn interpret(ctx:&mut Context, input:&Value,event:&Value,history:&Value)->Result<Value,String>{
    let mut value=ctx.call("interpret_execution",&json!({"instruction":"You interpret VIA user inputs from source evidence and actual conversation state. The current event may also appear in the capture registry; that duplicate is not an earlier interpreted event. Infer the relation from genuinely earlier inputs and actual delivery state. Return JSON {relation:'NEW'|'FOLLOWUP'|'CORRECTION'|'ANSWER',target_request_id:string|null,question_id:string|null,task_id:string|null,answer:string}. Preserve all unresolved goals. A correction changes only the specified scope. An answer must be associated with the actually displayed Agent question, not whichever other task is active. Give a detailed concise factual answer. Do not invent source facts.","input":{"event":event,"sources":input["sources"],"conversation":history},"max_tokens":1600}))?;
    value["event_id"]=event["id"].clone();value["revision"]=event["revision"].clone();Ok(value)
}
fn side_eligible(value:&Value)->bool{
    value["eligible"].as_bool()==Some(true)
        && value["relation"].as_str()==Some("NEW")
        && value["requires_tools"].as_bool()==Some(false)
        && value["requires_source_reads"].as_bool()==Some(false)
        && value["depends_on_pending_result"].as_bool()==Some(false)
        && matches!(value["pending_request_effect"].as_str(),Some("NONE")|Some("CONTINUE_UNCHANGED"))
        && value["target_request_id"].is_null() && value["question_id"].is_null()
        && value["answer"].as_str().map(|s|!s.trim().is_empty() && s.chars().count()<=1200).unwrap_or(false)
}
fn bounded_side_question(ctx:&mut Context,event:&Value,history:&Value,displayed_question:&Value)->Result<Option<Value>,String>{
    let mut decision=match ctx.call("dp02_A1_bounded_side_question",&json!({"instruction":"You may answer only a short independent tool-free side question while the SAME conversation's existing semantic step continues. Decide suitability from the actual utterance and actually delivered conversation, not from an event ID. The existing pending request keeps its original lifetime/checkpoint. A statement to continue that work unchanged is compatible with a separate independent question; any correction/cancellation/answer to a pending Agent question, dependency on the pending answer, need for source reads/tools, or long/new work is unsuitable and must return eligible=false with answer=null. For a suitable question use own stable knowledge and already delivered content only, with a concise answer (at most 1200 characters). No source files or tools are available here. Return JSON {eligible:boolean,relation:'NEW'|'FOLLOWUP'|'CORRECTION'|'ANSWER',target_request_id:string|null,question_id:string|null,task_id:string|null,requires_tools:boolean,requires_source_reads:boolean,depends_on_pending_result:boolean,pending_request_effect:'NONE'|'CONTINUE_UNCHANGED'|'MODIFY'|'CANCEL'|'UNRESOLVED',side_question:string|null,answer:string|null,reason:string}. Do not alter, complete, cancel, or invent the original work. If unsuitable, do not answer it here; the original conversation will process it at its normal step boundary.","input":{"current_utterance":event["text"],"active_conversation":history,"actually_displayed_agent_question":displayed_question},"max_tokens":800})){Ok(value)=>value,Err(error)=>{ctx.event("A1_SIDE_CALL_FAILED",&json!({"input_event":event["id"],"error":error,"original_request_preserved":true,"fallback":"MAIN_STEP_BOUNDARY"}));return Ok(None);}};
    let eligible=side_eligible(&decision);
    ctx.event("A1_SIDE_ELIGIBILITY",&json!({"input_event":event["id"],"model_decision":decision,"host_contract_eligible":eligible,"original_checkpoint_mutated":false}));
    if eligible {decision["event_id"]=event["id"].clone();decision["revision"]=event["revision"].clone();Ok(Some(decision))}else{ctx.event("A1_RETURN_TO_MAIN_BOUNDARY",&json!({"input_event":event["id"],"reason":"unsuitable_or_invalid_bounded_side_contract"}));Ok(None)}
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
        // A1 is one bounded side call on the existing conversation; no request-owned worker/state is introduced.
        let side_result=if a && input["tactic"].as_str()==Some("A1") {
            bounded_side_question(ctx,second,&history,&input["agent_question"])?
        }else{None};
        let side_accepted=side_result.is_some();
        if a && !side_accepted{first_result=Some(handle.take().unwrap().join().map_err(|_|"worker panic")??);ctx.event("STEP_BOUNDARY",&json!({"request_id":first["id"]}));}
        let common_or_retained=if a {json!({"active_requests":history["active_requests"],"retained_pending_interpretation":first_result,"actually_delivered":[]})}else{history.clone()};
        ctx.event("INTERPRETATION_CONTEXT_SELECTED",&json!({"variant":input["variant"],"kind":if a {"retained conversation step evidence + draft"}else{"common adopted state; R1 draft remains request-owned"}}));
        let second_result=if let Some(value)=side_result{value}else{interpret(ctx,input,second,&common_or_retained)?};
        display(ctx,second,&second_result);outputs.push(json!({"goal_id":second["id"],"text":second_result["answer"]}));associations.push(second_result.clone());
        if side_accepted{ctx.event("A1_ACTUAL_CONVERSATION_DELIVERY",&json!({"input_event":second["id"],"revision":second["revision"],"text":second_result["answer"],"original_request_id":first["id"],"original_request_continues":true,"retained_as_actual_delivery":true}));}


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
