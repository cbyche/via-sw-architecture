//! The setup gate and the instance lease, exercised through the built binary.
//!
//! `docs/architecture.md` §15: phase 1 ends when *"`via gateway` refuses to
//! start unconfigured, with the exact missing-key message, in all three
//! locales"*. Phase 5 kept every one of those refusals and changed only what
//! happens when none of them fires — the Gateway now serves — so the cases
//! that used to end at the phase-5 stub now start a Gateway and stop it with a
//! signal. Everything here drives `CARGO_BIN_EXE_via` under a synthetic
//! machine, because the three things being asserted — the exit code, the
//! stream the sentence lands on, and the `cli.log` record carrying the error
//! code — do not exist inside the library.
//!
//! The `zh` sentence is the catalogued contract and is asserted **byte for
//! byte against `docs/reference/contracts.json`**, not against a retyped
//! copy. `en` and `ko` are authored peers (`docs/architecture.md` §16), so
//! they are asserted against the shipped `via-i18n` catalog and against each
//! other.

mod support;

use rstest::rstest;
use support::{Fixture, SIGINT, write_lease};
use via_i18n::{Locale, format, keys, t};
use via_lock::{AcquireOptions, LeaseUpdate, acquire_gateway_lease};
use via_protocol::CODE_GATEWAY_SETUP_REQUIRED;

/// The rendered refusal `locale` must produce, built from `via-i18n`.
///
/// This is the *shipped* sentence. `the_chinese_refusal_is_the_catalogued
/// _sentence` then checks the `zh` column of it against the catalogue, which
/// is what makes asserting the other two against this function meaningful
/// rather than circular.
fn expected_refusal(locale: Locale) -> String {
    let detail = format(
        locale,
        keys::GATEWAY_SETUP_REQUIRED_ITEM,
        &[
            ("key", "DASHSCOPE_API_KEY"),
            (
                "message",
                t(locale, keys::GATEWAY_MISSING_DASHSCOPE_API_KEY),
            ),
        ],
    );
    format(
        locale,
        keys::GATEWAY_SETUP_REQUIRED,
        &[("details", detail.as_str())],
    )
}

#[rstest]
#[case("en")]
#[case("zh")]
#[case("ko")]
fn the_gateway_refuses_to_start_unconfigured_in_every_locale(#[case] locale: &str) {
    let fixture = Fixture::new();
    let run = fixture.run(&[("VIA_LOCALE", locale)], &["gateway"]);

    assert_eq!(run.code, Some(1), "stderr: {}", run.stderr);
    assert_eq!(
        run.error_code(),
        Some(CODE_GATEWAY_SETUP_REQUIRED),
        "records: {:?}",
        run.records
    );

    let locale = locale.parse::<Locale>().expect("a VIA locale");
    assert_eq!(
        run.stderr,
        std::format!("via: {}\n", expected_refusal(locale))
    );
    assert!(
        run.stdout.is_empty(),
        "a refusal must not write to stdout: {:?}",
        run.stdout
    );
}

#[test]
fn the_chinese_refusal_is_the_catalogued_sentence_byte_for_byte() {
    // Upstream's own text, read out of the catalogue rather than retyped, with
    // only the identity substitution `docs/rebrand.md` mandates applied.
    // Looked up by upstream location rather than by name: the catalogue holds
    // the same error code three times, surveyed from three call sites.
    let template = support::quoted_after(
        &support::contract_in("error-code", "shared/gateway-setup.mjs:14-52").exact_value,
        "Error message ",
    )
    .expect("the catalogue states the message template");
    let missing = support::quoted_after(
        &support::contract("error-code", "missing configuration messages").exact_value,
        "setup-gate variant: ",
    )
    .expect("the catalogue states the setup-gate variant");

    let expected = support::rebranded(&template)
        .replace("<KEY>", "DASHSCOPE_API_KEY")
        .replace("<message>", &support::rebranded(&missing));

    assert_eq!(expected_refusal(Locale::Zh), expected);

    let fixture = Fixture::new();
    let run = fixture.run(&[("VIA_LOCALE", "zh")], &["gateway"]);
    assert_eq!(run.stderr, std::format!("via: {expected}\n"));
}

#[test]
fn the_three_locales_are_three_different_sentences() {
    let fixture = Fixture::new();
    let rendered: Vec<String> = ["en", "zh", "ko"]
        .into_iter()
        .map(|locale| fixture.run(&[("VIA_LOCALE", locale)], &["gateway"]).stderr)
        .collect();
    assert_ne!(rendered[0], rendered[1]);
    assert_ne!(rendered[1], rendered[2]);
    assert_ne!(rendered[0], rendered[2]);
    for sentence in &rendered {
        // The key an operator has to set is never localized — only the
        // sentence around it (`docs/architecture.md` §16).
        assert!(sentence.contains("DASHSCOPE_API_KEY"), "{sentence}");
        assert!(!sentence.contains("<via-i18n:"), "{sentence}");
        assert!(!sentence.contains('{'), "{sentence}");
    }
}

#[test]
fn the_refusal_never_touches_the_instance_lease() {
    // `server/src/index.mjs:50` gates before the lease exists, so a
    // misconfigured start cannot disturb a running Gateway. Here there is no
    // running Gateway, so the observable is that no lease file is created.
    let fixture = Fixture::new();
    let run = fixture.run(&[], &["gateway"]);
    assert_eq!(run.code, Some(1));
    assert!(
        !fixture.lease_file().exists(),
        "the setup gate must run before the lease is touched"
    );
}

#[test]
fn the_setup_gate_wins_over_a_lease_conflict() {
    // Both refusals are available at once. The gate must be the one reported,
    // because it is the one the user can act on — and because reporting the
    // conflict would mean the lease had already been probed.
    let fixture = Fixture::new();
    write_lease(
        &fixture.lease_file(),
        i64::from(std::process::id()),
        "http://127.0.0.1:3101",
    );
    let run = fixture.run(&[("VIA_LOCALE", "zh")], &["gateway"]);
    assert_eq!(run.error_code(), Some(CODE_GATEWAY_SETUP_REQUIRED));
    assert_eq!(
        run.stderr,
        std::format!("via: {}\n", expected_refusal(Locale::Zh))
    );
}

#[rstest]
#[case("1")]
fn allow_unconfigured_opts_out_of_the_gate(#[case] value: &str) {
    let fixture = Fixture::new();
    let running = fixture.start(&[("VIA_ALLOW_UNCONFIGURED", value)], &["gateway"]);
    assert!(
        running.is_serving(),
        "past the gate, past the lease, and listening",
    );
    let run = running.stop();
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
}

#[rstest]
#[case("true")]
#[case("TRUE")]
#[case("yes")]
#[case("0")]
#[case("")]
#[case("01")]
#[case(" 1")]
#[case("1 ")]
fn only_the_exact_string_one_opts_out(#[case] value: &str) {
    // `shared/gateway-setup.mjs:41` is an exact string comparison. A typo must
    // fail closed: a harness that never opens a voice connection sets this
    // knowingly, and everything else is a mistake.
    let fixture = Fixture::new();
    let run = fixture.run(&[("VIA_ALLOW_UNCONFIGURED", value)], &["gateway"]);
    assert_eq!(
        run.error_code(),
        Some(CODE_GATEWAY_SETUP_REQUIRED),
        "{value:?} opened the gate"
    );
}

#[test]
fn a_configured_machine_takes_the_lease_holds_it_and_gives_it_back() {
    let fixture = Fixture::new();
    let running = fixture.start(&[("DASHSCOPE_API_KEY", "sk-example")], &["gateway"]);
    assert!(running.is_serving(), "a configured machine serves");
    assert!(
        fixture.lease_file().exists(),
        "the lease is held for the process's life, not taken and released",
    );
    let origin = running.origin();
    assert!(origin.starts_with("http://"), "{origin}");

    let run = running.stop();
    assert_eq!(
        run.code,
        Some(0),
        "a clean shutdown exits 0: {}",
        run.stderr
    );
    assert_eq!(run.events(), vec!["cli.started", "cli.completed"]);
    assert!(
        !fixture.lease_file().exists(),
        "the lease must be released on the way out, not leaked",
    );
}

#[test]
fn sigint_shuts_a_running_gateway_down_as_cleanly_as_sigterm() {
    // `cli/src/launcher.mjs:387-394` handles both, and a user pressing ^C in
    // the terminal is the commoner of the two.
    let fixture = Fixture::new();
    let running = fixture.start(&[("DASHSCOPE_API_KEY", "sk-example")], &["gateway"]);
    assert!(running.is_serving());
    let run = running.stop_with(SIGINT);
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    assert!(!fixture.lease_file().exists());
}

#[test]
fn the_banner_names_the_origin_and_the_realtime_model() {
    // `gatewaySummary(health)`, `cli/src/launcher.mjs:88-102`: the Realtime
    // label, then the backend state, joined with ` · `. With no backend
    // configured the second half is the voice-chat-only sentence.
    let fixture = Fixture::new();
    let running = fixture.start(&[("DASHSCOPE_API_KEY", "sk-example")], &["gateway"]);
    let banner = running.banner.clone().unwrap_or_default();
    assert!(banner.contains(&running.origin()), "{banner}");
    let summary = running.summary.clone();
    assert!(
        summary.contains(t(Locale::En, keys::GATEWAY_FRONTEND_ONLY_MODE)),
        "{summary}",
    );
    assert!(summary.contains(" · "), "{summary}");
    let _ = running.stop();
}

#[test]
fn the_key_can_come_from_the_configuration_file_rather_than_the_environment() {
    let fixture = Fixture::new();
    fixture.seed_config("# hand written\nDASHSCOPE_API_KEY=sk-from-the-file\n");
    let running = fixture.start(&[], &["gateway"]);
    assert!(
        running.is_serving(),
        "config.env is loaded into the environment before the gate reads it",
    );
    let _ = running.stop();
}

#[test]
fn a_second_gateway_refuses_while_the_first_holds_the_lease() {
    let fixture = Fixture::new();
    // The "first invocation" is this test process, holding the lease through
    // `via-lock`'s own API — the same writer the binary uses, so the document
    // the binary reads is not a fixture's approximation of one.
    let mut handle =
        acquire_gateway_lease(&fixture.config_dir(), AcquireOptions::new().owner("cli"))
            .expect("the lease is free");
    handle
        .update(
            LeaseUpdate::new()
                .state(via_lock::LEASE_STATE_READY)
                .origin("http://127.0.0.1:3101"),
        )
        .expect("publish the origin");

    let run = fixture.run(
        &[("DASHSCOPE_API_KEY", "sk-example"), ("VIA_LOCALE", "zh")],
        &["gateway"],
    );
    assert_eq!(
        run.error_code(),
        Some(via_lock::VIA_GATEWAY_ALREADY_RUNNING)
    );
    assert_eq!(run.code, Some(1));
    assert_eq!(
        run.stderr,
        std::format!(
            "via: {}\n",
            format(
                Locale::Zh,
                keys::LOCK_GATEWAY_ALREADY_RUNNING_AT,
                &[("origin", "http://127.0.0.1:3101")],
            )
        )
    );

    // …and once the incumbent stands down, the same invocation gets through.
    handle.release().expect("release");
    let running = fixture.start(&[("DASHSCOPE_API_KEY", "sk-example")], &["gateway"]);
    assert!(running.is_serving());
    let _ = running.stop();
}

#[test]
fn a_lease_with_no_origin_reports_the_shorter_sentence() {
    let fixture = Fixture::new();
    let handle = acquire_gateway_lease(&fixture.config_dir(), AcquireOptions::new().owner("cli"))
        .expect("the lease is free");
    // A lease in `starting` has not published an origin yet, which is exactly
    // when a second start is most likely to race in.
    assert!(handle.lease().origin.is_empty());

    let run = fixture.run(
        &[("DASHSCOPE_API_KEY", "sk-example"), ("VIA_LOCALE", "zh")],
        &["gateway"],
    );
    assert_eq!(
        run.stderr,
        std::format!(
            "via: {}\n",
            t(Locale::Zh, keys::LOCK_GATEWAY_ALREADY_RUNNING)
        )
    );
    handle.release().expect("release");
}

#[rstest]
// A pid that cannot name a live process.
#[case::not_a_process(0)]
#[case::negative(-1)]
fn a_lease_whose_process_cannot_be_alive_is_reclaimed(#[case] pid: i64) {
    let fixture = Fixture::new();
    write_lease(&fixture.lease_file(), pid, "http://127.0.0.1:3101");
    let running = fixture.start(&[("DASHSCOPE_API_KEY", "sk-example")], &["gateway"]);
    assert!(
        running.is_serving(),
        "a lease naming nobody must be reclaimed, not obeyed",
    );
    let _ = running.stop();
}

#[rstest]
#[case::truncated("{\"schema\":\"via.gateway-lock/v1\"")]
#[case::empty("")]
#[case::not_json("not json at all")]
#[case::foreign_schema("{\"schema\":\"someone-else/v1\",\"instanceId\":\"x\",\"pid\":1}")]
fn a_lease_nobody_can_read_names_nobody(#[case] contents: &str) {
    // Upstream's own behaviour, and the right one: a lease document that will
    // not parse identifies no process, so it is reclaimed rather than treated
    // as an incumbent that can never be dislodged.
    let fixture = Fixture::new();
    std::fs::create_dir_all(fixture.config_dir()).expect("config dir");
    std::fs::write(fixture.lease_file(), contents).expect("write");
    let running = fixture.start(&[("DASHSCOPE_API_KEY", "sk-example")], &["gateway"]);
    assert!(
        running.is_serving(),
        "an unreadable lease identifies no process, so it is reclaimed",
    );
    let _ = running.stop();
}

#[test]
fn every_run_logs_a_start_and_an_outcome() {
    let fixture = Fixture::new();
    let run = fixture.run(&[], &["gateway"]);
    assert_eq!(run.events(), vec!["cli.started", "cli.failed"]);
    let started = &run.records[0];
    assert_eq!(started["command"], "gateway");
    assert_eq!(started["component"], "cli");
    assert_eq!(started["schema"], "via.log/v1");
}
