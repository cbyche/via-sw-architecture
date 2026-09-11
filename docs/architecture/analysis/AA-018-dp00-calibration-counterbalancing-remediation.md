# AA-018 — DP-00 Calibration Counterbalancing Remediation

## Status and scope

Session 6.4 calibration `dp00-calibration-20260911T035717Z-49b8bc0` used frozen
source `49b8bc0f624c1e2503bc0dcc4ecbc0b772b2a902`. Its immutable evidence shows a
monotonic CAPTURE FTOL cycle-p50 increase (`6,042 → 7,459 → 7,500 → 7,709 ns`)
and an early-to-late CAPTURE p50 shift of `+584 ns`. The paired delta median
changed by `876 ns` and reversed sign according to whether CAPTURE ran first or
second. Because the prior execution placed each mode in a time block, the pooled
mode-order result is confounded with cycle/time and cannot identify a stable
instrumentation effect.

This record approves protocol remediation only. It does not reinterpret Session
6.4 evidence, run a new calibration, rank A/B/C/D, change the 88 observed QA-02
nonconformances, or alter the approved requirements baseline.

## Decision

Before measured execution, the runner performs full pre-warm cycles `W0` and
`W1`. Each exercises all 96 scenario × alternative × instrumentation paths.
Their explicit path inventories and completion state are written to the
calibration manifest, but their timing is discarded and cannot enter measured
accounting, QA derivation, drift, or scoring. The existing invocation-level
warm-up count of one remains in place before each measured cycle; the two
policies are additive and do not conflict.

Measured execution uses exactly 16 cycles with adaptive stopping disabled. Each
cycle contains 48 adjacent CAPTURE/MINIMAL pairs and 96 executions. No other
measured pair may occur between the two members of one pair. This fixes the full
population at 768 pairs and 1,536 executions; P01–P03 supply 192 paired QA-01
observations and 192 observations per mode.

Alternative position retains the four-cycle rotation `ABCD`, `BCDA`, `CDAB`,
`DABC`, repeated four times. Mode order is assigned independently using stable
zero-based scenario and alternative ordinals:

```text
capture_first = parity(scenario_ordinal + alternative_ordinal + measured_cycle) == even
```

Thus every cycle has 24 CAPTURE-first and 24 MINIMAL-first pairs, every
alternative has 6/6, every scenario has 2/2, and the same semantic pair inverts
order in the next cycle. Across 16 cycles, every pair is CAPTURE-first eight
times and MINIMAL-first eight times, while every alternative occupies each
position four times.

## Explicit contracts

The protocol is `dp00-calibration-protocol-v1`; its manifest is
`dp00-calibration-manifest-v1`. Measured runs use
`dp00-pilot-provenance-v4`, which retains v3 paired identity and adds the
protocol version plus an explicit contiguous measured-execution ordinal.
Analysis `dp00-analysis-v6` validates the manifest, full-prewarm coverage,
fixed accounting, adjacency, balances, inversion, rotation, and manifest/run
identity. It rejects a malformed schedule rather than reordering or repairing
evidence.

`canonical-event-v3`, Measurement Spine semantics, QA-01 FTOL, QA-02
qualification, P12 compound routing, QA-04 final-required boundary,
architecture implementations, corpus meanings, Profile Z delay, and correctness
qualification remain unchanged.

## Consequences

The next calibration may begin only after this remediation commit is separately
approved as a frozen source. No numeric drift or instrumentation-overhead PASS
threshold is introduced here. If such a threshold is required for freeze
readiness, it remains a prospective Architecture/Measurement Decision that must
be approved before observing the next result.
