//! Taking the lease, keeping it warm, and giving it back.
//!
//! Ported from upstream `shared/gateway-instance-lock.mjs:44-166`.

use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::LeaseError;
use crate::lease::{
    DEFAULT_LEASE_OWNER, GATEWAY_LOCK_SCHEMA, GatewayLease, LEASE_STATE_STARTING,
    gateway_lock_path, lease_stale_path, lease_temp_path, read_lease_file,
};
use crate::probe::{
    Clock, ProcessProbe, SystemClock, SystemProcessProbe, current_pid, iso8601, process_is_alive,
};

/// How many times acquisition will lose the race before giving up.
///
/// **External contract.** Upstream `shared/gateway-instance-lock.mjs:98` —
/// `for (let attempt = 0; attempt < 4; attempt += 1)`. The bound matters: each
/// pass that finds a *dead* incumbent renames the lease aside and tries again,
/// and without a cap two processes reclaiming each other's leases would spin
/// forever instead of surfacing [`LeaseError::Exhausted`].
pub const MAX_ACQUIRE_ATTEMPTS: usize = 4;

/// Knobs for [`acquire_gateway_lease`], defaulted exactly as upstream defaults
/// them (`shared/gateway-instance-lock.mjs:87-93`).
#[derive(Debug)]
pub struct AcquireOptions {
    pid: i64,
    owner: String,
    instance_id: String,
    clock: Box<dyn Clock>,
    probe: Box<dyn ProcessProbe>,
}

impl Default for AcquireOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl AcquireOptions {
    /// Upstream's defaults: this process's pid, owner
    /// [`DEFAULT_LEASE_OWNER`], a fresh UUID v4 instance id, the system clock
    /// and the platform liveness probe.
    ///
    /// Note that this generates a random identity, so two calls are not
    /// interchangeable.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pid: current_pid(),
            owner: DEFAULT_LEASE_OWNER.to_owned(),
            instance_id: Uuid::new_v4().to_string(),
            clock: Box::new(SystemClock),
            probe: Box::new(SystemProcessProbe),
        }
    }

    /// The pid written into the lease and probed by any later challenger.
    #[must_use]
    pub fn pid(mut self, pid: i64) -> Self {
        self.pid = pid;
        self
    }

    /// Which form owns this Gateway — the process passes `"cli"` or
    /// `"desktop"`.
    #[must_use]
    pub fn owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = owner.into();
        self
    }

    /// Override the generated instance id. Also the token in the
    /// `<path>.stale.<token>` reclaim name.
    #[must_use]
    pub fn instance_id(mut self, instance_id: impl Into<String>) -> Self {
        self.instance_id = instance_id.into();
        self
    }

    /// Replace the clock. The handle keeps it for `heartbeatAt`.
    #[must_use]
    pub fn clock(mut self, clock: Box<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Replace the liveness probe, which decides whether an incumbent lease is
    /// a conflict or a corpse.
    #[must_use]
    pub fn probe(mut self, probe: Box<dyn ProcessProbe>) -> Self {
        self.probe = probe;
        self
    }
}

/// The fields of a held lease a caller may change.
///
/// **External contract, expressed as a type.** Upstream's `update(fields)` does
/// `Object.assign(lease, fields, { schema, instanceId, pid, heartbeatAt })`
/// (`shared/gateway-instance-lock.mjs:120-125`): the trailing object re-stamps
/// the identity fields *after* the caller's, so a caller cannot rewrite them
/// even by passing them. Here they are simply absent from `LeaseUpdate`, so the
/// same guarantee is structural rather than a runtime overwrite.
#[derive(Debug, Clone, Default)]
pub struct LeaseUpdate {
    owner: Option<String>,
    state: Option<String>,
    origin: Option<String>,
    started_at: Option<String>,
}

impl LeaseUpdate {
    /// An empty update — a bare heartbeat. This is what upstream's
    /// `setInterval(() => lease.update(), 15_000)` sends.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Change the owner form.
    #[must_use]
    pub fn owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// Change the lifecycle state — [`LEASE_STATE_READY`](crate::LEASE_STATE_READY)
    /// once the listener is bound.
    #[must_use]
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Publish where this Gateway is listening, `http://<host>:<port>`. Until
    /// this is set, [`find_running_gateway`](crate::find_running_gateway)
    /// cannot hand anyone the instance.
    #[must_use]
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    /// Rewrite `startedAt`. Upstream permits it; no upstream caller does it.
    #[must_use]
    pub fn started_at(mut self, started_at: impl Into<String>) -> Self {
        self.started_at = Some(started_at.into());
        self
    }
}

/// A lease this process currently holds.
///
/// # Dropping is not releasing
///
/// There is deliberately no `Drop` impl, and `release` takes `self` by value,
/// so "release" is something you can only spell out loud and only once. Three
/// reasons, in increasing order of how much they would cost to get wrong:
///
/// * Release does file I/O and can fail. `Drop` cannot return a `Result`, so a
///   `Drop`-based release would have to swallow the error or panic — and the
///   caller genuinely wants to know that the lease file outlived the process.
/// * Release is *conditional*: it deletes the file only while the file is still
///   ours (see [`GatewayLease::is_same_lease`]). A destructor firing on an
///   early return, inside a closure that took the handle by value, or in a
///   forked child would run that check at a moment nobody chose.
/// * Upstream releases at exactly two points — the `process.once('exit')`
///   handler and the startup `catch` (`server/src/index.mjs:121-126,141-147`) —
///   and nowhere else. Handing that decision to scope exit would be a
///   behavioural change disguised as idiom.
///
/// The absence is enforced by the compiler, not by this comment: see the
/// destructuring in [`Self::release`].
#[derive(Debug)]
pub struct GatewayLeaseHandle {
    path: PathBuf,
    lease: GatewayLease,
    clock: Box<dyn Clock>,
}

impl GatewayLeaseHandle {
    /// The lease file this handle owns.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// This instance's id — also what `/api/health` reports as
    /// `gatewayInstanceId`.
    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.lease.instance_id
    }

    /// The pid recorded in the lease.
    #[must_use]
    pub fn pid(&self) -> i64 {
        self.lease.pid
    }

    /// The document as this process last wrote it.
    #[must_use]
    pub fn lease(&self) -> &GatewayLease {
        &self.lease
    }

    /// Rewrite the lease, re-stamping `heartbeatAt`.
    ///
    /// **External contract.** Upstream `update`
    /// (`shared/gateway-instance-lock.mjs:116-128`). Returns `Ok(false)`
    /// without touching the file when the lease on disk is no longer ours —
    /// another instance took over, or something deleted it — which is how a
    /// heartbeat discovers it has been evicted. The write is a temp file plus a
    /// rename, so a reader never sees a half-written document.
    ///
    /// # Errors
    ///
    /// [`LeaseError::Io`] if the temp file cannot be written or renamed.
    pub fn update(&mut self, fields: LeaseUpdate) -> Result<bool, LeaseError> {
        let Some(current) = read_lease_file(&self.path) else {
            return Ok(false);
        };
        if !current.is_same_lease(&self.lease) {
            return Ok(false);
        }

        if let Some(owner) = fields.owner {
            self.lease.owner = owner;
        }
        if let Some(state) = fields.state {
            self.lease.state = state;
        }
        if let Some(origin) = fields.origin {
            self.lease.origin = origin;
        }
        if let Some(started_at) = fields.started_at {
            self.lease.started_at = started_at;
        }
        // `schema`, `instanceId` and `pid` are re-stamped upstream because
        // `Object.assign` would otherwise let a caller-supplied field overwrite
        // them. `LeaseUpdate` cannot express those fields at all, so they are
        // already exactly what acquisition wrote.
        self.lease.heartbeat_at = iso8601(self.clock.now());

        replace_lease(&self.path, &self.lease)?;
        Ok(true)
    }

    /// A bare heartbeat: re-stamp `heartbeatAt` and change nothing else.
    ///
    /// The owning process calls this every
    /// [`GATEWAY_HEARTBEAT_INTERVAL`](crate::GATEWAY_HEARTBEAT_INTERVAL).
    ///
    /// # Errors
    ///
    /// As [`Self::update`].
    pub fn heartbeat(&mut self) -> Result<bool, LeaseError> {
        self.update(LeaseUpdate::new())
    }

    /// Delete the lease file, if it is still ours.
    ///
    /// **External contract.** Upstream `release`
    /// (`shared/gateway-instance-lock.mjs:129-141`). Returns `Ok(false)` — and
    /// leaves the file alone — when another instance now owns it, or when it
    /// has already gone. A clean shutdown must reach this; see the type-level
    /// note on why it is never automatic.
    ///
    /// Consuming `self` is what makes upstream's `released` flag unnecessary:
    /// a second release is not something this API can express.
    ///
    /// # Errors
    ///
    /// [`LeaseError::Io`] if the file exists, is ours, and cannot be removed.
    pub fn release(self) -> Result<bool, LeaseError> {
        // Taking `self` apart field by field is what keeps `Drop` off this
        // type: a type that implements `Drop` cannot be destructured (E0509),
        // so an accidental `impl Drop for GatewayLeaseHandle` added later
        // fails to compile on this line rather than quietly deleting a live
        // instance's lease at the end of some unrelated scope.
        let Self {
            path,
            lease,
            clock: _,
        } = self;

        let Some(current) = read_lease_file(&path) else {
            return Ok(false);
        };
        if !current.is_same_lease(&lease) {
            return Ok(false);
        }
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(source) if source.kind() == ErrorKind::NotFound => Ok(false),
            Err(source) => Err(LeaseError::io(&path, source)),
        }
    }
}

/// Take the single-instance lease for `config_directory`.
///
/// **External contract.** Upstream `acquireGatewayLease`
/// (`shared/gateway-instance-lock.mjs:86-166`). The sequence, which other
/// processes racing on the same directory depend on:
///
/// 1. Create the configuration directory if needed, mode
///    [`CONFIG_DIRECTORY_MODE`].
/// 2. Create `gateway.lock` with `O_EXCL` and mode
///    [`LEASE_FILE_MODE`](crate::LEASE_FILE_MODE) — the create *is* the lock,
///    so exactly one racer can win it.
/// 3. On `EEXIST`, read the incumbent. If its pid is alive, fail with
///    [`LeaseError::AlreadyRunning`]. Otherwise rename the stale file aside,
///    delete it, and retry.
/// 4. At most [`MAX_ACQUIRE_ATTEMPTS`] passes, then
///    [`LeaseError::Exhausted`].
///
/// Note the asymmetry in step 3: an *unreadable* incumbent — truncated,
/// foreign schema, half-written by a process that died mid-create — counts as
/// dead and is reclaimed. That is upstream's behaviour and it is the right one:
/// a lease nobody can parse names nobody.
///
/// # Errors
///
/// [`LeaseError::AlreadyRunning`] when a live Gateway holds the lease,
/// [`LeaseError::Exhausted`] when every attempt lost the race, and
/// [`LeaseError::Io`] for any filesystem failure other than the expected
/// `EEXIST`.
pub fn acquire_gateway_lease(
    config_directory: &Path,
    options: AcquireOptions,
) -> Result<GatewayLeaseHandle, LeaseError> {
    create_config_directory(config_directory)?;
    let path = gateway_lock_path(config_directory);
    let AcquireOptions {
        pid,
        owner,
        instance_id,
        clock,
        probe,
    } = options;

    for _attempt in 0..MAX_ACQUIRE_ATTEMPTS {
        let timestamp = iso8601(clock.now());
        let lease = GatewayLease {
            schema: GATEWAY_LOCK_SCHEMA.to_owned(),
            instance_id: instance_id.clone(),
            pid,
            owner: owner.clone(),
            state: LEASE_STATE_STARTING.to_owned(),
            origin: String::new(),
            started_at: timestamp.clone(),
            heartbeat_at: timestamp,
        };

        match write_new_lease(&path, &lease) {
            Ok(()) => return Ok(GatewayLeaseHandle { path, lease, clock }),
            Err(source) if source.kind() == ErrorKind::AlreadyExists => {
                if let Some(existing) = read_lease_file(&path)
                    && process_is_alive(existing.pid, probe.as_ref())
                {
                    return Err(LeaseError::AlreadyRunning {
                        lease: Box::new(existing),
                    });
                }
                // Either outcome retries: `false` means another process
                // reclaimed the stale lease first, `true` that this one did.
                let _reclaimed = move_stale_lease(&path, &instance_id)?;
            }
            Err(source) => return Err(LeaseError::io(&path, source)),
        }
    }

    Err(LeaseError::Exhausted)
}

/// `mkdir -p` with mode [`CONFIG_DIRECTORY_MODE`](crate::CONFIG_DIRECTORY_MODE),
/// as upstream's `mkdirSync(configDirectory, { recursive: true, mode: 0o700 })`.
fn create_config_directory(config_directory: &Path) -> Result<(), LeaseError> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(crate::lease::CONFIG_DIRECTORY_MODE);
    }
    builder
        .create(config_directory)
        .map_err(|source| LeaseError::io(config_directory, source))
}

/// Create `path` exclusively and write the lease plus a trailing newline.
///
/// **External contract.** Upstream `writeLease`
/// (`shared/gateway-instance-lock.mjs:44-52`) opens with the `'wx'` flag —
/// `O_WRONLY|O_CREAT|O_EXCL` — at mode `0o600`. `O_EXCL` is the mutual
/// exclusion primitive this whole crate rests on; `create_new(true)` is the
/// same syscall. On non-Unix the mode is left to the platform's inherited
/// ACLs, which is the one place the file is less protected than upstream's.
///
/// Returns the raw [`io::Error`] so the caller can recognise
/// [`ErrorKind::AlreadyExists`], which is not a failure but the contended path.
fn write_new_lease(path: &Path, lease: &GatewayLease) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(crate::lease::LEASE_FILE_MODE);
    }
    let mut file = options.open(path)?;
    let mut body = serde_json::to_string(lease).map_err(io::Error::other)?;
    body.push('\n');
    file.write_all(body.as_bytes())
}

/// Replace an existing lease atomically: write `<path>.<instanceId>.tmp`, then
/// rename it over the lease.
///
/// **External contract.** Upstream `replaceLease`
/// (`shared/gateway-instance-lock.mjs:54-71`), including the cleanup shape: on
/// failure the temp file is removed and the error propagates; on success the
/// rename has already consumed the temp name, so the trailing unlink is
/// expected to find nothing.
///
/// A leftover temp file from an earlier crash makes the exclusive create fail;
/// the failure path then unlinks it, so the next update succeeds. That
/// self-healing is upstream's and is reproduced rather than tidied away.
///
/// No `fsync`, matching upstream. A machine that loses power mid-rename may
/// leave a lease naming a process that no longer exists, which is precisely the
/// case the liveness probe handles.
fn replace_lease(path: &Path, lease: &GatewayLease) -> Result<(), LeaseError> {
    let temporary = lease_temp_path(path, &lease.instance_id);

    let written = write_new_lease(&temporary, lease).and_then(|()| fs::rename(&temporary, path));
    if let Err(source) = written {
        let _ = fs::remove_file(&temporary);
        return Err(LeaseError::io(&temporary, source));
    }

    match fs::remove_file(&temporary) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(()),
        Err(source) => Err(LeaseError::io(&temporary, source)),
    }
}

/// Rename a dead instance's lease to `<path>.stale.<token>` and delete it.
///
/// **External contract.** Upstream `moveStaleLease`
/// (`shared/gateway-instance-lock.mjs:73-85`). The rename is the atomic part:
/// several processes may all decide the incumbent is dead, but only one
/// `rename(2)` moves it, so only one gets to re-create the lease from a clean
/// slate. `false` means the file was already gone.
fn move_stale_lease(path: &Path, token: &str) -> Result<bool, LeaseError> {
    let stale = lease_stale_path(path, token);
    match fs::rename(path, &stale) {
        Ok(()) => {}
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(false),
        Err(source) => return Err(LeaseError::io(path, source)),
    }
    match fs::remove_file(&stale) {
        Ok(()) => Ok(true),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(true),
        Err(source) => Err(LeaseError::io(&stale, source)),
    }
}
