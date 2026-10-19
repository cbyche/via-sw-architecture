//! Screen grounding only. Shared runtime owns capture, adoption, display and goal completion.
#[path = "grounding/binding.rs"]
mod binding;
#[path = "grounding/joint.rs"]
mod joint;
#[path = "grounding/producers.rs"]
mod producers;
#[path = "grounding/wire.rs"]
mod wire;
use crate::{api::GroundingInput, context::Context};
use serde_json::{json, Value};

pub fn ground(ctx: &mut Context, variant: &str, input: &GroundingInput) -> Result<Value, String> {
    if input.frames.is_empty() {
        return Ok(
            json!({"bindings":[],"intent":null,"unknowns":[],"screen_path":"NO_SCREEN_OBSERVATION"}),
        );
    }
    match variant {
        "A" => joint::ground(ctx, input),
        "B" => {
            let screen = producers::screen(ctx, input, &Value::Null)?;
            let speech = producers::speech(ctx, input)?;
            binding::resolve(ctx, input, screen, speech)
        }
        _ => Err(format!("Unsupported grounding alternative {}", variant)),
    }
}
