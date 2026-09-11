"""DP-00 immutable raw-evidence analysis."""

ANALYSIS_VERSION = "dp00-analysis-v4"

from .loader import load_evidence
from .diagnostics import derive_summary

__all__ = ["ANALYSIS_VERSION", "derive_summary", "load_evidence"]
