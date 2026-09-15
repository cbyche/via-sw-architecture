//! Where a realtime session comes from.
//!
//! [`via_realtime::RealtimeSession`] has two doors, and which one is right
//! depends on the provider rather than on the caller:
//!
//! - [`RealtimeSession::connect`] dials [`RealtimeProvider::url`] over TCP.
//!   That is the door `dashscope` and every network provider uses.
//! - [`RealtimeSession::open`] takes a [`via_realtime::Transport`] that is
//!   already established. `via-realtime-mock`'s own documentation is blunt
//!   about why: *"`RealtimeSession::connect` is the wrong door — it dials
//!   `RealtimeProvider::url` over TCP, and the mock's endpoint names no
//!   socket."*
//!
//! A Gateway a user runs always takes the first. The phase-5 milestone —
//! *"`via chat` works end to end — no audio hardware, no model weights"* — takes
//! the second, and this trait is the one seam that lets the **same** Gateway
//! code do both. It is the reason the milestone test can start a real Gateway
//! rather than a stand-in for one.

use std::sync::Arc;

use async_trait::async_trait;
use via_realtime::{
    RealtimeError, RealtimeProvider, RealtimeSession, SessionEvents, SessionOptions,
};

/// Opens one realtime session against `provider`.
#[async_trait]
pub trait SessionOpener: Send + Sync + std::fmt::Debug {
    /// Open, and answer the session with its event stream.
    ///
    /// # Errors
    ///
    /// Whatever the provider or the transport refuses with, already localized.
    async fn open(
        &self,
        provider: &Arc<dyn RealtimeProvider>,
        options: SessionOptions,
    ) -> Result<(RealtimeSession, SessionEvents), RealtimeError>;
}

/// The opener a running Gateway uses: a real WebSocket to the provider's
/// endpoint.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConnectOpener;

#[async_trait]
impl SessionOpener for ConnectOpener {
    async fn open(
        &self,
        provider: &Arc<dyn RealtimeProvider>,
        options: SessionOptions,
    ) -> Result<(RealtimeSession, SessionEvents), RealtimeError> {
        RealtimeSession::connect(Arc::clone(provider), options).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_opener_is_the_network_one() {
        assert_eq!(format!("{:?}", ConnectOpener), "ConnectOpener");
    }
}
