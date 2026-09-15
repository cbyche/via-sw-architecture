# VIA Software Architecture

Engineering source of truth for Samsung PC-first, cross-device-capable Voice Interaction Agent architecture design and qualification.

## Start here — integrated architecture review

For one end-to-end view of the VIA problem definition, stakeholders, UC-01~UC-16, FR-01~FR-44, constraints/principles, active Decision Points, QA-v1 contract, DP-00 evaluation evidence, known measurement-validity issues, and current work plan, see:

- **[VIA vNext Interim Architecture & Evaluation Report](docs/architecture/VIA-vNext-Software-Architecture-Interim-Report.md)** — current integrated review entry point; **interim, not a final architecture specification**

## Current authority

- Approved immutable baseline: [`docs/requirements/requirements-v1.1.md`](docs/requirements/requirements-v1.1.md)
- vNext working requirements: [`docs/requirements/requirements-vNext.md`](docs/requirements/requirements-vNext.md)
- Active vNext DP catalog: [`docs/architecture/decision-points/catalog.md`](docs/architecture/decision-points/catalog.md)
- Active QA↔DP traceability: [`docs/architecture/qa-dp-traceability.md`](docs/architecture/qa-dp-traceability.md)
- QA Evaluation Contract v1: [`docs/evaluation/qa-contracts/v1/README.md`](docs/evaluation/qa-contracts/v1/README.md)
- Evaluation navigation: [`docs/evaluation/README.md`](docs/evaluation/README.md)

## Current DP-00

**DP-00 — Primary Reasoning & Execution Boundary** asks whether primary substantive reasoning and execution authority resides in:

- **R1 — Agent-neutral Control Plane:** VIA owns interaction-facing input handling, grounding, grounded goals, agent-neutral initial delegation, user-facing task lifecycle, consent interaction, and result binding. The selected Agent owns domain reasoning, planning, arbitrary tools, and domain workflow/execution state. Committed logical UserTurn authority remains open until DP-01.
- **R3 — Primary General-purpose Agent Runtime:** one vendor-neutral general-purpose Agent Runtime owns substantive interpretation, planning, tools/runtime, workflow state, and specialist delegation. VIA retains interaction, user-facing Task correlation, permission/context-boundary interaction, and result delivery.

**R1+@ is not a third base family.** It is R1 plus bounded deterministic read-only local execution, with no state-changing domain action, arbitrary tools, open-ended planning, durable workflow, or Agent execution state.

No base architecture family is preselected. DP-00 selection is driven by QA-v1 measured evidence, with QA-01/QA-02 examined first and QA-04/QA-05 as additional primary structural discriminators; all twelve QAs remain in the campaign.

The active definition is [`DP-00-primary-reasoning-execution-boundary.md`](docs/architecture/decision-points/vnext/DP-00-primary-reasoning-execution-boundary.md). Historical A/B/C/D specifications, prototypes, and results remain preserved without becoming R1/R3 QA-v1 evidence; see the [`migration ledger`](docs/architecture/decision-points/vnext/MIGRATION.md).

## Active QA contract

QA-01 through QA-12, their metrics, targets, score bands, populations, corpus counts, failure treatment, and Reference Environment are frozen by QA Evaluation Contract v1. Every active DP evaluates all twelve; a DP's Primary QAs only identify expected causal discriminators.

The evaluation flow is:

```text
Approved obligations + vNext question
  → active Decision Point alternatives
  → frozen QA-v1 contract/environment/corpora
  → causal executable candidate evaluation
  → measured decision + ADR
```

## Current DP-00 evaluation status

A first QA-v1 campaign has been executed and preserved under:

- pre-registration: `9bb81ef3ff3831342f39437c52e039f9015a79a1`
- result commit: `fdcbf6e3537a66a13e77496daed5b3fa1bd94020`
- report: [`results/reports/dp00-vnext-qa-v1/dp00-vnext-qa-v1-campaign-v1/comparative-report.md`](results/reports/dp00-vnext-qa-v1/dp00-vnext-qa-v1-campaign-v1/comparative-report.md)

**Campaign v1 is not sufficient R1-vs-R3 selection evidence.** Subsequent code review found that the reference candidate runtime did not causally materialize enough of the R1/R3 structural differences for several QA observations. The campaign remains valuable as pipeline/provenance/pre-registration validation and preliminary replay evidence, and its artifacts remain immutable.

Additionally, QA-07 is still officially unevaluable because an evidence-anchored physical-memory calibration is not available.

Therefore the current decision status is:

> **DP-00: NOT DECIDED**
>
> Next: **offline causal executable evaluation v2**, preserving the frozen QA-v1 contract/corpus and using no Cloud/API calls in that phase.

The rationale and QA-by-QA correction plan are documented in the [Interim Architecture & Evaluation Report](docs/architecture/VIA-vNext-Software-Architecture-Interim-Report.md).

## Repository layout

```text
docs/requirements/                       approved and working requirements
docs/architecture/                       integrated review, system views and traceability
docs/architecture/decision-points/vnext/ active Decision Point definitions
docs/architecture/analysis/              preserved reasoning and review checkpoints
docs/evaluation/qa-contracts/v1/         active frozen QA definitions
benchmark/contracts/dp-vnext/            active machine-readable DP/candidate contracts
benchmark/contracts/qa-v1/               QA contract/environment/corpus authority
benchmark/analysis/qa_v1/                shared QA-v1 evaluation implementation
prototypes/                               historical/prototype implementations
results/                                  immutable raw, derived, and reported evidence
```

Approved requirements are never edited in place, and architecture selection is recorded only after controlled, causally valid measured evidence and review.
