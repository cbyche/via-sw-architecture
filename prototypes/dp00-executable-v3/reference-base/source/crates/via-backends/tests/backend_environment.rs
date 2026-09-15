//! The credential boundary, asserted against the real twelve.
//!
//! A verbatim port of `server/test/backend-environment.test.mjs`, which
//! `via-acp` could not host because it may not name a backend. Upstream's own
//! note on why this matters, from
//! `env-var/child-process environment filter`: *"This is a trust boundary:
//! Gateway, realtime and memory secrets are deliberately NOT inherited.
//! Widening it in the Rust port is a security regression."*

mod support;

use support::{config, env, profile};
use via_acp::BackendEnv;
use via_catalog::{backend_definition, backend_names};
use via_core::EnvMap;

/// The fixture from `server/test/backend-environment.test.mjs:5-17`, with the
/// three VIA-owned names renamed per `docs/rebrand.md`.
fn parent() -> EnvMap {
    env(&[
        ("PATH", "/usr/bin"),
        ("HOME", "/home/user"),
        ("LANG", "zh_CN.UTF-8"),
        ("DASHSCOPE_API_KEY", "dashscope-secret"),
        ("DEEPSEEK_API_KEY", "deepseek-secret"),
        ("ANTHROPIC_API_KEY", "anthropic-secret"),
        ("OPENCLAW_GATEWAY_TOKEN", "openclaw-secret"),
        // Upstream QWEN_AUDIO_AGENT_AUTH_SECRET.
        ("VIA_AUTH_SECRET", "identity-secret"),
        // Upstream QWEN_AUDIO_REALTIME_API_KEY.
        ("VIA_REALTIME_API_KEY", "realtime-secret"),
        // Upstream QWEN_AUDIO_MEMORY_API_KEY.
        ("VIA_MEMORY_API_KEY", "memory-secret"),
        ("SPEECH_TO_SPEECH_AUTH_TOKEN", "speech-secret"),
    ])
}

/// Project the Gateway environment the way a driver does.
fn projected(id: &str) -> BackendEnv {
    let definition = backend_definition(id).expect("catalogued");
    BackendEnv::project(&definition.environment, &parent(), &[])
}

/// `server/test/backend-environment.test.mjs:19-29`.
#[test]
fn projects_only_operating_system_and_selected_backend_environment() {
    let deepseek = projected("deepseek");
    assert_eq!(deepseek.get("PATH"), Some("/usr/bin"));
    assert_eq!(deepseek.get("DEEPSEEK_API_KEY"), Some("deepseek-secret"));
    assert!(!deepseek.contains("DASHSCOPE_API_KEY"));
    assert!(!deepseek.contains("OPENCLAW_GATEWAY_TOKEN"));
    assert!(!deepseek.contains("VIA_AUTH_SECRET"));
    assert!(!deepseek.contains("VIA_REALTIME_API_KEY"));
    assert!(!deepseek.contains("VIA_MEMORY_API_KEY"));
    assert!(!deepseek.contains("SPEECH_TO_SPEECH_AUTH_TOKEN"));
}

/// `server/test/backend-environment.test.mjs:31-48`.
#[test]
fn keeps_each_backend_credential_namespace_isolated() {
    assert_eq!(
        projected("claude").get("ANTHROPIC_API_KEY"),
        Some("anthropic-secret")
    );
    assert_eq!(
        projected("openclaw").get("OPENCLAW_GATEWAY_TOKEN"),
        Some("openclaw-secret")
    );
    assert_eq!(
        projected("opencode").get("DASHSCOPE_API_KEY"),
        Some("dashscope-secret")
    );
    assert!(!projected("claude").contains("DEEPSEEK_API_KEY"));
}

/// `server/test/backend-environment.test.mjs:50-61`.
#[test]
fn generic_acp_forwards_additional_names_only_when_explicitly_requested() {
    let definition = backend_definition("acp").expect("catalogued");
    let mut environment = parent();
    environment.set("CUSTOM_AGENT_TOKEN", "custom-secret");
    // Upstream QWEN_AUDIO_AGENT_ACP_FORWARD_ENV.
    environment.set("VIA_ACP_FORWARD_ENV", "CUSTOM_AGENT_TOKEN");

    let projected = BackendEnv::project(&definition.environment, &environment, &[]);
    assert_eq!(projected.get("CUSTOM_AGENT_TOKEN"), Some("custom-secret"));
    assert!(!projected.contains("DASHSCOPE_API_KEY"));
    assert!(
        !projected.contains("VIA_ACP_FORWARD_ENV"),
        "the opt-in list must not forward itself"
    );

    // Without the opt-in the same variable does not cross.
    let without = BackendEnv::project(&definition.environment, &parent(), &[]);
    assert!(!without.contains("CUSTOM_AGENT_TOKEN"));
}

#[test]
fn only_the_generic_backend_has_an_opt_in_at_all() {
    for id in backend_names() {
        let definition = backend_definition(id).expect("catalogued");
        assert_eq!(
            definition.environment.explicit_list_environment.is_some(),
            id == "acp",
            "{id}"
        );
    }
}

#[test]
fn no_gateway_secret_reaches_any_of_the_twelve() {
    const SECRETS: [&str; 4] = [
        "VIA_AUTH_SECRET",
        "VIA_REALTIME_API_KEY",
        "VIA_MEMORY_API_KEY",
        "SPEECH_TO_SPEECH_AUTH_TOKEN",
    ];
    for id in backend_names() {
        let projected = projected(id);
        for secret in SECRETS {
            assert!(!projected.contains(secret), "{id} received {secret}");
        }
    }
}

#[test]
fn a_built_profile_carries_the_same_boundary_as_the_policy() {
    // The projection is not a helper a driver may forget: every spawn spec this
    // crate builds goes through it, so the boundary holds for the *profile*,
    // not just for the policy.
    let mut config = config();
    config.backends.acp.cli_path = "/opt/agent".to_owned();
    for id in backend_names() {
        let built = profile(id, &config, &parent(), "native")
            .unwrap_or_else(|error| panic!("{id} builds: {error}"));
        assert!(
            !built.acp.spawn.env.contains("VIA_AUTH_SECRET"),
            "{id} received the Gateway auth secret"
        );
        assert!(
            !built.acp.spawn.env.contains("SPEECH_TO_SPEECH_AUTH_TOKEN"),
            "{id} received the speech-to-speech token"
        );
        // Every child still gets the portable OS context it needs to run.
        assert_eq!(built.acp.spawn.env.get("PATH"), Some("/usr/bin"), "{id}");
        assert_eq!(built.acp.spawn.env.get("VIA_ENV_LOADED"), Some("1"), "{id}");
    }
}

#[test]
fn a_backends_own_namespace_still_reaches_it_through_the_profile() {
    let config = config();
    let built = profile("deepseek", &config, &parent(), "native").expect("builds");
    assert_eq!(
        built.acp.spawn.env.get("DEEPSEEK_API_KEY"),
        Some("deepseek-secret")
    );
    assert!(
        !built.acp.spawn.env.contains("ANTHROPIC_API_KEY"),
        "another backend's credential namespace never crosses"
    );
}
