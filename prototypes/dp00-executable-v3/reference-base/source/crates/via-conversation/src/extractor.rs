//! [`MemoryExtractor`] — reconciling what the realtime tool missed.
//!
//! Ported from `server/src/conversation/memory-extractor.mjs`.
//!
//! After a voice connection closes, a lightweight text model reads the session
//! transcript and proposes durable personal facts and standing directives that
//! the realtime `memory` tool did not capture in the moment. The pipeline is
//! deliberately conservative: a voice-only product has no confirmation UI, so
//! correctness comes from writing rarely, from easy voice-native correction,
//! and from the audit trail — never from asking the user.
//!
//! # The invariants that make an unsupervised writer safe
//!
//! Each of these is load-bearing and none may be weakened:
//!
//! - **Every write goes through [`FrontendMemoryService`]**, the same service
//!   the realtime tool uses. The extractor never opens a Markdown file, so the
//!   revision check, the fragment-uniqueness rule and the document caps all
//!   apply to it exactly as they apply to the model.
//! - **`ASSISTANT.md` is never touched.** It is not one of the two documents
//!   the service knows about, so there is no path from here to it.
//! - **`USER.md` accepts only explicit interaction directives**, and only when
//!   the *user's own turns* contain one ([`has_explicit_user_directive`]). A
//!   model that infers a directive from the assistant's side, or from
//!   atmosphere, is refused with
//!   [`SkipReason::UserDirectiveNotExplicit`](crate::audit::SkipReason).
//! - **`MEMORY.md` rejects directive-shaped content**, which is the same
//!   classifier read the other way round
//!   ([`SkipReason::DocumentBoundary`](crate::audit::SkipReason)). The two
//!   documents differ by authority, so putting a standing instruction in the
//!   factual one would launder it past the hierarchy.
//! - **A second sensitive-content gate** runs behind the prompt's own rule.
//! - **Failure is silent.** Extraction must never delay session close, never
//!   break it, and never produce speech. Every outcome lands in the audit file
//!   and nowhere else.
//! - **No API key, no extractor.** [`create_extractor_llm`] returns `None`
//!   without one and [`MemoryExtractor::enabled`] is then `false` — the
//!   product degrades without a sound.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Map, Value};
use via_i18n::{Locale, format as i18n_format, keys, t};

use crate::audit::{AuditEvent, MemoryAudit, SkipReason};
use crate::markdown_store::{MarkdownEdit, MemoryDocument};
use crate::memory_service::{FrontendMemoryService, MemoryChange};
use crate::sync::{ConversationSyncHandle, Message, MessageRole, SessionRef};
use crate::text::{bounded_code_points, clean, normalize, trim, utf16_len};

/// The two documents the extractor may name.
///
/// **External contract** — `memory-extractor.mjs:16`. Note that this is the
/// extractor's own set and is **not** alias-resolved: a change naming `profile`
/// is [`SkipReason::InvalidChange`], not a `user` write. The extractor writes
/// what the prompt told it to write or nothing at all.
pub const EXTRACTOR_DOCUMENTS: [&str; 2] = ["user", "memory"];

/// Cap on exact edits per change.
///
/// **External contract** — `memory-extractor.mjs:17` (`MAX_OPS_PER_RUN`).
pub const MAX_OPS_PER_RUN: usize = 5;

/// Code-point cap on each patch text field.
///
/// **External contract** — `memory-extractor.mjs:18` (`MAX_PATCH_CHARS`).
pub const MAX_PATCH_CHARS: usize = 1000;

/// Cap on changes per run — one per document, and there are two.
///
/// **External contract** — `memory-extractor.mjs:124` (`.slice(0, 2)`).
pub const MAX_CHANGES_PER_RUN: usize = 2;

/// How long after a run the same owner is skipped.
///
/// **External contract** — `memory-extractor.mjs:203` (`30 * 60_000`).
pub const DEFAULT_DEBOUNCE_MS: i64 = 30 * 60_000;

/// How many user turns a session needs before it is worth extracting from.
///
/// **External contract** — `memory-extractor.mjs:204`.
pub const DEFAULT_MIN_USER_MESSAGES: usize = 4;

/// The transcript's character budget.
///
/// **External contract** — `memory-extractor.mjs:205`.
pub const DEFAULT_MAX_TRANSCRIPT_CHARS: usize = 6000;

/// How long the one HTTP call may take.
///
/// **External contract** — `memory-extractor.mjs:160` (`timeoutMs = 10_000`).
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_millis(10_000);

/// The path appended to the configured base URL.
///
/// **External contract** — the catalogued *memory extractor LLM request*:
/// `` POST `${baseUrl}/chat/completions` ``.
pub const CHAT_COMPLETIONS_PATH: &str = "/chat/completions";

/// The sampling temperature. Zero, because the same transcript must produce the
/// same patch.
///
/// **External contract** — `memory-extractor.mjs:179`.
pub const EXTRACTOR_TEMPERATURE: f64 = 0.0;

/// The secondary sensitive-content gate.
///
/// **External contract** — `memory-extractor.mjs:23-29`. Conservative by
/// design, and upstream says why: *a false positive drops one candidate fact, a
/// false negative persists a secret.* The last two patterns are shape-based —
/// a long digit run, a long base64-ish run — and will occasionally refuse an
/// innocent fact. That is the trade this gate exists to make.
pub const SENSITIVE_PATTERNS: [&str; 5] = [
    r"(?i)(?:api[_-]?key|access[_-]?key|secret|token|password|passwd)",
    r"(?:密码|密钥|口令|验证码|令牌|证件号|身份证)",
    r"\bsk-[A-Za-z0-9]{8,}",
    r"\b\d{11,19}\b",
    r"[A-Za-z0-9+/]{40,}={0,2}",
];

/// The document-boundary classifier.
///
/// **External contract** — `memory-extractor.mjs:33-45`. One classifier, read
/// in both directions: content it matches belongs in `USER.md` and is refused
/// from `MEMORY.md`; content it does not match is the reverse. Writing it once
/// is what keeps the two rules from drifting apart into a gap that leaks.
pub const USER_PREFERENCE_PATTERNS: [&str; 11] = [
    r"(?:我叫|我的名字|用户姓名|用户身份)",
    r"(?:叫我|称呼我|如何称呼|我的称呼)",
    r"(?:你|助手).{0,12}(?:叫|自称|名字|称为)",
    r"(?:你|助手).{0,12}(?:像|作为|当作).{0,8}(?:朋友|伙伴|老师|教练|秘书|助理)",
    r"(?:回复|回答|说话|表达).{0,12}(?:简短|简洁|详细|展开|正式|随意|语气|风格)",
    r"(?:默认|以后).{0,12}(?:中文|英文|语言|时区|称呼|回复|回答)",
    r"(?:用户)?.{0,8}(?:希望|要求|让|请).{0,8}(?:助手|你).{0,30}(?:每次|每回|始终|总是|以后|默认|开头|结尾|加上|带上|说一句)",
    r"(?:助手|你).{0,20}(?:回复|回答|说话|表达|对话).{0,20}(?:每次|开头|结尾|加上|带上|说)",
    r"(?:每次|每回|以后|默认).{0,12}(?:回复|回答|说话|表达|称呼)",
    r"(?:回复|回答|说话|表达|对话).{0,12}(?:开头|结尾|加上|带上|说一句)",
    r"(?:助手称呼用户|用户称呼助手)",
];

/// Evidence that the user themselves asked for a standing change.
///
/// **External contract** — `memory-extractor.mjs:77-82`. Applied to the
/// **user's own transcript lines only**, which is the point: a `USER.md` write
/// changes how the assistant behaves from then on, so the sentence that
/// authorizes it has to have come from the user.
pub const EXPLICIT_DIRECTIVE_PATTERNS: [&str; 4] = [
    r"(?:以后|今后|从现在|每次|每回|始终|总是|默认|不要再|别再).{0,40}(?:叫|称呼|回复|回答|说|使用|用|加|带)",
    r"(?:叫我|称呼我|你叫|你以后叫|我叫你)",
    r"(?:我希望|我想让你|请你|你要).{0,40}(?:叫|称呼|回复|回答|说|表达|使用|加|带)",
    r"(?:回复|回答|说话|表达).{0,20}(?:简短|简洁|详细|正式|随意|温柔|慢一点|快一点)",
];

fn compile(patterns: &[&str]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
}

static SENSITIVE: Lazy<Vec<Regex>> = Lazy::new(|| compile(&SENSITIVE_PATTERNS));
static USER_PREFERENCE: Lazy<Vec<Regex>> = Lazy::new(|| compile(&USER_PREFERENCE_PATTERNS));
static EXPLICIT_DIRECTIVE: Lazy<Vec<Regex>> = Lazy::new(|| compile(&EXPLICIT_DIRECTIVE_PATTERNS));

/// Whether `value` trips the secondary sensitive-content gate.
#[must_use]
pub fn contains_sensitive_content(value: &str) -> bool {
    SENSITIVE.iter().any(|pattern| pattern.is_match(value))
}

/// Whether `value` reads as a standing interaction directive.
#[must_use]
pub fn belongs_to_user_preferences(value: &str) -> bool {
    USER_PREFERENCE
        .iter()
        .any(|pattern| pattern.is_match(value))
}

/// Whether the user's own turns carry an explicit standing directive.
///
/// `lines` are the rendered transcript lines; only those beginning with the
/// user's role prefix are considered.
#[must_use]
pub fn has_explicit_user_directive(lines: &[String], locale: Locale) -> bool {
    let prefix = format!("{}:", t(locale, keys::REALTIME_ROLE_USER));
    let user_text = lines
        .iter()
        .filter(|line| line.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    EXPLICIT_DIRECTIVE
        .iter()
        .any(|pattern| pattern.is_match(&user_text))
}

/// The transcript the model is shown, newest-first within a budget then
/// restored to order.
///
/// **External contract** — `memory-extractor.mjs:136-149`. The *most recent*
/// turns are what survive a long session, because a standing directive the user
/// gave five minutes ago matters more than one they gave two hours ago and then
/// revised.
#[must_use]
pub fn transcript_lines(messages: &[Message], max_chars: usize, locale: Locale) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut used = 0usize;
    for message in messages.iter().rev() {
        let content = clean(&message.content);
        if content.is_empty() {
            continue;
        }
        let role = t(
            locale,
            match message.role {
                MessageRole::User => keys::REALTIME_ROLE_USER,
                MessageRole::Assistant => keys::REALTIME_ROLE_ASSISTANT,
            },
        );
        let line = format!("{role}: {content}");
        if !lines.is_empty() && used + utf16_len(&line) > max_chars {
            break;
        }
        used += utf16_len(&line);
        lines.push(line);
    }
    lines.reverse();
    lines
}

/// `String(value || '').replaceAll('\0','').replace(/\r\n?/g,'\n').trim()`,
/// bounded to [`MAX_PATCH_CHARS`] code points.
///
/// **External contract** — `memory-extractor.mjs:85-89`.
#[must_use]
pub fn clean_patch(value: &str) -> String {
    bounded_code_points(trim(&normalize(value)), MAX_PATCH_CHARS)
}

/// Extract exactly the first complete JSON object from `value`.
///
/// **External contract** — `memory-extractor.mjs:94-115`. The model may wrap
/// its answer in a fence or append a sentence of commentary despite the
/// instruction; a harmless suffix must not discard an otherwise valid patch.
/// String contents and backslash escapes are tracked so a `}` inside a memory
/// fragment cannot end the object early.
///
/// Unbalanced input returns `value` unchanged, which then fails to parse — a
/// refusal, not a partial patch.
#[must_use]
pub fn first_json_object(value: &str) -> &str {
    let Some(start) = value.find('{') else {
        return value;
    };
    let mut depth = 0i64;
    let mut quoted = false;
    let mut escaped = false;
    for (offset, character) in value[start..].char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &value[start..start + offset + character.len_utf8()];
                }
            }
            _ => {}
        }
    }
    value
}

/// Strip a Markdown code fence, if the model wrapped its answer in one.
///
/// **External contract** — `memory-extractor.mjs:119`
/// (`/^```(?:json)?\s*/i` and `/```\s*$/`).
#[must_use]
pub fn strip_fence(value: &str) -> &str {
    let mut out = value;
    if let Some(rest) = out.strip_prefix("```") {
        let rest = rest
            .strip_prefix("json")
            .or_else(|| rest.strip_prefix("JSON"))
            .or_else(|| rest.strip_prefix("Json"))
            .unwrap_or(rest);
        out = rest.trim_start_matches(crate::text::is_js_whitespace);
    }
    let trimmed_end = out.trim_end_matches(crate::text::is_js_whitespace);
    if let Some(rest) = trimmed_end.strip_suffix("```") {
        out = rest;
    }
    out
}

/// One document's worth of proposed change, after parsing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProposedChange {
    /// The document name, trimmed and lower-cased. **Not** alias-resolved.
    pub document: String,
    /// Exact replacements, capped at [`MAX_OPS_PER_RUN`].
    pub edits: Vec<MarkdownEdit>,
    /// A Markdown block to append.
    pub append: String,
}

impl ProposedChange {
    /// Everything this change would add to a document: replacement texts plus
    /// the append, joined by a newline, with empty parts dropped.
    ///
    /// This is what the boundary classifier reads — deliberately **not** the
    /// `old_text` fragments, because those come from the existing document and
    /// classifying them would judge the change by what it removes.
    #[must_use]
    pub fn additions(&self) -> String {
        self.edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .chain(std::iter::once(self.append.as_str()))
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Every proposed text, joined — including empty parts, which is what the
    /// sensitive gate reads.
    ///
    /// **External contract** — `memory-extractor.mjs:283-286`, which does
    /// **not** filter, so the joined string carries the blank lines. Reproduced
    /// because a pattern anchored on `\b` can behave differently across one.
    #[must_use]
    pub fn proposed(&self) -> String {
        self.edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .chain(std::iter::once(self.append.as_str()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Why a patch could not be parsed.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PatchError {
    /// The text was not JSON.
    #[error("{0}")]
    Json(String),
    /// The JSON had no `changes` array.
    ///
    /// Contract — `memory-extractor.mjs:122`.
    #[error("extractor output has no changes array")]
    NoChangesArray,
}

/// Parse the model's answer into at most [`MAX_CHANGES_PER_RUN`] changes.
///
/// **External contract** — the catalogued *memory extractor expected output
/// schema*. Every tolerance here is deliberate: a fence is stripped, trailing
/// commentary is ignored, and each text field is bounded. What is **not**
/// tolerated is a missing `changes` array, because that means the model
/// answered a different question.
///
/// # Errors
///
/// [`PatchError`].
pub fn parse_patch(text: &str) -> Result<Vec<ProposedChange>, PatchError> {
    let raw = trim(text);
    let unfenced = strip_fence(raw);
    let candidate = first_json_object(unfenced);
    let parsed: Value =
        serde_json::from_str(candidate).map_err(|error| PatchError::Json(error.to_string()))?;
    let changes = parsed
        .get("changes")
        .and_then(Value::as_array)
        .ok_or(PatchError::NoChangesArray)?;
    Ok(changes
        .iter()
        .take(MAX_CHANGES_PER_RUN)
        .map(|change| ProposedChange {
            document: change
                .get("document")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_lowercase(),
            edits: change
                .get("edits")
                .and_then(Value::as_array)
                .map(|edits| {
                    edits
                        .iter()
                        .take(MAX_OPS_PER_RUN)
                        .map(|edit| MarkdownEdit {
                            old_text: clean_patch(
                                edit.get("old_text").and_then(Value::as_str).unwrap_or(""),
                            ),
                            new_text: clean_patch(
                                edit.get("new_text").and_then(Value::as_str).unwrap_or(""),
                            ),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            append: clean_patch(change.get("append").and_then(Value::as_str).unwrap_or("")),
        })
        .collect())
}

// ── the model call ──────────────────────────────────────────────────────────

/// Why the extraction model could not be reached, or did not answer.
#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    /// The endpoint answered with a non-success status.
    ///
    /// Contract — `memory-extractor.mjs:184`:
    /// `` `memory extractor request failed: ${response.status}` ``.
    #[error("memory extractor request failed: {status}")]
    Request {
        /// The HTTP status.
        status: u16,
    },
    /// The request could not be made or the answer could not be read.
    #[error("{0}")]
    Transport(String),
    /// The request outlived its timeout.
    #[error("memory extractor request timed out")]
    Timeout,
}

/// The one model call the extractor makes.
///
/// A trait so the pipeline can be exercised with no network at all: every rule
/// in [`MemoryExtractor::run`] is about what to do with an answer, and none of
/// them should need a socket to test.
#[async_trait]
pub trait ExtractorLlm: Send + Sync + std::fmt::Debug {
    /// One stateless completion.
    ///
    /// # Errors
    ///
    /// [`ExtractorError`].
    async fn complete(&self, system: &str, user: &str) -> Result<String, ExtractorError>;
}

/// Where the transcript comes from.
///
/// [`ConversationSyncHandle`] implements it; an embedder replaying a stored
/// session can implement it over anything.
#[async_trait]
pub trait TranscriptSource: Send + Sync + std::fmt::Debug {
    /// Every retained message for one conversation, oldest first.
    async fn list(&self, owner_id: &str, session_id: &str) -> Vec<Message>;
}

#[async_trait]
impl TranscriptSource for ConversationSyncHandle {
    async fn list(&self, owner_id: &str, session_id: &str) -> Vec<Message> {
        ConversationSyncHandle::list(self, &SessionRef::new(owner_id, session_id))
            .await
            .unwrap_or_default()
    }
}

/// The production [`ExtractorLlm`]: one OpenAI-compatible chat completion.
#[cfg(feature = "http")]
#[derive(Debug, Clone)]
pub struct HttpExtractorLlm {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    timeout: Duration,
}

#[cfg(feature = "http")]
impl HttpExtractorLlm {
    /// Build the client, or `None` when it would have nothing to talk to.
    ///
    /// **External contract** — `memory-extractor.mjs:162`
    /// (`if (!apiKey || !baseUrl || !model) return null`). Returning `None`
    /// rather than a client that fails at call time is what makes the
    /// extractor *silently disabled* instead of *quietly broken*: a Gateway
    /// with no `VIA_MEMORY_API_KEY` never schedules a run at all.
    #[must_use]
    pub fn new(base_url: &str, api_key: &str, model: &str, timeout: Duration) -> Option<Self> {
        if api_key.is_empty() || base_url.is_empty() || model.is_empty() {
            return None;
        }
        Some(Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_owned(),
            api_key: api_key.to_owned(),
            model: model.to_owned(),
            timeout,
        })
    }

    /// The endpoint this client posts to.
    #[must_use]
    pub fn endpoint(&self) -> String {
        format!("{}{CHAT_COMPLETIONS_PATH}", self.base_url)
    }
}

#[cfg(feature = "http")]
#[async_trait]
impl ExtractorLlm for HttpExtractorLlm {
    async fn complete(&self, system: &str, user: &str) -> Result<String, ExtractorError> {
        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
            "temperature": EXTRACTOR_TEMPERATURE,
        });
        let request = self
            .client
            .post(self.endpoint())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send();

        let response = tokio::time::timeout(self.timeout, request)
            .await
            .map_err(|_| ExtractorError::Timeout)?
            .map_err(|error| ExtractorError::Transport(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(ExtractorError::Request {
                status: status.as_u16(),
            });
        }
        let payload: Value = response
            .json()
            .await
            .map_err(|error| ExtractorError::Transport(error.to_string()))?;
        Ok(payload
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned())
    }
}

/// Build the production client from configuration.
///
/// A thin alias for [`HttpExtractorLlm::new`] that names upstream's factory.
#[cfg(feature = "http")]
#[must_use]
pub fn create_extractor_llm(
    base_url: &str,
    api_key: &str,
    model: &str,
    timeout: Duration,
) -> Option<Arc<dyn ExtractorLlm>> {
    HttpExtractorLlm::new(base_url, api_key, model, timeout)
        .map(|client| Arc::new(client) as Arc<dyn ExtractorLlm>)
}

// ── the extractor ───────────────────────────────────────────────────────────

/// Epoch-millisecond clock.
pub type NowFn = Arc<dyn Fn() -> i64 + Send + Sync>;

/// What one call to [`MemoryExtractor::maybe_run`] did.
///
/// Upstream returns `null` for the first four and a promise otherwise; the
/// distinction is preserved by [`ExtractionOutcome::was_gated`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtractionOutcome {
    /// No model is configured. The product degrades silently.
    Disabled,
    /// No owner id was supplied.
    NoOwner,
    /// This owner ran too recently.
    Debounced,
    /// The session had fewer than `min_user_messages` user turns.
    TooQuiet,
    /// Nothing in the transcript survived cleaning.
    EmptyTranscript,
    /// The model answered and the pipeline declined to write.
    Skipped(SkipReason),
    /// A patch was applied. Carries how many operations changed something.
    Completed {
        /// The sum of every change's `changed` count.
        changed: usize,
    },
    /// The model, the parser or the write failed. Recorded, never raised.
    Failed(String),
}

impl ExtractionOutcome {
    /// Whether the run was refused before the model was called at all.
    ///
    /// These four are upstream's `return null` branches: synchronous, free, and
    /// the reason a skipped session-close costs nothing.
    #[must_use]
    pub fn was_gated(&self) -> bool {
        matches!(
            self,
            Self::Disabled | Self::NoOwner | Self::Debounced | Self::TooQuiet
        )
    }
}

/// The session-end extractor.
#[derive(Clone)]
pub struct MemoryExtractor {
    inner: Arc<ExtractorInner>,
}

struct ExtractorInner {
    memory_service: FrontendMemoryService,
    transcripts: Arc<dyn TranscriptSource>,
    audit: Option<MemoryAudit>,
    llm: Option<Arc<dyn ExtractorLlm>>,
    locale: Locale,
    now: NowFn,
    debounce_ms: i64,
    min_user_messages: usize,
    max_transcript_chars: usize,
    last_run_at: Mutex<HashMap<String, i64>>,
}

impl std::fmt::Debug for MemoryExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryExtractor")
            .field("enabled", &self.enabled())
            .field("debounce_ms", &self.inner.debounce_ms)
            .field("min_user_messages", &self.inner.min_user_messages)
            .finish_non_exhaustive()
    }
}

/// Builder for [`MemoryExtractor`].
pub struct MemoryExtractorBuilder {
    memory_service: FrontendMemoryService,
    transcripts: Arc<dyn TranscriptSource>,
    audit: Option<MemoryAudit>,
    llm: Option<Arc<dyn ExtractorLlm>>,
    locale: Locale,
    now: Option<NowFn>,
    debounce_ms: i64,
    min_user_messages: usize,
    max_transcript_chars: usize,
}

impl std::fmt::Debug for MemoryExtractorBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryExtractorBuilder")
            .field("debounce_ms", &self.debounce_ms)
            .field("min_user_messages", &self.min_user_messages)
            .finish_non_exhaustive()
    }
}

impl MemoryExtractorBuilder {
    /// Start from the two things the extractor cannot work without.
    #[must_use]
    pub fn new(
        memory_service: FrontendMemoryService,
        transcripts: Arc<dyn TranscriptSource>,
    ) -> Self {
        Self {
            memory_service,
            transcripts,
            audit: None,
            llm: None,
            locale: Locale::default(),
            now: None,
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            min_user_messages: DEFAULT_MIN_USER_MESSAGES,
            max_transcript_chars: DEFAULT_MAX_TRANSCRIPT_CHARS,
        }
    }

    /// Where outcomes are recorded. Without one, runs are invisible.
    #[must_use]
    pub fn audit(mut self, audit: MemoryAudit) -> Self {
        self.audit = Some(audit);
        self
    }

    /// The model. `None` leaves the extractor disabled.
    #[must_use]
    pub fn llm(mut self, llm: Option<Arc<dyn ExtractorLlm>>) -> Self {
        self.llm = llm;
        self
    }

    /// Which locale the prompt and the transcript are rendered in.
    #[must_use]
    pub fn locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Override the clock.
    #[must_use]
    pub fn now(mut self, now: NowFn) -> Self {
        self.now = Some(now);
        self
    }

    /// How long after a run the same owner is skipped.
    #[must_use]
    pub fn debounce_ms(mut self, debounce_ms: i64) -> Self {
        self.debounce_ms = debounce_ms;
        self
    }

    /// How many user turns a session needs.
    #[must_use]
    pub fn min_user_messages(mut self, min_user_messages: usize) -> Self {
        self.min_user_messages = min_user_messages;
        self
    }

    /// The transcript budget.
    #[must_use]
    pub fn max_transcript_chars(mut self, max_transcript_chars: usize) -> Self {
        self.max_transcript_chars = max_transcript_chars;
        self
    }

    /// Finish the extractor.
    #[must_use]
    pub fn build(self) -> MemoryExtractor {
        MemoryExtractor {
            inner: Arc::new(ExtractorInner {
                memory_service: self.memory_service,
                transcripts: self.transcripts,
                audit: self.audit,
                llm: self.llm,
                locale: self.locale,
                now: self.now.unwrap_or_else(|| Arc::new(system_now_ms)),
                debounce_ms: self.debounce_ms,
                min_user_messages: self.min_user_messages,
                max_transcript_chars: self.max_transcript_chars,
                last_run_at: Mutex::new(HashMap::new()),
            }),
        }
    }
}

fn system_now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

impl MemoryExtractor {
    /// Whether a run would do anything.
    ///
    /// **External contract** — `memory-extractor.mjs:218-220`. Without a model
    /// this is `false` and every session close is a no-op.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.inner.llm.is_some()
    }

    /// The system prompt sent verbatim to the extraction model.
    ///
    /// **External contract** — the catalogued *MEMORY EXTRACTOR system prompt
    /// (EXTRACTOR_SYSTEM_PROMPT)*, truncated in the catalogue at 600
    /// characters. The catalogued text is a prefix of this one, and the four
    /// truncated tail rules (不提取 / 绝不提取 / 已有内容覆盖 / 同一文档合并 /
    /// 没有值得修改) are all present — `tests/contracts.rs` asserts both.
    #[must_use]
    pub fn system_prompt(&self) -> String {
        // Rendered rather than looked up: the JSON braces in the format example
        // are `{{`-escaped in the catalog, and `render` is what unescapes them.
        i18n_format(self.inner.locale, keys::MEMORY_EXTRACTOR_SYSTEM_PROMPT, &[])
    }

    /// The user message: both current documents, then the transcript.
    ///
    /// **External contract** — the catalogued *memory extractor user message
    /// layout*. The current documents are included so the model can propose an
    /// `edits` correction rather than appending a contradiction, and so it can
    /// see what is already covered.
    #[must_use]
    pub fn user_message(&self, documents: &[MemoryDocument], lines: &[String]) -> String {
        let locale = self.inner.locale;
        let find = |scope: &str| {
            documents
                .iter()
                .find(|document| document.scope == scope)
                .map(|document| document.content.clone())
        };
        [
            t(locale, keys::MEMORY_EXTRACTOR_SECTION_USER).to_owned(),
            find("user").unwrap_or_else(|| "# USER".to_owned()),
            String::new(),
            t(locale, keys::MEMORY_EXTRACTOR_SECTION_MEMORY).to_owned(),
            find("memory").unwrap_or_else(|| "# MEMORY".to_owned()),
            String::new(),
            t(locale, keys::MEMORY_EXTRACTOR_SECTION_TRANSCRIPT).to_owned(),
            lines.join("\n"),
        ]
        .join("\n")
    }

    /// The session-close hook.
    ///
    /// Every gate before the model call is cheap, so a skipped close costs
    /// nothing. The caller **spawns** this rather than awaiting it: extraction
    /// must never delay a session close, and it never produces speech.
    ///
    /// **External contract** — `memory-extractor.mjs:225-251`. The debounce is
    /// checked before the transcript is fetched and recorded only once the
    /// session has qualified, so a quiet session does not consume the window a
    /// later talkative one needs.
    pub async fn maybe_run(&self, owner_id: &str, session_id: &str) -> ExtractionOutcome {
        if !self.enabled() {
            return ExtractionOutcome::Disabled;
        }
        if owner_id.is_empty() {
            return ExtractionOutcome::NoOwner;
        }
        if self.debounced(owner_id) {
            return ExtractionOutcome::Debounced;
        }

        let messages = self.inner.transcripts.list(owner_id, session_id).await;
        let user_messages = messages
            .iter()
            .filter(|message| message.role == MessageRole::User)
            .count();
        if user_messages < self.inner.min_user_messages {
            return ExtractionOutcome::TooQuiet;
        }

        // Re-check under the lock before claiming the window. Upstream is
        // single-threaded and gets check-and-set for free; two concurrent
        // session closes here would otherwise both pass the first check and
        // both call the model.
        if !self.claim_run(owner_id) {
            return ExtractionOutcome::Debounced;
        }

        let outcome = self.run(owner_id, &messages).await;
        if let ExtractionOutcome::Failed(error) = &outcome {
            tracing::warn!(event = "memory.extract_failed", error = %error);
        }
        outcome
    }

    fn debounced(&self, owner_id: &str) -> bool {
        let now = (self.inner.now)();
        let guard = self
            .inner
            .last_run_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // "Debounce only after a previous run; the first close always
        // qualifies" — `memory-extractor.mjs:229-233`.
        guard
            .get(owner_id)
            .is_some_and(|last| now - *last < self.inner.debounce_ms)
    }

    fn claim_run(&self, owner_id: &str) -> bool {
        let now = (self.inner.now)();
        let mut guard = self
            .inner
            .last_run_at
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard
            .get(owner_id)
            .is_some_and(|last| now - *last < self.inner.debounce_ms)
        {
            return false;
        }
        guard.insert(owner_id.to_owned(), now);
        true
    }

    /// Run the pipeline over an explicit transcript.
    ///
    /// Public so the whole decision ladder can be exercised without the
    /// debounce and quiet-session gates in the way.
    pub async fn run(&self, owner_id: &str, messages: &[Message]) -> ExtractionOutcome {
        let locale = self.inner.locale;
        let lines = transcript_lines(messages, self.inner.max_transcript_chars, locale);
        if lines.is_empty() {
            return ExtractionOutcome::EmptyTranscript;
        }

        let existing = self
            .inner
            .memory_service
            .list(owner_id, None)
            .unwrap_or_default();
        let user = self.user_message(&existing, &lines);

        let Some(llm) = self.inner.llm.as_ref() else {
            return ExtractionOutcome::Disabled;
        };
        let answer = match llm.complete(&self.system_prompt(), &user).await {
            Ok(answer) => answer,
            Err(error) => return self.fail(owner_id, &error.to_string()),
        };
        let changes = match parse_patch(&answer) {
            Ok(changes) => changes,
            Err(error) => return self.fail(owner_id, &error.to_string()),
        };

        if changes.is_empty() {
            return self.skip(owner_id, SkipReason::NoChange);
        }

        // One change per document, each naming a document the extractor knows,
        // and each actually proposing something.
        let mut named: Vec<&str> = Vec::new();
        let mut invalid = false;
        for change in &changes {
            if !EXTRACTOR_DOCUMENTS.contains(&change.document.as_str())
                || named.contains(&change.document.as_str())
                || (change.edits.is_empty() && change.append.is_empty())
            {
                invalid = true;
                break;
            }
            named.push(&change.document);
        }
        if invalid {
            return self.skip(owner_id, SkipReason::InvalidChange);
        }

        let proposed = changes
            .iter()
            .map(ProposedChange::proposed)
            .collect::<Vec<_>>()
            .join("\n");
        if contains_sensitive_content(&proposed) {
            return self.skip(owner_id, SkipReason::Sensitive);
        }

        let boundary_broken = changes.iter().any(|change| {
            let additions = change.additions();
            if additions.is_empty() {
                return false;
            }
            if change.document == "user" {
                !belongs_to_user_preferences(&additions)
            } else {
                belongs_to_user_preferences(&additions)
            }
        });
        if boundary_broken {
            return self.skip(owner_id, SkipReason::DocumentBoundary);
        }

        if changes.iter().any(|change| change.document == "user")
            && !has_explicit_user_directive(&lines, locale)
        {
            return self.skip(owner_id, SkipReason::UserDirectiveNotExplicit);
        }

        let before: Vec<(String, Option<String>)> = changes
            .iter()
            .map(|change| {
                (
                    change.document.clone(),
                    existing
                        .iter()
                        .find(|document| document.scope == change.document)
                        .map(|document| document.revision.clone()),
                )
            })
            .collect();
        let prepared: Vec<MemoryChange> = changes
            .iter()
            .map(|change| MemoryChange {
                document: change.document.clone(),
                edits: change.edits.clone(),
                append: change.append.clone(),
                // The revision the model was shown. A concurrent realtime
                // `memory` call between the read and this write moves it, and
                // the write is refused rather than silently clobbering.
                expected_revision: existing
                    .iter()
                    .find(|document| document.scope == change.document)
                    .map(|document| document.revision.clone())
                    .unwrap_or_default(),
            })
            .collect();

        let edits: usize = prepared.iter().map(|change| change.edits.len()).sum();
        let appended = prepared.iter().any(|change| !change.append.is_empty());
        let documents: Vec<String> = prepared
            .iter()
            .map(|change| change.document.clone())
            .collect();

        match self.inner.memory_service.apply(owner_id, &prepared) {
            Ok(outcome) => {
                let mut before_revisions = Map::new();
                for (document, revision) in before {
                    before_revisions.insert(document, revision.map_or(Value::Null, Value::from));
                }
                let mut after_revisions = Map::new();
                for document in &outcome.documents {
                    after_revisions.insert(
                        document.scope.clone(),
                        Value::from(document.revision.clone()),
                    );
                }
                self.record(&AuditEvent::Patch {
                    op: "patch",
                    owner_id: owner_id.to_owned(),
                    documents: documents.clone(),
                    changed: outcome.changed,
                    before_revisions,
                    after_revisions,
                    edits,
                    appended,
                });
                tracing::debug!(
                    event = "memory.extract_completed",
                    documents = ?documents,
                    edits,
                    appended,
                    changed = outcome.changed,
                );
                ExtractionOutcome::Completed {
                    changed: outcome.changed,
                }
            }
            Err(error) => self.fail(owner_id, &error.to_string()),
        }
    }

    fn skip(&self, owner_id: &str, reason: SkipReason) -> ExtractionOutcome {
        self.record(&AuditEvent::skip(owner_id, reason));
        ExtractionOutcome::Skipped(reason)
    }

    fn fail(&self, owner_id: &str, error: &str) -> ExtractionOutcome {
        self.record(&AuditEvent::error(owner_id, error));
        ExtractionOutcome::Failed(error.to_owned())
    }

    fn record(&self, event: &AuditEvent) {
        if let Some(audit) = self.inner.audit.as_ref() {
            audit.record(event);
        }
    }
}

/// Test doubles, behind the `testing` feature.
///
/// `default = ["http", "testing"]` so this crate's own integration tests reach
/// them under a bare `cargo test -p via-conversation`, and a sibling that
/// depends on `via-conversation` need not restate the feature.
#[cfg(feature = "testing")]
pub mod testing {
    use super::{ExtractorError, ExtractorLlm, Message, TranscriptSource};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    /// An [`ExtractorLlm`] that answers from a script.
    ///
    /// Each call takes the next answer; the last one repeats once the script is
    /// exhausted, so a test that runs twice does not need to say the same thing
    /// twice.
    #[derive(Debug, Default)]
    pub struct ScriptedLlm {
        answers: Mutex<Vec<Result<String, String>>>,
        calls: Mutex<Vec<(String, String)>>,
    }

    impl ScriptedLlm {
        /// One answer, repeated.
        #[must_use]
        pub fn answering(answer: &str) -> Arc<Self> {
            Arc::new(Self {
                answers: Mutex::new(vec![Ok(answer.to_owned())]),
                calls: Mutex::new(Vec::new()),
            })
        }

        /// One transport failure, repeated.
        #[must_use]
        pub fn failing(error: &str) -> Arc<Self> {
            Arc::new(Self {
                answers: Mutex::new(vec![Err(error.to_owned())]),
                calls: Mutex::new(Vec::new()),
            })
        }

        /// A script of answers.
        #[must_use]
        pub fn scripted(answers: Vec<Result<String, String>>) -> Arc<Self> {
            Arc::new(Self {
                answers: Mutex::new(answers),
                calls: Mutex::new(Vec::new()),
            })
        }

        /// Every `(system, user)` pair the extractor sent.
        #[must_use]
        pub fn calls(&self) -> Vec<(String, String)> {
            self.calls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone()
        }

        /// How many times the model was called.
        #[must_use]
        pub fn call_count(&self) -> usize {
            self.calls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .len()
        }
    }

    #[async_trait]
    impl ExtractorLlm for ScriptedLlm {
        async fn complete(&self, system: &str, user: &str) -> Result<String, ExtractorError> {
            self.calls
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push((system.to_owned(), user.to_owned()));
            let mut answers = self
                .answers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let answer = if answers.len() > 1 {
                answers.remove(0)
            } else {
                answers
                    .first()
                    .cloned()
                    .unwrap_or_else(|| Ok(String::new()))
            };
            answer.map_err(ExtractorError::Transport)
        }
    }

    /// A [`TranscriptSource`] over a fixed transcript.
    #[derive(Debug, Default)]
    pub struct VecTranscripts {
        messages: Vec<Message>,
    }

    impl VecTranscripts {
        /// Wrap a transcript.
        #[must_use]
        pub fn new(messages: Vec<Message>) -> Arc<Self> {
            Arc::new(Self { messages })
        }
    }

    #[async_trait]
    impl TranscriptSource for VecTranscripts {
        async fn list(&self, _owner_id: &str, _session_id: &str) -> Vec<Message> {
            self.messages.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_json_object_survives_a_fence_and_trailing_prose() {
        assert_eq!(
            parse_patch("```json\n{\"changes\":[]}\n```").expect("fenced"),
            Vec::new()
        );
        assert_eq!(
            parse_patch("{\"changes\":[]}\n记忆整理完成。").expect("suffixed"),
            Vec::new()
        );
    }

    #[test]
    fn a_brace_inside_a_string_does_not_end_the_object() {
        assert_eq!(first_json_object(r#"{"a":"}"} tail"#), r#"{"a":"}"}"#);
        assert_eq!(first_json_object(r#"{"a":"\""} tail"#), r#"{"a":"\""}"#);
        assert_eq!(first_json_object("no braces"), "no braces");
        assert_eq!(first_json_object("{unbalanced"), "{unbalanced");
    }

    #[test]
    fn a_missing_changes_array_is_a_refusal_not_an_empty_patch() {
        assert_eq!(parse_patch("{}"), Err(PatchError::NoChangesArray));
        assert_eq!(
            parse_patch(r#"{"changes":{}}"#),
            Err(PatchError::NoChangesArray)
        );
        assert!(matches!(parse_patch("not json"), Err(PatchError::Json(_))));
    }

    #[test]
    fn a_patch_is_capped_at_two_changes_and_five_edits() {
        let payload = serde_json::json!({
            "changes": (0..4).map(|index| serde_json::json!({
                "document": format!("doc{index}"),
                "edits": (0..9).map(|edit| serde_json::json!({
                    "old_text": format!("old{edit}"),
                    "new_text": format!("new{edit}"),
                })).collect::<Vec<_>>(),
                "append": "",
            })).collect::<Vec<_>>(),
        })
        .to_string();
        let changes = parse_patch(&payload).expect("valid");
        assert_eq!(changes.len(), MAX_CHANGES_PER_RUN);
        assert_eq!(changes[0].edits.len(), MAX_OPS_PER_RUN);
    }

    #[test]
    fn clean_patch_bounds_by_code_point_and_folds_newlines() {
        assert_eq!(clean_patch("  a\r\nb  "), "a\nb");
        assert_eq!(clean_patch("a\0b"), "ab");
        assert_eq!(
            clean_patch(&"字".repeat(1200)).chars().count(),
            MAX_PATCH_CHARS
        );
    }

    #[test]
    fn the_boundary_classifier_reads_both_ways() {
        assert!(belongs_to_user_preferences("- 用户希望回答简洁一点"));
        assert!(belongs_to_user_preferences(
            "- 用户要求助手在每次回复开头加上“爱你哟”"
        ));
        assert!(!belongs_to_user_preferences("- 用户每天早上跑步"));
    }

    #[test]
    fn the_sensitive_gate_catches_shapes_as_well_as_words() {
        assert!(contains_sensitive_content("- 用户的密码是 123456"));
        assert!(contains_sensitive_content("api_key"));
        assert!(contains_sensitive_content("sk-abcdefgh12345678"));
        // A long digit run, e.g. a card or an identity number.
        assert!(contains_sensitive_content("12345678901"));
        assert!(!contains_sensitive_content("- 用户每天早上跑步"));
    }
}
