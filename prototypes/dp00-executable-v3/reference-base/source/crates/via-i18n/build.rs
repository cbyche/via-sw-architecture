//! Turns `assets/i18n/*.json` into typed catalog keys.
//!
//! The catalog is one table per file, each file a flat JSON object mapping a
//! dotted key to an object with exactly the three locale fields `en`, `zh` and
//! `ko`. This script is where "a missing translation is a compile error" is
//! actually enforced: every check below fails the build rather than emitting a
//! table with a hole in it. In particular:
//!
//! - a key present in one locale and absent in another cannot be expressed, so
//!   the key trees are identical by construction;
//! - an empty string in any locale is rejected;
//! - the `{placeholder}` set must be identical across the three locales, which
//!   is `docs/architecture.md` §16's "same `{{variable}}` set" parity rule;
//! - a duplicate key across two asset files is rejected.
//!
//! Output is `$OUT_DIR/catalog.rs`, `include!`d by `src/catalog.rs`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// The three locales, in the order the generated `Entry` declares them.
const LOCALES: [&str; 3] = ["en", "zh", "ko"];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest = PathBuf::from(env_var("CARGO_MANIFEST_DIR")?);
    let assets = manifest.join("assets").join("i18n");
    let out = PathBuf::from(env_var("OUT_DIR")?).join("catalog.rs");

    println!("cargo::rerun-if-changed=assets/i18n");
    println!("cargo::rerun-if-changed=build.rs");

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&assets)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            println!("cargo::rerun-if-changed=assets/i18n/{}", file_name(&path)?);
            files.push(path);
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("no catalog files in {}", assets.display()).into());
    }

    // BTreeMap: the generated table is sorted by key, so the output is stable
    // no matter what order the asset files happen to be read in.
    let mut entries: BTreeMap<String, Record> = BTreeMap::new();
    for path in &files {
        let source = file_name(path)?;
        let text = fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|err| format!("{source}: not valid JSON: {err}"))?;
        let object = value
            .as_object()
            .ok_or_else(|| format!("{source}: the catalog file must be a JSON object"))?;
        for (key, body) in object {
            let record = parse_record(&source, key, body)?;
            if let Some(previous) = entries.insert(key.clone(), record) {
                return Err(format!(
                    "duplicate catalog key `{key}`: defined in {} and again in {source}",
                    previous.source
                )
                .into());
            }
        }
    }

    fs::write(&out, render(&entries)?)?;
    Ok(())
}

/// One catalog key with all three of its values.
struct Record {
    /// The asset file it was read from, for duplicate-key diagnostics.
    source: String,
    /// `en`, `zh`, `ko` — in [`LOCALES`] order.
    values: [String; 3],
    /// The placeholder names, sorted and deduplicated.
    placeholders: Vec<String>,
}

fn parse_record(source: &str, key: &str, body: &serde_json::Value) -> Result<Record, String> {
    validate_key(source, key)?;
    let object = body
        .as_object()
        .ok_or_else(|| format!("{source}: `{key}` must map to an object of locale values"))?;

    for name in object.keys() {
        if !LOCALES.contains(&name.as_str()) {
            return Err(format!(
                "{source}: `{key}` has an unknown locale field `{name}` (expected en, zh, ko)"
            ));
        }
    }

    let mut values: [String; 3] = [String::new(), String::new(), String::new()];
    let mut placeholders: Option<Vec<String>> = None;
    for (index, locale) in LOCALES.iter().enumerate() {
        let raw = object
            .get(*locale)
            .ok_or_else(|| format!("{source}: `{key}` has no `{locale}` translation"))?;
        let text = raw
            .as_str()
            .ok_or_else(|| format!("{source}: `{key}`.{locale} must be a string"))?;
        if text.is_empty() {
            return Err(format!(
                "{source}: `{key}`.{locale} is empty — an empty translation is a hole, not a value"
            ));
        }
        let found =
            placeholders_of(text).map_err(|err| format!("{source}: `{key}`.{locale}: {err}"))?;
        match &placeholders {
            None => placeholders = Some(found),
            Some(first) if *first == found => {}
            Some(first) => {
                return Err(format!(
                    "{source}: `{key}` placeholder sets differ — en has {first:?}, {locale} has \
                     {found:?}. Every locale must carry every value."
                ));
            }
        }
        values[index] = text.to_owned();
    }

    Ok(Record {
        source: source.to_owned(),
        values,
        placeholders: placeholders.unwrap_or_default(),
    })
}

/// Dotted, lowercase, at least two segments: `namespace.name`.
fn validate_key(source: &str, key: &str) -> Result<(), String> {
    let bad = |why: &str| format!("{source}: catalog key `{key}` {why}");
    if !key.contains('.') {
        return Err(bad(
            "has no namespace — keys are dotted, e.g. `gateway.already_running`",
        ));
    }
    for segment in key.split('.') {
        if segment.is_empty() {
            return Err(bad("has an empty segment"));
        }
        if !segment.starts_with(|c: char| c.is_ascii_lowercase()) {
            return Err(bad(
                "has a segment that does not start with a lowercase letter",
            ));
        }
        if !segment
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(bad("has a segment outside [a-z0-9_]"));
        }
    }
    Ok(())
}

/// The placeholder names in a template, sorted and deduplicated.
///
/// Kept deliberately in lockstep with `src/render.rs`: this is the same grammar,
/// checked at build time so a malformed template can never reach the runtime.
fn placeholders_of(template: &str) -> Result<Vec<String>, String> {
    let bytes = template.as_bytes();
    let mut names: Vec<String> = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'{' if bytes.get(index + 1) == Some(&b'{') => index += 2,
            b'}' if bytes.get(index + 1) == Some(&b'}') => index += 2,
            b'}' => return Err(format!("stray `}}` at byte {index}")),
            b'{' => {
                let rest = &template[index + 1..];
                let len = rest
                    .find('}')
                    .ok_or_else(|| format!("unterminated `{{` at byte {index}"))?;
                let name = &rest[..len];
                if name.is_empty() {
                    return Err(format!("empty placeholder at byte {index}"));
                }
                if !name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                {
                    return Err(format!(
                        "placeholder `{{{name}}}` at byte {index} is not snake_case ASCII"
                    ));
                }
                if !names.iter().any(|existing| existing == name) {
                    names.push(name.to_owned());
                }
                index += 1 + len + 1;
            }
            _ => index += 1,
        }
    }
    names.sort();
    Ok(names)
}

fn render(entries: &BTreeMap<String, Record>) -> Result<String, Box<dyn Error>> {
    let mut out = String::new();
    out.push_str("// @generated by build.rs from assets/i18n/*.json — do not edit.\n\n");

    out.push_str("/// One `pub const` per catalog key.\n");
    out.push_str("///\n");
    out.push_str("/// Generated from `assets/i18n/*.json`. A key that is not in the catalog\n");
    out.push_str("/// has no const, so a typo is a compile error rather than a lookup that\n");
    out.push_str("/// fails at runtime.\n");
    out.push_str("pub mod keys {\n");
    out.push_str("    use super::{Entry, Key};\n");
    for (key, record) in entries {
        out.push('\n');
        writeln!(out, "    /// Catalog key `{key}`.")?;
        out.push_str("    ///\n");
        writeln!(out, "    /// en: {}", doc_safe(&record.values[0]))?;
        writeln!(
            out,
            "    pub const {}: Key = Key(&Entry {{",
            const_name(key)
        )?;
        writeln!(out, "        name: {},", quote(key))?;
        writeln!(out, "        en: {},", quote(&record.values[0]))?;
        writeln!(out, "        zh: {},", quote(&record.values[1]))?;
        writeln!(out, "        ko: {},", quote(&record.values[2]))?;
        let placeholders = record
            .placeholders
            .iter()
            .map(|name| quote(name))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(out, "        placeholders: &[{placeholders}],")?;
        out.push_str("    });\n");
    }
    out.push_str("}\n\n");

    // A `static`, not a `const`: a const of this size trips
    // `clippy::large_const_arrays`, and a const cannot reference a static
    // anyway (E0013), which is why each key above owns a promoted `Entry`
    // rather than indexing one shared table.
    out.push_str("/// Every key in the catalog, sorted by dotted name.\n");
    writeln!(out, "pub(crate) static ALL: [Key; {}] = [", entries.len())?;
    for key in entries.keys() {
        writeln!(out, "    keys::{},", const_name(key))?;
    }
    out.push_str("];\n");
    Ok(out)
}

/// `gateway.already_running` -> `GATEWAY_ALREADY_RUNNING`.
fn const_name(key: &str) -> String {
    key.replace('.', "_").to_ascii_uppercase()
}

/// A Rust string literal for `text`, escaping what a literal cannot carry raw.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// One line of prose, safe to drop into a `///` doc comment.
fn doc_safe(text: &str) -> String {
    let flattened: String = text
        .chars()
        .map(|ch| match ch {
            '\n' | '\r' | '\t' => ' ',
            // A backtick would open a code span the rest of the line never
            // closes; a bracket would look like an intra-doc link.
            '`' => '\'',
            '[' => '(',
            ']' => ')',
            other => other,
        })
        .collect();
    // `<foo>` in a doc comment is parsed as an HTML tag rustdoc then reports
    // as unclosed. The catalog is full of them (`<name>`, `<restored_context>`).
    let flattened = flattened.replace('<', "&lt;").replace('>', "&gt;");
    let trimmed = flattened.trim();
    if trimmed.chars().count() > 160 {
        let head: String = trimmed.chars().take(160).collect();
        format!("{head}…")
    } else {
        trimmed.to_owned()
    }
}

fn env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|err| format!("{name}: {err}"))
}

fn file_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .ok_or_else(|| format!("{} has no usable file name", path.display()))
}
