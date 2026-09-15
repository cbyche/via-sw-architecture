//! Startup and shutdown order.
//!
//! Ported from `server/src/index.mjs` and `test/gateway-setup.test.mjs`. The one
//! thing worth a whole file: **the setup gate runs before the lease is
//! touched**, so a misconfigured start cannot disturb a running Gateway.

use std::sync::Arc;

use pretty_assertions::assert_eq;
use via_app::testing::test_services;
use via_app::{AppError, GatewayApplication, InstanceIdentity, Startup};
use via_core::config::{Overrides, resolve};
use via_core::{Config, EnvMap};

fn logger() -> Arc<via_log::Logger> {
    Arc::new(via_log::Logger::with_sinks(
        via_log::LoggerOptions::detached("gateway"),
        Vec::new(),
    ))
}

fn config_in(directory: &std::path::Path, env: &EnvMap) -> Config {
    let overrides = Overrides {
        home_directory: directory.to_path_buf(),
        working_directory: directory.to_path_buf(),
        ..Overrides::default()
    };
    resolve(env, None, &overrides).expect("a resolvable configuration")
}

fn env(pairs: &[(&str, &str)]) -> EnvMap {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn an_unconfigured_start_is_refused_before_the_lease_is_touched() {
    let directory = tempfile::tempdir().expect("a temp directory");
    let environment = env(&[]);
    let config = config_in(directory.path(), &environment);

    let refusal = Startup::begin(&config, &environment, logger())
        .expect_err("an unconfigured start is refused");
    assert!(matches!(refusal, AppError::Setup(_)), "got {refusal:?}");
    assert!(
        refusal.to_string().contains("DASHSCOPE_API_KEY"),
        "the refusal names what is missing: {refusal}",
    );

    let lock = config.config_directory().join("gateway.lock");
    assert!(
        !lock.exists(),
        "the gate must refuse BEFORE the lease is touched; {} exists",
        lock.display(),
    );
}

#[test]
fn a_configured_start_takes_the_lease_and_reports_its_instance() {
    let directory = tempfile::tempdir().expect("a temp directory");
    let environment = env(&[("DASHSCOPE_API_KEY", "sk-test")]);
    let config = config_in(directory.path(), &environment);

    let started = Startup::begin(&config, &environment, logger()).expect("the gate passes");
    let instance_id = started
        .identity
        .instance_id
        .clone()
        .expect("a lease was taken");
    assert!(!instance_id.is_empty());
    assert_eq!(started.runtime.instance_id(), Some(instance_id.as_str()));
    assert!(
        started.identity.started_at.is_some(),
        "/api/health reports when this instance started",
    );
    assert!(config.config_directory().join("gateway.lock").exists());
}

#[test]
fn a_second_start_in_the_same_directory_is_refused() {
    let directory = tempfile::tempdir().expect("a temp directory");
    let environment = env(&[("DASHSCOPE_API_KEY", "sk-test")]);
    let config = config_in(directory.path(), &environment);

    let _held = Startup::begin(&config, &environment, logger()).expect("the first start");
    let refusal = Startup::begin(&config, &environment, logger())
        .expect_err("one Gateway per configuration directory");
    assert!(matches!(refusal, AppError::Lease(_)), "got {refusal:?}");
}

#[test]
fn the_explicit_opt_out_passes_the_gate() {
    let directory = tempfile::tempdir().expect("a temp directory");
    // Debugging and harness setups that never open a voice connection can skip
    // the gate explicitly (`shared/gateway-setup.mjs`).
    let environment = env(&[("VIA_ALLOW_UNCONFIGURED", "1")]);
    let config = config_in(directory.path(), &environment);
    assert!(Startup::begin(&config, &environment, logger()).is_ok());
}

#[tokio::test]
async fn shutdown_releases_every_input_suspension() {
    let services = test_services();
    let application = GatewayApplication::build(services.clone(), InstanceIdentity::default());
    let serving = application.bind("127.0.0.1:0").await.expect("binds");
    let shutdown = serving.shutdown_token();
    let serving = tokio::spawn(serving.run());

    services
        .input_arbitration
        .suspend("host-app", "dictation", Some(300_000))
        .await
        .expect("an owner was supplied");
    assert!(services.input_arbitration.suspended().await);

    shutdown.cancel();
    let _ = serving.await;

    assert!(
        !services.input_arbitration.suspended().await,
        "a Gateway that stops serving cannot honour a resume, so held state \
         must not survive into the next run",
    );
}

#[tokio::test]
async fn the_heartbeat_is_cleared_before_anything_is_stopped() {
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct Witness {
        heartbeat: tokio_util::sync::CancellationToken,
        cleared_first: Arc<AtomicBool>,
    }

    #[async_trait]
    impl via_app::Stoppable for Witness {
        async fn stop(&self) -> Result<(), String> {
            // A shutdown that takes seconds must not keep refreshing a lease it
            // is about to release, so the heartbeat is already gone by now.
            self.cleared_first
                .store(self.heartbeat.is_cancelled(), Ordering::SeqCst);
            Ok(())
        }
    }

    let mut started = Startup::detached(logger());
    let heartbeat = started.runtime.heartbeat_token();
    let cleared_first = Arc::new(AtomicBool::new(false));
    started.runtime.register(Arc::new(Witness {
        heartbeat: heartbeat.clone(),
        cleared_first: cleared_first.clone(),
    }));

    assert!(!heartbeat.is_cancelled());
    via_app::shutdown(started.runtime).await;

    assert!(heartbeat.is_cancelled(), "the heartbeat is cleared");
    assert!(
        cleared_first.load(Ordering::SeqCst),
        "and it is cleared FIRST, before the stops run",
    );
}

#[tokio::test]
async fn a_detached_start_takes_no_lease() {
    let started = Startup::detached(logger());
    assert_eq!(started.runtime.instance_id(), None);
    assert_eq!(started.identity.instance_id, None);
    via_app::shutdown(started.runtime).await;
}
