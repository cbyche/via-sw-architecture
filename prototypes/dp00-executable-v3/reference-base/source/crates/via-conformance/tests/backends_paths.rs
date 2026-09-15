//! `via-backends`-owned rows about the Node shim mechanism upstream used to
//! launch and manage backends.
//!
//! `docs/architecture.md` §10 removes `scripts/*.mjs` entirely: *"Node's `npm
//! install -g`, the `npx` shims and `scripts/*-acp.mjs` disappear: a Rust
//! binary is its own installer, and the shims become `via-backends` launch
//! specs."* `docs/deviations/phase-2.md` (via-backends' phase) records the
//! concrete shape of that for both catalogued contracts asserted here: each
//! backend that upstream would have run through `process.execPath
//! scripts/<name>.mjs` is instead launched as the executable the shim would
//! have `exec`'d, with a fixed argv VIA builds directly rather than resolving
//! through the shim's own logic. VIA does not ship any of the eight shim
//! files, launch through one, or reproduce the resolution algorithm they
//! implemented — so both rows are `Divergent`: the catalogue's shim-shaped
//! description stays on record, and a real profile / real managed-launch spec
//! is built here to show what VIA does instead.

use std::path::PathBuf;

use via_backends::detect::MissingFinder;
use via_backends::{
    BackendProfile, LaunchContext, create_backend_profile, managed_launch, spawns_separate_process,
};
use via_catalog::{Ownership, backend_definitions};
use via_conformance::expect_contract;
use via_core::{Config, EnvMap};

/// A configuration with every excerpted backend's directory set. Mirrors
/// `via-backends`' own fixture (`tests/support/mod.rs::config`) closely enough
/// to reproduce the same launch specs; not imported from it because that
/// module is private to `via-backends`' own integration-test binaries, and
/// duplicating the handful of fields this file actually reads is cheaper than
/// restructuring that crate to export a test fixture — the same call
/// `tests/acp_wire.rs::config` already makes.
fn config() -> Config {
    let mut config = Config {
        root: PathBuf::from("/opt/via"),
        agent_timeout_ms: 120_000,
        ..Config::default()
    };
    let work = PathBuf::from("/work");
    config.backends.opencode.directory = work.clone();
    config.backends.openclaw.directory = work.clone();
    config.backends.openclaw.base_url = "http://127.0.0.1:18789".to_owned();
    config.backends.codex.directory = work.clone();
    config.backends.claude.directory = work.clone();
    config.backends.pi.directory = work.clone();
    config.backends.deepseek.directory = work;
    config
}

/// Build one backend's real profile the way the Gateway does: an owned
/// service, native permission, and no executable pre-resolved
/// (`MissingFinder`), which is exactly the case that falls back to each
/// driver's own catalogued default command.
fn build(id: &str, config: &Config) -> BackendProfile {
    create_backend_profile(
        id,
        &LaunchContext {
            config,
            env: &EnvMap::new(),
            ownership: Ownership::Owned,
            permission_mode: "native",
            owner_id: "user_personal",
            finder: &MissingFinder,
        },
    )
    .unwrap_or_else(|error| panic!("{id} builds: {error}"))
}

/// The eight shim files the catalogue names, none of which VIA ships.
const CATALOGUED_SCRIPTS: [&str; 8] = [
    "scripts/claude-code-acp.mjs",
    "scripts/codex-acp.mjs",
    "scripts/deepseek-harness-acp.mjs",
    "scripts/pi-acp.mjs",
    "scripts/opencode.mjs",
    "scripts/openclaw.mjs",
    "scripts/opencode-server.mjs",
    "scripts/openclaw-gateway.mjs",
];

#[test]
fn driver_launcher_scripts_run_as_the_direct_executable_not_a_node_shim() {
    let entry = expect_contract("file-path", "driver launcher scripts");
    let catalogued = &entry.exact_value;

    // The catalogue still names every one of upstream's eight shim files, and
    // states how they were resolved and spawned...
    for script in CATALOGUED_SCRIPTS {
        assert!(catalogued.contains(script), "{catalogued}");
    }
    assert!(
        entry.why.contains("resolve(root, 'scripts/<name>')"),
        "{}",
        entry.why
    );
    assert!(
        entry.why.contains("spawned with process.execPath"),
        "{}",
        entry.why
    );

    // ...none of which VIA ships anywhere in the workspace.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    for script in CATALOGUED_SCRIPTS {
        assert!(!root.join(script).exists(), "{script} should not exist");
    }

    // The four ACP-bridge drivers each launch the executable the shim would
    // have exec'd, per docs/deviations/phase-2.md — "opencode acp",
    // "openclaw acp --url … --verbose", "codex-acp", "claude-code-acp",
    // "pi-acp", "dsh-acp-demo --config <cordis.yml>" — with a fixed argv this
    // crate builds directly, not the shim's own runtime-resolution logic.
    let config = config();
    for (id, command) in [
        ("codex", "codex-acp"),
        ("claude", "claude-code-acp"),
        ("pi", "pi-acp"),
        ("deepseek", "dsh-acp-demo"),
        ("opencode", "opencode"),
        ("openclaw", "openclaw"),
    ] {
        let profile = build(id, &config);
        assert_eq!(profile.acp.spawn.command, command, "{id}");
        assert_ne!(profile.acp.spawn.command, "process.execPath", "{id}");
        assert!(
            profile
                .acp
                .spawn
                .args
                .iter()
                .all(|arg| !arg.contains("scripts/")),
            "{id}: {:?}",
            profile.acp.spawn.args
        );
    }

    // And the two managed services launch their own binary directly too,
    // rather than `process.execPath` plus a resolved `scripts/<name>.mjs`.
    for id in ["opencode", "openclaw"] {
        let launch = managed_launch(id, &EnvMap::new()).expect("has a managed launch");
        assert_eq!(launch.command, id);
        assert!(
            launch.arguments.iter().all(|arg| !arg.contains("scripts/")),
            "{id}: {:?}",
            launch.arguments
        );
    }
}

#[test]
fn the_managed_spawn_spec_launches_the_service_binary_directly() {
    let catalogued = &expect_contract("state-name", "managed backend spawn spec").exact_value;
    assert!(
        catalogued.contains("command: process.execPath"),
        "{catalogued}"
    );
    assert!(catalogued.contains("opencode-server.mjs"), "{catalogued}");
    assert!(catalogued.contains("openclaw-gateway.mjs"), "{catalogued}");
    assert!(
        catalogued.contains(
            "Every other backend uses managedProcessDriver with separateManagedProcess:false (no child process at all)"
        ),
        "{catalogued}"
    );

    // VIA's real shape matches the catalogue exactly on *which* two backends
    // spawn a service — this is the part the module docs on
    // `via_backends::runtime` quote verbatim from this same contract.
    for definition in backend_definitions() {
        assert_eq!(
            spawns_separate_process(definition.id),
            matches!(definition.id, "opencode" | "openclaw"),
            "{}",
            definition.id
        );
        assert_eq!(
            managed_launch(definition.id, &EnvMap::new()).is_some(),
            matches!(definition.id, "opencode" | "openclaw"),
            "{}",
            definition.id
        );
    }

    // ...but the two that do spawn one launch their own binary directly, with
    // a fixed argv, rather than `process.execPath` plus a resolved
    // `scripts/<name>.mjs` path. docs/deviations/phase-2.md — via-process:
    // "managedScript -> ManagedLaunch ... the driver carries a command + args
    // resolved on the child's PATH."
    let opencode = managed_launch("opencode", &EnvMap::new()).expect("has one");
    assert_eq!(opencode.command, "opencode");
    assert_ne!(opencode.command, "process.execPath");
    assert!(
        !opencode
            .arguments
            .iter()
            .any(|arg| arg.contains("scripts/"))
    );

    let openclaw = managed_launch("openclaw", &EnvMap::new()).expect("has one");
    assert_eq!(openclaw.command, "openclaw");
    assert_ne!(openclaw.command, "process.execPath");
    assert!(
        !openclaw
            .arguments
            .iter()
            .any(|arg| arg.contains("scripts/"))
    );
}
