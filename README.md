# VIA Software Architecture

Engineering source of truth for Samsung PC-first, cross-device-capable Voice Interaction Agent architecture design and qualification.

## Current authority

- Approved immutable baseline: [`docs/requirements/requirements-v1.1.md`](docs/requirements/requirements-v1.1.md)
- vNext working requirements: [`docs/requirements/requirements-vNext.md`](docs/requirements/requirements-vNext.md)
- Active vNext DP catalog: [`docs/architecture/decision-points/catalog.md`](docs/architecture/decision-points/catalog.md)
- Active QA↔DP traceability: [`docs/architecture/qa-dp-traceability.md`](docs/architecture/qa-dp-traceability.md)
- QA Evaluation Contract v1: [`docs/evaluation/qa-contracts/v1/README.md`](docs/evaluation/qa-contracts/v1/README.md)
- Evaluation navigation: [`docs/evaluation/README.md`](docs/evaluation/README.md)

## Current DP-00

**DP-00 — Primary Reasoning & Execution Boundary** asks whether primary substantive reasoning and execution authority resides in:

- **R1 — Agent-neutral Control Plane:** VIA owns interaction, canonical turns, grounding, grounded goals, agent-neutral initial delegation, user-facing task lifecycle, consent interaction, and result binding. The selected Agent owns domain reasoning, planning, arbitrary tools, and domain workflow/execution state.
- **R3 — Primary General-purpose Agent Runtime:** one vendor-neutral general-purpose Agent Runtime owns substantive interpretation, planning, tools/runtime, workflow state, and specialist delegation. VIA retains interaction, user-facing Task correlation, permission/context-boundary interaction, and result delivery.

**R1+@ is not a third base family.** It is R1 plus bounded deterministic read-only local execution, with no state-changing domain action, arbitrary tools, open-ended planning, durable workflow, or Agent execution state.

No base architecture family is preselected. DP-00 selection will be driven first by measured QA-01 and QA-02 results under QA Evaluation Contract v1, with the remaining QAs used for trade-off, regression, hard-gate, and mitigation analysis.

The active definition is [`DP-00-primary-reasoning-execution-boundary.md`](docs/architecture/decision-points/vnext/DP-00-primary-reasoning-execution-boundary.md). Historical A/B/C/D specifications, prototypes, and results remain preserved without becoming R1/R3 QA-v1 evidence; see the [`migration ledger`](docs/architecture/decision-points/vnext/MIGRATION.md).

## Active QA contract

QA-01 through QA-12, their metrics, targets, score bands, populations, corpus counts, failure treatment, and Reference Environment are frozen by QA Evaluation Contract v1. Every active DP evaluates all twelve; a DP's Primary QAs only identify expected causal discriminators.

The evaluation flow is:

```text
Approved obligations + vNext question
  → active Decision Point alternatives
  → frozen QA-v1 contract/environment/corpora
  → integrated candidate evaluation
  → measured decision + ADR
```

No DP-00 vNext campaign has run yet. Candidate contracts and the protocol skeleton are ready under `benchmark/contracts/dp-vnext/` and `docs/evaluation/dp00-vnext-evaluation-protocol-v1.md`.

## Repository layout

```text
docs/requirements/                     approved and working requirements
docs/architecture/decision-points/vnext/ active Decision Point definitions
docs/architecture/analysis/            preserved reasoning and review checkpoints
docs/evaluation/qa-contracts/v1/       active frozen QA definitions
benchmark/contracts/dp-vnext/          active machine-readable DP/candidate contracts
benchmark/analysis/qa_v1/              shared QA-v1 evaluation implementation
prototypes/                             historical/prototype implementations
results/                                immutable raw, derived, and reported evidence
```

Approved requirements are never edited in place, and architecture selection is recorded only after controlled measured evidence and review.
