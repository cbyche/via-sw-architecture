//! Reading a catalogued `exactValue`.
//!
//! The catalogue is a survey document, so a value is written the way a reader
//! wants to read it: a JS array literal for an array, a comma- or
//! pipe-separated list for a vocabulary, a quoted scalar for a scalar. These
//! helpers turn those spellings back into data so a test compares against the
//! catalogue rather than against a literal a test author retyped — which is the
//! whole point, since a retyped literal drifts silently.
//!
//! Nothing here guesses. Each helper does one documented transformation and a
//! test that needs more slices the value explicitly.

/// Strip one matched pair of surrounding `'` or `"` quotes, and trim.
///
/// The catalogue quotes a scalar when it came from a JS string literal
/// (`"'2.0.0'"`) and leaves it bare when it came from a test's expected value
/// (`2.0.0`). Both spellings appear for the same contract, so every scalar
/// comparison goes through here.
pub fn unquote(value: &str) -> &str {
    let value = value.trim();
    let mut chars = value.chars();
    match (chars.next(), chars.next_back()) {
        (Some('\''), Some('\'')) | (Some('"'), Some('"')) if value.len() >= 2 => {
            &value[1..value.len() - 1]
        }
        _ => value,
    }
}

/// Split a catalogued vocabulary into its members.
///
/// Vocabularies are written both ways in the catalogue — `a, b, c` from the
/// defining module and `a | b | c` from the upstream test that locks it — and
/// the two spellings must yield the same list, which is exactly what the
/// wire-vocabulary tests assert. Pipes win when both are present, because a
/// pipe-separated list never contains a comma but a comma-separated one can
/// contain a pipe inside a member's prose.
pub fn list(value: &str) -> Vec<&str> {
    let separator = if value.contains('|') { '|' } else { ',' };
    value
        .split(separator)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect()
}

/// Parse a JS array-of-strings literal, with or without an `Object.freeze(…)`
/// wrapper.
///
/// Returns `None` when the value has no bracketed array in it at all, so a
/// caller can fail with its own message rather than silently seeing an empty
/// list.
pub fn js_string_array(value: &str) -> Option<Vec<&str>> {
    let open = value.find('[')?;
    let close = value.rfind(']')?;
    if close < open {
        return None;
    }
    Some(
        value[open + 1..close]
            .split(',')
            .map(unquote)
            .filter(|item| !item.is_empty())
            .collect(),
    )
}

/// Split the outermost `{…}` of a value into its top-level fields, trimmed.
///
/// The catalogue writes object shapes several ways — JSON with quoted keys
/// (`{"schema":"…","pid":<number>}`), a JavaScript object literal with bare
/// keys (`{ schema: 'x', pid: <int> }`), and a spread sketch with no values at
/// all (`{...base, schema, time (ISO8601), pid}`) — and the *order* of the
/// fields is the contract in every one of them. This does the one thing all
/// three need: find the braced region and cut it at the commas that are not
/// inside a nested `{}`, `[]`, `<>` or a quoted string.
///
/// Fields are returned **raw**, exactly as the catalogue wrote them, because
/// the normalisations differ per contract — some fields carry a type
/// (`pid: <int>`), some a parenthetical (`time (ISO8601)`), some an optionality
/// marker (`message?`), and some are spreads (`...base`) that are not fields at
/// all. [`field_name`] applies the common normalisations; a test that needs
/// something else slices explicitly.
///
/// The region taken is the first `{` to its **matching** `}`, not to the last
/// `}` in the value: several entries continue past the object into prose that
/// contains template braces (`` `${time} ${LEVEL}` ``), and taking the last
/// one would swallow them.
///
/// `None` when the value has no balanced braced region, so a caller fails with
/// its own message rather than silently seeing an empty list.
///
/// ```
/// use via_conformance::value::js_object_fields;
///
/// assert_eq!(
///     js_object_fields("{ owner: <string, default 'gateway'>, pid: <int> }"),
///     Some(vec!["owner: <string, default 'gateway'>", "pid: <int>"]),
/// );
/// // Prose after the object, with braces of its own, is not swallowed.
/// assert_eq!(
///     js_object_fields("record = {a, b}. Console: `${time} ${LEVEL}`"),
///     Some(vec!["a", "b"]),
/// );
/// assert_eq!(js_object_fields("no braces"), None);
/// assert_eq!(js_object_fields("{ unterminated"), None);
/// ```
pub fn js_object_fields(value: &str) -> Option<Vec<&str>> {
    let open = value.find('{')?;

    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut close = None;
    for (offset, character) in value[open..].char_indices() {
        match (quote, character) {
            (Some(open_quote), c) if c == open_quote => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(character),
            (None, '{' | '[' | '<') => depth += 1,
            (None, '}' | ']' | '>') => {
                depth -= 1;
                if depth == 0 && character == '}' {
                    close = Some(open + offset);
                    break;
                }
            }
            (None, _) => {}
        }
    }
    let body = &value[open + 1..close?];

    let mut fields = Vec::new();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut start = 0usize;
    for (index, character) in body.char_indices() {
        match (quote, character) {
            (Some(open_quote), c) if c == open_quote => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(character),
            (None, '{' | '[' | '<') => depth += 1,
            (None, '}' | ']' | '>') => depth -= 1,
            (None, ',') if depth == 0 => {
                fields.push(body[start..index].trim());
                start = index + 1;
            }
            (None, _) => {}
        }
    }
    fields.push(body[start..].trim());
    fields.retain(|field| !field.is_empty());
    Some(fields)
}

/// The field name out of one [`js_object_fields`] entry.
///
/// Three normalisations, each because the catalogue really writes it that way:
///
/// | Written | Name |
/// | --- | --- |
/// | `"schema":"qwaudio.log/v1"` | `schema` |
/// | `time (ISO8601)` | `time` |
/// | `message?` | `message` |
///
/// A spread (`...base`) is not a field and is returned unchanged, including its
/// dots, so a caller can recognise and reject it rather than silently reading
/// `base` as a key.
///
/// ```
/// use via_conformance::value::field_name;
///
/// assert_eq!(field_name("\"pid\":<number>"), "pid");
/// assert_eq!(field_name("time (ISO8601)"), "time");
/// assert_eq!(field_name("message?"), "message");
/// assert_eq!(field_name("...base"), "...base");
/// ```
pub fn field_name(field: &str) -> &str {
    if field.starts_with("...") {
        return field.trim();
    }
    let head = match field.find(':') {
        Some(at) => &field[..at],
        None => field,
    };
    let head = match head.find('(') {
        Some(at) => &head[..at],
        None => head,
    };
    unquote(head.trim().trim_end_matches('?').trim())
}

/// The part of a value before `marker`, trimmed.
///
/// Several catalogue entries append a clause to a list — "…, acp - plus the
/// sentinel 'none' …". Slicing that off explicitly, at the call site, keeps the
/// list parser from having to guess where data ends and prose begins.
pub fn before<'a>(value: &'a str, marker: &str) -> &'a str {
    match value.find(marker) {
        Some(at) => value[..at].trim(),
        None => value.trim(),
    }
}

/// The part of a value after `marker`, trimmed.
///
/// The companion of [`before`]. Together they let a test say "the list between
/// `modelCapabilities keys:` and the next full stop" without a regex and
/// without guessing.
pub fn after<'a>(value: &'a str, marker: &str) -> &'a str {
    match value.find(marker) {
        Some(at) => value[at + marker.len()..].trim(),
        None => value.trim(),
    }
}

/// Every single-quoted substring, in order.
///
/// The catalogue writes JS object fragments verbatim, so a contract like
/// `voice: 'longanqian'; turnDetection: { type: 'smart_turn' }` carries its two
/// literals inside quotes and nothing else does. Pulling them out in order
/// gives an exact, ordered comparison without having to parse JavaScript.
///
/// Unterminated quotes yield nothing rather than running to the end of the
/// value, so a malformed catalogue entry produces an empty list a caller can
/// reject rather than a plausible-looking wrong one.
pub fn quoted_literals(value: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while let Some(open) = bytes[index..].iter().position(|byte| *byte == b'\'') {
        let start = index + open + 1;
        match bytes[start..].iter().position(|byte| *byte == b'\'') {
            Some(close) => {
                found.push(&value[start..start + close]);
                index = start + close + 1;
            }
            None => break,
        }
    }
    found
}

/// Apply `docs/rebrand.md`'s environment-variable rename rules to an upstream
/// name.
///
/// The rules, in the order they must be tried — the `QWEN_AUDIO_AGENT_` prefix
/// has to be tested before `QWEN_AUDIO_`, or every `_AGENT_` name would come
/// out as `VIA_AGENT_`:
///
/// | Upstream prefix | VIA prefix | `docs/rebrand.md` |
/// | --- | --- | --- |
/// | `QWEN_AUDIO_AGENT_` | `VIA_` | RENAME, `QWEN_AUDIO_AGENT_*` → `VIA_*` |
/// | `QWAUDIO_` | `VIA_` | RENAME, `QWAUDIO_*` → `VIA_*` |
/// | `QWEN_OMNI_` | `VIA_OMNI_` | RENAME, and the row that says the Audio/Omni family distinction must survive the rename |
/// | `QWEN_AUDIO_` | `VIA_` | RENAME, `QWEN_AUDIO_*` (non-AGENT variants) |
///
/// Everything else is returned unchanged, which is the KEEP half of the table:
/// `DASHSCOPE_*`, `QWEN_CODE_*`, `ANTHROPIC_*`, `OPENAI_*`, `OPENCODE_*` and the
/// rest name someone else's product.
///
/// Deriving VIA's name from the upstream one, rather than retyping it, is what
/// makes a *partial* rename fail — `docs/architecture.md` §13 calls that out as
/// the failure mode with real risk.
pub fn rebranded(upstream: &str) -> String {
    const RULES: &[(&str, &str)] = &[
        ("QWEN_AUDIO_AGENT_", "VIA_"),
        ("QWAUDIO_", "VIA_"),
        ("QWEN_OMNI_", "VIA_OMNI_"),
        ("QWEN_AUDIO_", "VIA_"),
    ];
    for (from, to) in RULES {
        if let Some(rest) = upstream.strip_prefix(from) {
            return format!("{to}{rest}");
        }
    }
    upstream.to_owned()
}

/// Apply `docs/rebrand.md`'s schema-string rename to an upstream schema id.
///
/// Two schema strings cross a process boundary and both carry the old product
/// prefix: `qwaudio.gateway-lock/v1` (written into `<configDir>/gateway.lock`
/// and **required to match on read**) and `qwaudio.log/v1` (stamped on every
/// log line). `docs/rebrand.md` renames the prefix and nothing else — the
/// suffix after the dot, including the `/v1` version, is untouched, so a
/// version bump is still a version bump and not a rebrand.
///
/// Anything without the upstream prefix is returned unchanged, so calling this
/// on an already-renamed value is a no-op rather than a double rename.
///
/// ```
/// use via_conformance::value::rebranded_schema;
///
/// let upstream = "qwaudio.log/v1";
/// assert_eq!(rebranded_schema(upstream), "via.log/v1");
/// // Idempotent: applying it to an already-renamed value is a no-op.
/// assert_eq!(rebranded_schema("via.log/v1"), "via.log/v1");
/// ```
pub fn rebranded_schema(upstream: &str) -> String {
    // The upstream prefix, spelled once. `docs/rebrand.md`:
    // `qwaudio.gateway-lock/v1` -> `via.gateway-lock/v1`, `qwaudio.log/v1` ->
    // `via.log/v1`.
    const UPSTREAM_PREFIX: &str = "qwaudio.";
    const VIA_PREFIX: &str = "via.";
    match upstream.strip_prefix(UPSTREAM_PREFIX) {
        Some(rest) => format!("{VIA_PREFIX}{rest}"),
        None => upstream.to_owned(),
    }
}

/// A rough indicator of whether an `exactValue` is a literal or prose.
///
/// **A heuristic, deliberately not a gate.** It exists so the coverage summary
/// can report how much of the catalogue is a value that can be string-compared
/// at all, which is a fair signal of how much of the remaining work is
/// byte-equality assertion and how much is behavioural testing. Nothing branches
/// on it.
///
/// The rule: prose-shaped if the value contains a sentence break (`". "`) or
/// runs past 240 characters. Both are cheap, both are stable, and both are
/// wrong at the margins — a long JSON Schema counts as prose, and a one-sentence
/// behavioural note counts as a literal.
pub fn is_prose_shaped(exact_value: &str) -> bool {
    exact_value.contains(". ") || exact_value.chars().count() > 240
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unquote_strips_one_matched_pair() {
        assert_eq!(unquote("'2.0.0'"), "2.0.0");
        assert_eq!(unquote("\"2.0.0\""), "2.0.0");
        assert_eq!(unquote("2.0.0"), "2.0.0");
        assert_eq!(unquote("  'a'  "), "a");
        // Not a matched pair: left alone rather than mangled.
        assert_eq!(unquote("'a\""), "'a\"");
        assert_eq!(unquote("'"), "'");
    }

    #[test]
    fn list_accepts_both_catalogue_spellings() {
        assert_eq!(list("a, b, c"), ["a", "b", "c"]);
        assert_eq!(list("a | b | c"), ["a", "b", "c"]);
        assert_eq!(list(""), Vec::<&str>::new());
    }

    #[test]
    fn js_string_array_unwraps_object_freeze() {
        assert_eq!(
            js_string_array("Object.freeze(['a','b'])"),
            Some(vec!["a", "b"])
        );
        assert_eq!(js_string_array("['a', 'b']"), Some(vec!["a", "b"]));
        assert_eq!(js_string_array("not an array"), None);
    }

    #[test]
    fn rebranded_tries_the_agent_prefix_first() {
        assert_eq!(
            rebranded("QWEN_AUDIO_AGENT_ACP_FORWARD_ENV"),
            "VIA_ACP_FORWARD_ENV"
        );
        assert_eq!(rebranded("QWAUDIO_CONFIG_DIR"), "VIA_CONFIG_DIR");
        assert_eq!(rebranded("QWEN_AUDIO_REALTIME_VOICE"), "VIA_REALTIME_VOICE");
        assert_eq!(
            rebranded("QWEN_OMNI_REALTIME_VOICE"),
            "VIA_OMNI_REALTIME_VOICE"
        );
        // KEEP: someone else's namespace.
        assert_eq!(rebranded("QWEN_CODE_WORKSPACE"), "QWEN_CODE_WORKSPACE");
        assert_eq!(rebranded("DASHSCOPE_API_KEY"), "DASHSCOPE_API_KEY");
    }

    #[test]
    fn js_object_fields_stops_at_the_matching_brace() {
        // The log record envelope's real shape: spreads, quoted keys, angle
        // brackets, an optionality marker, and template braces in the prose
        // that follows.
        let envelope = "One JSON object per line: {...base, \"level\":<a|b>, \
                        \"message\"?:<string>}. Console: `${time} ${LEVEL}`";
        assert_eq!(
            js_object_fields(envelope),
            Some(vec!["...base", "\"level\":<a|b>", "\"message\"?:<string>"])
        );
        // A comma inside <> or inside quotes is not a field separator.
        assert_eq!(
            js_object_fields("{a: <x, y>, b: 'p, q'}"),
            Some(vec!["a: <x, y>", "b: 'p, q'"])
        );
        assert_eq!(js_object_fields("{}"), Some(Vec::new()));
        assert_eq!(js_object_fields("nothing"), None);
        assert_eq!(js_object_fields("{ never closed"), None);
    }

    #[test]
    fn field_name_strips_types_parentheticals_and_optionality() {
        assert_eq!(field_name("\"pid\":<number>"), "pid");
        assert_eq!(field_name("time (ISO-8601)"), "time");
        assert_eq!(field_name("\"message\"?:<string>"), "message");
        assert_eq!(field_name("message?"), "message");
        assert_eq!(field_name("schema: 'x'"), "schema");
        // A spread is not a field, and says so rather than reading as `base`.
        assert_eq!(field_name("...base"), "...base");
    }

    #[test]
    fn rebranded_schema_renames_only_the_prefix() {
        let upstream_log = "qwaudio.log/v1";
        let upstream_lock = "qwaudio.gateway-lock/v1";
        assert_eq!(rebranded_schema(upstream_log), "via.log/v1");
        assert_eq!(rebranded_schema(upstream_lock), "via.gateway-lock/v1");
        // Idempotent, and inert on anything that is not an upstream schema.
        assert_eq!(rebranded_schema("via.log/v1"), "via.log/v1");
        assert_eq!(rebranded_schema("acp/v1"), "acp/v1");
        assert_eq!(rebranded_schema(""), "");
    }

    #[test]
    fn before_slices_prose_off_a_list() {
        assert_eq!(before("a, b - plus the sentinel", " - plus"), "a, b");
        assert_eq!(before("a, b", " - plus"), "a, b");
    }

    #[test]
    fn after_slices_the_lead_in_off_a_list() {
        assert_eq!(after("keys: a, b. rest", "keys:"), "a, b. rest");
        assert_eq!(after("a, b", "keys:"), "a, b");
    }

    #[test]
    fn quoted_literals_are_returned_in_order() {
        assert_eq!(
            quoted_literals("voice: 'longanqian'; turnDetection: { type: 'smart_turn' }"),
            ["longanqian", "smart_turn"]
        );
        assert_eq!(quoted_literals("no quotes here"), Vec::<&str>::new());
        // An unterminated quote yields nothing rather than a plausible lie.
        assert_eq!(quoted_literals("'unterminated"), Vec::<&str>::new());
    }
}
