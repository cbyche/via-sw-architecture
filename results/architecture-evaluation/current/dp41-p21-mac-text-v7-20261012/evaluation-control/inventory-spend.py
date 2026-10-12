"""Inventory actual provider records, separating measurement and preparation."""
from collections import Counter, defaultdict
import csv
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path

CAMPAIGN = Path(__file__).resolve().parents[1]
CURRENT = CAMPAIGN.parent

def bucket(path):
    relative = path.relative_to(CURRENT)
    if relative.parts[0] == CAMPAIGN.name:
        if relative.parts[1] == "workers": return "FORMAL_VIA"
        if relative.parts[1] == "evaluation": return "FORMAL_JUDGE"
        if relative.parts[1] == "changes-semantic": return "CHANGE_SEMANTIC"
        if relative.parts[1] == "preflight": return "CURRENT_PREFLIGHT"
        if "service-health" in relative.parts: return "SERVICE_HEALTH"
        return "CURRENT_OTHER_ACTUAL_API"
    if "service-health" in relative.parts or any("identical-recheck" in p for p in relative.parts):
        return "SERVICE_HEALTH"
    if relative.parts[0].startswith("dp41-p21-mac-text-v"):
        return "EXCLUDED_GENERATION"
    return "PREPARATION"

def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()

def run():
    completion = json.loads((CAMPAIGN / "final/completion.json").read_text())
    assert completion["status"] == "COMPLETE" and completion["observed_trials"] == 210
    final = CAMPAIGN / "final"
    destination = final / "spend-inventory.json"
    if destination.exists(): raise FileExistsError(destination)
    unique, source_files, conflicts = {}, [], []
    for root in sorted(CURRENT.glob("dp41-*")):
        for path in sorted(root.rglob("model-calls.jsonl")):
            data = path.read_bytes()
            source_files.append({"ref": str(path.relative_to(CURRENT)), "sha256": hashlib.sha256(data).hexdigest()})
            for line_number, line in enumerate(data.decode().splitlines(), 1):
                if not line.strip(): continue
                row = json.loads(line)
                if row.get("provider") not in ("qwen", "openai") or row.get("evidence_level") != "MEASURED_MODEL":
                    continue
                call_id = row["call_id"]
                proof = {"ref": str(path.relative_to(CURRENT)), "line": line_number, "row_sha256": hashlib.sha256(canonical(row)).hexdigest()}
                if call_id in unique:
                    old = unique[call_id]["row"]
                    if old["provider"] != row["provider"] or (old.get("cost_usd") is not None and row.get("cost_usd") is not None
                            and abs(old["cost_usd"] - row["cost_usd"]) > 1e-9):
                        conflicts.append(call_id)
                    unique[call_id]["proof"].append(proof)
                    if old.get("cost_usd") is None and row.get("cost_usd") is not None:
                        unique[call_id]["row"] = row
                else:
                    unique[call_id] = {"row": row, "proof": [proof], "bucket": bucket(path)}
    groups, csv_records = {}, []
    for call_id, item in unique.items():
        row = item["row"]
        key = (item["bucket"], row["provider"])
        group = groups.setdefault(key, {"bucket": key[0], "provider": key[1], "calls": 0,
            "known_usage_price_usd": 0., "unknown_usage_calls": [], "input_tokens": 0, "output_tokens": 0,
            "cached_input_tokens": 0, "retry_attempts": 0, "discarded_returns": 0})
        group["calls"] += 1
        if row.get("cost_usd") is None or row.get("usage") is None:
            group["unknown_usage_calls"].append(call_id)
        else:
            group["known_usage_price_usd"] += row["cost_usd"]
            usage = row["usage"]
            # The earliest connectivity probe preserves the provider's wire
            # schema; subsequent runners preserve the normalized schema.
            input_tokens = usage.get("input_tokens", usage.get("prompt_tokens"))
            output_tokens = usage.get("output_tokens", usage.get("completion_tokens"))
            if not isinstance(input_tokens, int) or not isinstance(output_tokens, int):
                raise ValueError("Known-cost record has no recognized token schema")
            group["input_tokens"] += input_tokens
            group["output_tokens"] += output_tokens
            details = usage.get("input_tokens_details", usage.get("prompt_tokens_details", {}))
            group["cached_input_tokens"] += details.get("cached_tokens", 0)
        group["retry_attempts"] += row.get("attempt", 1) > 1
        group["discarded_returns"] += bool(row.get("discard_reason"))
        csv_records.append({"call_id": call_id, "bucket": item["bucket"],
            **{k: row.get(k) for k in ("provider", "model", "role", "purpose", "variant", "repeat", "case_id", "status", "http_status", "cost_usd", "usage", "discard_reason", "usage_state", "invoice_state")},
            "raw_evidence": item["proof"]})
    budgets = []
    for provider in ("qwen", "openai"):
        path = CURRENT / "dp41-p21-mac-20261012-preparation/global-budgets" / (provider + ".json")
        value = json.loads(path.read_text())
        unmatched = {call: amount for call, amount in value["calls"].items() if call not in unique}
        budgets.append({"provider": provider, "limit_usd": value["limit_usd"], "reserved_or_settled_budget_total_usd": sum(value["calls"].values()),
            "terminal_records_not_observed": unmatched,
            "budget_is_not_invoice_or_measured_spend": True, "ref": str(path.relative_to(CURRENT))})
    formal = groups[("FORMAL_VIA", "qwen")]
    assert formal["calls"] == 824 and not formal["unknown_usage_calls"]
    assert not conflicts
    result = {"status": "COMPLETE", "checked_utc": datetime.now(timezone.utc).isoformat(),
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "unique_actual_call_records": len(unique), "duplicate_record_conflicts": conflicts,
        "groups": list(groups.values()), "provider_budget_snapshots": budgets, "source_files": source_files,
        "known_standard_usage_price_total_usd": {p: sum(g["known_usage_price_usd"] for g in groups.values() if g["provider"] == p) for p in ("qwen", "openai")},
        "account_invoice_reconciled": False, "audio_api_calls": 0,
        "cash_free_quota_adjustment": "NOT_ASSUMED_FROM_THIS_RUN; ACCOUNT_SHARED_QUOTA_AND_INVOICE_NOT_USED_FOR_COMPARISON",
        "unknown_usage_policy": "Keep excluded preparation unknowns and interrupted-budget reservations explicit; do not assume text errors were free. All formal VIA calls have known usage.",
    }
    destination.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
    with (final / "all-actual-api-calls.csv").open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(csv_records[0]))
        writer.writeheader()
        for record in csv_records:
            writer.writerow({k: json.dumps(v, ensure_ascii=False) if isinstance(v, (list, dict)) else v for k, v in record.items()})
    print(json.dumps({"status": "COMPLETE", "calls": len(unique), "known_usage_price_usd": result["known_standard_usage_price_total_usd"],
        "unknown_terminal_usage_records": sum(len(g["unknown_usage_calls"]) for g in groups.values())}, ensure_ascii=False))

if __name__ == "__main__":
    run()
