# Phase 0 deviations

Every place a phase-0 crate does **not** reproduce an upstream value or behaviour
exactly, with the reason. This is the per-phase detail behind
[`fidelity.md`](../fidelity.md); that document carries the policy, this one
carries the receipts.

**77 deviations across 8 crates, 475 tests.**

A deviation here is not a defect. It is a place where reproducing the upstream
literally would have been wrong (a JavaScript idiom with no Rust meaning), or
impossible (a value the upstream computes from a runtime VIA does not have), or
a decision VIA took deliberately and wrote down. What would be a defect is an
undocumented one.

> **Gap closure, 2026-08-22.** The original 73 were reviewed one by one against
> that test, and the 31 `Pending` contracts whose owning crate already shipped
> were all closed: 11 `Asserted`, 15 `Divergent`, 5 `Partial`. The
> `gaps_in_shipped_crates` snapshot went from 31 lines to none, and locked
> contracts from `46 → 73` records / `43 → 69` rows (+26, the asserted and
> divergent ones). Deviation arithmetic: 73 − 2 closed + 2 new `via-lock`
> records + 4 new `via-conformance` records = 77.
>
> - **Two entries were not decisions but unfinished jobs and are CLOSED**:
>   `via-log`'s test-process detection, which could not fire for any Rust
>   build, and `via-conformance`'s note that `client input capability profiles`
>   was a real gap in a shipped crate. Both are marked **CLOSED** in place
>   below rather than deleted, so the record of what was wrong survives the
>   fix, and neither is counted in the 77.
> - **One contract pair had no code at all**: `file-path/CLI instance lock` and
>   `error-code/CLI lock conflicts` name `<configDir>/cli.lock`, which
>   `via-lock` simply did not implement. It does now (`src/cli.rs`), so those
>   two rows are contracts rather than plans.
> - **One deviation was re-examined and KEPT**: `via-lock`'s Windows liveness
>   probe, with the reasoning made explicit and a runtime
>   `SystemProcessProbe::proves_liveness()` added so a Windows host can say why
>   it refuses instead of refusing opaquely.
> - **Six new records** appear: two under `via-lock` for the CLI instance lock
>   and its race-only exhausted branch, and four under `via-conformance` for
>   the ownership correction and the classification decisions this pass had to
>   take.
>
> Everything else stands. In particular the mkdir lock, the consuming
> `release(self)`, the absent `Drop` impls, the synchronous sinks and stores,
> the prose-vs-literal splits and every "the message belongs in `via-i18n`"
> entry are decisions, not debt, and were left alone.


## `via-protocol`

*101 tests · clippy clean · 7 deviations*

- GATEWAY_PROTOCOL_VERSION is "1.0.0", not upstream's "2.0.0". Deliberate, per the task brief and docs/fidelity.md: VIA advertises a strict subset of upstream's capabilities and a removed capability is a breaking change under upstream's own rule. A test asserts the value is NOT "2.0.0".

- GATEWAY_CAPABILITIES is 7 of upstream's 16 entries — the gateway.* and input.* families, in upstream's relative order. Dropped: the 7 web.*/desktop.* entries (no SPA, no Electron orb, no skin store) plus host.electron-entry and host.gateway-process, which name CommonJS/ESM package entry points a single Rust binary does not have. The 9 dropped names are kept in DROPPED_UPSTREAM_CAPABILITIES so via-conformance can assert the two lists partition upstream's 16 exactly.

- WorkState has FIVE variants (active | scheduled | completed | failed | cancelled), not the four named in the task brief. contracts.json's 'workState values' entry lists five, and upstream computes workState as `ACTIVE.has(status) ? 'active' : status` (task-manager.mjs:58) — `scheduled` is not in ACTIVE, so a scheduled Work publishes workState:"scheduled" verbatim. With only four variants WorkStatus::Scheduled.state() would have nothing correct to return. Followed contracts.json over the brief; flagged here in case via-work or via-conformance expected four.

- ProtocolError's Display strings are developer-facing English, not upstream's Chinese literals ('Gateway 启动被拒绝，缺少必填配置：…', '已有 Gateway 正在运行：<origin>', 'input.suspend 需要 owner'). Per docs/architecture.md §16 error codes are not localized but the message beside them is, and via-protocol is a leaf crate with no via-i18n dependency. The codes and the structured payloads (MissingSetting{field,key,message}, origin) are reproduced exactly; the localized text belongs in via-i18n's zh catalog. If a conformance test wants byte-equality on those three messages, they need to live in via-i18n, not here.

- ProtocolError::AlreadyRunning carries only `origin: Option<String>`, not upstream's full `error.lease` object. The lease document shape (via.gateway-lock/v1) belongs to via-lock; via-protocol would have to grow a dependency on it to carry the whole thing.

- Two VIA-owned error codes were invented because upstream has no code for either case: VIA_PROTOCOL_UNKNOWN_WIRE_VALUE (FromStr failure) and VIA_PROTOCOL_ILLEGAL_TRANSITION (refused status change). They carry a VIA_PROTOCOL_ prefix rather than a bare VIA_ one so they are never mistaken for an inherited contract; CONTRACT_ERROR_CODES lists only the three that are.

- The transition graph is the union of docs/architecture.md §4's ASCII diagram and the transitions upstream actually performs, because the diagram abbreviates: it draws the cancel branch only off `queued` and the completed edge only off `running`/`finalizing`. The 17 edges each cite the upstream line that performs them. Crash recovery (restore() rewriting active -> failed, or a recoverable delegated/finalizing -> queued) is deliberately NOT an edge — it is a store-level rewrite of a record from a dead process, and admitting it would legalise running -> queued for live records.


## `via-catalog`

*44 tests · clippy clean · 12 deviations*

- THE BUG NOT PORTED (mandated): `resolve_dashscope_realtime_model_profile` returns `Err(CatalogError::UnknownRealtimeModel)` for an unrecognised id instead of upstream's all-capabilities-false profile, and `ModelFamily` has no `unknown` variant. Upstream's fallback + providers/dashscope.mjs:84-87 opens a session with no input_audio_format and no turn_detection. Upstream's fail-closed property (no name-based capability inference) is preserved and strengthened; tests/realtime_model.rs uses upstream's own input `qwen3.5-omni-plus-realtime-future` and additionally asserts no catalog entry is all-false.

- NEW `local` model family (no upstream peer): `local_realtime_model_profile(id)` builds an owned profile with REAL flags — model {text,audio in; text,audio out; functionCalling} true, image/video false; transport {text,audio} true, rest false. `audio_input: true` is the flag the deafness bug hinged on. Local ids are user-chosen GGUF/ONNX coordinates so there is no fixed table; the label mirrors the id, as upstream did for ids it did not recognise.

- INVENTED, DOCUMENTED AS SUCH: `TurnDetectionKind::ServerVad` ("server_vad") for the local family — the OpenAI-Realtime dialect name `local-omni:endpoint` will actually speak; `DEFAULT_LOCAL_REALTIME_VOICE = "af_heart"` (Kokoro-82M's stock voice), which via-realtime-local is expected to override; and `LOCAL_FAMILY_VOICE_ENV = "VIA_LOCAL_REALTIME_VOICE"`. None is an upstream contract and each says so in its doc comment.

- THREE NEW PROVIDERS: `openai` (label "OpenAI Realtime", default wss://api.openai.com/v1/realtime — sourced from ARGO tinicore/src/llm/providers/openai_live.rs:9, not from qwen), `local-omni` (label "Local Omni", no default URL: `:pipeline` needs none and `:endpoint` must be configured; the mode suffix is a session setting, not a second provider key), `mock` (label "Mock Realtime", no default URL). All three carry no aliases. Upstream's two entries stay first so the ordered name list keeps upstream's prefix.

- IDENTITY SHAPE is a declared provider property rather than an inline `provider === 'dashscope'` branch, so new providers cannot accidentally emit null model/voice keys into the hash. dashscope/openai/local-omni -> ModelAndVoice; speech-to-speech/mock -> EndpointOnly.

- ERROR MESSAGE TEXT NOT REPRODUCED: upstream interpolates the provider list into `不支持的 Realtime 前台：X（可选 dashscope、speech-to-speech）`. VIA's list is five entries long, so that sentence cannot be byte-identical. Per docs/fidelity.md the crate returns a typed `CatalogError` with a stable `code()` and the interpolation values; the sentence belongs to via-i18n. Same for the backend ownership/unsupported-backend refusals and the missingConfigurationMessage strings.

- ENV NAMES RENAMED per docs/rebrand.md, so the backend allow-lists are not byte-identical to upstream: QWEN_AUDIO_AGENT_OPENCODE_ISOLATE_USER_CONFIG/-XDG_CONFIG_HOME -> VIA_OPENCODE_*; QWAUDIO_CONFIG_DIR -> VIA_CONFIG_DIR; QWEN_AUDIO_AGENT_OPENCLAW_{MODEL,MODEL_ID,STATE_DIR,WORKSPACE} -> VIA_OPENCLAW_*; QWEN_AUDIO_AGENT_ACP_FORWARD_ENV -> VIA_ACP_FORWARD_ENV; openclaw workspaceEnvironment -> VIA_OPENCLAW_WORKSPACE; QWEN_AUDIO_REALTIME_VOICE -> VIA_REALTIME_VOICE; QWEN_OMNI_REALTIME_VOICE -> VIA_OMNI_REALTIME_VOICE. List positions and every vendor-owned name (DASHSCOPE_API_KEY, AGENT_API_KEY, QWEN_CODE_, OPENAI_*, ANTHROPIC_*, CLAUDE_API_KEY, DEEPSEEK_API_KEY, GEMINI/GOOGLE_API_KEY, all prefixes) are unchanged.

- **CLOSED (phase 2), as planned.** `InstallationSpec.steps` is declared and EMPTY in `via-catalog` for all 11 installable backends, and it stays that way: the pinned coordinates are execution machinery, and they landed where this record said they would — `crates/via-backends/src/install.rs`, as `InstallStepSpec` tables (`OPENCODE_STEPS`, `OPENCLAW_STEPS`, …), asserted against the catalogued `default-value/backend pinned install packages and scripts`. The pinned coordinates (opencode-ai@1.18.5, openclaw@2026.6.33, @qoder-ai/qodercli@1.1.13, @qwen-code/qwen-code@0.21.6, @moonshot-ai/kimi-code@0.32.0, @tencent-ai/codebuddy-code@2.132.0, @openai/codex@0.146.0 + @agentclientprotocol/codex-acp@1.1.7, @anthropic-ai/claude-code@2.1.221 + @zed-industries/claude-code-acp@0.16.2, @earendil-works/pi-coding-agent@0.84.1 + pi-acp@0.0.33, the 11 @deepseek-ai/dsh* @0.1.0-rc.6 with ACP LAST, and the two Hermes curl/iex scripts) are all there. The `installation: None` vs `Some` distinction was populated and tested in phase 0 — `acp` is the only backend VIA never installs — as is deepseek's `verifyInstalledPackages: true`.

- **CLOSED (phase 2), as planned.** Upstream's `onboarding.hint` Chinese sentences and the install-step `label` strings ('ACP 适配器', '运行组件', 'ACP Runtime', 'DeepSeek CLI') are absent from `via-catalog` and stay absent: per `docs/fidelity.md` user-facing text lives in `via-i18n` keyed by backend id. They landed there — `via_backends::onboarding::onboarding_hint_key(id)` resolves the id to a `via-i18n` key and `tests/registry.rs` asserts every backend has one. `onboarding.command` and the full auth-probe machinery (kind, args, parser) were populated and tested in phase 0.

- NOT PORTED — env reading: upstream's `resolveDashScopeRealtimeVoiceOverride(model, env)` and `resolveRealtimeFrontendConfiguration(env)` read process.env. via-catalog is a pure table (no env, fs or clock); it publishes the family->variable mapping via `ModelFamily::voice_override_env()` and the identity/signature machinery, and via-core does the lookups. The `configured` gating booleans (Boolean(apiKey) / the s2s three-way OR) are likewise via-core's, since they are env predicates rather than table data.

- TYPE-ENFORCED instead of runtime-validated: upstream's `validateBackendSkillsSpec` throws at registration when `skills` is undefined. `SkillsSpec` is a non-optional field with no unset variant, so the same guarantee holds at compile time. Upstream's SKILLS_INSTALLER_PATTERN regex is reproduced as `is_valid_skills_installer` (no regex dependency) and asserted against every catalog installer id.

- `effective_backend_permission_mode` returns `String`, not an enum, deliberately: upstream states it performs no validity check ("本函数不做合法性校验") and passes an unrecognised mode through for the caller to reject in its own context.


## `via-log`

*72 tests · clippy clean · 10 deviations*

- `[Circular]`: upstream detects cycles with a WeakSet. A `serde_json::Value` is acyclic by construction, so the marker is reproduced as the public constant `CIRCULAR` and asserted by test, but this crate never produces it. Upstream's test/logger.test.mjs:124-131 case is therefore asserted as a constant rather than reproduced.

- Correlation context: upstream uses `AsyncLocalStorage`, which follows a value across `await`. `run_with_log_context` is a thread-local and does not. Async call sites are expected to carry correlation on a `tracing` span; `ViaLayer` folds span fields into the same record position (base → thread-local context → span fields root→leaf → event fields → spine). Recorded in the crate docs.

- String truncation counts Unicode scalar values where upstream's `String.prototype.length`/`slice` count UTF-16 code units. Identical for ASCII; a lone surrogate cannot exist in a Rust `String`, so upstream's surrogate-splitting behaviour is not reproducible and was not attempted.

- `fatal` has no `tracing::Level`. Records at `fatal` are reachable only through `Logger::fatal` / `Logger::log_record`, never through the Layer.

- `bigint` handling (upstream `typeof value === 'bigint' → String(value)`) has no `serde_json::Value` counterpart. The equivalent case surfaces in the tracing visitor: `record_i128`/`record_u128` outside i64/u64 range are recorded as decimal text rather than truncated, and non-finite `f64` likewise.

- `ErrorRecord::from_error` sets `name = "Error"` (upstream's own default) and leaves `stack` unset — Rust errors carry no name and no captured stack. The `code`/`stack`/`extra` fields exist so a caller that has those values reproduces upstream's exact field order; nothing is invented.

- Sink writes are synchronous under a `Mutex` rather than upstream's promise-queued async drain, so `flush()` is close to a no-op and no records can be dropped by a queue reset. Upstream clears its pending queue on failure; here a failed write loses only that one record. `on_error` still fires exactly once per transition into the failed state, and `Logger` adds upstream's once-per-logger stderr guard.

- **CLOSED.** `is_test_process` reproduced upstream's `NODE_ENV === 'test'` and the argv `test/`-segment check and nothing else, and the note here guessed that only the argv half was live. That guess was wrong: a `cargo test` binary is invoked as `target/<profile>/deps/<name>-<hash>`, which contains no `test/` segment either, so **neither** upstream arm could ever fire for a Rust build and the Gateway logger would have written into the developer's real log directory — exactly the outcome the upstream check exists to prevent. A third arm was added: `ENV_TEST_PROCESS` (`VIA_TEST`), matched against `TEST_PROCESS_VALUE` (`"1"`) exactly, in the same style as `VIA_LOG_CONSOLE`/`VIA_LOG_FILE`'s exact `"0"`. Both upstream arms are kept verbatim for fidelity, both are asserted (including their near misses — `NODE_ENV=testing` is not a match), and the cargo-binary case is asserted in both directions. `VIA_TEST` is VIA's own signal, not an upstream contract, and says so at its declaration; `via-core` or a test harness sets it.

- Windows: `LOG_DIRECTORY_MODE` (0o700) and `LOG_FILE_MODE` (0o600) are applied under `#[cfg(unix)]` only; on Windows the directory and file inherit the parent ACL. Stated in the crate docs rather than silently stubbed.

- The two `(?-u:\b)` groups in `AUTH_VALUE_PATTERN` / `API_KEY_VALUE_PATTERN` are not in upstream's source: they are the Rust spelling of JavaScript's ASCII-only `\b` (Rust's default `\b` is Unicode-aware). Behaviour matches; the literal differs. Documented at each constant.

- **CLOSED, and not the way the marker said.** `LOG_SINK_FAILURE_PREFIX_ZH` was a `const` carrying only the `zh` text, with a `TODO(via-i18n)` promising it would "become a lookup" once the catalog landed. `via-i18n` has landed and the lookup is not available: `docs/architecture.md` §9's adjacency table gives `shared → ∅`, and `via-arch-test`'s `leaf_violations` fails the build on any edge out of a Leaf-band crate — so `via-log` cannot call `via_i18n::t`, now or later. (The marker was written before that rule was executable.) The constant stays, and it is the right thing for its one call site anyway: it is the line written to stderr when the logger itself cannot write, which is the one place a catalog lookup cannot be assumed to work. What was actually owed was a guard against the two copies drifting, and that is now `via-conformance`'s `tests/localized_leaf_messages.rs`, which asserts the constant equals `log.sink_write_failed`'s `zh` value rendered with an empty detail. The `TODO` is replaced by that reasoning at the declaration.


## `via-store`

*50 tests · clippy clean · 12 deviations*

- The lock is mkdir-based, NOT `std::fs::File::lock` — deliberately not a deviation. The on-disk mechanism is itself the compatibility surface: the other side may be a Node Gateway, an older VIA or the CLI, and two processes using different primitives on the same file exclude nothing. `<path>.lock` as a 0o700 directory, `owner.json` at 0o600 with compact `{token,pid,createdAt}` + newline, reclaim to `<lockPath>.stale.<token>`, release to `<lockPath>.stale.released.<token>`, and the legacy file-shaped-lock fallback in readOwner are all reproduced.

- Interpolated engine text inside warnings cannot match Node. Upstream splices `${error.message}` (e.g. "ENOENT: no such file or directory, rename '…'"); Rust splices `io::Error`/`serde_json::Error` Display. All six templates around it are verbatim, and the tests assert the fixed prefixes/suffixes rather than the engine text.

- `load` returns `Option<Map<String, Value>>` instead of taking a `fallback` closure and calling it. `None` means exactly what upstream's `fallback()` meant, at every one of the four sites (absent / unreadable / corrupt / rejected). The `validate` callback is kept as-is.

- Coalescing is synchronous and runtime-agnostic. Upstream chains deferred writes onto a promise (`task-store.mjs:149`) and `flush()` awaits it; here writes serialize on a mutex and `flush()` performs the pending write on the calling thread. The scheme itself is reproduced exactly: mutations bump a generation, the temp name carries it, the generation is re-checked after the temp write and before the rename so a superseded write deletes its own staging file instead of renaming, and a superseded write's failure never disables persistence. Upstream's `clearTimeout` becomes a schedule sequence number — a ticket already handed to the host finds itself stale and no-ops.

- Two fsync barriers were added that upstream does not have: `sync_all()` on the temp file before the rename, and `sync_all()` on the parent directory after it. The directory fsync is compiled out on Windows (no directory handle) and is best-effort elsewhere — some filesystems reject it with EINVAL, and a failure there means "the rename may not survive a power cut", not "the write failed", so it must not escalate into disabling persistence.

- The `now` injection is kept on the store (it decides the observable `<path>.corrupt-<now>` filename) but dropped from the lock, where upstream exposes it only as a test seam and nothing external observes it. Lock staleness compares the system clock against the lock directory's mtime, as upstream does.

- `save`/`save_deferred` take any `Serialize`. A payload that does not serialize to a JSON object contributes no keys — JS spread of a primitive does the same. Unlike upstream, a payload that fails to serialize at all writes nothing rather than producing a bare `{version}` document, because a placeholder would silently replace good state.

- `StoreMessages::TASK_STORE` is provided here although those strings live in `server/src/task/task-store.mjs`. Upstream duplicates the whole state machine to vary four of the six strings; here the string table is a parameter, so via-work can take the catalogued wording without a second store. The frontend-notes set (which adds a third quarantine suffix and a thrown 'frontend notes persistence is unavailable') is deliberately left to via-conversation — same extension point.

- The migration hook is new. Upstream has no hook and always quarantines a version mismatch; a store built without `.migrate(...)` is byte-for-byte upstream, and a hook that returns `None` falls through to the same quarantine. A successful migration does not write back — the caller's next `save` does.

- `LockError` is a `thiserror` enum with a `code()` accessor rather than a JS `Error` carrying a `.code` property, per docs/fidelity.md ('Ad-hoc { ok, error, code } returns → thiserror enums with a code() accessor'). The message and the `shared_file_busy` code are verbatim.

- `replace_file`'s Windows ladder is gated with a runtime `cfg!(windows)` check rather than `#[cfg(windows)]` so it compiles and is linted on every host, and `ReplaceOptions::windows_semantics` lets a test force it. Its backup/rollback branch is still unexercised on Unix, where `rename(2)` never raises the EACCES/EBUSY/EPERM class that triggers it.

- A panicking `on_warning` callback is contained with `catch_unwind(AssertUnwindSafe(..))` — upstream's bare `try {} catch {}` and its comment 'Diagnostics must never prevent startup'. The default panic hook still prints the panic unless the caller silences it, which upstream's silent catch does not.


## `via-lock`

*45 tests · clippy clean · 12 deviations*

- find_running_gateway is synchronous where upstream is async. Phase 0 has no tokio in the workspace dependency table and the only await was the poll delay; the delay is std::thread::sleep. The health probe is an injected trait (HealthProbe, with a blanket impl for Fn(&str) -> Option<serde_json::Value>) because there is no HTTP client in the workspace table either — tested with fakes.

- release(self) consumes the handle instead of flipping upstream's internal `released` flag. A second release is a compile error rather than a `false` return. There is deliberately no Drop impl, and that absence is compiler-enforced, not merely documented: release destructures `self` field by field, so adding `impl Drop for GatewayLeaseHandle` fails with E0509 on that line (verified by temporarily adding one).

- LeaseUpdate cannot express schema / instanceId / pid. Upstream accepts them in `fields` and then overwrites them via the trailing Object.assign argument; here the same guarantee is structural. Net behaviour is identical for every upstream caller.

- gateway_lock_path uses Path::join, not Node's resolve(), so it does not absolutize a relative config directory. Callers pass the already-absolute directory that VIA_CONFIG_DIR resolution produces (contracts.json documents it as an absolute path).

- `pid` is typed i64 with serde default. A document whose pid is a non-integer (e.g. 3.5) or a non-number fails to deserialize and the whole lease reads as absent, where upstream reads the lease and treats the pid as not-alive. Both paths reclaim the lease; only the contents of error.lease differ, and only for an already-corrupt file. Values outside i32 read as dead rather than reaching kill(2).

- Unrecognised fields in a foreign lease document are ignored on read rather than preserved. Upstream returns the raw parsed object, so its error.lease would carry them. Nothing upstream reads them, and this crate never writes a lease it did not construct.

- **KEPT, re-examined.** Windows: SystemProcessProbe answers SignalOutcome::Unsupported, which is_alive() reads as ALIVE, so a Windows build never reclaims a lease it cannot prove stale — a stale gateway.lock must be deleted by hand. File mode 0o600 and directory mode 0o700 are also not applied there. Upstream gets the probe for free because Node implements process.kill(pid, 0) on Windows. Doing it properly needs OpenProcess + GetExitCodeProcess **plus a process creation-time comparison** — a Windows pid alone is ambiguous because the kernel recycles ids, and a probe that omits the creation-time check will eventually call a recycled pid "alive" and refuse to start forever. Every route to those calls is a raw FFI binding (`windows-sys`/`winapi`), and `unsafe` is banned workspace-wide with `#![forbid(unsafe_code)]` on this crate; a safe wrapper such as `sysinfo` would be a process-table dependency taken for one boolean, on a platform this port has no machine to test on. A probe that is wrong in the *alive* direction wedges an install; wrong in the *dead* direction it steals a running Gateway's lease and corrupts a live session. An untested implementation of the second failure mode is worse than the documented refusal, so the refusal stands. What changed: `SystemProcessProbe::proves_liveness()` now reports the platform's capability at runtime (asserted on both arms), so a host can tell the user the incumbent may simply be a stale `gateway.lock` rather than reporting a Gateway that is not there — and `ProcessProbe` was already an injected seam, so a Windows host with a tested implementation supplies it without this crate changing at all.

- LeaseError::code() returns Option<&'static str>. Only the conflict carries a code upstream; Exhausted and Io answer None rather than inventing a string no client knows.

- LeaseError::Io from replace_lease names the temp path; upstream rethrows the original error unchanged. Cosmetic — the code path and the retry behaviour are identical.

- **CLOSED, and not the way the marker said.** The two lease literals ('已有 Gateway 正在运行[：<origin>]' and '无法获取 Gateway 实例租约') and the two CLI-lock literals (`另一个 via CLI 已在运行`, `无法获取 via CLI 实例锁`) are kept verbatim as the contract expectation. Each carried a `TODO(via-i18n)` saying it would become a catalog lookup once `via-i18n` existed. It exists, and the lookup is forbidden: `docs/architecture.md` §9 gives `shared → ∅` and `via-arch-test`'s `leaf_violations` enforces it, so a Leaf-band crate may not depend on `via-i18n` at all. `via-protocol` took the other exit — its `Display` is developer-facing English and only the catalog carries the sentence — but `via-lock`'s messages *are* the contract upstream's own callers match on, so that exit is not open here. The catalog carries the same four sentences (`lock.gateway_already_running`, `lock.gateway_already_running_at`, `lock.lease_exhausted`, `lock.cli_already_running`, `lock.cli_lease_exhausted`) for a caller that has a locale, and `via-conformance`'s `tests/localized_leaf_messages.rs` asserts the shipped `Display` output equals the catalog's `zh` value for every one — so the duplication cannot drift. The CLI pair are still NOT verbatim upstream, because `docs/rebrand.md` renames the CLI binary name inside them; via-conformance derives VIA's text from upstream's with that one substitution rather than retyping it.

- NEW in the gap-closure pass, and not a deviation so much as a hole that had to be filled: `src/cli.rs` implements `<configDir>/cli.lock` (upstream `cli/src/instance-lock.mjs`), which phase 0 left unwritten. It is deliberately *not* built on the gateway-lease machinery — upstream's two locks differ in file, document shape, attempt count, reclaim strategy and error surface, and the differences are upstream's rather than simplifications, so folding them together would have invented behaviour. Two consequences inherited from the lease port and repeated here: `release(self)` consumes the handle with no `Drop` impl (compiler-enforced by the same destructuring trick), and a string-typed `pid` in a foreign document reads as dead where upstream's `Number(existing?.pid)` would coerce it. `CliLockError` has no `code()` accessor at all, because upstream throws a bare `Error` for both refusals and inventing a code no client knows would be worse than none.

- `CliLockError::Exhausted` is unreachable without a race: it needs a second process to recreate `cli.lock` between this one's unlink and its create. Upstream's branch is race-only in exactly the same way. The message is asserted in both the crate and via-conformance so it cannot drift while the path stays unexercised — the same treatment via-store gives its Windows replace ladder.


## `via-audio`

*68 tests · clippy clean · 5 deviations*

- `FrameBuffer::append_pcm16le` deliberately does NOT drop an odd trailing byte, where the contract `PCM16 to float conversion` says 'an odd trailing byte is dropped'. Upstream's `pcm16Base64ToFloat32` decodes one self-contained base64 payload at a time, so its leftover byte is genuinely junk. A streaming accumulator fed by a socket gets chunks split at arbitrary offsets, and dropping the byte there does not lose one sample — it shifts every subsequent sample in the stream by one byte and turns the rest of the chunk into noise. The byte is carried into the next append instead. The contract behaviour IS reproduced exactly by the stateless `pcm16le_to_i16` / `pcm16le_to_f32`, which is where the contract test asserts it (tests/contracts.rs::an_odd_trailing_byte_is_dropped_by_the_stateless_decoder), and a second test proves the streaming path reassembles a stream split at every offset from 1 to 6 bytes.

- Base64 is not implemented. The contract `pcm16 -> f32 conversion` is stated over base64 input ('input is base64 PCM16LE'); this crate implements only the PCM16LE -> f32 half of it and leaves base64 to the wire codec, where the transport concern belongs. The divisor, the boundary values and the floor(bytes/2) sample count are all asserted here.

- Resampling, WAV, the frame buffer and duration accounting have no upstream to reproduce — they are new to VIA, so there is no upstream value to deviate from. The only upstream numbers that touch them are the rates and block sizes, reproduced exactly.

- The `max(160, ...)` and `max(240, ...)` floors from `tui/native/portaudio-voice-io.py:133,146` are reproduced even though they never bind at 16/24/48 kHz (where rate/50 is 320/480/960). They are part of the upstream expression, so they are implemented and tested at 4 kHz, 8 kHz and 12 kHz rather than simplified away.

- Not claimed as a locked contract, though a test exists: `sherpa KWS detection config` pins `featConfig {samplingRate:16000, featureDim:80}` plus nine other fields. Only the 16000 belongs to this crate, and tests/contracts.rs asserts it so a change to `SampleRate::HZ_16000` cannot desynchronise the two. The contract as a whole belongs to `via-wake-word`. **Split confirmed** (gap closure): `via-conformance`'s registry carries the row as `Entry::pending("default-value", "sherpa KWS detection config", Crate::ViaWakeWord)` — so it is counted against `via-wake-word`, never against `via-audio`, and it is *not* in the `gaps_in_shipped_crates` snapshot because its owner does not exist yet. The row now also carries a comment naming `via-audio`'s test, so whoever closes it reads the sample rate from `SampleRate::HZ_16000` rather than retyping `16000` a second time.


## `via-arch-test`

*29 tests · clippy clean · 7 deviations*

- docs/reference/contracts.json contains no entry in this crate's slice — the 707 contracts are wire/protocol/prompt values, and none is a crate-graph rule. This crate's contracts are upstream's dependency-boundaries.test.mjs and architecture.md §9, both asserted directly against their sources (the §9 tables are parsed out of the document at test time).

- Upstream's `app` row does not list `shared`, but VIA's App band may depend on Leaf. Not an invention: upstream's test special-cases its `root` layer (server/src/*.mjs entry files) as `root → app, process, shared`, and `apps/via` is VIA's counterpart to those entry files. UpstreamLayer::Root is therefore folded into Band::App and the union produces the leaf edge. Recorded in UPSTREAM_ADJACENCY's doc comment and asserted by tests/architecture_doc.rs.

- Four upstream layers merge pairwise into VIA bands, because upstream splits by directory and VIA by architectural layer: agent+process → layer3, conversation+task → layer2, app+root → app. Documented on UpstreamLayer::band().

- Band::Tests has no upstream row (upstream's tests live in server/test/ and import freely), so its allow-set is given explicitly as every band rather than derived. A test guards that every *other* band still maps from at least one upstream row, so the derivation cannot silently go empty.

- The band rule and the via-backends rule apply to dev-dependencies as well as normal/build; upstream's regex walked server/src only and never server/test. In Rust a dev-dependency is a real workspace edge, and a leaf whose tests reach into core is exactly the drift the gate exists to catch. Reachability and cycle detection deliberately exclude dev edges — cargo permits a dev back-edge and via-realtime dev-depending on via-realtime-mock is a normal pattern, and closing over dev edges would make every crate 'reach' via-backends through via-conformance. Both directions are asserted.

- I created apps/via, which is outside my assignment. The workspace was unresolvable: root members includes `apps/*`, apps/ was an empty directory, and a members glob expanding to nothing is not ignored — cargo treats `apps/*` as a literal path and fails the whole workspace ('failed to read apps/*/Cargo.toml'). cargo build -p via-lock failed too, so this blocked every crate, not just mine. I did not edit the root Cargo.toml. apps/via is a behaviour-free skeleton (fn main() {}) whose manifest comment states exactly why it exists and that the CLI surface — gateway/chat/config/backend/mcp-serve/service, all contract-bearing — belongs to the phase that implements it. It should be overwritten freely by whoever owns that crate.

- via-arch-test has a real src/ rather than an empty lib.rs. The task permitted either; a library lets the rules be exercised against synthetic graphs and gives every public item the required doc comment. The assertions still live in tests/.


## `via-conformance`

*66 tests · clippy clean · 12 deviations*

- GATEWAY_PROTOCOL_VERSION: upstream is '2.0.0', via-protocol ships 1.0.0. Not reproduced, by via-protocol's own recorded decision (it advertises a strict subset of upstream's capabilities, and a removed capability is a breaking change under upstream's own rule). Classified `Divergent`; `tests/gateway_protocol.rs::protocol_version_starts_a_via_line` asserts the upstream literal (both catalogue spellings), VIA's value, that they differ, and that both are semver triples.

- GATEWAY_CAPABILITIES: upstream advertises 16, VIA advertises 7. Classified `Divergent`; `capability_list_partitions_the_upstream_sixteen` asserts that upstream's 16 partition exactly into `GATEWAY_CAPABILITIES` (7) and `DROPPED_UPSTREAM_CAPABILITIES` (9) *in upstream order*, that the two are disjoint, and that `advertises_capability` agrees with both. Nothing is dropped silently.

- The DashScope unknown-model-id fallback (6 contracts: `DashScope model catalog ids and profiles`, `DashScope realtime model catalog`, `model profile session defaults`, `turn detection by model family`, `model profile capability flag sets`, `model capability shape (fail-closed for unknown ids)`): upstream returns an all-capabilities-false profile with `family: 'unknown'`; via-catalog returns `Err` (architecture.md §7). Classified `Divergent` — the four profiles, the five constants, both capability-flag key orders and the two turn-detection modes are all asserted byte for byte, and the replacement behaviour is asserted for three unknown-id shapes including a local GGUF id.

- backend credential allow-lists (`env-var/backend credential allowlist per backend`): the upstream names `QWEN_AUDIO_AGENT_*` and `QWAUDIO_CONFIG_DIR` are renamed to `VIA_*` per docs/rebrand.md. Classified `Divergent`; the test parses all 12 upstream segments out of the catalogue, pushes every name/prefix through `value::rebranded` (the documented rule, not a retyped constant), compares against the shipped `EnvironmentPolicy`, and asserts no upstream identity prefix survives anywhere — a *partial* rename fails.

- `state-name/Work status values` and `state-name/workState values` are asserted as SETS, not as ordered lists. Upstream declares no order for either: `workState` is computed (`ACTIVE.has(status) ? 'active' : status`, task-manager.mjs:58) and `status` is a set of literals assigned across the file. The catalogue's listing order is the surveyor's, so locking it would assert something nobody promised. WorkStatus's declaration order is additionally asserted, but as VIA's own lifecycle order (architecture.md §4), and the test says so.

- `env-var/backend credential allowlist per backend` abbreviates openclaw's last three names to their suffixes (`QWEN_AUDIO_AGENT_OPENCLAW_MODEL, _MODEL_ID, _STATE_DIR, _WORKSPACE`). It cannot be read literally, so the test expands the three abbreviations once, explicitly and with a comment, before parsing.

- 7 contracts are classified `Partial` rather than `Asserted`, because part of the exactValue belongs to a crate that does not exist yet: `default-value/backend default base URLs` (port envs → via-backends), `default-value/provider aliases and configured gating` (isConfigured → via-realtime), `env-var/backend workspace overrides` and `env-var/${QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE}` (resolution rule → via-core), `state-name/backend integration modes and configuration modes` (inspectBackend outputs → via-backends), `json-field/pi backend capability declaration` (driver capability flags → via-backends), `cli-flag/--backend` (the flag → apps/via). The part VIA owns is asserted; the summary counts Partial separately so it can never read as done.

- 1 contract is classified `Behavioural`: `ws-event/input.suspend / input.resume / input.suspend.ack`. Its three names are locked by the vocabulary tests, but the exactValue is prose about payloads and the `playback.clear` coupling, which is via-voice's. It is owned by a named test in that crate rather than string-compared here.

- ~~`default-value/client input capability profiles` is `Pending` against via-protocol even though via-protocol ships.~~ **CLOSED.** `via-protocol` grew `src/client.rs` (`ClientType`, `ClientInputCapabilities`), and `tests/client_input.rs::client_input_capability_profiles` parses all three profiles — key order included — out of the catalogue rather than retyping them. `gaps_in_shipped_crates` is now empty.

- OWNERSHIP CORRECTED: the three `file transaction lock` rows (`default-value/file transaction lock parameters and on-disk artifacts`, `error-code/file transaction lock timeout code`, `file-path/shared file transaction lock`) were owned by `via-lock` and are owned by `via-store` now. `with_file_transaction` and the mkdir-based lock live in `via-store` because the lock exists to guard exactly the shared documents that crate reads and writes; `via-lock` owns the Gateway *instance lease*, which is a different file answering a different question. The row's `owner` is defined as "the crate whose code has to carry it", so this is a correction, not a reclassification.

- SIX ROWS ARE `Divergent` FOR A RENAME AND NOTHING ELSE (`via-log`'s schema string and `QWEN_AUDIO_LOG_*`/config-dir prefixes; `via-lock`'s lease schema, `QWAUDIO_` error-code prefix and the CLI binary name inside two messages). Every one is derived in the test from the upstream literal via `value::rebranded` / the new `value::rebranded_schema`, never retyped, so a *partial* rename fails. A seventh, `default-value/logger redaction patterns and caps`, is `Divergent` for a spelling rather than a rename: the test translates each JavaScript regex literal into the Rust pattern it must become by two documented rules (`/…/i` → `(?i)…`, `\b` → `(?-u:\b)` because JavaScript's word boundary is ASCII-only) and compares byte for byte, so the four patterns cannot be hand-edited apart.

- FIVE MORE `Partial` ROWS, on top of the original seven, because part of the exactValue belongs to a crate that does not exist yet: `default-value/gateway polling timeouts` (waitForGateway / waitForGatewayStop / the health fetch timeout → apps/via), `json-field/health fields the CLI reads` (everything but `gatewayInstanceId` → via-app), `env-var/QWAUDIO_CONFIG_DIR` and `file-path/files and directories the product creates` (the platform base directory and everything created under it → via-core), `file-path/backend session state file` (the registry's path, its env override and its two record shapes → via-acp). Each test asserts the owned half *and* asserts the owed half is still catalogued, so a Partial row cannot quietly become stale.

- `via-conformance` now depends on five crates rather than two (`via-protocol`, `via-catalog`, `via-log`, `via-store`, `via-lock`). `via-audio` is deliberately absent: it owns no catalogued contract, so an edge to it would claim coverage that does not exist.

