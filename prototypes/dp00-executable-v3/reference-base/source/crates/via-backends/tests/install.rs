//! The installer's branching, against a scripted runner.
//!
//! Ported from `server/test/backend-install.test.mjs`. Every collaborator is
//! injected for the reason upstream injects them: the behaviour worth asserting
//! is *which* steps run, in what order, and which failure code comes out — and
//! none of that should require installing anything.

mod support;

use std::sync::Mutex;

use async_trait::async_trait;
use support::env;
use via_backends::HostPlatform;
use via_backends::detect::{ExecutableFinder, MissingFinder};
use via_backends::install::{
    ComponentReadiness, DEFAULT_STEP_TIMEOUT, DeclineScripts, InstallCancel, InstallError,
    InstallObserver, InstallRequest, ProgressEvent, ProgressPhase, SetupInspector, StepConfirmer,
    StepOutcome, StepRequest, StepRunner, install_backend, install_support,
};
use via_core::EnvMap;
use via_i18n::Locale;

/// A finder that answers everything, so `npm` is always present.
#[derive(Debug)]
struct EverythingFinder;

impl ExecutableFinder for EverythingFinder {
    fn find(&self, command: &str) -> String {
        std::format!("/usr/bin/{command}")
    }
}

/// A runner that records what it was asked to do.
#[derive(Debug, Default)]
struct RecordingRunner {
    calls: Mutex<Vec<Vec<String>>>,
    outcome: StepOutcome,
}

impl RecordingRunner {
    fn succeeding() -> Self {
        Self::default()
    }

    fn with(outcome: StepOutcome) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            outcome,
        }
    }

    fn commands(&self) -> Vec<Vec<String>> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl StepRunner for RecordingRunner {
    async fn run(
        &self,
        request: StepRequest,
        _cancel: &InstallCancel,
        _on_output: &(dyn for<'a> Fn(via_backends::install::OutputStream, &'a str) + Send + Sync),
    ) -> StepOutcome {
        let mut recorded = vec![request.command.clone()];
        recorded.extend(request.arguments.clone());
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(recorded);
        self.outcome.clone()
    }
}

/// A setup report a test describes.
#[derive(Debug)]
struct FixedInspector {
    before: ComponentReadiness,
    after: ComponentReadiness,
    calls: Mutex<usize>,
}

impl FixedInspector {
    fn new(before: ComponentReadiness, after: ComponentReadiness) -> Self {
        Self {
            before,
            after,
            calls: Mutex::new(0),
        }
    }

    fn ready() -> ComponentReadiness {
        ComponentReadiness {
            backend_ready: true,
            adapter_ready: true,
            ready: true,
            packages: Vec::new(),
            first_issue: None,
        }
    }

    fn missing() -> ComponentReadiness {
        ComponentReadiness::default()
    }
}

#[async_trait]
impl SetupInspector for FixedInspector {
    async fn inspect(&self, _id: &str, _env: &EnvMap) -> ComponentReadiness {
        let mut calls = self
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *calls += 1;
        if *calls == 1 {
            self.before.clone()
        } else {
            self.after.clone()
        }
    }
}

#[derive(Debug, Default)]
struct RecordingObserver(Mutex<Vec<(usize, ProgressPhase)>>);

impl RecordingObserver {
    fn phases(&self) -> Vec<(usize, ProgressPhase)> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl InstallObserver for RecordingObserver {
    fn progress(&self, event: &ProgressEvent) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((event.step, event.phase));
    }
}

#[derive(Debug)]
struct AcceptScripts;

#[async_trait]
impl StepConfirmer for AcceptScripts {
    async fn confirm(&self, _index: usize, _display: &str, _command: &str) -> bool {
        true
    }
}

fn request<'a>(
    id: &'a str,
    finder: &'a dyn ExecutableFinder,
    runner: &'a dyn StepRunner,
    inspector: &'a dyn SetupInspector,
    confirmer: &'a dyn StepConfirmer,
    observer: &'a dyn InstallObserver,
) -> InstallRequest<'a> {
    InstallRequest {
        id,
        env: env(&[("PATH", "/usr/bin")]),
        platform: HostPlatform::Linux,
        locale: Locale::En,
        finder,
        runner,
        inspector,
        confirmer,
        observer,
        cancel: InstallCancel::new(),
        step_timeout: DEFAULT_STEP_TIMEOUT,
    }
}

#[tokio::test]
async fn an_already_installed_backend_runs_nothing() {
    let runner = RecordingRunner::succeeding();
    let inspector = FixedInspector::new(FixedInspector::ready(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let report = install_backend(request(
        "codex",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect("already installed");
    assert!(report.already_installed);
    assert!(runner.commands().is_empty());
    assert!(observer.phases().is_empty());
}

#[tokio::test]
async fn only_the_missing_component_is_installed() {
    // Codex itself is present; only its ACP adapter is missing.
    let before = ComponentReadiness {
        backend_ready: true,
        adapter_ready: false,
        ready: false,
        ..ComponentReadiness::default()
    };
    let runner = RecordingRunner::succeeding();
    let inspector = FixedInspector::new(before, FixedInspector::ready());
    let observer = RecordingObserver::default();
    install_backend(request(
        "codex",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect("installs");

    let commands = runner.commands();
    assert_eq!(commands.len(), 1, "{commands:?}");
    assert_eq!(
        commands[0],
        [
            "/usr/bin/npm",
            "install",
            "-g",
            "@agentclientprotocol/codex-acp@1.1.7"
        ]
    );
    assert_eq!(
        observer.phases(),
        [
            (0, ProgressPhase::Skip),
            (1, ProgressPhase::Start),
            (1, ProgressPhase::Done),
        ]
    );
}

#[tokio::test]
async fn a_script_step_is_declined_by_default() {
    let runner = RecordingRunner::succeeding();
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "hermes",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("nobody said yes");
    assert_eq!(error.code(), "DECLINED");
    assert!(
        runner.commands().is_empty(),
        "an unattended `curl | bash` must never run"
    );
}

#[tokio::test]
async fn a_confirmed_script_step_runs_through_the_platform_shell() {
    let runner = RecordingRunner::succeeding();
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    install_backend(request(
        "hermes",
        &EverythingFinder,
        &runner,
        &inspector,
        &AcceptScripts,
        &observer,
    ))
    .await
    .expect("installs");
    let commands = runner.commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0][0], "/bin/sh");
    assert_eq!(commands[0][1], "-c");
    assert_eq!(
        commands[0][2],
        "curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash"
    );
}

#[tokio::test]
async fn a_missing_npm_stops_before_anything_runs() {
    let runner = RecordingRunner::succeeding();
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "qwen",
        &MissingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("no npm");
    assert_eq!(error.code(), "NPM_MISSING");
    assert!(runner.commands().is_empty());
}

#[tokio::test]
async fn a_timed_out_step_names_the_command_that_hung() {
    let runner = RecordingRunner::with(StepOutcome {
        code: -1,
        timeout: true,
        ..StepOutcome::default()
    });
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "qwen",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("timed out");
    assert_eq!(error.code(), "STEP_TIMEOUT");
    assert!(
        error
            .message(Locale::En)
            .contains("npm install -g @qwen-code/qwen-code@0.21.6"),
        "{}",
        error.message(Locale::En)
    );
}

#[tokio::test]
async fn a_failing_step_carries_its_exit_code_and_output_tail() {
    let runner = RecordingRunner::with(StepOutcome {
        code: 7,
        output: "[stderr] EACCES\n".to_owned(),
        ..StepOutcome::default()
    });
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "qwen",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("failed");
    assert_eq!(error.code(), "STEP_FAILED");
    match error {
        InstallError::StepFailed {
            exit_code, cause, ..
        } => {
            assert_eq!(exit_code, 7);
            assert_eq!(cause, "[stderr] EACCES");
        }
        other => panic!("wrong variant: {other:?}"),
    }
}

#[tokio::test]
async fn a_cancelled_step_is_distinguishable_from_a_failed_one() {
    let runner = RecordingRunner::with(StepOutcome {
        code: -1,
        aborted: true,
        ..StepOutcome::default()
    });
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "qwen",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("cancelled");
    assert_eq!(error.code(), "CANCELLED");
    assert_eq!(error.message(Locale::Zh), "安装已取消");
}

#[tokio::test]
async fn a_successful_run_that_still_does_not_check_out_is_a_verify_failure() {
    let runner = RecordingRunner::succeeding();
    let after = ComponentReadiness {
        first_issue: Some("still not on PATH".to_owned()),
        ..ComponentReadiness::default()
    };
    let inspector = FixedInspector::new(FixedInspector::missing(), after);
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "qwen",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("verification failed");
    assert_eq!(error.code(), "VERIFY_FAILED");
    assert_eq!(error.message(Locale::En), "still not on PATH");
}

#[tokio::test]
async fn the_generic_acp_backend_is_never_installed() {
    let runner = RecordingRunner::succeeding();
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "acp",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("nothing to install");
    assert_eq!(error.code(), "UNSUPPORTED");
    assert!(runner.commands().is_empty());
}

#[tokio::test]
async fn an_unknown_backend_is_unsupported_rather_than_a_panic() {
    let runner = RecordingRunner::succeeding();
    let inspector = FixedInspector::new(FixedInspector::missing(), FixedInspector::ready());
    let observer = RecordingObserver::default();
    let error = install_backend(request(
        "nope",
        &EverythingFinder,
        &runner,
        &inspector,
        &DeclineScripts,
        &observer,
    ))
    .await
    .expect_err("no such backend");
    assert_eq!(error.code(), "UNSUPPORTED");
    assert_eq!(error.message(Locale::En), "unsupported backend: nope");
}

#[test]
fn install_support_reports_what_would_run_without_running_it() {
    let support = install_support("codex", &EnvMap::new(), HostPlatform::Linux, Locale::En);
    assert!(support.supported);
    assert!(!support.requires_confirmation);
    assert_eq!(support.steps.len(), 2);
    assert_eq!(support.steps[0].title, "step 1");
    assert_eq!(support.steps[1].title, "step 2 (ACP adapter)");
    assert_eq!(
        support.steps[1].display,
        "npm install -g @agentclientprotocol/codex-acp@1.1.7"
    );
    assert_eq!(
        support
            .authentication
            .expect("codex declares a sign-in")
            .command,
        Some("codex login")
    );
}
