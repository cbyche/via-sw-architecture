//! `via chat`, as a user runs it.
//!
//! [`milestone`](../milestone.rs) drives the Gateway in-process with the two
//! seams swapped, which is what makes the phase-5 milestone assertable. This
//! file asserts the other half: that the **binary** connects, that its command
//! surface answers, and that the two lifecycle behaviours a person notices —
//! autostart and the single-instance refusal — do what they say.
//!
//! Both halves are needed and neither replaces the other. A green milestone
//! test with a `via chat` that cannot open a socket would be a green test and a
//! broken product.

mod support;

use support::Fixture;
use via_i18n::{Locale, keys, t};

/// A port nothing is listening on.
///
/// Bound and released, which is the only way to learn one. The window between
/// the release and the Gateway's own bind is a real race and is why every test
/// here takes its own port rather than sharing one.
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("an ephemeral port");
    let port = listener.local_addr().expect("a bound address").port();
    drop(listener);
    port
}

/// Strip the ANSI escapes upstream's text CLI paints with, so an assertion
/// reads as the sentence a user sees.
fn plain(text: &str) -> String {
    let mut result = String::new();
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' {
            for escape in characters.by_ref() {
                if escape == 'm' {
                    break;
                }
            }
            continue;
        }
        result.push(character);
    }
    result
}

#[test]
fn chat_connects_to_a_running_gateway_and_answers_its_commands() {
    let fixture = Fixture::new();
    let port = free_port();
    let url = std::format!("http://127.0.0.1:{port}");
    let gateway = fixture.start(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &["gateway", "--url", &url],
    );
    assert!(gateway.is_serving(), "the Gateway is up");

    // `/help` prints the catalogued help; `/tasks` prints the empty-list
    // sentence; `/exit` ends the session — all three, in one stdin.
    let run = fixture.run_with_input(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &[
            "chat",
            "--url",
            &url,
            "--no-autostart",
            "--session",
            "voice-cli",
        ],
        "/help\n/tasks\n/exit\n",
    );
    let out = plain(&run.stdout);

    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    assert!(
        out.contains(&via_i18n::format(
            Locale::En,
            keys::CHAT_CONNECTED,
            &[("session", "voice-cli")],
        )),
        "the connect line names the session: {out}",
    );
    assert!(out.contains(t(Locale::En, keys::CHAT_HELP)), "{out}");
    assert!(out.contains(t(Locale::En, keys::CHAT_NO_TASKS)), "{out}");

    let _ = gateway.stop();
}

#[test]
fn an_unknown_command_is_named_and_the_session_continues() {
    let fixture = Fixture::new();
    let port = free_port();
    let url = std::format!("http://127.0.0.1:{port}");
    let gateway = fixture.start(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &["gateway", "--url", &url],
    );
    assert!(gateway.is_serving());

    let run = fixture.run_with_input(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &["chat", "--url", &url, "--no-autostart"],
        "/nonesuch\n/tasks\n/q\n",
    );
    let out = plain(&run.stdout);
    assert!(
        out.contains(&via_i18n::format(
            Locale::En,
            keys::CHAT_UNKNOWN_COMMAND,
            &[("command", "/nonesuch")],
        )),
        "{out}",
    );
    assert!(
        out.contains(t(Locale::En, keys::CHAT_NO_TASKS)),
        "an unknown command must not end the session: {out}",
    );
    assert_eq!(run.code, Some(0));
    let _ = gateway.stop();
}

#[test]
fn no_autostart_refuses_with_the_catalogued_sentence_and_starts_nothing() {
    let fixture = Fixture::new();
    let port = free_port();
    let url = std::format!("http://127.0.0.1:{port}");
    let run = fixture.run(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &["chat", "--url", &url, "--no-autostart"],
    );
    assert_eq!(run.code, Some(1));
    assert_eq!(
        run.stderr,
        std::format!(
            "via: {}\n",
            via_i18n::format(Locale::En, keys::CLI_GATEWAY_NOT_RUNNING, &[("url", &url)]),
        ),
    );
    assert!(
        !fixture.lease_file().exists(),
        "a refusal must not have started a Gateway",
    );
}

#[test]
fn chat_starts_a_gateway_when_none_is_running_and_stops_it_again() {
    let fixture = Fixture::new();
    let port = free_port();
    let url = std::format!("http://127.0.0.1:{port}");
    let run = fixture.run_with_input(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &["chat", "--url", &url, "--session", "voice-autostart"],
        "/exit\n",
    );
    let out = plain(&run.stdout);
    assert_eq!(run.code, Some(0), "stderr: {}", run.stderr);
    assert!(
        out.contains(&via_i18n::format(
            Locale::En,
            keys::CHAT_GATEWAY_STARTING,
            &[("url", &url)],
        )),
        "the start is announced rather than silent: {out}",
    );
    assert!(
        out.contains(&via_i18n::format(
            Locale::En,
            keys::CHAT_CONNECTED,
            &[("session", "voice-autostart")],
        )),
        "…and then it connects: {out}",
    );
    // The child it started is stopped with it, and stopping it releases the
    // lease. A leaked `gateway.lock` naming a dead pid is exactly what
    // `Started::stop`'s SIGTERM exists to prevent.
    assert!(
        !fixture.lease_file().exists(),
        "the Gateway `via chat` started is stopped, and gives its lease back",
    );
}

#[test]
fn a_second_gateway_refuses_while_the_first_holds_the_lease() {
    // The single-instance contract, between two **processes** — which is the
    // only way it is ever violated. `tests/setup_gate.rs` asserts the same
    // refusal against a lease this test process holds through `via-lock`'s own
    // API; this one asserts it against a Gateway that is genuinely serving.
    let fixture = Fixture::new();
    let first = fixture.start(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &[
            "gateway",
            "--url",
            &std::format!("http://127.0.0.1:{}", free_port()),
        ],
    );
    assert!(first.is_serving(), "the first Gateway is up");
    assert!(fixture.lease_file().exists(), "…and holds the lease");

    let second = fixture.run(
        &[("DASHSCOPE_API_KEY", "sk-example")],
        &[
            "gateway",
            "--url",
            &std::format!("http://127.0.0.1:{}", free_port()),
        ],
    );
    assert_eq!(second.code, Some(1));
    assert_eq!(
        second.error_code(),
        Some(via_lock::VIA_GATEWAY_ALREADY_RUNNING),
        "records: {:?}",
        second.records,
    );
    assert!(
        second.stderr.contains(&first.origin()),
        "the refusal names where the incumbent is listening: {}",
        second.stderr,
    );

    // …and the first is still serving, untouched.
    assert!(fixture.lease_file().exists());
    let run = first.stop();
    assert_eq!(run.code, Some(0));
    assert!(!fixture.lease_file().exists());
}
