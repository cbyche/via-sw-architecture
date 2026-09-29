# VIA Repository Instructions for Agents

This file is the operating contract for LLMs and automation working in this repository. The Git repository is the persistent source of truth; a chat session is not.

## 1. Understand the project before changing it

VIA is PC software that owns the continuous Voice/Text/screen interaction with the user. It may answer directly or delegate real work to a Downstream Agent, then reconnect progress and results to the same conversation and Task state.

VIA owns interaction and orchestration. A Downstream Agent owns domain reasoning, planning, tool selection, and work execution. S2S and VIA semantic models are dependencies whose integration belongs to the Architecture; their internals and training do not.

For any non-trivial Architecture or measurement task, read these files in order:

1. `README.md`
2. `docs/architecture/README.md`
3. `docs/architecture/01-system-mission-and-boundary.md`
4. `docs/architecture/03-fixed-architecture-scope.md` and `docs/architecture/05-representative-use-cases.md` for the fixed functional requirements
5. `docs/architecture/08-quality-attributes/core-asr-contract.md` for the current quality semantics
6. `docs/architecture/12-decisions/README.md` and `docs/architecture/12-decisions/target-architecture/README.md` for the current Architecture work
7. the relevant target-Architecture or quality-attribute document
8. `docs/architecture/11-measurement/event-boundary-contract.md` for endpoint work
9. `docs/architecture/11-measurement/README.md` and the relevant ADR for measurement or accepted-decision work

Do not infer the current project from filenames found by search alone.

## 2. Source-of-truth precedence

When documents disagree, use this order:

1. Explicit instructions in the current user request
2. `docs/architecture/` active baseline
3. `docs/adr/` decision records, interpreted with their status and caveats
4. Active code under `benchmark/architecture/`, `prototypes/candidates/`, and `scripts/architecture/`
5. Current evidence under `results/architecture-evaluation/current/`
6. `docs/references/` as supporting context only
7. Any `archive/` path as historical provenance only

Never treat content under `docs/archive/`, `benchmark/archive/`, `prototypes/archive/`, `results/architecture-evaluation/archive/`, or `results/gate2/archive/` as a current requirement, contract, result, or recommendation. Historical files may contain old paths and commands preserved from their original layout.

Do not modify an approved baseline, an accepted ADR, or archived evidence unless the task explicitly requires that change. If a current definition supersedes an old one, update active cross-references and preserve the prior generation in the archive.

## 3. Current project state agents must preserve

- The authoritative branch is `main`.
- The active Architecture baseline is `docs/architecture/`.
- The current Architecture work is **Target Architecture Definition** under `docs/architecture/12-decisions/target-architecture/`. Complete and agree on the target structure before deriving new Decision Packages or measurement freezes.
- After the target Architecture is agreed, extract only structural choices that materially affect QA-19 semantic accuracy, QA-09 responsiveness, QA-29 modifiability, or QA-39 reliability/recoverability. Put new packages under `docs/architecture/12-decisions/decision-packages/`.
- New packages are retrospective Architecture rationale. They must include a steelman alternative, costs, conditions favoring that alternative, and falsification conditions. Do not map them back to VIA-DP-01~18 or manufacture a package for every Component.
- VIA-DP-01~18 and the previous Core set **VIA-DP-03, VIA-DP-05, VIA-DP-06, VIA-DP-07, VIA-DP-15, and VIA-DP-17** remain active reference artifacts for prior work, not the reading order or decomposition constraint for the target Architecture. Their A/B winners remain unselected.
- Preserve previous DP reports, measurement drafts, ADR-003 and other accepted/deferred ADRs with their original status and caveats. Do not reinterpret them as evidence for the new target Architecture.
- The active `benchmark/architecture/`, `prototypes/candidates/`, and `results/architecture-evaluation/current/` trees were reset after the Core ASR/Core DP selection. They contain no current runner, executable candidate, or Core-ASR result yet. The 2026-09-27 generation is historical provenance under the matching archive trees; do not resume it in place.
- The target Architecture uses exactly one S2S model and one semantic LLM. Never load or replicate models per Component, Task, or semantic stage. Role-specific prompts, calls, sessions and buffers may differ; they share the same models. The extra timestamp-capable Streaming ASR in the preserved VIA-DP-03 A report belongs to that previous alternative and is not automatically part of the target Architecture. This does not authorize arbitrary TTS/helper models or per-Component replication. Downstream Agent internals remain external.
- When target-derived validation begins, hold user goals, completion conditions and external conditions fixed, freeze the contract before results, and preserve failures. Validation tests the chosen structure's claimed property and its falsification condition; it is not neutral winner discovery.
- QC-01~QC-10 are top-level quality concerns. The active QA catalog is organized by category ranges. Core IDs QA-09, QA-19, QA-29, and QA-39 coexist with detailed IDs QA-01~05, QA-11~15, QA-21~23, QA-31/32, QA-41, QA-51, and QA-61/62. Preserve the intentional gaps and do not describe pending targets or measurement freezes as final.
- The previous-generation QA-04/06/10/12 definitions are preserved only in `docs/archive/qa-catalog-draft-v1/`. Previous-generation QA-05 migrated to QA-11. Current QA-04, QA-05, and QA-12 are new category-range definitions. Never mix the prior meanings with the active IDs.
- QA-09, QA-19, QA-29, and QA-39 remain the four confirmed core ASRs. Existing VIA-DP applicability records remain preserved; target-derived packages must state only the quality paths they physically affect and must not manufacture participation. Their targets, score bands, package-specific populations, machine contract, harness, and results remain pending.
- For the current target-Architecture design, the priority order is QA-19 semantic accuracy first, QA-09 responsiveness second, QA-29 modifiability third, and QA-39 reliability/recoverability fourth. All four remain required design concerns; lower priority does not mean optional. This is a design priority, not a comparison filter for later target-derived Decision Packages. Every package must report all four as `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, or `UNRESOLVED`, and measure every applicable axis under the same frozen contract. Do not hide a trade-off by excluding a candidate before measuring another axis or by collapsing the axes into a weighted score. Qualification failures may bar final selection when the frozen contract says so, but their measurements and failure evidence remain in the comparison.
- QA-01~QA-04 are Voice responsiveness drafts defined by `docs/architecture/08-quality-attributes/voice-responsiveness.md`.
- QA-09 is the arithmetic mean of VIA-attributable time across applicable QA-01/02/03/05 interaction trials; QA-04 remains a Voice interruption regression. QA-19 is the micro-average of non-duplicate applicable QA-11/12 oracle fields; QA-13~15 remain regression diagnostics. QA-29 is the simple mean changed Architecture Element count across the DP-specific applicable QA-21~23 change superset. QA-39 is the simple pooled success rate of applicable fault trials that satisfy containment, correct recovery, deadline, no-duplicate, and evidence conditions using QA-31/32 evidence.
- QA-41 is a target-device memory diagnostic, not a core ASR. QA-51 and the action/access rules are safety qualifications; QA-61/62 are evidence qualifications. Their existing detailed semantics remain active, but the new core machine contract, targets, score bands, DP-specific populations, harness, and current results are not yet implemented or run. Never relabel predecessor reference results as QA-09/19/29/39 results.
- Existing predecessor implementation and results under the `w12-g1` archive are historical/superseded. `Gate 1` and `Gate 2` are not active lifecycle names.
- Current accepted decisions have caveats: AGENT-DP01=A, TASK-DP01=B, EXEC-DP01=B; IR-DP01 is deferred with A only as an interim reference. Read the ADRs before describing them.

Do not silently turn a pending definition into an implemented fact or a deferred decision into an accepted one.

## 4. Where changes belong

| Change type | Active location |
| --- | --- |
| System definition, use cases, quality attributes, target Architecture, rationale packages, measurement contract | `docs/architecture/` |
| Accepted or deferred decision record | `docs/adr/` |
| External/internal source material | `docs/references/` |
| Measurement fixtures, adapters, runners, analyzers | `benchmark/architecture/` |
| Candidate Architecture implementation | `prototypes/candidates/` |
| Evidence produced by the current frozen contract | `results/architecture-evaluation/current/` |
| Active consistency and review scripts | `scripts/architecture/` |
| Superseded generations | the matching `archive/` tree |

The current solution-first lifecycle is **Target Architecture Definition → Decision Reconstruction → Rationale & Falsification → Measurement & Revalidation**. Existing DP documents may retain their original lifecycle terminology as preserved prior work. Do not introduce `Gate 1` or `Gate 2` as current phase names. Exact legacy names may appear only when identifying archived paths or historical evidence.

Do not place new active files under a historical namespace. Do not overwrite prior result directories; create a new freeze/result directory when a new campaign is authorized.

## 5. Architecture and evaluation rules

- Keep user goals, completion conditions, fixtures, and external dependency profiles equal across comparisons and revalidation.
- Validate one target-derived structural claim at a time where practical. Record fixed context and `source_execution_key` when a result is reused; do not duplicate it as independent samples.
- Keep TASK and AGENT non-applicable to a QA path when they do not physically participate. Do not manufacture causality to fill a matrix.
- Freeze definitions, fixtures, repetition counts, aggregation, failure treatment, target, and score boundaries before seeing candidate results.
- Keep candidate input separate from evaluator-only oracle data.
- Record failures and timeouts; never select only successful samples to improve percentiles.
- A package must trace from the target Architecture to a structural difference such as authority, contract, state ownership, call graph, deployment, persistence, or fault boundary.

## 6. Measurement integrity

Use exact evidence labels and state limitations near the claim.

- `ESTIMATED_MODEL_ONLY`: token/rate or other model-only calculation
- `HYBRID_REFERENCE_ESTIMATE`: measured component spans combined with estimated/reference spans
- `MEASURED_MOCK_E2E` or `MEASURED_REFERENCE_HARNESS`: an executed mock/reference path
- `MEASURED_MODEL`: only when the named model actually ran and was observed
- `PRODUCT_E2E`: only for an actual product path with the required physical endpoints

Do not call mock/reference evidence `LIVE_S2S`, `MEASURED_MODEL`, `PRODUCT_E2E`, or target-device production latency.

For QA-01~QA-04 specifically:

- Use Voice input and audible Voice output as the primary path.
- The endpoint is first meaningful audible audio onset, not payload delivery to an instrumented sink; silence, earcons, filler, and generic acknowledgements do not end the metric.
- QA-01 excludes Downstream Agent queue/execution/completion time but preserves full user wall-clock as secondary evidence.
- QA-02 contains no Downstream Agent execution.
- QA-03 starts when a valid Agent status is available at its source.
- QA-04 starts at actual acoustic barge-in onset and ends at the interrupted response's last audible sample. It is not Agent Task cancellation latency.
- Qwen3-Omni 234 ms is a theoretical first-audio-packet reference under published conditions. It is not a QA start point, generic TTS latency, or actual VIA measurement. Use it only as a frozen scheduled dependency span when the contract permits.
- VIA LLM token-rate planning must freeze serialized prompts, tokenizer, input/output token counts, dependency graph, and rate profile. Label the result as an estimate, not measured wall-clock.
- Keep network, IPC, validation, Context access, speech generation, playback queue, audio buffer, and device onset as separate spans rather than hiding them in model time.

## 7. Work protocol

Before editing:

1. Run `git status --short --branch`.
2. Identify the active source files and check for unrelated user changes.
3. Read repository-local instructions and relevant current contracts completely.
4. State material assumptions when they affect scope or evidence.

While editing:

- Preserve unrelated user work.
- Use the project virtual environment at `.venv/`; do not install packages globally.
- Keep generated artifacts out of source directories unless the documented workflow requires them.
- Never add secrets, API keys, credentials, `.env` files, private user data, or machine-specific tokens.
- Update navigation and traceability when moving or superseding documents.

Before completion:

1. Run the narrowest relevant tests plus repository consistency checks.
2. Review `git diff --check`, `git diff --stat`, and the actual diff.
3. Confirm active links do not resolve into an archive as normative evidence.
4. Report what changed, what was verified, and what remains unimplemented or unmeasured.

Minimum documentation checks:

```bash
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
```

Candidate Rust checks apply only after an active Cargo workspace has been created under
`prototypes/candidates/`. Add the exact frozen toolchain commands to this section and CI together
with that workspace; do not run the archived workspace as a substitute.

## 8. Git policy

- Inspect status before and after work.
- `main` is the authoritative default branch. Do not push directly to it unless the user explicitly requests that action and repository protection permits it.
- On a non-default feature branch, commit and push completed, scoped implementation work by default only after relevant checks pass and no unrelated user changes would be included.
- Do not commit or push review-only, diagnosis-only, exploratory, incomplete, or failing work unless the user explicitly asks for that exact state.
- Never force-push, rewrite published history, amend another author's commit, or use destructive reset/checkout to discard user work.
- Do not create, delete, or merge branches merely as cleanup unless the user explicitly asks.
- Honor explicit requests to leave changes uncommitted or not pushed.
- After any push, report the commit SHA and CI status with a link when available.
