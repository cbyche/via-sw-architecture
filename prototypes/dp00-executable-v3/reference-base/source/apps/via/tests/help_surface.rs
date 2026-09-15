//! The documented flag surface, locked as snapshots.
//!
//! `--help` is the only place the argument surface is *published*, so it is
//! the right thing to lock. A snapshot per command means that adding,
//! renaming, re-defaulting or moving a flag — or changing which environment
//! variable feeds it — shows up as a reviewable diff rather than as a silent
//! change to a documented interface.
//!
//! Environment *values* are hidden (`hide_env_values`), so the snapshots do
//! not depend on what the developer running the tests happens to have
//! exported. The variable *names* are shown, which is the part that is
//! contract.

use clap::CommandFactory;

use via::cli::Cli;

/// The long help for a command path, e.g. `["backend", "install"]`.
fn long_help(path: &[&str]) -> String {
    let mut command = Cli::command();
    command.build();
    let mut current = &mut command;
    for name in path {
        current = current
            .find_subcommand_mut(name)
            .unwrap_or_else(|| panic!("no subcommand `{name}` in {path:?}"));
    }
    current.render_long_help().to_string()
}

#[test]
fn the_binary_surface() {
    insta::assert_snapshot!("via", long_help(&[]));
}

#[test]
fn the_gateway_surface() {
    insta::assert_snapshot!("via-gateway", long_help(&["gateway"]));
}

#[test]
fn the_chat_surface() {
    insta::assert_snapshot!("via-chat", long_help(&["chat"]));
}

#[test]
fn the_config_surface() {
    insta::assert_snapshot!("via-config", long_help(&["config"]));
    insta::assert_snapshot!("via-config-show", long_help(&["config", "show"]));
    insta::assert_snapshot!("via-config-set", long_help(&["config", "set"]));
}

#[test]
fn the_backend_surface() {
    insta::assert_snapshot!("via-backend", long_help(&["backend"]));
    insta::assert_snapshot!("via-backend-install", long_help(&["backend", "install"]));
    insta::assert_snapshot!("via-backend-status", long_help(&["backend", "status"]));
    insta::assert_snapshot!("via-backend-auth", long_help(&["backend", "auth"]));
}

#[test]
fn the_mcp_serve_surface() {
    insta::assert_snapshot!("via-mcp-serve", long_help(&["mcp-serve"]));
}

#[test]
fn the_service_surface() {
    insta::assert_snapshot!("via-service", long_help(&["service"]));
}

#[test]
fn every_environment_variable_in_the_help_is_a_via_name_or_a_kept_one() {
    // `docs/rebrand.md`: `VIA_*` is ours; `AGENT_PROTOCOL` is upstream's own
    // brand-free name and is KEPT. Anything else appearing as an `[env: …]`
    // annotation would be a name nobody has classified.
    let mut seen: Vec<String> = Vec::new();
    for path in [
        vec![],
        vec!["gateway"],
        vec!["chat"],
        vec!["config"],
        vec!["config", "set"],
        vec!["backend"],
        vec!["backend", "install"],
        vec!["backend", "status"],
        vec!["mcp-serve"],
        vec!["service"],
    ] {
        let help = long_help(&path);
        let mut rest = help.as_str();
        while let Some(start) = rest.find("[env: ") {
            rest = &rest[start + "[env: ".len()..];
            let end = rest.find(']').expect("an unterminated env annotation");
            seen.push(rest[..end].trim_end_matches('=').to_owned());
            rest = &rest[end..];
        }
    }
    assert!(
        !seen.is_empty(),
        "the help publishes no environment surface"
    );
    for name in seen {
        assert!(
            name.starts_with("VIA_") || name == "AGENT_PROTOCOL",
            "`{name}` is neither a VIA name nor a documented KEEP"
        );
    }
}

#[test]
fn the_help_names_every_verb_in_section_ten() {
    let help = long_help(&[]);
    for verb in [
        "gateway",
        "chat",
        "config",
        "backend",
        "mcp-serve",
        "service",
    ] {
        assert!(help.contains(verb), "`{verb}` is missing from the help");
    }
}

#[test]
fn the_help_does_not_advertise_a_command_via_does_not_have() {
    let help = long_help(&[]);
    for absent in ["tui", "webui", "skill", "setup"] {
        assert!(
            !help.split_whitespace().any(|word| word == absent),
            "the help advertises `{absent}`, which VIA does not implement"
        );
    }
}
