//! The composition root.
//!
//! `createGatewayApplication({...})` — `server/src/app/gateway-application.mjs`
//! — as a builder and a handle.
//!
//! # Constructing does not bind a port
//!
//! Upstream separates the two with an `autoStart` flag that defaults to `true`,
//! and `bootstrap.mjs` is nine lines whose only job is to call the factory at
//! module scope so importing it starts a server. Its own test then passes
//! `autoStart: false` and asserts `server.listening === false`
//! (`server/test/gateway-application.test.mjs:41`).
//!
//! Here the separation is structural. [`GatewayApplication::build`] takes
//! [`Services`] and returns a handle that owns **no socket**;
//! [`GatewayApplication::bind`] is a separate, `async`, fallible call that
//! returns the bound address; [`Serving::run`] is a third. There is no flag to
//! get wrong and no module-level side effect to import, so `bootstrap.mjs` has
//! no counterpart and needs none.
//!
//! ```no_run
//! # async fn example(services: via_app::Services) -> Result<(), via_app::AppError> {
//! use via_app::{GatewayApplication, InstanceIdentity};
//!
//! let application = GatewayApplication::build(services, InstanceIdentity::default());
//! // Nothing is listening yet.
//! let serving = application.bind("127.0.0.1:0").await?;
//! let origin = serving.origin();   // the resolved port, when 0 was asked for
//! serving.run().await;
//! # Ok(()) }
//! ```
//!
//! # Shutdown, in upstream's order
//!
//! `gateway-application.mjs:509-530`, and the ordering is not incidental —
//! upstream's own comment on the third line is *"a Gateway that stops serving
//! cannot honour a resume, so held state must not survive into the next run"*:
//!
//! 1. stop the backend-availability probe;
//! 2. unsubscribe the offline notifications;
//! 3. stop the reminder scheduler;
//! 4. **release every input suspension**;
//! 5. close the realtime gateway;
//! 6. flush the task store;
//! 7. stop listening.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::offline::{CHANNEL_DEPTH, OfflineNotification, OfflineNotifications, install};
use crate::realtime::EngineFactory;
use crate::serve::BoundServer;
use crate::services::Services;
use crate::state::{AppState, InstanceIdentity};

/// A constructed, not-yet-listening Gateway.
#[derive(Debug)]
pub struct GatewayApplication {
    state: AppState,
    offline: OfflineNotifications,
    notifications: mpsc::Receiver<OfflineNotification>,
}

impl GatewayApplication {
    /// Compose a Gateway from `services`.
    ///
    /// Binds nothing. Must be called from inside a Tokio runtime: the offline
    /// notification subscriber is an owning task.
    #[must_use]
    pub fn build(services: Services, identity: InstanceIdentity) -> Self {
        Self::build_with(services, identity, None)
    }

    /// Compose a Gateway with an explicit realtime engine factory.
    ///
    /// Without one, every connection runs
    /// [`NoEngineFactory`](crate::realtime::NoEngineFactory) — see
    /// [`crate::realtime::engine`] for why that is a supported configuration
    /// rather than a broken one.
    #[must_use]
    pub fn build_with(
        services: Services,
        identity: InstanceIdentity,
        engines: Option<Arc<dyn EngineFactory>>,
    ) -> Self {
        let delay = Duration::from_millis(
            u64::try_from(services.config.offline_notification_delay_ms).unwrap_or(0),
        );
        let (sink, notifications) = mpsc::channel(CHANNEL_DEPTH);
        let offline = install(services.work.clone(), delay, sink);
        let mut state = AppState::new(services, identity);
        if let Some(engines) = engines {
            state = state.with_engines(engines);
        }
        Self {
            state,
            offline,
            notifications,
        }
    }

    /// The shared handler state, for tests and for embedders.
    #[must_use]
    pub const fn state(&self) -> &AppState {
        &self.state
    }

    /// The injected services.
    #[must_use]
    pub const fn services(&self) -> &Services {
        self.state.services()
    }

    /// The offline hand-offs nobody claimed.
    ///
    /// Upstream posts these to an Electron host; VIA hands them to whoever
    /// embeds the Gateway. Taking the receiver is what subscribes: an embedder
    /// that never takes it lets the channel fill and the notifications drop,
    /// which is the same outcome as upstream's absent `parentPort`.
    pub fn offline_notifications(&mut self) -> &mut mpsc::Receiver<OfflineNotification> {
        &mut self.notifications
    }

    /// Bind the listening socket.
    ///
    /// # Errors
    ///
    /// [`AppError::Bind`] when the address cannot be bound.
    pub async fn bind(self, address: &str) -> Result<Serving, AppError> {
        let server = BoundServer::bind(address).await?;
        Ok(Serving {
            address: server.address(),
            server,
            state: self.state,
            offline: self.offline,
            notifications: self.notifications,
            shutdown: CancellationToken::new(),
        })
    }

    /// Bind `config.host:config.port`.
    ///
    /// # Errors
    ///
    /// [`AppError::Bind`].
    pub async fn bind_configured(self) -> Result<Serving, AppError> {
        let address = format!(
            "{}:{}",
            self.services().config.host,
            self.services().config.port
        );
        self.bind(&address).await
    }
}

/// A bound Gateway.
#[derive(Debug)]
pub struct Serving {
    server: BoundServer,
    address: SocketAddr,
    state: AppState,
    offline: OfflineNotifications,
    notifications: mpsc::Receiver<OfflineNotification>,
    shutdown: CancellationToken,
}

impl Serving {
    /// The address actually bound.
    #[must_use]
    pub const fn address(&self) -> SocketAddr {
        self.address
    }

    /// `http://<host>:<boundPort>`.
    ///
    /// **External contract** — the `origin` of the ready report
    /// (`gateway-application.mjs:488-492`), which is how an embedding host
    /// learns the port when `PORT=0`.
    #[must_use]
    pub fn origin(&self) -> String {
        format!("http://{}", self.address)
    }

    /// The shared handler state.
    #[must_use]
    pub const fn state(&self) -> &AppState {
        &self.state
    }

    /// The offline hand-offs nobody claimed.
    pub fn offline_notifications(&mut self) -> &mut mpsc::Receiver<OfflineNotification> {
        &mut self.notifications
    }

    /// A handle that stops [`run`](Self::run).
    #[must_use]
    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown.clone()
    }

    /// Serve until the shutdown token is cancelled, then close in upstream's
    /// order.
    pub async fn run(self) {
        let router = crate::http::router(self.state.clone());
        let shutdown = self.shutdown.clone();
        let state = self.state.clone();
        let offline = self.offline;

        state.services().logger.info(
            "gateway.ready",
            via_log::fields([
                ("origin", format!("http://{}", self.address).into()),
                (
                    "backend",
                    state.services().config.agent_protocol.clone().into(),
                ),
                (
                    "realtimeProvider",
                    state
                        .services()
                        .realtime_provider
                        .clone()
                        .unwrap_or_default()
                        .into(),
                ),
            ]),
            &format!("VIA running at http://{}", self.address),
        );

        self.server.serve(router, shutdown).await;
        close(&state, offline).await;
    }
}

/// The close sequence, in upstream's order.
///
/// Steps 1-3 of `gateway-application.mjs:512-521` have no VIA counterpart in
/// this crate — the backend-availability probe and the reminder scheduler are
/// owned by whoever built them and are closed by their own handles — so what is
/// reproduced here is the part `via-app` owns, in the positions upstream puts
/// it.
async fn close(state: &AppState, offline: OfflineNotifications) {
    // 2. `unsubscribeOfflineNotifications?.()`
    offline.close().await;
    // 4. A Gateway that stops serving cannot honour a resume, so held state
    //    must not survive into the next run.
    state.services().input_arbitration.release_all().await;
    // 6. `await taskStore?.flush?.()`
    state.services().work.flush().await;
    state.services().logger.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn binding_port_zero_reports_the_resolved_port() {
        let services = crate::testing::test_services();
        let application = GatewayApplication::build(services, InstanceIdentity::default());
        let serving = application
            .bind("127.0.0.1:0")
            .await
            .expect("binds an ephemeral port");
        assert!(serving.address().port() > 0);
        assert_eq!(serving.origin(), format!("http://{}", serving.address()));
    }

    #[tokio::test]
    async fn a_taken_port_is_an_error_not_a_panic() {
        let first = crate::testing::test_services();
        let held = GatewayApplication::build(first, InstanceIdentity::default())
            .bind("127.0.0.1:0")
            .await
            .expect("binds");
        let address = held.address().to_string();
        let second = crate::testing::test_services();
        let error = GatewayApplication::build(second, InstanceIdentity::default())
            .bind(&address)
            .await
            .expect_err("the port is taken");
        assert!(matches!(error, AppError::Bind { .. }));
    }
}
