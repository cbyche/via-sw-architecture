//! The single-instance Gateway lease, and the CLI instance lock beside it.
//!
//! One VIA Gateway per configuration directory. The lease is a single JSON
//! line in `<configDir>/gateway.lock` that answers three questions for anyone
//! who can read the directory:
//!
//! 1. *Is a Gateway running?* — the `pid` field, checked with `kill(pid, 0)`.
//! 2. *Where is it?* — the `origin` field, which is how a second launch finds
//!    the first without any port bookkeeping.
//! 3. *Is the thing on that origin actually mine?* — the `instanceId` field,
//!    which [`find_running_gateway`] cross-checks against `/api/health`'s
//!    `gatewayInstanceId`, so a port that some unrelated service happens to
//!    have reused reads as "not running".
//!
//! This is a full port of upstream `shared/gateway-instance-lock.mjs`
//! (qwen-audio-agent v1.11.0). The on-disk format, the file name, the retry
//! count, the temp/stale file names and the conflict error code are external
//! contracts — other processes, including a still-running Node Gateway during
//! migration, read and write the same file. Every such value carries a doc
//! comment naming the upstream file it came from.
//!
//! # Two locks, one directory
//!
//! `<configDir>/cli.lock` is a second, independent lock with its own module
//! ([`acquire_cli_instance`], ported from upstream
//! `cli/src/instance-lock.mjs`). It answers "is another interactive CLI
//! running?", not "is another Gateway running?" — a CLI attaches to a Gateway
//! it did not start, and a Gateway runs with no CLI. The two files never
//! interact and the differences between them are upstream's, not
//! simplifications; the table is on [`acquire_cli_instance`]'s module in
//! `src/cli.rs`.
//!
//! # What this crate deliberately does not do
//!
//! [`GatewayLeaseHandle`] has **no `Drop` impl**. Releasing a lease is
//! conditional (it must still be ours) and fallible (it does file I/O), and
//! neither fits `Drop`. Release is spelled `handle.release()`, it consumes the
//! handle, and dropping a handle instead simply leaves the lease on disk for
//! the liveness check to reclaim. See [`GatewayLeaseHandle::release`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod acquire;
mod cli;
mod error;
mod find;
mod lease;
mod probe;

pub use acquire::{
    AcquireOptions, GatewayLeaseHandle, LeaseUpdate, MAX_ACQUIRE_ATTEMPTS, acquire_gateway_lease,
};
pub use cli::{
    CLI_LOCK_FILE_MODE, CLI_LOCK_FILE_NAME, CliAcquireOptions, CliInstanceHandle, CliLockDocument,
    CliLockError, MAX_CLI_ACQUIRE_ATTEMPTS, acquire_cli_instance, cli_lock_path,
};
pub use error::LeaseError;
pub use find::{
    FindOptions, HEALTH_INSTANCE_ID_FIELD, HealthProbe, RunningGateway, find_running_gateway,
    health_instance_id,
};
pub use lease::{
    CONFIG_DIRECTORY_MODE, DEFAULT_LEASE_OWNER, GATEWAY_HEARTBEAT_INTERVAL, GATEWAY_LOCK_FILE_NAME,
    GATEWAY_LOCK_SCHEMA, GatewayLease, LEASE_FILE_MODE, LEASE_STATE_READY, LEASE_STATE_STARTING,
    VIA_GATEWAY_ALREADY_RUNNING, gateway_lock_path, lease_stale_path, lease_temp_path,
    read_gateway_lease,
};
pub use probe::{
    Clock, ProcessProbe, SignalOutcome, SystemClock, SystemProcessProbe, current_pid, iso8601,
    process_is_alive,
};

/// The `/api/health` fields the CLI reads back after a Gateway starts.
///
/// **Partial contract, and deliberately only the part this crate owns.**
/// `docs/reference/contracts.json` catalogues *"health fields the CLI reads"*
/// as one list spanning `backend.*`, `realtimeModelProfile.*`, the four
/// `realtime*` scalars, `voiceConfigured`, `voiceClients.byType.desktop` and
/// `gatewayInstanceId`. Everything but the last belongs to the crate that
/// *serves* `/api/health`; this crate reads exactly one field, to prove the
/// origin in the lease is the Gateway the lease names.
///
/// [`HEALTH_INSTANCE_ID_FIELD`] is that field. It is named here as well so the
/// coverage registry can point at a single item and record that the rest is
/// still owed by `via-app`.
pub const HEALTH_FIELDS_READ_BY_THIS_CRATE: [&str; 1] = [HEALTH_INSTANCE_ID_FIELD];
