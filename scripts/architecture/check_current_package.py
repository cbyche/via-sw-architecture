#!/usr/bin/env python3
"""Check current DP package navigation, QA bands and preserved artifact identity.

No model calls, presentation generation or performance measurement. Existing
campaign validators keep their own original, pinned inputs.
"""

from pathlib import Path
import hashlib
import json
import re
import sys
from urllib.parse import unquote, urlsplit
import xml.etree.ElementTree as ET
from zipfile import ZipFile

from check_qa_candidate_package import check_scores
from qa_candidate_population import validate as validate_common28


ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / "docs/architecture/12-decisions/decision-packages/current"
LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def markdown_anchors(path):
    body = path.read_text(encoding="utf-8")
    ids = set(re.findall(r'<(?:a|h[1-6])\s+[^>]*id=["\']([^"\']+)', body))
    counts = {}
    for heading in re.findall(r"^#{1,6}\s+(.*)$", body, re.M):
        heading = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", heading)
        heading = re.sub(r"<[^>]*>", "", heading).replace("`", "").strip().lower()
        slug = re.sub(r"[^\w\-\s]", "", heading).replace(" ", "-")
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        ids.add(slug + (f"-{count}" if count else ""))
    return ids


def within_root(relative):
    path = (ROOT / relative).resolve()
    assert path.is_relative_to(ROOT), f"Path outside repository: {relative}"
    return path


def main():
    manifest = json.loads((PACKAGE / "package-manifest.json").read_text())
    assert manifest["schema_version"] == 1
    assert re.fullmatch(r"[0-9a-f]{40}", manifest["source_commit"])
    sync = manifest["content_sync"]["presentation_vs_common_qa_v3"]
    assert sync in {"PENDING_CONTENT_REVISION", "SYNCHRONIZED_WITH_REVIEW"}
    if sync == "SYNCHRONIZED_WITH_REVIEW":
        assert within_root(manifest["content_sync"]["review_record"]).is_file(), (
            "A synchronized deck needs an explicit content review record"
        )
    files = manifest["imports"]
    destinations = [item["destination"] for item in files]
    assert len(destinations) == len(set(destinations)), "Duplicate current file owner"
    preserved = 0
    editable = 0
    for item in files:
        path = within_root(item["destination"])
        assert path.is_relative_to(PACKAGE), f"Import outside package: {path}"
        assert path.is_file(), f"Missing import: {path}"
        assert len(item["source_sha256"]) == 64
        policy = item["artifact_policy"]
        assert policy in {"IMMUTABLE_SNAPSHOT", "EDITABLE_SOURCE"}
        if policy == "IMMUTABLE_SNAPSHOT":
            assert item["byte_preserved"], f"Snapshot needs an original identity: {path}"
            assert digest(path) == item["source_sha256"], f"Preserved artifact changed: {path}"
            preserved += 1
        else:
            editable += 1
    required = manifest["entrypoints"]
    for name, relative in required.items():
        assert within_root(relative).is_file(), f"Missing {name}: {relative}"
    links = 0
    anchors = 0
    anchor_cache = {}
    for path in PACKAGE.rglob("*.md"):
        body = path.read_text(encoding="utf-8")
        for raw in LINK.findall(body):
            ref = raw.strip().strip("<>")
            if not ref or urlsplit(ref).scheme:
                continue
            part, _, fragment = ref.partition("#")
            part = unquote(part.split("?", 1)[0])
            target = (path.parent / part).resolve() if part else path
            assert target.exists(), f"Broken link {path.relative_to(ROOT)}: {ref}"
            links += 1
            if fragment and target.suffix == ".md":
                if target not in anchor_cache:
                    anchor_cache[target] = markdown_anchors(target)
                assert unquote(fragment) in anchor_cache[target], (
                    f"Broken anchor {path.relative_to(ROOT)}: {ref}"
                )
                anchors += 1
    qa = PACKAGE / "qa"
    bands = 0
    for number in range(1, 7):
        candidates = list(qa.glob(f"qa{number:02d}-*.md"))
        assert len(candidates) == 1, f"QA{number:02d} needs exactly one editable definition"
        if number in (1, 3):
            body = candidates[0].read_text()
            version = "QA01-VOICE-v2" if number == 1 else "QA03-COMPLETION-v1"
            score_version = "QA01-VOICE-score-v1" if number == 1 else "QA03-COMPLETION-score-v1"
            assert version in body and score_version in body
            assert "USER_AGREED_2026-10-23" in body
            assert "TARGET_PENDING_REVIEW" not in body and "SCORE_BANDS_PENDING" not in body
            assert "timing-measurement-contract.md" in body
            if number == 1:
                assert all(term in body for term in ("NORMAL", "WAIT_USER", "WAIT_PREVIOUS", "WAIT_BOTH", "N_NORMAL"))
        bands += check_scores(candidates[0])
    meaning = (qa / "qa07-request-meaning-accuracy.md").read_text()
    assert any(
        term in meaning for term in ("미정", "미확정", "미합의")
    ), "QA07 must retain unagreed adoption/priority"
    assert "QA07-REQUEST-RESPONSE-v2" in meaning
    assert "TARGET_PENDING_REVIEW" in meaning and "SCORE_BANDS_PENDING" in meaning
    assert all(term in meaning for term in ("REQUEST_INTERPRETATION", "RESPONSE_COMPOSITION", "ACTUAL_DELIVERY"))
    assert not re.search(r"^\|\s*[0-6]\s*\|", meaning, re.M), "Expanded QA07 must not inherit prior bands"
    assert "제안" in meaning and "미측정" in meaning
    assert not list(qa.glob("qa09-*.md")), "QA09 is no longer a standalone active QA"
    timing = (qa / "timing-measurement-contract.md").read_text()
    assert "01a15dfe-8b3c-7fe3-87ab-750cd0e2db99" in timing
    assert "VIA-TIMING-CONTRACT-v2" in timing
    assert "USER_AGREED_2026-10-23" in timing
    assert re.search(r"\*\*(?:READY_TO_SEND|SENT) /", timing), "Approved handoff status required"
    for path in qa.glob("qa08-*.md"):
        bands += check_scores(path)
    common28 = within_root(required["common28_dataset"])
    counts = validate_common28(json.loads(common28.read_text()))
    deck = within_root(required["presentation"])
    with ZipFile(deck) as package:
        assert package.testzip() is None, "Damaged PPTX archive"
        root = ET.fromstring(package.read("ppt/presentation.xml"))
        ns = {"p": "http://schemas.openxmlformats.org/presentationml/2006/main"}
        slide_count = len(root.findall("p:sldIdLst/p:sldId", ns))
        assert slide_count == 36, f"Expected preserved v10 with 36 slides, found {slide_count}"
    assert len(list((PACKAGE / "presentation/previews").glob("slide-*.png"))) == 36
    pairs = 0
    for directory in ("dp02", "dp03", "dp05", "dp44", "design-review"):
        for drawio in (PACKAGE / directory).rglob("*.drawio"):
            assert drawio.with_suffix(".svg").is_file(), f"Missing SVG for {drawio}"
            ET.parse(drawio)
            ET.parse(drawio.with_suffix(".svg"))
            pairs += 1
    report = {
        "status": "PASS_WITH_RECORDED_CONTENT_SYNC_GAP" if sync == "PENDING_CONTENT_REVISION" else "PASS",
        "imports": len(files), "immutable_snapshots": preserved,
        "editable_sources": editable,
        "markdown_links": links, "markdown_anchors": anchors, "qa_score_probes": bands,
        "common28_scenarios": counts["scenarios"], "pptx_slides": slide_count,
        "diagram_pairs": pairs,
        "presentation_vs_common_qa_v3": manifest["content_sync"]["presentation_vs_common_qa_v3"],
        "measurement_performed": False,
    }
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, ValueError, KeyError, ET.ParseError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        raise SystemExit(1)
