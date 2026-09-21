# 12-02. Gate 2 — Core 6개 DP의 후보 구조 리뷰

> **G2-DESIGN-v1 / 2026-09-22 / Gate 2 리뷰본 작성 완료·사용자 승인 전**
> Git source snapshot: `cf21c7361d392b30ad95cd8ec48a7d97d847a19b`.
> Gate 1의 Core 6 / Supporting 3, Working ASR 12개, UC18·variation94·Change24 유지.
> **목적: 합리적인 두 구조의 trade-off를 검증할 수 있는 후보를 동결하는 것. 아직 점수·승자·최종 Architecture 없음.**

## 1. 무엇을 검토하는가

이제 family 이름만 비교하지 않는다. 각 DP에 대해 **실제 책임 위치, 소비 계약, 상태 writer, 정상/오류/재시작 경로, C/I/S/D ID, 공통 보완책, 측정 지점, 동점 가능성**을 작성했다. Candidate pair의 비교 범위를 한정하되, 실무에서 hybrid가 존재한다는 이유로 무조건 금지하지 않는다.

| DP 상세 문서 | A | B | 점수 전에 확인할 핵심 |
|---|---|---|---|
| [INT-DP01](./12-gate2/INT-DP01.md) | Voice가 음성 direct release | Core가 Voice/Text release 통합 | B에 불필요한 LLM/IPC를 강제하지 않았는가 |
| [IR-DP01](./12-gate2/IR-DP01.md) | 통합 semantic output | 3개 독립 semantic stage | B도 원문·provenance·수정 경로를 갖는가 |
| [TASK-DP01](./12-gate2/TASK-DP01.md) | stateless transaction handlers | durable Task별 writer/mailbox | A도 Task별 병렬 준비, B도 같은 DB 비용을 갖는가 |
| [AGENT-DP01](./12-gate2/AGENT-DP01.md) | edge가 수명 차이를 정규화 | Core typed handler가 해석 | 두 안 모두 동일 기능 보존·미지원 보고를 하는가 |
| [TASK-DP02](./12-gate2/TASK-DP02.md) | query 완료가 정상 갱신 trigger | event가 정상 갱신 trigger | 같은 Agent truth와 조회 보완을 사용하는가 |
| [EXEC-DP01](./12-gate2/EXEC-DP01.md) | 동일 process 내 논리 격리 | integration worker process 격리 | 같은 adapter code·fault·총 resource budget인가 |

전체 후보는 [공통 계약과 Element 원장](./12-gate2/common-contract.md)과 각 DP 선택의 합집합이다. 다른 DP의 참조 선택은 시험을 위한 좌표일 뿐 최종 승자가 아니다. **12개 pair-labelled 후보는 7개 고유 configuration**으로 조립되며 동일 reference 실행을 중복 실적으로 세지 않는다.

## 2. 이 단계에서 강화한 부분

### 실제 비교 경로

INT는 'Core box를 하나 지나면 느림'을 가정하지 않는다. IR은 단순 stage 수가 아니라 실제 prompt/출력·재조회·critical path를 비교한다. TASK는 중앙 서비스와 actor에 서로 다른 저장소를 주지 않는다. AGENT는 A의 capability를 공통분모로 잘라 B의 기능을 유리하게 만들지 않는다.

TASK-DP02에서 query/event는 진실의 서로 다른 소유자가 아니다. Agent가 실행 사실을 소유하며, 두 후보는 그 사실을 VIA에 반영하는 정상 경로가 다르다. EXEC는 같은 연동 코드가 어디서 실행되는지를 바꾸고 실제 local abort의 host 경계를 관찰한다.

### 유지한 제품 경계

Voice Connection/Conversation/Request/Task/Agent Execution은 계속 구분한다. S2S Direct Response도 기록하며, 기존 Task로 이어지는 follow-up과 compound 네 관계를 유지한다. User Memory·restart·승인은 후보가 직접 담당하고 시험기가 정답 state를 보충하지 않는다.

MODEL/STATE/ORCH를 새 주요 DP로 만들지 않는다. Supporting 3개는 공통 참조 구현으로 유지하고, 이 선택에 결과가 민감하면 후속 교차 확인 대상으로 남긴다. 원천 요구·metric/target·0~5 band를 바꾸지 않는다.

## 3. 예상 trade-off와 반증 조건

| DP | 우선 확인할 W-ASR | 확인하려는 trade-off | 동점/반대 결과도 인정 |
|---|---|---|---|
| INT | W-01 대화 / W-02 인계 / W-08 변경 | 직접 release 경로와 공통 route 관리 범위 | Core inline 경유면 지연 차이가 작을 수 있음 |
| IR | W-01 / W-02 / W-05 충실도 / W-08 | joint evidence·호출 경로와 단계별 변경·수정 | 통합 prompt/repair가 더 비싸질 수도 있음 |
| TASK-01 | W-02 / W-04 동시성 / W-08 / W-09 복구 | transaction coordination과 Task별 ownership | 같은 DB 병목이면 동시성 동점 가능 |
| AGENT | W-02 / W-03 피드백 / W-07 교체 / W-08 | edge 적응 책임과 Core capability 해석 | 공통 native client만 바뀌면 변경량 동점 가능 |
| TASK-02 | W-03 / W-04 / W-07 / W-09 | query 대기·부하와 stream 복구·순서 계약 | 짧은 polling/동일 reconciliation이면 차이가 작음 |
| EXEC | W-01 / W-04 / W-09 / W-10 격리 | 직접 호출 비용과 fatal fault 영향 범위 | 빠른 재시작이면 W-10도 둘 다 100% 가능 |

이 표는 Primary 선정이나 성능 결과가 아니라 **사전 가설**이다. 12개 전수 raw metric을 보존하고 평균·모델 성능을 p95 실측처럼 표시하지 않는다. 발표 문구는 실제 trace/변경 원장이 뒷받침할 때 확정한다.

## 4. 상태와 실행 가능성

**설계 리뷰 가능과 실제 benchmark 실행 가능은 다르다.** 이번에 준비한 것은 6개 pair의 상세 구조, 전수 Element 조립, 12개 W와 24 change의 빈 원장, 문서·구조 검증 도구다. 실제 Rust 후보, Windows 계측, 모델 inference는 아직 수행하지 않았다.

특히 base repository에는 11-D가 참조하는 `working12/baseline.json`이 없고, 실제 S2S 입력 전사/시각 API와 모델 실행 digest도 미확인이다. 이런 공백을 0점이나 성공으로 채우지 않았다. 상세 [근거·실행 준비 원장](./12-gate2/evidence-and-readiness.md)에 담당 작업과 확인 조건을 남겼다.

검증 도구: `python benchmark/rebaseline/gate2/catalog_check.py --output results/gate2-review`.
이 명령은 설계 ID/조합/coverage와 빈 원장을 검증·생성하며 candidate 성능을 측정하지 않는다.

## 5. 사용자 Gate 2 리뷰 포인트

**① 두 안 모두 실제 채택할 만하게 설계됐는가, ② 결정 범위와 공통 보완책이 공정한가, ③ 중요 ASR에 대한 구조 인과가 납득되는가**를 확인한다. Component 이름 변경만 있는 비교나 손쉽게 동일해지는 두 안은 이 단계에서 고친다.

승인 전에는 점수를 산출하지 않는다. 승인 후에는 먼저 실행 준비 공백을 닫고 고정된 후보/자산 revision으로 12-A 전수 sensitivity sweep을 수행한다. 최종 선택·weakness·tactic은 그 결과 이후다.
