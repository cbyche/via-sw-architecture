//! Every catalogued contract `apps/via` owns, asserted against
//! `docs/reference/contracts.json` rather than against a retyped copy.
//!
//! The slice is 14 `cli-flag` rows, 4 `cli-commands` rows, one `binary-name`
//! and one `exit-code`, plus the CLI-side `error-code`, `prompt-text` and
//! `file-path` rows that come with them. Where VIA renames an upstream value,
//! the rename is *applied to the catalogued string* by
//! [`support::rebranded`] — so a build that renamed half of a sentence fails
//! here, which retyping the finished string would not catch.
//!
//! Where VIA does **not** reproduce a row, the test says so explicitly and
//! names the reason. A silently-absent contract is the failure mode this file
//! exists to prevent.

mod support;

use std::collections::BTreeSet;

use support::{Fixture, backticked, contract, contract_in, list_after, quoted_after, rebranded};
use via::cli::{DEFAULT_COMMAND, URL_ENV};
use via::error::{EXIT_FAILURE, EXIT_SUCCESS};
use via::origin::{ALLOWED_SCHEMES, DEFAULT_GATEWAY_URL, UrlLabel, clean_origin};
use via::session::{SESSION_ID_ENV, VOICE_SESSION_HEX_LENGTH, VOICE_SESSION_PREFIX};
use via::{BINARY_NAME, LOG_COMPONENT, LOG_EVENTS, STDERR_PREFIX};
use via_i18n::Locale;

#[test]
fn the_binary_name_is_the_catalogued_one_with_the_rename_applied() {
    let row = contract_in("binary-name", "cli/package.json:7, package.json:50");
    assert_eq!(rebranded(&row.exact_value), BINARY_NAME);
    // And the rename is not a no-op: the catalogued value is somebody else's.
    assert_ne!(row.exact_value, BINARY_NAME);
}

#[test]
fn the_exit_codes_are_zero_and_one() {
    let row = contract("exit-code", "process exit codes");
    assert!(
        row.exact_value
            .starts_with("0 = success; 1 = any thrown error"),
        "the catalogue moved: {}",
        row.exact_value
    );
    assert_eq!(EXIT_SUCCESS, 0);
    assert_eq!(EXIT_FAILURE, 1);
}

#[test]
fn the_stderr_prefix_is_the_catalogued_one() {
    let row = contract("error-code", "stderr prefix");
    // `` `<binary>: ${error.message}\n` ``
    let template = backticked(&row.exact_value)
        .into_iter()
        .next()
        .expect("the catalogue quotes the template");
    let expected = rebranded(&template);
    assert!(
        expected.starts_with(STDERR_PREFIX),
        "{expected:?} does not start with {STDERR_PREFIX:?}"
    );
    assert_eq!(STDERR_PREFIX, std::format!("{BINARY_NAME}: "));
}

#[test]
fn a_refusal_is_written_with_that_prefix_and_exits_one() {
    let fixture = Fixture::new();
    let run = fixture.run(&[], &["gateway"]);
    assert!(run.stderr.starts_with(STDERR_PREFIX), "{}", run.stderr);
    assert!(run.stderr.ends_with('\n'), "{:?}", run.stderr);
    assert_eq!(run.code, Some(i32::from(EXIT_FAILURE)));
}

#[test]
fn help_and_version_exit_zero_before_any_command_runs() {
    // *cli-flag/--help / -h*: "prints helpText() + '\n' and returns exit code
    // 0, before any command runs". The observable half of "before any command
    // runs" is that nothing was scaffolded.
    let fixture = Fixture::new();
    for flag in ["-h", "--help", "-V", "--version"] {
        let run = fixture.run(&[], &[flag]);
        assert_eq!(run.code, Some(i32::from(EXIT_SUCCESS)), "{flag}");
        assert!(!run.stdout.is_empty(), "{flag} printed nothing");
        assert!(run.stderr.is_empty(), "{flag} wrote to stderr");
    }
    assert!(
        !fixture.config_file().exists(),
        "--help scaffolded a machine it was only asked about"
    );
}

#[test]
fn the_default_gateway_url_and_its_variable_come_from_the_url_contract() {
    let row = contract("cli-flag", "--url");
    let default = quoted_after(&row.exact_value, "||").expect("the catalogue quotes the default");
    assert_eq!(default, DEFAULT_GATEWAY_URL);
    assert!(
        rebranded(&row.exact_value).contains(URL_ENV),
        "the renamed variable is not in {}",
        row.exact_value
    );
    assert!(row.exact_value.contains("http/https only"));
    assert_eq!(ALLOWED_SCHEMES, ["http", "https"]);
}

#[test]
fn the_session_default_and_its_variable_come_from_the_session_contract() {
    let row = contract("cli-flag", "--session");
    assert!(rebranded(&row.exact_value).contains(SESSION_ID_ENV));
    assert!(row.exact_value.contains("createVoiceSessionId()"));

    let generated = contract("default-value", "createVoiceSessionId()");
    let shape = backticked(&generated.exact_value)
        .into_iter()
        .next()
        .expect("the catalogue quotes the shape");
    assert!(shape.starts_with(VOICE_SESSION_PREFIX), "{shape}");
    // `voice-3f2b1c9d4e5a4f8b9c0d1e2f3a4b5c6d`
    let example = generated
        .exact_value
        .rsplit("e.g. ")
        .next()
        .and_then(|rest| rest.split(')').next())
        .expect("the catalogue gives an example");
    assert_eq!(
        example.trim().len(),
        VOICE_SESSION_PREFIX.len() + VOICE_SESSION_HEX_LENGTH,
        "{example}"
    );
}

#[test]
fn the_backend_id_list_is_the_catalogs_in_catalog_order() {
    let row = contract("cli-flag", "--backend");
    let catalogued = list_after(&row.exact_value, "valid ids: ");
    assert_eq!(catalogued, via_catalog::backend_names());
}

#[test]
fn the_two_permission_modes_are_the_catalogued_ones() {
    let row = contract("cli-flag", "--backend-permission-mode");
    assert!(rebranded(&row.exact_value).contains("VIA_BACKEND_PERMISSION_MODE"));
    assert!(
        row.exact_value
            .contains(via::commands::gateway::DEFAULT_BACKEND_PERMISSION_MODE)
    );
    assert!(
        row.exact_value
            .contains(via::commands::gateway::FULL_PERMISSION_MODE)
    );
    assert_eq!(
        via_core::config::BACKEND_PERMISSION_MODES,
        [
            via::commands::gateway::DEFAULT_BACKEND_PERMISSION_MODE,
            via::commands::gateway::FULL_PERMISSION_MODE
        ]
    );
}

#[test]
fn the_backend_base_urls_are_the_catalogued_defaults() {
    let row = contract("cli-flag", "--backend-url");
    for (id, variable, default) in [
        ("opencode", "OPENCODE_BASE_URL", "http://127.0.0.1:4096"),
        ("openclaw", "OPENCLAW_BASE_URL", "http://127.0.0.1:18789"),
    ] {
        assert!(row.exact_value.contains(variable), "{variable}");
        assert!(row.exact_value.contains(default), "{default}");
        let definition = via_catalog::backend_definition(id).expect("in the catalog");
        assert_eq!(definition.base_url_environment, Some(variable));
        assert_eq!(definition.default_base_url, Some(default));
    }
}

#[test]
fn the_invalid_url_messages_are_the_catalogued_ones() {
    let row = contract("error-code", "invalid URL");
    let quoted = backticked(&row.exact_value);
    // `无效的 Gateway URL：<value>` / `无效的后台地址：<value>` and the two
    // scheme sentences. Rendered here with `<value>` as the literal value, so
    // the comparison is against the catalogued shape with nothing filled in.
    let rendered: BTreeSet<String> = [
        clean_origin("<value>", UrlLabel::Gateway, Locale::Zh),
        clean_origin("<value>", UrlLabel::Backend, Locale::Zh),
        clean_origin("ws://x", UrlLabel::Gateway, Locale::Zh),
        clean_origin("ws://x", UrlLabel::Backend, Locale::Zh),
    ]
    .into_iter()
    .map(|outcome| {
        outcome
            .expect_err("every input here is refused")
            .message(Locale::Zh)
    })
    .collect();
    let catalogued: BTreeSet<String> = quoted.into_iter().collect();
    assert_eq!(rendered, catalogued);
}

#[test]
fn the_install_target_messages_are_the_catalogued_ones() {
    let row = contract("error-code", "install target validation");
    let quoted = backticked(&row.exact_value);
    let names = via_catalog::backend_names().join("、");
    assert!(
        quoted
            .iter()
            .any(|sentence| sentence.contains(&names) && sentence.starts_with("install")),
        "the catalogue's id list is not the catalog's: {quoted:?}"
    );
    // The generic-ACP refusal, byte for byte.
    let acp = quoted
        .iter()
        .find(|sentence| sentence.contains("ACP_COMMAND"))
        .expect("the catalogue names the generic-ACP refusal");
    assert_eq!(
        acp.as_str(),
        via_i18n::t(Locale::Zh, via_i18n::keys::CLI_INSTALL_GENERIC_ACP)
    );
}

#[test]
fn the_config_actions_are_show_set_and_nothing() {
    let row = contract("cli-commands", "config actions");
    assert!(row.exact_value.starts_with("show, set"));
    assert!(
        row.exact_value
            .contains("no action = print the config file path")
    );
    // The unknown-action sentence, with the rename applied.
    let refusal = backticked(&row.exact_value)
        .into_iter()
        .next()
        .expect("the catalogue quotes the refusal");
    let expected = refusal.replace("<x>", "bogus");
    let fixture = Fixture::new();
    let run = fixture.run(&[("VIA_LOCALE", "zh")], &["config", "bogus"]);
    assert_eq!(run.stderr, std::format!("{STDERR_PREFIX}{expected}\n"));
    assert_eq!(run.code, Some(1));
}

#[test]
fn the_default_command_is_the_catalogued_one() {
    let row = contract("cli-commands", "COMMANDS");
    assert!(
        row.exact_value
            .contains("default when argv[0] is absent or starts with '-'"),
        "{}",
        row.exact_value
    );
    let catalogued_default = row
        .exact_value
        .rsplit(": ")
        .next()
        .unwrap_or_default()
        .trim_end_matches(')');
    assert_eq!(catalogued_default, DEFAULT_COMMAND);

    // A bare `via` and a `via --flag` are both Gateway runs, which on an
    // unconfigured machine means the setup refusal rather than a usage error.
    let fixture = Fixture::new();
    for args in [vec![], vec!["--backend", "none"]] {
        let run = fixture.run(&[], &args);
        assert_eq!(
            run.error_code(),
            Some(via_protocol::CODE_GATEWAY_SETUP_REQUIRED),
            "{args:?} did not default to `{DEFAULT_COMMAND}`: {}",
            run.stderr
        );
    }
}

#[test]
fn the_service_verbs_are_the_catalogued_gateway_actions_minus_the_owed_four() {
    let row = contract("cli-commands", "GATEWAY_ACTIONS");
    let catalogued: BTreeSet<&str> = row
        .exact_value
        .split(" (default")
        .next()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .collect();
    let implemented: BTreeSet<&str> = via::commands::service::SERVICE_ACTIONS
        .into_iter()
        .collect();
    assert!(
        implemented.is_subset(&catalogued),
        "VIA invented a service verb: {implemented:?} vs {catalogued:?}"
    );
    // `run` became `via gateway`; the other three are owed, and
    // `docs/deviations/phase-1.md` records them.
    let owed: BTreeSet<&str> = catalogued.difference(&implemented).copied().collect();
    assert_eq!(
        owed,
        ["restart", "run", "status", "uninstall"]
            .into_iter()
            .collect()
    );
}

#[test]
fn the_skill_actions_are_catalogued_and_deliberately_absent() {
    let row = contract("cli-commands", "SKILL_ACTIONS");
    assert!(row.exact_value.starts_with("install, list, remove, update"));
    // `docs/architecture.md` §10 has no `skill` verb: it is a branded
    // passthrough over `skills.sh`, which the Rust binary does not carry. The
    // test asserts the absence deliberately rather than leaving it unstated.
    let fixture = Fixture::new();
    let run = fixture.run(&[], &["skill", "list"]);
    assert_eq!(run.code, Some(1));
    assert!(run.stdout.is_empty());
}

/// Every catalogued top-level verb, and what `apps/via` does with it.
///
/// `docs/architecture.md` §10 replaced upstream's eight with six, and this is
/// the mapping. A verb added to the catalogue fails
/// [`every_catalogued_verb_has_a_recorded_disposition`] until somebody
/// decides what VIA does about it.
const VERB_DISPOSITION: [(&str, &str); 8] = [
    ("gateway", "kept; the service actions move to `via service`"),
    ("config", "kept, and complete"),
    ("tui", "becomes `via chat`, per docs/fidelity.md"),
    ("setup", "becomes `via backend status`"),
    ("install", "becomes `via backend install`"),
    ("webui", "dropped with web/, per docs/fidelity.md"),
    ("status", "an alias for `gateway status`, owed with it"),
    (
        "skill",
        "dropped: a branded passthrough over skills.sh, absent from §10",
    ),
];

#[test]
fn every_catalogued_verb_has_a_recorded_disposition() {
    let row = contract("cli-commands", "COMMANDS");
    let catalogued: BTreeSet<String> = row
        .exact_value
        .split(" (default")
        .next()
        .unwrap_or_default()
        .split(',')
        .map(|verb| verb.trim().to_owned())
        .collect();
    let recorded: BTreeSet<String> = VERB_DISPOSITION
        .iter()
        .map(|(verb, _)| (*verb).to_owned())
        .collect();
    assert_eq!(catalogued, recorded);

    // The six §10 verbs, each of which must parse. `chat` carries
    // `--no-autostart` because parsing is all this asserts, and without it the
    // verb does what it is for: starts a Gateway and waits for it.
    let fixture = Fixture::new();
    for args in [
        vec!["gateway"],
        vec!["chat", "--no-autostart"],
        vec!["config"],
        vec!["backend", "status"],
        vec!["mcp-serve"],
        vec!["service", "start"],
    ] {
        let run = fixture.run(&[], &args);
        assert!(
            !run.stderr.contains("unrecognized subcommand"),
            "{args:?}: {}",
            run.stderr
        );
    }
}

/// Every catalogued `cli-flag`, and what `apps/via` does with it.
///
/// The table is asserted to be exactly the catalogue's 14 rows, so a flag
/// added to the catalogue fails this test until somebody decides what VIA
/// does about it.
const FLAG_DISPOSITION: [(&str, &str); 14] = [
    ("--url", "implemented on gateway, chat and mcp-serve"),
    (
        "--backend",
        "implemented on gateway; also selects one backend for `backend status`",
    ),
    ("--backend-permission-mode", "implemented on gateway"),
    ("--backend-url", "implemented on gateway"),
    ("--backend-agent", "implemented on gateway"),
    ("--realtime-model", "implemented on `config set`"),
    ("--session", "implemented on chat"),
    (
        "--json",
        "implemented on `backend status`, which is upstream's `setup`",
    ),
    (
        "--yes / -y",
        "implemented on `backend install`, which is upstream's `install`",
    ),
    ("--help / -h", "clap's, exiting 0"),
    (
        "--takeover",
        "moved from tui/webui to chat, the client that replaces them",
    ),
    (
        "--audio-mode",
        "deferred with the TUI: `via chat` carries no audio",
    ),
    ("--no-open", "dropped with `webui`, per docs/fidelity.md"),
    (
        "--skill / --list",
        "dropped with the `skill` verb, which §10 does not have",
    ),
];

#[test]
fn every_catalogued_flag_has_a_recorded_disposition() {
    let catalogued: BTreeSet<String> = support::contracts()
        .into_iter()
        .filter(|row| row.kind == "cli-flag")
        .map(|row| row.name)
        .collect();
    let recorded: BTreeSet<String> = FLAG_DISPOSITION
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();
    assert_eq!(
        catalogued, recorded,
        "the catalogue and the disposition table disagree"
    );
    assert_eq!(catalogued.len(), 14);
}

#[test]
fn the_flags_via_kept_are_accepted_and_the_ones_it_dropped_are_not() {
    let fixture = Fixture::new();
    let accepted: [(&[&str], &str); 10] = [
        (&["gateway", "--url", "http://127.0.0.1:1"], "--url"),
        (&["gateway", "--backend", "none"], "--backend"),
        (
            &["gateway", "--backend-permission-mode", "native"],
            "--backend-permission-mode",
        ),
        (
            &["gateway", "--backend-url", "http://127.0.0.1:1"],
            "--backend-url",
        ),
        (&["gateway", "--backend-agent", "build"], "--backend-agent"),
        // `--no-autostart` on both: this case is about the *parser*, and
        // without it `via chat` would do exactly what it is supposed to —
        // start a Gateway and wait thirty seconds for it.
        (&["chat", "--session", "s", "--no-autostart"], "--session"),
        (&["chat", "--takeover", "--no-autostart"], "--takeover"),
        (&["chat", "--no-autostart"], "--no-autostart"),
        (&["backend", "status", "--json"], "--json"),
        (&["backend", "install", "codex", "--yes"], "--yes"),
    ];
    for (args, flag) in accepted {
        let run = fixture.run(&[], args);
        assert!(
            !run.stderr.contains("unexpected argument"),
            "{flag} was rejected: {}",
            run.stderr
        );
    }

    let dropped: [(&[&str], &str); 4] = [
        (&["chat", "--audio-mode", "half"], "--audio-mode"),
        (&["chat", "--no-open"], "--no-open"),
        (&["chat", "--skill", "x"], "--skill"),
        (&["chat", "--list"], "--list"),
    ];
    for (args, flag) in dropped {
        let run = fixture.run(&[], args);
        assert_eq!(run.code, Some(1), "{flag} should not be accepted");
        assert!(
            run.stderr.contains("unexpected argument"),
            "{flag}: {}",
            run.stderr
        );
    }
}

#[test]
fn the_log_component_and_events_are_the_catalogued_ones() {
    assert_eq!(LOG_COMPONENT, "cli");
    assert_eq!(LOG_EVENTS, ["cli.started", "cli.completed", "cli.failed"]);
    let fixture = Fixture::new();
    let run = fixture.run(&[], &["config"]);
    assert_eq!(run.code, Some(0));
    assert_eq!(run.events(), vec![LOG_EVENTS[0], LOG_EVENTS[1]]);
    assert_eq!(run.records[1]["exitCode"], 0);
}
