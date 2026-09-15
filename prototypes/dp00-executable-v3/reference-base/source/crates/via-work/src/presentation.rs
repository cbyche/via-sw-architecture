//! `resultMetadata` — the only part of a runner's metadata that escapes the
//! process.
//!
//! `publicResultMetadata`, `server/src/task/task-manager.mjs:31-51`. A runner
//! returns whatever metadata it likes; upstream keeps that object internally
//! and publishes exactly one projection of it. The catalogue calls the
//! difference an *information-disclosure boundary*
//! (`json-field` / *task resultMetadata projection*):
//!
//! > `backendRef` carries a filesystem path (e.g. `/private/project`) that must
//! > not reach clients — this is an information-disclosure boundary,
//! > deep-equality asserted.
//!
//! Two properties follow, and both are load-bearing:
//!
//! 1. **The projection is what is persisted.** `persistedTask` is built from
//!    `publicTask` (`task-manager.mjs:234`), so `backendRef` never reaches
//!    `tasks.json` either — not just the wire.
//! 2. **The legacy `decision.presentation` shape is projected forward.**
//!    Already-persisted `tasks.json` files hold it, so the fallback survives
//!    the port and re-persists in the new shape.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::text::slice_units;

/// Bound on `presentation.inline.title`.
///
/// Contract — `server/src/task/task-manager.mjs:40`.
pub const INLINE_TITLE_BOUND: usize = 120;

/// How an inline block should be rendered.
///
/// Contract — `server/src/task/task-manager.mjs:42`: the accepted values are
/// `markdown`, `code` and `link`, and **anything else becomes `markdown`**
/// rather than being rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InlineFormat {
    /// `markdown` — also the fallback for an unrecognised value.
    #[default]
    Markdown,
    /// `code`.
    Code,
    /// `link`.
    Link,
}

impl InlineFormat {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Code => "code",
            Self::Link => "link",
        }
    }

    /// `['markdown','code','link'].includes(format) ? format : 'markdown'`.
    #[must_use]
    pub fn from_wire_or_default(value: Option<&str>) -> Self {
        match value {
            Some("code") => Self::Code,
            Some("link") => Self::Link,
            _ => Self::Markdown,
        }
    }
}

/// An inline presentation block: something to render beside the conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineBlock {
    /// Bounded to [`INLINE_TITLE_BOUND`]; `""` when the source carried no
    /// string title.
    pub title: String,
    /// `markdown` unless the source named one of the other two.
    pub format: InlineFormat,
    /// The block's body, **unbounded** — upstream copies `source.inline.content`
    /// with no slice, because truncating a code block silently would be worse
    /// than a large payload.
    pub content: String,
}

/// What the user is told, and what they are shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presentation {
    /// The spoken form. `""` when the source carried no string speech.
    pub speech: String,
    /// The inline block, or `null`.
    pub inline: Option<InlineBlock>,
}

/// The published projection of a runner's metadata.
///
/// Exactly one key, `presentation`. `docs/reference/contracts.json`
/// (`json-field` / *resultMetadata (publicResultMetadata)*).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicResultMetadata {
    /// The presentation, which is the only thing that escapes.
    pub presentation: Presentation,
}

/// Project a runner's raw metadata into what a client may see.
///
/// Reproduces `publicResultMetadata` decision for decision:
///
/// - the source is `metadata.presentation`, **or** the legacy
///   `metadata.decision.presentation`;
/// - an inline block survives only when `inline.content` is a string with
///   non-whitespace in it — anything else becomes `null`;
/// - `title` is sliced to [`INLINE_TITLE_BOUND`], and a non-string title
///   becomes `""`;
/// - `speech` is the source's speech when it is a string, `""` otherwise;
/// - and if there is neither speech nor an inline block, the whole projection
///   is `None`. Note `!speech` in JavaScript: an *empty* speech with no inline
///   block publishes nothing.
#[must_use]
pub fn project(metadata: Option<&Value>) -> Option<PublicResultMetadata> {
    let metadata = metadata?.as_object()?;
    let source = metadata
        .get("presentation")
        .or_else(|| metadata.get("decision")?.get("presentation"))?
        .as_object()?;

    let inline = source
        .get("inline")
        .and_then(Value::as_object)
        .and_then(|inline| {
            let content = inline.get("content")?.as_str()?;
            if content.trim().is_empty() {
                return None;
            }
            Some(InlineBlock {
                title: inline
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| slice_units(title, INLINE_TITLE_BOUND))
                    .unwrap_or_default(),
                format: InlineFormat::from_wire_or_default(
                    inline.get("format").and_then(Value::as_str),
                ),
                content: content.to_owned(),
            })
        });

    let speech = source
        .get("speech")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();

    if speech.is_empty() && inline.is_none() {
        return None;
    }
    Some(PublicResultMetadata {
        presentation: Presentation { speech, inline },
    })
}

#[cfg(test)]
mod tests {
    use super::{INLINE_TITLE_BOUND, InlineFormat, project};
    use serde_json::json;

    #[test]
    fn a_backend_reference_never_survives_the_projection() {
        let projected = project(Some(&json!({
            "presentation": { "speech": "done", "inline": null },
            "backendRef": { "sessionId": "s", "directory": "/private/project" },
            "delegation": { "id": "d", "sessionId": "t" },
        })))
        .expect("speech is present");
        let rendered = serde_json::to_value(&projected).expect("serializes");
        assert_eq!(
            rendered,
            json!({"presentation": {"speech": "done", "inline": null}})
        );
    }

    #[test]
    fn the_legacy_decision_shape_is_projected_forward() {
        let projected = project(Some(&json!({
            "decision": { "presentation": { "speech": "old", "inline": null } },
            "backendRef": { "directory": "/private/legacy" },
        })))
        .expect("legacy speech is present");
        assert_eq!(projected.presentation.speech, "old");
    }

    #[test]
    fn presentation_wins_over_the_legacy_shape() {
        let projected = project(Some(&json!({
            "presentation": { "speech": "new", "inline": null },
            "decision": { "presentation": { "speech": "old", "inline": null } },
        })))
        .expect("present");
        assert_eq!(projected.presentation.speech, "new");
    }

    #[test]
    fn an_unrecognised_format_becomes_markdown() {
        let projected = project(Some(&json!({
            "presentation": {
                "speech": "",
                "inline": { "title": "t", "format": "html", "content": "x" },
            },
        })))
        .expect("inline is present");
        let inline = projected.presentation.inline.expect("inline");
        assert_eq!(inline.format, InlineFormat::Markdown);
    }

    #[test]
    fn a_blank_inline_body_is_not_an_inline_block() {
        assert!(
            project(Some(&json!({
                "presentation": { "speech": "", "inline": { "content": "   " } },
            })))
            .is_none(),
            "no speech and a whitespace-only body publishes nothing",
        );
    }

    #[test]
    fn a_non_string_title_becomes_empty_and_a_long_one_is_sliced() {
        let long = "t".repeat(INLINE_TITLE_BOUND + 40);
        let projected = project(Some(&json!({
            "presentation": {
                "speech": "s",
                "inline": { "title": long, "format": "code", "content": "x" },
            },
        })))
        .expect("present");
        let inline = projected.presentation.inline.expect("inline");
        assert_eq!(inline.title.chars().count(), INLINE_TITLE_BOUND);

        let projected = project(Some(&json!({
            "presentation": {
                "speech": "s",
                "inline": { "title": 7, "format": "code", "content": "x" },
            },
        })))
        .expect("present");
        assert_eq!(
            projected.presentation.inline.expect("inline").title,
            "",
            "a non-string title is not coerced, it is dropped",
        );
    }

    #[test]
    fn nothing_at_all_projects_to_nothing() {
        assert!(project(None).is_none());
        assert!(project(Some(&json!(null))).is_none());
        assert!(project(Some(&json!("a string"))).is_none());
        assert!(project(Some(&json!({}))).is_none());
        assert!(project(Some(&json!({"presentation": "not an object"}))).is_none());
        assert!(
            project(Some(
                &json!({"presentation": {"speech": "", "inline": null}})
            ))
            .is_none(),
            "empty speech with no inline block is upstream's falsy `!speech`",
        );
    }

    #[test]
    fn a_non_string_speech_becomes_empty() {
        let projected = project(Some(&json!({
            "presentation": { "speech": 42, "inline": { "content": "x" } },
        })))
        .expect("the inline block carries it");
        assert_eq!(projected.presentation.speech, "");
    }

    #[test]
    fn the_projection_is_idempotent() {
        let once = project(Some(&json!({
            "presentation": {
                "speech": "s",
                "inline": { "title": "t", "format": "link", "content": "c" },
            },
        })))
        .expect("present");
        let rendered = serde_json::to_value(&once).expect("serializes");
        let twice = project(Some(&rendered)).expect("present");
        assert_eq!(once, twice, "re-projecting a restored record is a no-op");
    }
}
