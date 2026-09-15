# DP-00 Executable Reference v3 Benchmark Protocol

Status: frozen at preregistration. QA Evaluation Contract v1 remains authoritative and unchanged.

The campaign builds Rust 1.94 release binaries once and starts each topology in a candidate-only temporary working directory containing only `RuntimeInput`, `SemanticReplay`, and `AgentReplay`. Oracle files remain in the evaluator process and are not copied. R1 launches `r1-via`, which launches Semantic Model, Policy Engine, General Agent and Specialist Agent processes. R3 launches `r3-shell`, which launches Semantic Model, Policy Engine and Primary Runtime; Primary Runtime launches Specialist. R1+@ launches the separate `r1-via-fastpath` artifact and uses its bounded deterministic local read implementation only when the replay marks a non-consenting `bounded-read` as read-only.

All boundaries use newline-delimited JSON over OS pipes with actual serialization, queueing and process scheduling. Candidate behavior follows the launched executable; the external display label is never sent. Every execution captures PIDs/parent PIDs, selected path, state owner, bytes, IPC/process hops, spans and persisted workflow records.

QA-01 uses five warmups per topology, seven measured repetitions per each of the original 150 cases, deterministic counterbalanced R1/R3/R1+@ order rotated by case and repetition, per-case median, and nearest-rank p95 over 150 medians. Semantic delay is the common official Reference Environment TTFT p50 (70 ms); Agent and local-tool replay delay is 10 ms. Architecture overhead is observed wall clock minus those identical requested dependency delays and is diagnostic only. No candidate-specific sleep, probability or additive hop constant exists.

QA-02 uses all 600 goals. QA-03 executes all 400 chronological event scripts, including required concurrency and lifecycle variants, through the same running topology. QA-08 executes the frozen 200 fault schedules; process-owner classes restart actual child processes and all cases verify persisted identity/version/idempotency. QA-09 crosses each applicable executable trust boundary through an independently evaluating Policy process. QA-10 reconstructs graphs only from normal spans. QA-11 uses the same process event propagation path. QA-12 consumes fixed 10 ms local frames and clears the ring after speech onset.

QA-04/05/06 each use a clean detached Git worktree at this preregistration SHA per case. A change-request classifier routes natural semantic terms—not family IDs, expected zones or oracle data—to a real source module, appends a unique compile-visible constant, runs `cargo test --lib --offline`, captures `git diff --no-index`/changed modules, then maps those modules through the frozen ownership manifest. Evaluation consults the oracle only after observations are complete. Shared Cargo target storage is a build cache only.

Official QA-07 remains UNEVALUABLE because the frozen physical-memory denominator is unavailable. The campaign records a non-scoring local structural diagnostic using process RSS where the host exposes it and never invents a denominator.

Preflight requires unit/integration/E2E success, exact baseline hashes, static anti-cheating checks, label swap invariance, runtime topology/state/IPC inspection, evolution build/diff inspection, coverage gates and real defect mutations. A failed sensitivity mutation or missing stratum blocks preregistration. Post-campaign review recomputes every scalar from raw observations and fails if sources changed after preregistration.

