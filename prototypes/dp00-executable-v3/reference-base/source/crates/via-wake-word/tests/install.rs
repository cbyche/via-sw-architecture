//! The model manager: download, verify, extract, promote.
//!
//! Ported from upstream `server/src/voice/wake-word/model-manager.mjs`, and
//! tested against archives built in memory — including several no honest
//! release page would ever serve.
//!
//! # The property every failure test asserts
//!
//! Not just "it returned an error" but **"and nothing was installed"**. A model
//! manager that reports a checksum mismatch and leaves half an archive on disk
//! has failed at the one job the checksum is there to do. Every negative case
//! below checks the install directory afterwards, and the ones that run against
//! an existing install check that the *previous* model is still intact.

mod common;

use std::path::Path;
use std::sync::Arc;

use common::{Member, body_for, complete_members, fixture_artifact, tar_bz2};
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use via_i18n::Locale;
use via_wake_word::{
    FetchResponse, ModelArtifact, ModelManager, ScriptedFetch, WAKE_WORD_MODEL_FILES, WakeWordError,
};

/// A keyword file that is valid but obviously a fixture.
const KEYWORDS: &str = "w a k e u p @wake_up\n";

fn manager(root: &Path, fetcher: &Arc<ScriptedFetch>) -> ModelManager {
    ModelManager::new(
        root,
        Arc::clone(fetcher) as Arc<dyn via_wake_word::ModelFetch>,
    )
}

/// A complete archive and the artifact whose digest matches it.
fn good_archive() -> (Vec<u8>, ModelArtifact) {
    let archive = tar_bz2(&complete_members(&WAKE_WORD_MODEL_FILES));
    let artifact = fixture_artifact(&archive);
    (archive, artifact)
}

fn entries(directory: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut names: Vec<String> = read
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// ── the happy path ──────────────────────────────────────────────────────────

#[tokio::test]
async fn a_fresh_install_writes_the_four_members_and_the_generated_keyword_file() {
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);

    assert!(!manager.is_installed(&artifact));
    let install = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("the fixture archive installs");

    assert_eq!(install.directory(), root.path().join(artifact.id.as_ref()));
    assert!(manager.is_installed(&artifact));
    assert_eq!(fetcher.requested(), vec![artifact.url()]);

    // The four downloaded members carry the bytes the archive carried…
    for name in artifact.files.archived() {
        let path = install.directory().join(name);
        let contents = std::fs::read_to_string(&path).expect("the member is readable");
        assert_eq!(contents, body_for(name), "{name} has the wrong bytes");
    }
    // …and the fifth is generated, not downloaded.
    assert_eq!(
        std::fs::read_to_string(install.keywords_path()).expect("keywords.txt is readable"),
        KEYWORDS
    );

    // The install is directly usable as a detection configuration.
    let config = install.detection_config(KEYWORDS);
    assert_eq!(
        config.model.encoder,
        install.directory().join(artifact.files.encoder.as_ref())
    );
    assert_eq!(config.keywords_buf, KEYWORDS);

    // Nothing is left behind: five files, no staging directory, no archive.
    assert_eq!(entries(install.directory()).len(), 5);
    assert_eq!(entries(root.path()), vec![artifact.id.to_string()]);
}

#[cfg(unix)]
#[tokio::test]
async fn the_install_uses_the_catalogued_modes() {
    use std::os::unix::fs::PermissionsExt;

    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    // A nested root, so `create_dir_all` has a directory of its own to create
    // with the catalogued mode rather than inheriting the temp dir's.
    let nested = root.path().join("config").join("models").join("wake-word");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(&nested, &fetcher);
    let install = manager.ensure(&artifact, KEYWORDS).await.expect("installs");

    for name in artifact.files.required() {
        let mode = std::fs::metadata(install.directory().join(name))
            .expect("the file exists")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, via_store::FILE_MODE, "{name} is not 0o600");
    }

    let mode = std::fs::metadata(&nested)
        .expect("the root exists")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, via_wake_word::MODEL_DIR_MODE, "the root is not 0o700");
}

#[tokio::test]
async fn an_archive_member_in_a_subdirectory_is_extracted_by_its_basename() {
    // Real k2-fsa release archives put every file under a top-level directory
    // named after the model. Upstream takes `basename(header.name)`; so does
    // VIA, and this is that behaviour rather than an accident of the fixture.
    let members: Vec<Member> = WAKE_WORD_MODEL_FILES
        .archived()
        .iter()
        .map(|name| Member::file(&format!("some-model-dir/{name}"), body_for(name).as_bytes()))
        .collect();
    let archive = tar_bz2(&members);
    let artifact = fixture_artifact(&archive);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let install = manager(root.path(), &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("nested members install");

    for name in artifact.files.archived() {
        assert!(
            install.directory().join(name).is_file(),
            "{name} is missing"
        );
    }
    assert!(!install.directory().join("some-model-dir").exists());
}

#[tokio::test]
async fn members_that_are_not_part_of_the_model_are_ignored() {
    let mut members = complete_members(&WAKE_WORD_MODEL_FILES);
    members.push(Member::file("README.md", b"not part of the model"));
    members.push(Member::file("LICENSE", b"nor this"));
    members.push(Member::directory("test_wavs"));
    let archive = tar_bz2(&members);
    let artifact = fixture_artifact(&archive);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let install = manager(root.path(), &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("installs");

    assert_eq!(entries(install.directory()).len(), 5);
    assert!(!install.directory().join("README.md").exists());
}

// ── the short circuit ───────────────────────────────────────────────────────

#[tokio::test]
async fn an_installed_model_is_not_downloaded_again() {
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);

    manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("first install");
    assert_eq!(fetcher.requests(), 1);

    // The script has nothing left, so a second download would fail loudly.
    manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("second call short-circuits");
    assert_eq!(fetcher.requests(), 1);
}

#[tokio::test]
async fn a_changed_phrase_rewrites_the_keyword_file_without_downloading() {
    // The deviation from upstream that matters most: upstream's completeness
    // check includes keywords.txt, so a phrase change against an installed
    // model would never take effect. VIA's short circuit tests the four
    // downloaded members and rewrites the fifth every time.
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);

    let install = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("first install");
    let changed = "n e w o n e @a_different_phrase\n";
    manager
        .ensure(&artifact, changed)
        .await
        .expect("second call");

    assert_eq!(fetcher.requests(), 1);
    assert_eq!(
        std::fs::read_to_string(install.keywords_path()).expect("readable"),
        changed
    );
    // The downloaded members are untouched — this is a keyword rewrite, not a
    // reinstall.
    for name in artifact.files.archived() {
        assert_eq!(
            std::fs::read_to_string(install.directory().join(name)).expect("readable"),
            body_for(name)
        );
    }
    // The atomic replace left no temp file behind.
    assert_eq!(entries(install.directory()).len(), 5);
}

#[tokio::test]
async fn a_missing_member_reinstalls_rather_than_short_circuiting() {
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive.clone()));
    let manager = manager(root.path(), &fetcher);
    let install = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("first install");

    // Someone deleted the joiner. The four-member check must notice.
    std::fs::remove_file(install.directory().join(artifact.files.joiner.as_ref()))
        .expect("the joiner is removable");
    assert!(!manager.is_installed(&artifact));

    fetcher.push_ok(archive);
    manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("reinstall");
    assert_eq!(fetcher.requests(), 2);
    assert!(manager.is_installed(&artifact));
}

// ── the download failures ───────────────────────────────────────────────────

#[tokio::test]
async fn a_non_success_status_is_reported_with_its_status_and_installs_nothing() {
    let (_, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::new());
    fetcher.push_status(404);
    let manager = manager(root.path(), &fetcher);

    let error = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("a 404 is not an install");
    assert!(matches!(error, WakeWordError::Download { status: 404 }));
    assert!(!manager.is_installed(&artifact));
    assert_eq!(entries(root.path()), Vec::<String>::new());
}

#[tokio::test]
async fn a_success_status_with_no_body_is_reported_as_that_status() {
    // Upstream checks `!response.ok || !response.body` and reports either as
    // the status (`model-manager.mjs:51-53`). A 200 with no body is the second
    // half, and it is a real answer from a proxy that dropped the payload.
    let (_, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::new());
    fetcher.push_response(FetchResponse {
        status: 200,
        body: None,
    });
    let manager = manager(root.path(), &fetcher);

    let error = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("no body");
    assert!(matches!(error, WakeWordError::Download { status: 200 }));
    assert!(!manager.is_installed(&artifact));
}

#[tokio::test]
async fn a_transport_failure_names_the_url_and_is_not_a_status() {
    let (_, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::new());
    fetcher.push_failure("connection reset by peer");
    let manager = manager(root.path(), &fetcher);

    let error = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("no answer");
    match &error {
        WakeWordError::Fetch { url, detail } => {
            assert_eq!(*url, artifact.url());
            assert_eq!(detail, "connection reset by peer");
        }
        other => panic!("expected a transport failure, got {other:?}"),
    }
    assert!(error.to_string().contains(&artifact.url()));
}

#[tokio::test]
async fn a_failed_download_can_be_retried() {
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::new());
    fetcher.push_status(503);
    fetcher.push_ok(archive);
    let manager = manager(root.path(), &fetcher);

    manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("first attempt fails");
    manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("second attempt installs");
    assert_eq!(fetcher.requests(), 2);
    assert!(manager.is_installed(&artifact));
}

// ── the integrity check ─────────────────────────────────────────────────────

#[tokio::test]
async fn an_archive_that_does_not_match_the_pinned_digest_installs_nothing() {
    let (archive, mut artifact) = good_archive();
    let real = artifact.sha256.to_string();
    artifact.sha256 = std::borrow::Cow::Owned("00".repeat(32));

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);

    let error = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("the digest does not match");
    match &error {
        WakeWordError::Checksum { expected, actual } => {
            assert_eq!(*expected, artifact.sha256.to_string());
            assert_eq!(*actual, real);
        }
        other => panic!("expected a checksum failure, got {other:?}"),
    }

    // Nothing reached the disk — not even a staging directory.
    assert!(!manager.is_installed(&artifact));
    assert_eq!(entries(root.path()), Vec::<String>::new());
}

#[tokio::test]
async fn a_digest_mismatch_leaves_a_previous_install_intact() {
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);
    let install = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("first install");

    // A second artifact with the same id and a digest that will not match.
    let poisoned = ModelArtifact {
        sha256: std::borrow::Cow::Owned("11".repeat(32)),
        ..artifact.clone()
    };
    // Break the install so the short circuit does not hide the download.
    std::fs::remove_file(install.directory().join(artifact.files.tokens.as_ref()))
        .expect("removable");
    fetcher.push_ok(tar_bz2(&complete_members(&WAKE_WORD_MODEL_FILES)));

    manager
        .ensure(&poisoned, KEYWORDS)
        .await
        .expect_err("the digest does not match");
    // The three remaining members of the previous install are untouched.
    for name in [
        artifact.files.encoder.as_ref(),
        artifact.files.decoder.as_ref(),
        artifact.files.joiner.as_ref(),
    ] {
        assert_eq!(
            std::fs::read_to_string(install.directory().join(name)).expect("readable"),
            body_for(name)
        );
    }
    assert_eq!(entries(root.path()), vec![artifact.id.to_string()]);
}

#[tokio::test]
async fn an_uppercase_digest_still_verifies() {
    let (archive, mut artifact) = good_archive();
    artifact.sha256 = std::borrow::Cow::Owned(artifact.sha256.to_uppercase());

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    manager(root.path(), &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("hex case is not part of a digest's identity");
}

// ── the archive failures ────────────────────────────────────────────────────

#[tokio::test]
async fn an_archive_missing_a_member_is_incomplete_and_installs_nothing() {
    let members: Vec<Member> = complete_members(&WAKE_WORD_MODEL_FILES)
        .into_iter()
        .filter(|member| member.name != WAKE_WORD_MODEL_FILES.joiner.as_ref())
        .collect();
    let archive = tar_bz2(&members);
    let artifact = fixture_artifact(&archive);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);

    let error = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("incomplete");
    match &error {
        WakeWordError::Incomplete { missing } => {
            assert_eq!(*missing, vec![artifact.files.joiner.to_string()]);
        }
        other => panic!("expected an incomplete install, got {other:?}"),
    }
    assert_eq!(entries(root.path()), Vec::<String>::new());
}

#[tokio::test]
async fn a_truncated_archive_is_an_archive_error() {
    let (archive, _) = good_archive();
    let truncated = archive[..archive.len() / 2].to_vec();
    let artifact = fixture_artifact(&truncated);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(truncated));
    let manager = manager(root.path(), &fetcher);

    let error = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("truncated");
    assert!(
        matches!(error, WakeWordError::Archive { .. }),
        "expected an archive error, got {error:?}"
    );
    assert_eq!(entries(root.path()), Vec::<String>::new());
}

#[tokio::test]
async fn bytes_that_are_not_an_archive_at_all_are_an_archive_error() {
    let body = b"this is not bzip2".to_vec();
    let artifact = fixture_artifact(&body);
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(body));

    let error = manager(root.path(), &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("not an archive");
    assert!(
        matches!(error, WakeWordError::Archive { .. }),
        "expected an archive error, got {error:?}"
    );
}

// ── the hostile archives ────────────────────────────────────────────────────

#[tokio::test]
async fn a_member_named_with_a_traversal_cannot_escape_the_install_directory() {
    let mut members = complete_members(&WAKE_WORD_MODEL_FILES);
    // Wear the encoder's name, but try to land two directories up.
    members[0] = Member::file(
        &format!("../../{}", WAKE_WORD_MODEL_FILES.encoder),
        b"hostile encoder",
    );
    let archive = tar_bz2(&members);
    let artifact = fixture_artifact(&archive);

    let root = TempDir::new().expect("a temp root");
    let outside = root.path().join("outside");
    std::fs::create_dir(&outside).expect("a directory to try to escape into");
    let nested = root.path().join("a").join("b");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let install = manager(&nested, &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("the traversal is flattened, not rejected — upstream behaviour");

    // It landed inside, under its basename.
    assert_eq!(
        std::fs::read_to_string(install.directory().join(artifact.files.encoder.as_ref()))
            .expect("readable"),
        "hostile encoder"
    );
    // And nowhere else.
    assert_eq!(entries(&outside), Vec::<String>::new());
    assert_eq!(
        entries(root.path()),
        vec!["a".to_owned(), "outside".to_owned()]
    );
    assert!(!root.path().join(artifact.files.encoder.as_ref()).exists());
}

#[tokio::test]
async fn a_symlink_wearing_a_member_name_is_not_extracted() {
    let mut members: Vec<Member> = complete_members(&WAKE_WORD_MODEL_FILES)
        .into_iter()
        .filter(|member| member.name != WAKE_WORD_MODEL_FILES.tokens.as_ref())
        .collect();
    members.push(Member::symlink(
        WAKE_WORD_MODEL_FILES.tokens.as_ref(),
        "/etc/passwd",
    ));
    let archive = tar_bz2(&members);
    let artifact = fixture_artifact(&archive);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let error = manager(root.path(), &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("a symlink is not a model file");

    match &error {
        WakeWordError::Incomplete { missing } => {
            assert_eq!(*missing, vec![artifact.files.tokens.to_string()]);
        }
        other => panic!("expected an incomplete install, got {other:?}"),
    }
    assert_eq!(entries(root.path()), Vec::<String>::new());
}

#[tokio::test]
async fn a_directory_wearing_a_member_name_is_not_extracted() {
    let mut members: Vec<Member> = complete_members(&WAKE_WORD_MODEL_FILES)
        .into_iter()
        .filter(|member| member.name != WAKE_WORD_MODEL_FILES.decoder.as_ref())
        .collect();
    members.push(Member::directory(WAKE_WORD_MODEL_FILES.decoder.as_ref()));
    let archive = tar_bz2(&members);
    let artifact = fixture_artifact(&archive);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let error = manager(root.path(), &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect_err("a directory is not a model file");
    assert!(
        matches!(error, WakeWordError::Incomplete { .. }),
        "expected an incomplete install, got {error:?}"
    );
}

#[tokio::test]
async fn a_repeated_member_leaves_the_last_copy() {
    let mut members = complete_members(&WAKE_WORD_MODEL_FILES);
    members.push(Member::file(
        WAKE_WORD_MODEL_FILES.encoder.as_ref(),
        b"the second copy",
    ));
    let archive = tar_bz2(&members);
    let artifact = fixture_artifact(&archive);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let install = manager(root.path(), &fetcher)
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("installs");
    assert_eq!(
        std::fs::read_to_string(install.directory().join(artifact.files.encoder.as_ref()))
            .expect("readable"),
        "the second copy"
    );
}

// ── the keyword guard ───────────────────────────────────────────────────────

#[tokio::test]
async fn installing_with_no_keywords_is_refused_before_anything_is_downloaded() {
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);

    for blank in ["", "   ", "\n", "\t \n"] {
        let error = manager
            .ensure(&artifact, blank)
            .await
            .expect_err("a model with no keywords hears nothing");
        assert!(
            matches!(
                error,
                WakeWordError::Keyword(via_wake_word::KeywordError::Empty { field: "keywords" })
            ),
            "expected an empty-keywords error, got {error:?}"
        );
    }
    assert_eq!(fetcher.requests(), 0, "nothing was downloaded");
    assert_eq!(entries(root.path()), Vec::<String>::new());
}

// ── concurrency ─────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_concurrent_installs_download_once() {
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    // One response in the script: a second download would fail, so the
    // assertion below is not merely a counter.
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = Arc::new(manager(root.path(), &fetcher));

    let (first, second) = tokio::join!(
        {
            let manager = Arc::clone(&manager);
            let artifact = artifact.clone();
            async move { manager.ensure(&artifact, KEYWORDS).await }
        },
        {
            let manager = Arc::clone(&manager);
            let artifact = artifact.clone();
            async move { manager.ensure(&artifact, KEYWORDS).await }
        }
    );

    first.expect("the first install succeeds");
    second.expect("the second waits and finds it installed");
    assert_eq!(fetcher.requests(), 1);
    assert!(manager.is_installed(&artifact));
    assert_eq!(entries(root.path()), vec![artifact.id.to_string()]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_artifacts_install_side_by_side() {
    let first_archive = tar_bz2(&complete_members(&WAKE_WORD_MODEL_FILES));
    let first = fixture_artifact(&first_archive);
    let second_archive = tar_bz2(&{
        let mut members = complete_members(&WAKE_WORD_MODEL_FILES);
        members.push(Member::file("extra.txt", b"so the two archives differ"));
        members
    });
    let second = common::other_artifact(&second_archive);
    assert_ne!(first.id, second.id);

    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::new());
    fetcher.push_ok(first_archive);
    fetcher.push_ok(second_archive);
    let manager = manager(root.path(), &fetcher);

    manager
        .ensure(&first, KEYWORDS)
        .await
        .expect("the first installs");
    manager
        .ensure(&second, KEYWORDS)
        .await
        .expect("the second installs");

    assert!(manager.is_installed(&first));
    assert!(manager.is_installed(&second));
    let mut expected = vec![first.id.to_string(), second.id.to_string()];
    expected.sort();
    assert_eq!(entries(root.path()), expected);
}

// ── the user-facing messages ────────────────────────────────────────────────

#[test]
fn the_three_catalogued_failures_render_through_via_i18n() {
    let download = WakeWordError::Download { status: 503 };
    let checksum = WakeWordError::Checksum {
        expected: "a".repeat(64),
        actual: "b".repeat(64),
    };
    let incomplete = WakeWordError::Incomplete {
        missing: vec!["tokens.txt".to_owned()],
    };

    for locale in Locale::ALL {
        assert_eq!(
            download.localized(*locale),
            via_i18n::format(
                *locale,
                via_i18n::keys::REALTIME_WAKE_WORD_MODEL_DOWNLOAD_FAILED,
                &[("status", "503")]
            )
        );
        assert_eq!(
            checksum.localized(*locale),
            via_i18n::t(
                *locale,
                via_i18n::keys::REALTIME_WAKE_WORD_MODEL_CHECKSUM_FAILED
            )
        );
        assert_eq!(
            incomplete.localized(*locale),
            via_i18n::t(*locale, via_i18n::keys::REALTIME_WAKE_WORD_MODEL_INCOMPLETE)
        );
        // No `<via-i18n: cannot render …>` diagnostic leaked through.
        assert!(!download.localized(*locale).contains("via-i18n:"));
    }

    // The status reaches the sentence.
    assert!(download.localized(Locale::En).contains("503"));
    // And the diagnostic keeps the detail the sentence deliberately omits.
    assert!(checksum.to_string().contains(&"a".repeat(64)));
    assert!(checksum.to_string().contains(&"b".repeat(64)));
    assert!(!checksum.localized(Locale::En).contains(&"a".repeat(64)));
}

#[test]
fn every_other_failure_folds_into_the_detection_stopped_sentence() {
    let error = WakeWordError::Archive {
        detail: "unexpected end of stream".to_owned(),
    };
    for locale in Locale::ALL {
        let rendered = error.localized(*locale);
        assert_eq!(
            rendered,
            via_i18n::format(
                *locale,
                via_i18n::keys::REALTIME_WAKE_WORD_DETECTION_STOPPED,
                &[("detail", &error.to_string())]
            )
        );
        assert!(rendered.contains("unexpected end of stream"));
    }
}

// ── the token guard ─────────────────────────────────────────────────────────

#[tokio::test]
async fn a_keyword_the_model_cannot_encode_is_refused_and_installs_nothing() {
    // The failure this prevents is not a bad error message. `sherpa-onnx` does
    // not return an error for an unencodable token — it logs and ends the
    // process. Writing this keyword file next to the model would mean the next
    // detector open takes the Gateway down.
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);

    let error = manager
        .ensure(&artifact, "w a k ZZZ QQQ @wake_up\n")
        .await
        .expect_err("the fixture inventory has no ZZZ");
    match &error {
        WakeWordError::UnknownTokens { unknown } => {
            assert_eq!(*unknown, vec!["ZZZ".to_owned(), "QQQ".to_owned()]);
        }
        other => panic!("expected unknown tokens, got {other:?}"),
    }
    assert!(error.to_string().contains("ZZZ"));

    assert!(!manager.is_installed(&artifact));
    assert_eq!(entries(root.path()), Vec::<String>::new());
}

#[tokio::test]
async fn a_multi_word_display_label_is_the_case_the_guard_actually_catches() {
    // `@wake up` is two words to the parser: `@wake` is the label and `up` has
    // to be a token. It is the mistake anyone writing a keyword file by hand
    // makes first, and the reason `WakePhrase::label` joins words with `_`.
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));

    let error = manager(root.path(), &fetcher)
        .ensure(&artifact, "w a k e u p @wake up\n")
        .await
        .expect_err("`up` is read as a token, not as part of the label");
    match &error {
        WakeWordError::UnknownTokens { unknown } => {
            assert_eq!(*unknown, vec!["up".to_owned()]);
        }
        other => panic!("expected unknown tokens, got {other:?}"),
    }
}

#[tokio::test]
async fn the_per_keyword_markers_are_not_tokens() {
    // `:score` and `#threshold` are keyword-file markers. VIA does not emit
    // them, but the guard has to agree with the parser about what a token is or
    // it would reject a file the engine accepts.
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let install = manager(root.path(), &fetcher)
        .ensure(&artifact, "w a k e u p @wake_up :2.0 #0.35\n")
        .await
        .expect("markers are not tokens");
    assert_eq!(
        std::fs::read_to_string(install.keywords_path()).expect("readable"),
        "w a k e u p @wake_up :2.0 #0.35\n"
    );
}

#[tokio::test]
async fn a_bad_phrase_change_leaves_the_working_keyword_file_in_place() {
    // The short-circuit path, which is where a `VIA_WAKE_WORD` change lands.
    let (archive, artifact) = good_archive();
    let root = TempDir::new().expect("a temp root");
    let fetcher = Arc::new(ScriptedFetch::serving(archive));
    let manager = manager(root.path(), &fetcher);
    let install = manager
        .ensure(&artifact, KEYWORDS)
        .await
        .expect("first install");

    let error = manager
        .ensure(&artifact, "n o t a t o k e n ZZZ @new_phrase\n")
        .await
        .expect_err("the model cannot encode ZZZ");
    assert!(
        matches!(error, WakeWordError::UnknownTokens { .. }),
        "expected unknown tokens, got {error:?}"
    );

    assert_eq!(
        std::fs::read_to_string(install.keywords_path()).expect("readable"),
        KEYWORDS,
        "the previous, working keyword file survived"
    );
    assert!(manager.is_installed(&artifact));
    assert_eq!(fetcher.requests(), 1, "nothing was re-downloaded");
}
