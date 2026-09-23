import importlib.util
import json
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONTRACT_PATH = HERE / "w01-foreground-strata.json"
MODULE_PATH = HERE / "w01_observation_adapter.py"

spec = importlib.util.spec_from_file_location("w01_observation_adapter", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

SHA = "a" * 64


class W01ObservationAdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.contract = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))

    def voice_trace(self, case_id="TC-01.1", correct=True):
        return {
            "schema_version": "W01-OBS-v1",
            "candidate_id": "candidate-A",
            "trial_id": "trial-1",
            "case_id": case_id,
            "input_sha256": SHA,
            "model_digest": "model-build",
            "runtime_digest": "runtime-build",
            "modality": "voice",
            "semantic_correct": correct,
            "output_kind": "clarification" if case_id == "TC-06.2" else "response",
            "sink": "HEADLESS_TEST_ONLY",
            "s2s": {
                "kind": "synthetic",
                "sample_id": "sample-1",
                "trace_sha256": SHA,
            },
            "events": [
                {"name": "user_input_end", "at_ns": 100, "clock": "controller-1"},
                {"name": "valid_text_start", "at_ns": 600, "clock": "controller-1"},
                {"name": "valid_audio_start", "at_ns": 700, "clock": "controller-1"},
            ],
        }

    def test_voice_uses_max_of_valid_text_and_audio_start(self):
        result = module.adapt_trace(self.contract, self.voice_trace())
        self.assertEqual(600, result["candidate_observed_latency_ns"])
        self.assertEqual("SIMULATED_E2E_HEADLESS", result["evidence"])
        self.assertFalse(result["representative_metric_eligible"])
        self.assertIsNone(result["score"])

    def test_text_uses_valid_text_start_only(self):
        trace = self.voice_trace("TC-01.4")
        trace["modality"] = "text"
        trace["s2s"] = None
        trace["events"] = trace["events"][:2]
        result = module.adapt_trace(self.contract, trace)
        self.assertEqual(500, result["candidate_observed_latency_ns"])
        self.assertEqual("RAW_ENDPOINT_OBSERVATION_HEADLESS", result["evidence"])

    def test_wrong_semantics_are_censored_from_latency(self):
        result = module.adapt_trace(self.contract, self.voice_trace(correct=False))
        self.assertEqual("CENSORED_CORRECTNESS_FAILURE", result["latency_status"])
        self.assertEqual(600, result["observed_endpoint_ns"])
        self.assertIsNone(result["candidate_observed_latency_ns"])

    def test_candidate_cannot_supply_derived_endpoint(self):
        trace = self.voice_trace()
        trace["events"].append(
            {
                "name": "first_meaningful_output",
                "at_ns": 110,
                "clock": "controller-1",
            }
        )
        with self.assertRaisesRegex(ValueError, "derived"):
            module.adapt_trace(self.contract, trace)

    def test_tc062_requires_clarification_output_kind(self):
        trace = self.voice_trace("TC-06.2")
        trace["output_kind"] = "response"
        with self.assertRaisesRegex(ValueError, "output_kind=clarification"):
            module.adapt_trace(self.contract, trace)

    def test_voice_requires_both_text_and_audio_start(self):
        trace = self.voice_trace()
        trace["events"] = trace["events"][:2]
        with self.assertRaisesRegex(
            ValueError,
            "both valid_text_start and valid_audio_start",
        ):
            module.adapt_trace(self.contract, trace)

    def test_text_and_audio_must_share_monotonic_clock(self):
        trace = self.voice_trace()
        trace["events"][2]["clock"] = "worker-clock"
        with self.assertRaisesRegex(ValueError, "different monotonic clocks"):
            module.adapt_trace(self.contract, trace)

    def test_actual_delivery_is_observed_but_not_self_approved_for_scoring(self):
        trace = self.voice_trace()
        trace["sink"] = "ACTUAL_USER_DELIVERY"
        trace["s2s"]["kind"] = "live"
        result = module.adapt_trace(self.contract, trace)
        self.assertTrue(result["actual_user_delivery_observed"])
        self.assertFalse(result["representative_metric_eligible"])
        self.assertIsNone(result["representative_metric_value_ns"])

    def test_unknown_case_is_rejected(self):
        trace = self.voice_trace()
        trace["case_id"] = "TC-99.9"
        with self.assertRaisesRegex(ValueError, "not a frozen"):
            module.adapt_trace(self.contract, trace)


if __name__ == "__main__":
    unittest.main()
