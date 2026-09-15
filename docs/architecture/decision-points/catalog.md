# VIA vNext Decision Point Catalog

## Authority

This is the active entry point for vNext architecture decisions. The authoritative definitions are in [`vnext/README.md`](vnext/README.md), and the executable index is `benchmark/contracts/dp-vnext/dp-catalog-v1.json`.

The documents retained beside this file are **LEGACY / HISTORICAL DECISION RECORDS**. They preserve their original identifiers, links, alternatives, and evidence but are not part of the active vNext catalog.

No base architecture family is preselected. DP-00 selection will be driven first by measured QA-01 and QA-02 results under QA Evaluation Contract v1, with the remaining QAs used for trade-off, regression, hard-gate, and mitigation analysis.

## Active catalog

| DP | Decision | Alternatives | Primary QAs |
| --- | --- | --- | --- |
| [DP-00](vnext/DP-00-primary-reasoning-execution-boundary.md) | Primary reasoning and execution boundary | R1 / R3; evaluate R1+@ tactic | 01, 02, 04, 05 |
| [DP-01](vnext/DP-01-canonical-turn-authority.md) | Canonical UserTurn authority | Native/S2S / VIA canonical turn | 01, 02, 03, 05 |
| [DP-02](vnext/DP-02-semantic-decision-topology.md) | VIA semantic responsibility topology | Separate / unified authority | 01, 02, 04, 05 |
| [DP-03](vnext/DP-03-temporal-evidence-retention-authority.md) | Temporal evidence retention authority | VIA materialized / source-owned history | 01, 02, 05, 07 |
| [DP-04](vnext/DP-04-context-materialization-boundary.md) | Agent request context materialization | Pre-admission package / scoped handles | 01, 02, 04, 05 |
| [DP-05](vnext/DP-05-task-agent-session-binding.md) | UserTask to native Agent session binding | Shared / task-scoped | 01, 02, 03, 04 |
| [DP-06](vnext/DP-06-task-supervisor-runtime-boundary.md) | Frontend to Task Supervisor runtime boundary | Co-hosted / detached service | 03, 05, 08, 11 |
| [DP-07](vnext/DP-07-response-semantic-authority.md) | Voice/Text response semantic authority | One author / channel authors | 01, 02, 03, 05 |
| [DP-08](vnext/DP-08-inference-resource-topology.md) | On-device inference resource topology | Shared / realtime-isolated | 01, 07, 11, 12 |
| [DP-09](vnext/DP-09-device-capability-abstraction-boundary.md) | Device capability abstraction boundary | Core-neutral / device-family authority | 06, 05, 02, 03 |

All twelve QA-v1 attributes are measured for every active DP. “Primary” only identifies the QAs expected to discriminate the alternatives most directly.

See [`vnext/MIGRATION.md`](vnext/MIGRATION.md) for the complete legacy disposition and A/B/C/D evidence mapping.
