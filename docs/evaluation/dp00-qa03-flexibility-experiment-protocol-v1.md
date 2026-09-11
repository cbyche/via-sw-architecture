# DP-00 QA-03 Flexibility Experiment Protocol v1

## Status and identity

**Finalized prospective experiment contract; measured campaign not executed.**

| Item | Frozen value |
| --- | --- |
| Branch | `exp/dp00-qa03-flexibility` |
| Source SHA | `d4059eccd2883b3029b1e8c94fc86d092a5041f8` |
| Protocol | `dp00-qa03-flexibility-protocol-v1` |
| Scenario catalog | `dp00-qa03-evolution-catalog-v1` |
| Role map | `dp00-qa03-role-map-v1` |
| Analyzer | `dp00-qa03-analyzer-v1` |
| Baseline signature | `dp00-qa03-baseline-regression-v1` |
| Manifest schema | `dp00-qa03-execution-manifest-v1` |

The initial local QA03-1 freeze was clarified before push and before any measured run; the five scenarios, Expected Change Areas, and role map did not change, so the clarification finalizes v1 rather than creating a new catalog or map. After this clarification commit, a measured v1 campaign must use the contract unchanged. A later contract defect is preserved, documented, and corrected prospectively as v2. Measured evidence is never reinterpreted by rewriting v1.

This track is independent from the Session 7.x runtime campaign. Runtime latency, Profile Z, and R1-R4 timings are not QA-03 evidence. The Session 7.1 profiles remain unchanged and serve only as regression-protected repository contracts.

## Authoritative QA definition

`QA-03` means current vNext **Flexibility / Change Containment**. Historical conversation/task recovery is named **Legacy v1.1 QA-03** wherever ambiguity is possible.

The experiment asks:

> When an expected product evolution occurs, does the required architectural change remain confined to the roles/components explicitly designed to absorb that change, without propagating into unrelated architecture areas?

The Primary Metric is Change Containment Ratio (CCR):

```text
CCR (%) = CONTAINED valid evaluated scenarios
          / total valid evaluated scenarios
          * 100
```

CCR is calculated separately for A, B, C, and D over one common comparative set, `S_primary`. A scenario enters `S_primary` only when A/B/C/D each have exactly one valid, complete, comparable result (`CONTAINED` or `NOT_CONTAINED`). Thus:

```text
CCR_alt = CONTAINED cells for alt within S_primary / |S_primary| * 100
```

A/B/C/D always have the same denominator. Every eligible scenario has weight 1. Category results are diagnostic. There is no combined architecture score, owner-approved weighting, or F1-F5 mapping in the repository; therefore **QA-03 numeric grading thresholds remain unfrozen; this experiment reports CCR directly.**

## Frozen taxonomy and coverage

The authoritative E1-E5 taxonomy is preserved without additions.

| Category | Evolution kind | Frozen scenario |
| --- | --- | --- |
| E1 | Agent evolution | `QA03-E1-SPECIALIST-AGENT-001` |
| E2 | Voice / Model evolution | `QA03-E2-MODEL-PROVIDER-001` |
| E3 | Context / Input evolution | `QA03-E3-CONTEXT-SOURCE-001` |
| E4 | Capability-placement evolution | `QA03-E4-CAPABILITY-PLACEMENT-001` |
| E5 | Contract evolution | `QA03-E5-CONTRACT-METADATA-001` |

The normative change request, rationale, alternative-neutral semantics, acceptance criteria, regression criteria, Expected Change Area, forbidden roles, and evidence list are in `benchmark/contracts/dp00-qa03-evolution-catalog-v1.json`. Five scenarios are intentionally used: one strong scenario per category avoids an artificial matrix while preserving complete taxonomy coverage.

## Frozen architecture-role vocabulary

The normalized vocabulary is:

- interaction ingress;
- context handling, Context Source adapter, and context representation contract;
- intent refinement;
- route decision;
- execution-policy/control plane;
- local capability execution;
- Model Gateway contract, provider, and provider configuration;
- Agent delegation adapter;
- Downstream Agent capability contract and capability registration;
- task/orchestration state;
- Voice Runtime;
- alternative orchestration core;
- observability/evaluation.

The exact identifiers, paths, multi-role candidates, prospective extension wildcards, and explicitly absent roles are normative in `benchmark/contracts/dp00-qa03-role-map-v1.json`.

Physical topology is not normalized by pretending that A/B/C/D have identical files. For example, A/C separate Agent routing from intent refinement, B combines primary interpretation/routing/delegation in `argo_primary_controller.rs`, C has deterministic fast eligibility, and D has a per-turn execution-path selector. A multi-role file remains evidence of co-location: its changed symbol/hunk is reviewed against only the prospectively listed candidate roles. Review cannot invent a new role or path after the diff is known.

### Alternative summaries

| Alternative | Normalized implementation shape |
| --- | --- |
| A — Thin VIA | context engine; intent refiner; Agent router/capability lookup; Agent harness; task manager; orchestration core. Local capability execution and placement policy are absent by definition. |
| B — ARGO-centric | thin context packager; combined ARGO primary interpretation/route/delegation controller; task projection; orchestration core. VIA-local execution is absent by definition. |
| C — Hybrid Fast Path | A-like context/intent/Agent seams plus fast eligibility/control plane, local executor, task manager, and orchestration core. |
| D — Adaptive Per-turn | context/intent seams, combined route/placement selector, local executor, Agent client, task manager, and orchestration core. |

Voice Runtime is outside all four executable prototype crates and is a held-common ingress dependency. Its absence is explicit rather than silently mapped to another role.

## Prospective Expected Change Areas

Expected Change Areas are scenario-level role sets, frozen in the catalog before any measured implementation. They are resolved through the frozen A/B/C/D map.

E4 evaluates flexibility of the capability-placement boundary; it does not require every alternative to become Hybrid VIA Fast Path. If an alternative implements the bounded placement evolution without redefinition, inside the Expected Change Area, with acceptance and regression passing, the cell is `CONTAINED`. If the request remains meaningful but requires out-of-area propagation, regression degradation, or architecture-boundary change beyond the scenario's allowance, the valid cell is `NOT_CONTAINED`. Thin VIA or another resistant boundary is not by itself experiment invalidity. E4 is `INVALID_EXPERIMENT` only when comparative semantics/applicability or experiment-contract integrity is broken. An implementer who silently converts A/B into C/D to conduct or pass E4 creates an invalid experiment; an unchanged A/B definition that cannot contain the valid evolution produces architecture evidence.

No implementation result may broaden an Expected Change Area. A legitimate correction requires v2 and a complete rerun for all alternatives.

## Changed-file to architecture-role procedure

For every result commit:

1. Diff the result against the exact baseline with rename detection and collect status, paths, and line diagnostics.
2. Assign each path one `change_class` from the manifest schema.
3. Resolve architecture-affecting paths against all frozen shared and alternative-specific patterns.
4. For a single-role match, assign that role. For a multi-role file, inspect the changed symbol/hunk and assert only roles from the frozen candidate set; preserve review evidence.
5. Treat an architecture-affecting path with no frozen match as `UNMAPPED_ARCHITECTURE_CHANGE`, which is outside the Expected Change Area.
6. Compare the resolved roles with the scenario's Expected Change Area. Any extra role is unexpected propagation, whether or not it is explicitly listed as forbidden.
7. Preserve the complete structured change list and secondary counts. `git diff --name-only` alone is insufficient.

The analyzer rejects an asserted role that is not a frozen candidate for that path. Ambiguous classification produces `INCONCLUSIVE`, not a favorable guess.

### What counts

| Change type | Primary containment treatment |
| --- | --- |
| Production source | Counts. |
| Contract/schema | Counts. |
| Runtime configuration | Counts. |
| Capability/provider registration | Counts. |
| Generator source | Counts. |
| Generated output | Does not count independently; its generator source must count. |
| Tests | Non-scoring evidence. A production change exposed by a test still counts. |
| Documentation and result evidence | Non-scoring. |
| Fixture-only data | Non-scoring only when it does not implement registration, policy, contract, or production behavior. Otherwise classify by that architecture function. |
| Formatting-only | Non-scoring only after semantic-zero review. A mixed semantic/formatting edit counts in full by role. |
| Benchmark-only instrumentation | Non-scoring when it only observes. A change that alters scenario semantics, acceptance, regression, mapping, or classification invalidates the experiment. |

Files, components, roles touched, LOC, contract surfaces, and propagation patterns are Secondary diagnostics. They never replace the binary result or imply that fewer lines are inherently better.

## Exact classification semantics

### `CONTAINED`

All of the following are true:

1. every frozen acceptance test passes;
2. baseline-relative regression introduces no new or worsened required failure;
3. every required architecture-affecting implementation/contract change is inside the frozen Expected Change Area.

### `NOT_CONTAINED`

The scenario is a valid, architecture-neutral expected evolution, is meaningful for the alternative, and ran against the correct frozen contract, but at least one of acceptance, regression, or containment fails. This includes architecture resistance: if supporting a valid evolution requires changes outside the Expected Change Area or an impermissible boundary change, the result is `NOT_CONTAINED`. There is no partial Primary credit.

### `INVALID_EXPERIMENT`

The attempt cannot enter the denominator because experiment validity failed: wrong baseline/version, changed prospective contract, contaminated workspace, unequal scenario semantics, silent alternative redefinition, non-independent execution, source/provenance violation, evaluator/role-map drift, or a scenario that is not meaningfully applicable under the frozen comparative definition. The reason must be retained. "Alternative redefinition" means the evaluator or implementer silently changed the frozen alternative to conduct or pass the scenario; it does not mean that a valid evolution merely pressures the alternative's boundary. An unfavorable valid outcome may never be relabeled invalid.

### `INCONCLUSIVE`

The protocol was not shown invalid, but required evidence is unavailable or ambiguous—for example interrupted tests, incomplete diff, or unresolved multi-role classification. First attempt resolution without changing production implementation, Expected Change Area, scenario semantics, baseline, acceptance, or regression criteria. It is neither success nor failure and cannot create an asymmetric denominator.

Only `CONTAINED` and `NOT_CONTAINED` are valid evaluated cells. If one cell is `INVALID_EXPERIMENT` or remains `INCONCLUSIVE`, the entire scenario is excluded across A/B/C/D from `S_primary`; alternatively, stop and correct a protocol defect prospectively under a new version. Never retain the other three cells only for Primary CCR. The report retains every attempted cell, identifies invalid/inconclusive cells and reasons, records scenario-wide exclusion, and states the resulting common denominator. Secondary per-cell/category diagnostics retain the full history.

The preferred matrix is 5 scenarios x 4 alternatives = 20 valid cells and a common denominator of 5. Because v1 has exactly one scenario in each E1-E5 category, removing any scenario loses a taxonomy category and normally requires:

`QA-03 COMPARATIVE CAMPAIGN INCOMPLETE — TAXONOMY COVERAGE LOST`

The campaign must not claim unconditional completion unless all five categories remain validly comparable. A replacement requires a prospectively approved new protocol/catalog version and consistent rerun; it is not patched into measured v1 evidence.

## Acceptance and regression protocol

Acceptance tests prove the evolution itself and use stable IDs from the catalog. They are authored/materialized before production implementation in each disposable scenario workspace. The same behavior and deterministic dependencies apply to A/B/C/D; topology-specific observations may differ only where the frozen acceptance criterion permits the architecture-defined route.

Regression comprises:

- full existing Python and Rust unit/integration tests;
- DP-00 P01-P12 behavior;
- QA-02 Required/Allowed/Forbidden evidence;
- QA-04 route-contract evidence;
- P12 compound route-plan barrier;
- A/B/C/D semantics and primary ownership;
- Profile Z and R1-R4 frozen contract invariance.

QA-01 latency ranking is not run or interpreted as flexibility evidence. QA-02 and QA-04 retain their existing semantics.

### Baseline defect handling

The frozen baseline signature records P04/P11 failures for A/C, P04/P06/P07/P11 for B, and P04/P07/P11 for D, plus the decomposed QA-04 qualification failures. These remain visible as pre-existing defects.

A scenario regression passes when its required failure signature is not worse than that alternative's baseline. It fails if it introduces a new failure, turns a previously passing requirement into a failure, adds a new forbidden effect/missing required observation to an existing defect, breaks P12, or newly disqualifies a required QA-04 observation. A scenario is not required to repair an unrelated pre-existing defect. An apparent repair is diagnostic and must be checked for semantic drift; it does not offset a new failure.

## Isolation and execution protocol

The measured campaign must not run on this contract-freeze branch. It uses a dedicated Git worktree not shared with the Session 7.2 runtime campaign, another Codex session, or a main working directory actively switched by another process. Each Alternative x Scenario uses one disposable worktree and unique branch created directly from source SHA `d4059e...`, with a collision-checked name such as `exp/qa03-v1/<scenario-id>/<alternative>`. A checkout in another worktree must not be able to change QA03-2's checked-out source.

For every one of the 20 cells:

1. verify and record dedicated worktree path, branch, exact HEAD SHA, and clean status before every campaign stage;
2. create a new branch/worktree from the exact source SHA—never from another scenario result;
3. copy/reference the committed frozen v1 contract without modifying it;
4. materialize the frozen acceptance test first;
5. give Codex the exact scenario request and instruction: **make the minimum architecture-consistent change needed; follow the alternative's natural extension mechanism; do not optimize for CCR or redesign the alternative**;
6. run acceptance and regression and capture baseline-relative evidence;
7. commit the attempted result on its unique branch so the diff is immutable;
8. produce a schema-valid execution manifest and run the frozen analyzer twice;
9. archive raw evidence under `results/raw/`, derived classification under `results/derived/`, and reports under `results/reports/` in the future campaign branch;
10. remove/discard only the disposable worktree after evidence is safely recorded; never reset or reuse it.

No A result enters B/C/D, and no E1 result becomes an E2-E5 baseline. Worktree/branch provenance, baseline SHA, result SHA, and contract versions are mandatory manifest fields. A branch collision or dirty baseline is `INVALID_EXPERIMENT`, not grounds for cleanup by reset.

## Human/Codex bias control

The scenario request, acceptance tests, allowed behavior, role map, and Expected Change Area are frozen before implementation. Implementation order should be counterbalanced or rotated across scenarios so growing repository familiarity does not consistently favor one alternative. Each alternative receives the same request and minimum-change instruction. Manual architecture redesign for one alternative and mechanical extension for another is prohibited. Reviewers judge natural architecture consistency, not whether Codex was explicitly coached to touch fewer roles.

## Evidence and manifest

Each raw manifest conforms to `benchmark/schemas/dp00-qa03-execution-manifest-schema-v1.json` and records identity, isolation, contract versions, complete change entries, hunk role assertions, acceptance evidence, baseline and observed regression signatures, and notes. The analyzer returns classification, reasons, changed roles, unexpected roles, and details sufficient to recompute CCR.

Raw manifests and test logs are immutable. Derived CCR can be recomputed. No final A/B/C/D CCR is produced in QA03-1.

## Evaluator qualification

Synthetic tests—not measured scenario implementations—must prove:

1. in-area production change -> `CONTAINED`;
2. out-of-area and unmapped changes -> `NOT_CONTAINED`;
3. acceptance failure -> `NOT_CONTAINED`;
4. new/worsened regression -> `NOT_CONTAINED`;
5. unchanged baseline QA-02/QA-04 failure -> no false regression;
6. test-only change -> non-scoring;
7. all A/B/C/D mappings resolve and absent roles are explicit;
8. wrong baseline/isolation -> `INVALID_EXPERIMENT`;
9. incomplete/ambiguous evidence -> `INCONCLUSIVE`;
10. repeated classification and CCR calculation are deterministic.
11. one invalid/inconclusive cell excludes its scenario symmetrically and preserves one denominator;
12. losing one v1 scenario blocks completion because E1-E5 coverage is lost;
13. a normal 20-cell matrix yields denominator 5 for A/B/C/D.

## Limitations

QA-03 measures containment for five expected evolution requests against the current executable DP-00 prototypes. It does not measure coding speed, developer effort, general maintainability, total design quality, runtime latency, production dependency fidelity, or the cost of unexpected categories of change. Binary CCR hides severity differences, so propagation depth and touched roles remain diagnostics. The prototype's Voice Runtime is external to the executable crates, and E2 samples Model rather than Voice evolution.

## Freeze and decision state

When all protocol qualification checks pass, the disposition is:

`QA-03 EXPERIMENT CONTRACT FINALIZED — READY FOR FLEXIBILITY CAMPAIGN`

This record prepares evidence only. DP-00 remains:

`NOT READY FOR ARCHITECTURE DECISION`
