mod allocation;mod context;mod model_wire;mod source_contract;mod dp02;
use serde_json::{Value,json};
use std::io::{self,BufRead};
use std::sync::Arc;
#[global_allocator]static ALLOC:allocation::Tracked=allocation::Tracked;
fn main(){
    let mut line=String::new();io::stdin().lock().read_line(&mut line).unwrap();let mut command:Value=serde_json::from_str(&line).unwrap();
    if command["wire_checks"]==true{
        let mut count=0;for call in command["calls"].as_array().unwrap(){
            let trace=json!({"role":"VIA","stage":call["stage"]});
            let mut owned=json!({"instruction":call["instruction"],"input":call["request_input"],"max_tokens":call["max_tokens"]});
            let source=owned["input"]["sources"].clone();let mut shared=owned.clone();shared["input"]["sources"]=Value::Null;owned["_trace"]=trace.clone();
            let expected=model_wire::encode(1,call["stage"].as_str().unwrap(),&owned);let mut bytes=vec![b'{'];
            model_wire::write_borrowed_fields(&mut bytes,1,call["stage"].as_str().unwrap(),&shared,&trace,&source).unwrap();bytes.push(b'}');
            let actual:Value=serde_json::from_slice(&bytes).unwrap();assert_eq!(actual,expected);assert_eq!(model_wire::borrowed_payload_len(&shared,&trace,&source),owned.to_string().len());count+=1;
        }println!("{}",json!({"wire_equivalence":"PASS","actual_recorded_payloads":count}));return;
    }
    let count=command["N"].as_u64().unwrap()as usize;assert!([1,2,4].contains(&count));let sharing=command["mode"]=="B1";
    let input=Arc::new(command["input"].take());drop(command);drop(line);assert_eq!(input["variant"],"B");
    let ctx=context::Context::new();ctx.event("EXPERIMENT_READY",&json!({"N":count,"mode":if sharing{"B1"}else{"B"},"input_serialized_bytes":input.to_string().len(),"source_serialized_bytes":input["sources"].to_string().len(),"source_identity":"one original immutable root input; baseline worker owns clone; B1 worker Arc references"}));
    let mut workers=Vec::new();for session in 1..=count{let i=input.clone();let mut c=ctx.for_session(session,sharing);workers.push(std::thread::spawn(move||{c.event("EPISODE_START",&json!({"id":i["id"],"dp":"02","variant":"B"}));match dp02::run(&i,&mut c){Ok(v)=>c.event("EPISODE_RESULT",&v),Err(e)=>c.event("EPISODE_ERROR",&json!({"error":e}))}}));}
    for w in workers{w.join().unwrap();}ctx.event("EXPERIMENT_COMPLETE",&json!({"N":count,"mode":if sharing{"B1"}else{"B"}}));
}
