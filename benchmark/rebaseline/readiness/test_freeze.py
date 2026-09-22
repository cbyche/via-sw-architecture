import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from freeze import authorize_run, candidate_manifest, fingerprint, host_manifest, inventory


class FreezeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "prototype/gate2/src/lib.rs"
        self.source.parent.mkdir(parents=True)
        self.source.write_text("// frozen source\n", encoding="utf-8")
        (self.root / "prototype/gate2/Cargo.lock").write_text("# frozen dependencies\n", encoding="utf-8")
        contract = self.root / "benchmark/rebaseline/gate2/measurement-freeze.json"
        contract.parent.mkdir(parents=True)
        contract.write_text(json.dumps({"evidence_scope_policy": {
            "s2s_synthetic_replay": {"representative_score_eligible": False,
                "final_absolute_product_latency_claim": False},
            "w01_w04_delivery": {"required_sink": "ACTUAL_USER_DELIVERY",
                "when_missing": "BLOCKED_NOT_RUN", "headless_score_eligible": False,
                "not_run_is_zero": False, "imputation_allowed": False}}}), encoding="utf-8")

    def approval(self, manifest):
        return {"approved": True, "purpose": "COMPARATIVE_MEASUREMENT",
                "fingerprint": manifest["fingerprint"],
                "evidence_scope_decisions": manifest["evidence_scope_policy"]}

    def test_consistent_fingerprint(self):
        self.assertEqual(candidate_manifest(self.root), candidate_manifest(self.root))

    def test_default_not_approved(self):
        self.assertFalse(candidate_manifest(self.root)["comparative_measurement_approved"])

    def test_exact_approved_manifest_is_allowed(self):
        manifest = candidate_manifest(self.root)
        authorize_run(manifest, self.approval(manifest), self.root)

    def test_no_approval_no_benchmark(self):
        with self.assertRaises(PermissionError):
            authorize_run(candidate_manifest(self.root), {}, self.root)

    def test_code_change_invalidates_approval(self):
        manifest = candidate_manifest(self.root)
        self.source.write_text("// optimized A only\n", encoding="utf-8")
        with self.assertRaises(PermissionError):
            authorize_run(manifest, self.approval(manifest), self.root)

    def test_new_source_invalidates_approval(self):
        manifest = candidate_manifest(self.root)
        self.source.with_name("new.rs").write_text("// changed structure\n", encoding="utf-8")
        with self.assertRaises(PermissionError):
            authorize_run(manifest, self.approval(manifest), self.root)

    def test_dependency_lock_required(self):
        (self.root / "prototype/gate2/Cargo.lock").unlink()
        manifest = candidate_manifest(self.root)
        with self.assertRaises(PermissionError):
            authorize_run(manifest, self.approval(manifest), self.root)

    def test_evidence_scope_decisions_are_required(self):
        manifest = candidate_manifest(self.root)
        approval = self.approval(manifest)
        del approval["evidence_scope_decisions"]
        with self.assertRaises(PermissionError):
            authorize_run(manifest, approval, self.root)

    def test_weakened_delivery_scope_is_rejected(self):
        manifest = candidate_manifest(self.root)
        approval = self.approval(manifest)
        approval["evidence_scope_decisions"] = json.loads(json.dumps(approval["evidence_scope_decisions"]))
        approval["evidence_scope_decisions"]["w01_w04_delivery"]["headless_score_eligible"] = True
        with self.assertRaises(PermissionError):
            authorize_run(manifest, approval, self.root)

    def test_generated_output_not_source(self):
        base = inventory(self.root)
        target = self.root / "prototype/gate2/target/file.json"
        target.parent.mkdir()
        target.write_text("{}", encoding="utf-8")
        self.assertEqual(base, inventory(self.root))

    def test_model_identity_required(self):
        manifest = candidate_manifest(self.root)
        approval = self.approval(manifest)
        with self.assertRaises(PermissionError):
            authorize_run(manifest, approval, self.root, real_model=True)

    def test_hosted_reference_profile_is_allowed(self):
        manifest = candidate_manifest(self.root)
        approval = {**self.approval(manifest),
                    "model_execution_profile": {"kind":"OPENROUTER_HOSTED_REFERENCE","model_id":"qwen/qwen3-8b",
                    "base_url":"https://openrouter.ai/api","provider":"alibaba","allow_fallbacks":False,
                    "target_pc_absolute_latency_claim":False}}
        authorize_run(manifest, approval, self.root, real_model=True)

    def test_hosted_reference_fallback_is_rejected(self):
        manifest = candidate_manifest(self.root)
        approval = {**self.approval(manifest),
                    "model_execution_profile": {"kind":"OPENROUTER_HOSTED_REFERENCE","model_id":"qwen/qwen3-8b",
                    "base_url":"https://openrouter.ai/api","provider":"alibaba","allow_fallbacks":True,
                    "target_pc_absolute_latency_claim":False}}
        with self.assertRaises(PermissionError):
            authorize_run(manifest, approval, self.root, real_model=True)

    def test_inventory_sort_order_does_not_change_hash(self):
        self.assertEqual(fingerprint({"a":"1", "b":"2"}), fingerprint({"b":"2", "a":"1"}))

    def test_mac_measurement_is_not_windows_measurement(self):
        with patch("freeze.platform.system", return_value="Darwin"), patch("freeze.command", return_value=None):
            manifest = host_manifest(self.root)
        self.assertEqual(manifest["measurement_os"], "Darwin")
        self.assertIsNone(manifest["chip_detected"])
        self.assertNotIn("M5", str(manifest))

    def test_no_serial_or_credentials(self):
        with patch("freeze.command", return_value=None):
            manifest = host_manifest(self.root)
        for forbidden in ("serial_number", "username", "hostname", "api_key", "environment_variables"):
            self.assertNotIn(forbidden, manifest)


if __name__ == "__main__":
    unittest.main()
