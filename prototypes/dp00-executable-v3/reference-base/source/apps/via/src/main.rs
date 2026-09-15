//! The `via` binary's process boundary.
//!
//! Everything with behaviour lives in the library beside this file; `main` is
//! the four things that can only happen at the edge of a process — sampling
//! the real environment, writing to the real streams, choosing an exit code,
//! and logging the run.
//!
//! Ported from `cli/bin/<binary>.mjs`, whose whole job is the same:
//!
//! ```js
//! main(process.argv.slice(2)).then(code => { … }).catch(error => {
//!   process.stderr.write(`<binary>: ${error.message}\n`)
//!   process.exitCode = 1
//! })
//! ```

#![forbid(unsafe_code)]

use std::io::Write;
use std::process::ExitCode;
use std::time::Instant;

use via::cli::{Cli, parse_from};
use via::error::{EXIT_FAILURE, EXIT_SUCCESS};
use via::{BINARY_NAME, CliError, Host, LOG_COMPONENT, LOG_EVENTS, STDERR_PREFIX};
use via_i18n::Locale;
use via_log::{Logger, LoggerOptions, fields, is_test_process};

fn main() -> ExitCode {
    let started = Instant::now();
    let host = Host::from_process();
    let logger = build_logger(&host);

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // Upstream logs `process.argv[2] || 'gateway'` — the verb as typed, before
    // it is validated, so a run that fails to parse is still attributable.
    let verb = arguments
        .first()
        .filter(|first| !first.starts_with('-'))
        .cloned()
        .unwrap_or_else(|| via::cli::DEFAULT_COMMAND.to_owned());
    logger.info(
        LOG_EVENTS[0],
        fields([("command", verb.clone().into())]),
        "cli started",
    );

    let code = run(&arguments, host, &logger, &verb, started);
    logger.flush();
    ExitCode::from(code)
}

/// Parse, dispatch, and turn the outcome into an exit code.
fn run(arguments: &[String], mut host: Host, logger: &Logger, verb: &str, started: Instant) -> u8 {
    let cli: Cli = match parse_from(arguments.iter().cloned()) {
        Ok(cli) => cli,
        Err(error) => return report_usage(error, logger, verb, started),
    };

    let locale = host.locale();
    // `stdout()`, **not** `stdout().lock()`. Upstream's CLI is the Gateway's
    // parent process, so the two write to different streams; here they are one
    // process, and the Gateway's console log sink writes to this same stdout
    // from a worker thread. Holding the process-wide lock across `dispatch`
    // deadlocks `via gateway` the instant it logs `gateway.ready`.
    //
    // Nothing is lost by not holding it: `Stdout`'s `write_fmt` takes the lock
    // once for the whole format, so a `write!` of a multi-part message is still
    // written atomically, and the console sink writes one record per call.
    let mut stdout = std::io::stdout();
    let outcome = via::commands::dispatch(&cli, &mut host, &mut stdout);
    // Flush before the exit code is chosen: a `via config` answer that never
    // reached the pipe is a failure however the process exits.
    let flushed = stdout.flush();

    match outcome.and_then(|()| flushed.map_err(|error| CliError::io("flush", "<stdout>", error))) {
        Ok(()) => {
            complete(logger, verb, started, EXIT_SUCCESS);
            EXIT_SUCCESS
        }
        // `host.locale()` rather than the locale sampled above: `config.env`
        // may have set `VIA_LOCALE`, and the refusal should be in the language
        // the machine is configured for.
        Err(error) => report(&error, logger, verb, started, host.locale(), locale),
    }
}

/// Write a refusal to stderr and return the failure code.
///
/// **External contract** — *error-code/stderr prefix*, `cli/bin/<binary>.mjs:29`:
/// `` `<binary>: ${error.message}\n` ``.
fn report(
    error: &CliError,
    logger: &Logger,
    verb: &str,
    started: Instant,
    locale: Locale,
    fallback: Locale,
) -> u8 {
    let message = error.message(locale);
    let message = if message.is_empty() {
        error.message(fallback)
    } else {
        message
    };
    let mut stderr = std::io::stderr().lock();
    // Nothing useful remains if stderr itself is gone, and the exit code still
    // carries the refusal.
    let _ = writeln!(stderr, "{STDERR_PREFIX}{message}");
    let _ = stderr.flush();
    logger.error(
        LOG_EVENTS[2],
        fields([
            ("command", verb.into()),
            ("durationMs", elapsed(started).into()),
            ("code", error.code().into()),
        ]),
        "cli failed",
    );
    EXIT_FAILURE
}

/// Print `clap`'s own diagnostic and map it onto the catalogued exit codes.
///
/// `--help` and `--version` are `clap` "errors" with exit code 0 and are
/// written to stdout; a usage failure goes to stderr. The exit code is 1
/// rather than `clap`'s default 2, because *exit-code/process exit codes* says
/// every thrown error is 1 and a script must not have to know which kind of
/// mistake it made to read the number.
fn report_usage(error: clap::Error, logger: &Logger, verb: &str, started: Instant) -> u8 {
    let _ = error.print();
    let code = if error.exit_code() == 0 {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    };
    if code == EXIT_SUCCESS {
        complete(logger, verb, started, code);
    } else {
        logger.error(
            LOG_EVENTS[2],
            fields([
                ("command", verb.into()),
                ("durationMs", elapsed(started).into()),
                ("code", "VIA_CLI_USAGE".into()),
            ]),
            "cli failed",
        );
    }
    code
}

/// Log the successful completion record.
fn complete(logger: &Logger, verb: &str, started: Instant, code: u8) {
    logger.info(
        LOG_EVENTS[1],
        fields([
            ("command", verb.into()),
            ("exitCode", u64::from(code).into()),
            ("durationMs", elapsed(started).into()),
        ]),
        "cli completed",
    );
}

/// Milliseconds since `started`, as a JSON number.
fn elapsed(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// The `cli` logger.
///
/// **External contract** — `cli/bin/<binary>.mjs:6-10`: component `cli`, file
/// `cli.log`, console disabled. The console sink is off because stdout is the
/// command's answer, and a log line interleaved with `via config`'s path would
/// corrupt whatever is reading it.
fn build_logger(host: &Host) -> Logger {
    let mut options = LoggerOptions::new(LOG_COMPONENT, host.env(), host.home_directory());
    options.console_enabled = false;
    if is_test_process(host.env(), std::env::args()) {
        // Upstream disables both sinks for a test process
        // (`server/src/core/logger.mjs:8-9`); a test run must not write into
        // the developer's real log directory.
        options.file_enabled = false;
    }
    debug_assert_eq!(options.file_name, std::format!("{LOG_COMPONENT}.log"));
    debug_assert_eq!(BINARY_NAME, "via");
    Logger::new(options)
}
