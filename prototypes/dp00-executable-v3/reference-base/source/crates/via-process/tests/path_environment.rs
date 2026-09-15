//! The child's `PATH`, and the spawn spec built around it.
//!
//! Upstream's `server/test/path-environment.test.mjs` exercises
//! `shared/path-environment.mjs` directly. That module is already ported as
//! [`via_core::search_path`] and tested there, so what this file asserts is
//! the layer above it: that `via-process` *uses* it rather than restating it,
//! and that the composition in `shared/backend-install.mjs:206-231` — the part
//! that makes a `node` installed under a version manager findable from a
//! spawned `npx` — behaves.
//!
//! The platform is a parameter throughout, so a Windows child environment is
//! asserted from a developer's Mac. That is the whole reason
//! [`via_core::search_path::Platform`] is not a `cfg`.

mod common;

use std::path::{Path, PathBuf};

use common::{FixedResolver, env, owned_service_definition, service_driver};
use pretty_assertions::assert_eq;
use via_core::search_path::{Platform, command_directory, merge_search_path};
use via_process::{
    ChildStdio, CommandResolver, EXECUTABLE_PATH_SUFFIXES, ProcessError, WhichResolver,
    compose_child_search_path, spawn_spec,
};

const ROOT: &str = "/repo";

// ── composition ─────────────────────────────────────────────────────────────

#[test]
fn composition_is_the_shared_merge_with_executable_entries_dropped() {
    // Same answer as `merge_search_path` when nothing needs dropping — proving
    // the reuse rather than a second implementation.
    for platform in [Platform::Posix, Platform::Windows] {
        let (current, command) = match platform {
            Platform::Posix => ("/usr/bin:/bin", "/opt/node/bin/npx"),
            Platform::Windows => ("C:\\Windows;C:\\System32", "C:\\Node\\npx.cmd"),
        };
        assert_eq!(
            compose_child_search_path(current, command, platform),
            merge_search_path(
                current,
                &command_directory(command, platform),
                platform,
                true
            ),
        );
    }
}

#[test]
fn every_executable_suffix_is_dropped_case_insensitively() {
    for suffix in EXECUTABLE_PATH_SUFFIXES {
        for spelling in [suffix.to_lowercase(), suffix.to_uppercase()] {
            let current = format!("C:\\tools\\npm{spelling};C:\\Windows");
            assert_eq!(
                compose_child_search_path(&current, "C:\\Node\\npx.cmd", Platform::Windows),
                "C:\\Node;C:\\Windows",
                "`{spelling}` was not dropped",
            );
        }
    }
}

#[test]
fn a_directory_that_merely_contains_an_extension_is_kept() {
    // Only a *trailing* extension marks an entry as a file. A directory named
    // `/opt/x.exe.d` is still a directory.
    assert_eq!(
        compose_child_search_path(
            "/opt/x.exe.d:/usr/bin",
            "/opt/node/bin/npx",
            Platform::Posix
        ),
        "/opt/node/bin:/opt/x.exe.d:/usr/bin",
    );
}

#[test]
fn the_version_managers_bin_directory_leads_the_child_path() {
    // The case the composition exists for: `npx` resolved under a version
    // manager, whose sibling `node` must win over any system one.
    let composed = compose_child_search_path(
        "/usr/local/bin:/usr/bin",
        "/Users/dev/.local/share/fnm/node-versions/v24.3.0/installation/bin/npx",
        Platform::Posix,
    );
    assert!(
        composed.starts_with("/Users/dev/.local/share/fnm/node-versions/v24.3.0/installation/bin:"),
        "{composed}",
    );
}

#[test]
fn an_empty_parent_path_still_publishes_the_command_directory() {
    assert_eq!(
        compose_child_search_path("", "/opt/node/bin/npx", Platform::Posix),
        "/opt/node/bin",
    );
    assert_eq!(
        compose_child_search_path("  :  : ", "/opt/node/bin/npx", Platform::Posix),
        "/opt/node/bin",
    );
}

// ── the spawn spec ──────────────────────────────────────────────────────────

#[test]
fn the_spawn_spec_is_rooted_at_the_installation_and_inherits_stdio() {
    let definition = owned_service_definition();
    let driver = service_driver(definition);
    let spec = spawn_spec(
        &driver,
        Path::new(ROOT),
        &env(&[("PATH", "/usr/bin")]),
        Platform::Posix,
        &FixedResolver::new("/opt/fixture/bin"),
    )
    .expect("a spec");
    assert_eq!(spec.working_directory, PathBuf::from(ROOT));
    assert_eq!(spec.stdio, ChildStdio::Inherit);
    assert_eq!(
        spec.command,
        PathBuf::from("/opt/fixture/bin/fixture-runner"),
    );
    assert_eq!(spec.arguments, vec!["--serve".to_owned()]);
    assert_eq!(
        spec.environment.get("PATH"),
        Some("/opt/fixture/bin:/usr/bin"),
        "the resolved command's directory leads the child PATH",
    );
}

#[test]
fn the_command_is_resolved_against_the_child_path_not_the_parents() {
    // The projection runs first, so a `PATH` the policy does not forward is
    // not the one the command is looked up on. Here the policy forwards `PATH`
    // (it is a system name), so what matters is that the *projected* value is
    // what reaches the resolver.
    #[derive(Debug)]
    struct AssertingResolver;

    impl CommandResolver for AssertingResolver {
        fn resolve(
            &self,
            command: &str,
            search_path: &str,
            working_directory: &Path,
        ) -> Result<PathBuf, ProcessError> {
            assert_eq!(search_path, "/child/only");
            assert_eq!(working_directory, Path::new(ROOT));
            Ok(PathBuf::from(format!("/child/only/{command}")))
        }
    }

    let definition = owned_service_definition();
    let driver = service_driver(definition);
    spawn_spec(
        &driver,
        Path::new(ROOT),
        &env(&[("PATH", "/child/only")]),
        Platform::Posix,
        &AssertingResolver,
    )
    .expect("a spec");
}

#[test]
fn a_windows_child_environment_is_built_correctly_from_any_host() {
    let definition = owned_service_definition();
    let driver = service_driver(definition);
    let spec = spawn_spec(
        &driver,
        Path::new("C:\\repo"),
        &env(&[("PATH", "C:\\Windows;C:\\tools\\npm.cmd")]),
        Platform::Windows,
        &FixedResolver::new("C:\\Node"),
    )
    .expect("a spec");
    assert_eq!(
        spec.environment.get("PATH"),
        Some("C:\\Node;C:\\Windows"),
        "the `;` delimiter, the dropped `.cmd` entry and the leading command directory",
    );
}

#[test]
fn a_driver_with_no_launch_has_no_spec() {
    let definition = owned_service_definition();
    let mut driver = service_driver(definition);
    driver.launch = None;
    assert!(matches!(
        spawn_spec(
            &driver,
            Path::new(ROOT),
            &env(&[]),
            Platform::Posix,
            &FixedResolver::new("/opt"),
        ),
        Err(ProcessError::DriverMissingManagedLaunch { .. })
    ));
}

#[test]
fn driver_additions_reach_the_child_environment() {
    let definition = owned_service_definition();
    let mut driver = service_driver(definition);
    driver
        .launch
        .as_mut()
        .expect("a launch")
        .environment_additions
        .push(("FIXTURE_RUN_AS".to_owned(), "1".to_owned()));
    let spec = spawn_spec(
        &driver,
        Path::new(ROOT),
        &env(&[("PATH", "/usr/bin")]),
        Platform::Posix,
        &FixedResolver::new("/opt/fixture/bin"),
    )
    .expect("a spec");
    assert_eq!(spec.environment.get("FIXTURE_RUN_AS"), Some("1"));
}

// ── the real resolver ───────────────────────────────────────────────────────

#[test]
fn a_command_with_a_separator_is_taken_as_written() {
    // A shell does the same thing, and it is what makes an absolute path in
    // configuration work without a `PATH` entry for it.
    for command in ["/opt/agent/bin/agent", "./agent", "C:\\Node\\npx.cmd"] {
        assert_eq!(
            WhichResolver
                .resolve(command, "", Path::new(ROOT))
                .expect("taken as written"),
            PathBuf::from(command),
        );
    }
}

#[test]
fn a_bare_command_is_looked_up_on_the_supplied_path_only() {
    let directory = tempfile::tempdir().expect("a temp directory");
    let executable = directory.path().join("via-process-fixture-runner");
    std::fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("write the fixture");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
            .expect("make it executable");
    }

    let found = WhichResolver
        .resolve(
            "via-process-fixture-runner",
            &directory.path().to_string_lossy(),
            Path::new(ROOT),
        )
        .expect("found on the supplied PATH");
    assert_eq!(found.file_name(), executable.file_name());

    // The same command is *not* found when the path does not name it, even
    // though it exists on disk — the lookup never falls back to the host's own
    // environment.
    assert!(matches!(
        WhichResolver.resolve(
            "via-process-fixture-runner",
            "/nonexistent",
            Path::new(ROOT)
        ),
        Err(ProcessError::CommandNotFound { .. })
    ));
}

// ── argv resolved at spawn time ─────────────────────────────────────────────

/// The regression this exists for: upstream's shim reads `<X>_PORT` from the
/// environment at start-up, so a port `start_managed_backend` **reallocated**
/// reaches the child. VIA spawns the binary, so the port is on the argv — and
/// baking it when the driver was built meant the reallocation reached the
/// environment and never the command line.
#[test]
fn a_placeholder_argument_is_expanded_from_the_childs_own_environment() {
    let definition = owned_service_definition();
    let mut driver = service_driver(definition);
    driver.launch = Some(via_process::ManagedLaunch::new(
        "fixture-runner",
        vec!["--port".to_owned(), "${FIXTURE_PORT:-4096}".to_owned()],
    ));

    // The reallocated port, as `apply_backend_address` would have published it.
    let spec = spawn_spec(
        &driver,
        Path::new(ROOT),
        &env(&[("PATH", "/usr/bin"), ("FIXTURE_PORT", "5123")]),
        Platform::Posix,
        &FixedResolver::new("/opt/fixture/bin"),
    )
    .expect("a spec");
    assert_eq!(spec.arguments, vec!["--port".to_owned(), "5123".to_owned()]);
}

#[test]
fn a_placeholder_falls_back_when_nothing_published_a_value() {
    let definition = owned_service_definition();
    let mut driver = service_driver(definition);
    driver.launch = Some(via_process::ManagedLaunch::new(
        "fixture-runner",
        vec!["--port".to_owned(), "${FIXTURE_PORT:-4096}".to_owned()],
    ));
    let spec = spawn_spec(
        &driver,
        Path::new(ROOT),
        &env(&[("PATH", "/usr/bin")]),
        Platform::Posix,
        &FixedResolver::new("/opt/fixture/bin"),
    )
    .expect("a spec");
    assert_eq!(spec.arguments, vec!["--port".to_owned(), "4096".to_owned()]);
}

/// An unresolvable placeholder with no fallback stays verbatim. A `--port `
/// that silently became empty would read as a configuration problem; one that
/// visibly did not expand reads as the bug it is.
#[test]
fn an_unresolvable_placeholder_with_no_fallback_is_left_alone() {
    let definition = owned_service_definition();
    let mut driver = service_driver(definition);
    driver.launch = Some(via_process::ManagedLaunch::new(
        "fixture-runner",
        vec!["--port".to_owned(), "${FIXTURE_PORT}".to_owned()],
    ));
    let spec = spawn_spec(
        &driver,
        Path::new(ROOT),
        &env(&[("PATH", "/usr/bin")]),
        Platform::Posix,
        &FixedResolver::new("/opt/fixture/bin"),
    )
    .expect("a spec");
    assert_eq!(
        spec.arguments,
        vec!["--port".to_owned(), "${FIXTURE_PORT}".to_owned()],
    );
}

/// An argument with no placeholder is untouched, and one with several expands
/// all of them — the substitution must not be a special case for `--port`.
#[test]
fn plain_arguments_pass_through_and_several_placeholders_all_expand() {
    let definition = owned_service_definition();
    let mut driver = service_driver(definition);
    driver.launch = Some(via_process::ManagedLaunch::new(
        "fixture-runner",
        vec![
            "--serve".to_owned(),
            "${FIXTURE_HOST}:${FIXTURE_PORT:-4096}".to_owned(),
        ],
    ));
    let spec = spawn_spec(
        &driver,
        Path::new(ROOT),
        &env(&[
            ("PATH", "/usr/bin"),
            ("FIXTURE_HOST", "127.0.0.1"),
            ("FIXTURE_PORT", "5123"),
        ]),
        Platform::Posix,
        &FixedResolver::new("/opt/fixture/bin"),
    )
    .expect("a spec");
    assert_eq!(
        spec.arguments,
        vec!["--serve".to_owned(), "127.0.0.1:5123".to_owned()],
    );
}
