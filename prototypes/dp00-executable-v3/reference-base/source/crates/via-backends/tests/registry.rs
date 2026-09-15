//! The driver registry and the twelve launch specs.
//!
//! A port of `server/test/backend-driver-registry.test.mjs` and
//! `server/test/backend-driver-launch.test.mjs`, with the launcher assertions
//! adapted to VIA's launch specs — upstream asserts
//! `command === process.execPath` and `args[0] === scripts/<name>.mjs`, and
//! `docs/architecture.md` §10 replaces both with the executable the shim would
//! have `exec`'d.

mod support;

use support::{TableFinder, config, env, openclaw_config, profile, profile_with};
use via_backends::driver::{DSH_PERMISSION_FULL, DSH_PERMISSION_WORKSPACE};
use via_backends::{
    BackendsError, backend_driver, backend_drivers, backend_ids, runtime_registry,
    spawns_separate_process,
};
use via_catalog::{
    Ownership, backend_definition, backend_names, effective_backend_permission_mode,
    resolve_backend_ownership,
};
use via_core::EnvMap;
use via_downstream::{BackendCapabilities, CAPABILITY_FLAGS};

/// `server/test/backend-driver-registry.test.mjs:16-29`.
#[test]
fn every_advertised_backend_has_agent_and_runtime_drivers() {
    let environment = EnvMap::new();
    let runtime = runtime_registry(&environment).expect("the runtime drivers validate");
    for protocol in backend_names() {
        let driver = backend_driver(protocol).expect("an agent driver");
        assert_eq!(driver.id, protocol);

        let definition = backend_definition(protocol).expect("catalogued");
        let resolved = runtime.driver(protocol).expect("a runtime driver");
        assert_eq!(resolved.id, protocol);
        assert_eq!(
            resolved
                .service
                .as_ref()
                .map(|service| service.base_url_environment.as_str()),
            definition.base_url_environment,
            "{protocol} base URL environment"
        );
        assert_eq!(
            resolved
                .service
                .as_ref()
                .map(|service| service.default_base_url.as_str()),
            definition.default_base_url,
            "{protocol} default base URL"
        );
        assert_eq!(
            resolved.supports_external_service, definition.supports_external_service,
            "{protocol} external service"
        );
    }
}

/// `server/test/backend-driver-registry.test.mjs:31-49`.
///
/// Upstream checks that all seven flags are booleans. In Rust that is the type,
/// so what is left worth asserting is that the seven names still resolve and
/// that every declaration is internally consistent.
#[test]
fn every_agent_driver_publishes_one_validated_capability_contract() {
    for driver in backend_drivers() {
        for flag in CAPABILITY_FLAGS {
            assert!(
                driver.capabilities.flag(flag).is_some(),
                "{}.{flag}",
                driver.id
            );
        }
        assert!(
            driver.capabilities.is_consistent(),
            "{}: {:?}",
            driver.id,
            driver.capabilities.faults()
        );
    }
}

/// `server/test/backend-driver-registry.test.mjs:51-67` — the frozen set,
/// asserted by value.
#[test]
fn profile_construction_applies_the_immutable_driver_capability_contract() {
    let config = config();
    let built = profile("deepseek", &config, &EnvMap::new(), "native").expect("deepseek builds");
    assert_eq!(
        built.capabilities,
        BackendCapabilities {
            delegation: false,
            permissions: true,
            backend_ui: false,
            native_session_history: false,
            external_mcp: false,
            native_delegation: false,
            session_mcp: false,
        }
    );
    // Upstream freezes the object; `BackendCapabilities` is `Copy` with no
    // interior mutability, so the profile and the ACP half can never disagree.
    assert_eq!(built.acp.capabilities, built.capabilities);
}

/// `server/test/backend-driver-registry.test.mjs:69-80`.
#[test]
fn external_ownership_is_available_only_to_declared_backend_services() {
    assert_eq!(
        resolve_backend_ownership("openclaw", true, "").expect("openclaw"),
        Ownership::External
    );
    assert_eq!(
        resolve_backend_ownership("openclaw", false, "").expect("openclaw"),
        Ownership::Owned
    );
    assert_eq!(
        resolve_backend_ownership("opencode", true, "").expect("opencode"),
        Ownership::Owned
    );
    let error = resolve_backend_ownership("opencode", false, "external")
        .expect_err("opencode has no external service");
    assert!(matches!(
        error,
        via_catalog::CatalogError::ExternalServiceUnsupported { label: "OpenCode" }
    ));
}

/// `test/backend-catalog.test.mjs:8-21` and
/// `json-field/pi backend capability declaration`.
#[test]
fn pi_declares_the_always_full_permission_capability() {
    let pi = backend_definition("pi").expect("catalogued");
    assert!(pi.always_full_permission);
    assert!(pi.supports_full_permission);
    assert_eq!(effective_backend_permission_mode("pi", "native"), "full");
    assert_eq!(effective_backend_permission_mode("pi", ""), "full");

    let driver = backend_driver("pi").expect("a driver");
    assert!(!driver.capabilities.external_mcp);
    assert!(!driver.capabilities.session_mcp);
    assert!(!driver.capabilities.delegation);
    assert!(!driver.capabilities.permissions);

    let built = profile("pi", &config(), &EnvMap::new(), "full").expect("pi builds");
    assert!(
        built
            .session_instructions
            .expect("pi replaces the default paragraph")
            .text()
            .contains("does not expose Gateway Session tools")
    );
}

// ── launch specs — `server/test/backend-driver-launch.test.mjs` ─────────────

#[test]
fn opencode_launches_its_own_binary_with_the_acp_subcommand() {
    let built = profile("opencode", &config(), &EnvMap::new(), "native").expect("builds");
    assert_eq!(built.acp.spawn.command, "opencode");
    assert_eq!(built.acp.spawn.args, ["acp"]);
    assert_eq!(
        built.acp.spawn.cwd.as_deref(),
        Some(std::path::Path::new("/work"))
    );
}

#[test]
fn openclaw_launches_the_bridge_with_the_websocket_url_and_token_file() {
    let built = profile("openclaw", &config(), &EnvMap::new(), "native").expect("builds");
    assert_eq!(built.acp.spawn.command, "openclaw");
    assert_eq!(
        built.acp.spawn.args,
        [
            "acp",
            "--url",
            "ws://127.0.0.1:18789",
            "--token-file",
            "/config/backends/openclaw/state/gateway-token",
            "--verbose",
        ]
    );
    // The bridge's device identity is kept apart from the user's own CLI
    // identity by pointing it at the token file's directory.
    assert_eq!(
        built.acp.spawn.env.get("OPENCLAW_STATE_DIR"),
        Some("/config/backends/openclaw/state")
    );
}

#[test]
fn codex_is_launched_through_its_adapter_with_the_parent_cli_named() {
    let mut config = config();
    config.backends.codex.cli_path = "/opt/codex-acp".to_owned();
    let finder = TableFinder::new(&[("codex", "/usr/bin/codex")]);
    let built = profile_with(
        "codex",
        &config,
        &EnvMap::new(),
        "native",
        Ownership::Owned,
        &finder,
    )
    .expect("builds");
    assert_eq!(built.acp.spawn.command, "/opt/codex-acp");
    assert!(built.acp.spawn.args.is_empty());
    assert_eq!(
        built.acp.spawn.env.get("CODEX_ACP_BIN"),
        Some("/opt/codex-acp")
    );
    assert_eq!(
        built.acp.spawn.env.get("CODEX_PATH"),
        Some("/usr/bin/codex")
    );
    assert_eq!(built.acp.spawn.env.get("NO_BROWSER"), Some("1"));
    // No Bailian model configured: no provider blob at all.
    assert!(built.acp.spawn.env.get("CODEX_CONFIG").is_none());
    assert!(built.acp.spawn.env.get("MODEL_PROVIDER").is_none());
}

#[test]
fn codex_writes_its_provider_blob_when_a_model_url_is_configured() {
    let mut config = config();
    config.backends.codex.model = "qwen3.7-max".to_owned();
    config.backends.codex.model_url =
        "https://dashscope.aliyuncs.com/compatible-mode/v1".to_owned();
    let built = profile("codex", &config, &EnvMap::new(), "native").expect("builds");
    assert_eq!(built.acp.spawn.env.get("MODEL_PROVIDER"), Some("via"));
    let blob = built
        .acp
        .spawn
        .env
        .get("CODEX_CONFIG")
        .expect("a config blob");
    assert_eq!(
        blob,
        r#"{"model":"qwen3.7-max","model_provider":"via","model_providers":{"via":{"name":"via","base_url":"https://dashscope.aliyuncs.com/compatible-mode/v1","env_key":"DASHSCOPE_API_KEY","wire_api":"responses"}}}"#,
        "key order is observable and must stay JS insertion order"
    );
}

#[test]
fn codex_switches_its_agent_mode_only_in_full_permission() {
    let built = profile("codex", &config(), &EnvMap::new(), "full").expect("builds");
    assert_eq!(
        built.acp.spawn.env.get("INITIAL_AGENT_MODE"),
        Some("agent-full-access")
    );
    let native = profile("codex", &config(), &EnvMap::new(), "native").expect("builds");
    assert!(native.acp.spawn.env.get("INITIAL_AGENT_MODE").is_none());
}

#[test]
fn claude_names_the_executable_its_adapter_drives() {
    let finder = TableFinder::new(&[("claude", "/usr/bin/claude")]);
    let built = profile_with(
        "claude",
        &config(),
        &EnvMap::new(),
        "native",
        Ownership::Owned,
        &finder,
    )
    .expect("builds");
    assert_eq!(built.acp.spawn.command, "claude-code-acp");
    assert_eq!(
        built.acp.spawn.env.get("CLAUDE_CODE_EXECUTABLE"),
        Some("/usr/bin/claude")
    );
}

#[test]
fn claude_copies_its_own_key_variable_into_the_anthropic_one() {
    let environment = env(&[("CLAUDE_API_KEY", "sk-claude")]);
    let built = profile("claude", &config(), &environment, "native").expect("builds");
    assert_eq!(
        built.acp.spawn.env.get("ANTHROPIC_API_KEY"),
        Some("sk-claude")
    );

    // An explicit ANTHROPIC_API_KEY is never overwritten.
    let both = env(&[
        ("CLAUDE_API_KEY", "sk-claude"),
        ("ANTHROPIC_API_KEY", "sk-anthropic"),
    ]);
    let built = profile("claude", &config(), &both, "native").expect("builds");
    assert_eq!(
        built.acp.spawn.env.get("ANTHROPIC_API_KEY"),
        Some("sk-anthropic")
    );
}

/// `server/test/backend-driver-launch.test.mjs:83-96`.
#[test]
fn deepseek_isolates_its_acp_limitations() {
    let built = profile("deepseek", &config(), &EnvMap::new(), "native").expect("builds");
    assert!(!built.capabilities.external_mcp);
    assert!(!built.capabilities.session_mcp);
    assert!(!built.capabilities.delegation);
    assert!(!built.capabilities.native_session_history);
    assert_eq!(
        built.acp.spawn.env.get("DSH_PERMISSION_MODE"),
        Some(DSH_PERMISSION_WORKSPACE)
    );
    assert_eq!(
        built.acp.spawn.env.get("DSH_MODEL"),
        Some("deepseek-v4-pro")
    );
    assert_eq!(
        built.acp.spawn.env.get("DEEPSEEK_HARNESS_CONFIG"),
        Some("/opt/via/config/deepseek-harness/cordis.yml")
    );
    assert_eq!(
        built.acp.spawn.env.get("DEEPSEEK_HARNESS_SESSION_ROOT"),
        Some("/config/backends/deepseek-harness/sessions")
    );
    assert!(built.process_model_configuration);
    assert_eq!(
        built.acp.spawn.args,
        ["--config", "/opt/via/config/deepseek-harness/cordis.yml"]
    );

    let full = profile("deepseek", &config(), &EnvMap::new(), "full").expect("builds");
    assert_eq!(
        full.acp.spawn.env.get("DSH_PERMISSION_MODE"),
        Some(DSH_PERMISSION_FULL)
    );
}

#[test]
fn pi_names_the_pi_binary_for_its_adapter() {
    let mut config = config();
    config.backends.pi.cli_path = "/opt/pi-acp".to_owned();
    let finder = TableFinder::new(&[("pi", "/usr/bin/pi")]);
    let built = profile_with(
        "pi",
        &config,
        &EnvMap::new(),
        "native",
        Ownership::Owned,
        &finder,
    )
    .expect("builds");
    assert_eq!(built.acp.spawn.command, "/opt/pi-acp");
    assert_eq!(built.acp.spawn.env.get("PI_ACP_BIN"), Some("/opt/pi-acp"));
    assert_eq!(built.acp.spawn.env.get("PI_BIN"), Some("/usr/bin/pi"));
    assert_eq!(
        built.acp.spawn.env.get("PI_ACP_PI_COMMAND"),
        Some("/usr/bin/pi")
    );
}

#[test]
fn the_four_local_acp_backends_launch_their_own_command() {
    for (id, command, expected) in [
        ("qoder", "qodercli", vec!["--acp"]),
        ("qwen", "qwen", vec!["--acp"]),
        ("kimi", "kimi", vec!["acp"]),
        ("hermes", "hermes", vec!["acp", "--accept-hooks"]),
    ] {
        let built = profile(id, &config(), &EnvMap::new(), "native").expect("builds");
        assert_eq!(built.acp.spawn.command, command, "{id}");
        assert_eq!(built.acp.spawn.args, expected, "{id}");
    }
}

#[test]
fn full_permission_reaches_each_local_backend_the_way_it_expects() {
    // Qoder takes a flag; Kimi takes a session config option; Qwen and Hermes
    // take neither, and inventing one for them would be a fabricated contract.
    let qoder = profile("qoder", &config(), &EnvMap::new(), "full").expect("builds");
    assert_eq!(
        qoder.acp.spawn.args,
        ["--acp", "--dangerously-skip-permissions"]
    );

    let kimi = profile("kimi", &config(), &EnvMap::new(), "full").expect("builds");
    assert_eq!(kimi.acp.spawn.args, ["acp"]);
    assert_eq!(kimi.session_config_options.len(), 1);
    assert_eq!(kimi.session_config_options[0].id, "mode");
    assert_eq!(kimi.session_config_options[0].value, "auto");

    let kimi_native = profile("kimi", &config(), &EnvMap::new(), "native").expect("builds");
    assert!(kimi_native.session_config_options.is_empty());

    for id in ["qwen", "hermes"] {
        let built = profile(id, &config(), &EnvMap::new(), "full").expect("builds");
        assert!(built.session_config_options.is_empty(), "{id}");
    }
}

#[test]
fn codebuddy_puts_its_model_on_the_command_line_and_its_endpoint_in_the_environment() {
    let mut config = config();
    config.backends.codebuddy.model = "qwen3.7-max".to_owned();
    config.backends.codebuddy.model_url = "https://example.test/v1/chat/completions".to_owned();
    let built = profile("codebuddy", &config, &EnvMap::new(), "full").expect("builds");
    assert_eq!(built.acp.spawn.command, "codebuddy");
    assert_eq!(
        built.acp.spawn.args,
        [
            "--acp",
            "--model",
            "qwen3.7-max",
            "--dangerously-skip-permissions"
        ]
    );
    assert_eq!(
        built.acp.spawn.env.get("CODEBUDDY_MODEL_URL"),
        Some("https://example.test/v1/chat/completions")
    );
}

#[test]
fn the_generic_acp_backend_uses_the_command_and_label_it_was_given() {
    let mut config = config();
    config.backends.acp.cli_path = "/opt/my-agent".to_owned();
    config.backends.acp.args = vec!["--stdio".to_owned(), "--verbose".to_owned()];
    config.backends.acp.label = "My Agent".to_owned();
    let built = profile("acp", &config, &EnvMap::new(), "native").expect("builds");
    assert_eq!(built.acp.spawn.command, "/opt/my-agent");
    assert_eq!(built.acp.spawn.args, ["--stdio", "--verbose"]);
    assert_eq!(built.acp.label, "My Agent");

    // With no label it falls back to the catalog's.
    config.backends.acp.label = String::new();
    let built = profile("acp", &config, &EnvMap::new(), "native").expect("builds");
    assert_eq!(built.acp.label, "ACP Agent");
}

#[test]
fn registering_backends_skips_only_the_unconfigured_generic_entry_point() {
    let sessions = std::sync::Arc::new(via_acp::AcpSessionRegistry::builder().build());
    let config = config();
    let environment = EnvMap::new();
    let finder = via_backends::detect::MissingFinder;
    let registry = via_backends::register_backends(
        &via_backends::LaunchContext {
            config: &config,
            env: &environment,
            ownership: Ownership::Owned,
            permission_mode: "native",
            owner_id: "user_personal",
            finder: &finder,
        },
        &sessions,
    )
    .expect("every declaration validates");
    assert_eq!(registry.ids().len(), 11, "acp has no ACP_COMMAND");
    assert!(!registry.contains("acp"));
    for id in backend_ids() {
        if id == "acp" {
            continue;
        }
        assert!(registry.contains(id), "{id}");
    }
}

#[test]
fn an_openclaw_bridge_binary_prepares_its_token_file() {
    let config = openclaw_config("/opt/openclaw", "s3cret");
    let built = profile("openclaw", &config, &EnvMap::new(), "native").expect("builds");
    match built.prepare {
        Some(via_backends::PrepareAction::WritePrivateFile { path, contents }) => {
            assert_eq!(
                path,
                std::path::PathBuf::from("/config/backends/openclaw/state/gateway-token")
            );
            assert_eq!(contents, "s3cret\n", "the trailing newline is the format");
        }
        other => panic!("expected a token-file write, got {other:?}"),
    }

    // Without a direct bridge binary there is nothing to prepare: the bundled
    // launcher resolves the token itself.
    let mut without = config.clone();
    without.backends.openclaw.cli_path = String::new();
    let built = profile("openclaw", &without, &EnvMap::new(), "native").expect("builds");
    assert!(built.prepare.is_none());
}

#[test]
fn an_external_openclaw_gets_no_local_warm_up_notice() {
    let config = config();
    let owned = profile_with(
        "openclaw",
        &config,
        &EnvMap::new(),
        "native",
        Ownership::Owned,
        &via_backends::detect::MissingFinder,
    )
    .expect("builds");
    assert_eq!(
        owned.readiness_message.as_deref(),
        Some("the OpenClaw Gateway is starting"),
    );

    let external = profile_with(
        "openclaw",
        &config,
        &EnvMap::new(),
        "native",
        Ownership::External,
        &via_backends::detect::MissingFinder,
    )
    .expect("builds");
    assert!(
        external.readiness_message.is_none(),
        "an external Gateway's real errors must not be hidden behind a warm-up notice"
    );
}

#[test]
fn only_two_backends_are_started_as_a_service() {
    for id in backend_ids() {
        assert_eq!(
            spawns_separate_process(id),
            matches!(id, "opencode" | "openclaw"),
            "{id}"
        );
    }
}

#[test]
fn an_unknown_backend_is_refused_by_both_registries() {
    assert!(matches!(
        backend_driver("nope"),
        Err(BackendsError::UnsupportedBackend { .. })
    ));
    let runtime = runtime_registry(&EnvMap::new()).expect("validates");
    assert!(runtime.driver("nope").is_err());
    // `none` is the frontend-only sentinel and resolves to no backend at all.
    assert!(backend_driver("none").is_err());
    assert!(runtime.driver("none").is_err());
}
