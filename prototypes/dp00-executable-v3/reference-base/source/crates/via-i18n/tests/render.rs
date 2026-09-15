//! Interpolation, and every way it can refuse.

use rstest::rstest;
use via_i18n::{FormatError, Locale, format, keys, render, t, try_format};

// ── the happy path ─────────────────────────────────────────────────────────

#[rstest]
#[case("no holes", &[], "no holes")]
#[case("{a}", &[("a", "A")], "A")]
#[case("x{a}y", &[("a", "")], "xy")]
#[case("{a}{b}", &[("a", "1"), ("b", "2")], "12")]
#[case("{a} and {a}", &[("a", "x")], "x and x")]
#[case("{{a}}", &[], "{a}")]
#[case("{{{a}}}", &[("a", "v")], "{v}")]
#[case("100%% {a}", &[("a", "done")], "100%% done")]
#[case("已有 Gateway 正在运行：{origin}", &[("origin", "http://x")], "已有 Gateway 正在运行：http://x")]
#[case("{a}", &[("a", "{b}")], "{b}")]
fn render_interpolates(#[case] template: &str, #[case] args: &[(&str, &str)], #[case] want: &str) {
    assert_eq!(render(template, args).as_deref(), Ok(want));
}

/// A value that itself looks like a placeholder is inserted, not re-scanned.
/// Otherwise a user-supplied string could smuggle in a second substitution.
#[test]
fn substituted_values_are_never_re_scanned() {
    let out = render("{a}{b}", &[("a", "{b"), ("b", "}")]);
    assert_eq!(out.as_deref(), Ok("{b}"));
}

// ── refusals ───────────────────────────────────────────────────────────────

#[test]
fn an_unfilled_placeholder_is_an_error() {
    assert_eq!(
        render("a {origin} b", &[]),
        Err(FormatError::MissingArgument {
            name: "origin".to_owned()
        })
    );
}

#[test]
fn an_unused_argument_is_an_error() {
    assert_eq!(
        render("a b", &[("origin", "x")]),
        Err(FormatError::UnusedArgument {
            name: "origin".to_owned()
        })
    );
}

#[test]
fn the_first_unused_argument_is_the_one_reported() {
    assert_eq!(
        render("{a}", &[("a", "1"), ("b", "2"), ("c", "3")]),
        Err(FormatError::UnusedArgument {
            name: "b".to_owned()
        })
    );
}

#[test]
fn a_repeated_argument_name_is_an_error() {
    assert_eq!(
        render("{a}", &[("a", "1"), ("a", "2")]),
        Err(FormatError::DuplicateArgument {
            name: "a".to_owned()
        })
    );
}

/// The duplicate is caught before the walk, so an otherwise-unused duplicate
/// still fails rather than being silently dropped.
#[test]
fn a_repeated_argument_fails_even_when_the_template_ignores_it() {
    assert!(matches!(
        render("plain", &[("a", "1"), ("a", "2")]),
        Err(FormatError::DuplicateArgument { .. })
    ));
}

#[rstest]
#[case("{", FormatError::UnterminatedPlaceholder { at: 0 })]
#[case("a{b", FormatError::UnterminatedPlaceholder { at: 1 })]
#[case("{}", FormatError::EmptyPlaceholder { at: 0 })]
#[case("}", FormatError::StrayCloseBrace { at: 0 })]
#[case("a}b", FormatError::StrayCloseBrace { at: 1 })]
#[case("{{}", FormatError::StrayCloseBrace { at: 2 })]
fn a_malformed_template_is_an_error(#[case] template: &str, #[case] want: FormatError) {
    assert_eq!(render(template, &[]), Err(want));
}

#[rstest]
#[case("{A}")]
#[case("{a b}")]
#[case("{a-b}")]
#[case("{a.b}")]
#[case("{名字}")]
#[case("{$a}")]
fn a_placeholder_name_outside_snake_case_is_an_error(#[case] template: &str) {
    assert!(matches!(
        render(template, &[]),
        Err(FormatError::InvalidPlaceholderName { .. })
    ));
}

/// Byte offsets have to survive multi-byte text, or the error points at the
/// wrong place in exactly the catalog this crate is built for.
#[test]
fn error_offsets_are_byte_offsets_into_the_original() {
    // "已有 " is 3 + 3 + 1 = 7 bytes.
    assert_eq!(
        render("已有 }", &[]),
        Err(FormatError::StrayCloseBrace { at: 7 })
    );
}

#[test]
fn errors_display_something_a_human_can_act_on() {
    let error = render("{origin}", &[]).unwrap_err();
    assert_eq!(error.to_string(), "no value for placeholder `{origin}`");
    let error = render("}", &[]).unwrap_err();
    assert!(error.to_string().contains("stray `}`"));
}

// ── the catalog API on top ─────────────────────────────────────────────────

#[test]
fn try_format_reports_the_fault() {
    assert_eq!(
        try_format(Locale::En, keys::LOCK_GATEWAY_ALREADY_RUNNING_AT, &[]),
        Err(FormatError::MissingArgument {
            name: "origin".to_owned()
        })
    );
}

#[test]
fn try_format_renders_every_locale_of_the_same_key() {
    for locale in Locale::ALL {
        let out = try_format(
            *locale,
            keys::LOCK_GATEWAY_ALREADY_RUNNING_AT,
            &[("origin", "http://127.0.0.1:3101")],
        )
        .expect("every locale carries the same placeholder");
        assert!(out.contains("http://127.0.0.1:3101"), "{locale}: {out}");
        assert!(!out.contains('{'), "{locale} left a hole: {out}");
    }
}

/// The infallible wrapper must not paper over the fault: no half-filled
/// sentence, no panic, and the key and locale in the text so the report says
/// where to look.
#[test]
fn format_renders_a_diagnostic_rather_than_a_sentence_with_a_hole() {
    let out = format(Locale::Zh, keys::LOCK_GATEWAY_ALREADY_RUNNING_AT, &[]);
    assert!(out.starts_with("<via-i18n: cannot render "), "{out}");
    assert!(out.contains("lock.gateway_already_running_at"), "{out}");
    assert!(out.contains(" in zh: "), "{out}");
    assert!(!out.contains("已有 Gateway"), "{out}");
}

#[test]
fn format_of_a_key_with_no_placeholders_equals_t() {
    assert_eq!(
        format(Locale::Ko, keys::LOCK_LEASE_EXHAUSTED, &[]),
        t(Locale::Ko, keys::LOCK_LEASE_EXHAUSTED)
    );
}

#[test]
fn passing_an_argument_to_a_message_that_takes_none_is_refused() {
    let out = format(Locale::En, keys::LOCK_LEASE_EXHAUSTED, &[("origin", "x")]);
    assert!(out.contains("is not used by this message"), "{out}");
}

/// Every message in the catalog renders in every locale when it is given
/// exactly its declared placeholders. This is the crate's whole promise, over
/// the whole table.
#[test]
fn every_key_renders_in_every_locale_with_its_declared_arguments() {
    for key in via_i18n::all_keys() {
        let args: Vec<(&str, &str)> = key
            .placeholders()
            .iter()
            .map(|name| (*name, "<value>"))
            .collect();
        for locale in Locale::ALL {
            let out = try_format(*locale, *key, &args)
                .unwrap_or_else(|err| panic!("{key} in {locale}: {err}"));
            assert!(!out.is_empty(), "{key} in {locale} rendered to nothing");
        }
    }
}

/// …and refuses when even one of them is withheld.
#[test]
fn every_key_with_placeholders_refuses_when_one_is_withheld() {
    for key in via_i18n::all_keys() {
        let declared = key.placeholders();
        if declared.is_empty() {
            continue;
        }
        for withheld in declared {
            let args: Vec<(&str, &str)> = declared
                .iter()
                .filter(|name| *name != withheld)
                .map(|name| (*name, "<value>"))
                .collect();
            assert_eq!(
                try_format(Locale::En, *key, &args),
                Err(FormatError::MissingArgument {
                    name: (*withheld).to_owned()
                }),
                "{key} rendered without `{withheld}`"
            );
        }
    }
}
