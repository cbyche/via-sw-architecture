#!/usr/bin/env python3
"""Check the synthetic DP46 time-evidence examples, without performing semantic inference.

This is a document checker, not a VIA implementation, provider capability test,
measurement harness, or frozen validation contract. No raw image/audio is loaded.
"""

from __future__ import annotations

import argparse
import copy
import json
import math
from pathlib import Path


DEFAULT = (Path(__file__).resolve().parents[2] / "docs/architecture/12-decisions/"
           "decision-packages/contracts/dp46-temporal-evidence-examples.json")
KINDS = {"ACOUSTIC_OBSERVATION_OVERLAP", "OBSERVATION_BEFORE", "POINTER_PATH"}
FORBIDDEN_KEYS = {"final_referent", "intended_table", "intended_task", "task_match",
                  "semantic_answer", "execution_authorized", "executable_expression"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def number(value: object) -> bool:
    return (isinstance(value, (int, float)) and not isinstance(value, bool)
            and math.isfinite(value))


def interval(value: object, name: str) -> list:
    require(isinstance(value, list) and len(value) == 2, f"{name}: two endpoints required")
    require(all(number(v) for v in value) and value[0] < value[1],
            f"{name}: finite ascending half-open interval required")
    return value


def lookup(data: dict, ref: str) -> object:
    value = data
    for part in ref.split("."):
        require(isinstance(value, dict) and part in value, f"unknown reference: {ref}")
        value = value[part]
    return value


def no_semantic_answers(value: object) -> None:
    if isinstance(value, dict):
        require(not (FORBIDDEN_KEYS & value.keys()), "semantic answer/executable expression in evidence")
        for child in value.values():
            no_semantic_answers(child)
    elif isinstance(value, list):
        for child in value:
            no_semantic_answers(child)


def overlaps(span: dict, observation: dict) -> str | None:
    """Finite interval evidence only, preserving declared uncertainty.

    Expanded intervals determine POSSIBLE; endpoint bounds determine DEFINITE.
    The errors bound timestamp uncertainty; they do not establish intended meaning.
    """
    require(span["local_epoch"] == observation["local_epoch"], "cannot overlap different clock epochs")
    a, b = span["interval_ms"], observation["capture_interval_ms"]
    ea, eb = span["error_ms"], observation["error_ms"]
    if not (a[0] - ea < b[1] + eb and b[0] - eb < a[1] + ea):
        return None
    if a[0] + ea < b[1] - eb and b[0] + eb < a[1] - ea:
        return "DEFINITE"
    return "POSSIBLE"


def expected_edges(spans: list, observations: list) -> list:
    edges = []
    for span in spans:
        for obs in observations:
            classification = overlaps(span, obs)
            if classification:
                edges.append({"kind": "ACOUSTIC_OBSERVATION_OVERLAP",
                              "span_ref": span["ref"], "observation_ref": obs["ref"],
                              "classification": classification})
    return edges


def fence_status(data: dict, case: dict) -> str:
    artifact = lookup(data, case["artifact"])
    at = case["at_ms"]
    expiry = (artifact["query_lease"]["expires_ms"] if "query_lease" in artifact
              else artifact["view_expires_ms"])
    if at >= min(expiry, data["window"]["raw_expires_ms"]):
        return "EXPIRED"
    if case["current_policy_epoch"] != artifact["policy_epoch"]:
        return "DENIED"
    if (case["current_local_epoch"] != artifact["local_epoch"]
            or lookup(data, case["current_source_vector_ref"])
            != lookup(data, artifact["source_vector_ref"])):
        return "DIRTY"
    return "VALID"


def availability_status(data: dict, case: dict) -> str:
    # The individual fixtures isolate causes. This precedence is an example,
    # not a universal runtime policy for simultaneous errors.
    if case["requested_kind"] not in data["relation_schema"]["supported_kinds"]:
        return "UNSUPPORTED_RELATION"
    if not case["raw_available"]:
        return "SOURCE_GAP"
    if case["requested_kind"] == "ACOUSTIC_OBSERVATION_OVERLAP" and not case["timing_available"]:
        return "UNKNOWN_TIMING"
    if not case["range_published"]:
        return "NOT_COVERED"
    return "VALID"


def validate(data: dict) -> dict:
    require(data["contract_status"] == "DESIGN_REVIEW_ILLUSTRATION_NOT_IMPLEMENTED_NOT_MEASURED_NOT_FROZEN",
            "example status must preserve implementation/measurement/freeze limits")
    require(data["illustration"]["synthetic"] is True, "synthetic illustration required")
    require(data["illustration"]["provider_guarantee"] is False, "provider timing must not be guaranteed")
    require(data["schema_version"] == 1, "unsupported example schema")
    require(set(data["relation_schema"]["supported_kinds"]) == KINDS, "finite supported kinds changed")
    no_semantic_answers(data)
    inp, window, mapping = data["input"], data["window"], data["clock_mapping"]
    require(inp["input_ref"] == "u46" and inp["window_ref"] == window["ref"] == "W46",
            "same u46/W46 example required")
    require(inp["local_vad"]["word_timing_produced"] is False, "VAD cannot supply word timing")
    bounds = interval(window["interval_ms"], "window")
    require(bounds == [inp["local_vad"]["onset_ms"], inp["local_vad"]["end_ms"]], "VAD/window mismatch")
    require(inp["transcript_received_ms"] > bounds[1], "fixture must separate end from transcript receipt")
    require(window["raw_expires_ms"] > bounds[1] and window["pin_extends_raw_expiry"] is False,
            "raw expiry must be finite and independent of pin")
    require(mapping["local_epoch"] == window["local_epoch"], "window/mapping epoch mismatch")
    require(mapping["original_sample_rate_hz"] == 48000 and mapping["provider_sample_rate_hz"] == 24000
            and mapping["original_to_provider_sample_ratio"] == 2, "illustrative resampling mapping mismatch")
    require(number(mapping["sample_mapping_error_ms"]) and mapping["sample_mapping_error_ms"] >= 0,
            "nonnegative mapping error required")
    window_samples = interval(window["original_sample_range"], "window samples")
    require([mapping["local_time_origin_ms"] + (n - mapping["original_sample_origin"]) * 1000
             / mapping["original_sample_rate_hz"] for n in window_samples] == bounds,
            "sample window differs from local VAD interval")

    def check_span(span: dict, revision: int) -> None:
        text_range = interval(span["text_span"], "text span")
        require(all(type(x) is int for x in text_range), "text span needs integer codepoint offsets")
        require(0 <= text_range[0] < text_range[1] <= len(inp["text"]), "text span outside text")
        require(inp["text"][text_range[0]:text_range[1]] == span["text"], "acoustic/text span mismatch")
        require(span["input_ref"] == inp["input_ref"] and span["input_revision"] == revision,
                "span input revision mismatch")
        samples = interval(span["original_sample_range"], "original samples")
        require(all(type(x) is int for x in samples), "sample offsets must be integers")
        require(window["original_sample_range"][0] <= samples[0] < samples[1]
                <= window["original_sample_range"][1], "sample range outside retained window")
        computed = [mapping["local_time_origin_ms"] +
                    (n - mapping["original_sample_origin"]) * 1000 / mapping["original_sample_rate_hz"]
                    for n in samples]
        require(computed == span["interval_ms"], "sample to capture-time mapping mismatch")
        require([n / mapping["original_to_provider_sample_ratio"] for n in samples]
                == span["provider_sample_range"], "provider/original sample mapping mismatch")
        require(span["clock_mapping_ref"] == mapping["ref"] and span["mapping_epoch"] == mapping["mapping_epoch"]
                and span["local_epoch"] == mapping["local_epoch"], "span clock epoch/mapping mismatch")
        require(span["error_ms"] == mapping["sample_mapping_error_ms"], "span mapping uncertainty lost")

    spans = data["acoustic_spans"]
    require([s["ref"] for s in spans] == ["a", "b"], "a/b acoustic spans required")
    for span in spans:
        check_span(span, 1)
    observations = data["observations"]
    require([o["ref"] for o in observations] == ["S1", "P1", "S2", "P2"], "same S1/P1,S2/P2 raw sources required")
    for obs in observations:
        capture = interval(obs["capture_interval_ms"], obs["ref"])
        require(bounds[0] <= capture[0] < capture[1] <= bounds[1], "capture outside W46")
        require(number(obs["error_ms"]) and obs["error_ms"] >= 0, "capture uncertainty missing")
        require(obs["local_epoch"] == window["local_epoch"] and obs["window_ref"] == "W46", "source clock/window mismatch")
        require(obs["received_ms"] >= capture[1], "receipt precedes capture completion")
        require(obs["raw_locator"].startswith("fixture://W46/"), "original source reference required")
        require(obs["coordinate_system"] == "VIEWPORT_PIXELS", "coordinate system required")
        require(len(obs["viewport_rect_px"]) == 4 and len(obs["document_scroll_px"]) == 2,
                "viewport/scroll geometry missing")
        if obs["kind"] == "POINTER_PATH":
            points = obs["points"]
            require(len(points) > 1 and [p["t_ms"] for p in points] == sorted(p["t_ms"] for p in points),
                    "recorded path must preserve time order")
            for p in points:
                require(capture[0] <= p["t_ms"] < capture[1], "point time outside path interval")
                x, y = p["xy_px"]
                vx, vy, width, height = obs["viewport_rect_px"]
                require(number(x) and number(y) and vx <= x < vx + width and vy <= y < vy + height,
                        "point outside declared coordinate space")
    for screen, path in ((observations[0], observations[1]), (observations[2], observations[3])):
        for key in ("display_ref", "app_window_ref", "document_ref", "viewport_ref", "coordinate_space_ref",
                    "coordinate_system", "viewport_rect_px", "document_scroll_px"):
            require(screen[key] == path[key], f"screen/path {key} mismatch")
    require(observations[0]["coordinate_space_ref"] != observations[2]["coordinate_space_ref"],
            "two viewports must not silently share a coordinate frame")
    expected_vector = {"u46": 1, "W46": window["revision"], mapping["ref"]: mapping["revision"]}
    expected_vector.update({o["ref"]: o["revision"] for o in observations})
    require(data["source_vector_r1"] == expected_vector, "r1 source vector omits or misversions a source")
    require(data["relations_r1"]["candidate_edges"] == expected_edges(spans, observations),
            "r1 interval candidates/uncertainty mismatch")
    a, b = data["A"]["result"], data["B"]["view"]
    require(a["type"] == "TemporalQueryResult" and b["type"] == "TemporalView", "A query versus B publication required")
    require(a["source_vector_ref"] == b["source_vector_ref"] == "source_vector_r1"
            and a["relations_ref"] == b["relations_ref"] == "relations_r1", "A/B must consume equal sources/candidates")
    require(a["query_lease"]["expires_ms"] != window["raw_expires_ms"], "query lease is not raw lifetime")
    require(a["constructed_at_ms"] >= inp["transcript_received_ms"]
            and b["published_at_ms"] >= inp["transcript_received_ms"], "final-revision evidence before arrival")
    require(data["A"]["query"]["requested_kinds"] == b["coverage"]["kinds"], "A/B requested coverage differs")
    r2 = data["r2"]
    require(r2["text_unchanged"] is True and r2["input_revision"] == 2, "r2 alignment revision required")
    check_span(r2["corrected_span"], 2)
    revised = [dict(spans[0], input_revision=2), r2["corrected_span"]]
    require(r2["source_vector"] == dict(expected_vector, u46=2), "r2 must preserve unchanged source refs")
    require(r2["relations"]["candidate_edges"] == expected_edges(revised, observations), "r2 re-alignment candidates mismatch")
    require(r2["relations"]["candidate_edges"] != data["relations_r1"]["candidate_edges"], "r2 must change actual time evidence")

    for relation_set in (data["relations_r1"], r2["relations"]):
        by_ref = {o["ref"]: o for o in observations}
        for before in relation_set["before"]:
            require(before["kind"] == "OBSERVATION_BEFORE", "unknown finite relation")
            left, right = by_ref[before["left_ref"]], by_ref[before["right_ref"]]
            require(left["capture_interval_ms"][1] + left["error_ms"]
                    <= right["capture_interval_ms"][0] - right["error_ms"], "uncertain relation asserted as before")
        for path in relation_set["paths"]:
            obs = by_ref[path["observation_ref"]]
            require(path["kind"] == "POINTER_PATH" and obs["kind"] == "POINTER_PATH"
                    and path["coordinate_space_ref"] == obs["coordinate_space_ref"]
                    and path["point_count"] == len(obs["points"]), "path source/coordinate mismatch")
    for view in (b, r2["B_republication"]):
        coverage = view["coverage"]
        require(view["producer"]["module"] == "Temporal Evidence Publisher" and view["schema_version"] == 1,
                "published producer/schema required")
        require(coverage["atomic_with_relations"] is True and coverage["source_vector_ref"] == view["source_vector_ref"]
                and coverage["input_revision"] == view["input_revision"] and coverage["window_ref"] == "W46"
                and coverage["span_refs"] == ["a", "b"] and coverage["interval_ms"] == bounds
                and set(coverage["kinds"]) == KINDS, "relation/coverage/source vector not atomically paired")
    require(r2["A_requery"]["source_vector_ref"] == r2["B_republication"]["source_vector_ref"] == "r2.source_vector"
            and r2["A_requery"]["relations_ref"] == r2["B_republication"]["relations_ref"] == "r2.relations",
            "r2 A/B outputs differ in source truth")
    require(set(r2["A_requery"]) == set(a) and set(r2["B_republication"]) == set(b),
            "r2 outputs must retain the same concrete schema fields as r1")
    require(r2["A_query"]["ref"] == r2["A_requery"]["query_ref"] == "Q2"
            and r2["A_query"]["input_revision"] == 2 and r2["A_query"]["window_ref"] == "W46",
            "r2 requery must have a current bounded query")
    for case in data["usage_fence_cases"]:
        require(case["use_boundary"] in {"CLOUD_PROVIDING", "MEANING_ADOPTION", "AGENT_PROVIDING", "RESPONSE_PUBLICATION"}
                and case["artifact_is_dependency_not_execution_permission"] is True,
                "use boundary must recheck evidence dependencies, not grant execution permission")
        require(fence_status(data, case) == case["expected"], f"usage fence mismatch: {case['name']}")
        if "retention_pin_expires_ms" in case:
            artifact = lookup(data, case["artifact"])
            require(case["at_ms"] == window["raw_expires_ms"] < case["retention_pin_expires_ms"]
                    and case["at_ms"] < artifact["view_expires_ms"] and case["expected"] == "EXPIRED",
                    "raw expiry must reject use despite a live pin/view deadline")
    for case in data["publication_cases"]:
        matches = (lookup(data, case["expected_source_vector_ref"]) == lookup(data, case["current_source_vector_ref"])
                   and case["expected_generation"] == case["current_generation"])
        require(case["expected"] == ("PUBLISHED" if matches else "REJECTED_STALE"), "stale production accepted")
    for case in data["availability_cases"]:
        requested = interval(case["requested_interval_ms"], "requested coverage")
        published = interval(case["published_interval_ms"], "published coverage")
        require(set(case["published_kinds"]) <= KINDS, "publication claims an unsupported kind")
        covered = (case["requested_kind"] in case["published_kinds"] and
                   published[0] <= requested[0] < requested[1] <= published[1])
        require(case["range_published"] == covered, "coverage kind/range claim differs from metadata")
        require(availability_status(data, case) == case["expected"], f"availability mismatch: {case['name']}")
        if case["expected"] == "UNKNOWN_TIMING":
            require(case["invent_word_times"] is False, "missing word timing must remain unknown")
        if case["expected"] == "SOURCE_GAP":
            require(case["fresh_capture_replaces_lost_raw"] is False, "fresh capture cannot replace historical source")
    unknown = data["no_timing_variant"]
    require(unknown["status"] == "UNKNOWN_TIMING" and unknown["provider_acoustic_spans"] is None
            and unknown["acoustic_word_intervals_ms"] is None
            and unknown["constructed_from_transcript_receipt_or_vad"] is False
            and unknown["whole_utterance_interval_ms"] == bounds
            and unknown["observation_candidate_refs"] == [o["ref"] for o in observations],
            "no-timing variant must preserve whole-utterance evidence without fabricating word times")
    return {"source_refs": len(expected_vector), "r1_candidates": len(data["relations_r1"]["candidate_edges"]),
            "r2_candidates": len(r2["relations"]["candidate_edges"]),
            "usage_fences": len(data["usage_fence_cases"]), "publication_cases": len(data["publication_cases"]),
            "availability_cases": len(data["availability_cases"])}


def negative_checks(data: dict) -> int:
    mutations = [
        ("invented acoustic timestamp", lambda d: d["acoustic_spans"][0].update(interval_ms=[2400, 2800])),
        ("clock epoch mismatch", lambda d: d["observations"][0].update(local_epoch="PCBoot2")),
        ("wrong text span", lambda d: d["acoustic_spans"][1].update(text_span=[4, 8])),
        ("wrong coordinate space", lambda d: d["observations"][1].update(coordinate_space_ref="SpaceB2")),
        ("coverage wrong source vector", lambda d: d["B"]["view"]["coverage"].update(source_vector_ref="r2.source_vector")),
        ("old query accepted after revision", lambda d: d["usage_fence_cases"][2].update(expected="VALID")),
        ("stale producer accepted", lambda d: d["publication_cases"][0].update(expected="PUBLISHED")),
        ("unknown timing treated as coverage miss", lambda d: next(c for c in d["availability_cases"]
            if c["name"] == "provider_has_no_acoustic_span_timing").update(expected="NOT_COVERED")),
        ("fabricated word timing in no-timing variant", lambda d: d["no_timing_variant"].update(acoustic_word_intervals_ms=[2000,2800])),
        ("semantic referent smuggled into evidence", lambda d: d["relations_r1"].update(final_referent="Table1")),
    ]
    for label, mutate in mutations:
        invalid = copy.deepcopy(data)
        mutate(invalid)
        try:
            validate(invalid)
        except (ValueError, KeyError, TypeError):
            continue
        raise ValueError(f"negative check was accepted: {label}")
    return len(mutations)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", nargs="?", type=Path, default=DEFAULT)
    parser.add_argument("--skip-negative-checks", action="store_true")
    args = parser.parse_args()
    try:
        data = json.loads(args.path.read_text(encoding="utf-8"))
        summary = validate(data)
        if not args.skip_negative_checks:
            summary["malformed_examples_rejected"] = negative_checks(data)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f"FAIL: {error}\n")
    print("PASS: DP46 synthetic temporal examples " + json.dumps(summary, sort_keys=True))
    print("Static document consistency only; no provider timing guarantee, semantic oracle, implementation or measured quality.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
