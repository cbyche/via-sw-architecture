"""Read-only independent ledger/OS-sample audit; never executes candidate or API."""
from __future__ import annotations

from collections import Counter, defaultdict
from datetime import datetime, timezone
from decimal import Decimal
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import statistics

CAMPAIGN = Path(__file__).resolve().parents[1]
OUT = Path(__file__).resolve().parent
MIB = 1048576
EXPECTED_DIGEST = "44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce"
checks = []
inventory = {}
findings = []


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def track(path, data, rows=None):
    inventory[str(path)] = {"sha256": sha(data), "bytes": len(data), "rows": rows}


def load(path, required=True, mutable=False):
    if not path.is_file():
        if required:
            check("REQUIRED_FILE", False, str(path))
        return []
    data = path.read_bytes()
    # A live evaluation snapshot is allowed to end between JSONL writes.
    committed = data if data.endswith(b"\n") else data.rsplit(b"\n", 1)[0] + b"\n"
    if not mutable:
        check("TERMINAL_NEWLINE", data.endswith(b"\n"), str(path))
        committed = data
    rows = []
    for number, line in enumerate(committed.splitlines(), 1):
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except (ValueError, TypeError) as error:
            check("JSONL_PARSE", False, f"{path}:{number}: {error}")
    track(path, data, len(rows))
    if mutable:
        inventory[str(path)]["append_only_snapshot"] = True
        inventory[str(path)]["committed_bytes"] = len(committed)
    return rows


def check(name, passed, evidence):
    checks.append({"check": name, "passed": bool(passed), "evidence": evidence})


def counter(rows, key):
    return dict(sorted(Counter(str(r.get(key)) for r in rows).items()))


def money(call):
    u, p = call.get("usage"), call.get("price", {})
    if u is None:
        return None
    d = u.get("input_tokens_details", {})
    cache, write = int(d.get("cached_tokens", 0)), int(d.get("cache_write_tokens", 0))
    return (Decimal(u["input_tokens"] - cache - write) * Decimal(str(p["input"]))
             + Decimal(cache) * Decimal(str(p["cached"]))
             + Decimal(write) * Decimal(str(p.get("cache_write", p["input"])))
             + Decimal(u["output_tokens"]) * Decimal(str(p["output"]))) / Decimal(1000000)


def token_totals(rows):
    return {"input": sum(c.get("usage", {}).get("input_tokens", 0) for c in rows if c.get("usage")),
            "output": sum(c.get("usage", {}).get("output_tokens", 0) for c in rows if c.get("usage")),
            "cached": sum(c.get("usage", {}).get("input_tokens_details", {}).get("cached_tokens", 0) for c in rows if c.get("usage")),
            "cache_write": sum(c.get("usage", {}).get("input_tokens_details", {}).get("cache_write_tokens", 0) for c in rows if c.get("usage"))}


def check_artifact(worker, call, key):
    a = call.get(key)
    if not a:
        check("MODEL_ARTIFACT", False, {"worker": worker.name, "call_id": call["call_id"], "kind": key})
        return None
    path = worker / a["path"]
    if not path.is_file():
        check("MODEL_ARTIFACT", False, str(path))
        return None
    data = path.read_bytes()
    value = json.loads(data)
    check("MODEL_ARTIFACT_CANONICAL_HASH", sha(canonical(value).encode()) == a["sha256"], str(path))
    track(path, data)
    return value


def small_sample(row):
    return {key: row.get(key) for key in (
        "sample_id", "evidence_id", "process_id", "generation", "phase", "at_seconds", "monotonic_ns",
        "sample_start_ns", "sample_end_ns", "phys_footprint_bytes", "resident_size_bytes", "success")}


def memory_worker(rows):
    check("MEMORY_SEQUENCE", [r["sample_id"] for r in rows] == list(range(1, len(rows)+1)), rows[0]["variant"] + str(rows[0]["repeat"]))
    identities = {(r["process_id"], r["generation"]) for r in rows}
    check("MEMORY_ONE_WORKER_IDENTITY", len(identities) == 1, sorted(identities))
    check("MEMORY_KERNEL_ABI", all(r.get("task_info_flavor") == 22 and r.get("task_info_count", 0) >= 38 for r in rows), sorted({(r.get("task_info_flavor"), r.get("task_info_count")) for r in rows}))
    good = [r for r in rows if r.get("success") is True and all(isinstance(r.get(k), int) and r[k] >= 0 for k in ("phys_footprint_bytes", "resident_size_bytes"))]
    check("MEMORY_COMPLETE_SAMPLES", len(good) == len(rows), {"total": len(rows), "valid": len(good)})
    check("MEMORY_INCLUDE_ALL_WORKER_ROWS", all(r.get("included_in_via_sum") is True for r in rows), len(rows))
    check("MEMORY_CLOCK_ORDER", all(a["sample_start_ns"] <= a["sample_end_ns"] <= b["sample_start_ns"] for a,b in zip(rows,rows[1:])), rows[0]["variant"] + str(rows[0]["repeat"]))
    intervals = [(b["sample_start_ns"] - a["sample_start_ns"]) / 1e9 for a,b in zip(rows,rows[1:])]
    durations = [(r["sample_end_ns"] - r["sample_start_ns"]) / 1e9 for r in rows]
    phases = {}
    for phase in ("PREPARATION", "ACTIVITY", "DRAIN", "IDLE"):
        selected = [r for r in good if r["phase"] == phase]
        check("MEMORY_PHASE_PRESENT", bool(selected), phase)
        phases[phase] = {"n": len(selected), "first": small_sample(selected[0]) if selected else None,
                         "last": small_sample(selected[-1]) if selected else None,
                         "sample_span_seconds": (selected[-1]["sample_start_ns"]-selected[0]["sample_start_ns"])/1e9 if selected else None}
    idle = [r for r in good if r["phase"] == "IDLE"]
    fp = max(good, key=lambda r: r["phys_footprint_bytes"])
    rss = max(good, key=lambda r: r["resident_size_bytes"])
    return {"n": len(rows), "failed_samples": len(rows)-len(good), "process_identities": [list(x) for x in sorted(identities)],
            "first": small_sample(rows[0]), "last": small_sample(rows[-1]), "phases": phases,
            "observed_footprint_peak_bytes": fp["phys_footprint_bytes"], "observed_footprint_peak_mib": fp["phys_footprint_bytes"]/MIB,
            "footprint_peak_sample": small_sample(fp), "observed_rss_peak_bytes": rss["resident_size_bytes"],
            "observed_rss_peak_mib": rss["resident_size_bytes"]/MIB, "rss_peak_sample": small_sample(rss),
            "idle_footprint_signed_change_bytes": idle[-1]["phys_footprint_bytes"]-idle[0]["phys_footprint_bytes"],
            "idle_footprint_nonnegative_increase_bytes": max(0,idle[-1]["phys_footprint_bytes"]-idle[0]["phys_footprint_bytes"]),
            "idle_rss_signed_change_bytes": idle[-1]["resident_size_bytes"]-idle[0]["resident_size_bytes"],
            "interval_seconds": {"mean": statistics.mean(intervals), "min": min(intervals), "max": max(intervals),
                                 "over_100ms": sum(x>.1 for x in intervals)},
            "kernel_read_seconds": {"mean": statistics.mean(durations), "max": max(durations)}}


manifest_path = CAMPAIGN / "manifest.json"
manifest = json.loads(manifest_path.read_bytes())
track(manifest_path, manifest_path.read_bytes())
source_man = json.loads((CAMPAIGN / "evaluation-tool-manifest.json").read_text())
source_digest = source_man.get("source_digest", source_man.get("digest", source_man.get("sha256")))
if not source_digest:
    source_digest = json.loads((CAMPAIGN / "evaluation-tools-current.json").read_text()).get("source_digest")
check("EXPECTED_CAMPAIGN_SOURCE_DIGEST", EXPECTED_DIGEST in canonical(source_man), str(CAMPAIGN / "evaluation-tool-manifest.json"))
rows_by_variant = defaultdict(list)
memory_by_variant = defaultdict(list)
worker_results = {}
all_ids = []
expected_cases = set(manifest["case_ids"])
for worker in sorted((CAMPAIGN / "workers").iterdir()):
    if not worker.is_dir():
        continue
    calls = load(worker / "model-calls.jsonl")
    starts = load(worker / "model-call-starts.jsonl")
    billing = load(worker / "billing.jsonl")
    memory = load(worker / "memory.jsonl")
    allocations = load(worker / "allocations.jsonl")
    summary = load(worker / "worker-summary.jsonl")
    trials = load(worker / "trials-unjudged.jsonl")
    events = load(worker / "events.jsonl")
    validation = load(worker / "model-validation.jsonl", required=False)
    variant, repeat = worker.name[0], int(worker.name[1:])
    check("COMPLETE_WORKER_SUMMARY", len(summary)==1 and summary[0].get("status")=="COMPLETE" and summary[0].get("case_count")==21, worker.name)
    check("COMPLETE_CASE_POPULATION", len(trials)==21 and {r["case_id"] for r in trials}==expected_cases, worker.name)
    check("WORKER_ROW_SCOPE", all(r.get("variant")==variant and r.get("repeat")==repeat for r in calls+starts+billing+memory+allocations), worker.name)
    callmap = {c["call_id"]: c for c in calls}
    all_ids.extend(callmap)
    check("UNIQUE_WORKER_CALL_ID", len(callmap)==len(calls), worker.name)
    check("START_TERMINAL_RECONCILIATION", Counter(c["call_id"] for c in calls)==Counter(c["call_id"] for c in starts), worker.name)
    check("BILLING_MIRROR_RECONCILIATION", Counter(c["call_id"] for c in calls)==Counter(c["call_id"] for c in billing), worker.name)
    strip = lambda r: {k:v for k,v in r.items() if k not in {"evidence_id","at_seconds","monotonic_ns"}}
    bmap={c["call_id"]:c for c in billing}
    check("BILLING_MIRROR_CONTENT", all(strip(c)==strip(bmap[c["call_id"]]) for c in calls), worker.name)
    exact = Decimal(0)
    roles = defaultdict(lambda: {"calls":0,"exact_usd":Decimal(0),"tokens":Counter()})
    discrepancies=[]
    for c in calls:
        amount=money(c)
        check("CALL_KNOWN_USAGE", amount is not None and c.get("usage_state")=="PROVIDER_REPORTED", c["call_id"])
        if amount is None:
            continue
        u=c["usage"]; d=u.get("input_tokens_details",{})
        raw=c.get("raw_usage") or {}
        check("RAW_NORMALIZED_TOKEN_COUNTS", u["input_tokens"]==raw.get("prompt_tokens",raw.get("input_tokens")) and u["output_tokens"]==raw.get("completion_tokens",raw.get("output_tokens")), c["call_id"])
        rawd=raw.get("prompt_tokens_details",raw.get("input_tokens_details",{})) or {}
        check("RAW_NORMALIZED_CACHE_COUNTS", d.get("cached_tokens",0)==rawd.get("cached_tokens",0) and d.get("cache_write_tokens",0)==rawd.get("cache_write_tokens",0), c["call_id"])
        check("NONNEGATIVE_USAGE", all(isinstance(t,int) and t>=0 for t in [u["input_tokens"],u["output_tokens"],d.get("cached_tokens",0),d.get("cache_write_tokens",0)]) and d.get("cached_tokens",0)+d.get("cache_write_tokens",0)<=u["input_tokens"], c["call_id"])
        check("ACTUAL_FORMAL_ROLE_AND_MODEL", c["role"] in ("CINTERPRET","CRESPONSE") and c.get("provider")=="qwen" and c.get("model")=="qwen3.6-35b-a3b" and c.get("evidence_level")=="MEASURED_MODEL", c["call_id"])
        check("CALL_CLOCK_ORDER", c["enqueue_ns"]<=c["send_ns"]<=c["terminal_ns"], c["call_id"])
        diff=abs(amount-Decimal(str(c["cost_usd"])))
        if diff>Decimal("0.000000000001"):
            discrepancies.append({"call_id":c["call_id"],"exact_usd":str(amount),"recorded_usd":c["cost_usd"]})
        context=check_artifact(worker,c,"context")
        output=check_artifact(worker,c,"output")
        if context:
            check("MODEL_CONTEXT_MESSAGE_HASH", sha(canonical(context["messages"]).encode())==c["context_sha256"], c["call_id"])
        if output:
            check("MODEL_OUTPUT_RAW_USAGE", output.get("usage")==c.get("raw_usage"), c["call_id"])
        exact+=amount
        role=roles[c["role"]]; role["calls"]+=1; role["exact_usd"]+=amount; role["tokens"].update(token_totals([c]))
    check("TOKEN_PRICE_RECOMPUTE", not discrepancies, {"worker":worker.name,"discrepancies":discrepancies})
    check("SUMMARY_USAGE_PRICE_SUM", abs(exact-Decimal(str(summary[0]["provider_usage_price_usd"])))<=Decimal("0.000000000001"), {"worker":worker.name,"exact":str(exact),"summary":summary[0]["provider_usage_price_usd"]})
    discarded=[e for e in events if "DISCARD" in e.get("kind","")]
    discard_scopes=[{k:r.get(k) for k in ("kind","request_id","scene_key","reason","role","evidence_id")} for r in discarded]
    cancelled=[c for c in calls if "CANCEL" in c.get("status","")]
    call_discard_scope=[c for c in calls if any(e.get("request_id")==c.get("request_id") and e.get("scene_key")==c.get("case_id") for e in discarded if e.get("request_id"))]
    frames=allocations[0]["frames"]
    check("TOUCHED_RGBA_BUFFER_BYTES", len(frames)==15 and allocations[0]["actual_bytes"]==311040000 and all(f.get("touched") is True for f in frames), worker.name)
    check("ALLOCATION_ACCOUNTING_NOT_ADDITIVE", all("NOT_ADDED_TO_OS_MEMORY" in a.get("accounting","") for a in allocations[:2]), worker.name)
    scoped=[a for a in allocations if a.get("kind")=="SCOPED_ORIGINALS_MATERIALIZED"]
    check("SCOPED_ORIGINAL_PAYLOAD", len(scoped)==21 and {a["scene_key"] for a in scoped}==expected_cases and all(a["canonical_payload_bytes"]==20971520 for a in scoped), worker.name)
    baseline_path=worker/"activity-baseline.json.gz"
    encoded=gzip.decompress(baseline_path.read_bytes())
    check("BASELINE_SERIALIZATION_COPY", sha(encoded)==allocations[1]["buffer_sha256"] and len(encoded)==allocations[1]["actual_serialization_copy_bytes"], worker.name)
    track(baseline_path,baseline_path.read_bytes())
    m=memory_worker(memory)
    check("SUMMARY_AFTER_FINAL_MEMORY_SAMPLE", summary[0]["monotonic_ns"]>=memory[-1]["monotonic_ns"], worker.name)
    check("ALL_HTTP_WITHIN_MEMORY_WINDOW", all(memory[0]["sample_start_ns"]<=c["enqueue_ns"]<=c["terminal_ns"]<=memory[-1]["sample_end_ns"] for c in calls), worker.name)
    worker_results[worker.name]={"variant":variant,"repeat":repeat,"model_call_count":len(calls),"start_count":len(starts),"billing_mirror_count":len(billing),
        "model_cost_usd_exact":str(exact),"recorded_cost_usd_sum":str(sum((Decimal(str(c["cost_usd"])) for c in calls),Decimal(0))),
        "worker_summary_cost_usd":summary[0]["provider_usage_price_usd"],"roles":{k:{**v,"exact_usd":str(v["exact_usd"]),"tokens":dict(v["tokens"])} for k,v in roles.items()},
        "tokens":token_totals(calls),"provider_status_counts":counter(calls,"status"),"http_status_counts":counter(calls,"http_status"),
        "attempt_counts":counter(calls,"attempt"),"purpose_counts":counter(calls,"purpose"),"model_validation_retry_count":len(validation),
        "unknown_usage_call_ids":[c["call_id"] for c in calls if c.get("usage") is None],
        "cancelled_after_send_calls":[{"call_id":c["call_id"],"case_id":c["case_id"],"cost_usd":c["cost_usd"],"usage":c["usage"]} for c in cancelled],
        "runtime_discard_events":discard_scopes,"calls_associated_with_discarded_request_count":len(call_discard_scope),
        "calls_associated_with_discarded_request_cost_usd":str(sum((money(c) for c in call_discard_scope),Decimal(0))),
        "discard_association_is_request_scope_not_unique_per_call_disposition":True,
        "allocation_record_count":len(allocations),"rgba_total_bytes":allocations[0]["actual_bytes"],"rgba_frame_count":len(frames),
        "original_payload_bytes":allocations[1]["payload_bytes"],"temporary_serialization_copy_bytes":allocations[1]["actual_serialization_copy_bytes"],
        "activity_original_sha256":allocations[1]["buffer_sha256"],"memory":m}
    rows_by_variant[variant].extend(calls)
    memory_by_variant[variant].extend(memory)

check("TEN_EXPECTED_WORKERS", set(worker_results)=={f"{v}{r}" for v in "AB" for r in range(1,6)}, sorted(worker_results))
check("UNIQUE_CAMPAIGN_CALL_ID", len(all_ids)==len(set(all_ids)), len(all_ids))
check("NO_OVERLAPPING_WORKER_SAMPLE_INTERVALS", all(a["memory"]["last"]["sample_end_ns"]<b["memory"]["first"]["sample_start_ns"] for a,b in zip(sorted(worker_results.values(),key=lambda x:x["memory"]["first"]["sample_start_ns"]),sorted(worker_results.values(),key=lambda x:x["memory"]["first"]["sample_start_ns"])[1:])), "One isolated measured worker at a time")

spec=importlib.util.spec_from_file_location("frozen_metrics_for_comparison_only",CAMPAIGN/"frozen-source/benchmark/architecture/dp41_45_poc/metrics.py")
metrics=importlib.util.module_from_spec(spec);spec.loader.exec_module(metrics)
variants={}
for variant in "AB":
    workers=[worker_results[f"{variant}{r}"] for r in range(1,6)]
    total=sum((Decimal(w["model_cost_usd_exact"]) for w in workers),Decimal(0))
    mean_cost=total/Decimal(5)
    fp=max(workers,key=lambda w:w["memory"]["observed_footprint_peak_bytes"])
    rss=max(workers,key=lambda w:w["memory"]["observed_rss_peak_bytes"])
    frozen_billing=metrics._billing(rows_by_variant[variant],variant,[1,2,3,4,5],manifest)
    frozen_memory=metrics._memory(memory_by_variant[variant],[1,2,3,4,5],manifest)
    check("FROZEN_BILLING_INDEPENDENT_MATCH", abs(Decimal(str(frozen_billing["value"]))-mean_cost)<Decimal("0.000000000001"), variant)
    check("FROZEN_MEMORY_INDEPENDENT_MATCH", frozen_memory["phys_footprint"]["peak_bytes"]==fp["memory"]["observed_footprint_peak_bytes"] and frozen_memory["resident_size_rss"]["peak_bytes"]==rss["memory"]["observed_rss_peak_bytes"] and all(row["idle_increase_bytes"]==workers[i]["memory"]["idle_footprint_nonnegative_increase_bytes"] for i,row in enumerate(frozen_memory["phys_footprint"]["per_repeat"])), variant)
    variants[variant]={"qa04":{"calls":len(rows_by_variant[variant]),"usd_total_five_p21_exact":str(total),"usd_per_p21_mean_exact":str(mean_cost),
        "usd_per_p21_mean":float(mean_cost),"tokens":token_totals(rows_by_variant[variant]),"roles":counter(rows_by_variant[variant],"role"),
        "provider_status_counts":counter(rows_by_variant[variant],"status"),"attempt_counts":counter(rows_by_variant[variant],"attempt"),
        "known_usage_calls":len([c for c in rows_by_variant[variant] if c.get("usage") is not None]),"invoice_reconciled_usd":None,
        "voice_api_calls":0,"voice_cost_is_not_measured":True,"actual_external_agent_api_cost":None,
        "primary_basis":"ACTUAL_PROVIDER_REPORTED_SEMANTIC_USAGE_X_FROZEN_PRICE; NOT_ACCOUNT_INVOICE",
        "per_repeat":[{"repeat":w["repeat"],"calls":w["model_call_count"],"usd":w["model_cost_usd_exact"]} for w in workers],
        "frozen_metrics_value":frozen_billing["value"]},
        "qa06":{"primary_metric":"MAC_PHYS_FOOTPRINT_SIMULTANEOUS_PROCESS_SUM_SAMPLED_PEAK_OVER_ALL_FIVE_REPEATS",
        "footprint_peak_bytes":fp["memory"]["observed_footprint_peak_bytes"],"footprint_peak_mib":fp["memory"]["observed_footprint_peak_mib"],"footprint_peak_worker":f'{variant}{fp["repeat"]}',"footprint_peak_sample":fp["memory"]["footprint_peak_sample"],
        "rss_peak_bytes":rss["memory"]["observed_rss_peak_bytes"],"rss_peak_mib":rss["memory"]["observed_rss_peak_mib"],"rss_peak_worker":f'{variant}{rss["repeat"]}',"rss_peak_sample":rss["memory"]["rss_peak_sample"],
        "mean_repeat_footprint_peaks_mib_auxiliary":statistics.mean(w["memory"]["observed_footprint_peak_mib"] for w in workers),
        "mean_repeat_rss_peaks_mib_auxiliary":statistics.mean(w["memory"]["observed_rss_peak_mib"] for w in workers),
        "samples":sum(w["memory"]["n"] for w in workers),"failed_samples":sum(w["memory"]["failed_samples"] for w in workers),
        "footprint_within_4096mib_reference":fp["memory"]["observed_footprint_peak_bytes"]<=4096*MIB,
        "rss_within_6144mib_reference":rss["memory"]["observed_rss_peak_bytes"]<=6144*MIB,
        "all_idle_footprint_increases_within_16mib":all(w["memory"]["idle_footprint_nonnegative_increase_bytes"]<=16*MIB for w in workers),
        "windows_commit_bytes":None,"windows_commit_score":None,"conversion_to_windows":False,
        "per_repeat":[{"repeat":w["repeat"],"footprint_peak_mib":w["memory"]["observed_footprint_peak_mib"],"rss_peak_mib":w["memory"]["observed_rss_peak_mib"],"idle_footprint_increase_bytes":w["memory"]["idle_footprint_nonnegative_increase_bytes"]} for w in workers]}}

separate=[]
readiness=json.loads((CAMPAIGN/"preflight-readiness.json").read_text())
paths=[(Path(row["raw_directory"])/"model-calls.jsonl","V7_READINESS_PREFLIGHT_NOT_P21") for row in readiness["actual_preflight"]]
paths += [(p,"JUDGE_NOT_VIA_QA04") for p in (CAMPAIGN/"evaluation").glob("model-calls.jsonl")]
paths += [(p,"NEW_FEATURE_SEMANTIC_VALIDATION_NOT_FORMAL_P21") for p in (CAMPAIGN/"changes-semantic").glob("*/model-calls.jsonl")]
for path,scope in paths:
    calls=load(path,mutable=True)
    starts=load(path.with_name("model-call-starts.jsonl"),required=False,mutable=True)
    pending=sorted({c["call_id"] for c in starts}-{c["call_id"] for c in calls})
    known=[money(c) for c in calls if c.get("usage") is not None]
    separate.append({"path":str(path),"scope":scope,"snapshot_utc":datetime.now(timezone.utc).isoformat(),"calls":len(calls),
                     "roles":counter(calls,"role"),"providers":counter(calls,"provider"),"status":counter(calls,"status"),
                     "known_usage_price_usd_exact":str(sum(known,Decimal(0))),"unknown_usage_calls":[c["call_id"] for c in calls if c.get("usage") is None],
                     "pending_call_ids_at_snapshot":pending,"not_final_account_invoice":True,"excluded_from_primary_qa04":True})

limitations=[
    {"id":"L01","kind":"DECLARED_CAMPAIGN_BOUNDARY","text":"Actual semantic HTTP usage at frozen standard prices is measured. Account invoices, physical voice model, physical microphone/ASR, and external Agent internals are not measured. Simulated voice estimates must remain separate."},
    {"id":"L02","kind":"DECLARED_CAMPAIGN_BOUNDARY","text":"Mac task_info samples include one isolated worker PID per activity, its HTTP and sampling threads, VIA, local Agent/speech stubs, source buffers, and RAM evidence recorder. They exclude campaign controller, separate post-timing judge/change processes, server RAM and OS desktop. This is a PoC worker scope, not a whole-product process census."},
    {"id":"L03","kind":"SAMPLING_LIMITATION","text":"Observed peaks are maxima of sampled TASK_VM_INFO values, not a kernel-maintained all-time peak. The sampler starts after module imports and stops after idle. Between-sample allocations and earlier import-time peaks cannot be recovered. Actual interval/read durations are preserved for every worker."},
    {"id":"L04","kind":"MEMORY_ACCOUNTING_LIMITATION","text":"For the current one-PID runtime, simultaneous process sum equals that PID's observation. No per-process peaks from different times/repeats are added. RSS is mapped resident size and phys_footprint is kernel footprint; neither is Windows commit. For future multi-PID/shared mappings, this sum would not establish unique physical RAM or automatically deduplicate shared pages."},
    {"id":"L05","kind":"ALLOCATION_TRACE_LIMITATION","text":"Allocations trace 15 real touched RGBA buffers, the actual serialization copy and 21 materialized canonical original payloads. It is explanatory rather than a complete Python object/alias census. Common buffers/client are shared within each worker; source/owner/context/event snapshots and serialization copies are accounted in OS samples when present. Declared allocation byte counts are never added to OS metrics."},
    {"id":"L06","kind":"EVIDENCE_ATTRIBUTION_LIMITATION","text":"All attempted HTTP costs remain included regardless of semantic adoption or local cancellation. MEANING_DISCARDED has Request/scene provenance, but not a terminal call_id; request-associated calls are disclosed without claiming a unique per-call semantic discard mapping. Frozen synthetic OUTPUT_DISCARDED events do not imply a newly billed voice/model call."},
    {"id":"L07","kind":"LIVE_NON_VIA_LEDGER_SNAPSHOT","text":"Preparation, judge and change-feature validation ledgers are separate snapshots and excluded from formal VIA mean. They can continue appending after this audit. The complete account spending ledger, including invalidated earlier campaigns/preparation and invoices, is not reconstructed by this QA04/QA06 audit."},
]

for path,meta in list(inventory.items()):
    if not meta.get("append_only_snapshot"):
        check("IMMUTABLE_RAW_UNCHANGED_DURING_AUDIT", Path(path).is_file() and sha(Path(path).read_bytes())==meta["sha256"], path)

failures=[c for c in checks if not c["passed"]]
report={"schema":"DP41-QA04-QA06-INDEPENDENT-RAW-AUDIT-v1","campaign":str(CAMPAIGN),"campaign_contract":manifest["campaign_contract"],
    "generated_utc":datetime.now(timezone.utc).isoformat(),"author":"poc_qa_readiness","api_calls_by_audit":0,
    "source_mutation":False,"raw_mutation":False,"audit_script_sha256":sha(Path(__file__).read_bytes()),"expected_frozen_source_digest":EXPECTED_DIGEST,
    "method":"Independent Decimal token-price reconstruction and direct ordered OS-sample maximum/idle calculation, followed by comparison with frozen metrics._billing/_memory only.",
    "status":"PASS_WITH_DECLARED_LIMITATIONS" if not failures else "FAIL_REQUIRES_REVIEW","workers":worker_results,"variants":variants,
    "checks_summary":{"total":len(checks),"passed":len(checks)-len(failures),"failed":len(failures)},"failed_checks":failures,
    "findings":findings,"limitations":limitations,"separate_non_via_ledgers_snapshot":separate,
    "raw_file_inventory":inventory,"full_checks_file":"qa04-qa06-independent-checks.json"}
(OUT/"qa04-qa06-independent-checks.json").write_text(json.dumps(checks,ensure_ascii=False,indent=2)+"\n")
(OUT/"qa04-qa06-independent-review.json").write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")

lines=["# QA04 / QA06 원시 원장 독립 검수", "", f"생성 UTC: {report['generated_utc']}", "", f"상태: **{report['status']}**. API 0회, 후보/동결/측정 원시 자료 수정 0개. 모든 호출의 token × 기록된 가격을 Decimal로 다시 계산하고, 순서가 있는 OS 표본을 직접 계산한 뒤 frozen metrics와 대조했다.", "", "| 방안 | 실제 의미 호출 | 5회 총 usage × 가격 USD | P21당 평균 USD | 전체 5회 footprint peak MiB | 전체 5회 RSS peak MiB |", "| --- | ---: | ---: | ---: | ---: | ---: |"]
for v in "AB":
    q,m=variants[v]["qa04"],variants[v]["qa06"]
    lines.append(f"| {v} | {q['calls']} | {q['usd_total_five_p21_exact']} | {q['usd_per_p21_mean_exact']} | {m['footprint_peak_mib']:.6f} | {m['rss_peak_mib']:.6f} |")
lines += ["", "QA04 값은 provider가 반환한 실제 의미 모델 token을 동결 단가로 계산한 값이며 청구서 확인 금액은 아니다. 음성 API 호출은 없고 음성 비용 추정은 이 주지표에 더하지 않았다. 청구용 billing.jsonl은 model-calls.jsonl의 같은 call_id 기록을 복제하므로 한 번만 셌다. 실패/폐기/취소된 호출을 제외하지 않았다.", "", "| Worker | 호출 | 실제 USD | footprint peak MiB | RSS peak MiB | idle footprint 증가 bytes | 메모리 표본 |", "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"]
for name,w in sorted(worker_results.items()):
    m=w["memory"]
    lines.append(f"| {name} | {w['model_call_count']} | {w['model_cost_usd_exact']} | {m['observed_footprint_peak_mib']:.6f} | {m['observed_rss_peak_mib']:.6f} | {m['idle_footprint_nonnegative_increase_bytes']} | {m['n']} |")
lines += ["", f"자동 대조 {len(checks)}개 중 실패 {len(failures)}개. call 시작/종료/billing의 ID와 내용, Context/output의 canonical SHA, 실제 usage와 cache 항목, summary 합계, 전체 21사례 및 10 worker를 확인했다. 재시도는 실제 attempt 기록으로 집계했다. OS 실패 표본, phase 누락, 순서/중복/process identity를 확인했고 포함된 각 호출은 메모리 관측 구간 안에서 끝났다.", "", "메모리 주지표는 다섯 회 전체에서 같은 관측의 process 합 최대이며 반복 peak의 평균은 보조값이다. 현재 실제 구성은 worker마다 PID 하나여서 합은 그 PID의 kernel 값과 같다. footprint 최대 시점의 RSS와 RSS 자체 최대 시점은 JSON에 따로 저장했다. 10초 idle 구간 첫/마지막의 같은 process 값을 비교하며 감소는 보호조건용 증가값 0으로 둔다. Windows commit은 None이며 Windows 점수/환산을 주장하지 않는다.", "", "준비/평가/추가 변경 원장 스냅샷:", "", "| 범위 | 호출 | snapshot known usage × price USD | pending |", "| --- | ---: | ---: | ---: |"]
for x in separate:
    lines.append(f"| {x['scope']} / {Path(x['path']).parent.name} | {x['calls']} | {x['known_usage_price_usd_exact']} | {len(x['pending_call_ids_at_snapshot'])} |")
lines += ["", "범위와 검증 한계:", ""]
lines += [f"- {x['id']}: {x['text']}" for x in limitations]
if failures:
    lines += ["", "실패 대조:", ""]+[f"- {c['check']}: {c['evidence']}" for c in failures]
lines += ["", "증거: qa04-qa06-independent-review.json의 worker별 peak 표본, 역할별 token/비용, 폐기 사건, phase 경계, 배분 기록, 원본 SHA를 참조한다. 모든 개별 대조는 qa04-qa06-independent-checks.json에 보존했다. 이 검수는 QA01~QA03/QA05 수치나 QA 우열을 선주입하지 않는다."]
(OUT/"qa04-qa06-independent-review.md").write_text("\n".join(lines)+"\n")
print(json.dumps({"status":report["status"],"checks":report["checks_summary"],"variants":variants,"failed_checks":failures},ensure_ascii=False,indent=2))
