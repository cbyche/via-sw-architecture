//! The lease document, its file name, and reading it back.
//!
//! Ported from upstream `shared/gateway-instance-lock.mjs:13-40`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The lease document's `schema` field, and the value a reader must match.
///
/// **External contract.** The upstream constant `GATEWAY_LOCK_SCHEMA`
/// (`shared/gateway-instance-lock.mjs:13`) reads `qwaudio.gateway-lock/v1`;
/// `docs/rebrand.md` renames the product prefix, so VIA writes
/// `via.gateway-lock/v1`.
///
/// [`read_gateway_lease`] returns `None` for any document whose `schema` is
/// not exactly this string, which is what keeps a mixed-version pair of
/// processes from half-understanding each other's lease.
pub const GATEWAY_LOCK_SCHEMA: &str = "via.gateway-lock/v1";

/// The lease file's name inside the configuration directory.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:15`. The
/// name is brand-free and is KEPT verbatim per `docs/rebrand.md`, so the
/// `~/.config` migration stays a pure directory move.
pub const GATEWAY_LOCK_FILE_NAME: &str = "gateway.lock";

/// The conflict error code reported when another live Gateway holds the lease.
///
/// **External contract.** The upstream code is `QWAUDIO_GATEWAY_ALREADY_RUNNING`
/// (`shared/gateway-instance-lock.mjs:157`); `docs/rebrand.md` renames the
/// prefix. Callers (the CLI, the desktop host) branch on this code to attach to
/// the running instance instead of failing.
pub const VIA_GATEWAY_ALREADY_RUNNING: &str = "VIA_GATEWAY_ALREADY_RUNNING";

/// The `owner` written when the caller does not supply one.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:89`. The
/// Gateway process itself passes `"desktop"` or `"cli"`
/// (upstream `server/src/index.mjs:51-53`).
pub const DEFAULT_LEASE_OWNER: &str = "gateway";

/// The `state` a freshly acquired lease is stamped with.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:107`.
pub const LEASE_STATE_STARTING: &str = "starting";

/// The `state` the Gateway publishes once its listener is bound.
///
/// **External contract.** Upstream `server/src/index.mjs:134-137`, together
/// with `origin = http://<host>:<port>`.
pub const LEASE_STATE_READY: &str = "ready";

/// How often a running Gateway re-stamps `heartbeatAt`.
///
/// **External contract.** Upstream `server/src/index.mjs:138` —
/// `setInterval(() => lease.update(), 15_000)`. This crate does not run the
/// timer; the owning process does, by calling
/// [`GatewayLeaseHandle::heartbeat`](crate::GatewayLeaseHandle::heartbeat).
pub const GATEWAY_HEARTBEAT_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(15_000);

/// Unix mode of the lease file: owner read/write only.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:46` —
/// `openSync(path, 'wx', 0o600)`. As with `open(2)` anywhere, the effective
/// mode is `0o600 & !umask`.
pub const LEASE_FILE_MODE: u32 = 0o600;

/// Unix mode of the configuration directory when this crate has to create it.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:96` —
/// `mkdirSync(configDirectory, { recursive: true, mode: 0o700 })`.
pub const CONFIG_DIRECTORY_MODE: u32 = 0o700;

/// The lease document written to `<configDir>/gateway.lock`.
///
/// **External contract — field order included.** Upstream
/// `shared/gateway-instance-lock.mjs:103-112` builds the object literal in the
/// order `schema, instanceId, pid, owner, state, origin, startedAt,
/// heartbeatAt` and writes `JSON.stringify(lease) + '\n'`. `serde` serializes
/// struct fields in declaration order, so the declaration order below *is* the
/// on-disk order; do not reorder it.
///
/// Reading is deliberately lenient in exactly the way upstream's is. Upstream
/// validates only that `schema` matches and that `instanceId` is a string, and
/// returns whatever else the document happens to contain — so the five
/// remaining fields default rather than fail, and unrecognised fields are
/// ignored. A partial document like `{schema, instanceId, pid}` is a real
/// input: upstream's own tests write one to simulate a stale or foreign lease.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayLease {
    /// Always [`GATEWAY_LOCK_SCHEMA`]. A document with any other value is not
    /// a lease this build understands.
    pub schema: String,
    /// Opaque per-process identity, a UUID v4 by default. Cross-checked
    /// against `/api/health`'s `gatewayInstanceId` by
    /// [`find_running_gateway`](crate::find_running_gateway).
    pub instance_id: String,
    /// The owning process id, probed with `kill(pid, 0)`.
    ///
    /// Widened to `i64` rather than `i32` so that a corrupt or foreign
    /// document does not fail to parse; out-of-range and non-positive values
    /// read as "not alive" (see
    /// [`process_is_alive`](crate::process_is_alive)).
    #[serde(default)]
    pub pid: i64,
    /// Which form owns the Gateway: [`DEFAULT_LEASE_OWNER`] here, `"cli"` or
    /// `"desktop"` from the Gateway process.
    #[serde(default)]
    pub owner: String,
    /// [`LEASE_STATE_STARTING`], then [`LEASE_STATE_READY`]. Kept as a
    /// `String` rather than an enum so a document written by a newer build
    /// still reads back instead of vanishing.
    #[serde(default)]
    pub state: String,
    /// Empty until the listener is bound, then `http://<host>:<port>`.
    #[serde(default)]
    pub origin: String,
    /// ISO-8601 UTC millisecond timestamp, stamped once at acquisition.
    #[serde(default)]
    pub started_at: String,
    /// ISO-8601 UTC millisecond timestamp, re-stamped on every update.
    #[serde(default)]
    pub heartbeat_at: String,
}

impl GatewayLease {
    /// True when both leases carry the same non-empty `instanceId` **and** the
    /// same `pid`.
    ///
    /// **External contract.** Upstream `sameLease`
    /// (`shared/gateway-instance-lock.mjs:41-48`). This is the guard that makes
    /// `update` and `release` refuse to touch a file another instance has
    /// taken over: an empty `instanceId` is falsy upstream and is rejected here
    /// for the same reason.
    #[must_use]
    pub fn is_same_lease(&self, other: &Self) -> bool {
        !self.instance_id.is_empty()
            && self.instance_id == other.instance_id
            && self.pid == other.pid
    }
}

/// `<configDirectory>/gateway.lock`.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:15-17`.
///
/// Unlike upstream's `resolve()`, this does not absolutize a relative
/// `config_directory`; callers pass the already-absolute directory that
/// `VIA_CONFIG_DIR` resolution produces.
#[must_use]
pub fn gateway_lock_path(config_directory: &Path) -> PathBuf {
    config_directory.join(GATEWAY_LOCK_FILE_NAME)
}

/// `<lockPath>.<instanceId>.tmp` — the temp file an update writes before it
/// renames over the lease.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:59`. It is
/// public because it is an artifact a crash can leave behind, and any tool that
/// cleans VIA state needs the exact name.
#[must_use]
pub fn lease_temp_path(lock_path: &Path, instance_id: &str) -> PathBuf {
    sibling(lock_path, &format!(".{instance_id}.tmp"))
}

/// `<lockPath>.stale.<token>` — where a stale lease is renamed before it is
/// deleted.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:77`. The
/// rename is what makes reclaiming a dead instance's lease atomic: exactly one
/// racing process gets the rename, and only that one retries the create.
#[must_use]
pub fn lease_stale_path(lock_path: &Path, token: &str) -> PathBuf {
    sibling(lock_path, &format!(".stale.{token}"))
}

/// Append a suffix to the final path component, as JavaScript template-string
/// concatenation on a path does.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut raw = path.as_os_str().to_os_string();
    raw.push(suffix);
    PathBuf::from(raw)
}

/// Read `<configDirectory>/gateway.lock`, or `None` if there is no readable,
/// parseable lease with a matching schema.
///
/// **External contract.** Upstream `readGatewayLease`
/// (`shared/gateway-instance-lock.mjs:29-39`): every failure — missing file,
/// unreadable file, malformed JSON, wrong schema, absent `instanceId` — is one
/// and the same `None`. Callers must not be able to tell a corrupt lease from
/// an absent one, because both mean "no incumbent I can talk to".
#[must_use]
pub fn read_gateway_lease(config_directory: &Path) -> Option<GatewayLease> {
    read_lease_file(&gateway_lock_path(config_directory))
}

/// [`read_gateway_lease`] against an explicit path.
pub(crate) fn read_lease_file(path: &Path) -> Option<GatewayLease> {
    let raw = fs::read_to_string(path).ok()?;
    let lease: GatewayLease = serde_json::from_str(&raw).ok()?;
    (lease.schema == GATEWAY_LOCK_SCHEMA).then_some(lease)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_file_names_match_upstream() {
        let lock = Path::new("/cfg/gateway.lock");
        assert_eq!(
            lease_temp_path(lock, "abc"),
            Path::new("/cfg/gateway.lock.abc.tmp")
        );
        assert_eq!(
            lease_stale_path(lock, "abc"),
            Path::new("/cfg/gateway.lock.stale.abc")
        );
    }

    #[test]
    fn same_lease_requires_a_non_empty_instance_id_and_a_matching_pid() {
        let base = GatewayLease {
            schema: GATEWAY_LOCK_SCHEMA.to_owned(),
            instance_id: "a".to_owned(),
            pid: 7,
            owner: DEFAULT_LEASE_OWNER.to_owned(),
            state: LEASE_STATE_STARTING.to_owned(),
            origin: String::new(),
            started_at: String::new(),
            heartbeat_at: String::new(),
        };

        assert!(base.is_same_lease(&base.clone()));

        let mut other_pid = base.clone();
        other_pid.pid = 8;
        assert!(!base.is_same_lease(&other_pid));

        let mut other_id = base.clone();
        other_id.instance_id = "b".to_owned();
        assert!(!base.is_same_lease(&other_id));

        let mut anonymous = base.clone();
        anonymous.instance_id = String::new();
        assert!(!anonymous.is_same_lease(&anonymous.clone()));
    }
}
