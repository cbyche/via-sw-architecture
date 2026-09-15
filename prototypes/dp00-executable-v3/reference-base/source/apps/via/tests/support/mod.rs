//! Running the real binary against a synthetic machine.
//!
//! Every integration test here drives `CARGO_BIN_EXE_via` rather than calling
//! into the library, because three of the four things `main` is responsible
//! for — the exit code, the stream a message lands on, and the `cli.log`
//! record — are only observable from outside the process.
//!
//! The child's environment is **cleared**, not inherited. `std::env::set_var`
//! is `unsafe` in edition 2024 and the parent's environment is shared by every
//! test in the binary, so an inherited environment would make these tests both
//! order-dependent and dependent on whatever the developer happens to export.
//! `env_clear()` plus an explicit allow-list is the only way a test can say
//! "this machine has no `DASHSCOPE_API_KEY`" and mean it.

#![allow(dead_code)]

pub mod gateway;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use tempfile::TempDir;

/// A throwaway machine: a home directory, a configuration directory, a data
/// directory and a log directory, none of which exist until the binary makes
/// them.
pub struct Fixture {
    root: TempDir,
}

/// One completed run of the binary.
pub struct Run {
    /// The process exit code, or `None` if it was killed by a signal.
    pub code: Option<i32>,
    /// Everything written to stdout.
    pub stdout: String,
    /// Everything written to stderr.
    pub stderr: String,
    /// Every `via.log/v1` record in `cli.log`, oldest first.
    pub records: Vec<serde_json::Value>,
}

impl Run {
    /// The `code` field of the last `cli.failed` record.
    ///
    /// This is how a machine-readable error code reaches a caller of the
    /// binary. The catalogued stderr shape is `` `via: <message>` `` with no
    /// code in it, so printing one would break the contract; the structured
    /// log is where it belongs, and `cli.failed` carries it.
    pub fn error_code(&self) -> Option<&str> {
        self.records
            .iter()
            .rev()
            .find(|record| record.get("event").and_then(|v| v.as_str()) == Some("cli.failed"))
            .and_then(|record| record.get("code"))
            .and_then(|value| value.as_str())
    }

    /// The events logged, in order.
    pub fn events(&self) -> Vec<&str> {
        self.records
            .iter()
            .filter_map(|record| record.get("event").and_then(|v| v.as_str()))
            .collect()
    }

    /// stdout and stderr together, which is what upstream's own
    /// `consumer-install.test.mjs` matches against.
    pub fn output(&self) -> String {
        std::format!("{}{}", self.stdout, self.stderr)
    }
}

impl Fixture {
    /// A machine with nothing on it.
    pub fn new() -> Self {
        let root = TempDir::new().expect("a temporary directory");
        // `config` is deliberately **not** created: whether the binary creates
        // it, and with which mode, is itself a contract.
        for name in ["home", "logs", "cwd"] {
            std::fs::create_dir_all(root.path().join(name)).expect("scaffold");
        }
        Self { root }
    }

    /// `<root>/home`.
    pub fn home(&self) -> PathBuf {
        self.root.path().join("home")
    }

    /// `<root>/config` — both `VIA_CONFIG_DIR` and `VIA_DATA_DIR`.
    pub fn config_dir(&self) -> PathBuf {
        self.root.path().join("config")
    }

    /// `<root>/logs`.
    pub fn log_dir(&self) -> PathBuf {
        self.root.path().join("logs")
    }

    /// The working directory the binary runs in.
    pub fn cwd(&self) -> PathBuf {
        self.root.path().join("cwd")
    }

    /// `<config>/config.env`.
    pub fn config_file(&self) -> PathBuf {
        self.config_dir().join("config.env")
    }

    /// `<config>/gateway.lock`.
    pub fn lease_file(&self) -> PathBuf {
        self.config_dir().join(via_lock::GATEWAY_LOCK_FILE_NAME)
    }

    /// Write `contents` to `config.env`, creating the directory if needed.
    pub fn seed_config(&self, contents: &str) {
        std::fs::create_dir_all(self.config_dir()).expect("config dir");
        std::fs::write(self.config_file(), contents).expect("seed config.env");
    }

    /// Run the binary with `args` and `extra` on top of the baseline
    /// environment.
    ///
    /// The child's environment is a `C` locale — "no localization", so
    /// `resolve_locale` skips it and falls through to `en`, the documented
    /// default — unless a case sets `VIA_LOCALE`.
    pub fn run(&self, extra: &[(&str, &str)], args: &[&str]) -> Run {
        let output = self.command(extra, args).output().expect("the binary runs");
        Run {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            records: self.records(),
        }
    }

    /// Start the binary and wait for its first line of stdout.
    ///
    /// `via gateway` blocks until a signal, so a test that wants to observe a
    /// *running* Gateway cannot use [`Self::run`]. The banner is the handshake:
    /// `gateway::serve` writes it after the socket is bound and the signal
    /// handlers are installed, and flushes, so a line on stdout means "up".
    ///
    /// An ephemeral port unless the caller says otherwise — tests run in
    /// parallel, and the catalogued default of 3101 would make them collide
    /// with each other rather than with the thing under test. It is passed as
    /// `--url` rather than as `PORT`, because `via gateway` derives `HOST` and
    /// `PORT` from `--url` (`cli/src/runtime.mjs:294-316`) and the flag
    /// therefore wins.
    pub fn start(&self, extra: &[(&str, &str)], args: &[&str]) -> Running<'_> {
        let ephemeral: Vec<&str> = if args.first() == Some(&"gateway") && !args.contains(&"--url") {
            let mut with_url = args.to_vec();
            with_url.extend_from_slice(&["--url", "http://127.0.0.1:0"]);
            with_url
        } else {
            args.to_vec()
        };
        let mut command = self.command(extra, &ephemeral);
        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut child = command.spawn().expect("the binary starts");
        let stdout = child.stdout.take().expect("stdout is piped");
        let mut reader = BufReader::new(stdout);
        // The Gateway's own console log shares stdout with the CLI's banner —
        // one process, two writers — so the banner is *found*, not assumed to
        // be first, and it is recognised by its own catalogued prefix rather
        // than by a guess about what a log line looks like.
        let prefix = banner_prefix();
        let mut banner = None;
        let mut line = String::new();
        while reader.read_line(&mut line).unwrap_or(0) > 0 {
            if line.starts_with(&prefix) {
                banner = Some(std::mem::take(&mut line));
                break;
            }
            line.clear();
        }
        // `gateway::serve` writes the banner and the summary in one `write!`,
        // so the summary is the very next line and cannot have a log record
        // wedged between them — `Stdout::write_fmt` takes the lock once.
        let mut summary = String::new();
        if banner.is_some() {
            let _ = reader.read_line(&mut summary);
        }
        Running {
            fixture: self,
            child,
            reader,
            banner,
            summary,
        }
    }

    fn command(&self, extra: &[(&str, &str)], args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_via"));
        command.env_clear();
        command.current_dir(self.cwd());
        command.env("HOME", self.home());
        command.env("VIA_CONFIG_DIR", self.config_dir());
        command.env("VIA_DATA_DIR", self.config_dir());
        command.env("VIA_LOG_DIR", self.log_dir());
        command.env("LANG", "C");
        command.env("PORT", "0");
        for (key, value) in extra {
            command.env(key, value);
        }
        command.args(args);
        command
    }

    /// Run the binary with `args`, feeding `input` to its stdin.
    ///
    /// `via chat` is a REPL, so a test that wants to exercise its command
    /// surface has to *type* at it. Closing stdin after `input` is what a
    /// heredoc does, and the client treats the resulting EOF the same way it
    /// treats `/exit`.
    pub fn run_with_input(&self, extra: &[(&str, &str)], args: &[&str], input: &str) -> Run {
        use std::io::Write as _;
        let mut command = self.command(extra, args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        let mut child = command.spawn().expect("the binary starts");
        if let Some(mut pipe) = child.stdin.take() {
            let _ = pipe.write_all(input.as_bytes());
            let _ = pipe.flush();
            // Dropping the handle closes the pipe, which is the EOF the REPL
            // reads as the end of the session.
        }
        let output = child.wait_with_output().expect("the binary exits");
        Run {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            records: self.records(),
        }
    }

    /// Every record in `cli.log`.
    pub fn records(&self) -> Vec<serde_json::Value> {
        let path = self.log_dir().join("cli.log");
        let Ok(text) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("every log line is one JSON object"))
            .collect()
    }
}

/// One row of `docs/reference/contracts.json`.
pub struct Contract {
    /// The contract's kind, e.g. `error-code`.
    pub kind: String,
    /// The contract's name.
    pub name: String,
    /// The upstream file and line range it was surveyed from.
    pub file: String,
    /// The catalogued value — sometimes a literal, sometimes prose about one.
    pub exact_value: String,
    /// Why the survey recorded it. Some values are only stated here.
    pub why: String,
}

/// Every row of `docs/reference/contracts.json`.
///
/// Read at test time rather than vendored, so a contract asserted here is the
/// same string the catalogue holds. Retyping one would assert only that two
/// copies of a value agree with each other.
pub fn contracts() -> Vec<Contract> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("reference")
        .join("contracts.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(&text).expect("the catalogue is a JSON array");
    rows.into_iter()
        .map(|row| Contract {
            kind: field(&row, "kind"),
            name: field(&row, "name"),
            file: field(&row, "file"),
            exact_value: field(&row, "exactValue"),
            why: field(&row, "why"),
        })
        .collect()
}

fn field(row: &serde_json::Value, key: &str) -> String {
    row.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// The one contract with this kind and name.
pub fn contract(kind: &str, name: &str) -> Contract {
    let mut found: Vec<Contract> = contracts()
        .into_iter()
        .filter(|row| row.kind == kind && row.name == name)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "expected exactly one `{kind}` contract named `{name}`, found {}",
        found.len()
    );
    found.remove(0)
}

/// The one contract with this kind, surveyed from this file and line range.
///
/// Some catalogue rows share a name — the survey recorded the same error code
/// from two call sites — so the upstream location is the discriminator.
pub fn contract_in(kind: &str, file: &str) -> Contract {
    let mut found: Vec<Contract> = contracts()
        .into_iter()
        .filter(|row| row.kind == kind && row.file == file)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "expected exactly one `{kind}` contract from `{file}`, found {}",
        found.len()
    );
    found.remove(0)
}

/// The single-quoted run that follows `marker` in `haystack`.
///
/// Catalogue values are prose *about* a literal as often as they are the
/// literal, and the literal inside them is quoted. This pulls it out so the
/// assertion compares the catalogued string rather than a paraphrase of it.
pub fn quoted_after(haystack: &str, marker: &str) -> Option<String> {
    let rest = &haystack[haystack.find(marker)? + marker.len()..];
    let open = rest.find('\'')? + 1;
    let close = rest[open..].find('\'')? + open;
    Some(rest[open..close].to_owned())
}

/// Every backtick-quoted run in `haystack`, in order.
pub fn backticked(haystack: &str) -> Vec<String> {
    haystack
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

/// The comma-separated list that follows `marker`, trimmed.
pub fn list_after(haystack: &str, marker: &str) -> Vec<String> {
    let Some(start) = haystack.find(marker) else {
        return Vec::new();
    };
    haystack[start + marker.len()..]
        .split([';', '('])
        .next()
        .unwrap_or_default()
        .split(',')
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

/// The identity substitutions `docs/rebrand.md` mandates, longest prefix
/// first.
///
/// Applying the rule to a catalogued value — rather than retyping the value
/// with the rename already done — is what makes a *partial* rename fail: a
/// build that renamed the sentence but not the binary name inside it would
/// produce a different string here.
const IDENTITY: [(&str, &str); 5] = [
    ("QWEN_AUDIO_AGENT_", "VIA_"), // upstream env prefix, docs/rebrand.md row 93
    ("QWEN_AUDIO_", "VIA_"),       // upstream env prefix, docs/rebrand.md row 91
    ("QWAUDIO_", "VIA_"),          // upstream code prefix, docs/rebrand.md row 64
    ("qwaudio.", "via."),          // upstream schema prefix, docs/rebrand.md row 66
    ("qwenaudio", "via"),          // upstream binary name, docs/rebrand.md row 136
];

/// Apply [`IDENTITY`] to a catalogued value.
pub fn rebranded(value: &str) -> String {
    let mut text = value.to_owned();
    for (from, to) in IDENTITY {
        text = text.replace(from, to);
    }
    text
}

/// Write a lease naming `pid` into `path`.
///
/// Built through `via-lock`'s own `GatewayLease` rather than a hand-written
/// JSON literal, so the document a test presents to the binary is the one
/// `via-lock` writes — field names, field order and schema string included.
pub fn write_lease(path: &Path, pid: i64, origin: &str) -> via_lock::GatewayLease {
    let lease = via_lock::GatewayLease {
        schema: via_lock::GATEWAY_LOCK_SCHEMA.to_owned(),
        instance_id: "11111111-2222-3333-4444-555555555555".to_owned(),
        pid,
        owner: "cli".to_owned(),
        state: via_lock::LEASE_STATE_READY.to_owned(),
        origin: origin.to_owned(),
        started_at: "2026-08-22T00:00:00.000Z".to_owned(),
        heartbeat_at: "2026-08-22T00:00:00.000Z".to_owned(),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("config dir");
    }
    let body = serde_json::to_string(&lease).expect("a lease serializes");
    std::fs::write(path, std::format!("{body}\n")).expect("write the lease");
    lease
}

/// A binary that is still running.
///
/// Dropping one terminates it, so a test that fails mid-way does not leave a
/// Gateway holding a lease and a port.
pub struct Running<'fixture> {
    fixture: &'fixture Fixture,
    child: Child,
    reader: BufReader<std::process::ChildStdout>,
    /// The banner line, or `None` when the process refused and exited.
    pub banner: Option<String>,
    /// The line after the banner — `gatewaySummary(health)`.
    pub summary: String,
}

impl Running<'_> {
    /// Whether the Gateway got as far as listening.
    pub fn is_serving(&self) -> bool {
        self.banner.is_some()
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

    /// Send `signal` and collect the finished run.
    ///
    /// SIGTERM rather than `Child::kill`'s SIGKILL: the graceful close is the
    /// thing under test, and a killed process would leave the lease on disk
    /// and prove nothing.
    pub fn stop_with(mut self, signal: i32) -> Run {
        terminate(&self.child, signal);
        let status = self.child.wait().expect("the child exits");
        let mut stdout = self.banner.take().unwrap_or_default();
        stdout.push_str(&std::mem::take(&mut self.summary));
        let mut rest = String::new();
        use std::io::Read as _;
        let _ = self.reader.read_to_string(&mut rest);
        stdout.push_str(&rest);
        let mut stderr = String::new();
        if let Some(mut pipe) = self.child.stderr.take() {
            let _ = pipe.read_to_string(&mut stderr);
        }
        Run {
            code: status.code(),
            stdout,
            stderr,
            records: self.fixture.records(),
        }
    }

    /// Send SIGTERM and collect the finished run.
    pub fn stop(self) -> Run {
        self.stop_with(SIGTERM)
    }
}

impl Drop for Running<'_> {
    fn drop(&mut self) {
        terminate(&self.child, SIGKILL);
        let _ = self.child.wait();
    }
}

/// The fixed part of `cli.gateway_banner_started`, up to its `{url}`.
///
/// Read out of the shipped catalogue rather than retyped: a test that waited
/// for a hand-written "Gateway started" would keep passing after somebody
/// changed the catalogued sentence, and the whole point of the wait is that the
/// banner is the handshake.
pub fn banner_prefix() -> String {
    via_i18n::t(
        via_i18n::Locale::En,
        via_i18n::keys::CLI_GATEWAY_BANNER_STARTED,
    )
    .split("{url}")
    .next()
    .unwrap_or_default()
    .to_owned()
}

/// `SIGINT`, as `kill(2)` numbers it.
pub const SIGINT: i32 = 2;
/// `SIGTERM`, as `kill(2)` numbers it.
pub const SIGTERM: i32 = 15;
/// `SIGKILL`, the drop path's last resort.
pub const SIGKILL: i32 = 9;

#[cfg(unix)]
fn terminate(child: &Child, signal: i32) {
    // `libc::kill` through `std::process::Child::id`, without adding a
    // dependency for three constants. `Command::kill` sends SIGKILL only, and
    // SIGKILL is precisely the signal that proves nothing about a graceful
    // shutdown.
    let pid = child.id();
    let output = Command::new("kill")
        .arg(std::format!("-{signal}"))
        .arg(pid.to_string())
        .output();
    debug_assert!(output.is_ok(), "kill -{signal} {pid}");
}

#[cfg(not(unix))]
fn terminate(child: &Child, _signal: i32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .output();
}
