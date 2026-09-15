//! OpenClaw: the one backend that is not a plain ACP stdio child.
//!
//! Every other backend is a process VIA starts that speaks ACP on stdin and
//! stdout. OpenClaw is a *Gateway* — a long-lived service with its own agents,
//! sessions, channels and device pairing — and VIA reaches it through OpenClaw's
//! own ACP bridge. That makes two independent axes, and keeping them apart is
//! the whole of this module:
//!
//! | | Service ownership | ACP connection |
//! | --- | --- | --- |
//! | no `OPENCLAW_BASE_URL` | `owned` — VIA starts a Gateway with isolated runtime and session state | a local `process` |
//! | `OPENCLAW_BASE_URL` set | `external` — the user's Gateway, untouched | still a local `process` |
//!
//! `via-process`'s module docs state the same rule generically: *"a service
//! someone else published can still be reached through a locally spawned adapter
//! process. Collapsing them would make that shape unrepresentable, and it is the
//! normal shape for a bridged backend."*
//!
//! # What VIA does not do to an external Gateway
//!
//! It does not read its agent state, does not copy its configuration, and does
//! not write its credentials anywhere. [`isolated_config`] — the one path that
//! reads a user config at all — runs only for a VIA-owned Gateway, and it
//! *drops* the `channels` block before writing the isolated copy. Upstream's
//! reason, verbatim: *"A VIA-owned Gateway only provides the local ACP backend.
//! Reusing channels would connect the user's messaging accounts a second time."*
//!
//! # Ported from
//!
//! `server/src/agent/backends/openclaw.mjs`, `server/src/agent/openclaw-adapter.mjs`,
//! `server/src/process/backend-drivers/openclaw.mjs`, `openclaw-auth.mjs`,
//! `scripts/openclaw.mjs` and `config/openclaw/openclaw.json5`.

use std::path::{Path, PathBuf};

use via_catalog::backend_definition;
use via_core::EnvMap;
use via_core::config::names;
use via_core::runtime::{IfExists, write_file};
use via_i18n::{Locale, format, keys};

use crate::json5;
use crate::profile::{CoordinatorMeta, encode_uri_component};

/// The managed OpenClaw configuration, shipped verbatim.
///
/// **Copied byte for byte** from `config/openclaw/openclaw.json5`, with the
/// three identity renames `docs/rebrand.md` mandates and nothing else:
/// `${QWEN_AUDIO_AGENT_OPENCLAW_MODEL_ID}` → `${VIA_OPENCLAW_MODEL_ID}`,
/// `${QWEN_AUDIO_AGENT_OPENCLAW_MODEL}` → `${VIA_OPENCLAW_MODEL}`,
/// `${QWEN_AUDIO_AGENT_OPENCLAW_WORKSPACE}` → `${VIA_OPENCLAW_WORKSPACE}`,
/// plus the agent id `qwen-audio-agent-backend` → `via-backend` and its display
/// name. `${DASHSCOPE_API_KEY}` and `${OPENCLAW_GATEWAY_TOKEN}` are third-party
/// and stay. `docs/architecture.md` §14 lists this file under *"copied verbatim,
/// byte for byte, with a test"*; `tests/openclaw_config.rs` is that test.
///
/// The `${…}` references are expanded by **OpenClaw**, not by VIA: the file is
/// copied to disk and the variables are handed to the child in its environment.
pub const OPENCLAW_CONFIG_JSON5: &str = include_str!("../assets/openclaw/openclaw.json5");

/// The agent id VIA registers inside OpenClaw.
///
/// **External contract** — `state-name/OpenClaw agent id`
/// (`config/openclaw/openclaw.json5:32`), renamed per `docs/rebrand.md`
/// (`qwen-audio-agent-backend` → `via-backend`). OpenClaw persists it in its own
/// state directory, so it is also the value
/// [`via_core::config::backend::VIA_BACKEND_AGENT_ID`] carries.
pub const OPENCLAW_AGENT_ID: &str = via_core::config::backend::VIA_BACKEND_AGENT_ID;

/// The default OpenClaw Gateway port.
///
/// **External contract** — `default-value/OPENCLAW_PORT default`
/// (`scripts/openclaw.mjs:91`).
pub const DEFAULT_GATEWAY_PORT: &str = "18789";

/// The managed Gateway's argv.
///
/// **External contract** — `ws-event/OpenClaw managed gateway argv`:
/// `gateway run --port <OPENCLAW_PORT||18789> --bind loopback`. Loopback binding
/// is a security property, not a default — a Gateway bound to `0.0.0.0` with a
/// shared token is a remote-code-execution surface.
#[must_use]
pub fn managed_gateway_arguments(env: &EnvMap) -> Vec<String> {
    // The port tracks `OPENCLAW_BASE_URL` rather than falling back to an
    // unrelated constant — see `runtime::managed_service_port` for why, and for
    // why deriving still answers `18789` when nothing is overridden.
    // A placeholder `via-process` expands at spawn time — see
    // `runtime::managed_launch` for why the port cannot be baked here.
    let fallback = backend_definition("openclaw").map_or_else(
        || DEFAULT_GATEWAY_PORT.to_owned(),
        |definition| crate::runtime::managed_service_port(definition, env, DEFAULT_GATEWAY_PORT),
    );
    let port = format!("${{OPENCLAW_PORT:-{fallback}}}");
    vec![
        "gateway".to_owned(),
        "run".to_owned(),
        "--port".to_owned(),
        port,
        "--bind".to_owned(),
        "loopback".to_owned(),
    ]
}

/// The bundle-launcher noise stripped from surfaced process output.
///
/// **External contract** — `error-code/openclaw bridge diagnostics`
/// (`backends/openclaw.mjs:11-18`). The bundled launcher writes a banner on
/// every line of stderr; leaving it in makes an operator read a decorated log
/// instead of the actual error.
pub const BUNDLE_NOISE_MARKER: &str = "🦞 [openclaw-bundle]";

/// `sanitizeProcessOutput(value)` — `backends/openclaw.mjs:11-18`.
///
/// Split on `\r?\n`, drop every line containing the marker, rejoin with `\n`,
/// trim.
#[must_use]
pub fn sanitize_process_output(value: &str) -> String {
    value
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .filter(|line| !line.contains(BUNDLE_NOISE_MARKER))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

/// `formatRequestError({ details })` — `backends/openclaw.mjs:20-28`.
///
/// A JSON-RPC failure whose details name a missing OAuth scope is rewritten
/// into the one action that fixes it: approve the ACP device in OpenClaw.
/// Anything else returns `None`, which leaves the transport's own message
/// alone.
#[must_use]
pub fn format_request_error(details: &str, locale: Locale) -> Option<String> {
    let scope = missing_scope(details)?;
    Some(format(
        locale,
        keys::OPENCLAW_MISSING_DEVICE_SCOPE,
        &[("scope", &scope)],
    ))
}

/// `/\bmissing scope:\s*([a-z0-9._-]+)/i` — reproduced without a regex, because
/// the pattern is one literal prefix and one character class.
fn missing_scope(details: &str) -> Option<String> {
    let lowered = details.to_lowercase();
    let mut search = 0usize;
    while let Some(offset) = lowered[search..].find("missing scope:") {
        let start = search + offset;
        // `\b` before `missing`: the preceding character must not be a word
        // character.
        let boundary = start == 0
            || !lowered[..start]
                .chars()
                .next_back()
                .is_some_and(|previous| previous.is_alphanumeric() || previous == '_');
        if boundary {
            let rest = lowered[start + "missing scope:".len()..].trim_start_matches([' ', '\t']);
            let scope: String = rest
                .chars()
                .take_while(|value| {
                    value.is_ascii_lowercase()
                        || value.is_ascii_digit()
                        || matches!(value, '.' | '_' | '-')
                })
                .collect();
            if !scope.is_empty() {
                return Some(scope);
            }
        }
        search = start + "missing scope:".len();
    }
    None
}

/// The retry schedule for a conflicted coordinator prompt, in milliseconds.
///
/// **External contract** — `error-code/openclaw bridge diagnostics`
/// (`backends/openclaw.mjs:30-38`): `[150, 500, 1000]` indexed by attempt, and
/// **only** for OpenClaw's `reply session initialization conflicted for …`.
/// A different failure is not retried at all, because retrying an arbitrary
/// error against a live agent re-runs whatever it already did.
pub const PROMPT_RETRY_DELAYS_MS: [u64; 3] = [150, 500, 1000];

/// The upstream error text that alone justifies a retry.
pub const RETRYABLE_CONFLICT: &str = "reply session initialization conflicted for ";

/// `promptRetryDelay({ error, attempt })` — `backends/openclaw.mjs:30-38`.
///
/// `diagnostic` is upstream's `` `${error.message}\n${error.body}` ``. Returns
/// `None` when the failure is not the conflict, and when the attempts are
/// exhausted.
#[must_use]
pub fn prompt_retry_delay(diagnostic: &str, attempt: usize) -> Option<u64> {
    if !is_reply_conflict(diagnostic) {
        return None;
    }
    PROMPT_RETRY_DELAYS_MS.get(attempt).copied()
}

/// `/reply session initialization conflicted for \S+/i`.
fn is_reply_conflict(diagnostic: &str) -> bool {
    let lowered = diagnostic.to_lowercase();
    let mut search = 0usize;
    while let Some(offset) = lowered[search..].find(RETRYABLE_CONFLICT) {
        let start = search + offset + RETRYABLE_CONFLICT.len();
        // `\S+` — at least one non-whitespace character has to follow.
        if lowered[start..]
            .chars()
            .next()
            .is_some_and(|value| !value.is_whitespace())
        {
            return true;
        }
        search = start;
    }
    false
}

/// `coordinatorMeta(ownerId)` — `backends/openclaw.mjs:108-118`.
///
/// **External contract** — `state-name/openclaw coordinator _meta.sessionKey`:
/// `agent:<coordinatorAgent>:via:<owner>:backend`, where the owner is
/// percent-encoded and then the whole segment lower-cased. `None` when no
/// coordinator agent is configured, in which case `_meta` is omitted from
/// `session/new` and `session/resume` entirely.
///
/// The product segment carries the rebrand (`docs/rebrand.md`:
/// `agent:${coordinatorAgent}:qwen-audio-agent:${owner}:backend` →
/// `agent:${coordinatorAgent}:via:${owner}:backend`). OpenClaw persists this
/// string, so it decides which session VIA reattaches to after an upgrade.
#[must_use]
pub fn coordinator_meta(coordinator_agent: &str, owner_id: &str) -> Option<CoordinatorMeta> {
    let agent = coordinator_agent.trim();
    if agent.is_empty() {
        return None;
    }
    let owner = non_empty(owner_id.trim()).unwrap_or_else(|| DEFAULT_OWNER.to_owned());
    Some(CoordinatorMeta {
        session_key: std::format!(
            "agent:{agent}:{PRODUCT_SEGMENT}:{}:backend",
            encode_uri_component(&owner).to_lowercase()
        ),
    })
}

/// The owner segment used when no owner id is supplied.
///
/// **External contract** — `backends/openclaw.mjs:111`
/// (`clean(ownerId) || 'personal'`). Note this is the literal `personal`, not
/// [`via_downstream::DEFAULT_OWNER_ID`]: OpenClaw's key predates that constant
/// and changing it would orphan sessions.
pub const DEFAULT_OWNER: &str = "personal";

/// The product segment of the coordinator session key.
///
/// **Renamed** per `docs/rebrand.md` from `qwen-audio-agent`.
pub const PRODUCT_SEGMENT: &str = "via";

/// `websocketUrl(httpUrl)` — `backends/shared.mjs:52-56`.
///
/// The bridge's `--url`. Only the scheme changes; the path and query survive,
/// and trailing slashes are stripped. Distinct from [`gateway_websocket_url`],
/// which upstream also calls `websocketUrl` in a different file and which does
/// clear the path.
///
/// # Errors
///
/// [`url::ParseError`] when `http_url` is not a URL.
pub fn bridge_websocket_url(http_url: &str) -> Result<String, url::ParseError> {
    let mut parsed = url::Url::parse(http_url)?;
    let scheme = if matches!(parsed.scheme(), "https" | "wss") {
        "wss"
    } else {
        "ws"
    };
    // A scheme change between two special schemes always succeeds.
    let _ = parsed.set_scheme(scheme);
    Ok(parsed.as_str().trim_end_matches('/').to_owned())
}

/// `websocketUrl(baseUrl)` — `server/src/agent/openclaw-adapter.mjs:15-22`.
///
/// The Gateway RPC endpoint: scheme swapped, path reset to `/`, query and
/// fragment cleared. Unlike [`bridge_websocket_url`] this one keeps the
/// trailing slash, because `new URL(...).toString()` produces one for a
/// root path and upstream does not strip it here.
///
/// # Errors
///
/// [`url::ParseError`].
pub fn gateway_websocket_url(base_url: &str) -> Result<String, url::ParseError> {
    let mut parsed = url::Url::parse(base_url)?;
    let scheme = if parsed.scheme() == "https" {
        "wss"
    } else {
        "ws"
    };
    let _ = parsed.set_scheme(scheme);
    parsed.set_path("/");
    parsed.set_query(None);
    parsed.set_fragment(None);
    Ok(parsed.to_string())
}

/// `uiUrl({ baseUrl })` — `backends/openclaw.mjs:138-153`.
///
/// The dashboard, with the Gateway address (and the token, when there is one)
/// in the **fragment**. The fragment matters: a fragment is never sent to a
/// server, so the token does not reach an access log the way a query string
/// would.
///
/// # Errors
///
/// [`url::ParseError`].
pub fn dashboard_url(base_url: &str, token: &str) -> Result<String, url::ParseError> {
    let gateway = gateway_websocket_url(base_url)?;
    let mut settings = form_urlencoded_pair("gatewayUrl", &gateway);
    if !token.is_empty() {
        settings.push('&');
        settings.push_str(&form_urlencoded_pair("token", token));
    }
    let mut dashboard = url::Url::parse(base_url)?;
    dashboard.set_path("/");
    dashboard.set_query(None);
    dashboard.set_fragment(Some(&settings));
    Ok(dashboard.to_string())
}

/// One `application/x-www-form-urlencoded` pair, as `URLSearchParams` writes it.
fn form_urlencoded_pair(name: &str, value: &str) -> String {
    std::format!("{}={}", form_encode(name), form_encode(value))
}

/// `URLSearchParams` percent-encoding: space becomes `+`, and everything outside
/// `A-Za-z0-9*-._` is percent-encoded.
fn form_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b' ' => encoded.push('+'),
            b'*' | b'-' | b'.' | b'_' => encoded.push(*byte as char),
            _ if byte.is_ascii_alphanumeric() => encoded.push(*byte as char),
            _ => {
                use std::fmt::Write as _;
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

/// `openClawConfigPath({ env, homeDirectory })` —
/// `backend-drivers/openclaw-auth.mjs:34-44`.
///
/// `OPENCLAW_CONFIG_PATH` wins; otherwise `<OPENCLAW_STATE_DIR | ~/.openclaw>/openclaw.json`.
/// Both variables are OpenClaw's own (KEEP), so this resolves the **user's**
/// installation, not VIA's managed copy.
#[must_use]
pub fn openclaw_config_path(env: &EnvMap, home_directory: &str) -> Option<PathBuf> {
    if let Some(configured) = non_empty(env.get_trimmed(names::OPENCLAW_CONFIG_PATH)) {
        return Some(PathBuf::from(configured));
    }
    let state = non_empty(env.get_trimmed("OPENCLAW_STATE_DIR")).or_else(|| {
        non_empty(home_directory.trim()).map(|home| {
            PathBuf::from(home)
                .join(".openclaw")
                .to_string_lossy()
                .into_owned()
        })
    })?;
    Some(PathBuf::from(state).join("openclaw.json"))
}

/// `configuredToken(value, env)` — `backend-drivers/openclaw-auth.mjs:12-27`.
///
/// **External contract** — the three shapes OpenClaw admits for
/// `gateway.auth.token`:
///
/// 1. a literal string,
/// 2. a `"${ENV_NAME}"` reference,
/// 3. an object `{ source | provider: "env", id | name | key | env: <ENV_NAME> }`.
///
/// Anything else resolves to nothing.
#[must_use]
pub fn configured_token(value: &serde_json::Value, env: &EnvMap) -> String {
    if let Some(text) = value.as_str() {
        let trimmed = text.trim();
        return match environment_reference(trimmed) {
            Some(name) => env.get_trimmed(&name).to_owned(),
            None => trimmed.to_owned(),
        };
    }
    let Some(object) = value.as_object() else {
        return String::new();
    };
    let field = |name: &str| {
        object
            .get(name)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };
    let source = non_empty(&field("source"))
        .or_else(|| non_empty(&field("provider")))
        .unwrap_or_default()
        .to_lowercase();
    let name = ["id", "name", "key", "env"]
        .into_iter()
        .map(field)
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    if source == "env" && !name.is_empty() {
        env.get_trimmed(&name).to_owned()
    } else {
        String::new()
    }
}

/// `/^\$\{([A-Za-z_][A-Za-z0-9_]*)\}$/`.
fn environment_reference(value: &str) -> Option<String> {
    let inner = value.strip_prefix("${")?.strip_suffix('}')?;
    let mut characters = inner.chars();
    let first = characters.next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    if !characters.all(|value| value.is_ascii_alphanumeric() || value == '_') {
        return None;
    }
    Some(inner.to_owned())
}

/// `resolveOpenClawGatewayToken({ env, homeDirectory })` —
/// `backend-drivers/openclaw-auth.mjs:46-58`.
///
/// An explicit `OPENCLAW_GATEWAY_TOKEN` wins. Otherwise the user's own config is
/// read — JSON5, not JSON — and `gateway.auth.token` resolved through
/// [`configured_token`]. A missing or unparseable file is not an error: it means
/// there is no token to reuse.
#[must_use]
pub fn resolve_gateway_token(env: &EnvMap, home_directory: &str, source: Option<&str>) -> String {
    let explicit = env.get_trimmed(names::OPENCLAW_GATEWAY_TOKEN);
    if !explicit.is_empty() {
        return explicit.to_owned();
    }
    let Some(text) = source else {
        return String::new();
    };
    let _ = home_directory;
    let Some(parsed) = json5::parse(text) else {
        return String::new();
    };
    let token = parsed
        .get("gateway")
        .and_then(|gateway| gateway.get("auth"))
        .and_then(|auth| auth.get("token"));
    token.map_or_else(String::new, |value| configured_token(value, env))
}

/// `writeIsolatedOpenClawConfig({ sourcePath, targetPath })` —
/// `backend-drivers/openclaw-auth.mjs:75-97`.
///
/// Returns the JSON to write for a VIA-owned Gateway: the user's configuration
/// with `channels` removed. See the module docs for why that block in
/// particular.
///
/// `None` when the source is not parseable, which upstream signals by throwing
/// — here it means "there is nothing to isolate", and the caller falls back to
/// the shipped managed configuration.
#[must_use]
pub fn isolated_config(source: &str) -> Option<String> {
    let mut parsed = json5::parse(source)?;
    if let Some(object) = parsed.as_object_mut() {
        object.remove("channels");
    }
    // Upstream writes `JSON.stringify(parsed, null, 2)` plus a trailing newline.
    let rendered = serde_json::to_string_pretty(&parsed).ok()?;
    Some(std::format!("{rendered}\n"))
}

/// `writePrivateFile(path, contents)` — `backends/private-file.mjs`.
///
/// Parent directory at `0700`, file at `0600`, staged through a temporary name
/// and renamed into place. Delegated to `via-core`, which already performs
/// exactly this sequence through `via-store`'s atomic replace — this crate does
/// not hand-roll a second temp-and-rename.
///
/// # Errors
///
/// Any [`std::io::Error`] from the write.
pub fn write_private_file(path: &Path, contents: &str) -> std::io::Result<()> {
    write_file(path, contents, IfExists::Replace)
        .map(|_| ())
        .map_err(|error| std::io::Error::other(error.to_string()))
}

/// The token file's contents.
///
/// **External contract** — `env-var/OPENCLAW_GATEWAY_TOKEN …`: written as
/// `` `${token}\n` ``. *"The token file is read by the OpenClaw process; the
/// trailing newline is part of the format."*
#[must_use]
pub fn token_file_contents(token: &str) -> String {
    std::format!("{token}\n")
}

fn non_empty(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn bundle_banner_lines_are_stripped_from_surfaced_output() {
        let raw = "starting\r\n🦞 [openclaw-bundle] resolving\nready\n";
        assert_eq!(sanitize_process_output(raw), "starting\nready");
        assert_eq!(sanitize_process_output("🦞 [openclaw-bundle] only"), "");
    }

    #[test]
    fn a_missing_scope_becomes_an_actionable_sentence() {
        assert_eq!(
            format_request_error("error: missing scope: acp.device.pair", Locale::Zh),
            Some("需要在 OpenClaw 中批准 ACP 设备权限（缺少 acp.device.pair）".to_owned())
        );
        assert_eq!(
            format_request_error("Missing Scope:   operator.admin", Locale::En),
            Some(
                "approve the ACP device permission in OpenClaw (missing operator.admin)".to_owned()
            )
        );
        assert_eq!(format_request_error("connection refused", Locale::En), None);
        // `\b` before `missing`: `dismissing scope:` is not a scope error.
        assert_eq!(
            format_request_error("dismissing scope: x", Locale::En),
            None
        );
    }

    #[test]
    fn only_the_conflict_is_retried_and_only_three_times() {
        let conflict = "reply session initialization conflicted for agent:x:via:y:backend";
        assert_eq!(prompt_retry_delay(conflict, 0), Some(150));
        assert_eq!(prompt_retry_delay(conflict, 1), Some(500));
        assert_eq!(prompt_retry_delay(conflict, 2), Some(1000));
        assert_eq!(prompt_retry_delay(conflict, 3), None);
        assert_eq!(prompt_retry_delay("some other failure", 0), None);
        // `\S+` — the message must actually name a session.
        assert_eq!(
            prompt_retry_delay("reply session initialization conflicted for ", 0),
            None
        );
    }

    #[test]
    fn the_coordinator_session_key_is_the_catalogued_shape() {
        assert_eq!(
            coordinator_meta("voice-coordinator", "owner one"),
            Some(CoordinatorMeta {
                session_key: "agent:voice-coordinator:via:owner%20one:backend".to_owned(),
            })
        );
        assert_eq!(
            coordinator_meta("via-backend", "").map(|meta| meta.session_key),
            Some("agent:via-backend:via:personal:backend".to_owned())
        );
        assert_eq!(coordinator_meta("", "owner"), None);
        assert_eq!(coordinator_meta("   ", "owner"), None);
    }

    #[test]
    fn the_encoded_owner_segment_is_lowercased() {
        // `.toLowerCase()` runs *after* `encodeURIComponent`, so the percent
        // escapes are lowercased too.
        assert_eq!(
            coordinator_meta("a", "Érin").map(|meta| meta.session_key),
            Some("agent:a:via:%c3%89rin:backend".to_owned())
        );
    }

    #[test]
    fn the_bridge_url_keeps_its_path_and_the_gateway_url_does_not() {
        assert_eq!(
            bridge_websocket_url("https://openclaw.example.test/base/").expect("valid"),
            "wss://openclaw.example.test/base"
        );
        assert_eq!(
            bridge_websocket_url("http://127.0.0.1:18789").expect("valid"),
            "ws://127.0.0.1:18789"
        );
        assert_eq!(
            gateway_websocket_url("https://openclaw.example.test/base?x=1#y").expect("valid"),
            "wss://openclaw.example.test/"
        );
    }

    #[test]
    fn the_dashboard_carries_the_token_in_the_fragment_only() {
        let url = dashboard_url("http://127.0.0.1:18789", "s3cret").expect("valid");
        assert!(url.starts_with("http://127.0.0.1:18789/#"), "{url}");
        assert!(
            url.contains("gatewayUrl=ws%3A%2F%2F127.0.0.1%3A18789%2F"),
            "{url}"
        );
        assert!(url.contains("token=s3cret"), "{url}");
        assert!(!url.contains('?'), "a token must not reach a query string");

        let anonymous = dashboard_url("http://127.0.0.1:18789", "").expect("valid");
        assert!(!anonymous.contains("token="));
    }

    #[test]
    fn the_managed_gateway_binds_loopback() {
        assert_eq!(
            managed_gateway_arguments(&EnvMap::new()),
            [
                "gateway",
                "run",
                "--port",
                "${OPENCLAW_PORT:-18789}",
                "--bind",
                "loopback"
            ]
        );
        assert_eq!(
            managed_gateway_arguments(&env(&[("OPENCLAW_PORT", "19000")]))[3],
            "${OPENCLAW_PORT:-19000}"
        );
    }

    /// `env-var/backend env derived from base URL` — `cli/src/runtime.mjs:282-291`.
    ///
    /// The argv's port tracks `OPENCLAW_BASE_URL`. Before it did, a custom base
    /// URL with no matching `OPENCLAW_PORT` left VIA routing to the custom port
    /// while the Gateway it started listened on `18789`.
    #[test]
    fn the_managed_gateway_port_tracks_a_custom_base_url() {
        assert_eq!(
            managed_gateway_arguments(&env(&[("OPENCLAW_BASE_URL", "http://127.0.0.1:9999")]))[3],
            "${OPENCLAW_PORT:-9999}",
            "the fallback tracks the base URL; the placeholder is expanded at \
             spawn time from the child's own environment",
        );
        // An explicit port still wins over the base URL.
        assert_eq!(
            managed_gateway_arguments(&env(&[
                ("OPENCLAW_BASE_URL", "http://127.0.0.1:9999"),
                ("OPENCLAW_PORT", "19000"),
            ]))[3],
            "${OPENCLAW_PORT:-19000}",
        );
        // A scheme with no explicit port takes the scheme's default.
        assert_eq!(
            managed_gateway_arguments(&env(&[("OPENCLAW_BASE_URL", "wss://gateway.example")]))[3],
            "${OPENCLAW_PORT:-443}",
        );
    }

    #[test]
    fn the_config_path_prefers_the_explicit_override() {
        assert_eq!(
            openclaw_config_path(&env(&[("OPENCLAW_CONFIG_PATH", "/tmp/x.json")]), "/home/u"),
            Some(PathBuf::from("/tmp/x.json"))
        );
        assert_eq!(
            openclaw_config_path(&env(&[("OPENCLAW_STATE_DIR", "/state")]), "/home/u"),
            Some(PathBuf::from("/state/openclaw.json"))
        );
        assert_eq!(
            openclaw_config_path(&EnvMap::new(), "/home/u"),
            Some(PathBuf::from("/home/u/.openclaw/openclaw.json"))
        );
        assert_eq!(openclaw_config_path(&EnvMap::new(), ""), None);
    }

    #[test]
    fn all_three_token_reference_shapes_resolve() {
        let environment = env(&[("SOME_TOKEN", "from-env")]);
        assert_eq!(
            configured_token(&serde_json::json!("literal"), &environment),
            "literal"
        );
        assert_eq!(
            configured_token(&serde_json::json!("${SOME_TOKEN}"), &environment),
            "from-env"
        );
        assert_eq!(
            configured_token(
                &serde_json::json!({ "source": "env", "id": "SOME_TOKEN" }),
                &environment
            ),
            "from-env"
        );
        assert_eq!(
            configured_token(
                &serde_json::json!({ "provider": "ENV", "env": "SOME_TOKEN" }),
                &environment
            ),
            "from-env"
        );
        // A non-env source, an array, and a null all resolve to nothing.
        assert_eq!(
            configured_token(
                &serde_json::json!({ "source": "file", "id": "SOME_TOKEN" }),
                &environment
            ),
            ""
        );
        assert_eq!(
            configured_token(&serde_json::json!([1, 2]), &environment),
            ""
        );
        assert_eq!(configured_token(&serde_json::Value::Null, &environment), "");
    }

    #[test]
    fn an_explicit_token_wins_over_the_user_config() {
        let environment = env(&[("OPENCLAW_GATEWAY_TOKEN", "explicit")]);
        assert_eq!(
            resolve_gateway_token(
                &environment,
                "/home/u",
                Some("{ gateway: { auth: { token: 'from-file' } } }")
            ),
            "explicit"
        );
    }

    #[test]
    fn the_user_config_is_read_as_json5() {
        // Comments, unquoted keys, single quotes and a trailing comma: all
        // JSON5, none of them JSON.
        let source = r#"{
          // the gateway block
          gateway: {
            mode: 'local',
            auth: { mode: 'token', token: '${SOME_TOKEN}' },
          },
        }"#;
        assert_eq!(
            resolve_gateway_token(&env(&[("SOME_TOKEN", "resolved")]), "/home/u", Some(source)),
            "resolved"
        );
        assert_eq!(resolve_gateway_token(&EnvMap::new(), "", None), "");
        assert_eq!(
            resolve_gateway_token(&EnvMap::new(), "", Some("not a config")),
            ""
        );
    }

    #[test]
    fn isolating_a_config_drops_the_channels_block() {
        let source = r#"{ channels: { slack: { token: 'x' } }, gateway: { mode: 'local' } }"#;
        let isolated = isolated_config(source).expect("parses");
        assert!(!isolated.contains("channels"), "{isolated}");
        assert!(isolated.contains("gateway"));
        assert!(isolated.ends_with('\n'), "upstream appends a newline");
        assert!(isolated_config("{{{").is_none());
    }

    #[test]
    fn the_token_file_keeps_its_trailing_newline() {
        assert_eq!(token_file_contents("abc"), "abc\n");
    }
}
