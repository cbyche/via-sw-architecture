# Deviations index

Sixteen files, one per phase (phase 5 and 6 split further, one file per crate),
record every place VIA's behaviour differs from `qwen-audio-agent` or from
ARGO. Read individually they are receipts: each entry names a file and a
reason, written at the moment the crate that owns it shipped. Read together
they answer a question no single phase file can: *of everything that
differs, how much is VIA doing better, how much is VIA doing differently on
purpose, and how much is VIA simply not doing yet?*

This document sorts all of it — **454 entries** pulled from all sixteen
files — into exactly those three answers.

| List | What it means | Count |
| --- | --- | --- |
| [Corrections](#corrections) | VIA behaves *better*: a bug not ported, a hang avoided, a hole closed | 49 |
| [Divergences](#divergences) | VIA behaves *differently*, on purpose, and both sides are fine with it | 375 |
| [Gaps](#gaps) | VIA does *less* — the honest list of what still needs work, owner named | 30 historical + the current pending contracts below |

The three lists are not equally sized by design. A port this size *should*
have far more intentional divergences than corrections — most of what
differs is a Rust idiom or a dropped GUI surface, not a defect either side
needed fixing. The Gaps list is the shortest by count but the one worth
reading closely, because it is the only one of the three that changes as
work continues.

## How to read this

Each entry below is a one-line paraphrase of the fuller record in its phase
file — follow the phase-file link for the file/line citation, the upstream
snippet, and the reasoning in full. Entries are grouped by phase file, in
delivery order (0 → 9), matching `docs/fidelity.md`'s own table.

A note on **staleness**: the phase files are point-in-time records, written
the day their crate shipped. A Gap that says "owed to `via-backends`, which
does not exist yet" was true at phase 0 and is not a claim about today —
several of the phase-0-era "owed" gaps were closed to `Partial` once the
named crate landed. Corrections and Divergences do not have this problem:
they describe a permanent, intentional property of the shipped code, not a
todo. For the same reason, the historical
[Gaps as recorded per phase](#gaps-as-recorded-per-phase) section at the end
of this document is kept for completeness and citation, but the section
immediately below it —
[**the currently pending catalogue rows**](#the-current-picture-via-conformances-registry)
— is the one that answers "what, right now, is still not done," because it
is generated from the same registry `cargo test -p via-conformance` checks on
every run, not from a snapshot in time.

---

## Corrections

### Phase 0 — workspace, gates, leaf crates

[`phase-0.md`](./phase-0.md)

- via-catalog: `resolve_dashscope_realtime_model_profile` returns `Err(UnknownRealtimeModel)` instead of upstream's all-capabilities-false profile — upstream bug (silent false-capability profile causing a session with no audio format/turn detection) mandated NOT to be ported; fail-closed strengthened.
- via-store: two added fsync barriers (temp file + parent dir before/after rename) that upstream lacks — closes a durability hole (rename may not survive a power cut) upstream doesn't close.
- via-store: on serialization failure, writes nothing rather than upstream's placeholder `{version}` document — avoids silently replacing good state with a near-empty one.
- via-arch-test: band/via-backends dependency rule also checks dev-dependencies, which upstream's regex (server/src only) never covered — catches a class of drift (leaf test reaching into core) upstream's own check would miss.
- via-conformance: DashScope unknown-model-id fallback contracts (6) — mirrors via-catalog's mandated bug-not-ported fail-closed correction (conformance's own internal label is "Divergent" but by this doc's rubric it's a correction since it's an upstream defect not ported).

### Phase 1 — via-core, via-i18n, setup gate, binary skeleton

[`phase-1.md`](./phase-1.md)

- via-core: an unknown `VIA_REALTIME_MODEL` id is a hard resolve error — upstream's all-capabilities-false fallback silently opens a session that connects and never hears anything.
- via-core: an opaque `data:`/`blob:` Origin is refused (`false`) rather than reaching upstream's `new URL('null')` throw, which Express turns into an unhandled 500 on a security boundary.
- via-core: `Secret` newtype redacts `Debug`/`Serialize` output for all credential fields — closes the hole where a `{:?}`-formatted `Config` during a startup failure would leak a key straight into a log.

### Phase 2 — via-acp, via-backends, via-process, via-downstream

[`phase-2.md`](./phase-2.md)

- via-downstream: `CancelOutcome::Requested` reports `cancelling`, not upstream's `cancelled` written before the cancel is confirmed — "the deliberate correction docs/architecture.md §4/§6 asks for."
- via-acp: start-up failure classification (`进程意外退出` vs `初始化失败`) is decided deterministically (try_wait → stderr → transport failure) instead of upstream's race between `initialize` and the child's `exit` event, which nondeterministically picks the reported message.
- via-process: `validate_runtime_driver`'s requirement that a separately-managed process also declare a service address avoids upstream reaching `new URL(null)` and throwing an untyped `TypeError`.
- via-process: the generic full-permission refusal now runs for every driver, not only `managedProcessDriver`-built ones — "closes a hole a hand-built driver could otherwise walk through."
- via-process: `backend.process_started` publishes the real `childPid` field name; upstream logs `pid`, which its own logger silently overwrites with the Gateway's own pid, so the child's pid never reaches the log.
- via-backends: every managed child is contained (process-group leader on unix, Job Object via process-wrap on Windows); upstream "has no Windows containment at all," leaving orphaned/undetached children possible on that platform.

### Phase 3 — via-work, via-coordinator, via-mcp-tools

[`phase-3.md`](./phase-3.md)

- via-work: cancellation only finishes once a canceler reports `Confirmed` or the abort actually settles the runner; upstream's `cancel()` calls `finishCancellation` as soon as its fire-and-forget ACP notification returns, so it can report `cancelled` for a runner that never stopped.
- via-work: a timed-out `scheduled_task` now resolves its waiters — upstream's watchdog path emits failure events but never calls `task.resolve`, so "every `wait()` on a timed-out scheduled task hangs for the life of the process." Explicitly "Fixed."
- via-work: `task.scheduled.fired` is emitted from both the on-time and restart-catch-up fire paths; upstream only emits it from the restart path, so a reminder that fires on time is invisible to every client.
- via-work: a `RunnerEvent` for an already-terminal Work is dropped; upstream's listener has no terminal guard, so a late `backend.permission.requested` can put a pending permission back onto a completed record.
- via-work: a persisted record with no usable `id` is dropped on load; upstream's filter lets it through and stores it under `this.tasks.set(undefined, task)`, corrupting the map.
- via-coordinator: `CancelOutcome::Requested` on the coordinator route is promoted to `Confirmed` with a caller-supplied timestamp — the coordinator route genuinely is a confirmation (the model's own cancel tool returned), continuing via-downstream's "confirmed, not optimistic" fix.
- via-coordinator: `PermissionBroker::respond` takes a typed decision instead of upstream's `decision === 'always'` string check, so "there is no path by which a typo silently denies" a permission.
- via-coordinator: a permission request carries its scope as a value rather than upstream's mutable `session.permissionScopeId` set-before/cleared-in-`finally` pattern (upstream itself guards with "if it is still the current value," i.e. defends against its own reentrancy hazard).
- via-coordinator: `NativeToolUpdate` (not `RawSessionUpdate`) has nowhere to carry `rawOutput` — the exact field a session id can hide in — so detection can read it without ever publishing it, closing a potential session-id leak.

### Phase 4 — via-conversation

[`phase-4.md`](./phase-4.md)

- via-conversation: CLOSED — consolidating four drifted copies of `is_js_whitespace` into `via-core` uncovered that the `via-voice::text` copy was a *superset* of ECMAScript's `\s` (it also matched U+0085), so `via-voice`'s `clean` was silently collapsing a character upstream leaves alone; the shared, tested implementation fixes that latent bug. (closed/resolved)
- via-conversation: list keys fold with `to_lowercase` (locale-independent) rather than `toLocaleLowerCase`, avoiding upstream's host-locale-dependent behavior — "a list key that folded differently per host would be worse than one that folds the same everywhere."
- via-conversation: `fileMtimeMs` does not rethrow; upstream's `statSync` propagates any non-ENOENT error out of `lists()`/`show()`, "which would make a `stat` failure crash a read." VIA treats an unreadable mtime as changed instead of panicking.

### Phase 5 — apps/via

[`phase-5-apps-via.md`](./phase-5-apps-via.md)

- apps-via: `via_app::heartbeat` now returns the `GatewayLeaseHandle` instead of consuming it and returning `()` — the old signature made releasing the lease unexpressible, so every Gateway would have left a `gateway.lock` naming a dead pid behind; fixed so `gateway::run` releases it as the last close step.
- apps-via: `main` no longer holds the process-wide stdout lock across `dispatch` — doing so was a genuine deadlock for `via gateway` (Gateway's console-log worker thread blocks on the same lock main never releases), observed to hang on the first `gateway.ready` line.
- apps-via: signal handlers (`gateway::shutdown_signal`) are registered eagerly before the banner prints, closing a real race where `tokio::signal` registers only on first poll — without this fix SIGTERM could take default disposition and kill the process with the lease still on disk.
- apps-via: `/cancel` excludes `scheduled` Work from `CANCELLABLE_STATUSES` — closes a safety hole where the model's own cancel tool, picking "the first cancellable one" from a list the user can't see, could silently kill a reminder/timer the user set on purpose.
- apps-via: a realtime front end that fails to open still reaches the client with an error frame (matching upstream's `send(ws,{type:'error',...})`) rather than logging and returning no engine — the logged-and-silent first draft left a `via chat` user staring at `turn.started` then silence, the one failure a text client cannot diagnose.

### Phase 5 — via-app

[`phase-5-via-app.md`](./phase-5-via-app.md)

- via-app: the SSE stream now sends a periodic keep-alive comment — upstream sends nothing between events, so a stream behind a proxy with an idle timeout is silently closed; comments aren't `EventSource` events so the catalogued `data: {…}\n\n` framing is untouched.

### Phase 5 — via-realtime-dashscope

[`phase-5-via-realtime-dashscope.md`](./phase-5-via-realtime-dashscope.md)

- via-realtime-dashscope: the "deafness" bug is not ported — upstream's `resolveDashScopeRealtimeModelProfile()` answers an unrecognized model id with an all-capabilities-false profile that silently produces a session with no input audio format and no turn detection (opens fine, never hears the user, no error at all); VIA removes the `unknown`-family fallback entirely (`resolve_dashscope_realtime_model_profile` returns `Err`) and adds `DashScopeProvider::preflight()` to refuse before any socket opens, run ahead of `is_configured()`, so the dangerous payload is unreachable rather than merely unlikely (upstream's own mitigation is a separate, fragile check in a different file, "one forgotten line away from not working").
- via-realtime-dashscope: a GA tool with a missing field is serialized with an explicit `null` rather than silently dropped — upstream's `JSON.stringify` omits `undefined` fields, making a missing `parameters` (for example) the harder bug to find; VIA's explicit `null` lets the service's own schema validation name the field.
- via-realtime-dashscope: a tool outside the beta shape is passed through unchanged instead of upstream's behavior of emptying it into a no-op `{type:'function'}` tool that "exists and does nothing" — VIA's version makes the service report a schema error instead of silently losing the capability.

### Phase 5 — via-realtime

[`phase-5-via-realtime.md`](./phase-5-via-realtime.md)

- via-realtime: `response.created` with no usable response id no longer hangs — upstream removes the pending response from the correlation queue before checking the id, so it never settles and the whole output queue's tail awaits it forever; VIA settles it `failed / correlation`.
- via-realtime: a zero sample rate is refused — upstream's `Number.isFinite(0)` is true so `inputSampleRate: 0` passes validation and causes division-by-zero in the resampler and a client told to capture at 0 Hz; VIA rejects it with the "missing rate" message.

### Phase 5 — via-voice

[`phase-5-via-voice.md`](./phase-5-via-voice.md)

- via-voice: `clean`'s whitespace predicate originally got the ECMAScript-vs-Unicode direction wrong (collapsed a U+0085 upstream leaves alone) — a genuine defect introduced in VIA's own port, fixed in the phase-4 conformance catch-up by re-exporting `via_core::text::is_js_whitespace` (closed/resolved).
- via-voice: a malformed announcement instant now drops only its `completed_at` line — upstream's `new Date(ms).toISOString()` throws on an unrepresentable value and would lose the whole announcement; VIA's `format::Announcement::block` omits just the bad line.
- via-voice: a retry no longer cancels the lease heartbeat — a mutation-testing find showed the four announcement timers shared one generation counter, so cancelling the acknowledgement timer on retry also cancelled claim renewal, letting a retrying batch's notification claim expire (risking two live frontends presenting the same result); fixed by giving each timer kind its own epoch.
- via-voice: retiring a batch no longer strands a queued one — `finish_acknowledged_batch`/`abandon_now` bumped the same counter the delivery epilogue checks, stranding a result queued mid-delivery until an unrelated flush; restored to upstream's rule of bumping that counter only in `pause()`.

### Phase 6 — via-realtime-openai

[`phase-6-via-realtime-openai.md`](./phase-6-via-realtime-openai.md)

- via-realtime-openai: the WebSocket upgrade is now bounded — ARGO's `connect_async` has no timeout of its own, so a peer that accepts the TCP connection and never completes the upgrade would outlive ARGO's own documented 12s connect budget; VIA gives the upgrade the same timeout window as the first-frame probe.

### Phase 6 — via-wake-word

[`phase-6-via-wake-word.md`](./phase-6-via-wake-word.md)

- via-wake-word: a keyword file the model cannot encode used to end the whole process (sherpa-onnx's `EncodeBase`/`InitKeywords` log and exit, with no panic/backtrace) — latent in upstream and invisible until a phrase changes; VIA adds `TokenInventory`, validated at install and at open, returning `WakeWordError::UnknownTokens` instead of taking the Gateway down.
- via-wake-word: the same C++ function throws an uncaught exception on a malformed `:score`/`#threshold` marker (`std::stof` with no `try`) — VIA adds `TokenInventory::malformed_markers`/`WakeWordError::MalformedMarker`, deliberately as lenient as `stof` itself so it never under-rejects a value the engine would accept.
- via-wake-word: a multi-word keyword label silently mis-parses in the vendor's line format (`EncodeBase` splits on whitespace) — latent in upstream, which never hits it because its phrase is a single CJK word; VIA adds `WakePhrase::label()` to join words with `_` and separates a `Keyword`'s spoken text from its engine-reported label.
- via-wake-word: the model archive never touches disk — upstream streams the body to a temp file, re-reads it to hash it, re-reads it again to extract it, and cleans it up in a `finally`; VIA hashes and extracts the same in-memory buffer, removing the temp file, its cleanup path, and the window between hashing a file and reading it back.
- via-wake-word: concurrent installs serialize on a per-target async mutex instead of sharing upstream's module-level promise — identical for the success case (one download), and better for the failure case, where upstream hands every waiting caller the first caller's error while VIA lets each retry independently.

### Phase 7 — via-context

[`phase-7.md`](./phase-7.md)

- via-context: Provenance lattice classifies FirstParty-tool + External-content as ThirdParty, flooring trust to Untrusted so it's always fenced ("the second row is the one that did not exist" in ARGO's model) — closes a hole where third-party bytes could reach a rendered prompt unfenced; forgetting the fence becomes structurally unrepresentable rather than merely a discipline every call site must remember.

### Phase 8 — via-realtime-local

[`phase-8-via-realtime-local.md`](./phase-8-via-realtime-local.md)

- via-realtime-local: Completes via-catalog's phase-0 fix for the upstream/ARGO bug where an unknown realtime model id silently opens a session with turn detection disabled ("hears nothing") — `model_profile` now returns real flags for the `local` family, `preflight` refuses missing weights by name, and `WeightsSet::verify` distinguishes "nothing installed" from "one file missing"; the defect is deliberately not reproduced.

---

## Divergences

### Phase 0 — workspace, gates, leaf crates

[`phase-0.md`](./phase-0.md)

- via-protocol: GATEWAY_PROTOCOL_VERSION is "1.0.0" not upstream's "2.0.0" — VIA advertises a strict capability subset; a removed capability would be a breaking change under upstream's own semver rule.
- via-protocol: GATEWAY_CAPABILITIES lists 7 of upstream's 16 (drops web.*/desktop.*/electron/gateway-process entries) — no SPA/Electron/skin-store surface in a core Rust port; dropped names tracked in DROPPED_UPSTREAM_CAPABILITIES.
- via-protocol: WorkState has 5 variants, not the 4 named in the task brief — followed contracts.json (upstream computes a real "scheduled" workState) over the brief.
- via-protocol: ProtocolError Display strings are English, not upstream's Chinese literals — leaf crate has no via-i18n dependency; codes/payloads reproduced exactly, text deferred to via-i18n.
- via-protocol: AlreadyRunning carries only `origin`, not upstream's full `error.lease` object — avoids pulling a via-lock dependency into a leaf crate.
- via-protocol: two VIA-owned error codes invented (VIA_PROTOCOL_UNKNOWN_WIRE_VALUE, VIA_PROTOCOL_ILLEGAL_TRANSITION) with a distinct prefix so they're never mistaken for inherited contracts.
- via-protocol: transition graph is the union of the architecture doc's diagram and upstream's actual code transitions (diagram abbreviates); crash-recovery rewrites deliberately excluded as edges.
- via-catalog: new `local` model family with no upstream peer (real capability flags incl. `audio_input: true`) for on-device GGUF/ONNX models.
- via-catalog: invented values documented as such — `TurnDetectionKind::ServerVad`, `DEFAULT_LOCAL_REALTIME_VOICE` ("af_heart"), `LOCAL_FAMILY_VOICE_ENV` — none are upstream contracts.
- via-catalog: three new providers (openai, local-omni, mock) added, none aliased; upstream's two entries kept first to preserve prefix ordering.
- via-catalog: identity shape (ModelAndVoice/EndpointOnly) is a declared provider property, not an inline `provider === 'dashscope'` branch — type-safety idiom.
- via-catalog: error message text not reproduced byte-for-byte (5-provider list vs upstream's 2) — typed `CatalogError` with stable `code()`, sentence deferred to via-i18n.
- via-catalog: env var names renamed per docs/rebrand.md (QWEN_AUDIO_AGENT_* / QWAUDIO_* → VIA_*); vendor-owned names (DASHSCOPE_API_KEY, OPENAI_*, etc.) unchanged.
- via-catalog: InstallationSpec.steps closed/resolved — pinned install coordinates deliberately live in via-backends (execution machinery), not via-catalog; landed as planned.
- via-catalog: onboarding hint/label Chinese strings closed/resolved — user-facing text lives in via-i18n keyed by backend id, as planned.
- via-catalog: env-reading functions (resolveDashScopeRealtimeVoiceOverride, resolveRealtimeFrontendConfiguration) not ported here — via-catalog is a pure table (no env/fs/clock); via-core does the lookups.
- via-catalog: SkillsSpec is a type-enforced non-optional field instead of upstream's runtime `validateBackendSkillsSpec` throw — same guarantee at compile time.
- via-catalog: `effective_backend_permission_mode` returns `String` not an enum — upstream itself performs no validity check and passes unrecognised modes through.
- via-log: `[Circular]` marker reproduced as a constant only — a `serde_json::Value` is acyclic by construction so the cycle-detection behavior itself can't occur.
- via-log: correlation context is a thread-local, not upstream's `AsyncLocalStorage` (doesn't follow `await`) — async call sites expected to carry correlation via a `tracing` span instead.
- via-log: string truncation counts Unicode scalar values, not upstream's UTF-16 code units — identical for ASCII; JS surrogate-splitting isn't reproducible in a Rust `String`.
- via-log: `fatal` has no `tracing::Level` — reachable only via `Logger::fatal`/`log_record`, never the tracing Layer.
- via-log: `bigint` JS handling has no `serde_json::Value` counterpart — analogous case (i128/u128 out of range, non-finite f64) handled in the tracing visitor instead.
- via-log: `ErrorRecord::from_error` sets `name = "Error"` and leaves `stack` unset — Rust errors carry no name/captured stack by default; fields exist for callers that do have them.
- via-log: sink writes are synchronous under a Mutex, not upstream's promise-queued async drain — a failed write loses only that record rather than the whole pending queue.
- via-log: `is_test_process` gap closed by adding a third VIA-owned signal (`VIA_TEST` env var) after discovering neither upstream arm (NODE_ENV, argv `test/` segment) can ever fire for a `cargo test` binary.
- via-log: Windows omits directory/file mode enforcement (0o700/0o600) — applied `#[cfg(unix)]` only; Windows inherits parent ACL, documented rather than silently stubbed.
- via-log: `(?-u:\b)` groups in redaction regexes are the Rust spelling of JS's ASCII-only `\b` (Rust's default `\b` is Unicode-aware) — behavior matches, literal differs.
- via-log: `LOG_SINK_FAILURE_PREFIX_ZH` closed/resolved — architecture rule (`shared → ∅` for Leaf-band crates) forbids a via-i18n lookup here even now that via-i18n exists; constant kept, drift guarded by a via-conformance test instead.
- via-store: mkdir-based lock, not `std::fs::File::lock` — the on-disk mechanism itself is the compatibility surface shared with a Node Gateway/older VIA/CLI.
- via-store: interpolated engine error text can't match Node's `error.message` — fixed template prefixes/suffixes reproduced verbatim, only the engine-generated substring differs.
- via-store: `load` returns `Option<Map>` instead of taking a `fallback` closure — same four-site semantics via a Rust idiom.
- via-store: write coalescing is synchronous/mutex-based, not upstream's promise-chained deferred writes — same generation-bump/stale-write-detection scheme, different concurrency primitive.
- via-store: `now` injection kept on the store (observable `.corrupt-<now>` filename) but dropped from the lock (upstream exposes it there only as a test seam).
- via-store: `StoreMessages::TASK_STORE` string table is a parameter rather than upstream's duplicated state machine — via-work reuses the catalogued wording without a second store.
- via-store: migration hook is new (upstream has no such hook); a hook returning `None` falls through to upstream's quarantine behavior unchanged.
- via-store: `LockError` is a `thiserror` enum with `code()` instead of a JS `Error.code` property — per docs/fidelity.md's ad-hoc-object-to-enum rule.
- via-store: `replace_file`'s Windows ladder is gated by a runtime `cfg!(windows)` check instead of `#[cfg(windows)]` so it's linted/tested on every host.
- via-store: panicking `on_warning` callback wrapped in `catch_unwind` as the Rust equivalent of upstream's bare try/catch ("diagnostics must never prevent startup").
- via-lock: `find_running_gateway` is synchronous, not async — no tokio/HTTP client in the phase-0 workspace; health probe is an injected trait instead.
- via-lock: `release(self)` consumes the handle (compiler-enforced no-Drop) instead of flipping an internal `released` flag — a second release is a compile error, not a runtime `false`.
- via-lock: `LeaseUpdate` structurally cannot express schema/instanceId/pid, matching upstream's overwrite-via-Object.assign behavior with a type-system guarantee instead.
- via-lock: `gateway_lock_path` uses `Path::join`, not Node's absolutizing `resolve()` — callers always pass an already-absolute config dir.
- via-lock: `pid` is typed `i64` with serde default — a non-integer/out-of-range pid reads the whole lease as absent/dead rather than reaching `kill(2)`.
- via-lock: unrecognised fields in a foreign lease document are dropped on read, not preserved as upstream's raw object would.
- via-lock: `LeaseError::code()` returns `Option<&'static str>` — only the conflict case carries a code upstream defines; others deliberately return `None` rather than inventing one.
- via-lock: `LeaseError::Io` from `replace_lease` names the temp path where upstream rethrows the original error unchanged — cosmetic, same code path/retry behavior.
- via-lock: lease/CLI-lock Chinese literals closed/resolved — kept verbatim (Leaf-band crates can't depend on via-i18n even now it exists); via-i18n carries duplicate copies guarded against drift by a conformance test; CLI pair further diverge for the rebranded binary name.
- via-lock: `cli.lock` (`src/cli.rs`) implemented in the gap-closure pass, deliberately NOT built on the gateway-lease machinery since upstream's two locks differ in file/shape/attempts/reclaim/error surface; inherits the same consumed-handle/no-Drop and string-pid-reads-as-dead patterns.
- via-lock: `CliLockError::Exhausted` is a race-only unreachable branch, matching upstream's own race-only branch; message asserted by test despite being unexercised.
- via-audio: `FrameBuffer::append_pcm16le` does NOT drop an odd trailing byte the way the stated contract says, for a streaming accumulator (byte carried to next append instead) — the stateless one-shot decoder functions still reproduce the literal contract exactly.
- via-audio: base64 decoding not implemented here — this crate does only PCM16LE→f32; base64 belongs to the wire/transport codec.
- via-audio: resampling, WAV, frame buffer and duration accounting are new to VIA with no upstream equivalent to deviate from.
- via-audio: the `max(160,...)`/`max(240,...)` floors from upstream are reproduced and tested even though they never bind at the sample rates VIA actually uses (16/24/48 kHz).
- via-arch-test: uses upstream's dependency-boundaries.test.mjs and architecture.md §9 directly as its contract source since docs/reference/contracts.json has no crate-graph entries.
- via-arch-test: upstream's `app` row omits `shared`, but VIA's App band may depend on Leaf — upstream's `root` layer (entry files) is folded into Band::App, producing the leaf edge.
- via-arch-test: four upstream directory-based layers merge pairwise into VIA's architectural bands (agent+process→layer3, conversation+task→layer2, app+root→app).
- via-arch-test: `Band::Tests` has no upstream row; its allow-set is given explicitly as every band rather than derived.
- via-arch-test: author created `apps/via` as an empty skeleton crate (outside their assignment) because an empty `apps/*` glob broke the whole cargo workspace; left for the owning phase to overwrite.
- via-arch-test: has a real `src/` rather than an empty `lib.rs` so the rules can be exercised against synthetic graphs.
- via-conformance: GATEWAY_PROTOCOL_VERSION/GATEWAY_CAPABILITIES asserted as intentionally-reduced (mirrors via-protocol entries above).
- via-conformance: backend credential allow-list env names asserted as fully rebranded (`VIA_*`) via a documented `value::rebranded` rule rather than retyped constants.
- via-conformance: Work `status`/`workState` values asserted as unordered SETS since upstream declares no order for either.
- via-conformance: backend credential allow-list test expands 3 abbreviated openclaw env names before parsing, since the catalogue text can't be read literally.
- via-conformance: `client input capability profiles` gap closed — via-protocol grew `src/client.rs`; profiles parsed out of the catalogue rather than retyped.
- via-conformance: ownership of 3 "file transaction lock" contract rows corrected from via-lock to via-store (the crate whose code actually carries the lock, not the Gateway instance lease).
- via-conformance: 7 rows classified Divergent purely for renames (via-log schema/env prefixes, via-lock lease schema/error prefix/CLI binary name, logger redaction regex spelling) — each derived from the upstream literal via a `value::rebranded`-style rule, never retyped.
- via-conformance: depends on 5 crates, not 2; via-audio deliberately excluded since it owns no catalogued contract.

### Phase 1 — via-core, via-i18n, setup gate, binary skeleton

[`phase-1.md`](./phase-1.md)

- via-core: `Config::realtime: RealtimeFrontend` nests ten upstream-flat fields into one struct, since one function resolves them together; each keeps its upstream name so `/api/health` still assembles field-for-field.
- via-core: `Config::default()` leaves path fields empty because they are host-derived, not literals; `resolve()` fills them from `via_core::paths`.
- via-core: `DASHSCOPE_COMPATIBLE_BASE_URL` is one named constant referenced twice instead of the two literal copies upstream carries.
- via-core: `resolve()` takes host facts (`home_directory`, `working_directory`, `runtime_root`) as `Overrides` parameters rather than reading `os.homedir()`/`process.cwd()` inline, keeping it a pure function.
- via-core: `root` is normalised against the working directory once, up front, rather than left for each later `resolve(root, value)` call as upstream does — equivalent, since Node's `path.resolve` would do the same.
- via-core: env var is `VIA_SLEEP_TIMEOUT_SECONDS`, the narrowest of three candidate rebrand spellings, chosen because there is no orb to auto-hide once `desktop/` is dropped; no legacy alias is implemented.
- via-core: `config.wakeWord` becomes the configurable `VIA_WAKE_WORD` (default empty string) instead of upstream's hard-coded `你好千问` literal, per architecture.md §16's "never a literal" rule and `scripts/brand_leak.py`; the actual value is a phase-6/`via-wake-word` decision.
- via-core: the audio-family voice variable is `VIA_REALTIME_VOICE`, reusing `via-catalog`'s phase-0 constant rather than introducing rebrand.md's alternative spelling.
- via-core: `webDistributionPath()` is not ported — VIA drops the packaged React SPA (`web/`) entirely as a core-only Rust port with no same-origin UI to locate.
- via-core: the setup gate gains `openai`/`local-omni`/`mock` provider arms (five providers vs upstream's two), each marked as a VIA extension.
- via-core: the voice override for non-DashScope providers reads `VIA_LOCAL_REALTIME_VOICE` rather than a DashScope-family variable, since only DashScope ids are in the catalog; `via-realtime-openai` may narrow this further.
- via-core: numeric settings are truncated toward zero via `integer_setting` (counts/durations/ports) while `number_setting` preserves JS `Number()` semantics exactly, since a bare port of JS's fractional-float behavior would be invisible/meaningless for these fields.
- via-core: the env-file parser reproduces Node's `util.parseEnv` except `\n`-unescaping inside double quotes, since no VIA template or documented value uses it.
- via-core: `ACP_LABEL` etc. are trimmed with Rust's `str::trim`, which agrees with JS `String.prototype.trim` on every character either would meet in practice.
- via-core: `assert_full_permission_allowed` is exported but applied by `via-process` at spawn time, not by `resolve()`, matching upstream's own separation of concerns (`managed-backend.mjs` checks it at spawn, not in `config.mjs`).
- via-core: seeded templates (`config.env`, `USER.md`, `MEMORY.md`) render from `via-i18n` instead of upstream's Chinese literals, so `en` installs get English.
- via-core: three untranslated `<!-- 例如：… -->` example lines and one DeepSeek prose line are dropped/simplified from the templates since no `via-i18n` key exists for them.
- via-core: three dependencies (`hmac`, `url`, `getrandom`) were added to `[workspace.dependencies]` for the auth-secret CSPRNG and the origin allow-list's WHATWG URL semantics, rather than hand-rolling either.
- apps/via: six CLI verbs replace upstream's eight — `webui` and `skill` are dropped entirely (no GUI/npm surface in a core-only Rust port); other verbs are remapped/consolidated (`gateway run`→`via gateway`, `gateway install|start|stop`→`via service …`, `tui`→`via chat`, `setup`→`via backend status`, `install NAME`→`via backend install NAME`).
- apps/via: `--no-open` and `--skill`/`--list` have no home in this build because the verbs that owned them (`webui`, `skill`) were dropped along with them; each is asserted to be rejected by the parser, not silently accepted.
- apps/via: `--takeover` moved from `tui`/`webui` to `chat`, since `via chat` is the client that replaces both removed surfaces.
- apps/via: `clap` rejects a misplaced flag structurally (declared per-command) instead of upstream's runtime-checked "flag X only applies to command Y" refusals — same constraint, enforced earlier and visible in `--help`.
- apps/via: a usage error exits 1, matching the catalogued "1 = any thrown error" contract, rather than `clap`'s default exit code 2.
- apps/via: `config set` without `--realtime-model` keeps the catalogued refusal message because the flag is `Option<String>` rather than `required`, avoiding `clap`'s generic "required argument" text; same treatment for `config <unknown>`/`service <unknown>`.
- apps/via: `--url`'s `[env: VIA_URL]` help annotation hides the actual value so help-text snapshots don't depend on the developer's environment.
- apps/via: the lease is taken and immediately released in each stub command, which is what makes `VIA_GATEWAY_ALREADY_RUNNING` reachable now and prevents a build with no liveness probe from inheriting a lease naming an already-exited pid.
- apps/via: the CLI's machine-readable error code travels through the `cli.log` structured record rather than stderr, since the catalogued stderr shape (`` via: <message> ``) has no room for one.
- apps/via: `--url` sets `HOST`/`PORT` via `GatewayOptions` (since there's no child process to derive them for), reproducing upstream's `localhost`→`127.0.0.1` rewrite and `|| '80'` fallback quirk exactly, including the default-port-yields-80-not-443 case.
- apps/via: the `config.env` staging temp file follows `via-store`'s `<path>.<pid>.tmp` naming (via `via_core::runtime::write_file`) instead of upstream's `.config.env.<pid>.<uuid>.tmp`; every catalogued durability property (0600/0700/atomic replace/fsync) is still reproduced.
- apps/via: two upstream config-reader quirks (`^\s*KEY\s*=\s*(.*?)\s*$` keeps quotes; `export KEY=x` doesn't match; `||` vs `??` on empty vs whitespace env values) are reproduced rather than corrected, because the reader must see the file the way the rewriter does.
- apps/via: the locale is resolved twice — once from the process environment (for pre-config-load messages/templates), once from the merged environment (so `VIA_LOCALE` in `config.env` governs later messages).
- apps/via: `Host::runtime_root` duplicates two lines of `via-core`'s root resolver because the root must be known before `load_runtime_environment` runs; tests assert the two answers always agree.
- apps/via: `clap = { version = "4", features = ["derive", "env"] }` was added to `[workspace.dependencies]` under a new `cli` heading.
- apps/via: three `via-i18n` keys (`cli.config_show_model_item`, `cli.list_separator`, `cli.not_implemented_until_phase`) were added to `cli.json` for sentences the catalogue didn't have, the first two logged in `NOT_CHINESE_PROSE` since they're punctuation.
- apps/via: `apps/via` ships a library target alongside the binary so integration tests can drive both the process and the underlying functions directly.
- apps/via: `apps/via` depends on `via-catalog` (Realtime model/backend tables) and `via-store` (the `config.env` file transaction) beyond the five crates the brief named; both are Leaf-band per `via-arch-test`.

### Phase 2 — via-acp, via-backends, via-process, via-downstream

[`phase-2.md`](./phase-2.md)

- via-downstream: `bounded()` counts UTF-16 code units like JS `slice` but stops one character short rather than emit a lone surrogate a Rust `String` can't hold — identical to upstream for all BMP text.
- via-downstream: `clean()`/`bounded()` use ECMAScript's exact `\s` set rather than Rust's `char::is_whitespace`, narrowing phase-1's `via-core` convention (plain `str::trim`); flagged for reviewers as a possible inconsistency to unify.
- via-downstream: `HarnessRegistry::resolve("")` returns `NotConfigured` rather than upstream's "不支持的后台 Agent" error, since frontend-only is a supported mode (architecture §2), not misconfiguration.
- via-downstream: two capability-consistency rules (permissions must negate `alwaysFullPermission`; `backendUi` must agree with `defaultBaseUrl`) and three self-consistency rules are VIA additions with no upstream statement, derived from and tested against all twelve backend declarations.
- via-downstream: three catalogued driver-validation refusals (missing `createProfile`, two skills-unset refusals) have no runtime analogue in Rust's typed struct fields and are not reproduced; `tests/contracts.rs` keeps the catalogue segments honest.
- via-downstream: `backendFailureCode()`'s regex classifier is deliberately not ported here — it belongs beside the state machine that produces the error text, in `via-acp`.
- via-downstream: duplicate backend registration keeps upstream's last-wins Map semantics, but `register()` returns the displaced `Arc` instead of discarding it silently, since registration is runtime configuration in VIA, not a hard-coded list.
- via-downstream: `ActivityTracker` is deliberately left unbounded, exactly matching upstream's unbounded `known` Map, so both project updates identically; `len()`/`clear()` let `via-acp` end the lifetime instead of inventing a cap.
- via-acp: two i18n keys (`acp.process_error`, `acp.process_error_with_stderr`) were added to `acp.json` so the message/stderr separator isn't a bare literal, mirroring the existing `acp.request_failed` pair.
- via-acp: `initialize.clientCapabilities` serializes its two `fs` flags spelled out (`{"fs":{"readTextFile":false,...}}`) rather than upstream's literal `{}` — semantically identical, byte-different because of the SDK's typed struct.
- via-acp: `VIA_NODE` stays a recognized internal name but is never stamped, since VIA has no Node interpreter of its own to publish via `process.execPath`.
- via-acp: stderr is attached to a request failure via the SDK's own `is_incoming_transport_closed` discriminator rather than upstream's `error.name === 'RequestError'` string check; behavior matches on both tested cases.
- via-acp: Windows teardown uses process-wrap's JobObject `start_kill` in one step rather than upstream's `taskkill /PID /T` then `/F` ladder, since Windows has no "please exit" signal to escalate from.
- via-acp: `stopProcessTree` idempotence is a single `stopped` flag on `AcpChildHandle` rather than upstream's per-child `WeakMap` of in-flight cleanups — same guarantee, one owner.
- via-acp: `backend-environment.test.mjs`'s generic-ACP case is ported here against invented namespaces (since this crate may not name a real backend); the same properties against the real twelve backends were owed to `via-backends` and are paid there in `backend_environment.rs`. (closed/resolved)
- via-acp: the scripted fake ACP agent is a `[[bin]]` at `tests/bin/fake_agent.rs` rather than a test file, since `CARGO_BIN_EXE_<name>` is Cargo's only mechanism for a companion executable.
- via-acp: `clean` on a JSON array/object renders as JSON rather than JS `String()`'s `"1,2"`/`"[object Object]"` — unreachable in practice since every call site is a declared string field.
- via-acp: dropped unused `via-protocol` and `tokio-util` dependencies from `Cargo.toml` so the crate graph `via-arch-test` reads has no false edges.
- via-acp: the catalogue's "47 OS names" prose is trusted against the actual 39-name list (matching `shared/backend-environment.mjs:7-47`), and the list — not the prose count — is treated as authoritative.
- via-process: `managedScript` becomes `ManagedLaunch` (command + args resolved on PATH) since Node shims don't survive the port; upstream's refusal message is kept unchanged.
- via-process: `VIA_NODE` is not stamped since VIA spawns a real command with no interpreter path to publish; the name stays recognized for embedders that set it.
- via-process: `ELECTRON_RUN_AS_NODE=1` is not hard-coded; it becomes an opt-in `ManagedLaunch::environment_additions` entry since it's an Electron-runtime concern.
- via-process: two of `validateRuntimeDriver`'s five checks (missing `resolve` function, missing boolean process-ownership flag) are unrepresentable, since Rust structs are typed and neither invalid state can exist.
- via-process: `validate_runtime_driver` strengthens upstream's check by requiring the driver's base-URL variable/default to agree with the catalog — upstream only asserts this in a test because its driver map is a static literal, while VIA's caller-built registry needs the runtime check to avoid manufacturing an address-less backend.
- via-process: `validate_runtime_driver` takes the `BackendDefinition` as a parameter (matching upstream's own signature) rather than looking it up, letting tests exercise every refusal with a synthetic definition and no forbidden backend name.
- via-process: `start_managed_backend` returns `ManagedBackendStart { backend, runtime }` rather than the runtime alone, since the caller needs `base_url` after a possible port reallocation without re-deriving which env variable to read.
- via-process: the failure/stderr separator (full-width `：`) is a per-locale const in this crate rather than a `via-i18n` key, since `via-i18n` has no punctuation-only key and is already shipped.
- via-process: `backend_failure_code` does not classify Korean messages (upstream's regexes cover en/zh only); widening the patterns would be a behavior change rather than a port, so it's left as-is and asserted.
- via-process: `applyLocalAddress`'s port-fallback quirk (explicit port wins, else 443 for `https:`/80 for everything else including `wss:`) is reproduced intact even though it's inert in practice (only reached with an explicit port already present).
- via-process: `stop()` does not wait again after escalating to SIGKILL, matching upstream, so shutdown doesn't block longer than necessary.
- via-process: `SupervisedChild` gains `take_stdout`/`take_stderr` and `SpawnSpec` gains `ChildStdio::Piped` (defaulting to unused) since `BackendRuntimeState::failed` needs a captured stderr tail that upstream's always-inherited stdio never provides.
- via-process: PATH composition reuses `via_core::search_path` (already the port of `shared/path-environment.mjs`); `compose_child_search_path` adds only the executable-entry cleaning from `shared/backend-install.mjs`.
- via-process: the OpenClaw-specific full-permission refusal is reachable only via `ProcessError::DriverRefused` carrying a `via-i18n` *key* rather than a rendered sentence, since producing that key is `via-backends`' job under the no-backend-names rule.
- via-backends: `ELECTRON_RUN_AS_NODE=1` is not stamped on any child, since architecture.md §10 removes the Node-shim mechanism it existed to serve.
- via-backends: the six shim-launched backends spawn the final executable directly (e.g. `opencode acp`, `codex-acp`) instead of `process.execPath scripts/<name>.mjs`, while every environment variable the shim set is still stamped.
- via-backends: managed-service argv carries a fixed `--port` (set when the runtime driver is built) rather than upstream's shim reading `OPENCLAW_PORT`/`OPENCODE_PORT` at start-up; `managed_launch` is exposed so a caller can rebuild after a port reallocation.
- via-backends: `src/json5.rs` is a scoped JSON5→JSON normaliser (comments, unquoted keys, single quotes, trailing commas) rather than a full parser; unsupported forms (hex/leading-dot numbers, `Infinity`/`NaN`, non-ASCII keys) yield `None`, which every caller already treats as "no token to reuse."
- via-backends: the three `sessionInstructions` paragraphs (OpenClaw, DeepSeek, Pi) are `pub const` English literals rather than `via-i18n` keys, since upstream doesn't localize them either and `contracts.json` pins the English text.
- via-backends: two negative-lookahead regexes are hand-reproduced line-by-line in Rust since the `regex` crate has no lookahead support.
- via-backends: `installBackend`'s Windows npm PowerShell/`npm config get prefix` fallbacks are not ported, since the PATH walk already answers the same question and upstream's own hard-coded fallback values are used directly.
- via-backends: installer cancellation is an explicit `InstallCancel` flag rather than dropping the future, since the installer must distinguish `CANCELLED` from `STEP_FAILED` in its result.
- via-backends: `BackendAvailability::start()` combines construction and one eager background refresh into one Gateway call site; `snapshot()`'s background refresh is a no-op with no ambient tokio runtime rather than a panic.
- via-backends: `write_private_file` delegates entirely to `via_core::runtime::write_file(IfExists::Replace)` rather than hand-rolling a second mkdir/temp/chmod/rename sequence.
- via-backends: `inspect_backend_setups`'s OpenCode/OpenClaw source-runtime checks stat the source directory and toolchain binary but skip the extra `packages/opencode/src/index.ts`/`node_modules/@opencode-ai/tui` stats, which only refine an issue message.
- via-backends: `register_backends` only skips the generic ACP backend's two configuration refusals (inheriting via-downstream's `UnsupportedBackend`/`NotConfigured` split), so a Gateway with no `ACP_COMMAND` still registers the other eleven backends instead of failing startup.

### Phase 3 — via-work, via-coordinator, via-mcp-tools

[`phase-3.md`](./phase-3.md)

- via-work: one `Sleep` drives the whole overdue reminder backlog via a deadline-override stagger (`now + index × stagger`) rather than upstream's N extra `setTimeout`s, preserving the same order/spacing while never polling.
- via-work: every status change goes through `WorkStatus::transition_to`'s validated graph instead of upstream's eleven direct field assignments, so `backend.delegated` from `finalizing`, `backend.delegation.completed` from `running`, and a second completion for the same delegation are each refused rather than applied (each logs `task.illegal_transition`).
- via-work: delegation correlation (`DelegationRef::correlates_with`) is enforced on the record itself per architecture.md §11 invariant 4, rather than relying solely on the adapter's `delegatedWorkRuns` map to have correlated first as upstream does.
- via-work: `recover_delegated`'s refusal is a direct restore-time field rewrite, not a graph transition, since `queued → failed` isn't an edge `via-protocol`'s graph models (crash recovery is explicitly excluded from it).
- via-work: `PersistedWork::result_metadata` stays raw JSON rather than a typed `PublicResultMetadata`, so it can accept the legacy `{decision: {presentation}}` shape already on disk; it's re-projected on read and re-persisted in the new shape.
- via-work: subscription uses `tokio::sync::broadcast`, so a slow observer can lag and is told, rather than upstream's synchronous listener set that cannot lag; a send with no receivers still isn't an error, preserving "one observer must not break the work queue."
- via-work: `create`/`create_scheduled` return `Result<_, ManagerStopped>` rather than the `None`/empty every other method returns on a stopped manager, since a submission must not appear to succeed when nothing will ever run.
- via-work: the six `tasks.json` warnings are built from `via-i18n` keys (`store::task_store_messages(locale)`) rather than `via_store::StoreMessages`'s plain `fn` pointers, which can't capture a locale.
- via-work: `text::slice_units` exists beside `via_downstream::text::bounded` because the Work record's own title bounds use plain code-unit slicing, and reusing `bounded`'s whitespace-collapsing would rewrite a delegation title the coordinator chose.
- via-coordinator: the catalogue's "16 lines" prose is trusted against the actual 15-line array value (the 17th line is the `join`), matching via-acp's "47 vs 39" resolution.
- via-coordinator: `parseCoordinatorDecision`'s unconditionally-`null` `task`/`targetSession` fields are not reproduced since nothing reads them; `state`/`mode` become `const fn`s instead.
- via-coordinator: `resultEnvelope.raw` is not reproduced — `via_downstream::PromptOutcome` has already reduced the `session/prompt` result to content plus a stop reason, so only `stop_reason` survives.
- via-coordinator: `is_busy` counts the lane's queued depth, not just started turns like upstream's `activeCoordinatorTurns`, making an urgent cancel take the transport route slightly more often — "the safe side of the trade," matching upstream's own "cancellation is urgent" comment.
- via-coordinator: `PermissionBroker::request` answers in `via_acp::PermissionDecision` rather than an ACP reply directly — the broker decides *what*, `via-acp` decides *which option says it*.
- via-coordinator: `SESSION_ID_KEYS`'s lookup order matches the adapter's `||` chain, deliberately kept distinct from `acp-backend-session-utils.mjs`'s differently-ordered chain, since the two upstream orderings genuinely disagree on some inputs and both are pinned by test.
- via-coordinator: `work_lines`' `'unknown'` status fallback is dropped as dead code, since `via_work::PublicWork::status` is a `via_protocol::WorkStatus` that always spells itself (no unrepresentable state exists).
- via-coordinator: `DelegationRegistry` is a `Mutex`, not an owning task, since it's a pure lookup table with no `await` inside its critical section; the three actual ordering invariants each get their own owning task elsewhere.
- via-coordinator: `KeyedSerialExecutor` releases permits on an unbounded channel (bounded in practice by live-lane count) since a release sent from `Drop` cannot await, and a bounded `try_send` hitting a full channel would drop the release and wedge the lane forever.
- via-coordinator: the MCP tool context is built fresh per turn and registered by the caller, rather than upstream's one cached `AcpSessionToolServer` registration per owner that gets `update()`d, since the caching responsibility belongs with `via_mcp_tools::SessionToolServer`.
- via-coordinator: `ProjectSessionDirectory` is a seam rather than upstream's direct `client.listSessions()` ACP call, since not every harness shape supports listing; a harness that can't list makes `via_session_send` refuse explicitly rather than resume into a guessed directory.

### Phase 4 — via-conversation

[`phase-4.md`](./phase-4.md)

- via-conversation: `get_current_time`'s `local_time` is a deterministic 24-hour rendering rather than full ICU prose, since VIA has no ICU4X dependency (the catalogue itself records this gap); the instant/zone/sibling fields that `schedule_reminder` actually computes from remain exact.
- via-conversation: two dependencies (`chrono-tz`, `iana-time-zone`) were added to the root manifest for IANA zone conversion and the host's own zone-name fallback.
- via-conversation: BCP-47 validation is structural (subtag shape check) rather than `Intl.DateTimeFormat`-based; it agrees with ICU on the catalogued invalid case and every real tag, while also accepting well-formed-but-unknown tags as ICU does.
- via-conversation: `ConversationSync` is a synchronous state machine wrapped by an owning-task actor (`ConversationSyncHandle`, bounded mpsc + per-command oneshot) even though none of architecture.md §11's four ordering invariants live in this crate, since `seq`/message order are still observable.
- via-conversation: `FrontendNotesStore`/`MarkdownContextStore` use a `Mutex`, not a task, since the cross-process ordering contract is already `via_store::with_file_transaction`'s and in-process only needs mutual exclusion over a cache.
- via-conversation: the notes document is a `via_store::VersionedJsonStore` rather than upstream's hand-rolled atomic write/`version:1` envelope/corrupt-quarantine/disable-not-clobber logic, all four of which `via-store` already owns.
- via-conversation: notes order uses `IndexMap`, not a sorted map, since order is observable in file key order, list tie-breaks, and capped ambiguous-name candidates; `drop` uses `shift_remove` for the same reason.
- via-conversation: `MemoryExtractor::maybe_run` is `async` with the caller spawning it, since the transcript source is async here, rather than upstream's synchronous-gated fire-and-forget; the four "returned null" branches survive as `ExtractionOutcome::was_gated()`.
- via-conversation: the debounce is check-then-claim under a lock rather than upstream's check-then-set, since Rust concurrency doesn't get upstream's free single-threaded check-and-set; behavior for one close is identical.
- via-conversation: `ExtractorLlm`/`TranscriptSource` are traits (with the HTTP client behind an `http` feature) — the Rust-idiom substitution for upstream's `fetchImpl` parameter.
- via-conversation: `MemoryToolOutcome::changed()` reports a count and lets the caller (`via-voice`, which owns the realtime session) decide what to do, rather than upstream's callback directly into the session via `notifyMemoryChanged()`.
- via-conversation: the `memory`/`notes` tool surfaces (codes, action lists) live in this crate rather than `via-voice`'s `tool-call-handler.mjs`, so the refusal ladder is testable without a realtime session; JSON schemas/descriptions stay `via-voice`'s.
- via-conversation: two truncation units are both reproduced (`[...value].slice` by code point in most places, UTF-16 code-unit slicing for `locale`/`workingDirectory`), with `bounded_utf16` stopping one character short rather than splitting a surrogate.
- via-conversation: `speech_ngrams` slices by code point where upstream slices by UTF-16 code unit; they agree for BMP text and astral characters are stripped before the bigram pass anyway.
- via-conversation: the extractor's `USER_PREFERENCE_PATTERNS`/`EXPLICIT_DIRECTIVE_PATTERNS` classifiers are reproduced Chinese-pattern-only, as upstream's are, so English/Korean transcripts always fail the directive check (conservative in both directions); widening them would be a behavior change, not a port.
- via-conversation: `. ` in those patterns only excludes `\n` (JS also excludes `\r`/U+2028/U+2029), which is moot since prior `clean()` already collapses all four.
- via-conversation: the notes change-detector hashes with SHA-256 instead of upstream's SHA-1, even though the hash never leaves the process, since "reaching for a broken primitive to match an implementation detail would be the wrong kind of fidelity."
- via-conversation: the `<recent_conversation>` input-summary separators (` · `, `；`) are Rust constants, not `via-i18n` keys, since they're punctuation rather than prose (same call `via-process` made for its failure/stderr separator).
- via-conversation: `FrontendMemoryService::apply` deliberately does not hold a file transaction across its two document persists — a refusal on the second write still leaves the first written, but taking two cross-process locks in sequence would introduce a deadlock the single-document path (`MarkdownContextStore::edit`) doesn't have.
- via-conversation: `clear`/`drop` have no code-level confirmation, matching upstream, because the destructive-intent guard is prompt-level (the catalogued `notes` tool description) — a voice product has no turn in which to ask.
- via-conversation: `drop` still writes the owner's access timestamp for an owner it may have just removed; harmless and reproduced as-is.

### Phase 5 — apps/via

[`phase-5-apps-via.md`](./phase-5-apps-via.md)

- apps-via: `via_app::EngineFactory::open_for` plus a shared `SharedGate` (`Arc<Mutex<InjectionGate>>`) replace upstream's closure-based `ensureFrontend()` — a Rust trait can't close over a connection scope, so the four things the closure used are passed explicitly as `EngineContext`; `open_for` defaults to `open` so existing factories/tests are untouched.
- apps-via: the Gateway lease lives in `gateway::boot` (this binary), not in `via_app::Startup` — `Startup` keeps the lease handle privately and `heartbeat` needs it by value, so rather than add a second accessor, `gateway::boot` reproduces `index.mjs`'s literal ordering (gate, lease, compose, bind, publish); `Startup::detached` remains for an embedder owning both.
- apps-via: each CLI verb builds its own tokio reactor rather than using a single `#[tokio::main]` — four of six verbs need no reactor at all, so `gateway::run`/`chat::run` each construct one instead of paying pool cost for verbs like `via config`.
- apps-via: `DelegationRunner`/`CoordinatorCanceler` (not the Coordinator itself) own the cancellation race between a turn and `WorkContext::signal`, calling `CancelRequest::abort()` in both branches — an idiomatic Rust placement of upstream's `task-manager.mjs` rule ("`delegated` goes to the coordinator, everything else aborts locally") using a future-race instead of upstream's local abort call.
- apps-via: `via_voice::merge_response_context` assigns/preserves a named list of progress flags instead of reproducing JavaScript's object-spread semantics literally — achieves the same "empty patch changes nothing" behavior without a spread operator, merging on first sighting and afterward only when the provider echoed a correlation.
- apps-via: `SinkObserver` is a bounded channel plus one forwarding task rather than a `tokio::spawn` per event — `CoordinationObserver` is synchronous while `WorkEventSink::emit` is async, and a spawn per event would reorder `delegated`/`delegation.completed` under load.
- apps-via: `via chat` connects with `voiceEnabled: false` plus explicit `outputEnabled: true` instead of upstream's overloaded `voiceEnabled: true` (used only to enable output/announcements) — a clearer statement of the same resolved capability set (`{input:false, output:true, arbitration:false}`), not a behavior change.
- apps-via: `via chat` prints one dim line per Work-plane transition (`chat.task_event`) — upstream's text CLI drops all `task.*` frames entirely, an intentional addition since `via chat` doubles as the harness the Work queue is debugged with.
- apps-via: `--no-autostart` is a VIA-only flag — upstream has no analogous choice because its `tui`/desktop paths are two separate hardcoded behaviors; `via chat` is both clients at once so the choice must be expressible (default: autostart).
- apps-via: `/cancel` carries no client-side timeout, matching upstream's `api()` helper (only its health probe has `AbortSignal.timeout`) — recorded rather than "fixed" because a client timeout would report a stop nothing confirmed, violating the confirmed-cancellation invariant.
- apps-via: the Gateway's console log and the CLI banner share one stdout in a single process (unlike upstream's parent/child processes) — both write whole records so lines never interleave, but ordering isn't guaranteed, so tests locate the banner by catalogued prefix rather than position.
- apps-via: the banner's second line is upstream's `gatewaySummary()` reproduced field-for-field, with the `WebUI: {url}/` line dropped along with the (nonexistent) web UI.

### Phase 5 — via-app

[`phase-5-via-app.md`](./phase-5-via-app.md)

- via-app: this crate runs its own hyper accept loop instead of `axum::serve`, because a WebSocket upgrade to any path but `/api/realtime` must get zero HTTP response (`socket.destroy()`), which an axum handler (which can only return a response) can't express.
- via-app: Layer 3 is reached through the `backend::GatewayBackend` trait (four methods) rather than upstream's module-level singleton `agent` object queried ad hoc — a Rust-trait substitution for the duck-typed singleton, with `FrontendOnlyBackend`/`HarnessBackend` as the two implementations.
- via-app: `describe()` answers a `serde_json::Map`, not a typed struct, reproducing upstream's object-spread of `agent.describe()`+`agent.status()` so the adapter-defined key set (per `docs/rebrand.md`) stays open rather than frozen by a struct shape.
- via-app: the realtime route runs through a separate router merged in by `http::router`, carrying none of the HTTP middleware — mirrors upstream's own separation (`attachRealtimeGateway` on `server.on('upgrade')`, not on Express) since the two refusal styles (JSON 403 vs. plain-text upgrade refusal) and identity rules (issue vs. only resolve a cookie) genuinely differ.
- via-app: the model turn sits behind the `realtime::VoiceEngine` trait, splitting upstream's `realtime-gateway.mjs` into this crate's socket plumbing plus `via-voice`'s decisions — lets `dictation`/`agent`-without-harness/`interface` modes and `RecordingEngine` exercise the whole frame vocabulary with no real provider.
- via-app: `taskManager` and `taskStore` are merged into one injected `via_work::WorkManager` service (via `ServicesBuilder::work`) rather than upstream's two separately-injected objects, since `WorkManager` owns its own store.
- via-app: the permission policy is a plain `tokio::sync::Mutex` (map reads/writes, no ordering requirement) rather than an owning task — deliberately not one of the four owning-task invariants `docs/architecture.md` §11 names.
- via-app: HTTP request bodies are read through a hand-written `http::coerce` reproducing JavaScript's loose coercions (`String(x||'')`, `Number(x)`) instead of typed serde deserialization, so malformed bodies reach the catalogued 400 rather than an uncatalogued 422; the one unreproduced case (`String()` of an array/object) never matches a valid decision value anyway.
- via-app: `/api/health` gains a `sessionModes` block (no upstream equivalent) appended after the 26 contract keys, per `docs/architecture.md` §2's mode-degradation-must-be-visible requirement; the `interface`-row limitation this introduced was superseded/closed in phase 7 once `via-context` landed.
- via-app: `gatewayInstanceId`/`gatewayStartedAt` are passed as an `InstanceIdentity` value into the composition root instead of round-tripped through an environment variable (`QWEN_AUDIO_GATEWAY_INSTANCE_ID`) — `std::env::set_var` is `unsafe` in edition 2024 and a process-global would cross-contaminate identities between two Gateways in one test binary.
- via-app: an unmatched path answers a JSON `{"error":"not found"}` 404 instead of upstream's catch-all `index.html` — there is no web UI to serve, so the body reused is upstream's own `/skins` 404 body; named at `http::FALLBACK_IS_JSON_404`.
- via-app: an origin rejection deliberately carries no `X-Request-Id` header, reproducing upstream's `enforceSameOrigin` returning before `next()` runs (which is what sets the header) — recorded explicitly (with a test asserting the absence) so it can't be "fixed" by accident.
- via-app: request log correlation (`requestId`/`ownerId`) is attached as a typed request extension via `identity_layer`, read explicitly by handlers, instead of upstream's ambient `AsyncLocalStorage` that follows every `await` — a thread-local can't do that in Rust (also recorded in via-log's own deviations), so a handler that forgets is missing fields rather than correlating the wrong ones.
- via-app: raw-socket upgrade refusals carry two extra headers (`content-length`, `date`) that hyper adds automatically, vs. upstream's exactly-three-header-line raw write — status, `connection: close`, content type and body (everything a client parses) are identical.
- via-app: `VoiceClientRegistry::activate` sends `playback.clear` then `voice.deactivated` (carrying the replacement's descriptor) itself, rather than upstream's `deactivate(replacement)` doing so — a `&dyn VoiceClient` trait object can't supply a descriptor, so the registry (which can see the winner) sends both frames instead.
- via-app: the offline hand-off uses a channel instead of Electron's `parentPort.postMessage` (VIA has no Electron host); the `error` field is typed `Option<Option<String>>` to faithfully preserve upstream's three real states (absent for progress, `null` or a string for terminal) rather than collapsing to two.
- via-app: offline timers need no injected `setTimer` seam — `#[tokio::test(start_paused=true)]` advances every `tokio::time::sleep` at once, so (unlike upstream, which injects a timer seam purely for its own tests) the production code path is what's under test.
- via-app: `Startup::detached` exists (skips the gate and lease) purely for `via chat`'s in-process Gateway, which has already taken the CLI lease — no upstream equivalent since upstream never runs Gateway and CLI in one process.
- via-app: `/skins/*` and `express.static(web/dist)` are dropped entirely (no web UI ships) rather than ported — recorded in `via_protocol::DROPPED_UPSTREAM_CAPABILITIES` rather than advertised, a permanent scope decision, not a to-do.
- via-app: construction of `MemoryExtractor`, `MarkdownContextStore`, `ReminderScheduler` and `BackendAvailability` (inline in upstream's `gateway-application.mjs`) moves to their own builders in the already-existing `via-conversation`/`via-work`/`via-backends` crates; the composition root takes them pre-built, which is what makes `Services` replaceable field by field.
- via-app: `x-powered-by` has no VIA counterpart to disable — axum never sets such a header, so there's nothing to port.

### Phase 5 — via-realtime-dashscope

[`phase-5-via-realtime-dashscope.md`](./phase-5-via-realtime-dashscope.md)

- via-realtime-dashscope: both DashScope and speech-to-speech providers ship from one crate rather than the two crates `docs/architecture.md` §15 names — `s2s.mjs` is 138 lines needing no new dependency, and upstream itself registers the pair as one built-in set, so splitting would invent a seam the reference implementation doesn't have.
- via-realtime-dashscope: each provider is constructed from a narrow settings struct (`DashScopeSettings`/`SpeechToSpeechSettings`) it's handed, rather than closing over upstream's global `config` singleton re-read on every call — motivated by testability, a checkable dependency statement, and (since the trait requires `Debug`) keeping the credential, a `via_core::Secret`, unprintable.
- via-realtime-dashscope: the three response-instruction sentences are read via `via-i18n` keys (a small local accessor) rather than imported from `via-voice`'s `frontend-tools.mjs` equivalent, because `via-voice` sits above the provider crates in the dependency graph and importing it would invert that graph.
- via-realtime-dashscope: a provider carries its own `Locale` field (for two message-building trait methods) instead of upstream's single hardcoded locale, since the model-visible instructions it composes must reflect the session's own locale.
- via-realtime-dashscope: `configuration_signature()` is computed independently by each provider (each builds its own `via_catalog::RealtimeIdentity`) rather than upstream's registry-level fallback to whichever provider is currently active, since VIA's trait requires each provider to answer for itself.
- via-realtime-dashscope: the provider's public label is `Qwen-Audio-Realtime`, distinct from the catalog/configuration label `DashScope` — reproduces upstream's own internal disagreement between two labels verbatim, per `docs/rebrand.md`.
- via-realtime-dashscope: `patterns_compile()` returns `Option<RegexSet>` instead of an `expect()`-or-panic — added robustness with no upstream equivalent, so a compile failure degrades classification to the user-visible `Other` class rather than crashing.
- via-realtime-dashscope: error-classification patterns are written ASCII-lowercase with `(?-u:\b)` and the haystack lowercased via `to_ascii_lowercase`, deliberately not using Rust's Unicode-aware `(?i)` — matches JavaScript's ASCII-only `/i` flag (without `/u`) exactly; an exactness measure to avoid an accidental behavior mismatch, not a functional change.

### Phase 5 — via-realtime-mock

[`phase-5-via-realtime-mock.md`](./phase-5-via-realtime-mock.md)

- via-realtime-mock: `RealtimeSession::open` (not `connect`) is the mock's entry point — the session above the mock is the real session (same tasks/correlation/watchdogs), only the transport differs; `RealtimeSession::connect` on the mock intentionally fails since `url()` names no real socket (`mock://in-process`).
- via-realtime-mock: the script server is an owning task communicating over a bounded `mpsc` with `oneshot` replies, not a mutex over a transcript — needed because three invariants (step cursor, id counters, emission queue) move together and a mutex could hand a concurrent reader a torn snapshot.
- via-realtime-mock: scripted emission delays are one FIFO queue with cumulative deadlines (each relative to the previous emission), not N independent timers — guarantees arrival order is never a function of the scheduler, which is the whole point of a deterministic mock.
- via-realtime-mock: a script is expressed as matching rules (`ScriptStep{on,repeat,emit}`), not a linear tape, so behavior like "answer only the second `response.create` differently" is expressible; `Script::turn`/`turns` remain linear-tape sugar over it.
- via-realtime-mock: with no matching step, the mock still behaves like a working provider (acknowledges `session.update`, echoes item creation, answers `response.create`) instead of staying mute — an empty script is a functioning default rather than requiring every test to spell out its own handshake.
- via-realtime-mock: `EventLog`/`MockSession::with_event_log()` is public API, not `#[cfg(test)]`-gated — every session-driving test needs the same drainer to respect via-realtime's "don't await from the same task that drains" rule, so it's provided once rather than reimplemented per test.
- via-realtime-mock: `Script` deserializes through a private wire struct so `provider` can be optional on the wire but never `Option` on the struct — avoids a Rust/serde pitfall where `#[serde(default)]` on a plain field would silently hand a `dictation`-kind fixture (which should mount no model) a conversing default provider.
- via-realtime-mock: `ProviderCapabilities` flags are load-bearing behavior, not mere declarations — four of five flags each change concrete script-server behavior (withholding `session.updated`, refusing a racing `response.create`, echoing correlation, replacing item ids); the fifth (`per_response_instructions`) is asserted to have no session-side behavior at all, guarding against the declaration going stale.
- via-realtime-mock: `response_metadata_correlation` is gated on dialect as well as the capability flag, since the beta dialect has no portable correlation contract — a mock that echoed metadata the beta dialect can't actually carry would validate against a shape that never travels on the wire.
- via-realtime-mock: emitted ids are auto-stamped (response id, item id) unless a step opts out via `Stamp::Verbatim` — the opt-out exists specifically to let tests construct the adversarial "no response.id at all" case that `via-realtime` had to correct-not-copy from upstream.
- via-realtime-mock: the one-response-slot refusal is service state checked before the script is consulted and consumes no step, letting the session's busy-retry ladder replay into the still-unconsumed step.
- via-realtime-mock: a new generic i18n key `realtime.connect_timeout` (with a `{label}` placeholder) was added because the catalog only had product-named connect-timeout keys (`..._dashscope`, `..._speech_to_speech`); modelled on the DashScope one for reuse by `local-omni`/`openai`.
- via-realtime-mock: `script::messages` (the six simulated third-party error phrases) deliberately bypass `via-i18n` entirely — translating them would break their purpose, which is to be matched by the two real providers' `classifyError` corpora byte-for-byte.
- via-realtime-mock: `MockError` has no `message(locale)` — every other VIA error type splits a developer sentence from a user sentence, but a `MockError` means a broken test fixture, which has no end-user audience.

### Phase 5 — via-realtime

[`phase-5-via-realtime.md`](./phase-5-via-realtime.md)

- via-realtime: `response-lifecycle.mjs` (41 lines upstream, in `server/src/voice/`) is ported into `via-realtime`, not `via-voice`, because the crate that owns the session has to own the predicate the session branches on; `via-voice` reaches it through `via_realtime::lifecycle`.
- via-realtime: `AgentContext` carries an already-composed session payload (`instructions`/`tools` built by `via-voice`, `extra` for the rest) rather than upstream's raw `agentContext` object, so the transport layer never depends on the prompt layer.
- via-realtime: `project_user_input`'s default falls back to a caller-composed `fallback_text` rather than upstream's `frontendInputProjection` (prompt-layer text from `shared/input-parts.mjs`), keeping prompt composition out of the transport crate.
- via-realtime: `RealtimeProvider::preflight()` is the seam for upstream's `modelProfile?.family === 'unknown'` gate, phrased as the provider raising the check itself because `via-catalog` deliberately has no `unknown` family to inspect; ordering (`preflight()`, then `is_configured()`, then the socket) is kept.
- via-realtime: the output queue is two owning tasks (`session/state.rs` for socket+correlation, `session/queue.rs` running one job at a time) rather than upstream's one promise chain, because running a job inside the state task would deadlock on its first await; a two-step upstream sequence (`whenIdle()` then check-and-register) becomes one atomic `BeginResponse` command to close the race window between them.
- via-realtime: `Transport` is an injectable seam — unlike upstream's inline `ws` socket construction inside `connect()` — so the session, correlation, queue and watchdogs are testable without a real socket; `RealtimeSession::open` takes any `Transport`.
- via-realtime: the four separate callbacks (`onEvent`/`onError`/`onDiagnostic`/`onClose`) become one ordered `SessionEvent` stream on a bounded channel, because the order between them is itself information (an error before `Closed` explains the close).
- via-realtime: `ResponseContext` is kept opaque (three accessors: `turn_id`, `task_id`, `authorization_id`) rather than typed like upstream's arbitrary `context` object, so the transport layer never depends on the Gateway's turn model.
- via-realtime: `testing` ships as a published module rather than `#[cfg(test)]`, since `via-realtime-mock`, `via-realtime-dashscope` and `via-voice` all need to drive a session without reimplementing a provider.
- via-realtime: `emit` takes `&mut self` rather than a shared borrow, because the state owns a `dyn Sink` that is `Send` but not `Sync`; costs nothing since every caller already holds the exclusive borrow.
- via-realtime: four of upstream's five `provider-registry.mjs` validation loops (190 lines proving an object literal is shaped like a provider) become compile-time guarantees via the type system — `ProviderCapabilities`, `via_catalog::{ModelCapabilities, TransportCapabilities}`, `Visibility`, `TurnDetectionKind` — with the method-name lists kept as constants only because `via-conformance` asserts vocabulary, not mechanism.
- via-realtime: `pending_responses` can hold at most one entry through the public API (a real FIFO queue vs. upstream's promise chain a test can push into), making upstream's "Realtime 响应关联冲突" correlation-conflict guard unreachable; kept anyway in both places upstream has it as fail-closed behavior for a future caller that registers outside the queue.
- via-realtime: `audio_commit()` is added — upstream never commits an input buffer explicitly since both of its providers do server-side turn detection; VIA's `dictation` mode and push-to-talk clients need the explicit form.
- via-realtime: `ActionKind::Barrier`, reached as `RealtimeSession::drain()`, is the Rust spelling of upstream's `await frontend.outputQueue`, which JavaScript lets callers reach into directly.
- via-realtime: `SessionMode` gating is added (VIA-specific) — every response-creating call answers `skipped / no_model_turn` in `dictation` without touching the socket, and `restore_recent_conversation` is skipped there too.
- via-realtime: `OutcomePhase::NoModelTurn` is added, the one phase upstream does not have, to support `SessionMode` gating.
- via-realtime: `ErrorClass` is added as a closed enum (vs. upstream's `classifyError` free string compared against seven literals), plus `is_suppressed()` consolidating the Gateway's own branch table in one place.
- via-realtime: `STABLE_CONNECTION_MS` is added beside the reconnect ladder so the catalogued "reset backoff only after >=10000ms up" rule lives with the ladder instead of being restated by the caller.
- via-realtime: `ReconnectBackoff`'s jitter source is injected (defaulting to `getrandom`, as upstream's `random` option is); a CSPRNG failure returns `0.5` rather than panicking, degrading only the spread (equivalent to `jitterRatio: 0`).
- via-realtime: `RESPONSE_CORRELATION_KEY` stays `qwen_audio_request_id` — a `docs/rebrand.md` KEEP, since the key travels to and is echoed back by a third-party endpoint VIA does not control.
- via-realtime: one i18n key (`realtime.unsupported_frontend_with_options`) was added to restore the options-list upstream's message includes but the shipped catalogue had dropped, with locale-appropriate list joiners (`、` in `zh`, `, ` in `en`/`ko`).
- via-realtime: three error messages (`NotConfigured`, `ConnectTimeout`, `Transport`) are deliberately not localized — the first two carry the provider's own sentence (as upstream reads `provider.missingConfigurationMessage`/`connectTimeoutMessage`), and the transport message is a contract the catalogued DashScope `fatal` regex depends on matching verbatim.
- via-realtime: a note recorded for `via-realtime-dashscope` — `tungstenite` phrases a refused WebSocket upgrade differently than Node's `ws` does, so a Rust provider's `classify_error` must cover both phrasings or an expired credential is misclassified as retryable instead of fatal; pinned by a dedicated status-line test.

### Phase 5 — via-voice

[`phase-5-via-voice.md`](./phase-5-via-voice.md)

- via-voice: `via-realtime` owns the provider/session-machine seam entirely; `via-voice` declares only what it consumes (`provider::ProviderView`, `frontend::VoiceFrontend`), implemented over `via_realtime`'s session by `via-app` — avoids recompiling every gateway branch on a transport change. (`ProviderCapabilities` temporarily restates `DEFAULT_CAPABILITIES` since two flags are branched on here; becomes a `From` impl once `via-realtime`'s equivalent stabilizes.)
- via-voice: three Layer-3 facts upstream reads directly (`BackendAvailability`, `PermissionResponder`, `DelegationRunners`) arrive here as traits, implemented in `via-app` over `via_backends`/`via_coordinator`, because `via-arch-test`'s `layer1_does_not_reach_layer3_directly` forbids Layer 1 from reaching Layer 3 directly.
- via-voice: `RealtimeFrontend` (the session machine) is not ported into this crate — architecture.md §3 places it in `via-realtime`; this crate ports only what sits above a normalized event.
- via-voice: the gateway's socket plumbing (`attachRealtimeGateway`'s WS server, the `send()` helper, the frame codec) belongs to `via-app`; `session.rs` carries only the decisions that plumbing makes (`upgrade_decision`, seven timing constants, `TurnTracker`), testable without a socket.
- via-voice: `schedule_reminder.execute_at` accepts RFC 3339 plus two naive forms only, narrower than everything `Date.parse` accepts, since the tool's schema says ISO 8601 and the model is told to compute values from RFC-3339 output.
- via-voice: `backend_unavailable (disconnected)` instructions are composed from two catalog keys (`head + " " + tail`) rather than duplicating a third catalog entry, to avoid un-sharing text another consumer of the shared key also renders (the space-vs-`\n` join is `via-i18n`'s own choice, not re-litigated here).
- via-voice: `accepted_instructions` is composed by slicing the shared sentence pair back out of the `duplicate_submission` catalog key rather than storing a fourth, separate sentence pair.
- via-voice: `InputArbitration::status()` sorts holders by `(since, owner)` via a stable sort, since the actor holds a `HashMap` (unlike upstream's insertion-ordered JS `Map`); judged strictly better for the health-diff use case, though not upstream's exact tie-break order.
- via-voice: `SessionPermissionPolicy`'s key (`ownerId\0sessionId`) is non-injective for arbitrary NUL-containing input, same as upstream's scheme — recorded as intentional rather than fixed, since neither an owner id nor a session id can carry a NUL in practice and changing the encoding would alter a stored key shape for no reachable gain.
- via-voice: a UTF-16 bound (`bounded_utf16`) stops one unit short of a surrogate pair rather than splitting it, since Rust's `String` cannot hold the unpaired surrogate that JavaScript's `.slice(0, n)` can produce.
- via-voice: the four literal regexes are held as `Lazy<Option<Regex>>` rather than `expect`, per the workspace's no-unwrap rule; each call site fails toward its own safe default (deny a correction, or refuse an attachment).
- via-voice: `clean` explicitly reproduces ECMAScript's `\s` whitespace set (includes U+FEFF, excludes U+0085) rather than Rust's `White_Space`, since the two disagree in both directions.
- via-voice: `with_attachment_anchors` deliberately reproduces upstream's `-1` sentinel (from `indexOf`) rather than "correcting" it to `None`, to stay compatible with OpenCode clients that expect the sentinel.
- via-voice: `SessionMode` dispatch (`mode.rs`/`ModePlan`) is new to VIA — resolves requested-mode plus harness-availability into an effective mode and reports both degradations rather than applying them silently; the `interface` row was `direct`/`context_engine_unavailable` at phase 5 and is superseded/resolved in phase 7 once `via-context` landed (closed/resolved).
- via-voice: `gate::InjectionGate` names as one object what upstream spells out at four separate call sites, and adds `via_audio::PlaybackCursor` as a second opinion so a `playback.ended` receipt lost to a dropped socket doesn't leave the gate stuck.
- via-voice: `zh` frontend-agent assets are upstream's `config/frontend-agent/` byte for byte except the single mandated rebrand string (`你叫千问Audio。` → `你叫 VIA。`).
- via-voice: `en`/`ko` frontend-agent assets are authored peers, not machine translations, with structural parity (headings, sub-headings, tool/tag/field names) asserted by test rather than assumed.
- via-voice: a delivery attempt is spawned outside the announcement actor's loop rather than awaited inline, so a `playback.started` receipt, a barge-in, or a `pause` can interrupt it while it's in flight — upstream gets the equivalent behavior for free from the JS event loop.
- via-voice: `SleepController` and `InputAssetRegistry` hold a `std::sync::Mutex` rather than an owning task, since neither has an ordering invariant and both are read far more than written; a poisoned lock is recovered rather than propagated so the worst case is a slightly late sleep rather than dropping a live voice session.

### Phase 6 — via-realtime-openai

[`phase-6-via-realtime-openai.md`](./phase-6-via-realtime-openai.md)

- via-realtime-openai: the event decoder is deliberately not ported — ARGO's `decode_server_event` maps the wire into ARGO's own `LiveInboundEvent` vocabulary, but VIA's Layer 1 already consumes provider events directly and handles both pre-GA and GA event-name spellings, so porting the decoder would just be a second vocabulary for the same events.
- via-realtime-openai: ARGO's `is_benign_cancel_race` judgment (not its mechanism) becomes an `ErrorClass::NoActiveResponse` classification rather than a swallow, since `via-realtime` already suppresses that class.
- via-realtime-openai: the connect walk returns a `Transport` and `RealtimeSession::open` is the door — `connect::connect` walks/probes candidates and hands back a proven transport (`connect::open_session` is walk-then-open); `RealtimeProvider::url()` still answers with the first candidate for callers going through `RealtimeSession::connect` directly.
- via-realtime-openai: the schema repair and the duplicate-`response.create` conflict filter live below the session machine (`connect::InboundFilter`), because an error arriving before the session is ready would reject `connect()` and kill the very session the repair exists to fix.
- via-realtime-openai: ARGO's two-constructor `Dialect` (`new`/`azure`) becomes a single `OpenAiSettings::dialect` field, defaulted from the host via `OpenAiDialect::for_host`, since VIA has one provider key (`openai`) rather than ARGO's provider-label-based construction.
- via-realtime-openai: ARGO's `RealtimeProtocol` (the payload-schema question) is renamed `RealtimeSchema` to avoid colliding with `via_realtime::RealtimeProtocol` (the wire-dialect trait); the `Debug` spelling of each variant is unchanged so ARGO's log markers still read the same.
- via-realtime-openai: a second GA-discriminator rejection is swallowed via a new `InboundVerdict::DropRepaired` — needed because VIA has two GA payloads in flight (the walk's probe and the session machine's own `session.update`) where ARGO has one, so a second rejection is a knock-on from an already-in-flight payload rather than a new problem; nothing is resent.
- via-realtime-openai: the walk's budget (`ConnectBudget`) is derived from the caller's deadline, scaling ARGO's two timing constants only when the caller's window is tight, since aborting the walk from outside would otherwise drop the rejection-list record of how far it got.
- via-realtime-openai: `transcribe_input` defaults to `true` (ARGO defaults `false` to avoid billing an extra model), since VIA's `dictation` session mode makes the transcript the session's entire output.
- via-realtime-openai: `ws://` is reachable (opt-in, only for an endpoint explicitly written as `ws://`/`http://`) versus ARGO hard-coding `wss://`, since VIA already ships a plaintext loopback default for another provider.
- via-realtime-openai: `response_metadata_correlation` stays `false` at the catalogued baseline, since `via-realtime` reads capability flags once at construction, before the endpoint reveals GA vs. pre-GA; only `single_response_slot`/`per_response_instructions` differ from `DEFAULT_CAPABILITIES`.
- via-realtime-openai: `OpenAiSettings::from_frontend` substitutes OpenAI's own defaults into the shared `RealtimeFrontend` fields, which are named/defaulted for DashScope, since `via-core` (shipped) owns the one shared env-var chain and cannot be edited to add `openai_*` fields.
- via-realtime-openai: the retired `gpt-4o-realtime-preview*` family is documented but not enforced, since an Azure deployment name is arbitrary and a name check would break the ordinary Azure deployment case.
- via-realtime-openai: an OpenAI-specific quota sentence is classified `fatal` here versus `other` in the shared DashScope-authored corpus (correct per endpoint, since the wording is provider-specific); DashScope's own quota sentences remain recognized here too since litellm relays its upstream's prose verbatim.
- via-realtime-openai: the `session.update` payload honours `configured` (sends the full negotiated payload only once) unlike ARGO, which resends the whole payload every generation — because OpenAI refuses a voice change once audio has been generated; the GA discriminator still rides every update, and the schema-repair payload is always a full, byte-identical resend.
- via-realtime-openai: tools are flattened (not nested under `function`, DashScope's convention) in both pre-GA and GA generations, matching ARGO's own payload builder; a tool not in the nested shape is passed through unchanged rather than silently dropped.
- via-realtime-openai: the input sample rate is 24000, not the catalogued 16000 for dashscope/s2s, since the OpenAI Realtime input buffer is 24kHz PCM16 in both generations and GA accepts no other rate.
- via-realtime-openai: the three model-visible instruction sentences are reached via `via-i18n` keys directly rather than imported from `via-voice`'s ported `frontend-tools.mjs`, since `via-voice` sits above the provider crates and importing would invert the crate graph (ARGO's own realtime provider has no injections at all).
- via-realtime-openai: `CreateAccounting` names and encapsulates what ARGO leaves as a bare `AtomicUsize` (`creates_in_flight`) shared between the sink and inbound task, making the saturating decrement and `conflict_is_ours` logic independently testable.
- via-realtime-openai: `ProbeVerdict`/`InboundVerdict` split the decision from the I/O that ARGO inlines as `match` arms inside a 470-line `open()`, so probe/conflict logic is assertable without a socket.
- via-realtime-openai: `OpenAiDialect::parse` additionally accepts `azure-openai` and `azure_openai` alongside `azure`, matching the provider-label string ARGO's own app used, so a migrating operator doesn't need to discover a third spelling.

### Phase 6 — via-wake-word

[`phase-6-via-wake-word.md`](./phase-6-via-wake-word.md)

- via-wake-word: the crate is split at a feature boundary (default pipeline / `sherpa` engine / `http` fetcher) per `docs/adr/0001-placement.md`, so `cargo build`/`cargo test` need no C toolchain, prebuilt native library, or network by default — a build-organization difference upstream (a single Node module) has no counterpart for.
- via-wake-word: `DetectionConfig::sample_rate` is a typed `SampleRate::HZ_16000` identity, asserted via `tests/contracts.rs` against the catalogue rather than compared as a bare literal.
- via-wake-word: there is no default wake phrase anywhere — upstream hard-codes `config.wakeWord` with no override; VIA ships no constant, `Default`, or fallback (per architecture.md §16's "a configuration value, never a literal"), and an unconfigured phrase resolves to the handled state `DisabledReason::NoPhraseConfigured`.
- via-wake-word: the token/pronunciation line is configuration alongside the phrase itself (`WakePhrase` carries both and refuses to guess one from the other), since which alphabet a phrase uses (ARPAbet vs. tone-marked pinyin) is a property of the model's own symbol inventory and can't be derived.
- via-wake-word: per-locale keyword sets use a `BTreeMap<Locale, Keyword>` (`MAX_KEYWORD_MODELS = Locale::ALL.len()`); two locales may share one model artifact, deduplicated by artifact id — a multi-locale feature upstream has no counterpart for.
- via-wake-word: added — the announced phrase is cross-checked against the keyword model, reporting a mismatch as `DisabledReason::PhraseMismatch`, a failure mode only reachable because VIA's phrase is configurable (upstream has nothing to disagree with).
- via-wake-word: `realtime.asleep` is filled from the resolved (possibly per-locale) phrase via a `{wake_word}` placeholder, versus upstream's literal-interpolated sentence.
- via-wake-word: `keywords.txt` is rewritten on every install call (the completeness check instead validates the four downloaded archive members) so that changing `VIA_WAKE_WORD` doesn't leave the detector silently listening for the old phrase — a scenario upstream's hard-coded phrase never creates.
- via-wake-word: the four downloaded archive members are validated before the keyword file is written (upstream's single completeness check runs after), so a broken archive is reported as itself rather than surfacing as a token-guard I/O error.
- via-wake-word: an empty keyword file is refused before the download completes — a state only reachable because the phrase is configuration; upstream cannot reach it.
- via-wake-word: the model digest comparison ignores hex case, so a digest configured in upper case still verifies (the value is public, so there is no side-channel concern).
- via-wake-word: the HTTP client is an injected trait (`ModelFetch`: `ScriptedFetch` in the default build, `HttpModelFetch` behind `http`), making every failure branch (404, empty 2xx, transport failure, truncated archive, bad digest, hostile member name) testable with no network.
- via-wake-word: `keywordsBufSize` isn't a field VIA carries at all — the Rust binding derives the length from the buffer, where the Node binding needs a paired length argument.
- via-wake-word: VIA keeps the keyword buffer (not a path, as upstream's WASM-motivated code does) for a different reason: the phrase is configuration and can change between runs against one already-installed model.
- via-wake-word: added — a resampling front end (`WakeWordStream`) sits between the socket and the detector, built from `WakeWordDetector::sample_rate()` rather than assuming 16 kHz, since upstream's host always captured at exactly the extractor's rate.
- via-wake-word: a torn PCM frame is carried forward (via `FrameBuffer::append_pcm16le`) rather than dropped, because a streaming front end receives arbitrary chunk boundaries where dropping a trailing byte would corrupt every following sample — unlike upstream's stateless one-shot `pcm16le_to_f32` decoder, which does drop it; both behaviors are asserted side by side.
- via-wake-word: `WakeWordStream` additionally never calls the detector on an empty chunk (beyond upstream's own early return), since an FFT resampler can legitimately buffer without yet producing output.
- via-wake-word: `ScriptedDetector`/`ScriptedFetch` ship in the default build rather than behind `cfg(test)`, so `via-voice`/`via-app` can exercise "the user said the phrase" without a real detector.
- via-wake-word: `bzip2` (pure-Rust `libbz2-rs-sys` backend, toolchain-free) and `tar` (`default-features = false`, dropping `xattr`) are added to the root manifest to open the verified `.tar.bz2` archive.

### Phase 7 — via-context

[`phase-7.md`](./phase-7.md)

- via-context: The cross-crate equality proof (`pack.instructions() == via_voice::assemble_frontend_instructions(...)`) lives in via-voice's tests as a `[dev-dependencies]` edge, not in via-context, because only via-voice — an existing, named crate — can see both sides of the equality.
- via-context: Keeps ARGO's three-tier `PromptTier` naming/semantics but gives `SemiStatic` a new VIA-specific occupant (the ordered-deixis view) since VIA declares tools via `session.update` rather than in-prompt, which would otherwise leave the tier empty.
- via-context: Generalizes ARGO's tag-specific fence escaping into a blocklist-free "anything that could open a tag" pattern (VIA's six-plus model-visible tags vs ARGO's one), uses structural open/close sentinel lines instead of named framing, and applies the length cap before neutralization (per ARGO's own past `#2069` review lesson) rather than after.
- via-context: `<runtime_context>` and `<recent_conversation>` are deliberately left unfenced (fields are already sanitized upstream / the user's own words sit at the top of the prompt hierarchy), unlike the general fence-everything-untrusted rule.
- via-context: Fence-sentinel neutralization runs only on untrusted bodies, so a sentinel literal inside trusted text (e.g. a user's `USER.md`) is left verbatim — an asymmetric, deliberately one-directional safety property (a stray marker can only downgrade trust, never escape a real fence).
- via-context: Introduces "referents" (`Screen`/`Input`/`Work`/`NoteList`/`Directory`) with a 2-turn lifetime, monotonic never-reused handles, and 4 staleness reasons (`Expired`/`Moved`/`Gone`/`SurfaceUnknown`) — an entirely new capability with no ARGO or qwen-audio-agent equivalent.
- via-context: Introduces ordered deixis (enumeration follows the host's own order; a snapshot generation bump is the only thing that renumbers; truncation at 50 objects refuses to guess `Ordinal::Last`) — a new capability with no upstream equivalent; the screen enumeration is treated as third-party and fenced.
- via-context: Token estimation is not ARGO's `chars / 4` heuristic; it charges ASCII code points at 4-per-token and everything else (e.g. Han characters) at 1-per-token, adapted for VIA's multi-locale catalog.
- via-context: `ReferentRegistry`/`ContextEngine` deliberately use plain `&mut self` state with no owning task or interior mutability (unlike the four FIFO-ordered invariants in architecture.md §11), because a per-session referent registry has no cross-task ordering requirement.
- via-context (closed/resolved): `SessionMode::Interface`'s `ContextEngineUnavailable` degradation, previously reported unconditionally by via-voice because via-context didn't exist yet, is now conditional/real — `ModePlan` gained `with_context_engine`, and via-voice/via-app health/test-route/engine code was updated accordingly; a deliberate wiring change, not a bug fix.

### Phase 8 — via-realtime-local

[`phase-8-via-realtime-local.md`](./phase-8-via-realtime-local.md)

- via-realtime-local: Implements the on-device path as a cascade (VAD→ASR→LLM→TTS via sherpa-onnx/llama.cpp) instead of a single omni model runtime, forced by three verified constraints — Qwen3.5-Omni has no open weights (DashScope-API-only), no Rust runtime (candle/mistral.rs/llama.cpp/mlx-rs) runs the Qwen3-Omni Talker, and sherpa-onnx bundles VAD+ASR+TTS behind one interface.
- via-realtime-local: Native deps (sherpa-onnx, llama-cpp-2) are gated behind optional `sherpa`/`llama` cargo features (mirroring via-wake-word's trait-boundary pattern) so default builds/CI need no C toolchain; `tests/sherpa.rs`/`tests/llama.rs` compile to empty binaries without the feature rather than failing.
- via-realtime-local: One owning task services three sources (inbound, generation, synthesis) with `biased` select and inbound read first, so barge-in audio is never processed after the turn it should interrupt.
- via-realtime-local: Cancellation is implemented as `drop`-ing the `ResponseStream`/`SpeechStream`, with no `cancel()` method on the trait, so there is only one way to stop a turn.
- via-realtime-local: Speech synthesis is per-sentence and overlaps generation (rather than waiting for the full reasoning turn) to bound time-to-first-audio.
- via-realtime-local: Sentence splitting treats `.`/`!`/`?` as terminators only when followed by whitespace (so `3.14`/`example.com` don't split) and caps unpunctuated runs at `MAX_SENTENCE_CHARS`; an admitted, tested false-boundary case is an abbreviation followed by a space (e.g. "e.g. ").
- via-realtime-local: Barge-in emits `input_audio_buffer.speech_started` before dropping the streams and before `response.done(cancelled)`, so the caller's outcome settles immediately instead of waiting on the 120s inactivity watchdog.
- via-realtime-local: Uses two `PlaybackCursor`s (one on input audio, one on the live response) so `audio_start_ms`/`audio_end_ms` and `conversation.item.truncate`'s position are real session-audio positions, not wall-clock guesses.
- via-realtime-local: A single response slot backed by a 2-deep queue (rather than refusing concurrent creates) since there is no remote service to say no; a third request is refused with ARGO's catalogued conflict vocabulary.
- via-realtime-local: The response queue is drained once per run-loop iteration rather than recursively at each slot-freeing site, avoiding an unbounded async call graph.
- via-realtime-local: The two machine-readable `error` wire messages are deliberately left un-localized (`classify_error` and via-realtime-openai's corpus match on them literally); all human-facing text still goes through via-i18n/`LocalError`.
- via-realtime-local: The inbound channel is bounded (client-driven backpressure is safe) while the outbound channel is unbounded, because the machine must never block on write or the session deadlocks.
- via-realtime-local (`LocalEndpointProvider`): `is_configured` checks for a configured endpoint rather than delegating to the inner via-realtime-openai client's API-key check, since a loopback local server needs no credential and via-catalog's `local-omni` row declares no default URL.
- via-realtime-local (`LocalEndpointProvider`): Builds its own `headers` (bearer only if present, plus subprotocol) instead of delegating, because the inner provider sends an empty `api-key:` header that a strict local server could reject with a confusing 400.
- via-realtime-local (`LocalEndpointProvider`): `transcribe_input` defaults to `false` (vs via-realtime-openai's default-on) because a local server produces its own transcript and `whisper-1` is an OpenAI-hosted id it likely lacks; explicitly flagged as "the deviation most likely to be wrong" since it can't be verified against real local-server software in this repo.
- via-realtime-local: `diagnose_connect_failure` turns a refused TCP connect / TLS failure into a plain-language local-server diagnosis while leaving an HTTP-status ("answered") failure untouched, with `ANSWERED_MARKERS` deliberately excluding the walk's own aggregate wrapper so a partially-answered walk still reports the status.
- via-realtime-local: Local mode (`Pipeline` vs `Endpoint`) is derived from whether an endpoint is configured rather than read from a dedicated env var, since via-core ships no local-mode variable and this phase doesn't edit via-core; `LocalSettings::with_mode`/`LocalMode::parse` exist as an override point for a future config surface.
- via-realtime-local: The model root is derived from `InstallPaths` (`<config>/models/local-omni`) rather than an environment variable, mirroring via-wake-word's pattern.
- via-realtime-local: A `dashscope_realtime_url` equal to the DashScope default is read as "unset" (reusing via-realtime-openai's convention) so it can't accidentally point local endpoint mode at a cloud host nobody chose.
- via-realtime-local: The pipeline's model id is the reasoning GGUF file's filename stem (no fixed id table), and `WeightsSet::discover` refuses to pick between two `.gguf` files rather than depending on directory order.
- via-realtime-local: The local/pipeline-vs-endpoint mode is included in the configuration signature so switching modes (different sample rates/voice/server requirement) correctly invalidates cached clients.
- via-realtime-local: `StageOrigin` (`Weights` vs `Supplied`) makes "does this provider need real model files" a declared fact rather than an implicit guess, letting CI run the full pipeline against `Supplied`/`scripted` stages without a 9-file fixture tree.
- via-realtime-local (sherpa binding): Wraps the VAD's continuous "is speech detected" signal into a single `Started` edge per utterance (rather than passing through the level), since the state machine keys barge-in on it.
- via-realtime-local (sherpa binding): Publishes the recognizer's running transcript as a delta rather than resending the whole current hypothesis verbatim (the underlying API returns the full hypothesis on every call).
- via-realtime-local (sherpa binding): Recognizer endpointing is turned off because the pipeline's own VAD already owns turn detection, avoiding two disagreeing endpoint detectors on one microphone.
- via-realtime-local (llama binding): Buffers partial UTF-8 token bytes rather than using `from_utf8_lossy` per token, avoiding replacement characters mid-glyph for CJK/emoji spans that cross multiple tokens.
- via-realtime-local (llama binding): `LlamaBackend::init` is enforced once-per-process via a `OnceLock` (answering `BackendAlreadyInitialized` otherwise), with the first initialization's error reported to every subsequent caller.
- via-realtime-local: `SherpaSpeaker::with_voice` selects a Kokoro voice by name or index and falls back to speaker 0 for an unrecognized value rather than failing the utterance, treating a wrong voice as recoverable and silence as not.
- via-realtime-local: No `cpal` (audio device) dependency — capture/playback are deliberately left to the host shell so only one component ever opens the microphone.
- via-realtime-local: No model downloader is shipped (unlike via-wake-word's pinned-artifact downloader), because GGUFs/voice packs are operator-chosen from operator-chosen hosts with no safe single digest to pin.

### Phase 9 — via-e2e

[`phase-9-via-e2e.md`](./phase-9-via-e2e.md)

- via-e2e: Introduces a test-only Gateway binary (`via-gateway-e2e`) that calls the same `via::gateway::serve` as the shipped `via gateway` but swaps two composition seams (a scripted `SessionOpener` and a scripted `DownstreamAgent`) read from files on disk, rather than duplicating the in-process coverage `apps/via/tests/milestone.rs` already provides.
- via-e2e: Resolves the shipped `via` binary's path via `support::via_binary()` (building it on demand) instead of `CARGO_BIN_EXE_via`, because cargo only sets that env var for the declaring package's (`apps/via`'s) own tests, not for via-e2e.
- via-e2e: Declares `via` and `via-conformance` as `{ path = ... }` dependencies rather than `{ workspace = true }`, to avoid adding an App-band binary and a Tests-band crate into the `[workspace.dependencies]` table every shipping crate inherits from; the root manifest was not edited.
- via-e2e: `logs/cli.log` (the CLI's own log file) has no corresponding "file-path" contract row; `tests/first_run.rs`'s audit carries it as a single named exception derived from `via::LOG_COMPONENT` rather than requiring the contracts catalogue to be updated, so a renamed component keeps the exception in sync.
- via-e2e: Asserts the header names case-insensitively for the identity-required upgrade refusal, because hyper lower-cases header names and adds `content-length`/`date` versus upstream's raw exact bytes — a deviation already recorded in phase-5-via-app.md.
- via-e2e: Deliberately does not redirect `VIA_LOG_DIR` in its fixture (unlike apps/via's fixture), because `<config>/logs/gateway.log` is itself a catalogued path that `tests/first_run.rs`/`tests/security.rs` need to check at its real location.
- via-e2e: Runs its 9 hand-applied product mutations against a `cp -R` copy of the workspace under its own `CARGO_TARGET_DIR`, with the script left uncommitted, because via-e2e's own source is too thin to usefully mutate (the code under test is the other 25 crates) and mutating the shared tree would break concurrent builds.
- via-e2e: No test is `#[ignore]`d; every wait (banner read, `wait_until`, SSE read, raw-socket exchange, HTTP/WebSocket requests) is given an explicit bound (`START_BUDGET`, `REQUEST_BUDGET`, a derived `FILE_BUDGET`, etc.) after an early unbound banner-read bug (matching only the `en` banner) hung the suite under `VIA_LOCALE=zh`.

---

## Gaps

This is the list worth reading closely. Two sources feed it, and they answer
different questions:

- **[The current picture](#the-current-picture-via-conformances-registry)** —
  every contract `via-conformance`'s registry still classifies `Pending`,
  right now, regenerated by `cargo test -p via-conformance` on every run. This
  is the number that cannot go stale, because the build fails if it silently
  disagrees with `docs/reference/contracts.json`.
- **[Gaps as recorded per phase](#gaps-as-recorded-per-phase)** — the
  narrative record each phase file wrote about what it knowingly left undone,
  kept for the reasoning and the file/line citations a bare registry row
  can't carry.

Read the first section for *what*. Read the second for *why*, and for the
handful of real functional gaps — like [the on-device provider that is built
but never registered](#functional-gaps-with-no-catalogued-contract) — that
don't correspond to a single catalogued contract at all.

### The current picture: via-conformance's registry

This pass (phase 9's conformance close-out) moved the registry from **85
locked / 432 behavioural / 176 pending records** (81 / 414 / 173 rows) to
**158 locked / 472 behavioural / 52 pending records** (152 / 453 / 52 rows) —
every one of the twenty-six crates is in scope now (`Crate::exists_today` is
`true` everywhere; four of them — `via-context`, `via-realtime-local`,
`via-realtime-openai` and `via-wake-word` — were excluded from the gate's own
"work available now" accounting until this pass, even though the latter two
had shipped real code for some time, not the scaffolds the predicate still
assumed). Locked very nearly
doubled. Pending dropped by more than two-thirds.

A later pass — described below — bound `via_realtime::RealtimeSession` to
`via-app`'s `VoiceEngine` trait for real and closed more rows across
`via-voice` and `via-backends`. The registry now reads **159 locked / 482
behavioural / 41 pending records** (153 / 463 / 41 rows), which is what
`cargo test -p via-conformance` reports today. What's left is **41 rows**,
named here by owning crate with the reason closing them honestly wasn't
possible in this pass — not "needs a test," but a real, specific hole.

#### The engine binding landed; most of `via-voice`'s rows did not close because of it

An earlier version of this document attributed thirteen of `via-voice`'s
wire-payload rows to one cause: *"no shipped crate binds a real
`via_realtime::RealtimeSession` to `via-app`'s `realtime::VoiceEngine`
trait."* That was true when [`phase-5-via-app.md`](./phase-5-via-app.md) was
written. It is not true any more — `apps/via/src/gateway/engine.rs`'s
`impl VoiceEngine for RealtimeEngine` is the binding, and
`compose.rs`'s `RealtimeEngineFactory` wires it into the composition root —
and driving a real session through it (`via-realtime-mock`, the same way
`apps/via/tests/milestone.rs` drives the phase-5 milestone) is exactly what
closed three rows this pass: `ws-event-payload/client connect frame`,
`json-field/server audio.delta payload` and `json-field/memory read/write
outputs` (`apps/via/tests/wire_payloads.rs`, `crates/via-conformance/tests/voice_wire.rs`,
`crates/via-voice/tests/tool_call_handler.rs`).

The other fourteen did not close, and the honest reason is no longer "the
binding is missing." It is that `RealtimeEngine::on_provider` and `via-app`'s
connection loop, now that both exist and run, simply do not build every frame
the wire vocabulary declares. `via_protocol::GatewayServerEvent`
(`crates/via-protocol/src/events.rs`) has constants for `voice.connection`,
`client.state`, `response.interrupted` and `transcript.discard`; a search of
every shipped crate for a place that actually *constructs* the last three
finds nothing — not a stub, not a `#[cfg(test)]` helper, nothing.
`voice.connection` is now the partial case rather than the empty one: three of
upstream's five emit sites are built — the connected and unavailable frames
either side of engine open (`crates/via-app/src/realtime/connection.rs:342`
and `:361`) and the provider-status frame the engine pump forwards
(`apps/via/src/gateway/engine.rs:132`). The two that remain are both
*mid-session* transitions, and neither is a wire-up: upstream's
`realtime-gateway.mjs:1389-1394` classifies a fatal provider error arriving on
a live connection and latches a blocked state that later opens refuse, and
`:1504-1509` reacts to the provider closing the socket by announcing
`unavailable` and then climbing a reconnect ladder with backoff. VIA has
neither half. There is no engine-to-connection notification path at all — the
only channel between them, `PlaybackControl`
(`apps/via/src/gateway/engine.rs:183`), runs connection-to-engine — and
nothing in `via-app` or `apps/via` reconnects a realtime session after it
drops: `via_realtime::ReconnectBackoff` exists but its only reference outside
its own crate is the conformance registry. Emitting the two frames without the
ladder would announce `unavailable` and then never recover, which is worse
than today's behaviour, where an explicit unmute reopens the engine. Closing
this row means building the reconnect ladder, so it is sized as a feature. The
WebSocket `timeline.inline` frame is the same shape of gap, with an HTTP-side
twin that makes it easy to miss: `via_app::http::tasks::project_timeline`
builds the identical item for `GET /api/timeline`, but nothing pushes one onto
a live socket when a Work completes. And `publicResponseContext` — the
seven-key object the catalogue says every `transcript.*`/`response.*` frame
spreads in — is fully computed (`via_voice::ResponseContext`,
`CorrelatedContext`) and then never attached to an outbound frame at all:
`response.started` carries only `responseId` and `turnId`. None of this is a
missing wire-up between two already-correct halves the way the engine binding
was; it is code that was never written, and the fix for each is a feature, not
a test.

#### Pending, by crate

**`via-voice`** (15 rows) — four are frames the wire vocabulary declares that
nothing shipped constructs in full, as described above: `json-field/client.state`,
`json-field/voice.connection` (built at three of five emit sites; the two
mid-session ones wait on a reconnect ladder), the `voice.connection` half of the combined
`ws-event/voice.connection / voice.ready / voice.sleep / voice.ownership /
voice.deactivated payloads` row (its other four sub-events — `voice.ready`,
`voice.ownership`, `voice.deactivated`, and the requestable states of
`voice.sleep` — *are* asserted for real), and `json-field/timeline.inline
frame`, whose only shipped implementation is the HTTP `GET /api/timeline`
projection. Two more are the `publicResponseContext` gap itself:
`json-field/publicResponseContext (spread into transcript/response events)`,
and `json-field/response.started / response.interrupted / audio.done`, where
`audio.done` matches the catalogue exactly, `response.started` is missing the
spread, and `response.interrupted` — barge-in cutting off audio already
playing — has no construction site anywhere. `json-field/transcript events` is
the same shape again: the assistant half (`transcript.delta`/`transcript.final`)
matches exactly; the user half (`replace: true`, a spoken or typed turn echoed
back) and `transcript.discard` are never sent — `via-app`'s connection loop
answers a typed message with `turn.started` and `voice.state` but never echoes
the text itself back as a transcript event. `ws-event/audio.delta / audio.done
/ transcript.* / timeline.inline / response.* payloads` is the combined
version of the same three gaps and stays `Pending` as a whole for the same
reason.

`json-field/voice.sleep` and `ws-event/voice.sleep states` moved to
`Behavioural`: `via-app`'s `connect{wakeWordOnly}` now prepares a
`via_voice::WakeWordLifecycle` (an opener seam over `via-wake-word`'s
default-build pipeline, scripted-double-only until `--features sherpa` is
composed at the root) and emits the detector's own four states —
`preparing`/`enabled`/`disabled`/`detected` — alongside the three
client-requested ones, asserted byte-for-byte by
`frames.rs::each_voice_sleep_state_has_its_own_catalogued_key_order` and
end to end by `via-app`'s `tests/realtime.rs`.
`json-field/voice.state` and `ws-event/voice.state payload` are the same shape
of partial: between them, `via-app`'s connection loop and
`apps/via`'s pump send exactly two of the four states (`processing` and
`idle`, each with a real `origin`), and `listening`/`speaking` are not a
literal anywhere in the tree.

`json-field/connect event fields (client -> server)` stays `Pending` as a
whole, though every catalogued key — `wakeWordOnly` included, now — is
captured by `Connect` (`via-app`'s `src/realtime/frames.rs`) and means what
upstream's `requestExplicitSleep()` means it to: no realtime session before
the phrase fires. What is missing is one test that compares the full literal
in one place rather than a field at a time; the *subset* the catalogue names
separately, `ws-event-payload/client connect frame`, omits that field and is
asserted for real now. `json-field/conversation record ids written from the
voice path` is
a composed-but-unwired service, not a missing formatting rule:
`via_conversation::ConversationSyncHandle` is built at composition
(`via-app`'s `src/services.rs`) and carried on `Services`, but nothing in the
voice path ever calls `.record()` on it, so no `voice:assistant:<id>` /
`voice:user:<id>` id is ever formatted from a live session.

The remaining three are closed-set vocabulary claims spanning more than one
crate with no enum to check them against: `error-code/tool result error_code
vocabulary` (six codes literal in `via-voice::tools::handler`, seven more in
`via-conversation`'s `ToolFailure`/`memory_tool`/`notes_tool`),
`json-field/non-error status vocabulary` (eighteen values, the same two crates
plus `via_work::WorkStatus`), and `json-field/tool result status vocabulary`
(eight of those eighteen). Every value in every list is real and traced to a
call site; what a test cannot prove without inventing a new enum is the other
direction — that nothing outside the catalogued list is ever produced —
which is exactly the bar every other *vocabulary* row in this registry clears
against a `wire_enum!`-declared `ALL` (`GatewayClientEvent`,
`GatewayServerEvent`, `GatewayTaskEvent`) or an equivalent closed enum
(`HarnessStatusCode`). Closing these three properly means giving `via-voice`'s
and `via-conversation`'s tool-result statuses that same closed shape, which is
a bigger change than this pass's scope.

**`via-backends`** (9 rows, down from 16) — one hole closed this pass, one
still open. `assets/deepseek-harness/cordis.yml` and
`assets/codebuddy/workspace/.codebuddy/models.json` — the two assets
`docs/fidelity.md` had recorded as shipped when they were not — now exist,
copied verbatim with only the identity substitutions `docs/rebrand.md`
mandates, and `via-backends`' own `tests/contracts.rs` asserts each against
`contracts.json` byte for byte (`the_cordis_*` and
`the_shipped_codebuddy_models_json_is_byte_for_byte_the_catalogued_template`).
That closes seven rows (`json-field/cordis acp-agent config`, `.../cordis
compaction-basic config`, `.../cordis llm-deepseek config`, `.../cordis
plugin id list`, `.../cordis sandbox/approval policy expressions`,
`prompt-text/cordis acp-agent persona`, `json-field/CodeBuddy models.json
full template`) plus, in `via-core`, `file-path/CodeBuddy models.json
target` — the target path, directory mode `0o700`, file mode `0o600` and the
exclusive-create ("`wx`") write are now a real, tested function,
`via_core::runtime::seed_codebuddy_models_json`, built on the same
`IfExists::Keep` primitive `load_runtime_environment` already uses for every
other first-run seed. What is still open: the **OpenClaw managed-config
pipeline is specified but not wired end to end** — `via_core::config::backend::
backend_model_name` computes the right values and the environment allow-list
admits them, but nothing stamps `VIA_OPENCLAW_MODEL`/`_MODEL_ID` into a spawned
process's environment, and nothing materializes `openclaw.json5` to its
documented runtime path (`env-var/${QWEN_AUDIO_AGENT_OPENCLAW_MODEL_ID}`,
`.../MODEL}`, `file-path/openclaw.json5 materialised path`,
`file-path/OpenClaw runtime layout`) — the same shape of gap `seed_codebuddy_
models_json` closed for CodeBuddy, not yet paid for OpenClaw. The rest is the
dropped `skill`/`skills.sh` surface (`default-value/skills CLI invocation`,
`file-path/skills.sh lockfile and installer skill directories`,
`tool-name/skills.sh passthrough argv` — consistent with `apps/via` dropping
the `skill` verb, see Divergences), `env-var/OPENCODE_* contract` (the
`OPENCODE_RUNTIME` launch-mode override is never read), and
`env-var/QWEN_AUDIO_AGENT_PREPARE` (npm's `prepare` lifecycle guard has no
counterpart in a self-contained binary).

**`apps/via`** (6 rows) — all six are one fact: **`via service
install|start|stop` is still `Unimplemented`.** No systemd/launchd unit is
built (`file-path/systemd unit`), no service metadata sidecar is written
(`file-path/service metadata sidecar`, `state-name/GATEWAY_SERVICE_LABEL`), no
log path or environment block exists for a service that is never installed
(`file-path/service log paths`, `env-var/service environment block`). The
last row, `env-var/gateway child spawn environment`, is half-real: the
`HOST`/`PORT` derivation `apps/via::origin::listen_address` performs is
correct and tested, but the surrounding two-process shape (`execPath`,
`index.mjs`, `ELECTRON_RUN_AS_NODE`, `detached`, `stdio`) has no VIA
counterpart — a single binary doesn't fork a second copy of itself.

**`via-acp`** (5 rows) — two are a second, genuinely separate WebSocket client
upstream has that VIA never built: `ws-event/OpenClaw RPC methods` and
`ws-event/OpenClaw connect handshake` describe `openclaw-adapter.mjs`'s direct
WS protocol, distinct from the ACP-bridge subprocess VIA already launches and
tests; there is no `tokio-tungstenite` dependency anywhere in the tree to
speak it. `error-code/session config / model enforcement errors` is a case
where the *text* shipped (all eight Chinese templates are in
`via-i18n/assets/i18n/acp.json`) but the *behaviour* — raising, HTTP status
mapping, the twelve-item truncation rule — was never wired into
`set_session_config_option`/`set_legacy_session_model`, which are raw,
unwrapped RPC calls today. `error-code/OpenClaw retry discriminator` has half
its logic (`via-backends::openclaw::prompt_retry_delay`'s per-attempt
schedule) but not the other half (deciding same-session vs. fresh-session
retry). `json-field/describe() backend descriptor` is deliberate on the VIA
side — `via-app`'s `BackendDescription` is an intentionally untyped
passthrough, by design — but that means the full catalogued shape has no
single struct to compare against.

**`via-app`** (4 rows) — `default-value/MarkdownContextStore document
limits`: no composition root actually constructs the `user`(6000)/`memory`(8000)
pair with these bounds. `state-name/composition root injection points`:
upstream's 13-key `services` map has a partial, undocumented mapping onto
VIA's `Services` struct (`coordinator`, `backendAvailability`,
`realtimeGateway` don't land on any single field), so a byte-exact test would
either hardcode a fragile list or overclaim. `default-value/gateway origin
validation rule` and `file-path/DEFAULT_GATEWAY_ENTRY` both belong to
upstream's Electron/embedding-host surface (`shared/gateway-process.mjs`),
dropped along with the desktop shell — real, but not yet written down as a
divergence anywhere, which is why they're `Pending` rather than `Divergent`.

`via-core` owns no pending row any more: `file-path/CodeBuddy models.json
target` — the same asset named under `via-backends` above — closed alongside
it, since `via_core::runtime::seed_codebuddy_models_json` is what materializes
the file this path names.

---

### Functional gaps with no catalogued contract

Not everything left undone shows up as a `Pending` row — some of it is a
functional gap the upstream contract catalogue never described in the first
place, because it's about VIA's own composition, not a value upstream
carries. These are still open as of this pass:

- ~~**`via-app`: the on-device provider is built but never registered.**~~
  **Closed.** `apps/via/src/gateway/compose.rs` now calls
  `via_realtime_openai::register_openai_provider` and
  `via_realtime_local::register_local_provider` beside
  `via_realtime_dashscope::builtin_provider_registry`, so both `openai` (phase
  6) and `local-omni` (phase 8) have a runtime path into a shipped Gateway —
  registered unconditionally, appearing in `/api/health`'s `realtimeProviders`
  once configured and resolvable-but-unconfigured before that, exactly as an
  unconfigured DashScope already behaved.
  `via_realtime_local::local_health` is wired the same way, so `/api/health`
  carries the reason an unconfigured or partially-configured local provider
  isn't usable. `via-realtime-mock` is deliberately still not one of them — a
  test double reaching tests through `via_app::testing::test_registry`, not a
  composition-root citizen. (`docs/deviations/phase-8-via-realtime-local.md`)
- ~~**`via-backends` / `via-process`: a reallocated port does not reach the
  child's argv.**~~ **Closed.** The argv now carries a placeholder —
  `${OPENCODE_PORT:-<derived>}` — that `via_process::spawn_spec` expands from
  the child's own environment. That is the same moment upstream's
  `scripts/*.mjs` shim reads `process.env.<X>_PORT`, and because `spawn_spec`
  runs *after* `apply_backend_address` has published a reallocated port
  (`start.rs:185`), the child is told the port VIA actually reserved rather
  than the one it gave up on. The substitution is generic `${NAME}` /
  `${NAME:-fallback}`, so `via-process` still names no backend and
  `via-arch-test` stays green.
- ~~**`apps/via`: `WorkManager::recover_delegated` is never called.**~~
  **Closed.** `Composition::compose` now calls `work.recover_delegated(...)`
  once, right after the coordinator is built, wired to a new
  `via_coordinator::Coordinator::recover_native_delegation` (with
  `apps/via/src/gateway/delegation.rs::CoordinatorRecovery` as the
  `DelegatedWorkRecovery` adapter; a coordinator-less Gateway gets
  `NoRecovery`, which declines the same way and drains the same candidate
  list). `can_recover` is `Coordinator::native_delegation_configured` — VIA's
  composition wires no `NativeDelegationAdapter`, so every candidate is
  declined today and reaches the catalogued *unrecoverable delegated work*
  restart sentence rather than being stranded `queued` in
  `recovery_candidates` forever; `CoordinatorError::NotRecoverable` (localized,
  previously constructed nowhere in shipped code) is what carries the
  refusal. The reattachment path itself is real, not a stub —
  `crates/via-coordinator/tests/delegation_lifecycle.rs`'s
  `recovering_a_delegation_reattaches_and_finalizes_through_the_result_turn`
  drives it end to end with a `ScriptedNativeDelegation` — so a future
  composition that does root a native adapter recovers without touching this
  wiring again. (`docs/deviations/phase-9-via-e2e.md`)
- **`apps/via`: `via chat` does not scan typed text for file references.**
  `inputPartsFromText` (promoting a typed `@path` or a pasted absolute path
  into a real file input part) has no client-side implementation; every
  supporting primitive (`via_voice::normalize_input_parts`,
  `input_file_parts`) and the wire acceptance (`input.parts`) are already
  there — only the client's own path-scanning is missing.
  (`docs/deviations/phase-5-apps-via.md`)
- **`apps/via`: no autostart fallback to a Gateway on another port, and no
  compatibility check before reusing one.** `findRunningGateway` and
  `assertGatewayCompatibility` are unported; both belong with the
  not-yet-implemented `via backend` verb work, and the catalogue already
  carries the refusal sentences (`cli.reuse_*`) waiting for them.
  (`docs/deviations/phase-5-apps-via.md`)
- **`via-realtime-openai`: no environment variable for the Azure API version
  or dialect override** — both are hardcoded-default fields because
  `via-core`, already shipped by phase 6, owns every `VIA_*` env-var name and
  this phase couldn't edit it. Named as a follow-up, mirroring an equivalent
  open item in ARGO's own bring-up notes.
  (`docs/deviations/phase-6-via-realtime-openai.md`)
- **`via-wake-word`: real end-to-end detection is not exercised in CI.** The
  one test that installs the actual catalogued model artifact and detects a
  real spoken phrase is `#[ignore]`d behind `VIA_WAKE_WORD_LIVE_MODEL=1`,
  deliberately, so CI never depends on a third party's release page staying
  up — it has been run manually and passes, but nothing automated proves it
  on every commit. (`docs/deviations/phase-6-via-wake-word.md`)
- **`via-context`: a host-registered screen surface doesn't exist.** Phase 7
  made `SessionMode::Interface`'s degradation real for conversation-only
  referents, but VIA has no screen capture and no connect-time frame for one,
  so an `interface` session's `SurfaceStatus` always reports `Unbound` and
  screen referents can never resolve. (`docs/deviations/phase-7.md`)
- **`via-realtime-local`: `LlamaResponder` cannot author or parse tool
  calls.** A GGUF model emits them as model-specific text with no parser in
  this crate for any such format; the `local` family's catalogued
  `function_calling: true` describes what the pipeline contract allows for, not
  what this responder does today. A deployment that needs tool calls from a
  local model has to supply its own wrapping `Responder`.
  (`docs/deviations/phase-8-via-realtime-local.md`)
- **`via-acp`: `AcpHarnessSession::events()` returns an empty stream.** Wiring
  `via_downstream::ActivityTracker`'s projection to a live broadcast fan-out
  needs to know which Work a session's activity belongs to — knowledge
  `via-backends` has and `via-acp` does not. Documented on the method itself
  so a subscriber gets a stream that ends cleanly rather than one that lies
  about being live. (`docs/deviations/phase-2.md`)

---

## Gaps as recorded per phase

The narrative record, kept for citation and reasoning. Read the note on
staleness above before treating an entry here as still open — several
phase-0-era rows below were closed to `Partial` once their named crate
shipped, and the wake-word row about "eight unclaimed registry rows" was
closed to `Behavioural` in this same pass. The
[current picture](#the-current-picture-via-conformances-registry) above is
authoritative for what remains; this section is authoritative for why each
one was left, and
where.

### Phase 0 — workspace, gates, leaf crates

[`phase-0.md`](./phase-0.md)

- Windows `SystemProcessProbe` cannot prove liveness (`Unsupported` reads as ALIVE) — no real OpenProcess/GetExitCodeProcess + creation-time check implemented (blocked by workspace-wide `#![forbid(unsafe_code)]` and no Windows test machine); a stale `gateway.lock` on Windows must be deleted by hand. `proves_liveness()` at least reports the platform's capability at runtime now.
- `sherpa KWS detection config` contract (featConfig samplingRate/featureDim + 9 other fields) is only partially covered — via-audio asserts just the 16000 sample-rate sub-value; the full contract belongs to via-wake-word, which does not exist yet at phase 0.
- `ws-event/input.suspend / input.resume / input.suspend.ack` contract's exactValue (prose about payloads and the `playback.clear` coupling) is not asserted in via-conformance; owed to a named test in via-voice.
- `default-value/backend default base URLs` (port envs) — only partially asserted; via-backends doesn't exist yet at phase 0.
- `state-name/backend integration modes and configuration modes` (inspectBackend outputs) — same, owed to via-backends.
- `json-field/pi backend capability declaration` (driver capability flags) — same, owed to via-backends.
- `default-value/provider aliases and configured gating` (isConfigured logic) — owed to via-realtime, which doesn't exist yet at phase 0.
- `env-var/backend workspace overrides` and `env-var/${QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE}` (resolution rule) — owed to via-core.
- `env-var/QWAUDIO_CONFIG_DIR` and `file-path/files and directories the product creates` (platform base directory + everything created under it) — owed to via-core.
- `cli-flag/--backend` (the flag itself) — owed to apps/via, which doesn't exist yet at phase 0.
- `default-value/gateway polling timeouts` (waitForGateway / waitForGatewayStop / health fetch timeout) — owed to apps/via.
- `json-field/health fields the CLI reads` (everything but `gatewayInstanceId`) — owed to via-app.
- `file-path/backend session state file` (registry path, env override, two record shapes) — owed to via-acp.

### Phase 1 — via-core, via-i18n, setup gate, binary skeleton

[`phase-1.md`](./phase-1.md)

- A top-level `via status` alias for `gateway status` is owed — not yet its own verb.
- Three of upstream's seven `GATEWAY_ACTIONS` (`restart`, `status`, `uninstall`) have no `via service` disposition; `ServiceAction::Unknown` reports them as unsupported rather than silently dropping them.
- `--audio-mode` is deferred along with audio support for `via chat` — `via chat` currently carries no audio mode at all.
- `--help` is English only; the catalogued localized `helpText()`/`cli.help` contract for `zh`/`ko` exists in `via-i18n` but "this binary does not read it."
- `via gateway`, `via chat`, `via backend`, `via mcp-serve`, and `via service` each run their setup steps and then refuse with `VIA_CLI_NOT_IMPLEMENTED`, naming a later phase, instead of doing real work — the command bodies are stubs at this point in the port.

### Phase 2 — via-acp, via-backends, via-process, via-downstream

[`phase-2.md`](./phase-2.md)

- via-acp: `AcpHarnessSession::events()` returns an empty stream — wiring `via_downstream::ActivityTracker`'s projection to a live broadcast fan-out needs to know which Work a session's activity belongs to, which is `via-backends`' knowledge, not `via-acp`'s. Documented on the method so a subscriber gets a stream that ends rather than one that lies about being live.

### Phase 5 — apps/via

[`phase-5-apps-via.md`](./phase-5-apps-via.md)

- apps-via: `findRunningGateway` (autostart fallback to a Gateway that moved to another local port) and `assertGatewayCompatibility` (refusing to reuse a Gateway whose backend/ownership/permission-mode/realtime-model differs) are not ported — explicitly "owed", belongs with the not-yet-implemented `via backend` verb; the catalogue already carries the `cli.reuse_*` sentences for it.
- apps-via: `inputPartsFromText` (promoting `@path` mentions and pasted absolute paths in typed text into file input parts) has no client-side implementation — `via-voice` already owns the supporting primitives (`normalize_input_parts`, `input_file_parts`) and the socket already accepts `input.parts`; only the `via chat` client's own path-scanning is missing.
- apps-via: the input-side transcription branches of the realtime event switch (`conversation.item.input_audio_transcription.{delta,completed,failed}`, `input_audio_buffer.committed`) are unported — `via_voice::TurnCorrelation`/`ServerEvent::streaming_input_transcript` exist and are waiting, but nothing emits them yet; explicitly deferred to land with `via-realtime-openai` in phase 6, the first provider that produces a user transcript at all.

### Phase 5 — via-app

[`phase-5-via-app.md`](./phase-5-via-app.md)

- via-app: binding `via_realtime::RealtimeSession` to the `realtime::VoiceEngine` trait is explicitly called out as "the remaining phase-5 step" — not done in this crate; `NoEngineFactory` ships as the default (a supported configuration, not a broken one) until that binding lands. The binding has since landed in `apps/via` — `src/gateway/engine.rs`'s `impl VoiceEngine for RealtimeEngine`, wired by `compose.rs`'s `RealtimeEngineFactory` — so `NoEngineFactory` is now the fallback for a Gateway with no usable realtime credential rather than the only implementation (closed/resolved; see [the current picture](#the-current-picture-via-conformances-registry) for what closing it did, and did not, unlock).

### Phase 6 — via-realtime-openai

[`phase-6-via-realtime-openai.md`](./phase-6-via-realtime-openai.md)

- No environment variable exists yet for the Azure `api-version` or the dialect override — both are hardcoded-default `OpenAiSettings` fields because `via-core` (shipped) owns every `VIA_*` env-var name and can't be edited here; explicitly called "a follow-up," mirroring the same open item in ARGO's own bring-up doc for its per-provider realtime model setting.

### Phase 6 — via-wake-word

[`phase-6-via-wake-word.md`](./phase-6-via-wake-word.md)

- A real end-to-end wake-word detection is not exercised in CI — `tests/sherpa.rs` covers only the guards in front of the engine with no model files; the one test that installs the real catalogued artifact and detects a real spoken keyword is `#[ignore]`d and gated on `VIA_WAKE_WORD_LIVE_MODEL=1`, since CI should not depend on a third party's release page staying up (it has been run manually and passes).
- The eight `via-conformance` registry rows asserted against `Crate::ViaWakeWord` remain unclaimed — the text states claiming them "is a `via-conformance` edit rather than a `via-wake-word` one."

### Phase 7 — via-context

[`phase-7.md`](./phase-7.md)

- via-context/via-app: Closing `SessionMode::Interface`'s degradation makes the mode "real" for conversation-only referents, but a host-registered screen surface (new protocol-level connect-time frames) is not added in phase 7 — VIA has no screen capture, so an `interface` session's `SurfaceStatus` reports `Unbound` and screen referents never resolve.

### Phase 8 — via-realtime-local

[`phase-8-via-realtime-local.md`](./phase-8-via-realtime-local.md)

- via-realtime-local: `LlamaResponder` does not author/parse tool calls — a GGUF emits them as model-specific text and no parser for that format exists in this crate; a deployment wanting tool calls from a local model must supply its own wrapping `Responder`, so the `local` family's `function_calling: true` describes the pipeline contract, not this responder's actual capability.
- via-app: `local_provider`, `register_local_provider`, and `local_health` exist but are not wired into the shipped Gateway — this phase does not edit via-app, so the local on-device provider has no runtime registration path yet.

### Phase 9 — via-e2e

[`phase-9-via-e2e.md`](./phase-9-via-e2e.md)

- apps/via: `WorkManager::recover_delegated` is documented as "call once, at composition" but nothing in apps/via calls it, so a `delegated` Work that was addressable and whose recovery declined is restored as `queued` and left in `recovery_candidates` — never re-attached, never force-failed — and the catalogued "unrecoverable delegated work" restart sentence is unreachable in practice; the fix is one `work.recover_delegated(runner).await` at composition.
