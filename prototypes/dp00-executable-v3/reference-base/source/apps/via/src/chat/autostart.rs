//! Starting a Gateway when none is running.
//!
//! Ported from `cli/src/runtime.mjs:369-430` (`ensureRuntime`), narrowed to the
//! two decisions that survive into VIA:
//!
//! ```js
//! if (!health) {
//!   if (!local) throw new Error(`无法连接远程 Gateway：${options.url}`)
//!   const target = new URL(options.url)
//!   if (target.protocol !== 'http:') {
//!     throw new Error('本地自动启动 Gateway 只支持 http 地址')
//!   }
//!   const gateway = spawnImpl(spec.command, spec.args, spec.options)
//!   health = await waitForReadiness(gateway, waitForGateway(options.url, …))
//! }
//! ```
//!
//! # Two refusals before anything is spawned
//!
//! **A remote Gateway is never started.** There is nothing to start: the
//! process would have to run on the other machine. `cli.remote_gateway_unreachable`.
//!
//! **A local Gateway is only started over `http`.** `https` needs a certificate
//! and a terminating proxy, neither of which a spawned child has.
//! `cli.local_autostart_http_only`.
//!
//! # What is not ported
//!
//! Upstream's `findRunningGateway` fallback — *"an owned Gateway may move its
//! private backend to a free local port"* — and `assertGatewayCompatibility`,
//! which refuses to reuse a Gateway whose backend, ownership, permission mode
//! or realtime model differs from the one this invocation asked for. Both
//! belong with `via backend` and the reuse-mismatch sentences already in the
//! catalogue (`cli.reuse_*`); `docs/deviations/phase-5-apps-via.md` records
//! them as owed rather than pretending they are here.

use std::process::Stdio;
use std::time::Duration;

use via_i18n::{Locale, keys};

use crate::commands::chat::ChatPlan;
use crate::error::{CODE_INVALID_ARGUMENT, CliError};
use crate::host::Host;

/// How long a spawned Gateway is given to answer `/api/health`.
///
/// **External contract** — `cli/src/runtime.mjs`'s `waitForGateway` default,
/// which polls until this budget is spent.
pub const READINESS_TIMEOUT: Duration = Duration::from_secs(30);

/// How often readiness is re-probed.
pub const READINESS_INTERVAL: Duration = Duration::from_millis(100);

/// Whether `origin` names this machine.
///
/// **External contract** — `isLocalGateway`, over
/// [`via_core::security::LOOPBACK_HOSTS`] so the CLI and the Gateway's
/// origin allow-list agree on what "local" is rather than keeping two lists.
#[must_use]
pub fn is_local(origin: &str) -> bool {
    url::Url::parse(origin)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .is_some_and(|host| {
            via_core::security::LOOPBACK_HOSTS.contains(&host.as_str())
                || host.eq_ignore_ascii_case("localhost")
        })
}

/// A Gateway this process started, and is therefore responsible for stopping.
#[derive(Debug)]
pub struct Started {
    child: tokio::process::Child,
}

impl Started {
    /// Stop it, gracefully.
    ///
    /// `ManagedRuntime.close(signal)` sends SIGTERM and lets the Gateway run
    /// its own close sequence — which is what releases the lease. Killing it
    /// would leave a `gateway.lock` naming a dead pid, and the next start would
    /// have to prove the incumbent was dead before reclaiming it.
    pub async fn stop(mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.child.id() {
            // `Child::kill` is SIGKILL, and SIGKILL is the one signal that
            // proves nothing about a graceful shutdown.
            let _ = tokio::process::Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
            if tokio::time::timeout(super::autostart::READINESS_TIMEOUT, self.child.wait())
                .await
                .is_ok()
            {
                return;
            }
        }
        let _ = self.child.kill().await;
    }
}

/// Start a Gateway for `plan`, and wait for it to answer.
///
/// # Errors
///
/// * [`CliError::Refused`] with `cli.gateway_not_running` when the user asked
///   for no autostart;
/// * `cli.remote_gateway_unreachable` for a URL that is not this machine's;
/// * `cli.local_autostart_http_only` for a local `https` URL;
/// * `gateway.start_timeout` when the child never answers.
pub async fn start(
    plan: &ChatPlan,
    host: &Host,
    out: &mut dyn std::io::Write,
    locale: Locale,
) -> Result<Started, CliError> {
    if !plan.autostart {
        return Err(refusal(locale, keys::CLI_GATEWAY_NOT_RUNNING, &plan.url));
    }
    if !is_local(&plan.url) {
        return Err(refusal(
            locale,
            keys::CLI_REMOTE_GATEWAY_UNREACHABLE,
            &plan.url,
        ));
    }
    if !plan.url.starts_with("http://") {
        return Err(CliError::refused(
            CODE_INVALID_ARGUMENT,
            locale,
            keys::CLI_LOCAL_AUTOSTART_HTTP_ONLY,
        ));
    }

    let executable = std::env::current_exe()
        .map_err(|error| CliError::io("locate this executable", "<self>", error))?;
    let mut command = tokio::process::Command::new(executable);
    // `.env_clear()` and an explicit copy of *this* host's environment, per
    // `docs/architecture.md` §17's ninth review question: Rust's
    // inherit-by-default `Command` breaks the credential boundary silently, and
    // the child must see the environment this process resolved rather than the
    // one it happened to be launched with.
    command.env_clear();
    for (key, value) in host.env().iter() {
        command.env(key, value);
    }
    command.current_dir(host.working_directory());
    command.args(["gateway", "--url", &plan.url]);
    command.stdin(Stdio::null());
    let child = command
        .spawn()
        .map_err(|error| CliError::io("start the Gateway", "<self>", error))?;

    let _ = writeln!(
        out,
        "{}{}{}",
        super::style::DIM,
        via_i18n::format(locale, keys::CHAT_GATEWAY_STARTING, &[("url", &plan.url)]),
        super::style::RESET,
    );
    let _ = out.flush();

    let started = Started { child };
    if wait_for_ready(&plan.url, locale).await {
        return Ok(started);
    }
    started.stop().await;
    Err(refusal(locale, keys::GATEWAY_START_TIMEOUT, &plan.url))
}

/// Poll `/api/health` until it answers or the budget is spent.
async fn wait_for_ready(origin: &str, locale: Locale) -> bool {
    let deadline = tokio::time::Instant::now() + READINESS_TIMEOUT;
    while tokio::time::Instant::now() < deadline {
        if let Ok(mut client) = super::GatewayClient::new(origin, locale)
            && matches!(client.health().await, Ok(Some(_)))
        {
            return true;
        }
        tokio::time::sleep(READINESS_INTERVAL).await;
    }
    false
}

fn refusal(locale: Locale, key: via_i18n::Key, url: &str) -> CliError {
    CliError::refused_with(CODE_INVALID_ARGUMENT, locale, key, &[("url", url)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("http://127.0.0.1:3101", true)]
    #[case("http://localhost:3101", true)]
    #[case("http://LOCALHOST:3101", true)]
    #[case("http://[::1]:3101", true)]
    #[case("https://127.0.0.1:3101", true)]
    #[case("http://10.0.0.2:3101", false)]
    #[case("https://voice.example.com", false)]
    #[case("not a url", false)]
    fn only_this_machine_is_local(#[case] origin: &str, #[case] local: bool) {
        assert_eq!(is_local(origin), local, "{origin}");
    }

    #[test]
    fn the_loopback_list_is_via_cores_own() {
        // Keeping a second list here is how the CLI and the Gateway's origin
        // allow-list drift apart.
        for host in via_core::security::LOOPBACK_HOSTS {
            let origin = if host.starts_with('[') || host.contains(':') && !host.contains('.') {
                std::format!("http://[{}]:3101", host.trim_matches(['[', ']']))
            } else {
                std::format!("http://{host}:3101")
            };
            assert!(is_local(&origin), "{origin}");
        }
    }

    #[tokio::test]
    async fn no_autostart_refuses_with_the_catalogued_sentence() {
        let host = Host::new(
            [("VIA_LOCALE", "en")].into_iter().collect(),
            "/home/via".into(),
            "/srv/via".into(),
        );
        let plan = ChatPlan {
            url: "http://127.0.0.1:3101".to_owned(),
            session_id: "voice-1".to_owned(),
            takeover: false,
            autostart: false,
        };
        let mut out = Vec::new();
        let error = start(&plan, &host, &mut out, Locale::En)
            .await
            .expect_err("--no-autostart means no autostart");
        assert_eq!(
            error.message(Locale::En),
            via_i18n::format(
                Locale::En,
                keys::CLI_GATEWAY_NOT_RUNNING,
                &[("url", "http://127.0.0.1:3101")],
            ),
        );
        assert!(
            out.is_empty(),
            "nothing is announced when nothing is started"
        );
    }

    #[tokio::test]
    async fn a_remote_gateway_is_never_started() {
        let host = Host::new(Default::default(), "/home/via".into(), "/srv/via".into());
        let plan = ChatPlan {
            url: "https://voice.example.com".to_owned(),
            session_id: "voice-1".to_owned(),
            takeover: false,
            autostart: true,
        };
        let mut out = Vec::new();
        let error = start(&plan, &host, &mut out, Locale::En)
            .await
            .expect_err("there is nothing on this machine to start");
        assert!(
            error.message(Locale::En).contains("voice.example.com"),
            "{}",
            error.message(Locale::En),
        );
    }

    #[tokio::test]
    async fn a_local_https_gateway_is_refused_rather_than_spawned() {
        let host = Host::new(Default::default(), "/home/via".into(), "/srv/via".into());
        let plan = ChatPlan {
            url: "https://127.0.0.1:3101".to_owned(),
            session_id: "voice-1".to_owned(),
            takeover: false,
            autostart: true,
        };
        let mut out = Vec::new();
        let error = start(&plan, &host, &mut out, Locale::En)
            .await
            .expect_err("a spawned child has no certificate");
        assert_eq!(
            error.message(Locale::En),
            via_i18n::t(Locale::En, keys::CLI_LOCAL_AUTOSTART_HTTP_ONLY),
        );
    }
}
