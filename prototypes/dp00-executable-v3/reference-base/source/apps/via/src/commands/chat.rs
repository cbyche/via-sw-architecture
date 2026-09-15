//! `via chat` — the text client.
//!
//! `docs/fidelity.md` puts it exactly: *"`tui/` — upstream's terminal UI … is
//! deferred, not dropped. VIA ships `via chat` (a port of upstream's
//! `tui/src/text-cli.mjs`) so the core is drivable end to end."*
//!
//! This module is the **argument surface**: `--url` and `--session` carry
//! catalogued contracts, and both are validated here so that a wrong value is
//! a refusal a user can act on rather than a connection failure later. The
//! client itself is [`crate::chat`].

use crate::cli::ChatArgs;
use crate::error::CliError;
use crate::host::Host;
use crate::origin::{UrlLabel, clean_origin};
use crate::session::{create_voice_session_id, normalize_session_id};

/// What `via chat` resolved before it connected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatPlan {
    /// The Gateway origin.
    pub url: String,
    /// The voice session to join.
    pub session_id: String,
    /// Whether to seize voice control from the current client.
    pub takeover: bool,
    /// Whether a Gateway may be started when none is running.
    pub autostart: bool,
}

/// Validate `args`.
///
/// **External contract** — *cli-flag/--session*:
/// `env.<PREFIX>_SESSION_ID || createVoiceSessionId()`, then trimmed, then
/// refused if empty. The generated id is only reached when neither the flag
/// nor the variable supplied one, so a session is never silently *replaced*
/// by a fresh one.
///
/// # Errors
///
/// [`CliError::Refused`] for a URL that is not `http(s)`, or a `--session`
/// that is blank after trimming.
pub fn plan(args: &ChatArgs, host: &Host) -> Result<ChatPlan, CliError> {
    let locale = host.locale();
    let url = clean_origin(&args.url, UrlLabel::Gateway, locale)?;
    let session_id = match args.session.as_deref() {
        Some(requested) => normalize_session_id(requested, locale)?,
        None => create_voice_session_id(),
    };
    Ok(ChatPlan {
        url,
        session_id,
        takeover: args.takeover,
        autostart: !args.no_autostart,
    })
}

/// Run `via chat`.
///
/// Blocks until the user quits or the socket closes. The reactor is built here
/// rather than by a `#[tokio::main]` on `main`, for the same reason
/// [`crate::commands::gateway::run`] builds its own: four of the six verbs need
/// no reactor at all.
///
/// # Errors
///
/// * [`CliError::Refused`] for an invalid argument, for a Gateway that is not
///   running and may not be started, and for a socket the Gateway refused.
/// * [`CliError::Io`] when stdout cannot be written.
pub fn run(args: &ChatArgs, host: &Host, out: &mut dyn std::io::Write) -> Result<(), CliError> {
    let plan = plan(args, host)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| CliError::io("start the runtime", "<tokio>", error))?;
    runtime.block_on(crate::chat::run(&plan, host, out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CODE_INVALID_ARGUMENT;
    use crate::origin::DEFAULT_GATEWAY_URL;
    use crate::session::VOICE_SESSION_PREFIX;
    use via_i18n::Locale;

    fn host() -> Host {
        host_in(Locale::En)
    }

    fn host_in(locale: Locale) -> Host {
        Host::new(
            [("VIA_LOCALE", locale.as_str())].into_iter().collect(),
            "/home/via".into(),
            "/srv/via".into(),
        )
    }

    fn args(session: Option<&str>, url: &str, takeover: bool) -> ChatArgs {
        ChatArgs {
            url: url.to_owned(),
            session: session.map(str::to_owned),
            takeover,
            no_autostart: false,
        }
    }

    #[test]
    fn a_missing_session_is_generated() {
        let plan = plan(&args(None, DEFAULT_GATEWAY_URL, false), &host()).expect("planned");
        assert!(plan.session_id.starts_with(VOICE_SESSION_PREFIX));
        assert_eq!(plan.url, DEFAULT_GATEWAY_URL);
        assert!(!plan.takeover);
    }

    #[test]
    fn an_explicit_session_is_kept_and_trimmed() {
        let plan = plan(
            &args(Some("  project-one  "), DEFAULT_GATEWAY_URL, true),
            &host(),
        )
        .expect("planned");
        assert_eq!(plan.session_id, "project-one");
        assert!(plan.takeover);
    }

    #[test]
    fn a_blank_session_is_refused_rather_than_regenerated() {
        let error = plan(
            &args(Some("   "), DEFAULT_GATEWAY_URL, false),
            &host_in(Locale::Zh),
        )
        .expect_err("a blank session must not silently become a new one");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
        assert_eq!(error.message(Locale::Zh), "--session 不能为空");
    }

    #[test]
    fn a_bad_url_is_refused() {
        let error = plan(&args(None, "ws://127.0.0.1:3101", false), &host())
            .expect_err("ws is not an origin VIA speaks");
        assert_eq!(error.code(), CODE_INVALID_ARGUMENT);
    }

    #[test]
    fn autostart_is_on_unless_the_flag_turns_it_off() {
        let on = plan(&args(None, DEFAULT_GATEWAY_URL, false), &host()).expect("planned");
        assert!(on.autostart, "a user who typed `via chat` wants to chat");

        let mut refused = args(None, DEFAULT_GATEWAY_URL, false);
        refused.no_autostart = true;
        let off = plan(&refused, &host()).expect("planned");
        assert!(!off.autostart);
    }
}
