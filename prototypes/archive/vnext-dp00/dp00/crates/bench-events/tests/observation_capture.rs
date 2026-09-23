use std::sync::atomic::{AtomicU64, Ordering};

use bench_core::{
    ArchitectureEvent, ArchitectureObservation, Clock, MonotonicTimestamp, ObservationPort,
    ProductCorrelation,
};
use bench_events::{InMemoryObservationCollector, ObservationContext, serialization};

struct ControlledClock(AtomicU64);

impl Clock for ControlledClock {
    fn now(&self) -> MonotonicTimestamp {
        MonotonicTimestamp(self.0.fetch_add(10, Ordering::Relaxed))
    }
}

fn correlation() -> ProductCorrelation {
    ProductCorrelation {
        turn_id: None,
        task_id: None,
        execution_id: None,
        dispatch_id: None,
        result_id: None,
        clarification_id: None,
        ..ProductCorrelation::default()
    }
}

#[test]
fn capture_is_typed_in_memory_and_serialization_is_explicitly_post_capture() {
    let collector = InMemoryObservationCollector::with_capacity(
        ObservationContext {
            run_id: "run-1".into(),
            episode_id: "episode-1".into(),
            scenario_id: "scenario-1".into(),
            scenario_version: "v1".into(),
            alternative_id: "future-aut".into(),
            benchmark_version: "v0".into(),
            schema_version: "v0".into(),
            source_git_commit: "test".into(),
        },
        ControlledClock(AtomicU64::new(100)),
        2,
    );

    collector.emit(ArchitectureObservation {
        event: ArchitectureEvent::ProcessingStarted,
        product_correlation: correlation(),
    });
    collector.emit(ArchitectureObservation {
        event: ArchitectureEvent::TaskCreated,
        product_correlation: correlation(),
    });

    let captures = collector.captured_snapshot();
    assert_eq!(captures.len(), 2);

    // Both provenance enrichment and serialization are explicit post-capture steps.
    let events = collector.canonical_snapshot();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].sequence_number(), 0);
    assert_eq!(events[1].monotonic_timestamp(), MonotonicTimestamp(110));

    let mut jsonl = Vec::new();
    serialization::write_jsonl(&events, &mut jsonl).expect("post-capture serialization succeeds");
    assert_eq!(jsonl.iter().filter(|byte| **byte == b'\n').count(), 2);
}
