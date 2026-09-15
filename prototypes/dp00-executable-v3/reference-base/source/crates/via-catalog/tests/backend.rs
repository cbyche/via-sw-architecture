//! Contract assertions for the backend agent catalog.
//!
//! Values from `shared/backend-catalog.mjs`. The environment allow-lists are a
//! security boundary and are asserted name by name.

use pretty_assertions::assert_eq;
use via_catalog::backend::{
    BACKEND_NONE_SENTINEL, CommandParser, ConfigurationMode, Integration, Ownership, ProbeKind,
    SkillsSpec, backend_definition, backend_definitions, backend_names,
    effective_backend_permission_mode, is_valid_skills_installer, normalize_backend_protocol,
    resolve_backend_ownership, skills_installer_agents,
};

/// `shared/backend-catalog.mjs:6-390`. The order is contract: it is the order
/// `backendNames().join('、')` prints inside error messages and the order
/// `--backend` documents.
#[test]
fn twelve_backend_ids_in_catalog_order() {
    assert_eq!(
        backend_names(),
        vec![
            "opencode",
            "openclaw",
            "qoder",
            "qwen",
            "kimi",
            "hermes",
            "codebuddy",
            "codex",
            "claude",
            "deepseek",
            "pi",
            "acp",
        ]
    );
}

/// `shared/backend-catalog.mjs:9,41,80,102,128,151,178,200,233,265,329,372`.
/// Rendered in CLI output and interpolated into error messages such as
/// `<label> <version> 低于最低版本 <min>`.
#[test]
fn backend_labels_in_catalog_order() {
    let labels: Vec<&str> = backend_definitions().iter().map(|d| d.label).collect();
    assert_eq!(
        labels,
        vec![
            "OpenCode",
            "OpenClaw",
            "Qoder",
            // KEEP: the third-party Qwen Code CLI, not VIA.
            "Qwen Code",
            "Kimi Code",
            "Hermes",
            "CodeBuddy",
            "Codex",
            "Claude Code",
            "DeepSeek",
            "Pi",
            "ACP Agent",
        ]
    );
}

/// `shared/backend-catalog.mjs:462-465`. `none` in any case means frontend-only.
#[test]
fn the_none_sentinel_normalizes_to_empty() {
    assert_eq!(BACKEND_NONE_SENTINEL, "none");
    assert_eq!(normalize_backend_protocol("none"), "");
    assert_eq!(normalize_backend_protocol("NONE"), "");
    assert_eq!(normalize_backend_protocol("  None  "), "");
    assert_eq!(normalize_backend_protocol(""), "");
    // Everything else is trimmed and lowercased but passed through; validity is
    // the lookup's call, not the normaliser's.
    assert_eq!(normalize_backend_protocol("  OpenCode "), "opencode");
    assert_eq!(normalize_backend_protocol("nonesuch"), "nonesuch");

    assert!(backend_definition("none").is_none());
    assert!(backend_definition("").is_none());
    assert!(backend_definition("nonesuch").is_none());
    assert_eq!(
        backend_definition("  QWEN ")
            .expect("case-insensitive lookup")
            .id,
        "qwen"
    );
}

/// **The credential-namespace security boundary.**
///
/// `shared/backend-catalog.mjs:30-388`, with only the identity renames
/// `docs/rebrand.md` mandates applied: `QWEN_AUDIO_AGENT_*` → `VIA_*` and
/// `QWAUDIO_CONFIG_DIR` → `VIA_CONFIG_DIR`. Every vendor-owned name is KEEP.
#[test]
fn environment_allow_policy_is_exact() {
    let policy = |id: &str| backend_definition(id).expect("catalog id").environment;

    let opencode = policy("opencode");
    assert_eq!(
        opencode.names,
        &[
            "DASHSCOPE_API_KEY",
            "VIA_OPENCODE_ISOLATE_USER_CONFIG",
            "VIA_OPENCODE_XDG_CONFIG_HOME",
        ]
    );
    assert_eq!(opencode.prefixes, &["OPENCODE_"]);
    assert_eq!(opencode.explicit_list_environment, None);

    let openclaw = policy("openclaw");
    assert_eq!(
        openclaw.names,
        &[
            "DASHSCOPE_API_KEY",
            "AGENT_API_KEY",
            "VIA_CONFIG_DIR",
            "VIA_OPENCLAW_MODEL",
            "VIA_OPENCLAW_MODEL_ID",
            "VIA_OPENCLAW_STATE_DIR",
            "VIA_OPENCLAW_WORKSPACE",
        ]
    );
    assert_eq!(openclaw.prefixes, &["OPENCLAW_"]);

    let qoder = policy("qoder");
    assert!(qoder.names.is_empty());
    assert_eq!(qoder.prefixes, &["QODER_", "QODERCLI_"]);

    let qwen = policy("qwen");
    assert_eq!(
        qwen.names,
        &["DASHSCOPE_API_KEY", "OPENAI_API_KEY", "OPENAI_BASE_URL"]
    );
    // KEEP: the Qwen Code CLI's own namespace.
    assert_eq!(qwen.prefixes, &["QWEN_CODE_"]);

    assert!(policy("kimi").names.is_empty());
    assert_eq!(policy("kimi").prefixes, &["KIMI_"]);

    assert!(policy("hermes").names.is_empty());
    assert_eq!(policy("hermes").prefixes, &["HERMES_"]);

    assert!(policy("codebuddy").names.is_empty());
    assert_eq!(policy("codebuddy").prefixes, &["CODEBUDDY_"]);

    let codex = policy("codex");
    assert_eq!(
        codex.names,
        &["DASHSCOPE_API_KEY", "OPENAI_API_KEY", "OPENAI_BASE_URL"]
    );
    assert_eq!(codex.prefixes, &["CODEX_"]);

    let claude = policy("claude");
    assert_eq!(
        claude.names,
        &["ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL", "CLAUDE_API_KEY"]
    );
    assert_eq!(claude.prefixes, &["CLAUDE_"]);

    let deepseek = policy("deepseek");
    assert_eq!(deepseek.names, &["DEEPSEEK_API_KEY"]);
    assert_eq!(deepseek.prefixes, &["DEEPSEEK_", "DSH_"]);

    let pi = policy("pi");
    assert_eq!(
        pi.names,
        &[
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_BASE_URL",
            "OPENAI_API_KEY",
            "OPENAI_BASE_URL",
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
        ]
    );
    assert_eq!(pi.prefixes, &["PI_"]);

    let acp = policy("acp");
    assert!(acp.names.is_empty());
    assert_eq!(acp.prefixes, &["ACP_"]);
    // Upstream QWEN_AUDIO_AGENT_ACP_FORWARD_ENV. The generic ACP backend has no
    // vendor namespace, so the user names the variables explicitly.
    assert_eq!(acp.explicit_list_environment, Some("VIA_ACP_FORWARD_ENV"));
    assert!(
        backend_definitions()
            .iter()
            .filter(|d| d.environment.explicit_list_environment.is_some())
            .count()
            == 1,
        "only the generic acp backend takes an explicit forward list"
    );
}

/// The boundary's *behaviour*: names match exactly, prefixes match literally, and
/// no gateway secret crosses into any backend.
#[test]
fn gateway_secrets_reach_no_backend() {
    let openclaw = backend_definition("openclaw")
        .expect("catalog id")
        .environment;
    assert!(openclaw.allows("OPENCLAW_GATEWAY_TOKEN"));
    assert!(openclaw.allows("DASHSCOPE_API_KEY"));
    assert!(openclaw.allows("VIA_OPENCLAW_STATE_DIR"));
    // Not a prefix match, not an exact match.
    assert!(!openclaw.allows("OPENCLAW"));
    assert!(
        !openclaw.allows("openclaw_bin"),
        "matching is case-sensitive"
    );
    assert!(!openclaw.allows("ANTHROPIC_API_KEY"));

    // The Gateway's own secrets, which upstream keeps out of every child.
    for secret in [
        "VIA_AUTH_SECRET",
        "VIA_REALTIME_API_KEY",
        "VIA_MEMORY_API_KEY",
        "SPEECH_TO_SPEECH_AUTH_TOKEN",
        "S2S_API_KEY",
    ] {
        for definition in backend_definitions() {
            assert!(
                !definition.environment.allows(secret),
                "{} forwards {secret} to a backend process",
                definition.id
            );
        }
    }
}

/// `shared/backend-catalog.mjs:27-28,58-63`. Where the gateway connects when the
/// backend is externally managed.
#[test]
fn base_urls_and_external_service_credentials() {
    let opencode = backend_definition("opencode").expect("catalog id");
    assert_eq!(opencode.base_url_environment, Some("OPENCODE_BASE_URL"));
    assert_eq!(opencode.default_base_url, Some("http://127.0.0.1:4096"));
    assert!(!opencode.supports_external_service);

    let openclaw = backend_definition("openclaw").expect("catalog id");
    assert_eq!(openclaw.base_url_environment, Some("OPENCLAW_BASE_URL"));
    assert_eq!(openclaw.default_base_url, Some("http://127.0.0.1:18789"));
    assert!(openclaw.supports_external_service);
    assert_eq!(
        openclaw
            .external_service
            .expect("openclaw is reachable externally")
            .credential_environment,
        "OPENCLAW_GATEWAY_TOKEN"
    );

    // Only these two declare a base URL at all.
    let with_base_urls: Vec<&str> = backend_definitions()
        .iter()
        .filter(|d| d.default_base_url.is_some())
        .map(|d| d.id)
        .collect();
    assert_eq!(with_base_urls, vec!["opencode", "openclaw"]);
}

/// `shared/backend-catalog.mjs:10,42,81,103,129,152,179,201,234,266,330,373`.
#[test]
fn workspace_override_variables() {
    let workspaces: Vec<&str> = backend_definitions()
        .iter()
        .map(|d| d.workspace_environment)
        .collect();
    assert_eq!(
        workspaces,
        vec![
            "OPENCODE_WORKSPACE",
            // Upstream QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE.
            "VIA_OPENCLAW_WORKSPACE",
            "QODER_WORKSPACE",
            // KEEP: the Qwen Code CLI's own variable.
            "QWEN_CODE_WORKSPACE",
            "KIMI_WORKSPACE",
            "HERMES_WORKSPACE",
            "CODEBUDDY_WORKSPACE",
            "CODEX_WORKSPACE",
            "CLAUDE_WORKSPACE",
            "DEEPSEEK_HARNESS_WORKSPACE",
            "PI_WORKSPACE",
            "ACP_WORKSPACE",
        ]
    );
}

/// `shared/backend-catalog.mjs:16,109,136,339` — only four backends gate on a
/// minimum version.
#[test]
fn minimum_versions() {
    let minimums: Vec<(&str, Option<&str>)> = backend_definitions()
        .iter()
        .map(|d| (d.id, d.setup.minimum_version))
        .filter(|(_, v)| v.is_some())
        .collect();
    assert_eq!(
        minimums,
        vec![
            ("opencode", Some("1.18.0")),
            ("qwen", Some("0.21.6")),
            ("kimi", Some("0.31.0")),
            ("pi", Some("0.80.4")),
        ]
    );
}

/// `shared/backend-catalog.mjs:15,20,47,54,90,206,378,382`. Consumed as
/// discriminants by CLI rendering and desktop settings.
#[test]
fn integration_and_configuration_modes() {
    let integration = |id: &str| {
        backend_definition(id)
            .expect("catalog id")
            .setup
            .integration
    };
    assert_eq!(integration("opencode"), Integration::Native);
    assert_eq!(integration("openclaw"), Integration::Bridge);
    assert_eq!(integration("codex"), Integration::Adapter);
    assert_eq!(integration("claude"), Integration::Adapter);
    assert_eq!(integration("pi"), Integration::Adapter);
    // DeepSeek declares an adapter command but integrates natively.
    assert_eq!(integration("deepseek"), Integration::Native);
    assert_eq!(integration("acp"), Integration::Generic);

    let configuration = |id: &str| {
        backend_definition(id)
            .expect("catalog id")
            .lifecycle
            .configuration
    };
    assert_eq!(
        configuration("opencode"),
        ConfigurationMode::BailianOrBackendOwned
    );
    assert_eq!(
        configuration("openclaw"),
        ConfigurationMode::BailianOrBackendOwned
    );
    assert_eq!(configuration("qwen"), ConfigurationMode::BackendOwned);
    assert_eq!(configuration("acp"), ConfigurationMode::UserManaged);

    assert_eq!(
        serde_json::to_string(&ConfigurationMode::BailianOrBackendOwned).expect("mode serializes"),
        r#""bailian-or-backend-owned""#
    );
    assert_eq!(
        serde_json::to_string(&Integration::Generic).expect("mode serializes"),
        r#""generic""#
    );
}

/// `shared/backend-catalog.mjs`. `backend-auth-status.mjs` dispatches on the
/// probe kind, so the two must agree or a backend silently reports `unknown`.
#[test]
fn auth_probe_kinds_and_parsers() {
    let probe = |id: &str| {
        backend_definition(id)
            .expect("catalog id")
            .onboarding
            .and_then(|o| o.probe)
    };

    let opencode = probe("opencode").expect("opencode probes");
    assert_eq!(opencode.kind, ProbeKind::Command);
    assert_eq!(opencode.args, &["auth", "list"]);
    assert_eq!(opencode.parser, Some(CommandParser::CredentialCount));

    let qoder = probe("qoder").expect("qoder probes");
    assert_eq!(qoder.args, &["status"]);
    assert_eq!(qoder.parser, Some(CommandParser::QoderStatus));

    let codex = probe("codex").expect("codex probes");
    assert_eq!(codex.args, &["login", "status"]);
    assert_eq!(codex.parser, Some(CommandParser::CodexStatus));

    assert_eq!(
        probe("qwen").expect("qwen probes").kind,
        ProbeKind::QwenSettings
    );
    assert_eq!(probe("pi").expect("pi probes").kind, ProbeKind::PiAuthCheck);
    assert_eq!(
        probe("deepseek").expect("deepseek probes").kind,
        ProbeKind::DeepseekCredentials
    );
    assert_eq!(
        probe("codebuddy").expect("codebuddy probes").kind,
        ProbeKind::CodebuddyCredentials
    );
    assert_eq!(
        probe("openclaw").expect("openclaw probes").kind,
        ProbeKind::OpenclawState
    );

    // Kimi, Hermes and Claude declare onboarding with no probe; acp declares no
    // onboarding at all.
    assert!(probe("kimi").is_none());
    assert!(probe("hermes").is_none());
    assert!(probe("claude").is_none());
    assert!(
        backend_definition("acp")
            .expect("catalog id")
            .onboarding
            .is_none()
    );
}

/// `shared/backend-setup.mjs:329,337,448`: the two adapter behaviours DeepSeek
/// alone declares. Absence of `managedAdapterFallback` means the fallback is
/// enabled, because upstream tests it with `=== false`.
#[test]
fn deepseek_is_the_only_backend_that_opts_out_of_the_adapter_fallback() {
    for definition in backend_definitions() {
        let expected_deepseek = definition.id == "deepseek";
        assert_eq!(
            definition.setup.managed_adapter_fallback, !expected_deepseek,
            "{} managed_adapter_fallback",
            definition.id
        );
        assert_eq!(
            definition.setup.inspect_adapter_independently, expected_deepseek,
            "{} inspect_adapter_independently",
            definition.id
        );
    }
    let deepseek = backend_definition("deepseek").expect("catalog id");
    assert!(
        deepseek
            .lifecycle
            .installation
            .expect("deepseek installs")
            .verify_installed_packages
    );
}

/// `acp` is the one backend VIA never installs — upstream models it as
/// `installation: null`. The step lists themselves are deferred to
/// `via-backends`; the `None`/`Some` distinction is not.
#[test]
fn only_the_generic_acp_backend_has_no_installation() {
    let without: Vec<&str> = backend_definitions()
        .iter()
        .filter(|d| d.lifecycle.installation.is_none())
        .map(|d| d.id)
        .collect();
    assert_eq!(without, vec!["acp"]);

    // Phase 0 deferral: the shape is present, the pinned coordinates are not.
    for definition in backend_definitions() {
        if let Some(installation) = definition.lifecycle.installation {
            assert!(
                installation.steps.is_empty(),
                "{} carries install steps; they belong to via-backends",
                definition.id
            );
        }
    }
}

/// `shared/backend-catalog.mjs:398-433`. The declaration is mandatory upstream
/// and unrepresentable-as-missing here; the id charset guards against an
/// installer name smuggling extra flags into a command line.
#[test]
fn skills_declarations() {
    assert_eq!(
        skills_installer_agents(),
        vec![
            "opencode",
            "openclaw",
            "qoder",
            // KEEP: the skills.sh agent id for the Qwen Code CLI.
            "qwen-code",
            "kimi-code-cli",
            "hermes-agent",
            "codebuddy",
            "codex",
            "claude-code",
            "pi",
        ]
    );
    assert_eq!(
        backend_definition("deepseek").expect("catalog id").skills,
        SkillsSpec::NoInstaller
    );
    assert_eq!(
        backend_definition("acp").expect("catalog id").skills,
        SkillsSpec::NoConvention
    );

    for agent in skills_installer_agents() {
        assert!(
            is_valid_skills_installer(agent),
            "{agent} is not kebab-case"
        );
    }
    assert!(!is_valid_skills_installer(""));
    assert!(!is_valid_skills_installer("-leading"));
    assert!(!is_valid_skills_installer("trailing-"));
    assert!(!is_valid_skills_installer("double--dash"));
    assert!(!is_valid_skills_installer("Upper"));
    assert!(!is_valid_skills_installer("with space"));
    assert!(!is_valid_skills_installer("flag --force"));
}

/// `test/backend-catalog.test.mjs:8-28`. Pi executes commands and edits files
/// with no approval gate in any configuration, so the declaration is a safety
/// disclosure and the effective mode ignores what the user configured.
#[test]
fn pi_is_always_full_permission() {
    let pi = backend_definition("pi").expect("catalog id");
    assert!(pi.always_full_permission);
    assert!(pi.supports_full_permission);

    assert_eq!(effective_backend_permission_mode("pi", "native"), "full");
    assert_eq!(effective_backend_permission_mode("pi", "full"), "full");
    assert_eq!(effective_backend_permission_mode("pi", ""), "full");

    // Every other backend passes the configured mode through.
    assert_eq!(
        effective_backend_permission_mode("codex", "native"),
        "native"
    );
    assert_eq!(effective_backend_permission_mode("codex", "full"), "full");
    assert_eq!(effective_backend_permission_mode("codex", ""), "native");
    assert_eq!(
        effective_backend_permission_mode("openclaw", "NATIVE"),
        "native"
    );
    // No backend selected: the default still applies.
    assert_eq!(effective_backend_permission_mode("", "full"), "full");
    assert_eq!(effective_backend_permission_mode("", ""), "native");

    // Pi is the only one.
    let always_full: Vec<&str> = backend_definitions()
        .iter()
        .filter(|d| d.always_full_permission)
        .map(|d| d.id)
        .collect();
    assert_eq!(always_full, vec!["pi"]);
}

/// `shared/backend-catalog.mjs:443-460`.
#[test]
fn ownership_resolution() {
    // OpenClaw is the only backend that may be an external service, and a
    // configured base URL flips it there by default.
    assert_eq!(
        resolve_backend_ownership("openclaw", true, "").expect("resolves"),
        Ownership::External
    );
    assert_eq!(
        resolve_backend_ownership("openclaw", false, "").expect("resolves"),
        Ownership::Owned
    );
    // An explicit request wins over the base-URL heuristic.
    assert_eq!(
        resolve_backend_ownership("openclaw", true, "owned").expect("resolves"),
        Ownership::Owned
    );
    assert_eq!(
        resolve_backend_ownership("opencode", true, "").expect("resolves"),
        Ownership::Owned
    );

    // A backend the gateway must launch cannot be declared external.
    let error =
        resolve_backend_ownership("opencode", true, "external").expect_err("must be refused");
    assert_eq!(error.code(), "VIA_BACKEND_EXTERNAL_SERVICE_UNSUPPORTED");
    assert_eq!(
        error,
        via_catalog::CatalogError::ExternalServiceUnsupported { label: "OpenCode" }
    );

    let error = resolve_backend_ownership("openclaw", false, "shared").expect_err("bad value");
    assert_eq!(error.code(), "VIA_BACKEND_OWNERSHIP_UNSUPPORTED");

    let error = resolve_backend_ownership("nonesuch", false, "").expect_err("unknown backend");
    assert_eq!(error.code(), "VIA_BACKEND_UNSUPPORTED");

    assert_eq!(Ownership::Owned.as_str(), "owned");
    assert_eq!(Ownership::External.as_str(), "external");
}

/// The commands and executable-path overrides. Qoder is the one backend that
/// declares two path variables, in priority order.
#[test]
fn setup_commands_and_executable_overrides() {
    let setup = |id: &str| backend_definition(id).expect("catalog id").setup;

    assert_eq!(setup("opencode").command, Some("opencode"));
    assert_eq!(setup("qoder").command, Some("qodercli"));
    assert_eq!(
        setup("qoder").executable_environment,
        &["QODERCLI_PATH", "QODER_CLI_PATH"]
    );
    // KEEP: Qwen Code's own binary and variable.
    assert_eq!(setup("qwen").command, Some("qwen"));
    assert_eq!(setup("qwen").executable_environment, &["QWEN_CODE_BIN"]);
    assert_eq!(setup("deepseek").command, Some("dsh"));
    assert_eq!(
        setup("claude").executable_environment,
        &["CLAUDE_CODE_EXECUTABLE"]
    );

    assert_eq!(setup("codex").adapter_command, Some("codex-acp"));
    assert_eq!(setup("claude").adapter_command, Some("claude-code-acp"));
    assert_eq!(setup("pi").adapter_command, Some("pi-acp"));
    assert_eq!(setup("deepseek").adapter_command, Some("dsh-acp-demo"));

    // The generic ACP backend is named entirely by configuration.
    assert_eq!(setup("acp").command, None);
    assert_eq!(setup("acp").command_environment, Some("ACP_COMMAND"));
    assert!(setup("acp").executable_environment.is_empty());
}
