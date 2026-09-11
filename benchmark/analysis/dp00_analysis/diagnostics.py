"""Compose deterministic machine-readable analysis diagnostics."""

from __future__ import annotations

import platform
from dataclasses import asdict
from typing import Any

from . import ANALYSIS_VERSION
from .models import EvidenceSet
from .qa01 import derive_qa01
from .qa02 import derive_qa02
from .qa04 import derive_qa04


def derive_summary(evidence: EvidenceSet) -> dict[str, Any]:
    qa02 = derive_qa02(evidence.episodes, evidence.coverage_map)
    return {
        "analysis_version": ANALYSIS_VERSION,
        "validation": {
            "valid": evidence.validation.valid,
            "errors": [asdict(x) for x in evidence.validation.errors],
            "warnings": [asdict(x) for x in evidence.validation.warnings],
        },
        "qa01": derive_qa01(evidence.episodes),
        "qa02": qa02,
        "qa04": derive_qa04(evidence.episodes, qa02),
        "provenance": {
            "analysis_language": "python", "python_interpreter_version": platform.python_version(),
            "analysis_version": ANALYSIS_VERSION, "raw_root": str(evidence.raw_root),
            "campaign_id": evidence.campaign_provenance.get("campaign_id") if evidence.campaign_provenance else None,
            "result_identity": "OFFICIAL_CAMPAIGN" if evidence.campaign_provenance else "DEVELOPMENT_OR_SYNTHETIC",
            "method_config": {
                "percentiles": ["nearest-rank", "linear-interpolated-(n-1)"],
                "qa04_aggregates": ["overall-episode-mean", "scenario-class-macro-average"],
                "qa04_contract_policy": "qa04-route-contract-policy-v1",
            },
        },
    }
