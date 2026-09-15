# Legacy → vNext Decision Point Migration Ledger

## Preservation rule

Legacy documents, prototype identifiers, result directories, and A/B/C/D scores are unchanged historical evidence. None is a QA-v1 score. Files at the legacy paths are prominently labeled and remain linkable; only the active navigation and catalog authority move to this directory.

## Historical DP-00 family mapping

| Historical identity | vNext relevance | Current status | Historical evidence preserved? |
| --- | --- | --- | --- |
| A — Thin VIA | Structurally informative for R1 | HISTORICAL_ONLY; no score transfer | Yes: legacy DP specs, prototypes, analyses, and `results/` retain A identity |
| B — ARGO-centric | Structurally informative for vendor-neutral R3 | HISTORICAL_ONLY; no score transfer | Yes: legacy DP specs, prototypes, analyses, and `results/` retain B identity |
| C — Hybrid VIA Fast Path | Local-execution lessons only; state-changing behavior means it is not automatically R1+@ | HISTORICAL_ONLY; no relabeling | Yes: C identity and behavior remain intact |
| D — Adaptive Per-turn | Adaptive-routing historical evidence; no active base family | HISTORICAL_ONLY | Yes: D identity and results remain intact |

Historical A/B results cannot become R1/R3 QA-v1 results. Historical C must not be renamed `@`, and D is not an active architecture family.

## Legacy catalog disposition

The “Legacy DP” column preserves the IDs as defined by the superseded catalog. Reused vNext numbers have new names only inside the active `vnext/` namespace.

| Legacy DP | Historical question | Current disposition | vNext replacement | Evidence preserved? |
| --- | --- | --- | --- | --- |
| DP-00 | A/B/C/D primary execution topology | MIGRATE | vNext DP-00 R1 vs R3; R1+@ tactic | Yes; legacy DP-00 files and all result identities |
| DP-01 | Speculative input processing/commit | DEMOTE_TO_TACTIC | May be evaluated within DP-01/02 implementations without changing authority | Yes; superseded catalog/analysis |
| DP-02 | Interaction evidence timeline authority | MIGRATE | DP-03 temporal retention; current-turn authority in DP-01 | Yes; superseded catalog/analysis |
| DP-03 | Local capability placement | ABSORB | DP-00 owns base execution boundary; strict read-only `@` is a tactic | Yes; dedicated legacy file |
| DP-04 | Common execution-contract topology | DEMOTE_TO_SUPPORTING_DECISION | Supporting contract design constrained by DP-00/04/05 | Yes; superseded catalog/analysis |
| DP-05 | Cross-task resource arbitration | DEMOTE_TO_SUPPORTING_DECISION | Supporting Task Supervisor/runtime design under DP-06/08 | Yes; superseded catalog/analysis |
| DP-06 | Voice runtime composition | DEMOTE_TO_TACTIC | Technology composition under DP-01/08 authority/resource boundaries | Yes; superseded catalog/analysis |
| DP-07 | Model/provider/policy values | DEMOTE_TO_POLICY | Versioned evaluation/runtime policy | Yes; superseded catalog/analysis |
| DP-08 | Voice/Text modality unification | MIGRATE | DP-01 canonical UserTurn authority | Yes; superseded catalog/analysis |
| DP-09 | Existing-task association authority | ABSORB | DP-05 session binding plus DP-06 Task supervision | Yes; superseded catalog/analysis |
| DP-10 | Intent/grounding responsibility decomposition | MIGRATE | DP-02 semantic responsibility topology | Yes; superseded catalog/analysis |
| DP-11 | Eligibility/selection/route-commit split | ABSORB | DP-00 and DP-02 authority; algorithms are tactics | Yes; superseded catalog/analysis |
| DP-12 | Capability catalog/registration authority | DEMOTE_TO_SUPPORTING_DECISION | Supporting interface decision constrained by DP-00/09 | Yes; superseded catalog/analysis |
| DP-13 | Task projection/execution recovery authority | ABSORB | DP-05 state isolation and DP-06 durable supervision | Yes; dedicated legacy file |
| DP-14 | Execution lifecycle/cancellation authority | ABSORB | DP-06 supervision; interaction control remains common obligation | Yes; dedicated legacy file |
| DP-15 | Failure containment/resilience authority | ABSORB | DP-06 runtime and DP-08 resource/failure boundaries | Yes; dedicated legacy file |
| DP-16 | VIA-local executor hosting/isolation | HISTORICAL_ONLY | Not applicable to strict read-only `@`; future stateful local execution requires a new DP | Yes; dedicated legacy file |

## Navigation effect

`decision-points/catalog.md`, the repository README, system overview, QA↔DP traceability, and evaluation README point to the active vNext catalog. Historical analyses and results retain their original links and wording but cannot claim current authority.
