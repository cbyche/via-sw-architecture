use alternative_a::ThinVia;
use alternative_b::ArgoCentric;
use alternative_c::HybridVia;
use alternative_d::AdaptiveVia;
use bench_core::{
    ArchitectureEvent, ArchitectureUnderTest, ContextEvidence, ExecutionRoute, ExecutionRouteKind,
    ExecutorId, InitialProductState, InitialTaskState, InputModality, ProductFact, TaskId, TurnId,
    UserTurn,
};
use bench_events::{CanonicalEvent, CanonicalEventKind, EventEmitter, validate_episode};
use bench_runner::{RawEvidence, Runner, ScenarioStimulus};
use bench_smoke::{Scenario, SmokePorts};

struct RunResult {
    evidence: RawEvidence,
    events: Vec<CanonicalEvent>,
    ports: SmokePorts,
}

fn turn(id: &str, content: &str) -> UserTurn {
    UserTurn {
        turn_id: TurnId::from(id),
        modality: InputModality::Text,
        content: content.into(),
        context_evidence: vec![ContextEvidence {
            source: "controlled-interaction".into(),
            kind: "raw-context".into(),
            value: "fixture evidence only".into(),
            observed_at_product_revision: None,
        }],
    }
}

fn stimulus(scenario: Scenario) -> ScenarioStimulus {
    let turns = match scenario {
        Scenario::S1Local => vec![turn("turn-1", "볼륨 조금 줄여줘.")],
        Scenario::S2General => vec![turn("turn-1", "다운로드 폴더를 정리해줘.")],
        Scenario::S3Specialized => vec![turn("turn-1", "현재 Wi-Fi 문제를 진단해줘.")],
        Scenario::S4FollowUp => vec![turn("turn-2", "그럼 DNS도 확인해봐.")],
        Scenario::S5Clarification => vec![
            turn("turn-1", "그 문서 열어줘."),
            turn("turn-2", "오른쪽에 있는 거."),
        ],
    };
    let tasks = if scenario == Scenario::S4FollowUp {
        vec![InitialTaskState {
            task_id: TaskId::from("T1"),
            execution_id: None,
            prior_route: Some(ExecutionRoute {
                route_kind: ExecutionRouteKind::ExecutorDirect,
                initial_executor_id: ExecutorId::from("NetworkAgent"),
                final_executor_id_if_known: Some(ExecutorId::from("NetworkAgent")),
                delegation_chain: vec![ExecutorId::from("NetworkAgent")],
            }),
            status: "ACTIVE".into(),
        }]
    } else {
        Vec::new()
    };
    ScenarioStimulus {
        initial_state: InitialProductState {
            conversation_facts: Vec::new(),
            tasks,
            capability_facts: vec![
                ProductFact {
                    key: "fast_capability".into(),
                    value: "local_volume".into(),
                },
                ProductFact {
                    key: "fast_capability".into(),
                    value: "local_document_open".into(),
                },
                ProductFact {
                    key: "executor".into(),
                    value: "ARGO".into(),
                },
                ProductFact {
                    key: "executor".into(),
                    value: "NetworkAgent".into(),
                },
            ],
            policy_facts: vec![ProductFact {
                key: "local_execution".into(),
                value: "allowed".into(),
            }],
        },
        turns,
    }
}

fn run<A>(
    alternative: &str,
    scenario: Scenario,
    mut architecture: A,
    ports: SmokePorts,
) -> RunResult
where
    A: ArchitectureUnderTest<Error = String>,
{
    let mut evidence_source = ports.clone();
    let mut lifecycle = ports.clone();
    let evidence = Runner
        .run(
            &mut architecture,
            &mut evidence_source,
            &mut lifecycle,
            stimulus(scenario),
        )
        .unwrap_or_else(|error| panic!("{alternative}-{} failed: {error:?}", scenario.id()));
    let events = ports.events();
    RunResult {
        evidence,
        events,
        ports,
    }
}

fn execute(alternative: &str, scenario: Scenario) -> RunResult {
    let ports = SmokePorts::new(alternative, scenario);
    match alternative {
        "A" => run(
            "A",
            scenario,
            ThinVia::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
        ),
        "B" => run(
            "B",
            scenario,
            ArgoCentric::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
        ),
        "C" => run(
            "C",
            scenario,
            HybridVia::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
        ),
        "D" => run(
            "D",
            scenario,
            AdaptiveVia::new(ports.clone(), ports.clone(), ports.clone()),
            ports,
        ),
        _ => panic!("unknown alternative"),
    }
}

fn assert_case(alternative: &str, scenario: Scenario, expected_calls: usize) {
    let result = execute(alternative, scenario);
    assert_eq!(
        result.evidence.model_calls.len(),
        expected_calls,
        "{alternative}-{} ModelCall count",
        scenario.id()
    );
    assert_eq!(result.ports.model_requests().len(), expected_calls);
    let owners: Vec<_> = result
        .ports
        .model_requests()
        .into_iter()
        .map(|request| request.decision_owner.0)
        .collect();
    assert_eq!(owners, expected_owners(alternative, scenario));
    assert_eq!(result.ports.executions().len(), 1);
    assert_eq!(result.ports.outcomes(), vec![expected_outcome(scenario)]);
    assert_route(alternative, scenario, &result.events);
    assert_event_contract(scenario, &result.events);
    assert_authority(&result.events);
    assert_eq!(validate_episode(&result.events), Vec::new());
}

fn expected_owners(alternative: &str, scenario: Scenario) -> Vec<String> {
    let owners: &[&str] = match (alternative, scenario) {
        ("A", Scenario::S4FollowUp) => &["A.IntentRefiner"],
        ("A", Scenario::S5Clarification) => {
            &["A.IntentRefiner", "A.IntentRefiner", "A.AgentRouter"]
        }
        ("A", _) => &["A.IntentRefiner", "A.AgentRouter"],
        ("B", Scenario::S5Clarification) => &["B.ARGOPrimary", "B.ARGOPrimary"],
        ("B", _) => &["B.ARGOPrimary"],
        ("C", Scenario::S1Local | Scenario::S4FollowUp) => &["C.IntentRefiner"],
        ("C", Scenario::S5Clarification) => &["C.IntentRefiner", "C.IntentRefiner"],
        ("C", _) => &["C.IntentRefiner", "C.AgentRouter"],
        ("D", Scenario::S4FollowUp) => &["D.IntentRefiner"],
        ("D", Scenario::S5Clarification) => &[
            "D.IntentRefiner",
            "D.IntentRefiner",
            "D.ExecutionPathSelector",
        ],
        ("D", _) => &["D.IntentRefiner", "D.ExecutionPathSelector"],
        _ => panic!("unknown alternative"),
    };
    owners.iter().map(|owner| (*owner).to_owned()).collect()
}

fn expected_outcome(scenario: Scenario) -> String {
    match scenario {
        Scenario::S1Local => "volume_reduced",
        Scenario::S2General => "downloads_organized",
        Scenario::S3Specialized | Scenario::S4FollowUp => "wifi_diagnosis_updated",
        Scenario::S5Clarification => "right_document_opened",
    }
    .into()
}

fn committed_route(events: &[CanonicalEvent]) -> &ExecutionRoute {
    events
        .iter()
        .find_map(|event| match event.event() {
            CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted {
                route, ..
            }) => Some(route),
            _ => None,
        })
        .expect("route committed")
}

fn assert_route(alternative: &str, scenario: Scenario, events: &[CanonicalEvent]) {
    let route = committed_route(events);
    let (kind, initial, final_executor) = match (alternative, scenario) {
        ("C" | "D", Scenario::S1Local | Scenario::S5Clarification) => (
            ExecutionRouteKind::LocalDirect,
            if scenario == Scenario::S1Local {
                "VIA_LOCAL_VOLUME"
            } else {
                "VIA_LOCAL_DOCUMENT"
            },
            if scenario == Scenario::S1Local {
                "VIA_LOCAL_VOLUME"
            } else {
                "VIA_LOCAL_DOCUMENT"
            },
        ),
        ("B", Scenario::S3Specialized | Scenario::S4FollowUp) => (
            ExecutionRouteKind::ExecutorDelegated,
            "ARGO",
            "NetworkAgent",
        ),
        (_, Scenario::S3Specialized | Scenario::S4FollowUp) => (
            ExecutionRouteKind::ExecutorDirect,
            "NetworkAgent",
            "NetworkAgent",
        ),
        _ => (ExecutionRouteKind::ExecutorDirect, "ARGO", "ARGO"),
    };
    assert_eq!(route.route_kind, kind);
    assert_eq!(route.initial_executor_id, ExecutorId::from(initial));
    assert_eq!(
        route.final_executor_id_if_known,
        Some(ExecutorId::from(final_executor))
    );
}

fn assert_event_contract(scenario: Scenario, events: &[CanonicalEvent]) {
    let processing = position(events, |event| {
        matches!(
            event,
            CanonicalEventKind::Architecture(ArchitectureEvent::ProcessingStarted)
        )
    });
    let model = position(events, |event| {
        matches!(event, CanonicalEventKind::ModelGenerationStarted)
    });
    let commit = position(events, |event| {
        matches!(
            event,
            CanonicalEventKind::Architecture(ArchitectureEvent::RouteCommitted { .. })
        )
    });
    let execution = position(events, |event| {
        matches!(event, CanonicalEventKind::ExecutionStarted { .. })
    });
    let outcome = position(events, |event| {
        matches!(event, CanonicalEventKind::UsefulOutcomeObserved { .. })
    });
    let completed = position(events, |event| {
        matches!(event, CanonicalEventKind::EpisodeCompleted)
    });
    assert!(
        processing < model
            && model < commit
            && commit < execution
            && execution < outcome
            && outcome < completed
    );

    if scenario == Scenario::S4FollowUp {
        assert!(events.iter().any(|event| matches!(
            event.event(),
            CanonicalEventKind::Architecture(ArchitectureEvent::TaskReused)
        )));
        assert!(!events.iter().any(|event| matches!(
            event.event(),
            CanonicalEventKind::Architecture(ArchitectureEvent::TaskCreated)
        )));
    }
    if scenario == Scenario::S5Clarification {
        let requested = position(events, |event| {
            matches!(
                event,
                CanonicalEventKind::Architecture(ArchitectureEvent::ClarificationRequested { .. })
            )
        });
        let resolved = position(events, |event| {
            matches!(
                event,
                CanonicalEventKind::Architecture(ArchitectureEvent::ClarificationResolved { .. })
            )
        });
        assert!(requested < resolved);
    }
}

fn position(events: &[CanonicalEvent], predicate: impl Fn(&CanonicalEventKind) -> bool) -> usize {
    events
        .iter()
        .position(|event| predicate(event.event()))
        .expect("required canonical event")
}

fn assert_authority(events: &[CanonicalEvent]) {
    for event in events {
        match event.event() {
            CanonicalEventKind::AcousticEos => {
                assert_eq!(event.emitter(), EventEmitter::InteractionFixture)
            }
            CanonicalEventKind::ModelGenerationStarted
            | CanonicalEventKind::ModelGenerationCompleted => {
                assert_eq!(event.emitter(), EventEmitter::ModelFixture)
            }
            CanonicalEventKind::UsefulOutcomeObserved { .. } => {
                assert_eq!(event.emitter(), EventEmitter::OutcomeProbe)
            }
            CanonicalEventKind::EpisodeCompleted => {
                assert_eq!(event.emitter(), EventEmitter::Benchmark)
            }
            CanonicalEventKind::Architecture(_)
            | CanonicalEventKind::ExecutionStarted { .. }
            | CanonicalEventKind::EpisodeFailed { .. } => {}
        }
    }
}

macro_rules! smoke_case {
    ($name:ident, $alternative:literal, $scenario:expr, $calls:literal) => {
        #[test]
        fn $name() {
            assert_case($alternative, $scenario, $calls);
        }
    };
}

smoke_case!(s1_a_thin_via, "A", Scenario::S1Local, 2);
smoke_case!(s1_b_argo_centric, "B", Scenario::S1Local, 1);
smoke_case!(s1_c_hybrid, "C", Scenario::S1Local, 1);
smoke_case!(s1_d_adaptive, "D", Scenario::S1Local, 2);
smoke_case!(s2_a_thin_via, "A", Scenario::S2General, 2);
smoke_case!(s2_b_argo_centric, "B", Scenario::S2General, 1);
smoke_case!(s2_c_hybrid, "C", Scenario::S2General, 2);
smoke_case!(s2_d_adaptive, "D", Scenario::S2General, 2);
smoke_case!(s3_a_thin_via, "A", Scenario::S3Specialized, 2);
smoke_case!(s3_b_argo_centric, "B", Scenario::S3Specialized, 1);
smoke_case!(s3_c_hybrid, "C", Scenario::S3Specialized, 2);
smoke_case!(s3_d_adaptive, "D", Scenario::S3Specialized, 2);
smoke_case!(s4_a_thin_via, "A", Scenario::S4FollowUp, 1);
smoke_case!(s4_b_argo_centric, "B", Scenario::S4FollowUp, 1);
smoke_case!(s4_c_hybrid, "C", Scenario::S4FollowUp, 1);
smoke_case!(s4_d_adaptive, "D", Scenario::S4FollowUp, 1);
smoke_case!(s5_a_thin_via, "A", Scenario::S5Clarification, 3);
smoke_case!(s5_b_argo_centric, "B", Scenario::S5Clarification, 2);
smoke_case!(s5_c_hybrid, "C", Scenario::S5Clarification, 2);
smoke_case!(s5_d_adaptive, "D", Scenario::S5Clarification, 3);
