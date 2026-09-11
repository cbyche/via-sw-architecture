# AA-026 — DP-00 Trade-space Synthesis and Conditional Architecture Guidance

## 1. Status

**Architecture Analysis Record — final DP-00 comparative synthesis.**

**DP-00 TRADE SPACE CHARACTERIZED — FINAL SELECTION DEFERRED PENDING DOWNSTREAM ARCHITECTURE EVIDENCE**

DP-00 has completed its comparative architecture characterization phase. This
is not `DP-00 BLOCKED`: the comparative evidence is complete. It is not
`DP-00 SELECTED`: the evidence supports legitimate, priority-dependent choices
among A/B/C/D, and the relevant product and architecture priorities have not
been fixed. Downstream Decision Points may proceed under the explicit
conditional assumptions in this record.

No overall score, implicit weighting, or unconditional winner is introduced.

## 2. Evidence identities

| Evidence | Identity | Role in this synthesis |
| --- | --- | --- |
| Runtime synthesis base branch | `exp/dp00-r1-r4-clean-rerun` at `760679b5d542827de6aa307b87256e8a03afaed6` | Integrated QA-01/02/04 and P12 history |
| Runtime evidence commit | `83cc4070decdd8a4305317cbe0bca6b0f86543dd` | Completed R1–R4 evidence |
| Runtime campaign | `dp00-r1-r4-20260911T101059Z-d4059ecc` | Clean 7.2R measurement campaign |
| Runtime measured baseline | `d4059eccd2883b3029b1e8c94fc86d092a5041f8` | Frozen architecture source |
| QA-03 measured branch | `exp/dp00-qa03-flexibility-measured` at `5e6eb627401a2e0e18736d1377ed835f9f050aed` | Completed flexibility evidence |
| QA-03 contract/control-plane | `752c6ac66fd0a77d7d6efac00d46ad7277bc5247` | Prospectively frozen QA-03 contract |
| QA-03 campaign | `dp00-qa03-20260911T091004Z-752c6ac` | Twenty measured evolution cells |
| Profile Z / Session 7.0 | Retained by the runtime lineage and AA-025 | Software-path overhead reference |
| R1–R4 profile freeze | AA-022 | Synthetic dependency-sensitivity contract |
| QA-03 contract | AA-023 | Evolution scenarios, roles, and evaluator contract |
| QA-03 measured campaign | AA-024 | Official CCR and scenario diagnoses |
| R1–R4 clean rerun | AA-025 | Authoritative QA-01/02/04 and P12 synthesis |
| Trade-space synthesis | AA-026 | Conditional guidance and final disposition |

The requirements baseline `docs/requirements/requirements-v1.1.md` remains
unchanged. Raw, derived, and report evidence is consumed as immutable evidence;
no campaign was rerun for this synthesis.

## 3. DP-00 question

> **VIA는 사용자 요청을 어디까지 직접 판단·실행하고, 어디부터 Downstream Agent에 위임할 것인가?**

The four frozen Base Architecture families remain unchanged:

- **A — Thin VIA / Agent-neutral Orchestration:** VIA owns interaction,
  context, intent, routing, and task lifecycle; substantive domain reasoning,
  planning, and tool execution remain behind the Downstream Agent boundary.
- **B — ARGO-centric Primary Execution:** ARGO owns primary substantive
  reasoning and execution; VIA is a thinner interaction/context/task-correlation
  layer; ARGO may delegate to Specialists.
- **C — Hybrid VIA Fast Path:** A-like Agent-neutral delegation plus bounded
  VIA-local execution; substantive work remains delegated.
- **D — Adaptive Per-turn Execution:** VIA chooses VIA Fast, ARGO Primary, or
  Specialist Direct for each new request; an existing-task continuation reuses
  its committed route.

The synthesis question is not “Which architecture wins overall?” It is:

> **Under which architectural and product priorities does each alternative
> become the rational choice?**

Preference is therefore a conditional function of latency and model-call
priority, correctness, flexibility, Agent neutrality, tolerance for ARGO
coupling, local-execution strategy, value of adaptive topology, and tolerance
for control-plane complexity. These dimensions are not collapsed into a
weighted score.

## 4. Evidence completeness

| Evidence track | Completion | Remaining concern |
| --- | --- | --- |
| QA-01 | Comparative evaluation complete | Synthetic sensitivity, not absolute production latency |
| QA-02 | Comparative evaluation complete | Deterministic correctness defects require remediation |
| QA-03 | Flexibility campaign complete | Representative corpus, not all future evolution |
| QA-04 | Comparative evaluation complete | Every alternative fails qualification and requires remediation |
| P12 | Common regression evidence complete | Not a current differentiator |

“Evaluation complete” does not mean “quality requirement satisfied.” The
remaining correctness and qualification work is explicit remediation evidence,
not missing comparative evidence.

## 5. QA-01 synthesis

R1–R4 are synthetic Model/Agent/Tool sensitivity profiles. They establish how
the frozen topologies react to dependency-cost regimes; they do not establish
production latency.

| Profile | A p50 / p95 ms | B p50 / p95 ms | C p50 / p95 ms | D p50 / p95 ms | Pooled p50 order |
| --- | ---: | ---: | ---: | ---: | --- |
| R1 | 172.584 / 180.135 | 115.573 / 120.132 | 174.057 / 180.143 | 173.664 / 179.644 | B < A < D < C |
| R2 | 243.041 / 250.080 | 135.260 / 140.111 | 241.033 / 250.132 | 241.950 / 249.166 | B < C < D < A |
| R3 | 162.676 / 170.127 | 133.094 / 140.098 | 162.146 / 170.125 | 160.049 / 170.121 | B < D < C < A |
| R4 | 83.425 / 90.084 | 56.752 / 60.105 | 81.862 / 89.974 | 85.442 / 169.520 | B < C < A < D |

B has the lowest p50 and p95 in every R1–R4 profile because the evaluated
paths remove one logical Model generation. Its advantage is largest in
Model-dominant R2. D gains relative to delegated alternatives in Agent-dominant
R3 by substituting local Tool execution for some Agent interaction, but its
tool-heavy R4 tail is materially worse. C's bounded local path is not present
in the P01–P03 QA-01 cohort, so no C Fast Path latency benefit is directly
measured.

Profile Z measures framework/software-path overhead only. Its CAPTURE p50/p95
milliseconds are A 0.005437/0.006625, B 0.006063/0.007025,
C 0.005459/0.006859, and D 0.005959/0.007164. The change from Profile Z to
R1–R4 demonstrates topology sensitivity to dependency costs, especially Model
generation count; it is not a production crossover claim.

Measured structural totals per profile explain the behavior:

| Alternative | Model calls | Agent interactions | Tool executions |
| --- | ---: | ---: | ---: |
| A | 384 | 128 | 0 |
| B | 208 | 112 | 0 |
| C | 384 | 112 | 16 |
| D | 384 | 80 | 80 |

These counts are architectural diagnostics, not independently scored virtues.

## 6. QA-02 synthesis

The deterministic conformance result is invariant across Profile Z and R1–R4:

| Alternative | Conformance | Deterministic failing scenarios |
| --- | ---: | --- |
| A | 83.333% | P04, P11 |
| B | 66.667% | P04, P06, P07, P11 |
| C | 83.333% | P04, P11 |
| D | 75.000% | P04, P07, P11 |

A and C tie for the strongest measured conformance. B is weakest, while D is
between B and A/C. Every listed failure repeats 16/16 and matches MINIMAL;
none is intermittent noise. No pass threshold is invented. QA-02 is both
comparative evidence and a candidate-specific remediation inventory.

## 7. QA-03 synthesis

Official CCR remains:

| Alternative | CCR |
| --- | ---: |
| A | 60% |
| B | 60% |
| C | 80% |
| D | 80% |

E1 Agent evolution, E2 Model evolution, and E5 Contract evolution are contained
for all alternatives. E3 Context Source is `NOT_CONTAINED` for all alternatives
because the shared module-registration change is unmapped. That is a common
infrastructure penalty, not an A/B/C/D differentiator, and remains in official
CCR.

E4 Capability Placement is the principal differentiator: A and B are
`NOT_CONTAINED`; C and D are `CONTAINED`. The result follows from the frozen
topology seams: C owns a bounded local-execution seam and D owns an adaptive
placement selector, whereas introducing VIA-local execution into A or B would
redefine those alternatives.

## 8. QA-04 synthesis

All alternatives **FAIL** the frozen R1–R4 qualification. The profile-invariant
relative behavior is:

| Alternative | Required | Qualified | Forbidden | Route not committed | QA-02 exclusions | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| A | 128 | 96 | 16 | 32 | 16 | FAIL |
| B | 128 | 80 | 16 | 48 | 48 | FAIL |
| C | 128 | 96 | 16 | 32 | 16 | FAIL |
| D | 128 | 96 | 16 | 0 | 32 | FAIL |

Universal failure remains visible. Comparative behavior is nevertheless
complete: D uniquely has zero route-not-committed cases; A/C qualify more cases
than B; and the QA-02 exclusion burden differs. Qualification remediation is a
separate workstream from topology selection.

## 9. P12/common invariants

P12 supplies positive common evidence rather than a discriminator. All 512/512
executions and 256/256 pairs pass:

- S1→S2 ordering;
- common parent and distinct children;
- both route commits before either execution;
- route-plan barrier preservation;
- two required effects;
- final boundary at S2;
- exact QA-02 conformance and QA-04 qualification; and
- semantic/provenance equality.

Compound-request route-plan barrier semantics are therefore a common required
behavior for every surviving architecture family.

## 10. Alternative A conditional assessment

**When favored.** A is rational when Agent-neutrality and a clean execution
boundary are mandatory; domain execution should remain entirely behind the
Downstream Agent boundary; ARGO must remain a replaceable peer; clarity and
lower control-plane complexity matter more than local placement flexibility;
and the existing v1.1 responsibility boundary is strategically preferred.

**Evidence-supported strengths.** A ties C for best QA-02 at 83.333%, has the
clearest responsibility boundary, preserves Agent ecosystem neutrality, has
less control-plane complexity than D, and aligns closely with v1.1.

**Evidence-supported weaknesses.** A gives up B's model-call and R1–R4 latency
advantage, has 60% QA-03 CCR, does not contain the E4 placement change, and
retains delegation/boundary overhead.

**Questions before preference.** Is Agent neutrality more important than local
execution flexibility? Is the measured model-call/delegation penalty acceptable
under realistic workloads? Is VIA-local execution intentionally outside product
scope?

## 11. Alternative B conditional assessment

**When favored.** B is rational when Model-call efficiency and responsiveness
dominate; ARGO is accepted as the long-lived primary execution runtime;
coupling to ARGO is strategically acceptable; the workload is predominantly
ARGO-native; and a thinner VIA is desired.

**Evidence-supported strengths.** B is fastest in every R1–R4 profile, removes
one logical Model generation on evaluated paths, benefits especially in the
Model-dominant profile, and provides the most efficient measured primary path.

**Evidence-supported weaknesses.** B has the lowest QA-02 result at 66.667%,
QA-03 CCR of 60%, no contained E4 local-placement seam, and the strongest
strategic coupling to ARGO. Specialist access can become ARGO-mediated, and
ARGO replacement can become a topology change instead of an adapter change.

**Questions before preference.** Is ARGO expected to remain an architecture
center? Is B's latency/model-call advantage worth reduced neutrality and
flexibility? Can P04/P06/P07/P11 and QA-04 be corrected without recreating a
second VIA orchestration layer?

## 12. Alternative C conditional assessment

**When favored.** C is rational when correctness and flexibility must be
balanced; substantive work should retain an Agent-neutral downstream boundary;
bounded VIA-local execution is strategically useful; moderate complexity is
acceptable; and capability placement is expected to evolve.

**Evidence-supported strengths.** C ties A for best QA-02 at 83.333%, reaches
80% QA-03 CCR, contains E4, preserves Agent-neutral delegation for substantive
work, offers a simpler ownership model than D, and provides a bounded local
extension seam.

**Evidence-supported weaknesses.** C does not match B's measured QA-01/model-call
efficiency. Its Fast Path requires strict governance; eligibility becomes an
architectural responsibility; and QA-04 still fails. The P01–P03 cohort does
not directly measure its local-path latency benefit.

**Questions before preference.** Which capabilities are legitimately local?
How stable can eligibility policy be? Can Fast Path growth be governed without
creating a second general-purpose Agent runtime?

## 13. Alternative D conditional assessment

**When favored.** D is rational when per-request adaptive topology has material
product value; workloads are highly heterogeneous; local, ARGO, and Specialist
Direct paths all need first-class status; placement flexibility matters; and
the product can bear a more complex control plane.

**Evidence-supported strengths.** D reaches 80% QA-03 CCR, contains E4, has
zero route-not-committed QA-04 cases, reduces Agent interaction through local
execution, gains under Agent-dominant R3 conditions, and natively represents
multiple topologies.

**Evidence-supported weaknesses.** D's QA-02 result is 75.000%, below A/C. The
selector owns a broad semantic responsibility; lifecycle and ownership transfer
are most complex; tool-heavy R4 behavior is weaker; and observability, retry,
cancel, and state semantics become harder.

**Questions before preference.** Does the product require per-turn topology
selection? Can selector correctness be made reliable? Can ownership transfer
remain directed and understandable? Does workload diversity justify the added
complexity?

## 14. Conditional recommendation matrix

| Priority or validated assumption | Favored candidate(s) | Evidence-grounded reason | Trade-off accepted |
| --- | --- | --- | --- |
| Minimum Model-call / latency overhead | B | Lowest p50/p95 in R1–R4; one fewer logical generation on evaluated paths | ARGO coupling; weaker QA-02 and QA-03 |
| Agent neutrality and clean delegation | A or C | VIA retains Agent-neutral orchestration; substantive work stays delegated | More Model/delegation overhead than B |
| Correctness plus flexibility balance | C | QA-02 83.333% and QA-03 80% | Slower measured cohort than B; Fast Path governance |
| Capability-placement flexibility | C or D | E4 is contained only for C/D | More VIA execution/control responsibility |
| Per-turn heterogeneous topology is mandatory | D | Explicit Fast/ARGO/Specialist topology selection and zero route gaps | Highest selector/lifecycle complexity |
| Lowest control-plane complexity | A | Simplest explicit execution-authority model | Least local/adaptive flexibility |
| Strategic ARGO centralization | B | Primary-execution topology matches that strategy | Loss of Agent-neutral center and replacement ease |
| Bounded local execution without adaptive topology | C | Local seam is contained while substantive work stays delegated | Eligibility boundary must remain narrow |

No global ranking follows from this matrix. Under Model-call dominance B is the
relevant leader. Under Agent-neutrality plus measured correctness A/C form the
relevant family, with C favored only if placement flexibility is valued. If
adaptive topology is mandatory, D is the relevant family. Each statement is
conditional on an explicit priority.

## 15. Stable conclusions frozen by DP-00

- A/B/C/D are the validated primary topology families for subsequent work.
- ARGO-default routing under Thin VIA is not Alternative B; B relocates primary
  substantive execution authority.
- Fast Path eligibility is semantic, not an arbitrary timing or tool-count rule.
- Capability-placement policy is a major architecture discriminator.
- Per-turn topology selection creates a first-class execution-path decision
  responsibility.
- Execution ownership and transfer must remain explicit; unrestricted ownership
  bouncing is unacceptable.
- P12 route-plan barrier semantics are common mandatory behavior.
- Logical Model-call count materially affects responsiveness when dependency
  cost is nontrivial.
- QA-02 and QA-04 deterministic defects remain visible until remediated.
- Comparative evidence must remain a vector; local ownership, coupling, and
  call classes are not intrinsically good or bad without product priorities.

## 16. Intentionally unresolved choices

- final A/B/C/D selection;
- exact VIA-local capability set;
- exact Execution Path Selector design;
- exact Agent routing mechanism;
- exact intent-refinement decomposition;
- exact ARGO-versus-Specialist selection policy;
- exact capability-contract shape beyond frozen requirements; and
- production Model, Agent, and Tool latency distributions.

These are deliberate downstream questions, not defects in the completed
comparative campaign.

## 17. Conditional downstream DP roadmap

### If A remains favored

Prioritize DP-11 Downstream Agent routing architecture, DP-12 Downstream Agent
capability contract, then DP-04 generic versus specialized Agent integration.
A's value depends on a clean, scalable, Agent-neutral routing/delegation seam.
DP-10, DP-09, and DP-05 follow once routing and capability boundaries are firm.

### If B remains favored

Prioritize DP-10 Intent refinement, DP-11 routing/delegation from the
ARGO-centric runtime, and DP-07 Model gateway selection. Evaluate ARGO
task/thread authority, Specialist delegation, and VIA projection/correlation at
the same time. B already constrains primary execution state toward ARGO and VIA
toward user-facing projection; downstream work must test whether that boundary
remains coherent rather than rebuilding Thin VIA semantics.

### If C remains favored

Prioritize DP-03 Capability placement, then DP-12 capability contract, DP-11
Agent routing, and DP-10 intent refinement. The decisive question is which
capabilities may legitimately execute in VIA without making Fast Path a second
general-purpose Agent runtime. DP-03 evidence should explicitly strengthen or
weaken C.

### If D remains favored

Prioritize DP-03 Capability placement, DP-09 existing-task versus new-task
association, DP-10 intent refinement, and DP-11 topology/routing selection;
bring DP-12 forward when ownership-transfer contracts become concrete. Test
when local, ARGO Primary, and Specialist Direct are valid; when an existing
route is reused; and how ownership is transferred safely.

### Cross-candidate dependency value

| DP | Value regardless of final candidate | Discriminating value |
| --- | --- | --- |
| DP-02 Interaction context representation | Defines portable context/provenance crossing any execution boundary | Reveals whether B/D authority transfer needs materially different context ownership |
| DP-09 Task association | Protects continuation and route reuse for every family | Strongly tests D's adaptive lifecycle consequences |
| DP-12 Capability contract | Provides capability identity and compatibility across all routes | Tests A/C neutrality, B-mediated delegation, and D selector inputs |
| DP-10 Intent refinement | Defines how much semantic authority exists before execution | Tests whether B can stay thin and D can avoid duplicated decisions |
| DP-11 Agent routing | Makes dispatch authority and escalation explicit | Tests B centralization and D selector complexity against A/C neutrality |

These DPs are not all immediate. Their order follows the conditional family and
the information needed to reduce DP-00 uncertainty.

## 18. Highest-information-gain next DP

**Recommend DP-03 — Capability placement boundary as the immediate next
architecture investigation.**

DP-03 most directly separates the remaining topology families because E4 is
already the principal measured QA-03 discriminator: A/B could not contain a
VIA-local placement change, while C/D could. DP-03 can determine whether there
is a stable, strategically valuable local capability set. If there is none, A/B
strengthen and C/D lose a defining benefit. If a narrow stable set exists, C
strengthens. If placement must vary materially per request, D strengthens and
its selector/lifecycle burden becomes testable. This provides more immediate
information gain than refining a routing mechanism before the placement policy
is known.

## 19. QA remediation roadmap

### QA-02 remediation

| Candidate | Deterministic failures to preserve and correct |
| --- | --- |
| A | P04, P11 |
| B | P04, P06, P07, P11 |
| C | P04, P11 |
| D | P04, P07, P11 |

Remediation should be implemented conditionally for candidates still under
consideration and regression-tested against P12 and frozen topology boundaries.
B remediation must not reconstruct a second VIA orchestration authority; D
remediation must not erase its explicit selector responsibility.

### QA-04 remediation

Every candidate must remove forbidden route behavior and address its
qualification gaps. A/C have 32 route-not-committed cases and 16 QA-02
exclusions; B has 48 and 48; D has zero and 32. The universal `FAIL` remains the
qualification baseline until measured remediation passes. These defects do not
invalidate the comparative evidence; a candidate is eliminated only if repair
would violate its defining boundary or cause unacceptable architecture
consequences.

## 20. Real-stack validation role

Topology sensitivity is established; absolute production performance is not.
Future real-stack validation may shift conditional preference:

- B strengthens if realistic Model generation dominates UX and its one-call
  advantage persists; it weakens if ARGO/Specialist mediation costs or quality
  losses dominate.
- A strengthens if delegation overhead is modest relative to neutrality and
  replaceability value; it weakens if boundary/model overhead dominates common
  interactions.
- C strengthens if a meaningful bounded local cohort produces material gains
  without broad eligibility cost; it weakens if the cohort is rare or costly to
  govern.
- D strengthens if heterogeneous workloads gain materially from local/direct
  paths; it weakens if selector overhead, Tool-heavy tails, or ownership-transfer
  failures dominate.

Production Model/Agent/Tool distributions should update the relevant condition,
not retroactively convert synthetic R1–R4 into production claims.

## 21. Candidate retirement criteria

An alternative may be retired only when at least one of these conditions is
supported by downstream evidence or explicit product strategy:

- its defining topology cannot satisfy a mandatory product requirement;
- required remediation would violate its defining architecture boundary;
- it becomes dominated on a newly validated, decision-relevant dimension with
  no compensating required structural value; or
- owner/product strategy explicitly rejects its required coupling, neutrality,
  placement, or complexity model.

One better metric, universal QA-04 failure, or a nonselected conditional
priority is insufficient by itself to retire a candidate.

## 22. DP-00 revisit triggers

- **Toward A:** Agent-neutrality or ecosystem replaceability becomes mandatory;
  DP-03 shows local placement has low value or excessive risk; simplicity is
  preferred over marginal latency.
- **Toward B:** realistic validation confirms Model-call latency dominates UX;
  ARGO centralization is accepted; B's correctness/qualification defects are
  repaired without recreating Thin VIA routing.
- **Toward C:** DP-03 produces a stable bounded local-capability policy; Fast
  Path governance stays simple; correctness remains strong after remediation.
- **Toward D:** heterogeneous workloads demonstrate material value from direct
  topology selection; selector correctness is proven; DP-09 shows lifecycle and
  ownership transfer remain manageable.
- **Away from any candidate:** a mandatory requirement cannot be satisfied, or
  remediation fundamentally contradicts the frozen alternative definition.

DP-00 should be revisited when one or more triggers become concrete enough to
change the conditional matrix, not merely because another downstream DP has
started.

## 23. Final disposition

**DP-00 TRADE SPACE CHARACTERIZED — FINAL SELECTION DEFERRED PENDING DOWNSTREAM ARCHITECTURE EVIDENCE**

The comparative architecture characterization phase is complete. A/B/C/D
remain viable under different explicit priorities; no evidence justifies an
unconditional winner or candidate retirement. Downstream architecture may
proceed under the conditional roadmaps above, beginning with DP-03 for highest
information gain. DP-00 will be revisited when capability placement, routing,
intent, lifecycle, realistic latency, or explicit product strategy makes the
decision conditions sufficiently concrete.
