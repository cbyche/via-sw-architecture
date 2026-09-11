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

Session 7.1 adds the prospectively frozen synthetic sensitivity matrix
`dp00-realistic-sensitivity-v1`: R1 balanced (50/50/50 ms), R2 model-dominant
(100/20/20 ms), R3 Agent/delegation-dominant (20/100/20 ms), and R4
tool-dominant (20/20/100 ms). Values are model/Agent/tool constant charges per
semantic invocation. They are sensitivity-analysis inputs, not empirical
production latency. The authoritative contract is
`benchmark/contracts/dp00-realistic-execution-profiles-v1.json`. Profile Z and
provisional Profile C are unchanged.

## Counterbalanced calibration runner

`dp00-calibration-runner` implements `dp00-calibration-protocol-v1` for the next
approved frozen-source calibration. It retains one invocation-level warm-up per
measured cycle and adds two full pre-warm cycles, W0 and W1, that cover all
P01–P12 × A–D × CAPTURE/MINIMAL paths. Pre-warm timing is discarded; explicit
coverage and completion are retained only in `calibration-manifest.json`.

The measured population is fixed at 16 cycles, 768 adjacent mode pairs, and
1,536 executions; adaptive stopping is disabled. Within each cycle mode order is
selected by the parity of zero-based scenario ordinal + alternative ordinal +
cycle. The same pair therefore inverts in the next cycle. Alternative position
independently rotates ABCD/BCDA/CDAB/DABC four times. The runner persists only
measured raw evidence with `dp00-pilot-provenance-v4` and an explicit contiguous
execution ordinal. This runner must be invoked only after its exact source SHA is
approved for calibration. It defaults to Profile Z and accepts one explicit
frozen qualification profile with `--latency-profile Z|R1|R2|R3|R4`; Profile C
is deliberately rejected by this qualification path because it remains a
provisional calibration parameter.

## Base architecture smoke

The `alternative-a`, `alternative-b`, `alternative-c`, and `alternative-d` crates
contain independent Base Architecture implementations. `bench-smoke` supplies
responsibility-keyed replay, controlled fixtures, external outcome evidence, and
the S1-S5 assertion matrix without exposing its scenario/oracle context to an AUT.

Run the 20 paths separately with:

```shell
cargo test -p bench-smoke --test smoke_matrix
```
