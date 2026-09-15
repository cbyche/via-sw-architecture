//! `via-acp`'s owning rows whose comparison spans a `via-backends` driver.
//!
//! One catalogued contract lives here: `file-path` / *backend ACP launch
//! specs (excerpt, all deep-equality asserted)*. It excerpts the real launch
//! spec of ten different backends, each built by a different `via-backends`
//! driver (`crates/via-backends/src/driver.rs`), so no single existing test in
//! that crate asserts the whole excerpt at once — `via-backends/tests/registry.rs`
//! covers every one of the ten individually, across eleven separate `#[test]`
//! functions, which is the right shape for *that* crate's own suite but not a
//! single name this gate's registry can point at. The test below builds every
//! named backend's real profile directly (`via_backends::create_backend_profile`,
//! the same public entry point `via-backends`' own fixtures use) and compares
//! it against the catalogue in one place instead.
//!
//! Five of the ten backends are not a straight match: upstream launches them
//! through a Node shim (`process.execPath` plus a `scripts/*.mjs` launcher),
//! and `docs/deviations/phase-2.md` records that VIA launches the executable
//! the shim would have `exec`'d instead, because a Rust binary has no Node
//! interpreter to shim through (`docs/fidelity.md` — *"Adapted, with the
//! reason"*, the `npm global install, npx shims, scripts/*-acp.mjs` row). That
//! is a documented, deliberate divergence, not a gap, so the contract as a
//! whole is catalogued `Divergent`: the test asserts both the catalogued
//! upstream Node-shim path and VIA's real, adapted value.

use std::path::PathBuf;

use via_backends::detect::MissingFinder;
use via_backends::{LaunchContext, create_backend_profile};
use via_catalog::Ownership;
use via_conformance::expect_contract;
use via_core::{Config, EnvMap};

const CONTRACT_KIND: &str = "file-path";
const CONTRACT_NAME: &str = "backend ACP launch specs (excerpt, all deep-equality asserted)";

/// A configuration with every excerpted backend's directory set, mirroring
/// `via-backends`' own fixture (`tests/support/mod.rs::config`) closely enough
/// to reproduce the same launch specs. Not imported directly: that module is
/// test-only and private to `via-backends`' integration test binaries, so a
/// crate outside it cannot reach it, and duplicating the handful of fields
/// this file actually reads is cheaper than restructuring that crate to
/// export a test fixture.
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
    config.backends.openclaw.token_file =
        PathBuf::from("/config/backends/openclaw/state/gateway-token");
    config.backends.qoder.directory = work.clone();
    config.backends.qwen.directory = work.clone();
    config.backends.kimi.directory = work.clone();
    config.backends.hermes.directory = work.clone();
    config.backends.codebuddy.directory = work.clone();
    config.backends.codex.directory = work.clone();
    config.backends.claude.directory = work.clone();
    config.backends.pi.directory = work;
    config
}

/// Build one backend's real profile the way the Gateway does: an owned
/// service, the catalogued permission mode, and no executable pre-resolved
/// (`MissingFinder`), which is exactly the case that falls back to each
/// driver's own catalogued default command.
fn build(id: &str, config: &Config, permission_mode: &str) -> via_backends::BackendProfile {
    create_backend_profile(
        id,
        &LaunchContext {
            config,
            env: &EnvMap::new(),
            ownership: Ownership::Owned,
            permission_mode,
            owner_id: "user_personal",
            finder: &MissingFinder,
        },
    )
    .unwrap_or_else(|error| panic!("{id} builds: {error}"))
}

#[test]
fn the_launch_specs_excerpt_matches_the_catalogue_or_its_documented_divergence() {
    let value = &expect_contract(CONTRACT_KIND, CONTRACT_NAME).exact_value;
    let config = config();

    // ── five backends: VIA's real spawn spec matches the excerpt directly ──

    // qwen: cliPath + ['--acp']
    assert!(value.contains("qwen: cliPath + ['--acp']"), "{value}");
    let qwen = build("qwen", &config, "native");
    assert_eq!(qwen.acp.spawn.command, "qwen");
    assert_eq!(qwen.acp.spawn.args, ["--acp"]);

    // hermes: ['acp','--accept-hooks']
    assert!(
        value.contains("hermes: ['acp','--accept-hooks']"),
        "{value}"
    );
    let hermes = build("hermes", &config, "native");
    assert_eq!(hermes.acp.spawn.command, "hermes");
    assert_eq!(hermes.acp.spawn.args, ["acp", "--accept-hooks"]);

    // qoder full: ['--acp','--dangerously-skip-permissions']
    assert!(
        value.contains("qoder full: ['--acp','--dangerously-skip-permissions']"),
        "{value}"
    );
    let qoder = build("qoder", &config, "full");
    assert_eq!(qoder.acp.spawn.command, "qodercli");
    assert_eq!(
        qoder.acp.spawn.args,
        ["--acp", "--dangerously-skip-permissions"]
    );

    // kimi: ['acp'] with sessionConfigOptions [{id:'mode',value:'auto'}]
    assert!(
        value.contains("kimi: ['acp'] with sessionConfigOptions [{id:'mode',value:'auto'}]"),
        "{value}"
    );
    let kimi = build("kimi", &config, "full");
    assert_eq!(kimi.acp.spawn.command, "kimi");
    assert_eq!(kimi.acp.spawn.args, ["acp"]);
    assert_eq!(kimi.session_config_options.len(), 1);
    assert_eq!(kimi.session_config_options[0].id, "mode");
    assert_eq!(kimi.session_config_options[0].value, "auto");

    // codebuddy full: ['--acp','--model',<m>,'--dangerously-skip-permissions']
    assert!(
        value.contains("codebuddy full: ['--acp','--model',<m>,'--dangerously-skip-permissions']"),
        "{value}"
    );
    let mut with_model = config.clone();
    with_model.backends.codebuddy.model = "example-model".to_owned();
    let codebuddy = build("codebuddy", &with_model, "full");
    assert_eq!(codebuddy.acp.spawn.command, "codebuddy");
    assert_eq!(
        codebuddy.acp.spawn.args,
        [
            "--acp",
            "--model",
            "example-model",
            "--dangerously-skip-permissions"
        ]
    );

    // openclaw's arguments and OPENCLAW_STATE_DIR match the excerpt directly;
    // only its launcher prefix (checked below) diverges.
    assert!(
        value.contains("'acp','--url','ws://127.0.0.1:18789','--token-file',<path>,'--verbose'"),
        "{value}"
    );
    assert!(value.contains("env.OPENCLAW_STATE_DIR"), "{value}");
    let openclaw = build("openclaw", &config, "native");
    assert_eq!(
        openclaw.acp.spawn.args,
        [
            "acp",
            "--url",
            "ws://127.0.0.1:18789",
            "--token-file",
            "/config/backends/openclaw/state/gateway-token",
            "--verbose",
        ]
    );
    assert_eq!(
        openclaw.acp.spawn.env.get("OPENCLAW_STATE_DIR"),
        Some("/config/backends/openclaw/state")
    );

    // ── five backends: the launcher prefix is a documented divergence ──
    //
    // docs/deviations/phase-2.md: "The six shim-launched backends launch the
    // executable the shim would have exec'd, not `process.execPath
    // scripts/<name>.mjs`: `opencode acp`, `openclaw acp --url … --verbose`,
    // `codex-acp`, `claude-code-acp`, `pi-acp`, `dsh-acp-demo --config
    // <cordis.yml>`." Five of those six are in this excerpt (the sixth,
    // DeepSeek Harness, is not); `docs/fidelity.md` — *"Adapted, with the
    // reason"* records the same adaptation.

    // The catalogue still records the upstream Node-shim launcher...
    assert!(
        value.contains("opencode: [<root>/scripts/opencode.mjs,'acp']"),
        "{value}"
    );
    assert!(
        value.contains(
            "codex/claude/pi: process.execPath + <root>/scripts/{codex-acp,claude-code-acp,pi-acp}.mjs"
        ),
        "{value}"
    );
    assert!(
        value.contains("openclaw: [<root>/scripts/openclaw.mjs,'acp'"),
        "{value}"
    );

    // ...but VIA has no Node interpreter to shim through, so opencode and
    // openclaw (already built above) run their own binary directly...
    assert_eq!(
        build("opencode", &config, "native").acp.spawn.command,
        "opencode"
    );
    assert_eq!(openclaw.acp.spawn.command, "openclaw");

    // ...and codex, claude and pi — with no `cli_path` override and nothing on
    // the finder's table — each fall back to the catalog's own adapter
    // command: `codex-acp`, `claude-code-acp`, `pi-acp`, exactly the binaries
    // the upstream shims would have `exec`'d, with no extra arguments, since
    // the adapter itself takes none.
    for (id, adapter) in [
        ("codex", "codex-acp"),
        ("claude", "claude-code-acp"),
        ("pi", "pi-acp"),
    ] {
        let built = build(id, &config, "native");
        assert_eq!(built.acp.spawn.command, adapter, "{id}");
        assert!(built.acp.spawn.args.is_empty(), "{id}");
    }
}
