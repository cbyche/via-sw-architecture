//! **First run** — what an empty machine ends up with.
//!
//! `apps/via/tests/setup_gate.rs` already asserts the refusal sentence in three
//! locales and that a refused start creates no lease. Neither of those is
//! repeated here. What this file asserts is the thing only a real process on a
//! real filesystem can show: **every path the binary created, and its mode**,
//! compared against `docs/reference/contracts.json`'s `file-path` rows rather
//! than against a list somebody typed.
//!
//! # The ordering that surprises people
//!
//! Scaffolding happens **before** the gate. `via::commands::dispatch` calls
//! `Host::load_runtime(read_only = false)` on its way in, so by the time
//! `assert_gateway_setup` refuses, `config.env` and `state.env` already exist.
//! That is upstream's order and it is the right one — a user who is told to set
//! `DASHSCOPE_API_KEY` needs a `config.env` to set it in. What must *not* have
//! happened is the lease, and that is what the first case pins down: the
//! machine is scaffolded and the lease is untouched, in the same run.

mod support;

use pretty_assertions::assert_eq;
use support::Machine;
use via_core::paths;

/// The directories the catalogue says are `0o700`, relative to the config
/// directory.
///
/// Read from `via-core`'s own constants rather than spelled here: the mode is a
/// contract, the *names* belong to the crate that resolves them.
const CATALOGUED_DIRECTORIES: [&str; 2] =
    [paths::LOG_DIRECTORY_NAME, paths::OPENCLAW_STATE_DIRECTORY];

/// The files the catalogue says are `0o600` and a first run creates.
const CATALOGUED_FILES: [&str; 4] = [
    paths::CONFIG_FILE_NAME,
    paths::STATE_FILE_NAME,
    paths::USER_MODEL_FILE_NAME,
    paths::MEMORY_FILE_NAME,
];

/// Every catalogued sentence that names a path the product creates.
///
/// Two rows, unioned, because the survey split the same fact across them: one
/// enumerates the directories and their modes, the other the managed files.
fn catalogued_paths() -> String {
    format!(
        "{}\n{}\n{}",
        support::catalogued_raw("file-path", "files and directories the product creates"),
        support::catalogued_raw(
            "file-path",
            "managed files inside the config/data directories"
        ),
        support::catalogued_raw("file-path", "default state/asset paths"),
    )
}

#[test]
fn an_unconfigured_first_run_scaffolds_the_machine_and_leaves_the_lease_alone() {
    let machine = Machine::new();
    assert!(
        !machine.config_dir().exists(),
        "the fixture must not have created the directory the binary is being asked to create",
    );

    let run = machine.run_via(&[], &["gateway"]);

    assert_eq!(run.code, Some(1), "stderr: {}", run.stderr);
    assert_eq!(
        run.error_code(),
        Some(via_protocol::CODE_GATEWAY_SETUP_REQUIRED),
        "records: {:?}",
        run.records,
    );
    assert!(
        run.stderr
            .contains(via_core::config::names::DASHSCOPE_API_KEY),
        "the refusal names the key an operator has to set: {}",
        run.stderr,
    );

    // Scaffolded…
    assert!(
        machine.config_dir().join(paths::CONFIG_FILE_NAME).exists(),
        "the gate runs after `load_runtime`, so the user has a file to edit",
    );
    // …and the lease untouched, in the same run. `server/src/index.mjs:50`
    // gates before the lease exists precisely so a misconfigured start cannot
    // disturb a running Gateway.
    assert!(
        !machine.lease_file().exists(),
        "a refused start must not have taken the single-instance lease",
    );
    assert!(
        !machine.tasks_file().exists(),
        "…nor created a Work store it never used",
    );
}

#[test]
fn every_path_a_first_run_creates_is_a_catalogued_one() {
    let machine = Machine::new();
    let gateway = machine.start_via_gateway(&[("VIA_ALLOW_UNCONFIGURED", "1")]);
    // `VIA_ALLOW_UNCONFIGURED=1` is the documented opt-out and must start a
    // Gateway; anything else and the audit below would be auditing nothing.
    gateway.require_serving();
    assert!(machine.lease_file().exists(), "…which holds the lease");

    let catalogue = catalogued_paths();
    // The audit is only worth anything if the catalogue does not mention
    // everything. This is the vacuity guard.
    assert!(
        !catalogue.contains("surprise.txt"),
        "the catalogue cannot be matching arbitrary names",
    );

    // `logs/cli.log` is the one created path the survey does not carry: it is
    // the *CLI's* log file (`cli/bin/<binary>.mjs` creates the logger with
    // `fileName: 'cli.log'`), and the survey catalogued only the Gateway's.
    // Derived from `via::LOG_COMPONENT` rather than spelled, so a renamed
    // component moves this with it.
    let cli_log = format!("{}/{}.log", paths::LOG_DIRECTORY_NAME, via::LOG_COMPONENT);

    let created = support::tree(&machine.config_dir());
    assert!(!created.is_empty(), "a first run created nothing at all");
    for entry in &created {
        let bare = entry.trim_end_matches('/');
        let leaf = bare.rsplit('/').next().unwrap_or(bare);
        assert!(
            catalogue.contains(bare) || catalogue.contains(leaf) || bare == cli_log,
            "`{entry}` is not named by any `file-path` contract. Everything the product \
             creates is catalogued; a new artifact needs a catalogue row before it needs \
             a test. Created: {created:?}",
        );
    }

    // The two modes are contracts of their own — `via_core::paths`'
    // DIRECTORY_MODE and FILE_MODE, from `shared/runtime-environment.mjs`.
    #[cfg(unix)]
    {
        assert_eq!(
            support::mode_of(&machine.config_dir()),
            Some(paths::DIRECTORY_MODE),
            "the configuration directory",
        );
        for name in CATALOGUED_DIRECTORIES {
            let path = machine.config_path(name);
            assert!(path.is_dir(), "{name} was not created");
            assert_eq!(
                support::mode_of(&path),
                Some(paths::DIRECTORY_MODE),
                "{name}"
            );
        }
        for name in CATALOGUED_FILES {
            let path = machine.config_path(name);
            assert!(path.is_file(), "{name} was not created");
            assert_eq!(support::mode_of(&path), Some(paths::FILE_MODE), "{name}");
        }
    }

    // `state.env` holds the generated local identity: 64 lowercase hex
    // characters, which is what every client cookie is signed with.
    let state =
        std::fs::read_to_string(machine.config_path(paths::STATE_FILE_NAME)).expect("state.env");
    let secret = state
        .trim()
        .strip_prefix(&format!("{}=", via_core::config::names::AUTH_SECRET))
        .expect("state.env carries exactly the secret assignment");
    assert_eq!(secret.len(), via_core::runtime::AUTH_SECRET_HEX_LENGTH);
    assert!(
        secret
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
        "{secret}",
    );

    // `config.env`'s first line is a catalogued contract, and it is localized:
    // the machine ran under `LANG=C`, which resolves to `en`.
    let config =
        std::fs::read_to_string(machine.config_path(paths::CONFIG_FILE_NAME)).expect("config.env");
    let first = config.lines().next().unwrap_or_default();
    assert_eq!(
        first,
        via_i18n::t(via_i18n::Locale::En, via_i18n::keys::RUNTIME_CONFIG_HEADER),
    );
    // …and the catalogued value is the `zh` one, with only the product name
    // renamed. `# qwen-audio-agent 用户配置` → `# VIA 用户配置`.
    assert!(
        support::catalogued_raw("file-path", "runtime config file (the real one)")
            .contains("# qwen-audio-agent 用户配置"),
        "the catalogue moved",
    );
    assert_eq!(
        via_i18n::t(via_i18n::Locale::Zh, via_i18n::keys::RUNTIME_CONFIG_HEADER),
        "# VIA 用户配置",
    );

    // The Gateway's own log file is inside the config directory, which is what
    // makes `tests/security.rs`'s grep a grep of the real thing.
    assert!(
        machine.gateway_log().exists(),
        "a serving Gateway writes `logs/gateway.log`",
    );

    let run = gateway.stop();
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    assert!(
        !machine.lease_file().exists(),
        "the lease is given back on the way out, not leaked",
    );
}

#[test]
fn a_second_run_re_uses_the_first_runs_secret_rather_than_regenerating_it() {
    // Every seed is create-if-absent (`O_CREAT | O_EXCL`), and the secret is
    // the one where it matters: regenerating invalidates every existing client
    // cookie. Two real runs against one directory is the only way to see it.
    let machine = Machine::new();
    let first = machine.start_via_gateway(&[("VIA_ALLOW_UNCONFIGURED", "1")]);
    first.require_serving();
    let secret =
        std::fs::read_to_string(machine.config_path(paths::STATE_FILE_NAME)).expect("state.env");
    let _ = first.stop();

    // A user edit that must survive, in a file the seeder would otherwise
    // rewrite.
    let user_model = machine.config_path(paths::USER_MODEL_FILE_NAME);
    std::fs::write(&user_model, "# mine\n").expect("edit USER.md");

    let second = machine.start_via_gateway(&[("VIA_ALLOW_UNCONFIGURED", "1")]);
    second.require_serving();
    assert_eq!(
        std::fs::read_to_string(machine.config_path(paths::STATE_FILE_NAME)).ok(),
        Some(secret),
        "the auth secret is adopted, never regenerated",
    );
    assert_eq!(
        std::fs::read_to_string(&user_model).ok(),
        Some("# mine\n".to_owned()),
        "a seeded document a user edited is kept, not replaced",
    );
    let _ = second.stop();
}
