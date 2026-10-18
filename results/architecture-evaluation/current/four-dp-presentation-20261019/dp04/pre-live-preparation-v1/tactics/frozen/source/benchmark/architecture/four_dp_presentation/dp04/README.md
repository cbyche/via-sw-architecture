# DP04 presentation follow-up

The original v2 sources/results remain unchanged. `candidate/` copies its BASIC
frozen crate; only DP04 adds the explicitly named `A2A5` path. Existing A, B and B4
instructions, output caps and common gates remain unchanged. This is not a native
object API or new visual parser.

Structure for presentation:

- A: raw screen + speech/events → joint owner → common gate → callback/completion.
- B: raw screen → screen producer; speech/events → speech producer → code/model
  binder → common gate → callback/completion.
- A2+A5: same first joint call → only on rejection, mechanically rendered raw-cue
  crop or whole-frame coordinate view → same joint owner's affected-expression
  patches → merge unaffected entries → revalidate entire proposal. The owner can
  instead request full rejudgment within the same one recovery.
- B4: same B, with its initial independent producers overlapping in the same two
  slots; semantic ownership does not change.

`coordinate_views.py` reads only the PNG and raw event coordinates. Its grid is
100 original pixels; raw pointer/path bounding unions receive128 pixels of
padding. No cue means full frame. No target bbox or expected ID chooses a crop.
The original and existing raw overlay remain supplied. Rendering happens inside
the actual post-input recovery interval and contributes to its latency and RSS.
Explicit local-view boxes are mapped with recorded affine metadata, then the
original gate runs; malformed/overflow boxes are not clamped into success.

`control.py` reuses the unchanged v2 runner/metric code by pinned source hashes.
Every trial has fresh Python/Rust processes. It enforces the approved local
$0.55 cap as well as the locked parent $1.50 ledger. No credential bytes or actual
workspace URL are saved in the preflight result.

Execution order and eligibility are in the result `plan.json`, frozen before any
provider call. Forty-eight planned episodes retain all four original cases and
three repetitions: BASIC A/B first, then A2+A5/B4. Each configuration retains
QA01 15 inputs, QA03 12 goals and QA02 24 exact sets. Conditional means and missing
endpoints remain separate. Failed finite common endpoints cannot establish an
overall QA01 speed winner. This is the existing functionality prerequisite,
not a new accuracy score or a retrospective penalty.

Offline checks use deterministic service doubles (including authored SVG only
inside that test service); they are not model-performance evidence. Initial
offline setup/admission failures are preserved. Live code never calls that
service or supplies its geometry to the provider. The combined tactic's outcome
does not isolate A2 from A5, prove pure visual recognition superiority, or rerun
QA04 modification exercises.
