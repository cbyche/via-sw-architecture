# Review v2 common execution contract (draft for pre-call review)

Parent conditionally approved only a 12-episode CALIBRATION after offline readiness (2026-10-18). Formal BASIC calls remain blocked until their separate freeze/approval. Each phase has immutable snapshots and an exact authorized job list.
Old campaign evidence is immutable. New campaign ceiling: USD 4 globally, including all smoke,
preparation, failed, discarded, retry, receiver and tactic calls.

## Stable Rust API

`run(&Value, &mut Context) -> Result<Value,String>`; existing `call/event/present/source` remain.

- `goal_start(&self, goal_id: &str, data: &Value)` emits GOAL_START using the original capture timestamp. Data includes
  stimulus_id, raw text, goal revision and goal origin. Capture first; call after actual semantic goal adoption. If not already captured it captures immediately as a compatibility fallback.
- `capture(&self, stimulus_id: &str, data: &Value)` emits INPUT_CAPTURED and STIMULUS at actual input ingress.
  Classification is a model output, never inferred from fixture IDs. GOAL_LINK records the
  subsequent input -> goal/revision/intent relation. Corrections do not create a second QA03 goal.
- `progress(&self, goal_id: &str, stimulus_id: &str, adoption: &Value) -> Result<(),String>` emits
  ADOPTION_COMMITTED then DISPLAY_PROXY. Required: adoption_id, nonempty text, targets (array),
  scope, handling, source_revisions (object), resolved=true. Each DP validates its actual source,
  rights, version, shape and resolved semantic state before calling. `raw_utterance` enables
  common exact-echo rejection; unresolved generic acknowledgements are invalid.
- `complete(&self, goal_id: &str, result: &Value) -> Result<(),String>` emits final DISPLAY_PROXY
  then GOAL_COMPLETE. Required: nonempty text, content_contract_checked=true; include stimulus_id,
  revision, supporting spans/targets and the performed checks. A host contract check is not an
  oracle claim of semantic correctness. Wrong nonempty content retains its real display time.
- `fail(&self, goal_id: &str, reason: &Value)` emits GOAL_FAILED. Failure does not become a
  successful endpoint and does not receive an invented time penalty.
- `scoped(&self, goal_ids: &[String], parents: &[String]) -> Context` clones the broker with a
  local causal scope. `call` creates OPERATION_START/END and supplies `_trace` metadata to the
  bridge (metadata is excluded from provider prompt). `last_node()` returns the operation node.
- `logical_buffer(&self,name,bytes,action)` records logical serialized bytes, not object heap/RSS.

## Events and causal timing

Every event has Rust monotonic `elapsed_ns`, sequence event_id and local `goal_ids`/`parent_ids`.
Python records receipt `observed_ns`; QA callbacks use Rust event time mapped onto the per-trial
origin. Callback proxy was accepted by the user; browser frame rendering is outside this endpoint.

QA01 population: actual user inputs and external notices requiring a substantive UI response;
first concrete adopted handling, substantive clarification or result. Notice display may be QA01
but is never automatically a QA03 goal. Null/empty/unresolved echo => NOT_OBSERVED.

QA03 population: initial/new user goals and explicitly pre-existing Task goals, not notices or
correction messages as separate goals. A correction updates the same goal revision. An answer to
an Agent question resolves the linked Task goal; do not silently borrow the active unrelated goal.
Include model queue, step wait, source reads, actual transport and host processing. Only actual
external execution/user wait on the goal's dependency path may be zeroed. Parallel work must not
be subtracted twice or removed from unrelated goals. Graph gaps/unknown dependencies are reported,
not replaced with a falsely exact QA03. Report elapsed and attributable paths separately.

## Completion and recovery

Every DP uses at most one owner-directed recovery per failed semantic stage (initial + one repair).
Keep all failed proposal, validation reason, callback, cost and final failure evidence. Recoveries
receive actual supplied evidence/context, never oracle facts. Output capacity is frozen before
measured runs. Separate response coverage, completion coverage, host-contract failures and oracle
accuracy. Missing/failed runs remain in coverage denominators even when elapsed means are conditional.

## Memory

Fresh Python runner and Rust child per trial. Capture role, PID, PPID, sample start/end and actual
observed PIDs, RSS at the same sample, nominal cadence and incomplete-coverage counts. Peak is the
maximum simultaneous sum, with per-role contributions at that timestamp; do not sum per-role maxima.
Logical input/state/image/base64/request/response/event buffer sizes are separate lifecycle
diagnostics. They overlap RSS and each other; never add them to RSS or claim exhaustive attribution.
macOS RSS is a proxy, shared pages are not deduplicated; browser/cloud weights are excluded.

## Basics

Ten alternatives: 01 ABC, 02 AB, 04 AB, 05 ABC; same four episode goals per DP, three repetitions.
Same-source quiescent preparation is disclosed, and all its model usage is charged. Optional
tactics are disabled. QA04 task definitions and operational module registry freeze before patching.
