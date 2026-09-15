//! OpenCode's two behaviours beyond a launch spec.
//!
//! Ported from `server/src/agent/backends/opencode.mjs:5-30,52-60` and the
//! configuration isolation in `scripts/opencode.mjs:61-70`, which
//! `server/test/opencode-launcher.test.mjs` locks.

use std::path::Path;

use via_core::EnvMap;

use crate::profile::encode_uri_component;

/// `VIA_OPENCODE_XDG_CONFIG_HOME` — upstream
/// `QWEN_AUDIO_AGENT_OPENCODE_XDG_CONFIG_HOME`, renamed per `docs/rebrand.md`.
pub const XDG_CONFIG_HOME_OVERRIDE: &str = "VIA_OPENCODE_XDG_CONFIG_HOME";

/// `VIA_OPENCODE_ISOLATE_USER_CONFIG` — upstream
/// `QWEN_AUDIO_AGENT_OPENCODE_ISOLATE_USER_CONFIG`.
pub const ISOLATE_USER_CONFIG: &str = "VIA_OPENCODE_ISOLATE_USER_CONFIG";

/// The value that turns isolation on.
///
/// **External contract** — `scripts/opencode.mjs:68`
/// (`=== 'true'`). Exactly the string `true`, not truthiness: `1` does not
/// enable it upstream and must not here.
pub const ISOLATE_USER_CONFIG_ON: &str = "true";

/// Where an isolated OpenCode configuration lives, relative to the install root.
///
/// **External contract** — `scripts/opencode.mjs:69`
/// (`${QWEN_AUDIO_AGENT_ROOT || ROOT}/runtime/opencode-xdg`), asserted by
/// `server/test/opencode-launcher.test.mjs:15`.
pub const ISOLATED_XDG_DIRECTORY: &str = "runtime/opencode-xdg";

/// The `XDG_CONFIG_HOME` an OpenCode child should be given, if any.
///
/// **External contract** — `scripts/opencode.mjs:65-70`. An explicit override
/// wins; otherwise isolation is opt-in and points inside the install root, so a
/// managed OpenCode never writes into the user's own `~/.config/opencode`.
#[must_use]
pub fn isolated_xdg_config_home(env: &EnvMap, root: &Path) -> Option<String> {
    let explicit = env.get_trimmed(XDG_CONFIG_HOME_OVERRIDE);
    if !explicit.is_empty() {
        return Some(explicit.to_owned());
    }
    if env.get_trimmed(ISOLATE_USER_CONFIG) == ISOLATE_USER_CONFIG_ON {
        return Some(
            root.join(ISOLATED_XDG_DIRECTORY)
                .to_string_lossy()
                .into_owned(),
        );
    }
    None
}

/// One `session/set_config_option` entry, as OpenCode advertises it.
///
/// The shape [`accepts_resumed_session`] reads; a subset of ACP's
/// `SessionConfigOption` with only the fields that decision needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdvertisedOption {
    /// The option id, e.g. `mode`.
    pub id: String,
    /// The option kind, e.g. `select`.
    pub kind: String,
    /// The value currently selected.
    pub current_value: String,
    /// Every selectable value, with nested groups flattened.
    pub values: Vec<String>,
}

/// Whether a recorded OpenCode session may be resumed.
///
/// **External contract** — `acceptsResumedSession`,
/// `server/src/agent/backends/opencode.mjs:19-30`.
///
/// The case it exists for: a coordinator session was created under a named
/// OpenCode agent that has since been removed or renamed. OpenCode will happily
/// resume it and then answer as a *different* agent, so VIA would silently be
/// talking to something else. Refusing the resume starts a fresh session
/// instead.
///
/// It applies only to the coordinator session with no explicitly configured
/// agent — a project session, or one whose agent VIA chose, is always
/// resumable. A `mode` option that is not a `select`, or one whose current
/// value is still among the advertised ones, is fine.
#[must_use]
pub fn accepts_resumed_session(
    is_coordinator: bool,
    coordinator_agent: &str,
    options: &[AdvertisedOption],
) -> bool {
    if !is_coordinator || !coordinator_agent.trim().is_empty() {
        return true;
    }
    let Some(option) = options.iter().find(|option| option.id.trim() == "mode") else {
        return true;
    };
    if option.kind != "select" {
        return true;
    }
    let current = option.current_value.trim();
    let values: Vec<&str> = option
        .values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect();
    !(!current.is_empty() && !values.is_empty() && !values.contains(&current))
}

/// `uiUrl({ baseUrl, sessionId })` —
/// `server/src/agent/backends/opencode.mjs:52-60`.
///
/// `<baseUrl>/server/<base64url(baseUrl)>/session/<encodeURIComponent(sessionId)>`.
/// A missing base URL or session id falls back to the base URL itself, which is
/// what `GET /api/backend/ui` then redirects to.
#[must_use]
pub fn ui_url(base_url: &str, session_id: &str) -> String {
    if base_url.is_empty() || session_id.is_empty() {
        return base_url.to_owned();
    }
    std::format!(
        "{base_url}/server/{}/session/{}",
        base64url(base_url.as_bytes()),
        encode_uri_component(session_id)
    )
}

/// `Buffer.from(value).toString('base64url')` — unpadded, `-` and `_`.
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).copied().map_or(0, u32::from);
        let b2 = chunk.get(2).copied().map_or(0, u32::from);
        let triple = (b0 << 16) | (b1 << 8) | b2;
        let indices = [
            (triple >> 18) & 0x3f,
            (triple >> 12) & 0x3f,
            (triple >> 6) & 0x3f,
            triple & 0x3f,
        ];
        let take = chunk.len() + 1;
        for index in indices.iter().take(take) {
            out.push(ALPHABET[*index as usize] as char);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    #[test]
    fn isolation_is_opt_in_and_the_override_wins() {
        let root = Path::new("/opt/via");
        assert_eq!(isolated_xdg_config_home(&EnvMap::new(), root), None);
        assert_eq!(
            isolated_xdg_config_home(&env(&[(ISOLATE_USER_CONFIG, "true")]), root),
            Some("/opt/via/runtime/opencode-xdg".to_owned())
        );
        // Exactly the string `true`.
        assert_eq!(
            isolated_xdg_config_home(&env(&[(ISOLATE_USER_CONFIG, "1")]), root),
            None
        );
        assert_eq!(
            isolated_xdg_config_home(
                &env(&[
                    (XDG_CONFIG_HOME_OVERRIDE, "/elsewhere"),
                    (ISOLATE_USER_CONFIG, "true")
                ]),
                root
            ),
            Some("/elsewhere".to_owned())
        );
    }

    fn mode(current: &str, values: &[&str], kind: &str) -> Vec<AdvertisedOption> {
        vec![AdvertisedOption {
            id: "mode".to_owned(),
            kind: kind.to_owned(),
            current_value: current.to_owned(),
            values: values.iter().map(|value| (*value).to_owned()).collect(),
        }]
    }

    #[test]
    fn a_coordinator_session_on_a_vanished_agent_is_not_resumed() {
        assert!(!accepts_resumed_session(
            true,
            "",
            &mode("gone", &["build", "plan"], "select")
        ));
    }

    #[test]
    fn every_other_shape_is_resumable() {
        // An explicitly configured agent: VIA chose it, so it is not a guess.
        assert!(accepts_resumed_session(
            true,
            "via-backend",
            &mode("gone", &["build"], "select")
        ));
        // A project session.
        assert!(accepts_resumed_session(
            false,
            "",
            &mode("gone", &["build"], "select")
        ));
        // The current value is still advertised.
        assert!(accepts_resumed_session(
            true,
            "",
            &mode("build", &["build", "plan"], "select")
        ));
        // No `mode` option at all.
        assert!(accepts_resumed_session(true, "", &[]));
        // Not a select.
        assert!(accepts_resumed_session(
            true,
            "",
            &mode("gone", &["build"], "text")
        ));
        // No current value, or no advertised values.
        assert!(accepts_resumed_session(
            true,
            "",
            &mode("", &["build"], "select")
        ));
        assert!(accepts_resumed_session(
            true,
            "",
            &mode("gone", &[], "select")
        ));
    }

    #[test]
    fn the_ui_url_is_a_per_server_per_session_deep_link() {
        assert_eq!(
            ui_url("http://127.0.0.1:4096", "ses 1"),
            "http://127.0.0.1:4096/server/aHR0cDovLzEyNy4wLjAuMTo0MDk2/session/ses%201"
        );
        assert_eq!(ui_url("http://127.0.0.1:4096", ""), "http://127.0.0.1:4096");
        assert_eq!(ui_url("", "ses"), "");
    }

    #[test]
    fn base64url_uses_the_url_alphabet_without_padding() {
        assert_eq!(base64url(b""), "");
        assert_eq!(base64url(b"f"), "Zg");
        assert_eq!(base64url(b"fo"), "Zm8");
        assert_eq!(base64url(b"foo"), "Zm9v");
        assert_eq!(base64url(&[0xfb, 0xff, 0xfe]), "-__-");
    }
}
