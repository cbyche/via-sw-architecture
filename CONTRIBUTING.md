# Contributing

## Choose the correct generation

- Current Architecture documents: `docs/architecture/`
- Accepted decisions: `docs/adr/`
- Current measurement implementation: `benchmark/architecture/`
- Current candidate prototypes: `prototypes/gate2/`
- Current results: `results/gate2/current/`
- Historical material: paths named `archive/`

Do not update an archived artifact to express a current decision. Create or edit the corresponding current artifact and link historical evidence explicitly when needed.

## Measurement changes

Freeze definitions, fixtures, evidence labels, repetition and aggregation rules before viewing comparative results. Compare each Decision Point's A/B alternatives directly while holding other DP conditions fixed. Never present calculated, mock, reference, or archived values as actual model or product measurements.

## Checks

```bash
.venv/bin/python scripts/gate2/check_active_w_metric_terms.py
.venv/bin/python scripts/gate2/check_active_markdown_links.py
cd prototypes/gate2
cargo +1.98.1 fmt --all -- --check
cargo +1.98.1 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.98.1 test --locked --workspace --all-targets
```

Follow the Git and secret-handling rules in [`AGENTS.md`](AGENTS.md).
