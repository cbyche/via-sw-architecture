# Review v2: evidence supply and handoff implementation

2026-10-18. Constructed synthetic PoC fixtures, not a user study or product integration.
This describes only the NEW `four_dp_poc_v2` implementation. Original candidates, freezes,
raw failures and reported values remain in `four_dp_poc` and the first campaign.
The original post-adoption handoff timing and mandatory A compose path do not describe v2.

## Function and measurement contract

Read [CONTRACT.md](CONTRACT.md) and the root [v2 contract](../../../docs/architecture/12-decisions/decision-packages/seven-problem-review/14-corrected-poc-contract.md).
All model stages use the same frozen Qwen configuration through the shared runtime/ledger.
No candidate reads any oracle or branches on fixture/episode IDs. Offline responders below are
explicit protocol test doubles, never substituted into actual model measurements.

- Capture the actual user utterance before semantic calls. Goal clocks retain that input time.
- Commit actual selected target/intent state, then `Context.progress` displays a concrete handling
  intention naming resolved source titles. Metadata can identify a target, but cannot supply facts.
  A semantically wrong yet structurally valid target adoption retains its real first display time.
- Detailed completion is a separate callback after the actual acquired evidence and output contract
  pass. Direct complete-from-evidence can make these close; no delay is inserted to separate QAs.
- The displayed answer may paraphrase structured fact claims. Each claim still needs an exact quote
  within the actually supplied cited source lines at the current allowed revision. Exact quotes must be nonempty; a valid short
  value such as `8W` is not rejected by an arbitrary three-character threshold. For derived
  ACTUALLY_DELIVERED_ANSWER evidence, quote only actual previously displayed text and retain its
  separate original source provenance; never treat an unshown facts.statement as delivered.
  Empty/null, source-version declarations alone, unseen manifest facts and plain request/expression
  echoes fail. The former verbatim claim-in-display coupling was removed after calibration-v1
  rejected valid paraphrases; all those failures/costs remain in their original snapshot.
- This is a mechanical source-backing check, **not proof that a statement follows from its quote or
  answers every semantic requirement**. Frozen referent, content and handoff diagnostics remain separate.
- A failed semantic output stage gets at most one repair. Read-selection recovery and final-answer
  repair are distinct stages. Failed proposals, reasons and every call cost remain recorded. Exhausted
  recovery emits GOAL_FAILED, never GOAL_COMPLETE or an invented successful time.
- Source reading still uses `read_to_string` plus range selection; a returned-range reduction is not
  proof of physical disk I/O reduction. Rights and current revisions are checked before use.

## Stable operational module registry (9 owned Rust modules)

All are under `prototypes/candidates/four_dp_poc_v2/src/`. Physical module boundaries reflect
source authority, independently produced state, request consumption and downstream execution.
They were established before QA04 feature patches; they are not split to obtain different scores.

| Module | Stable responsibility | Variant use |
| --- | --- | --- |
| `dp01.rs` | Conversation/source lifecycle, view/recent/publication ownership | ABC |
| `dp01_evidence.rs` | Current original reads, actual-delivery bounds, common completion guard | ABC |
| `dp01_producer.rs` | Purpose-neutral relationship production/publication | C |
| `dp01_request.rs` | Request interpretation/adoption, evidence consumption, completion/recovery | ABC |
| `dp05.rs` | Public lifecycle, common actual user prefix, mandatory admission and result binding | ABC |
| `dp05_sources.rs` | Current authorized spans/read serving and common receiver output guard | ABC |
| `dp05_record.rs` | Continuous purpose-neutral work-record production/publication | B |
| `dp05_handoff.rs` | VIA purpose-specific detail selection and optional compose; C returns manifest-only | A/B semantic work |
| `dp05_receiver.rs` | Identical real receiver select-or-complete, original reads and result/repair | ABC |

The shared runtime's context, source adapter, model wire and main are separately owned common modules.
QA04 counts actual changed operational modules for an independently applied task, not compiled files,
all registry entries, or the sum of different tasks. Equal results are valid.

## DP01 mechanisms and actual stages

All four original episodes remain: same-source followup, topic change, partial delivery plus source
correction, and new conversation reusing old source relations. The old original-ID/version oracle
is preserved. `oracles/dp01.json` adds pre-calibration final-content groups for all 12 bounded goals.
`score_dp01.py` checks the final detailed outputs, never the earlier handling intention. Numeric
format aliases are explicit; this finite lexical diagnostic is not a general semantic judge.

| Design | Retained state and request mechanism | Preparation cost |
| --- | --- | --- |
| A | Retain actual read passages in the same conversation; reuse current evidence. Source correction invalidates the whole view; new conversation resets it. Keep a finite 14KB serialized view by oldest-passage eviction if necessary. | No independent relation producer. Any actual view maintenance is logged. Partial invalidation is still a separate tactic. |
| B | Start from the common recent actually delivered answers plus metadata; read a request-local evidence bundle. Do not retain extra original read passages after completion. | No independent relation producer. |
| C | Independently publish source-backed semantic relations on observed source activity; request consumes retained published records and their supporting passages, with original fallback. Future utterance is absent from production input. | All production calls/reads, replaced intermediate publications and unused work are charged, even when before input capture. |

Actual stage names:

1. `dp01_publish_relations` only for C source activity. Validate spans against current actual producer
   reads; attach their exact supporting passages to the publication. Production is quiescent before
   the next authored input in this finite fixture; no busy-producer superiority claim follows.
2. `dp01_select_adopt_or_complete` for every request. Its model-selected original IDs and concrete
   intent can be committed/displayed before further factual work. If actual supplied evidence and
   the answer contract suffice, COMPLETE needs no second answer call.
3. Otherwise service model-selected reads. In particular, rejected COMPLETE plus empty reads
   **cannot enter an empty final answer**: read the current allowed spans of its actual selected
   targets, or use the single `dp01_recover_read_selection` if targets/reads are absent. No oracle
   target or expected fact supplies this fallback.
4. `dp01_source_backed_answer` completes from actual evidence. A null/echo/stale/unbacked output
   is rejected and `dp01_repair_source_backed_answer` receives the actual evidence and error reasons.
   After repair failure, mark the goal failed. Successful outputs update actual delivered history;
   only A additionally commits acquired passages to its view.

| QA | Intuitive expected effect | Actual evidence to report |
| --- | --- | --- |
| QA01 | Reused A view or C relations may resolve targets before fresh original construction. Explicit B metadata may identify a target equally quickly. | Input→actual adoption progress, proposal/first wrong adoption, first-stage calls and tokens; nonresponse coverage. |
| QA02 | Retained relations can help or confuse the original referent set. | First committed original-ID set and required revisions, all 12 judgments per repetition; content success separately. |
| QA03 | Reuse can remove reads/another answer stage; fresh request evidence may have smaller input. | All goal-linked reads/calls/repairs through actual detailed completion; incomplete goals preserved. |
| QA04 | Delivered provenance and range withdrawal may affect view, bundle or producer/consumer contracts. | Independent C05/C06 patches and functional acceptance, actual changed modules; C01/C03 controls. |
| QA05 | Retention adds recurring input, fresh construction adds calls, production adds preparation/output. | Per-stage whole-call input/output tokens and standard-price cost, preparation/replaced/unused work included. |
| QA06 | View, request evidence, publication and production have different lifetimes. | Same-time per-role RSS plus separate logical source/view/recent/publication sizes; remote weights excluded. |

## DP05 mechanisms and actual stages

All four original episodes remain: no handoff, one handoff, three purposes, large document with
bounded reads plus correction/revocation. Every `delegate`/`observe` now contains an actual raw
user `utterance`, not already-resolved purpose/recipient/completion fields. Prior `adopt` events
represent existing public Task conditions, not the meaning of that new user input. All ABC get
the SAME `dp05_interpret_user_request` prefix, actual source catalog and current public state.
There is no fixed common timing constant and no freely supplied expected semantic answer.

The model proposes CLOSE/DELEGATE, purpose, completion, source targets, concrete handling and
recipient from the same actual minimal adapter. After schema/current-rights validation (one
`dp05_repair_user_interpretation` if needed), commit/display that real intention. CLOSE has a
real parsed prefix and explicit no-delegation terminal. The mandatory contract uses the adopted
model purpose/recipient/completion and authoritative prior constraints/permissions/actions.

| Design | Actual construction path | Cost boundary |
| --- | --- | --- |
| A | `dp05_select_handoff_details` may directly select adequate valid source-span references or already acquired evidence. **No mandatory compose call.** If it chooses missing original detail, actual reads and `dp05_compose_capsule` build the explanation. | Prefix + actual selection/read/optional compose, then actual receiver. No capsule cache/native session tactic. |
| B | `dp05_produce_work_record` maintains purpose-neutral source-backed reusable explanation on public activity. `dp05_extract_record` can directly select sufficient valid blocks; raw fallback/compose remains only for missing detail. | Prefix, all ongoing record production including unused/replaced work, actual extraction/fallback and receiver. No batching/partial-update tactic. |
| C | Same actual input prefix and mandatory admission, manifest and current source authority; receiver selects needed originals. | Prefix cost is VIA and therefore no longer zero. Detail-selection/draft cost is receiver; serving, delivery and result binding remain VIA. No read cache/prefetch tactic. |

References are valid transport, but not source facts. Host-attached actual original passages,
or valid record supporting passages, permit the identical receiver to finish directly. Manifest
titles or a reference-only capsule force original acquisition before factual completion.

Receiver stages are `dp05_receiver_select_or_complete`, optional one
`dp05_receiver_recover_read_selection`, then `dp05_receiver_result` if reads were needed,
and at most one `dp05_receiver_repair_result`. Every final result is checked against actual
supplied source spans/quotes and current access. A/B/C share all those capabilities and guards.

`AGENT_INGRESS` is VIA dispatch. `DOWNSTREAM_SHIM_ACCEPTED` and `DOWNSTREAM_SHIM_RETURN`
bracket actual logical receiver execution in this process, including its real Qwen dependencies.
`SERVING_START`/`SERVING_END` identify VIA original-read servicing within that interval, so it is
included again in VIA processing rather than silently excluded. The common analyzer owns exact
interval handling. This is an API-boundary logical-shim proxy, not commercial Agent transport,
native session, process-isolation or provider-internal-compute measurement. Report full elapsed,
excluded receiver interval and VIA serving/prefix/post-return alongside QA03.

The receiver preserves conditional antecedents: a tie-break cannot become an unconditional
recommendation. This is a general rule, not a supplied product decision. Its actual adherence is
reviewed separately on final text under `analysis/dp05-semantic-review-rubric.md`, including
body/constraints_respected disagreements. Numeric content markers use digit boundaries.

N-H structure/rights/currentness remains distinct from purpose/completion wording and receiver
content. Oracle original expected prose is retained under `original_expected_prose`; pre-authored
`purpose_contract_groups` accepts bounded paraphrases. `score_dp05.py` returns structural N-H,
purpose-group diagnostics, their combined contract result, and receiver-content groups separately.
Exact four-sentence formatting in E2 additionally needs explicit inspection of the preserved text.
Candidate code never sees these groups. No success claim should use structural N-H alone.

| QA | Intuitive expected effect | Actual evidence to report |
| --- | --- | --- |
| QA01 | Same real prefix may tie; later purpose/detail preparation need not delay adopted handling. | Raw input→actual prefix adoption progress, first wrong intention and missing response. Never old post-adoption 0.02ms. |
| QA02 | Handoff conditions are not referent-set QA02. | N/A; report N-H, actual purpose diagnostics and content separately. |
| QA03 | Sufficient A refs/B records can avoid VIA composition; C moves detail reasoning to the receiver. | Prefix/selection/compose/serving/binding and logical receiver excluded interval, plus full wall time and coverage. |
| QA04 | Partial artifact updates/range withdrawal touch distinct durable and serving state. | Independent C04/C06 functioning patches on all ABC, same fixed feature acceptance; C01/C03 controls. |
| QA05 | B prep can be unused or reused; C can reduce VIA detail cost while increasing receiver cost. | Stage-level VIA and receiver calls/tokens/cost, both totals and combined cost; real prep included. |
| QA06 | Extra work records persist; read serving may still read a large whole file. | Shared-time RSS and separate logical source/acquired/record/capsule/read buffers; no I/O gain inferred from smaller prompts. |

## Preflight and next phase

`offline_dp01.py` and `offline_dp05.py` construct test responses from actual supplied text and
do not load oracles. They intentionally do not perform accurate referent/goal semantics. Repetition
1 exercises normal schema, 2 injects null final output followed by a valid repair, 3 exhausts that
repair to verify failures remain failures. ABC run through real Rust callbacks/source reads.
These counts/times are OFFLINE protocol evidence, not model quality or provider latency.

Owned pure Rust guards after the parent-approved calibration-v2 correction: 11 DP01 + 11 DP05. Tests cover metadata-only completion, actual evidence,
request/expression echo, accepted paraphrase without duplicate field wording, current/stale/revoked/undelivered spans, quote location, direct refs/direct record and no-unshown-recent reuse.
The shared runtime archives preflight records separately and runs any actual CALIBRATION only
after parent authorization and a separate snapshot. Every provider call uses the new USD4 ledger.

Future QA04 patches are independent scratch implementations after the basic registry freeze:
DP01 C05 actually delivered answer source/revision/extent query and C06 range-level withdrawal;
DP05 C04 section-specific Agent artifact update and C06 range-level withdrawal. Baseline whole-source
rights checks remain. C05 excludes unpresented proposals; C04 preserves unrelated sections/tasks;
C06 blocks withdrawn spans and preserves allowed spans. Actual diffs, regressions, module counts,
development time and revisions will be reported; no metric superiority is assigned in advance.
