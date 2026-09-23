use bench_core::{
    ArchitectureEvent, ArchitectureObservation, Clock, MonotonicTimestamp, ObservationPort,
    ProductCorrelation, ReferentRole, TurnId,
};
use bench_events::{
    InMemoryObservationCollector, ObservationContext, project_actual_semantic_trace,
};

struct ClockOnce;

impl Clock for ClockOnce {
    fn now(&self) -> MonotonicTimestamp {
        MonotonicTimestamp(1)
    }
}

#[test]
fn intentionally_wrong_actual_referent_is_preserved_without_oracle_substitution() {
    let collector = InMemoryObservationCollector::with_capacity(
        ObservationContext {
            run_id: "wrong-actual".into(),
            episode_id: "episode".into(),
            scenario_id: "synthetic".into(),
            scenario_version: "v1".into(),
            alternative_id: "A".into(),
            benchmark_version: "test".into(),
            schema_version: "canonical-event-v1".into(),
            source_git_commit: "test".into(),
        },
        ClockOnce,
        1,
    );
    collector.emit(ArchitectureObservation {
        event: ArchitectureEvent::ReferentBound {
            referent_role: ReferentRole::Source,
            resolved_referent_id: "file_B".into(),
        },
        product_correlation: ProductCorrelation {
            turn_id: Some(TurnId::from("U1")),
            ..ProductCorrelation::default()
        },
    });

    let trace = project_actual_semantic_trace(&collector.canonical_snapshot());
    assert_eq!(trace.referent_bindings[0].resolved_referent_id, "file_B");
}
