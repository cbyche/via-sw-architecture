# VIA DP-00 Offline Causal Architecture Evaluation v2

## Evidence status

Campaign v1 validated QA-v1 population handling, provenance, scoring, candidate pre-registration, and evaluation plumbing. Its executable candidate runtimes did not causally materialize enough R1/R3 architecture behavior to support R1-vs-R3 architecture equivalence or selection claims.

The v1 numbers remain historical preliminary evidence and were not used to select DP-00.

## Primary QAs

| QA | R1 raw / score / target | R3 raw / score / target | R1+@ raw / score / target | R1−R3 | causal explanation |
| --- | ---: | ---: | ---: | ---: | --- |
| QA-01 | 0.925603 seconds / 5 / met | 0.925603 seconds / 5 / met | 0.912544 seconds / 5 / met | 0 | R1 and R3 tie at population p95. Per-case graphs still preserve structural costs: R3 specialist delegation adds a common-cost hop, while eligible R1+@ reads omit model inference and downstream return, producing its small aggregate advantage. |
| QA-02 | 100 percent / 5 / met | 100 percent / 5 / met | 100 percent / 5 / met | 0 | Common semantic and Agent replay capabilities are equal; candidate-owned state machines bind independently produced versioned results. |
| QA-04 | 100 percent / 5 / met | 100 percent / 5 / met | 100 percent / 5 / met | 0 | The preregistered dependency graph integrates Agent evolution at each architecture's authoritative Agent boundary without propagating into forbidden Core zones. |
| QA-05 | 100 percent / 5 / met | 100 percent / 5 / met | 100 percent / 5 / met | 0 | The frozen component-to-zone graph propagates each requested product change through its declared dependency seam; no expected zone is read during propagation. |

## Secondary QAs

| QA | R1 | R3 | R1+@ |
| --- | ---: | ---: | ---: |
| QA-03 | 100 / 5 | 100 / 5 | 100 / 5 |
| QA-06 | 100 / 5 | 100 / 5 | 100 / 5 |
| QA-08 | 0.209562 / 5 | 0.209562 / 5 | 0.209562 / 5 |
| QA-09 | 100 / 5 | 100 / 5 | 100 / 5 |
| QA-10 | 100 / 5 | 100 / 5 | 100 / 5 |
| QA-11 | 0.005 / 5 | 0.005 / 5 | 0.005 / 5 |
| QA-12 | 282.268 / 2 | 282.268 / 2 | 282.268 / 2 |

All candidates meet the targets for QA-03, QA-06, QA-08, QA-09, QA-10, and QA-11. All miss the unchanged QA-12 target: the shared VoicePlaybackController applies the same detection/cancel primitive and scenario buffer drain, so this is a common product regression rather than a DP-00 discriminator. QA-08 ties because equal reconnect/restore operations execute at the candidate-specific state owner. QA-09 ties because both real delegation paths derive the same minimal scope set before the common PolicyEngine. QA-10 ties because architecture-native telemetry reconstructs the same required abstraction. QA-11 and QA-12 tie because VIA owns their shared delivery/control paths.

## Representative architecture paths

- r1_general: Input → VIA grounding → VIA agent-neutral selection → SelectedGeneralAgent → Agent workflow/execution → VIA result binding → Response
- r1_specialist: Input → VIA grounding → VIA agent-neutral selection → SpecialistAgent → Specialist execution → VIA result binding → Response
- r3_general: Input → VIA interaction boundary → Primary Agent Runtime → Primary interpretation/planning → Primary execution → VIA Task/result correlation → Response
- r3_specialist: Input → VIA → Primary Agent Runtime → Primary planning → Specialist delegation → Specialist Agent → Primary workflow/result state → VIA Task/result correlation → Response

Representative QA-01 event graph (`GOAL-001-01`): R1 emits `INPUT_ACCEPTED → TEMPORAL_EVIDENCE_READ → GROUNDED_GOAL_CREATED → SEMANTIC_DECISION_CREATED → AGENT_SELECTED → EXECUTION_STARTED → TOOL_REQUESTED/COMPLETED → RESULT_CREATED/RECEIVED/BOUND_TO_TASK → RESPONSE_CREATED → TEXT_AVAILABLE`. R3 emits `PRIMARY_RUNTIME_ADMITTED` before its runtime-owned semantic decision and otherwise reaches the same observed endpoint. Common primitive totals make both representative general cases 732.536 ms.

Representative dependency graph: an Agent onboarding request reaches `via-agent-adapter → agent-integration:agent-onboarding` for R1 and `primary-runtime-agent-integration → agent-integration:agent-onboarding` for R3; neither reaches a Core semantic zone. A product S2S-provider change reaches `component:z1 → Z1 → z1:s2s-provider-replacement` for all candidates.

## Measurement-validity mutations

These are non-campaign tests; official candidate mutation flags were all false.

| Mutation | Detected consequence |
| --- | --- |
| Break R3 specialist result correlation | QA-02 result binding and QA-03 execution/result relation fail |
| Remove R1 Task/result binding | QA-02 binding condition and QA-03 execution/result relation fail |
| Add unauthorized scope | QA-09 hard-gate trigger emitted |
| Remove trace edge | QA-10 reconstructed edge set shrinks |
| Delay feedback | QA-11 observed freshness increases by the injected delay |
| Increase cancellation buffering | QA-12 last-audible-sample delta increases |
| Duplicate recovery action | QA-08 unsafe-recovery hard gate fails |
| Add semantic dependency leak | QA-05 containment fails |
| Force Core change on Agent replacement | QA-04 forbidden-zone containment fails |
| Add a real boundary | QA-01 endpoint increases by the shared 5 ms `BOUNDARY_HOP` sample |

## Qualification

QA-07: **UNEVALUABLE / calibration missing**.

Hard qualification: **INCOMPLETE — QA-07 calibration missing**.

Interpretation: **NO DP-00 PREFERENCE FROM CURRENT QA EVIDENCE**.

All observations were derived as scenario → candidate state/events → independent adapter; evaluator oracles entered only during comparison. No Cloud/API/network calls occurred.
