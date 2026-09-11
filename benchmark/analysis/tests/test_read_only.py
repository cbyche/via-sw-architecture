import hashlib

from dp00_analysis.diagnostics import derive_summary
from dp00_analysis.loader import load_evidence
from dp00_analysis.serialization import read_summary, write_summary


def hashes(root):
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(root.rglob("*")) if p.is_file()}


def test_validate_derive_summary_do_not_modify_raw(raw_run, tmp_path):
    before = hashes(raw_run)
    evidence = load_evidence(raw_run)
    output = write_summary(tmp_path / "derived", derive_summary(evidence))
    assert output.name == "analysis-summary.json"
    assert read_summary(output.parent)["analysis_version"] == "dp00-analysis-v2"
    assert hashes(raw_run) == before
