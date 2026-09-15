//! Just enough JSON5 to read a third-party OpenClaw configuration.
//!
//! `docs/reference/contracts.json` (`env-var/OPENCLAW_CONFIG_PATH / OPENCLAW_STATE_DIR / HOME`)
//! records the requirement: *"JSON5 (not JSON) and the three token-reference
//! shapes are exact parsing contracts against a third-party config file."*
//! Upstream imports the `json5` npm package; VIA has no JSON5 crate in its
//! dependency set, and adding one for a single field read would be a poor
//! trade.
//!
//! So this is a **normaliser**, not a parser: it rewrites JSON5 source into JSON
//! source, then hands it to `serde_json`. The four extensions that appear in
//! real OpenClaw configurations — and in VIA's own shipped
//! [`openclaw.json5`](crate::openclaw::OPENCLAW_CONFIG_JSON5) — are covered:
//!
//! 1. `//` and `/* … */` comments,
//! 2. unquoted identifier keys,
//! 3. single-quoted strings,
//! 4. trailing commas in objects and arrays.
//!
//! # What it deliberately does not do
//!
//! The rest of JSON5 — hexadecimal and leading-`.` numbers, `Infinity`, `NaN`,
//! `+1`, escaped newlines inside strings, and non-ASCII identifier characters in
//! keys — is not translated. None of it can appear in the one path that is read
//! (`gateway.auth.token` is a string or an object of strings), and a file that
//! uses it simply yields `None`, which every caller already treats as *"there is
//! no token to reuse"* rather than as an error. Recorded as a deviation.

/// Parse JSON5 source into a JSON value.
///
/// `None` when the source is not valid JSON5 in the subset described above.
#[must_use]
pub fn parse(source: &str) -> Option<serde_json::Value> {
    serde_json::from_str(&to_json(source)?).ok()
}

/// Rewrite JSON5 source as JSON source.
///
/// Exposed for testing the translation itself, which is where the subtleties
/// are: a `//` inside a string is not a comment, and a `,` inside a string is
/// not a trailing comma.
#[must_use]
pub fn to_json(source: &str) -> Option<String> {
    let mut out = String::with_capacity(source.len());
    let characters: Vec<char> = source.chars().collect();
    let mut index = 0usize;
    while index < characters.len() {
        let current = characters[index];
        match current {
            '/' if characters.get(index + 1) == Some(&'/') => {
                while index < characters.len() && characters[index] != '\n' {
                    index += 1;
                }
            }
            '/' if characters.get(index + 1) == Some(&'*') => {
                index += 2;
                loop {
                    if index >= characters.len() {
                        // An unterminated block comment is not JSON5.
                        return None;
                    }
                    if characters[index] == '*' && characters.get(index + 1) == Some(&'/') {
                        index += 2;
                        break;
                    }
                    index += 1;
                }
            }
            '"' => {
                let (text, next) = read_string(&characters, index, '"')?;
                out.push_str(&render_string(&text));
                index = next;
            }
            '\'' => {
                let (text, next) = read_string(&characters, index, '\'')?;
                out.push_str(&render_string(&text));
                index = next;
            }
            ',' => {
                // A trailing comma is one followed only by whitespace or a
                // comment before the closing bracket.
                let next = skip_trivia(&characters, index + 1)?;
                if matches!(characters.get(next), Some('}' | ']')) {
                    index = next;
                } else {
                    out.push(',');
                    index += 1;
                }
            }
            _ if is_identifier_start(current) => {
                let start = index;
                while index < characters.len() && is_identifier_part(characters[index]) {
                    index += 1;
                }
                let word: String = characters[start..index].iter().collect();
                // A bare word followed by `:` is a key and needs quoting; the
                // three JSON literals are values and must not be.
                let next = skip_trivia(&characters, index)?;
                if characters.get(next) == Some(&':')
                    && !matches!(word.as_str(), "true" | "false" | "null")
                {
                    out.push_str(&render_string(&word));
                } else {
                    out.push_str(&word);
                }
            }
            _ => {
                out.push(current);
                index += 1;
            }
        }
    }
    Some(out)
}

fn is_identifier_start(value: char) -> bool {
    value.is_ascii_alphabetic() || value == '_' || value == '$'
}

fn is_identifier_part(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_' || value == '$'
}

/// Skip whitespace and comments, returning the next significant index.
fn skip_trivia(characters: &[char], mut index: usize) -> Option<usize> {
    loop {
        while index < characters.len() && characters[index].is_whitespace() {
            index += 1;
        }
        if characters.get(index) == Some(&'/') && characters.get(index + 1) == Some(&'/') {
            while index < characters.len() && characters[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if characters.get(index) == Some(&'/') && characters.get(index + 1) == Some(&'*') {
            index += 2;
            loop {
                if index >= characters.len() {
                    return None;
                }
                if characters[index] == '*' && characters.get(index + 1) == Some(&'/') {
                    index += 2;
                    break;
                }
                index += 1;
            }
            continue;
        }
        return Some(index);
    }
}

/// Read a quoted string, returning its decoded contents and the index after the
/// closing quote.
fn read_string(characters: &[char], start: usize, quote: char) -> Option<(String, usize)> {
    let mut text = String::new();
    let mut index = start + 1;
    while index < characters.len() {
        let current = characters[index];
        if current == '\\' {
            let escaped = *characters.get(index + 1)?;
            match escaped {
                'n' => text.push('\n'),
                't' => text.push('\t'),
                'r' => text.push('\r'),
                'b' => text.push('\u{8}'),
                'f' => text.push('\u{c}'),
                'u' => {
                    let hex: String = characters.get(index + 2..index + 6)?.iter().collect();
                    let code = u32::from_str_radix(&hex, 16).ok()?;
                    text.push(char::from_u32(code)?);
                    index += 4;
                }
                other => text.push(other),
            }
            index += 2;
            continue;
        }
        if current == quote {
            return Some((text, index + 1));
        }
        text.push(current);
        index += 1;
    }
    None
}

/// Render a string as a JSON string literal.
fn render_string(value: &str) -> String {
    serde_json::Value::String(value.to_owned()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_shipped_openclaw_configuration() {
        let parsed = parse(crate::openclaw::OPENCLAW_CONFIG_JSON5)
            .expect("the shipped asset is valid JSON5");
        assert_eq!(
            parsed
                .get("gateway")
                .and_then(|gateway| gateway.get("bind"))
                .and_then(serde_json::Value::as_str),
            Some("loopback")
        );
    }

    #[test]
    fn handles_comments_bare_keys_single_quotes_and_trailing_commas() {
        let parsed = parse(
            r#"{
              // a line comment
              /* and a block one */
              mode: 'local',
              nested: { a: 1, b: [1, 2, 3,], },
            }"#,
        )
        .expect("valid JSON5");
        assert_eq!(parsed["mode"], serde_json::json!("local"));
        assert_eq!(parsed["nested"]["b"], serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn does_not_treat_string_contents_as_syntax() {
        let parsed =
            parse(r#"{ url: "https://example.test//path", note: 'a, b', }"#).expect("valid JSON5");
        assert_eq!(
            parsed["url"],
            serde_json::json!("https://example.test//path")
        );
        assert_eq!(parsed["note"], serde_json::json!("a, b"));
    }

    #[test]
    fn keeps_json_literals_unquoted() {
        let parsed = parse("{ a: true, b: false, c: null }").expect("valid JSON5");
        assert_eq!(parsed["a"], serde_json::json!(true));
        assert_eq!(parsed["b"], serde_json::json!(false));
        assert_eq!(parsed["c"], serde_json::Value::Null);
    }

    #[test]
    fn decodes_escapes_and_re_encodes_them_as_json() {
        let parsed = parse(r#"{ a: 'line\nbreak', b: 'A', c: 'quote\'s' }"#).expect("valid JSON5");
        assert_eq!(parsed["a"], serde_json::json!("line\nbreak"));
        assert_eq!(parsed["b"], serde_json::json!("A"));
        assert_eq!(parsed["c"], serde_json::json!("quote's"));
    }

    #[test]
    fn plain_json_still_parses() {
        let parsed = parse(r#"{"gateway":{"auth":{"token":"abc"}}}"#).expect("JSON is JSON5");
        assert_eq!(parsed["gateway"]["auth"]["token"], serde_json::json!("abc"));
    }

    #[test]
    fn refuses_what_it_cannot_translate() {
        assert!(parse("{ a: ").is_none());
        assert!(parse("{ a: 'unterminated }").is_none());
        assert!(parse("{ /* unterminated").is_none());
        assert!(parse("not a config").is_none());
    }
}
