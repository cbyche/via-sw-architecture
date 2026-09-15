//! Turning VIA input parts into ACP `ContentBlock`s, and back.
//!
//! Ported from `server/src/agent/acp-content.mjs`, with the `parseDataUrl` /
//! `inputFileParts` half of `shared/input-parts.mjs` that it reads through.
//!
//! # The attachment URI scheme is externally visible
//!
//! Every attachment reaches the backend model carrying
//! `via://input/<encoded filename>` as its `uri`. That is catalogued twice
//! (*"attachment resource URI scheme"* and *"ACP attachment URI scheme"*) and
//! renamed from upstream's `qwen-audio-agent://input/` by `docs/rebrand.md`.
//! Audio blocks deliberately carry **no** `uri` — upstream's `audio` branch
//! omits it, and an ACP `AudioContent` has no `uri` field to put it in.
//!
//! # Five block kinds, and what decides each
//!
//! | Input part | Block |
//! | --- | --- |
//! | a non-`data:` URL | `resource_link` — the agent fetches it itself |
//! | `image/*` data URL | `image`, with `uri` |
//! | `audio/*` data URL | `audio`, no `uri` |
//! | a text-ish MIME data URL | `resource` with `text`, decoded from base64 |
//! | anything else | `resource` with `blob`, base64 left as-is |
//!
//! The text-ish set is upstream's exact list, not a heuristic: `text/*` plus
//! five named `application/*` types.
//!
//! # Scope
//!
//! `shared/input-parts.mjs` is a larger surface — normalization, size caps,
//! protocol allow-listing, the `via/inputRef` meta key — belonging to the crate
//! that owns the client input vocabulary. Only the two functions
//! `acp-content.mjs` actually imports are here, over
//! [`via_downstream::PromptAttachment`], which is already the
//! `{url, filename, mime}` triple a turn carries across the Layer-3 seam.

use agent_client_protocol::schema::v1::{
    AudioContent, BlobResourceContents, ContentBlock, EmbeddedResource, EmbeddedResourceResource,
    ImageContent, PromptCapabilities, ResourceLink, TextContent, TextResourceContents,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use via_downstream::PromptAttachment;
use via_downstream::text::encode_uri_component;

use crate::error::{AcpError, Result};

/// The URI scheme every prompt attachment is announced under.
///
/// **External contract** — `acp-content.mjs:8`, renamed per `docs/rebrand.md`
/// (`qwen-audio-agent://input/` → `via://input/`). Asserted upstream in
/// `server/test/acp-content.test.mjs:22`.
pub const ATTACHMENT_URI_PREFIX: &str = "via://input/";

/// MIME types upstream inlines as decoded text rather than as a base64 blob.
///
/// **External contract** — `acp-content.mjs:11-19`. `text/*` matches by prefix;
/// these five match exactly. The list is closed on purpose: a type that is not
/// on it round-trips as a blob, which is lossless, whereas guessing wrong makes
/// the agent read mojibake.
pub const TEXT_MIME_TYPES: &[&str] = &[
    "application/json",
    "application/javascript",
    "application/xml",
    "application/yaml",
    "application/x-yaml",
];

/// The parts of a base64 `data:` URL.
///
/// Ported from `shared/input-parts.mjs:26-34`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataUrl {
    /// The MIME type, lowercased.
    pub mime_type: String,
    /// The `charset=` parameter, or empty.
    pub charset: String,
    /// The base64 payload with all whitespace removed.
    pub data: String,
}

/// Parse a base64 `data:` URL, or `None` if it is not one.
///
/// **Contract** — `shared/input-parts.mjs:26-34`. The regex is
/// `^data:([^;,]+)(?:;charset=([^;,]+))?;base64,([a-z\d+/=\s]+)$`, case
/// insensitive, and only `;base64,` form is accepted: a percent-encoded
/// `data:` URL returns `None` and therefore becomes a `resource_link` rather
/// than being silently mis-decoded.
#[must_use]
pub fn parse_data_url(value: &str) -> Option<DataUrl> {
    let rest = strip_prefix_ignore_ascii_case(value, "data:")?;
    let (mime_type, rest) = rest.split_once(';')?;
    if mime_type.is_empty() || mime_type.contains(',') {
        return None;
    }
    let (charset, rest) = match strip_prefix_ignore_ascii_case(rest, "charset=") {
        Some(after) => {
            let (charset, rest) = after.split_once(';')?;
            if charset.is_empty() || charset.contains(',') {
                return None;
            }
            (charset, rest)
        }
        None => ("", rest),
    };
    let payload = strip_prefix_ignore_ascii_case(rest, "base64,")?;
    if payload.is_empty() {
        return None;
    }
    if !payload.chars().all(|c| {
        c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=') || c.is_ascii_whitespace()
    }) {
        return None;
    }
    Some(DataUrl {
        mime_type: mime_type.to_ascii_lowercase(),
        charset: charset.to_owned(),
        data: payload.chars().filter(|c| !c.is_whitespace()).collect(),
    })
}

fn strip_prefix_ignore_ascii_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let head = value.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &value[prefix.len()..])
}

/// `qwen-audio-agent://input/${encodeURIComponent(filename)}`, rebranded.
///
/// The filename defaults to `attachment-<index + 1>` — one-based, because it
/// is shown to a model and read by a person.
fn resource_uri(filename: Option<&str>, index: usize) -> String {
    let name = filename
        .filter(|name| !name.is_empty())
        .map_or_else(|| default_filename(index), str::to_owned);
    std::format!("{ATTACHMENT_URI_PREFIX}{}", encode_uri_component(&name))
}

fn default_filename(index: usize) -> String {
    std::format!("attachment-{}", index + 1)
}

/// Whether a MIME type is inlined as decoded text.
///
/// **Contract** — `acp-content.mjs:11-19`.
#[must_use]
pub fn is_text_mime(mime: &str) -> bool {
    mime.starts_with("text/") || TEXT_MIME_TYPES.contains(&mime)
}

/// Convert every attachment into an ACP `ContentBlock`.
///
/// **Contract** — `inputPartsToAcpBlocks`, `acp-content.mjs:25-66`. Upstream
/// calls `inputFileParts(parts)` first, so the index that names an unnamed
/// attachment counts **file parts only**; a [`PromptAttachment`] is already a
/// file part, so the enumeration is over the whole slice.
#[must_use]
pub fn input_parts_to_acp_blocks(parts: &[PromptAttachment]) -> Vec<ContentBlock> {
    parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            file_part_to_block(&part.mime, Some(part.filename.as_str()), &part.url, index)
        })
        .collect()
}

fn file_part_to_block(mime: &str, filename: Option<&str>, url: &str, index: usize) -> ContentBlock {
    let Some(embedded) = parse_data_url(url) else {
        // Not inlined: hand the agent the URL and let it fetch.
        return ContentBlock::ResourceLink(
            ResourceLink::new(
                filename
                    .filter(|name| !name.is_empty())
                    .map_or_else(|| default_filename(index), str::to_owned),
                url.to_owned(),
            )
            .mime_type(mime.to_owned()),
        );
    };

    if mime.starts_with("image/") {
        return ContentBlock::Image(
            ImageContent::new(embedded.data, mime.to_owned()).uri(resource_uri(filename, index)),
        );
    }
    if mime.starts_with("audio/") {
        // No `uri` — upstream's audio branch omits it, and `AudioContent` has
        // no field for one.
        return ContentBlock::Audio(AudioContent::new(embedded.data, mime.to_owned()));
    }

    let uri = resource_uri(filename, index);
    let resource = if is_text_mime(mime) {
        EmbeddedResourceResource::TextResourceContents(
            TextResourceContents::new(decode_utf8_lossy(&embedded.data), uri)
                .mime_type(mime.to_owned()),
        )
    } else {
        EmbeddedResourceResource::BlobResourceContents(
            BlobResourceContents::new(embedded.data, uri).mime_type(mime.to_owned()),
        )
    };
    ContentBlock::Resource(EmbeddedResource::new(resource))
}

/// `Buffer.from(data, 'base64').toString('utf8')`.
///
/// Node is forgiving twice over: it ignores characters outside the base64
/// alphabet, and `toString('utf8')` replaces invalid sequences rather than
/// throwing. Both are reproduced — a malformed attachment must not fail the
/// whole turn, because the turn is a person talking.
fn decode_utf8_lossy(data: &str) -> String {
    BASE64
        .decode(data)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

/// The prompt a turn sends: bare text, or text plus attachments.
///
/// **Contract** — `promptWithInputParts`, `acp-content.mjs:68-75`. With no
/// attachments upstream returns the **string**, not a one-element array, and
/// `normalizeAcpPrompt` then wraps it. [`Prompt`] models that distinction
/// because it is observable: an empty array throws, an empty string does not.
#[derive(Debug, Clone, PartialEq)]
pub enum Prompt {
    /// A bare string, wrapped into a single text block on the way out.
    Text(String),
    /// An explicit block list.
    Blocks(Vec<ContentBlock>),
}

impl Prompt {
    /// A bare-text prompt.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }
}

impl From<&str> for Prompt {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<String> for Prompt {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<ContentBlock>> for Prompt {
    fn from(value: Vec<ContentBlock>) -> Self {
        Self::Blocks(value)
    }
}

/// Compose a prompt from text plus input parts.
///
/// **Contract** — `acp-content.mjs:68-75`: with no file parts the result is the
/// text alone; otherwise the text becomes block 0 and the attachments follow in
/// order, **including when the text is empty**.
#[must_use]
pub fn prompt_with_input_parts(text: &str, parts: &[PromptAttachment]) -> Prompt {
    let attachments = input_parts_to_acp_blocks(parts);
    if attachments.is_empty() {
        return Prompt::Text(text.to_owned());
    }
    let mut blocks = Vec::with_capacity(attachments.len() + 1);
    blocks.push(ContentBlock::Text(TextContent::new(text.to_owned())));
    blocks.extend(attachments);
    Prompt::Blocks(blocks)
}

/// Normalize a prompt into the `ContentBlock[]` that goes on the wire.
///
/// **Contract** — `normalizeAcpPrompt`, `acp-content.mjs:77-88`:
///
/// * a non-array becomes `[{type: 'text', text: String(prompt || '')}]`,
/// * an **empty** array throws [`AcpError::PromptEmpty`],
/// * a block that is not an object with a `type` throws
///   [`AcpError::PromptInvalidBlock`].
///
/// The third case cannot arise from a typed [`ContentBlock`], which is the
/// point of the type; [`normalize_acp_prompt_value`] is the entry point for
/// blocks that arrived as untyped JSON and still needs it.
///
/// # Errors
///
/// [`AcpError::PromptEmpty`] for an explicitly empty block list.
pub fn normalize_acp_prompt(prompt: &Prompt) -> Result<Vec<ContentBlock>> {
    match prompt {
        Prompt::Text(text) => Ok(vec![ContentBlock::Text(TextContent::new(text.clone()))]),
        Prompt::Blocks(blocks) if blocks.is_empty() => Err(AcpError::PromptEmpty),
        Prompt::Blocks(blocks) => Ok(blocks.clone()),
    }
}

/// [`normalize_acp_prompt`] for blocks that arrived as untyped JSON.
///
/// This is where upstream's "包含无效的 ContentBlock" case actually lives in a
/// typed port: a JSON value that is not an object, or has no `type`, or has a
/// `type` the protocol does not define.
///
/// # Errors
///
/// [`AcpError::PromptEmpty`] for an empty array, [`AcpError::PromptInvalidBlock`]
/// for a member that is not a well-formed content block.
pub fn normalize_acp_prompt_value(prompt: &serde_json::Value) -> Result<Vec<ContentBlock>> {
    let Some(blocks) = prompt.as_array() else {
        let text = match prompt {
            serde_json::Value::String(text) => text.clone(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };
        return Ok(vec![ContentBlock::Text(TextContent::new(text))]);
    };
    if blocks.is_empty() {
        return Err(AcpError::PromptEmpty);
    }
    blocks
        .iter()
        .map(|block| {
            if !block.is_object() || !block.get("type").is_some_and(serde_json::Value::is_string) {
                return Err(AcpError::PromptInvalidBlock);
            }
            serde_json::from_value::<ContentBlock>(block.clone())
                .map_err(|_| AcpError::PromptInvalidBlock)
        })
        .collect()
}

/// Refuse a prompt the agent has not declared it can read.
///
/// **Contract** — `assertPromptCapabilities`, `acp-content.mjs:90-103`. The
/// check is `!== true`, so an agent that omits `promptCapabilities` entirely
/// gets text only. Text and `resource_link` blocks are always allowed:
/// upstream checks exactly three kinds and a resource *link* is a URL, not
/// embedded content.
///
/// # Errors
///
/// [`AcpError::ImageInputUndeclared`], [`AcpError::AudioInputUndeclared`] or
/// [`AcpError::EmbeddedFileUndeclared`], for the first offending block.
pub fn assert_prompt_capabilities(
    blocks: &[ContentBlock],
    capabilities: &PromptCapabilities,
) -> Result<()> {
    for block in blocks {
        match block {
            ContentBlock::Image(_) if !capabilities.image => {
                return Err(AcpError::ImageInputUndeclared);
            }
            ContentBlock::Audio(_) if !capabilities.audio => {
                return Err(AcpError::AudioInputUndeclared);
            }
            ContentBlock::Resource(_) if !capabilities.embedded_context => {
                return Err(AcpError::EmbeddedFileUndeclared);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Rewrite the **first** text block, preserving every other block.
///
/// **Contract** — `transformPromptText`, `acp-content.mjs:105-117`. Three
/// behaviours that all matter:
///
/// * only the first text block is transformed,
/// * a bare-string prompt is transformed and stays a bare string,
/// * a prompt with **no** text block gains one at the front, built by calling
///   the transform with the empty string — which is how a wrapper reaches a
///   prompt that is nothing but attachments.
#[must_use]
pub fn transform_prompt_text<F>(prompt: &Prompt, transform: F) -> Prompt
where
    F: Fn(&str) -> String,
{
    let blocks = match prompt {
        Prompt::Text(text) => return Prompt::Text(transform(text)),
        Prompt::Blocks(blocks) => blocks,
    };

    let mut transformed = false;
    let mut out: Vec<ContentBlock> = Vec::with_capacity(blocks.len() + 1);
    for block in blocks {
        match block {
            ContentBlock::Text(text) if !transformed => {
                transformed = true;
                let mut replaced = text.clone();
                replaced.text = transform(&text.text);
                out.push(ContentBlock::Text(replaced));
            }
            other => out.push(other.clone()),
        }
    }
    if !transformed {
        out.insert(0, ContentBlock::Text(TextContent::new(transform(""))));
    }
    Prompt::Blocks(out)
}

/// Every block that is not text.
///
/// **Contract** — `nonTextPromptBlocks`, `acp-content.mjs:119-123`. A bare
/// string has none.
#[must_use]
pub fn non_text_prompt_blocks(prompt: &Prompt) -> Vec<ContentBlock> {
    match prompt {
        Prompt::Text(_) => Vec::new(),
        Prompt::Blocks(blocks) => blocks
            .iter()
            .filter(|block| !matches!(block, ContentBlock::Text(_)))
            .cloned()
            .collect(),
    }
}

/// Append blocks to a prompt, normalizing it first.
///
/// **Contract** — `appendPromptBlocks`, `acp-content.mjs:125-129`. Appending
/// **nothing** returns the prompt untouched — importantly *without*
/// normalizing, so appending an empty list to an empty block list does not
/// raise [`AcpError::PromptEmpty`].
///
/// # Errors
///
/// Whatever [`normalize_acp_prompt`] raises, and only when `blocks` is
/// non-empty.
pub fn append_prompt_blocks(prompt: &Prompt, blocks: &[ContentBlock]) -> Result<Prompt> {
    if blocks.is_empty() {
        return Ok(prompt.clone());
    }
    let mut normalized = normalize_acp_prompt(prompt)?;
    normalized.extend_from_slice(blocks);
    Ok(Prompt::Blocks(normalized))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attachment(mime: &str, filename: &str, url: &str) -> PromptAttachment {
        PromptAttachment {
            url: url.to_owned(),
            filename: filename.to_owned(),
            mime: mime.to_owned(),
        }
    }

    fn image_part() -> PromptAttachment {
        attachment(
            "image/png",
            "reference.png",
            "data:image/png;base64,aGVsbG8=",
        )
    }

    #[test]
    fn maps_image_parts_to_image_blocks() {
        // Upstream: server/test/acp-content.test.mjs:17-26.
        let blocks = input_parts_to_acp_blocks(&[image_part()]);
        let ContentBlock::Image(image) = &blocks[0] else {
            panic!("expected an image block, got {:?}", blocks[0]);
        };
        assert_eq!(image.mime_type, "image/png");
        assert_eq!(image.data, "aGVsbG8=");
        assert_eq!(image.uri.as_deref(), Some("via://input/reference.png"));
    }

    #[test]
    fn maps_inline_text_files_to_embedded_resources() {
        // Upstream: server/test/acp-content.test.mjs:28-37.
        let blocks = input_parts_to_acp_blocks(&[attachment(
            "text/markdown",
            "SKILL.md",
            "data:text/markdown;base64,IyBTa2lsbA==",
        )]);
        let ContentBlock::Resource(resource) = &blocks[0] else {
            panic!("expected a resource block");
        };
        let EmbeddedResourceResource::TextResourceContents(text) = &resource.resource else {
            panic!("expected inline text");
        };
        assert_eq!(text.text, "# Skill");
        assert_eq!(text.uri, "via://input/SKILL.md");
    }

    #[test]
    fn a_non_data_url_becomes_a_resource_link() {
        let blocks = input_parts_to_acp_blocks(&[attachment(
            "application/pdf",
            "report.pdf",
            "https://example.test/report.pdf",
        )]);
        let ContentBlock::ResourceLink(link) = &blocks[0] else {
            panic!("expected a resource link");
        };
        assert_eq!(link.uri, "https://example.test/report.pdf");
        assert_eq!(link.name, "report.pdf");
        assert_eq!(link.mime_type.as_deref(), Some("application/pdf"));
    }

    #[test]
    fn audio_blocks_carry_no_uri() {
        let blocks = input_parts_to_acp_blocks(&[attachment(
            "audio/wav",
            "clip.wav",
            "data:audio/wav;base64,AAAA",
        )]);
        assert!(matches!(blocks[0], ContentBlock::Audio(_)));
        let json = serde_json::to_value(&blocks[0]).expect("audio block serializes");
        assert!(json.get("uri").is_none(), "audio must not carry a uri");
    }

    #[test]
    fn an_unnamed_attachment_is_numbered_from_one_among_file_parts() {
        let blocks = input_parts_to_acp_blocks(&[
            attachment("image/png", "", "data:image/png;base64,AA=="),
            attachment("image/png", "", "data:image/png;base64,AA=="),
        ]);
        let uris: Vec<_> = blocks
            .iter()
            .map(|block| match block {
                ContentBlock::Image(image) => image.uri.clone().unwrap_or_default(),
                _ => String::new(),
            })
            .collect();
        assert_eq!(
            uris,
            ["via://input/attachment-1", "via://input/attachment-2"]
        );
    }

    #[test]
    fn filenames_reach_the_uri_percent_encoded() {
        // The encoder is `via-downstream`'s; what is asserted here is that the
        // attachment URI actually goes through it, because a raw `/` or `#` in
        // a filename would otherwise forge a path or a fragment.
        let blocks = input_parts_to_acp_blocks(&[attachment(
            "image/png",
            "a b/c#d.png",
            "data:image/png;base64,AA==",
        )]);
        let ContentBlock::Image(image) = &blocks[0] else {
            panic!("expected an image block");
        };
        assert_eq!(image.uri.as_deref(), Some("via://input/a%20b%2Fc%23d.png"));
    }

    #[test]
    fn data_urls_parse_only_in_base64_form() {
        let parsed = parse_data_url("data:text/plain;charset=utf-8;base64,aGk=")
            .expect("charset form parses");
        assert_eq!(parsed.mime_type, "text/plain");
        assert_eq!(parsed.charset, "utf-8");
        assert_eq!(parsed.data, "aGk=");
        assert_eq!(
            parse_data_url("data:text/plain,hello"),
            None,
            "percent-encoded data URLs are not inlined"
        );
        assert_eq!(parse_data_url("https://example.test/a.png"), None);
        assert_eq!(parse_data_url("data:text/plain;base64,"), None);
        assert_eq!(
            parse_data_url("DATA:TEXT/PLAIN;BASE64,aGk=")
                .expect("the regex is case-insensitive")
                .mime_type,
            "text/plain",
            "the mime type is lowercased"
        );
        assert_eq!(
            parse_data_url("data:text/plain;base64,aG\n k=")
                .expect("whitespace is allowed inside the payload")
                .data,
            "aGk="
        );
        assert_eq!(
            parse_data_url("data:text/plain;base64,héllo"),
            None,
            "a payload outside the base64 alphabet is not a data URL"
        );
    }

    #[test]
    fn the_text_mime_set_is_closed() {
        assert!(is_text_mime("text/anything"));
        for mime in TEXT_MIME_TYPES {
            assert!(is_text_mime(mime), "{mime}");
        }
        assert!(!is_text_mime("application/toml"));
        assert!(!is_text_mime("application/json5"));
    }

    #[test]
    fn capability_checks_reject_exactly_what_upstream_rejects() {
        // Upstream: server/test/acp-content.test.mjs:39-48.
        let image = input_parts_to_acp_blocks(&[image_part()]);
        assert_eq!(
            assert_prompt_capabilities(&image, &PromptCapabilities::default()),
            Err(AcpError::ImageInputUndeclared)
        );
        let mut declared = PromptCapabilities::default();
        declared.image = true;
        assert_eq!(assert_prompt_capabilities(&image, &declared), Ok(()));

        let audio =
            input_parts_to_acp_blocks(&[attachment("audio/wav", "", "data:audio/wav;base64,AA==")]);
        assert_eq!(
            assert_prompt_capabilities(&audio, &declared),
            Err(AcpError::AudioInputUndeclared)
        );

        let resource = input_parts_to_acp_blocks(&[attachment(
            "application/pdf",
            "",
            "data:application/pdf;base64,AA==",
        )]);
        assert_eq!(
            assert_prompt_capabilities(&resource, &declared),
            Err(AcpError::EmbeddedFileUndeclared)
        );

        let link = input_parts_to_acp_blocks(&[attachment(
            "application/pdf",
            "",
            "https://example.test/a.pdf",
        )]);
        assert_eq!(
            assert_prompt_capabilities(&link, &PromptCapabilities::default()),
            Ok(()),
            "a resource link is a URL, not embedded content"
        );
    }

    #[test]
    fn a_bare_string_prompt_stays_a_string_until_normalized() {
        // Upstream: `promptWithInputParts('hello', []).constructor === String`.
        assert_eq!(
            prompt_with_input_parts("hello", &[]),
            Prompt::Text("hello".into())
        );
        let Prompt::Blocks(blocks) = prompt_with_input_parts("hello", &[image_part()]) else {
            panic!("attachments force the array form");
        };
        let ContentBlock::Text(text) = &blocks[0] else {
            panic!("text leads");
        };
        assert_eq!(text.text, "hello");
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn an_empty_block_list_is_refused_but_an_empty_string_is_not() {
        assert_eq!(
            normalize_acp_prompt(&Prompt::Blocks(vec![])),
            Err(AcpError::PromptEmpty)
        );
        let blocks = normalize_acp_prompt(&Prompt::Text(String::new()))
            .expect("an empty string is a valid one-block prompt");
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn untyped_blocks_are_validated_the_way_javascript_validates_them() {
        assert_eq!(
            normalize_acp_prompt_value(&serde_json::json!([])),
            Err(AcpError::PromptEmpty)
        );
        for invalid in [
            serde_json::json!([null]),
            serde_json::json!(["text"]),
            serde_json::json!([{ "text": "no type" }]),
            serde_json::json!([{ "type": "nonesuch" }]),
        ] {
            assert_eq!(
                normalize_acp_prompt_value(&invalid),
                Err(AcpError::PromptInvalidBlock),
                "{invalid}"
            );
        }
        let wrapped = normalize_acp_prompt_value(&serde_json::json!("plain"))
            .expect("a non-array becomes one text block");
        assert!(matches!(&wrapped[0], ContentBlock::Text(t) if t.text == "plain"));
        let null = normalize_acp_prompt_value(&serde_json::Value::Null)
            .expect("String(null || '') is the empty string");
        assert!(matches!(&null[0], ContentBlock::Text(t) if t.text.is_empty()));
    }

    #[test]
    fn transform_touches_only_the_first_text_block() {
        // Upstream: server/test/acp-content.test.mjs:50-57.
        let prompt = Prompt::Blocks(vec![
            ContentBlock::Text(TextContent::new("request".to_owned())),
            ContentBlock::Image(ImageContent::new("x".to_owned(), "image/png".to_owned())),
            ContentBlock::Text(TextContent::new("second".to_owned())),
        ]);
        let Prompt::Blocks(out) = transform_prompt_text(&prompt, |text| format!("wrapped:{text}"))
        else {
            panic!("blocks stay blocks");
        };
        assert!(matches!(&out[0], ContentBlock::Text(t) if t.text == "wrapped:request"));
        assert!(matches!(out[1], ContentBlock::Image(_)));
        assert!(
            matches!(&out[2], ContentBlock::Text(t) if t.text == "second"),
            "only the first text block is transformed"
        );
    }

    #[test]
    fn transform_prepends_when_a_prompt_is_all_attachments() {
        let prompt = Prompt::Blocks(vec![ContentBlock::Image(ImageContent::new(
            "x".to_owned(),
            "image/png".to_owned(),
        ))]);
        let Prompt::Blocks(out) = transform_prompt_text(&prompt, |text| format!("[{text}]")) else {
            panic!("blocks stay blocks");
        };
        assert!(matches!(&out[0], ContentBlock::Text(t) if t.text == "[]"));
        assert!(matches!(out[1], ContentBlock::Image(_)));
    }

    #[test]
    fn transform_of_a_bare_string_stays_a_bare_string() {
        assert_eq!(
            transform_prompt_text(&Prompt::text("hi"), |t| format!("<{t}>")),
            Prompt::Text("<hi>".into())
        );
    }

    #[test]
    fn appending_nothing_never_raises() {
        let empty = Prompt::Blocks(vec![]);
        assert_eq!(
            append_prompt_blocks(&empty, &[]),
            Ok(empty.clone()),
            "the empty-block guard runs before normalization"
        );
        assert_eq!(
            append_prompt_blocks(
                &empty,
                &[ContentBlock::Text(TextContent::new("x".to_owned()))]
            ),
            Err(AcpError::PromptEmpty)
        );
    }

    #[test]
    fn non_text_blocks_of_a_string_prompt_are_none() {
        assert!(non_text_prompt_blocks(&Prompt::text("hi")).is_empty());
        let prompt = Prompt::Blocks(vec![
            ContentBlock::Text(TextContent::new("t".to_owned())),
            ContentBlock::Image(ImageContent::new("x".to_owned(), "image/png".to_owned())),
        ]);
        assert_eq!(non_text_prompt_blocks(&prompt).len(), 1);
    }
}
