//! Reading and rewriting `config.env` without disturbing it.
//!
//! Ported from `cli/src/config-command.mjs:16-96`. `config.env` is a file the
//! user hand-edits: it ships with a commented template, it carries credentials
//! for backends VIA does not know about, and a `via config set` that rewrote it
//! as "the keys I understand, serialized" would silently delete all of that.
//!
//! **The catalogued contract** — `docs/reference/contracts.json`
//! *file-path/`config.env` key written by `config set`* — spells out exactly
//! what survives:
//!
//! > `VIA_REALTIME_MODEL=<id>` (single line, existing duplicates collapsed,
//! > comments/unknown keys preserved, CRLF preserved if the file already used
//! > it, file mode 0600, directory mode 0700)
//!
//! So the rewrite is line-oriented: every line that is not an assignment to the
//! key being written is copied through byte for byte, the *first* assignment to
//! that key is replaced in place, any later duplicates are dropped, and a file
//! that had no such assignment gains one at the end.
//!
//! # Two behaviours reproduced rather than corrected
//!
//! **The reader is not the env-file parser.** `resolveConfigModel`
//! (`config-command.mjs:23-26`) matches `^\s*KEY\s*=\s*(.*?)\s*$` against the
//! raw text, so `KEY="quoted"` reads back as `"quoted"` *with* the quotes,
//! `export KEY=x` does not match at all, and a `# KEY=x` comment is correctly
//! ignored. `via-core`'s `parse_env_map` would unquote and would accept
//! `export`. Both are used, for different jobs: this reader answers "what does
//! the *file* say", which is what `config show` reports and what `config set`
//! rewrites, and it must therefore see the file the same way the rewriter
//! does.
//!
//! **`||` is not `??`.** An environment variable set to the empty string falls
//! through to the file, and one set to whitespace does not — it wins, and is
//! then trimmed to nothing. Reproduced, and pinned by a test, because a
//! resolver that "fixed" it would disagree with the Gateway about which model
//! is configured.

use std::path::Path;

use via_core::EnvMap;
use via_core::runtime::{IfExists, write_file};
use via_i18n::{Locale, keys};
use via_store::{LockError, LockOptions, with_file_transaction};

use crate::error::CliError;

/// The one key `via config set` writes.
///
/// **External contract** — `cli/src/config-command.mjs:52`, renamed by
/// `docs/rebrand.md` row 128. Taken from `via-core` rather than retyped, so
/// the CLI and the resolver cannot disagree about the spelling.
pub const REALTIME_MODEL_KEY: &str = via_core::config::names::REALTIME_MODEL;

/// Whether `line` assigns `key`.
///
/// **External contract** — `cli/src/config-command.mjs:60`,
/// `/^\s*(KEY)\s*=.*$/`. Leading whitespace is allowed; `export ` is not; a
/// comment marker is not whitespace, so `# KEY=x` is not an assignment; and
/// `KEY_SUFFIX=x` is not one either, because `=` must follow the key with only
/// whitespace between.
#[must_use]
pub fn is_assignment(line: &str, key: &str) -> bool {
    let Some(rest) = line.trim_start().strip_prefix(key) else {
        return false;
    };
    rest.trim_start().starts_with('=')
}

/// The value of the first assignment to `key`, or `None`.
///
/// **External contract** — `cli/src/config-command.mjs:24`,
/// `/^\s*KEY\s*=\s*(.*?)\s*$/m`. The lazy group plus the trailing `\s*`
/// trims the value; nothing else about it is interpreted, so quotes and `#`
/// survive verbatim.
#[must_use]
pub fn first_assignment<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    contents.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix(key)?;
        let rest = rest.trim_start().strip_prefix('=')?;
        Some(rest.trim())
    })
}

/// Split `contents` the way JavaScript's `split(/\r?\n/)` splits it.
///
/// A lone `\r` is **not** a separator — only `\n`, optionally preceded by one
/// `\r`. Getting this wrong turns a classic-Mac line ending into a spurious
/// split and silently reflows a user's file.
fn split_lines(contents: &str) -> Vec<&str> {
    if contents.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&str> = contents
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    // `'a\n'.split(/\r?\n/)` is `['a', '']`; upstream pops that trailing empty
    // so the terminator is re-added exactly once at the end.
    if lines.last() == Some(&"") {
        lines.pop();
    }
    lines
}

/// Rewrite `contents` so that `key` is assigned `value` exactly once.
///
/// **External contract** — `cli/src/config-command.mjs:47-72`. Every property
/// the catalogue lists is a property of this function:
///
/// * the first assignment to `key` is replaced **in place**, so its position
///   in the file — and the comment above it — is kept;
/// * later duplicates are dropped, collapsing to one line;
/// * every other line is copied through unchanged, comments included;
/// * `\r\n` is preserved when the file already used it anywhere;
/// * the result always ends with exactly one line terminator.
///
/// The replacement line is `KEY=value` with no spaces around `=`, whatever
/// spacing the line it replaces used. That is upstream's, and it is what makes
/// the output stable under repeated writes.
#[must_use]
pub fn rewrite_assignment(contents: &str, key: &str, value: &str) -> String {
    let newline = if contents.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let assignment = std::format!("{key}={value}");
    let mut replaced = false;
    let mut out: Vec<&str> = Vec::new();
    for line in split_lines(contents) {
        if is_assignment(line, key) {
            if !replaced {
                out.push(&assignment);
                replaced = true;
            }
            continue;
        }
        out.push(line);
    }
    if !replaced {
        out.push(&assignment);
    }
    let mut text = out.join(newline);
    text.push_str(newline);
    text
}

/// The Realtime model in force: environment, then file, then the catalog
/// default.
///
/// **External contract** — `cli/src/config-command.mjs:23-26`. `||`, not `??`:
/// an empty environment value falls through to the file. The trim is applied
/// once, at the end, to whichever source won.
#[must_use]
pub fn resolve_config_model(env: &EnvMap, contents: &str) -> String {
    let from_env = env.get(REALTIME_MODEL_KEY).filter(|v| !v.is_empty());
    let from_file = first_assignment(contents, REALTIME_MODEL_KEY).filter(|v| !v.is_empty());
    from_env
        .or(from_file)
        .unwrap_or(via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL)
        .trim()
        .to_owned()
}

/// Read `config.env`, treating "not there" as empty.
///
/// **External contract** — `cli/src/config-command.mjs:16-21`: `ENOENT` is the
/// empty string, every other error propagates. A machine that has never been
/// configured is not a machine with a broken configuration.
///
/// # Errors
///
/// [`CliError::Io`] for a read failure that is not "not found".
pub fn read_config_file(path: &Path) -> Result<String, CliError> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(CliError::io("read", path, error)),
    }
}

/// Refuse a Realtime model id the catalog does not have.
///
/// **External contract** — `cli/src/config-command.mjs:28-34`, which throws
/// `不支持的 Realtime 模型：<id>` for anything outside the four catalog ids.
/// The lookup is `via-catalog`'s, which — per `docs/architecture.md` §7 —
/// already refuses an unknown id rather than returning upstream's
/// all-capabilities-false profile.
///
/// # Errors
///
/// [`CliError::Core`] carrying `VIA_REALTIME_MODEL_UNKNOWN`.
pub fn assert_known_realtime_model(model: &str) -> Result<(), CliError> {
    via_catalog::resolve_dashscope_realtime_model_profile(model)
        .map(|_| ())
        .map_err(|error| CliError::Core(via_core::CoreError::Catalog(error)))
}

/// Write `contents` to `path` inside the cross-process file transaction.
///
/// **External contract** — `cli/src/config-command.mjs:36-45,73-90`:
/// `withFileTransaction(configPath, …)` around a `mkdir` at mode `0700`, a
/// temp file at mode `0600`, an `fsync`, a `replaceFileSync` and an `fsync` of
/// the directory. All of that is `via-store`'s and `via-core`'s; nothing here
/// re-implements it.
///
/// The lock is not decoration. `config.env` is read by a running Gateway and
/// written by the CLI, the desktop host and possibly a second CLI, and the
/// lock is a `mkdir`-based one precisely so a Node process on the other side
/// of a migration excludes the same way.
///
/// # Errors
///
/// [`CliError::Refused`] with `shared_file_busy` when another process held the
/// lock for the whole timeout, and [`CliError::Io`] for a filesystem failure.
pub fn write_config_file(path: &Path, contents: &str, locale: Locale) -> Result<(), CliError> {
    let outcome = with_file_transaction(Some(path), LockOptions::default(), || {
        write_file(path, contents, IfExists::Replace).map(|_| ())
    });
    match outcome {
        Ok(result) => Ok(result?),
        Err(LockError::Busy { .. }) => Err(CliError::refused(
            via_store::SHARED_FILE_BUSY,
            locale,
            keys::LOCK_SHARED_FILE_BUSY,
        )),
        Err(LockError::Io(error)) => Err(CliError::io("lock", path, error)),
    }
}

/// Validate `model`, then rewrite `path` so it names that model.
///
/// **External contract** — `cli/src/config-command.mjs:36-96`. Validation
/// happens *before* the lock is taken, exactly as upstream validates before
/// `withFileTransaction`: a bad id must not make a well-formed file wait on a
/// lock, and must never reach the writer.
///
/// # Errors
///
/// As [`assert_known_realtime_model`], [`read_config_file`] and
/// [`write_config_file`].
pub fn update_realtime_model(path: &Path, model: &str, locale: Locale) -> Result<(), CliError> {
    assert_known_realtime_model(model)?;
    let existing = read_config_file(path)?;
    let updated = rewrite_assignment(&existing, REALTIME_MODEL_KEY, model);
    write_config_file(path, &updated, locale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    const KEY: &str = "VIA_REALTIME_MODEL";

    #[rstest]
    #[case("VIA_REALTIME_MODEL=x", true)]
    #[case("  VIA_REALTIME_MODEL=x", true)]
    #[case("\tVIA_REALTIME_MODEL = x", true)]
    #[case("VIA_REALTIME_MODEL=", true)]
    #[case("# VIA_REALTIME_MODEL=x", false)]
    #[case("#VIA_REALTIME_MODEL=x", false)]
    #[case("export VIA_REALTIME_MODEL=x", false)]
    #[case("VIA_REALTIME_MODEL_EXTRA=x", false)]
    #[case("XVIA_REALTIME_MODEL=x", false)]
    #[case("VIA_REALTIME_MODEL", false)]
    #[case("", false)]
    fn an_assignment_is_recognised_exactly_as_upstream_recognises_it(
        #[case] line: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(is_assignment(line, KEY), expected, "{line:?}");
    }

    #[test]
    fn the_first_assignment_wins_and_is_trimmed() {
        let contents = "# comment\nVIA_REALTIME_MODEL =  first  \nVIA_REALTIME_MODEL=second\n";
        assert_eq!(first_assignment(contents, KEY), Some("first"));
    }

    #[test]
    fn a_quoted_value_keeps_its_quotes() {
        // Upstream's regex does no unquoting; `via-core`'s env parser does.
        // The two answers differ, deliberately, and this pins which one this
        // module gives.
        assert_eq!(
            first_assignment("VIA_REALTIME_MODEL=\"quoted\"\n", KEY),
            Some("\"quoted\"")
        );
    }

    #[test]
    fn a_missing_key_reads_as_absent() {
        assert_eq!(first_assignment("OTHER=1\n", KEY), None);
        assert_eq!(first_assignment("", KEY), None);
    }

    #[test]
    fn an_empty_file_gains_the_single_assignment() {
        assert_eq!(rewrite_assignment("", KEY, "m"), "VIA_REALTIME_MODEL=m\n");
    }

    #[test]
    fn comments_and_unknown_keys_are_preserved_byte_for_byte() {
        let original = concat!(
            "# VIA user configuration\n",
            "DASHSCOPE_API_KEY=sk-secret\n",
            "\n",
            "# a comment nobody but the user understands\n",
            "SOME_FUTURE_KEY = with spaces \n",
        );
        let updated = rewrite_assignment(original, KEY, "m");
        assert_eq!(
            updated,
            concat!(
                "# VIA user configuration\n",
                "DASHSCOPE_API_KEY=sk-secret\n",
                "\n",
                "# a comment nobody but the user understands\n",
                "SOME_FUTURE_KEY = with spaces \n",
                "VIA_REALTIME_MODEL=m\n",
            )
        );
    }

    #[test]
    fn the_assignment_is_replaced_in_place_keeping_the_comment_above_it() {
        let original = concat!(
            "A=1\n",
            "# the model the Gateway uses\n",
            "VIA_REALTIME_MODEL=old\n",
            "B=2\n",
        );
        assert_eq!(
            rewrite_assignment(original, KEY, "new"),
            concat!(
                "A=1\n",
                "# the model the Gateway uses\n",
                "VIA_REALTIME_MODEL=new\n",
                "B=2\n",
            )
        );
    }

    #[test]
    fn duplicates_collapse_onto_the_first_position() {
        let original = "VIA_REALTIME_MODEL=a\nX=1\n  VIA_REALTIME_MODEL = b\nY=2\n";
        assert_eq!(
            rewrite_assignment(original, KEY, "c"),
            "VIA_REALTIME_MODEL=c\nX=1\nY=2\n"
        );
    }

    #[test]
    fn crlf_is_preserved_when_the_file_already_used_it() {
        let original = "A=1\r\nVIA_REALTIME_MODEL=old\r\n";
        assert_eq!(
            rewrite_assignment(original, KEY, "new"),
            "A=1\r\nVIA_REALTIME_MODEL=new\r\n"
        );
    }

    #[test]
    fn one_crlf_anywhere_makes_the_whole_rewrite_crlf() {
        // Upstream's test is `existing.includes('\r\n')`, not "every line
        // ends with it". A mixed file is normalised to CRLF, which is what
        // upstream does and therefore what a round-trip must produce.
        assert_eq!(
            rewrite_assignment("A=1\r\nB=2\n", KEY, "m"),
            "A=1\r\nB=2\r\nVIA_REALTIME_MODEL=m\r\n"
        );
    }

    #[test]
    fn a_lone_carriage_return_is_not_a_line_break() {
        assert_eq!(
            rewrite_assignment("A=1\rB=2\n", KEY, "m"),
            "A=1\rB=2\nVIA_REALTIME_MODEL=m\n"
        );
    }

    #[test]
    fn a_file_without_a_trailing_newline_gains_exactly_one() {
        assert_eq!(
            rewrite_assignment("A=1", KEY, "m"),
            "A=1\nVIA_REALTIME_MODEL=m\n"
        );
    }

    #[test]
    fn a_blank_last_line_survives_but_is_not_doubled() {
        // "A=1\n\n" splits to ["A=1", "", ""]; only the final empty is popped,
        // so the deliberate blank line in the middle is kept.
        assert_eq!(
            rewrite_assignment("A=1\n\n", KEY, "m"),
            "A=1\n\nVIA_REALTIME_MODEL=m\n"
        );
    }

    #[test]
    fn rewriting_is_idempotent() {
        let once = rewrite_assignment("A=1\nVIA_REALTIME_MODEL=old\n", KEY, "new");
        assert_eq!(rewrite_assignment(&once, KEY, "new"), once);
    }

    #[test]
    fn a_value_that_looks_like_a_comment_is_not_reinterpreted() {
        // The rewriter never parses the value it writes, so a hostile id is
        // written literally and read back literally rather than truncated.
        let updated = rewrite_assignment("", KEY, "m # not a comment");
        assert_eq!(updated, "VIA_REALTIME_MODEL=m # not a comment\n");
        assert_eq!(first_assignment(&updated, KEY), Some("m # not a comment"));
    }

    #[test]
    fn a_value_carrying_a_newline_cannot_forge_a_second_assignment() {
        // `assert_known_realtime_model` makes this unreachable through the
        // command, but the rewriter is public and the property is worth
        // holding on its own: the injected second line is data, and the *first*
        // assignment — the one every reader takes — is still the whole value.
        let updated = rewrite_assignment("", KEY, "m\nDASHSCOPE_API_KEY=stolen");
        assert_eq!(
            first_assignment(&updated, KEY),
            Some("m"),
            "the reader must not see the forged line as the model"
        );
        assert!(
            update_realtime_model_would_refuse("m\nDASHSCOPE_API_KEY=stolen"),
            "and the command must refuse the id before it ever reaches here"
        );
    }

    fn update_realtime_model_would_refuse(model: &str) -> bool {
        assert_known_realtime_model(model).is_err()
    }

    #[test]
    fn the_default_model_applies_when_neither_source_has_one() {
        let env = EnvMap::new();
        assert_eq!(
            resolve_config_model(&env, ""),
            via_catalog::DEFAULT_DASHSCOPE_REALTIME_MODEL
        );
    }

    #[test]
    fn the_file_is_consulted_when_the_environment_is_silent() {
        let env = EnvMap::new();
        assert_eq!(
            resolve_config_model(&env, "VIA_REALTIME_MODEL=from-file\n"),
            "from-file"
        );
    }

    #[test]
    fn the_environment_overrides_the_file() {
        let env: EnvMap = [("VIA_REALTIME_MODEL", "from-env")].into_iter().collect();
        assert_eq!(
            resolve_config_model(&env, "VIA_REALTIME_MODEL=from-file\n"),
            "from-env"
        );
    }

    #[test]
    fn an_empty_environment_value_falls_through_but_a_blank_one_does_not() {
        // `||`, not `??`. Both halves of that are reproduced here: the empty
        // string is falsy and loses to the file; a whitespace string is truthy,
        // wins, and is then trimmed to nothing.
        let empty: EnvMap = [("VIA_REALTIME_MODEL", "")].into_iter().collect();
        assert_eq!(
            resolve_config_model(&empty, "VIA_REALTIME_MODEL=from-file\n"),
            "from-file"
        );
        let blank: EnvMap = [("VIA_REALTIME_MODEL", "   ")].into_iter().collect();
        assert_eq!(
            resolve_config_model(&blank, "VIA_REALTIME_MODEL=from-file\n"),
            ""
        );
    }

    #[test]
    fn an_unknown_model_is_refused_and_a_catalogued_one_is_not() {
        assert!(assert_known_realtime_model("definitely-not-a-model").is_err());
        for profile in via_catalog::dashscope_realtime_model_profiles() {
            assert_known_realtime_model(&profile.id)
                .unwrap_or_else(|_| panic!("{} is in the catalog", profile.id));
        }
    }

    #[test]
    fn writing_creates_the_directory_and_the_file_with_the_contracted_modes() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let nested = dir.path().join("profile");
        let path = nested.join("config.env");
        write_config_file(&path, "A=1\n", Locale::En).expect("write");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "A=1\n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let file = std::fs::metadata(&path).expect("stat file");
            assert_eq!(file.permissions().mode() & 0o777, 0o600);
            let directory = std::fs::metadata(&nested).expect("stat dir");
            assert_eq!(directory.permissions().mode() & 0o777, 0o700);
        }
    }

    #[test]
    fn a_full_update_round_trips_through_the_filesystem() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().join("config.env");
        std::fs::write(&path, "# header\nDASHSCOPE_API_KEY=sk\n").expect("seed");
        let model = via_catalog::realtime_model::DASHSCOPE_OMNI_PLUS_REALTIME_MODEL;
        update_realtime_model(&path, model, Locale::En).expect("update");
        let updated = std::fs::read_to_string(&path).expect("read");
        assert_eq!(
            updated,
            std::format!("# header\nDASHSCOPE_API_KEY=sk\n{REALTIME_MODEL_KEY}={model}\n")
        );
        assert_eq!(first_assignment(&updated, REALTIME_MODEL_KEY), Some(model));
    }

    #[test]
    fn a_rejected_model_leaves_the_file_untouched() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = dir.path().join("config.env");
        std::fs::write(&path, "A=1\n").expect("seed");
        let error = update_realtime_model(&path, "not-a-model", Locale::En).expect_err("refused");
        assert_eq!(error.code(), "VIA_REALTIME_MODEL_UNKNOWN");
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "A=1\n");
    }

    #[test]
    fn a_read_failure_that_is_not_absence_propagates() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        // A directory is not a file: `read_to_string` fails with something
        // other than NotFound, and that must not read as "no configuration".
        let error = read_config_file(dir.path()).expect_err("a directory is not readable as text");
        assert_eq!(error.code(), crate::error::CODE_IO);
    }

    #[test]
    fn absence_reads_as_the_empty_string() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        assert_eq!(
            read_config_file(&dir.path().join("nope.env")).expect("absent is not an error"),
            ""
        );
    }
}
