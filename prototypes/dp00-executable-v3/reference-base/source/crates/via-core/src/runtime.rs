//! First-run scaffolding: the files VIA creates before it can run.
//!
//! Ported from `loadRuntimeEnvironment` (`shared/runtime-environment.mjs:458-563`).
//! On a fresh machine it creates the two directories, seeds `config.env`,
//! `USER.md`, `MEMORY.md` and `ASSISTANT.md`, and generates the local identity
//! secret into `state.env`. On every subsequent run it does nothing except read
//! them — every seed is **create-if-absent**, never a replace.
//!
//! # Why create-exclusive, not atomic-replace
//!
//! `via-store` owns durable writes and this crate uses it wherever a file is
//! *replaced* ([`write_file`] with [`IfExists::Replace`]). The seeds are not
//! replacements: upstream opens them with Node's `'wx'` flag, which is
//! `O_CREAT | O_EXCL`, and treats `EEXIST` as success. That is the contract, and
//! an atomic replace would do the opposite of what it guarantees — it would
//! clobber a user's edited `USER.md`, or overwrite a secret another process
//! generated a microsecond earlier. So [`IfExists::Keep`] is `create_new`, and
//! `AlreadyExists` is not an error.
//!
//! The secret's `EEXIST` branch is the one that matters: two Gateways starting
//! at once must converge on **one** secret, so the loser re-reads the winner's
//! file rather than overwriting it. Regenerating invalidates every existing
//! client cookie.
//!
//! # The templates are localized
//!
//! Upstream's templates are Chinese literals. VIA renders them from `via-i18n`,
//! so an `en` install gets an English `config.env` and a `zh` install gets
//! upstream's own wording (with the product name renamed per
//! `docs/rebrand.md`). The header line — a catalogued contract — is
//! `runtime.config_header`.

use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use via_i18n::{Locale, keys, t};

use crate::config::names;
use crate::env::EnvMap;
use crate::envfile::parse_env_map;
use crate::error::CoreError;
use crate::paths::{FILE_MODE, InstallPaths};

/// Length in bytes of the generated auth secret.
///
/// **External contract** — `shared/runtime-environment.mjs:132`
/// (`randomBytes(32)`), rendered as 64 lowercase hex characters.
pub const AUTH_SECRET_BYTES: usize = 32;

/// Length of the generated auth secret as written to `state.env`.
pub const AUTH_SECRET_HEX_LENGTH: usize = AUTH_SECRET_BYTES * 2;

/// The env files loaded at startup, in precedence order.
///
/// **External contract** — `shared/runtime-environment.mjs:471-476`. The first
/// source to define a key wins, and the process environment — already populated
/// — sits above all three.
pub const ENV_FILE_NAMES: &[&str] = &[".env.local", ".env"];

/// What [`write_file`] does when the target already exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfExists {
    /// Leave it alone. `O_CREAT | O_EXCL`, upstream's `'wx'`.
    Keep,
    /// Replace it atomically, through `via-store`.
    Replace,
}

/// Whether a [`write_file`] call created the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteOutcome {
    /// The file did not exist and now does.
    Created,
    /// The file already existed and was left as it was.
    Kept,
    /// The file was replaced.
    Replaced,
}

/// Options for [`load_runtime_environment`].
#[derive(Clone, Debug)]
pub struct RuntimeOptions {
    /// The installation root. `.env.local` and `.env` are looked for here, and
    /// `ASSISTANT.md`'s seed template is `<root>/config/frontend-agent/`.
    pub root: PathBuf,
    /// The OS home directory.
    pub home_directory: PathBuf,
    /// The process working directory.
    pub working_directory: PathBuf,
    /// Which locale the seeded templates are written in.
    pub locale: Locale,
    /// Whether to generate the auth secret when none is configured.
    pub generate_secret: bool,
    /// When set, nothing is created: directories are resolved and files are
    /// named, but the filesystem is not touched beyond reading.
    ///
    /// **External contract** — `shared/runtime-environment.mjs:463`
    /// (`readOnly`), which `via config` uses to report paths without
    /// scaffolding a machine that is only being inspected.
    pub read_only: bool,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            home_directory: PathBuf::new(),
            working_directory: PathBuf::new(),
            locale: Locale::default(),
            generate_secret: true,
            read_only: false,
        }
    }
}

/// What scaffolding produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeEnvironment {
    /// The two directories.
    pub paths: InstallPaths,
    /// `<data>/config.env`.
    pub config_path: PathBuf,
    /// `<data>/state.env`, or `None` when no secret was needed.
    pub state_path: Option<PathBuf>,
    /// `<data>/USER.md`.
    pub user_model_path: PathBuf,
    /// `<data>/ASSISTANT.md`, or the package template when there is none to
    /// seed from — upstream returns the template path in that case
    /// (`shared/runtime-environment.mjs:186`).
    pub assistant_profile_path: PathBuf,
    /// `<data>/MEMORY.md`.
    pub frontend_memory_path: PathBuf,
    /// `<data>/frontend-notes.json`.
    pub frontend_notes_path: PathBuf,
    /// `<config>/tasks.json`.
    pub task_state_path: PathBuf,
    /// `<data>/workspace`.
    pub shared_workspace: PathBuf,
    /// `<config>/backends/openclaw/state`.
    pub openclaw_state_directory: PathBuf,
    /// Which of the candidate env files actually existed, in load order.
    pub loaded_files: Vec<PathBuf>,
    /// Whether this run generated the auth secret.
    pub generated_secret: bool,
}

/// Resolve directories, load env files, and scaffold what is missing.
///
/// `env` is mutated the way upstream mutates `process.env`: file values fill
/// keys it does not already define, and the generated secret is written back
/// into it so the caller's next [`crate::config::resolve`] sees it.
///
/// # Errors
///
/// [`CoreError::Io`] for any filesystem failure other than the expected
/// `AlreadyExists` and `NotFound`, and
/// [`CoreError::InvalidGeneratedSecret`] when `state.env` exists but defines no
/// secret.
pub fn load_runtime_environment(
    env: &mut EnvMap,
    options: &RuntimeOptions,
) -> Result<RuntimeEnvironment, CoreError> {
    let paths = InstallPaths::from_env(env, &options.home_directory, &options.working_directory);

    let mut candidates: Vec<PathBuf> = ENV_FILE_NAMES
        .iter()
        .map(|name| options.root.join(name))
        .collect();
    candidates.push(paths.config_file());

    let mut loaded_files = Vec::new();
    for candidate in candidates {
        if load_env_file(env, &candidate)? {
            loaded_files.push(candidate);
        }
    }

    if !options.read_only {
        create_directory(paths.config_directory())?;
        if paths.data_directory() != paths.config_directory() {
            create_directory(paths.data_directory())?;
        }
    }

    let config_path = paths.config_file();
    let user_model_path = paths.user_model_file();
    let frontend_memory_path = paths.memory_file();
    let assistant_template = options
        .root
        .join(crate::paths::FRONTEND_PROMPT_DIRECTORY)
        .join(crate::paths::ASSISTANT_PROFILE_FILE_NAME);
    let mut assistant_profile_path = paths.assistant_profile_file();

    if !options.read_only {
        write_file(
            &config_path,
            &user_config_template(options.locale),
            IfExists::Keep,
        )?;
        write_file(
            &user_model_path,
            &user_model_template(options.locale),
            IfExists::Keep,
        )?;
        write_file(
            &frontend_memory_path,
            &memory_template(options.locale),
            IfExists::Keep,
        )?;
        match std::fs::read_to_string(&assistant_template) {
            Ok(template) => {
                write_file(&assistant_profile_path, &template, IfExists::Keep)?;
            }
            // No packaged template: upstream returns the template path itself
            // rather than seeding an empty persona.
            Err(error) if error.kind() == ErrorKind::NotFound => {
                assistant_profile_path = assistant_template;
            }
            Err(error) => return Err(CoreError::io("read", assistant_template, error)),
        }
        create_directory(&paths.openclaw_state_directory())?;
    }

    let (generated_secret, state_path) = if options.generate_secret && !options.read_only {
        ensure_generated_secret(env, &paths)?
    } else {
        (false, None)
    };

    Ok(RuntimeEnvironment {
        config_path,
        state_path,
        user_model_path,
        assistant_profile_path,
        frontend_memory_path,
        frontend_notes_path: paths.frontend_notes_file(),
        task_state_path: paths.task_state_file(),
        shared_workspace: paths.shared_workspace(),
        openclaw_state_directory: paths.openclaw_state_directory(),
        loaded_files,
        generated_secret,
        paths,
    })
}

/// Read an env file into `env`, filling only keys it does not already define.
///
/// Returns whether the file existed. A missing file is not an error — all three
/// candidates are optional.
///
/// # Errors
///
/// [`CoreError::Io`] for a read failure that is not "not found".
pub fn load_env_file(env: &mut EnvMap, path: &Path) -> Result<bool, CoreError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(CoreError::io("read", path, error)),
    };
    // A key repeated inside one file keeps its last assignment, as
    // `parseEnv`'s object does; the file as a whole still loses to anything
    // already defined.
    for (key, value) in parse_env_map(&contents) {
        env.set_if_absent(key, value);
    }
    Ok(true)
}

/// Generate the local identity secret, or adopt one that already exists.
///
/// **External contract** — `shared/runtime-environment.mjs:115-148`. The order
/// is exact and each step earns its place:
///
/// 1. A secret already in the environment wins outright.
/// 2. Otherwise the key is **removed** from the environment first, so an empty
///    shell assignment (`VIA_AUTH_SECRET=`) cannot mask the persisted one.
/// 3. `state.env` is read; if it supplies a secret, its mode is repaired to
///    `0600` and nothing is generated.
/// 4. Otherwise 32 random bytes are written as
///    `VIA_AUTH_SECRET=<64 hex>\n` with `O_EXCL`.
/// 5. If that races and loses, the file is re-read rather than overwritten —
///    regenerating would invalidate every existing client cookie.
fn ensure_generated_secret(
    env: &mut EnvMap,
    paths: &InstallPaths,
) -> Result<(bool, Option<PathBuf>), CoreError> {
    if env.get_truthy(names::AUTH_SECRET).is_some() {
        return Ok((false, None));
    }
    let state_path = paths.state_file();
    env.remove(names::AUTH_SECRET);
    load_env_file(env, &state_path)?;
    if env.get_truthy(names::AUTH_SECRET).is_some() {
        set_file_mode(&state_path)?;
        return Ok((false, Some(state_path)));
    }

    create_directory(paths.data_directory())?;
    let secret = generate_auth_secret()?;
    let line = format!("{}={secret}\n", names::AUTH_SECRET);
    match write_file(&state_path, &line, IfExists::Keep)? {
        WriteOutcome::Created => {
            env.set(names::AUTH_SECRET, secret);
            Ok((true, Some(state_path)))
        }
        // Lost the race: adopt the winner's secret.
        WriteOutcome::Kept | WriteOutcome::Replaced => {
            load_env_file(env, &state_path)?;
            if env.get_truthy(names::AUTH_SECRET).is_none() {
                return Err(CoreError::InvalidGeneratedSecret { path: state_path });
            }
            Ok((false, Some(state_path)))
        }
    }
}

/// 32 CSPRNG bytes as 64 lowercase hex characters.
///
/// # Errors
///
/// [`CoreError::Io`] when the operating system declines to supply randomness.
/// Refusing to start is the only safe answer: a predictable auth secret lets
/// anyone mint any owner id.
pub fn generate_auth_secret() -> Result<String, CoreError> {
    let mut bytes = [0u8; AUTH_SECRET_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| {
        CoreError::io(
            "read randomness for",
            PathBuf::from("state.env"),
            std::io::Error::other(error),
        )
    })?;
    let mut hex = String::with_capacity(AUTH_SECRET_HEX_LENGTH);
    for byte in bytes {
        use std::fmt::Write as _;
        // Writing to a `String` cannot fail.
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

/// Write a file at mode `0600`, creating parents as needed.
///
/// # Errors
///
/// [`CoreError::Io`].
pub fn write_file(
    path: &Path,
    contents: &str,
    if_exists: IfExists,
) -> Result<WriteOutcome, CoreError> {
    if let Some(parent) = path.parent() {
        create_directory(parent)?;
    }
    match if_exists {
        IfExists::Keep => {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt as _;
                options.mode(FILE_MODE);
            }
            match options.open(path) {
                Ok(mut file) => {
                    file.write_all(contents.as_bytes())
                        .map_err(|error| CoreError::io("write", path, error))?;
                    set_file_mode(path)?;
                    Ok(WriteOutcome::Created)
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    set_file_mode(path)?;
                    Ok(WriteOutcome::Kept)
                }
                Err(error) => Err(CoreError::io("create", path, error)),
            }
        }
        IfExists::Replace => {
            // The one durable *replace* in this crate, and it goes through
            // via-store rather than a hand-rolled temp-and-rename.
            let temp = temp_path(path);
            via_store::write_atomic(&temp, path, contents)
                .map_err(|error| CoreError::io("replace", path, error))?;
            set_file_mode(path)?;
            Ok(WriteOutcome::Replaced)
        }
    }
}

/// Seed the CodeBuddy model table if it is not already there.
///
/// **External contract** — `file-path/CodeBuddy models.json target`
/// (`shared/runtime-environment.mjs:366-410,:550-556`):
/// `<codeBuddyWorkspace>/.codebuddy/models.json`, directory mode `0o700`, file
/// mode `0o600`, written with the exclusive-create flag upstream calls
/// `'wx'` — exactly [`IfExists::Keep`], which this delegates to, so a
/// VIA-written table can never silently clobber a file the user hand-edited.
/// `contents` is the caller's packaged template — `via-backends`' own
/// `CODEBUDDY_MODELS_JSON_TEMPLATE`, shipped verbatim from
/// `config/codebuddy/workspace/.codebuddy/models.json` — so via-core stays
/// free of a dependency on via-backends' asset.
///
/// Upstream additionally rewrites the template's default model id to the
/// operator's configured one, and deletes the file when the configured model
/// is cleared and the on-disk copy is still byte-identical to what VIA would
/// regenerate. Both are still open — see `docs/deviations/README.md`'s
/// `via-backends` entry — so this seeds the template unmodified rather than
/// guessing at the rewrite.
///
/// # Errors
///
/// [`CoreError::Io`].
pub fn seed_codebuddy_models_json(
    codebuddy_workspace: &Path,
    contents: &str,
) -> Result<WriteOutcome, CoreError> {
    let path = crate::config::backend::codebuddy_models_json_path(codebuddy_workspace);
    write_file(&path, contents, IfExists::Keep)
}

/// `<path>.<pid>.tmp`, the staging name `via-store` documents as a contract.
fn temp_path(path: &Path) -> PathBuf {
    let mut raw = path.to_path_buf().into_os_string();
    raw.push(format!(".{}.tmp", std::process::id()));
    PathBuf::from(raw)
}

/// Create a directory and its parents at mode `0700`.
///
/// # Errors
///
/// [`CoreError::Io`].
pub fn create_directory(path: &Path) -> Result<(), CoreError> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(crate::paths::DIRECTORY_MODE);
    }
    builder
        .create(path)
        .map_err(|error| CoreError::io("create directory", path, error))
}

/// Repair an existing file's mode to `0600`.
///
/// **External contract** — `shared/runtime-environment.mjs:124-129,158-162`
/// re-`chmod`s every seed on every run, so a file created by an older version
/// with a looser umask is tightened. A missing file is not an error.
///
/// No-op on Windows, where the file inherits the parent ACL — the same
/// limitation `via-log` and `via-store` record.
fn set_file_mode(path: &Path) -> Result<(), CoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        match std::fs::set_permissions(path, std::fs::Permissions::from_mode(FILE_MODE)) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(CoreError::io("chmod", path, error)),
        }
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// The seeded `config.env`.
///
/// **External contract** — `shared/runtime-environment.mjs:21-47`. The header
/// line is catalogued by name; every other comment is a `via-i18n` key, and
/// every assignment carries the renamed `VIA_*` variable.
#[must_use]
pub fn user_config_template(locale: Locale) -> String {
    let comment = |key| t(locale, key);
    [
        comment(keys::RUNTIME_CONFIG_HEADER),
        "DASHSCOPE_API_KEY=",
        // Written active, unlike `.env.example`, so a fresh install has an
        // explicit provider (`docs/reference/contracts.json`, *runtime config
        // file (the real one)*).
        "VIA_REALTIME_PROVIDER=dashscope",
        comment(keys::RUNTIME_CONFIG_COMMENT_SPEECH_TO_SPEECH),
        "# SPEECH_TO_SPEECH_REALTIME_URL=ws://127.0.0.1:8765/v1/realtime",
        "",
        comment(keys::RUNTIME_CONFIG_COMMENT_AGENT_PROTOCOL),
        comment(keys::RUNTIME_CONFIG_COMMENT_AGENT_CHOICES),
        // Empty, unlike `.env.example`'s `AGENT_PROTOCOL=openclaw`.
        "AGENT_PROTOCOL=",
        comment(keys::RUNTIME_CONFIG_COMMENT_PERMISSION_MODE),
        comment(keys::RUNTIME_CONFIG_COMMENT_PI_PERMISSION),
        "# VIA_BACKEND_PERMISSION_MODE=native",
        comment(keys::RUNTIME_CONFIG_COMMENT_BACKEND_MODEL),
        "# VIA_BACKEND_MODEL=",
        comment(keys::RUNTIME_CONFIG_COMMENT_BACKEND_AGENT),
        comment(keys::RUNTIME_CONFIG_COMMENT_KIMI),
        // Upstream's DeepSeek line is prose with no catalog key; the variable
        // it names is kept as a commented assignment so the surface is not
        // lost. See `docs/deviations/phase-1.md`.
        "# DEEPSEEK_API_KEY=",
        comment(keys::RUNTIME_CONFIG_COMMENT_GENERIC_ACP),
        comment(keys::RUNTIME_CONFIG_COMMENT_GENERIC_ACP_ENV),
        "",
        comment(keys::RUNTIME_CONFIG_COMMENT_LOGGING),
        "# VIA_LOG_LEVEL=info",
        "# VIA_LOG_MAX_BYTES=10485760",
        "# VIA_LOG_MAX_FILES=5",
        "",
    ]
    .join("\n")
}

/// The seeded `USER.md`.
///
/// **External contract** — `shared/runtime-environment.mjs:48-68`. Upstream's
/// three `<!-- 例如 -->` example lines have no `via-i18n` key and are omitted;
/// the headings and the guidance block are reproduced. See
/// `docs/deviations/phase-1.md`.
#[must_use]
pub fn user_model_template(locale: Locale) -> String {
    let line = |key| t(locale, key);
    [
        "# USER",
        "",
        "<!--",
        line(keys::RUNTIME_USER_MD_GUIDANCE_1),
        line(keys::RUNTIME_USER_MD_GUIDANCE_2),
        line(keys::RUNTIME_USER_MD_GUIDANCE_3),
        line(keys::RUNTIME_USER_MD_GUIDANCE_4),
        "-->",
        "",
        line(keys::RUNTIME_USER_MD_HEADING_ADDRESS),
        "",
        line(keys::RUNTIME_USER_MD_HEADING_INTERACTION),
        "",
    ]
    .join("\n")
}

/// The seeded `MEMORY.md`.
///
/// **External contract** — `shared/runtime-environment.mjs:69-77`.
#[must_use]
pub fn memory_template(locale: Locale) -> String {
    let line = |key| t(locale, key);
    [
        "# MEMORY",
        "",
        "<!--",
        line(keys::RUNTIME_MEMORY_MD_GUIDANCE_1),
        line(keys::RUNTIME_MEMORY_MD_GUIDANCE_2),
        "-->",
        "",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_secret_is_sixty_four_hex_characters() {
        let secret = generate_auth_secret().expect("the OS supplies randomness");
        assert_eq!(secret.len(), AUTH_SECRET_HEX_LENGTH);
        assert!(
            secret
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
        );
        assert_ne!(
            secret,
            generate_auth_secret().expect("the OS supplies randomness")
        );
    }

    #[test]
    fn the_config_template_header_is_the_first_line() {
        for locale in [Locale::En, Locale::Zh, Locale::Ko] {
            let template = user_config_template(locale);
            assert_eq!(
                template.lines().next(),
                Some(t(locale, keys::RUNTIME_CONFIG_HEADER))
            );
            assert!(template.contains("\nAGENT_PROTOCOL=\n"));
        }
    }
}
