//! A synthetic machine, two real binaries, and a consumer's clients.
//!
//! Everything here is about the *process boundary*. The child's environment is
//! **cleared** and rebuilt from an explicit allow-list, because
//! `std::env::set_var` is `unsafe` in edition 2024 and an inherited environment
//! would make these tests depend on whatever the developer happens to export —
//! including a real `DASHSCOPE_API_KEY`, which would silently turn the
//! first-run case into a no-op.
//!
//! # Two binaries, found two different ways
//!
//! [`VIA`] is the shipped binary. `CARGO_BIN_EXE_via` is **not** available here:
//! cargo sets it only for integration tests of the package that declares the
//! bin, and that package is `apps/via`. So it is located by path, next to this
//! crate's own binary, and built on demand if it is not there — which is what
//! `escargot` does, minus a dependency.
//!
//! [`HARNESS_GATEWAY`] is this crate's own `via-gateway-e2e`, so
//! `CARGO_BIN_EXE_via-gateway-e2e` *is* set and the path is exact.
//!
//! # The log directory is not redirected
//!
//! `apps/via`'s own fixture points `VIA_LOG_DIR` at a scratch directory. This
//! one deliberately does not: `<config>/logs/gateway.log` is a catalogued path
//! (`file-path` / *files and directories the product creates*), the first-run
//! case audits it, and the security case greps it. Redirecting it would make
//! both of those assert against a location the product never uses.

#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tempfile::TempDir;
use via_realtime_mock::Script;

/// The shipped binary's target name.
pub const VIA: &str = "via";

/// This crate's harness Gateway.
///
/// Its path is exact — `CARGO_BIN_EXE_via-gateway-e2e` is set for this test
/// binary because the bin target belongs to this package.
pub const HARNESS_GATEWAY: &str = env!("CARGO_BIN_EXE_via-gateway-e2e");

/// How long a Work item is given to travel the whole queue.
///
/// Generous on purpose, and for the same reason `apps/via/tests/milestone.rs`
/// is: the path crosses four owning tasks and one scheduler admission, and here
/// it also crosses a process boundary and a real TCP socket. A tight budget
/// buys nothing but flake on a loaded machine.
pub const BUDGET: Duration = Duration::from_secs(30);

/// How long a file the Gateway writes is given to appear.
///
/// **Derived from the delay it has to outlast**, not chosen: `via-work`
/// coalesces saves on [`via_work::store::DEFERRED_DELAY`], so a test waiting on
/// `tasks.json` is waiting on that timer plus a real `write` + `rename` in
/// another process. A hand-picked number here would silently stop being a
/// margin the day the coalescing delay changed.
pub const FILE_BUDGET: Duration = via_work::store::DEFERRED_DELAY.saturating_mul(80);

/// Poll interval for [`wait_until`].
pub const POLL: Duration = Duration::from_millis(25);

/// The ceiling on one HTTP request and on one WebSocket upgrade.
///
/// Comfortably above [`BUDGET`], because the longest request in the suite —
/// the SSE read — carries [`BUDGET`] of its own and must reach it before this
/// fires. It exists for the case neither budget covers: a Gateway that accepts
/// the connection and then answers nothing at all.
pub const REQUEST_BUDGET: Duration = Duration::from_secs(60);

/// How long a spawned Gateway is given to print its banner.
///
/// The banner is the handshake, and waiting for it is the one place in this
/// suite where a blocking read could outlive the test. The bound is what turns
/// "the Gateway never became ready" from a hung suite into a named failure.
///
/// Deliberately far larger than a Gateway ever needs — it boots in under a
/// second on a warm build — because the *first* execution of a freshly linked
/// binary can stall for tens of seconds while the OS validates it, and a bound
/// that flaked on that would be worse than no bound at all. It is a ceiling on
/// a wedge, not a performance assertion.
pub const START_BUDGET: Duration = Duration::from_secs(120);

/// The `--url` every Gateway in this suite is started on.
///
/// Port 0 asks the kernel for a free one; the bound port is only knowable
/// afterwards, from the banner the process published.
pub const EPHEMERAL_URL: &str = "http://127.0.0.1:0";

/// `SIGINT`, as `kill(2)` numbers it.
pub const SIGINT: i32 = 2;
/// `SIGTERM`.
pub const SIGTERM: i32 = 15;
/// `SIGKILL` — the *unclean* death, which is the one the lease and restart
/// cases are about.
pub const SIGKILL: i32 = 9;

/// The directory holding both binaries — `<target>/<profile>`.
fn profile_directory() -> &'static Path {
    Path::new(HARNESS_GATEWAY)
        .parent()
        .unwrap_or_else(|| Path::new("."))
}

/// The path of the shipped `via` binary, building it if it is not there.
///
/// Built at most once per test binary. `cargo test -p via-e2e` on its own does
/// not build another package's bin targets, so without this the whole suite
/// would depend on someone having run `cargo build -p via` first — a suite that
/// fails for a reason unrelated to the product is not a gate.
///
/// The nested `cargo` call is safe: cargo releases the target-directory lock
/// before it runs tests, which is exactly why `escargot` and `trycmd` work the
/// same way.
pub fn via_binary() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let path = profile_directory().join(VIA);
        if path.exists() {
            return path;
        }
        let cargo = option_env!("CARGO").unwrap_or("cargo");
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("Cargo.toml");
        let target = profile_directory()
            .parent()
            .unwrap_or_else(|| Path::new("target"));
        let mut command = Command::new(cargo);
        command
            .arg("build")
            .arg("--manifest-path")
            .arg(&workspace)
            .arg("--target-dir")
            .arg(target)
            .args(["--package", VIA, "--bin", VIA]);
        if profile_directory()
            .file_name()
            .and_then(|name| name.to_str())
            == Some("release")
        {
            command.arg("--release");
        }
        let built = command.status();
        assert!(
            built.is_ok_and(|status| status.success()) && path.exists(),
            "the shipped binary is not at {} and `cargo build -p {VIA}` did not produce it; \
             run it by hand and re-run the suite",
            path.display(),
        );
        path
    })
}

/// A throwaway machine: a home directory, a configuration directory that does
/// not exist yet, and a working directory.
pub struct Machine {
    root: TempDir,
}

/// One completed run of a binary.
pub struct Run {
    /// The exit code, or `None` when a signal killed it.
    pub code: Option<i32>,
    /// Everything written to stdout.
    pub stdout: String,
    /// Everything written to stderr.
    pub stderr: String,
    /// Every `via.log/v1` record in `logs/cli.log`, oldest first.
    pub records: Vec<Value>,
}

impl Run {
    /// The `code` field of the last `cli.failed` record.
    ///
    /// The catalogued stderr shape is `` `via: <message>` `` with no code in
    /// it, so the machine-readable code reaches a caller through the structured
    /// log rather than through the stream.
    pub fn error_code(&self) -> Option<&str> {
        self.records
            .iter()
            .rev()
            .find(|record| record.get("event").and_then(Value::as_str) == Some("cli.failed"))
            .and_then(|record| record.get("code"))
            .and_then(Value::as_str)
    }
}

impl Machine {
    /// A machine with nothing on it.
    ///
    /// `config` is deliberately **not** created: whether the binary creates it,
    /// and with which mode, is itself a contract.
    pub fn new() -> Self {
        let root = TempDir::new().expect("a temporary directory");
        for name in ["home", "cwd"] {
            std::fs::create_dir_all(root.path().join(name)).expect("scaffold");
        }
        Self { root }
    }

    /// `<root>/home`.
    pub fn home(&self) -> PathBuf {
        self.root.path().join("home")
    }

    /// `<root>/cwd` — the process working directory.
    pub fn cwd(&self) -> PathBuf {
        self.root.path().join("cwd")
    }

    /// `<root>/config` — both `VIA_CONFIG_DIR` and `VIA_DATA_DIR`.
    pub fn config_dir(&self) -> PathBuf {
        self.root.path().join("config")
    }

    /// `<root>/fixtures` — scripts and harness specifications, created lazily.
    pub fn fixtures(&self) -> PathBuf {
        self.root.path().join("fixtures")
    }

    /// `<config>/<name>`, for the catalogued names in [`via_core::paths`].
    pub fn config_path(&self, relative: &str) -> PathBuf {
        let mut path = self.config_dir();
        for segment in relative.split('/').filter(|s| !s.is_empty()) {
            path.push(segment);
        }
        path
    }

    /// `<config>/gateway.lock`.
    pub fn lease_file(&self) -> PathBuf {
        self.config_path(via_core::paths::GATEWAY_LOCK_FILE_NAME)
    }

    /// `<config>/tasks.json`.
    pub fn tasks_file(&self) -> PathBuf {
        self.config_path(via_core::paths::TASK_STATE_FILE_NAME)
    }

    /// `<config>/logs`.
    pub fn log_dir(&self) -> PathBuf {
        self.config_path(via_core::paths::LOG_DIRECTORY_NAME)
    }

    /// `<config>/logs/gateway.log`.
    pub fn gateway_log(&self) -> PathBuf {
        self.log_dir().join(via_core::paths::GATEWAY_LOG_FILE_NAME)
    }

    /// The lease document as `via-lock` wrote it, or `None`.
    pub fn lease(&self) -> Option<via_lock::GatewayLease> {
        let text = std::fs::read_to_string(self.lease_file()).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// The persisted Work records, or an empty list.
    ///
    /// Read as `via-work` writes them — `{"version":1,"tasks":[…]}` — rather
    /// than as an opaque blob, so a test can assert on a field.
    pub fn persisted_tasks(&self) -> Vec<Value> {
        let Ok(text) = std::fs::read_to_string(self.tasks_file()) else {
            return Vec::new();
        };
        let Ok(document) = serde_json::from_str::<Value>(&text) else {
            return Vec::new();
        };
        document
            .get("tasks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }

    /// Where [`Self::write_script`] puts the script the harness Gateway reads.
    ///
    /// Named rather than implicit because one case corrupts the file *after*
    /// the process has started — the opener re-reads it on every open, which is
    /// how a provider that stops answering is reproduced.
    pub fn script_path(&self) -> PathBuf {
        self.fixtures().join("script.json")
    }

    /// Write `script` where the harness Gateway will read it.
    pub fn write_script(&self, script: &Script) -> PathBuf {
        let path = self.script_path();
        std::fs::create_dir_all(self.fixtures()).expect("fixtures");
        std::fs::write(&path, script.to_json_string().expect("a script serializes"))
            .expect("write the script");
        path
    }

    /// Write a harness specification where the harness Gateway will read it.
    pub fn write_harness(&self, spec: &Value) -> PathBuf {
        let path = self.fixtures().join("harness.json");
        std::fs::create_dir_all(self.fixtures()).expect("fixtures");
        std::fs::write(&path, spec.to_string()).expect("write the harness");
        path
    }

    /// A path inside `<root>/fixtures` that does not exist yet.
    ///
    /// The rendezvous a held backend turn waits on — see
    /// `via-gateway-e2e`'s `VIA_E2E_HOLD`.
    pub fn hold_file(&self) -> PathBuf {
        self.fixtures().join("release")
    }

    /// Create [`Self::hold_file`], releasing every held turn.
    pub fn release(&self) {
        std::fs::create_dir_all(self.fixtures()).expect("fixtures");
        std::fs::write(self.hold_file(), b"go").expect("release");
    }

    /// Seed `config.env` with `contents`.
    pub fn seed_config(&self, contents: &str) {
        std::fs::create_dir_all(self.config_dir()).expect("config dir");
        std::fs::write(
            self.config_dir().join(via_core::paths::CONFIG_FILE_NAME),
            contents,
        )
        .expect("seed config.env");
    }

    /// A command for `binary` under this machine's environment.
    ///
    /// `LANG=C` is "no localization", so `resolve_locale` falls through to
    /// `en`, the documented default, unless a case sets `VIA_LOCALE`.
    pub fn command(&self, binary: &Path, extra: &[(&str, &str)], args: &[&str]) -> Command {
        let mut command = Command::new(binary);
        command.env_clear();
        command.current_dir(self.cwd());
        command.env("HOME", self.home());
        command.env("VIA_CONFIG_DIR", self.config_dir());
        command.env("VIA_DATA_DIR", self.config_dir());
        command.env("LANG", "C");
        // An ephemeral port: the catalogued default of 3101 would make parallel
        // tests collide with each other rather than with the thing under test.
        command.env("PORT", "0");
        command.env("HOST", "127.0.0.1");
        for (key, value) in extra {
            command.env(key, value);
        }
        command.args(args);
        command
    }

    /// Run the shipped binary to completion.
    pub fn run_via(&self, extra: &[(&str, &str)], args: &[&str]) -> Run {
        let output = self
            .command(via_binary(), extra, args)
            .output()
            .expect("the binary runs");
        Run {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            records: self.cli_records(),
        }
    }

    /// Start the shipped `via gateway` and wait for its banner.
    ///
    /// `--url` rather than `PORT`: `via gateway` derives `HOST` and `PORT` from
    /// the URL (`cli/src/runtime.mjs:294-316`) and the flag therefore wins, so
    /// a `PORT=0` in the environment alone would be overwritten by the
    /// catalogued default of 3101 — and every parallel test would then collide
    /// with every other rather than with the thing under test.
    pub fn start_via_gateway(&self, extra: &[(&str, &str)]) -> Running<'_> {
        self.start(via_binary(), extra, &["gateway", "--url", EPHEMERAL_URL])
    }

    /// Start the harness Gateway with `script` in front of the model and
    /// `harness` behind the coordinator, and wait for its banner.
    ///
    /// `harness` of `None` is frontend-only mode — no Layer 3 at all.
    pub fn start_harness_gateway(
        &self,
        script: &Script,
        harness: Option<&Value>,
        extra: &[(&str, &str)],
    ) -> Running<'_> {
        let script_path = self.write_script(script);
        let harness_path = harness.map(|spec| self.write_harness(spec));
        let mut environment: Vec<(String, String)> = vec![
            (
                "VIA_E2E_SCRIPT".to_owned(),
                script_path.display().to_string(),
            ),
            // The mock is a fixture, so `via-core`'s setup gate treats it as
            // configured and no credential is needed.
            (
                "VIA_REALTIME_PROVIDER".to_owned(),
                via_realtime_mock::MOCK_PROVIDER_KEY.to_owned(),
            ),
        ];
        if let Some(path) = harness_path {
            environment.push(("VIA_E2E_HARNESS".to_owned(), path.display().to_string()));
        }
        for (key, value) in extra {
            environment.push(((*key).to_owned(), (*value).to_owned()));
        }
        let borrowed: Vec<(&str, &str)> = environment
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        self.start(Path::new(HARNESS_GATEWAY), &borrowed, &[])
    }

    /// Start `binary` and wait for the Gateway banner on stdout.
    ///
    /// The banner is the handshake: `via::gateway::serve` writes it after the
    /// socket is bound and the signal handlers are installed, and flushes — so
    /// a banner line means *up*. It is **found** rather than assumed to be
    /// first, because the Gateway's console log sink shares this stdout, and it
    /// is recognised by its own catalogued prefix rather than by a guess.
    pub fn start(&self, binary: &Path, extra: &[(&str, &str)], args: &[&str]) -> Running<'_> {
        let mut command = self.command(binary, extra, args);
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut child = command.spawn().expect("the binary starts");
        let stdout = child.stdout.take().expect("stdout is piped");

        // **The read is bounded.** A process that starts, serves, and never
        // prints a line this reader recognises would otherwise block here
        // forever — a hung suite instead of a failing test. So the read runs on
        // its own thread; if the banner has not arrived inside
        // [`START_BUDGET`], the child is killed, which gives the reader an EOF
        // and lets the thread finish.
        let (sender, inbox) = std::sync::mpsc::channel();
        let prefixes = banner_prefixes();
        let reading = std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut banner = None;
            let mut seen = String::new();
            let mut line = String::new();
            while reader.read_line(&mut line).unwrap_or(0) > 0 {
                if prefixes.iter().any(|prefix| line.starts_with(prefix)) {
                    banner = Some(std::mem::take(&mut line));
                    break;
                }
                seen.push_str(&line);
                line.clear();
            }
            let _ = sender.send(());
            (banner, seen, Some(reader))
        });
        let timed_out = inbox.recv_timeout(START_BUDGET).is_err();
        if timed_out {
            signal_pid(child.id(), SIGKILL);
        }
        let (banner, seen, reader) =
            reading
                .join()
                .unwrap_or((None, "the reader thread panicked".to_owned(), None));

        // No banner means the process refused and exited, and the reason is on
        // *stderr* — which nothing would ever read, because a refused start is
        // asserted through `is_serving()`. Draining it here is what turns
        // "the Gateway did not start" into a sentence naming why.
        let refusal = if banner.is_none() {
            let mut stderr = String::new();
            if let Some(pipe) = child.stderr.as_mut() {
                let _ = pipe.read_to_string(&mut stderr);
            }
            let timing = if timed_out {
                format!("no banner within {START_BUDGET:?}; ")
            } else {
                String::new()
            };
            format!("{timing}stderr: {stderr}\nstdout: {seen}")
        } else {
            String::new()
        };
        Running {
            machine: self,
            child,
            reader,
            banner,
            refusal,
        }
    }

    /// Every record in `logs/cli.log`.
    pub fn cli_records(&self) -> Vec<Value> {
        records(&self.log_dir().join("cli.log"))
    }

    /// Every record in `logs/gateway.log`.
    pub fn gateway_records(&self) -> Vec<Value> {
        records(&self.gateway_log())
    }
}

/// Every JSON-lines record in `path`, or an empty list.
pub fn records(path: &Path) -> Vec<Value> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// A binary that is still running.
///
/// Dropping one kills it, so a test that fails mid-way does not leave a Gateway
/// holding a lease and a port.
pub struct Running<'machine> {
    machine: &'machine Machine,
    child: Child,
    reader: Option<BufReader<std::process::ChildStdout>>,
    /// The banner line, or `None` when the process refused and exited.
    pub banner: Option<String>,
    /// Whatever the process said instead of a banner.
    pub refusal: String,
}

impl Running<'_> {
    /// Whether the Gateway got as far as listening.
    pub fn is_serving(&self) -> bool {
        self.banner.is_some()
    }

    /// Assert the Gateway is listening, quoting its refusal when it is not.
    #[track_caller]
    pub fn require_serving(&self) {
        assert!(
            self.is_serving(),
            "the Gateway did not reach its banner.\n{}",
            self.refusal,
        );
    }

    /// The child's process id.
    pub fn pid(&self) -> i64 {
        i64::from(self.child.id())
    }

    /// The origin out of the banner — `http://127.0.0.1:<port>`.
    ///
    /// Parsed rather than assumed: `PORT=0` means the port is only knowable
    /// from what the process published.
    pub fn origin(&self) -> String {
        let banner = self.banner.as_deref().unwrap_or_default();
        let start = banner.find("http://").expect("the banner names an origin");
        banner[start..]
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_end_matches(['\n', '\r'])
            .to_owned()
    }

    /// An HTTP client pointed at this Gateway.
    pub fn api(&self) -> Api {
        Api::new(&self.origin())
    }

    /// Send `signal` and collect the finished run.
    pub fn stop_with(mut self, signal: i32) -> Run {
        signal_pid(self.child.id(), signal);
        let status = self.child.wait().expect("the child exits");
        let mut stdout = self.banner.take().unwrap_or_default();
        let mut rest = String::new();
        if let Some(reader) = self.reader.as_mut() {
            let _ = reader.read_to_string(&mut rest);
        }
        stdout.push_str(&rest);
        let mut stderr = String::new();
        if let Some(mut pipe) = self.child.stderr.take() {
            let _ = pipe.read_to_string(&mut stderr);
        }
        Run {
            code: status.code(),
            stdout,
            stderr,
            records: self.machine.cli_records(),
        }
    }

    /// SIGTERM, and wait: the graceful close is what releases the lease.
    pub fn stop(self) -> Run {
        self.stop_with(SIGTERM)
    }

    /// **SIGKILL**, and wait.
    ///
    /// The signal a graceful close cannot intercept, so nothing runs: no
    /// `taskStore.flush()`, no `lease.release()`, no close sequence. This is the
    /// death the stale-lease and restart-recovery cases are about, and
    /// `Child::kill` alone would not say so at the call site.
    pub fn kill(self) -> Run {
        self.stop_with(SIGKILL)
    }
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        signal_pid(self.child.id(), SIGKILL);
        let _ = self.child.wait();
    }
}

/// Send `signal` to `pid`.
pub fn signal_pid(pid: u32, signal: i32) {
    let Ok(raw) = i32::try_from(pid) else {
        return;
    };
    let Ok(signal) = nix::sys::signal::Signal::try_from(signal) else {
        return;
    };
    let _ = nix::sys::signal::kill(nix::unistd::Pid::from_raw(raw), signal);
}

/// The fixed part of `cli.gateway_banner_started`, up to its `{url}`, **in
/// every locale**.
///
/// Read out of the shipped catalog rather than retyped: a test that waited for
/// a hand-written "Gateway started" would keep passing after somebody changed
/// the catalogued sentence, and the whole point of the wait is that the banner
/// is the handshake.
///
/// All three, because the banner is localized and some cases run the Gateway
/// under `VIA_LOCALE=zh` or `ko`. Matching only `en` would leave the reader
/// waiting on a line that is never coming — which is a hung suite rather than a
/// failing test.
pub fn banner_prefixes() -> Vec<String> {
    [
        via_i18n::Locale::En,
        via_i18n::Locale::Zh,
        via_i18n::Locale::Ko,
    ]
    .into_iter()
    .map(|locale| {
        via_i18n::t(locale, via_i18n::keys::CLI_GATEWAY_BANNER_STARTED)
            .split("{url}")
            .next()
            .unwrap_or_default()
            .to_owned()
    })
    .filter(|prefix| !prefix.is_empty())
    .collect()
}

/// Poll `condition` until it holds or `budget` is spent. Answers whether it
/// held.
///
/// Every wait in this suite is bounded, including the ones whose property is
/// only "this eventually happens": an unbounded poll turns a regression into a
/// hung suite rather than a failing test.
pub async fn wait_until(budget: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let deadline = tokio::time::Instant::now() + budget;
    loop {
        if condition() {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(POLL).await;
    }
}

// ── the HTTP surface, as a consumer reaches it ─────────────────────────────

/// A client for one Gateway's HTTP surface.
///
/// `reqwest` rather than `via::chat::GatewayClient`, for one reason: the
/// security cases need to set `Origin` and `Cookie` by hand and to read a raw
/// status, and a typed client exists precisely to stop callers doing that.
pub struct Api {
    client: reqwest::Client,
    origin: String,
}

/// One answered request.
pub struct Answer {
    /// The HTTP status.
    pub status: u16,
    /// The body, parsed as JSON, or `Value::Null` when it is not JSON.
    pub body: Value,
    /// The response headers.
    pub headers: reqwest::header::HeaderMap,
}

impl Answer {
    /// The `X-Request-Id` header, when the response carries one.
    pub fn request_id(&self) -> Option<&str> {
        self.headers
            .get(via_app::http::middleware::REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok())
    }
}

impl Api {
    /// A client for `origin`.
    ///
    /// Redirects are **not** followed: `GET /api/backend/ui` answers 302 and the
    /// status is the contract, so a client that chased it would assert the
    /// wrong thing. And every request is bounded — a Gateway that accepts the
    /// connection and then answers nothing must fail a test rather than hang
    /// the suite, which is the same rule [`START_BUDGET`] applies to a boot.
    pub fn new(origin: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(REQUEST_BUDGET)
                .build()
                .expect("a client"),
            origin: origin.to_owned(),
        }
    }

    /// `<origin><path>`.
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.origin)
    }

    /// A request builder with no headers set.
    pub fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.client.request(method, self.url(path))
    }

    /// Send `request` and read the answer.
    pub async fn send(&self, request: reqwest::RequestBuilder) -> Answer {
        let response = request.send().await.expect("the Gateway answers");
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let text = response.text().await.unwrap_or_default();
        Answer {
            status,
            body: serde_json::from_str(&text).unwrap_or(Value::Null),
            headers,
        }
    }

    /// `GET <path>`.
    pub async fn get(&self, path: &str) -> Answer {
        self.send(self.request(reqwest::Method::GET, path)).await
    }

    /// `DELETE <path>`.
    pub async fn delete(&self, path: &str) -> Answer {
        self.send(self.request(reqwest::Method::DELETE, path)).await
    }

    /// `GET <path>`, reading only the first Server-Sent Event.
    ///
    /// The stream never ends on its own — that is what an event stream is — so
    /// the read is bounded and the connection is dropped afterwards.
    pub async fn first_sse_frame(
        &self,
        path: &str,
        budget: Duration,
    ) -> Option<(u16, String, Value)> {
        let response = self
            .request(reqwest::Method::GET, path)
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .send()
            .await
            .expect("the Gateway answers");
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        if status != 200 {
            let text = response.text().await.unwrap_or_default();
            return Some((
                status,
                content_type,
                serde_json::from_str(&text).unwrap_or(Value::Null),
            ));
        }
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let deadline = tokio::time::Instant::now() + budget;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return None;
            }
            let Ok(Some(Ok(chunk))) = tokio::time::timeout(remaining, stream.next()).await else {
                return None;
            };
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            // `data: {…}\n\n`. Comments (`:` lines) are keep-alive and are not
            // events; a client's `EventSource` never surfaces them either.
            for line in buffer.lines() {
                if let Some(payload) = line.strip_prefix("data: ")
                    && let Ok(frame) = serde_json::from_str::<Value>(payload)
                {
                    return Some((status, content_type, frame));
                }
            }
        }
    }
}

// ── the socket, as `via chat` opens it ─────────────────────────────────────

/// One client socket, and the frames it has seen.
pub struct Socket {
    sink: futures::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::Message,
    >,
    stream: futures::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    /// Everything received, in order.
    pub frames: Vec<Value>,
}

/// Build the upgrade request `via chat` builds, optionally with a cookie.
///
/// The URL comes from `via::chat::protocol::websocket_url`, so the path and the
/// query are the client's own rather than a re-derivation of them.
pub fn upgrade_request(
    origin: &str,
    session_id: &str,
    cookie: Option<&str>,
) -> tokio_tungstenite::tungstenite::handshake::client::Request {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
    let url = via::chat::protocol::websocket_url(origin, session_id).expect("a socket url");
    let mut request = url.as_str().into_client_request().expect("a request");
    if let Some(cookie) = cookie {
        request
            .headers_mut()
            .insert("cookie", cookie.parse().expect("a header value"));
    }
    request
}

impl Socket {
    /// Open a socket, exactly as `via chat` does.
    pub async fn connect(origin: &str, session_id: &str) -> Self {
        Self::try_connect(origin, session_id, None)
            .await
            .expect("the upgrade is accepted")
    }

    /// Open a socket, keeping the refusal when there is one.
    ///
    /// Bounded by [`REQUEST_BUDGET`]: a Gateway that accepts the TCP connection
    /// and never completes the handshake is a wedge, and a wedge has to be a
    /// failing test rather than a hung suite. A timeout is reported as the
    /// transport error it is.
    pub async fn try_connect(
        origin: &str,
        session_id: &str,
        cookie: Option<&str>,
    ) -> Result<Self, tokio_tungstenite::tungstenite::Error> {
        let request = upgrade_request(origin, session_id, cookie);
        let opened =
            tokio::time::timeout(REQUEST_BUDGET, tokio_tungstenite::connect_async(request))
                .await
                .map_err(|_| {
                    tokio_tungstenite::tungstenite::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        format!("the upgrade did not complete within {REQUEST_BUDGET:?}"),
                    ))
                })?;
        let (socket, _response) = opened?;
        let (sink, stream) = socket.split();
        Ok(Self {
            sink,
            stream,
            frames: Vec::new(),
        })
    }

    /// Send one frame.
    pub async fn send(&mut self, frame: Value) {
        self.sink
            .send(tokio_tungstenite::tungstenite::Message::Text(
                frame.to_string().into(),
            ))
            .await
            .expect("the socket accepts a frame");
    }

    /// Send the `connect` frame `via chat` sends.
    pub async fn hello(&mut self, locale: via_i18n::Locale) {
        self.send(via::chat::protocol::connect_frame(false, "UTC", locale))
            .await;
    }

    /// Type a line, exactly as `via chat` does.
    pub async fn say(&mut self, text: &str) {
        self.send(serde_json::json!({
            "type": "input.message",
            "text": text,
            "textOnly": true,
        }))
        .await;
    }

    /// Read frames until `predicate` matches one, or the budget is spent.
    pub async fn wait_for(
        &mut self,
        budget: Duration,
        predicate: impl Fn(&Value) -> bool,
    ) -> Option<Value> {
        let deadline = tokio::time::Instant::now() + budget;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return None;
            }
            let Ok(Some(Ok(message))) = tokio::time::timeout(remaining, self.stream.next()).await
            else {
                return None;
            };
            let tokio_tungstenite::tungstenite::Message::Text(text) = message else {
                continue;
            };
            let Ok(frame) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            self.frames.push(frame.clone());
            if predicate(&frame) {
                return Some(frame);
            }
        }
    }

    /// Every frame of `kind` seen so far.
    pub fn seen(&self, kind: &str) -> Vec<&Value> {
        self.frames
            .iter()
            .filter(|frame| frame["type"] == kind)
            .collect()
    }

    /// The index of the first frame of `kind`, if any.
    pub fn position_of(&self, kind: &str) -> Option<usize> {
        self.frames.iter().position(|frame| frame["type"] == kind)
    }

    /// The index of the first frame matching `predicate`, if any.
    ///
    /// Ordering assertions need this rather than [`Self::position_of`]: one
    /// conversation produces several `transcript.final` frames, and *which* one
    /// came after the completion is the whole question.
    pub fn position_where(&self, predicate: impl Fn(&Value) -> bool) -> Option<usize> {
        self.frames.iter().position(predicate)
    }
}

/// A frame matcher for one `type`.
pub fn is(kind: &'static str) -> impl Fn(&Value) -> bool {
    move |frame| frame["type"] == kind
}

/// The catalogued value for `kind` / `name`, with the product rename applied.
///
/// `docs/rebrand.md` renames upstream's product name and nothing else inside a
/// sentence, which is the rule `via-work/tests/contracts.rs` applies to the
/// same rows. Applying it *to the catalogued value* — rather than retyping the
/// value with the rename already done — is what makes a partial rename fail.
pub fn catalogued(kind: &str, name: &str) -> String {
    via_conformance::catalogue::expect_contract(kind, name)
        .exact_value
        .replace("qwen-audio-agent", "VIA")
}

/// The catalogued value for `kind` / `name`, verbatim.
pub fn catalogued_raw(kind: &str, name: &str) -> &'static str {
    &via_conformance::catalogue::expect_contract(kind, name).exact_value
}

/// The Unix permission bits of `path`, or `None` off Unix / for a missing path.
#[cfg(unix)]
pub fn mode_of(path: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt as _;
    Some(std::fs::metadata(path).ok()?.permissions().mode() & 0o777)
}

#[cfg(not(unix))]
pub fn mode_of(_path: &Path) -> Option<u32> {
    None
}

/// Every path under `root`, relative to it, `/`-separated, directories
/// included.
pub fn tree(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    walk(root, root, &mut found);
    found.sort();
    found
}

fn walk(root: &Path, directory: &Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(relative) = path.strip_prefix(root) {
            let mut name = relative.display().to_string().replace('\\', "/");
            if path.is_dir() {
                name.push('/');
                found.push(name);
                walk(root, &path, found);
            } else {
                found.push(name);
            }
        }
    }
}
