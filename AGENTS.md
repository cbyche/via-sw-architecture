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
- The target Architecture under `docs/architecture/12-decisions/target-architecture/` is **REVIEWED_BASELINE**, explicitly confirmed by the user on 2026-09-29 after full-document review. Target Architecture Definition is complete; the next work is **Decision Reconstruction**: jointly develop a shortlist of structural choices, strong steelman alternatives and related ASRs before formal packages. Do not restart target design, map choices to previous DP IDs, or begin implementation, model runs, measurement freezes or performance measurement without authorization for that work.
- Baseline confirmation means fixed UCs, variants, ownership, contracts, normal/exception behavior and major policies are specified and accepted as the reference design. It is not measured superiority, an A/B winner, an immutable constraint on alternatives, or an override of accepted/deferred ADRs. Read control-and-lifecycle.md, memory-and-context-lifecycle.md, shared-omni-runtime.md and design-completeness.md with architecture.md. Do not reopen chosen policies merely because implementation or device-profile numbers are pending. Record the reason and affected contracts when changing the baseline. The four current ASRs are working design priorities, not a fixed future comparison set. Revisit additions, changes, definitions, priorities, applicability and evaluation criteria alongside structural choices and steelman candidates; preserve prior QA definitions/results until explicitly revised. Memory usage is a user-raised important ASR candidate; QA-41 remains diagnostic pending that review. Scheduling, Agent selection, response control and process placement are discussion topics, not automatically selected packages.
- Target-Architecture figures use paired editable `.drawio` sources and GitHub-renderable `.svg` previews under `docs/architecture/12-decisions/target-architecture/diagrams/`. Embed the SVG preview in Markdown, link the draw.io source next to it, and update both files together. Do not add Mermaid diagrams to the active target-Architecture documents.
- Use the exact full Component names from target `architecture.md` section 4 in all target prose, tables, contracts and figure labels. Do not shorten names or introduce aliases; adjust diagram layout instead. Distinguish data/concepts such as Task, Context and Response from their owning Components. Preserve historical reference terminology outside the current target design.
- Derive structural choices and steelman candidates from the reviewed target baseline together with a review of the ASR set; retain choices that materially affect the ASRs defined through that process. Do not restrict discovery to the current four ASRs. Put the candidate shortlist and later packages under `docs/architecture/12-decisions/decision-packages/`.
- New packages are retrospective Architecture rationale. They must include a steelman alternative, costs, conditions favoring that alternative, and falsification conditions. Do not map them back to VIA-DP-01~18 or manufacture a package for every Component.
- The 2026-09-30 Component-boundary candidate set was withdrawn after user review and archived. Reconstruct decisions from how major target functions are implemented and plausible alternative mechanisms, not from merging/splitting boxes. A candidate must change actual data acquisition, execution dependencies, intermediate state, commit or recovery behavior and explain the resulting costs. Component changes should express that mechanism; names, code placement, settings or generic maintainability claims alone are insufficient. The subsequent eight-candidate uniform-layout generation was also archived on 2026-10-01. The active four detailed comparison proposals cover semantic retrieval, durable request orchestration, speech evidence production and recovery authority. Option 1 must be the actual reviewed target; option 2 must change real Component/subsystem presence, execution or storage authority, not only arrows or names. Read `docs/architecture/12-decisions/decision-packages/00-workplan.md` after compaction or resumption for the durable plan and checkpoints. The user has subsequently approved a six-stage problem-first reconstruction: validate problem coverage, separate requirements from chosen means, identify quality conflicts, explore multiple structures, compare strong alternatives with the actual target, then select and detail DPs. The ten conversation questions are a seed list, reviewed in Stage 1 against the fixed scope; the four packages are re-evaluation inputs, not proof of exhaustive discovery. The current Stage 1 registry proposes sixteen problem areas, not sixteen DPs. The Stage 1 joint review has taken place: after reviewing the sixteen problems and VIA-specific characteristics, the user explicitly authorized Stage 2 and publication to GitHub. On 2026-10-01 the user explicitly approved 02-00-requirements-and-choices.md and requested punctuation cleanup of 01-00 followed by commit/push before Stage 3. That cleanup was pushed as e8a01e4 and CI passed. After reviewing the revised Stage 3 quality scenarios in 03-00-quality-scenarios.md, the user authorized Stage 4 and directed uneven exploration effort based on the agreed quality priorities. Stage 4 is ready for user review in 04-00-structural-alternatives.md: seven focused problems, six structural questions and seven non-target alternatives, with all thirteen quality viewpoints reviewed for each concrete alternative. Linked problems and deferred exploration have reasons and revisit triggers. The user explicitly requested independent agent review and commit/push. Three independent reviewers reported three distinct P2 findings after deduplication; all were corrected and rechecked with no unresolved P1/P2 reported. After user review, 04-00 became the overview; S-01~06 now have individual explanations in 04-01~06, each with paired SVG/draw.io figures, same-event walkthroughs, state/failure contracts and all thirteen viewpoints. Each document records selective use of the four older references. Independent review found three further diagram P2 issues; they were fixed and rechecked. Read 04-09-structural-review.md for findings and verification limits. S-02 retains A as a candidate needing further structural justification, with the strengthening direction unresolved. A adds separately produced and managed screen-item and relationship interpretations from the same observations to request interpretation. These are derived interpretations, not independent source evidence. Early execution is not exclusive to A; actual work replacement and quality gains remain unverified. Do not invent a stronger mechanism, drop A, or treat it as a selected DP. Questions and explicit selection are common clarification tactics, not a third primary architecture. Explain input data, separate record buffering from input-time alignment, and show speech input, triggers and conditional model calls with integer-numbered flows. Await user review before proceeding to Stage 5. The S markers are exploration questions, not DP IDs. Do not infer completion of Stage 4 or authorization for target changes, implementation or measurement. Stage 5 comparison and Stage 6 DP selection remain future work. Start with the current section of 00-workplan.md and 01-00-problem-coverage.md; older completion checklists apply only to their historical scope. Each proposal has background/comparison SVG and draw.io, concrete lifecycle contracts and ASR/QA trade-off tables. They remain unselected: document completion is not baseline change, ASR freeze, measured superiority or implementation authorization.
- VIA-DP-01~18 and the previous Core set **VIA-DP-03, VIA-DP-05, VIA-DP-06, VIA-DP-07, VIA-DP-15, and VIA-DP-17** remain active reference artifacts for prior work, not the reading order or decomposition constraint for the target Architecture. Their A/B winners remain unselected.
- Preserve previous DP reports, measurement drafts, ADR-003 and other accepted/deferred ADRs with their original status and caveats. Do not reinterpret them as evidence for the new target Architecture.
- The active `benchmark/architecture/`, `prototypes/candidates/`, and `results/architecture-evaluation/current/` trees were reset after the Core ASR/Core DP selection. They contain no current runner, executable candidate, or Core-ASR result yet. The 2026-09-27 generation is historical provenance under the matching archive trees; do not resume it in place.
- The target Architecture uses one shared on-device Omni model for two logical roles: S2S interaction and semantic interpretation. Target size is approximately 10B for the Thinker/backbone, using the Qwen3-Omni naming convention; inventory encoders, Talker/decoder and all helper weights separately. Never replicate Omni weights per Component, Task or role. Isolate role context, sessions, KV state and authority. User speech capture and recognition must continue during semantic inference; shared inference must support concurrent sessions with bounded scheduling, not a whole-call mutex. A dedicated lightweight Streaming ASR is the reviewed input-evidence design; count its full resource and fault costs without claiming implementation or measured benefits. Additional learned helpers must be explicit and justified by required capabilities. Prior DP model portfolios remain preserved references, not the current target constraint. Model/runtime interface requirements are owned by this Architecture; training and internal algorithms remain outside this repository scope. Downstream Agent internals remain external.
- When target-derived validation begins, hold user goals, completion conditions and external conditions fixed, freeze the contract before results, and preserve failures. Validation tests the chosen structure's claimed property and its falsification condition; it is not neutral winner discovery.
- QC-01~QC-10 are top-level quality concerns. The active QA catalog is organized by category ranges. Core IDs QA-09, QA-19, QA-29, and QA-39 coexist with detailed IDs QA-01~05, QA-11~15, QA-21~23, QA-31/32, QA-41, QA-51, and QA-61/62. Preserve the intentional gaps and do not describe pending targets or measurement freezes as final.
- The previous-generation QA-04/06/10/12 definitions are preserved only in `docs/archive/qa-catalog-draft-v1/`. Previous-generation QA-05 migrated to QA-11. Current QA-04, QA-05, and QA-12 are new category-range definitions. Never mix the prior meanings with the active IDs.
- QA-09, QA-19, QA-29, and QA-39 remain the four confirmed core ASRs in the existing QA baseline; this does not freeze the ASR set for future target-derived comparisons. Existing VIA-DP applicability records remain preserved; target-derived packages must state only the quality paths they physically affect and must not manufacture participation. Their targets, score bands, package-specific populations, machine contract, harness, and results remain pending.
- For the current target-Architecture design, the priority order is QA-19 semantic accuracy first, QA-09 responsiveness second, QA-29 modifiability third, and QA-39 reliability/recoverability fourth. All four remain required design concerns; lower priority does not mean optional. This is a design priority, not a comparison filter for later target-derived Decision Packages. Every target-derived package must report every ASR in the set defined during Decision Point and steelman development as `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, or `UNRESOLVED`, and measure every applicable axis under the same frozen contract. Do not hide a trade-off by excluding a candidate before measuring another axis or by collapsing the axes into a weighted score. Qualification failures may bar final selection when the frozen contract says so, but their measurements and failure evidence remain in the comparison.
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

- **2026-10-02 user correction — mechanism families and consequential choices:** The objective is to explain which SW structure serves VIA's important quality goals and why. Major alternatives must use fundamentally different mechanisms for the same user purpose, with substantial redesign needed to switch; local feature/Component additions, removals, adapters or internal implementation swaps are insufficient. Difficulty of reversal is a significance criterion, not a goal to manufacture. Do not defend a candidate merely by listing changed contracts, generic migration work or irrecoverable past data. Follow `docs/architecture/12-decisions/decision-packages/selection-principles.md` and the top checkpoint in `00-workplan.md`. This supersedes the earlier Component-presence admission criterion and Stage 4 review-waiting status above. The user authorized recording these principles on GitHub and executing Stage 4 reconstruction. Preserve S-01~06 as re-evaluation inputs; their completion/reviews do not qualify them as major DPs. First group easily combined T/A options into a mechanism family, distinguishing actual T from a hypothetical superset. Derive alternatives from important quality conflicts, describe the same normal/correction/revocation/failure/restart events, attempt the cheapest bidirectional conversion, and state both alternatives' credible favorable conditions and costs. Separate feature-development, live-state migration and fundamental redesign costs. Do not force six candidates, add needless complexity, invent new product scope or rewrite the reviewed target. Report the family map, strongest concrete candidates, conversion challenges, all thirteen viewpoints, deferrals and limits before Stage 5 or final DP selection. This is exploratory documentation and publication authorization, not implementation, model-run or measurement authorization.
- Stage 3 quality discovery is organized by the thirteen viewpoints V-01~13 in 03-00 section 2, not by previous QA IDs or the old core/additional split. Carry all thirteen through the conflict summary, concrete scenarios and Stage 4/5 review. For every explored structure and compared alternative, explain causal relevance, supported absence of effect or unresolved evidence for each viewpoint; do not silently omit an unfavorable axis or manufacture relevance. The viewpoint markers are not new QA IDs or thirteen ISO top-level characteristics. Preserve distinctions within grouped subcharacteristics and review any regrouping consistently across the whole set. Keep existing QA assets as separate references, not inherited formulas or methods.
- **Stage 3 user revision (2026-10-01):** V-02 is functional appropriateness; V-03 is functional completeness; V-04 is interaction responsiveness; V-05 is VIA-attributable request completion time. Former V-04~11 move to V-06~13. Carry forward priorities as V-01~03 jointly first, V-04/05 jointly second, V-08 third, V-06 fourth, V-07 fifth and V-09~13 jointly sixth; do not invent within-rank weights.
- **Partial-function alternatives are allowed in the current reconstruction.** Do not require full functional completeness before exploring or comparing a plausible structure. Keep the same reference requirements and explicitly record supported, partial, unsupported and unverified functions, lost user outcomes, manual workarounds and costs alongside quality gains. Do not silently shrink the population or treat unsupported cases as successful, equal or fast completion. This user instruction supersedes earlier all-functions-required candidate admission rules for the current reconstruction; preserve the approved requirements and reviewed target as reference documents. No particular function sacrifice or target change has been selected yet.
- Both V-04 and V-05 evaluate VIA-attributable time. Downstream Agent internal queue, reasoning, planning and execution durations are external conditions, not VIA performance. Under common external profiles, account for VIA preparation, queuing, contention, handoff, dependency release and result delivery. Do not freeze a naive elapsed-minus-external sum across overlapping work. Shared definitions and methods are fixed before measurement, not in Stage 3.
- **Stage 4 effort allocation:** prioritize problems that directly affect the agreed high-priority qualities. Do not give P-01~16 equal exploration depth, candidate counts or document length. Use the Stage 3 mapping plus structural causal relevance and scope, not the number of linked viewpoints; broad V-03/04/05 links do not make every P equally urgent. Record focused exploration, linked exploration or brief screening/deferment with reasons and revisit conditions. Deepen lower-priority problems when they enable or constrain a high-priority effect. This is effort allocation, not DP selection or proof of no effect. Every concretely explored alternative still receives review across all thirteen viewpoints; preserve unfavorable effects, unknowns and shared QA semantics.
- Stage 4 S-01~06 titles use the user-approved form: quality goal + function design, followed by a plain-language T/A question with each option summarized in 3–4 words. Keep document and main comparison-figure titles aligned; split long figure titles into purpose/function and comparison lines. A title states the design goal, not measured superiority or a restriction of the thirteen-viewpoint review. Retain additional alternatives such as S-04 B explicitly below the title and in the comparison.
- **Beginner-readable comparison invariant (2026-10-01):** every Stage 4~6 explanation must stand on its own for a first-time reader. Define a term when it first appears and name the responsible Component instead of using an ambiguous word such as `host`. Explain the shared input and preparation path before the alternatives diverge. For every call, return and commit, state who sends what to whom. Text and figures must each communicate the whole compared flow without requiring the other. In comparison figures, place common Components and data at the same coordinates in each option, put Component or subsystem names inside boxes, put actions beside numbered arrows, and draw request and return as separate directed arrows. Arrow numbers follow event order and match the prose. Black denotes shared structure, blue denotes T-only structure or paths, and green denotes A/B-only structure or paths, never preference. The active detailed rules are in `04-00-structural-alternatives.md` section 1.2.
- **Architecture diagram review standard (2026-10-01):** read `docs/architecture/12-decisions/decision-packages/diagram-design-guide.md` before creating or revising comparison figures. Show the design question, VIA responsibility boundary, external model dependencies and the mechanism that changes. Distinguish logical grouping, actual execution boundaries, data ownership and storage. Use focused subsystem detail or a separate sequence view when one plate cannot explain both structure and execution. Keep common geometry aligned, align exact Component names in consistent header compartments, and inspect rendered SVGs for text overlap, clipping and ambiguous routes. Model dependencies outside VIA responsibility do not imply remote deployment. Maintain editable draw.io and SVG together; layout validation is not architectural validation or measurement evidence.
- **Mechanism and notation revision (2026-10-01):** use the local Component/Module/Subsystem definitions in `diagram-design-guide.md`. Components own externally visible responsibilities; internal Modules must be drawn inside their owning Component. Do not label alternative Components as Modules merely because they are new. Use component header compartments, internal rounded modules, cylinders for storage, folded documents for data, and hexagons for integration targets/models (label internal owners explicitly). Show the actual replacement path or functional loss when removing a mechanism. Color shared parts black, T-only parts blue and A/B-only parts green; if only an internal module changes, keep the enclosing shared Component black. Use plain Korean action labels and short explanations instead of unexplained domain contracts. This notation applies to Stage 4 and later; preserve older reference-figure legends.
- **Cross-DP QA consistency is a user-mandated invariant (2026-10-01).** Each selected QA, including Functional correctness, must use one shared, versioned definition, metric and measurement method across ALL DPs and ALL alternatives. Sharing an ISO name or QA ID is not enough. Do not redefine correctness, evaluation units, oracle rules, endpoints, aggregation, failure treatment or measurement procedures per DP or option. Common population, applicability, denominator, weighting and exclusion rules must also prevent favorable case selection. Structural participation may differ; the common contract governs applicability, and non-participation or unknown support is not a zero score or equality.
- Stage 3 establishes quality meaning, scenarios, required outcomes and candidate observations without freezing formulas. Stages 4 and 5 explore multiple structures, then compare the actual target with a strong alternative to identify quality effects. Before measurement, define and freeze a shared QA contract for all DPs and alternatives. Reassess existing QA definitions and ASR priorities for suitability; preserving previous definitions/results does not mandate their reuse. Existing DP-specific evaluation records are preserved references, not exceptions to the cross-DP invariant.
- Keep authoritative QA definitions and measurement contracts in one shared source; DP documents reference the same version and explain causal participation. Candidate-specific instrumentation/adapters may map internal events to the common observation boundary but must not change evaluation semantics. Additional local diagnostics cannot replace or redefine the shared QA. If a QA needs revision, update the shared contract and all affected DPs together, identify incomparable old evidence, and do not present results from different versions as comparable. Before freeze, explicitly check every DP and alternative against the same contract; do not silently waive this check.
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

- In new or revised prose, avoid the Korean middle dot punctuation; use natural conjunctions or commas. Preserve link targets and historical records unless their cleanup is requested.
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
