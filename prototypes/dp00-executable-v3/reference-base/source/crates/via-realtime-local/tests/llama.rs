//! The reasoning engine — `--features llama` only.
//!
//! Without the feature this is an empty test binary rather than a compile
//! error. No GGUF is checked in, so what is asserted is the guard that does not
//! need one: a missing model is refused **by name, before the library is
//! asked**, because `LlamaModel::load_from_file` on an absent path is a
//! `debug_assert` in a debug build and an opaque failure in a release one.
//!
//! The prompt assembly is pure and *is* fully asserted — in the module's own
//! unit tests, because it needs no model at all. The decode loop is not
//! asserted anywhere in this repository, and
//! `docs/deviations/phase-8-via-realtime-local.md` records that.

#![cfg(feature = "llama")]

use via_realtime_local::llama::{LlamaOptions, LlamaResponder};
use via_realtime_local::{LocalError, Stage, WeightsSet};

#[test]
fn a_missing_gguf_is_refused_by_name_before_the_library_is_asked() {
    let weights = WeightsSet::resolve("/nonexistent-model-root");
    assert_eq!(
        LlamaResponder::open(&weights, LlamaOptions::default()).err(),
        Some(LocalError::model_file_missing(
            Stage::Reasoning,
            &weights.reasoning
        ))
    );
}

#[test]
fn the_generation_bound_fits_inside_the_context() {
    // The bound that makes "the pipeline eventually stops speaking" a property:
    // a model that never emits its end-of-generation token stops at
    // `max_tokens` rather than decoding until the KV cache fills.
    let options = LlamaOptions::default();
    assert!(options.max_tokens > 0);
    assert!(options.max_tokens < options.context_tokens);
}
