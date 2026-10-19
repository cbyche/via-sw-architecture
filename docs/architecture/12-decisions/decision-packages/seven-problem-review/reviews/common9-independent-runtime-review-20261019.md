# Common9 independent runtime, authority and data-chain review

2026-10-19, pre-formal. No model API call or frozen candidate mutation was performed by this review. The execution lead owns subsequent fixes/freezes. Review scope: common runtime, evidence supply, handoff/receiver, source registry and actual external driver.

## Findings

### P2: Task/question ownership at CONTROL admission — fixed, independent confirmation

The earlier guard separately checked an existing Task and an existing question in any Task. TA with question qA and TB with qB could therefore admit `{task_id:TA,question_id:qB}`. The execution lead changed `runtime.rs:448–455` to require the operation Task, containing Task and question's own Task all match under the same source/state gate. Read-only inspection confirms the fixed predicate. Independent raw-trace review of `development/runtime-guards-1961759997325333/question-ownership/raw/runs/AAAA-S20-r1-eb6862/trace.json` confirms the injected TA/Q-TB operation remains in SEMANTIC_ADOPTION, produces WORKFLOW_FAILED/CONTROL_QUESTION_NOT_DISPLAYED and creates no CONTROL_INGRESS. Its `question-ownership-check.json` passes. Source diff and actual negative transport evidence are verified; the lead owns the new baseline freeze.

This checks actual observed ownership. It does not infer the user's intended answer from the evaluator or repair an incorrectly understood approval.

### P2: Derived result source lacks upstream provenance — fixed, independently verified within stated limits

`harness.py:28–30` copies actual receiver body into a new delegated artifact source with `local_allowed:true` and `external_allowed:true`, but does not carry original-source provenance. `runtime.rs:603–604` registers the supplied result source unchanged. `sources.rs` recursively propagates access restrictions only for sources with a `provenance` array.

Consequently a DOC-A-derived TASK:result could remain directly readable/exportable after DOC-A's authority was withdrawn. Existing captured-history/Task span sanitization did not affect that separate source-registry entry. Findings were sent to root and the execution owner before formal freeze.

The lead fixed `harness.py:30` to carry actual receipt `actually_supplied_source_spans`. Independently reviewed `runtime.rs:605–616` now overwrites supplied artifact provenance with admitted actual EvidenceResult spans and derives local/external rights from original authority. It does not use expected target sets or evaluator prose. `development/runtime-guards-1961759997325333/external-artifact-provenance.json` confirms that the actual external adapter propagates the actual readset. `preflight/cargo16-tests.log` records16 passed Rust checks; `sources.rs:248–268` exercises DOC→RESPONSE→TASK:result and verifies local read/export both fail after upstream DOC withdrawal while SAFE remains readable/exportable.

Verification limits: adapter readset propagation is a deterministic actual-function check; recursive withdrawal is an actual registry unit check; runtime provenance overwrite is independently source-reviewed. This is not claimed as one end-to-end runtime transport/withdrawal test. The final full63-trajectory offline-v11 rerun passed independent regression/evaluator review:301 substantive callbacks,161 final goals with resolved causal paths and105 exact sets. All63 episodes have valid sampled RSS under the pre-formal v3 policy;13 confirmed-exit boundary samples were separately retained/excluded, no invalid sample remained, and no startup-zero exception happened to be needed in this run. The48 synthetic checks and their manifest/source hashes were verified. These are offline functional preparation results, not model accuracy or comparative performance evidence. No unresolved P1/P2 remains from this focused review; future findings or failed final regressions require another preserved fix/freeze.

## Paths reviewed without another comparative-validity finding

- `evidence.rs:377–492`: A's actual working originals, C's supported records and actual recent delivered state are consumed; missing originals use the common registry. Supplied acquisition/producer node IDs join the real dependency graph. Actual old-version locators remain distinct from the latest source catalog.
- `handoff.rs:235–298`: actual previously acquired originals can satisfy A/B purpose-specific handoff without forced rereading. C uses the same permitted source manifest and capable receiver's bounded read interface. No one-variant prohibition on raw access was found.
- `handoff.rs:350–407`: receiver choices, actual original reads, retries and completion are actual model/local operations. Receiver model service and VIA source serving are separately traced; every call reaches the common cost ledger. Literal quotes are checked against actual supplied authorized lines.
- `runtime.rs:244–292`: publication rechecks goal revision and current source-range authority under the source/state gate. Model proposals do not directly publish themselves.
- `runtime.rs:354–362` and `sources.rs:permission`: actual model-selected permission operations use registered source/declared-segment identifiers and are recorded. Whether those operations correctly interpret the user's consent is a semantic diagnostic; this review does not add an oracle-based intent guard.
- Formal paths receive the authored current source/capture inputs rather than oracle response/goal/binding rows. Offline semantic/vision doubles are isolated from the formal client. No answer-key injection was found in the inspected formal paths.

## Limits

This is a focused source review and offline trace inspection, not an adversarial proof. Exact original quote support establishes provenance bounds; it does not prove that an answer satisfies the purpose, Task association or conditional restrictions. Those remain separate body/N-R/N-H NEEDS_REVIEW records. The external adapter generates bounded synthetic drafts/state outcomes under the declared8s profile; no complete commercial Agent, file editor, mail sender or Windows UI was validated. The9-case suite does not establish behavior for arbitrary source graphs or repeated artifact revisions outside its authored scenarios.
