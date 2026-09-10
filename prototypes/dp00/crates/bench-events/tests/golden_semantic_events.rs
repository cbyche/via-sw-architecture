use std::fs;
use std::path::PathBuf;

use bench_events::CanonicalEvent;
use serde::Deserialize;

#[derive(Deserialize)]
struct Expectations {
    schema_version: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    file: String,
    valid: bool,
}

#[test]
fn rust_enforces_shared_semantic_event_golden_expectations() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .join("benchmark/fixtures/pilot-v0/golden/raw-events");
    let expectations: Expectations = serde_json::from_slice(
        &fs::read(root.join("validation-expectations.json")).expect("expectations fixture"),
    )
    .expect("expectations parse");
    assert_eq!(
        expectations.schema_version,
        "canonical-event-golden-expectations-v1"
    );
    for case in expectations.cases {
        let result = serde_json::from_slice::<CanonicalEvent>(
            &fs::read(root.join(&case.file)).expect("raw event fixture"),
        );
        assert_eq!(result.is_ok(), case.valid, "golden case {}", case.file);
    }
}
