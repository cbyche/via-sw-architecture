"""Finite checks for comparison accounting; no candidate or provider executes."""
import unittest

from comparison_audit_v1 import compare, endpoint_value


def endpoint(seconds, members=None):
    return {"seconds": seconds, "start_event": "U1", "member_events": members or ["U1"]}


class ComparisonAuditChecks(unittest.TestCase):
    def test_all_four_outcomes_keep_the_planned_denominator(self):
        keys = [("S", 1, str(i)) for i in range(4)]
        left = dict(zip(keys, map(endpoint, [1, 2, None, None])))
        right = dict(zip(keys, map(endpoint, [3, None, 4, None])))
        ledger = []
        p = compare("DP", "POP", "QA03", "A", "B", left, right, ledger)
        self.assertEqual([p[k] for k in ("both_observed", "left_only", "right_only", "neither_observed")], [1, 1, 1, 1])
        self.assertEqual(p["planned"], 4)
        self.assertEqual(p["paired_right_minus_left_seconds"], 2)
        self.assertIsNone(p["whole_population_speed_winner"])
        self.assertIsNone(ledger[-1]["left"]["seconds"])

    def test_unpaired_order_reversal_is_detected(self):
        keys = [("S", 1, str(i)) for i in range(3)]
        left = dict(zip(keys, map(endpoint, [10, 100, None])))
        right = dict(zip(keys, map(endpoint, [11, None, 1])))
        p = compare("DP", "POP", "QA03", "A", "B", left, right, [])
        self.assertTrue(p["unpaired_order_reverses_on_pairing"])
        self.assertEqual(p["paired_right_minus_left_seconds"], 1)

    def test_no_observation_does_not_become_zero_seconds(self):
        rows = {("S", 1, "G"): endpoint(None)}
        p = compare("DP", "POP", "QA03", "A", "B", rows, rows, [])
        self.assertEqual(p["neither_observed"], 1)
        self.assertIsNone(p["paired_left_mean_seconds"])
        self.assertIsNone(p["unpaired_right_minus_left_seconds"])

    def test_goal_members_must_match(self):
        left = {("S", 1, "G"): endpoint(1)}
        right = {("S", 1, "G"): endpoint(2, ["U1", "U2"])}
        with self.assertRaises(AssertionError):
            compare("DP", "POP", "QA03", "A", "B", left, right, [])

    def test_endpoint_status_and_time_must_agree(self):
        self.assertIsNone(endpoint_value({"status": "NOT_OBSERVED", "via_ns": None}, "QA03"))
        with self.assertRaises(AssertionError):
            endpoint_value({"status": "NOT_OBSERVED", "latency_ns": 1}, "QA01")


if __name__ == "__main__":
    unittest.main()
