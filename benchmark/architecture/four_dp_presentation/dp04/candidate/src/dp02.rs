//! Interpretation execution facade. Operational ownership is in state, interpretation,
//! response and execution; both alternatives share semantic validation and completion rules.
mod state;mod interpretation;mod response;mod execution;
use serde_json::Value;
use crate::context::Context;
pub fn run(input:&Value,ctx:&mut Context)->Result<Value,String>{execution::run(input,ctx)}
