//! Downloading, verifying and installing a keyword model.
//!
//! Ported from upstream `server/src/voice/wake-word/model-manager.mjs`.
//!
//! # The sequence
//!
//! ```text
//! already installed? ──yes──► rewrite keywords.txt ──► done
//!         │no
//!         ▼
//!   GET <release url>          ── not 2xx, or no body ──► Download
//!         │
//!   SHA-256 == artifact.sha256 ── no ─────────────────► Checksum
//!         │yes
//!   extract 4 members into  <root>/.<id>-<nonce>/      (0o700, 0o600)
//!   write keywords.txt into <root>/.<id>-<nonce>/      (0o600)
//!   all 5 present?             ── no ─────────────────► Incomplete
//!         │yes
//!   rename staging ──► <root>/<id>
//! ```
//!
//! Nothing is written outside the staging directory until every check has
//! passed, so a failed install leaves the previous one — or no install at all —
//! exactly as it was. That is the property the digest is there to buy, and
//! staging is what makes it true for the extraction as well as the download.
//!
//! # Four deliberate departures from upstream
//!
//! **The archive never touches the disk.** Upstream streams the body to a temp
//! file, re-reads it to hash it, then re-reads it again to extract it, and
//! cleans the file up in a `finally`. VIA hashes and extracts the same buffer
//! it downloaded. It removes a temp file, its cleanup path, and the window
//! between hashing a file and reading it back. The cost is holding the archive
//! — roughly 33 MB, once, during a first run.
//!
//! **`keywords.txt` is rewritten on every call.** Upstream's completeness check
//! includes the keyword file, so once a model is installed the keyword file is
//! never regenerated. Upstream can afford that: its phrase is a hard-coded
//! literal. VIA's phrase is configuration (`docs/architecture.md` §16), so an
//! operator changing `VIA_WAKE_WORD` against an already-installed model would
//! otherwise keep listening for the previous phrase forever, with no error
//! anywhere. The short-circuit therefore tests the four *downloaded* members,
//! and the keyword file is written afterwards either way — atomically, through
//! `via_store::write_atomic`, since it is being replaced under a detector that
//! may be reading it.
//!
//! **Concurrent installs serialize instead of sharing a promise.** Upstream
//! keys a module-level `Map` on the target directory so two callers await one
//! download. VIA holds a per-target async mutex instead: the second caller
//! waits, then finds the model installed and takes the short-circuit. The
//! observable behaviour is the same for the success case — one download — and
//! better for the failure case, where upstream hands the second caller the
//! first caller's error and VIA lets it retry.
//!
//! **Only a basename is ever joined.** Upstream takes `basename(header.name)`
//! for its own reasons; VIA keeps that and states the consequence, which is
//! that a `../../` member name in a hostile archive cannot escape the staging
//! directory. It is tested that way.

use std::collections::HashMap;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bzip2::read::BzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;
use via_store::FILE_MODE;

use crate::artifact::ModelArtifact;
use crate::detect::{DetectionConfig, ModelPaths};
use crate::error::{KeywordError, Result, WakeWordError};
use crate::tokens::TokenInventory;

/// Mode for the model root and every staging directory.
///
/// **External contract** — `model-manager.mjs:78,83`
/// (`mkdirSync(..., { mode: 0o700 })`). Enforced on Unix; Windows has no mode
/// bits and inherits the parent ACL, exactly as `via-store` documents for its
/// own directories.
pub const MODEL_DIR_MODE: u32 = 0o700;

/// One HTTP response, reduced to what the installer branches on.
///
/// **External contract** — `model-manager.mjs:50-53`: upstream checks
/// `response.ok` *and* `response.body`, and reports either as the same error
/// carrying the status. Both halves are representable here, which is why
/// [`Self::body`] is an `Option` rather than an empty `Vec`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchResponse {
    /// The HTTP status.
    pub status: u16,
    /// The body, or `None` for a response that carried none.
    pub body: Option<Vec<u8>>,
}

impl FetchResponse {
    /// A 200 with this body.
    #[must_use]
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            body: Some(body),
        }
    }

    /// A failure status with no body.
    #[must_use]
    pub fn status(status: u16) -> Self {
        Self { status, body: None }
    }

    /// JavaScript's `response.ok`: a status in `200..=299`.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The body, when the response both succeeded and carried one.
    fn usable_body(self) -> core::result::Result<Vec<u8>, u16> {
        let status = self.status;
        match (self.is_ok(), self.body) {
            (true, Some(body)) => Ok(body),
            _ => Err(status),
        }
    }
}

/// The one network call the installer makes.
///
/// A trait, not a `reqwest::Client`, so every branch below — a 404, a 200 with
/// no body, a truncated archive, a digest mismatch, a hostile member name — is
/// reachable in a test with no network and no fixture server. The production
/// implementation is [`HttpModelFetch`], behind the `http` feature; a Gateway
/// that already owns a client can implement this over that one instead of
/// linking a second.
#[async_trait]
pub trait ModelFetch: Send + Sync + core::fmt::Debug {
    /// `GET url`, following redirects, reading the whole body.
    ///
    /// # Errors
    ///
    /// [`WakeWordError::Fetch`] when the request could not be made or the body
    /// could not be read. A non-2xx *answer* is not an error here — it is a
    /// [`FetchResponse`] the installer classifies.
    async fn fetch(&self, url: &str) -> Result<FetchResponse>;
}

/// The production [`ModelFetch`]: one `reqwest` GET.
#[cfg(feature = "http")]
#[derive(Clone, Debug, Default)]
pub struct HttpModelFetch {
    client: reqwest::Client,
}

#[cfg(feature = "http")]
impl HttpModelFetch {
    /// A fetcher with its own client.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A fetcher over a client the caller already has.
    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[cfg(feature = "http")]
#[async_trait]
impl ModelFetch for HttpModelFetch {
    async fn fetch(&self, url: &str) -> Result<FetchResponse> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| WakeWordError::Fetch {
                url: url.to_owned(),
                detail: error.to_string(),
            })?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Ok(FetchResponse::status(status));
        }
        let body = response
            .bytes()
            .await
            .map_err(|error| WakeWordError::Fetch {
                url: url.to_owned(),
                detail: error.to_string(),
            })?;
        Ok(FetchResponse {
            status,
            body: Some(body.to_vec()),
        })
    }
}

/// An installed keyword model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelInstall {
    directory: PathBuf,
    artifact: ModelArtifact,
}

impl ModelInstall {
    /// `<root>/<artifact id>`.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// What was installed.
    #[must_use]
    pub fn artifact(&self) -> &ModelArtifact {
        &self.artifact
    }

    /// The four model files, resolved.
    #[must_use]
    pub fn model_paths(&self) -> ModelPaths {
        ModelPaths::resolve(&self.directory, &self.artifact)
    }

    /// Where the generated keyword file lives.
    #[must_use]
    pub fn keywords_path(&self) -> PathBuf {
        self.directory.join(self.artifact.files.keywords.as_ref())
    }

    /// The catalogued detection configuration for this install.
    #[must_use]
    pub fn detection_config(&self, keywords_buf: impl Into<String>) -> DetectionConfig {
        DetectionConfig::new(self.model_paths(), keywords_buf)
    }
}

/// Installs keyword models under one root, one directory per artifact id.
#[derive(Debug)]
pub struct ModelManager {
    root: PathBuf,
    fetch: Arc<dyn ModelFetch>,
    locks: Mutex<HashMap<PathBuf, Arc<tokio::sync::Mutex<()>>>>,
}

impl ModelManager {
    /// A manager rooted at `root`, downloading through `fetch`.
    ///
    /// `root` is `<config>/models/wake-word` in a real Gateway — see
    /// [`WakeWordSettings::model_root`](crate::WakeWordSettings::model_root),
    /// which reads it from `Config`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, fetch: Arc<dyn ModelFetch>) -> Self {
        Self {
            root: root.into(),
            fetch,
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// The root every artifact installs under.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Where `artifact` installs.
    ///
    /// **External contract** — `model-manager.mjs:104`
    /// (`resolve(directory, WAKE_WORD_MODEL_NAME)`): the model id is the
    /// directory name.
    #[must_use]
    pub fn install_directory(&self, artifact: &ModelArtifact) -> PathBuf {
        self.root.join(artifact.id.as_ref())
    }

    /// Whether every one of `artifact`'s five files is already present.
    ///
    /// **External contract** — `model-manager.mjs:37-39` (`complete`).
    #[must_use]
    pub fn is_installed(&self, artifact: &ModelArtifact) -> bool {
        let directory = self.install_directory(artifact);
        artifact
            .files
            .required()
            .iter()
            .all(|name| directory.join(name).is_file())
    }

    /// Install `artifact` if it is not installed, and write `keywords` either
    /// way.
    ///
    /// `keywords` is the whole `keywords.txt` content —
    /// [`KeywordSet::keywords_file`](crate::KeywordSet::keywords_file) produces
    /// it.
    ///
    /// # Errors
    ///
    /// - [`KeywordError::Empty`] when `keywords` is blank. A model installed
    ///   with no keywords listens for nothing and reports nothing, which is the
    ///   single hardest wake-word failure to notice; it is refused before the
    ///   download rather than after it.
    /// - [`WakeWordError::Download`] for a non-2xx answer or a missing body.
    /// - [`WakeWordError::Checksum`] when the archive is not the pinned one.
    /// - [`WakeWordError::Archive`] when it cannot be decompressed or walked.
    /// - [`WakeWordError::UnknownTokens`] when the model cannot encode
    ///   `keywords`. Checked against the model's own `tokens.txt` **before** the
    ///   keyword file is written, because the library's answer to a token it
    ///   cannot find is to end the process.
    /// - [`WakeWordError::Incomplete`] when a member was missing from it.
    /// - [`WakeWordError::Io`] for anything the filesystem refused.
    pub async fn ensure(&self, artifact: &ModelArtifact, keywords: &str) -> Result<ModelInstall> {
        if keywords.trim().is_empty() {
            return Err(KeywordError::Empty { field: "keywords" }.into());
        }
        let directory = self.install_directory(artifact);
        let install = ModelInstall {
            directory: directory.clone(),
            artifact: artifact.clone(),
        };

        if has_archive_members(&directory, artifact) {
            refresh_keywords(&directory, artifact, keywords)?;
            return Ok(install);
        }

        let gate = self.gate_for(&directory);
        let _held = gate.lock().await;

        // Another task may have finished while this one waited.
        if has_archive_members(&directory, artifact) {
            refresh_keywords(&directory, artifact, keywords)?;
            return Ok(install);
        }

        let url = artifact.url();
        let body = self
            .fetch
            .fetch(&url)
            .await?
            .usable_body()
            .map_err(|status| WakeWordError::Download { status })?;

        let root = self.root.clone();
        let owned_artifact = artifact.clone();
        let owned_keywords = keywords.to_owned();
        tokio::task::spawn_blocking(move || {
            install_verified(&root, &owned_artifact, &body, &owned_keywords)
        })
        .await
        .map_err(|error| WakeWordError::InstallAbandoned {
            detail: error.to_string(),
        })??;

        Ok(install)
    }

    /// The per-target gate, created on first use.
    fn gate_for(&self, directory: &Path) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = lock_map(&self.locks);
        Arc::clone(
            locks
                .entry(directory.to_path_buf())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))),
        )
    }
}

/// Lock the gate map, treating a poisoned mutex as a live one.
///
/// The map holds `Arc`s and nothing else; a panic while it is held cannot leave
/// a half-updated value behind, so refusing to proceed would turn an unrelated
/// panic into a permanently broken installer.
fn lock_map<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Verify, extract and promote — the whole blocking half, off the runtime.
fn install_verified(
    root: &Path,
    artifact: &ModelArtifact,
    body: &[u8],
    keywords: &str,
) -> Result<PathBuf> {
    verify_digest(artifact, body)?;
    create_dir_all_mode(root)?;

    let staging = root.join(format!(".{}-{}", artifact.id, nonce()));
    create_dir_exclusive(&staging)?;

    let outcome = stage_and_promote(root, &staging, artifact, body, keywords);
    if outcome.is_err() {
        // Best effort: the install already failed, and failing to clean up a
        // staging directory must not replace the reason it failed.
        let _ = fs::remove_dir_all(&staging);
    }
    outcome
}

fn stage_and_promote(
    root: &Path,
    staging: &Path,
    artifact: &ModelArtifact,
    body: &[u8],
    keywords: &str,
) -> Result<PathBuf> {
    extract_members(body, staging, artifact)?;
    // The four downloaded members first, so an archive that was missing one is
    // reported as the incomplete archive it is rather than as whatever the next
    // step trips over trying to read it.
    assert_present(staging, &artifact.files.archived())?;
    // Then: the model has to be able to encode the keyword file that is about to
    // be written next to it. See `crate::tokens` — the library's answer to a
    // token it cannot find is to end the process, so this check is the
    // difference between a bad configuration and a Gateway that will not start.
    TokenInventory::read(&staging.join(artifact.files.tokens.as_ref()))?.validate(keywords)?;
    write_file_mode(
        &staging.join(artifact.files.keywords.as_ref()),
        keywords.as_bytes(),
    )?;
    assert_complete(staging, artifact)?;

    let target = root.join(artifact.id.as_ref());
    if target.exists() {
        fs::remove_dir_all(&target)
            .map_err(|error| WakeWordError::io("remove", &target, &error))?;
    }
    fs::rename(staging, &target).map_err(|error| WakeWordError::io("rename", staging, &error))?;
    Ok(target)
}

/// SHA-256 the archive and compare it to the pinned digest.
///
/// **External contract** — `model-manager.mjs:87-91`. The comparison ignores
/// hex case so a digest configured in upper case still verifies; the value
/// itself is public, so there is nothing here for a timing side channel to
/// leak.
fn verify_digest(artifact: &ModelArtifact, body: &[u8]) -> Result<()> {
    let actual = hex::encode(Sha256::digest(body));
    if actual.eq_ignore_ascii_case(artifact.sha256.as_ref()) {
        return Ok(());
    }
    Err(WakeWordError::Checksum {
        expected: artifact.sha256.to_string(),
        actual,
    })
}

/// Extract the four archive members, by basename, into `staging`.
fn extract_members(body: &[u8], staging: &Path, artifact: &ModelArtifact) -> Result<()> {
    let mut archive = Archive::new(BzDecoder::new(body));
    let entries = archive.entries().map_err(archive_error)?;
    for entry in entries {
        let mut entry = entry.map_err(archive_error)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(archive_error)?;
        // `file_name` is the whole traversal defence: it is `None` for `..` and
        // for a path that ends in a separator, and it can never contain one, so
        // the join below stays inside `staging` for any member name at all.
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !artifact.files.is_archived(name) {
            continue;
        }
        let destination = staging.join(name);
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(archive_error)?;
        write_file_mode(&destination, &bytes)?;
    }
    Ok(())
}

fn archive_error(error: std::io::Error) -> WakeWordError {
    WakeWordError::Archive {
        detail: error.to_string(),
    }
}

/// Whether the four *downloaded* members are present.
///
/// The short-circuit condition. Deliberately not the five-file check upstream
/// uses — see the module docs on why `keywords.txt` is rewritten every time.
fn has_archive_members(directory: &Path, artifact: &ModelArtifact) -> bool {
    artifact
        .files
        .archived()
        .iter()
        .all(|name| directory.join(name).is_file())
}

/// Fail unless every one of the five files is present.
///
/// **External contract** — `model-manager.mjs:97-99`.
fn assert_complete(directory: &Path, artifact: &ModelArtifact) -> Result<()> {
    assert_present(directory, &artifact.files.required())
}

/// Fail unless every name in `names` is a regular file in `directory`.
fn assert_present(directory: &Path, names: &[&str]) -> Result<()> {
    let missing: Vec<String> = names
        .iter()
        .filter(|name| !directory.join(name).is_file())
        .map(|name| (*name).to_owned())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    Err(WakeWordError::Incomplete { missing })
}

/// Validate and rewrite the keyword file of a model that is already installed,
/// then confirm the install is complete.
///
/// The short-circuit path. The validation is not a formality: the phrase is
/// configuration, so this is the call that runs when an operator has changed
/// `VIA_WAKE_WORD` against a model that was installed for the previous one. A
/// token line the model cannot encode is refused here, leaving the working
/// keyword file in place, rather than being written and taking the process down
/// the next time a detector opens.
fn refresh_keywords(directory: &Path, artifact: &ModelArtifact, keywords: &str) -> Result<()> {
    TokenInventory::read(&directory.join(artifact.files.tokens.as_ref()))?.validate(keywords)?;
    write_keywords(directory, artifact, keywords)?;
    assert_complete(directory, artifact)
}

/// Write the keyword file into an installed model, atomically.
///
/// **External contract** — `model-manager.mjs:91-95` writes it with
/// `mode: 0o600`, which is `via_store::FILE_MODE`.
/// [`via_store::write_atomic`] adds the two durability barriers VIA gives every
/// other file it owns, and — the reason it is used here rather than a plain
/// write — makes the replacement atomic for a detector that may be reading the
/// file at the same moment.
fn write_keywords(directory: &Path, artifact: &ModelArtifact, keywords: &str) -> Result<()> {
    let target = directory.join(artifact.files.keywords.as_ref());
    let temp = directory.join(format!(".{}.{}", artifact.files.keywords, nonce()));
    via_store::write_atomic(&temp, &target, keywords)
        .map_err(|error| WakeWordError::io("write", &target, &error))
}

/// Create `path` and its parents with [`MODEL_DIR_MODE`].
///
/// **External contract** — `model-manager.mjs:78`.
fn create_dir_all_mode(path: &Path) -> Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(MODEL_DIR_MODE);
    }
    builder
        .create(path)
        .map_err(|error| WakeWordError::io("create", path, &error))
}

/// Create `path` and nothing else, failing if it already exists.
///
/// **External contract** — `model-manager.mjs:83`
/// (`mkdirSync(stagingDirectory, { recursive: false, mode: 0o700 })`). The
/// exclusivity is the point: a staging directory that already exists is either
/// a nonce collision or something else's, and either way its contents must not
/// be promoted over the installed model.
fn create_dir_exclusive(path: &Path) -> Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(MODEL_DIR_MODE);
    }
    builder
        .create(path)
        .map_err(|error| WakeWordError::io("create", path, &error))
}

/// Write `bytes` to `path` with [`via_store::FILE_MODE`], creating it.
///
/// **External contract** — `model-manager.mjs:62-64,91-95` (`mode: 0o600`).
fn write_file_mode(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(FILE_MODE);
    }
    let mut file = options
        .open(path)
        .map_err(|error| WakeWordError::io("create", path, &error))?;
    file.write_all(bytes)
        .map_err(|error| WakeWordError::io("write", path, &error))
}

/// A staging suffix no two installs can share.
///
/// **External contract** — `model-manager.mjs:79`
/// (`` `${process.pid}-${Date.now()}` ``), plus a process-local counter,
/// because two installs started inside the same millisecond by the same process
/// is not a theoretical case here: it is what
/// [`WakeWordSettings::artifacts`](crate::WakeWordSettings::artifacts) produces
/// when two locales resolve to two different models.
fn nonce() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis())
        .unwrap_or(0);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{millis}-{sequence}", std::process::id())
}
