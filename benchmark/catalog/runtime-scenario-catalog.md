# Runtime Scoring Scenario Catalog

## Status

**Initial Catalog Skeleton — Pre-Pilot / not the final scoring corpus**

This catalog seeds the DP-00 Runtime Episode Benchmark taxonomy and initial scenarios.

It is intentionally distinct from the executable-architecture smoke walkthroughs `S1~S5` in `AA-006`.

```text
S1~S5
  = architecture consistency / smoke cases

R1~R10
  = runtime scoring benchmark taxonomy
```

Scenario counts, class weighting, QA-04 aggregation, no-route-commit treatment and final corpus composition remain TBD until Pilot/calibration.

Authoritative scenario schema:

`benchmark/schemas/runtime-scenario-schema.md`

---

# 1. Runtime taxonomy

| Class | Purpose | Typical architecture sensitivity |
| --- | --- | --- |
| **R1 Local / Bounded** | Exercise bounded/local execution ownership and Fast Path placement. | `FAST_PATH`, `ARGO_PRIMARY` |
| **R2 General Agent** | Exercise general-purpose ARGO/Agent execution path. | `ARGO_PRIMARY`, `SPECIALIST_ROUTING` |
| **R3 Specialized Agent** | Exercise direct vs delegated specialist routing. | `SPECIALIST_ROUTING`, `ARGO_PRIMARY` |
| **R4 Context / Referent** | Exercise screen/file/pointer grounding without leaking ground truth. | `REFERENT`, `MODEL_VALIDATION` |
| **R5 Existing-task Follow-up** | Exercise task association and previous-route reuse. | `FOLLOW_UP`, `RESULT_BINDING` |
| **R6 Ambiguity / Clarification** | Exercise clarification correctness and deterministic user reply branch. | `CLARIFICATION`, `REFERENT` |
| **R7 Execution-path Trap** | Exercise false-fast/wrong-route semantic proposals and validation. | `FAST_PATH`, `MODEL_VALIDATION`, `ERROR_RECOVERY` |
| **R8 Concurrent Task / Result** | Exercise interleaved task/result identity and binding. | `CONCURRENCY`, `RESULT_BINDING` |
| **R9 Compound Request** | Exercise decomposition and multiple execution paths in one user goal. | `COMPOUND`, `FAST_PATH`, `SPECIALIST_ROUTING` |
| **R10 Dependency / Model Error** | Exercise malformed/timeout/wrong semantic dependency behavior and recovery. | `ERROR_RECOVERY`, `MODEL_VALIDATION` |

Architecture-sensitivity tags are **coverage metadata, not scoring answers**.

---

# 2. QA eligibility rules

General eligibility:

```text
QA-01
  -> Fast-task semantics
  -> Voice fixture
  -> ground-truth acoustic EOS
  -> externally observable useful outcome

QA-02
  -> versioned Required/Allowed/Forbidden constraint manifest

QA-04
  -> representative initial execution-routing episode
  -> route_commit_expectation REQUIRED/FORBIDDEN/OPTIONAL declared
  -> final no-route-commit aggregation treatment TBD
```

The three QA populations do not have to match exactly.

---

# 3. Seed scenario list

## R1 — Local / Bounded

### R1-001 — Volume decrease

User goal:

```text
"볼륨 조금 줄여줘."
```

Purpose:

- bounded local capability;
- compare delegated/ARGO path vs VIA-owned local path;
- verify common tool capability across all alternatives.

Tags:

```text
FAST_PATH
ARGO_PRIMARY
```

Initial QA eligibility:

```text
QA-01 = yes, with Voice fixture
QA-02 = yes
QA-04 = yes
```

### R1-002 — Pause media

User goal:

```text
"음악 잠깐 멈춰줘."
```

Tags:

```text
FAST_PATH
```

Initial QA eligibility:

```text
QA-01 = yes, with Voice fixture
QA-02 = yes
QA-04 = yes
```

---

## R2 — General Agent

### R2-001 — Organize Downloads requiring planning

```text
"다운로드 폴더를 정리해줘."
```

Fixture semantics explicitly require judgment over multiple files and a plan; not a bounded Fast Path case.

Tags:

```text
ARGO_PRIMARY
MODEL_VALIDATION
```

Initial QA eligibility:

```text
QA-01 = normally no unless a separately bounded short version is defined
QA-02 = yes
QA-04 = yes
```

### R2-002 — Short general-Agent task

A bounded general-purpose Agent request with deterministic short domain-executor behavior.

Purpose: preserve a QA-01 eligible Agent path without conflating it with long task duration.

Tags:

```text
ARGO_PRIMARY
```

Initial QA eligibility:

```text
QA-01 = yes when Voice fixture and bounded outcome are defined
QA-02 = yes
QA-04 = yes
```

---

## R3 — Specialized Agent

### R3-001 — Diagnose Wi-Fi via NetworkAgent

```text
"현재 Wi-Fi 문제를 진단해줘."
```

Allowed correctness routes may include:

```text
EXECUTOR_DIRECT(initial=NetworkAgent)
EXECUTOR_DELEGATED(initial=ARGO, final=NetworkAgent)
```

subject to scenario constraints.

Tags:

```text
SPECIALIST_ROUTING
ARGO_PRIMARY
```

Eligibility:

```text
QA-01 = no by default; may have a separate short diagnostic fixture
QA-02 = yes
QA-04 = yes
```

### R3-002 — Specialized application/domain request

Synthetic task that clearly belongs to one specialized Agent but remains architecture-neutral between direct and delegated route shapes.

Tags:

```text
SPECIALIST_ROUTING
```

Eligibility:

```text
QA-01 = optional depending bounded Voice fixture
QA-02 = yes
QA-04 = yes
```

---

## R4 — Context / Referent

### R4-001 — Open pointed document

User points to one of several visible documents and asks to open it.

AUT-visible context contains raw object/pointer evidence only.

Evaluator oracle contains correct referent.

Tags:

```text
REFERENT
```

Eligibility:

```text
QA-01 = yes if Voice Fast-task fixture is bounded
QA-02 = yes
QA-04 = yes
```

### R4-002 — file_A to corrected folder_C referent

User initially references one destination and self-corrects to another within the same episode/turn evidence.

Purpose: exercise temporal/multiple referent grounding without pre-resolved benchmark labels.

Tags:

```text
REFERENT
MODEL_VALIDATION
```

Eligibility:

```text
QA-01 = no by default
QA-02 = yes
QA-04 = yes
```

---

## R5 — Existing-task Follow-up

### R5-001 — Wi-Fi T1 follow-up

Precondition:

```text
T1 = diagnose_wifi
previous executor/route exists
```

Follow-up:

```text
"그럼 DNS도 확인해봐."
```

Purpose: clear continuation and previous-route reuse.

Tags:

```text
FOLLOW_UP
RESULT_BINDING
```

Eligibility:

```text
QA-01 = no by default
QA-02 = yes
QA-04 = yes
```

### R5-002 — Follow-up while unrelated T2 is active

Precondition:

```text
T1 = Wi-Fi diagnosis, WAITING_USER
T2 = document summary, RUNNING
```

User issues a continuation that should bind to T1.

Tags:

```text
FOLLOW_UP
CONCURRENCY
RESULT_BINDING
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = yes
```

---

## R6 — Ambiguity / Clarification

### R6-001 — Ambiguous document

Initial:

```text
"그 문서 열어줘."
```

Two documents are equally plausible.

Runner provides deterministic reply only after `clarification.requested`:

```text
"오른쪽에 있는 거."
```

Tags:

```text
CLARIFICATION
REFERENT
```

Eligibility:

```text
QA-01 = no by default
QA-02 = yes
QA-04 = yes
```

### R6-002 — Ambiguous contact

Request references one of two contacts with the same/similar display name.

Deterministic user reply resolves the intended contact only when clarification is requested.

Tags:

```text
CLARIFICATION
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = yes
```

---

## R7 — Execution-path Trap

### R7-001 — Local-looking request requiring domain planning

The wording superficially resembles a bounded local action, but scenario semantics require planning or external Agent state.

Semantic Behavior Plan includes a wrong Fast/local proposal for the relevant route responsibility.

Tags:

```text
FAST_PATH
MODEL_VALIDATION
ERROR_RECOVERY
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = yes
```

### R7-002 — Complex multi-file request with wrong Fast proposal

A multi-file operation that violates bounded Fast Path semantics while replay proposes local execution.

Tags:

```text
FAST_PATH
MODEL_VALIDATION
ERROR_RECOVERY
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = yes
```

---

## R8 — Concurrent Task / Result

### R8-001 — Interleaved T1/T2 progress/result

Two active tasks emit deterministic progress/results in interleaved order.

Purpose: verify result/progress binding independent of timing order.

Tags:

```text
CONCURRENCY
RESULT_BINDING
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = optional for the request that initiates/continues a route
```

Pilot-v0.2 realization P11 is a result-delivery-only episode: both results
already exist, so a new domain route is not authorized and its predeclared
route contract is `FORBIDDEN`. This is distinct from an R8 request that starts
or continues execution, for which QA-04 may be `OPTIONAL`.

### R8-002 — Out-of-order completion

T2 completes before T1 despite being initiated later; results must remain bound to correct task/user goal.

Tags:

```text
CONCURRENCY
RESULT_BINDING
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = optional
```

---

## R9 — Compound Request

### R9-001 — Local action + Agent task in one utterance

Example shape:

```text
"음악 멈추고 다운로드 폴더도 정리해줘."
```

Requires decomposition into a bounded local-capable subgoal and a planning-required Agent subgoal while preserving one user-goal episode.

Tags:

```text
COMPOUND
FAST_PATH
ARGO_PRIMARY
```

Eligibility:

```text
QA-01 = no for overall episode Primary
QA-02 = yes
QA-04 = yes under the future frozen compound-route accounting rule
```

Pilot-v0.2 realization P12 declares `REQUIRED`: every domain-execution subgoal
must have a committed route, and the QA-04 boundary is the final required
subgoal route commit. Correctness-qualified comparability is defined separately
by `qa04-route-contract-policy-v1`.

### R9-002 — Multi-domain compound request

Two domain subgoals requiring different specialized/general execution capabilities.

Tags:

```text
COMPOUND
SPECIALIST_ROUTING
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = yes, accounting semantics to be frozen before final scoring
```

---

## R10 — Dependency / Model Error

### R10-001 — Malformed structured model output

Semantic Behavior Plan:

```text
attempt 1 = MALFORMED
```

Optional attempt 2 can be `CORRECT` if the architecture initiates a logical retry.

Tags:

```text
ERROR_RECOVERY
MODEL_VALIDATION
```

Eligibility:

```text
QA-01 = no by default
QA-02 = yes
QA-04 = yes
```

### R10-002 — Wrong Agent / route proposal

Frozen semantic condition contains a valid intent plus wrong route/Agent candidate.

Purpose: compare deterministic validation, clarification/fallback, or retry structure under identical model error.

Tags:

```text
ERROR_RECOVERY
MODEL_VALIDATION
SPECIALIST_ROUTING
```

Eligibility:

```text
QA-01 = no
QA-02 = yes
QA-04 = yes
```

### R10-003 — Model timeout / no response

Behavior plan includes `TIMEOUT` or `NO_RESPONSE` for a route-relevant semantic operation.

Tags:

```text
ERROR_RECOVERY
```

Eligibility:

```text
QA-01 = no by default
QA-02 = yes
QA-04 = yes; terminal no-route-commit aggregation remains TBD
```

---

# 4. Coverage matrix

Initial qualitative coverage only; not a scoring weight.

| Class | Fast/Local | ARGO | Specialist | Referent | Follow-up | Clarify | Concurrency | Error | Compound |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| R1 | X | X |  |  |  |  |  |  |  |
| R2 |  | X |  |  |  |  |  |  |  |
| R3 |  | X | X |  |  |  |  |  |  |
| R4 |  |  |  | X |  |  |  |  |  |
| R5 |  | X | X |  | X |  | X |  |  |
| R6 | X |  |  | X |  | X |  |  |  |
| R7 | X | X | X |  |  |  |  | X |  |
| R8 |  |  |  |  | X |  | X |  |  |
| R9 | X | X | X |  |  |  |  |  | X |
| R10 |  | X | X |  |  |  |  | X |  |

The final scoring corpus must be reviewed for architecture sensitivity and class balance during Pilot.

---

# 5. Scoring population and weighting status

Not frozen in this checkpoint:

```text
final scenario count
per-class sample count
production-frequency weight
QA-04 overall mean vs macro-average
QA-04 no-route-commit treatment
compound-route accounting details
score thresholds
```

Raw evidence must preserve:

```text
scenario_id
scenario_class
tags
QA eligibility
route-commit observations
constraint results
ModelCall records
```

so micro average, macro average and sensitivity analyses can be recomputed after Pilot without rerunning merely because aggregation changes.

---

# 6. Catalog freeze rule

This initial catalog is a skeleton.

Before final A/B/C/D qualification:

1. Pilot scenario implementability and architecture sensitivity;
2. review coverage gaps and topology bias;
3. define final scenario versions and class composition;
4. define QA eligibility and aggregation rules;
5. freeze `runtime-scenario-catalog-v1` (or equivalent);
6. run final qualification without post-hoc corpus changes.

A final result must never silently add/remove scenarios after seeing which alternative benefits.
