//! The twelve backend agents.
//!
//! `shared/backend-catalog.mjs` is the table every other backend surface reads.
//! Two things in it are contract in a way that is easy to lose:
//!
//! **The order.** `backendNames().join('、')` is interpolated into several error
//! messages and `--backend` documents the ids in this order, so the catalog is
//! an ordered list rather than a map.
//!
//! **The environment allow-policy.** `EnvironmentPolicy` is the credential
//! namespace boundary: a child agent process receives exactly the names and
//! prefixes declared for it, and the Gateway's own secrets — the auth secret,
//! the realtime API key, the memory-extractor key — reach no backend. The only
//! edits VIA makes are the identity renames `docs/rebrand.md` mandates, and
//! `backend_environment_allow_lists` derives those from the rule rather than
//! trusting a retyped constant. A *partial* rename is the failure mode
//! `docs/architecture.md` §13 singles out as carrying real risk.

use pretty_assertions::assert_eq;
use via_catalog::backend::{ConfigurationMode, ProbeKind};
use via_catalog::{
    BACKEND_NONE_SENTINEL, EnvironmentPolicy, backend_definition, backend_definitions,
    backend_names, effective_backend_permission_mode, normalize_backend_protocol,
};
use via_conformance::expect_contract;
use via_conformance::value::{after, before, list, rebranded};

/// The wire spelling of a value, from its own `Serialize` impl.
fn wire_name<T: serde::Serialize>(value: &T) -> String {
    let json = serde_json::to_value(value).expect("value serialises");
    match json {
        serde_json::Value::String(name) => name,
        other => panic!("expected a string, got {other}"),
    }
}

#[test]
fn backend_ids_in_catalog_order() {
    // `opencode, openclaw, … , acp - plus the sentinel 'none' which
    //  normalizeBackendProtocol maps to ''`
    let contract = expect_contract("state-name", "backend ids (all 12, in catalog order)");
    let expected = list(before(&contract.exact_value, " - plus"));
    assert_eq!(
        expected,
        backend_names(),
        "the twelve backend ids, in catalog order ({})",
        contract.file
    );
    assert_eq!(expected.len(), 12);

    // The sentinel, and the normalisation around it.
    assert_eq!(BACKEND_NONE_SENTINEL, "none");
    assert!(
        contract.exact_value.contains("the sentinel 'none'"),
        "the catalogue still records the sentinel ({})",
        contract.file
    );
    for spelling in ["none", "NONE", "  None  ", ""] {
        assert_eq!(
            normalize_backend_protocol(spelling),
            "",
            "`{spelling}` means frontend-only"
        );
    }

    // `--backend NAME — value normalized lowercase; 'none' (any case) and ''
    //  mean frontend-only; valid ids: opencode, …`
    //
    // The flag itself belongs to `apps/via`; the id list is this catalog's.
    let flag = expect_contract("cli-flag", "--backend");
    assert_eq!(
        list(after(&flag.exact_value, "valid ids:")),
        backend_names(),
        "`--backend` documents the ids in catalog order ({})",
        flag.file
    );
    for id in backend_names() {
        assert_eq!(normalize_backend_protocol(&id.to_uppercase()), id);
        assert!(backend_definition(id).is_some());
    }
}

#[test]
fn backend_labels() {
    let contract = expect_contract("state-name", "backend labels");
    let expected = list(&contract.exact_value);
    let shipped: Vec<&str> = backend_definitions().iter().map(|b| b.label).collect();
    assert_eq!(
        expected, shipped,
        "the twelve labels, in catalog order ({})",
        contract.file
    );
    assert_eq!(expected.len(), 12);
}

#[test]
fn backend_minimum_versions() {
    // `opencode 1.18.0, qwen 0.21.6, kimi 0.31.0, pi 0.80.4 (all others: none)`
    let contract = expect_contract("default-value", "backend minimum versions");
    let mut expected: Vec<(&str, &str)> = Vec::new();
    for pair in list(before(&contract.exact_value, " (all others")) {
        let mut parts = pair.split_whitespace();
        match (parts.next(), parts.next(), parts.next()) {
            (Some(id), Some(version), None) => expected.push((id, version)),
            _ => panic!(
                "`{pair}` is not an `<id> <version>` pair ({})",
                contract.file
            ),
        }
    }
    assert_eq!(expected.len(), 4);

    for definition in backend_definitions() {
        let declared = expected
            .iter()
            .find(|(id, _)| *id == definition.id)
            .map(|(_, version)| *version);
        assert_eq!(
            definition.setup.minimum_version, declared,
            "minimum version for `{}` ({})",
            definition.id, contract.file
        );
    }
}

#[test]
fn backend_base_urls() {
    // `opencode http://127.0.0.1:4096 (OPENCODE_BASE_URL); openclaw
    //  http://127.0.0.1:18789 (OPENCLAW_BASE_URL, external service credential
    //  env OPENCLAW_GATEWAY_TOKEN)`
    let contract = expect_contract(
        "default-value",
        "backend default base URLs and external service credential",
    );
    for (id, url, env) in [
        ("opencode", "http://127.0.0.1:4096", "OPENCODE_BASE_URL"),
        ("openclaw", "http://127.0.0.1:18789", "OPENCLAW_BASE_URL"),
    ] {
        assert!(
            contract.exact_value.contains(&format!("{id} {url} ({env}")),
            "`{id} {url} ({env}` is not in the catalogued base-URL rule ({})",
            contract.file
        );
        let definition = backend_definition(id).expect("the backend is in the catalog");
        assert_eq!(definition.default_base_url, Some(url));
        assert_eq!(definition.base_url_environment, Some(env));
    }

    let openclaw = backend_definition("openclaw").expect("the backend is in the catalog");
    assert!(openclaw.supports_external_service);
    assert_eq!(
        openclaw.external_service.map(|s| s.credential_environment),
        Some("OPENCLAW_GATEWAY_TOKEN"),
        "the external-service credential is a vendor name and is KEEP ({})",
        contract.file
    );

    // Nothing else declares a base URL, which is what makes "the Gateway only
    // starts local backends" checkable at all.
    for definition in backend_definitions() {
        if definition.id != "opencode" && definition.id != "openclaw" {
            assert_eq!(
                definition.default_base_url, None,
                "`{}` must declare no base URL",
                definition.id
            );
        }
    }

    // The second catalogue record adds the two port variables, which belong to
    // `via-backends`; this contract stays partial until that crate lands.
    let with_ports = expect_contract("default-value", "backend default base URLs");
    for env in ["OPENCODE_PORT", "OPENCLAW_PORT"] {
        assert!(
            with_ports.exact_value.contains(env),
            "`{env}` is still catalogued and still unowned ({})",
            with_ports.file
        );
    }
}

#[test]
fn backend_auth_probe_kinds() {
    // `command (parsers: credential-count | qoder-status | codex-status) |
    //  qwen-settings | pi-auth-check | deepseek-credentials |
    //  codebuddy-credentials | openclaw-state`
    let contract = expect_contract("state-name", "backend auth probe kinds");
    let expected_parsers = list(before(after(&contract.exact_value, "(parsers:"), ")"));
    let mut expected_kinds = vec!["command"];
    expected_kinds.extend(list(after(&contract.exact_value, ")")));
    assert_eq!(
        expected_parsers,
        ["credential-count", "qoder-status", "codex-status"],
        "{}",
        contract.file
    );
    assert_eq!(
        expected_kinds.len(),
        6,
        "six probe kinds ({})",
        contract.file
    );

    let mut shipped_kinds: Vec<String> = Vec::new();
    let mut shipped_parsers: Vec<String> = Vec::new();
    for definition in backend_definitions() {
        let Some(probe) = definition.onboarding.and_then(|o| o.probe) else {
            continue;
        };
        let kind = wire_name(&probe.kind);
        assert!(
            expected_kinds.contains(&kind.as_str()),
            "`{}` declares probe kind `{kind}`, which the catalogue does not list",
            definition.id
        );
        if !shipped_kinds.contains(&kind) {
            shipped_kinds.push(kind);
        }
        match probe.parser {
            Some(parser) => {
                assert_eq!(
                    probe.kind,
                    ProbeKind::Command,
                    "only a `command` probe has a parser ({})",
                    definition.id
                );
                let name = wire_name(&parser);
                assert!(
                    expected_parsers.contains(&name.as_str()),
                    "`{}` declares parser `{name}`, which the catalogue does not list",
                    definition.id
                );
                if !shipped_parsers.contains(&name) {
                    shipped_parsers.push(name);
                }
            }
            None => assert_ne!(
                probe.kind,
                ProbeKind::Command,
                "a `command` probe must declare a parser ({})",
                definition.id
            ),
        }
    }

    shipped_kinds.sort();
    shipped_parsers.sort();
    let mut sorted_kinds = expected_kinds.clone();
    sorted_kinds.sort_unstable();
    let mut sorted_parsers = expected_parsers.clone();
    sorted_parsers.sort_unstable();
    assert_eq!(
        shipped_kinds, sorted_kinds,
        "every catalogued probe kind is used, and no other ({})",
        contract.file
    );
    assert_eq!(
        shipped_parsers, sorted_parsers,
        "every catalogued parser is used, and no other ({})",
        contract.file
    );
}

#[test]
fn backend_environment_allow_lists() {
    let contract = expect_contract("env-var", "backend credential allowlist per backend");

    // The catalogue abbreviates openclaw's last three names to their suffixes:
    // `QWEN_AUDIO_AGENT_OPENCLAW_MODEL, _MODEL_ID, _STATE_DIR, _WORKSPACE`.
    // Expanded once, here, so everything below compares full names.
    let catalogued = contract
        .exact_value
        .replace(", _MODEL_ID,", ", QWEN_AUDIO_AGENT_OPENCLAW_MODEL_ID,")
        .replace(", _STATE_DIR,", ", QWEN_AUDIO_AGENT_OPENCLAW_STATE_DIR,")
        .replace(", _WORKSPACE]", ", QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE]");

    let segments: Vec<&str> = catalogued.split(" | ").collect();
    assert_eq!(
        segments.len(),
        12,
        "one segment per backend ({})",
        contract.file
    );

    for (segment, definition) in segments.iter().zip(backend_definitions()) {
        let (id, body) = segment
            .split_once(':')
            .unwrap_or_else(|| panic!("`{segment}` is not `<id>: <policy>`"));
        assert_eq!(
            id.trim(),
            definition.id,
            "the allow-list is catalogued in catalog order ({})",
            contract.file
        );

        // names [A, B, C]
        let upstream_names = if body.contains("names [") {
            list(before(after(body, "names ["), "]"))
        } else {
            Vec::new()
        };
        // "+ prefix X" or "+ prefixes X, Y"
        let upstream_prefixes = if body.contains("prefix") {
            let tail = after(body, "prefix");
            let tail = tail.strip_prefix("es").unwrap_or(tail);
            list(before(before(tail, "+"), "explicitListEnvironment"))
        } else {
            Vec::new()
        };
        // "explicitListEnvironment X"
        let upstream_explicit = body
            .contains("explicitListEnvironment")
            .then(|| after(body, "explicitListEnvironment").trim());

        let expected = EnvironmentPolicyShape {
            names: upstream_names.iter().map(|n| rebranded(n)).collect(),
            prefixes: upstream_prefixes.iter().map(|p| rebranded(p)).collect(),
            explicit: upstream_explicit.map(rebranded),
        };
        assert_eq!(
            expected,
            shape_of(&definition.environment),
            "environment allow-policy for `{}`, after applying docs/rebrand.md ({})",
            definition.id,
            contract.file
        );

        // The rename is real: no upstream identity prefix survives anywhere in
        // the shipped policy, and no VIA name appears that the rule did not
        // produce.
        for name in definition
            .environment
            .names
            .iter()
            .chain(definition.environment.prefixes)
            .chain(definition.environment.explicit_list_environment.iter())
        {
            for upstream_prefix in ["QWEN_AUDIO_AGENT_", "QWAUDIO_", "QWEN_AUDIO_", "QWEN_OMNI_"] {
                assert!(
                    !name.starts_with(upstream_prefix),
                    "`{name}` still carries the upstream prefix `{upstream_prefix}` on `{}`",
                    definition.id
                );
            }
        }
    }

    // The Gateway's own secrets reach no backend. None of these is in any
    // policy, by name or by prefix.
    for secret in [
        "VIA_AUTH_SECRET",
        "VIA_REALTIME_API_KEY",
        "SPEECH_TO_SPEECH_AUTH_TOKEN",
    ] {
        for definition in backend_definitions() {
            assert!(
                !definition.environment.allows(secret),
                "`{secret}` must not reach the `{}` backend",
                definition.id
            );
        }
    }
}

/// An owned, comparable view of an [`EnvironmentPolicy`].
#[derive(Debug, PartialEq, Eq)]
struct EnvironmentPolicyShape {
    names: Vec<String>,
    prefixes: Vec<String>,
    explicit: Option<String>,
}

fn shape_of(policy: &EnvironmentPolicy) -> EnvironmentPolicyShape {
    EnvironmentPolicyShape {
        names: policy.names.iter().map(|n| (*n).to_owned()).collect(),
        prefixes: policy.prefixes.iter().map(|p| (*p).to_owned()).collect(),
        explicit: policy.explicit_list_environment.map(str::to_owned),
    }
}

#[test]
fn backend_workspace_environment() {
    // `OPENCODE_WORKSPACE, QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE, …,
    //  ACP_WORKSPACE. Set -> resolve(root, value); …`
    //
    // The resolution rule and the 0o700 mkdir belong to `via-core`; the twelve
    // variable names belong here.
    let contract = expect_contract("env-var", "backend workspace overrides");
    let expected: Vec<String> = list(before(&contract.exact_value, ". Set ->"))
        .iter()
        .map(|name| rebranded(name))
        .collect();
    let shipped: Vec<String> = backend_definitions()
        .iter()
        .map(|b| b.workspace_environment.to_owned())
        .collect();
    assert_eq!(
        expected, shipped,
        "workspace overrides, in catalog order, after docs/rebrand.md ({})",
        contract.file
    );
    assert_eq!(expected.len(), 12);

    // The openclaw variable is catalogued a second time on its own.
    let openclaw = expect_contract("env-var", "${QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE}");
    assert_eq!(
        rebranded("QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE"),
        backend_definition("openclaw")
            .expect("the backend is in the catalog")
            .workspace_environment,
        "{}",
        openclaw.file
    );
}

#[test]
fn integration_and_configuration_modes() {
    // `integration: native | bridge | adapter | generic.
    //  lifecycle.configuration.mode: backend-owned | bailian-or-backend-owned |
    //  user-managed. …`
    let contract = expect_contract(
        "state-name",
        "backend integration modes and configuration modes",
    );
    let integrations = list(before(after(&contract.exact_value, "integration:"), "."));
    let modes = list(before(
        after(&contract.exact_value, "lifecycle.configuration.mode:"),
        ".",
    ));
    assert_eq!(integrations, ["native", "bridge", "adapter", "generic"]);
    assert_eq!(
        modes,
        ["backend-owned", "bailian-or-backend-owned", "user-managed"]
    );

    let mut seen_integrations: Vec<String> = Vec::new();
    let mut seen_modes: Vec<String> = Vec::new();
    for definition in backend_definitions() {
        let integration = wire_name(&definition.setup.integration);
        assert!(
            integrations.contains(&integration.as_str()),
            "`{}` declares integration `{integration}`",
            definition.id
        );
        if !seen_integrations.contains(&integration) {
            seen_integrations.push(integration);
        }

        let mode = wire_name(&definition.lifecycle.configuration);
        assert!(
            modes.contains(&mode.as_str()),
            "`{}` declares configuration mode `{mode}`",
            definition.id
        );
        if !seen_modes.contains(&mode) {
            seen_modes.push(mode);
        }
    }
    seen_integrations.sort();
    seen_modes.sort();
    let mut sorted_integrations = integrations.clone();
    sorted_integrations.sort_unstable();
    let mut sorted_modes = modes.clone();
    sorted_modes.sort_unstable();
    assert_eq!(
        seen_integrations, sorted_integrations,
        "every integration mode is used by some backend"
    );
    assert_eq!(
        seen_modes, sorted_modes,
        "every configuration mode is used by some backend"
    );

    // `acp` is the one backend VIA never installs, and that `installation:
    // null` is contract rather than an empty plan.
    let acp = backend_definition("acp").expect("the backend is in the catalog");
    assert_eq!(acp.lifecycle.installation, None);
    assert_eq!(acp.lifecycle.configuration, ConfigurationMode::UserManaged);
}

#[test]
fn pi_is_always_full_permission() {
    // `alwaysFullPermission true, supportsFullPermission true, … ;
    //  effectiveBackendPermissionMode('pi', anything) === 'full'`
    //
    // The four backend-driver capability flags in the same record belong to
    // `via-backends`; this contract stays partial until that crate lands.
    let contract = expect_contract("json-field", "pi backend capability declaration");
    for expected in ["alwaysFullPermission true", "supportsFullPermission true"] {
        assert!(
            contract.exact_value.contains(expected),
            "`{expected}` is not in the catalogued declaration ({})",
            contract.file
        );
    }

    let pi = backend_definition("pi").expect("the backend is in the catalog");
    assert!(pi.always_full_permission);
    assert!(pi.supports_full_permission);
    for requested in ["", "native", "NATIVE", "  full ", "nonsense"] {
        assert_eq!(
            effective_backend_permission_mode("pi", requested),
            "full",
            "pi has no approval gate in any configuration"
        );
    }

    // It is the only one, which is what makes the disclosure meaningful.
    for definition in backend_definitions() {
        if definition.id != "pi" {
            assert!(
                !definition.always_full_permission,
                "`{}` must not declare alwaysFullPermission",
                definition.id
            );
            assert_eq!(
                effective_backend_permission_mode(definition.id, ""),
                "native"
            );
        }
    }
}
