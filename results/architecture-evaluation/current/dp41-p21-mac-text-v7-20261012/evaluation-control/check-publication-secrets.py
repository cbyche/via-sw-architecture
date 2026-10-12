"""Check intended evidence for plaintext API credentials without printing values."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re

CAMPAIGN = Path(__file__).resolve().parents[1]
ROOT = CAMPAIGN.parents[3]
PATTERNS = [re.compile(r"sk-(?:proj-)?[A-Za-z0-9_-]{24,}"),
            re.compile(r"Bearer\s+[A-Za-z0-9._-]{24,}"),
            re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----")]
OMIT_ENCRYPTED_KEYS = {"encrypted_content"}


def inspect(value, path, field, findings):
    if isinstance(value, dict):
        for key, child in value.items():
            if key not in OMIT_ENCRYPTED_KEYS:
                inspect(child, path, f"{field}.{key}", findings)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            inspect(child, path, f"{field}[{index}]", findings)
    elif isinstance(value, str) and any(pattern.search(value) for pattern in PATTERNS):
        findings.append({"path": str(path.relative_to(ROOT)), "field": field})


def run():
    paths = set()
    for folder in (ROOT / "benchmark/architecture/dp41_45_poc",
                   ROOT / "prototypes/candidates/dp41_45_poc"):
        paths.update(path for path in folder.rglob("*") if path.is_file())
    for folder in (ROOT / "results/architecture-evaluation/current").glob("dp41-*"):
        paths.update(path for path in folder.rglob("*") if path.is_file())
    checked, findings = 0, []
    for path in sorted(paths):
        if path.suffix not in {".json", ".jsonl", ".py", ".md", ".csv", ".log"}:
            continue
        checked += 1
        if path.suffix == ".json":
            inspect(json.loads(path.read_text()), path, "$", findings)
        elif path.suffix == ".jsonl":
            for number, line in enumerate(path.open(), 1):
                if line.strip():
                    inspect(json.loads(line), path, f"line:{number}", findings)
        else:
            for number, line in enumerate(path.open(), 1):
                if any(pattern.search(line) for pattern in PATTERNS):
                    findings.append({"path": str(path.relative_to(ROOT)), "field": f"line:{number}"})
    report = {"status": "PASS" if not findings else "FAIL", "utc": datetime.now(timezone.utc).isoformat(),
              "checked_files": checked, "plaintext_credential_findings": findings,
              "encrypted_reasoning_not_decoded_or_reported": True,
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    output = CAMPAIGN / "final/publication-secret-check.json"
    if output.exists():
        raise FileExistsError(output)
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "checked_files": checked, "finding_count": len(findings)}))
    if findings:
        raise SystemExit(1)


if __name__ == "__main__":
    run()
