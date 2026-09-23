use std::collections::{BTreeMap, BTreeSet};

use bench_events::InstrumentationMode;
use bench_runner::{
    Alternative, CalibrationPopulation, FULL_PREWARM_CYCLES, MEASURED_CALIBRATION_CYCLES,
    build_counterbalanced_calibration_schedule,
};

fn schedule() -> Vec<bench_runner::CalibrationExecutionPlan> {
    build_counterbalanced_calibration_schedule(
        "calibration-next",
        &(1..=12).map(|n| format!("P{n:02}")).collect::<Vec<_>>(),
        &Alternative::ALL,
        1,
    )
}

fn measured() -> Vec<bench_runner::CalibrationExecutionPlan> {
    schedule()
        .into_iter()
        .filter(|item| item.population == CalibrationPopulation::Measured)
        .collect()
}

#[test]
fn generates_exactly_two_full_prewarm_cycles() {
    let rows = schedule();
    let cycles = rows
        .iter()
        .filter(|row| row.population == CalibrationPopulation::FullPrewarm)
        .map(|row| row.cycle_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(FULL_PREWARM_CYCLES, 2);
    assert_eq!(cycles, BTreeSet::from(["W0", "W1"]));
}

#[test]
fn full_prewarm_covers_every_scenario_alternative_mode_path() {
    let rows = schedule();
    for cycle in ["W0", "W1"] {
        let paths = rows
            .iter()
            .filter(|row| {
                row.population == CalibrationPopulation::FullPrewarm && row.cycle_id == cycle
            })
            .map(|row| {
                format!(
                    "{}:{}:{:?}",
                    row.scenario_id,
                    row.alternative.id(),
                    row.instrumentation_mode
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(paths.len(), 96);
    }
}

#[test]
fn prewarm_and_invocation_warmup_are_excluded_from_measured_accounting() {
    let rows = schedule();
    assert!(
        rows.iter()
            .filter(|row| row.population != CalibrationPopulation::Measured)
            .all(|row| row.measured_execution_ordinal.is_none())
    );
    assert!(
        rows.iter()
            .filter(|row| row.population == CalibrationPopulation::Measured)
            .all(|row| row.measured_execution_ordinal.is_some())
    );
}

#[test]
fn fixes_sixteen_measured_cycles() {
    let cycles = measured()
        .into_iter()
        .map(|row| row.cycle_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(MEASURED_CALIBRATION_CYCLES, 16);
    assert_eq!(cycles.len(), 16);
}

#[test]
fn measured_accounting_is_1536_executions_and_768_pairs() {
    let rows = measured();
    assert_eq!(rows.len(), 1_536);
    assert_eq!(
        rows.iter()
            .map(|row| row.pair_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        768
    );
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(row.scenario_id.as_str(), "P01" | "P02" | "P03"))
            .map(|row| row.pair_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        192
    );
}

#[test]
fn pair_members_are_adjacent_with_consecutive_ordinals() {
    let rows = measured();
    for pair in rows.chunks_exact(2) {
        assert_eq!(pair[0].pair_id, pair[1].pair_id);
        assert_eq!([pair[0].mode_order_slot, pair[1].mode_order_slot], [0, 1]);
        assert_ne!(pair[0].instrumentation_mode, pair[1].instrumentation_mode);
        assert_eq!(
            pair[0].measured_execution_ordinal.unwrap() + 1,
            pair[1].measured_execution_ordinal.unwrap()
        );
    }
}

#[test]
fn every_cycle_has_48_pairs_and_24_24_mode_order_balance() {
    for cycle in 0..16 {
        let rows = measured()
            .into_iter()
            .filter(|row| row.cycle_index == cycle)
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 96);
        let first = rows
            .iter()
            .filter(|row| row.mode_order_slot == 0)
            .collect::<Vec<_>>();
        assert_eq!(first.len(), 48);
        assert_eq!(
            first
                .iter()
                .filter(|row| row.instrumentation_mode == InstrumentationMode::Capture)
                .count(),
            24
        );
        assert_eq!(
            first
                .iter()
                .filter(|row| row.instrumentation_mode == InstrumentationMode::Minimal)
                .count(),
            24
        );
    }
}

#[test]
fn every_cycle_has_per_alternative_6_6_balance() {
    let rows = measured();
    for cycle in 0..16 {
        for alternative in Alternative::ALL {
            let first = rows
                .iter()
                .filter(|row| {
                    row.cycle_index == cycle
                        && row.alternative == alternative
                        && row.mode_order_slot == 0
                })
                .collect::<Vec<_>>();
            assert_eq!(
                first
                    .iter()
                    .filter(|row| row.instrumentation_mode == InstrumentationMode::Capture)
                    .count(),
                6
            );
            assert_eq!(
                first
                    .iter()
                    .filter(|row| row.instrumentation_mode == InstrumentationMode::Minimal)
                    .count(),
                6
            );
        }
    }
}

#[test]
fn every_cycle_has_per_scenario_2_2_balance() {
    let rows = measured();
    for cycle in 0..16 {
        for scenario in 1..=12 {
            let scenario_id = format!("P{scenario:02}");
            let first = rows
                .iter()
                .filter(|row| {
                    row.cycle_index == cycle
                        && row.scenario_id == scenario_id
                        && row.mode_order_slot == 0
                })
                .collect::<Vec<_>>();
            assert_eq!(
                first
                    .iter()
                    .filter(|row| row.instrumentation_mode == InstrumentationMode::Capture)
                    .count(),
                2
            );
            assert_eq!(
                first
                    .iter()
                    .filter(|row| row.instrumentation_mode == InstrumentationMode::Minimal)
                    .count(),
                2
            );
        }
    }
}

#[test]
fn each_pair_inverts_mode_order_in_the_next_cycle() {
    let rows = measured();
    for cycle in 0..15 {
        for scenario in 1..=12 {
            for alternative in Alternative::ALL {
                let scenario_id = format!("P{scenario:02}");
                let first = |at_cycle| {
                    rows.iter()
                        .find(|row| {
                            row.cycle_index == at_cycle
                                && row.scenario_id == scenario_id
                                && row.alternative == alternative
                                && row.mode_order_slot == 0
                        })
                        .unwrap()
                        .instrumentation_mode
                };
                assert_ne!(first(cycle), first(cycle + 1));
            }
        }
    }
}

#[test]
fn every_semantic_pair_is_capture_first_eight_times_and_minimal_first_eight_times() {
    let mut counts: BTreeMap<(String, u32), (usize, usize)> = BTreeMap::new();
    for row in measured()
        .into_iter()
        .filter(|row| row.mode_order_slot == 0)
    {
        let entry = counts
            .entry((row.scenario_id, row.alternative.ordinal()))
            .or_default();
        match row.instrumentation_mode {
            InstrumentationMode::Capture => entry.0 += 1,
            InstrumentationMode::Minimal => entry.1 += 1,
        }
    }
    assert_eq!(counts.len(), 48);
    assert!(counts.values().all(|count| *count == (8, 8)));
}

#[test]
fn alternative_rotation_repeats_abcd_bcda_cdab_dabc_four_times() {
    let rows = measured();
    let expected = [
        Alternative::ALL.to_vec(),
        vec![
            Alternative::B,
            Alternative::C,
            Alternative::D,
            Alternative::A,
        ],
        vec![
            Alternative::C,
            Alternative::D,
            Alternative::A,
            Alternative::B,
        ],
        vec![
            Alternative::D,
            Alternative::A,
            Alternative::B,
            Alternative::C,
        ],
    ];
    for cycle in 0..16 {
        let mut first_scenario = rows
            .iter()
            .filter(|row| {
                row.cycle_index == cycle && row.scenario_id == "P01" && row.mode_order_slot == 0
            })
            .collect::<Vec<_>>();
        first_scenario.sort_by_key(|row| row.order_slot);
        assert_eq!(
            first_scenario
                .iter()
                .map(|row| row.alternative)
                .collect::<Vec<_>>(),
            expected[(cycle % 4) as usize]
        );
    }
}

#[test]
fn every_alternative_occupies_each_position_four_times() {
    let rows = measured();
    for alternative in Alternative::ALL {
        let mut positions = [0; 4];
        for row in rows.iter().filter(|row| {
            row.scenario_id == "P01" && row.alternative == alternative && row.mode_order_slot == 0
        }) {
            positions[row.order_slot as usize] += 1;
        }
        assert_eq!(positions, [4, 4, 4, 4]);
    }
}

#[test]
fn mode_order_is_balanced_within_every_position_not_derived_from_position() {
    let rows = measured();
    for cycle in 0..16 {
        for position in 0..4 {
            let first = rows
                .iter()
                .filter(|row| {
                    row.cycle_index == cycle
                        && row.order_slot == position
                        && row.mode_order_slot == 0
                })
                .collect::<Vec<_>>();
            assert_eq!(
                first
                    .iter()
                    .filter(|row| row.instrumentation_mode == InstrumentationMode::Capture)
                    .count(),
                6
            );
            assert_eq!(
                first
                    .iter()
                    .filter(|row| row.instrumentation_mode == InstrumentationMode::Minimal)
                    .count(),
                6
            );
        }
    }
}
