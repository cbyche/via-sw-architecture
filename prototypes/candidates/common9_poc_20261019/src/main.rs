mod agent_wire;
mod api;
mod context;
mod evidence;
mod grounding;
mod handoff;
mod lifecycle;
mod model_wire;
mod runtime;
mod sources;
use serde_json::Value;
use std::io::{self, BufRead};
fn main() {
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap();
    let input: Value = serde_json::from_str(&line).unwrap();
    let ctx = context::Context::new();
    match runtime::run(input, ctx.clone()) {
        Ok(value) => ctx.event("EPISODE_RESULT", &value),
        Err(error) => ctx.event("EPISODE_ERROR", &serde_json::json!({"error":error})),
    }
}
