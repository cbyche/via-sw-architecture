//! **All three locales, end to end through the binary.**
//!
//! `docs/architecture.md` §16: three locales, `en` (default), `zh` and `ko`,
//! resolved `VIA_LOCALE` → OS locale → `en`. `apps/via/tests/setup_gate.rs`
//! already asserts the *refusal* in all three — the phase-1 milestone. What it
//! cannot reach is everything after the refusal, because in phase 1 there was
//! nothing after it.
//!
//! So this file walks a whole locale through a running product: the machine it
//! scaffolds, the banner it prints, the sentences it serves over HTTP, the
//! sentence it writes into a Work record, and the refusal it exits with. Five
//! surfaces, three languages, one rule:
//!
//! > **Error codes are not localized — only the message beside them.**
//!
//! and one property that is easy to lose and impossible to notice by hand:
//! every surface differs between locales, and none of them leaks an
//! untranslated key or an unsubstituted `{placeholder}`.

mod support;

use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use support::{BUDGET, Machine, Socket, is};
use via_i18n::{Locale, format, keys, t};
use via_protocol::CODE_GATEWAY_SETUP_REQUIRED;
use via_realtime_mock::{Script, script::turns};

/// The three locales VIA ships.
const LOCALES: [Locale; 3] = [Locale::En, Locale::Zh, Locale::Ko];

/// The refusal the setup gate renders in `locale`, built from `via-i18n`.
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

/// Whether `text` still carries a rendering failure.
///
/// `via-i18n` renders a missing key as `<via-i18n:…>` and leaves an
/// unsubstituted variable as `{name}`. Either one reaching a user is a bug that
/// no amount of reading the catalog by hand would catch.
fn assert_rendered(text: &str, what: &str) {
    assert!(!text.contains("<via-i18n:"), "{what}: {text}");
    assert!(!text.contains('{'), "{what}: {text}");
    assert!(!text.trim().is_empty(), "{what} rendered nothing");
}

#[rstest]
#[case(Locale::En)]
#[case(Locale::Zh)]
#[case(Locale::Ko)]
fn a_whole_locale_reaches_every_surface(#[case] locale: Locale) {
    let language = locale.as_str();
    let machine = Machine::new();

    // ── 1. the refusal ──────────────────────────────────────────────────────
    let refused = machine.run_via(&[("VIA_LOCALE", language)], &["gateway"]);
    assert_eq!(refused.code, Some(1), "stderr: {}", refused.stderr);
    assert_eq!(
        refused.stderr,
        format!("{}{}\n", via::STDERR_PREFIX, expected_refusal(locale)),
    );
    assert_rendered(&refused.stderr, "the refusal");
    // The code is never localized — a script branches on it.
    assert_eq!(
        refused.error_code(),
        Some(CODE_GATEWAY_SETUP_REQUIRED),
        "records: {:?}",
        refused.records,
    );
    assert!(
        refused.stderr.contains("DASHSCOPE_API_KEY"),
        "the key an operator sets is a wire name, not prose: {}",
        refused.stderr,
    );

    // ── 2. the scaffolded machine ───────────────────────────────────────────
    // `config.env` is seeded in the locale that was resolved when it was
    // written, so a `ko` install gets a Korean file rather than upstream's
    // Chinese one.
    let header = std::fs::read_to_string(machine.config_path(via_core::paths::CONFIG_FILE_NAME))
        .expect("config.env");
    assert_eq!(
        header.lines().next().unwrap_or_default(),
        t(locale, keys::RUNTIME_CONFIG_HEADER),
    );

    // ── 3. the banner ───────────────────────────────────────────────────────
    let gateway = machine.start_via_gateway(&[
        ("VIA_LOCALE", language),
        ("DASHSCOPE_API_KEY", "sk-e2e-locale"),
    ]);
    gateway.require_serving();
    let banner = gateway.banner.clone().unwrap_or_default();
    assert!(
        banner.starts_with(
            t(locale, keys::CLI_GATEWAY_BANNER_STARTED)
                .split("{url}")
                .next()
                .unwrap_or_default()
        ),
        "the banner is in the machine's language: {banner}",
    );
    assert!(banner.contains(&gateway.origin()), "{banner}");
    assert_rendered(&banner, "the banner");

    let run = gateway.stop();
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
}

#[rstest]
#[case(Locale::En)]
#[case(Locale::Zh)]
#[case(Locale::Ko)]
#[tokio::test(flavor = "multi_thread")]
async fn a_whole_locale_reaches_the_http_surface_and_a_work_record(#[case] locale: Locale) {
    let machine = Machine::new();
    let gateway = machine.start_harness_gateway(
        &Script::conversation().turn(turns::call_tool(
            "call_1",
            "spawn_thinking",
            &json!({ "objective": "index the archive" }),
        )),
        Some(&json!({
            "backend": "opencode",
            // A backend that fails, so the Work carries a *sentence* rather
            // than a result — and the sentence is the localized one.
            "turns": [{
                "kind": "failing",
                "message": t(locale, keys::VOICE_ERROR_SUBMIT_FAILED),
                "status": 0,
            }],
        })),
        &[("VIA_LOCALE", locale.as_str())],
    );
    gateway.require_serving();
    let api = gateway.api();

    // ── the 404 body, which is deliberately *not* localized ─────────────────
    // The catalogue: `{"error":"task not found"}`, in English in every locale,
    // because clients branch on the string. So the assertion is that a Gateway
    // running in `zh` or `ko` still answers the English one.
    let missing = api.get("/api/tasks/work_nonesuch").await;
    assert_eq!(missing.status, 404);
    let sentence = missing.body["error"].as_str().unwrap_or_default();
    assert_eq!(sentence, "task not found", "locale {locale:?}");
    assert_eq!(sentence, t(locale, keys::GATEWAY_TASK_NOT_FOUND));
    assert_rendered(sentence, "the 404 body");

    // ── a Work's own error text ─────────────────────────────────────────────
    let mut client = Socket::connect(&gateway.origin(), "voice-e2e-locale").await;
    client.hello(locale).await;
    client.say("index the archive").await;
    let failed = client
        .wait_for(BUDGET, is("task.failed"))
        .await
        .expect("the backend refuses and the Work fails");
    let error = failed["task"]["error"].as_str().unwrap_or_default();
    assert_rendered(error, "the Work's error");
    assert_eq!(
        failed["task"]["status"],
        via_protocol::WorkStatus::Failed.as_str(),
    );
    // The sentence is spoken to the user verbatim, so it is in their language
    // and not in the developer's.
    assert!(
        error.contains(t(locale, keys::VOICE_ERROR_SUBMIT_FAILED)),
        "{locale:?}: {error}",
    );

    drop(client);
    let _ = gateway.stop();
}

#[test]
fn the_three_locales_are_three_different_products() {
    // The guard against a catalog that "supports" three languages by shipping
    // one three times. Every surface the cases above assert is compared across
    // the three, and all three must differ.
    let machine = Machine::new();
    let refusals: Vec<String> = LOCALES
        .iter()
        .map(|locale| {
            machine
                .run_via(&[("VIA_LOCALE", locale.as_str())], &["gateway"])
                .stderr
        })
        .collect();
    for (left, right) in [(0, 1), (1, 2), (0, 2)] {
        assert_ne!(refusals[left], refusals[right], "{left} vs {right}");
    }

    // Prose a person reads. All three must differ.
    for key in [
        keys::RUNTIME_CONFIG_HEADER,
        keys::CLI_GATEWAY_BANNER_STARTED,
        keys::WORK_RESTART_INTERACTIVE_INCOMPLETE,
        keys::GATEWAY_SETUP_REQUIRED,
        keys::GATEWAY_FRONTEND_ONLY_MODE,
    ] {
        let rendered: Vec<&str> = LOCALES.iter().map(|locale| t(*locale, key)).collect();
        for (left, right) in [(0, 1), (1, 2), (0, 2)] {
            assert_ne!(
                rendered[left], rendered[right],
                "`{key}` is the same string in two locales",
            );
        }
    }

    // …and the other half of the rule, which matters more. The two Work
    // refusals are **wire strings**, not prose: the catalogue records them as
    // `{"error":"task not found"}` and `{"error":"task is no longer active"}`
    // *in English in every locale, because clients branch on the string*.
    // Localizing them would break every consumer that ever wrote
    // `if (body.error === 'task not found')`.
    for key in [keys::GATEWAY_TASK_NOT_FOUND, keys::GATEWAY_TASK_NOT_ACTIVE] {
        let rendered: Vec<&str> = LOCALES.iter().map(|locale| t(*locale, key)).collect();
        assert_eq!(
            rendered[0], rendered[1],
            "`{key}` is a value clients branch on and must not be localized",
        );
        assert_eq!(rendered[1], rendered[2], "{key}");
    }
    assert_eq!(
        t(Locale::Zh, keys::GATEWAY_TASK_NOT_FOUND),
        "task not found"
    );
    assert_eq!(
        t(Locale::Ko, keys::GATEWAY_TASK_NOT_ACTIVE),
        "task is no longer active",
    );
}

#[test]
fn an_unusable_locale_falls_back_to_english_rather_than_failing() {
    // `VIA_LOCALE` → OS locale → `en`. A machine whose `LANG` names a language
    // VIA does not ship still starts, in English — a Gateway that refused to
    // run because of a locale would be a Gateway nobody outside three countries
    // could use.
    let machine = Machine::new();
    for environment in [
        vec![("LANG", "fr_FR.UTF-8")],
        vec![("VIA_LOCALE", "fr")],
        vec![("VIA_LOCALE", "")],
    ] {
        let run = machine.run_via(&environment, &["gateway"]);
        assert_eq!(
            run.stderr,
            format!("{}{}\n", via::STDERR_PREFIX, expected_refusal(Locale::En)),
            "{environment:?} did not fall back to English",
        );
    }
    // …and the OS locale is consulted when `VIA_LOCALE` is not set.
    let run = machine.run_via(&[("LANG", "ko_KR.UTF-8")], &["gateway"]);
    assert_eq!(
        run.stderr,
        format!("{}{}\n", via::STDERR_PREFIX, expected_refusal(Locale::Ko)),
    );
}
