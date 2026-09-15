//! `apps/via`'s remaining Pending rows, closed against the shipped binary.
//!
//! Most of these describe the npm package `qwen-audio-agent` ships as, and
//! the third-party-embedding surface it publishes alongside the CLI —
//! `docs/fidelity.md`'s Runtime table names the whole category by its most
//! visible member: *"npm global install, `npx` shims, `scripts/*-acp.mjs`" →
//! "A self-contained binary … a Rust binary is its own installer."*
//! `docs/architecture.md` §10 says the same thing from the other side: *"Node's
//! `npm install -g`, the `npx` shims … disappear."* VIA ships one binary with
//! no package manifest, no `files[]` allow-list and no subpath-import surface,
//! so these rows are `Divergent` against that record rather than `Pending` —
//! each test below asserts VIA's real, shipped replacement alongside the
//! upstream literal it does not reproduce.
//!
//! Two rows are `Divergent` against a different record —
//! `docs/deviations/phase-1.md`'s *"Six verbs, not upstream's eight"*, which
//! names `webui` and `skill` as dropped outright — and one, `client requires a
//! Gateway`, is fully `Asserted`: VIA kept the sentence, renamed only the
//! binary it names.

use std::path::{Path, PathBuf};

use via_conformance::expect_contract;
use via_i18n::{Locale, keys, t};

/// Every backtick-quoted run in `text`, in order.
///
/// The catalogue writes several of these contracts as prose with the literal
/// sentence backtick-quoted inside it, exactly the way `apps/via`'s own
/// `tests/support` reads them — this is the same extraction, local to this
/// file because via-conformance's `value` module does not need it anywhere
/// else.
fn backticked(text: &str) -> Vec<&str> {
    text.split('`').skip(1).step_by(2).collect()
}

/// Read a file relative to the repository root.
fn read_repo_file(relative: &str) -> String {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "..", relative]
        .iter()
        .collect();
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", Path::new(&path).display()))
}

#[test]
fn client_requires_a_gateway_is_the_rebranded_sentence_verbatim() {
    let contract = expect_contract("error-code", "client requires a Gateway");
    let template = backticked(&contract.exact_value)
        .into_iter()
        .next()
        .expect("the catalogue quotes the sentence");
    assert!(template.contains("qwenaudio gateway"));

    // `docs/rebrand.md` row 136: only the binary name is renamed.
    let expected = template.replace("qwenaudio", via::BINARY_NAME);
    assert_ne!(expected, template, "the rename must not be a no-op");

    let shipped = t(Locale::Zh, keys::CLI_GATEWAY_NOT_RUNNING).replace("{url}", "<url>");
    assert_eq!(
        shipped, expected,
        "VIA's shipped sentence must be upstream's, with only the binary renamed",
    );
}

#[test]
fn node_runtime_pin_has_no_via_equivalent_by_design() {
    let contract = expect_contract("default-value", "Node runtime pin");
    assert!(contract.exact_value.contains(".node-version"));
    assert!(contract.exact_value.contains("22.22.2"));

    // VIA's real replacement: a pinned Rust toolchain, not a pinned Node one —
    // read from the real files rather than retyped, so a future re-pin is
    // caught here too.
    let manifest = read_repo_file("Cargo.toml");
    assert!(
        manifest.contains("rust-version = \"1.94\""),
        "VIA's own toolchain pin moved",
    );
    let toolchain = read_repo_file("rust-toolchain.toml");
    assert!(
        toolchain.contains("channel = \"1.94.0\""),
        "VIA's own toolchain pin moved",
    );
    assert!(
        !manifest.contains(".node-version") && !manifest.contains("engines"),
        "no Node toolchain is pinned anywhere in VIA's own manifest",
    );
}

#[test]
fn npm_package_identity_has_no_via_equivalent_by_design() {
    let contract = expect_contract("file-path", "npm package identity");
    assert!(contract.exact_value.contains("qwen-audio-agent"));
    assert!(contract.exact_value.contains("workspaces"));

    // VIA's real value: one binary crate, not an npm package with workspace
    // members per surface.
    let manifest = read_repo_file("apps/via/Cargo.toml");
    assert!(manifest.contains("name = \"via\""));
    let root = read_repo_file("Cargo.toml");
    assert!(root.contains(r#"members = ["crates/*", "apps/*"]"#));
    assert!(
        !root.contains("qwen-audio-agent"),
        "the upstream package name must not survive into VIA's own manifest",
    );
}

#[test]
fn npm_packaging_surface_is_a_self_contained_binary() {
    let contract = expect_contract("file-path", "npm packaging surface");
    assert!(
        contract
            .exact_value
            .contains("npm install -g qwen-audio-agent")
    );
    assert!(contract.exact_value.contains("registry.npmjs.org"));

    // VIA's real value: `publish = false`, no registry, no install command —
    // `cargo build --release` produces the one artifact there is.
    let manifest = read_repo_file("apps/via/Cargo.toml");
    assert!(manifest.contains("publish = false"));
    assert!(
        !manifest.contains("registry.npmjs.org") && !manifest.contains("publishConfig"),
        "no package registry is configured for a binary nothing publishes",
    );
}

#[test]
fn packaged_config_assets_are_compiled_in_not_packaged() {
    let contract = expect_contract("file-path", "packaged config assets (package.json files[])");
    assert!(contract.exact_value.contains("config/frontend-agent/"));

    // VIA's real value: the frontend-agent documents are `include_str!`'d
    // straight into `via-voice` — there is no `files[]` allow-list because
    // there is no separate packaging step to allow-list into.
    assert!(!via_voice::prompt::PACKAGED_PROMPT_ZH.is_empty());
    assert!(!via_voice::prompt::PACKAGED_ASSISTANT_ZH.is_empty());
    assert!(!via_voice::prompt::PACKAGED_PROMPT_EN.is_empty());
    assert!(!via_voice::prompt::PACKAGED_PROMPT_KO.is_empty());
    let manifest = read_repo_file("apps/via/Cargo.toml");
    assert!(
        !manifest.contains("files = ["),
        "a Cargo manifest has no packaging allow-list to compare against",
    );
}

#[test]
fn npm_package_bin_and_exports_has_no_via_equivalent_by_design() {
    let contract = expect_contract("package-identity", "npm package, bin and exports");
    assert!(contract.exact_value.contains("Contract subpaths"));
    assert!(contract.exact_value.contains("gateway-process"));

    // VIA's real value: the binary name survives the rename, but there is no
    // subpath-export surface at all — a third party cannot `require` half of
    // a compiled binary the way it could `require('qwen-audio-agent/electron')`.
    assert_eq!(via::BINARY_NAME, "via");
    let manifest = read_repo_file("apps/via/Cargo.toml");
    assert!(!manifest.contains("exports"));
    assert!(!manifest.contains("workspaces"));
}

#[test]
fn webui_url_shape_is_dropped_with_the_web_ui() {
    let contract = expect_contract("file-path", "WebUI URL shape");
    assert!(contract.exact_value.contains("qwenaudio WebUI: <url>"));

    // VIA's real value: `webui` is not a verb this binary parses at all —
    // `docs/deviations/phase-1.md` records it plainly: "webui and skill
    // dropped".
    let parsed = via::cli::parse_from(["webui"]);
    assert!(
        parsed.is_err(),
        "`webui` must not parse; VIA ships no web UI for it to point at",
    );
}

#[test]
fn skill_install_follow_up_has_no_via_equivalent() {
    let contract = expect_contract("prompt-text", "skill install follow-up");
    assert!(contract.exact_value.contains("技能已安装"));
    assert!(contract.exact_value.contains("qwenaudio gateway restart"));

    // VIA's real value: the `skill` verb does not parse at all —
    // `docs/deviations/phase-1.md`'s "webui and skill dropped" — so there is
    // no follow-up sentence to print and no catalog key that carries one.
    let parsed = via::cli::parse_from(["skill", "install", "x"]);
    assert!(
        parsed.is_err(),
        "`skill install` must not parse; VIA carries no skill verb",
    );
}

#[test]
fn foreground_gateway_banner_drops_the_webui_line() {
    let contract = expect_contract("prompt-text", "foreground gateway banner");
    assert!(contract.exact_value.contains("Gateway 已启动：<url>"));
    assert!(contract.exact_value.contains("WebUI：<url>/"));
    assert!(contract.exact_value.contains("Gateway 已在运行：<url>"));

    // VIA's real value: the "started" line is reproduced verbatim (with
    // `{url}` in place of `<url>`, `via-i18n`'s own placeholder syntax), but
    // the WebUI line has nothing to point at and never prints —
    // `docs/deviations/phase-1.md` records `webui` itself as dropped, and
    // `apps/via/src/gateway/mod.rs::serve` builds the banner from this one key
    // alone.
    let started = t(Locale::Zh, keys::CLI_GATEWAY_BANNER_STARTED);
    assert_eq!(started, "Gateway 已启动：{url}\n");
    assert!(
        !started.contains("WebUI"),
        "the WebUI line must not have crept back into the started banner",
    );
}
