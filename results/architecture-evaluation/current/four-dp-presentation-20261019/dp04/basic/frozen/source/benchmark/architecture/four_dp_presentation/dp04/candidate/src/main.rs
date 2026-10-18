mod context; mod model_wire; mod source_contract;
mod dp01; mod dp02; mod dp04; mod dp05;
use serde_json::{json,Value};
use std::io::{self,BufRead};
fn main(){
    let mut line=String::new(); io::stdin().lock().read_line(&mut line).unwrap();
    let input:Value=serde_json::from_str(&line).unwrap();
    let mut ctx=context::Context::new();
    ctx.event("EPISODE_START",&json!({"id":input["id"],"dp":input["dp"],"variant":input["variant"]}));
    let result=match input["dp"].as_str().unwrap_or("") {"01"=>dp01::run(&input,&mut ctx),"02"=>dp02::run(&input,&mut ctx),"04"=>dp04::run(&input,&mut ctx),"05"=>dp05::run(&input,&mut ctx),_=>Err("Unknown DP".into())};
    match result {Ok(value)=>ctx.event("EPISODE_RESULT",&value),Err(error)=>ctx.event("EPISODE_ERROR",&json!({"error":error}))}
}
