//! Every value in this crate that `docs/reference/contracts.json` pins.
//!
//! The catalogue is **parsed**, never retyped: a test that restated the
//! expected value would assert that this crate agrees with itself.
//! `docs/architecture.md` §14 names `reference/contracts.md` the acceptance
//! criteria, and this file is `via-backends`'s slice of it.
//!
//! # Reading a contract that carries an upstream identity
//!
//! Several `exactValue`s contain `qwen-audio-agent` or a `QWEN_AUDIO_AGENT_*`
//! variable that `docs/rebrand.md` renames. Those assertions apply
//! [`rebranded`] to the catalogued text rather than hard-coding the VIA
//! spelling, so a change to *either* document still fails the test.

mod support;

use std::sync::OnceLock;

use support::{config, env, profile};
use via_backends::driver::{CODEBUDDY_MODELS_JSON_TEMPLATE, DEEPSEEK_HARNESS_CORDIS_YAML};
use via_backends::install::{
    DEEPSEEK_REGISTRY, HERMES_INSTALL_COMMAND, HERMES_INSTALL_COMMAND_WINDOWS, InstallError,
    MAX_INSTALL_OUTPUT_CHARS, NPM_CONFIG_YES, POSIX_SHELL, ProgressPhase, WINDOWS_SHELL,
    WINDOWS_SHELL_FLAGS, install_steps, step_package,
};
use via_backends::openclaw::{
    DEFAULT_GATEWAY_PORT, OPENCLAW_CONFIG_JSON5, PROMPT_RETRY_DELAYS_MS, managed_gateway_arguments,
};
use via_backends::profile::{
    DEEPSEEK_SESSION_INSTRUCTIONS, OPENCLAW_SESSION_INSTRUCTIONS, PI_SESSION_INSTRUCTIONS,
};
use via_backends::{AuthStatus, HostPlatform, backend_ids};
use via_catalog::{backend_definition, backend_names};
use via_core::EnvMap;

/// One catalogued contract.
#[derive(Debug, serde::Deserialize)]
struct Contract {
    kind: String,
    name: String,
    #[serde(rename = "exactValue")]
    exact_value: String,
    /// Why the value is a contract. Several of these carry the *reason* a
    /// boundary exists, which is worth asserting: the reason is what a future
    /// change would have to argue with.
    why: String,
}

fn contracts() -> &'static [Contract] {
    static CONTRACTS: OnceLock<Vec<Contract>> = OnceLock::new();
    CONTRACTS.get_or_init(|| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/reference/contracts.json");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        serde_json::from_str(&text).expect("contracts.json is a JSON array of contracts")
    })
}

/// One catalogued contract, by kind and name.
fn entry(kind: &str, name: &str) -> &'static Contract {
    contracts()
        .iter()
        .find(|entry| entry.kind == kind && entry.name == name)
        .unwrap_or_else(|| panic!("no catalogued contract {kind}/{name}"))
}

/// The `exactValue` of one contract, by kind and name.
fn contract(kind: &str, name: &str) -> &'static str {
    entry(kind, name).exact_value.as_str()
}

/// Apply the identity renames `docs/rebrand.md` mandates to catalogued text.
///
/// Only the product identity moves. Vendor names, third-party variables and
/// protocol strings are KEEP and are deliberately absent from this list.
fn rebranded(value: &str) -> String {
    value
        .replace("QWEN_AUDIO_AGENT_", "VIA_")
        .replace("QWAUDIO_", "VIA_")
        .replace("qwen-audio-agent-backend", "via-backend")
        .replace("qwen-audio-agent", "via")
}

// ── identity ────────────────────────────────────────────────────────────────

#[test]
fn the_twelve_ids_are_in_catalog_order() {
    let catalogued = contract("state-name", "backend ids (all 12, in catalog order)");
    let expected: Vec<&str> = catalogued
        .split(" - plus")
        .next()
        .expect("the id list")
        .split(',')
        .map(str::trim)
        .collect();
    assert_eq!(expected.len(), 12);
    assert_eq!(backend_ids(), expected);
    assert_eq!(backend_names(), expected);
}

#[test]
fn every_driver_declares_the_catalogued_label() {
    let expected: Vec<&str> = contract("state-name", "backend labels")
        .split(',')
        .map(str::trim)
        .collect();
    let actual: Vec<&str> = via_backends::backend_drivers()
        .iter()
        .map(|driver| driver.label)
        .collect();
    assert_eq!(actual, expected);
}

// ── install ─────────────────────────────────────────────────────────────────

#[test]
fn every_pinned_install_coordinate_is_the_catalogued_one() {
    let catalogued = contract(
        "default-value",
        "backend pinned install packages and scripts",
    );
    let environment = EnvMap::new();
    for id in backend_names() {
        for platform in HostPlatform::ALL {
            for step in install_steps(id, platform).steps {
                let value = if step.package.is_empty() {
                    step.command.to_owned()
                } else {
                    step_package(step, &environment)
                };
                // DeepSeek's ten Developer Preview components are catalogued as
                // a family (`plus 10 @deepseek-ai/dsh-* @0.1.0-rc.6 components`)
                // rather than enumerated; they are asserted structurally in
                // `deepseeks_ten_components_match_the_catalogued_family`.
                if value.starts_with("@deepseek-ai/dsh-") {
                    continue;
                }
                assert!(
                    catalogued.contains(&value),
                    "{id}/{platform:?}: `{value}` is not in the catalogued spec"
                );
            }
        }
    }
    // And the two halves that are easy to drop.
    assert!(catalogued.contains(DEEPSEEK_REGISTRY));
    assert!(catalogued.contains(HERMES_INSTALL_COMMAND));
    assert!(catalogued.contains(HERMES_INSTALL_COMMAND_WINDOWS));
}

#[test]
fn deepseeks_ten_components_match_the_catalogued_family() {
    let catalogued = contract(
        "default-value",
        "backend pinned install packages and scripts",
    );
    assert!(catalogued.contains("plus 10 @deepseek-ai/dsh-* @0.1.0-rc.6 components"));
    let steps = install_steps("deepseek", HostPlatform::Linux).steps;
    let components: Vec<&str> = steps
        .iter()
        .map(|step| step.package)
        .filter(|package| package.starts_with("@deepseek-ai/dsh-"))
        .collect();
    assert_eq!(components.len(), 10, "{components:?}");
    for package in &components {
        assert!(package.ends_with("@0.1.0-rc.6"), "{package}");
    }
    // Every step, component or not, is pulled from the explicit registry.
    for step in &steps {
        assert_eq!(step.registry, Some(DEEPSEEK_REGISTRY), "{}", step.package);
    }
}

#[test]
fn deepseeks_acp_package_is_last_as_the_catalogue_requires() {
    let why = &entry(
        "default-value",
        "backend pinned install packages and scripts",
    )
    .exact_value;
    assert!(why.contains("ACP package LAST"), "{why}");
    let steps = install_steps("deepseek", HostPlatform::Linux).steps;
    let last = steps.last().expect("eleven steps");
    assert_eq!(last.package, "@deepseek-ai/dsh-acp-demo@0.1.0-rc.6");
}

#[test]
fn the_installer_error_codes_are_the_catalogued_closed_set() {
    let expected: Vec<&str> = contract("error-code", "installBackend error codes (closed set)")
        .split('|')
        .map(str::trim)
        .collect();
    let actual = [
        InstallError::Unsupported {
            reason: String::new(),
        },
        InstallError::NpmMissing,
        InstallError::Declined,
        InstallError::Cancelled,
        InstallError::StepTimeout {
            display: String::new(),
        },
        InstallError::StepFailed {
            display: String::new(),
            exit_code: 1,
            cause: String::new(),
        },
        InstallError::VerifyFailed {
            detail: String::new(),
        },
    ];
    let codes: Vec<&str> = actual.iter().map(InstallError::code).collect();
    for code in &expected {
        assert!(
            codes.contains(code),
            "{code} is not produced by any variant"
        );
    }
    assert_eq!(codes.len(), expected.len(), "the set is closed");
}

#[test]
fn the_install_execution_parameters_are_the_catalogued_ones() {
    let catalogued = contract(
        "default-value",
        "install execution parameters and progress phases",
    );
    assert!(catalogued.contains("DEFAULT_STEP_TIMEOUT_MS = 600000"));
    assert_eq!(
        via_backends::install::DEFAULT_STEP_TIMEOUT.as_millis(),
        600_000
    );
    assert!(catalogued.contains("MAX_INSTALL_OUTPUT_CHARS = 65536"));
    assert_eq!(MAX_INSTALL_OUTPUT_CHARS, 65_536);
    assert!(catalogued.contains("npm_config_yes='true'"));
    assert_eq!(NPM_CONFIG_YES, ("npm_config_yes", "true"));
    assert!(catalogued.contains("powershell.exe -ExecutionPolicy Bypass -Command"));
    assert_eq!(WINDOWS_SHELL, "powershell.exe");
    assert_eq!(
        WINDOWS_SHELL_FLAGS,
        ["-ExecutionPolicy", "Bypass", "-Command"]
    );
    assert!(catalogued.contains("/bin/sh -c"));
    assert_eq!(POSIX_SHELL, "/bin/sh");
    for phase in ["skip", "start", "output", "done"] {
        assert!(catalogued.contains(&std::format!("'{phase}'")), "{phase}");
    }
    assert_eq!(
        serde_json::to_string(&ProgressPhase::Skip).expect("serializes"),
        "\"skip\""
    );
}

#[test]
fn the_minimum_versions_are_the_catalogued_ones() {
    let catalogued = contract("default-value", "backend minimum versions");
    for id in backend_names() {
        let definition = backend_definition(id).expect("catalogued");
        match definition.setup.minimum_version {
            Some(version) => assert!(
                catalogued.contains(&std::format!("{id} {version}")),
                "{id} {version}"
            ),
            None => assert!(
                !catalogued.contains(&std::format!("{id} ")),
                "{id} declares no minimum but the catalogue names one"
            ),
        }
    }
}

// ── onboarding and authentication ───────────────────────────────────────────

#[test]
fn the_three_authentication_statuses_are_the_catalogued_ones() {
    let catalogued = contract("state-name", "authentication status values");
    for status in [
        AuthStatus::Authenticated,
        AuthStatus::Unauthenticated,
        AuthStatus::Unknown,
    ] {
        assert!(
            catalogued.contains(&std::format!("'{}'", status.as_str())),
            "{}",
            status.as_str()
        );
    }
    assert!(
        catalogued.contains("inconclusive probes MUST return 'unknown'"),
        "the rule this module exists to keep"
    );
}

#[test]
fn every_catalogued_probe_kind_is_declared_by_a_backend() {
    let catalogued = contract("state-name", "backend auth probe kinds");
    for kind in [
        "qwen-settings",
        "pi-auth-check",
        "deepseek-credentials",
        "codebuddy-credentials",
        "openclaw-state",
    ] {
        assert!(catalogued.contains(kind), "{kind}");
        assert!(
            backend_names().iter().any(|id| {
                backend_definition(id)
                    .and_then(|definition| definition.onboarding)
                    .and_then(|onboarding| onboarding.probe)
                    .map(|probe| serde_json::to_value(probe.kind).unwrap_or_default())
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .is_some_and(|value| value == kind)
            }),
            "{kind} is declared by no backend"
        );
    }
    for parser in ["credential-count", "qoder-status", "codex-status"] {
        assert!(catalogued.contains(parser), "{parser}");
    }
}

#[test]
fn the_onboarding_state_machine_is_the_catalogued_one() {
    let catalogued = contract("state-name", "onboarding state machine values");
    for state in ["not-installed", "installed", "configuration-required"] {
        assert!(catalogued.contains(state), "{state}");
    }
    assert!(
        catalogued.contains("readiness.status: always 'not-connected'"),
        "readiness is never inferred from local files"
    );
    let resolved = via_backends::resolve_backend_onboarding(true, true);
    assert_eq!(
        serde_json::to_value(resolved.readiness).expect("serializes"),
        serde_json::json!("not-connected")
    );
}

// ── launch ──────────────────────────────────────────────────────────────────

#[test]
fn the_per_driver_launch_injections_are_the_catalogued_ones() {
    let catalogued = contract("env-var", "per-driver launch env injections");
    assert!(catalogued.contains("DSH_MODEL (default 'deepseek-v4-pro')"));
    assert!(catalogued.contains("'danger-full-access'|'workspace-write'"));

    let config = config();
    let native = profile("deepseek", &config, &EnvMap::new(), "native").expect("builds");
    assert_eq!(
        native.acp.spawn.env.get("DSH_MODEL"),
        Some("deepseek-v4-pro")
    );
    assert_eq!(
        native.acp.spawn.env.get("DSH_PERMISSION_MODE"),
        Some("workspace-write")
    );
    let full = profile("deepseek", &config, &EnvMap::new(), "full").expect("builds");
    assert_eq!(
        full.acp.spawn.env.get("DSH_PERMISSION_MODE"),
        Some("danger-full-access")
    );
}

#[test]
fn the_deepseek_harness_config_asset_is_where_the_catalogue_says() {
    let catalogued = contract("file-path", "deepseek harness config asset");
    assert_eq!(catalogued, "config/deepseek-harness/cordis.yml");
    assert_eq!(
        via_backends::driver::DEEPSEEK_HARNESS_CONFIG_PATH,
        catalogued
    );
    let built = profile("deepseek", &config(), &EnvMap::new(), "native").expect("builds");
    assert_eq!(
        built.acp.spawn.env.get("DEEPSEEK_HARNESS_CONFIG"),
        Some(std::format!("{}/{catalogued}", support::ROOT).as_str())
    );
}

// ── DeepSeek Harness (cordis.yml) ───────────────────────────────────────────
//
// The composition is YAML with `!!js "…"` tags this crate never parses (a
// Rust YAML parser rejects them outright — they are JavaScript the harness
// process itself evaluates), so these compare literal substrings rather than
// a deserialized document, the same shape `the_shipped_openclaw_configuration_
// carries_every_catalogued_field` uses for OpenClaw's JSON5.

#[test]
fn the_deepseek_default_model_matches_the_shipped_cordis_asset() {
    // `DEEPSEEK_DEFAULT_MODEL` duplicates the harness's own default; this
    // derives the check from the shipped asset so the two cannot drift apart.
    assert!(
        DEEPSEEK_HARNESS_CORDIS_YAML.contains(&std::format!(
            "?? '{}'\"",
            via_backends::driver::DEEPSEEK_DEFAULT_MODEL
        )),
        "DEEPSEEK_DEFAULT_MODEL no longer matches cordis.yml's own DSH_MODEL default"
    );
}

#[test]
fn the_cordis_acp_agent_config_carries_every_catalogued_field() {
    let catalogued = contract("json-field", "cordis acp-agent config");
    assert!(catalogued.contains("provider: deepseek-official"));
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains("provider: deepseek-official"));
    assert!(catalogued.contains("DSH_MODEL"));
    assert!(
        DEEPSEEK_HARNESS_CORDIS_YAML
            .contains(r#"model: !!js "process.env.DSH_MODEL ?? 'deepseek-v4-pro'""#)
    );
    assert!(catalogued.contains("DEEPSEEK_HARNESS_SESSION_ROOT"));
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains(
        r#"persistenceRoot: !!js "process.env.DEEPSEEK_HARNESS_SESSION_ROOT ?? './.sessions'""#
    ));
    // `workspaceContext.maxBytes: 65536` summarizes nested YAML with a dot the
    // asset does not spell out as one line; check the parent key and the
    // assignment separately.
    assert!(catalogued.contains("workspaceContext.maxBytes: 65536"));
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains("workspaceContext:"));
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains("maxBytes: 65536"));
}

#[test]
fn the_cordis_llm_deepseek_config_carries_every_catalogued_field() {
    let catalogued = contract("json-field", "cordis llm-deepseek config");
    for literal in [
        "id: llm-deepseek",
        "name: '@deepseek-ai/dsh-llm-deepseek'",
        "thinking: enabled",
        "reasoningEffort: max",
        "id: deepseek-v4-flash",
        "id: deepseek-v4-pro",
    ] {
        assert!(
            catalogued.contains(literal),
            "cordis llm-deepseek config: `{literal}` is missing from the catalogue"
        );
        assert!(
            DEEPSEEK_HARNESS_CORDIS_YAML.contains(literal),
            "cordis llm-deepseek config: `{literal}` is missing from the shipped asset"
        );
    }
}

#[test]
fn the_cordis_compaction_basic_config_carries_every_catalogued_field() {
    let catalogued = contract("json-field", "cordis compaction-basic config");
    for field in catalogued.split(", ").map(str::trim) {
        assert!(
            DEEPSEEK_HARNESS_CORDIS_YAML.contains(field),
            "cordis compaction-basic config: `{field}` is missing from the shipped asset"
        );
    }
}

#[test]
fn the_cordis_plugin_id_list_matches_the_catalogued_order() {
    let catalogued = contract("json-field", "cordis plugin id list (order matters)");
    let ids: Vec<&str> = catalogued.split(", ").map(str::trim).collect();
    assert_eq!(ids.len(), 12, "the catalogue names all twelve plugins");
    let mut cursor = 0usize;
    for id in ids {
        let needle = std::format!("id: {id}\n");
        let relative = DEEPSEEK_HARNESS_CORDIS_YAML[cursor..]
            .find(&needle)
            .unwrap_or_else(|| {
                panic!("`{needle:?}` missing, or out of order, after byte {cursor}")
            });
        cursor += relative + needle.len();
    }
}

#[test]
fn the_cordis_sandbox_and_approval_policy_expressions_are_the_catalogued_ones() {
    let catalogued = contract("json-field", "cordis sandbox/approval policy expressions");
    assert!(catalogued.contains("DSH_PERMISSION_MODE"));
    assert!(
        DEEPSEEK_HARNESS_CORDIS_YAML
            .contains(r#"mode: !!js "process.env.DSH_PERMISSION_MODE ?? 'workspace-write'""#)
    );
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains(
        r#"policy: !!js "process.env.DSH_PERMISSION_MODE === 'danger-full-access' ? 'never' : 'ask'""#
    ));
    assert!(catalogued.contains("timeoutMs: 60000"));
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains("timeoutMs: 60000"));
    assert!(catalogued.contains("process.cwd()"));
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains("workspaceRoot: !!js process.cwd()"));
    assert!(DEEPSEEK_HARNESS_CORDIS_YAML.contains("cwd: !!js process.cwd()"));
}

/// Pull the YAML block-literal value of `acp-agent`'s `persona:` field —
/// the one field this suite compares byte for byte, because it is
/// model-visible prompt text rather than harness configuration.
fn cordis_persona() -> String {
    let marker = "persona: |\n";
    let start = DEEPSEEK_HARNESS_CORDIS_YAML
        .find(marker)
        .expect("the acp-agent block has a persona")
        + marker.len();
    const INDENT: &str = "      ";
    let mut lines = Vec::new();
    for line in DEEPSEEK_HARNESS_CORDIS_YAML[start..].lines() {
        if let Some(stripped) = line.strip_prefix(INDENT) {
            lines.push(stripped);
        } else if line.trim().is_empty() {
            lines.push("");
        } else {
            break;
        }
    }
    while lines.last() == Some(&"") {
        lines.pop();
    }
    let mut rendered = lines.join("\n");
    rendered.push('\n');
    rendered
}

#[test]
fn the_cordis_persona_is_byte_for_byte_the_catalogued_prompt() {
    assert_eq!(
        contract(
            "prompt-text",
            "cordis acp-agent persona (model-visible, backend layer)"
        ),
        cordis_persona()
    );
}

// ── CodeBuddy (models.json) ─────────────────────────────────────────────────

#[test]
fn the_shipped_codebuddy_models_json_is_byte_for_byte_the_catalogued_template() {
    let catalogued = contract("json-field", "CodeBuddy models.json full template");
    // The catalogue records the logical JSON text; the on-disk file (like
    // upstream's) carries one trailing newline on top of it, and the `why`
    // field calls that newline load-bearing.
    assert_eq!(
        CODEBUDDY_MODELS_JSON_TEMPLATE.trim_end_matches('\n'),
        catalogued
    );
    assert!(
        CODEBUDDY_MODELS_JSON_TEMPLATE.ends_with('\n'),
        "the trailing newline is load-bearing (contracts.json's `why` field)"
    );
    // The two `${…}` placeholders are expanded by CodeBuddy, not VIA; assert
    // them against the environment-variable names VIA actually sets, so a
    // rename on either side cannot drift silently.
    assert!(CODEBUDDY_MODELS_JSON_TEMPLATE.contains(&std::format!(
        "\"${{{}}}\"",
        via_core::config::names::DASHSCOPE_API_KEY
    )));
    assert!(CODEBUDDY_MODELS_JSON_TEMPLATE.contains(&std::format!(
        "\"${{{}}}\"",
        via_core::config::names::CODEBUDDY_MODEL_URL
    )));
    assert!(
        !CODEBUDDY_MODELS_JSON_TEMPLATE.contains("qwen-audio-agent"),
        "no product identity belongs in a third-party vendor model table"
    );
}

#[test]
fn the_codebuddy_models_json_target_matches_the_catalogue() {
    let catalogued = contract("file-path", "CodeBuddy models.json target");
    assert!(catalogued.contains("<codeBuddyWorkspace>/.codebuddy/models.json"));
    assert!(catalogued.contains("dir mode 0o700"));
    assert!(catalogued.contains("file mode 0o600"));
    assert!(catalogued.contains("'wx'"));

    let workspace = tempfile::TempDir::new().expect("tempdir");
    let path = via_core::config::backend::codebuddy_models_json_path(workspace.path());
    assert_eq!(path, workspace.path().join(".codebuddy/models.json"));
    assert_eq!(
        via_core::runtime::seed_codebuddy_models_json(
            workspace.path(),
            CODEBUDDY_MODELS_JSON_TEMPLATE
        )
        .expect("seeds"),
        via_core::runtime::WriteOutcome::Created
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        CODEBUDDY_MODELS_JSON_TEMPLATE
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            std::fs::metadata(&path).expect("stat").permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(path.parent().expect("parent"))
                .expect("stat")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
    // `'wx'` — `IfExists::Keep` — never clobbers a file already on disk.
    assert_eq!(
        via_core::runtime::seed_codebuddy_models_json(workspace.path(), "{}").expect("kept"),
        via_core::runtime::WriteOutcome::Kept
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        CODEBUDDY_MODELS_JSON_TEMPLATE
    );
}

#[test]
fn the_two_default_base_urls_agree_with_the_catalogue() {
    let catalogued = contract("default-value", "backend default base URLs");
    for (id, url, variable) in [
        ("opencode", "http://127.0.0.1:4096", "OPENCODE_BASE_URL"),
        ("openclaw", "http://127.0.0.1:18789", "OPENCLAW_BASE_URL"),
    ] {
        assert!(catalogued.contains(url), "{id}");
        let definition = backend_definition(id).expect("catalogued");
        assert_eq!(definition.default_base_url, Some(url));
        assert_eq!(definition.base_url_environment, Some(variable));
    }
}

// ── OpenClaw ────────────────────────────────────────────────────────────────

#[test]
fn the_coordinator_session_key_is_the_catalogued_shape_after_the_rebrand() {
    let catalogued = contract("json-field", "OpenClaw coordinator session key");
    // The catalogue records the example twice; the second entry is the literal.
    let expected = rebranded("agent:voice-coordinator:qwen-audio-agent:owner%20one:backend");
    assert_eq!(expected, "agent:voice-coordinator:via:owner%20one:backend");
    assert!(
        rebranded(catalogued).contains(":via:") || catalogued.contains("qwen-audio-agent"),
        "the catalogued key names the product segment"
    );
    let meta = via_backends::openclaw::coordinator_meta("voice-coordinator", "owner one")
        .expect("a coordinator agent is configured");
    assert_eq!(meta.session_key, expected);
}

#[test]
fn the_openclaw_retry_policy_is_the_catalogued_one() {
    let catalogued = contract("error-code", "openclaw bridge diagnostics");
    assert!(catalogued.contains("[150,500,1000]"));
    assert_eq!(PROMPT_RETRY_DELAYS_MS, [150, 500, 1000]);
    assert!(catalogued.contains("🦞 [openclaw-bundle]"));
    assert_eq!(
        via_backends::openclaw::BUNDLE_NOISE_MARKER,
        "🦞 [openclaw-bundle]"
    );
}

#[test]
fn the_managed_gateway_binds_loopback_on_the_catalogued_port() {
    let catalogued = contract("ws-event", "OpenClaw managed gateway argv");
    assert!(catalogued.contains("gateway run --port"));
    assert!(catalogued.contains("--bind loopback"));
    assert_eq!(
        contract("default-value", "OPENCLAW_PORT default"),
        DEFAULT_GATEWAY_PORT
    );
    // The argv `via-backends` builds carries a placeholder rather than a baked
    // literal, so a port reallocated after the driver was constructed still
    // reaches the child. What the catalogue pins is the argv the child
    // *receives*, so assert that — through the real spawn path — for both
    // branches of the placeholder.
    assert_eq!(
        managed_gateway_arguments(&EnvMap::new()).join(" "),
        std::format!(
            "gateway run --port ${{OPENCLAW_PORT:-{DEFAULT_GATEWAY_PORT}}} --bind loopback"
        )
    );

    let catalogued_argv = std::format!("gateway run --port {DEFAULT_GATEWAY_PORT} --bind loopback");
    // Nothing published a port: the fallback is the catalogued default.
    assert_eq!(resolved_gateway_argv(&env(&[])), catalogued_argv);
    // `apply_backend_address` reallocated one: the child follows it there.
    assert_eq!(
        resolved_gateway_argv(&env(&[("OPENCLAW_PORT", "18999")])),
        "gateway run --port 18999 --bind loopback"
    );
}

/// The OpenClaw gateway argv as the spawned child actually receives it, with
/// `${OPENCLAW_PORT:-…}` expanded by `via-process` at spawn time.
fn resolved_gateway_argv(env: &EnvMap) -> String {
    /// Resolving a command must not touch the developer's filesystem.
    #[derive(Debug)]
    struct AnywhereResolver;

    impl via_process::CommandResolver for AnywhereResolver {
        fn resolve(
            &self,
            command: &str,
            _search_path: &str,
            _working_directory: &std::path::Path,
        ) -> Result<std::path::PathBuf, via_process::ProcessError> {
            Ok(std::path::PathBuf::from("/usr/local/bin").join(command))
        }
    }

    let definition = backend_definition("openclaw").expect("openclaw is catalogued");
    let driver = via_backends::openclaw_runtime_driver(definition, env);
    via_process::spawn_spec(
        &driver,
        std::path::Path::new("/repo"),
        env,
        via_core::search_path::Platform::Posix,
        &AnywhereResolver,
    )
    .expect("openclaw spawns a gateway")
    .arguments
    .join(" ")
}

#[test]
fn the_shipped_openclaw_configuration_carries_every_catalogued_field() {
    for (kind, name) in [
        ("json-field", "openclaw provider baseUrl"),
        ("json-field", "openclaw provider apiKey binding"),
        ("json-field", "openclaw provider api / compat"),
        ("json-field", "openclaw model limits and cost"),
        ("json-field", "openclaw gateway + tools block"),
    ] {
        let catalogued = contract(kind, name);
        // Each of these records a JSON fragment; assert the load-bearing
        // literals inside it rather than its prose formatting.
        for literal in catalogued
            .split(['•', ',', ':', '{', '}', '"'])
            .map(str::trim)
            .filter(|value| value.starts_with("https://") || value.starts_with("dashscope"))
        {
            assert!(
                OPENCLAW_CONFIG_JSON5.contains(literal),
                "{name}: `{literal}` is missing from the shipped asset"
            );
        }
    }
    assert!(OPENCLAW_CONFIG_JSON5.contains("https://dashscope.aliyuncs.com/apps/anthropic"));
    assert!(
        OPENCLAW_CONFIG_JSON5
            .contains(r#"{ source: "env", provider: "default", id: "DASHSCOPE_API_KEY" }"#)
    );
    assert!(OPENCLAW_CONFIG_JSON5.contains(r#"api: "anthropic-messages""#));
    assert!(OPENCLAW_CONFIG_JSON5.contains("contextWindow: 128000"));
    assert!(OPENCLAW_CONFIG_JSON5.contains("maxTokens: 8192"));
    assert!(
        OPENCLAW_CONFIG_JSON5
            .contains("cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 }")
    );
    assert!(OPENCLAW_CONFIG_JSON5.contains(r#"compat: { thinkingFormat: "openai" }"#));
    assert!(OPENCLAW_CONFIG_JSON5.contains(r#"sessions: { visibility: "all" }"#));
    assert!(OPENCLAW_CONFIG_JSON5.contains(r#"mode: "local""#));
    assert!(OPENCLAW_CONFIG_JSON5.contains(r#"bind: "loopback""#));
    assert!(OPENCLAW_CONFIG_JSON5.contains(r#"token: "${OPENCLAW_GATEWAY_TOKEN}""#));
}

#[test]
fn the_openclaw_asset_carries_the_rebranded_identity_and_nothing_else() {
    assert_eq!(
        contract("state-name", "OpenClaw agent id"),
        "qwen-audio-agent-backend"
    );
    assert!(OPENCLAW_CONFIG_JSON5.contains(&std::format!(
        r#"id: "{}""#,
        rebranded("qwen-audio-agent-backend")
    )));
    assert!(OPENCLAW_CONFIG_JSON5.contains(r#"name: "VIA Backend Agent""#));
    assert!(
        !OPENCLAW_CONFIG_JSON5.contains("qwen-audio-agent"),
        "the product identity is renamed"
    );
    assert!(
        !OPENCLAW_CONFIG_JSON5.contains("QWEN_AUDIO_AGENT_"),
        "the three ${{…}} references are renamed together with their setters"
    );
    for variable in [
        "${VIA_OPENCLAW_MODEL_ID}",
        "${VIA_OPENCLAW_MODEL}",
        "${VIA_OPENCLAW_WORKSPACE}",
    ] {
        assert!(OPENCLAW_CONFIG_JSON5.contains(variable), "{variable}");
    }
    // Third-party references stay.
    assert!(OPENCLAW_CONFIG_JSON5.contains("DASHSCOPE_API_KEY"));
    assert!(OPENCLAW_CONFIG_JSON5.contains("${OPENCLAW_GATEWAY_TOKEN}"));
}

// ── model-facing prompt text ────────────────────────────────────────────────

#[test]
fn the_three_session_instruction_paragraphs_are_byte_for_byte() {
    assert_eq!(
        contract("prompt-text", "deepseek sessionInstructions"),
        DEEPSEEK_SESSION_INSTRUCTIONS
    );
    assert_eq!(
        contract("prompt-text", "pi sessionInstructions"),
        PI_SESSION_INSTRUCTIONS
    );
    assert_eq!(
        contract("prompt-text", "openclaw sessionInstructions"),
        OPENCLAW_SESSION_INSTRUCTIONS
    );
}

// ── the credential boundary ─────────────────────────────────────────────────

/// Whether the catalogued allow-list names this variable.
///
/// The catalogue abbreviates a run of same-prefixed names — OpenClaw's four are
/// written `QWEN_AUDIO_AGENT_OPENCLAW_MODEL, _MODEL_ID, _STATE_DIR, _WORKSPACE`
/// — so a name that shares the run's prefix is accepted by its suffix.
fn catalogues_name(catalogued: &str, name: &str) -> bool {
    let renamed = rebranded(catalogued);
    if renamed.contains(name) {
        return true;
    }
    ["VIA_OPENCLAW", "VIA_OPENCODE"]
        .into_iter()
        .filter_map(|prefix| name.strip_prefix(prefix))
        .any(|suffix| {
            renamed.contains(&std::format!(" {suffix},"))
                || renamed.contains(&std::format!(" {suffix}]"))
        })
}

#[test]
fn every_backends_declared_namespace_matches_the_catalogue() {
    let catalogued = contract("env-var", "backend credential allowlist per backend");
    for id in backend_names() {
        let definition = backend_definition(id).expect("catalogued");
        for name in definition.environment.names {
            assert!(catalogues_name(catalogued, name), "{id}: name {name}");
        }
        for prefix in definition.environment.prefixes {
            assert!(catalogued.contains(prefix), "{id}: prefix {prefix}");
        }
        if let Some(explicit) = definition.environment.explicit_list_environment {
            assert!(
                catalogues_name(catalogued, explicit),
                "{id}: explicit list {explicit}"
            );
        }
    }
    assert!(
        entry("env-var", "backend credential allowlist per backend")
            .why
            .contains("Gateway secrets (auth secret, realtime key, memory key, speech-to-speech token) reach no backend."),
        "the sentence the boundary exists for"
    );
}

#[test]
fn the_desktop_installed_only_switch_is_the_catalogued_one() {
    let catalogued = contract("env-var", "QWEN_AUDIO_AGENT_DESKTOP_INSTALLED_ONLY");
    assert!(catalogued.starts_with("'1' forbids every npx package-mode fallback"));
    assert_eq!(
        via_backends::setup::DESKTOP_INSTALLED_ONLY,
        rebranded("QWEN_AUDIO_AGENT_DESKTOP_INSTALLED_ONLY")
    );
    assert_eq!(via_backends::setup::DESKTOP_INSTALLED_ONLY_ON, "1");

    // And it actually forbids one.
    let environment = env(&[
        ("DASHSCOPE_API_KEY", "key"),
        ("VIA_BACKEND_MODEL", "qwen3.7-max"),
        (via_backends::setup::DESKTOP_INSTALLED_ONLY, "1"),
    ]);
    let report = via_backends::inspect_backend_setups(&via_backends::SetupInspection {
        env: &environment,
        platform: HostPlatform::Linux,
        backend: "opencode",
        finder: &via_backends::detect::MissingFinder,
        versions: &via_backends::setup::NoVersions,
        packages: &via_backends::setup::NoPackages,
        locale: via_i18n::Locale::En,
    });
    assert!(!report.backends[0].ready);
}
