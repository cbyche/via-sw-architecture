//! Locale resolution: `VIA_LOCALE` → OS locale → `en`.
//!
//! Every case runs against an injected env reader, so these tests are
//! parallel-safe and touch no process state. `std::env::set_var` is `unsafe`
//! in edition 2024 and would have forced the whole file to serialize.

use std::cell::RefCell;
use std::str::FromStr;

use rstest::rstest;
use via_i18n::{LOCALE_ENV, Locale, OS_LOCALE_ENV_ORDER, resolve_locale};

/// An env reader backed by a fixed table.
fn reader<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name: &str| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| (*value).to_owned())
    }
}

// ── the enum ───────────────────────────────────────────────────────────────

#[test]
fn the_default_is_english() {
    assert_eq!(Locale::default(), Locale::En);
    assert_eq!(Locale::ALL.first().copied(), Some(Locale::En));
}

#[test]
fn all_lists_the_three_locales_once_each() {
    assert_eq!(Locale::ALL, &[Locale::En, Locale::Zh, Locale::Ko]);
    let mut wires: Vec<&str> = Locale::ALL.iter().map(Locale::as_str).collect();
    wires.sort_unstable();
    wires.dedup();
    assert_eq!(wires, ["en", "ko", "zh"]);
}

#[test]
fn as_str_round_trips_through_from_wire() {
    for locale in Locale::ALL {
        assert_eq!(Locale::from_wire(locale.as_str()), Some(*locale));
        assert_eq!(locale.to_string(), locale.as_str());
        assert_eq!(Locale::from_str(locale.as_str()), Ok(*locale));
    }
}

#[rstest]
#[case("EN")]
#[case("En")]
#[case("en_US")]
#[case("en-GB")]
#[case("english")]
#[case("")]
#[case(" en")]
#[case("zh-Hans")]
#[case("jp")]
#[case("ja")]
fn from_wire_is_exact_only(#[case] value: &str) {
    assert_eq!(Locale::from_wire(value), None, "{value:?}");
    assert!(Locale::from_str(value).is_err(), "{value:?}");
}

#[test]
fn the_from_str_error_names_the_value_and_the_alternatives() {
    let error = Locale::from_str("fr").unwrap_err();
    assert_eq!(error.value, "fr");
    let rendered = error.to_string();
    assert!(rendered.contains("fr"), "{rendered}");
    assert!(rendered.contains("en, zh or ko"), "{rendered}");
}

// ── OS tags ────────────────────────────────────────────────────────────────

#[rstest]
#[case("zh_CN.UTF-8", Some(Locale::Zh))]
#[case("zh_TW", Some(Locale::Zh))]
#[case("zh-Hant-HK", Some(Locale::Zh))]
#[case("zh", Some(Locale::Zh))]
#[case("ZH_CN.utf8", Some(Locale::Zh))]
#[case("ko_KR", Some(Locale::Ko))]
#[case("ko_KR.UTF-8", Some(Locale::Ko))]
#[case("ko-KR", Some(Locale::Ko))]
#[case("en_US", Some(Locale::En))]
#[case("en_US.UTF-8@euro", Some(Locale::En))]
#[case("  en_GB  ", Some(Locale::En))]
#[case("C", None)]
#[case("C.UTF-8", None)]
#[case("POSIX", None)]
#[case("", None)]
#[case("   ", None)]
#[case(".UTF-8", None)]
#[case("_CN", None)]
#[case("fr_FR.UTF-8", None)]
#[case("kot_KR", None)]
#[case("zhx", None)]
#[case("💥", None)]
fn os_tags_parse_leniently(#[case] tag: &str, #[case] want: Option<Locale>) {
    assert_eq!(Locale::from_language_tag(tag), want, "{tag:?}");
}

// ── resolution ─────────────────────────────────────────────────────────────

#[test]
fn nothing_set_resolves_to_english() {
    assert_eq!(resolve_locale(&reader(&[])), Locale::En);
}

#[rstest]
#[case(&[("VIA_LOCALE", "zh")], Locale::Zh)]
#[case(&[("VIA_LOCALE", "ko")], Locale::Ko)]
#[case(&[("VIA_LOCALE", "en")], Locale::En)]
#[case(&[("VIA_LOCALE", "  ko  ")], Locale::Ko)]
#[case(&[("LC_ALL", "zh_CN.UTF-8")], Locale::Zh)]
#[case(&[("LC_MESSAGES", "ko_KR.UTF-8")], Locale::Ko)]
#[case(&[("LANG", "en_US.UTF-8")], Locale::En)]
#[case(&[("AppleLocale", "ko_KR")], Locale::Ko)]
#[case(&[("LANG", "not a locale")], Locale::En)]
#[case(&[("LANG", "")], Locale::En)]
#[case(&[("VIA_LOCALE", ""), ("LANG", "zh_CN.UTF-8")], Locale::Zh)]
#[case(&[("VIA_LOCALE", "klingon"), ("LANG", "ko_KR")], Locale::Ko)]
#[case(&[("LC_ALL", "C"), ("LANG", "zh_CN.UTF-8")], Locale::Zh)]
#[case(&[("LC_ALL", "ko_KR"), ("LC_MESSAGES", "zh_CN"), ("LANG", "en_US")], Locale::Ko)]
#[case(&[("LC_MESSAGES", "zh_CN"), ("LANG", "en_US")], Locale::Zh)]
#[case(&[("LANG", "en_US"), ("AppleLocale", "ko_KR")], Locale::En)]
#[case(&[("VIA_LOCALE", "zh"), ("LC_ALL", "ko_KR"), ("LANG", "en_US")], Locale::Zh)]
fn resolution_follows_the_documented_order(#[case] env: &[(&str, &str)], #[case] want: Locale) {
    assert_eq!(resolve_locale(&reader(env)), want, "{env:?}");
}

/// `VIA_LOCALE` is the wire spelling, not an OS tag: `zh_CN` there is a
/// mistake, and a mistake must not out-vote the OS setting.
#[test]
fn via_locale_does_not_accept_an_os_tag() {
    assert_eq!(
        resolve_locale(&reader(&[("VIA_LOCALE", "zh_CN.UTF-8"), ("LANG", "ko_KR")])),
        Locale::Ko
    );
    assert_eq!(
        resolve_locale(&reader(&[("VIA_LOCALE", "zh_CN.UTF-8")])),
        Locale::En
    );
}

#[test]
fn resolution_stops_at_the_first_source_that_answers() {
    let seen = RefCell::new(Vec::new());
    let env = |name: &str| {
        seen.borrow_mut().push(name.to_owned());
        match name {
            "LC_ALL" => Some("zh_CN.UTF-8".to_owned()),
            _ => None,
        }
    };
    assert_eq!(resolve_locale(&env), Locale::Zh);
    assert_eq!(seen.into_inner(), [LOCALE_ENV, "LC_ALL"]);
}

#[test]
fn every_documented_source_is_actually_consulted() {
    let seen = RefCell::new(Vec::new());
    let env = |name: &str| {
        seen.borrow_mut().push(name.to_owned());
        None
    };
    assert_eq!(resolve_locale(&env), Locale::En);

    let mut expected = vec![LOCALE_ENV.to_owned()];
    expected.extend(OS_LOCALE_ENV_ORDER.iter().map(|name| (*name).to_owned()));
    assert_eq!(seen.into_inner(), expected);
}

#[test]
fn the_os_order_is_posix_precedence_plus_the_macos_store() {
    assert_eq!(
        OS_LOCALE_ENV_ORDER,
        &["LC_ALL", "LC_MESSAGES", "LANG", "AppleLocale"]
    );
    assert_eq!(LOCALE_ENV, "VIA_LOCALE");
}

/// A hostile value must not crash, hang, or be mistaken for a locale.
#[rstest]
#[case("zh\0CN")]
#[case("../../etc/passwd")]
#[case("zh; rm -rf /")]
#[case("\u{202e}hz")]
fn hostile_values_fall_through_to_english(#[case] value: &str) {
    assert_eq!(resolve_locale(&reader(&[(LOCALE_ENV, value)])), Locale::En);
    assert_eq!(resolve_locale(&reader(&[("LANG", value)])), Locale::En);
}

/// A very long value is a parse input like any other; the language subtag is
/// bounded by the first separator, so length is not a cost.
#[test]
fn an_absurdly_long_value_is_still_just_a_miss() {
    let value = "z".repeat(100_000);
    assert_eq!(resolve_locale(&reader(&[("LANG", &value)])), Locale::En);
    let padded = format!("zh_{}", "X".repeat(100_000));
    assert_eq!(resolve_locale(&reader(&[("LANG", &padded)])), Locale::Zh);
}
