"""Append completed trace-review findings to the frozen renderer's report."""
from collections import Counter
import csv
import hashlib
import json
from pathlib import Path

CAMPAIGN = Path(__file__).resolve().parents[1]
MARKER = "## 실행 기록 교차 검수와 오류 해석"


def rows(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def run():
    final = CAMPAIGN / "final"
    completion = json.loads((final / "completion.json").read_text())
    if completion.get("status") != "COMPLETE" or completion["observed_trials"] != 210:
        raise ValueError("A complete final population is required")
    results = json.loads((final / "qa-results.json").read_text())
    review = json.loads((CAMPAIGN / "reviews/root-adjudication-completion.json").read_text())
    spend = json.loads((final / "spend-inventory.json").read_text())
    integrity = json.loads((final / "evidence-integrity.json").read_text())
    if integrity["status"] != "PASS":
        raise ValueError("Raw evidence integrity must pass")
    target = final / "report.md"
    original = target.read_text()
    if MARKER in original:
        raise FileExistsError("Do not append the same review twice")

    counts = Counter()
    examples = {}
    for directory in sorted((CAMPAIGN / "workers").iterdir()):
        for trial in rows(directory / "trials-unjudged.jsonl"):
            for event in trial["events"]:
                if event["kind"] not in ("dp41_output_rejected", "dp41_resolution_failed", "REQUEST_FAILED", "GOAL_FAILED"):
                    continue
                reason = event.get("reason") or event.get("detail") or event.get("error") or "NO_REASON"
                key = trial["variant"], event["kind"], str(reason)
                counts[key] += 1
                examples.setdefault(key, {"worker": directory.name, "case_id": trial["case_id"],
                                          "evidence_id": event["evidence_id"]})
    with (final / "failure-events.csv").open("w", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["variant", "event_kind", "reason", "event_count", "example_worker", "example_case", "example_evidence_id"])
        writer.writeheader()
        for key, count in sorted(counts.items()):
            example = examples[key]
            writer.writerow({"variant": key[0], "event_kind": key[1], "reason": key[2], "event_count": count,
                             "example_worker": example["worker"], "example_case": example["case_id"],
                             "example_evidence_id": example["evidence_id"]})
    semantic = {variant: rows(CAMPAIGN / "changes-semantic" / f"M04-{variant}-attempt1" / "change-semantic-validation.jsonl")[-1]
                for variant in "AB"}
    if any(record["status"] != "FAIL" for record in semantic.values()):
        raise ValueError("Review text must match the preserved actual semantic probes")
    extras = ["", MARKER, "",
        f"원래 OpenAI 판정은 그대로 보존했다. 별도 코드·실행 기록 검수에서 {review['proposals']}개 제안의 명시적 disposition을 기록했고, {len(review['accepted_proposals'])}개 제안을 {review['overridden_trials']}개 사례에 반영했다. 검수자는 LLM sub-agent와 root이며 사람 평가로 표현하지 않는다. 근거 ID와 실제 출력의 정확한 구절을 같은 validator로 다시 검사했다. [root 결정](../reviews/root-adjudication-decisions.json), [검수 완료](../reviews/root-adjudication-completion.json), [별도 최종 override](../evaluation/trace-adjudication-overrides.jsonl).", "",
        "원래 평가기의 publication 조회가 중단 영수증의 없는 publication_id를 직접 읽어 KeyError를 내는 결함을 실제 S4-02/S5-04에서 재현했다. 후보 53개 frozen 파일과 원래 평가기는 보존하고, 평가기 사본의 조회 두 곳만 `.get`으로 고친 뒤 누락 판정을 이어서 생성했다. 프롬프트·정답·경계·모델은 동일하며 8개 API 없는 회귀 검사는 정확한 발화 구절의 검증과 가짜 ID/미발화 구절/중단 publication의 거부를 확인했다. 이 오류와 판정의 재시도 비용은 후보 품질 실패에 넣지 않았다. [평가기 수정 manifest](../evaluation-tool-manifest-v2.json), [회귀 검사](../evaluation-control/validator-repair-tests.json), [진행 기록](../evaluation-control/stages.jsonl).", "",
        "이후 schema 재시도에서 source/Task/입력/인과 node의 실제 값 ID를 사건 evidence ID로 반복 인용하는 문제를 확인했다. V3는 첫 판정 프롬프트와 입력, 의미 판정 기준을 유지하고 재시도 안내에 유효하지 않은 인용과 그것을 포함하는 실제 제공 사건 ID를 명시했다. 코드가 의미의 PASS/FAIL이나 시간 endpoint를 정답으로 바꾸지는 않는다. 원래 출력과 모든 재시도 비용은 보존했다. [V3 manifest](../evaluation-tool-manifest-v3.json), [API 없는 참조 검증](../evaluation-control/evidence-feedback-repair-tests.json), [판정 원장별 provenance](../evaluation/evaluation-origin-manifest.json).", "",
        "남은 publication UUID/구절 오류에는 V4가 잘못된 필드의 위치, 실제 publication ID와 해당 실제 발화 문자열을 재시도 안내에 추가했다. 최초 SYSTEM/입력·기준·validator는 V3와 같고 코드가 의미 판정 또는 endpoint를 자동 바꾸지 않았다. 후보 재실행은 없다. [V4 manifest](../evaluation-tool-manifest-v4.json), [실제 구절 참조 회귀 검수](../evaluation-control/exact-emission-feedback-repair-tests.json).", "",
        "실패 여부는 실제 채택된 의미와 수행·전달에서 판정한다. 거절된 올바른 제안만으로 C1을 통과시키지 않고, 전달 문장의 오류를 발생하지 않은 외부 효과로 만들지 않는다. 기존 완료 Task의 이력을 현재의 신규 명령으로 세지 않는다. C2의 근거 오류와 C5의 실제 효과, C6의 checkpoint와 cycle의 최종 전달은 각각 해당 필수 사실을 확인한다. 정상적인 접수 뒤 완료는 모순이 아니다.", "",
        "QA02의 `wrong_first`는 잘못된 목표·payload·상태, 불필요한 확인 질문, 또는 같은 목표를 종결하는 답변의 필수 사실 누락을 뜻한다. 실제 올바른 목표의 정상 단계별 진행·접수는 다른 단계가 남았다는 이유만으로 오답으로 표시하지 않는다. 이때도 지정된 유효 핵심 반응이 없으면 `valid=null`, cap=5초이며 정답 첫반응 평균에서는 제외한다. 오답 뒤 정정은 최초 오답의 실측시간과 후기 정정시간을 별도로 보존한다.", "",
        "| 실제 오류 경로 | 관측과 해석 |", "| --- | --- |",
        "| A의 의미 출력 계약 | 선언하지 않은 meaning version, goal kind/operation, 잘못된 binding ID와 관계로 거절·수리가 발생했다. 모델이 선택·재해석하는 실제 반복 비용이며 A 구조가 그 목표를 표현할 수 없다는 증거는 아니다. |",
        "| B의 Frame 계약·해결 | input_ref 불일치, 미선언 preservation/question reference, 잘못된 goal/reference 필드와 예산 소진이 발생했다. 유한 코드 해결의 표현·검증 비용과 모델의 계약 준수 문제를 구분한다. |",
        "| 공통 처리·응답 경계 | 실제 접수만 말하고 필요한 Task/범위/보존 사실을 생략한 안내, 유효 질문의 잘못된 target으로 인한 거절은 실제 품질 실패다. 양안에 공통인 처리·응답 구현의 한계를 DP41 고유 구조의 한계로 확대하지 않는다. |",
        "| 내부 거절과 실제 금지 효과 | 내부 잘못된 후보가 차단된 것과 실제 잘못된 새 Task/Execution에 명령이 접수된 것을 구분한다. 필수 위반에는 실제 효과·연결의 증거만 사용한다. |", "",
        "[실패 사건 분류와 예시 근거](./failure-events.csv)는 사건 횟수다. 하나의 실패가 여러 거절/예산 사건을 만들 수 있으므로 이 합을 정확성 실패 조건 수로 사용하지 않는다. 정확성의 실제 실패 수는 conditions.csv에서 산출했다.", "",
        "## QA05의 코드 변경과 실제 새 문장 처리", "",
        "QA05의 책임 수는 같은 기능 계약을 수용한 실제 코드 diff, 지정 출력으로 수행한 새 분기·receipt·복원 검사, P21의 실제 기록된 의미 재생과 두 책임 검수에서 산출했다. 재생은 기존 성공·실패를 유지하는 회귀 검사이며 실제 모델의 새 의미 정확성을 검증한 것으로 합치지 않는다.", "",
        "M04는 코드가 선행의 실제 실패 뒤 초안을 한 번 실행하고 발송을 막는지를 검증해 양안 모두 4개 책임을 셌다. 추가로 새 자연어 입력을 실제 Qwen으로 실행한 결과는 **양안 모두 FAIL**이다. A는 producer의 실제 실행 상태를 잘못 참조하고 draft를 보존 대상으로 묶어 초안을 실행하지 못했다. B는 관계 방향/producer 참조를 잘못 생산해 Frame 거절 뒤 의미 해결을 끝내지 못했다. 이 실패를 반복해 성공 결과만 고르거나 코드 점수에 가려서 제시하지 않았다.", "",
        "| M04 증거 수준 | A | B |", "| --- | --- | --- |",
        "| 코드·실행 계약 probe와 책임 수 | PASS · 4개 | PASS · 4개 |",
        f"| 실제 새 문장의 Qwen 실행 | {semantic['A']['status']} · ${semantic['A']['provider_usage_cost_usd']:.6f} | {semantic['B']['status']} · ${semantic['B']['provider_usage_cost_usd']:.6f} |",
        "| 실제 원문 변경/메일 발송 | 0 / 0 | 0 / 0 |", "",
        "따라서 QA05의 수치 비교는 실제 코드 변경 전파 범위의 동점이다. 새 기능을 자연어부터 안정적으로 사용할 수 있다는 판정은 양안 모두 실패다. 낮은 변경 책임 수를 제품 준비 완료나 의미 품질의 통과로 읽지 않는다. [A 실제 신규 실행](../changes-semantic/M04-A-attempt1/change-semantic-validation.jsonl), [B 실제 신규 실행](../changes-semantic/M04-B-attempt1/change-semantic-validation.jsonl), [12개 선택 원장](../qa05-selection-manifest.json).", "",
        "## 메모리 차이의 원인과 선택 해석", "",
        "A의 전체 최대값은 A5/S2-02 부근에 나타났다. 이 시점의 30건 RECENT_TASK 조회와 20MiB 원문을 순위 계산 전에 문자열화하는 공통 조회 경로가 겹친다. 임시 문자열 할당이 peak에 기여했을 가능성은 실행 기록에 근거한 추론이며 object별 allocation을 추적해 확정한 원인은 아니다. 최대값과 활동별 peak 원값을 함께 유지했고 낮은 반복값으로 대체하지 않았다. 이 한 peak의 차이를 A 구조가 본질적으로 그만큼 RAM을 더 필요로 한다는 주장으로 이전하지 않는다.", "",
        "실제 선택은 높은 비용/메모리 점수로 정확성·유효 반응·완료 실패를 상쇄하지 않는다. 원값, 다섯 paired 차이, 보호 조건과 실제 실패의 종류를 함께 비교한다. 같은 코드 변경 범위 또는 실용 차이 기준 미충족도 측정 결론으로 인정한다. 현재 두 구현·고정 P21·실제 Qwen의 결과를 전체 VIA/모든 모델/모든 16개 조합의 우열로 일반화하지 않는다.", "",
        "## 전체 호출 비용과 재가공 안내", "",
        "본 P21 호출 비용과 준비·제외 실행·변경·judge 비용은 구분했다. [모든 실제 API 호출 CSV](./all-actual-api-calls.csv)와 [전 세대 비용 inventory](./spend-inventory.json)에 실제 usage, 알려진 비용과 usage 미확인 호출, 예산 예약액을 각각 보존한다. 무료 token 한도와 실제 invoice를 사용량 환산액과 혼동하지 않는다.", "",
        f"원시 증거 검증은 {integrity['frozen_files']}개 frozen source, 10개 활동, {integrity['model_calls']}개 본 실행·판정 호출 및 {integrity['verified_model_artifacts']}개 Context/출력 artifact를 검사해 PASS했다. [무결성 검수](./evidence-integrity.json), [raw 필드·경로별 재사용 안내](../raw-audits/raw-data-guide.md), [QA04/06 독립 검수](../raw-audits/qa04-qa06-independent-review.json), [QA05 선택 독립 검수](../raw-audits/qa05-selection-independent-review.json).", "",
        "V1~6은 본 수치에 섞지 않은 준비/제외 실행이다. V3의 큰 원시 JSONL 세 개는 원본을 로컬에 그대로 보존하고 Git에는 bytes가 동일하게 복원되는 gzip과 원본/gzip SHA-256 manifest를 게시했다. 다른 세대의 실패·비용 기록도 보존하며 준비에 든 비용을 지우지 않는다. [V3 대형 raw 보존 manifest](../../dp41-p21-mac-text-v3-20261012/git-large-raw-preservation.json).", ""]
    target.write_text(original.rstrip() + "\n" + "\n".join(extras))
    (final / "report-supplement-provenance.json").write_text(json.dumps({
        "status": "COMPLETE", "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "frozen_rendered_report_sha256": hashlib.sha256(original.encode()).hexdigest(),
        "completed_report_sha256": hashlib.sha256(target.read_bytes()).hexdigest(),
        "source_changes": 0, "candidate_calls": 0, "root_review_ref": "../reviews/root-adjudication-completion.json",
        "overall_M04_actual_natural_language": {variant: semantic[variant]["status"] for variant in "AB"},
        "formal_accuracy_qualification": {variant: results["variants"][variant]["qualification"]["qa01"] for variant in "AB"}
    }, ensure_ascii=False, indent=2) + "\n")
    print(target)


if __name__ == "__main__":
    run()
