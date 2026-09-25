# VIA 후보 공통 계약

> 적용 대상: VIA-DP-01~18. 후보 설계 초안이며 새 제품 구현·측정 결과가 아니다. 이전 네 후보의 ledger와 참조값은 historical 사본에 보존했고 전체 18개에 자동 확장하지 않는다.

## 1. 비교 불변식

S2S 모델 1개와 semantic LLM 1개를 공유한다. 역할별 prompt·세션·호출은 모델 추가 적재가 아니다. 같은 목표·원천 데이터·정보 시점·Agent capability·모델 profile·총 worker budget을 A/B에 적용한다. 새 ASR/VAD/TTS/helper 모델은 허용하지 않는다. 고정 모델의 기능이 부족하면 capability 미충족으로 기록한다.

한 번에 하나의 DP만 바꾸고 나머지 선택을 명시적으로 고정한다. 이전의 A/A/A/A 참조값을 자동 적용하지 않는다. 주요 상호작용은 별도 교차 검증하며 후보 결과를 독립 표본처럼 복제하지 않는다. Model 배치는 동일 조건이며 외부 Agent Runtime은 양쪽 모두 외부 dependency다.

## 2. Identity·메시지·상태

| 계약 | 보존할 내용 | 금지할 오해 |
| --- | --- | --- |
| 입력 | conversation/turn/request identity, source time·observed time, 원문·전사 revision | partial을 final로 확정하지 않음 |
| 의미 판단 | source version·근거·요청 관계·handling·정정 provenance | SemanticDecision은 실행 권한이 아님 |
| Task command | task ID, command ID, expected revision, 승인·action revision | 취소 요청을 외부 취소 완료로 표시하지 않음 |
| Agent 연결 | execution ID, provider run ID, submission key, attempt·active 여부 | transport retry와 새로운 업무 의도를 혼동하지 않음 |
| Agent 관측 | source ID·revision·event ID·capability·실제 상태·질문/승인 identity | 지원하지 않는 replay/revision을 VIA가 만들어내지 않음 |
| 응답 | request/task/result ID, generation, 출력 offset·interrupted 상태 | 생성한 답과 실제 사용자에게 전달한 답을 구분 |
| 증거 | schema version, trace/event/causation ID, 시간·입출력 digest·판정 근거 | trace ID는 상태 변경 권한이 아님 |

최종 writer·응답 권한·관측 확정·복구 기록은 각각 VIA-DP-14/04/15/08에서 정한다. 이름이 비슷하다는 이유로 한 축의 선택이 나머지까지 결정하지 않는다.

## 3. Durable handoff와 실패

Task owner가 intent·submission key·outbox를 짧은 transaction으로 남기고 EffectDispatcher가 transaction 밖에서 Agent를 호출한다. 접수 확인 후 owner가 ExecutionLink를 갱신한다. 메모리 queue 수락은 디스크 commit이나 Agent 접수가 아니다.

접수 직후 crash가 발생하면 Agent의 실제 조회·중복 억제 capability 범위 안에서만 복구한다. 확인 불가한 외부 Action을 자동 재발행하지 않는다. Outbox만으로 외부 exactly-once를 보장하지 않는다. 네트워크·LLM 응답을 기다리며 DB transaction 또는 Task lock을 잡지 않는다.

VIA-DP-14의 기존 부분 코드는 양쪽 모두 동일 SQLite WAL/FULL 저장 조건이며 write mutex도 공통이다. per-Task owner는 자동 DB 병렬성이나 OS 장애 격리를 뜻하지 않는다. 다른 DP의 저장 구조는 그 비교에서 별도 고정한다.

음성 interruption은 출력 generation을 중지한다. 명시적 Task 제어 없이 외부 업무를 취소하지 않는다. 늦은 결과는 request/task/execution revision과 현재 권한을 확인한 후 반영한다.

## 4. QA·요소 집계·증거

현행 [QA catalog](../08-quality-attributes/quality-model.md)·[event endpoint](../11-measurement/event-boundary-contract.md)·[요소 집계](../10-element-definition.md)·[평가 방법](evaluation-method.md)을 따른다. 19개 QA를 모두 검토하되 물리적으로 비참여인 경로에 인과를 만들지 않는다. QA-41은 자원 확인이며 예산·유의미한 차이 근거 없이 주요 ASR로 삼지 않는다.

그림의 상위 박스·Task 인스턴스·메서드를 추가 Architecture Element로 세지 않는다. 전체 변경 pack과 동결된 element ledger를 작성하기 전 QA-21~23의 수치 우세를 확정하지 않는다. 현재 부분 코드는 구현 근거의 일부일 뿐 제품 E2E·실제 모델·현재 QA 실측 증거가 아니다. 모든 새 QA 결과는 NOT_RUN이다.

## 5. 탐색

[전체 보고서](README.md), [매핑](./dp-executive-summary.md#legacy-mapping), [공통 검토 절차](dp-review-protocol.md). 이전 계약·element ledger 원문은 docs/archive/dp-review-pre-inventory-2026-09-25/candidates/common-contract.md에 보존했다. 현재 계약을 대신하는 규범 근거로 사용하지 않는다.
