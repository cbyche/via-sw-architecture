//! `via config`, driven through the built binary.
//!
//! The one command phase 1 implements end to end, so it gets the treatment a
//! finished command deserves: the two catalogued outputs are asserted byte for
//! byte against `docs/reference/contracts.json`, and every property the
//! `config.env` rewrite promises is exercised against a real file on a real
//! filesystem.
//!
//! The rewrite is the part worth being careful about. `config.env` is
//! hand-edited: it carries the user's `DASHSCOPE_API_KEY`, comments they
//! wrote, and variables for backends this build has never heard of. A
//! "rewrite" that serialised the keys it understands would delete all of it,
//! and would do so quietly.

mod support;

use support::{Fixture, backticked, contract, rebranded};

/// The catalogued `zh` output of `via config show`, with the model filled in.
fn catalogued_show_output(model: &str) -> String {
    let row = contract("prompt-text", "config show output");
    let template = backticked(&row.exact_value)
        .into_iter()
        .next()
        .expect("the catalogue quotes the whole report");
    rebranded(&template)
        .replace("\\n", "\n")
        .replace("<model>", model)
}

#[test]
fn show_is_the_catalogued_report_byte_for_byte() {
    let fixture = Fixture::new();
    let run = fixture.run(&[("VIA_LOCALE", "zh")], &["config", "show"]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        std::format!(
            "{}\n",
            catalogued_show_output(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL)
        )
    );
}

#[test]
fn show_reports_whichever_model_the_file_names() {
    let fixture = Fixture::new();
    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    fixture.seed_config(&std::format!("VIA_REALTIME_MODEL={model}\n"));
    let run = fixture.run(&[("VIA_LOCALE", "zh")], &["config", "show"]);
    assert_eq!(
        run.stdout,
        std::format!("{}\n", catalogued_show_output(model))
    );
}

#[test]
fn show_creates_nothing() {
    let fixture = Fixture::new();
    let run = fixture.run(&[], &["config", "show"]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(
        !fixture.config_dir().exists(),
        "`config show` is read-only and must not scaffold"
    );
}

#[test]
fn the_bare_form_prints_a_path_that_now_exists() {
    let fixture = Fixture::new();
    let run = fixture.run(&[], &["config"]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout.trim_end(),
        fixture.config_file().display().to_string()
    );
    assert!(fixture.config_file().exists());
}

#[test]
fn the_directory_and_the_file_carry_the_contracted_modes() {
    // *file-path/`config.env` key written by `config set`*: "file mode 0600,
    // directory mode 0700". A configuration file holding a DashScope key is
    // world-readable if this is wrong.
    let row = contract("file-path", "config.env key written by `config set`");
    assert!(row.exact_value.contains("file mode 0600"));
    assert!(row.exact_value.contains("directory mode 0700"));

    let fixture = Fixture::new();
    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    let run = fixture.run(&[], &["config", "set", "--realtime-model", model]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let file = std::fs::metadata(fixture.config_file()).expect("stat file");
        assert_eq!(file.permissions().mode() & 0o777, 0o600);
        let directory = std::fs::metadata(fixture.config_dir()).expect("stat dir");
        assert_eq!(directory.permissions().mode() & 0o777, 0o700);
    }
}

#[test]
fn set_writes_the_catalogued_key_and_keeps_everything_else() {
    let row = contract("file-path", "config.env key written by `config set`");
    let key = rebranded(&row.exact_value)
        .split('=')
        .next()
        .expect("the catalogue names the key")
        .to_owned();

    let fixture = Fixture::new();
    let original = concat!(
        "# a header the user wrote\n",
        "DASHSCOPE_API_KEY=sk-secret\n",
        "\n",
        "# a variable this build has never heard of\n",
        "SOME_FUTURE_BACKEND_TOKEN=keep-me\n",
    );
    fixture.seed_config(original);

    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_FLASH_REALTIME_MODEL;
    let run = fixture.run(&[], &["config", "set", "--realtime-model", model]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);

    let updated = std::fs::read_to_string(fixture.config_file()).expect("read");
    assert_eq!(
        updated,
        std::format!("{original}{key}={model}\n"),
        "the rewrite disturbed something it should not have"
    );
    assert!(updated.contains("sk-secret"), "the credential was lost");
    assert!(updated.contains("keep-me"), "an unknown key was lost");
    assert!(
        updated.contains("# a header the user wrote"),
        "a comment was lost"
    );
}

#[test]
fn set_collapses_duplicates_onto_the_first_position() {
    let fixture = Fixture::new();
    fixture.seed_config(concat!(
        "VIA_REALTIME_MODEL=first\n",
        "A=1\n",
        "  VIA_REALTIME_MODEL = second\n",
        "B=2\n",
    ));
    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    fixture.run(&[], &["config", "set", "--realtime-model", model]);
    assert_eq!(
        std::fs::read_to_string(fixture.config_file()).expect("read"),
        std::format!("VIA_REALTIME_MODEL={model}\nA=1\nB=2\n")
    );
}

#[test]
fn set_preserves_crlf_when_the_file_already_used_it() {
    let fixture = Fixture::new();
    fixture.seed_config("A=1\r\nVIA_REALTIME_MODEL=old\r\n");
    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    fixture.run(&[], &["config", "set", "--realtime-model", model]);
    assert_eq!(
        std::fs::read_to_string(fixture.config_file()).expect("read"),
        std::format!("A=1\r\nVIA_REALTIME_MODEL={model}\r\n")
    );
}

#[test]
fn set_prints_the_catalogued_follow_up() {
    let row = contract("prompt-text", "config set follow-up");
    let quoted = backticked(&row.exact_value);
    let restart = rebranded(&quoted[0]);
    let overridden = rebranded(&quoted[1]);
    assert_ne!(restart, overridden);

    let fixture = Fixture::new();
    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    let run = fixture.run(
        &[("VIA_LOCALE", "zh")],
        &["config", "set", "--realtime-model", model],
    );
    assert_eq!(run.stdout, std::format!("{restart}\n"));

    // The other sentence, which is the one that matters: with the model pinned
    // in the *process* environment, editing the file changes nothing the
    // Gateway will see.
    let pinned = via_catalog::realtime_model::DASHSCOPE_OMNI_FLASH_REALTIME_MODEL;
    let run = fixture.run(
        &[("VIA_LOCALE", "zh"), ("VIA_REALTIME_MODEL", pinned)],
        &["config", "set", "--realtime-model", model],
    );
    assert_eq!(run.stdout, std::format!("{overridden}\n"));
}

#[test]
fn a_model_pinned_in_the_file_does_not_trigger_the_override_warning() {
    // The regression: if the override were read after `config.env` is merged
    // into the environment, every machine that already had a model configured
    // would get the wrong follow-up.
    let row = contract("prompt-text", "config set follow-up");
    let restart = rebranded(&backticked(&row.exact_value)[0]);

    let fixture = Fixture::new();
    let old = via_catalog::realtime_model::DASHSCOPE_OMNI_FLASH_REALTIME_MODEL;
    fixture.seed_config(&std::format!("VIA_REALTIME_MODEL={old}\n"));
    let new = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    let run = fixture.run(
        &[("VIA_LOCALE", "zh")],
        &["config", "set", "--realtime-model", new],
    );
    assert_eq!(run.stdout, std::format!("{restart}\n"));
}

#[test]
fn set_refuses_a_model_outside_the_catalog_and_leaves_the_file_alone() {
    // The refusal is stated in the row's rationale rather than its value:
    // *"`config set` rejects anything outside the four catalog ids with
    // `不支持的 Realtime 模型：<x>`"*.
    let row = support::contract_in("default-value", "shared/realtime-model-catalog.mjs:1");
    assert_eq!(
        row.exact_value,
        via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL
    );
    let refusal = backticked(&row.why)
        .into_iter()
        .find(|quoted| quoted.contains("<x>"))
        .expect("the catalogue quotes the refusal");
    let expected = refusal.replace("<x>", "not-a-model");

    let fixture = Fixture::new();
    fixture.seed_config("A=1\n");
    let run = fixture.run(
        &[("VIA_LOCALE", "zh")],
        &["config", "set", "--realtime-model", "not-a-model"],
    );
    assert_eq!(run.code, Some(1));
    assert_eq!(run.stderr, std::format!("via: {expected}\n"));
    assert_eq!(run.error_code(), Some("VIA_REALTIME_MODEL_UNKNOWN"));
    assert_eq!(
        std::fs::read_to_string(fixture.config_file()).expect("read"),
        "A=1\n",
        "a refused model must not reach the writer"
    );
}

#[test]
fn every_catalogued_model_id_round_trips_through_set_and_show() {
    let fixture = Fixture::new();
    for profile in via_catalog::dashscope_realtime_model_profiles() {
        let run = fixture.run(&[], &["config", "set", "--realtime-model", &profile.id]);
        assert_eq!(run.code, Some(0), "{}: {}", profile.id, run.stderr);
        let shown = fixture.run(&[("VIA_LOCALE", "zh")], &["config", "show"]);
        assert_eq!(
            shown.stdout,
            std::format!("{}\n", catalogued_show_output(&profile.id))
        );
    }
}

#[test]
fn set_is_idempotent_on_disk() {
    let fixture = Fixture::new();
    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    fixture.run(&[], &["config", "set", "--realtime-model", model]);
    let once = std::fs::read_to_string(fixture.config_file()).expect("read");
    fixture.run(&[], &["config", "set", "--realtime-model", model]);
    let twice = std::fs::read_to_string(fixture.config_file()).expect("read");
    assert_eq!(once, twice);
}

#[test]
fn set_on_a_seeded_template_edits_it_rather_than_replacing_it() {
    // The realistic path: `via config` seeds the template, then `via config
    // set` edits it. Every commented line the template ships must survive.
    let fixture = Fixture::new();
    fixture.run(&[], &["config"]);
    let seeded = std::fs::read_to_string(fixture.config_file()).expect("read");
    let comments: Vec<&str> = seeded
        .lines()
        .filter(|line| line.trim_start().starts_with('#'))
        .collect();
    assert!(comments.len() > 5, "the template lost its comments already");

    let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
    fixture.run(&[], &["config", "set", "--realtime-model", model]);
    let updated = std::fs::read_to_string(fixture.config_file()).expect("read");
    for comment in comments {
        assert!(updated.contains(comment), "lost `{comment}`");
    }
    assert!(updated.contains("DASHSCOPE_API_KEY="));
}

#[test]
fn set_without_a_value_is_a_usage_failure_and_writes_nothing() {
    let fixture = Fixture::new();
    fixture.seed_config("A=1\n");
    let run = fixture.run(&[], &["config", "set", "--realtime-model"]);
    assert_eq!(run.code, Some(1));
    assert_eq!(
        std::fs::read_to_string(fixture.config_file()).expect("read"),
        "A=1\n"
    );
}
