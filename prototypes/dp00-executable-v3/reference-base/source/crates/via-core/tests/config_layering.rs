//! Layering, coercion and every refusal the resolver can produce.

mod common;

use common::{env, overrides};
use pretty_assertions::assert_eq;
use via_core::CoreError;
use via_core::config::{
    BackendSelection, GatewayOptions, backend, is_no_backend, names, resolve, resolve_acp_args,
    resolve_backend_models,
};
use via_core::env::{Bounds, EnvMap, integer_setting, js_number, number_setting};
use via_core::envfile::parse_env_map;

fn resolve_with(pairs: &[(&str, &str)], file: Option<&str>) -> Result<via_core::Config, CoreError> {
    resolve(&env(pairs), file, &overrides())
}

// ── layering ────────────────────────────────────────────────────────────────

#[test]
fn the_file_fills_only_what_the_environment_left_undefined() {
    let config = resolve_with(
        &[("HOST", "10.0.0.1")],
        Some("HOST=192.0.2.1\nVIA_PERSONAL_OWNER_ID=user_from_file\n"),
    )
    .expect("resolves");
    assert_eq!(config.host, "10.0.0.1");
    assert_eq!(config.personal_owner_id, "user_from_file");
}

#[test]
fn an_empty_shell_assignment_masks_the_file_without_taking_its_place() {
    // `HOST=` in the shell is *defined*, so the file may not fill it — and an
    // empty value is falsy, so the default applies. This is the exact contract
    // `docs/reference/contracts.json` calls "an empty shell assignment masks
    // all files".
    let config = resolve_with(&[("HOST", "")], Some("HOST=192.0.2.1\n")).expect("resolves");
    assert_eq!(config.host, "127.0.0.1");
}

#[test]
fn a_repeated_key_in_the_file_keeps_the_last_assignment() {
    let config = resolve_with(&[], Some("HOST=first\nHOST=second\n")).expect("resolves");
    assert_eq!(config.host, "second");
}

#[test]
fn a_comment_only_file_changes_nothing() {
    let config = resolve_with(&[], Some("# nothing here\n\n   \n")).expect("resolves");
    assert_eq!(config, resolve_with(&[], None).expect("resolves"));
}

#[test]
fn host_options_emit_nothing_for_an_option_that_was_not_passed() {
    assert!(GatewayOptions::default().to_environment().is_empty());
}

#[test]
fn the_backend_none_selection_empties_agent_protocol_rather_than_deleting_it() {
    let emitted = GatewayOptions {
        backend: Some(BackendSelection::None),
        ..GatewayOptions::default()
    }
    .to_environment();
    assert!(emitted.contains_key(names::AGENT_PROTOCOL));
    assert_eq!(emitted.get(names::AGENT_PROTOCOL), Some(""));

    // Which is what stops the file from putting it back.
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
    assert!(!config.has_backend());
}

// ── numeric coercion ────────────────────────────────────────────────────────

#[test]
fn number_setting_follows_javascript_number_not_parse_int() {
    let bounds = Bounds::UNBOUNDED;
    assert_eq!(number_setting(Some("1e4"), 1.0, bounds), 10_000.0);
    assert_eq!(number_setting(Some("0x10"), 1.0, bounds), 16.0);
    // `parseInt` would answer 12 here; `Number` answers NaN, so the fallback
    // applies.
    assert_eq!(number_setting(Some("12abc"), 7.0, bounds), 7.0);
    for blank in [None, Some(""), Some("   "), Some("\t\n")] {
        assert_eq!(
            number_setting(blank, 120.0, Bounds::range(0.0, 1_000.0)),
            120.0
        );
    }
    // An explicit zero survives, where a truthiness test would lose it.
    assert_eq!(
        number_setting(Some("0"), 120.0, Bounds::range(0.0, 1_000.0)),
        0.0
    );
    // Infinity is parseable but not finite.
    assert_eq!(number_setting(Some("Infinity"), 5.0, bounds), 5.0);
    assert_eq!(js_number("Infinity"), Some(f64::INFINITY));
}

#[test]
fn integer_setting_truncates_toward_zero() {
    assert_eq!(integer_setting(Some("2.9"), 1, Bounds::UNBOUNDED), 2);
    assert_eq!(integer_setting(Some("-2.9"), 1, Bounds::UNBOUNDED), -2);
    assert_eq!(
        integer_setting(Some("1e30"), 1, Bounds::UNBOUNDED),
        i64::MAX
    );
}

#[test]
fn a_crossed_bound_lets_the_maximum_win() {
    // `Math.min(max, Math.max(min, parsed))` applies min first, so max wins.
    assert_eq!(
        number_setting(Some("50"), 0.0, Bounds::range(100.0, 10.0)),
        10.0
    );
}

// ── backend selection ───────────────────────────────────────────────────────

#[test]
fn an_unknown_backend_is_refused_by_name() {
    let error = resolve_with(&[("AGENT_PROTOCOL", "nonesuch")], None)
        .expect_err("an unknown backend must not start");
    assert_eq!(error.code(), "VIA_BACKEND_UNSUPPORTED");
}

#[test]
fn none_and_empty_both_mean_frontend_only() {
    for value in ["", "none", " NONE ", "None"] {
        assert!(is_no_backend(value));
        let config = resolve_with(&[("AGENT_PROTOCOL", value)], None).expect("resolves");
        assert_eq!(
            config.agent_protocol, "",
            "{value:?} must select no backend"
        );
    }
}

#[test]
fn an_unsupported_permission_mode_is_refused_only_when_a_backend_is_configured() {
    let error = resolve_with(
        &[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::BACKEND_PERMISSION_MODE, "root"),
        ],
        None,
    )
    .expect_err("only native and full exist");
    assert_eq!(
        error.code(),
        "VIA_CONFIG_BACKEND_PERMISSION_MODE_UNSUPPORTED"
    );

    // With no backend the same stale variable must not refuse the start.
    let config = resolve_with(&[(names::BACKEND_PERMISSION_MODE, "root")], None)
        .expect("a frontend-only install ignores it");
    assert_eq!(config.backend_permission_mode, "root");
}

#[test]
fn pi_is_always_full_permission_whatever_is_configured() {
    let config = resolve_with(
        &[
            ("AGENT_PROTOCOL", "pi"),
            (names::BACKEND_PERMISSION_MODE, "native"),
        ],
        None,
    )
    .expect("resolves");
    assert_eq!(config.backend_permission_mode, "full");
}

#[test]
fn an_explicit_base_url_flips_openclaw_to_external() {
    let owned = resolve_with(&[("AGENT_PROTOCOL", "openclaw")], None).expect("resolves");
    assert_eq!(owned.backend_ownership.as_str(), "owned");

    let external = resolve_with(
        &[
            ("AGENT_PROTOCOL", "openclaw"),
            ("OPENCLAW_BASE_URL", "wss://openclaw.example.com"),
        ],
        None,
    )
    .expect("resolves");
    assert_eq!(external.backend_ownership.as_str(), "external");
}

#[test]
fn external_ownership_is_refused_for_a_backend_that_cannot_be_external() {
    let error = resolve_with(
        &[
            ("AGENT_PROTOCOL", "opencode"),
            (names::BACKEND_OWNERSHIP, "external"),
        ],
        None,
    )
    .expect_err("OpenCode must be launched by the Gateway");
    assert_eq!(error.code(), "VIA_BACKEND_EXTERNAL_SERVICE_UNSUPPORTED");

    let error = resolve_with(
        &[
            ("AGENT_PROTOCOL", "openclaw"),
            (names::BACKEND_OWNERSHIP, "borrowed"),
        ],
        None,
    )
    .expect_err("only owned and external exist");
    assert_eq!(error.code(), "VIA_BACKEND_OWNERSHIP_UNSUPPORTED");
}

#[test]
fn full_permission_needs_an_owned_backend() {
    use via_catalog::Ownership;
    use via_core::config::assert_full_permission_allowed;

    assert!(assert_full_permission_allowed("full", Ownership::Owned).is_ok());
    assert!(assert_full_permission_allowed("native", Ownership::External).is_ok());
    let error = assert_full_permission_allowed("full", Ownership::External)
        .expect_err("full against an external service is refused");
    assert_eq!(error.code(), "VIA_CONFIG_FULL_PERMISSION_REQUIRES_OWNED");
}

#[test]
fn one_backend_model_maps_to_every_backend() {
    let models = resolve_backend_models(&env(&[(names::BACKEND_MODEL, "bailian/qwen3.7-plus")]));
    assert_eq!(models.common, "bailian/qwen3.7-plus");
    assert_eq!(models.open_code, "alibaba-cn/qwen3.7-plus");
    assert_eq!(models.open_claw, "bailian/qwen3.7-plus");
    assert_eq!(models.qoder, "qwen3.7-plus");
    assert_eq!(models.kimi, "bailian/qwen3.7-plus");
    assert_eq!(models.deep_seek_harness, "");

    // DeepSeek takes its own variable first, then a deepseek-prefixed name.
    let explicit = resolve_backend_models(&env(&[
        ("DEEPSEEK_HARNESS_MODEL", "deepseek-v4-flash"),
        (names::BACKEND_MODEL, "qwen3.7-max"),
    ]));
    assert_eq!(explicit.deep_seek_harness, "deepseek-v4-flash");
    let inferred = resolve_backend_models(&env(&[(names::BACKEND_MODEL, "deepseek-v4-pro")]));
    assert_eq!(inferred.deep_seek_harness, "deepseek-v4-pro");
}

#[test]
fn a_backend_native_model_variable_is_not_a_gateway_override() {
    let models = resolve_backend_models(&env(&[
        ("OPENCODE_MODEL", "custom-open/code-model"),
        ("QODER_MODEL", "qoder-model"),
    ]));
    assert_eq!(models, Default::default());
}

// ── ACP arguments ───────────────────────────────────────────────────────────

#[test]
fn acp_args_have_three_shapes_and_two_refusals() {
    assert_eq!(
        resolve_acp_args(None).expect("empty is empty"),
        Vec::<String>::new()
    );
    assert_eq!(
        resolve_acp_args(Some("--acp   --verbose")).expect("whitespace form"),
        vec!["--acp".to_owned(), "--verbose".to_owned()]
    );
    assert_eq!(
        resolve_acp_args(Some(r#"["--acp", "a b"]"#)).expect("json form"),
        vec!["--acp".to_owned(), "a b".to_owned()]
    );
    // The whitespace fallback does NOT honour quotes; that is why the JSON
    // form exists and why `.env.example` recommends it.
    assert_eq!(
        resolve_acp_args(Some("--flag \"a b\"")).expect("whitespace form"),
        vec!["--flag".to_owned(), "\"a".to_owned(), "b\"".to_owned()]
    );
    assert_eq!(
        resolve_acp_args(Some("[oops"))
            .expect_err("not JSON")
            .code(),
        "VIA_CONFIG_ACP_ARGS_NOT_JSON"
    );
    assert_eq!(
        resolve_acp_args(Some("[1, 2]"))
            .expect_err("not strings")
            .code(),
        "VIA_CONFIG_ACP_ARGS_NOT_STRING_ARRAY"
    );
    // An object is not the array form at all, so it is split on whitespace.
    assert_eq!(
        resolve_acp_args(Some("{\"a\":1}")).expect("not the array form"),
        vec!["{\"a\":1}".to_owned()]
    );
}

#[test]
fn an_acp_args_refusal_propagates_out_of_resolve() {
    let error = resolve_with(&[("ACP_ARGS", "[nope")], None).expect_err("malformed ACP_ARGS");
    assert_eq!(error.code(), "VIA_CONFIG_ACP_ARGS_NOT_JSON");
}

// ── paths ───────────────────────────────────────────────────────────────────

#[test]
fn a_relative_path_override_anchors_to_the_runtime_root_not_the_cwd() {
    let config =
        resolve_with(&[(names::TASK_STATE_PATH, "state/tasks.json")], None).expect("resolves");
    assert_eq!(
        config.task_state_path,
        std::path::Path::new("/opt/via/state/tasks.json")
    );
}

#[test]
fn the_legacy_path_names_still_work_and_the_modern_one_wins() {
    let legacy =
        resolve_with(&[(names::USER_PROFILE_PATH, "legacy/USER.md")], None).expect("resolves");
    assert_eq!(
        legacy.user_model_path,
        std::path::Path::new("/opt/via/legacy/USER.md")
    );

    let both = resolve_with(
        &[
            (names::USER_MODEL_PATH, "modern/USER.md"),
            (names::USER_PROFILE_PATH, "legacy/USER.md"),
        ],
        None,
    )
    .expect("resolves");
    assert_eq!(
        both.user_model_path,
        std::path::Path::new("/opt/via/modern/USER.md")
    );
}

#[test]
fn runtime_root_overrides_the_installation_root() {
    let config = resolve_with(
        &[
            (names::RUNTIME_ROOT, "/elsewhere"),
            (names::TASK_STATE_PATH, "tasks.json"),
        ],
        None,
    )
    .expect("resolves");
    assert_eq!(config.root, std::path::Path::new("/elsewhere"));
    assert_eq!(
        config.task_state_path,
        std::path::Path::new("/elsewhere/tasks.json")
    );
}

#[test]
fn the_wake_word_model_directory_anchors_to_the_working_directory() {
    // Upstream uses single-argument `resolve()` here, unlike every other path
    // override, so a relative value lands beside the process, not the install.
    let config =
        resolve_with(&[(names::WAKE_WORD_MODEL_DIR, "models/kws")], None).expect("resolves");
    assert_eq!(
        config.wake_word_model_directory,
        std::path::Path::new("/srv/via/models/kws")
    );
}

// ── flags ───────────────────────────────────────────────────────────────────

#[test]
fn computer_use_is_the_one_default_on_toggle() {
    assert!(resolve_with(&[], None).expect("resolves").computer_use);
    for disabled in ["false", "OFF", "0", " no ", "disabled"] {
        assert!(
            !resolve_with(&[(names::COMPUTER_USE, disabled)], None)
                .expect("resolves")
                .computer_use,
            "{disabled:?} must disable it"
        );
    }
    for enabled in ["", "true", "1", "yes", "anything"] {
        assert!(
            resolve_with(&[(names::COMPUTER_USE, enabled)], None)
                .expect("resolves")
                .computer_use,
            "{enabled:?} must leave it enabled"
        );
    }
}

#[test]
fn the_sleep_timeout_is_configured_in_seconds_and_published_in_milliseconds() {
    assert_eq!(
        resolve_with(&[], None).expect("resolves").sleep_timeout_ms,
        0
    );
    assert_eq!(
        resolve_with(&[(names::SLEEP_TIMEOUT_SECONDS, "90")], None)
            .expect("resolves")
            .sleep_timeout_ms,
        90_000
    );
    // Clamped to a day, then multiplied.
    assert_eq!(
        resolve_with(&[(names::SLEEP_TIMEOUT_SECONDS, "999999")], None)
            .expect("resolves")
            .sleep_timeout_ms,
        86_400_000
    );
}

// ── realtime ────────────────────────────────────────────────────────────────

#[test]
fn an_unknown_realtime_model_is_an_error_not_a_deaf_session() {
    // Upstream returns an all-capabilities-false profile here, which opens a
    // session with no audio input. `docs/architecture.md` §7: not ported.
    let error = resolve_with(
        &[
            ("DASHSCOPE_API_KEY", "k"),
            (names::REALTIME_MODEL, "qwen3.5-omni-plus-realtime-future"),
        ],
        None,
    )
    .expect_err("an unknown model id must refuse");
    assert_eq!(error.code(), "VIA_REALTIME_MODEL_UNKNOWN");
}

#[test]
fn an_unknown_realtime_provider_is_refused() {
    let error = resolve_with(&[(names::REALTIME_PROVIDER, "nonesuch")], None)
        .expect_err("an unknown provider must refuse");
    assert_eq!(error.code(), "VIA_REALTIME_PROVIDER_UNSUPPORTED");
}

#[test]
fn the_qwen_alias_still_selects_dashscope() {
    // KEEP under docs/rebrand.md: this names the vendor's realtime runtime.
    let config = resolve_with(&[(names::REALTIME_PROVIDER, "qwen")], None).expect("resolves");
    assert_eq!(config.realtime.provider, "dashscope");
}

#[test]
fn a_workspace_id_rewrites_the_endpoint_but_an_explicit_url_wins() {
    let workspace = resolve_with(&[("DASHSCOPE_WORKSPACE_ID", "ws-42")], None).expect("resolves");
    assert_eq!(
        workspace.realtime.dashscope_realtime_url,
        "wss://ws-42.cn-beijing.maas.aliyuncs.com/api-ws/v1/realtime"
    );

    let explicit = resolve_with(
        &[
            ("DASHSCOPE_WORKSPACE_ID", "ws-42"),
            (names::REALTIME_BASE_URL, "wss://gateway.example/realtime??"),
        ],
        None,
    )
    .expect("resolves");
    assert_eq!(
        explicit.realtime.dashscope_realtime_url,
        "wss://gateway.example/realtime"
    );
}

// ── secrets ─────────────────────────────────────────────────────────────────

#[test]
fn a_credential_never_renders_itself() {
    let config = resolve_with(
        &[
            ("DASHSCOPE_API_KEY", "sk-super-secret"),
            (names::AUTH_SECRET, "another-secret-that-is-long-enough!!"),
            ("OPENCLAW_GATEWAY_TOKEN", "openclaw-secret"),
            ("AGENT_PROTOCOL", "openclaw"),
        ],
        None,
    )
    .expect("resolves");

    let rendered = format!("{config:?}");
    let serialized = serde_json::to_string(&config).expect("Config serializes");
    for secret in ["sk-super-secret", "another-secret", "openclaw-secret"] {
        assert!(!rendered.contains(secret), "{secret} leaked into Debug");
        assert!(
            !serialized.contains(secret),
            "{secret} leaked into serialization"
        );
    }
    // But the value is still reachable where it is needed.
    assert_eq!(
        config.realtime.dashscope_api_key.expose(),
        "sk-super-secret"
    );
    assert_eq!(config.backends.openclaw.token.expose(), "openclaw-secret");
}

// ── env-file parsing ────────────────────────────────────────────────────────

#[test]
fn the_env_parser_handles_the_shapes_a_user_writes() {
    let parsed = parse_env_map(
        "# a comment\n\
         export A=1\n\
         B = spaced \n\
         C=\"quoted # kept\"\n\
         D=trailing # dropped\n\
         E=\n\
         F='single'\n",
    );
    assert_eq!(parsed.get("A").map(String::as_str), Some("1"));
    assert_eq!(parsed.get("B").map(String::as_str), Some("spaced"));
    assert_eq!(parsed.get("C").map(String::as_str), Some("quoted # kept"));
    assert_eq!(parsed.get("D").map(String::as_str), Some("trailing"));
    assert_eq!(parsed.get("E").map(String::as_str), Some(""));
    assert_eq!(parsed.get("F").map(String::as_str), Some("single"));
}

#[test]
fn the_dashscope_compatible_endpoint_is_named_once() {
    // The memory extractor's default and Codex's derived endpoint are the same
    // string; a second copy is how they drift apart.
    let config = resolve_with(&[], None).expect("resolves");
    assert_eq!(
        config.memory_base_url,
        backend::DASHSCOPE_COMPATIBLE_BASE_URL
    );
    let codex = resolve_with(
        &[
            ("AGENT_PROTOCOL", "codex"),
            (names::BACKEND_MODEL, "qwen3.7-max"),
        ],
        None,
    )
    .expect("resolves");
    assert_eq!(
        codex.backends.codex.model_url,
        backend::DASHSCOPE_COMPATIBLE_BASE_URL
    );
    let codebuddy = resolve_with(
        &[
            ("AGENT_PROTOCOL", "codebuddy"),
            (names::BACKEND_MODEL, "qwen3.7-max"),
        ],
        None,
    )
    .expect("resolves");
    assert_eq!(
        codebuddy.backends.codebuddy.model_url,
        format!(
            "{}{}",
            backend::DASHSCOPE_COMPATIBLE_BASE_URL,
            backend::CHAT_COMPLETIONS_SUFFIX
        )
    );
}
