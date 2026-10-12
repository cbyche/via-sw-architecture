"""Prepare a new immutable campaign and separate post-timing evaluator copy."""
import datetime
import hashlib
import json
from pathlib import Path
import shutil
from benchmark.architecture.dp41_45_poc.campaign import freeze, snapshot
ROOT = Path.cwd()
PREPARATION = Path(__file__).resolve().parents[1]
CAMPAIGN = PREPARATION.parent / "dp41-p21-mac-text-v6-20261012"
PREVIOUS = PREPARATION.parent / "dp41-p21-mac-text-v5-20261012"
manifest = freeze(CAMPAIGN)
tools = CAMPAIGN / "evaluator-tools"
shutil.copytree(CAMPAIGN / "frozen-source", tools)
hashes = snapshot(tools)
assert hashes == manifest["frozen_source_hashes"]
tool_manifest = {
 "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
 "source_digest": manifest["frozen_source_digest"],
 "generation": "SAME_AS_MEASURED_V6_SOURCE", "tool_directory": str(tools),
 "baseline_source_digest": manifest["frozen_source_digest"], "file_hashes": hashes}
(CAMPAIGN / "evaluation-tool-manifest.json").write_text(json.dumps(tool_manifest, indent=2)+"\n")
(CAMPAIGN / "evaluation-tools-current.json").write_text(json.dumps({
 "directory": str(tools), "manifest": str(CAMPAIGN / "evaluation-tool-manifest.json"),
 "source_digest": manifest["frozen_source_digest"]}, indent=2)+"\n")
control = CAMPAIGN / "evaluation-control"
control.mkdir()
for filename in ("run-after-workers.py", "verify-evidence.py", "host-hardware.json", "host-observations.json"):
 shutil.copy2(PREVIOUS / "evaluation-control" / filename, control / filename)
reviews = CAMPAIGN / "reviews"
reviews.mkdir()
method = (PREVIOUS / "reviews/measurement-method.md").read_text().replace("MAC-TEXT-v5", "MAC-TEXT-v6").replace(
 "50개 파일", str(len(hashes))+"개 파일").replace(
 "2143b39761bd0b74e8d9d973d8b59c1e8ba722ccc24ebafcc0d7e03d85cda4b0", manifest["frozen_source_digest"])
method += "\nV5에서 실제 QUESTION_PRESENT 전달 뒤 지정된 사용자 답이 연결되지 않은 공통 입력 포트를 보완했다. 현재 질문·revision·Task/Execution·실제 전체 전달과 사용자 질문 주제 조건을 확인한 뒤 원래 지정 전사를 새 Request에 제공한다. 부적합 제시에는 답을 주지 않고 실제 답이 소비된 질문만 중복을 차단한다. B의 실제 QUESTION_PRESENT 참조는 현재 유효 pending 질문을 허용하며 ANSWER_QUESTION은 실제 제시 조건을 유지한다. A/B 프롬프트·입력·정답·지원 연산은 보존했다. V5 원시는 보존하고 최종 모집단에서 제외한다.\n"
(reviews / "measurement-method.md").write_text(method)
preflight = PREPARATION / "v6-question-followup-preflight"
preflight.mkdir()
old = (PREPARATION / "v5-query-label-preflight/run.py").read_text()
old = old.replace("mac-text-v5-", "mac-text-v6-").replace("v5-query-label-preflight-", "v6-question-followup-preflight-").replace("S2-02,S2-05", "S6-01,S6-02")
(preflight / "run.py").write_text(old)
record = {"status":"FROZEN_PENDING_ACTUAL_PREFLIGHT", "source_digest": manifest["frozen_source_digest"],
 "files":len(hashes), "control_script_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
print(json.dumps(record), flush=True)
