# DP05 receiver completion guard checks

`dp05-receiver-v2-with-guard-checks.rs` contains the exact production source followed by six test-only checks. Its production prefix is byte-identical to the persisted v2 candidate; see `guard-check-manifest.json`. The probe compiled in an isolated temporary copy of the Rust crate. `receiver-guard-tests.log` retains the actual output. These are deterministic access/provenance checks, not model accuracy or timing measurements.

The checks accept completion backed by a supplied current capsule and reject manifest-only content, larger unseen ranges, stale revisions, revoked sources, and an unbacked used source ID. No API call or canned model response was used.

To reproduce, copy the candidate crate into a temporary directory, replace its `src/dp05.rs` with this probe, and run `cargo test --offline --manifest-path <temporary crate>/Cargo.toml completion_guard_checks`.
