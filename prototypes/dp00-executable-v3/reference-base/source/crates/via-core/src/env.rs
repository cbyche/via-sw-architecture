//! The environment as data, and the two coercions upstream applies to it.
//!
//! Everything in VIA that reads configuration reads it through an [`EnvMap`].
//! The real process environment is sampled **once**, by the binary, via
//! [`EnvMap::from_process`]; every layer below that takes the map as a
//! parameter. That is not merely tidy: `std::env::set_var` is `unsafe` in
//! edition 2024, so a test that wanted to exercise a code path by mutating the
//! real environment could not do so without `unsafe` and process-wide
//! serialization. With a pure loader the configuration tests need neither.
//!
//! Two coercions carry contracts:
//!
//! * [`number_setting`] reproduces upstream's `numberSetting`
//!   (`server/src/core/config.mjs:23-33`), including that an out-of-range value
//!   is **clamped, not rejected**, and that `Number()` — not `parseInt` — is the
//!   parser. `docs/reference/contracts.json` calls the clamping out by name:
//!   "Clamping (not rejecting) is a behaviour a strict Rust parser would
//!   silently change."
//! * [`js_number`] is that `Number()`, because `parseInt` and `Number` disagree
//!   on exactly the inputs an operator types by accident (`"12abc"`, `"1e4"`,
//!   `"0x10"`).

use std::collections::BTreeMap;

use crate::text::is_js_whitespace;

/// A snapshot of an environment: variable name to value.
///
/// Ordering is deterministic (`BTreeMap`) so a debug rendering of a map is
/// stable, but no caller may depend on iteration order — upstream iterates
/// `process.env`, whose order is insertion order, and nothing observable is
/// derived from it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvMap(BTreeMap<String, String>);

impl EnvMap {
    /// An empty environment.
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Sample the real process environment.
    ///
    /// **The only impure entry point in this crate's configuration path.** Call
    /// it once, in `main`, and pass the result down.
    #[must_use]
    pub fn from_process() -> Self {
        Self(std::env::vars().collect())
    }

    /// The raw value of `key`, exactly as stored.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    /// The value of `key`, mapping the empty string to `None`.
    ///
    /// This is JavaScript's `env.X || fallback` truthiness test, which upstream
    /// uses at nearly every read site: an empty shell assignment behaves as
    /// though the variable were unset.
    #[must_use]
    pub fn get_truthy(&self, key: &str) -> Option<&str> {
        self.get(key).filter(|value| !value.is_empty())
    }

    /// `String(env.X || '').trim()` — the other upstream idiom.
    #[must_use]
    pub fn get_trimmed(&self, key: &str) -> &str {
        self.get(key).unwrap_or_default().trim()
    }

    /// The first of `keys` that is present and non-empty.
    ///
    /// Reproduces upstream's `env.A || env.B || …` fallback chains, e.g.
    /// `VIA_REALTIME_BASE_URL || VIA_REALTIME_URL`
    /// (`shared/realtime-provider-catalog.mjs:89-98`).
    #[must_use]
    pub fn first_truthy(&self, keys: &[&str]) -> Option<&str> {
        keys.iter().find_map(|key| self.get_truthy(key))
    }

    /// Whether `key` is present, even as the empty string.
    ///
    /// The distinction matters: upstream's env-file loader only fills a key
    /// whose current value is `undefined`, so `KEY=` in the shell masks every
    /// file (`shared/runtime-environment.mjs:86-88`).
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    /// Set `key`, replacing any existing value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    /// Set `key` only if it is not already present.
    ///
    /// The env-file layering rule: the first source to define a key wins
    /// (`shared/runtime-environment.mjs:86-88`).
    pub fn set_if_absent(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.entry(key.into()).or_insert_with(|| value.into());
    }

    /// Remove `key`, returning its value if it was present.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.0.remove(key)
    }

    /// Number of variables.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the map holds no variables.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate every variable in name order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Overlay every entry of `other` on top of this map.
    ///
    /// Used to apply explicit overrides, which sit above the environment in the
    /// layering order.
    pub fn overlay(&mut self, other: &Self) {
        for (key, value) in other.iter() {
            self.set(key, value);
        }
    }

    /// A reader closure of the shape `via_i18n::resolve_locale` expects.
    pub fn reader(&self) -> impl Fn(&str) -> Option<String> + '_ {
        move |key: &str| self.get(key).map(str::to_owned)
    }
}

impl<K: Into<String>, V: Into<String>> FromIterator<(K, V)> for EnvMap {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl via_log::EnvSource for EnvMap {
    fn get(&self, key: &str) -> Option<String> {
        Self::get(self, key).map(str::to_owned)
    }
}

/// JavaScript's `Number(string)`, returning `None` for `NaN`.
///
/// **External contract** — `server/src/core/config.mjs:30-31` calls `Number()`,
/// not `parseInt()`, and the two disagree on exactly the values an operator
/// mistypes:
///
/// | input | `Number` | `parseInt` |
/// | --- | --- | --- |
/// | `"12abc"` | `NaN` (→ fallback) | `12` |
/// | `"1e4"` | `10000` | `1` |
/// | `"0x10"` | `16` | `16` |
/// | `"  7 "` | `7` | `7` |
/// | `""` | `0` | `NaN` |
///
/// The empty string is `0` here, as in JavaScript; [`number_setting`] returns
/// the fallback for it *before* reaching this function, which is what upstream
/// does too.
///
/// `Infinity` parses to an infinity, which [`number_setting`] then rejects via
/// its `Number.isFinite` check. Rust's own `f64::from_str` additionally accepts
/// `"inf"` and `"nan"`; those are `NaN` in JavaScript and are rejected here.
#[must_use]
pub fn js_number(source: &str) -> Option<f64> {
    let text = source.trim_matches(is_js_whitespace);
    if text.is_empty() {
        return Some(0.0);
    }
    if let Some(value) = parse_infinity(text) {
        return Some(value);
    }
    // A radix literal, valid or not: `Number("0xZZ")` is NaN, it does not fall
    // back to decimal parsing.
    if let Some(parsed) = parse_radix_prefixed(text) {
        return parsed;
    }
    parse_decimal(text)
}

fn parse_infinity(text: &str) -> Option<f64> {
    match text {
        "Infinity" | "+Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        _ => None,
    }
}

/// `0x` / `0o` / `0b` literals. No sign is permitted, matching JavaScript.
///
/// The outer `Option` says "this was a radix literal"; the inner one says
/// "and it parsed". `Number("0x")` and `Number("0xZZ")` are both `NaN` — they
/// do **not** fall back to decimal parsing — which is why the two levels are
/// distinct.
fn parse_radix_prefixed(text: &str) -> Option<Option<f64>> {
    let (radix, digits) = match text.as_bytes() {
        [b'0', b'x' | b'X', rest @ ..] => (16u32, rest),
        [b'0', b'o' | b'O', rest @ ..] => (8u32, rest),
        [b'0', b'b' | b'B', rest @ ..] => (2u32, rest),
        _ => return None,
    };
    if digits.is_empty() {
        return Some(None);
    }
    let mut value = 0f64;
    for &byte in digits {
        let Some(digit) = (byte as char).to_digit(radix) else {
            return Some(None);
        };
        value = value * f64::from(radix) + f64::from(digit);
    }
    Some(Some(value))
}

/// `StrDecimalLiteral`: `[+-]? ( d+ ('.' d*)? | '.' d+ ) ([eE] [+-]? d+)?`.
///
/// Written out rather than delegated to `f64::from_str` because that accepts
/// `inf`, `infinity` and `nan` in any case, all of which are `NaN` in
/// JavaScript.
fn parse_decimal(text: &str) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut index = 0usize;

    if matches!(bytes.first(), Some(b'+' | b'-')) {
        index += 1;
    }

    let integral = count_digits(bytes, index);
    index += integral;

    let mut fractional = 0usize;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        fractional = count_digits(bytes, index);
        index += fractional;
    }
    if integral == 0 && fractional == 0 {
        return None;
    }

    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let exponent = count_digits(bytes, index);
        if exponent == 0 {
            return None;
        }
        index += exponent;
    }

    if index != bytes.len() {
        return None;
    }
    text.parse::<f64>().ok()
}

fn count_digits(bytes: &[u8], from: usize) -> usize {
    bytes[from..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count()
}

/// The clamp bounds of a [`number_setting`], as upstream's options object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    /// Lower clamp; `f64::NEG_INFINITY` when upstream omits `min`.
    pub min: f64,
    /// Upper clamp; `f64::INFINITY` when upstream omits `max`.
    pub max: f64,
}

impl Bounds {
    /// No clamping at all — upstream's `{}`.
    pub const UNBOUNDED: Self = Self {
        min: f64::NEG_INFINITY,
        max: f64::INFINITY,
    };

    /// `{ min }` only.
    #[must_use]
    pub const fn min(min: f64) -> Self {
        Self {
            min,
            max: f64::INFINITY,
        }
    }

    /// `{ min, max }`.
    #[must_use]
    pub const fn range(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}

/// Upstream's `numberSetting(value, fallback, { min, max })`.
///
/// **External contract** — `server/src/core/config.mjs:23-33`:
///
/// 1. absent → `fallback`;
/// 2. empty or whitespace-only → `fallback`;
/// 3. not a finite `Number()` → `fallback`;
/// 4. otherwise `Math.min(max, Math.max(min, parsed))`.
///
/// Step 4's order is reproduced literally: when `min > max` the *maximum* wins,
/// because `Math.min` is applied last. No catalogued setting has crossed bounds,
/// but a caller could construct them.
///
/// An out-of-range value is **clamped, never rejected**. A strict Rust parser
/// that errored here would refuse to start on a configuration upstream accepts.
#[must_use]
pub fn number_setting(value: Option<&str>, fallback: f64, bounds: Bounds) -> f64 {
    let Some(raw) = value else {
        return fallback;
    };
    let source = raw.trim();
    if source.is_empty() {
        return fallback;
    }
    match js_number(source) {
        Some(parsed) if parsed.is_finite() => parsed.max(bounds.min).min(bounds.max),
        _ => fallback,
    }
}

/// [`number_setting`] truncated toward zero.
///
/// Every catalogued numeric setting is a count, a millisecond duration or a
/// port, all of which VIA models as integers. JavaScript keeps a fractional
/// `Number` — `VIA_TASK_MAX_CONCURRENT=2.5` yields `2.5` upstream — and then
/// uses it in comparisons where the fraction is invisible. Truncating toward
/// zero is the one place this crate does not reproduce upstream bit for bit;
/// see `docs/deviations/phase-1.md`.
#[must_use]
pub fn integer_setting(value: Option<&str>, fallback: i64, bounds: Bounds) -> i64 {
    let resolved = number_setting(value, fallback as f64, bounds);
    // Saturating: the bounds of every catalogued setting are inside i64, and a
    // value beyond it can only come from an operator typing `1e300`.
    if resolved >= i64::MAX as f64 {
        i64::MAX
    } else if resolved <= i64::MIN as f64 {
        i64::MIN
    } else {
        resolved as i64
    }
}

/// `String(env.X || fallback).toLowerCase() === expected`.
///
/// Upstream's flag idiom, used for `VIA_ANNOUNCE_INTO_CONTEXT` (`'true'`),
/// `VIA_REMINDER_SCHEDULER` (`'true'`), `VIA_WAKE_WORD_ENABLED` (`''`) and
/// `VIA_MEMORY_AUTO` (`'on'`, inverted). The comparison is against the
/// lowercased value, so `TRUE` and `True` both match.
#[must_use]
pub fn lowercase_equals(value: Option<&str>, fallback: &str, expected: &str) -> bool {
    let raw = value.filter(|text| !text.is_empty()).unwrap_or(fallback);
    raw.to_lowercase() == expected
}

/// Split a comma-separated list, trim each entry and drop the empties.
///
/// **External contract** — `server/src/core/config.mjs:196-199`
/// (`VIA_ALLOWED_ORIGINS`) and `shared/backend-catalog.mjs:387`
/// (`VIA_ACP_FORWARD_ENV`).
#[must_use]
pub fn comma_list(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_number_matches_the_javascript_table() {
        assert_eq!(js_number("12abc"), None);
        assert_eq!(js_number("1e4"), Some(10_000.0));
        assert_eq!(js_number("0x10"), Some(16.0));
        assert_eq!(js_number("  7 "), Some(7.0));
        assert_eq!(js_number(""), Some(0.0));
        assert_eq!(js_number("inf"), None);
        assert_eq!(js_number("nan"), None);
        assert_eq!(js_number("NaN"), None);
        assert_eq!(js_number("Infinity"), Some(f64::INFINITY));
        assert_eq!(js_number("1_000"), None);
    }

    #[test]
    fn number_setting_clamps_rather_than_rejecting() {
        let bounds = Bounds::range(1.0, 65_535.0);
        assert_eq!(number_setting(Some("70000"), 3101.0, bounds), 65_535.0);
        assert_eq!(number_setting(Some("0"), 3101.0, bounds), 1.0);
        assert_eq!(number_setting(Some("nope"), 3101.0, bounds), 3101.0);
    }

    #[test]
    fn an_empty_shell_assignment_still_masks_a_file() {
        let mut env = EnvMap::new();
        env.set("VIA_BACKEND_MODEL", "");
        assert!(env.contains_key("VIA_BACKEND_MODEL"));
        assert_eq!(env.get_truthy("VIA_BACKEND_MODEL"), None);
        env.set_if_absent("VIA_BACKEND_MODEL", "from-file");
        assert_eq!(env.get("VIA_BACKEND_MODEL"), Some(""));
    }
}
