# VIA SW Architecture — Gate 1 Review

> **리뷰 목적: 비교할 구조 질문을 승인하는 것. 후보의 점수나 승자를 고르는 단계가 아닙니다.**
> W12-G1-v1 / Working ASR 12개 전수 유지 / UC18·variation94·Change24 유지.

## 1. 제안하는 DP 목록

| DP | 이번에 결정할 구조 질문 | 비교할 family |
|---|---|---|
| **DP-01 음성·Core 처리 중계** | S2S의 직접 응답과 VIA Core 처리를 어느 중계 구조에서 조정할 것인가? | Voice-led mediation / Core-led mediation |
| **DP-02 Context 조회·구체화** | Context Source의 실제 내용을 누가, 언제 읽어 어떤 표현으로 소비자에게 제공할 것인가? | Broker-materialized context / Handle-led demand resolution / Typed hybrid |
| **DP-03 요청 해석 단계 구성** | Referent·Request·Task·처리 경로·Agent 판단을 어떻게 묶거나 나눌 것인가? | Joint interpretation / Staged revisable pipeline / Grouped interpretation |
| **DP-04 과거 대화·업무 맥락 제공** | 보존한 Conversation/Task 기록 중 무엇을 현재 Model 입력으로 구성할 것인가? | Canonical context reconstruction / Indexed retrieval with provenance / Summary plus selective replay |
| **DP-05 제한된 정보 요청 처리 위치** | VIA가 직접 처리해도 되는 Read/Search/Understand를 Core와 Agent 중 어디에 배치할 것인가? | Core bounded service / Downstream bounded delegation |
| **DP-06 업무 상태 관리·재시작 복구** | VIA Task의 상태와 실행 연결을 어느 단위가 관리하고, 재시작 때 어떻게 복원할 것인가? | Shared transactional supervision / Durable per-task supervision |
| **DP-07 Agent 계약 통합** | 서로 다른 Agent의 capability·실행·질문·결과 계약을 어디서 어떻게 정규화할 것인가? | Capability-specific integration gateways / Canonical agent port with adapters / Stable kernel with typed extensions |
| **DP-08 업무 이벤트 수집·전달** | Agent 상태·질문·결과를 언제 수집하고 사용자 전달까지 어떤 event 경로를 둘 것인가? | Scheduled reconciliation / Streaming intake with reconciliation |
| **DP-09 권한·동의·정보 반출 통제** | 접근/반출 허용과 pending Action 승인을 실제 사용 경계에 어떻게 연결할 것인가? | Central mediation gateway / Distributed boundary guards / Scoped capability enforcement |
| **DP-10 실행 스케줄링·장애 격리** | Voice/Core/연동 작업을 어떤 queue·worker·process 경계에 배치해 간섭과 장애 전파를 제한할 것인가? | Single-process partitioned async runtime / Interaction/core and integration workers / Per-provider fault domains |

## 2. 핵심 논리

**DP-02/03/04**는 필요한 정보를 어떻게 확보하고 의미 판단에 넣는가, **DP-06/07/08**은 업무 상태·Agent 계약·feedback을 어떻게 연결하는가, **DP-01/05/09/10**은 음성 중계·bounded 처리 위치·통제·실행 배치를 결정합니다.

잘못된 기록 누락이나 지능이 낮은 모델을 한 후보에만 주어 차이를 만들지 않습니다. 정상적인 대안들에서 실제 latency, 변경 전파, recovery, 노출 범위가 달라지는지 관찰합니다. Completion/Continuity/Safety/Failure containment가 모두 동일하면 그 결과도 유지합니다.

10개는 설계 전수 검토용 목록이며 발표 본문 10장/10개 DP를 강제하지 않습니다. 실제 근거가 있는 trade-off를 본문에 확대하고 전체 coverage를 부록에 보존합니다.

## 3. 선행작업에서 정리한 점

- 세 responsiveness의 endpoint를 분리했습니다. 특히 **Agent accepted와 VIA의 복구 가능한 기록을 모두 확보한 때**를 handoff 완료로 정의했습니다.
- 12개 모두 metric·공통 분모·목표 제안·0~5 band가 있습니다. 2/3 changed-elements 목표는 기존 사용자 승인값입니다. 새 target은 승인값으로 위장하지 않았습니다.
- 동시성은 idle Task 수가 아니라 같은 rate의 background event 부하로 시험하고 절대 latency와 비율을 함께 보존합니다.
- Recovery는 복구 가능한 조건의 시간, Failure containment는 장애 중 무관 기능 유지, Safety는 0/24 목표로 분리했습니다.
- 공개 model 평균속도/234ms theoretical packet을 VIA p95로 쓰지 않습니다. 실제 model accuracy와 candidate 성능은 모두 미측정입니다.

## 4. 이번 리뷰 질문

| 검토할 것 | 현재 제안 |
|---|---|
| 질문의 누락/과다 | 25개 주제를 분류해 주요 DP 10개. 나머지 필수 기능/tactic/구현 선택은 owner DP에 연결. |
| boundary 중복 | DP-02 Source→Context / DP-04 기록→Model / DP-09 access/egress enforcement를 분리. DP-06 state owner / DP-08 feedback path / DP-10 runtime도 분리. |
| 다음 상세화 순서 | DP-02/03/04 묶음부터, 다음 DP-06/07/08. 서로의 winner를 미리 전제하지 않고 결합 효과를 확인. |

## 5. 결과 상태

**후보 선택 없음. 후보 성능값 없음.** 검증된 것은 문서/기준/계산도우미와 coverage입니다. Gate 1 승인 뒤 상세 candidate, C/I/S/D 목록, 공통 runtime/API 및 관측 자산을 만들고 **점수를 보기 전 Gate 2**에서 리뷰합니다.

상세: [Master Catalog](./12-01-dp-master-catalog.md) · [Coverage](./12-01a-scope-and-coverage-ledger.md) · [12개 평가 계약](./11d-working-12-measurement-and-scoring.md).