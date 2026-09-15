//! `via-core`'s `env-var` contracts: every environment variable the layered
//! configuration resolver reads, asserted against the real name — rebranded
//! per `docs/rebrand.md` where upstream's is renamed — and, wherever the
//! contract's `exactValue` is prose about precedence or defaulting rather than
//! a bare literal, against the real behaviour [`via_core::config::resolve`]
//! and its neighbours produce.
//!
//! Nothing here retypes a catalogued name or number: every expected value is
//! either parsed out of the contract's `exactValue` with a `value::` helper or
//! is a substring check against the catalogue's own prose, and every actual
//! value is read off `via_core::config::names`, a resolved [`via_core::Config`],
//! or another of `via-core`'s public functions.
//!
//! A handful of contracts bundle names that split across more than one crate —
//! a backend's executable-path variable is `via-core`'s to resolve, but the
//! adapter-runtime variable beside it in the same `.env.example` block is not
//! read anywhere yet. Those rows are [`Partial`](via_conformance::Coverage::Partial):
//! the half `via-core` owns is asserted here, real function calls and all, and
//! the rest is left named for whichever crate ends up reading it.

use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;
use via_catalog::backend_definition;
use via_conformance::expect_contract;
use via_conformance::value::{list, rebranded};
use via_core::config::{BackendSelection, GatewayOptions, Overrides, backend, names, resolve};
use via_core::env::EnvMap;
use via_core::runtime::{RuntimeOptions, load_runtime_environment, user_config_template};
use via_core::{Config, CoreError};
use via_i18n::Locale;

// ── shared test fixtures ────────────────────────────────────────────────────

const HOME: &str = "/home/via";
const CWD: &str = "/srv/via";
const ROOT: &str = "/opt/via";

fn env(pairs: &[(&str, &str)]) -> EnvMap {
    pairs.iter().copied().collect()
}

fn overrides() -> Overrides {
    Overrides {
        home_directory: PathBuf::from(HOME),
        working_directory: PathBuf::from(CWD),
        runtime_root: Some(PathBuf::from(ROOT)),
        gateway: GatewayOptions::default(),
    }
}

fn resolved(pairs: &[(&str, &str)]) -> Config {
    resolve(&env(pairs), None, &overrides()).expect("a well-formed environment resolves")
}

fn try_resolved(pairs: &[(&str, &str)]) -> Result<Config, CoreError> {
    resolve(&env(pairs), None, &overrides())
}

// ── 1. ACP_COMMAND / ACP_ARGS / ACP_LABEL ───────────────────────────────────

#[test]
fn acp_command_args_and_label_match_their_contract() {
    let contract = expect_contract("env-var", "ACP_COMMAND / ACP_ARGS / ACP_LABEL");
    // KEEP: all three name the generic ACP entry point, not VIA.
    for name in [names::ACP_COMMAND, names::ACP_ARGS, names::ACP_LABEL] {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` is not named in the catalogued contract ({})",
            contract.file
        );
    }
    assert_eq!(names::ACP_COMMAND, "ACP_COMMAND");
    assert_eq!(names::ACP_ARGS, "ACP_ARGS");
    assert_eq!(names::ACP_LABEL, "ACP_LABEL");
    assert!(
        contract
            .exact_value
            .contains("ACP_LABEL: default 'ACP Agent'")
    );

    // The three parsing shapes: JSON array, whitespace fallback, and the two
    // refusals — real `via_core::config::backend::resolve_acp_args`.
    assert_eq!(
        backend::resolve_acp_args(None).expect("unset is empty"),
        Vec::<String>::new()
    );
    assert_eq!(
        backend::resolve_acp_args(Some(r#"["--acp", "a b"]"#)).expect("JSON array form"),
        vec!["--acp".to_owned(), "a b".to_owned()]
    );
    assert_eq!(
        backend::resolve_acp_args(Some("--acp   --verbose")).expect("whitespace form"),
        vec!["--acp".to_owned(), "--verbose".to_owned()]
    );
    assert_eq!(
        backend::resolve_acp_args(Some("[nope")).unwrap_err().code(),
        "VIA_CONFIG_ACP_ARGS_NOT_JSON"
    );
    assert_eq!(
        backend::resolve_acp_args(Some("[1, 2]"))
            .unwrap_err()
            .code(),
        "VIA_CONFIG_ACP_ARGS_NOT_STRING_ARRAY"
    );

    // The default label, both unset and set-empty, end to end through
    // `resolve`.
    assert_eq!(resolved(&[]).backends.acp.label, "ACP Agent");
    assert_eq!(
        resolved(&[(names::ACP_LABEL, "   ")]).backends.acp.label,
        "ACP Agent"
    );
    assert_eq!(
        resolved(&[(names::ACP_LABEL, "My Agent")])
            .backends
            .acp
            .label,
        "My Agent"
    );
}

// ── 2. AGENT_TIMEOUT_MS ─────────────────────────────────────────────────────

#[test]
fn agent_timeout_ms_matches_its_contract() {
    let contract = expect_contract("env-var", "AGENT_TIMEOUT_MS");
    assert!(contract.exact_value.contains("300000"));
    assert!(contract.exact_value.contains("10000"));
    // KEEP — a generic convention name, not qwen's.
    assert_eq!(names::AGENT_TIMEOUT_MS, "AGENT_TIMEOUT_MS");

    assert_eq!(resolved(&[]).agent_timeout_ms, 300_000);
    assert_eq!(
        resolved(&[(names::AGENT_TIMEOUT_MS, "1")]).agent_timeout_ms,
        10_000,
        "clamped up to the catalogued minimum, not rejected"
    );
    assert_eq!(
        resolved(&[(names::AGENT_TIMEOUT_MS, "500000")]).agent_timeout_ms,
        500_000
    );
}

// ── 3. CLI env inputs (via-core's share; the rest is `apps/via`'s) ─────────

#[test]
fn cli_env_inputs_partial() {
    let contract = expect_contract("env-var", "CLI env inputs");
    let upstream_names = list(&contract.exact_value);
    assert_eq!(upstream_names.len(), 13, "{}", contract.file);

    // The nine (of thirteen) that `via-core::config::resolve` itself reads,
    // after `docs/rebrand.md`'s rename rule.
    let core_owned: &[(&str, &str)] = &[
        (
            "QWEN_AUDIO_AGENT_BACKEND_PERMISSION_MODE",
            names::BACKEND_PERMISSION_MODE,
        ),
        ("QWEN_AUDIO_AGENT_BACKEND_AGENT", names::BACKEND_AGENT),
        (
            "QWEN_AUDIO_AGENT_BACKEND_OWNERSHIP",
            names::BACKEND_OWNERSHIP,
        ),
        ("AGENT_PROTOCOL", names::AGENT_PROTOCOL),
        ("QWEN_AUDIO_REALTIME_MODEL", names::REALTIME_MODEL),
        ("QWAUDIO_CONFIG_DIR", names::CONFIG_DIR),
        ("QWAUDIO_DATA_DIR", names::DATA_DIR),
        ("HOST", names::HOST),
        ("PORT", names::PORT),
    ];
    for (upstream, shipped) in core_owned {
        assert!(
            upstream_names.contains(upstream),
            "`{upstream}` is missing from the catalogued list ({})",
            contract.file
        );
        assert_eq!(&rebranded(upstream), shipped);
    }

    // `XDG_CONFIG_HOME` is not in `core_owned` above (it needs no rename) but
    // is real and is `via-core`'s.
    assert!(upstream_names.contains(&"XDG_CONFIG_HOME"));
    assert_eq!(via_core::paths::ENV_XDG_CONFIG_HOME, "XDG_CONFIG_HOME");

    // The remaining three name the CLI process itself — `apps/via`'s.
    for remaining in [
        "QWEN_AUDIO_AGENT_URL",
        "QWEN_AUDIO_AGENT_SESSION_ID",
        "QWEN_AUDIO_AGENT_TUI_AUDIO_MODE",
    ] {
        assert!(upstream_names.contains(&remaining), "{}", contract.file);
    }
}

// ── 4. Claude Code block ────────────────────────────────────────────────────

#[test]
fn claude_code_block_partial() {
    let contract = expect_contract("env-var", "Claude Code block");
    // `via-core`'s share: the executable overrides it actually reads.
    for name in [names::CLAUDE_CODE_ACP_BIN, names::CLAUDE_CODE_EXECUTABLE] {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` is not in the catalogued block ({})",
            contract.file
        );
    }
    let claude = backend_definition("claude").expect("claude is in the catalog");
    assert_eq!(
        claude.setup.adapter_environment,
        Some(names::CLAUDE_CODE_ACP_BIN)
    );
    assert_eq!(
        claude.setup.executable_environment,
        &[names::CLAUDE_CODE_EXECUTABLE]
    );

    // The config-directory override, also `via-core`'s, resolved absolutely.
    let with_dir = resolved(&[(names::CLAUDE_CONFIG_DIR, "claude-home")]);
    assert_eq!(
        with_dir.backends.claude.config_directory,
        Path::new(CWD).join("claude-home")
    );
    assert_eq!(
        resolved(&[]).backends.claude.config_directory,
        PathBuf::new()
    );

    // Every var reads back through `resolve` end to end.
    let config = resolved(&[
        ("AGENT_PROTOCOL", "claude"),
        (names::CLAUDE_CODE_ACP_BIN, "/opt/claude-code-acp"),
        (names::CLAUDE_CODE_EXECUTABLE, "/opt/claude"),
    ]);
    assert_eq!(config.backends.claude.cli_path, "/opt/claude-code-acp");
    assert_eq!(config.backends.claude.claude_executable, "/opt/claude");

    // `CLAUDE_CODE_ACP_RUNTIME` and `CLAUDE_WORKSPACE` are catalogued
    // (`via-catalog`'s `adapter_runtime_environment` / `workspace_environment`)
    // but nothing yet reads the runtime override — that is `via-acp`'s.
    for remaining in ["CLAUDE_CODE_ACP_RUNTIME", "CLAUDE_WORKSPACE"] {
        assert!(
            contract.exact_value.contains(remaining),
            "{}",
            contract.file
        );
    }
    assert_eq!(
        claude.setup.adapter_runtime_environment,
        Some("CLAUDE_CODE_ACP_RUNTIME")
    );
    assert_eq!(claude.workspace_environment, "CLAUDE_WORKSPACE");
}

// ── 5. CodeBuddy block ──────────────────────────────────────────────────────

#[test]
fn codebuddy_block_matches_its_contract() {
    let contract = expect_contract("env-var", "CodeBuddy block");
    for name in [names::CODEBUDDY_BIN, names::CODEBUDDY_MODEL_URL] {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` is not in the catalogued block ({})",
            contract.file
        );
    }
    let codebuddy = backend_definition("codebuddy").expect("codebuddy is in the catalog");
    assert_eq!(codebuddy.workspace_environment, "CODEBUDDY_WORKSPACE");

    // The derived chat-completions URL, byte for byte against the catalogued
    // default — the memory extractor's base URL plus the chat-completions
    // suffix, named once rather than retyped twice.
    let derived = format!(
        "{}{}",
        backend::DASHSCOPE_COMPATIBLE_BASE_URL,
        backend::CHAT_COMPLETIONS_SUFFIX
    );
    assert!(
        contract
            .exact_value
            .contains(&format!("CODEBUDDY_MODEL_URL={derived}")),
        "the catalogued default does not match the derived URL: {derived} ({})",
        contract.file
    );
    let via_resolve = resolved(&[
        ("AGENT_PROTOCOL", "codebuddy"),
        (names::BACKEND_MODEL, "qwen3.7-max"),
    ]);
    assert_eq!(via_resolve.backends.codebuddy.model_url, derived);

    // End to end: the executable override and the workspace override both
    // resolve through `via-core`.
    let config = resolved(&[
        ("AGENT_PROTOCOL", "codebuddy"),
        (names::CODEBUDDY_BIN, "/opt/codebuddy"),
        ("CODEBUDDY_WORKSPACE", "cb-workspace"),
    ]);
    assert_eq!(config.backends.codebuddy.cli_path, "/opt/codebuddy");
    assert_eq!(
        config.backends.codebuddy.directory,
        Path::new(ROOT).join("cb-workspace")
    );
}

// ── 6. Codex block ───────────────────────────────────────────────────────────

#[test]
fn codex_block_partial() {
    let contract = expect_contract("env-var", "Codex block");
    for name in [names::CODEX_ACP_BIN, names::CODEX_BASE_URL] {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` is not in the catalogued block ({})",
            contract.file
        );
    }
    // The bare `/v1` base, unlike CodeBuddy's `/chat/completions` suffix.
    assert!(
        contract.exact_value.contains(&format!(
            "CODEX_BASE_URL={}",
            backend::DASHSCOPE_COMPATIBLE_BASE_URL
        )),
        "{}",
        contract.file
    );
    let config = resolved(&[
        ("AGENT_PROTOCOL", "codex"),
        (names::BACKEND_MODEL, "qwen3.7-max"),
        (names::CODEX_ACP_BIN, "/opt/codex-acp"),
    ]);
    assert_eq!(
        config.backends.codex.model_url,
        backend::DASHSCOPE_COMPATIBLE_BASE_URL
    );
    assert_eq!(config.backends.codex.cli_path, "/opt/codex-acp");

    // `CODEX_ACP_RUNTIME` (adapter runtime selection) and `CODEX_PATH` (the
    // bare `codex` executable, used by the adapter rather than by `via-core`)
    // are catalogued in `via-catalog` but read by neither `via-core` nor any
    // shipped crate yet.
    let codex = backend_definition("codex").expect("codex is in the catalog");
    assert_eq!(
        codex.setup.adapter_runtime_environment,
        Some("CODEX_ACP_RUNTIME")
    );
    assert_eq!(codex.setup.executable_environment, &["CODEX_PATH"]);
    for remaining in ["CODEX_ACP_RUNTIME", "CODEX_PATH", "CODEX_WORKSPACE"] {
        assert!(
            contract.exact_value.contains(remaining),
            "{}",
            contract.file
        );
    }
}

// ── 7. DASHSCOPE_API_KEY (`.env.example` line) ──────────────────────────────

#[test]
fn dashscope_api_key_line_matches_its_contract() {
    let contract = expect_contract("env-var", "DASHSCOPE_API_KEY");
    assert!(contract.exact_value.starts_with("DASHSCOPE_API_KEY="));
    assert_eq!(names::DASHSCOPE_API_KEY, "DASHSCOPE_API_KEY");

    // VIA's own generated template (a different document, deliberately, per
    // `runtime.rs`) still ships the same line, active and empty.
    let template = user_config_template(Locale::En);
    assert!(
        template.lines().any(|line| line == "DASHSCOPE_API_KEY="),
        "the shipped config.env template must still carry an active, empty \
         DASHSCOPE_API_KEY line"
    );

    // The credential fallback it feeds, end to end.
    let config = resolved(&[(names::DASHSCOPE_API_KEY, "sk-example")]);
    assert_eq!(config.realtime.dashscope_api_key.expose(), "sk-example");
    assert_eq!(config.memory_api_key.expose(), "sk-example");
}

// ── 8. DASHSCOPE_API_KEY / DASHSCOPE_WORKSPACE_ID ───────────────────────────

#[test]
fn dashscope_api_key_and_workspace_id_match_their_contract() {
    let contract = expect_contract("env-var", "DASHSCOPE_API_KEY / DASHSCOPE_WORKSPACE_ID");
    // KEEP — Alibaba Cloud's own identifiers.
    assert_eq!(names::DASHSCOPE_API_KEY, "DASHSCOPE_API_KEY");
    assert_eq!(names::DASHSCOPE_WORKSPACE_ID, "DASHSCOPE_WORKSPACE_ID");
    assert!(
        contract
            .exact_value
            .contains("cn-beijing.maas.aliyuncs.com")
    );

    // The realtime credential fallback: `VIA_REALTIME_API_KEY || DASHSCOPE_API_KEY`.
    let realtime_fallback = resolved(&[(names::DASHSCOPE_API_KEY, "ds-key")]);
    assert_eq!(
        realtime_fallback.realtime.dashscope_api_key.expose(),
        "ds-key"
    );
    let explicit_wins = resolved(&[
        (names::DASHSCOPE_API_KEY, "ds-key"),
        (names::REALTIME_API_KEY, "realtime-key"),
    ]);
    assert_eq!(
        explicit_wins.realtime.dashscope_api_key.expose(),
        "realtime-key"
    );

    // The memory-extractor key fallback.
    let memory_fallback = resolved(&[(names::DASHSCOPE_API_KEY, "ds-key")]);
    assert_eq!(memory_fallback.memory_api_key.expose(), "ds-key");

    // One of the managed-OpenClaw-Bailian preconditions.
    let managed = resolved(&[
        ("AGENT_PROTOCOL", "openclaw"),
        (names::BACKEND_MODEL, "qwen3.7-plus"),
        (names::DASHSCOPE_API_KEY, "ds-key"),
    ]);
    assert_eq!(managed.backends.openclaw.coordinator_agent, "via-backend");
    let not_managed_without_key = resolved(&[
        ("AGENT_PROTOCOL", "openclaw"),
        (names::BACKEND_MODEL, "qwen3.7-plus"),
    ]);
    assert_eq!(
        not_managed_without_key.backends.openclaw.coordinator_agent,
        ""
    );

    // The workspace-id endpoint rewrite, both the WebSocket and the
    // OpenAI-compatible HTTP endpoint.
    assert_eq!(
        via_catalog::realtime_provider::dashscope_workspace_realtime_url("ws-42"),
        "wss://ws-42.cn-beijing.maas.aliyuncs.com/api-ws/v1/realtime"
    );
    assert_eq!(
        backend::dashscope_compatible_base_url(Some("ws-42")),
        "https://ws-42.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"
    );
    let rewritten = resolved(&[(names::DASHSCOPE_WORKSPACE_ID, "ws-42")]);
    assert_eq!(
        rewritten.realtime.dashscope_realtime_url,
        "wss://ws-42.cn-beijing.maas.aliyuncs.com/api-ws/v1/realtime"
    );
}

// ── 9. HERMES_BIN / HERMES_WORKSPACE ────────────────────────────────────────

#[test]
fn hermes_bin_and_workspace_match_their_contract() {
    let contract = expect_contract("env-var", "HERMES_BIN / HERMES_WORKSPACE");
    assert!(contract.exact_value.contains(names::HERMES_BIN));
    assert_eq!(names::HERMES_BIN, "HERMES_BIN");
    let hermes = backend_definition("hermes").expect("hermes is in the catalog");
    assert_eq!(hermes.workspace_environment, "HERMES_WORKSPACE");

    let config = resolved(&[
        ("AGENT_PROTOCOL", "hermes"),
        (names::HERMES_BIN, "/opt/hermes"),
        ("HERMES_WORKSPACE", "hermes-ws"),
    ]);
    assert_eq!(config.backends.hermes.cli_path, "/opt/hermes");
    assert_eq!(
        config.backends.hermes.directory,
        Path::new(ROOT).join("hermes-ws")
    );
    // Unset falls back to the one shared workspace.
    let shared = resolved(&[("AGENT_PROTOCOL", "hermes")]);
    assert_eq!(
        shared.backends.hermes.directory,
        shared.data_directory().join("workspace")
    );
}

// ── 10. Kimi Code block ─────────────────────────────────────────────────────

#[test]
fn kimi_code_block_partial() {
    let contract = expect_contract("env-var", "Kimi Code block");
    assert!(contract.exact_value.contains(names::KIMI_CODE_BIN));
    assert_eq!(names::KIMI_CODE_BIN, "KIMI_CODE_BIN");
    let kimi = backend_definition("kimi").expect("kimi is in the catalog");
    assert_eq!(kimi.workspace_environment, "KIMI_WORKSPACE");
    assert!(kimi.environment.prefixes.contains(&"KIMI_"));

    let config = resolved(&[
        ("AGENT_PROTOCOL", "kimi"),
        (names::KIMI_CODE_BIN, "/opt/kimi"),
        ("KIMI_WORKSPACE", "kimi-ws"),
    ]);
    assert_eq!(config.backends.kimi.cli_path, "/opt/kimi");
    assert_eq!(
        config.backends.kimi.directory,
        Path::new(ROOT).join("kimi-ws")
    );

    // `KIMI_MODEL_NAME` / `_API_KEY` / `_BASE_URL` are the vendor's own
    // "official temporary model variables"; they pass through to the child
    // process under the `KIMI_` prefix policy above, but nothing in `via-core`
    // reads them, and forwarding them is `via-process`'s job.
    for remaining in [
        "KIMI_MODEL_NAME",
        "KIMI_MODEL_API_KEY",
        "KIMI_MODEL_BASE_URL",
    ] {
        assert!(
            contract.exact_value.contains(remaining),
            "{}",
            contract.file
        );
    }
}

// ── 11. OPENCLAW_BASE_URL ───────────────────────────────────────────────────

#[test]
fn openclaw_base_url_partial() {
    let contract = expect_contract("env-var", "OPENCLAW_BASE_URL");
    assert!(contract.exact_value.contains("http://127.0.0.1:18789"));
    assert_eq!(names::OPENCLAW_BASE_URL, "OPENCLAW_BASE_URL");

    assert_eq!(
        resolved(&[("AGENT_PROTOCOL", "openclaw")])
            .backends
            .openclaw
            .base_url,
        "http://127.0.0.1:18789"
    );
    // Trailing slashes stripped.
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::OPENCLAW_BASE_URL, "wss://openclaw.example.com///"),
        ])
        .backends
        .openclaw
        .base_url,
        "wss://openclaw.example.com"
    );
    // An explicit base URL flips ownership to external.
    assert_eq!(
        resolved(&[("AGENT_PROTOCOL", "openclaw")])
            .backend_ownership
            .as_str(),
        "owned"
    );
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::OPENCLAW_BASE_URL, "wss://openclaw.example.com"),
        ])
        .backend_ownership
        .as_str(),
        "external"
    );

    let openclaw = backend_definition("openclaw").expect("openclaw is in the catalog");
    assert!(openclaw.supports_external_service);

    // The protocol allow-list (`http:`/`https:`/`ws:`/`wss:`) and the
    // `OPENCLAW_PORT` reallocation belong to the driver that spawns and
    // supervises the process — `via-process`.
    assert!(contract.exact_value.contains("ws:"));
    assert!(contract.exact_value.contains("wss:"));
}

// ── 12/13. OPENCLAW_GATEWAY_TOKEN / OPENCLAW_GATEWAY_TOKEN_FILE ─────────────

#[test]
fn openclaw_gateway_token_fallback_chain_matches_its_contracts() {
    let behavioural = expect_contract(
        "env-var",
        "OPENCLAW_GATEWAY_TOKEN / OPENCLAW_GATEWAY_TOKEN_FILE / AGENT_API_KEY",
    );
    assert!(behavioural.exact_value.contains("gateway-token"));
    let env_example = expect_contract(
        "env-var",
        "OPENCLAW_GATEWAY_TOKEN / OPENCLAW_GATEWAY_TOKEN_FILE / OPENCLAW_ACP_BIN",
    );
    assert!(env_example.exact_value.contains(names::OPENCLAW_ACP_BIN));

    for name in [
        names::OPENCLAW_GATEWAY_TOKEN,
        names::OPENCLAW_GATEWAY_TOKEN_FILE,
        names::AGENT_API_KEY,
        names::OPENCLAW_ACP_BIN,
    ] {
        assert!(
            behavioural.exact_value.contains(name) || env_example.exact_value.contains(name),
            "`{name}` is not named in either catalogued contract"
        );
    }

    // `token = OPENCLAW_GATEWAY_TOKEN || AGENT_API_KEY || ''`.
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::AGENT_API_KEY, "fallback")
        ])
        .backends
        .openclaw
        .token
        .expose(),
        "fallback"
    );
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::OPENCLAW_GATEWAY_TOKEN, "explicit"),
            (names::AGENT_API_KEY, "fallback"),
        ])
        .backends
        .openclaw
        .token
        .expose(),
        "explicit"
    );
    assert_eq!(
        resolved(&[("AGENT_PROTOCOL", "openclaw")])
            .backends
            .openclaw
            .token
            .expose(),
        ""
    );

    // `tokenFile` default = `<configDirectory>/backends/openclaw/state/gateway-token`.
    let default_path = resolved(&[("AGENT_PROTOCOL", "openclaw")])
        .backends
        .openclaw
        .token_file;
    assert!(default_path.ends_with("backends/openclaw/state/gateway-token"));
    let overridden = resolved(&[
        ("AGENT_PROTOCOL", "openclaw"),
        (names::OPENCLAW_GATEWAY_TOKEN_FILE, "custom-token-file"),
    ])
    .backends
    .openclaw
    .token_file;
    assert_eq!(overridden, Path::new(ROOT).join("custom-token-file"));

    // The CLI override.
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::OPENCLAW_ACP_BIN, "/opt/openclaw"),
        ])
        .backends
        .openclaw
        .cli_path,
        "/opt/openclaw"
    );

    // Generating a fresh 32-byte token when the Gateway manages OpenClaw with
    // Bailian and none is set is `via-process`'s job, at spawn time — not
    // `via-core`'s, which only resolves the fallback chain above.
    assert!(behavioural.exact_value.contains("randomBytes(32)"));
}

// ── 14/15. OPENCODE_BASE_URL ────────────────────────────────────────────────

#[test]
fn opencode_base_url_matches_its_contracts() {
    let behavioural = expect_contract("env-var", "OPENCODE_BASE_URL");
    assert!(behavioural.exact_value.contains("http://127.0.0.1:4096"));
    let env_example = expect_contract("env-var", "OPENCODE_BASE_URL / OPENCLAW_BASE_URL");
    assert!(env_example.exact_value.contains(names::OPENCODE_BASE_URL));
    assert!(env_example.exact_value.contains(names::OPENCLAW_BASE_URL));

    assert_eq!(
        resolved(&[("AGENT_PROTOCOL", "opencode")])
            .backends
            .opencode
            .base_url,
        "http://127.0.0.1:4096"
    );
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "opencode"),
            (names::OPENCODE_BASE_URL, "http://127.0.0.1:9999///"),
        ])
        .backends
        .opencode
        .base_url,
        "http://127.0.0.1:9999"
    );

    // `config.mjs` keeps the full URL while the driver reduces it to origin —
    // the driver's `normalizeServiceEndpoint` and the `OPENCODE_PORT`
    // reallocation on conflict are `via-backends`'s (the process-driver crate),
    // not `via-core`'s.
    assert!(behavioural.exact_value.contains("normalizeServiceEndpoint"));
}

// ── 16. Pi block ─────────────────────────────────────────────────────────────

#[test]
fn pi_block_partial() {
    let contract = expect_contract("env-var", "Pi block");
    assert!(contract.exact_value.contains(names::PI_ACP_BIN));
    assert_eq!(names::PI_ACP_BIN, "PI_ACP_BIN");
    let pi = backend_definition("pi").expect("pi is in the catalog");
    assert_eq!(pi.workspace_environment, "PI_WORKSPACE");
    assert!(pi.always_full_permission);

    let config = resolved(&[
        ("AGENT_PROTOCOL", "pi"),
        (names::PI_ACP_BIN, "/opt/pi-acp"),
        ("PI_WORKSPACE", "pi-ws"),
    ]);
    assert_eq!(config.backends.pi.cli_path, "/opt/pi-acp");
    assert_eq!(config.backends.pi.directory, Path::new(ROOT).join("pi-ws"));
    // Security-relevant: Pi is always full permission regardless of what is
    // configured — already asserted end to end at the `via-catalog` layer by
    // `tests/backend_catalog.rs::pi_is_always_full_permission`; reproduced
    // here through `via-core`'s own resolution.
    assert_eq!(config.backend_permission_mode, "full");

    // `PI_BIN` (the bare `pi` executable, used by the adapter) and
    // `PI_ACP_RUNTIME` (adapter runtime selection) are catalogued in
    // `via-catalog` but read by neither `via-core` nor any shipped crate yet.
    assert_eq!(pi.setup.executable_environment, &["PI_BIN"]);
    assert_eq!(pi.setup.adapter_runtime_environment, Some("PI_ACP_RUNTIME"));
    for remaining in ["PI_BIN", "PI_ACP_RUNTIME"] {
        assert!(
            contract.exact_value.contains(remaining),
            "{}",
            contract.file
        );
    }
}

// ── 17. QODERCLI_PATH / QODER_AUTH_MODE ─────────────────────────────────────

#[test]
fn qoder_cli_path_matches_its_contract_and_qoder_auth_mode_is_dead() {
    let contract = expect_contract("env-var", "QODERCLI_PATH / QODER_AUTH_MODE");
    assert!(contract.exact_value.contains("QODERCLI_PATH"));
    assert!(contract.exact_value.contains("QODER_AUTH_MODE"));
    assert!(contract.why.contains("DEAD"));

    // The real, shipped name — and its legacy alias, both KEEP.
    assert_eq!(names::QODERCLI_PATH, "QODERCLI_PATH");
    assert_eq!(names::QODER_CLI_PATH, "QODER_CLI_PATH");
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "qoder"),
            (names::QODERCLI_PATH, "/opt/qodercli"),
        ])
        .backends
        .qoder
        .cli_path,
        "/opt/qodercli"
    );
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "qoder"),
            (names::QODER_CLI_PATH, "/opt/qodercli-legacy"),
        ])
        .backends
        .qoder
        .cli_path,
        "/opt/qodercli-legacy"
    );

    // `QODER_AUTH_MODE` upstream deviation: `docs/rebrand.md` documents it as
    // dead code — never read anywhere in the upstream tree — and instructs
    // that the port must not reproduce it. `QoderOptions` carries no field for
    // it; the struct's four fields are the whole of what `via-core` resolves
    // for this backend.
    let qoder = resolved(&[("AGENT_PROTOCOL", "qoder")]).backends.qoder;
    assert_eq!(
        qoder,
        via_core::config::backend::QoderOptions {
            model: String::new(),
            directory: qoder.directory.clone(),
            cli_path: String::new(),
            config_directory: PathBuf::new(),
        },
        "QoderOptions has exactly four fields, none of them QODER_AUTH_MODE"
    );
}

// ── 18. QWEN_AUDIO_AGENT_AUTH_SECRET ────────────────────────────────────────

#[test]
fn auth_secret_env_var_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_AGENT_AUTH_SECRET");
    assert!(
        contract
            .exact_value
            .contains("QWEN_AUDIO_AGENT_AUTH_SECRET")
    );
    assert!(contract.exact_value.contains("at least 32 characters"));
    assert_eq!(names::AUTH_SECRET, "VIA_AUTH_SECRET");
    assert_eq!(
        rebranded("QWEN_AUDIO_AGENT_AUTH_SECRET"),
        names::AUTH_SECRET
    );

    assert_eq!(
        via_core::identity::AUTH_SECRET_MIN_LENGTH,
        32,
        "{}",
        contract.file
    );
    let error = via_core::IdentityManager::new("short", via_core::IdentityMode::Personal, "user_x")
        .expect_err("31 characters is below the minimum");
    assert_eq!(
        error.to_string(),
        via_core::identity::AUTH_SECRET_LENGTH_MESSAGE
    );
    assert!(contract.why.contains("reaches the operator"));
    assert_eq!(
        via_core::identity::AUTH_SECRET_LENGTH_MESSAGE,
        "VIA_AUTH_SECRET must contain at least 32 characters"
    );

    // The 64-hex-character generated secret, and that the resolver reads it
    // straight through with no default of its own.
    assert_eq!(resolved(&[]).auth_secret.expose(), "");
    assert_eq!(
        resolved(&[(names::AUTH_SECRET, "0123456789abcdef")])
            .auth_secret
            .expose(),
        "0123456789abcdef"
    );
}

// ── 19. QWEN_AUDIO_AGENT_BACKEND_AGENT (`.env.example` line) ────────────────

#[test]
fn backend_agent_env_example_line_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_AGENT_BACKEND_AGENT");
    assert!(
        contract
            .exact_value
            .contains("QWEN_AUDIO_AGENT_BACKEND_AGENT=")
    );
    assert_eq!(names::BACKEND_AGENT, "VIA_BACKEND_AGENT");
    assert_eq!(
        rebranded("QWEN_AUDIO_AGENT_BACKEND_AGENT"),
        names::BACKEND_AGENT
    );
    assert_eq!(
        resolved(&[(names::BACKEND_AGENT, "custom-agent")])
            .backends
            .opencode
            .coordinator_agent,
        "custom-agent"
    );
}

// ── 20. Coordinator agent precedence (4 vars) ───────────────────────────────

#[test]
fn coordinator_agent_precedence_matches_its_contract() {
    let contract = expect_contract(
        "env-var",
        "QWEN_AUDIO_AGENT_BACKEND_AGENT / OPENCODE_COORDINATOR_AGENT / \
         OPENCLAW_COORDINATOR_AGENT / ACP_COORDINATOR_AGENT",
    );
    assert!(contract.exact_value.contains("voice-coordinator"));
    assert_eq!(names::BACKEND_AGENT, "VIA_BACKEND_AGENT");
    assert_eq!(
        names::OPENCODE_COORDINATOR_AGENT,
        "OPENCODE_COORDINATOR_AGENT"
    );
    assert_eq!(
        names::OPENCLAW_COORDINATOR_AGENT,
        "OPENCLAW_COORDINATOR_AGENT"
    );
    assert_eq!(names::ACP_COORDINATOR_AGENT, "ACP_COORDINATOR_AGENT");

    // OpenCode: the two reserved VIA ids blank to "use the backend's default";
    // `VIA_BACKEND_AGENT` wins over `OPENCODE_COORDINATOR_AGENT`.
    for reserved in [
        backend::VIA_BACKEND_AGENT_ID,
        backend::VIA_COORDINATOR_AGENT_ID,
    ] {
        assert_eq!(
            backend::resolve_opencode_coordinator_agent(&env(&[(names::BACKEND_AGENT, reserved)])),
            ""
        );
    }
    assert_eq!(
        backend::resolve_opencode_coordinator_agent(&env(&[(
            names::OPENCODE_COORDINATOR_AGENT,
            "custom"
        )])),
        "custom"
    );
    assert_eq!(
        backend::resolve_opencode_coordinator_agent(&env(&[
            (names::BACKEND_AGENT, "wins"),
            (names::OPENCODE_COORDINATOR_AGENT, "loses"),
        ])),
        "wins"
    );

    // OpenClaw: the pre-1.11 legacy id maps forward.
    assert_eq!(
        backend::legacy_backend_agent(
            Some("voice-coordinator"),
            backend::LEGACY_OPENCLAW_COORDINATOR_AGENT
        ),
        "via-backend"
    );
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::OPENCLAW_COORDINATOR_AGENT, "voice-coordinator"),
        ])
        .backends
        .openclaw
        .coordinator_agent,
        "via-backend"
    );
    // And unset, when managed-Bailian conditions hold, defaults the same way.
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::BACKEND_MODEL, "qwen3.7-plus"),
            (names::DASHSCOPE_API_KEY, "k"),
        ])
        .backends
        .openclaw
        .coordinator_agent,
        "via-backend"
    );

    // The generic ACP entry point has its own, unshared coordinator variable.
    assert_eq!(
        resolved(&[
            ("AGENT_PROTOCOL", "acp"),
            (names::ACP_COMMAND, "my-agent"),
            (names::ACP_COORDINATOR_AGENT, "acp-coordinator"),
        ])
        .backends
        .acp
        .coordinator_agent,
        "acp-coordinator"
    );
}

// ── 21. QWEN_AUDIO_AGENT_BACKEND_MODEL ──────────────────────────────────────

#[test]
fn backend_model_derivation_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_AGENT_BACKEND_MODEL");
    assert!(contract.exact_value.contains("alibaba-cn/"));
    assert!(contract.exact_value.contains("bailian/"));
    assert_eq!(names::BACKEND_MODEL, "VIA_BACKEND_MODEL");
    assert_eq!(
        rebranded("QWEN_AUDIO_AGENT_BACKEND_MODEL"),
        names::BACKEND_MODEL
    );

    // `auto`, case-insensitively, means "no override".
    for value in ["auto", "AUTO", "Auto"] {
        assert_eq!(
            backend::resolve_backend_models(&env(&[(names::BACKEND_MODEL, value)])),
            backend::BackendModels::default()
        );
    }

    let models = backend::resolve_backend_models(&env(&[(names::BACKEND_MODEL, "qwen3.7-plus")]));
    assert_eq!(models.common, "qwen3.7-plus");
    assert_eq!(models.open_code, "alibaba-cn/qwen3.7-plus");
    assert_eq!(models.open_claw, "bailian/qwen3.7-plus");
    assert_eq!(models.qoder, "qwen3.7-plus");
    assert_eq!(models.qwen, "qwen3.7-plus");
    assert_eq!(models.kimi, "qwen3.7-plus");
    assert_eq!(models.hermes, "qwen3.7-plus");
    assert_eq!(models.code_buddy, "qwen3.7-plus");
    assert_eq!(models.codex, "qwen3.7-plus");
    assert_eq!(models.claude, "qwen3.7-plus");
    assert_eq!(models.pi, "qwen3.7-plus");
    assert_eq!(models.acp, "qwen3.7-plus");
    assert_eq!(models.deep_seek_harness, "");

    // Three derived contracts named in the catalogue: managed-OpenClaw-Bailian
    // gating, the OpenClaw model id (the substring after the first `/`), and
    // the CodeBuddy model id (the same substring).
    assert_eq!(
        backend::backend_model_name("bailian/qwen3.7-plus"),
        "qwen3.7-plus"
    );
    let managed = resolved(&[
        ("AGENT_PROTOCOL", "openclaw"),
        (names::BACKEND_MODEL, "qwen3.7-plus"),
        (names::DASHSCOPE_API_KEY, "k"),
    ]);
    assert_eq!(managed.backends.openclaw.coordinator_agent, "via-backend");
    let codebuddy = resolved(&[
        ("AGENT_PROTOCOL", "codebuddy"),
        (names::BACKEND_MODEL, "qwen3.7-plus"),
    ]);
    assert_eq!(codebuddy.backends.codebuddy.model, "qwen3.7-plus");

    // DeepSeek: its own variable wins, then a `deepseek-`-prefixed name.
    let explicit = backend::resolve_backend_models(&env(&[
        ("DEEPSEEK_HARNESS_MODEL", "deepseek-v4-flash"),
        (names::BACKEND_MODEL, "qwen3.7-max"),
    ]));
    assert_eq!(explicit.deep_seek_harness, "deepseek-v4-flash");
    let inferred =
        backend::resolve_backend_models(&env(&[(names::BACKEND_MODEL, "deepseek-v4-pro")]));
    assert_eq!(inferred.deep_seek_harness, "deepseek-v4-pro");
}

// ── 22/23/39. Path overrides ─────────────────────────────────────────────────

#[test]
fn path_overrides_match_their_contract() {
    for (kind, name) in [
        ("env-var", "QWEN_AUDIO_AGENT_BACKEND_SESSION_STATE_PATH"),
        (
            "env-var",
            "QWEN_AUDIO_AGENT_FRONTEND_PROMPT_DIR / QWEN_AUDIO_AGENT_ASSISTANT_PROFILE_PATH",
        ),
        ("env-var", "path overrides"),
    ] {
        let contract = expect_contract(kind, name);
        assert!(!contract.exact_value.is_empty());
    }

    // Every override, and that it anchors to the runtime root, not the cwd.
    for (upstream, shipped) in [
        (
            "QWEN_AUDIO_AGENT_FRONTEND_PROMPT_DIR",
            names::FRONTEND_PROMPT_DIR,
        ),
        (
            "QWEN_AUDIO_AGENT_ASSISTANT_PROFILE_PATH",
            names::ASSISTANT_PROFILE_PATH,
        ),
        ("QWEN_AUDIO_AGENT_MEMORY_PATH", names::MEMORY_PATH),
        (
            "QWEN_AUDIO_AGENT_FRONTEND_MEMORY_PATH",
            names::FRONTEND_MEMORY_PATH,
        ),
        (
            "QWEN_AUDIO_AGENT_FRONTEND_NOTES_PATH",
            names::FRONTEND_NOTES_PATH,
        ),
        ("QWEN_AUDIO_AGENT_USER_MODEL_PATH", names::USER_MODEL_PATH),
        (
            "QWEN_AUDIO_AGENT_USER_PROFILE_PATH",
            names::USER_PROFILE_PATH,
        ),
        ("QWEN_AUDIO_AGENT_TASK_STATE_PATH", names::TASK_STATE_PATH),
        (
            "QWEN_AUDIO_AGENT_BACKEND_SESSION_STATE_PATH",
            names::BACKEND_SESSION_STATE_PATH,
        ),
    ] {
        assert_eq!(rebranded(upstream), shipped, "{upstream}");
    }

    assert_eq!(
        resolved(&[(names::TASK_STATE_PATH, "state/tasks.json")]).task_state_path,
        Path::new(ROOT).join("state/tasks.json")
    );
    assert_eq!(
        resolved(&[(names::BACKEND_SESSION_STATE_PATH, "sessions.json")])
            .backend_session_state_path,
        Path::new(ROOT).join("sessions.json")
    );
    assert!(
        resolved(&[])
            .backend_session_state_path
            .ends_with("state/acp-sessions.json")
    );
    assert_eq!(
        resolved(&[(names::FRONTEND_PROMPT_DIR, "custom-prompt")]).frontend_prompt_dir,
        Path::new(ROOT).join("custom-prompt")
    );
    assert_eq!(
        resolved(&[(names::ASSISTANT_PROFILE_PATH, "ASSISTANT.md")]).assistant_profile_path,
        Path::new(ROOT).join("ASSISTANT.md")
    );

    // The two legacy-name fallback chains: the legacy spelling still works,
    // and the modern one wins when both are set.
    assert_eq!(
        resolved(&[(names::USER_PROFILE_PATH, "legacy/USER.md")]).user_model_path,
        Path::new(ROOT).join("legacy/USER.md")
    );
    assert_eq!(
        resolved(&[
            (names::USER_MODEL_PATH, "modern/USER.md"),
            (names::USER_PROFILE_PATH, "legacy/USER.md"),
        ])
        .user_model_path,
        Path::new(ROOT).join("modern/USER.md")
    );
    assert_eq!(
        resolved(&[(names::FRONTEND_MEMORY_PATH, "legacy/MEMORY.md")]).frontend_memory_path,
        Path::new(ROOT).join("legacy/MEMORY.md")
    );
    assert_eq!(
        resolved(&[
            (names::MEMORY_PATH, "modern/MEMORY.md"),
            (names::FRONTEND_MEMORY_PATH, "legacy/MEMORY.md"),
        ])
        .frontend_memory_path,
        Path::new(ROOT).join("modern/MEMORY.md")
    );
}

// ── 24. QWEN_AUDIO_AGENT_RUNTIME_ROOT ────────────────────────────────────────

#[test]
fn runtime_root_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_AGENT_RUNTIME_ROOT");
    assert!(contract.exact_value.contains("package root"));
    assert_eq!(names::RUNTIME_ROOT, "VIA_RUNTIME_ROOT");
    assert_eq!(
        rebranded("QWEN_AUDIO_AGENT_RUNTIME_ROOT"),
        names::RUNTIME_ROOT
    );

    // Unset: the installation root (here, the pinned `runtime_root` override).
    assert_eq!(resolved(&[]).root, Path::new(ROOT));
    // Set: overrides it, and every relative path anchors against the new root.
    let overridden = resolved(&[
        (names::RUNTIME_ROOT, "/elsewhere"),
        (names::TASK_STATE_PATH, "tasks.json"),
    ]);
    assert_eq!(overridden.root, Path::new("/elsewhere"));
    assert_eq!(
        overridden.task_state_path,
        Path::new("/elsewhere/tasks.json")
    );
}

// ── 25. QWEN_AUDIO_LOG_LEVEL / _MAX_BYTES / _MAX_FILES ──────────────────────
//
// Owned by `via-log`, not `via-core`: the three names and their defaults are
// `via_log::env`'s, and `via-core` only resolves the directory they log into.

#[test]
fn log_env_vars_match_their_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_LOG_LEVEL / _MAX_BYTES / _MAX_FILES");
    assert!(contract.exact_value.contains("info"));
    assert!(contract.exact_value.contains("10485760"));
    assert!(contract.exact_value.contains(" 5"));
    assert!(contract.why.contains(".env.example"));

    assert_eq!(rebranded("QWEN_AUDIO_LOG_LEVEL"), via_log::ENV_LOG_LEVEL);
    assert_eq!(
        rebranded("QWEN_AUDIO_LOG_MAX_BYTES"),
        via_log::ENV_LOG_MAX_BYTES
    );
    assert_eq!(
        rebranded("QWEN_AUDIO_LOG_MAX_FILES"),
        via_log::ENV_LOG_MAX_FILES
    );

    let settings = via_log::EnvSettings::from_env(&env(&[]), Path::new(HOME));
    assert_eq!(via_log::DEFAULT_LOG_LEVEL, via_log::LogLevel::Info);
    assert_eq!(settings.max_bytes, via_log::DEFAULT_MAX_BYTES);
    assert_eq!(u64::from(10_485_760u32), via_log::DEFAULT_MAX_BYTES);
    assert_eq!(settings.max_files, via_log::DEFAULT_MAX_FILES);
    assert_eq!(via_log::DEFAULT_MAX_FILES, 5);

    // `via-core` resolves the directory `via-log` then writes into.
    assert_eq!(
        settings.directory,
        resolved(&[]).config_directory().join("logs")
    );

    // Present in the runtime `config.env` template `via-core` ships, absent
    // from `.env.example` (a document `via-core` does not generate at all).
    let template = user_config_template(Locale::En);
    for name in [
        via_log::ENV_LOG_LEVEL,
        via_log::ENV_LOG_MAX_BYTES,
        via_log::ENV_LOG_MAX_FILES,
    ] {
        assert!(
            template.contains(&format!("# {name}=")),
            "`{name}` is missing from the shipped config.env template"
        );
    }
}

// ── 26/40. Realtime provider environment (the DashScope half) ──────────────

#[test]
fn realtime_env_vars_match_their_contract() {
    let contract = expect_contract(
        "env-var",
        "QWEN_AUDIO_REALTIME_API_KEY / QWEN_AUDIO_REALTIME_BASE_URL / QWEN_AUDIO_REALTIME_URL / \
         QWEN_AUDIO_REALTIME_MODEL / QWEN_AUDIO_REALTIME_VOICE / QWEN_OMNI_REALTIME_VOICE",
    );
    assert!(
        contract
            .exact_value
            .contains("qwen-audio-3.0-realtime-plus")
    );

    for (upstream, shipped) in [
        ("QWEN_AUDIO_REALTIME_API_KEY", names::REALTIME_API_KEY),
        ("QWEN_AUDIO_REALTIME_BASE_URL", names::REALTIME_BASE_URL),
        ("QWEN_AUDIO_REALTIME_URL", names::REALTIME_URL),
        ("QWEN_AUDIO_REALTIME_MODEL", names::REALTIME_MODEL),
        ("QWEN_AUDIO_REALTIME_VOICE", names::REALTIME_VOICE),
        ("QWEN_OMNI_REALTIME_VOICE", names::OMNI_REALTIME_VOICE),
    ] {
        assert_eq!(rebranded(upstream), shipped, "{upstream}");
    }

    // `apiKey = REALTIME_API_KEY || DASHSCOPE_API_KEY`.
    assert_eq!(
        resolved(&[(names::DASHSCOPE_API_KEY, "d")])
            .realtime
            .dashscope_api_key
            .expose(),
        "d"
    );
    // `url = REALTIME_BASE_URL || REALTIME_URL || <workspace> || default`,
    // trailing `?` stripped.
    assert_eq!(
        resolved(&[(names::REALTIME_URL, "wss://legacy.example/realtime??")])
            .realtime
            .dashscope_realtime_url,
        "wss://legacy.example/realtime"
    );
    assert_eq!(
        resolved(&[
            (names::REALTIME_BASE_URL, "wss://wins.example"),
            (names::REALTIME_URL, "wss://loses.example"),
        ])
        .realtime
        .dashscope_realtime_url,
        "wss://wins.example"
    );
    assert_eq!(
        resolved(&[]).realtime.dashscope_realtime_url,
        "wss://dashscope.aliyuncs.com/api-ws/v1/realtime"
    );
    // model default.
    assert_eq!(
        resolved(&[]).realtime.dashscope_model,
        "qwen-audio-3.0-realtime-plus"
    );

    // The voice override is family-scoped: `VIA_REALTIME_VOICE` only for
    // `audio`, `VIA_OMNI_REALTIME_VOICE` only for `omni`.
    let audio = resolved(&[
        (names::DASHSCOPE_API_KEY, "k"),
        (names::REALTIME_MODEL, "qwen-audio-3.0-realtime-plus"),
        (names::REALTIME_VOICE, "Cherry"),
        (names::OMNI_REALTIME_VOICE, "Ethan-custom"),
    ]);
    assert_eq!(audio.realtime.dashscope_voice, "Cherry");
    let omni = resolved(&[
        (names::DASHSCOPE_API_KEY, "k"),
        (names::REALTIME_MODEL, "qwen3.5-omni-plus-realtime"),
        (names::REALTIME_VOICE, "Cherry"),
        (names::OMNI_REALTIME_VOICE, "Ethan-custom"),
    ]);
    assert_eq!(omni.realtime.dashscope_voice, "Ethan-custom");
}

// ── 40. realtime configuration environment (the aggregate contract) ────────

#[test]
fn realtime_configuration_environment_matches_its_contract() {
    let contract = expect_contract("env-var", "realtime configuration environment");
    for name in [
        "DASHSCOPE_API_KEY",
        "QWEN_AUDIO_REALTIME_MODEL",
        "QWEN_AUDIO_REALTIME_BASE_URL",
        "QWEN_AUDIO_REALTIME_API_KEY",
        "QWEN_AUDIO_REALTIME_PROVIDER",
        "SPEECH_TO_SPEECH_REALTIME_URL",
        "QWEN_AUDIO_REALTIME_VOICE",
        "QWEN_OMNI_REALTIME_VOICE",
    ] {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` is missing from the catalogued list ({})",
            contract.file
        );
    }
    assert!(contract.why.contains("FAMILY-SCOPED"));

    // With neither voice variable set, the resolved voice is the empty
    // string — not a profile default — so a provider can tell "unset" from
    // "explicitly the default" apart.
    let neither_set = resolved(&[
        (names::DASHSCOPE_API_KEY, "k"),
        (names::REALTIME_MODEL, "qwen-audio-3.0-realtime-plus"),
    ]);
    assert_eq!(neither_set.realtime.dashscope_voice, "");
    let neither_set_omni = resolved(&[
        (names::DASHSCOPE_API_KEY, "k"),
        (names::REALTIME_MODEL, "qwen3.5-omni-plus-realtime"),
    ]);
    assert_eq!(neither_set_omni.realtime.dashscope_voice, "");
}

// ── 27. QWEN_AUDIO_REALTIME_MODEL (`.env.example` line) ─────────────────────

#[test]
fn realtime_model_env_example_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_REALTIME_MODEL");
    assert!(contract.exact_value.contains("qwen3.5-omni-flash-realtime"));
    assert_eq!(names::REALTIME_MODEL, "VIA_REALTIME_MODEL");

    // The actual default when unset, which the comment names explicitly.
    assert_eq!(
        resolved(&[]).realtime.dashscope_model,
        "qwen-audio-3.0-realtime-plus"
    );
    assert_eq!(
        resolved(&[(names::REALTIME_MODEL, "qwen3.5-omni-flash-realtime")])
            .realtime
            .dashscope_model,
        "qwen3.5-omni-flash-realtime"
    );
}

// ── 28. QWEN_AUDIO_REALTIME_PROVIDER ─────────────────────────────────────────

#[test]
fn realtime_provider_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_REALTIME_PROVIDER");
    assert!(contract.exact_value.contains("dashscope"));
    assert!(contract.exact_value.contains("speech-to-speech"));
    assert_eq!(names::REALTIME_PROVIDER, "VIA_REALTIME_PROVIDER");
    assert_eq!(
        rebranded("QWEN_AUDIO_REALTIME_PROVIDER"),
        names::REALTIME_PROVIDER
    );

    assert_eq!(resolved(&[]).realtime.provider, "dashscope");
    assert_eq!(resolved(&[]).realtime.label, "DashScope");
    // The `qwen` alias, KEEP, still selects `dashscope`.
    assert_eq!(
        resolved(&[(names::REALTIME_PROVIDER, "qwen")])
            .realtime
            .provider,
        "dashscope"
    );
    assert_eq!(
        resolved(&[(names::REALTIME_PROVIDER, "s2s")])
            .realtime
            .provider,
        "speech-to-speech"
    );
    assert_eq!(
        resolved(&[(names::REALTIME_PROVIDER, "speech-to-speech")])
            .realtime
            .label,
        "Hugging Face Speech-to-Speech"
    );
    let error =
        try_resolved(&[(names::REALTIME_PROVIDER, "nonesuch")]).expect_err("unknown provider");
    assert_eq!(error.code(), "VIA_REALTIME_PROVIDER_UNSUPPORTED");
}

// ── 29. QWEN_AUDIO_REALTIME_VOICE (`.env.example` line) ─────────────────────

#[test]
fn realtime_voice_env_example_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_AUDIO_REALTIME_VOICE");
    assert!(contract.exact_value.contains("longanqian"));
    assert_eq!(names::REALTIME_VOICE, "VIA_REALTIME_VOICE");
    assert_eq!(
        via_catalog::realtime_model::DEFAULT_DASHSCOPE_REALTIME_VOICE,
        "longanqian"
    );
    // Applies only to the `audio` family.
    assert_eq!(
        resolved(&[
            (names::DASHSCOPE_API_KEY, "k"),
            (names::REALTIME_MODEL, "qwen-audio-3.0-realtime-plus"),
            (names::REALTIME_VOICE, "custom-audio"),
        ])
        .realtime
        .dashscope_voice,
        "custom-audio"
    );
    assert_eq!(
        resolved(&[
            (names::DASHSCOPE_API_KEY, "k"),
            (names::REALTIME_MODEL, "qwen3.5-omni-plus-realtime"),
            (names::REALTIME_VOICE, "custom-audio"),
        ])
        .realtime
        .dashscope_voice,
        ""
    );
}

// ── 30. QWEN_CODE_BIN / QWEN_CODE_WORKSPACE ─────────────────────────────────

#[test]
fn qwen_code_block_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_CODE_BIN / QWEN_CODE_WORKSPACE");
    assert!(contract.exact_value.contains(names::QWEN_CODE_BIN));
    assert_eq!(names::QWEN_CODE_BIN, "QWEN_CODE_BIN");
    let qwen = backend_definition("qwen").expect("qwen is in the catalog");
    assert_eq!(qwen.workspace_environment, "QWEN_CODE_WORKSPACE");

    let config = resolved(&[
        ("AGENT_PROTOCOL", "qwen"),
        (names::QWEN_CODE_BIN, "/opt/qwen"),
        ("QWEN_CODE_WORKSPACE", "qwen-ws"),
    ]);
    assert_eq!(config.backends.qwen.cli_path, "/opt/qwen");
    assert_eq!(
        config.backends.qwen.directory,
        Path::new(ROOT).join("qwen-ws")
    );
}

// ── 31. QWEN_OMNI_REALTIME_VOICE (`.env.example` line) ──────────────────────

#[test]
fn omni_realtime_voice_env_example_matches_its_contract() {
    let contract = expect_contract("env-var", "QWEN_OMNI_REALTIME_VOICE");
    assert!(contract.exact_value.contains("QWEN_OMNI_REALTIME_VOICE"));
    assert_eq!(names::OMNI_REALTIME_VOICE, "VIA_OMNI_REALTIME_VOICE");
    assert_eq!(
        rebranded("QWEN_OMNI_REALTIME_VOICE"),
        names::OMNI_REALTIME_VOICE
    );
    assert_eq!(
        via_catalog::realtime_model::DASHSCOPE_OMNI_REALTIME_VOICE,
        "Ethan"
    );

    assert_eq!(
        resolved(&[
            (names::DASHSCOPE_API_KEY, "k"),
            (names::REALTIME_MODEL, "qwen3.5-omni-plus-realtime"),
            (names::OMNI_REALTIME_VOICE, "custom-omni"),
        ])
        .realtime
        .dashscope_voice,
        "custom-omni"
    );
}

// ── 32/33. SPEECH_TO_SPEECH_REALTIME_URL ────────────────────────────────────

#[test]
fn speech_to_speech_fallback_chain_matches_its_contracts() {
    let env_example = expect_contract("env-var", "SPEECH_TO_SPEECH_REALTIME_URL");
    assert!(
        env_example
            .exact_value
            .contains("ws://127.0.0.1:8765/v1/realtime")
    );
    let behavioural = expect_contract(
        "env-var",
        "SPEECH_TO_SPEECH_REALTIME_URL / S2S_REALTIME_URL / SPEECH_TO_SPEECH_AUTH_TOKEN / \
         S2S_API_KEY",
    );
    assert!(behavioural.exact_value.contains("speechToSpeechConfigured"));

    assert_eq!(
        names::SPEECH_TO_SPEECH_REALTIME_URL,
        "SPEECH_TO_SPEECH_REALTIME_URL"
    );
    assert_eq!(names::S2S_REALTIME_URL, "S2S_REALTIME_URL");
    assert_eq!(
        names::SPEECH_TO_SPEECH_AUTH_TOKEN,
        "SPEECH_TO_SPEECH_AUTH_TOKEN"
    );
    assert_eq!(names::S2S_API_KEY, "S2S_API_KEY");
    assert_eq!(
        via_catalog::DEFAULT_SPEECH_TO_SPEECH_REALTIME_URL,
        "ws://127.0.0.1:8765/v1/realtime"
    );

    assert_eq!(
        resolved(&[]).realtime.speech_to_speech_realtime_url,
        "ws://127.0.0.1:8765/v1/realtime"
    );
    assert!(!resolved(&[]).realtime.speech_to_speech_configured);
    // URL fallback chain, trailing slash stripped.
    assert_eq!(
        resolved(&[(names::S2S_REALTIME_URL, "ws://legacy.example/rt/")])
            .realtime
            .speech_to_speech_realtime_url,
        "ws://legacy.example/rt"
    );
    assert_eq!(
        resolved(&[
            (names::SPEECH_TO_SPEECH_REALTIME_URL, "ws://wins.example"),
            (names::S2S_REALTIME_URL, "ws://loses.example"),
        ])
        .realtime
        .speech_to_speech_realtime_url,
        "ws://wins.example"
    );
    // A default endpoint alone never advertises the provider; an explicit one
    // does.
    assert!(
        resolved(&[(
            names::SPEECH_TO_SPEECH_REALTIME_URL,
            "ws://explicit.example"
        )])
        .realtime
        .speech_to_speech_configured
    );
    assert!(
        resolved(&[(names::REALTIME_PROVIDER, "speech-to-speech")])
            .realtime
            .speech_to_speech_configured
    );
    // Token fallback chain.
    assert_eq!(
        resolved(&[(names::S2S_API_KEY, "fallback-token")])
            .realtime
            .speech_to_speech_auth_token
            .expose(),
        "fallback-token"
    );
    assert_eq!(
        resolved(&[
            (names::SPEECH_TO_SPEECH_AUTH_TOKEN, "wins-token"),
            (names::S2S_API_KEY, "loses-token"),
        ])
        .realtime
        .speech_to_speech_auth_token
        .expose(),
        "wins-token"
    );
}

// ── 34/35. Announcement engine env vars ──────────────────────────────────────

#[test]
fn announcement_settings_match_their_contracts() {
    for name in ["announcement env vars", "announcement tuning"] {
        let contract = expect_contract("env-var", name);
        assert!(!contract.exact_value.is_empty());
    }

    let settings: &[(&str, &str)] = &[
        (
            "QWEN_AUDIO_AGENT_ANNOUNCE_INTO_CONTEXT",
            names::ANNOUNCE_INTO_CONTEXT,
        ),
        (
            "QWEN_AUDIO_AGENT_RESULT_CONTEXT_MAX_CHARS",
            names::RESULT_CONTEXT_MAX_CHARS,
        ),
        (
            "QWEN_AUDIO_AGENT_ANNOUNCEMENT_BATCH_MS",
            names::ANNOUNCEMENT_BATCH_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_ANNOUNCEMENT_MAX_BATCH_ITEMS",
            names::ANNOUNCEMENT_MAX_BATCH_ITEMS,
        ),
        (
            "QWEN_AUDIO_AGENT_ANNOUNCEMENT_QUIET_MS",
            names::ANNOUNCEMENT_QUIET_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_ANNOUNCEMENT_ACK_TIMEOUT_MS",
            names::ANNOUNCEMENT_ACK_TIMEOUT_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_ANNOUNCEMENT_MAX_RETRIES",
            names::ANNOUNCEMENT_MAX_RETRIES,
        ),
    ];

    for (upstream, shipped) in settings {
        assert_eq!(rebranded(upstream), *shipped, "{upstream}");
    }

    assert!(resolved(&[]).announce_into_context);
    assert!(!resolved(&[(names::ANNOUNCE_INTO_CONTEXT, "false")]).announce_into_context);

    assert_eq!(resolved(&[]).result_context_max_chars, 6_000);
    assert_eq!(
        resolved(&[(names::RESULT_CONTEXT_MAX_CHARS, "1")]).result_context_max_chars,
        256
    );

    assert_eq!(resolved(&[]).announcement_batch_ms, 120);
    assert_eq!(
        resolved(&[(names::ANNOUNCEMENT_BATCH_MS, "5000")]).announcement_batch_ms,
        1_000
    );

    assert_eq!(resolved(&[]).announcement_max_batch_items, 8);
    assert_eq!(
        resolved(&[(names::ANNOUNCEMENT_MAX_BATCH_ITEMS, "0")]).announcement_max_batch_items,
        1
    );
    assert_eq!(
        resolved(&[(names::ANNOUNCEMENT_MAX_BATCH_ITEMS, "1000")]).announcement_max_batch_items,
        32
    );

    assert_eq!(resolved(&[]).announcement_quiet_ms, 350);
    assert_eq!(
        resolved(&[(names::ANNOUNCEMENT_QUIET_MS, "9999")]).announcement_quiet_ms,
        2_000
    );

    assert_eq!(
        resolved(&[]).announcement_acknowledgement_timeout_ms,
        120_000
    );
    assert_eq!(
        resolved(&[(names::ANNOUNCEMENT_ACK_TIMEOUT_MS, "1")])
            .announcement_acknowledgement_timeout_ms,
        10_000
    );

    assert_eq!(resolved(&[]).announcement_max_retry_attempts, 8);
    assert_eq!(
        resolved(&[(names::ANNOUNCEMENT_MAX_RETRIES, "0")]).announcement_max_retry_attempts,
        1
    );
    assert_eq!(
        resolved(&[(names::ANNOUNCEMENT_MAX_RETRIES, "1000")]).announcement_max_retry_attempts,
        32
    );
}

// ── 36. Backend CLI path and config-dir overrides ───────────────────────────

#[test]
fn backend_cli_path_and_config_dir_overrides_match_their_contract() {
    let contract = expect_contract("env-var", "backend CLI path and config-dir overrides");
    let all_via_core: &[&str] = &[
        names::OPENCLAW_ACP_BIN,
        names::QODERCLI_PATH,
        names::QODER_CLI_PATH,
        names::QODER_CONFIG_DIR,
        names::QWEN_CODE_BIN,
        names::KIMI_CODE_BIN,
        names::HERMES_BIN,
        names::CODEBUDDY_BIN,
        names::CODEX_ACP_BIN,
        names::CLAUDE_CODE_ACP_BIN,
        names::CLAUDE_CODE_EXECUTABLE,
        names::CLAUDE_CONFIG_DIR,
        names::DEEPSEEK_HARNESS_ACP_BIN,
        names::PI_ACP_BIN,
    ];
    assert_eq!(
        all_via_core.len(),
        14,
        "OPENCLAW_ACP_BIN + QODERCLI_PATH's two spellings + 11 others"
    );
    for name in all_via_core {
        assert!(
            contract.exact_value.contains(name),
            "`{name}` is not in the catalogued list ({})",
            contract.file
        );
    }

    // All default to empty, trimmed strings.
    let empty = resolved(&[]);
    assert_eq!(empty.backends.openclaw.cli_path, "");
    assert_eq!(empty.backends.qoder.cli_path, "");
    assert_eq!(empty.backends.qwen.cli_path, "");
    assert_eq!(empty.backends.kimi.cli_path, "");
    assert_eq!(empty.backends.hermes.cli_path, "");
    assert_eq!(empty.backends.codebuddy.cli_path, "");
    assert_eq!(empty.backends.codex.cli_path, "");
    assert_eq!(empty.backends.claude.cli_path, "");
    assert_eq!(empty.backends.claude.claude_executable, "");
    assert_eq!(empty.backends.deepseek.cli_path, "");
    assert_eq!(empty.backends.pi.cli_path, "");
    assert_eq!(empty.backends.qoder.config_directory, PathBuf::new());
    assert_eq!(empty.backends.claude.config_directory, PathBuf::new());

    // Trimmed, and each read.
    let config = resolved(&[
        (names::QODERCLI_PATH, "  /opt/qodercli  "),
        (names::QODER_CONFIG_DIR, "qoder-config"),
        (names::CLAUDE_CONFIG_DIR, "claude-config"),
    ]);
    assert_eq!(config.backends.qoder.cli_path, "/opt/qodercli");
    // `QODER_CONFIG_DIR` / `CLAUDE_CONFIG_DIR` are resolved *absolutely*
    // against the working directory (single-argument `resolve()`), unlike
    // every other override, which anchors to the runtime root.
    assert_eq!(
        config.backends.qoder.config_directory,
        Path::new(CWD).join("qoder-config")
    );
    assert_eq!(
        config.backends.claude.config_directory,
        Path::new(CWD).join("claude-config")
    );

    // The legacy `QODER_CLI_PATH` alias.
    assert_eq!(
        resolved(&[(names::QODER_CLI_PATH, "/legacy/qodercli")])
            .backends
            .qoder
            .cli_path,
        "/legacy/qodercli"
    );
}

// ── 37. Env file load precedence ────────────────────────────────────────────

#[test]
fn env_file_load_precedence_matches_its_contract() {
    let contract = expect_contract("env-var", "env file load precedence");
    assert!(contract.exact_value.contains(".env.local"));
    assert!(contract.exact_value.contains("config.env"));
    assert!(contract.exact_value.contains("FIRST source wins"));

    // The file only fills what the environment left undefined.
    let filled_by_file =
        resolve(&env(&[]), Some("HOST=192.0.2.1\n"), &overrides()).expect("resolves");
    assert_eq!(filled_by_file.host, "192.0.2.1");
    let masked_by_env = resolve(
        &env(&[("HOST", "10.0.0.1")]),
        Some("HOST=192.0.2.1\n"),
        &overrides(),
    )
    .expect("resolves");
    assert_eq!(masked_by_env.host, "10.0.0.1");
    // An empty shell assignment still masks the file, and the default applies
    // rather than the file's value — the general rule.
    let empty_shell = resolve(
        &env(&[("HOST", "")]),
        Some("HOST=192.0.2.1\n"),
        &overrides(),
    )
    .expect("resolves");
    assert_eq!(empty_shell.host, "127.0.0.1");

    // The single documented exception: `VIA_AUTH_SECRET` is deleted from the
    // environment before `state.env` is read, so an empty shell assignment
    // cannot mask the persisted secret. Exercised through the real,
    // file-system-touching `load_runtime_environment`.
    let temp = tempfile::tempdir().expect("a temp directory");
    let root = temp.path().join("root");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&root).expect("create root");
    std::fs::create_dir_all(&home).expect("create home");
    let mut process_env: EnvMap = [(names::AUTH_SECRET, ""), (names::CONFIG_DIR, "config")]
        .into_iter()
        .collect();
    let options = RuntimeOptions {
        root: root.clone(),
        home_directory: home,
        working_directory: root.clone(),
        locale: Locale::En,
        generate_secret: true,
        read_only: false,
    };
    let outcome = load_runtime_environment(&mut process_env, &options)
        .expect("scaffolding a fresh install succeeds");
    assert!(
        outcome.generated_secret,
        "an empty shell assignment must not suppress generation"
    );
    assert_eq!(
        process_env.get(names::AUTH_SECRET).map(str::len),
        Some(64),
        "a fresh 64-hex-character secret must replace the empty shell assignment"
    );
}

// ── 38. gatewayOptionsEnvironment mapping ───────────────────────────────────

#[test]
fn gateway_options_environment_mapping_matches_its_contract() {
    let contract = expect_contract("env-var", "gatewayOptionsEnvironment mapping");
    assert!(contract.exact_value.contains("QWAUDIO_CONFIG_DIR"));
    assert!(contract.exact_value.contains("emit NOTHING"));

    // An option that was not passed emits nothing at all.
    assert!(GatewayOptions::default().to_environment().is_empty());

    let full = GatewayOptions {
        config_dir: Some(PathBuf::from("/cfg")),
        host: Some("0.0.0.0".to_owned()),
        port: Some(9000),
        backend: Some(BackendSelection::Named("openclaw".to_owned())),
        wake_word: Some(true),
        owner: Some("desktop".to_owned()),
        log_console: Some(false),
    };
    let mapped = full.to_environment();
    assert_eq!(mapped.get(names::CONFIG_DIR), Some("/cfg"));
    assert_eq!(mapped.get(names::HOST), Some("0.0.0.0"));
    assert_eq!(mapped.get(names::PORT), Some("9000"));
    assert_eq!(mapped.get(names::AGENT_PROTOCOL), Some("openclaw"));
    assert_eq!(mapped.get(names::WAKE_WORD_ENABLED), Some("true"));
    assert_eq!(mapped.get(names::GATEWAY_OWNER), Some("desktop"));
    assert_eq!(mapped.get(via_log::ENV_LOG_CONSOLE), Some("0"));

    // `backend: None` (the "no backend" host request) sets the empty string
    // rather than deleting the key, so a child Gateway cannot silently reload
    // a stale `AGENT_PROTOCOL` from its config file.
    let none_selected = GatewayOptions {
        backend: Some(BackendSelection::None),
        ..GatewayOptions::default()
    }
    .to_environment();
    assert!(none_selected.contains_key(names::AGENT_PROTOCOL));
    assert_eq!(none_selected.get(names::AGENT_PROTOCOL), Some(""));
    let config = resolve(
        &EnvMap::new(),
        Some("AGENT_PROTOCOL=openclaw\n"),
        &overrides().with_gateway(GatewayOptions {
            backend: Some(BackendSelection::None),
            ..GatewayOptions::default()
        }),
    )
    .expect("resolves");
    assert_eq!(config.agent_protocol, "");
}

// ── 41. Realtime frontend environment variables ─────────────────────────────

#[test]
fn realtime_frontend_environment_variables_match_their_contract() {
    let contract = expect_contract("env-var", "realtime frontend environment variables");
    assert!(contract.why.contains("configurationSignature"));
    assert!(
        contract
            .exact_value
            .contains("QWEN_AUDIO_AGENT_ALLOWED_ORIGINS")
    );

    for (upstream, shipped) in [
        ("QWEN_AUDIO_WAKE_WORD_ENABLED", names::WAKE_WORD_ENABLED),
        ("QWEN_AUDIO_WAKE_WORD_MODEL_DIR", names::WAKE_WORD_MODEL_DIR),
        ("QWEN_AUDIO_AGENT_ALLOWED_ORIGINS", names::ALLOWED_ORIGINS),
    ] {
        assert_eq!(rebranded(upstream), shipped, "{upstream}");
    }
    assert!(
        contract
            .exact_value
            .contains("QWEN_AUDIO_DESKTOP_AUTO_HIDE_SECONDS")
    );
    // `docs/rebrand.md` gives three legal spellings for this one upstream
    // variable; VIA takes the narrowest of them rather than the naive
    // `rebranded()` rule's `VIA_DESKTOP_AUTO_HIDE_SECONDS` — recorded at
    // `docs/deviations/phase-1.md` and in `names::SLEEP_TIMEOUT_SECONDS`'s own
    // doc comment.
    assert_eq!(names::SLEEP_TIMEOUT_SECONDS, "VIA_SLEEP_TIMEOUT_SECONDS");

    // The seconds-to-milliseconds conversion, default disabled (0).
    assert_eq!(resolved(&[]).sleep_timeout_ms, 0);
    assert_eq!(
        resolved(&[(names::SLEEP_TIMEOUT_SECONDS, "90")]).sleep_timeout_ms,
        90_000
    );

    // The configuration-identity signature: changing only the model changes
    // the signature, and the signature is computed from the resolved
    // provider/endpoint/model/voice/credential, all of them from this
    // environment surface.
    let shared: &[(&str, &str)] = &[
        (names::DASHSCOPE_API_KEY, "same-key"),
        (names::REALTIME_BASE_URL, "wss://gateway.example/realtime"),
        (names::REALTIME_VOICE, "same-voice"),
    ];
    let mut first: Vec<(&str, &str)> = shared.to_vec();
    first.push((names::REALTIME_MODEL, "qwen-audio-3.0-realtime-plus"));
    let mut second: Vec<(&str, &str)> = shared.to_vec();
    second.push((names::REALTIME_MODEL, "qwen3.5-omni-plus-realtime"));
    assert_ne!(
        resolved(&first).realtime.signature,
        resolved(&second).realtime.signature
    );
    assert_eq!(
        resolved(&first).realtime.signature,
        resolved(&first).realtime.signature,
        "the same environment must hash to the same signature"
    );
}

// ── 42/44. Scheduler, sleep and wake word ───────────────────────────────────

#[test]
fn scheduler_sleep_and_wake_word_env_vars_match_their_contract() {
    for name in ["scheduler and sleep", "wake word / sleep env vars"] {
        let contract = expect_contract("env-var", name);
        assert!(contract.exact_value.contains("WAKE_WORD"));
    }

    for (upstream, shipped) in [
        (
            "QWEN_AUDIO_AGENT_REMINDER_SCHEDULER",
            names::REMINDER_SCHEDULER,
        ),
        (
            "QWEN_AUDIO_AGENT_REMINDER_MAX_PER_OWNER",
            names::REMINDER_MAX_PER_OWNER,
        ),
        (
            "QWEN_AUDIO_AGENT_SCHEDULED_TASK_TIMEOUT_MS",
            names::SCHEDULED_TASK_TIMEOUT_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_BACKGROUND_TASK_PROGRESS_CHECK_MS",
            names::BACKGROUND_TASK_PROGRESS_CHECK_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_SCHEDULED_TASK_PROGRESS_CHECK_MS",
            names::SCHEDULED_TASK_PROGRESS_CHECK_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_OFFLINE_NOTIFICATION_DELAY_MS",
            names::OFFLINE_NOTIFICATION_DELAY_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_REMINDER_STAGGER_MS",
            names::REMINDER_STAGGER_MS,
        ),
        ("QWEN_AUDIO_WAKE_WORD_ENABLED", names::WAKE_WORD_ENABLED),
        ("QWEN_AUDIO_WAKE_WORD_MODEL_DIR", names::WAKE_WORD_MODEL_DIR),
    ] {
        assert_eq!(rebranded(upstream), shipped, "{upstream}");
    }

    assert!(resolved(&[]).reminder_scheduler_enabled);
    assert!(!resolved(&[(names::REMINDER_SCHEDULER, "false")]).reminder_scheduler_enabled);

    assert_eq!(resolved(&[]).reminder_max_per_owner, 50);
    assert_eq!(
        resolved(&[(names::REMINDER_MAX_PER_OWNER, "0")]).reminder_max_per_owner,
        1
    );
    assert_eq!(
        resolved(&[(names::REMINDER_MAX_PER_OWNER, "9999")]).reminder_max_per_owner,
        500
    );

    assert_eq!(resolved(&[]).scheduled_task_timeout_ms, 1_800_000);
    assert_eq!(
        resolved(&[(names::SCHEDULED_TASK_TIMEOUT_MS, "1")]).scheduled_task_timeout_ms,
        60_000
    );

    assert_eq!(resolved(&[]).background_task_progress_check_ms, 300_000);
    assert_eq!(
        resolved(&[(names::SCHEDULED_TASK_PROGRESS_CHECK_MS, "31000")])
            .background_task_progress_check_ms,
        31_000,
        "the legacy spelling is a real fallback"
    );
    assert_eq!(
        resolved(&[
            (names::BACKGROUND_TASK_PROGRESS_CHECK_MS, "40000"),
            (names::SCHEDULED_TASK_PROGRESS_CHECK_MS, "31000"),
        ])
        .background_task_progress_check_ms,
        40_000,
        "the modern spelling wins"
    );

    assert_eq!(resolved(&[]).offline_notification_delay_ms, 5_000);
    assert_eq!(
        resolved(&[(names::OFFLINE_NOTIFICATION_DELAY_MS, "1")]).offline_notification_delay_ms,
        1_000
    );
    assert_eq!(
        resolved(&[(names::OFFLINE_NOTIFICATION_DELAY_MS, "999999")]).offline_notification_delay_ms,
        120_000
    );

    assert_eq!(resolved(&[]).reminder_stagger_ms, 30_000);
    assert_eq!(
        resolved(&[(names::REMINDER_STAGGER_MS, "999999")]).reminder_stagger_ms,
        300_000
    );

    assert!(!resolved(&[]).wake_word_enabled);
    assert!(resolved(&[(names::WAKE_WORD_ENABLED, "true")]).wake_word_enabled);
    assert!(
        resolved(&[])
            .wake_word_model_directory
            .ends_with("models/wake-word")
    );

    // DELIBERATE DIVERGENCE, recorded at `docs/deviations/phase-1.md`: upstream
    // hard-codes the wake phrase `你好千问` with no override at all; VIA reads
    // it from `VIA_WAKE_WORD` and, because no phrase has been chosen for VIA
    // yet (`docs/architecture.md` §16 defers the choice), defaults to empty
    // rather than shipping a placeholder.
    assert_eq!(names::WAKE_WORD, "VIA_WAKE_WORD");
    assert_eq!(resolved(&[]).wake_word, "");
    assert_ne!(resolved(&[]).wake_word, "你好千问"); // upstream's literal, not VIA's default
    assert_eq!(
        resolved(&[(names::WAKE_WORD, "你好千问")]).wake_word, // upstream's literal
        "你好千问",                                            // upstream's literal
        "the upstream phrase is still a legal value, just not the default"
    );
}

// ── 43. Task and session retention ──────────────────────────────────────────

#[test]
fn task_and_session_retention_matches_its_contract() {
    let contract = expect_contract("env-var", "task and session retention");
    assert!(contract.exact_value.contains("86400000"));

    for (upstream, shipped) in [
        (
            "QWEN_AUDIO_AGENT_TASK_TERMINAL_TTL_MS",
            names::TASK_TERMINAL_TTL_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_TASK_NOTIFICATION_TTL_MS",
            names::TASK_NOTIFICATION_TTL_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_TASK_NOTIFICATION_CLAIM_TTL_MS",
            names::TASK_NOTIFICATION_CLAIM_TTL_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_MAX_TERMINAL_TASKS_PER_OWNER",
            names::MAX_TERMINAL_TASKS_PER_OWNER,
        ),
        (
            "QWEN_AUDIO_AGENT_TASK_MAX_CONCURRENT",
            names::TASK_MAX_CONCURRENT,
        ),
        (
            "QWEN_AUDIO_AGENT_TASK_MAX_CONCURRENT_PER_OWNER",
            names::TASK_MAX_CONCURRENT_PER_OWNER,
        ),
        ("QWEN_AUDIO_AGENT_SESSION_TTL_MS", names::SESSION_TTL_MS),
        ("QWEN_AUDIO_AGENT_MAX_SESSIONS", names::MAX_SESSIONS),
        (
            "QWEN_AUDIO_AGENT_MEMORY_OWNER_TTL_MS",
            names::MEMORY_OWNER_TTL_MS,
        ),
        (
            "QWEN_AUDIO_AGENT_MAX_MEMORY_OWNERS",
            names::MAX_MEMORY_OWNERS,
        ),
    ] {
        assert_eq!(rebranded(upstream), shipped, "{upstream}");
    }

    assert_eq!(resolved(&[]).task_terminal_ttl_ms, 86_400_000);
    assert_eq!(
        resolved(&[(names::TASK_TERMINAL_TTL_MS, "1")]).task_terminal_ttl_ms,
        60_000
    );
    assert_eq!(resolved(&[]).task_pending_notification_ttl_ms, 604_800_000);
    assert_eq!(resolved(&[]).task_notification_claim_ttl_ms, 60_000);
    assert_eq!(
        resolved(&[(names::TASK_NOTIFICATION_CLAIM_TTL_MS, "1")]).task_notification_claim_ttl_ms,
        5_000
    );
    assert_eq!(resolved(&[]).max_terminal_tasks_per_owner, 100);
    assert_eq!(
        resolved(&[(names::MAX_TERMINAL_TASKS_PER_OWNER, "1")]).max_terminal_tasks_per_owner,
        10
    );
    assert_eq!(resolved(&[]).task_max_concurrent, 4);
    assert_eq!(
        resolved(&[(names::TASK_MAX_CONCURRENT, "0")]).task_max_concurrent,
        1
    );
    assert_eq!(
        resolved(&[(names::TASK_MAX_CONCURRENT, "999")]).task_max_concurrent,
        64
    );
    assert_eq!(resolved(&[]).task_max_concurrent_per_owner, 2);
    assert_eq!(
        resolved(&[(names::TASK_MAX_CONCURRENT_PER_OWNER, "999")]).task_max_concurrent_per_owner,
        16
    );
    assert_eq!(resolved(&[]).conversation_session_ttl_ms, 21_600_000);
    assert_eq!(
        resolved(&[(names::SESSION_TTL_MS, "1")]).conversation_session_ttl_ms,
        60_000
    );
    assert_eq!(resolved(&[]).max_conversation_sessions, 500);
    assert_eq!(
        resolved(&[(names::MAX_SESSIONS, "1")]).max_conversation_sessions,
        10
    );
    // Zero keeps memories forever, and is the default.
    assert_eq!(resolved(&[]).frontend_memory_owner_ttl_ms, 0);
    assert_eq!(
        resolved(&[(names::MEMORY_OWNER_TTL_MS, "-5")]).frontend_memory_owner_ttl_ms,
        0
    );
    assert_eq!(resolved(&[]).max_frontend_memory_owners, 1_000);
    assert_eq!(
        resolved(&[(names::MAX_MEMORY_OWNERS, "1")]).max_frontend_memory_owners,
        10
    );
}
