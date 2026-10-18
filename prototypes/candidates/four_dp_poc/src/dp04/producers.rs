use crate::context::Context;
use serde_json::{json, Value};
const SCREEN:&str="You are the Screen Target Producer and vision parser, not the final request binder. Parse actual PNG pixels and observation events WITHOUT inventing intended answers. Return source-backed candidates for visible named objects (a whole card/chart/table or individual plotted sample), including visible target_id, frame_id, kind, label, bbox [x1,y1,x2,y2]. Preserve competing/unknown areas; do not claim complete coverage without examining the source. For each pointer event, optionally describe a geometric cue_relation {cue_id,target_ids,relation:'point_inside'|'within_region',complete:bool,unique:bool,navigation_alternative:bool}. A pointer move alone does not prove designation; keep navigation uncertainty where present. An expansion request may ask you to subdivide/produce a previously missing extent from the SAME original pixels. Only you may produce new candidate extents. No final per-expression adoptions. Output JSON {candidates:[{target_id,frame_id,kind,label,bbox}],cue_relations:[...],coverage:{complete:bool,unknown:[strings],competing:[strings]},source_refs:[frame_ids],revision:number}. Always include all relevant visible sample IDs, including excluded alternatives. Multiple preserved pages are distinct source revisions. max 30 candidates.";
const SPEECH:&str="You are the Speech Reference Producer. Interpret transcript, expression spans, event time metadata and prior accepted binding. You have no screen pixels and must not guess visible object IDs. Return JSON {expressions:[{expression_id,kind,count:number|null,relation:'designated_by_cue'|'within_region'|'explicit_ids'|'relative_position'|'correction'|'retain_prior'|'other',cue_ids:[event IDs],explicit_target_ids:[only IDs actually stated in speech],unresolved:[strings],meaning:string}],coverage:{complete:bool,unknown:[strings]},revision:number}. Connect a referring expression to a geometric cue only if the wording and event interval support it; distinguish navigation from pointing using the speech. For corrections record what is retained and replaced in meaning. Return every supplied expression_id exactly once. Do not output final targets or copy guesses from prior into new references.";
pub fn screen(
    obs: &Value,
    expansion: &Value,
    generation: u64,
    ctx: &mut Context,
) -> Result<Value, String> {
    let r = &obs["revision"];
    let input = json!({"revision":r["revision"],"frames":r["frames"],"events":r["events"],"native_selection":r["native_selection"],"capture_gaps":r["capture_gaps"],"expansion_request":expansion,"producer_generation":generation});
    let mut result = ctx.call(
        if generation == 1 {
            "dp04.screen_producer"
        } else {
            "dp04.screen_source_expansion"
        },
        &json!({"instruction":SCREEN,"input":input,"images":obs["images"],"max_tokens":3500}),
    )?;
    result["revision"] = r["revision"].clone();
    result["producer_generation"] = json!(generation);
    result["source_snapshot"] = obs["source_snapshot"].clone();
    ctx.event(
        "SCREEN_CANDIDATES_PUBLISHED",
        &json!({"revision":r["revision"],"generation":generation,"state":result}),
    );
    Ok(result)
}
pub fn speech(obs: &Value, prior: &Value, ctx: &mut Context) -> Result<Value, String> {
    let r = &obs["revision"];
    let mut result=ctx.call("dp04.speech_producer",&json!({"instruction":SPEECH,"input":{"revision":r["revision"],"transcript":r["transcript"],"expressions":r["expressions"],"events":r["events"],"prior":prior},"max_tokens":1800}))?;
    result["revision"] = r["revision"].clone();
    ctx.event(
        "SPEECH_CANDIDATES_PUBLISHED",
        &json!({"revision":r["revision"],"state":result}),
    );
    Ok(result)
}
