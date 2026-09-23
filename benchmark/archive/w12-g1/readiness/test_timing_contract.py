import unittest
from timing_contract import Event, duration, endpoint_sample, paired_samples, union_duration

CLOCK = "controller-1"
REPLAY = {"kind": "synthetic", "sample_id": "case1-trial0", "trace_sha256": "a" * 64}


def events(**values):
    return [Event(name, value, CLOCK) for name, value in values.items()]


class TimingContractTests(unittest.TestCase):
    def test_s2s_is_part_of_elapsed_time_not_added_twice(self):
        sample = endpoint_sample(events(user_input_end=0, core_input_ready=300, first_meaningful_output=900), "W-01", "voice", REPLAY)
        self.assertEqual(sample["wall_elapsed_ns"], 900)
        self.assertEqual(sample["post_core_ready_window_ns"], 600)
        self.assertIsNone(sample["p95"])
        self.assertIsNone(sample["score"])

    def test_handoff_needs_both_acceptance_and_durable_link(self):
        sample = endpoint_sample(events(user_input_end=10, core_input_ready=30, agent_acceptance_confirmed=60, recoverable_link_committed=90), "W-02", "voice", REPLAY)
        self.assertEqual(sample["wall_elapsed_ns"], 80)

    def test_acceptance_can_be_later_than_commit(self):
        sample = endpoint_sample(events(user_input_end=0, agent_acceptance_confirmed=100, recoverable_link_committed=20), "W-02", "text", None)
        self.assertEqual(sample["wall_elapsed_ns"], 100)

    def test_local_queue_is_not_handoff_completion(self):
        with self.assertRaises(ValueError):
            endpoint_sample(events(user_input_end=0, local_enqueued=20, recoverable_link_committed=30), "W-02", "text", None)

    def test_text_cannot_include_s2s(self):
        with self.assertRaises(ValueError):
            endpoint_sample(events(user_input_end=0, first_meaningful_output=30), "W-01", "text", REPLAY)

    def test_voice_replay_requires_provenance(self):
        with self.assertRaises(ValueError):
            endpoint_sample(events(user_input_end=0, first_meaningful_output=30), "W-01", "voice", None)

    def test_replayed_measured_s2s_still_is_not_live_system_measurement(self):
        replay = dict(REPLAY, kind="measured_trace_replay")
        sample = endpoint_sample(events(user_input_end=0, first_meaningful_output=30), "W-01", "voice", replay, sink="ACTUAL_USER_DELIVERY")
        self.assertEqual(sample["evidence"], "SIMULATED_E2E")

    def test_live_model_with_headless_sink_is_not_actual_ui_delivery(self):
        sample = endpoint_sample(events(user_input_end=0, first_meaningful_output=30), "W-01", "voice", dict(REPLAY, kind="live"))
        self.assertTrue(sample["evidence"].endswith("HEADLESS"))

    def test_feedback_starts_at_source_not_broker(self):
        sample = endpoint_sample(events(agent_event_available=10, broker_received=60, first_meaningful_output=90), "W-03", "text", None)
        self.assertEqual(sample["wall_elapsed_ns"], 80)

    def test_cross_process_clock_cannot_be_subtracted(self):
        with self.assertRaises(ValueError):
            duration(Event("a", 0, "process-a"), Event("b", 10, "process-b"))

    def test_durable_and_acceptance_clock_mismatch_rejected(self):
        data = [Event("user_input_end", 0, CLOCK), Event("agent_acceptance_confirmed", 10, CLOCK), Event("recoverable_link_committed", 20, "worker")]
        with self.assertRaises(ValueError):
            endpoint_sample(data, "W-02", "text", None)

    def test_human_wait_overlap_not_double_subtracted(self):
        sample = endpoint_sample(events(user_input_end=0, agent_acceptance_confirmed=90, recoverable_link_committed=100), "W-02", "text", None, [(10, 30), (20, 40)])
        self.assertEqual(sample["net_endpoint_ns"], 70)

    def test_wait_outside_interval_rejected(self):
        with self.assertRaises(ValueError):
            endpoint_sample(events(user_input_end=0, agent_acceptance_confirmed=90, recoverable_link_committed=100), "W-02", "text", None, [(10, 110)])

    def test_duplicate_endpoint_rejected(self):
        with self.assertRaises(ValueError):
            endpoint_sample(events(user_input_end=0, first_meaningful_output=30) + [Event("user_input_end", 1, CLOCK)], "W-01", "text", None)

    def test_negative_time_rejected(self):
        with self.assertRaises(ValueError):
            Event("negative", -1, CLOCK)

    def test_boolean_is_not_timestamp(self):
        with self.assertRaises(ValueError):
            Event("boolean", True, CLOCK)

    def test_core_ready_after_end_rejected(self):
        with self.assertRaises(ValueError):
            endpoint_sample(events(user_input_end=0, core_input_ready=50, first_meaningful_output=30), "W-01", "voice", REPLAY)

    def test_no_percentile_sum(self):
        self.assertEqual(union_duration([(0, 10), (1, 5), (8, 20)]), 20)

    def test_pair_samples_identical(self):
        item = {key: "same" for key in ("trial_id", "case_id", "input_sha256", "s2s_sample_id", "s2s_trace_sha256", "model_digest", "runtime_digest")}
        paired_samples(item, dict(item))

    def test_pair_model_change_rejected(self):
        item = {key: "same" for key in ("trial_id", "case_id", "input_sha256", "s2s_sample_id", "s2s_trace_sha256", "model_digest", "runtime_digest")}
        with self.assertRaises(ValueError):
            paired_samples(item, dict(item, model_digest="better-model"))


if __name__ == "__main__":
    unittest.main()
