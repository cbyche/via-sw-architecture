# AA-012 — QA-02 Constraint-to-Raw-Evidence Coverage Audit

## Status

**Architecture Analysis Amendment — DP-00 Runtime Pilot v0 Session 2.6**

This review performs no scoring and selects no architecture winner. It completes the raw measurement contract required before Python Session 3.

## Readiness amendment

AA-011 recorded a pre-analysis `QA-02 Raw Evidence Sufficiency = PASS` based on event-category coverage. During later Python derivation design, P01 supplied a counterexample: `VOLUME_CHANGED` with final value `35` cannot distinguish `50 -> 35`, `20 -> 35`, and `35 -> 35`. Copying the Oracle's `DECREASED` into Actual would violate Actual/Expected independence.

Session 2.6 replaced scenario-level sampling with an exhaustive constraint-level audit. The Pilot contains 36 QA-02 constraints across P01–P10. The machine-readable source is `benchmark/contracts/pilot-v0-constraint-evidence-map.json`; an executable test requires its constraint-id, scenario, and dimension set to equal the Oracle registry exactly.

## Constraint legitimacy review

Every constraint was traced to functional semantics, product requirement, policy, capability contract, or scenario semantics. No constraint requires an Alternative-private component or implementation-only call sequence. Route and owner constraints describe benchmark-visible execution responsibility and include topology-neutral allowed sets where multiple routes are correct. No `ORACLE_OVER_SPECIFIED`, `AUTHORITY_AMBIGUOUS`, or unresolved `SCHEMA_MISMATCH` remains.

## Constraint-to-evidence coverage matrix

| Scenario | Constraint / dimension | Expected value or rule | Expected source | Actual raw evidence / fields | Authority | Deterministic derivation | Independent / gap / status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| P01 | P01-REQ-EFFECT / observable_effect | REQUIRED equals `{"capability_id":"volume.decrease","state":"DECREASED"}` | `oracles.json#P01-REQ-EFFECT` | `canonical UsefulOutcomeObserved.effect` / `effect.capability_id, effect.before_value, effect.after_value` | OUTCOME_PROBE | capability is observed; DECREASED iff after_value < before_value | YES / NONE / PASS |
| P01 | P01-ALLOW-ROUTE / execution_path | ALLOWED in `["LOCAL_DIRECT:VIA_FAST","EXECUTOR_DIRECT:ARGO"]` | `oracles.json#P01-ALLOW-ROUTE` | `Architecture.RouteCommitted + ExecutionStarted` / `route.*, invocation.executor_id` | AUT + TOOL_FIXTURE/AGENT_FIXTURE | compare committed semantic path and accepted executor | YES / NONE / PASS |
| P01 | P01-FORBID-AGENT / delegated_agent | FORBIDDEN in `["MailAgent","FileAgent","NetworkAgent"]` | `oracles.json#P01-FORBID-AGENT` | `Architecture.RouteCommitted + ExecutionStarted` / `route.delegation_chain, invocation.executor_id` | AUT + EXECUTION_FIXTURE | forbidden iff any actual route/invocation names forbidden agent | YES / NONE / PASS |
| P02 | P02-REQ-RESULT / observable_effect | REQUIRED predicate `LATEST_DOWNLOAD_NAME_EMITTED` | `oracles.json#P02-REQ-RESULT` | `UsefulOutcomeObserved.effect + ResultBound` / `effect.*, product_correlation.*` | OUTCOME_PROBE + PRODUCT_CORRELATION | effect state and bound result are present | YES / NONE / PASS |
| P02 | P02-ALLOW-OWNER / execution_owner | ALLOWED in `["ARGO"]` | `oracles.json#P02-ALLOW-OWNER` | `ExecutionStarted` / `invocation.executor_id` | AGENT_FIXTURE | actual accepted executor determines owner | YES / NONE / PASS |
| P02 | P02-FORBID-FAST / execution_owner | FORBIDDEN equals `VIA_FAST` | `oracles.json#P02-FORBID-FAST` | `ExecutionStarted` / `invocation.executor_id` | AGENT_FIXTURE | forbidden iff actual accepted executor is VIA_FAST | YES / NONE / PASS |
| P03 | P03-REQ-RESULT / observable_effect | REQUIRED predicate `NETWORK_STATUS_OBSERVED` | `oracles.json#P03-REQ-RESULT` | `UsefulOutcomeObserved.effect` / `effect.capability_id, effect.subject_id, effect.state` | OUTCOME_PROBE | NETWORK_STATUS_OBSERVED from observed network.status effect | YES / NONE / PASS |
| P03 | P03-ALLOW-ROUTE / execution_path | ALLOWED in `["EXECUTOR_DIRECT:NetworkAgent","EXECUTOR_DELEGATED:ARGO->NetworkAgent"]` | `oracles.json#P03-ALLOW-ROUTE` | `RouteCommitted + ExecutionStarted` / `route.*, invocation.executor_id` | AUT + AGENT_FIXTURE | derive direct/delegated path from route and accepted executor | YES / NONE / PASS |
| P03 | P03-FORBID-WRONG / delegated_agent | FORBIDDEN in `["MailAgent","FileAgent"]` | `oracles.json#P03-FORBID-WRONG` | `RouteCommitted + ExecutionStarted` / `route.delegation_chain, invocation.executor_id` | AUT + AGENT_FIXTURE | forbidden iff wrong agent appears in committed/accepted execution | YES / NONE / PASS |
| P04 | P04-REQ-REFERENT / referent | REQUIRED equals `{"role":"source","object_id":"doc-right"}` | `oracles.json#P04-REQ-REFERENT` | `Architecture.ReferentBound` / `resolved_referent_id, referent_role` | AUT_COMMITTED_DECISION | compare actual bound object and role | YES / NONE / PASS |
| P04 | P04-REQ-OPEN / observable_effect | REQUIRED equals `{"object_id":"doc-right","state":"OPEN_USABLE"}` | `oracles.json#P04-REQ-OPEN` | `UsefulOutcomeObserved.effect` / `effect.subject_id, effect.state, effect.capability_id` | OUTCOME_PROBE | document.open on subject with OPEN_USABLE | YES / NONE / PASS |
| P04 | P04-ALLOW-ROUTE / execution_path | ALLOWED in `["EXECUTOR_DIRECT:FileAgent","EXECUTOR_DELEGATED:ARGO->FileAgent","EXECUTOR_DIRECT:ARGO"]` | `oracles.json#P04-ALLOW-ROUTE` | `RouteCommitted + ExecutionStarted` / `route.*, invocation.executor_id` | AUT + EXECUTION_FIXTURE | derive committed path and accepted executor | YES / NONE / PASS |
| P04 | P04-FORBID-LEFT / referent | FORBIDDEN equals `{"role":"source","object_id":"doc-left"}` | `oracles.json#P04-FORBID-LEFT` | `Architecture.ReferentBound` / `resolved_referent_id, referent_role` | AUT_COMMITTED_DECISION | forbidden iff actual source binding is doc-left | YES / NONE / PASS |
| P05 | P05-REQ-RELATION / task_association | REQUIRED equals `FOLLOW_UP` | `oracles.json#P05-REQ-RELATION` | `Architecture.TaskAssociated` / `task_relation` | AUT_COMMITTED_DECISION | actual task relation equals FOLLOW_UP | YES / NONE / PASS |
| P05 | P05-REQ-TASK / task_association | REQUIRED equals `{"task_id":"T1"}` | `oracles.json#P05-REQ-TASK` | `TaskAssociated product correlation` / `product_correlation.task_id` | PRODUCT_CORRELATION | actual associated task identity | YES / NONE / PASS |
| P05 | P05-REQ-RESULT / result_binding | REQUIRED equals `T1` | `oracles.json#P05-REQ-RESULT` | `Architecture.ResultBound correlation` / `task_id, result_id, execution_id` | PRODUCT_CORRELATION | actual result association identifies task | YES / NONE / PASS |
| P05 | P05-FORBID-T2 / task_association | FORBIDDEN equals `{"task_id":"T2"}` | `oracles.json#P05-FORBID-T2` | `TaskAssociated product correlation` / `product_correlation.task_id` | PRODUCT_CORRELATION | forbidden iff actual associated task is T2 | YES / NONE / PASS |
| P05 | P05-FORBID-NEW / task_association | FORBIDDEN equals `{"task_relation":"NEW"}` | `oracles.json#P05-FORBID-NEW` | `Architecture.TaskAssociated` / `task_relation` | AUT_COMMITTED_DECISION | forbidden iff actual relation is NEW | YES / NONE / PASS |
| P06 | P06-REQ-CLARIFY / clarification | REQUIRED equals `{"requested":true}` | `oracles.json#P06-REQ-CLARIFY` | `ClarificationRequested/Resolved` / `event order, turn correlations` | AUT_COMMITTED_DECISION | requested and resolved actions reconstructed by sequence | YES / NONE / PASS |
| P06 | P06-REQ-REFERENT / referent | REQUIRED equals `{"role":"source","object_id":"doc-right"}` | `oracles.json#P06-REQ-REFERENT` | `Architecture.ReferentBound` / `resolved_referent_id, referent_role` | AUT_COMMITTED_DECISION | actual resolved source identity | YES / NONE / PASS |
| P06 | P06-FORBID-EARLY / observable_effect | FORBIDDEN predicate `FILE_OPEN_BEFORE_CLARIFICATION_RESOLVED` | `oracles.json#P06-FORBID-EARLY` | `ClarificationResolved + UsefulOutcomeObserved` / `sequence_number, effect.capability_id` | AUT + OUTCOME_PROBE | forbidden iff document effect precedes resolution | YES / NONE / PASS |
| P06 | P06-FORBID-LUCKY / clarification | FORBIDDEN equals `{"requested":false}` | `oracles.json#P06-FORBID-LUCKY` | `ClarificationRequested` / `presence, sequence_number` | AUT_COMMITTED_DECISION | forbidden iff terminal trace lacks request | YES / NONE / PASS |
| P07 | P07-REQ-PLANNED / observable_effect | REQUIRED predicate `DOWNLOADS_ORGANIZED_BY_DOMAIN_EXECUTOR` | `oracles.json#P07-REQ-PLANNED` | `ExecutionStarted + UsefulOutcomeObserved` / `invocation.executor_id, effect.executor_id, effect.state` | EXECUTION_FIXTURE + OUTCOME_PROBE | domain executor and organized effect jointly establish predicate | YES / NONE / PASS |
| P07 | P07-ALLOW-ROUTE / execution_path | ALLOWED in `["EXECUTOR_DIRECT:ARGO"]` | `oracles.json#P07-ALLOW-ROUTE` | `RouteCommitted + ExecutionStarted` / `route.*, invocation.executor_id` | AUT + AGENT_FIXTURE | actual path must be accepted ARGO execution | YES / NONE / PASS |
| P07 | P07-FORBID-FAST / execution_owner | FORBIDDEN equals `VIA_FAST` | `oracles.json#P07-FORBID-FAST` | `ExecutionStarted` / `invocation.executor_id` | EXECUTION_FIXTURE | forbidden iff accepted executor is VIA fast/local | YES / NONE / PASS |
| P07 | P07-FORBID-LOCAL-EFFECT / observable_effect | FORBIDDEN predicate `LOCAL_FAST_CLAIMED_DOMAIN_PLANNING_SUCCESS` | `oracles.json#P07-FORBID-LOCAL-EFFECT` | `ExecutionStarted + UsefulOutcomeObserved` / `invocation.executor_id, effect.executor_id, effect.state` | EXECUTION_FIXTURE + OUTCOME_PROBE | detect local executor claiming organized domain effect | YES / NONE / PASS |
| P08 | P08-REQ-TERMINAL / failure_outcome | REQUIRED in `["SAFE_FAILURE:MALFORMED","RECOVERED_TO_NETWORK"]` | `oracles.json#P08-REQ-TERMINAL` | `model-call + EpisodeFailed` / `status, failure, reason` | MODEL_FIXTURE + BENCHMARK_TERMINAL | fault status and typed terminal reason jointly derive outcome | YES / NONE / PASS |
| P08 | P08-FORBID-COMMIT / routing | FORBIDDEN predicate `COMMIT_DERIVED_FROM_MALFORMED_OUTPUT` | `oracles.json#P08-FORBID-COMMIT` | `RouteCommitted over terminal trace` / `event presence, sequence_number` | AUT_COMMITTED_DECISION | forbidden iff any commit follows malformed model evidence | YES / NONE / PASS |
| P08 | P08-FORBID-EFFECT / observable_effect | FORBIDDEN predicate `UNAUTHORIZED_DOMAIN_EFFECT` | `oracles.json#P08-FORBID-EFFECT` | `UsefulOutcomeObserved over terminal trace` / `event presence, effect.*` | OUTCOME_PROBE | forbidden iff any unauthorized effect exists | YES / NONE / PASS |
| P09 | P09-REQ-SEMANTIC-CHECK / routing | REQUIRED predicate `MAIL_AGENT_REJECTED_OR_NOT_COMMITTED` | `oracles.json#P09-REQ-SEMANTIC-CHECK` | `model semantic candidate + route events` / `semantic_output_reference, RouteCandidateRejected, RouteCommitted` | MODEL_FIXTURE + AUT_COMMITTED_DECISION | MailAgent candidate must be rejected or absent from commit | YES / NONE / PASS |
| P09 | P09-ALLOW-OUTCOME / failure_outcome | ALLOWED in `["SAFE_FAILURE:WRONG_CANDIDATE","RECOVERED_TO_NETWORK","CLARIFICATION_REQUESTED"]` | `oracles.json#P09-ALLOW-OUTCOME` | `EpisodeFailed/route/clarification events` / `reason, route.*, clarification sequence` | BENCHMARK_TERMINAL + AUT | derive actual safe failure/recovery/clarification | YES / NONE / PASS |
| P09 | P09-FORBID-MAIL-COMMIT / delegated_agent | FORBIDDEN equals `MailAgent` | `oracles.json#P09-FORBID-MAIL-COMMIT` | `RouteCommitted + ExecutionStarted` / `route.*, invocation.executor_id` | AUT + EXECUTION_FIXTURE | forbidden iff MailAgent committed or accepted | YES / NONE / PASS |
| P09 | P09-FORBID-MAIL-EFFECT / observable_effect | FORBIDDEN predicate `MAIL_AGENT_EXECUTED_WIFI_REQUEST` | `oracles.json#P09-FORBID-MAIL-EFFECT` | `ExecutionStarted + UsefulOutcomeObserved` / `invocation.*, effect.*` | EXECUTION_FIXTURE + OUTCOME_PROBE | detect MailAgent execution/effect for Wi-Fi request | YES / NONE / PASS |
| P10 | P10-REQ-TERMINAL / failure_outcome | REQUIRED in `["SAFE_FAILURE:TIMEOUT","SAFE_FAILURE:NO_RESPONSE","RECOVERED_TO_NETWORK"]` | `oracles.json#P10-REQ-TERMINAL` | `model-call + EpisodeFailed` / `status, failure, reason` | MODEL_FIXTURE + BENCHMARK_TERMINAL | fault status and typed terminal reason jointly derive outcome | YES / NONE / PASS |
| P10 | P10-FORBID-FABRICATED / routing | FORBIDDEN predicate `ROUTE_COMMIT_WITHOUT_VALID_MODEL_OUTPUT` | `oracles.json#P10-FORBID-FABRICATED` | `RouteCommitted over terminal trace` / `event presence, sequence_number` | AUT_COMMITTED_DECISION | forbidden iff commit exists without completed model output | YES / NONE / PASS |
| P10 | P10-FORBID-EFFECT / observable_effect | FORBIDDEN predicate `UNAUTHORIZED_DOMAIN_EFFECT` | `oracles.json#P10-FORBID-EFFECT` | `UsefulOutcomeObserved over terminal trace` / `event presence, effect.*` | OUTCOME_PROBE | forbidden iff any unauthorized effect exists | YES / NONE / PASS |

## Gap disposition

The initial P01 rows were `RAW_EVIDENCE_INSUFFICIENT`: the probe retained only `VOLUME_CHANGED/value=35/state=CHANGED`, and no accepted-invocation capability identity existed. The fix records actual invocation identity and actual fixture state boundaries. All other rows were rechecked at constraint granularity and required no Oracle changes. No Oracle relaxation was made.

## Raw contract completion

`canonical-event-v2` adds a compact required `ExecutionInvocation { capability_id, executor_id }` to `execution.started`. It also adds `capability_id`, `before_value`, and `after_value` to observable effects. The controlled Tool Fixture changes volume from 50 to 35; the Outcome Probe preserves both values. Volume observations require both numeric boundaries, and `DECREASED := after_value < before_value`. A regression proves `20 -> 35` remains an increase even when Expected is decrease.

The execution fixture supplies invocation identity after accepting the actual request. The Outcome Probe supplies external effect state. AUT route, referent, task, clarification, and result events remain self-reports at committed product boundaries. Model Fixture status/candidate plus benchmark terminal failure provide failure evidence. Behavior plans and Oracles are never Actual sources.

P08 and P10 reconstruction jointly uses actual Model Fixture fault status and typed terminal failure. P09 retains `route_commit_expectation = OPTIONAL`; raw distinguishes MailAgent proposal, commit absence/presence, accepted execution, terminal failure, and effects. Commit absence alone is not a correctness verdict.

## Executable completion gate

The coverage test requires for every row: exact Oracle constraint identity, scenario and dimension match, explicit raw fields, authority, deterministic derivation rule, `independently_derivable = true`, and final `PASS/NONE`. A separate test constructs runtime inputs without reading the Oracle registry and reconstructs P01–P10 Actual facts. It does not compare them with Expected or calculate AECR.

## Timed-path and architecture review

The added timed-path work is limited to compact strings/integers in typed values, one controlled integer state update, and in-memory append. JSON construction, serialization, file I/O, Python, full prompts, and full generated bodies remain outside the timed path. No A/B/C/D source file changed. Routing, retry, Fast eligibility, B ARGO responsibility, D selection, and campaign semantics are unchanged.

## Revised verdict

| Gate | Verdict |
| --- | --- |
| P01 Evidence Gap | PASS |
| Full Constraint Coverage Audit | PASS |
| QA-02 Raw Evidence Sufficiency | PASS |
| Actual/Oracle Independence | PASS |
| Oracle Constraint Neutrality | PASS |
| Canonical Semantic Trace Completeness | PASS |
| QA-01 Timed-path Preservation | PASS |
| Python Session 3 Readiness | PASS, subject to the full repository regression recorded with the implementing commits |

AA-011 remains unchanged as historical evidence of the earlier methodology. This amendment records the counterexample, exhaustive replacement audit, and revised basis for readiness.
