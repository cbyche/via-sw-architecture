//! The read-only setup report: is this backend usable, and if not, why not.
//!
//! Ported from `shared/backend-setup.mjs`. Upstream's own footer states the
//! contract: *"Backend Agent setup (read-only — this installs nothing, signs in
//! to nothing, and changes no configuration)"*. Nothing in this module writes a
//! file, spawns an installer or reads a credential; the most it does is ask a
//! binary for its version.
//!
//! # Three independent questions
//!
//! A backend is ready when all three answer yes, and the report keeps them
//! apart because they have different remedies:
//!
//! 1. **The backend itself.** Found, or explicitly configured, or downloadable
//!    on demand — and new enough where a minimum is declared.
//! 2. **Its ACP adapter**, for the backends VIA reaches through one.
//! 3. **Its runtime components**, for the one composed backend that declares
//!    `verifyInstalledPackages`.
//!
//! `state-name/backend integration modes and configuration modes` pins the
//! discriminants a CLI and a settings page branch on, including the one that is
//! always the same value: `authentication` is the literal `backend-managed`,
//! because authentication is never VIA's.

use std::collections::BTreeMap;

use via_catalog::backend::{ConfigurationMode, Integration};
use via_catalog::{
    BackendDefinition, backend_definition, backend_definitions, normalize_backend_protocol,
};
use via_core::EnvMap;
use via_i18n::{Locale, format, keys, t};

use crate::detect::{ExecutableFinder, environment_value};
use crate::install::{InstallStepSpec, StepKind, install_steps, package_identity, step_package};
use crate::onboarding::uses_automatic_bailian_configuration;
use crate::platform::HostPlatform;

/// `VIA_DESKTOP_INSTALLED_ONLY` — upstream
/// `QWEN_AUDIO_AGENT_DESKTOP_INSTALLED_ONLY`.
///
/// **External contract** — `env-var/QWEN_AUDIO_AGENT_DESKTOP_INSTALLED_ONLY`:
/// *"`'1'` forbids every npx package-mode fallback … turning a missing adapter
/// into a fatal error … a policy switch that a Rust port must keep even though
/// the desktop app itself is out of scope — it also serves air-gapped/offline
/// installs."*
pub const DESKTOP_INSTALLED_ONLY: &str = via_acp::env::DESKTOP_INSTALLED_ONLY;

/// The value that turns it on.
pub const DESKTOP_INSTALLED_ONLY_ON: &str = "1";

/// The package runner a `managed` source is started through.
pub const PACKAGE_RUNNER: &str = "npx";

/// `OPENCODE_RUNTIME`'s accepted values — `shared/backend-setup.mjs:186-188`.
pub const OPENCODE_RUNTIMES: [&str; 5] = ["auto", "binary", "installed", "package", "source"];

/// `OPENCLAW_RUNTIME`'s accepted values — `shared/backend-setup.mjs:229-236`.
///
/// One more than OpenCode's: OpenClaw also ships an enterprise `bundle`.
pub const OPENCLAW_RUNTIMES: [&str; 6] =
    ["auto", "binary", "installed", "bundle", "package", "source"];

/// Where a usable executable came from.
///
/// **External contract** — the `source` discriminants
/// `shared/backend-setup.mjs` renders and desktop settings branch on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// Found on `PATH`.
    Installed,
    /// At a path the operator named.
    Configured,
    /// Downloaded on demand by the package runner.
    Managed,
    /// An explicit `package` runtime.
    Package,
    /// Built from a source checkout.
    SourceTree,
    /// The enterprise bundle.
    Bundle,
    /// The backend speaks ACP itself, so there is no adapter to find.
    Native,
    /// Reached through a bridge process.
    Bridge,
    /// A user-supplied ACP command.
    Generic,
}

impl Source {
    const fn from_integration(integration: Integration) -> Self {
        match integration {
            Integration::Native => Self::Native,
            Integration::Bridge => Self::Bridge,
            Integration::Adapter => Self::Installed,
            Integration::Generic => Self::Generic,
        }
    }
}

/// How a backend's model configuration is described in the report.
///
/// **External contract** — `state-name/backend integration modes and
/// configuration modes`: *"inspectBackend configuration output:
/// `command-managed | automatic-bailian | preserved`"*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationReport {
    /// The generic ACP backend: whatever the command does.
    CommandManaged,
    /// VIA writes a Bailian configuration.
    AutomaticBailian,
    /// The user's own configuration is reused untouched.
    Preserved,
}

/// The one value the report's `authentication` field ever takes.
///
/// **External contract** — `shared/backend-setup.mjs:499`: *"authentication
/// output is always the literal `backend-managed`"*. It is a field rather than
/// a constant because a settings page renders it, and it says the thing worth
/// saying: VIA never holds a backend's credentials.
pub const AUTHENTICATION_BACKEND_MANAGED: &str = "backend-managed";

/// One resolved component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    /// Whether it checks out.
    pub ready: bool,
    /// Where it came from.
    pub source: Option<Source>,
    /// The resolved path, when there is one.
    pub path: String,
    /// The observed version, when one was read.
    pub version: String,
    /// Why it does not check out.
    pub issue: Option<String>,
}

impl Component {
    fn ready(source: Source, path: impl Into<String>) -> Self {
        Self {
            ready: true,
            source: Some(source),
            path: path.into(),
            version: String::new(),
            issue: None,
        }
    }

    fn missing(issue: String) -> Self {
        Self {
            ready: false,
            source: None,
            path: String::new(),
            version: String::new(),
            issue: Some(issue),
        }
    }
}

/// One expected runtime package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageStatus {
    /// The package name, without its version.
    pub name: String,
    /// The pinned version the spec asks for.
    pub expected_version: String,
    /// The version actually installed, when npm could be asked.
    pub version: String,
    /// Whether it satisfies the spec.
    pub ready: bool,
}

/// One backend's whole setup report.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendSetup {
    /// The catalogued id.
    pub id: &'static str,
    /// The catalogued label.
    pub label: &'static str,
    /// Whether this is the configured backend.
    pub selected: bool,
    /// Whether every component checks out.
    pub ready: bool,
    /// The backend executable.
    pub backend: Component,
    /// Its ACP adapter.
    pub adapter: Component,
    /// Its runtime components.
    pub packages: Vec<PackageStatus>,
    /// How VIA talks to it.
    pub integration: Integration,
    /// How its model configuration is described.
    pub configuration: ConfigurationReport,
    /// Always [`AUTHENTICATION_BACKEND_MANAGED`].
    pub authentication: &'static str,
    /// Every reason it is not ready, in upstream's order: backend, adapter,
    /// packages.
    pub issues: Vec<String>,
    /// The platform this was inspected for.
    pub platform: &'static str,
}

/// The whole report.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupReport {
    /// The configured backend id, or empty.
    pub selected: String,
    /// Always `true`: this report changes nothing.
    pub read_only: bool,
    /// One entry per inspected backend.
    pub backends: Vec<BackendSetup>,
}

/// Reading an executable's `--version`.
pub trait VersionReader: Send + Sync {
    /// The first line of `<command> --version`, or empty.
    fn version(&self, command: &str) -> String;
}

/// A reader that never has an answer, for a report that does not need versions.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoVersions;

impl VersionReader for NoVersions {
    fn version(&self, _command: &str) -> String {
        String::new()
    }
}

/// What `npm list -g --depth=0 --json` reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstalledPackages {
    /// Whether npm could be asked at all.
    ///
    /// `false` makes every package check pass: an unanswerable question is not
    /// evidence of a missing component (`shared/backend-setup.mjs:200-201`).
    pub known: bool,
    /// Installed name → version.
    pub versions: BTreeMap<String, String>,
}

/// Reading npm's global package list.
pub trait GlobalPackages: Send + Sync {
    /// Ask npm. `command` is empty when npm was not found.
    fn installed(&self, command: &str) -> InstalledPackages;
}

/// A reader that never has an answer.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPackages;

impl GlobalPackages for NoPackages {
    fn installed(&self, _command: &str) -> InstalledPackages {
        InstalledPackages::default()
    }
}

/// Everything an inspection needs.
pub struct SetupInspection<'a> {
    /// The Gateway environment.
    pub env: &'a EnvMap,
    /// The platform.
    pub platform: HostPlatform,
    /// The configured backend, or empty to read `AGENT_PROTOCOL`.
    pub backend: &'a str,
    /// Locating executables.
    pub finder: &'a dyn ExecutableFinder,
    /// Reading versions.
    pub versions: &'a dyn VersionReader,
    /// Reading npm's global list.
    pub packages: &'a dyn GlobalPackages,
    /// The locale every issue is rendered in.
    pub locale: Locale,
}

/// `inspectBackendSetups(options)` — `shared/backend-setup.mjs:530-556`.
#[must_use]
pub fn inspect_backend_setups(inspection: &SetupInspection<'_>) -> SetupReport {
    let selected = if inspection.backend.trim().is_empty() {
        normalize_backend_protocol(inspection.env.get_trimmed("AGENT_PROTOCOL"))
    } else {
        normalize_backend_protocol(inspection.backend)
    };
    let ids: Vec<&'static str> = if inspection.backend.trim().is_empty() {
        backend_definitions()
            .iter()
            .map(|definition| definition.id)
            .collect()
    } else {
        backend_definition(&selected)
            .map(|definition| vec![definition.id])
            .unwrap_or_default()
    };
    SetupReport {
        backends: ids
            .into_iter()
            .filter_map(|id| backend_definition(id))
            .map(|definition| inspect_backend(definition, inspection, &selected))
            .collect(),
        selected,
        read_only: true,
    }
}

/// `inspectBackend(id, options)` — `shared/backend-setup.mjs:365-528`.
fn inspect_backend(
    definition: &'static BackendDefinition,
    inspection: &SetupInspection<'_>,
    selected: &str,
) -> BackendSetup {
    let env = inspection.env;
    let locale = inspection.locale;
    let spec = &definition.setup;
    let configured = if let Some(name) = spec.command_environment {
        environment_value(env, &[name])
    } else {
        environment_value(env, spec.executable_environment)
    };
    let command = if configured.1.is_empty() {
        spec.command.unwrap_or_default().to_owned()
    } else {
        configured.1.clone()
    };
    let runtime_variable = runtime_variable(definition.id);
    let runtime = runtime_variable
        .map(|name| runtime_value(env, name))
        .unwrap_or_default();
    let automatic_bailian = uses_automatic_bailian_configuration(definition.id, env);
    let installed_only = env.get_trimmed(DESKTOP_INSTALLED_ONLY) == DESKTOP_INSTALLED_ONLY_ON;

    let mut backend = explicit_runtime(definition.id, inspection).unwrap_or_else(|| {
        let path = inspection.finder.find(&command);
        let mut resolved = if path.is_empty() {
            Component::missing(if !configured.1.is_empty() {
                format(
                    locale,
                    keys::SETUP_CONFIGURED_COMMAND_UNAVAILABLE,
                    &[("variable", configured.0), ("value", &configured.1)],
                )
            } else if let Some(name) = spec.command_environment {
                format(
                    locale,
                    keys::SETUP_SET_COMMAND_ENVIRONMENT,
                    &[("variable", name)],
                )
            } else {
                format(
                    locale,
                    keys::SETUP_BACKEND_NOT_FOUND,
                    &[("label", definition.label)],
                )
            })
        } else {
            Component::ready(
                if configured.1.is_empty() {
                    Source::Installed
                } else {
                    Source::Configured
                },
                path,
            )
        };
        // `shared/backend-setup.mjs:395-410` — the two service backends can
        // download themselves at startup, but only when VIA has something to
        // configure them with.
        if !resolved.ready
            && !installed_only
            && matches!(definition.id, "opencode" | "openclaw")
            && runtime == "auto"
        {
            let runner = inspection.finder.find(PACKAGE_RUNNER);
            if !runner.is_empty() && automatic_bailian {
                resolved = Component::ready(Source::Managed, runner);
            } else if !runner.is_empty() {
                resolved.issue = Some(format(
                    locale,
                    keys::SETUP_BACKEND_NOT_FOUND_AUTO_DEPLOY,
                    &[
                        ("label", definition.label),
                        (
                            "requirements",
                            t(locale, keys::SETUP_AUTO_DEPLOY_REQUIREMENTS),
                        ),
                    ],
                ));
            }
        }
        resolved
    });

    apply_version_gate(
        definition,
        inspection,
        &mut backend,
        automatic_bailian,
        installed_only,
        &runtime,
    );

    let adapter = if backend.ready || spec.inspect_adapter_independently {
        inspect_adapter(definition, inspection, installed_only)
    } else {
        Component {
            ready: spec.integration != Integration::Adapter,
            source: Some(Source::from_integration(spec.integration)),
            path: String::new(),
            version: String::new(),
            issue: None,
        }
    };

    let (packages, package_issue) = inspect_packages(definition, inspection, &backend, &adapter);
    let issues: Vec<String> = [
        backend.issue.clone(),
        adapter.issue.clone(),
        package_issue.clone(),
    ]
    .into_iter()
    .flatten()
    .collect();

    BackendSetup {
        id: definition.id,
        label: definition.label,
        selected: definition.id == selected,
        ready: backend.ready && adapter.ready && package_issue.is_none(),
        backend,
        adapter,
        packages,
        integration: spec.integration,
        configuration: if definition.id == "acp" {
            ConfigurationReport::CommandManaged
        } else if automatic_bailian {
            ConfigurationReport::AutomaticBailian
        } else {
            ConfigurationReport::Preserved
        },
        authentication: AUTHENTICATION_BACKEND_MANAGED,
        issues,
        platform: inspection.platform.as_str(),
    }
}

/// The version gate — `shared/backend-setup.mjs:412-450`.
///
/// OpenCode is the special case: a version below the minimum is *recoverable*
/// by falling back to a downloaded copy, so it is only fatal when there is
/// nothing to fall back to. Every other backend with a minimum simply fails.
fn apply_version_gate(
    definition: &BackendDefinition,
    inspection: &SetupInspection<'_>,
    backend: &mut Component,
    automatic_bailian: bool,
    installed_only: bool,
    runtime: &str,
) {
    let locale = inspection.locale;
    let Some(minimum) = definition.setup.minimum_version else {
        return;
    };
    if definition.id == "opencode" {
        if !(backend.ready && backend.source == Some(Source::Installed)) {
            return;
        }
        let version = inspection.versions.version(&backend.path);
        backend.version = version.clone();
        if version_at_least(&version, minimum) {
            return;
        }
        let runner = if !installed_only && runtime == "auto" {
            inspection.finder.find(PACKAGE_RUNNER)
        } else {
            String::new()
        };
        if !runner.is_empty() && automatic_bailian {
            *backend = Component::ready(Source::Managed, runner);
            backend.version = version;
            return;
        }
        backend.ready = false;
        backend.issue = Some(if runner.is_empty() {
            if version.is_empty() {
                format(
                    locale,
                    keys::SETUP_VERSION_UNCONFIRMED,
                    &[("label", definition.label)],
                )
            } else {
                format(
                    locale,
                    keys::SETUP_VERSION_BELOW_MINIMUM,
                    &[
                        ("label", definition.label),
                        ("version", &version),
                        ("minimum", minimum),
                    ],
                )
            }
        } else {
            format(
                locale,
                keys::SETUP_VERSION_INCOMPATIBLE_AUTO_DEPLOY,
                &[
                    (
                        "version",
                        if version.is_empty() {
                            t(locale, keys::SETUP_VERSION_UNKNOWN)
                        } else {
                            &version
                        },
                    ),
                    (
                        "requirements",
                        t(locale, keys::SETUP_AUTO_DEPLOY_REQUIREMENTS),
                    ),
                ],
            )
        });
        return;
    }
    if !backend.ready {
        return;
    }
    let version = inspection.versions.version(&backend.path);
    backend.version = version.clone();
    if version_at_least(&version, minimum) {
        return;
    }
    backend.ready = false;
    backend.issue = Some(if version.is_empty() {
        format(
            locale,
            keys::SETUP_VERSION_UNCONFIRMED,
            &[("label", definition.label)],
        )
    } else {
        format(
            locale,
            keys::SETUP_VERSION_BELOW_MINIMUM,
            &[
                ("label", definition.label),
                ("version", &version),
                ("minimum", minimum),
            ],
        )
    });
}

/// `inspectAdapter(spec, env, find)` — `shared/backend-setup.mjs:295-345`.
fn inspect_adapter(
    definition: &BackendDefinition,
    inspection: &SetupInspection<'_>,
    installed_only: bool,
) -> Component {
    let spec = &definition.setup;
    let locale = inspection.locale;
    let Some(adapter_command) = spec.adapter_command else {
        return Component {
            ready: true,
            source: Some(Source::from_integration(spec.integration)),
            path: String::new(),
            version: String::new(),
            issue: None,
        };
    };
    let runtime = spec
        .adapter_runtime_environment
        .map(|name| runtime_value(inspection.env, name))
        .unwrap_or_else(|| "auto".to_owned());
    let configured = environment_value(
        inspection.env,
        &spec.adapter_environment.into_iter().collect::<Vec<_>>(),
    );
    if !["auto", "binary", "package"].contains(&runtime.as_str()) {
        return Component::missing(format(
            locale,
            keys::SETUP_UNSUPPORTED_RUNTIME,
            &[
                (
                    "variable",
                    spec.adapter_runtime_environment.unwrap_or_default(),
                ),
                ("runtime", &runtime),
            ],
        ));
    }
    if runtime == "package" {
        if installed_only {
            return Component::missing(format(
                locale,
                keys::SETUP_ADAPTER_REQUIRED_DESKTOP,
                &[("command", adapter_command)],
            ));
        }
        let runner = inspection.finder.find(PACKAGE_RUNNER);
        return if runner.is_empty() {
            Component::missing(t(locale, keys::SETUP_ADAPTER_PACKAGE_MODE_NEEDS_NPX).to_owned())
        } else {
            Component::ready(Source::Managed, runner)
        };
    }
    if !configured.1.is_empty() {
        let path = inspection.finder.find(&configured.1);
        return if path.is_empty() {
            Component::missing(format(
                locale,
                keys::SETUP_CONFIGURED_ADAPTER_UNAVAILABLE,
                &[("variable", configured.0)],
            ))
        } else {
            Component::ready(Source::Installed, path)
        };
    }
    let installed = inspection.finder.find(adapter_command);
    if !installed.is_empty() {
        return Component::ready(Source::Installed, installed);
    }
    if runtime == "binary" {
        return Component::missing(format(
            locale,
            keys::SETUP_ADAPTER_UNAVAILABLE,
            &[("command", adapter_command)],
        ));
    }
    let runner = if installed_only || !spec.managed_adapter_fallback {
        String::new()
    } else {
        inspection.finder.find(PACKAGE_RUNNER)
    };
    if !runner.is_empty() {
        return Component::ready(Source::Managed, runner);
    }
    Component::missing(if installed_only {
        format(
            locale,
            keys::SETUP_ADAPTER_MISSING_DESKTOP,
            &[("command", adapter_command)],
        )
    } else if !spec.managed_adapter_fallback {
        format(
            locale,
            keys::SETUP_ADAPTER_MISSING,
            &[("command", adapter_command)],
        )
    } else {
        format(
            locale,
            keys::SETUP_ADAPTER_MISSING_NO_NPX,
            &[("command", adapter_command)],
        )
    })
}

/// The package-set check — `shared/backend-setup.mjs:452-486`.
fn inspect_packages(
    definition: &BackendDefinition,
    inspection: &SetupInspection<'_>,
    backend: &Component,
    adapter: &Component,
) -> (Vec<PackageStatus>, Option<String>) {
    let verifies = definition.lifecycle.installation.is_some_and(|_| {
        install_steps(definition.id, inspection.platform).verify_installed_packages
    });
    if !verifies || !(backend.ready || adapter.ready) {
        return (Vec::new(), None);
    }
    let npm = npm_command(inspection);
    let installed = inspection.packages.installed(&npm);
    let statuses: Vec<PackageStatus> = install_steps(definition.id, inspection.platform)
        .steps
        .into_iter()
        .filter(|step: &&InstallStepSpec| step.kind == StepKind::Npm)
        .map(|step| {
            let (name, expected) = package_identity(&step_package(step, inspection.env));
            let actual = installed.versions.get(&name).cloned().unwrap_or_default();
            let ready = !installed.known
                || (!actual.is_empty() && (expected.is_empty() || actual == expected));
            PackageStatus {
                name,
                expected_version: expected,
                version: actual,
                ready,
            }
        })
        .collect();
    let missing: Vec<&str> = statuses
        .iter()
        .filter(|status| !status.ready)
        .map(|status| status.name.as_str())
        .collect();
    let issue = if missing.is_empty() {
        None
    } else {
        Some(format(
            inspection.locale,
            keys::SETUP_RUNTIME_PACKAGES_MISSING,
            &[(
                "names",
                &missing.join(t(inspection.locale, keys::BACKEND_CHOICES_JOIN)),
            )],
        ))
    };
    (statuses, issue)
}

/// `find(platform === 'win32' ? 'npm.cmd' : 'npm') || find('npm')` —
/// `shared/backend-setup.mjs:461`.
fn npm_command(inspection: &SetupInspection<'_>) -> String {
    let primary = if inspection.platform.is_windows() {
        "npm.cmd"
    } else {
        "npm"
    };
    let found = inspection.finder.find(primary);
    if found.is_empty() {
        inspection.finder.find("npm")
    } else {
        found
    }
}

/// `explicitRuntime(id, env, find)` — `shared/backend-setup.mjs:183-293`.
///
/// Only OpenCode and OpenClaw have one. `None` means *"no explicit runtime
/// applies; fall through to the ordinary PATH lookup"*.
fn explicit_runtime(id: &str, inspection: &SetupInspection<'_>) -> Option<Component> {
    let env = inspection.env;
    let locale = inspection.locale;
    let find = |command: &str| inspection.finder.find(command);
    match id {
        "opencode" => {
            let runtime = runtime_value(env, "OPENCODE_RUNTIME");
            if !OPENCODE_RUNTIMES.contains(&runtime.as_str()) {
                return Some(Component::missing(format(
                    locale,
                    keys::SETUP_UNSUPPORTED_RUNTIME,
                    &[("variable", "OPENCODE_RUNTIME"), ("runtime", &runtime)],
                )));
            }
            let configured_bin = env.get_trimmed("OPENCODE_BIN");
            if runtime == "package" {
                return Some(package_mode(&find(PACKAGE_RUNNER), locale));
            }
            if runtime == "binary" || (runtime == "auto" && !configured_bin.is_empty()) {
                let path = find(configured_bin);
                return Some(if path.is_empty() {
                    Component::missing(t(locale, keys::SETUP_OPENCODE_BIN_UNAVAILABLE).to_owned())
                } else {
                    Component::ready(Source::Configured, path)
                });
            }
            let source_dir = env.get_trimmed("OPENCODE_SOURCE_DIR");
            if runtime == "source" || !source_dir.is_empty() {
                // Upstream additionally stats the entry point and the bundled
                // TUI package; both live under the same directory, and the
                // directory is what the report shows.
                let bun = find(
                    non_empty(env.get_trimmed("BUN_BIN"))
                        .unwrap_or_else(|| "bun".to_owned())
                        .as_str(),
                );
                return Some(if source_dir.is_empty() || bun.is_empty() {
                    Component::missing(
                        t(locale, keys::SETUP_OPENCODE_SOURCE_UNAVAILABLE).to_owned(),
                    )
                } else {
                    Component::ready(Source::SourceTree, source_dir)
                });
            }
            if runtime == "installed" {
                let path = find("opencode");
                return Some(if path.is_empty() {
                    Component::missing(t(locale, keys::SETUP_OPENCODE_NOT_ON_PATH).to_owned())
                } else {
                    Component::ready(Source::Installed, path)
                });
            }
            None
        }
        "openclaw" => {
            let runtime = runtime_value(env, "OPENCLAW_RUNTIME");
            if !OPENCLAW_RUNTIMES.contains(&runtime.as_str()) {
                return Some(Component::missing(format(
                    locale,
                    keys::SETUP_UNSUPPORTED_RUNTIME,
                    &[("variable", "OPENCLAW_RUNTIME"), ("runtime", &runtime)],
                )));
            }
            let configured_bin = env.get_trimmed("OPENCLAW_BIN");
            if runtime == "package" {
                return Some(package_mode(&find(PACKAGE_RUNNER), locale));
            }
            if runtime == "binary" || (runtime == "auto" && !configured_bin.is_empty()) {
                let path = find(configured_bin);
                return Some(if path.is_empty() {
                    Component::missing(t(locale, keys::SETUP_OPENCLAW_BIN_UNAVAILABLE).to_owned())
                } else {
                    Component::ready(Source::Configured, path)
                });
            }
            let source_dir = env.get_trimmed("OPENCLAW_SOURCE_DIR");
            if runtime == "source" || !source_dir.is_empty() {
                let corepack = find("corepack");
                return Some(if source_dir.is_empty() || corepack.is_empty() {
                    Component::missing(
                        t(locale, keys::SETUP_OPENCLAW_SOURCE_UNAVAILABLE).to_owned(),
                    )
                } else {
                    Component::ready(Source::SourceTree, source_dir)
                });
            }
            let bundle = non_empty(env.get_trimmed("OPENCLAW_BUNDLE_BIN")).unwrap_or_else(|| {
                std::format!(
                    "{}/.openclaw-bundle/wrapper/openclaw",
                    env.get_trimmed("HOME")
                )
            });
            if ["auto", "bundle"].contains(&runtime.as_str()) {
                let path = find(&bundle);
                if !path.is_empty() {
                    return Some(Component::ready(Source::Bundle, path));
                }
            }
            if runtime == "bundle" {
                return Some(Component::missing(
                    t(locale, keys::SETUP_OPENCLAW_BUNDLE_UNAVAILABLE).to_owned(),
                ));
            }
            if runtime == "installed" {
                let path = find("openclaw");
                return Some(if path.is_empty() {
                    Component::missing(t(locale, keys::SETUP_OPENCLAW_NOT_ON_PATH).to_owned())
                } else {
                    Component::ready(Source::Installed, path)
                });
            }
            None
        }
        _ => None,
    }
}

fn package_mode(runner: &str, locale: Locale) -> Component {
    if runner.is_empty() {
        Component::missing(t(locale, keys::SETUP_PACKAGE_MODE_NEEDS_NPX).to_owned())
    } else {
        Component::ready(Source::Package, runner)
    }
}

const fn runtime_variable(id: &str) -> Option<&'static str> {
    match id.as_bytes() {
        b"opencode" => Some("OPENCODE_RUNTIME"),
        b"openclaw" => Some("OPENCLAW_RUNTIME"),
        _ => None,
    }
}

/// `clean(env[name] || 'auto').toLowerCase()`.
fn runtime_value(env: &EnvMap, name: &str) -> String {
    let value = env.get_trimmed(name);
    if value.is_empty() {
        "auto".to_owned()
    } else {
        value.to_lowercase()
    }
}

/// `versionTuple` + `versionAtLeast` — `shared/backend-setup.mjs:93-107`.
///
/// A version that does not contain a `<major>.<minor>.<patch>` triple is
/// **not** at least the minimum: an unreadable version is treated as too old,
/// which is the safe direction when the alternative is running an incompatible
/// agent.
#[must_use]
pub fn version_at_least(actual: &str, minimum: &str) -> bool {
    match (version_tuple(actual), version_tuple(minimum)) {
        (Some(left), Some(right)) => left >= right,
        _ => false,
    }
}

/// `/(\d+)\.(\d+)\.(\d+)/` — the first such triple anywhere in the string.
fn version_tuple(value: &str) -> Option<[u64; 3]> {
    let bytes: Vec<char> = value.trim().chars().collect();
    let mut index = 0usize;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let mut cursor = index;
        let mut parts = [0u64; 3];
        let mut parsed = 0usize;
        let mut valid = true;
        while parsed < 3 {
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if cursor == start {
                valid = false;
                break;
            }
            let number: String = bytes[start..cursor].iter().collect();
            match number.parse::<u64>() {
                Ok(value) => parts[parsed] = value,
                Err(_) => {
                    valid = false;
                    break;
                }
            }
            parsed += 1;
            if parsed < 3 {
                if bytes.get(cursor) == Some(&'.') {
                    cursor += 1;
                } else {
                    valid = false;
                    break;
                }
            }
        }
        if valid && parsed == 3 {
            return Some(parts);
        }
        index += 1;
    }
    None
}

fn non_empty(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.trim().to_owned())
    }
}

/// `formatBackendSetup(report)` — `shared/backend-setup.mjs:602-634`.
///
/// **External contract** — `prompt-text/setup human report header/footer`. The
/// three footer lines say the three things a person needs: VIA does not touch
/// your model, it does not touch your credentials, and two backends can bring
/// themselves.
#[must_use]
pub fn format_backend_setup(report: &SetupReport, locale: Locale) -> String {
    let mut lines = vec![
        t(locale, keys::SETUP_REPORT_HEADER).to_owned(),
        match backend_definition(&report.selected) {
            Some(definition) => {
                format(locale, keys::SETUP_SELECTED, &[("label", definition.label)])
            }
            None if report.selected.is_empty() => t(locale, keys::SETUP_SELECTED_NONE).to_owned(),
            None => format(locale, keys::SETUP_SELECTED, &[("label", &report.selected)]),
        },
        String::new(),
    ];
    for item in &report.backends {
        let details = if item.ready {
            vec![
                backend_text(item, locale),
                integration_text(item, locale).to_owned(),
                configuration_text(item, locale),
            ]
        } else {
            item.issues.clone()
        };
        lines.push(std::format!(
            "{} {}{}",
            if item.ready { "✓" } else { "✗" },
            item.label,
            if item.selected {
                t(locale, keys::SETUP_CURRENT_MARKER)
            } else {
                ""
            }
        ));
        lines.push(std::format!("  {}", details.join(" · ")));
        lines.push(String::new());
    }
    lines.push(t(locale, keys::SETUP_FOOTER_MODELS).to_owned());
    lines.push(t(locale, keys::SETUP_FOOTER_AUTO_DOWNLOAD).to_owned());
    lines.push(t(locale, keys::SETUP_FOOTER_OTHER_BACKENDS).to_owned());
    lines.join("\n")
}

fn backend_text(item: &BackendSetup, locale: Locale) -> String {
    match item.backend.source {
        _ if !item.backend.ready => item.backend.issue.clone().unwrap_or_default(),
        Some(Source::Managed) => format(
            locale,
            keys::SETUP_MODE_NPX_AUTO_DOWNLOAD,
            &[("path", &item.backend.path)],
        ),
        Some(Source::Package) => format(
            locale,
            keys::SETUP_MODE_EXPLICIT_PACKAGE,
            &[("path", &item.backend.path)],
        ),
        Some(Source::SourceTree) => format(
            locale,
            keys::SETUP_MODE_SOURCE,
            &[("path", &item.backend.path)],
        ),
        Some(Source::Bundle) => std::format!("Bundle（{}）", item.backend.path),
        _ => item.backend.path.clone(),
    }
}

fn integration_text(item: &BackendSetup, locale: Locale) -> &'static str {
    match item.integration {
        Integration::Native => t(locale, keys::SETUP_MODE_NATIVE_ACP),
        Integration::Bridge => t(locale, keys::SETUP_MODE_BUILTIN_BRIDGE),
        Integration::Generic => t(locale, keys::SETUP_MODE_USER_COMMAND),
        Integration::Adapter if item.adapter.source == Some(Source::Managed) => {
            t(locale, keys::SETUP_MODE_ADAPTER_ON_DEMAND)
        }
        Integration::Adapter => t(locale, keys::SETUP_MODE_ADAPTER_INSTALLED),
    }
}

fn configuration_text(item: &BackendSetup, locale: Locale) -> String {
    match item.configuration {
        ConfigurationReport::AutomaticBailian => t(locale, keys::SETUP_AUTO_CONFIGURE_DASHSCOPE),
        ConfigurationReport::Preserved => t(locale, keys::SETUP_REUSE_USER_CONFIG),
        ConfigurationReport::CommandManaged => t(locale, keys::SETUP_CONFIG_MANAGED_BY_ACP_AGENT),
    }
    .to_owned()
}

/// The configuration mode a backend declares, restated for a caller that has an
/// id rather than a definition.
#[must_use]
pub fn configuration_mode(id: &str) -> Option<ConfigurationMode> {
    backend_definition(id).map(|definition| definition.lifecycle.configuration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::MissingFinder;
    use std::collections::BTreeMap;

    #[derive(Debug, Default)]
    struct MapFinder(BTreeMap<String, String>);

    impl ExecutableFinder for MapFinder {
        fn find(&self, command: &str) -> String {
            self.0.get(command).cloned().unwrap_or_default()
        }
    }

    fn finder(pairs: &[(&str, &str)]) -> MapFinder {
        MapFinder(
            pairs
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
        )
    }

    struct FixedVersion(&'static str);
    impl VersionReader for FixedVersion {
        fn version(&self, _command: &str) -> String {
            self.0.to_owned()
        }
    }

    fn env(pairs: &[(&str, &str)]) -> EnvMap {
        pairs.iter().copied().collect()
    }

    fn inspection<'a>(
        environment: &'a EnvMap,
        found: &'a dyn ExecutableFinder,
        versions: &'a dyn VersionReader,
        backend: &'a str,
    ) -> SetupInspection<'a> {
        SetupInspection {
            env: environment,
            platform: HostPlatform::Linux,
            backend,
            finder: found,
            versions,
            packages: &NoPackages,
            locale: Locale::Zh,
        }
    }

    #[test]
    fn version_comparison_treats_an_unreadable_version_as_too_old() {
        assert!(version_at_least("1.18.5", "1.18.0"));
        assert!(version_at_least("opencode 1.18.0", "1.18.0"));
        assert!(!version_at_least("1.17.9", "1.18.0"));
        assert!(!version_at_least("", "1.18.0"));
        assert!(!version_at_least("unknown", "1.18.0"));
        assert!(version_at_least("2.0.0", "1.99.99"));
    }

    #[test]
    fn a_backend_on_path_is_ready_and_reports_where_it_came_from() {
        let environment = env(&[]);
        let found = finder(&[("qwen", "/usr/bin/qwen")]);
        let report = inspect_backend_setups(&inspection(
            &environment,
            &found,
            &FixedVersion("0.21.6"),
            "qwen",
        ));
        let item = &report.backends[0];
        assert!(item.ready, "{:?}", item.issues);
        assert_eq!(item.backend.source, Some(Source::Installed));
        assert_eq!(item.backend.path, "/usr/bin/qwen");
        assert_eq!(item.authentication, AUTHENTICATION_BACKEND_MANAGED);
        assert_eq!(item.configuration, ConfigurationReport::Preserved);
        assert!(report.read_only);
    }

    #[test]
    fn a_backend_below_its_minimum_version_is_not_ready() {
        let environment = env(&[]);
        let found = finder(&[("qwen", "/usr/bin/qwen")]);
        let report = inspect_backend_setups(&inspection(
            &environment,
            &found,
            &FixedVersion("0.20.0"),
            "qwen",
        ));
        let item = &report.backends[0];
        assert!(!item.ready);
        assert_eq!(item.issues[0], "Qwen Code 0.20.0 低于最低版本 0.21.6");
    }

    #[test]
    fn an_unreadable_version_says_so_rather_than_naming_a_number() {
        let environment = env(&[]);
        let found = finder(&[("qwen", "/usr/bin/qwen")]);
        let report =
            inspect_backend_setups(&inspection(&environment, &found, &FixedVersion(""), "qwen"));
        assert_eq!(report.backends[0].issues[0], "无法确认 Qwen Code 版本");
    }

    #[test]
    fn the_generic_acp_backend_asks_for_its_command_variable() {
        let environment = env(&[]);
        let report = inspect_backend_setups(&inspection(
            &environment,
            &MissingFinder,
            &NoVersions,
            "acp",
        ));
        let item = &report.backends[0];
        assert!(!item.ready);
        assert_eq!(item.issues[0], "请设置 ACP_COMMAND");
        assert_eq!(item.configuration, ConfigurationReport::CommandManaged);
    }

    #[test]
    fn opencode_only_offers_to_download_itself_when_it_can_be_configured() {
        let bare = env(&[]);
        let found = finder(&[("npx", "/usr/bin/npx")]);
        let report = inspect_backend_setups(&inspection(&bare, &found, &NoVersions, "opencode"));
        assert!(!report.backends[0].ready);
        assert_eq!(
            report.backends[0].issues[0],
            "未找到 OpenCode；自动部署需要 DASHSCOPE_API_KEY 和 VIA_BACKEND_MODEL"
        );

        let configured = env(&[
            ("DASHSCOPE_API_KEY", "key"),
            ("VIA_BACKEND_MODEL", "qwen3.7-max"),
        ]);
        let report =
            inspect_backend_setups(&inspection(&configured, &found, &NoVersions, "opencode"));
        assert!(report.backends[0].ready, "{:?}", report.backends[0].issues);
        assert_eq!(report.backends[0].backend.source, Some(Source::Managed));
        assert_eq!(
            report.backends[0].configuration,
            ConfigurationReport::AutomaticBailian
        );
    }

    #[test]
    fn the_desktop_installed_only_switch_forbids_every_package_fallback() {
        let environment = env(&[
            ("DASHSCOPE_API_KEY", "key"),
            ("VIA_BACKEND_MODEL", "qwen3.7-max"),
            (DESKTOP_INSTALLED_ONLY, "1"),
        ]);
        let found = finder(&[("npx", "/usr/bin/npx")]);
        let report =
            inspect_backend_setups(&inspection(&environment, &found, &NoVersions, "opencode"));
        assert!(!report.backends[0].ready);
        assert_eq!(
            report.backends[0].issues[0],
            "未找到 OpenCode，请先安装并完成原生配置"
        );
    }

    #[test]
    fn an_unsupported_runtime_value_is_named_in_the_issue() {
        let environment = env(&[("OPENCLAW_RUNTIME", "docker")]);
        let report = inspect_backend_setups(&inspection(
            &environment,
            &MissingFinder,
            &NoVersions,
            "openclaw",
        ));
        assert_eq!(
            report.backends[0].issues[0],
            "不支持的 OPENCLAW_RUNTIME：docker"
        );
    }

    #[test]
    fn deepseek_inspects_its_adapter_even_when_the_backend_is_missing() {
        // `inspectAdapterIndependently` — DeepSeek is the one backend whose
        // adapter is a separate composition worth reporting on its own.
        let environment = env(&[]);
        let found = finder(&[("npx", "/usr/bin/npx")]);
        let report =
            inspect_backend_setups(&inspection(&environment, &found, &NoVersions, "deepseek"));
        let item = &report.backends[0];
        assert!(!item.backend.ready);
        // `managedAdapterFallback: false` — npx must not stand in for it.
        assert!(!item.adapter.ready);
        assert_eq!(
            item.adapter.issue.as_deref(),
            Some("缺少 ACP Adapter dsh-acp-demo")
        );
    }

    #[test]
    fn an_adapter_on_path_is_preferred_over_the_package_runner() {
        let environment = env(&[]);
        let found = finder(&[
            ("codex", "/usr/bin/codex"),
            ("codex-acp", "/usr/bin/codex-acp"),
            ("npx", "/usr/bin/npx"),
        ]);
        let report =
            inspect_backend_setups(&inspection(&environment, &found, &NoVersions, "codex"));
        let item = &report.backends[0];
        assert!(item.ready, "{:?}", item.issues);
        assert_eq!(item.adapter.source, Some(Source::Installed));
        assert_eq!(item.adapter.path, "/usr/bin/codex-acp");
    }

    #[test]
    fn a_missing_adapter_falls_back_to_the_package_runner() {
        let environment = env(&[]);
        let found = finder(&[("codex", "/usr/bin/codex"), ("npx", "/usr/bin/npx")]);
        let report =
            inspect_backend_setups(&inspection(&environment, &found, &NoVersions, "codex"));
        assert_eq!(report.backends[0].adapter.source, Some(Source::Managed));
    }

    #[test]
    fn the_whole_catalog_is_inspected_when_no_backend_is_named() {
        let environment = env(&[("AGENT_PROTOCOL", "codex")]);
        let report =
            inspect_backend_setups(&inspection(&environment, &MissingFinder, &NoVersions, ""));
        assert_eq!(report.backends.len(), 12);
        assert_eq!(report.selected, "codex");
        assert!(report.backends.iter().any(|item| item.selected));
    }

    #[test]
    fn the_human_report_carries_its_header_and_three_footers() {
        let environment = env(&[]);
        let report =
            inspect_backend_setups(&inspection(&environment, &MissingFinder, &NoVersions, ""));
        let text = format_backend_setup(&report, Locale::Zh);
        assert!(text.starts_with("后台 Agent Setup（只读，不会安装、登录或修改配置）"));
        assert!(text.contains("当前选择：未设置"));
        assert!(
            text.contains("默认不覆盖后台模型；认证由后台 Agent 管理，此命令不会输出或验证凭据。")
        );
        assert!(text.contains(
            "OpenCode 和 OpenClaw 可在启动时自动下载；配置百炼 API Key 与后台模型后可一键接入。"
        ));
        assert!(text.contains("其他后台请先确认在原生终端中可以正常工作。"));
    }

    #[test]
    fn an_unanswerable_npm_makes_every_package_pass() {
        let environment = env(&[]);
        let found = finder(&[
            ("dsh", "/usr/bin/dsh"),
            ("dsh-acp-demo", "/usr/bin/dsh-acp-demo"),
        ]);
        let report =
            inspect_backend_setups(&inspection(&environment, &found, &NoVersions, "deepseek"));
        let item = &report.backends[0];
        assert_eq!(item.packages.len(), 11);
        assert!(item.packages.iter().all(|status| status.ready));
        assert!(item.ready, "{:?}", item.issues);
    }
}
