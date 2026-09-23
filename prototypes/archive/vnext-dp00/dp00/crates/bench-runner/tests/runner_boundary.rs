use bench_core::{ArchitectureUnderTest, InitialProductState, UserTurn};
use bench_runner::{
    ASYNC_RUNTIME, EvidenceSource, FixtureLifecycle, RUNTIME_WORKER_POLICY_VERSION, RawEvidence,
    Runner, ScenarioStimulus, build_qualification_runtime,
};

#[derive(Default)]
struct RecordingAut {
    setup_called: bool,
    turns: usize,
    teardown_called: bool,
}

impl ArchitectureUnderTest for RecordingAut {
    type Error = &'static str;

    fn setup(&mut self, _state: InitialProductState) -> Result<(), Self::Error> {
        self.setup_called = true;
        Ok(())
    }

    fn handle_user_turn(&mut self, _turn: UserTurn) -> Result<(), Self::Error> {
        self.turns += 1;
        Ok(())
    }

    fn teardown(&mut self) -> Result<(), Self::Error> {
        self.teardown_called = true;
        Ok(())
    }
}

#[derive(Default)]
struct EmptyEvidence;

impl EvidenceSource for EmptyEvidence {
    fn take_raw_evidence(&mut self) -> RawEvidence {
        RawEvidence::default()
    }
}

#[derive(Default)]
struct RecordingFixtures {
    before_called: bool,
    after_called: bool,
    failure_called: bool,
}

impl FixtureLifecycle for RecordingFixtures {
    type Error = &'static str;

    fn before_episode(&mut self) -> Result<(), Self::Error> {
        self.before_called = true;
        Ok(())
    }

    fn before_user_turn(&mut self, _turn_index: usize) -> Result<(), Self::Error> {
        Ok(())
    }

    fn after_episode(&mut self) -> Result<(), Self::Error> {
        self.after_called = true;
        Ok(())
    }

    fn after_failure(&mut self) -> Result<(), Self::Error> {
        self.failure_called = true;
        Ok(())
    }
}

#[test]
fn runner_invokes_terminal_failure_lifecycle_on_architecture_error() {
    struct FailingAut;
    impl ArchitectureUnderTest for FailingAut {
        type Error = &'static str;

        fn setup(&mut self, _state: InitialProductState) -> Result<(), Self::Error> {
            Ok(())
        }

        fn handle_user_turn(&mut self, _turn: UserTurn) -> Result<(), Self::Error> {
            Err("model timeout")
        }

        fn teardown(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    let mut aut = FailingAut;
    let mut evidence = EmptyEvidence;
    let mut fixtures = RecordingFixtures::default();
    let result = Runner.run(
        &mut aut,
        &mut evidence,
        &mut fixtures,
        ScenarioStimulus {
            initial_state: InitialProductState::default(),
            turns: vec![UserTurn {
                turn_id: "turn-1".into(),
                modality: bench_core::InputModality::Text,
                content: "test".into(),
                context_evidence: Vec::new(),
            }],
        },
    );

    assert!(matches!(
        result,
        Err(bench_runner::RunnerError::Architecture("model timeout"))
    ));
    assert!(fixtures.failure_called);
    assert!(!fixtures.after_called);
}

#[test]
fn runner_only_drives_lifecycle_and_returns_raw_evidence() {
    let mut aut = RecordingAut::default();
    let mut evidence = EmptyEvidence;
    let mut fixtures = RecordingFixtures::default();
    let raw = Runner
        .run(
            &mut aut,
            &mut evidence,
            &mut fixtures,
            ScenarioStimulus {
                initial_state: InitialProductState::default(),
                turns: Vec::new(),
            },
        )
        .expect("runner completes");

    assert!(aut.setup_called);
    assert!(aut.teardown_called);
    assert!(fixtures.before_called);
    assert!(fixtures.after_called);
    assert_eq!(aut.turns, 0);
    assert!(raw.events.is_empty());
}

#[test]
fn qualification_runtime_policy_is_common_and_frozen() {
    let runtime = build_qualification_runtime().expect("Tokio runtime builds");
    runtime.block_on(async { tokio::task::yield_now().await });
    assert_eq!(ASYNC_RUNTIME, "tokio-1.53.1");
    assert_eq!(RUNTIME_WORKER_POLICY_VERSION, "tokio-current-thread-v0");
}
