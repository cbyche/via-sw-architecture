# Phase 1 deviations

Every place a phase-1 crate does **not** reproduce an upstream value or
behaviour exactly, with the reason. This is the per-phase detail behind
[`fidelity.md`](../fidelity.md); that document carries the policy, this one
carries the receipts.

A deviation here is not a defect. It is a place where reproducing the upstream
literally would have been wrong (a JavaScript idiom with no Rust meaning), or
impossible (a value the upstream computes from a runtime VIA does not have), or
a decision VIA took deliberately and wrote down. What would be a defect is an
undocumented one.


## `via-core`

*175 tests · clippy clean · 19 deviations*

### Structural

- **The realtime frontend is nested, not flattened.** Upstream spreads it across
  `config.audioProvider`, `config.audioModel`, `config.audioVoice`,
  `config.audioRealtimeBaseUrl`, `config.dashscopeApiKey`,
  `config.realtimeConfigSignature` and four `speechToSpeech*` fields. VIA has one
  `Config::realtime: RealtimeFrontend`, because those ten values are resolved by
  one function and are meaningful only together. Every field keeps upstream's
  name inside it, so `/api/health` can still be assembled field for field.

- **`Config::default()` leaves every path field empty.** The paths are *derived*
  from the resolved config and data directories, which depend on the host, so
  there is no correct literal for them. `resolve()` fills them in; the filenames
  they are built from are `file-path` contracts owned by `via_core::paths`, not
  `default-value` contracts. The rule "one `impl Default` is the only place a
  default literal appears" is kept for every *value* default.

- **Two literals are named constants referenced from `Default` rather than typed
  inline.** `DASHSCOPE_COMPATIBLE_BASE_URL` is both the memory extractor's
  default base URL (`config.mjs:454`) and the endpoint Codex/CodeBuddy are
  pointed at when a Bailian model is configured (`config.mjs:280,293`). One
  constant, referenced twice, is the whole point of the rule; two copies is how
  they drift.

- **`resolve` takes the host facts as parameters.** `os.homedir()` and
  `process.cwd()` are not environment reads, so a pure function cannot invent
  them. They live on `Overrides` (`home_directory`, `working_directory`,
  `runtime_root`), and `Overrides::from_process()` is the impure convenience
  that fills them. `runtime_root` is upstream's `sourceRoot`
  (`config.mjs:19`) — the installation fact `VIA_RUNTIME_ROOT` overrides.

- **`root` is normalised against the working directory.** Upstream uses
  `process.env.QWEN_AUDIO_AGENT_RUNTIME_ROOT` raw and lets each later
  `resolve(root, value)` absolutise it. Folding that once up front is
  equivalent — Node's `path.resolve` would have resolved a relative `root`
  against the cwd anyway — and it makes `config.root` an absolute path every
  caller can rely on.

### Values

- **`VIA_SLEEP_TIMEOUT_SECONDS`, of three candidate names.** `docs/rebrand.md`
  gives three spellings for upstream's `QWEN_AUDIO_DESKTOP_AUTO_HIDE_SECONDS`:
  `VIA_SLEEP_TIMEOUT_SECONDS` (row 133), `VIA_DESKTOP_AUTO_HIDE_SECONDS`
  (row 134) and `VIA_AUTO_HIDE_SECONDS` (row 90). VIA takes row 133's, the most
  narrowly scoped of the three and the only one that still means anything once
  `desktop/` is dropped: there is no orb to auto-hide. Upstream's legacy
  `QWEN_AUDIO_SLEEP_TIMEOUT_SECONDS` is deliberately ignored there; VIA has no
  legacy name to ignore, so no alias is implemented.

- **`config.wakeWord` is configuration, not a literal.** Upstream hard-codes
  `你好千问` with no override (`config.mjs:503`). `docs/architecture.md` §16
  makes the phrase "a configuration value, never a literal" and defers the
  choice to phase 6, so `VIA_WAKE_WORD` is read and the default is the **empty
  string** rather than a placeholder that would ship. `scripts/brand_leak.py`
  would have rejected the upstream literal in any case.

- **The audio-family voice variable is `VIA_REALTIME_VOICE`.**
  `docs/rebrand.md` row 89 offers `VIA_AUDIO_REALTIME_VOICE` as an alternative.
  `via-catalog` already shipped `AUDIO_FAMILY_VOICE_ENV = "VIA_REALTIME_VOICE"`
  in phase 0 and owns the family→variable mapping; `via-core` reads it from
  there rather than introducing a second spelling.

- **`webDistributionPath()` is not ported.** `install-paths.mjs`'s only function
  resolves the packaged React SPA. VIA drops `web/` (`fidelity.md`) and does not
  advertise `web.same-origin-ui`, so there is no directory to locate.

- **The setup gate has three provider arms upstream does not.** Upstream
  branches `dashscope` / everything-else. VIA has five providers
  (`architecture.md` §7), so `openai` gets a `configured` predicate
  (a non-empty realtime API key) and a new settings field `realtimeApiKey`,
  while `local-omni` and `mock` are always configured — the pipeline runs
  in-process and the mock is a fixture. Each arm is marked in the source as a
  VIA extension.

- **The voice override for non-DashScope providers reads the `local` family's
  variable.** Only the `dashscope` provider consults the DashScope model
  catalog, because only its ids are in it. `openai` and `local-omni` carry
  user-chosen coordinates, so their override is `VIA_LOCAL_REALTIME_VOICE`
  rather than either DashScope-family variable. `via-realtime-openai` may
  narrow this; nothing upstream covers it.

### Behaviour

- **An unknown realtime model id is an error.** Inherited from `via-catalog`'s
  phase-0 decision (`architecture.md` §7): upstream's all-capabilities-false
  fallback opens a session that connects and never hears anything.
  `resolve()` therefore *fails* on `VIA_REALTIME_MODEL=<unknown>` where upstream
  starts.

- **Numeric settings are truncated toward zero.** `numberSetting` returns a
  JavaScript `Number`, so `VIA_TASK_MAX_CONCURRENT=2.5` is `2.5` upstream and is
  then used in comparisons where the fraction is invisible. Every catalogued
  setting is a count, a millisecond duration or a port; `integer_setting`
  truncates and saturates. `number_setting` keeps the exact `f64` semantics and
  is public and tested, including `Number()`-vs-`parseInt` disagreements
  (`"12abc"`, `"1e4"`, `"0x10"`) and the crossed-bounds case where `Math.min`
  applied last lets the maximum win.

- **An opaque `Origin` is refused rather than fatal.** Upstream's
  `normalizedOrigin` turns `data:`/`blob:` into the *string* `"null"`, and
  `new URL('null')` on the next line throws — which Express turns into a 500. A
  500 and a 403 are both a denial; a security boundary should not have a crash
  path, so VIA returns `false`.

- **The env-file parser is a faithful subset.** Node's `util.parseEnv` is
  reproduced for comments, `export`, quoting, `#`-truncation of an unquoted
  value and last-assignment-wins; its `\n` unescaping inside double quotes is
  not, because no VIA template or documented value uses it.

- **`ACP_LABEL` etc. are trimmed with Rust's `str::trim`.** Upstream's
  `String.prototype.trim` and Rust's `trim` agree on every character either
  would meet here; the difference is confined to a handful of exotic code
  points neither documents.

- **Secrets are a newtype.** `Config` holds `Secret` for `auth_secret`, the
  DashScope key, the memory key, the speech-to-speech token and the OpenClaw
  gateway token. `Debug` and `Serialize` both emit `[REDACTED]`; reading takes
  an explicit `.expose()`. Upstream has no such wrapper, and a `Config` formatted
  with `{:?}` while debugging a startup failure is exactly how a key reaches a
  log file.

- **`assert_full_permission_allowed` is exported but not applied by `resolve`.**
  Upstream checks "full permission needs an owned backend" in
  `managed-backend.mjs`, at spawn time, not in `config.mjs`. Applying it at
  configuration time would refuse a startup upstream accepts, so `via-process`
  calls it.

### Templates

- **The seeded templates are localized.** Upstream's `config.env`, `USER.md` and
  `MEMORY.md` templates are Chinese literals; VIA renders them from `via-i18n`,
  so an `en` install gets English and a `zh` install gets upstream's own wording
  with only the identity renamed. The header line — a catalogued contract — is
  `runtime.config_header`.

- **Three example lines and one prose line are dropped from the templates.**
  Upstream's `USER.md` carries three `<!-- 例如：… -->` illustrations and its
  `config.env` carries `# DeepSeek（Harness Developer Preview）：DEEPSEEK_API_KEY=your-key`.
  None has a `via-i18n` key. The examples are omitted; the DeepSeek line becomes
  the bare commented assignment `# DEEPSEEK_API_KEY=`, so the variable stays
  visible without shipping untranslated prose.

### Workspace

- **Three dependencies were added to `[workspace.dependencies]`**, plus the two
  phase-1 crates: `hmac` (RFC 2104 over the `sha2` already present — the owner
  cookie is an HMAC, and hand-rolling ipad/opad inside a security boundary is
  the wrong trade), `url` (the origin allow-list is expressed entirely in terms
  of WHATWG `new URL(...)`: default-port stripping, IPv4 shorthand
  normalisation, IPv6 bracket form, host-vs-hostname — a hand-rolled splitter
  gets all four wrong and each one is a hole), and `getrandom` (the auth secret
  is `randomBytes(32)`; there was no CSPRNG in the table and `getrandom` was
  already in the graph via `uuid`).


## `apps/via`

*277 tests · clippy clean · 21 deviations*

### The command surface

- **Six verbs, not upstream's eight.** `docs/architecture.md` §10 is the
  specification, and it is not `cli-commands/COMMANDS`. The mapping, asserted
  by `tests/contracts.rs::every_catalogued_verb_has_a_recorded_disposition`
  against the catalogue's own list, is: `gateway run` → `via gateway`;
  `gateway install|start|stop` → `via service install|start|stop`; `tui` →
  `via chat`; `setup` → `via backend status`; `install NAME` → `via backend
  install NAME`; `webui` and `skill` dropped; `status` (an alias for
  `gateway status`) owed. A catalogued verb with no disposition fails that
  test, so none of this can go quiet.

- **Three of upstream's seven `GATEWAY_ACTIONS` are owed.** §10 lists
  `install|start|stop`; `restart`, `status` and `uninstall` have no §10 verb.
  They are not silently dropped: `ServiceAction::Unknown` catches them and
  reports the catalogued *"unsupported Gateway service action: `<x>`"*, which
  is the sentence a user carrying upstream muscle memory should see.

- **Three of the fourteen `cli-flag` contracts have no home in this build.**
  `--audio-mode` is `tui`-only and `via chat` carries no audio, so it is
  deferred with the TUI; `--no-open` went with `webui`; `--skill` / `--list`
  went with the `skill` verb. Each is recorded in `FLAG_DISPOSITION`
  (`tests/contracts.rs`), which is asserted to be exactly the catalogue's
  fourteen names, and each is additionally asserted to be *rejected* by the
  parser rather than quietly accepted.

- **`--takeover` moved from `tui`/`webui` to `chat`.** It is catalogued as
  tui/webui-only; both are gone, and `via chat` is the client that replaces
  them (`docs/fidelity.md`: *"VIA ships `via chat` … so the core is drivable
  end to end"*). Upstream's own text CLI has no such flag, so this is an
  adaptation rather than a port.

### Argument parsing

- **`clap` rejects a misplaced flag structurally, so eight catalogued
  sentences are not reproduced.** Upstream accepts every flag on every command
  and then refuses one at a time — `--json 只适用于 setup`, `--yes 只适用于
  install`, `--audio-mode 只适用于 tui`, `--no-open 只适用于 webui`,
  `--takeover 只适用于 tui 或 webui`, `--skill 只适用于 skill install`,
  `--list 只适用于 skill install`, `--realtime-model 只适用于 config set`.
  Declaring each flag on the command that owns it enforces the same
  constraint, earlier, and makes it visible in `--help`. The *message* is
  `clap`'s; the *constraint* is asserted directly, flag by flag, in
  `src/cli.rs::a_misplaced_flag_is_a_usage_failure` and
  `tests/contracts.rs::the_flags_via_kept_are_accepted_and_the_ones_it_dropped_are_not`.
  The same applies to `未知参数：<x>` and `<option> 缺少参数`.

- **A usage error exits 1, not `clap`'s 2.** The catalogued exit-code contract
  says *"1 = any thrown error"*, and upstream throws for an unknown flag
  exactly as it throws for anything else. A script that branched on the number
  would otherwise see two values for one class of mistake.

- **`config set` without `--realtime-model` keeps its catalogued message.**
  The flag is `Option<String>` rather than `required`, so the refusal is
  `cli.config_set_needs_realtime_model` and not `clap`'s "required argument
  was not provided". Same reasoning for `config <unknown>` and
  `service <unknown>`, which are `external_subcommand` variants rather than
  `clap` "unrecognized subcommand" errors.

- **`--help` is English only.** `clap` generates it, and localizing it would
  mean three parallel `Command` trees. The catalogued `helpText()` contract —
  and the `cli.help` catalog key that carries its rebranded text — describe
  upstream's eight-verb surface, which VIA does not have; the key is left in
  place for whoever needs upstream parity, and this binary does not read it.
  What *is* locked is `render_long_help()` per command, as `insta` snapshots.

- **`--url`'s `[env: VIA_URL]` annotation hides its value.** `clap` prints
  `[env: NAME=value]` by default, which would make the help snapshots depend
  on the developer's own environment. The variable *name* is the contract and
  is shown.

### Behaviour

- **The Gateway body is a refusal, not a no-op.** `via gateway` normalises its
  arguments, applies them to the environment, resolves the configuration, runs
  the setup gate and takes the instance lease — then releases the lease and
  exits non-zero naming phase 5. `via chat`, `via backend`, `via mcp-serve`
  and `via service` do the same for their own phases. Upstream has no
  unimplemented command, so `VIA_CLI_NOT_IMPLEMENTED` is VIA's own code, with
  the `VIA_CLI_` prefix the workspace uses for codes no client inherited.

- **The lease is taken and immediately released.** Phase 5 holds it for the
  process's lifetime and heartbeats it. Taking it now is what makes
  `VIA_GATEWAY_ALREADY_RUNNING` reachable — and releasing it is what stops a
  build with no liveness probe (Windows, `docs/deviations/phase-0.md`) from
  inheriting a lease naming a pid that has already exited.

- **The machine-readable error code reaches a caller through `cli.log`, not
  stderr.** The catalogued stderr shape is `` `via: <message>` `` with no code
  in it, so printing one would break the contract. `cli.failed` carries
  `code`, which is what the integration tests assert on.

- **`--url` sets `HOST` and `PORT`.** Upstream derives them from the URL for
  the *child* Gateway (`env-var/gateway child spawn environment`); VIA has no
  child, so they are applied as `GatewayOptions` and land in the resolved
  configuration. The `localhost` → `127.0.0.1` rewrite and the `|| '80'`
  fallback are reproduced exactly, including the case where a default-port
  `https` origin yields `80` rather than `443` — that is upstream's arithmetic
  and a phase-5 child will be handed the same number.

- **The `config.env` staging file is `via-store`'s name, not
  `config-command.mjs`'s.** Upstream writes `.config.env.<pid>.<uuid>.tmp`;
  the rewrite goes through `via_core::runtime::write_file`, whose staging name
  is `via-store`'s documented `<path>.<pid>.tmp`. The temp file exists for
  microseconds and no external party observes it; every property the
  catalogue *does* state — 0600, 0700, atomic replace, directory fsync — is
  reproduced and tested.

- **Two upstream reader quirks are reproduced rather than corrected.**
  `resolveConfigModel` matches the raw text with `^\s*KEY\s*=\s*(.*?)\s*$`, so
  `KEY="quoted"` reads back *with* its quotes and `export KEY=x` does not match
  at all — different answers from `via-core`'s `parse_env_map`, and the right
  ones here, because the reader has to see the file the way the rewriter does.
  And `||` is not `??`: an empty environment value falls through to the file
  while a whitespace one wins and is then trimmed to nothing. Both are pinned
  by name in `src/configfile.rs`.

- **The locale is resolved twice.** Once from the process environment, which
  is what a failure *before* `config.env` is read is rendered in and which
  chooses the language of the seeded templates; then again from the merged
  environment, so a `VIA_LOCALE` in `config.env` governs every message after
  it. On a first run there is no file to disagree with, so the two answers
  cannot differ there.

- **`Host::runtime_root` duplicates two lines of `via-core`'s resolver.**
  `.env.local` and `.env` are looked for inside the installation root, so the
  root has to be known *before* `load_runtime_environment` runs, and
  `via-core` answers it only after. `tests` assert the two answers agree for
  every shape of `VIA_RUNTIME_ROOT`, which is what makes the duplication safe.

### Workspace

- **One dependency was added to `[workspace.dependencies]`:**
  `clap = { version = "4", features = ["derive", "env"] }`, under a new
  `# ── cli ──` heading. No existing line was touched.

- **Three keys were added to `via-i18n`'s `cli.json`**, because the commands
  this phase implements needed sentences the catalog did not have:
  `cli.config_show_model_item` (the `- <id>（<label>）` line of the catalogued
  `config show` output, whose brackets are fullwidth in `zh`),
  `cli.list_separator` (upstream's `backendNames().join('、')` — the separator
  is Chinese punctuation, so it is catalog data rather than a literal in
  code), and `cli.not_implemented_until_phase` (VIA's own, since upstream has
  no unimplemented command). The first two are punctuation and are recorded in
  `via-i18n`'s own `NOT_CHINESE_PROSE` allow-list with their reasons.

- **`apps/via` carries a library target as well as the binary,** so the
  integration tests can drive both the process (for the exit code, the stream
  and the log record) and the functions (for the cases where spawning would
  only obscure what is being asserted).

- **`apps/via` depends on `via-catalog` and `via-store` as well as the five
  crates the brief named.** `via-catalog` owns the Realtime model table
  `config show` prints and `config set` validates against, and the backend
  table `--backend` resolves against; `via-store` owns the cross-process file
  transaction the `config.env` rewrite runs inside. Both are Leaf-band, which
  the App band may depend on (`via-arch-test`).
