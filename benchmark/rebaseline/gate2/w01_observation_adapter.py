#!/usr/bin/env python3
"""Normalize one frozen W-01 raw trace into a pre-measurement endpoint sample.

This adapter derives first_meaningful_output from raw delivery events. It never
aggregates p95, computes a score, or declares a representative metric eligible.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
READINESS = HERE.parent / "readiness"
if str(READINESS) not in sys.path:
    sys.path.insert(0, str(READINESS))

from timing_contract import Event, endpoint_sample
from w04_foreground_contract import EXPECTED_CASES, validate_contract

HEX64 = re.compile(r"^[0-9a-f]{64}$")
VALID_SINKS = {"HEADLESS_TEST_ONLY", "ACTUAL_USER_DELIVERY"}


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def require_identity(trace):
    required = (
        "candidate_id",
        "trial_id",
        "case_id",
        "input_sha256",
        "model_digest",
        "runtime_digest",
    )
    for key in required:
        value = trace.get(key)
        if not isinstance(value, str) or not value:
            raise ValueError(f"W-01 trace requires non-empty {key}")
    if not HEX64.fullmatch(trace["input_sha256"]):
        raise ValueError("input_sha256 must be lowercase SHA-256 hex")


def raw_events(trace):
    values = trace.get("events")
    if not isinstance(values, list) or not values:
        raise ValueError("W-01 trace requires raw events")
    events = []
    names = set()
    for row in values:
        if not isinstance(row, dict):
            raise ValueError("W-01 raw event must be an object")
        name = row.get("name")
        if name == "first_meaningful_output":
            raise ValueError(
                "first_meaningful_output is derived by the adapter, not supplied by candidate"
            )
        if name in names:
            raise ValueError(f"duplicate W-01 raw event: {name}")
        names.add(name)
        events.append(Event(name, row.get("at_ns"), row.get("clock")))
    return events


def derive_output_event(modality, events):
    by_name = {event.name: event for event in events}
    if "user_input_end" not in by_name:
        raise ValueError("W-01 trace missing user_input_end")

    if modality == "text":
        if "valid_audio_start" in by_name:
            raise ValueError("Text-only W-01 trace must not contain valid_audio_start")
        try:
            text = by_name["valid_text_start"]
        except KeyError as exc:
            raise ValueError("Text W-01 trace missing valid_text_start") from exc
        return Event("first_meaningful_output", text.at_ns, text.clock)

    if modality != "voice":
        raise ValueError(f"unknown W-01 modality: {modality}")
    try:
        text = by_name["valid_text_start"]
        audio = by_name["valid_audio_start"]
    except KeyError as exc:
        raise ValueError(
            "Voice W-01 trace requires both valid_text_start and valid_audio_start"
        ) from exc
    if text.clock != audio.clock:
        raise ValueError("Text/audio output starts use different monotonic clocks")
    return Event(
        "first_meaningful_output",
        max(text.at_ns, audio.at_ns),
        text.clock,
    )


def adapt_trace(contract, trace):
    validate_contract(contract)
    require_identity(trace)
    if trace.get("schema_version") != "W01-OBS-v1":
        raise ValueError("unexpected W-01 observation trace version")
    if trace["case_id"] not in EXPECTED_CASES:
        raise ValueError(f"case is not a frozen W-01 stratum: {trace['case_id']}")

    case = next(
        row for row in contract["w01"]["cases"] if row["case_id"] == trace["case_id"]
    )
    modality = trace.get("modality")
    if modality != case.get("modality"):
        raise ValueError(
            f"W-01 modality differs from frozen contract for {trace['case_id']}"
        )

    correct = trace.get("semantic_correct")
    if type(correct) is not bool:
        raise ValueError("semantic_correct must be an explicit boolean")

    output_kind = trace.get("output_kind")
    expected_kind = "clarification" if trace["case_id"] == "TC-06.2" else "response"
    if output_kind != expected_kind:
        raise ValueError(f"{trace['case_id']} requires output_kind={expected_kind}")

    sink = trace.get("sink")
    if sink not in VALID_SINKS:
        raise ValueError(
            "W-01 sink must be HEADLESS_TEST_ONLY or ACTUAL_USER_DELIVERY"
        )

    events = raw_events(trace)
    derived = derive_output_event(modality, events)
    s2s = trace.get("s2s")
    if modality == "voice":
        if not isinstance(s2s, dict):
            raise ValueError("Voice W-01 trace requires S2S provenance")
        if not HEX64.fullmatch(str(s2s.get("trace_sha256", ""))):
            raise ValueError("voice S2S trace_sha256 must be lowercase SHA-256 hex")
    elif s2s is not None:
        raise ValueError("Text W-01 trace must not carry S2S provenance")

    sample = endpoint_sample(
        events + [derived],
        "W-01",
        modality,
        s2s,
        sink=sink,
    )
    observed = sample["net_endpoint_ns"]
    sample.update(
        {
            "schema_version": "W01-SAMPLE-v1",
            "candidate_id": trace["candidate_id"],
            "trial_id": trace["trial_id"],
            "case_id": trace["case_id"],
            "input_sha256": trace["input_sha256"],
            "s2s_trace_sha256": s2s["trace_sha256"] if modality == "voice" else None,
            "model_digest": trace["model_digest"],
            "runtime_digest": trace["runtime_digest"],
            "sink": sink,
            "output_kind": output_kind,
            "semantic_correct": correct,
            "observed_endpoint_ns": observed,
            "latency_status": (
                "RECORDED_CORRECT"
                if correct
                else "CENSORED_CORRECTNESS_FAILURE"
            ),
            "candidate_observed_latency_ns": observed if correct else None,
            "actual_user_delivery_observed": sink == "ACTUAL_USER_DELIVERY",
            "representative_metric_eligible": False,
            "representative_metric_value_ns": None,
            "p95": None,
            "score": None,
            "note": (
                "This is a pre-freeze normalized observation. Representative metric "
                "eligibility is decided only by the frozen measurement runner."
            ),
        }
    )
    return sample


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", required=True)
    parser.add_argument("--trace", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()

    result = adapt_trace(load_json(args.contract), load_json(args.trace))
    payload = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        Path(args.output).write_text(payload, encoding="utf-8")
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
