//! A Gateway on an ephemeral port, for the integration tests.

#![allow(dead_code)]

use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use via_app::realtime::engine::{RecordingEngine, RecordingEngineFactory};
use via_app::testing::test_services;
use via_app::{GatewayApplication, InstanceIdentity, Services};

/// A running Gateway and how to reach it.
pub struct Harness {
    /// `http://127.0.0.1:<port>`.
    pub origin: String,
    /// `ws://127.0.0.1:<port>`.
    pub ws_origin: String,
    /// The services it was built from.
    pub services: Services,
    /// The realtime engine every connection is handed.
    pub engine: Arc<RecordingEngine>,
    shutdown: CancellationToken,
    joined: Option<tokio::task::JoinHandle<()>>,
}

impl Harness {
    /// Start a Gateway with the default test services.
    pub async fn start() -> Self {
        Self::with(test_services(), InstanceIdentity::default()).await
    }

    /// Start a Gateway with a lease identity, as `index.mjs` does.
    pub async fn with_instance(identity: InstanceIdentity) -> Self {
        Self::with(test_services(), identity).await
    }

    /// Start a Gateway from `services`.
    pub async fn with(services: Services, identity: InstanceIdentity) -> Self {
        let engine = Arc::new(RecordingEngine::new("mock", 16_000));
        let application = GatewayApplication::build_with(
            services.clone(),
            identity,
            Some(Arc::new(RecordingEngineFactory::new(engine.clone()))),
        );
        let serving = application
            .bind("127.0.0.1:0")
            .await
            .expect("binds an ephemeral port");
        let address = serving.address();
        let shutdown = serving.shutdown_token();
        let joined = tokio::spawn(serving.run());
        Self {
            origin: format!("http://{address}"),
            ws_origin: format!("ws://{address}"),
            services,
            engine,
            shutdown,
            joined: Some(joined),
        }
    }

    /// The bound address, as `host:port`.
    pub fn authority(&self) -> String {
        self.origin.trim_start_matches("http://").to_owned()
    }

    /// Stop serving and wait for the close sequence.
    pub async fn stop(mut self) {
        self.shutdown.cancel();
        if let Some(joined) = self.joined.take() {
            let _ = joined.await;
        }
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.shutdown.cancel();
    }
}

/// A client that sends no `Origin` — the CLI path, which loopback admits.
pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("a client")
}
