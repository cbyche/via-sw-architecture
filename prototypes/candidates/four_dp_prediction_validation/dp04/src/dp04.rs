//! DP04 executable grounding alternatives. No fixture oracle is read here.
mod binding;
mod coordinate_wire;
mod gate;
mod geometry;
mod joint;
mod observations;
mod producers;
use crate::context::Context;
use serde_json::{json, Value};

pub fn run(input: &Value, ctx: &mut Context) -> Result<Value, String> {
    let variant = input["variant"].as_str().unwrap_or("A");
    if !["A", "B", "B4", "A4", "BQ"].contains(&variant) {
        return Err("Unsupported DP04 variant".into());
    }
    let mut prior = Value::Null;
    let mut all = Vec::new();
    let mut outputs = Vec::new();
    let mut trace = Vec::new();
    let mut old_proposal = Value::Null;
    let mut revision_gate = gate::RevisionGate::new();
    let goal = format!("{}:grounding",input["id"].as_str().unwrap_or("dp04"));
    let mut started = false;
    let mut prior_nodes = Vec::new();
    let initial_revision=input["revisions"].as_array().and_then(|x|x.first()).ok_or("revisions empty")?;
    let initial_stimulus=format!("{}:r{}",input["id"].as_str().unwrap_or("dp04"),initial_revision["revision"]);
    for revision in input["revisions"].as_array().ok_or("revisions missing")? {
        let rev = revision["revision"].as_u64().ok_or("revision missing")?;
        let stimulus = format!("{}:r{}", input["id"].as_str().unwrap_or("dp04"), rev);
        ctx.capture(&stimulus,&json!({"goal_id":goal,"revision":rev,"raw_text":revision["transcript"],"kind":"user_grounding_input"}));
        let mut scoped=ctx.scoped(&[goal.clone()],&prior_nodes);
        let ctx=&mut scoped;
        let mut observed = observations::read(revision, ctx)?;
        observed["parser_enabled"]=json!(variant!="A" && variant!="BQ");
        revision_gate.begin(rev, &observed["revision"]);
        ctx.event(
            "REVISION_REGISTERED",
            &json!({"revision":rev,"held_previous":!prior.is_null()}),
        );
        if input["late_result_replay"].as_bool() == Some(true) && !old_proposal.is_null() {
            let applied = revision_gate.accept(&old_proposal);
            ctx.event("REPLAYED_CONTROL_EVENT", &json!({"kind":"LATE_MODEL_RESULT_PUBLICATION_ATTEMPT","source":"actual earlier revision model proposal held and replayed; no model/network delay injected","proposal":old_proposal,"accepted":applied.is_ok(),"reason":applied.err()}));
        }
        let mut proposal = if variant == "A" {
            joint::ground(&observed, &prior, ctx)?
        } else if variant == "A4" {
            let screen = producers::parser(&observed, ctx)?;
            joint::ground_with_parser(&observed, &prior, &screen, ctx)?
        } else {
            let (screen, speech) = if variant == "B4" {
                let branch_parents=ctx.last_node().into_iter().collect::<Vec<_>>();
                let mut parallel = ctx.scoped(&[goal.clone()],&branch_parents);
                let obs = observed.clone();
                let previous = prior.clone();
                let handle =
                    std::thread::spawn(move || {let speech=producers::speech(&obs,&previous,&mut parallel);(speech,parallel.last_node())});
                let screen = producers::screen(&observed, &Value::Null, 1, ctx)?;
                let (speech,speech_node) = handle.join().map_err(|_| "speech producer panicked")?;
                let speech=speech?;
                let mut joined=ctx.last_node().into_iter().collect::<Vec<_>>();joined.extend(speech_node);
                ctx.join(&joined);
                (screen, speech)
            } else {
                let screen = producers::screen(&observed, &Value::Null, 1, ctx)?;
                let speech = producers::speech(&observed, &prior, ctx)?;
                (screen, speech)
            };
            binding::resolve(&observed, &prior, screen, speech, ctx)?
        };
        // Host stamps the actual source snapshot and job revision, never evaluator values.
        proposal["revision"] = json!(rev);
        proposal["source_snapshot"] = observed["source_snapshot"].clone();
        old_proposal = proposal.clone();
        match revision_gate.accept(&proposal) {
            Ok(()) => {
                let adopted = gate::normalize_adoptions(&proposal, revision, rev);
                for adoption in &adopted {
                    ctx.event("ADOPTION", adoption);
                }
                all.extend(adopted.clone());
                prior = json!({"revision":rev,"adoptions":adopted,"original_transcript":revision["transcript"]});
                let resolved=adopted.iter().all(|a|a["status"]=="ready" && a["targets"].as_array().is_some_and(|x|!x.is_empty()));
                let intent=proposal["intent"].as_str().filter(|x|!x.trim().is_empty());
                if resolved && intent.is_some() {
                    if !started {ctx.goal_start(&goal,&json!({"stimulus_id":initial_stimulus,"adopted_at_stimulus":stimulus,"revision":rev,"raw_text":initial_revision["transcript"],"origin":"user_grounding_goal","intent":intent}));started=true;}
                    else {ctx.event("GOAL_LINK",&json!({"goal_id":goal,"stimulus_id":stimulus,"revision":rev,"relation":"CORRECTION_SAME_GROUNDING_GOAL","intent":intent}));}
                    let target_ids:Vec<Value>=adopted.iter().flat_map(|a|a["targets"].as_array().cloned().unwrap_or_default()).collect();
                    let used_frames:Vec<Value>=adopted.iter().flat_map(|a|a["extents"].as_array().cloned().unwrap_or_default()).map(|e|e["frame_id"].clone()).collect();
                    let mut source_revisions=serde_json::Map::new();
                    for f in observed["revision"]["frames"].as_array().unwrap().iter().filter(|f|used_frames.contains(&f["frame_id"])) {source_revisions.insert(format!("{}@{}",f["source_id"].as_str().unwrap_or("source"),f["frame_id"].as_str().unwrap_or("frame")),f["source_revision"].clone());}
                    let target_labels:Vec<String>=adopted.iter().flat_map(|a|a["extents"].as_array().cloned().unwrap_or_default()).map(|e|e["label"].as_str().filter(|x|!x.is_empty()).unwrap_or(e["target_id"].as_str().unwrap_or("?")).to_string()).collect();
                    let text=format!("지칭 대상 {}을(를) 채택했습니다. 수행 의도: {}",target_labels.join(", "),intent.unwrap());
                    let adoption=json!({"adoption_id":format!("{}:r{}",goal,rev),"resolved":true,"text":text,"targets":target_ids,"scope":adopted,"handling":intent,"source_revisions":source_revisions,"raw_utterance":revision["transcript"],"revision":rev});
                    ctx.progress(&goal,&stimulus,&adoption)?;
                    let output=json!({"goal_id":goal,"stimulus_id":stimulus,"text":text,"targets":target_ids,"revision":rev,"content_contract_checked":true,"checks":["current_revision","frame_snapshot","nonempty_target_extents","explicit_count","claimed_event_geometry"],"scope":adopted,"source_revisions":source_revisions,"observation_class":"DISPLAY_PROXY","same_endpoint_reason":"Grounding/adoption/display is the complete VIA scope; no artificial business work added"});
                    ctx.complete(&goal,&output)?;
                    outputs.push(output);
                } else {ctx.fail(&goal,&json!({"stimulus_id":stimulus,"revision":rev,"reason":"UNRESOLVED_GROUNDING_OR_INTENT","proposal":proposal}));}

            }
            Err(reason) => {
                ctx.event("PROPOSAL_REJECTED",&json!({"revision":rev,"reason":reason,"proposal":proposal}));
                ctx.fail(&goal,&json!({"stimulus_id":stimulus,"revision":rev,"reason":reason,"owner_recovery_exhausted":true}));
            },
        }
        ctx.logical_buffer("dp04_observations",0,"RELEASE");
        ctx.logical_buffer("dp04_prior_binding",prior.to_string().len(),"RETAIN");
        prior_nodes=ctx.last_node().into_iter().collect();
        trace.push(proposal);
        ctx.logical_buffer("dp04_retained_proposal_trace",serde_json::to_vec(&trace).map(|x|x.len()).unwrap_or(0),"ALLOCATE_OR_REPLACE");
    }
    Ok(
        json!({"adoptions":all,"outputs":outputs,"proposals":trace,"screen_parser_variant":if variant=="A" {"none"} else if variant=="BQ" {"Qwen-only separated producer diagnostic; no dedicated parser"} else {"Task-local real OmniParser detector+EasyOCR+Florence CPU diagnostic; original cloud deployment not measured"},"tactic":if variant=="B4" {"parallel independent producers"} else if variant=="A4" {"same screen parser auxiliary input"} else {"none"}}),
    )
}
