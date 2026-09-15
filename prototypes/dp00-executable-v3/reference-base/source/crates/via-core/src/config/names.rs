//! Every environment variable `via-core` reads, as a named constant.
//!
//! The names are an external contract in both directions: an operator types
//! them, and `docs/reference/contracts.json` catalogues them. They are constants
//! rather than inline literals so `via-conformance` can assert the rename was
//! applied to all of them **atomically** — a half-applied rename is the failure
//! `docs/architecture.md` §13 warns about, and it is invisible at any single
//! call site.
//!
//! # Rebrand
//!
//! Three upstream prefixes collapse into one (`docs/rebrand.md`):
//! `QWEN_AUDIO_AGENT_*`, `QWEN_AUDIO_*` and `QWAUDIO_*` all become `VIA_*`.
//! Each constant's doc comment names its upstream spelling.
//!
//! **Nothing else is renamed.** `DASHSCOPE_*`, `SPEECH_TO_SPEECH_*`, `S2S_*`,
//! `AGENT_PROTOCOL`, `AGENT_TIMEOUT_MS`, `AGENT_API_KEY`, `OPENCODE_*`,
//! `OPENCLAW_*`, `QODER*`, `QWEN_CODE_*`, `KIMI_*`, `HERMES_*`, `CODEBUDDY_*`,
//! `CODEX_*`, `CLAUDE_*`, `DEEPSEEK_*`, `PI_*`, `ACP_*`, `HOST` and `PORT` name
//! someone else's product or a generic convention and are KEEP. Renaming any of
//! them breaks an integration VIA does not own.

// ── directories and roots ───────────────────────────────────────────────────

/// Upstream `QWEN_AUDIO_AGENT_RUNTIME_ROOT`. The base every relative path
/// override resolves against.
pub const RUNTIME_ROOT: &str = "VIA_RUNTIME_ROOT";

/// Upstream `QWAUDIO_CONFIG_DIR`. See [`crate::paths`].
pub const CONFIG_DIR: &str = crate::paths::ENV_CONFIG_DIR;

/// Upstream `QWAUDIO_DATA_DIR`. See [`crate::paths`].
pub const DATA_DIR: &str = crate::paths::ENV_DATA_DIR;

// ── HTTP server ─────────────────────────────────────────────────────────────

/// KEEP. Listen address. Loopback-only by default, which is what makes the
/// implicit same-origin path in [`crate::security`] safe.
pub const HOST: &str = "HOST";

/// KEEP. Listen port. `0` means "let the OS choose".
pub const PORT: &str = "PORT";

// ── realtime frontend ───────────────────────────────────────────────────────

/// Upstream `QWEN_AUDIO_REALTIME_PROVIDER`.
pub const REALTIME_PROVIDER: &str = "VIA_REALTIME_PROVIDER";
/// Upstream `QWEN_AUDIO_REALTIME_API_KEY`.
pub const REALTIME_API_KEY: &str = "VIA_REALTIME_API_KEY";
/// Upstream `QWEN_AUDIO_REALTIME_BASE_URL`.
pub const REALTIME_BASE_URL: &str = "VIA_REALTIME_BASE_URL";
/// Upstream `QWEN_AUDIO_REALTIME_URL`, the legacy spelling of
/// [`REALTIME_BASE_URL`].
pub const REALTIME_URL: &str = "VIA_REALTIME_URL";
/// Upstream `QWEN_AUDIO_REALTIME_MODEL`.
pub const REALTIME_MODEL: &str = "VIA_REALTIME_MODEL";
/// Upstream `QWEN_AUDIO_REALTIME_VOICE`. Applies **only** to the `audio` model
/// family; see [`via_catalog::ModelFamily::voice_override_env`].
pub const REALTIME_VOICE: &str = via_catalog::realtime_model::AUDIO_FAMILY_VOICE_ENV;
/// Upstream `QWEN_OMNI_REALTIME_VOICE`. Applies **only** to the `omni` family.
pub const OMNI_REALTIME_VOICE: &str = via_catalog::realtime_model::OMNI_FAMILY_VOICE_ENV;
/// VIA-owned. Applies only to the `local` family, which has no upstream peer.
pub const LOCAL_REALTIME_VOICE: &str = via_catalog::realtime_model::LOCAL_FAMILY_VOICE_ENV;

/// KEEP — Alibaba Cloud's own credential variable.
pub const DASHSCOPE_API_KEY: &str = "DASHSCOPE_API_KEY";
/// KEEP — Alibaba Cloud Model Studio workspace id; rewrites the endpoint host.
pub const DASHSCOPE_WORKSPACE_ID: &str = "DASHSCOPE_WORKSPACE_ID";
/// KEEP — the huggingface/speech-to-speech endpoint.
pub const SPEECH_TO_SPEECH_REALTIME_URL: &str = "SPEECH_TO_SPEECH_REALTIME_URL";
/// KEEP — alias of [`SPEECH_TO_SPEECH_REALTIME_URL`].
pub const S2S_REALTIME_URL: &str = "S2S_REALTIME_URL";
/// KEEP — optional bearer token for a proxied speech-to-speech service.
pub const SPEECH_TO_SPEECH_AUTH_TOKEN: &str = "SPEECH_TO_SPEECH_AUTH_TOKEN";
/// KEEP — alias of [`SPEECH_TO_SPEECH_AUTH_TOKEN`].
pub const S2S_API_KEY: &str = "S2S_API_KEY";

// ── security and identity ───────────────────────────────────────────────────

/// Upstream `QWEN_AUDIO_AGENT_ALLOWED_ORIGINS`. Comma separated; empty by
/// default, and never `*`.
pub const ALLOWED_ORIGINS: &str = "VIA_ALLOWED_ORIGINS";
/// Upstream `QWEN_AUDIO_AGENT_AUTH_SECRET`. Generated into `state.env` on first
/// run and **never** forwarded to a backend process.
pub const AUTH_SECRET: &str = "VIA_AUTH_SECRET";
/// Upstream `QWEN_AUDIO_AGENT_IDENTITY_MODE`. `personal` or `browser`.
pub const IDENTITY_MODE: &str = "VIA_IDENTITY_MODE";
/// Upstream `QWEN_AUDIO_AGENT_PERSONAL_OWNER_ID`.
pub const PERSONAL_OWNER_ID: &str = "VIA_PERSONAL_OWNER_ID";

// ── setup gate, locale and gateway identity ─────────────────────────────────

/// Upstream `QWEN_AUDIO_ALLOW_UNCONFIGURED`. Compared to the exact string `1`.
pub const ALLOW_UNCONFIGURED: &str = "VIA_ALLOW_UNCONFIGURED";
/// Upstream `QWEN_AUDIO_GATEWAY_OWNER`. `cli`, `desktop` or `service`.
pub const GATEWAY_OWNER: &str = "VIA_GATEWAY_OWNER";
/// VIA-owned, read by `via-i18n`. Named here so the config surface is complete.
pub const LOCALE: &str = "VIA_LOCALE";

// ── backend selection ───────────────────────────────────────────────────────

/// KEEP — the backend agent id, or empty for frontend-only.
pub const AGENT_PROTOCOL: &str = "AGENT_PROTOCOL";
/// KEEP — backend request timeout.
pub const AGENT_TIMEOUT_MS: &str = "AGENT_TIMEOUT_MS";
/// Upstream `QWEN_AUDIO_AGENT_BACKEND_PERMISSION_MODE`. `native` or `full`.
pub const BACKEND_PERMISSION_MODE: &str = "VIA_BACKEND_PERMISSION_MODE";
/// Upstream `QWEN_AUDIO_AGENT_BACKEND_OWNERSHIP`. Empty, `owned` or `external`.
pub const BACKEND_OWNERSHIP: &str = "VIA_BACKEND_OWNERSHIP";
/// Upstream `QWEN_AUDIO_AGENT_BACKEND_MODEL`.
pub const BACKEND_MODEL: &str = "VIA_BACKEND_MODEL";
/// Upstream `QWEN_AUDIO_AGENT_BACKEND_AGENT`.
pub const BACKEND_AGENT: &str = "VIA_BACKEND_AGENT";
/// Upstream `QWEN_AUDIO_AGENT_COMPUTER_USE`. Default-**on** feature toggle.
pub const COMPUTER_USE: &str = "VIA_COMPUTER_USE";

// ── per-backend options (all KEEP: third-party namespaces) ──────────────────

/// KEEP.
pub const OPENCODE_BASE_URL: &str = "OPENCODE_BASE_URL";
/// KEEP.
pub const OPENCODE_COORDINATOR_AGENT: &str = "OPENCODE_COORDINATOR_AGENT";
/// KEEP.
pub const OPENCLAW_BASE_URL: &str = "OPENCLAW_BASE_URL";
/// KEEP.
pub const OPENCLAW_GATEWAY_TOKEN: &str = "OPENCLAW_GATEWAY_TOKEN";
/// KEEP.
pub const OPENCLAW_GATEWAY_TOKEN_FILE: &str = "OPENCLAW_GATEWAY_TOKEN_FILE";
/// KEEP.
pub const OPENCLAW_ACP_BIN: &str = "OPENCLAW_ACP_BIN";
/// KEEP.
pub const OPENCLAW_COORDINATOR_AGENT: &str = "OPENCLAW_COORDINATOR_AGENT";
/// KEEP.
pub const OPENCLAW_CONFIG_PATH: &str = "OPENCLAW_CONFIG_PATH";
/// KEEP — OpenClaw's own credential, and the fallback for
/// [`OPENCLAW_GATEWAY_TOKEN`].
pub const AGENT_API_KEY: &str = "AGENT_API_KEY";
/// Upstream `QWEN_AUDIO_AGENT_OPENCLAW_STATE_DIR`.
pub const OPENCLAW_STATE_DIR: &str = "VIA_OPENCLAW_STATE_DIR";
/// KEEP.
pub const QODERCLI_PATH: &str = "QODERCLI_PATH";
/// KEEP — legacy alias of [`QODERCLI_PATH`].
pub const QODER_CLI_PATH: &str = "QODER_CLI_PATH";
/// KEEP.
pub const QODER_CONFIG_DIR: &str = "QODER_CONFIG_DIR";
/// KEEP — the Qwen Code CLI's own executable override.
pub const QWEN_CODE_BIN: &str = "QWEN_CODE_BIN";
/// KEEP.
pub const KIMI_CODE_BIN: &str = "KIMI_CODE_BIN";
/// KEEP.
pub const HERMES_BIN: &str = "HERMES_BIN";
/// KEEP.
pub const CODEBUDDY_BIN: &str = "CODEBUDDY_BIN";
/// KEEP.
pub const CODEBUDDY_MODEL_URL: &str = "CODEBUDDY_MODEL_URL";
/// KEEP.
pub const CODEX_ACP_BIN: &str = "CODEX_ACP_BIN";
/// KEEP.
pub const CODEX_BASE_URL: &str = "CODEX_BASE_URL";
/// KEEP.
pub const CLAUDE_CODE_ACP_BIN: &str = "CLAUDE_CODE_ACP_BIN";
/// KEEP.
pub const CLAUDE_CODE_EXECUTABLE: &str = "CLAUDE_CODE_EXECUTABLE";
/// KEEP.
pub const CLAUDE_CONFIG_DIR: &str = "CLAUDE_CONFIG_DIR";
/// KEEP.
pub const DEEPSEEK_HARNESS_ACP_BIN: &str = "DEEPSEEK_HARNESS_ACP_BIN";
/// KEEP — DeepSeek's own model override, which wins over
/// [`BACKEND_MODEL`].
pub const DEEPSEEK_HARNESS_MODEL: &str = "DEEPSEEK_HARNESS_MODEL";
/// KEEP.
pub const PI_ACP_BIN: &str = "PI_ACP_BIN";
/// KEEP.
pub const ACP_COMMAND: &str = "ACP_COMMAND";
/// KEEP.
pub const ACP_ARGS: &str = "ACP_ARGS";
/// KEEP.
pub const ACP_LABEL: &str = "ACP_LABEL";
/// KEEP.
pub const ACP_COORDINATOR_AGENT: &str = "ACP_COORDINATOR_AGENT";

// ── announcement engine ─────────────────────────────────────────────────────

/// Upstream `QWEN_AUDIO_AGENT_ANNOUNCE_INTO_CONTEXT`.
pub const ANNOUNCE_INTO_CONTEXT: &str = "VIA_ANNOUNCE_INTO_CONTEXT";
/// Upstream `QWEN_AUDIO_AGENT_RESULT_CONTEXT_MAX_CHARS`.
pub const RESULT_CONTEXT_MAX_CHARS: &str = "VIA_RESULT_CONTEXT_MAX_CHARS";
/// Upstream `QWEN_AUDIO_AGENT_ANNOUNCEMENT_BATCH_MS`.
pub const ANNOUNCEMENT_BATCH_MS: &str = "VIA_ANNOUNCEMENT_BATCH_MS";
/// Upstream `QWEN_AUDIO_AGENT_ANNOUNCEMENT_MAX_BATCH_ITEMS`.
pub const ANNOUNCEMENT_MAX_BATCH_ITEMS: &str = "VIA_ANNOUNCEMENT_MAX_BATCH_ITEMS";
/// Upstream `QWEN_AUDIO_AGENT_ANNOUNCEMENT_QUIET_MS`.
pub const ANNOUNCEMENT_QUIET_MS: &str = "VIA_ANNOUNCEMENT_QUIET_MS";
/// Upstream `QWEN_AUDIO_AGENT_ANNOUNCEMENT_ACK_TIMEOUT_MS`.
pub const ANNOUNCEMENT_ACK_TIMEOUT_MS: &str = "VIA_ANNOUNCEMENT_ACK_TIMEOUT_MS";
/// Upstream `QWEN_AUDIO_AGENT_ANNOUNCEMENT_MAX_RETRIES`.
pub const ANNOUNCEMENT_MAX_RETRIES: &str = "VIA_ANNOUNCEMENT_MAX_RETRIES";

// ── asset path overrides ────────────────────────────────────────────────────

/// Upstream `QWEN_AUDIO_AGENT_FRONTEND_PROMPT_DIR`.
pub const FRONTEND_PROMPT_DIR: &str = "VIA_FRONTEND_PROMPT_DIR";
/// Upstream `QWEN_AUDIO_AGENT_ASSISTANT_PROFILE_PATH`.
pub const ASSISTANT_PROFILE_PATH: &str = "VIA_ASSISTANT_PROFILE_PATH";
/// Upstream `QWEN_AUDIO_AGENT_MEMORY_PATH`. Wins over
/// [`FRONTEND_MEMORY_PATH`].
pub const MEMORY_PATH: &str = "VIA_MEMORY_PATH";
/// Upstream `QWEN_AUDIO_AGENT_FRONTEND_MEMORY_PATH`. Legacy spelling, kept for
/// compatibility.
pub const FRONTEND_MEMORY_PATH: &str = "VIA_FRONTEND_MEMORY_PATH";
/// Upstream `QWEN_AUDIO_AGENT_FRONTEND_NOTES_PATH`.
pub const FRONTEND_NOTES_PATH: &str = "VIA_FRONTEND_NOTES_PATH";
/// Upstream `QWEN_AUDIO_AGENT_USER_MODEL_PATH`. Wins over
/// [`USER_PROFILE_PATH`].
pub const USER_MODEL_PATH: &str = "VIA_USER_MODEL_PATH";
/// Upstream `QWEN_AUDIO_AGENT_USER_PROFILE_PATH`. Legacy spelling.
pub const USER_PROFILE_PATH: &str = "VIA_USER_PROFILE_PATH";
/// Upstream `QWEN_AUDIO_AGENT_TASK_STATE_PATH`.
pub const TASK_STATE_PATH: &str = "VIA_TASK_STATE_PATH";
/// Upstream `QWEN_AUDIO_AGENT_BACKEND_SESSION_STATE_PATH`.
pub const BACKEND_SESSION_STATE_PATH: &str = "VIA_BACKEND_SESSION_STATE_PATH";

// ── Work and session retention ──────────────────────────────────────────────

/// Upstream `QWEN_AUDIO_AGENT_TASK_TERMINAL_TTL_MS`.
pub const TASK_TERMINAL_TTL_MS: &str = "VIA_TASK_TERMINAL_TTL_MS";
/// Upstream `QWEN_AUDIO_AGENT_TASK_NOTIFICATION_TTL_MS`.
pub const TASK_NOTIFICATION_TTL_MS: &str = "VIA_TASK_NOTIFICATION_TTL_MS";
/// Upstream `QWEN_AUDIO_AGENT_TASK_NOTIFICATION_CLAIM_TTL_MS`.
pub const TASK_NOTIFICATION_CLAIM_TTL_MS: &str = "VIA_TASK_NOTIFICATION_CLAIM_TTL_MS";
/// Upstream `QWEN_AUDIO_AGENT_MAX_TERMINAL_TASKS_PER_OWNER`.
pub const MAX_TERMINAL_TASKS_PER_OWNER: &str = "VIA_MAX_TERMINAL_TASKS_PER_OWNER";
/// Upstream `QWEN_AUDIO_AGENT_TASK_MAX_CONCURRENT`.
pub const TASK_MAX_CONCURRENT: &str = "VIA_TASK_MAX_CONCURRENT";
/// Upstream `QWEN_AUDIO_AGENT_TASK_MAX_CONCURRENT_PER_OWNER`.
pub const TASK_MAX_CONCURRENT_PER_OWNER: &str = "VIA_TASK_MAX_CONCURRENT_PER_OWNER";
/// Upstream `QWEN_AUDIO_AGENT_SESSION_TTL_MS`.
pub const SESSION_TTL_MS: &str = "VIA_SESSION_TTL_MS";
/// Upstream `QWEN_AUDIO_AGENT_MAX_SESSIONS`.
pub const MAX_SESSIONS: &str = "VIA_MAX_SESSIONS";
/// Upstream `QWEN_AUDIO_AGENT_MEMORY_OWNER_TTL_MS`. Zero keeps memories
/// forever.
pub const MEMORY_OWNER_TTL_MS: &str = "VIA_MEMORY_OWNER_TTL_MS";
/// Upstream `QWEN_AUDIO_AGENT_MAX_MEMORY_OWNERS`.
pub const MAX_MEMORY_OWNERS: &str = "VIA_MAX_MEMORY_OWNERS";

// ── memory extractor ────────────────────────────────────────────────────────

/// Upstream `QWEN_AUDIO_MEMORY_AUTO`. Disabled only by the literal `off`.
pub const MEMORY_AUTO: &str = "VIA_MEMORY_AUTO";
/// Upstream `QWEN_AUDIO_MEMORY_MODEL`.
pub const MEMORY_MODEL: &str = "VIA_MEMORY_MODEL";
/// Upstream `QWEN_AUDIO_MEMORY_BASE_URL`.
pub const MEMORY_BASE_URL: &str = "VIA_MEMORY_BASE_URL";
/// Upstream `QWEN_AUDIO_MEMORY_API_KEY`.
pub const MEMORY_API_KEY: &str = "VIA_MEMORY_API_KEY";

// ── scheduler, sleep and wake word ──────────────────────────────────────────

/// Upstream `QWEN_AUDIO_AGENT_REMINDER_SCHEDULER`.
pub const REMINDER_SCHEDULER: &str = "VIA_REMINDER_SCHEDULER";
/// Upstream `QWEN_AUDIO_AGENT_REMINDER_MAX_PER_OWNER`.
pub const REMINDER_MAX_PER_OWNER: &str = "VIA_REMINDER_MAX_PER_OWNER";
/// Upstream `QWEN_AUDIO_AGENT_SCHEDULED_TASK_TIMEOUT_MS`.
pub const SCHEDULED_TASK_TIMEOUT_MS: &str = "VIA_SCHEDULED_TASK_TIMEOUT_MS";
/// Upstream `QWEN_AUDIO_AGENT_BACKGROUND_TASK_PROGRESS_CHECK_MS`.
pub const BACKGROUND_TASK_PROGRESS_CHECK_MS: &str = "VIA_BACKGROUND_TASK_PROGRESS_CHECK_MS";
/// Upstream `QWEN_AUDIO_AGENT_SCHEDULED_TASK_PROGRESS_CHECK_MS`, the legacy
/// spelling of [`BACKGROUND_TASK_PROGRESS_CHECK_MS`].
pub const SCHEDULED_TASK_PROGRESS_CHECK_MS: &str = "VIA_SCHEDULED_TASK_PROGRESS_CHECK_MS";
/// Upstream `QWEN_AUDIO_AGENT_OFFLINE_NOTIFICATION_DELAY_MS`.
pub const OFFLINE_NOTIFICATION_DELAY_MS: &str = "VIA_OFFLINE_NOTIFICATION_DELAY_MS";
/// Upstream `QWEN_AUDIO_AGENT_REMINDER_STAGGER_MS`.
pub const REMINDER_STAGGER_MS: &str = "VIA_REMINDER_STAGGER_MS";

/// Upstream `QWEN_AUDIO_DESKTOP_AUTO_HIDE_SECONDS`.
///
/// `docs/rebrand.md` gives three spellings for this one variable —
/// `VIA_SLEEP_TIMEOUT_SECONDS` (row 133), `VIA_DESKTOP_AUTO_HIDE_SECONDS`
/// (row 134) and `VIA_AUTO_HIDE_SECONDS` (row 90). VIA takes row 133's, the
/// most narrowly scoped of the three: it is the only one that still makes sense
/// once `desktop/` is dropped (`docs/fidelity.md`), because there is no orb to
/// auto-hide. Recorded in `docs/deviations/phase-1.md`.
///
/// Upstream's legacy `QWEN_AUDIO_SLEEP_TIMEOUT_SECONDS` is deliberately ignored
/// there; VIA has no legacy name to ignore, so no alias is implemented.
pub const SLEEP_TIMEOUT_SECONDS: &str = "VIA_SLEEP_TIMEOUT_SECONDS";

/// Upstream `QWEN_AUDIO_WAKE_WORD_ENABLED`.
pub const WAKE_WORD_ENABLED: &str = "VIA_WAKE_WORD_ENABLED";
/// Upstream `QWEN_AUDIO_WAKE_WORD_MODEL_DIR`.
pub const WAKE_WORD_MODEL_DIR: &str = "VIA_WAKE_WORD_MODEL_DIR";

/// VIA-owned: the wake phrase.
///
/// Upstream hard-codes `你好千问` with no override
/// (`server/src/core/config.mjs:503`). `docs/architecture.md` §16 makes the
/// phrase "a configuration value, never a literal" and defers the choice to
/// phase 6, so VIA reads it from the environment and defaults to empty.
pub const WAKE_WORD: &str = "VIA_WAKE_WORD";
