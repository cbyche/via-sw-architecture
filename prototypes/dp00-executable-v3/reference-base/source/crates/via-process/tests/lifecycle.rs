//! The start sequence: what happens, in what order, and what gets logged.
//!
//! Ported from `server/src/process/managed-backend.mjs:168-231`. Upstream's
//! own tests cover the outcomes; these cover the *ordering*, because the
//! ordering is where the contracts live — the ownership write-back has to
//! precede the spawn so the child sees the resolved value
//! (`env-var/QWEN_AUDIO_AGENT_BACKEND_OWNERSHIP`), and the loopback refusal has
//! to precede the port probe so the Gateway never opens a socket to a host it
//! was never going to launch on.

mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use common::{
    FixedResolver, RecordingSpawner, ScriptedAllocator, env, external_service_definition, registry,
    service_driver,
};
use pretty_assertions::assert_eq;
use serde_json::Value;
use via_catalog::Ownership;
use via_core::EnvMap;
use via_core::config::names;
use via_core::search_path::Platform;
use via_log::{LogLevel, Logger, LoggerOptions, MemorySink};
use via_process::{
    BackendRuntimeHooks, EVENT_ADDRESS_REALLOCATED, EVENT_PROCESS_START_FAILED,
    EVENT_PROCESS_STARTED, ManagedBackend, ProcessError, StartOptions, start_managed_backend,
};

const ROOT: &str = "/repo";

fn memory_logger() -> (Logger, Arc<MemorySink>) {
    let sink = Arc::new(MemorySink::new());
    let mut options = LoggerOptions::detached("via-process-test");
    options.level = LogLevel::Trace;
    (Logger::with_sinks(options, vec![sink.clone()]), sink)
}

fn events(sink: &MemorySink) -> Vec<String> {
    sink.records()
        .iter()
        .filter_map(|record| record["event"].as_str().map(str::to_owned))
        .collect()
}

fn record<'a>(records: &'a [Value], event: &str) -> &'a Value {
    records
        .iter()
        .find(|record| record["event"] == event)
        .unwrap_or_else(|| panic!("no `{event}` record"))
}

/// One observation: which hook ran, and what ownership the environment held.
type Observation = (String, Option<String>);

/// A hook that snapshots the environment it is handed.
#[derive(Debug, Default)]
struct Snapshotting {
    seen: Arc<Mutex<Vec<Observation>>>,
}

impl BackendRuntimeHooks for Snapshotting {
    fn prepare_environment(
        &self,
        env: &mut EnvMap,
        _backend: &ManagedBackend,
    ) -> Result<(), ProcessError> {
        self.seen.lock().expect("snapshot").push((
            "prepare_environment".to_owned(),
            env.get(names::BACKEND_OWNERSHIP).map(str::to_owned),
        ));
        Ok(())
    }
}

#[tokio::test]
async fn the_ownership_write_back_happens_before_the_driver_prepares_anything() {
    let definition = external_service_definition();
    let hooks = Arc::new(Snapshotting::default());
    let mut driver = service_driver(definition);
    driver.hooks = Some(hooks.clone());
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    let allocator = ScriptedAllocator::new(false, "");
    let spawner = RecordingSpawner::new();
    let options = StartOptions {
        root: PathBuf::from(ROOT),
        registry: &registry,
        platform: Platform::Posix,
        allocator: &allocator,
        spawner: &spawner,
        resolver: &FixedResolver::new("/opt/fixture/bin"),
        logger: None,
    };
    start_managed_backend(&mut environment, &options)
        .await
        .expect("the backend starts");

    assert_eq!(
        *hooks.seen.lock().unwrap(),
        vec![(
            "prepare_environment".to_owned(),
            Some(Ownership::Owned.as_str().to_owned()),
        )],
    );
}

#[tokio::test]
async fn moving_an_occupied_address_is_logged_with_the_address_that_was_asked_for() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);
    let (logger, sink) = memory_logger();

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    let allocator = ScriptedAllocator::new(true, "http://127.0.0.1:45678");
    let spawner = RecordingSpawner::new();
    let options = StartOptions {
        root: PathBuf::from(ROOT),
        registry: &registry,
        platform: Platform::Posix,
        allocator: &allocator,
        spawner: &spawner,
        resolver: &FixedResolver::new("/opt/fixture/bin"),
        logger: Some(&logger),
    };
    start_managed_backend(&mut environment, &options)
        .await
        .expect("the backend starts");

    assert_eq!(
        events(&sink),
        vec![
            EVENT_ADDRESS_REALLOCATED.to_owned(),
            EVENT_PROCESS_STARTED.to_owned(),
        ],
        "the move is reported before the process it made room for",
    );

    let records = sink.records();
    let moved = record(&records, EVENT_ADDRESS_REALLOCATED);
    assert_eq!(moved["backend"], definition.id);
    assert_eq!(moved["requestedBaseUrl"], "http://127.0.0.1:18789");

    let started = record(&records, EVENT_PROCESS_STARTED);
    assert_eq!(started["backend"], definition.id);
    assert_eq!(started["baseUrl"], "http://127.0.0.1:45678");
    assert_eq!(started["ownership"], "owned");
    assert_eq!(
        started["childPid"], 4242,
        "the child's pid must survive; `pid` is reserved by the logger",
    );
    assert_eq!(
        started["pid"],
        Value::from(std::process::id()),
        "`pid` is stamped by the logger and is the Gateway's own — which is \
         exactly why the child's is published under another name",
    );
}

#[tokio::test]
async fn a_free_address_is_not_reported_as_moved() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);
    let (logger, sink) = memory_logger();

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    let allocator = ScriptedAllocator::new(false, "");
    let spawner = RecordingSpawner::new();
    let options = StartOptions {
        root: PathBuf::from(ROOT),
        registry: &registry,
        platform: Platform::Posix,
        allocator: &allocator,
        spawner: &spawner,
        resolver: &FixedResolver::new("/opt/fixture/bin"),
        logger: Some(&logger),
    };
    start_managed_backend(&mut environment, &options)
        .await
        .expect("the backend starts");

    assert_eq!(events(&sink), vec![EVENT_PROCESS_STARTED.to_owned()]);
    assert!(allocator.allocated().is_empty());
}

#[tokio::test]
async fn a_failed_spawn_is_logged_at_error_and_nothing_is_reported_as_started() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);
    let (logger, sink) = memory_logger();

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    let allocator = ScriptedAllocator::new(false, "");
    let spawner = RecordingSpawner::failing("no such file or directory");
    let options = StartOptions {
        root: PathBuf::from(ROOT),
        registry: &registry,
        platform: Platform::Posix,
        allocator: &allocator,
        spawner: &spawner,
        resolver: &FixedResolver::new("/opt/fixture/bin"),
        logger: Some(&logger),
    };
    let error = start_managed_backend(&mut environment, &options)
        .await
        .unwrap_err();
    assert_eq!(error.code(), "VIA_PROCESS_IO");

    assert_eq!(events(&sink), vec![EVENT_PROCESS_START_FAILED.to_owned()]);
    let records = sink.records();
    let failed = record(&records, EVENT_PROCESS_START_FAILED);
    assert_eq!(failed["backend"], definition.id);
    assert_eq!(failed["level"], "error");
    assert!(
        failed["error"]
            .as_str()
            .expect("an error string")
            .contains("no such file"),
        "{failed}",
    );
}

#[tokio::test]
async fn a_start_with_no_logger_still_works() {
    // The logger is optional upstream (`logger?.info(…)`), and a Gateway that
    // has not built one yet must still be able to start a backend.
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    let mut environment = env(&[
        ("AGENT_PROTOCOL", definition.id),
        (names::BACKEND_OWNERSHIP, "owned"),
        (&service.base_url_environment, "http://127.0.0.1:18789"),
    ]);
    let allocator = ScriptedAllocator::new(true, "http://127.0.0.1:45678");
    let spawner = RecordingSpawner::new();
    let options = StartOptions {
        root: PathBuf::from(ROOT),
        registry: &registry,
        platform: Platform::Posix,
        allocator: &allocator,
        spawner: &spawner,
        resolver: &FixedResolver::new("/opt/fixture/bin"),
        logger: None,
    };
    let started = start_managed_backend(&mut environment, &options)
        .await
        .expect("the backend starts");
    assert!(started.runtime.owns_process());
    assert_eq!(started.runtime.pid(), Some(4242));
}

#[tokio::test]
async fn the_published_port_follows_the_reallocated_address() {
    let definition = external_service_definition();
    let driver = service_driver(definition);
    let service = driver.service.clone().expect("a service driver");
    let registry = registry(vec![driver]);

    for (allocated, expected_port) in [
        ("http://127.0.0.1:45678", "45678"),
        // No explicit port: `https:` publishes 443 and everything else 80 —
        // upstream's quirk, reproduced. `wss:` publishing 80 is the one that
        // looks wrong and is nevertheless what upstream does.
        ("https://127.0.0.1", "443"),
        ("http://127.0.0.1", "80"),
        ("wss://127.0.0.1", "80"),
    ] {
        let mut environment = env(&[
            ("AGENT_PROTOCOL", definition.id),
            (names::BACKEND_OWNERSHIP, "owned"),
            (&service.base_url_environment, "http://127.0.0.1:18789"),
        ]);
        let allocator = ScriptedAllocator::new(true, allocated);
        let spawner = RecordingSpawner::new();
        let options = StartOptions {
            root: PathBuf::from(ROOT),
            registry: &registry,
            platform: Platform::Posix,
            allocator: &allocator,
            spawner: &spawner,
            resolver: &FixedResolver::new("/opt/fixture/bin"),
            logger: None,
        };
        start_managed_backend(&mut environment, &options)
            .await
            .expect("the backend starts");
        assert_eq!(
            environment.get(&service.port_environment),
            Some(expected_port),
            "{allocated}",
        );
        assert_eq!(
            environment.get(&service.base_url_environment),
            Some(allocated),
        );
    }
}
