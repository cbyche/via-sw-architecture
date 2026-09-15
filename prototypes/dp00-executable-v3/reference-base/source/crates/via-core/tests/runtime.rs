//! First-run scaffolding against a real temporary directory.

mod common;

use std::path::Path;

use common::env;
use via_i18n::{Locale, keys, t};

use via_core::config::backend::codebuddy_models_json_path;
use via_core::config::names;
use via_core::env::EnvMap;
use via_core::runtime::{
    AUTH_SECRET_HEX_LENGTH, IfExists, RuntimeOptions, WriteOutcome, load_runtime_environment,
    memory_template, seed_codebuddy_models_json, user_config_template, user_model_template,
    write_file,
};

fn options(root: &Path, home: &Path) -> RuntimeOptions {
    RuntimeOptions {
        root: root.to_path_buf(),
        home_directory: home.to_path_buf(),
        working_directory: root.to_path_buf(),
        locale: Locale::En,
        generate_secret: true,
        read_only: false,
    }
}

#[cfg(unix)]
fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::metadata(path)
        .expect("the file exists")
        .permissions()
        .mode()
        & 0o777
}

#[test]
fn a_first_run_creates_every_seed_and_nothing_else() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");
    let mut environment = EnvMap::new();

    let runtime = load_runtime_environment(&mut environment, &options(root.path(), home.path()))
        .expect("scaffolding succeeds on a clean machine");

    let config_directory = home.path().join(".config/via");
    assert_eq!(runtime.paths.config_directory(), config_directory);
    assert_eq!(runtime.paths.data_directory(), config_directory);

    for path in [
        &runtime.config_path,
        &runtime.user_model_path,
        &runtime.frontend_memory_path,
    ] {
        assert!(path.exists(), "{} was not created", path.display());
        #[cfg(unix)]
        assert_eq!(mode_of(path), 0o600, "{} is not 0600", path.display());
    }
    assert!(runtime.openclaw_state_directory.is_dir());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            std::fs::metadata(&config_directory)
                .expect("it exists")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    // No packaged ASSISTANT.md template, so the path points at the template.
    assert!(!runtime.assistant_profile_path.exists());
    assert!(
        runtime
            .assistant_profile_path
            .ends_with("config/frontend-agent/ASSISTANT.md")
    );

    assert!(runtime.generated_secret);
    let secret = environment
        .get(names::AUTH_SECRET)
        .expect("the secret is written back into the environment");
    assert_eq!(secret.len(), AUTH_SECRET_HEX_LENGTH);
    assert!(secret.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn a_second_run_adopts_the_existing_secret_and_edits() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");

    let mut first = EnvMap::new();
    let created = load_runtime_environment(&mut first, &options(root.path(), home.path()))
        .expect("first run");
    let secret = first.get(names::AUTH_SECRET).expect("generated").to_owned();

    // The user edits USER.md.
    std::fs::write(&created.user_model_path, "# USER\n\nedited by hand\n")
        .expect("the seed is writable");

    let mut second = EnvMap::new();
    let again = load_runtime_environment(&mut second, &options(root.path(), home.path()))
        .expect("second run");

    assert!(!again.generated_secret);
    assert_eq!(second.get(names::AUTH_SECRET), Some(secret.as_str()));
    assert_eq!(
        std::fs::read_to_string(&created.user_model_path).expect("still there"),
        "# USER\n\nedited by hand\n",
        "a seed must never be replaced"
    );
}

#[test]
fn an_empty_shell_assignment_does_not_mask_the_persisted_secret() {
    // The one deliberate exception to "the first source wins": the key is
    // removed from the environment before state.env is read.
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");

    let mut first = EnvMap::new();
    load_runtime_environment(&mut first, &options(root.path(), home.path())).expect("first run");
    let secret = first.get(names::AUTH_SECRET).expect("generated").to_owned();

    let mut second: EnvMap = env(&[(names::AUTH_SECRET, "")]);
    let again = load_runtime_environment(&mut second, &options(root.path(), home.path()))
        .expect("second run");
    assert!(!again.generated_secret);
    assert_eq!(second.get(names::AUTH_SECRET), Some(secret.as_str()));
}

#[test]
fn a_configured_secret_wins_outright_and_writes_nothing() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");
    let mut environment = env(&[(names::AUTH_SECRET, "supplied-by-the-operator-and-long")]);

    let runtime = load_runtime_environment(&mut environment, &options(root.path(), home.path()))
        .expect("scaffolding succeeds");
    assert!(!runtime.generated_secret);
    assert_eq!(runtime.state_path, None);
    assert!(!home.path().join(".config/via/state.env").exists());
    assert_eq!(
        environment.get(names::AUTH_SECRET),
        Some("supplied-by-the-operator-and-long")
    );
}

#[test]
fn read_only_resolves_paths_without_touching_the_disk() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");
    let mut environment = EnvMap::new();
    let mut options = options(root.path(), home.path());
    options.read_only = true;

    let runtime =
        load_runtime_environment(&mut environment, &options).expect("read-only always succeeds");
    assert!(!runtime.config_path.exists());
    assert!(!home.path().join(".config/via").exists());
    assert!(!runtime.generated_secret);
    assert_eq!(runtime.state_path, None);
}

#[test]
fn env_files_load_in_precedence_order_and_the_first_source_wins() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");
    let data = home.path().join(".config/via");
    std::fs::create_dir_all(&data).expect("mkdir");

    std::fs::write(
        root.path().join(".env.local"),
        "A=from-local\nB=from-local\n",
    )
    .expect("write");
    std::fs::write(root.path().join(".env"), "B=from-env\nC=from-env\n").expect("write");
    std::fs::write(data.join("config.env"), "C=from-config\nD=from-config\n").expect("write");

    let mut environment = env(&[("A", "from-shell")]);
    let runtime = load_runtime_environment(&mut environment, &options(root.path(), home.path()))
        .expect("scaffolding succeeds");

    assert_eq!(environment.get("A"), Some("from-shell"));
    assert_eq!(environment.get("B"), Some("from-local"));
    assert_eq!(environment.get("C"), Some("from-env"));
    assert_eq!(environment.get("D"), Some("from-config"));

    let loaded: Vec<String> = runtime
        .loaded_files
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect();
    assert_eq!(loaded, vec![".env.local", ".env", "config.env"]);
}

#[test]
fn a_missing_env_file_is_not_an_error() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");
    let mut environment = EnvMap::new();
    let runtime = load_runtime_environment(&mut environment, &options(root.path(), home.path()))
        .expect("no env files at all");
    // Only `config.env`, which scaffolding just created, is absent at load
    // time — so nothing was loaded.
    assert!(runtime.loaded_files.is_empty());
}

#[test]
fn a_packaged_assistant_template_is_copied_once() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");
    let template_directory = root.path().join("config/frontend-agent");
    std::fs::create_dir_all(&template_directory).expect("mkdir");
    std::fs::write(template_directory.join("ASSISTANT.md"), "# ASSISTANT\n").expect("write");

    let mut environment = EnvMap::new();
    let runtime = load_runtime_environment(&mut environment, &options(root.path(), home.path()))
        .expect("scaffolding succeeds");
    assert_eq!(
        runtime.assistant_profile_path,
        home.path().join(".config/via/ASSISTANT.md")
    );
    assert_eq!(
        std::fs::read_to_string(&runtime.assistant_profile_path).expect("copied"),
        "# ASSISTANT\n"
    );

    // A user edit survives the next run.
    std::fs::write(&runtime.assistant_profile_path, "# ASSISTANT\n\nmine\n").expect("write");
    let mut second = EnvMap::new();
    load_runtime_environment(&mut second, &options(root.path(), home.path())).expect("second run");
    assert_eq!(
        std::fs::read_to_string(&runtime.assistant_profile_path).expect("still there"),
        "# ASSISTANT\n\nmine\n"
    );
}

#[test]
fn the_data_directory_can_be_split_from_the_config_directory() {
    let home = tempfile::TempDir::new().expect("tempdir");
    let root = tempfile::TempDir::new().expect("tempdir");
    let split = tempfile::TempDir::new().expect("tempdir");
    let mut environment = env(&[(names::DATA_DIR, &split.path().display().to_string())]);

    let runtime = load_runtime_environment(&mut environment, &options(root.path(), home.path()))
        .expect("scaffolding succeeds");

    // Assets follow the data directory…
    assert!(runtime.config_path.starts_with(split.path()));
    assert!(runtime.user_model_path.starts_with(split.path()));
    assert!(runtime.frontend_memory_path.starts_with(split.path()));
    // …and runtime state stays with the config directory.
    assert!(runtime.task_state_path.starts_with(home.path()));
    assert!(runtime.openclaw_state_directory.starts_with(home.path()));
}

#[test]
fn a_keep_write_never_clobbers_and_a_replace_write_always_does() {
    let directory = tempfile::TempDir::new().expect("tempdir");
    let path = directory.path().join("nested/file.txt");

    assert_eq!(
        write_file(&path, "first", IfExists::Keep).expect("creates"),
        WriteOutcome::Created
    );
    assert_eq!(
        write_file(&path, "second", IfExists::Keep).expect("keeps"),
        WriteOutcome::Kept
    );
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "first");

    assert_eq!(
        write_file(&path, "third", IfExists::Replace).expect("replaces"),
        WriteOutcome::Replaced
    );
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "third");
    #[cfg(unix)]
    assert_eq!(mode_of(&path), 0o600);
    // The staging file is gone.
    assert_eq!(
        std::fs::read_dir(directory.path().join("nested"))
            .expect("read_dir")
            .count(),
        1
    );
}

#[test]
fn every_template_is_localized_and_well_formed() {
    for locale in Locale::ALL.iter().copied() {
        let config = user_config_template(locale);
        assert_eq!(
            config.lines().next(),
            Some(t(locale, keys::RUNTIME_CONFIG_HEADER))
        );
        // Every active assignment carries a renamed or vendor-owned name, and
        // no upstream identity survives.
        for line in config.lines() {
            assert!(
                !line.contains("QWEN_AUDIO") && !line.contains("QWAUDIO"),
                "an upstream variable name survived: {line}"
            );
        }
        assert!(config.contains("\nDASHSCOPE_API_KEY=\n"));
        assert!(config.contains("\nVIA_REALTIME_PROVIDER=dashscope\n"));
        assert!(config.contains("\nAGENT_PROTOCOL=\n"));

        assert!(user_model_template(locale).starts_with("# USER\n"));
        assert!(memory_template(locale).starts_with("# MEMORY\n"));
    }

    // The three locales really do differ.
    assert_ne!(
        user_config_template(Locale::En),
        user_config_template(Locale::Zh)
    );
    assert_ne!(
        user_config_template(Locale::Zh),
        user_config_template(Locale::Ko)
    );
}

#[test]
fn the_seeded_config_file_parses_back_to_exactly_two_assignments() {
    // Everything else in the template is a comment. If a comment ever became a
    // live assignment by accident, this catches it.
    for locale in Locale::ALL.iter().copied() {
        let parsed = via_core::envfile::parse_env_map(&user_config_template(locale));
        let mut keys: Vec<&str> = parsed.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "AGENT_PROTOCOL",
                "DASHSCOPE_API_KEY",
                "VIA_REALTIME_PROVIDER"
            ],
            "the {locale} template must define exactly these three keys"
        );
        assert_eq!(
            parsed.get("VIA_REALTIME_PROVIDER").map(String::as_str),
            Some("dashscope")
        );
        assert_eq!(parsed.get("AGENT_PROTOCOL").map(String::as_str), Some(""));
    }
}

#[test]
fn codebuddy_models_json_seeds_at_the_catalogued_path_and_mode() {
    let workspace = tempfile::TempDir::new().expect("tempdir");
    let template = "{\n  \"models\": []\n}\n";

    let path = codebuddy_models_json_path(workspace.path());
    assert_eq!(
        path,
        workspace.path().join(".codebuddy").join("models.json")
    );
    assert!(!path.exists(), "nothing seeded yet");

    assert_eq!(
        seed_codebuddy_models_json(workspace.path(), template).expect("seeds"),
        WriteOutcome::Created
    );
    assert_eq!(std::fs::read_to_string(&path).expect("read"), template);
    #[cfg(unix)]
    {
        assert_eq!(
            mode_of(&path),
            0o600,
            "file-path/CodeBuddy models.json target: file mode 0o600"
        );
        assert_eq!(
            mode_of(path.parent().expect("parent")),
            0o700,
            "file-path/CodeBuddy models.json target: dir mode 0o700"
        );
    }

    // `'wx'` in upstream, `IfExists::Keep` here: a second seed never clobbers
    // a file already on disk, hand-edited or otherwise.
    assert_eq!(
        seed_codebuddy_models_json(workspace.path(), "{\"models\":[{\"id\":\"different\"}]}")
            .expect("kept"),
        WriteOutcome::Kept
    );
    assert_eq!(std::fs::read_to_string(&path).expect("read"), template);
}
