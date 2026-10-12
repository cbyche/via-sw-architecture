"""Prepare a new immutable campaign and separate post-timing evaluator copy."""
import datetime
import hashlib
import json
from pathlib import Path
import shutil
from benchmark.architecture.dp41_45_poc.campaign import freeze, snapshot
ROOT = Path.cwd()
PREPARATION = Path(__file__).resolve().parents[1]
CAMPAIGN = PREPARATION.parent / "dp41-p21-mac-text-v7-20261012"
PREVIOUS = PREPARATION.parent / "dp41-p21-mac-text-v6-20261012"
manifest = freeze(CAMPAIGN)
tools = CAMPAIGN / "evaluator-tools"
shutil.copytree(CAMPAIGN / "frozen-source", tools)
hashes = snapshot(tools)
assert hashes == manifest["frozen_source_hashes"]
tool_manifest = {
 "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
 "source_digest": manifest["frozen_source_digest"],
 "generation": "SAME_AS_MEASURED_V7_SOURCE", "tool_directory": str(tools),
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
method = (PREVIOUS / "reviews/measurement-method.md").read_text().replace("MAC-TEXT-v6", "MAC-TEXT-v7").replace(
 "51개 파일", str(len(hashes))+"개 파일").replace(
 "bcf3802bfad401059554d288b5f7d303ece0e16743fb43b62eb57cebbc075636", manifest["frozen_source_digest"])
method += "\nV6 사전 검증에서 확인한 기존 REGISTER_MEMORY의 공통 User Memory 조회 및 등록 receipt 응답 누락을 보완했다. 실제 Goal key/value/scope를 현재 사용자 owner 원본에 게시하고 공통 read/revision/권한/철회 경계를 연결한다. response=None일 때 실제 저장 receipt로 공통 응답을 생산한다. A/B 의미 알고리즘, Frame, 프롬프트, 입력과 정답은 유지했다. 같은 exact oracle helper의 before 실패와 after A/B 10조건 통과, 독립 리뷰를 보존했다. V6의 formal worker 및 QA05 job은 0이며 사전 자료는 최종 모집단에서 제외했다.\n"
method += "\n공통 provider 장애로 usage가 없는 경우 ASR-QA-v6 §3.5에 따라 해당 P21 A/B 활동 쌍의 전체 raw를 hash와 함께 보존하고 같은 동결 입력을 최대 3번 재실행한다. 의미/enum/스키마/지원 범위/후보 timeout 실패에는 이 경로를 적용하지 않는다. data_inspection_failed는 완전히 동일한 API Context의 정상 HTTP200/known usage 재전송 증거가 있을 때만 일시 장애로 분류한다. 별도 service health 응답은 후보에 채택하지 않으며 비용과 raw도 본 QA04와 분리한다.\n"
(reviews / "measurement-method.md").write_text(method)
preflight = PREPARATION / "v7-final-source-preflight"
preflight.mkdir()
old = (PREPARATION / "v6-question-followup-preflight/run.py").read_text()
old = old.replace("mac-text-v6-", "mac-text-v7-").replace("v6-question-followup-preflight-", "v7-final-source-preflight-").replace("S6-01,S6-02", "S6-01")
(preflight / "run.py").write_text(old)
for filename in ("v7-controller-integration-review.md", "v7-memory-integration-review.md", "v6-integration-review.md"):
 shutil.copy2(PREPARATION / filename, reviews / filename)
record = {"status":"FROZEN_PENDING_ACTUAL_PREFLIGHT", "source_digest": manifest["frozen_source_digest"],
 "files":len(hashes), "control_script_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
print(json.dumps(record), flush=True)
