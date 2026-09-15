//! Shared test scaffolding: the contract catalogue, name-free backend
//! fixtures, and recording collaborators.
//!
//! Two rules hold everywhere in this crate's tests.
//!
//! **No contract value is retyped.** Every number, code and environment name a
//! test asserts is parsed out of `docs/reference/contracts.json`. A retyped
//! literal proves only that two copies of the same mistake agree.
//!
//! **No backend is named.** `via-process` may not know that any particular
//! backend exists (`docs/architecture.md` §17 item 8), and a fixture in its
//! test tree would be exactly the leak the rule exists to prevent. Where a
//! test needs a *real* catalog entry it selects one by property —
//! "the first backend with no service address of its own" — and where it needs
//! a hostile one it builds a synthetic [`BackendDefinition`].

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;
use via_catalog::backend::{
    AuthProbe, ConfigurationMode, InstallationSpec, Integration, Lifecycle, Onboarding, ProbeKind,
    Setup,
};
use via_catalog::{BackendDefinition, EnvironmentPolicy, SkillsSpec};
use via_core::EnvMap;
use via_process::{
    AddressAllocator, BackendRuntimeDriver, BackendRuntimeRegistry, BackendSpawner, ChildExit,
    CommandResolver, ManagedLaunch, ProcessError, ServiceAddress, SpawnSpec, StopSignal,
    SupervisedChild,
};

// ── the contract catalogue ──────────────────────────────────────────────────

/// The repository root, found by walking up from this crate.
#[must_use]
pub fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/via-process sits two levels below the repository root")
        .to_path_buf()
}

/// The whole contract catalogue.
///
/// # Panics
///
/// If the catalogue is missing or malformed. That is deliberate: the catalogue
/// *is* the specification, and a run that silently skipped it would report
/// success while asserting nothing.
#[must_use]
pub fn contracts() -> Vec<Value> {
    let path = repository_root().join("docs/reference/contracts.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("contract catalogue at {}: {error}", path.display()));
    serde_json::from_str::<Vec<Value>>(&raw)
        .unwrap_or_else(|error| panic!("contract catalogue is not a JSON array: {error}"))
}

/// The `exactValue` of the one contract with this `kind` and `name`.
///
/// # Panics
///
/// If there is no such contract — the test is then asserting something that is
/// no longer a contract, which must fail loudly rather than pass vacuously.
#[must_use]
pub fn contract_value(kind: &str, name: &str) -> String {
    contracts()
        .iter()
        .find(|entry| entry["kind"] == kind && entry["name"] == name)
        .and_then(|entry| entry["exactValue"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("no `{kind}` contract named `{name}`"))
}

/// The `why` of the one contract with this `kind` and `name`.
///
/// Several behaviours are pinned only in the rationale — the transient
/// cold-start set is the important one — so the rationale is asserted too.
///
/// # Panics
///
/// If there is no such contract.
#[must_use]
pub fn contract_why(kind: &str, name: &str) -> String {
    contracts()
        .iter()
        .find(|entry| entry["kind"] == kind && entry["name"] == name)
        .and_then(|entry| entry["why"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| panic!("no `{kind}` contract named `{name}`"))
}

/// Pull one `<label> <digits>ms`-shaped number out of the timing catalogue.
///
/// # Panics
///
/// If the label is absent, which means the timing contract was edited away
/// from what the code claims to implement.
#[must_use]
pub fn timing_ms(label: &str) -> i64 {
    let catalogue = contract_value("default-value", "timing constants");
    let marker = catalogue
        .find(label)
        .unwrap_or_else(|| panic!("the timing catalogue does not mention `{label}`"));
    let tail = &catalogue[marker + label.len()..];
    let digits: String = tail
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("no number after `{label}` in the timing catalogue"))
}

/// Every `SCREAMING_SNAKE_CASE` token in a string, deduplicated.
#[must_use]
pub fn screaming_tokens(text: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    let mut current = String::new();
    for character in text.chars() {
        if character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_' {
            current.push(character);
        } else {
            if current.len() > 1 && current.chars().any(|c| c.is_ascii_uppercase()) {
                tokens.insert(current.clone());
            }
            current.clear();
        }
    }
    if current.len() > 1 {
        tokens.insert(current);
    }
    tokens
}

/// `docs/rebrand.md`'s rename rule, applied to one upstream variable name.
///
/// The three upstream prefixes all collapse into `VIA_`
/// (`docs/architecture.md` §13). Vendor namespaces are untouched.
#[must_use]
pub fn rebranded(name: &str) -> String {
    for prefix in ["QWEN_AUDIO_AGENT_", "QWEN_AUDIO_", "QWAUDIO_"] {
        if let Some(rest) = name.strip_prefix(prefix) {
            return format!("VIA_{rest}");
        }
    }
    name.to_owned()
}

// ── name-free backend fixtures ──────────────────────────────────────────────

/// The first catalogued backend matching `predicate`.
///
/// Selection is by *property*, never by id, so nothing in this crate's tests
/// writes a backend's name down.
///
/// # Panics
///
/// If no catalogued backend matches, which means the catalog changed shape
/// under a test that assumed it.
#[must_use]
pub fn catalogued(
    predicate: impl Fn(&BackendDefinition) -> bool,
    description: &str,
) -> &'static BackendDefinition {
    via_catalog::backend_definitions()
        .iter()
        .find(|definition| predicate(definition))
        .unwrap_or_else(|| panic!("the catalog has no backend that is {description}"))
}

/// A backend the Gateway hosts in-process: no service address at all.
#[must_use]
pub fn in_gateway_definition() -> &'static BackendDefinition {
    catalogued(
        |definition| definition.base_url_environment.is_none(),
        "hosted in-process (no base-URL variable)",
    )
}

/// A backend with an HTTP service the Gateway must launch itself.
#[must_use]
pub fn owned_service_definition() -> &'static BackendDefinition {
    catalogued(
        |definition| {
            definition.base_url_environment.is_some() && !definition.supports_external_service
        },
        "an HTTP service the Gateway owns",
    )
}

/// A backend that may be an already-running external service.
#[must_use]
pub fn external_service_definition() -> &'static BackendDefinition {
    catalogued(
        |definition| definition.supports_external_service,
        "reachable as an external service",
    )
}

/// A backend that is always effectively full-permission.
#[must_use]
pub fn always_full_definition() -> &'static BackendDefinition {
    catalogued(
        |definition| definition.always_full_permission,
        "always effectively full-permission",
    )
}

/// A backend the Gateway's single switch may not put into full permission.
#[must_use]
pub fn no_full_permission_definition() -> &'static BackendDefinition {
    catalogued(
        |definition| {
            !definition.supports_full_permission
                && !definition.always_full_permission
                && definition.base_url_environment.is_none()
        },
        "in-process and refusing gateway-wide full permission",
    )
}

/// A backend whose environment policy has a user-supplied allow-list.
#[must_use]
pub fn explicit_list_definition() -> &'static BackendDefinition {
    catalogued(
        |definition| definition.environment.explicit_list_environment.is_some(),
        "carrying an explicit environment allow-list",
    )
}

/// A driver for a service the Gateway owns, with a launch command.
#[must_use]
pub fn service_driver(definition: &'static BackendDefinition) -> BackendRuntimeDriver {
    let base_url_environment = definition
        .base_url_environment
        .expect("a service backend has a base-URL variable");
    let default_base_url = definition
        .default_base_url
        .expect("a service backend has a default base URL");
    let mut driver = BackendRuntimeDriver::in_gateway(definition);
    // The real policy is `via-catalog`'s to test. What this crate's tests care
    // about is the *mechanism*, so the fixture policy stands in — otherwise a
    // test would have to name the variables one particular backend forwards.
    driver.environment = FIXTURE_POLICY;
    driver.separate_managed_process = true;
    driver.service = Some(ServiceAddress {
        base_url_environment: base_url_environment.to_owned(),
        default_base_url: default_base_url.to_owned(),
        port_environment: format!(
            "{stem}_PORT",
            stem = base_url_environment
                .strip_suffix("_BASE_URL")
                .unwrap_or(base_url_environment)
        ),
        protocols: if definition.supports_external_service {
            via_process::DEFAULT_SERVICE_PROTOCOLS
                .iter()
                .map(|value| (*value).to_owned())
                .collect()
        } else {
            vec!["http:".to_owned(), "https:".to_owned()]
        },
    });
    driver.launch = Some(ManagedLaunch::new(
        "fixture-runner",
        vec!["--serve".to_owned()],
    ));
    driver
}

/// A registry holding exactly `drivers`.
///
/// # Panics
///
/// If a driver fails validation — a fixture that cannot register is a broken
/// fixture, not a test outcome.
#[must_use]
pub fn registry(drivers: Vec<BackendRuntimeDriver>) -> BackendRuntimeRegistry {
    let mut registry = BackendRuntimeRegistry::new();
    for driver in drivers {
        let id = driver.id.clone();
        registry
            .register(driver)
            .unwrap_or_else(|error| panic!("fixture driver `{id}` did not validate: {error}"));
    }
    registry
}

/// A synthetic definition, for exercising validation against a driver that
/// disagrees with its catalog entry.
#[must_use]
pub fn synthetic_definition(id: &'static str, label: &'static str) -> BackendDefinition {
    BackendDefinition {
        id,
        label,
        workspace_environment: "FIXTURE_WORKSPACE",
        skills: SkillsSpec::NoConvention,
        setup: Setup {
            command: Some("fixture"),
            command_environment: None,
            executable_environment: &[],
            integration: Integration::Native,
            minimum_version: None,
            adapter_command: None,
            adapter_environment: None,
            adapter_runtime_environment: None,
            managed_adapter_fallback: true,
            inspect_adapter_independently: false,
        },
        lifecycle: Lifecycle {
            installation: Some(InstallationSpec {
                verify_installed_packages: false,
                steps: &[],
            }),
            configuration: ConfigurationMode::BackendOwned,
        },
        onboarding: Some(Onboarding {
            command: "fixture login",
            probe: Some(AuthProbe {
                kind: ProbeKind::Command,
                args: &[],
                parser: None,
            }),
        }),
        base_url_environment: None,
        default_base_url: None,
        supports_external_service: false,
        external_service: None,
        supports_full_permission: false,
        always_full_permission: false,
        environment: FIXTURE_POLICY,
    }
}

/// An environment policy with one name, one prefix and an explicit list.
pub const FIXTURE_POLICY: EnvironmentPolicy = EnvironmentPolicy {
    names: &["FIXTURE_TOKEN"],
    prefixes: &["FIXTURE_"],
    explicit_list_environment: Some("VIA_FIXTURE_FORWARD_ENV"),
};

/// Build an [`EnvMap`] from pairs.
#[must_use]
pub fn env(pairs: &[(&str, &str)]) -> EnvMap {
    pairs.iter().copied().collect()
}

// ── recording collaborators ─────────────────────────────────────────────────

/// What an [`AddressAllocator`] fixture was asked to do.
#[derive(Debug, Default)]
pub struct AllocatorLog {
    /// Addresses that were probed.
    pub probed: Vec<String>,
    /// Addresses that were asked to move.
    pub allocated: Vec<String>,
}

/// An allocator that records its calls and answers from a script.
#[derive(Debug)]
pub struct ScriptedAllocator {
    /// What the probe answers. `None` makes any probe a test failure.
    in_use: Option<bool>,
    /// What the allocation answers.
    free_address: String,
    /// Every call, for assertions.
    pub log: Arc<Mutex<AllocatorLog>>,
}

impl ScriptedAllocator {
    /// An allocator whose probe answers `in_use` and whose allocation answers
    /// `free_address`.
    #[must_use]
    pub fn new(in_use: bool, free_address: &str) -> Self {
        Self {
            in_use: Some(in_use),
            free_address: free_address.to_owned(),
            log: Arc::new(Mutex::new(AllocatorLog::default())),
        }
    }

    /// An allocator that must never be called — upstream's
    /// *"external Gateway ownership must not probe or move the port"*.
    #[must_use]
    pub fn forbidden() -> Self {
        Self {
            in_use: None,
            free_address: String::new(),
            log: Arc::new(Mutex::new(AllocatorLog::default())),
        }
    }

    /// A snapshot of what was probed.
    #[must_use]
    pub fn probed(&self) -> Vec<String> {
        self.log.lock().expect("allocator log").probed.clone()
    }

    /// A snapshot of what was moved.
    #[must_use]
    pub fn allocated(&self) -> Vec<String> {
        self.log.lock().expect("allocator log").allocated.clone()
    }
}

#[async_trait]
impl AddressAllocator for ScriptedAllocator {
    async fn address_in_use(&self, base_url: &str) -> Result<bool, ProcessError> {
        self.log
            .lock()
            .expect("allocator log")
            .probed
            .push(base_url.to_owned());
        self.in_use
            .unwrap_or_else(|| {
                panic!("this backend must not probe the port, but probed {base_url}")
            })
            .pipe_ok()
    }

    async fn allocate(&self, base_url: &str) -> Result<String, ProcessError> {
        self.log
            .lock()
            .expect("allocator log")
            .allocated
            .push(base_url.to_owned());
        assert!(
            self.in_use.is_some(),
            "this backend must not move the port, but moved {base_url}"
        );
        Ok(self.free_address.clone())
    }
}

/// Tiny helper so the panic above reads as one expression.
trait PipeOk {
    fn pipe_ok(self) -> Result<bool, ProcessError>;
}

impl PipeOk for bool {
    fn pipe_ok(self) -> Result<bool, ProcessError> {
        Ok(self)
    }
}

/// A spawner that records specs and hands back a recording child.
#[derive(Debug, Default)]
pub struct RecordingSpawner {
    /// Every spec it was handed.
    pub specs: Arc<Mutex<Vec<SpawnSpec>>>,
    /// Every signal the children it handed out were sent.
    pub signals: Arc<Mutex<Vec<StopSignal>>>,
    /// When set, the spawn fails with this message instead.
    failure: Option<String>,
    /// When true, any spawn at all is a test failure.
    forbidden: bool,
}

impl RecordingSpawner {
    /// A spawner that succeeds.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A spawner that must never be called.
    #[must_use]
    pub fn forbidden() -> Self {
        Self {
            forbidden: true,
            ..Self::default()
        }
    }

    /// A spawner whose every spawn fails.
    #[must_use]
    pub fn failing(message: &str) -> Self {
        Self {
            failure: Some(message.to_owned()),
            ..Self::default()
        }
    }

    /// The one spec that was spawned.
    ///
    /// # Panics
    ///
    /// If nothing, or more than one thing, was spawned.
    #[must_use]
    pub fn only_spec(&self) -> SpawnSpec {
        let specs = self.specs.lock().expect("spawn log");
        assert_eq!(specs.len(), 1, "expected exactly one spawn");
        specs[0].clone()
    }

    /// Every signal sent to the children this spawner handed out.
    #[must_use]
    pub fn signals(&self) -> Vec<StopSignal> {
        self.signals.lock().expect("signal log").clone()
    }
}

#[async_trait]
impl BackendSpawner for RecordingSpawner {
    async fn spawn(&self, spec: &SpawnSpec) -> Result<Box<dyn SupervisedChild>, ProcessError> {
        assert!(
            !self.forbidden,
            "this backend must not spawn, but spawned {command}",
            command = spec.command.display()
        );
        self.specs.lock().expect("spawn log").push(spec.clone());
        if let Some(message) = &self.failure {
            return Err(ProcessError::io(
                "spawn the managed backend",
                std::io::Error::new(std::io::ErrorKind::NotFound, message.clone()),
            ));
        }
        Ok(Box::new(RecordingChild {
            signals: Arc::clone(&self.signals),
            exit: None,
        }))
    }
}

/// A child that records signals and only exits when killed.
#[derive(Debug)]
struct RecordingChild {
    signals: Arc<Mutex<Vec<StopSignal>>>,
    exit: Option<ChildExit>,
}

#[async_trait]
impl SupervisedChild for RecordingChild {
    fn id(&self) -> Option<u32> {
        Some(4242)
    }

    fn poll_exit(&mut self) -> Result<Option<ChildExit>, std::io::Error> {
        Ok(self.exit)
    }

    fn signal(&mut self, signal: StopSignal) -> Result<(), std::io::Error> {
        self.signals.lock().expect("signal log").push(signal);
        if signal == StopSignal::Kill {
            self.exit = Some(ChildExit {
                code: None,
                signal: Some(signal.as_i32()),
            });
        }
        Ok(())
    }

    async fn wait(&mut self) -> Result<ChildExit, std::io::Error> {
        loop {
            if let Some(exit) = self.exit {
                return Ok(exit);
            }
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
}

/// A resolver that answers with a fixed directory, so a spawn spec can be
/// asserted without a real executable on the host.
#[derive(Debug, Clone)]
pub struct FixedResolver {
    directory: String,
}

impl FixedResolver {
    /// A resolver that puts every command in `directory`.
    #[must_use]
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_owned(),
        }
    }
}

impl CommandResolver for FixedResolver {
    fn resolve(
        &self,
        command: &str,
        _search_path: &str,
        _working_directory: &Path,
    ) -> Result<PathBuf, ProcessError> {
        Ok(PathBuf::from(format!(
            "{directory}/{command}",
            directory = self.directory
        )))
    }
}

/// A resolver that finds nothing.
#[derive(Debug, Clone, Copy)]
pub struct MissingResolver;

impl CommandResolver for MissingResolver {
    fn resolve(
        &self,
        command: &str,
        _search_path: &str,
        _working_directory: &Path,
    ) -> Result<PathBuf, ProcessError> {
        Err(ProcessError::CommandNotFound {
            command: command.to_owned(),
        })
    }
}
