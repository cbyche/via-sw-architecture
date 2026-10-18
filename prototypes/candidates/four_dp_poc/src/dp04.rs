//! DP04 executable grounding alternatives. No fixture oracle is read here.
mod binding;
mod gate;
mod joint;
mod observations;
mod producers;
use crate::context::Context;
use serde_json::{json, Value};

pub fn run(input: &Value, ctx: &mut Context) -> Result<Value, String> {
    let variant = input["variant"].as_str().unwrap_or("A");
    if !["A", "B", "B4", "A4"].contains(&variant) {
        return Err("Unsupported DP04 variant".into());
    }
    let mut prior = Value::Null;
    let mut all = Vec::new();
    let mut outputs = Vec::new();
    let mut trace = Vec::new();
    let mut old_proposal = Value::Null;
    let mut revision_gate = gate::RevisionGate::new();
    for revision in input["revisions"].as_array().ok_or("revisions missing")? {
        let rev = revision["revision"].as_u64().ok_or("revision missing")?;
        let stimulus = format!("{}:r{}", input["id"].as_str().unwrap_or("dp04"), rev);
        let goal = stimulus.clone();
        ctx.event("STIMULUS", &json!({"stimulus_id":stimulus,"goal_id":goal,"revision":rev,"transcript":revision["transcript"]}));
        let observed = observations::read(revision, ctx)?;
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
            let screen = producers::screen(&observed, &Value::Null, 1, ctx)?;
            joint::ground_with_parser(&observed, &prior, &screen, ctx)?
        } else {
            let (screen, speech) = if variant == "B4" {
                let mut parallel = ctx.clone();
                let obs = observed.clone();
                let previous = prior.clone();
                let handle =
                    std::thread::spawn(move || producers::speech(&obs, &previous, &mut parallel));
                let screen = producers::screen(&observed, &Value::Null, 1, ctx)?;
                let speech = handle.join().map_err(|_| "speech producer panicked")??;
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
                let text = format!(
                    "화면 지칭 r{}: {}",
                    rev,
                    serde_json::to_string(&prior["adoptions"]).unwrap()
                );
                let output = json!({"goal_id":goal,"stimulus_id":stimulus,"text":text,"revision":rev,"observation_class":"DISPLAY_PROXY"});
                ctx.present(&output);
                ctx.event("GOAL_COMPLETE", &output);
                outputs.push(output);
            }
            Err(reason) => ctx.event(
                "PROPOSAL_REJECTED",
                &json!({"revision":rev,"reason":reason,"proposal":proposal}),
            ),
        }
        trace.push(proposal);
    }
    Ok(
        json!({"adoptions":all,"outputs":outputs,"proposals":trace,"screen_parser_variant":"Qwen vision screen producer; no external OmniParser dependency deployed","tactic":if variant=="B4" {"parallel independent producers"} else if variant=="A4" {"same screen parser auxiliary input"} else {"none"}}),
    )
}
