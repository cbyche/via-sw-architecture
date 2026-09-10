# DP-00 Qualification Prototype

Phase 1 provides only the neutral Rust workspace and evaluation-support
infrastructure. Architecture alternatives and their decision logic are deliberately
absent.

## Frozen runtime direction

- Rust 1.94.0, matching the specification/reference `1.94` direction.
- Rust edition 2024.
- Tokio 1.53.1, resolved and frozen by `Cargo.lock`, for the common async substrate.
- `tokio-current-thread-v0` is the Phase 1 worker policy. It removes
  alternative-specific scheduler choices; any later multi-thread/parallel policy
  requires an explicit versioned qualification-profile change.
- Ordinary development uses Cargo's `dev`/`test` profiles.
- Timed qualification uses `--profile qualification`, an optimized performance
  profile distinct from ordinary development. Its flags must remain common across
  all future alternatives and be re-frozen before Pilot.

The timed event path records a monotonic timestamp and appends a typed event to an
in-memory buffer. JSON/JSONL conversion is exposed only through the separate
post-capture serialization API.

## Pilot runner

`dp00-pilot-runner` executes `scenario × alternative × repetition` sequentially
in one Rust process. It supports deterministic cyclic A/B/C/D ordering, a
configurable warm-up population, Profile Z (zero-delay baseline), provisional
Profile C (1 ms per model/agent/tool dependency invocation), and `capture` or
`minimal` instrumentation sinks. Profile C is a Pilot calibration parameter,
not a production latency claim or scoring threshold. Its three per-dependency
delays can be overridden independently with `--model-delay-micros`,
`--agent-delay-micros`, and `--tool-delay-micros`; the selected values are
persisted in provenance and apply equally to every alternative.

Example development dry-run:

```shell
cargo run -p bench-runner --bin dp00-pilot-runner -- \
  --corpus pilot-v0 \
  --scenarios P01 \
  --alternatives A,B,C,D \
  --latency-profile Z \
  --warmup 0 \
  --repetitions 1 \
  --instrumentation capture \
  --output-root /tmp/dp00-pilot-dry-run \
  --development
```

Measured raw evidence is persisted only after the timed interval as an immutable
create-new directory containing `provenance.json`, `canonical-events.jsonl`,
`model-calls.jsonl`, and diagnostic `fixture-events.jsonl` when present. An
existing run id is an error. `--official` refuses a dirty Git working tree;
development runs are always marked non-official in provenance.

## Base architecture smoke

The `alternative-a`, `alternative-b`, `alternative-c`, and `alternative-d` crates
contain independent Base Architecture implementations. `bench-smoke` supplies
responsibility-keyed replay, controlled fixtures, external outcome evidence, and
the S1-S5 assertion matrix without exposing its scenario/oracle context to an AUT.

Run the 20 paths separately with:

```shell
cargo test -p bench-smoke --test smoke_matrix
```
