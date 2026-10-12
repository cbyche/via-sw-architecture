"""Dependency/attribution regressions using explicit synthetic clock evidence."""
import unittest

from prototypes.candidates.dp41_45_poc.shared.timing import CausalTimeline, TimingContractError


class TimingTests(unittest.TestCase):
    def test_parallel_external_agent_intervals_are_zeroed_without_sum_subtraction(self):
        timeline = CausalTimeline()
        start = timeline.point("REQUEST_AVAILABLE", 0)
        a = timeline.span("EXTERNAL_AGENT_WORK", "AGENT", 0, 8, [start])
        b = timeline.span("EXTERNAL_AGENT_WORK", "AGENT", 0, 6, [start])
        result = timeline.span("VIA_RESPONSE", "LOCAL_VIA", 8, 10, [a, b])
        value = timeline.attribute(0, result)
        self.assertEqual(value["wall_clock_seconds"], 10)
        self.assertEqual(value["attributed_seconds"], 2)
        self.assertEqual(value["actual_recomputed_seconds"], 10)

    def test_via_processing_overlapping_user_speech_remains_on_critical_path(self):
        timeline = CausalTimeline()
        start = timeline.point("EVENT_AVAILABLE", 4)
        speech = timeline.span("EXTERNAL_USER_SPEECH", "USER", 4, 6, [start])
        model = timeline.span("VIA_MODEL", "CLOUD_SEMANTIC", 4, 5, [start])
        onset = timeline.span("VOICE_ONSET", "LOCAL_VIA", 6, 6.25, [speech, model])
        value = timeline.attribute(4, onset)
        self.assertEqual(value["wall_clock_seconds"], 2.25)
        self.assertEqual(value["attributed_seconds"], 1.25)

    def test_semaphore_previous_occupant_preserves_queue_without_double_counting(self):
        timeline = CausalTimeline()
        origin = timeline.point("ACTIVITY_START", 0)
        previous = timeline.span("MODEL_NETWORK", "CLOUD_SEMANTIC", 0, 6, [origin])
        enqueue = timeline.point("MODEL_ENQUEUED", 4)
        network = timeline.span("MODEL_NETWORK", "CLOUD_SEMANTIC", 6, 14, [previous, enqueue])
        value = timeline.attribute(4, network)
        self.assertEqual(value["attributed_seconds"], 10)
        self.assertEqual(value["wall_clock_seconds"], 10)
        with self.assertRaises(TimingContractError):
            timeline.span("WRONG_QUEUE_PLUS_NETWORK", "LOCAL_VIA", 4, 14, [previous, enqueue])

    def test_actual_resource_order_survives_zeroing_agent_work(self):
        timeline = CausalTimeline()
        origin = timeline.point("REQUEST_AVAILABLE", 0)
        previous_model = timeline.span("MODEL_NETWORK", "CLOUD_SEMANTIC", 0, 6, [origin])
        work = timeline.span("EXTERNAL_AGENT_WORK", "AGENT", 0, 8, [origin])
        next_model = timeline.span("MODEL_NETWORK", "CLOUD_SEMANTIC", 8, 10, [previous_model, work])
        value = timeline.attribute(0, next_model)
        self.assertEqual(value["wall_clock_seconds"], 10)
        self.assertEqual(value["attributed_seconds"], 8)

    def test_previous_playback_full_span_is_parent_of_next_onset(self):
        timeline = CausalTimeline()
        previous = timeline.span("VOICE_PLAYBACK", "LOCAL_VIA", 3, 9)
        new_model = timeline.span("MODEL_NETWORK", "CLOUD_SEMANTIC", 4, 5)
        onset = timeline.span("VOICE_ONSET", "LOCAL_VIA", 9, 9.25, [previous, new_model])
        value = timeline.attribute(4, onset)
        self.assertEqual(value["attributed_seconds"], 5.25)

    def test_prep_and_scene_origin_resets_keep_absolute_coordinates(self):
        clock = iter([100., 105., 110.])
        timeline = CausalTimeline(clock=lambda: next(clock))
        preparation = timeline.point("PREPARATION", timeline.now())
        ready = timeline.point("SCENE_INPUT_AVAILABLE", timeline.now(), [preparation])
        endpoint = timeline.point("TEXT_PRESENTED", timeline.now(), [ready])
        result = timeline.attribute(105., endpoint)
        self.assertEqual(result["attributed_seconds"], 5)
        self.assertEqual(result["wall_clock_seconds"], 5)
        self.assertEqual(timeline.nodes[preparation]["start_s"], 100.)

    def test_followup_user_wait_speech_only_external_nodes_are_zero(self):
        timeline = CausalTimeline()
        question = timeline.span("QUESTION_PLAYBACK", "LOCAL_VIA", 0, 2)
        wait = timeline.span("EXTERNAL_USER_WAIT", "USER", 2, 10, [question])
        speech = timeline.span("EXTERNAL_USER_SPEECH", "USER", 10, 12, [wait])
        transcription = timeline.span("TRANSCRIPTION_SETTLE", "LOCAL_VIA", 12, 12.1, [speech])
        model = timeline.span("MODEL_NETWORK", "CLOUD_SEMANTIC", 12.1, 14.1, [transcription])
        self.assertAlmostEqual(timeline.attribute(0, model)["attributed_seconds"], 4.1)

    def test_unexplained_scheduler_gap_is_not_assumed_external(self):
        timeline = CausalTimeline()
        work = timeline.span("EXTERNAL_AGENT_WORK", "AGENT", 0, 8)
        response = timeline.span("RESPONSE_CODE", "LOCAL_VIA", 9, 10, [work])
        self.assertEqual(timeline.attribute(0, response)["attributed_seconds"], 2)

    def test_missing_cross_timeline_or_reversed_endpoints_fail_closed(self):
        a, b = CausalTimeline(), CausalTimeline()
        p = a.point("A", 1)
        with self.assertRaises(TimingContractError):
            b.point("B", 2, [p])
        with self.assertRaises(TimingContractError):
            a.attribute(0, "missing")
        with self.assertRaises(TimingContractError):
            a.attribute(2, p)
        with self.assertRaises(TimingContractError):
            a.span("BAD", "LOCAL_VIA", 2, 1)


if __name__ == "__main__":
    unittest.main()
