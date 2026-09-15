//! Input parts — the text and attachments one user turn carries.
//!
//! Ported from `shared/input-parts.mjs`.
//!
//! # The one security decision in this file
//!
//! [`ALLOWED_URL_PROTOCOLS`] is `data:`, `http:` and `https:`. A trusted client
//! such as the TUI resolves a local path and inlines the bytes itself; letting
//! a remote client submit `file:` directly would turn the Gateway into an
//! arbitrary local-file reference bridge for anyone who can reach the socket.
//! The check is on the parsed scheme, not on a prefix match, so
//! `FILE:` and `file:/x` are both rejected.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use via_i18n::{Locale, format as i18n_format, keys, t};

use crate::text::trim;

/// How many parts one submission may carry.
///
/// **External contract** — `input-parts.mjs:1` (`MAX_INPUT_PARTS = 16`).
pub const MAX_INPUT_PARTS: usize = 16;

/// The largest single attachment, in bytes.
///
/// **External contract** — `input-parts.mjs:2` (8 MiB).
pub const MAX_INPUT_FILE_BYTES: usize = 8 * 1024 * 1024;

/// The largest total attachment payload for one turn, in bytes.
///
/// **External contract** — `input-parts.mjs:3` (12 MiB). Lower than
/// `16 × MAX_INPUT_FILE_BYTES` on purpose: sixteen maximal attachments is
/// 128 MiB of base64 in one frame.
pub const MAX_INPUT_TOTAL_FILE_BYTES: usize = 12 * 1024 * 1024;

/// The `_meta` key an input reference is written under.
///
/// **External contract** — `input-parts.mjs:4`, rebranded per
/// `docs/rebrand.md` (`qwen-audio-agent/inputRef` → `via/inputRef`). It
/// crosses into ACP `ContentBlock._meta` and is read back by the voice layer.
pub const INPUT_REF_META_KEY: &str = "via/inputRef";

/// The three URL schemes an attachment may use.
///
/// **External contract** — `input-parts.mjs:9`.
pub const ALLOWED_URL_PROTOCOLS: [&str; 3] = ["data", "http", "https"];

/// The bound on a free-text field before any other check.
///
/// **External contract** — `input-parts.mjs:11` (`cleanText`'s default).
pub const MAX_TEXT_CHARS: usize = 100_000;

/// The bound on a filename.
///
/// **External contract** — `input-parts.mjs:16`.
pub const MAX_FILENAME_CHARS: usize = 240;

/// The bound on a MIME type.
///
/// **External contract** — `input-parts.mjs:59`.
pub const MAX_MIME_CHARS: usize = 160;

/// The bound on a content URL.
///
/// **External contract** — `input-parts.mjs:61` (20 MiB of characters, which
/// is the base64 envelope for an 8 MiB attachment plus slack).
pub const MAX_URL_CHARS: usize = 20 * 1024 * 1024;

/// The bound on a source path or a source text value.
///
/// **External contract** — `input-parts.mjs:45,50`.
pub const MAX_SOURCE_CHARS: usize = 2048;

/// The bound on a rendered attachment label.
///
/// **External contract** — `input-parts.mjs:181` (`inputPartLabel`).
pub const MAX_LABEL_CHARS: usize = 120;

/// Where an attachment came from.
///
/// **External contract** — `input-parts.mjs:40`; an unrecognized value
/// normalizes to `file` rather than being rejected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    /// Pasted.
    Clipboard,
    /// Read from disk by a trusted client.
    #[default]
    File,
    /// Named by a URI.
    Resource,
}

impl SourceKind {
    /// The wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clipboard => "clipboard",
            Self::File => "file",
            Self::Resource => "resource",
        }
    }

    /// Parse a wire spelling, falling back to [`File`](Self::File).
    #[must_use]
    pub fn from_wire_or_file(value: &str) -> Self {
        match value {
            "clipboard" => Self::Clipboard,
            "resource" => Self::Resource,
            _ => Self::File,
        }
    }
}

/// The textual anchor bound to an attachment.
///
/// The `start` / `end` offsets are OpenCode-compatible: they index into the
/// turn's text so a client can highlight which words the file is attached to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceText {
    /// The anchor text itself.
    pub value: String,
    /// The anchor's start offset in the turn text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    /// The anchor's end offset in the turn text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

/// Where an attachment came from, in full.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartSource {
    /// Clipboard, file or resource.
    #[serde(rename = "type")]
    pub kind: SourceKind,
    /// The originating path, when the client knows one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The textual anchor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<SourceText>,
}

/// One part of a user turn.
///
/// The two variants differ in size by about a hundred bytes, which is the
/// shape upstream's `{type, …}` object has and which every caller destructures
/// directly. Boxing the larger one would put an indirection in front of the
/// single hottest read in the crate — `part.mime()` and `part.url()` on every
/// attachment of every turn — to save a stack copy that never happens in a
/// loop.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InputPart {
    /// A text fragment.
    Text {
        /// The text.
        text: String,
    },
    /// An attachment.
    File {
        /// The MIME type, lowercased.
        mime: String,
        /// The original filename, when the client supplied one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        /// The content URL.
        url: String,
        /// Where it came from.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        source: Option<PartSource>,
        /// The `_meta` bag, holding [`INPUT_REF_META_KEY`].
        #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
        meta: Option<Map<String, Value>>,
    },
}

impl InputPart {
    /// A text part.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// A file part with no source and no meta.
    #[must_use]
    pub fn file(mime: impl Into<String>, url: impl Into<String>) -> Self {
        Self::File {
            mime: mime.into(),
            filename: None,
            url: url.into(),
            source: None,
            meta: None,
        }
    }

    /// Whether this is a file part.
    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    /// The MIME type, for a file part.
    #[must_use]
    pub fn mime(&self) -> &str {
        match self {
            Self::File { mime, .. } => mime,
            Self::Text { .. } => "",
        }
    }

    /// The content URL, for a file part.
    #[must_use]
    pub fn url(&self) -> &str {
        match self {
            Self::File { url, .. } => url,
            Self::Text { .. } => "",
        }
    }

    /// The filename, for a file part.
    #[must_use]
    pub fn filename(&self) -> Option<&str> {
        match self {
            Self::File { filename, .. } => filename.as_deref(),
            Self::Text { .. } => None,
        }
    }

    /// The source, for a file part.
    #[must_use]
    pub const fn source(&self) -> Option<&PartSource> {
        match self {
            Self::File { source, .. } => source.as_ref(),
            Self::Text { .. } => None,
        }
    }

    /// The `input_N` reference bound to this part, if any.
    ///
    /// **External contract** — `input-parts.mjs:132-134`.
    #[must_use]
    pub fn reference(&self) -> &str {
        let Self::File {
            meta: Some(meta), ..
        } = self
        else {
            return "";
        };
        meta.get(INPUT_REF_META_KEY)
            .and_then(Value::as_str)
            .map_or("", trim)
    }

    /// Bind `reference` to this part.
    ///
    /// **External contract** — `input-parts.mjs:136-146`. An empty reference
    /// returns the part unchanged rather than writing an empty key.
    #[must_use]
    pub fn with_reference(mut self, reference: &str) -> Self {
        let value = trim(reference);
        if value.is_empty() {
            return self;
        }
        if let Self::File { meta, .. } = &mut self {
            meta.get_or_insert_with(Map::new).insert(
                INPUT_REF_META_KEY.to_owned(),
                Value::String(value.to_owned()),
            );
        }
        self
    }
}

/// A rejected submission.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct InputError {
    /// The localized message, sent to the client as an `error` frame.
    pub message: String,
}

impl InputError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

/// A parsed `data:` URL.
///
/// **External contract** — `input-parts.mjs:26-36`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataUrl {
    /// The declared MIME type, lowercased.
    pub mime_type: String,
    /// The declared charset, or `""`.
    pub charset: String,
    /// The base64 payload with whitespace stripped.
    pub data: String,
    /// The decoded byte length.
    pub bytes: usize,
}

static DATA_URL: Lazy<Option<Regex>> = Lazy::new(|| {
    Regex::new(r"(?is)^data:([^;,]+)(?:;charset=([^;,]+))?;base64,([a-z\d+/=\s]+)$").ok()
});

/// Parse a base64 `data:` URL.
///
/// **External contract** — `input-parts.mjs:26-36`. Only the base64 form is
/// recognized; a percent-encoded `data:` URL returns `None` and is rejected
/// upstream by [`normalize_input_parts`].
///
/// A pattern that failed to compile answers `None`, which
/// [`normalize_file_part`] reads as *not base64* and refuses. The literal
/// cannot in fact fail; the fallback refuses the attachment rather than
/// admitting one whose declared MIME was never checked against its content.
#[must_use]
pub fn parse_data_url(value: &str) -> Option<DataUrl> {
    let captures = DATA_URL.as_ref()?.captures(value)?;
    let raw = captures.get(3).map_or("", |group| group.as_str());
    let data: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    Some(DataUrl {
        mime_type: captures
            .get(1)
            .map_or(String::new(), |group| group.as_str().to_lowercase()),
        charset: captures
            .get(2)
            .map_or(String::new(), |group| group.as_str().to_owned()),
        bytes: decoded_base64_bytes(&data),
        data,
    })
}

/// The decoded length of a base64 payload, without decoding it.
///
/// **External contract** — `input-parts.mjs:19-24`. Length arithmetic rather
/// than a decode, because the check runs before the size limit and decoding a
/// 20 MiB string to find out it is too large is the attack.
#[must_use]
pub fn decoded_base64_bytes(value: &str) -> usize {
    let content: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    if content.is_empty() {
        return 0;
    }
    let padding = if content.ends_with("==") {
        2
    } else if content.ends_with('=') {
        1
    } else {
        0
    };
    (content.len() * 3 / 4).saturating_sub(padding)
}

/// The byte size an attachment contributes to the turn's budget.
///
/// An `http(s):` attachment contributes zero: the Gateway never fetches it,
/// so it costs nothing here.
#[must_use]
pub fn part_bytes(part: &InputPart) -> usize {
    parse_data_url(part.url()).map_or(0, |data| data.bytes)
}

fn clean_text(value: &str, max: usize) -> String {
    let stripped: String = value.chars().filter(|c| *c != '\0').collect();
    crate::text::bounded_utf16(&stripped, max)
}

fn clean_filename(value: &str) -> String {
    let bounded = clean_text(value, MAX_FILENAME_CHARS);
    let collapsed: String = bounded
        .chars()
        .map(|c| {
            if matches!(c, '\r' | '\n' | '\t') {
                ' '
            } else {
                c
            }
        })
        .collect();
    trim(&collapsed).to_owned()
}

fn normalize_source(source: &PartSource) -> PartSource {
    let text = source.text.as_ref().map(|text| SourceText {
        value: clean_text(&text.value, MAX_SOURCE_CHARS),
        start: text.start,
        end: text.end,
    });
    PartSource {
        kind: source.kind,
        path: {
            let path = trim(&clean_text(
                source.path.as_deref().unwrap_or_default(),
                MAX_SOURCE_CHARS,
            ))
            .to_owned();
            (!path.is_empty()).then_some(path)
        },
        text: text.filter(|text| !text.value.is_empty()),
    }
}

/// Validate one attachment.
///
/// **External contract** — `input-parts.mjs:58-90`, check for check and in
/// upstream's order: MIME present and shaped `type/subtype`, URL present, URL
/// parseable, scheme allowed, `data:` base64, declared MIME agrees with the
/// `data:` MIME, size within [`MAX_INPUT_FILE_BYTES`].
///
/// The MIME-agreement check is not pedantry: the declared MIME is what reaches
/// the model and the backend, while the `data:` MIME is what a decoder acts
/// on, and a mismatch is how an image is smuggled past an image-only path.
///
/// # Errors
///
/// [`InputError`] with the localized message the client is shown.
pub fn normalize_file_part(part: &InputPart, locale: Locale) -> Result<InputPart, InputError> {
    let mime = trim(&clean_text(part.mime(), MAX_MIME_CHARS)).to_lowercase();
    if mime.is_empty() || !mime.contains('/') {
        return Err(InputError::new(
            t(locale, keys::INPUT_MISSING_MIME).to_owned(),
        ));
    }
    let url = trim(&clean_text(part.url(), MAX_URL_CHARS)).to_owned();
    if url.is_empty() {
        return Err(InputError::new(
            t(locale, keys::INPUT_MISSING_URL).to_owned(),
        ));
    }
    let parsed = url::Url::parse(&url)
        .map_err(|_| InputError::new(t(locale, keys::INPUT_INVALID_URL).to_owned()))?;
    let scheme = parsed.scheme().to_lowercase();
    if !ALLOWED_URL_PROTOCOLS.contains(&scheme.as_str()) {
        return Err(InputError::new(i18n_format(
            locale,
            keys::INPUT_UNSUPPORTED_URL_PROTOCOL,
            // Upstream interpolates `new URL(url).protocol`, which keeps the
            // colon. Reproduced so the message reads identically.
            &[("protocol", &format!("{scheme}:"))],
        )));
    }
    let data_url = (scheme == "data").then(|| parse_data_url(&url)).flatten();
    if scheme == "data" && data_url.is_none() {
        return Err(InputError::new(
            t(locale, keys::INPUT_DATA_URL_NEEDS_BASE64).to_owned(),
        ));
    }
    if let Some(data) = &data_url {
        if data.mime_type != mime {
            return Err(InputError::new(i18n_format(
                locale,
                keys::INPUT_MIME_MISMATCH,
                &[("declared", &mime), ("actual", &data.mime_type)],
            )));
        }
        if data.bytes > MAX_INPUT_FILE_BYTES {
            return Err(InputError::new(i18n_format(
                locale,
                keys::INPUT_FILE_TOO_LARGE,
                &[("limit", &megabytes(MAX_INPUT_FILE_BYTES))],
            )));
        }
    }
    let filename = clean_filename(part.filename().unwrap_or_default());
    Ok(InputPart::File {
        mime,
        filename: (!filename.is_empty()).then_some(filename),
        url,
        source: part.source().map(normalize_source),
        meta: None,
    })
}

fn megabytes(bytes: usize) -> String {
    // Upstream renders `MAX / 1024 / 1024`, which is an exact integer for both
    // bounds; `format!` on the quotient reproduces `8` and `12`.
    (bytes / 1024 / 1024).to_string()
}

/// Validate a whole submission.
///
/// **External contract** — `input-parts.mjs:92-118`. Four properties:
///
/// - the count bound is checked **before** anything is normalized, so an
///   oversized submission is refused without decoding it;
/// - a non-object part is skipped and an unknown `type` is rejected — the
///   asymmetry is upstream's, and it means a client sending `null` in an array
///   is tolerated while a client inventing a part type is not;
/// - `fallback_text` is unshifted only when **no** text part survived, so the
///   `text.message` body does not duplicate a text part the client already
///   sent;
/// - the total-size bound is checked last, on what actually survived.
///
/// # Errors
///
/// [`InputError`] with the localized message the client is shown.
pub fn normalize_input_parts(
    parts: &[InputPart],
    fallback_text: &str,
    locale: Locale,
) -> Result<Vec<InputPart>, InputError> {
    if parts.len() > MAX_INPUT_PARTS {
        return Err(InputError::new(i18n_format(
            locale,
            keys::INPUT_TOO_MANY_PARTS,
            &[("limit", &MAX_INPUT_PARTS.to_string())],
        )));
    }
    let mut normalized: Vec<InputPart> = Vec::with_capacity(parts.len());
    for part in parts {
        match part {
            InputPart::Text { text } => {
                let text = trim(&clean_text(text, MAX_TEXT_CHARS)).to_owned();
                if !text.is_empty() {
                    normalized.push(InputPart::text(text));
                }
            }
            InputPart::File { .. } => normalized.push(normalize_file_part(part, locale)?),
        }
    }
    if !normalized
        .iter()
        .any(|part| matches!(part, InputPart::Text { .. }))
    {
        let text = trim(&clean_text(fallback_text, MAX_TEXT_CHARS)).to_owned();
        if !text.is_empty() {
            normalized.insert(0, InputPart::text(text));
        }
    }
    let total: usize = normalized.iter().map(part_bytes).sum();
    if total > MAX_INPUT_TOTAL_FILE_BYTES {
        return Err(InputError::new(i18n_format(
            locale,
            keys::INPUT_TOTAL_TOO_LARGE,
            &[("limit", &megabytes(MAX_INPUT_TOTAL_FILE_BYTES))],
        )));
    }
    if normalized.is_empty() {
        return Err(InputError::new(t(locale, keys::INPUT_EMPTY).to_owned()));
    }
    Ok(normalized)
}

/// Every text part, joined with newlines.
///
/// **External contract** — `input-parts.mjs:120-126`.
#[must_use]
pub fn input_text(parts: &[InputPart]) -> String {
    parts
        .iter()
        .filter_map(|part| match part {
            InputPart::Text { text } => {
                let text = trim(text);
                (!text.is_empty()).then_some(text)
            }
            InputPart::File { .. } => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every file part.
#[must_use]
pub fn input_file_parts(parts: &[InputPart]) -> Vec<InputPart> {
    parts
        .iter()
        .filter(|part| part.is_file())
        .cloned()
        .collect()
}

/// The textual anchor for one attachment.
///
/// **External contract** — `input-parts.mjs:148-153`: a client-supplied
/// `source.text.value` wins; otherwise `[Image N]` for an image, `@filename`
/// when there is a filename, and `[File N]` as the last resort.
#[must_use]
pub fn input_part_reference(part: &InputPart, index: usize) -> String {
    let supplied = part
        .source()
        .and_then(|source| source.text.as_ref())
        .map_or("", |text| trim(&text.value));
    if !supplied.is_empty() {
        return crate::text::bounded_utf16(supplied, MAX_SOURCE_CHARS);
    }
    if part.mime().starts_with("image/") {
        return format!("[Image {}]", index + 1);
    }
    match part.filename() {
        Some(filename) if !filename.is_empty() => format!("@{filename}"),
        _ => format!("[File {}]", index + 1),
    }
}

/// The bounded label for one attachment.
///
/// **External contract** — `input-parts.mjs:180-182`.
#[must_use]
pub fn input_part_label(part: &InputPart, index: usize) -> String {
    crate::text::bounded_utf16(&input_part_reference(part, index), MAX_LABEL_CHARS)
}

/// Distinct anchors for a list of attachments.
///
/// **External contract** — `input-parts.mjs:184-203`. Two attachments with the
/// same natural anchor would make the offsets ambiguous, so the second gets
/// the next free `[Image N]` / `[File N]` instead.
#[must_use]
pub fn unique_attachment_references(files: &[InputPart]) -> Vec<String> {
    let mut used: Vec<String> = Vec::with_capacity(files.len());
    let mut references = Vec::with_capacity(files.len());
    for (index, part) in files.iter().enumerate() {
        let natural = input_part_reference(part, index);
        if !used.contains(&natural) {
            used.push(natural.clone());
            references.push(natural);
            continue;
        }
        let prefix = if part.mime().starts_with("image/") {
            "Image"
        } else {
            "File"
        };
        let mut ordinal = index + 1;
        let mut candidate = format!("[{prefix} {ordinal}]");
        while used.contains(&candidate) {
            ordinal += 1;
            candidate = format!("[{prefix} {ordinal}]");
        }
        used.push(candidate.clone());
        references.push(candidate);
    }
    references
}

/// Give every attachment a textual anchor and bind the offsets back.
///
/// **External contract** — `input-parts.mjs:212-239`. The file part stays
/// authoritative; the anchor preserves the user's visible prompt and makes a
/// multi-attachment reference replayable. Anchors the user's own text already
/// contains are not repeated.
#[must_use]
pub fn with_attachment_anchors(parts: &[InputPart]) -> Vec<InputPart> {
    let files = input_file_parts(parts);
    if files.is_empty() {
        return parts.to_vec();
    }
    let references = unique_attachment_references(&files);
    let original_text = input_text(parts);
    let missing: Vec<&String> = references
        .iter()
        .filter(|reference| !original_text.contains(reference.as_str()))
        .collect();
    let prefix = missing
        .iter()
        .map(|reference| reference.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let text = [prefix.as_str(), original_text.as_str()]
        .into_iter()
        .filter(|piece| !piece.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let mut anchored = Vec::with_capacity(files.len() + 1);
    anchored.push(InputPart::text(text.clone()));
    let mut search_from = 0usize;
    for (index, part) in files.into_iter().enumerate() {
        let reference = &references[index];
        // Upstream indexes with UTF-16 units, which is what the offsets mean
        // to a JavaScript client; `find` gives a byte offset, so convert.
        let start = utf16_index_of(&text, reference, search_from)
            .or_else(|| utf16_index_of(&text, reference, 0));
        let width = crate::text::utf16_len(reference);
        if let Some(start) = start {
            search_from = start + width;
        }
        // Upstream's `indexOf` answers `-1` when the anchor is absent, and it
        // writes that through rather than dropping the offsets.
        let start = start.map_or(-1i64, |value| i64::try_from(value).unwrap_or(i64::MAX));
        let source = part.source().cloned().unwrap_or_default();
        let InputPart::File {
            mime,
            filename,
            url,
            meta,
            ..
        } = part
        else {
            continue;
        };
        anchored.push(InputPart::File {
            mime,
            filename,
            url,
            source: Some(PartSource {
                kind: source.kind,
                path: source.path,
                text: Some(SourceText {
                    value: reference.clone(),
                    start: Some(start),
                    end: Some(start + i64::try_from(width).unwrap_or(0)),
                }),
            }),
            meta,
        });
    }
    anchored
}

/// `haystack.indexOf(needle, from)` in UTF-16 units.
fn utf16_index_of(haystack: &str, needle: &str, from_utf16: usize) -> Option<usize> {
    let mut units = 0usize;
    let mut byte_of_unit: Vec<(usize, usize)> = Vec::new();
    for (byte, character) in haystack.char_indices() {
        byte_of_unit.push((units, byte));
        units += character.len_utf16();
    }
    byte_of_unit.push((units, haystack.len()));
    let start_byte = byte_of_unit
        .iter()
        .find(|(unit, _)| *unit >= from_utf16)
        .map(|(_, byte)| *byte)?;
    let found = haystack.get(start_byte..)?.find(needle)? + start_byte;
    byte_of_unit
        .iter()
        .find(|(_, byte)| *byte == found)
        .map(|(unit, _)| *unit)
}

/// The model-visible projection of one turn's input.
///
/// **External contract** — `input-parts.mjs:241-268`. Three decisions:
///
/// - with no attachments it is just the text, so an ordinary turn carries no
///   envelope at all;
/// - the metadata is JSON with `&`, `<` and `>` escaped as `\u00XX`, so a
///   filename cannot close the `<input_parts>` tag and forge a block;
/// - a turn with attachments and no text gets one of two fixed sentences
///   depending on whether voice accompanied it, because the model needs to
///   know *something* was submitted.
#[must_use]
pub fn frontend_input_projection(
    parts: &[InputPart],
    accompanies_voice: bool,
    locale: Locale,
) -> String {
    let text = input_text(parts);
    let files = input_file_parts(parts);
    if files.is_empty() {
        return text;
    }
    let attachments: Vec<Value> = files
        .iter()
        .enumerate()
        .map(|(index, part)| {
            let mut entry = Map::new();
            let reference = part.reference();
            if !reference.is_empty() {
                entry.insert("id".to_owned(), Value::String(reference.to_owned()));
            }
            entry.insert("type".to_owned(), Value::String("file".to_owned()));
            if let Some(filename) = part.filename() {
                entry.insert("filename".to_owned(), Value::String(filename.to_owned()));
            }
            entry.insert("mime".to_owned(), Value::String(part.mime().to_owned()));
            let mut source = Map::new();
            source.insert(
                "type".to_owned(),
                Value::String(
                    part.source()
                        .map_or(SourceKind::Resource, |source| source.kind)
                        .as_str()
                        .to_owned(),
                ),
            );
            let mut source_text = Map::new();
            source_text.insert(
                "value".to_owned(),
                Value::String(input_part_label(part, index)),
            );
            source.insert("text".to_owned(), Value::Object(source_text));
            entry.insert("source".to_owned(), Value::Object(source));
            Value::Object(entry)
        })
        .collect();
    let metadata = serde_json::to_string(&Value::Array(attachments))
        .unwrap_or_else(|_| "[]".to_owned())
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");
    let head = if text.is_empty() {
        t(
            locale,
            if accompanies_voice {
                keys::INPUT_FALLBACK_WITH_VOICE
            } else {
                keys::INPUT_FALLBACK_WITHOUT_TEXT
            },
        )
        .to_owned()
    } else {
        text
    };
    [
        head.as_str(),
        "",
        "<input_parts>",
        &metadata,
        "</input_parts>",
    ]
    .join("\n")
}

/// What a client shows for a turn with no text of its own.
///
/// **External contract** — `input-parts.mjs:270-275`.
#[must_use]
pub fn display_input_text(parts: &[InputPart]) -> String {
    let text = input_text(parts);
    if !text.is_empty() {
        return text;
    }
    input_file_parts(parts)
        .iter()
        .enumerate()
        .map(|(index, part)| input_part_label(part, index))
        .collect::<Vec<_>>()
        .join(" ")
}

/// One attachment's metadata, as a conversation record stores it.
///
/// **External contract** — `input-parts.mjs:277-284`, field names included.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    /// The rendered anchor.
    pub label: String,
    /// The original filename, or `null`.
    pub name: Option<String>,
    /// The MIME type.
    pub mime_type: String,
    /// The decoded size, or `null` for a remote URL.
    pub bytes: Option<usize>,
}

/// Every attachment's metadata.
#[must_use]
pub fn input_attachment_metadata(parts: &[InputPart]) -> Vec<AttachmentMetadata> {
    input_file_parts(parts)
        .iter()
        .enumerate()
        .map(|(index, part)| AttachmentMetadata {
            label: input_part_label(part, index),
            name: part.filename().map(str::to_owned),
            mime_type: part.mime().to_owned(),
            bytes: parse_data_url(part.url()).map(|data| data.bytes),
        })
        .collect()
}

/// Merge attachment lists, keeping the first occurrence of each.
///
/// **External contract** — `tool-call-handler.mjs:18-29` (`mergeInputParts`).
/// Identity is the `input_N` reference when there is one, and `mime \0 url`
/// otherwise — so the same file submitted twice in one turn is delegated once.
/// Text parts are dropped entirely: only attachments travel with a delegation.
#[must_use]
pub fn merge_input_parts(groups: &[&[InputPart]]) -> Vec<InputPart> {
    let mut merged: Vec<InputPart> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for group in groups {
        for part in *group {
            if !part.is_file() {
                continue;
            }
            let reference = part.reference();
            let key = if reference.is_empty() {
                format!("{}\u{0}{}", part.mime(), part.url())
            } else {
                reference.to_owned()
            };
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);
            merged.push(part.clone());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const PNG: &str = "data:image/png;base64,iVBORw0KGgo=";

    fn image() -> InputPart {
        InputPart::file("image/png", PNG)
    }

    #[test]
    fn the_meta_key_is_rebranded() {
        assert_eq!(INPUT_REF_META_KEY, "via/inputRef");
        assert!(!INPUT_REF_META_KEY.contains("qwen"));
    }

    #[test]
    fn a_reference_round_trips_through_the_meta_bag() {
        let part = image().with_reference("input_3");
        assert_eq!(part.reference(), "input_3");
        // An empty reference writes nothing rather than an empty key.
        let untouched = image().with_reference("  ");
        assert_eq!(untouched.reference(), "");
    }

    #[test]
    fn a_file_url_is_rejected_however_it_is_spelled() {
        for url in ["file:///etc/passwd", "FILE:///etc/passwd", "ftp://host/x"] {
            let part = InputPart::file("text/plain", url);
            let error = normalize_file_part(&part, Locale::Zh)
                .expect_err("only data/http/https are allowed");
            assert!(error.message.contains(':'), "{url}: {}", error.message);
        }
    }

    #[test]
    fn every_rejection_path_has_its_own_message() {
        let cases: [(InputPart, via_i18n::Key); 5] = [
            (InputPart::file("", PNG), keys::INPUT_MISSING_MIME),
            (InputPart::file("notamime", PNG), keys::INPUT_MISSING_MIME),
            (InputPart::file("image/png", ""), keys::INPUT_MISSING_URL),
            (
                InputPart::file("image/png", "not a url"),
                keys::INPUT_INVALID_URL,
            ),
            (
                InputPart::file("image/png", "data:image/png,notbase64"),
                keys::INPUT_DATA_URL_NEEDS_BASE64,
            ),
        ];
        for (part, key) in cases {
            let error =
                normalize_file_part(&part, Locale::Zh).expect_err("this part must be rejected");
            assert_eq!(error.message, t(Locale::Zh, key), "{part:?}");
        }
    }

    #[test]
    fn a_mime_that_disagrees_with_the_data_url_is_rejected() {
        let part = InputPart::file("image/jpeg", PNG);
        let error = normalize_file_part(&part, Locale::Zh).expect_err("smuggling is refused");
        assert!(error.message.contains("image/jpeg"));
        assert!(error.message.contains("image/png"));
    }

    #[test]
    fn an_oversized_attachment_is_rejected_by_length_arithmetic() {
        let payload = "A".repeat(MAX_INPUT_FILE_BYTES / 3 * 4 + 8);
        let part = InputPart::file("image/png", format!("data:image/png;base64,{payload}"));
        let error = normalize_file_part(&part, Locale::Zh).expect_err("8 MiB is the bound");
        assert_eq!(
            error.message,
            i18n_format(Locale::Zh, keys::INPUT_FILE_TOO_LARGE, &[("limit", "8")]),
        );
    }

    #[test]
    fn the_count_bound_is_checked_before_anything_is_normalized() {
        let parts = vec![InputPart::file("bad", "also-bad"); MAX_INPUT_PARTS + 1];
        let error = normalize_input_parts(&parts, "", Locale::Zh).expect_err("too many parts");
        assert_eq!(
            error.message,
            i18n_format(Locale::Zh, keys::INPUT_TOO_MANY_PARTS, &[("limit", "16")]),
        );
    }

    #[test]
    fn the_fallback_text_is_used_only_when_no_text_part_survived() {
        let with_text =
            normalize_input_parts(&[InputPart::text("hello"), image()], "fallback", Locale::Zh)
                .expect("valid");
        assert_eq!(input_text(&with_text), "hello");

        let without = normalize_input_parts(&[image()], "fallback", Locale::Zh).expect("valid");
        assert_eq!(input_text(&without), "fallback");
        assert!(
            matches!(without[0], InputPart::Text { .. }),
            "unshifted to the front"
        );

        // A whitespace-only text part does not count as text.
        let blank =
            normalize_input_parts(&[InputPart::text("   "), image()], "fallback", Locale::Zh)
                .expect("valid");
        assert_eq!(input_text(&blank), "fallback");
    }

    #[test]
    fn an_empty_submission_is_rejected() {
        let error = normalize_input_parts(&[], "", Locale::Zh).expect_err("nothing to submit");
        assert_eq!(error.message, t(Locale::Zh, keys::INPUT_EMPTY));
    }

    #[test]
    fn a_nul_is_stripped_from_text_before_the_bound() {
        let parts =
            normalize_input_parts(&[InputPart::text("a\0b")], "", Locale::Zh).expect("valid");
        assert_eq!(input_text(&parts), "ab");
    }

    #[test]
    fn anchors_are_unique_and_bound_back_with_offsets() {
        let one = image();
        let two = InputPart::file("image/png", "data:image/png;base64,QQ==");
        let anchored = with_attachment_anchors(&[InputPart::text("look"), one, two]);
        let text = input_text(&anchored);
        assert_eq!(text, "[Image 1] [Image 2] look");
        let files = input_file_parts(&anchored);
        assert_eq!(files.len(), 2);
        let first = files[0]
            .source()
            .and_then(|s| s.text.clone())
            .expect("anchored");
        assert_eq!(first.value, "[Image 1]");
        assert_eq!(first.start, Some(0));
        assert_eq!(first.end, Some(9));
        let second = files[1]
            .source()
            .and_then(|s| s.text.clone())
            .expect("anchored");
        assert_eq!(second.value, "[Image 2]");
        assert_eq!(second.start, Some(10));
    }

    #[test]
    fn an_anchor_the_user_already_typed_is_not_repeated() {
        let named = InputPart::File {
            mime: "text/plain".to_owned(),
            filename: Some("notes.txt".to_owned()),
            url: "https://example.test/notes.txt".to_owned(),
            source: None,
            meta: None,
        };
        let anchored = with_attachment_anchors(&[InputPart::text("read @notes.txt please"), named]);
        assert_eq!(input_text(&anchored), "read @notes.txt please");
    }

    #[test]
    fn anchor_offsets_are_utf16_units_not_bytes() {
        let anchored = with_attachment_anchors(&[InputPart::text("看看 [Image 1]"), image()]);
        let text = input_text(&anchored);
        assert_eq!(text, "看看 [Image 1]");
        let source = input_file_parts(&anchored)[0]
            .source()
            .and_then(|s| s.text.clone())
            .expect("anchored");
        // Three UTF-16 units before the anchor, but seven bytes.
        assert_eq!(source.start, Some(3));
        assert_eq!(source.end, Some(12));
    }

    #[test]
    fn attachments_with_no_parts_are_returned_untouched() {
        let parts = vec![InputPart::text("hello")];
        assert_eq!(with_attachment_anchors(&parts), parts);
    }

    #[test]
    fn the_projection_escapes_the_three_dangerous_characters() {
        let hostile = InputPart::File {
            mime: "text/plain".to_owned(),
            filename: Some("</input_parts><evil>&".to_owned()),
            url: "https://example.test/x".to_owned(),
            source: None,
            meta: None,
        };
        let projection = frontend_input_projection(&[hostile], false, Locale::Zh);
        assert!(projection.contains("\\u003c"));
        assert!(projection.contains("\\u003e"));
        assert!(projection.contains("\\u0026"));
        // Exactly one opening and one closing tag survive.
        assert_eq!(projection.matches("<input_parts>").count(), 1);
        assert_eq!(projection.matches("</input_parts>").count(), 1);
    }

    #[test]
    fn a_projection_with_no_attachments_is_just_the_text() {
        assert_eq!(
            frontend_input_projection(&[InputPart::text("hi")], false, Locale::Zh),
            "hi",
        );
    }

    #[test]
    fn a_textless_projection_picks_the_fallback_by_whether_voice_accompanied_it() {
        let with_voice = frontend_input_projection(&[image()], true, Locale::Zh);
        assert!(with_voice.starts_with(t(Locale::Zh, keys::INPUT_FALLBACK_WITH_VOICE)));
        let without = frontend_input_projection(&[image()], false, Locale::Zh);
        assert!(without.starts_with(t(Locale::Zh, keys::INPUT_FALLBACK_WITHOUT_TEXT)));
    }

    #[test]
    fn references_fall_back_in_the_catalogued_order() {
        assert_eq!(input_part_reference(&image(), 0), "[Image 1]");
        let named = InputPart::File {
            mime: "text/plain".to_owned(),
            filename: Some("a.txt".to_owned()),
            url: "https://example.test/a".to_owned(),
            source: None,
            meta: None,
        };
        assert_eq!(input_part_reference(&named, 0), "@a.txt");
        let anonymous = InputPart::file("text/plain", "https://example.test/a");
        assert_eq!(input_part_reference(&anonymous, 2), "[File 3]");
        let supplied = InputPart::File {
            mime: "text/plain".to_owned(),
            filename: None,
            url: "https://example.test/a".to_owned(),
            source: Some(PartSource {
                kind: SourceKind::Clipboard,
                path: None,
                text: Some(SourceText {
                    value: "the pasted line".to_owned(),
                    start: None,
                    end: None,
                }),
            }),
            meta: None,
        };
        assert_eq!(input_part_reference(&supplied, 0), "the pasted line");
    }

    #[test]
    fn duplicate_natural_anchors_are_made_unique() {
        let same = InputPart::File {
            mime: "text/plain".to_owned(),
            filename: Some("a.txt".to_owned()),
            url: "https://example.test/a".to_owned(),
            source: None,
            meta: None,
        };
        let references = unique_attachment_references(&[same.clone(), same]);
        assert_eq!(references, vec!["@a.txt".to_owned(), "[File 2]".to_owned()]);
    }

    #[test]
    fn merging_keeps_the_first_of_each_and_drops_text() {
        let a = image().with_reference("input_1");
        let duplicate =
            InputPart::file("image/jpeg", "data:image/jpeg;base64,AA==").with_reference("input_1");
        let b = InputPart::file("image/gif", "data:image/gif;base64,AA==");
        let same_bytes = InputPart::file("image/gif", "data:image/gif;base64,AA==");
        let text = InputPart::text("ignored");
        let merged = merge_input_parts(&[&[text, a.clone(), duplicate], &[b.clone(), same_bytes]]);
        assert_eq!(merged, vec![a, b]);
    }

    #[test]
    fn attachment_metadata_reports_size_only_for_inlined_bytes() {
        let remote = InputPart::file("text/plain", "https://example.test/a");
        let metadata = input_attachment_metadata(&[image(), remote]);
        assert_eq!(metadata[0].mime_type, "image/png");
        assert!(metadata[0].bytes.is_some());
        assert_eq!(metadata[1].bytes, None);
        assert_eq!(metadata[1].name, None);
    }

    #[test]
    fn display_text_falls_back_to_the_attachment_labels() {
        assert_eq!(display_input_text(&[InputPart::text("hi"), image()]), "hi");
        assert_eq!(
            display_input_text(&[image(), image()]),
            "[Image 1] [Image 2]"
        );
    }

    #[test]
    fn base64_length_arithmetic_matches_the_padding_rules() {
        assert_eq!(decoded_base64_bytes(""), 0);
        assert_eq!(decoded_base64_bytes("QQ=="), 1);
        assert_eq!(decoded_base64_bytes("QUE="), 2);
        assert_eq!(decoded_base64_bytes("QUFB"), 3);
        assert_eq!(decoded_base64_bytes("Q U F B"), 3);
    }

    #[test]
    fn a_data_url_parses_with_its_charset_and_stripped_whitespace() {
        let parsed = parse_data_url("data:TEXT/Plain;charset=utf-8;base64,QU\n FB")
            .expect("a base64 data URL");
        assert_eq!(parsed.mime_type, "text/plain");
        assert_eq!(parsed.charset, "utf-8");
        assert_eq!(parsed.data, "QUFB");
        assert_eq!(parsed.bytes, 3);
        assert_eq!(parse_data_url("https://example.test/x"), None);
    }

    #[test]
    fn a_remote_attachment_costs_nothing_against_the_turn_budget() {
        let remote = InputPart::file("text/plain", "https://example.test/a");
        assert_eq!(part_bytes(&remote), 0);
        assert!(part_bytes(&image()) > 0);
    }
}
