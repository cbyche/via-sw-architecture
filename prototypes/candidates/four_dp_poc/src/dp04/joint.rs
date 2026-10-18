use crate::context::Context;
use serde_json::{json, Value};
const INSTRUCTION: &str = "You are the joint screen/speech grounding interpreter. Read the actual PNG pixels together with authored transcript spans, pointer/selection events, frame times and prior accepted bindings. Produce the exact target set for EACH expression_id of this input revision. Visible item/sample IDs identify source extents. Preserve the first expression if a later correction explicitly retains it; revise only what the complete new instruction changes, while rechecking the entire binding set. Pointer timing alone is not intent. A later page change does not erase a previously pointed frame. A free loop can denote a set of plotted samples. Do not answer the requested business task. Return JSON {adoptions:[{expression_id:string,targets:[visible_id strings],status:'ready'|'ambiguous'|'need_evidence',evidence:string}],need_evidence:false|{scope:string},reasoning_summary:string}. Do not invent a missing source or identifier. You may request one bounded reread of the preserved original source if needed; do not turn an unresolved target into a confident adoption.";
pub fn ground(obs: &Value, prior: &Value, ctx: &mut Context) -> Result<Value, String> {
    ground_impl(obs, prior, &Value::Null, ctx)
}
pub fn ground_with_parser(
    obs: &Value,
    prior: &Value,
    screen: &Value,
    ctx: &mut Context,
) -> Result<Value, String> {
    ground_impl(obs, prior, screen, ctx)
}
fn ground_impl(
    obs: &Value,
    prior: &Value,
    screen: &Value,
    ctx: &mut Context,
) -> Result<Value, String> {
    let input =
        json!({"observations":obs["revision"],"prior":prior,"auxiliary_screen_parser":screen});
    let mut result = ctx.call(
        "dp04.joint_grounding",
        &json!({"instruction":INSTRUCTION,"input":input,"images":obs["images"],"max_tokens":1700}),
    )?;
    if result["need_evidence"].is_object() {
        ctx.event("BOUNDED_SOURCE_REREAD",&json!({"scope":result["need_evidence"],"reason":"joint interpreter requested preserved source","revision":obs["revision"]["revision"]}));
        result=ctx.call("dp04.joint_source_reread",&json!({"instruction":INSTRUCTION,"input":{"original":input,"previous_attempt":result,"reread_budget_remaining":0},"images":obs["images"],"max_tokens":1700}))?;
    }
    Ok(result)
}
