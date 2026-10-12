"""Read-only independent audit of hash-pinned QA05 selection and evidence scope."""
import ast
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import statistics
import sys

sys.dont_write_bytecode = True
C = Path(__file__).resolve().parents[1]
O = Path(__file__).resolve().parent
EXPECTED = "44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce"
IGNORED = {".git", "__pycache__", ".pytest_cache", ".venv", "results"}
checks, files, jobs, supplements = [], {}, [], {}

def sha(data):
    return hashlib.sha256(data).hexdigest()

def read(path):
    path = Path(path)
    data = path.read_bytes()
    files[str(path)] = sha(data)
    return json.loads(data)

def rows(path):
    path = Path(path)
    data = path.read_bytes()
    files[str(path)] = sha(data)
    return [json.loads(line) for line in data.splitlines() if line]

def check(kind, passed, evidence):
    checks.append({"check":kind, "passed":bool(passed), "evidence":evidence})

def resolve(path):
    path = Path(path)
    return path if path.is_absolute() else C / path

def snapshot(root):
    result = {}
    for p in sorted(Path(root).rglob("*")):
        relative = p.relative_to(root)
        if any(x in IGNORED for x in relative.parts):
            continue
        if p.is_symlink():
            raise ValueError(f"unexpected source symlink: {p}")
        if p.is_file():
            data = p.read_bytes()
            result[str(relative)] = {"sha256":sha(data), "bytes":len(data), "mode":p.stat().st_mode & 0o777}
            files[str(p)] = sha(data)
    return result

def digest(value):
    return sha(json.dumps(value,sort_keys=True,separators=(",", ":")).encode())

def check_ref(ref, expected):
    path = resolve(ref)
    actual = sha(path.read_bytes()) if path.is_file() else None
    check("HASH_PINNED_REFERENCE", actual == expected, str(path))
    if actual is not None:
        files[str(path)] = actual

manifest = read(C/"manifest.json")
selection = read(C/"qa05-selection-manifest.json")
baseline = snapshot(C/"frozen-source")
baseline_digest = digest(baseline)
check("MEASURED_BASELINE_IDENTITY", manifest["frozen_source_digest"] == selection["frozen_source_digest"] == EXPECTED, "manifest + QA05 selection")
check("BASELINE_FLAT_HASHES", {p:v["sha256"] for p,v in baseline.items()} == manifest["frozen_source_hashes"], "all 53 measured source files")
check("SELECTION_COMPLETE", selection["status"] == "COMPLETE" and len(selection["selected"]) == 12, len(selection["selected"]))
check("SELECTION_IS_TWO_LLM_AGENT_REVIEWS", selection["is_human_review"] is False and selection["review_type"] == "TWO_DISTINCT_LLM_AGENT_SOURCE_AND_FUNCTIONAL_TRACE_REVIEWS", selection["review_type"])
review_files = []
for ref in selection["independent_reviews"]:
    check_ref(ref["path"], ref["sha256"])
    review_files.append(read(C/ref["path"]))
root, contract = review_files
check("TWO_REVIEW_BASELINES", root["frozen_source_digest"] == contract["frozen_source_digest"] == EXPECTED and root["source_snapshot_digest"] == contract["baseline_snapshot_digest"] == baseline_digest, "root + contract independent reviews")
check("CONTRACT_UNRESOLVED_P1_P2", contract["unresolved_P1_P2"] == [], contract["unresolved_P1_P2"])
root_jobs = {(r["variant"],r["task_id"]):r for r in root["jobs"]}
check("ROOT_TWELVE_DISTINCT_JOBS", len(root_jobs)==12 and len(root["jobs"])==12, len(root_jobs))
for path,expected in contract["evidence_file_sha256"].items():
    check_ref(path,expected)

seen = set()
expected_population = {(v,f"M{n:02d}") for v in "AB" for n in range(1,7)}
for selected in selection["selected"]:
    path = C/selected["reviewed_result_ref"]
    check_ref(path,selected["reviewed_result_sha256"])
    record = read(path)
    key = (record["variant"],record["task_id"])
    check("SELECTED_IDENTITY_AND_UNIQUENESS", key == (selected["variant"],selected["task_id"]) and key not in seen and key in expected_population and Path(record["result_ref"]).resolve()==path.resolve(), record["job_id"])
    seen.add(key)
    directory = path.parent
    initial = read(record["initial_result_ref"])
    check("INITIAL_EXECUTION_PRESERVED", initial["status"]=="AWAITING_RESPONSIBILITY_REVIEW" and initial["count"] is None and initial["responsibility_review_passed"] is False, record["job_id"])
    check("COMPLETED_FUNCTIONAL_RECORD", record["status"]=="COMPLETED" and record["evidence_level"]=="MEASURED_CHANGE_EXPERIMENT" and all(record[x] is True for x in ("completion_passed","regression_passed","patch_verified","responsibility_review_passed","original_baseline_unchanged")), record["job_id"])
    check("INDEPENDENT_SAME_BASELINE", Path(record["baseline_source_path"]).resolve()==(C/"frozen-source").resolve() and record["baseline_source_hash"]==baseline_digest and read(directory/"baseline-manifest.json")==baseline, record["job_id"])
    after = snapshot(directory/"source")
    after_digest = digest(after)
    check("SELECTED_CHANGED_SOURCE_HASH", after_digest==record["after_source_hash"]==selected["source_snapshot_digest"] and read(directory/"after-source-manifest.json")==after, record["job_id"])
    changed = sorted(p for p in set(baseline)|set(after) if baseline.get(p)!=after.get(p))
    check("ACTUAL_CHANGED_PATHS", changed==sorted(record["changed_paths"]), record["job_id"])
    check_ref(record["patch_ref"],record["patch_sha256"])
    check("UNIT_COUNT_SELECTION_MATCH", record["count"]==selected["count"]==len(set(record["changed_unit_set"])) and record["changed_unit_set"]==selected["changed_unit_set"], record["job_id"])
    reviewers = read(directory/"final-reviews.json")
    first,second = root_jobs[key],contract["jobs"][record["job_id"]]
    check("TWO_DISTINCT_ACCEPTED_REVIEWS", len(reviewers)==2 and reviewers==[first,second] and first["reviewer_id"]!=second["reviewer_id"] and first["accepted"] is True and second["accepted"] is True and first["changed_unit_set"]==second["changed_unit_set"]==record["changed_unit_set"], record["job_id"])
    check("ROOT_ALL18_REVIEW", {x["unit"] for x in first["all_18_assessment"]}=={f"U{i:02d}" for i in range(1,19)} and sorted(x["unit"] for x in first["all_18_assessment"] if x["changed"])==sorted(record["changed_unit_set"]), record["job_id"])
    check("CONTRACT_INITIAL_RESULT_HASH", sha(Path(record["initial_result_ref"]).read_bytes())==second["result_sha256"] and second["after_source_hash"]==after_digest and second["patch_sha256"]==record["patch_sha256"], record["job_id"])
    for e in first["evidence"]:
        check_ref(e["path"],e["sha256"])
    regression = read(directory/"regression.json")
    p21 = regression["P21"]
    check("P21_RECORDED_REPLAY_FULL_COVERAGE", p21["passed"] is True and p21["full_p21"] is True and p21["worker_complete"] is True and len(p21["cases"])==21 and {x["case_id"] for x in p21["cases"]}==set(manifest["case_ids"]) and all(x["passed"] is True for x in p21["cases"]), record["job_id"])
    check("REPLAY_EVIDENCE_SCOPE", p21["api_calls"]==0 and p21["semantic_accuracy_remeasured"] is False and p21["evidence_level"]=="RECORDED_PORT_REPLAY_NOT_SEMANTIC_REMEASUREMENT" and Path(p21["source_root"]).resolve()==(directory/"source").resolve(), record["job_id"])
    check("NAMED_FUNCTIONAL_REGRESSIONS", all(v.get("passed") is True for v in regression.values() if isinstance(v,dict)), record["job_id"])
    jobs.append({"job_id":record["job_id"],"variant":key[0],"task_id":key[1],"reviewed_result_ref":str(path),"reviewed_result_sha256":selected["reviewed_result_sha256"],"initial_result_ref":record["initial_result_ref"],"after_source_hash":after_digest,"patch_sha256":record["patch_sha256"],"actual_changed_paths":changed,"changed_unit_set":record["changed_unit_set"],"count":record["count"],"reviewer_ids":[first["reviewer_id"],second["reviewer_id"]],"recorded_replay_cases":len(p21["cases"]),"scripted_named_regressions":list(regression),"completion_evidence_scope":"EXECUTED_CODE_DELTA_WITH_RECORDED_OR_SCRIPTED_MODEL_PORTS; NOT_NEW_NATURAL_LANGUAGE_MODEL_SUCCESS"})
    if key[1]=="M04":
        original=read(C/f"changes/CH-DP41-{key[0]}-M04/result.json")
        check("ORIGINAL_M04_NOT_REPLACED_OR_DOUBLE_COUNTED", original["after_source_hash"]!=after_digest and not (C/f"changes/CH-DP41-{key[0]}-M04/reviewed-result.json").exists(), record["job_id"])
        check("M04_LINEAGE_REQUIRED_PROBES", all(regression[x]["passed"] for x in ["same_request_two_revisions","actual_owner_revision_and_stale_execution","AFTER","USE_RESULT","success_branch_preserved"]), record["job_id"])
        if key[0]=="B":
            check("TYPED_B_BINDER_BYTE_IDENTICAL", after["prototypes/candidates/dp41_45_poc/dp41/b.py"]==baseline["prototypes/candidates/dp41_45_poc/dp41/b.py"], record["job_id"])
    if key[1]=="M06":
        check("M06_GENUINE_ZERO_DIFF", record["count"]==0 and changed==[] and after==baseline and Path(record["patch_ref"]).read_bytes()==b"" and record["baseline_support_audit_passed"] is True and read(directory/"baseline-audit.json")["supports_requirement"] is True, record["job_id"])

check("EXACT_TWELVE_JOB_POPULATION", seen==expected_population, sorted(seen))

# Execute the existing pure selection resolver, which only reads/hashes artifacts.
sys.path.insert(0,str(C/"evaluator-tools"))
from benchmark.architecture.dp41_45_poc.finalize import selected_change_records
resolver_records=selected_change_records(C)
check("OFFICIAL_SELECTED_CHANGE_RECORDS_AGREE", len(resolver_records)==12 and {(r["variant"],r["task_id"]) for r in resolver_records}==expected_population and {r["result_ref"] for r in resolver_records}=={j["reviewed_result_ref"] for j in jobs}, "evaluator-tools finalize.selected_change_records")

for v in "AB":
    natural = rows(C/selection["M04_actual_natural_language_probes"][v]["ref"])
    n=natural[-1]
    check("M04_ACTUAL_MODEL_FAIL_PRESERVED", len(natural)==1 and n["status"]==selection["M04_actual_natural_language_probes"][v]["status"]=="FAIL" and n["scope"]=="EVALUATION_CHANGE" and n["evidence_level"]=="MEASURED_MODEL_CHANGE_VALIDATION" and n["excluded_from_p21_qa04"] is True and selection["M04_actual_natural_language_probes"][v]["included_in_P21"] is False, v)
    selected_m04=next(x for x in jobs if x["variant"]==v and x["task_id"]=="M04")
    check("M04_ACTUAL_IMPORTED_SELECTED_SOURCE", Path(n["source"]).resolve()==Path(selected_m04["reviewed_result_ref"]).parent.joinpath("source").resolve() and all(Path(p).is_relative_to(Path(n["source"])) for p in n["imported_paths"].values()), v)
    supplements[f"M04-{v}"]={"status":n["status"],"checks":n["checks"],"scope":n["scope"],"source":n["source"],"semantic_judgement":n["semantic_judgement"],"actual_model_fail_not_overridden_by_scripted_pass":True,"included_in_formal_p21":False}

    d=C/f"changes-supplemental/CH-DP41-{v}-M05-PUBLISHED-TOKEN-RESTART-v1"
    r=read(d/"result.json");pub=read(d/"publish-raw.json")["record"];restart=read(d/"restart-raw.json")["record"];stored=read(d/"published-owner-and-binding-snapshot.json")
    check("M05_FRESH_PROCESS_RESTORE", r["status"]=="PASSED" and r["source_unchanged"] is True and r["api_calls"]==0 and pub["passed"] is True and restart["passed"] is True and pub["pid"]!=restart["pid"] and stored["producer_process_pid"]==pub["pid"], v)
    check("M05_SAME_ACTUAL_PRESENTED_TOKEN", pub["token_sha256"]==restart["token_sha256"]==sha(stored["token"].encode()) and pub["snapshot_sha256"]==restart["snapshot_sha256"] and pub["publication_id"]==stored["actual_publication_id"], v)
    check("M05_ACTUAL_ANSWER_AND_STALE_GUARDS", all(restart[k] is True for k in ("same_published_token","corrupt_dialogue_token_rejected","one_actual_answer_command","duplicate_rejected")) and restart["owner_status"]=="ANSWERED", v)
    supplements[f"M05-{v}"]={"status":r["status"],"publish_pid":pub["pid"],"restart_pid":restart["pid"],"token_sha256":pub["token_sha256"],"actual_publication_id":pub["publication_id"],"restored_actual_answer_once":restart["one_actual_answer_command"],"api_calls":0,"evidence_level":"SCRIPTED_FUNCTIONAL_REGRESSION","ref":str(d/"result.json")}
    for phase in ("before","after"):
        p=C/f"changes-supplemental/M06-EXACT-S103-v3/{v}-{phase}/raw.json"
        raw=read(p);x=raw["result"]
        check("M06_EXACT_PREFERENCE_ORACLE", raw["returncode"]==0 and x["status"]=="PASSED" and x["input"]=="앞으로 비교 결과는 표로 보여주는 걸 선호한다고 기억해줘." and all(y["passed"] is True for y in x["checks"].values()) and {f"C{i}" for i in range(1,7)} <= set(x["checks"]), f"{v}-{phase}")
        check("M06_EXACT_HELPER_SCOPE", x["api_calls"]==0 and x["actual_model_semantic_accuracy_claim"] is False and x["P21_accuracy_denominator_addition"]==0 and x["evidence_level"]=="SCRIPTED_EXACT_ORACLE_FUNCTIONAL_REGRESSION", f"{v}-{phase}")
        expected_source=C/"frozen-source" if phase=="before" else C/f"changes/CH-DP41-{v}-M06/source"
        check("M06_IMPORTED_ACTUAL_BEFORE_AFTER_SOURCE", all(Path(path).is_relative_to(expected_source) for path in x["imported_modules"].values()), f"{v}-{phase}")
        supplements[f"M06-{v}-{phase}"]={"status":x["status"],"input":x["input"],"passed_checks":list(x["checks"]),"api_calls":x["api_calls"],"actual_model_semantic_accuracy_claim":False,"P21_accuracy_denominator_addition":0,"ref":str(p)}

for path,expected in list(files.items()):
    check("EVIDENCE_UNCHANGED_DURING_REVIEW", Path(path).is_file() and sha(Path(path).read_bytes())==expected, path)
failures=[x for x in checks if not x["passed"]]
summary={v:{"per_task":[j["count"] for j in jobs if j["variant"]==v],"sum":sum(j["count"] for j in jobs if j["variant"]==v),"mean":statistics.mean(j["count"] for j in jobs if j["variant"]==v)} for v in "AB"}
report={"schema":"DP41-QA05-SELECTION-INDEPENDENT-AUDIT-v1","campaign":str(C),"generated_utc":datetime.now(timezone.utc).isoformat(),"author":"poc_qa_readiness_selection_auditor","change_patch_author":True,"review_role":"Independent integrity/scope verification of the two completed responsibility reviews; not an additional count-voting reviewer", "api_calls":0,"candidate_or_raw_modifications":False,"frozen_source_digest":EXPECTED,"source_snapshot_digest":baseline_digest,"status":"PASS_WITH_DECLARED_EVIDENCE_SCOPE" if not failures else "FAIL_REQUIRES_REVIEW","script_sha256":sha(Path(__file__).read_bytes()),"selection_sha256":sha((C/"qa05-selection-manifest.json").read_bytes()),"check_summary":{"total":len(checks),"passed":len(checks)-len(failures),"failed":len(failures)},"failed_checks":failures,"selected_jobs":jobs,"supplemental_evidence":supplements,"responsibility_counts_as_reviewed":summary,"findings":[],"scope_limits":["Both responsibility reviews are distinct LLM-agent source/trace reviews, not human reviews.","Twelve selected functional jobs each preserve 21 recorded-port cases. Replay retains original semantic failures and is not a new semantic accuracy measurement.","M04 selected code paths pass scripted functional and actual local lineage probes, while the separate actual-Qwen natural-language extension probe FAILS for both A and B. New natural-language end-to-end feature success is therefore not established; its failures remain separate from the reviewed responsibility count.","M05 fresh processes exercise real local owner token/presentation/snapshot/restore/ANSWER paths with specified model ports, not real-model correctness or full durable product restart.","M06 has actual zero source/patch delta on the measured baseline, complete use-fence regressions and exact S1-03 preservation before/after. The common pre-freeze baseline repair is not counted as M06 new-feature cost."],"evidence_sha256":files}
(O/"qa05-selection-independent-checks.json").write_text(json.dumps(checks,ensure_ascii=False,indent=2)+"\n")
(O/"qa05-selection-independent-review.json").write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")
lines=["# QA05 선택 원장 독립 검수","",f"상태: **{report['status']}**. UTC {report['generated_utc']}. API 0회, 원본 변경 0개.","","선택 manifest의 reviewed-result 12개와 기존 `selected_change_records`가 반환하는 12개가 일치한다. hash로 지정된 두 LLM agent의 책임 검토, 변경 전/후 53개 source, patch, 원시 result와 회귀를 대조했다. 이 검수의 작성자는 변경 patch 작성자이므로 기존 두 독립 리뷰의 개수 판정에 추가 투표하지 않았다.","","| 방안 | M01 | M02 | M03 | M04 | M05 | M06 | 합계 | 평균 |","| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
for v,x in summary.items():
    lines.append("| "+v+" | "+" | ".join(str(n) for n in x["per_task"])+f" | {x['sum']} | {x['mean']} |")
lines += ["",f"대조 {len(checks)}개 중 실패 {len(failures)}개. 같은 baseline에서 독립 생성된 12개 변경 사본이며, 초기 result의 AWAITING 상태와 count=None은 보존되고 새 reviewed-result만 선택됐다. 원래 M04 사본과 revision collision 실패는 보존됐고 선택/집계에 중복되지 않는다. Typed B의 binder b.py는 baseline과 byte 단위로 동일하다.","","M04는 수정 코드의 지정 모델 port 기능 검사, 성공/실패 두 분기, AFTER/USE_RESULT 및 실제 Request revision/Task/Execution lineage 회귀가 통과했다. 별도 실제 Qwen 신규 자연어 probe는 A/B 모두 **FAIL**이다. manifest와 두 책임 리뷰가 이를 별도 실제 모델 실패로 유지하며 P21 정확성/비용에 더하지 않는다. 따라서 평균 1.5개는 검토한 기능 책임 변경 수이며 신규 자연어 기능의 실제 모델 end-to-end 성공을 뜻하지 않는다.","","M05는 실제 새 Task 질문 owner가 발급한 token을 실제 게시하고 snapshot한 뒤 다른 PID의 fresh process로 같은 token을 복원했다. A/B 모두 actual ANSWER 명령 1회, 손상 token/중복 거절을 확인했다. generic snapshot/restore를 그대로 쓰므로 U14를 추가하지 않은 기존 두 리뷰와 일치한다.","","M06은 before/after source 53개와 빈 patch가 완전히 같다. 원본/파생물/대기 후보의 사용 차단, 정직한 처리 receipt, 다른 자료/선호 유지, cache/late publication/consent 복원 회귀가 통과했다. exact S1-03 입력의 C1~C6과 사용자 partition/원본 M언어/owner ref+revision/철회 삭제를 A/B baseline과 unchanged job source에서 모두 확인했다. API 0회의 지정 모델 출력 기능 증거이며 실제 모델의 자유문 정확성이나 P21 모집단 추가로 사용하지 않았다.","","각 파일의 SHA, reviewer ID, patch/source digest, 역할별 증거 한계와 PID/token hash는 JSON에 보존했다. 확정 P1/P2는 없다."]
if failures:
    lines += ["","실패 대조:",""]+[f"- {x['check']}: {x['evidence']}" for x in failures]
(O/"qa05-selection-independent-review.md").write_text("\n".join(lines)+"\n")
print(json.dumps({"status":report["status"],"checks":report["check_summary"],"counts":summary,"failures":failures},ensure_ascii=False,indent=2))
