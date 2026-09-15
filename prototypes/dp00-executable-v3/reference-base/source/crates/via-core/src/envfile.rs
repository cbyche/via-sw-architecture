//! `config.env` / `.env` parsing and the layering rule around it.
//!
//! Upstream parses these files with `node:util`'s `parseEnv`
//! (`shared/runtime-environment.mjs:79`) and applies them with one rule that is
//! the whole contract:
//!
//! > each file only sets keys whose current value is strictly `undefined`, so
//! > the **first** source wins and an empty shell assignment masks all files
//! > — `docs/reference/contracts.json`, *env file load precedence*
//!
//! The load order is `<root>/.env.local`, then `<root>/.env`, then
//! `<dataDirectory>/config.env` (`shared/runtime-environment.mjs:471-476`), and
//! the process environment sits above all three because it is already populated
//! before the first file is read.
//!
//! One deliberate exception exists and is implemented in [`crate::runtime`]:
//! `VIA_AUTH_SECRET` is *deleted* from the environment before `state.env` is
//! read, so an empty shell assignment cannot mask the persisted local identity
//! (`shared/runtime-environment.mjs:118`).

use std::collections::BTreeMap;

/// Parse the contents of an env file into name/value pairs, in file order.
///
/// A faithful subset of Node's `util.parseEnv` (`src/node_dotenv.cc`):
///
/// * a line whose first non-blank character is `#` is a comment;
/// * `export ` before a key is stripped;
/// * a key runs to the first `=` and is trimmed;
/// * a value that opens with `'`, `"` or `` ` `` runs to the next matching
///   quote, may span lines, and is taken verbatim;
/// * an unquoted value runs to the end of the line, is truncated at the first
///   `#` anywhere in it, and is then trimmed;
/// * a trailing fragment with no `=` ends the parse.
///
/// Not reproduced: Node's `\n` unescaping inside double quotes, which no VIA
/// template or documented value uses. Recorded in `docs/deviations/phase-1.md`.
#[must_use]
pub fn parse_env(contents: &str) -> Vec<(String, String)> {
    let bytes = contents.as_bytes();
    let mut index = 0usize;
    let mut entries = Vec::new();

    while index < bytes.len() {
        // Skip blank space and whole comment lines.
        while index < bytes.len() && (bytes[index] as char).is_whitespace() {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        if bytes[index] == b'#' {
            index = end_of_line(bytes, index);
            continue;
        }

        let Some(equals) = find_byte(bytes, index, b'=') else {
            break;
        };
        let key = contents[index..equals].trim();
        let key = key.strip_prefix("export ").unwrap_or(key).trim();
        index = equals + 1;

        // Node skips spaces and tabs but not newlines between `=` and a value.
        while index < bytes.len() && matches!(bytes[index], b' ' | b'\t') {
            index += 1;
        }

        let value = match bytes.get(index) {
            Some(&quote @ (b'\'' | b'"' | b'`')) => {
                index += 1;
                match find_byte(bytes, index, quote) {
                    Some(closing) => {
                        let raw = &contents[index..closing];
                        index = closing + 1;
                        raw.to_owned()
                    }
                    None => {
                        // Unterminated quote: Node takes the rest of the file.
                        let raw = contents[index..].to_owned();
                        index = bytes.len();
                        raw
                    }
                }
            }
            _ => {
                let line_end = end_of_line(bytes, index);
                let raw = &contents[index..line_end];
                index = line_end;
                let truncated = raw.split('#').next().unwrap_or(raw);
                truncated.trim().to_owned()
            }
        };

        if !key.is_empty() {
            entries.push((key.to_owned(), value));
        }
    }

    entries
}

fn find_byte(bytes: &[u8], from: usize, needle: u8) -> Option<usize> {
    bytes[from..]
        .iter()
        .position(|byte| *byte == needle)
        .map(|offset| from + offset)
}

fn end_of_line(bytes: &[u8], from: usize) -> usize {
    find_byte(bytes, from, b'\n').unwrap_or(bytes.len())
}

/// Parse an env file into a map, last occurrence of a duplicated key winning.
///
/// `parseEnv` builds a JavaScript object, so a repeated key keeps the last
/// assignment.
#[must_use]
pub fn parse_env_map(contents: &str) -> BTreeMap<String, String> {
    parse_env(contents).into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_shapes_the_templates_use() {
        let parsed = parse_env_map(
            "# a comment\nDASHSCOPE_API_KEY=\nAGENT_PROTOCOL=openclaw\n\n# another\nVIA_REALTIME_PROVIDER=dashscope\n",
        );
        assert_eq!(
            parsed.get("DASHSCOPE_API_KEY").map(String::as_str),
            Some("")
        );
        assert_eq!(
            parsed.get("AGENT_PROTOCOL").map(String::as_str),
            Some("openclaw")
        );
        assert_eq!(
            parsed.get("VIA_REALTIME_PROVIDER").map(String::as_str),
            Some("dashscope")
        );
        assert_eq!(parsed.len(), 3);
    }

    #[test]
    fn truncates_an_unquoted_value_at_a_hash_and_keeps_a_quoted_one() {
        let parsed = parse_env_map("A=one # trailing\nB=\"two # kept\"\nC='three'\n");
        assert_eq!(parsed.get("A").map(String::as_str), Some("one"));
        assert_eq!(parsed.get("B").map(String::as_str), Some("two # kept"));
        assert_eq!(parsed.get("C").map(String::as_str), Some("three"));
    }

    #[test]
    fn a_repeated_key_keeps_the_last_assignment() {
        let parsed = parse_env_map("A=first\nA=second\n");
        assert_eq!(parsed.get("A").map(String::as_str), Some("second"));
    }

    #[test]
    fn strips_an_export_prefix_and_ignores_a_trailing_fragment() {
        let parsed = parse_env_map("export A=1\nnot-an-assignment");
        assert_eq!(parsed.get("A").map(String::as_str), Some("1"));
        assert_eq!(parsed.len(), 1);
    }
}
