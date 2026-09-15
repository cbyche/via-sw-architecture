# Phase 2 deviations

Layer 3 — the `DownstreamAgent` seam, the generic ACP client, process
supervision, and the twelve named backends.

**52 deviations across 4 crates, 660 tests.**

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.

## The rule this phase exists to enforce

`via-acp` and `via-process` never name a backend — not in code, not in a match
arm, not in a test fixture. Upstream asserts that with a regex over source text;
here it is a crate boundary, and `via-arch-test` proves the edge is absent.
Both crates took it further than the rule required: their test fixtures select
catalog entries by *property* ("the first backend with no service address",
"the first that is always full-permission") rather than by id.


## `via-downstream`

*191 tests · clippy clean · 10 deviations*

- bounded() counts UTF-16 code units (matching JS `slice`) but stops one character short rather than emitting a lone surrogate, which a Rust String cannot hold. Identical to upstream for all BMP text. Tested.

- clean()/bounded() use ECMAScript's `\s` set exactly (WhiteSpace ∪ LineTerminator ∪ U+FEFF) rather than Rust's `char::is_whitespace`. The two disagree in both directions — U+0085 is White_Space but not `\s`; U+FEFF is `\s` but not White_Space — and a U+FEFF that went uncollapsed would sit invisibly inside a 'bounded' detail. This narrows the shipped phase-1 convention (via-core used plain `str::trim`); flagged in case the reviewers want it unified.

- An unrecognised ActivityStatus becomes `running` where upstream forwards the string verbatim. The status is model-visible (interpolated into the progress prompt and session_status), so an unbounded attacker-chosen string on a typed state field is exactly what the boundary exists to stop. No downstream behaviour changes: an unknown status is not completed/failed upstream either, so the tracker keeps the entry identically. Tested, including a status crafted to break out of the JSON.

- CancelOutcome::Requested reports `cancelling`; upstream reports `cancelled` because cancelDelegation writes record.status = 'cancelled' before awaiting anything. This is the deliberate correction docs/architecture.md §4/§6 asks for ('cancelling is a state, not an action'). Asserted against the catalogued session_cancel result so both the upstream value and the divergence are on record.

- HarnessRegistry::resolve("") returns NotConfigured rather than upstream's 不支持的后台 Agent： with an empty id. Frontend-only is a supported mode (architecture §2, agent degrades to direct), not a misconfiguration; the i18n key agent.not_configured already exists for it.

- Two capability consistency rules are VIA additions with no upstream statement — permissions must be the negation of the catalog's alwaysFullPermission, and backendUi must agree with the catalog's defaultBaseUrl. Both were read off the twelve upstream declarations (Pi is the only alwaysFullPermission and the only permissions:false; OpenCode and OpenClaw are the only defaultBaseUrl and the only backendUi), and tests/capability.rs asserts all twelve satisfy every rule. Same for the three self-consistency rules (delegation == sessionMcp||nativeDelegation; the two transports are exclusive; sessionMcp implies externalMcp). All five render as upstream's single message, 后台 Driver 能力声明不完整：<id>.

- Three catalogued driver-validation refusals have no runtime analogue and are not reproduced: 后台 Driver 缺少 createProfile (DownstreamAgent cannot be implemented without open), and the two skills refusals (via_catalog::SkillsSpec has no unset variant). The 'incomplete capabilities' half of upstream's check — a missing flag, a stray non-boolean — is likewise unrepresentable: seven bool fields plus deny_unknown_fields. tests/contracts.rs asserts these segments are still in the catalogue so the deviation stays honest.

- backendFailureCode()'s regex classifier (error text -> NOT_INSTALLED / AUTH_REQUIRED / ...) is deliberately NOT ported here — it reads a transport's error text, so it belongs beside the state machine that produces it, in via-acp. The seam owns the closed vocabulary (HarnessStatusCode) those codes come from.

- Duplicate registration is upstream's last-wins Map semantics, but register() returns the displaced Arc rather than discarding it silently, because registration is configuration in VIA rather than a hard-coded list.

- ActivityTracker is unbounded, exactly as upstream's `known` Map is, so the two agree on which updates project to what. len()/clear() are exposed so the session's owner (via-acp) can end the lifetime. Noted rather than 'fixed' because inventing a cap would change projection behaviour.


## `via-acp`

*137 tests · clippy clean · 13 deviations*

- i18n: added two keys to crates/via-i18n/assets/i18n/acp.json — `acp.process_error` (`{label} ACP {detail}`) and `acp.process_error_with_stderr` (`…：{stderr}` in zh, `…: {stderr}` in en/ko). These are upstream's `processError()` composition wrapper. Without them the message/stderr separator would be a literal in via-acp, which the 'never a literal' rule forbids; they mirror the existing acp.request_failed / acp.request_failed_with_stderr pair. Additive only — via-i18n's build.rs parity checks and its own tests still pass.

- initialize.clientCapabilities serializes as {"fs":{"readTextFile":false,"writeTextFile":false},"terminal":false} rather than upstream's literal `{}`. Semantically identical (declares nothing); byte-different because the SDK's ClientCapabilities spells its two false fs flags out. `client_capabilities()` + `declares_no_capability()` make the actual contract — 'the agent is promised nothing' — the thing that is asserted.

- VIA_NODE stays on INTERNAL_NAMES but is never stamped: upstream stamps `process.execPath` for its scripts/*.mjs shims and VIA has no Node interpreter of its own. VIA_ENV_LOADED='1' is always stamped, and a launch spec that genuinely needs a runtime supplies VIA_NODE as an addition.

- Start-up failure classification is deterministic instead of raced. Upstream races `initialize` against the child's `exit` event, so which of `进程意外退出` / `初始化失败` appears depends on ordering. Here the question is asked: try_wait → exited? report the exit; else stderr non-empty? report `初始化失败`; else report the transport failure. Same message set, no race.

- stderr is attached to a request failure when `is_incoming_transport_closed(error)` (the SDK's own transport discriminator) rather than upstream's `error.name === 'RequestError'`. The SDK's Error type carries no such marker. Behaviour matches upstream on both tested cases: a JSON-RPC error response omits stderr, a dead transport attaches it.

- Windows teardown is process-wrap's JobObject `start_kill` rather than upstream's `taskkill /PID /T` then `/F`. The ladder collapses to one step on Windows because there is no signal meaning 'please exit'; the SIGTERM→poll→SIGKILL ladder is unix-only, as upstream's is.

- `stopProcessTree` idempotence is a `stopped` flag on AcpChildHandle rather than upstream's per-child WeakMap of in-flight cleanups. Same guarantee (a second call is a no-op), one owner.

- server/test/backend-environment.test.mjs is ported in two halves. Its third case (generic-ACP explicit forwarding) is ported here against via-catalog's catalogued `acp` policy. Its first two cases name deepseek/claude/openclaw/opencode, and this crate may not name a backend (`via-arch-test` says a test needing a real backend fixture belongs in via-backends), so the same security property is asserted here with invented namespaces — projection, isolation between two policies, case sensitivity, empty-value passthrough — and the named-backend version was owed by via-catalog/via-backends. **It is paid**: `crates/via-backends/tests/backend_environment.rs` runs the same four properties against the twelve real backends, and adds `no_gateway_secret_reaches_any_of_the_twelve`.

- The scripted fake ACP agent is a `[[bin]]` at tests/bin/fake_agent.rs, not a test file, because CARGO_BIN_EXE_<name> is the only mechanism Cargo gives an integration test for locating a companion executable. It is under tests/bin/ so auto-discovery does not also claim it as a test target.

- `clean` on a JSON array or object renders as JSON where JavaScript's String() gives "1,2" / "[object Object]". Unreachable in practice — every call site reads a protocol field declared as a string — and JSON is at least diagnosable.

- AcpHarnessSession::events() returns an empty stream. The projection itself is via_downstream::ActivityTracker (which owns backend.activity); wiring it to a broadcast fan-out needs to know which Work a session's activity belongs to, which is via-backends'. Documented on the method so a subscriber gets a stream that ends rather than one that lies about being live.

- Dropped `via-protocol` and `tokio-util` from crates/via-acp/Cargo.toml: neither is used, and an unused edge is a lie in the crate graph via-arch-test reads. Every other dependency named in the brief is used.

- The catalogue's prose for `backend child environment allowlist` says '47 OS names' but the list it then enumerates — and shared/backend-environment.mjs:7-47 — both hold 39. The list is treated as authoritative; the test asserts 39 in both directions against the catalogued names, so a widened boundary fails.


## `via-process`

*127 tests · clippy clean · 17 deviations*

- managedScript -> ManagedLaunch. Upstream's driver field is a `scripts/*.mjs` file name spawned with `process.execPath`; docs/architecture.md §10 says the Node shims do not survive the port, so the driver carries a command + args resolved on the child's PATH. Upstream's refusal message (`後台 Runtime Driver 缺少 managedScript：<id>`) is kept unchanged.

- VIA_NODE is not stamped. Upstream always adds QWEN_AUDIO_AGENT_NODE=process.execPath so a Node shim can re-exec the same interpreter. VIA spawns a real command, so there is no interpreter path to publish. The name stays on INTERNAL_NAMES so an embedder that sets it still gets it through.

- ELECTRON_RUN_AS_NODE=1 is not hard-coded. Upstream's spawnSpec adds it unconditionally; it is a Node/Electron-runtime concern, so it becomes a ManagedLaunch::environment_additions entry supplied by whoever builds the driver.

- Two of validateRuntimeDriver's five checks are unrepresentable. `後台 Runtime Driver 缺少 resolve` and `...缺少進程歸屬聲明` test that a JS object has a function-typed and a boolean-typed property; in Rust those are typed struct fields, so neither state exists. The other three are reproduced with upstream's messages.

- validate_runtime_driver is a STRENGTHENING in two places: (a) it requires the driver's base-URL variable and default base URL to agree with the catalog — upstream asserts that in its registry *test*, not its validator, and only gets away with it because its driver map is a module literal; VIA's registry is caller-built, so an unregistered service backend would otherwise be manufactured with no address at all and the Gateway would connect nowhere. (b) a driver declaring a separate managed process must declare a service address as well as a launch command — upstream would reach `new URL(null)` and throw an untyped TypeError. Both reuse upstream's existing message.

- validate_runtime_driver takes the BackendDefinition as a parameter (upstream's own `validateRuntimeDriver(driver, definition)` signature) rather than looking it up. That is what lets this crate's tests exercise every refusal against a synthetic definition without writing a backend's name into a file that is forbidden to hold one.

- The generic full-permission refusal runs for every driver, not only for `managedProcessDriver`-manufactured ones. Behaviour is identical for all 12 shipped backends (the one driver with a bespoke refusal registers a hook, which runs first); it just closes a hole a hand-built driver could otherwise walk through.

- `backend.process_started` publishes `childPid`, not `pid`. Upstream writes `pid: child.pid` and loses it: `pid` is one of the seven names its logger stamps *after* the caller's fields (shared/logger.mjs:227,302), so the record carries the Gateway's own pid and the child's never reaches the log. via-log reserves the same seven for the same reason. A test asserts both fields to keep the reasoning visible.

- start_managed_backend returns ManagedBackendStart { backend, runtime } rather than the runtime alone. The caller needs base_url *after* a reallocation may have changed it, and recovering it from the mutated env would mean re-deriving which variable to read — driver knowledge this crate has and the caller should not need.

- The failure/stderr separator is a per-locale const in this crate, not a via-i18n key. Upstream joins with the full-width colon `：` — Chinese punctuation inside an otherwise localized sentence — and via-i18n has no punctuation-only key and is already shipped.

- backend_failure_code does NOT classify Korean messages. Upstream's regexes cover en and zh only; VIA ships a `ko` locale, so a Korean failure lands on START_FAILED. Widening the patterns would be a behaviour change rather than a port, so it is left out and asserted as-is.

- `applyLocalAddress`'s port fallback is reproduced with its quirk intact: an explicit port wins, else 443 for `https:` and 80 for everything else *including* `wss:`. Only reachable for an owned backend whose address was just reallocated (and therefore always carries an explicit port), so the quirk is inert; a test pins it so a future 'fix' has to argue with it.

- `stop()` does not wait again after escalating to SIGKILL — upstream returns as soon as it has escalated, and adding a wait would change how long Gateway shutdown can block.

- SupervisedChild gains take_stdout/take_stderr (defaulting to None) and SpawnSpec gains ChildStdio::Piped. Upstream never pipes a managed backend (`stdio: 'inherit'`, which stays the default), but BackendRuntimeState::failed appends a captured stderr tail and something has to capture it.

- `detached: platform !== 'win32'` has no field. Every managed child is a process-group leader on unix / a Job Object on windows via process-wrap, which is strictly stronger and is what docs/architecture.md §12 chose the dependency for; upstream has no Windows containment at all.

- PATH composition is reused, not restated: via_core::search_path::{merge_search_path, command_directory} (the port of shared/path-environment.mjs) is called by compose_child_search_path, which adds only the executable-entry cleaning from shared/backend-install.mjs:206-231. A test asserts the two agree when nothing needs dropping.

- The OpenClaw-specific full-permission refusal (catalogued in error-code/runtime-ownership errors) is reachable here only through ProcessError::DriverRefused, which carries the via-i18n *key* rather than the rendered sentence. Producing that particular key is via-backends' job — naming it in this crate would breach the no-backend-names rule.


## `via-backends`

*205 tests · clippy clean · 12 deviations*

- ELECTRON_RUN_AS_NODE=1 is not stamped on any child. It exists upstream only because the command is `process.execPath` (an Electron binary under the desktop app) running a `scripts/*.mjs` shim. architecture.md §10 removes that whole mechanism, so the variable has nothing to act on; setting it would change the behaviour of any Electron tool the agent later spawns. Documented on driver.rs. VIA_NODE is likewise not stamped, matching via-acp's existing deviation.

- The six shim-launched backends launch the executable the shim would have exec'd, not `process.execPath scripts/<name>.mjs`: `opencode acp`, `openclaw acp --url … --verbose`, `codex-acp`, `claude-code-acp`, `pi-acp`, `dsh-acp-demo --config <cordis.yml>`. Every environment variable the shim set is still stamped (CODEX_PATH, NO_BROWSER, CLAUDE_CODE_EXECUTABLE, ANTHROPIC_API_KEY←CLAUDE_API_KEY, PI_BIN/PI_ACP_PI_COMMAND, OPENCODE_MODEL, XDG_CONFIG_HOME isolation).

- The managed-service argv carries `--port` and is therefore fixed when the runtime driver is built, where upstream's shim reads OPENCLAW_PORT/OPENCODE_PORT at start-up. `runtime_registry(env)` reads the environment it is given and `managed_launch` is exposed so a caller that lets start_managed_backend reallocate a busy port can rebuild. Documented at the top of runtime.rs.

- src/json5.rs is a JSON5→JSON normaliser rather than a full JSON5 parser: comments, unquoted identifier keys, single-quoted strings and trailing commas. Hex/leading-dot numbers, Infinity/NaN, +1 and non-ASCII identifier keys are not translated and yield None, which every caller already treats as 'no token to reuse'. Adding a `json5` crate to the root manifest for one field read was not worth the dependency.

- The three sessionInstructions paragraphs (OpenClaw, DeepSeek, Pi) are `pub const` literals in profile.rs rather than via-i18n keys. They are model-facing English, upstream does not localise them, and contracts.json pins the English text — translating them would break the assertion they exist to make.

- The two negative-lookahead regexes (`qoder-status`'s Account line, DeepSeek's `DEEPSEEK_API_KEY: ""` placeholder) are reproduced line-by-line in Rust because the `regex` crate has no lookahead. Both are covered by tests including the boundary cases ('Account: Not logged inbox' is a username, `DEEPSEEK_API_KEY: ''` is not a credential).

- installBackend's Windows npm fallbacks: the direct PATH scan and hard-coded-directory scan are kept in shape via find_executable('npm.cmd')→find_executable('npm'); the PowerShell `Get-Command npm` fallback is not ported (it spawns a shell to answer a question the PATH walk already answers). Likewise `npm config get prefix` is not spawned to learn the global bin directory — upstream's own fallback values are used directly.

- Installer cancellation is an explicit `InstallCancel` flag rather than dropping the future, because the installer must distinguish CANCELLED from STEP_FAILED in its result.

- BackendAvailability::new() does not probe (upstream's constructor does not either); BackendAvailability::start() is the two-line Gateway call site — new + one eager background refresh — in one call. snapshot() spawns its background refresh on the ambient tokio runtime and is a no-op when there is none, rather than panicking.

- write_private_file delegates to via_core::runtime::write_file(IfExists::Replace), which already performs upstream's mkdir-0700 → temp → chmod-0600 → rename through via-store; no second temp-and-rename is hand-rolled here.

- `inspect_backend_setups`'s OpenCode/OpenClaw `source` runtime branches check the source directory and the toolchain binary (bun / corepack) but do not additionally stat `packages/opencode/src/index.ts` and `node_modules/@opencode-ai/tui`; the report shows the directory either way and the extra stats only refine an issue message.

- HarnessError::UnsupportedBackend vs NotConfigured (via-downstream's existing split) is inherited: register_backends skips only the generic ACP backend's two configuration refusals, so a Gateway with no ACP_COMMAND still registers the other eleven instead of failing startup.

