# VIA Repository Instructions for Agents

This file is the operating contract for LLMs and automation working in this repository. The Git repository is the persistent source of truth; a chat session is not.

## 1. Understand the project before changing it

VIA is PC software that owns the continuous Voice/Text/screen interaction with the user. It may answer directly or delegate real work to a Downstream Agent, then reconnect progress and results to the same conversation and Task state.

VIA owns interaction and orchestration. A Downstream Agent owns domain reasoning, planning, tool selection, and work execution. S2S and VIA semantic models are dependencies whose integration belongs to the Architecture; their internals and training do not.

For any non-trivial Architecture or measurement task, read these files in order:

1. `README.md`
2. `docs/architecture/README.md`
3. `docs/architecture/01-system-mission-and-boundary.md`
4. the relevant use case or quality-attribute document
5. `docs/architecture/11-measurement/event-boundary-contract.md` for endpoint work
6. `docs/architecture/11-measurement/README.md` for measurement work
7. `docs/architecture/12-decisions/README.md` and the relevant ADR for decision work

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

Never treat content under `docs/archive/`, `benchmark/archive/`, `prototypes/archive/`, or `results/gate2/archive/` as a current requirement, contract, result, or recommendation. Historical files may contain old paths and commands preserved from their original layout.

Do not modify an approved baseline, an accepted ADR, or archived evidence unless the task explicitly requires that change. If a current definition supersedes an old one, update active cross-references and preserve the prior generation in the archive.

## 3. Current project state agents must preserve

- The authoritative branch is `main`.
- The active Architecture baseline is `docs/architecture/`.
- Core evaluation is a direct A/B comparison for each DP with other DP conditions held fixed.
- A 16-configuration full-factorial run is secondary interaction analysis, not the primary winner-selection method.
- W-01, W-02, and W-03 are Voice-in/Voice-out metrics defined by `docs/architecture/08-quality-attributes/voice-responsiveness.md`.
- Their semantic and event-boundary document drafts exist, but the machine-readable contract, harness, target, score bands, and current results are not yet implemented or run.
- Existing predecessor implementation and results under the `w12-g1` archive are historical/superseded. `Gate 1` and `Gate 2` are not active lifecycle names.
- Current accepted decisions have caveats: AGENT-DP01=A, TASK-DP01=B, EXEC-DP01=B; IR-DP01 is deferred with A only as an interim reference. Read the ADRs before describing them.

Do not silently turn a pending definition into an implemented fact or a deferred decision into an accepted one.

## 4. Where changes belong

| Change type | Active location |
| --- | --- |
| System definition, use cases, quality attributes, measurement contract, DP alternatives | `docs/architecture/` |
| Accepted or deferred decision record | `docs/adr/` |
| External/internal source material | `docs/references/` |
| Measurement fixtures, adapters, runners, analyzers | `benchmark/architecture/` |
| Candidate Architecture implementation | `prototypes/candidates/` |
| Evidence produced by the current frozen contract | `results/architecture-evaluation/current/` |
| Active consistency and review scripts | `scripts/architecture/` |
| Superseded generations | the matching `archive/` tree |

The active lifecycle names are **Measurement Contract Definition → Candidate Implementation → A/B Measurement & Evaluation → Architecture Decision**. Do not introduce `Gate 1` or `Gate 2` as current phase names. Exact legacy names may appear only when identifying archived paths or historical evidence.

Do not place new active files under a historical namespace. Do not overwrite prior result directories; create a new freeze/result directory when a new campaign is authorized.

## 5. Architecture and evaluation rules

- Keep user goals, completion conditions, fixtures, and external dependency profiles equal across candidate A/B comparisons.
- Compare one DP at a time. Record fixed context and `source_execution_key` when a result is reused for non-applicable axes; do not duplicate it as independent samples.
- Keep TASK and AGENT non-applicable to a W path when they do not physically participate. Do not manufacture causality to fill a matrix.
- Freeze definitions, fixtures, repetition counts, aggregation, failure treatment, target, and score boundaries before seeing candidate results.
- Keep candidate input separate from evaluator-only oracle data.
- Record failures and timeouts; never select only successful samples to improve percentiles.
- A decision must trace to a structural difference such as authority, contract, state ownership, call graph, deployment, persistence, or fault boundary.

## 6. Measurement integrity

Use exact evidence labels and state limitations near the claim.

- `ESTIMATED_MODEL_ONLY`: token/rate or other model-only calculation
- `HYBRID_REFERENCE_ESTIMATE`: measured component spans combined with estimated/reference spans
- `MEASURED_MOCK_E2E` or `MEASURED_REFERENCE_HARNESS`: an executed mock/reference path
- `MEASURED_MODEL`: only when the named model actually ran and was observed
- `PRODUCT_E2E`: only for an actual product path with the required physical endpoints

Do not call mock/reference evidence `LIVE_S2S`, `MEASURED_MODEL`, `PRODUCT_E2E`, or target-device production latency.

For W-01~W-03 specifically:

- Use Voice input and audible Voice output as the primary path.
- The endpoint is first meaningful audible audio onset, not payload delivery to an instrumented sink; silence, earcons, filler, and generic acknowledgements do not end the metric.
- W-01 excludes Downstream Agent queue/execution/completion time but preserves full user wall-clock as secondary evidence.
- W-02 contains no Downstream Agent execution.
- W-03 starts when a valid Agent status is available at its source.
- Qwen3-Omni 234 ms is a theoretical first-audio-packet reference under published conditions. It is not a W start point, generic TTS latency, or actual VIA measurement. Use it only as a frozen scheduled dependency span when the contract permits.
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

Candidate Rust checks when `prototypes/candidates/` changes:

```bash
cargo +1.98.1 fmt --manifest-path prototypes/candidates/Cargo.toml --all -- --check
cargo +1.98.1 clippy --locked --manifest-path prototypes/candidates/Cargo.toml --workspace --all-targets -- -D warnings
cargo +1.98.1 test --locked --manifest-path prototypes/candidates/Cargo.toml --workspace --all-targets
```

## 8. Git policy

- Inspect status before and after work.
- `main` is the authoritative default branch. Do not push directly to it unless the user explicitly requests that action and repository protection permits it.
- On a non-default feature branch, commit and push completed, scoped implementation work by default only after relevant checks pass and no unrelated user changes would be included.
- Do not commit or push review-only, diagnosis-only, exploratory, incomplete, or failing work unless the user explicitly asks for that exact state.
- Never force-push, rewrite published history, amend another author's commit, or use destructive reset/checkout to discard user work.
- Do not create, delete, or merge branches merely as cleanup unless the user explicitly asks.
- Honor explicit requests to leave changes uncommitted or not pushed.
- After any push, report the commit SHA and CI status with a link when available.
