//! What the composition root and the server can refuse with.

use std::io;

use via_core::CoreError;
use via_lock::LeaseError;
use via_realtime::RealtimeError;

/// A failure raised by `via-app` itself.
///
/// Every variant is a *startup* failure. Once the server is listening, a
/// request failure is an HTTP status and a body, never one of these — see
/// [`crate::http`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    /// The setup gate refused an unconfigured start.
    ///
    /// Raised **before** the lease is touched, reproducing
    /// `server/src/index.mjs:50`.
    #[error("{0}")]
    Setup(#[source] CoreError),

    /// The single-instance lease could not be taken.
    #[error(transparent)]
    Lease(#[from] LeaseError),

    /// The configured realtime provider is not registered.
    #[error(transparent)]
    Realtime(#[from] RealtimeError),

    /// The listening socket could not be bound.
    #[error("could not bind {address}: {source}")]
    Bind {
        /// The address that was attempted.
        address: String,
        /// The operating system's reason.
        #[source]
        source: io::Error,
    },

    /// The listener failed after it was bound.
    #[error(transparent)]
    Io(#[from] io::Error),
}
