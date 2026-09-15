//! Structural invariants of the catalog.
//!
//! These tests read `assets/i18n/*.json` back off disk rather than trusting the
//! generated table, so a build script that silently dropped, reordered or
//! rewrote an entry fails here rather than shipping.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use serde_json::Value;
use via_i18n::{Locale, all_keys, key_count, keys, t};

const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/i18n");

/// `key -> (locale -> text)`, read from the asset files.
fn assets() -> BTreeMap<String, BTreeMap<String, String>> {
    let mut files: Vec<PathBuf> = fs::read_dir(ASSETS)
        .expect("assets/i18n must exist")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "assets/i18n has no catalog files");

    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for path in files {
        let text = fs::read_to_string(&path).expect("readable catalog file");
        let value: Value = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("{} is not valid JSON: {err}", path.display()));
        let object = value
            .as_object()
            .unwrap_or_else(|| panic!("{} is not a JSON object", path.display()));
        for (key, body) in object {
            let locales = body
                .as_object()
                .unwrap_or_else(|| panic!("{key} is not an object"));
            let mut row = BTreeMap::new();
            for (locale, text) in locales {
                row.insert(
                    locale.clone(),
                    text.as_str()
                        .unwrap_or_else(|| panic!("{key}.{locale} is not a string"))
                        .to_owned(),
                );
            }
            assert!(
                out.insert(key.clone(), row).is_none(),
                "duplicate catalog key `{key}` across asset files"
            );
        }
    }
    out
}

/// The `{placeholder}` names in a template.
fn placeholders(template: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let bytes = template.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' if bytes.get(index + 1) == Some(&b'{') => index += 2,
            b'}' if bytes.get(index + 1) == Some(&b'}') => index += 2,
            b'{' => {
                let rest = &template[index + 1..];
                let len = rest.find('}').expect("terminated placeholder");
                names.insert(rest[..len].to_owned());
                index += 1 + len + 1;
            }
            _ => index += 1,
        }
    }
    names
}

fn has_han(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c as u32,
            0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x2FA1F)
    })
}

fn has_hangul(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c as u32, 0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7AF))
}

// ── the key tree ───────────────────────────────────────────────────────────

#[test]
fn every_key_carries_exactly_the_three_locales() {
    let expected: BTreeSet<String> = ["en", "ko", "zh"].iter().map(|s| (*s).to_owned()).collect();
    for (key, row) in assets() {
        let found: BTreeSet<String> = row.keys().cloned().collect();
        assert_eq!(found, expected, "{key} has the wrong locale set");
    }
}

/// The discipline ARGO already keeps: the three key trees are identical.
///
/// The catalog is authored as one row per key, so this cannot drift by
/// construction — which is the point. The test asserts the property against
/// the assets anyway, because "cannot drift by construction" is a claim about
/// the *format*, and the format is what a future edit would change.
#[test]
fn the_three_key_trees_are_identical() {
    let assets = assets();
    let mut trees: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (key, row) in &assets {
        for locale in row.keys() {
            trees.entry(locale).or_default().insert(key);
        }
    }
    assert_eq!(trees.len(), 3, "expected exactly three locale trees");
    let en = trees.get("en").expect("en tree").clone();
    assert_eq!(trees.get("zh").expect("zh tree").clone(), en, "zh vs en");
    assert_eq!(trees.get("ko").expect("ko tree").clone(), en, "ko vs en");
}

#[test]
fn no_translation_is_empty_or_whitespace_only() {
    for (key, row) in assets() {
        for (locale, text) in row {
            assert!(
                !text.trim().is_empty(),
                "{key}.{locale} is empty — an empty translation is a hole, not a value"
            );
        }
    }
}

#[test]
fn keys_are_dotted_lowercase_and_namespaced() {
    for key in assets().keys() {
        assert!(key.contains('.'), "{key} has no namespace");
        for segment in key.split('.') {
            assert!(!segment.is_empty(), "{key} has an empty segment");
            assert!(
                segment.starts_with(|c: char| c.is_ascii_lowercase()),
                "{key} has a segment that does not start with a lowercase letter"
            );
            assert!(
                segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{key} has a segment outside [a-z0-9_]"
            );
        }
    }
}

// ── the generated table agrees with the assets ─────────────────────────────

#[test]
fn generated_consts_reproduce_the_assets_exactly() {
    let assets = assets();
    assert_eq!(key_count(), assets.len(), "generated key count");

    let generated: BTreeMap<&str, [&str; 3]> = all_keys()
        .iter()
        .map(|key| {
            (
                key.as_str(),
                [
                    t(Locale::En, *key),
                    t(Locale::Zh, *key),
                    t(Locale::Ko, *key),
                ],
            )
        })
        .collect();

    assert_eq!(
        generated.keys().copied().collect::<BTreeSet<_>>(),
        assets.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        "generated key set vs asset key set"
    );

    for (key, row) in &assets {
        let found = generated.get(key.as_str()).expect("generated row");
        assert_eq!(found[0], row["en"], "{key}.en");
        assert_eq!(found[1], row["zh"], "{key}.zh");
        assert_eq!(found[2], row["ko"], "{key}.ko");
    }
}

#[test]
fn all_keys_is_sorted_and_free_of_duplicates() {
    let names: Vec<&str> = all_keys().iter().map(|key| key.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(names, sorted, "all_keys() must be sorted and unique");
}

// ── placeholders ───────────────────────────────────────────────────────────

/// `docs/architecture.md` §16's structural-parity rule: the authored `en`/`ko`
/// peers carry the same variable set as `zh`. A locale that drops a
/// placeholder silently loses a value in that language only, which is the
/// hardest kind of localization bug to see.
#[test]
fn placeholder_sets_are_identical_across_the_three_locales() {
    for (key, row) in assets() {
        let en = placeholders(&row["en"]);
        assert_eq!(placeholders(&row["zh"]), en, "{key}: zh vs en placeholders");
        assert_eq!(placeholders(&row["ko"]), en, "{key}: ko vs en placeholders");
    }
}

#[test]
fn key_placeholders_match_the_template() {
    for key in all_keys() {
        let declared: BTreeSet<String> =
            key.placeholders().iter().map(|s| (*s).to_owned()).collect();
        for locale in Locale::ALL {
            assert_eq!(
                placeholders(t(*locale, *key)),
                declared,
                "{key} in {locale}"
            );
        }
    }
}

#[test]
fn placeholder_names_are_snake_case_ascii() {
    for key in all_keys() {
        for name in key.placeholders() {
            assert!(
                !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{key} has placeholder `{name}` outside [a-z0-9_]"
            );
        }
    }
}

// ── the languages are actually different languages ─────────────────────────

/// Catches the laziest possible failure: a `zh` value pasted into the `en`
/// column and left there.
#[test]
fn the_english_catalog_contains_no_han_or_hangul() {
    for key in all_keys() {
        let text = t(Locale::En, *key);
        assert!(!has_han(text), "{key}.en contains Han characters");
        assert!(!has_hangul(text), "{key}.en contains Hangul");
    }
}

#[test]
fn the_korean_catalog_contains_no_han() {
    for key in all_keys() {
        assert!(
            !has_han(t(Locale::Ko, *key)),
            "{key}.ko contains Han characters — VIA's Korean is Hangul, not Hanja"
        );
    }
}

/// Every `zh` value should be Chinese. Two kinds of exception, and no others:
///
/// - **wire strings** — upstream emits them in English in every locale, so
///   translating one would break a client that matches on it;
/// - **punctuation and chrome** — a joiner, a bracket pair, or a line that is
///   nothing but a product name and a value. These still differ by locale
///   (`；` vs `; `, `：` vs `: `), which is exactly why they are keys.
///
/// The list is exhaustive and is asserted to contain no stale entry, so a key
/// cannot be quietly parked here after its text is emptied.
#[test]
fn the_chinese_catalog_is_chinese_except_for_wire_strings_and_punctuation() {
    const NOT_CHINESE_PROSE: &[&str] = &[
        // wire strings
        "gateway.permission_decision_invalid",
        // `{"error":"task not found"}` and `{"error":"task is no longer
        // active","task":…}` are the literal 404/409 bodies of
        // `GET|DELETE /api/tasks/:id` and `GET /api/tasks/:id/events`
        // (`docs/reference/contracts.json`, http-route *Work HTTP surface*).
        // Upstream writes them in English in every locale because a client
        // branches on the string; translating either one would break that
        // client. They live in the catalog rather than as `const`s so the one
        // rule — every user-visible string is a key — has no exceptions.
        "gateway.task_not_found",
        "gateway.task_not_active",
        "memory.ambiguous_edit_code",
        "memory.edit_not_found_code",
        "memory.invalid_edit_code",
        "memory.stale_document_code",
        "store.notes.unavailable",
        // punctuation and chrome
        "backend.choices_join",
        // `- <id>（<label>）`: a bullet, a vendor-owned model id and a
        // vendor-owned label, with only the bracket pair varying by locale.
        "cli.config_show_model_item",
        "cli.gateway_banner_webui",
        "cli.gateway_summary_realtime",
        "cli.label_gateway_url",
        // The list separator itself — `、` in zh, `, ` elsewhere.
        "cli.list_separator",
        "gateway.setup_required_item",
        "gateway.setup_required_join",
        "install.step_start",
        // `${label} ACP ${message}${stderr ? `：${stderr}` : ''}` — upstream's
        // template (acp-process-client.mjs:58) is a backend label, the wire
        // token `ACP` and an engine-supplied detail. The only thing that varies
        // by locale is the separator before the stderr tail: `：` in zh, `: `
        // elsewhere. Neither form carries prose of its own.
        "acp.process_error",
        "acp.process_error_with_stderr",
    ];
    for key in all_keys() {
        if NOT_CHINESE_PROSE.contains(&key.as_str()) {
            continue;
        }
        assert!(
            has_han(t(Locale::Zh, *key)),
            "{key}.zh has no Han characters — either it is untranslated or it belongs \
             in NOT_CHINESE_PROSE with a reason"
        );
    }
    for name in NOT_CHINESE_PROSE {
        let key = all_keys()
            .iter()
            .find(|key| key.as_str() == *name)
            .unwrap_or_else(|| panic!("NOT_CHINESE_PROSE names `{name}`, not a catalog key"));
        assert!(
            !has_han(t(Locale::Zh, *key)),
            "{name} is Chinese now — take it out of NOT_CHINESE_PROSE"
        );
    }
}

// ── identity ───────────────────────────────────────────────────────────────

/// `scripts/brand_leak.py` only scans `.rs` and `.toml`, so the catalog's JSON
/// assets are outside its reach. This is that gate, for this crate's data.
#[test]
fn the_catalog_carries_no_upstream_brand() {
    // allow-brand: the inbound names docs/rebrand.md renames, spelled here so
    // allow-brand: a copy that escaped the rename fails this test.
    const FORBIDDEN_SUBSTRINGS: &[&str] = &[
        "qwen-audio-agent", // allow-brand
        "qwen_audio_agent", // allow-brand
        "qwenaudio",        // allow-brand
        "qwaudio",          // allow-brand
        "\u{5343}\u{95ee}", // the upstream wake phrase's brand syllables
        "tinicore",         // outbound: ARGO must not surface in VIA's text
        "tiniffi",          // outbound
    ];
    // Whole words only: `argo` is a substring of `cargo`.
    const FORBIDDEN_WORDS: &[&str] = &["argo", "tini"];

    for (key, row) in assets() {
        for (locale, text) in row {
            let lowered = text.to_lowercase();
            for needle in FORBIDDEN_SUBSTRINGS {
                assert!(
                    !lowered.contains(*needle),
                    "{key}.{locale} leaks `{needle}`"
                );
            }
            for word in lowered.split(|c: char| !c.is_ascii_alphanumeric()) {
                assert!(
                    !FORBIDDEN_WORDS.contains(&word),
                    "{key}.{locale} leaks the word `{word}`"
                );
            }
        }
    }
}

/// The wake phrase is configuration, never a literal
/// (`docs/architecture.md` §16), so the sleep message interpolates it.
#[test]
fn the_sleep_message_takes_the_wake_word_as_a_value() {
    assert_eq!(
        keys::REALTIME_ASLEEP.placeholders(),
        ["wake_word"].as_slice(),
        "the sleep message must take the configured wake phrase, not bake one in"
    );
}

// ── the phase-0 handover ───────────────────────────────────────────────────

/// The keys the three shipped crates left `TODO(via-i18n)` markers for.
///
/// `via-lock` (`crates/via-lock/src/error.rs:26,44`), `via-log`
/// (`crates/via-log/src/sink.rs:68`) and `via-protocol`
/// (`crates/via-protocol/src/error.rs`, recorded in
/// `docs/deviations/phase-0.md`). None of those crates is edited here — this
/// test is the promise that the keys they were told to expect exist, with the
/// `zh` values they recorded.
///
/// The markers themselves are now closed. `via-protocol` moved its sentences
/// into this catalog and left English on `Display`; `via-log` and `via-lock`
/// cannot, because `docs/architecture.md` §9 gives `shared → ∅` and
/// `via-arch-test::leaf_violations` enforces it, so their literals stay where
/// they are and `via-conformance`'s `tests/localized_leaf_messages.rs` asserts
/// each one against the key below. That test compares *shipped constants* to
/// this catalog; the values here are retyped, which is what makes the pair
/// of tests a real cross-check rather than one source read twice.
#[test]
fn the_phase_zero_todo_keys_exist_with_the_recorded_text() {
    assert_eq!(
        t(Locale::Zh, keys::LOCK_GATEWAY_ALREADY_RUNNING),
        "已有 Gateway 正在运行"
    );
    assert_eq!(
        t(Locale::Zh, keys::LOCK_GATEWAY_ALREADY_RUNNING_AT),
        "已有 Gateway 正在运行：{origin}"
    );
    assert_eq!(
        t(Locale::Zh, keys::LOCK_LEASE_EXHAUSTED),
        "无法获取 Gateway 实例租约"
    );
    // via-log's LOG_SINK_FAILURE_PREFIX_ZH, already rebranded in that crate.
    assert_eq!(
        t(Locale::Zh, keys::LOG_SINK_WRITE_FAILED),
        "VIA 日志写入失败：{detail}"
    );
    // via-protocol's three ProtocolError variants.
    assert_eq!(
        t(Locale::Zh, keys::GATEWAY_SETUP_REQUIRED),
        "Gateway 启动被拒绝，缺少必填配置：{details}"
    );
    assert_eq!(
        t(Locale::Zh, keys::GATEWAY_INPUT_SUSPEND_REQUIRES_OWNER),
        "input.suspend 需要 owner"
    );
    assert_eq!(
        t(Locale::En, keys::GATEWAY_INPUT_SUSPEND_REQUIRES_OWNER),
        "input.suspend requires an owner"
    );
}

#[test]
fn the_catalog_is_large_enough_to_be_the_catalog() {
    // A floor, not a fixture: this fails when a merge drops a whole asset file.
    assert!(
        key_count() >= 300,
        "the catalog has shrunk to {} keys",
        key_count()
    );
}
