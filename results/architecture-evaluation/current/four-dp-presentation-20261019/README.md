# 네 DP 발표용 추가 증거

2026-10-19 / 추가 실행과 수치 집계 완료

이 디렉터리는 [실행 전 계약](../../../../docs/architecture/12-decisions/decision-packages/seven-problem-review/16-presentation-selection-contract.md)에 따른 후속 비교다. 기존 `four-dp-qwen37-review-v2-20261018` 기본120회와 과거 실패를 교체하지 않는다. 선택 해석의 정본은 [17 결과](../../../../docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md)다.

## Git 보존과 이전 재생 입력

사용자가 이번 발표자료와 관련 코드·문서·실험 결과의 commit/push를 승인했다. 원시 JSON/JSONL·CSV·LOG, 실패, frozen source/binary를 원래 경로와 bytes 그대로 직접 보존하며 실제 발행 상태는 Git 이력으로 확인한다. 요약 보고서를 읽는 데 API 호출이나 복원은 필요 없다.

A3/C2/B1의 재생과 이전 기본 수치 추적이 참조하는 과거 실제 응답까지 확인하려면 [기존 증거 패키지](../four-dp-poc-evidence-20261019/README.md)의 절차로 두 이전 캠페인을 먼저 복원한다. 이번 원시의 절대경로는 보존했으므로 다른 clone 위치에서 기존 실행·재분석 명령이 자동 재배치된다고 보장하지 않는다. 과거 패키지의 portable 명령은 그 두 캠페인 범위이며 이번 발표 실험 전체를 재실행하는 명령이 아니다.

## 증거별 진입점

| 저장소 DP / 발표 번호 | 수행 범위 | 먼저 읽을 자료 |
| --- | --- | --- |
| 01 / DP1 | 기존 첫 호출 분기 추적, E3 공통 출력 정책의 실제 모델18회, A3 보존 예산의 재생과 native 검사 | [발표 해석](dp01/presentation-report.md), [fresh18](dp01/fresh-summary.json), [본문 대조](dp01/content-review.md), [계획](dp01/plan.json), [첫 호출 추적](dp01/old-first-call-paths.json), [재생](dp01/replay-summary.json), [native](dp01/native/summary.json) |
| 02 / DP2 | B1 불변 입력 공유의 기존 응답 재생. N=1/2/4, 총72회 | [순서 제어 결과](dp02/strict-control-v2/report.md), [집계](dp02/strict-control-v2/summary.json), [동결 계획](dp02/strict-control-v2/plan.json) |
| 04 / DP3 | pixel 계약의 기본 A/B 24회와 A2+A5/B4 24회. 공통 정규화 계약 A/B는 별도24회 | [48회 비교](dp04/comparison.md), [정규화24회](dp04/normalized-comparison.md), [실패 원값](dp04/failure-evidence.json), [좌표 계약 검토](dp04/coordinate-contract-review.md), [발표 근거](dp04/presentation-brief.md) |
| 05 / DP4 | C1의 실제 모델 비교 E3×C/C1×3. C2의 기존 수신자 읽기 재생12쌍 | [발표 해석](dp05/presentation-report.md), [fresh6](dp05/fresh-summary.json), [메커니즘](dp05/mechanism-breakdown.json), [첫 안내](dp05/prefix-review.json), [본문](dp05/content-review.json), [독립 메일6](dp04/independent-dp05-prefetch-mail-review.md), [계획](dp05/plan.json), [재생](dp05/replay-summary.json) |

## 서로 합치지 않는 관측

- **실제 모델 실행:** 동결한 Qwen 요청, 원 응답, 사용량, 실패와 callback·종료 시점을 보존한다. 사용량이 없는 공급자 실패는 비용0으로 처리하지 않는다.
- **실제 응답 재생:** 기존 모델 응답을 사용해 입력·결과 동일성과 로컬 자료 구조의 자원 비용을 비교한다. API0이나 재생 시간을 새 Qwen 비용·지연 개선으로 해석하지 않는다.
- **native 검사:** 소유 할당·직렬화 상태·원문 재읽기와 프로세스 RSS를 구별한다. 모델 품질이나 물리 디스크 I/O를 측정한 결과가 아니다.
- **본문 진단:** host 종료, 지칭 집합 QA02와 사용자 조건의 의미 충족은 별도 판정이다. host 종료가 관측돼도 본문 조건 누락이 남을 수 있다.

DP02의 첫72회는 [이전 재생 결과](dp02/report.md)에 보존했다. 일부 동시 실행의 mutable snapshot 순서가 달라 순서를 제어한 새72회를 정본으로 사용한다. DP01/05의 API 전 준비 오류와 freeze 판본도 삭제하지 않는다. 화면의 pixel48과 정규화24는 서로 다른 표현 계약의 비교이며 성공 행을 골라 합치거나 이전 실패를 재채점하지 않는다.

## 모델, 비용과 범위

모델은 기존 비교와 같은 `qwen3.7-plus-2026-05-26`이다. 비용은 비교 계약의 고정 환산율인 입력 $0.40/M token, 출력 $1.60/M token으로 계산한다. 현재 판매 가격에 대한 안내가 아니다. 전체 추가 실행 관리 상한은 $1.50이며 [공통 원장](budget.json)에 호출별 확정 사용량과 예약·미확정 비용을 남긴다. 준비·복구·실패와 수신자 호출을 포함한다.

[FINAL 집계](budget-summary.json): 실제 모델96episode, 331호출. 사용량 확정330호출$0.4735528, 미확정1호출의 보수적 예약 상한$0.0729404, 예산 집계$0.5464932다. 진행 예약은0이며 실제 청구액을 뜻하지 않는다. native와 실제 응답 재생은 추가 API0이다. DP05의 공통 첫 함수라도 입력이 다른 사실은 [첫 요청63개 대조](dp05/basic-prefix-boundary-audit.md)에 보존했고 QA01을 구조적 동점에서 우열 미확정으로 정정했다.

실제 음성, 상용 Agent, Windows 제품 끝단과 on-device Omni 가중치는 이번 측정 범위에 없다. 메모리는 해당 표에 명시한 로컬 프로세스와 입력의 범위이며 전체 VIA 메모리로 확대하지 않는다. 이 자료의 완성은 accepted ADR 변경이나 외부 발행을 의미하지 않는다.
