//! The baseline computer-use MCP server injected into every backend session.
//!
//! A port of upstream `server/src/agent/builtin-mcp.mjs:1-76`. Its own header
//! explains why it exists, and the reasoning survives the port intact:
//!
//! > ACP treats stdio as the baseline MCP transport: descriptors passed via
//! > `session/new` are spawned and connected by the backend Agent itself, so
//! > the Gateway only needs to describe where the server lives. […]
//! > open-computer-use ships three platform runtimes in one npm package and
//! > provides click/type/screenshot-style tools, giving every backend a
//! > computer-use baseline even when the user has not configured one.
//!
//! So VIA does **not** run this server. It names it, and the backend spawns
//! it. That is the whole of this module's job, plus the toggle that suppresses
//! it and the process cleanup in [`crate::lifecycle`].
//!
//! # Finding the executable is the one part that could not be ported
//!
//! Upstream resolves the package through Node's own module resolution
//! (`require.resolve('@qwen-code/open-computer-use/package.json')`) and then
//! spawns `process.execPath` — the Node or Electron binary the Gateway itself
//! is running under — with the package's `bin` script as `argv[1]`.
//! `docs/architecture.md` §10 removes that whole mechanism: VIA is a Rust
//! binary, it has no interpreter to publish and no `node_modules` tree of its
//! own. `via-backends` already took the same decision for the six shim-launched
//! backends — *"launch the executable the shim would have exec'd"*
//! (`docs/deviations/phase-2.md`).
//!
//! What is portable is everything below module resolution, and it is ported:
//! [`resolve_package_bin`] reads a package manifest's `bin` field in both of
//! its shapes, and [`externally_readable`] is the archive-mirror rewrite.
//! *Which* directory to look in becomes a [`ComputerUseLocator`], so the
//! decision is data rather than a `require` VIA cannot make. [`PathLocator`]
//! is the shipped default; [`NodePackageLocator`] reproduces upstream's exact
//! descriptor for an embedder that does have a Node install to point at.

use std::fmt;
use std::path::{Component, Path, PathBuf};

use agent_client_protocol::schema::v1::{EnvVariable, McpServer, McpServerStdio};
use serde_json::Value;
use via_core::EnvMap;
use via_downstream::BackendCapabilities;

/// The environment variable that switches the baseline computer-use server
/// off.
///
/// **External contract** — upstream `builtin-mcp.mjs:54`, renamed by
/// `docs/rebrand.md` row 105. Note the unusual default: **unset means
/// enabled.**
pub const COMPUTER_USE_ENV: &str = "VIA_COMPUTER_USE";

/// The npm package the executable comes from.
///
/// **KEEP** under `docs/rebrand.md`: someone else's package name, published
/// under someone else's scope. VIA does not rename it and does not vendor it.
pub const COMPUTER_USE_PACKAGE: &str = "@qwen-code/open-computer-use";

/// The `bin` entry inside that package, and the MCP server name the model
/// sees.
///
/// **KEEP** and **external contract** — `builtin-mcp.mjs:57,61`. It is the
/// namespace a backend prefixes the computer-use tools with, so it is
/// model-visible for a third-party tool surface VIA does not own.
pub const COMPUTER_USE_SERVER_NAME: &str = "open-computer-use";

/// The single argument the executable is launched with.
///
/// **External contract** — `builtin-mcp.mjs:63`: `args: [binPath, 'mcp']`.
pub const COMPUTER_USE_MCP_ARGUMENT: &str = "mcp";

/// The environment variable upstream stamps so an Electron-hosted interpreter
/// behaves as plain Node.
///
/// **External contract** — `builtin-mcp.mjs:66`. VIA stamps it only when the
/// launch actually is an interpreter running a script; see
/// [`ComputerUseLaunch::node_runtime`].
pub const ELECTRON_RUN_AS_NODE: &str = "ELECTRON_RUN_AS_NODE";

/// The values that read as "off".
///
/// **External contract** — upstream `settingEnabled`,
/// `builtin-mcp.mjs:22-26`. Compared after `String(value ?? '').trim()
/// .toLowerCase()`, so `OFF` and ` Disabled ` are both off.
pub const DISABLED_VALUES: [&str; 5] = ["false", "off", "0", "no", "disabled"];

/// The lifecycle marker upstream hangs off the descriptor as a hidden
/// `Symbol`.
///
/// **Renamed** by `docs/rebrand.md` row 140. In JavaScript it has to be a
/// property, because a descriptor is a plain object and the lifecycle needs to
/// know which descriptors it manages; here that is [`BuiltinMcpServer::kind`],
/// a typed field the wire never sees. The string survives as the tracing name
/// the lifecycle logs under, so the rename still has something to be true of.
pub const BUILTIN_MCP_LIFECYCLE: &str = "via.builtin-mcp-lifecycle";

/// `settingEnabled(value, fallback)`.
///
/// **External contract** — `builtin-mcp.mjs:22-26`. Empty or absent takes the
/// fallback; anything in [`DISABLED_VALUES`] is off; **everything else is
/// on**, including values that look like nonsense. That is deliberate
/// upstream: a mistyped toggle leaves the feature working rather than silently
/// removing every backend's computer-use tools.
#[must_use]
pub fn setting_enabled(value: Option<&str>, fallback: bool) -> bool {
    let normalized = value.unwrap_or_default().trim().to_lowercase();
    if normalized.is_empty() {
        return fallback;
    }
    !DISABLED_VALUES.contains(&normalized.as_str())
}

/// Whether the baseline computer-use server is enabled in this environment.
#[must_use]
pub fn computer_use_enabled(env: &EnvMap) -> bool {
    setting_enabled(env.get(COMPUTER_USE_ENV), true)
}

/// Rewrite a path inside an application archive to its unpacked mirror.
///
/// **External contract** — upstream `externallyReadable`,
/// `builtin-mcp.mjs:31-36`: `${sep}app.asar${sep}` becomes
/// `${sep}app.asar.unpacked${sep}`. Backend agents are separate processes that
/// cannot read inside an archive, so a descriptor pointing into one describes
/// a server that will never start.
///
/// VIA has no Electron packaging today, so on the shipped path this is the
/// identity. It is ported because [`NodePackageLocator`] can be pointed at an
/// installation that does, and because leaving it out would make that
/// silently broken rather than obviously absent.
#[must_use]
pub fn externally_readable(path: &Path) -> PathBuf {
    let mut rewritten = PathBuf::new();
    let mut changed = false;
    for component in path.components() {
        if component == Component::Normal("app.asar".as_ref()) {
            rewritten.push("app.asar.unpacked");
            changed = true;
        } else {
            rewritten.push(component);
        }
    }
    if changed {
        rewritten
    } else {
        path.to_path_buf()
    }
}

/// The executable a package manifest's `bin` field names, if it exists on
/// disk.
///
/// **External contract** — upstream `resolvePackageBin`,
/// `builtin-mcp.mjs:38-51`. `bin` has two shapes — a bare string, or a map
/// from bin name to path — and upstream reads both. The path is resolved
/// against the package directory, put through [`externally_readable`], and
/// then **checked for existence**: a manifest may name a binary the install
/// never produced, and a descriptor pointing at a missing file is worse than
/// no descriptor.
///
/// Every failure is `None`, exactly as upstream's bare `catch {}` is.
#[must_use]
pub fn resolve_package_bin(package_directory: &Path, bin_name: &str) -> Option<PathBuf> {
    let manifest = std::fs::read_to_string(package_directory.join("package.json")).ok()?;
    let manifest: Value = serde_json::from_str(&manifest).ok()?;
    let relative = match manifest.get("bin") {
        Some(Value::String(single)) => single.as_str(),
        Some(Value::Object(map)) => map.get(bin_name).and_then(Value::as_str)?,
        _ => return None,
    };
    let candidate = externally_readable(&package_directory.join(relative));
    candidate.exists().then_some(candidate)
}

/// How the computer-use server is launched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerUseLaunch {
    /// The executable.
    pub command: PathBuf,
    /// Arguments before [`COMPUTER_USE_MCP_ARGUMENT`] — the script path, when
    /// [`Self::command`] is an interpreter.
    pub leading_args: Vec<String>,
    /// Whether [`Self::command`] is a Node-compatible interpreter running a
    /// script.
    ///
    /// The one thing [`ELECTRON_RUN_AS_NODE`] can act on. `via-backends`
    /// records the same reasoning for the six shim-launched backends: setting
    /// it on a command that is not an interpreter would change the behaviour
    /// of any Electron tool that command later spawns.
    pub node_runtime: bool,
}

impl ComputerUseLaunch {
    /// A plain executable on `PATH`.
    #[must_use]
    pub fn executable(command: impl Into<PathBuf>) -> Self {
        Self {
            command: command.into(),
            leading_args: Vec::new(),
            node_runtime: false,
        }
    }

    /// An interpreter running a script, as upstream launches it.
    #[must_use]
    pub fn node_script(interpreter: impl Into<PathBuf>, script: &Path) -> Self {
        Self {
            command: interpreter.into(),
            leading_args: vec![script.to_string_lossy().into_owned()],
            node_runtime: true,
        }
    }

    /// The full argument vector, `mcp` last.
    #[must_use]
    pub fn args(&self) -> Vec<String> {
        let mut args = self.leading_args.clone();
        args.push(COMPUTER_USE_MCP_ARGUMENT.to_owned());
        args
    }

    /// The environment entries the descriptor carries.
    ///
    /// An **array of `{name,value}`**, not a map — the catalogue flags this
    /// encoding as easy to get wrong, which is why the SDK's own
    /// [`EnvVariable`] is used rather than a hand-built object.
    #[must_use]
    pub fn env(&self) -> Vec<EnvVariable> {
        if self.node_runtime {
            vec![EnvVariable::new(ELECTRON_RUN_AS_NODE, "1")]
        } else {
            Vec::new()
        }
    }

    /// The ACP `mcpServers` entry.
    ///
    /// **External contract** — `json-field / builtin stdio MCP descriptor`:
    /// `{"name":"open-computer-use","command":…,"args":[…,"mcp"],
    /// "env":[{"name":"ELECTRON_RUN_AS_NODE","value":"1"}]}`. The stdio
    /// variant is serialized untagged, so — as upstream — there is **no
    /// `type` field**, unlike the HTTP descriptor in [`crate::server`].
    #[must_use]
    pub fn descriptor(&self) -> McpServer {
        McpServer::Stdio(
            McpServerStdio::new(COMPUTER_USE_SERVER_NAME, self.command.clone())
                .args(self.args())
                .env(self.env()),
        )
    }
}

/// Where the computer-use executable is.
///
/// The seam that replaces Node's `require.resolve`. Implementations do
/// filesystem work, so they are not called on a hot path: upstream resolves
/// once, at adapter construction.
pub trait ComputerUseLocator: fmt::Debug + Send + Sync {
    /// The launch, or `None` when the package is not installed.
    fn locate(&self) -> Option<ComputerUseLaunch>;
}

/// The shipped default: the first `open-computer-use` on `PATH`.
///
/// The same `findExecutable` walk `via-backends` uses to find a backend CLI,
/// including the `PATHEXT` handling Windows needs.
#[derive(Debug, Clone, Copy, Default)]
pub struct PathLocator;

impl ComputerUseLocator for PathLocator {
    fn locate(&self) -> Option<ComputerUseLaunch> {
        which::which(COMPUTER_USE_SERVER_NAME)
            .ok()
            .map(ComputerUseLaunch::executable)
    }
}

/// Upstream's own shape: an interpreter plus a resolved package `bin`.
///
/// For an embedder that ships a Node runtime and a `node_modules` tree — the
/// desktop packaging upstream was written for. The descriptor it produces is
/// byte-identical to upstream's, `ELECTRON_RUN_AS_NODE` included.
#[derive(Debug, Clone)]
pub struct NodePackageLocator {
    /// The interpreter — upstream's `process.execPath`.
    pub interpreter: PathBuf,
    /// The directory holding the package's `package.json`.
    pub package_directory: PathBuf,
}

impl ComputerUseLocator for NodePackageLocator {
    fn locate(&self) -> Option<ComputerUseLaunch> {
        let script = resolve_package_bin(&self.package_directory, COMPUTER_USE_SERVER_NAME)?;
        Some(ComputerUseLaunch::node_script(
            self.interpreter.clone(),
            &script,
        ))
    }
}

/// A locator that answers with whatever it was built from. **Test seam.**
#[derive(Debug, Clone, Default)]
pub struct FixedLocator(pub Option<ComputerUseLaunch>);

impl ComputerUseLocator for FixedLocator {
    fn locate(&self) -> Option<ComputerUseLaunch> {
        self.0.clone()
    }
}

/// Which builtin server a descriptor is, for the lifecycle's benefit.
///
/// Upstream's hidden `Symbol` payload, `{ kind: 'open-computer-use' }`
/// (`builtin-mcp.mjs:68-70`), as a type. There is one variant because upstream
/// ships one builtin server; the enum exists so adding a second cannot make
/// the lifecycle silently manage it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BuiltinMcpKind {
    /// The baseline computer-use server.
    OpenComputerUse,
}

impl BuiltinMcpKind {
    /// The `kind` string upstream stores on the marker.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenComputerUse => COMPUTER_USE_SERVER_NAME,
        }
    }
}

/// One builtin MCP server: what goes on the wire, and what it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltinMcpServer {
    /// The ACP `mcpServers` entry.
    pub descriptor: McpServer,
    /// The lifecycle marker.
    pub kind: BuiltinMcpKind,
}

/// `computerUseMcpServer(env)`.
///
/// **External contract** — upstream `builtin-mcp.mjs:53-72`. `None` when the
/// toggle is off, and `None` when the executable cannot be found: an absent
/// package is not a failure, it is a Gateway without a computer-use baseline.
#[must_use]
pub fn computer_use_mcp_server(
    env: &EnvMap,
    locator: &dyn ComputerUseLocator,
) -> Option<BuiltinMcpServer> {
    if !computer_use_enabled(env) {
        return None;
    }
    locator.locate().map(|launch| BuiltinMcpServer {
        descriptor: launch.descriptor(),
        kind: BuiltinMcpKind::OpenComputerUse,
    })
}

/// `builtinMcpServers(env)` — every builtin server, disabled entries filtered
/// out.
///
/// **External contract** — upstream `builtin-mcp.mjs:74-76`.
#[must_use]
pub fn builtin_mcp_servers(
    env: &EnvMap,
    locator: &dyn ComputerUseLocator,
) -> Vec<BuiltinMcpServer> {
    computer_use_mcp_server(env, locator).into_iter().collect()
}

/// The builtin servers a harness with these capabilities may be given.
///
/// **External contract** — upstream `acp-backend-adapter.mjs:191-193`:
/// `this.profile.sessionMcp === false ? [] : builtinMcp`. A backend whose
/// adapter drops `mcpServers` is given none rather than a descriptor it will
/// silently discard.
///
/// The two readings coincide for all twelve shipped backends, because
/// `via-downstream` already enforces that `sessionMcp` implies `externalMcp`.
#[must_use]
pub fn builtin_mcp_servers_for(
    capabilities: BackendCapabilities,
    env: &EnvMap,
    locator: &dyn ComputerUseLocator,
) -> Vec<BuiltinMcpServer> {
    if capabilities.session_mcp {
        builtin_mcp_servers(env, locator)
    } else {
        Vec::new()
    }
}

/// Just the descriptors, ready for `session/new`'s `mcpServers`.
#[must_use]
pub fn descriptors(servers: &[BuiltinMcpServer]) -> Vec<McpServer> {
    servers
        .iter()
        .map(|server| server.descriptor.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(value: Option<&str>) -> EnvMap {
        value.map_or_else(EnvMap::new, |value| {
            [(COMPUTER_USE_ENV, value)].into_iter().collect()
        })
    }

    fn locator() -> FixedLocator {
        FixedLocator(Some(ComputerUseLaunch::executable(
            "/usr/local/bin/open-computer-use",
        )))
    }

    #[test]
    fn the_toggle_defaults_to_on() {
        assert!(setting_enabled(None, true));
        assert!(setting_enabled(Some(""), true));
        assert!(setting_enabled(Some("   "), true));
        assert!(computer_use_enabled(&EnvMap::new()));
    }

    #[test]
    fn every_catalogued_disabled_value_is_off_case_insensitively() {
        for value in DISABLED_VALUES {
            assert!(!setting_enabled(Some(value), true), "{value}");
            assert!(
                !setting_enabled(Some(&value.to_uppercase()), true),
                "{value}"
            );
            assert!(
                !setting_enabled(Some(&format!("  {value}  ")), true),
                "{value}"
            );
        }
    }

    #[test]
    fn a_value_that_is_not_a_disabled_word_stays_on() {
        for value in ["true", "on", "1", "yes", "maybe", "off!"] {
            assert!(setting_enabled(Some(value), true), "{value}");
        }
    }

    #[test]
    fn the_descriptor_is_the_catalogued_shape() {
        let servers = builtin_mcp_servers(&env(None), &locator());
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].kind, BuiltinMcpKind::OpenComputerUse);
        let json = serde_json::to_value(&servers[0].descriptor).expect("json");
        assert_eq!(json["name"], COMPUTER_USE_SERVER_NAME);
        assert_eq!(json["command"], "/usr/local/bin/open-computer-use");
        assert_eq!(json["args"], serde_json::json!(["mcp"]));
        assert_eq!(json["env"], serde_json::json!([]));
        // Stdio is the untagged variant: no `type` discriminator, unlike the
        // HTTP descriptor.
        assert!(json.get("type").is_none());
    }

    #[test]
    fn a_node_launch_reproduces_upstreams_descriptor_exactly() {
        let launch =
            ComputerUseLaunch::node_script("/opt/node", Path::new("/pkg/bin/open-computer-use.js"));
        let json = serde_json::to_value(launch.descriptor()).expect("json");
        assert_eq!(json["command"], "/opt/node");
        assert_eq!(
            json["args"],
            serde_json::json!(["/pkg/bin/open-computer-use.js", "mcp"]),
        );
        assert_eq!(
            json["env"],
            serde_json::json!([{ "name": "ELECTRON_RUN_AS_NODE", "value": "1" }]),
        );
    }

    #[test]
    fn a_disabled_toggle_filters_the_entry_out() {
        assert!(builtin_mcp_servers(&env(Some("off")), &locator()).is_empty());
        assert!(computer_use_mcp_server(&env(Some("0")), &locator()).is_none());
    }

    #[test]
    fn an_unresolvable_package_is_no_descriptor_not_a_failure() {
        assert!(builtin_mcp_servers(&env(None), &FixedLocator(None)).is_empty());
    }

    #[test]
    fn a_backend_that_drops_mcp_servers_is_given_none() {
        let capabilities = |session_mcp: bool| BackendCapabilities {
            delegation: session_mcp,
            permissions: true,
            backend_ui: false,
            native_session_history: true,
            external_mcp: true,
            native_delegation: false,
            session_mcp,
        };
        assert!(builtin_mcp_servers_for(capabilities(false), &env(None), &locator()).is_empty());
        assert_eq!(
            builtin_mcp_servers_for(capabilities(true), &env(None), &locator()).len(),
            1,
        );
    }

    #[test]
    fn the_archive_mirror_is_rewritten() {
        assert_eq!(
            externally_readable(Path::new("/A/app.asar/node_modules/x/bin.js")),
            Path::new("/A/app.asar.unpacked/node_modules/x/bin.js"),
        );
        // Only a whole path component, never a substring.
        assert_eq!(
            externally_readable(Path::new("/A/app.asar.unpacked/x")),
            Path::new("/A/app.asar.unpacked/x"),
        );
        assert_eq!(
            externally_readable(Path::new("/A/notapp.asar/x")),
            Path::new("/A/notapp.asar/x"),
        );
    }

    #[test]
    fn a_bin_field_is_read_in_both_shapes_and_the_file_must_exist() {
        let dir = tempfile::TempDir::new().expect("a temp dir");
        let root = dir.path();
        std::fs::write(root.join("bin.js"), "//").expect("write");

        std::fs::write(root.join("package.json"), r#"{"bin":"bin.js"}"#).expect("write");
        assert_eq!(
            resolve_package_bin(root, COMPUTER_USE_SERVER_NAME),
            Some(root.join("bin.js")),
        );

        std::fs::write(
            root.join("package.json"),
            r#"{"bin":{"open-computer-use":"bin.js","other":"nope.js"}}"#,
        )
        .expect("write");
        assert_eq!(
            resolve_package_bin(root, COMPUTER_USE_SERVER_NAME),
            Some(root.join("bin.js")),
        );

        std::fs::write(root.join("package.json"), r#"{"bin":{"other":"nope.js"}}"#).expect("write");
        assert_eq!(resolve_package_bin(root, COMPUTER_USE_SERVER_NAME), None);

        std::fs::write(root.join("package.json"), r#"{"bin":"missing.js"}"#).expect("write");
        assert_eq!(resolve_package_bin(root, COMPUTER_USE_SERVER_NAME), None);

        std::fs::write(root.join("package.json"), "not json").expect("write");
        assert_eq!(resolve_package_bin(root, COMPUTER_USE_SERVER_NAME), None);

        assert_eq!(
            resolve_package_bin(
                Path::new("/nonexistent-package-root"),
                COMPUTER_USE_SERVER_NAME
            ),
            None,
        );
    }

    #[test]
    fn descriptors_projects_the_wire_form_and_drops_the_marker() {
        let servers = builtin_mcp_servers(&env(None), &locator());
        let wire = descriptors(&servers);
        assert_eq!(wire.len(), servers.len());
        assert_eq!(wire[0], servers[0].descriptor);
        // The lifecycle marker is a typed field, so it cannot reach the wire
        // the way upstream's `Symbol` property could not.
        let json = serde_json::to_value(&wire[0]).expect("json");
        assert!(json.get("kind").is_none(), "{json}");
        assert!(descriptors(&[]).is_empty());
    }

    #[test]
    fn the_lifecycle_marker_kind_is_the_server_name() {
        assert_eq!(
            BuiltinMcpKind::OpenComputerUse.as_str(),
            COMPUTER_USE_SERVER_NAME,
        );
    }
}
