# 2단계 — VIA가 지켜야 할 결과와 현재 선택한 해결 수단

> 상태: **STAGE_2_APPROVED / 사용자 검토 및 승인 완료** / 2026-10-01
> 사용자가 2026-10-01에 이 문서를 승인했다. 요구와 수단의 분류에 대한 승인으로, target 변경이나 새 DP 선정을 뜻하지 않는다.
> 입력: [1단계 P-01~16](./01-00-problem-coverage.md) / 절차: [00-workplan](./00-workplan.md)
> Target은 REVIEWED_BASELINE으로 보존한다. 이 문서는 요구와 설계 선택의 분류이며, 새 대안, DP 선정, ASR 변경, 구현, 측정 결과가 아니다.

## 1. 이번에 검토할 핵심

**대안은 VIA가 해줘야 할 일을 줄이지 않으면서, 그 일을 이루는 SW 구조를 바꿀 수 있어야 한다.** 예를 들어 “재시작 뒤 같은 업무를 이어간다”는 요구다. “현재 상태를 embedded DB에서 읽어 복구한다”는 target의 수단이다. 다른 저장 및 실행 구조를 탐색할 여지는 있지만, 복구할 수 있는 업무를 매번 사용자에게 다시 시키는 것은 같은 문제의 해결이 아니다.

이 문서는 16개 문제마다 **정상 결과 → 예외에서도 지킬 결과 → 현재 target의 실제 처리 → 바꿔 볼 구조적 지점 → 필요한 외부 지원**을 분리했다. 아직 방안 2를 정하지 않는다. ‘구조 자유도’는 다음 단계에서 탐색할 질문이며, 해당 구조 변경이 이미 타당하거나 DP로 선정됐다는 뜻이 아니다.

읽는 순서는 **§2의 공통 경계 → §3 요약표 → 관심 있는 P 상세**다. VIA 특성이 선명한 P-02, 04, 06, 08, 09, 10, 현재 on-device 조건에서 중요한 P-14를 먼저 봐도 좋다. P-01, 03, 05, 07, 11~13, 15, 16도 같은 기준으로 검토했다.

### 분류를 읽는 법

| 표시 | 의미 | 다음 설계에서의 취급 |
| --- | --- | --- |
| **요구 R** | 사용자가 얻어야 할 결과, 실패 시 구별해야 할 사실 | 같은 기능과 완료 조건을 비교한다. 실패 안내는 정상 성공을 대신하지 못한다. |
| **경계 B** | 책임 범위, 사용자가 명시한 제약과 합의한 행동 | 이번 탐색의 공통 조건이다. 바꾸려면 해당 합의 변경을 별도로 드러낸다. |
| **선택 T** | 검토 완료 target이 그 요구를 실현하는 구조, 계약, 정책 | 방안 1에서는 그대로 설명한다. 대안이 같은 수단까지 쓸 필요는 없다. |
| **지원 U** | 설계가 요구하지만 실제 dependency/build에서 확보하거나 검증하지 않은 기능 | 양안에 공짜로 제공하지 않는다. 누가 제공할지와 미지원 시 기능 한계를 밝힌다. |
| **평가 E** | 비교용 장비, 부하, 장애, 자료 조건 | 제품의 최대 기능이나 빈도로 확대하지 않는다. 비교 시 양안에 같은 조건을 둔다. |

R/B/T는 사실의 종류이고 U는 지원 확인 상태다. **선택된 계약(T)이면서 실현 capability는 미검증(U)일 수 있다.** 예컨대 target의 runtime 취소 계약은 결정되어 있지만 실제 GPU 작업이 그 계약대로 중단되는지는 미확인이다. 이를 ‘target 설계 미완료’나 ‘이미 동작’으로 바꾸지 않는다. 표에서 요구하는 정확한 결과는 관찰과 판정 기준이며 의미 오류가 절대 없다는 수학적 보장이 아니다.

## 2. 모든 문제에 공통으로 적용할 경계

<a id="21-명시-요구합의와-설계-자유도를-나누는-기준"></a>
### 2.1 명시된 요구와 합의를 설계 자유도와 구분하는 기준

| ID | 이번 탐색에서 유지할 결과와 경계 | 같은 것으로 고정하지 않을 수단 | 근거 |
| --- | --- | --- | --- |
| B-01 | VIA는 PC의 지속 Voice/Text/screen interaction과 orchestration을 담당한다. Agent는 도메인 추론, 업무 계획, Tool 선택, 외부 상태 변경을 담당한다. VIA 자신의 대화, 업무, 기억, 설정 변경은 VIA 책임이다. | Component 수, 배치, 호출 구조, 상태 표현. VIA 안에 일반 업무 실행기를 도입해 책임 경계를 바꾸는 것은 별도 범위 변경이다. | [Mission §1.1~1.4](../../01-system-mission-and-boundary.md), [Scope §3.4~3.8](../../03-fixed-architecture-scope.md) |
| B-02 | on-device Omni 한 개의 weights를 음성과 semantic 역할이 공유한다. 역할의 Context, session, 권한을 섞지 않는다. Semantic 수행 중에도 새 발화 수신과 인식이 계속되어야 한다. | 독립 ASR의 존재, Model Access의 내부 실행 구성, process 배치와 자원 관리 구조. 별도 helper의 필요성과 전체 비용은 공개해야 한다. weights 복제나 몰래 cloud 우회는 이 조건 안의 대안이 아니다. | [Mission §1.3](../../01-system-mission-and-boundary.md), [Omni §1~4](../target-architecture/shared-omni-runtime.md) |
| B-03 | S2S 직접 응답을 지원하되 명백한 자체 지식 독립 질문으로 좁힌다. 화면, 개인 자료, Task, 과거 대화 지칭, 최신 정보가 필요하면 해당 직접 경로로 답하지 않는다. | VoiceProposal 형식, host gate 구현, speculative 생성, buffer, 인계 계약. S2S 직접 기능을 통째로 없애면 전체 요구를 만족한 안이 아니다. | [사용자 지정 표](../target-architecture/README.md#사용자-지정과-검토-기준선의-구분), [제어 §2](../target-architecture/control-and-lifecycle.md#2-최소-s2s-직접-응답-계약) |
| B-04 | 합의한 기본 handling에 따라 한정 자료의 설명과 요약은 VIA가 맡고, 조사와 업무 계획 및 실행은 Agent가 맡는다. 동시에 고정 scope는 bounded 정보 요청의 Agent 위임도 허용한다. 사용자가 명시적으로 위임했거나 이미 맡긴 업무의 일부인 경우에는 Agent가 처리한다는 target의 예외도 유지한다. | 어떤 처리 구조로 직접 응답과 위임을 연결하는가. ‘모든 bounded 요청은 반드시 직접’ 또는 ‘VIA는 직접 응답을 전혀 하지 않음’으로 단순화하지 않는다. 기본 handling 방향 변경은 사용자 합의와의 차이를 명시한다. | [Scope §3.6](../../03-fixed-architecture-scope.md), [UC §5.2, 5.10](../../05-representative-use-cases.md), [제어 §5](../target-architecture/control-and-lifecycle.md#5-전송과-정정의-원자적-경계) |
| B-05 | 제한된 조회 뒤에도 대상 후보가 여럿이면 구분을 질문한다. 한 업무는 통째로 Agent에 맡기며 독립된 지속 업무는 같은 Agent여도 별도 Task로 구별한다. | 조회 서브시스템, 요청 관계, 질문, Task의 상태 저장과 실행 구조. 문장 속 동사마다 Task를 만드는 것은 요구가 아니다. | [사용자 합의 표](../target-architecture/README.md#사용자-지정과-검토-기준선의-구분), [구조 §5, 12](../target-architecture/architecture.md) |
| B-06 | 새 발화 때 같은 대화의 미전송 요청을 보류한다. 음성 중단과 업무 취소는 다르다. 이미 전송한 실행이나 완료한 외부 변경을 막았다고 주장하지 않는다. | input hold, epoch, transaction, 전송 선형화의 구현 수단. 물리적 발화 시작부터 host 인지까지의 지연을 없다고 가정하지 않는다. | [사용자 합의 표](../target-architecture/README.md#사용자-지정과-검토-기준선의-구분), [제어 §1, 5](../target-architecture/control-and-lifecycle.md) |
| B-07 | 모든 응답은 Text로 남고 Voice 활성 시 핵심을 전달한다. 사용자가 말하는 동안 음성 알림을 끼워 넣지 않는다. 중단 후 재개 의도가 모호할 때만 확인한다. | publication 저장과 조정 구조, 재생 허용 protocol, 대기열, receipt 표현. 문장 문자열이 같을 필요는 없지만 대상, 결론, 실패, 불확실성은 일치해야 한다. | [사용자 지정 표](../target-architecture/README.md#사용자-지정과-검토-기준선의-구분), [구조 §13](../target-architecture/architecture.md#13-응답-전달) |
| B-08 | 접근, 외부 제공, Action 승인의 현재 범위를 지킨다. 삭제한 기억은 새 사용과 자동 복원에서 제외한다. 확인 가능한 업무는 실제 복원하고 불명 상태를 성공으로 바꾸지 않는다. | 정책 강제 위치, 삭제 전파, 원본과 파생 자료의 저장, recovery mechanism. 과거 대화의 ‘응’이나 cache가 현재 권한의 원본이 될 수 없다. | [UC-16~18](../../05-representative-use-cases.md), [기억 §5~6](../target-architecture/memory-and-context-lifecycle.md), [FA-14~15](../../06-fixed-assumptions.md) |
| B-09 | 모델/runtime 연동 계약은 설계 범위이며 모델 내부 알고리즘과 학습, Agent 내부 실행은 범위 밖이다. 기존 QA 의미, ADR 상태, 검토 완료 target은 이번 문서로 변경하지 않는다. | VIA가 요구할 interface와 지원 조건은 탐색한다. dependency가 구현할 내부 학습 방식을 DP 대안으로 삼지 않는다. | [Scope §3](../../03-fixed-architecture-scope.md), [Core ASR 계약](../../08-quality-attributes/core-asr-contract.md) |

### 2.2 특히 잘못 고정하기 쉬운 항목

| 항목 | 분류와 이유 |
| --- | --- |
| Request Interpreter 제안 → Request Controller 확정, Context Manager 조회 | **T.** 현재 권위와 호출 구조다. 목표, 대상, 제약, 권한이 맞아야 한다는 R과 구별한다. 이름만 합치거나 나누는 것은 이후 구조 대안의 근거가 되지 않는다. |
| semantic 2회, 추가 조회 한 묶음, 구성 1회 | **T.** 현재 예산 정책이다. 유한 자원과 종료 동작은 필요하지만 횟수 변경만으로 주요 DP를 만들지는 않는다. |
| 독립 Speech Input Worker와 경량 Streaming ASR | **T + U.** 입력 지속성을 위한 선택이다. 인식과 시점 근거가 실제 지원되는지는 별도 확인 사항이다. B-02와 동등한 제품 행동을 다른 구조로 만들 수 있는지 탐색 가능하다. |
| embedded DB, 내구 outbox/inbox, current records, CAS, tombstone | **T.** 상태, 중복, 삭제 요구의 현재 수단이다. 다른 안에도 동일 DB나 동일 원장을 강제하지 않는다. |
| 원문/요약 구별, 재시작 후 source 확인 | 잘못된 내용이나 미확인을 사실로 취급하지 않는 **R**과, 현재 source/receipt/revision 체계라는 **T**를 구별한다. 요약이나 model KV를 무조건 모든 사실의 원본으로 바꾸면 요구를 어떻게 유지할지 설명해야 한다. |
| evidence cache 24시간, 진단 7일, LRU, scheduler 순서, credit | **T.** 선택된 보관 및 자원 정책이다. 미정으로 되돌리지 않으며 숫자나 순서만 바꾼 안을 구조적 DP로 포장하지 않는다. |
| FA-02의 장비, FA-08의 Task 수와 지칭 및 복합 요청 수, FA-11의 지연값 | **E.** 비교용 source pool과 조건이다. 새 QA workload가 승인되거나 동결됐다는 뜻도 아니다. 모든 VIA 사용자의 장비, 최대 Task 수, 실제 빈도, 제품 지연 보장으로 읽지 않는다. |
| FA-14 재시작과 외부 상태 확인 조건 | **E 안에서 지켜야 할 R.** 저장소가 정상이고 Agent 상태가 확인되는 경우의 복구는 필수다. 모든 경우를 UNKNOWN으로 끝내는 우회는 허용되지 않는다. |
| M-04~06 local/remote 전환 | **변화 시나리오 E.** 변경 대응을 살펴볼 대상이지, 현재 B-02를 일반 후보 비교에서 몰래 풀어 주는 허가가 아니다. |

## 3. 16개 문제의 요구와 선택 — 한눈에 보기

| 문제 | 바꾸면 안 되는 사용자 결과 | target이 선택한 주요 수단 | 다음에 탐색할 구조적 지점 |
| --- | --- | --- | --- |
| [P-01](#p-01) 요청 이해 | 빠진 조건을 보완하고 정정과 충돌을 반영한 목표 확정 | 통합 semantic proposal + host commit + 질문 상태 | 해석, 추가 근거, 확정이 진행되는 실행 구조 |
| [P-02](#p-02) 화면 지칭 | 말한 당시의 대상, 집합, 역할을 연결 | 시점별 RAM timeline + capture + timed transcript | 관측 근거의 생산, 시간 결합, 보존 구조 |
| [P-03](#p-03) Context 확보 | 의도한 자료를 출처, 시점, 조회 한계와 함께 확보 | owner read port + metadata/keyword index + cache | 요청 전 준비와 요청 시 획득을 담당할 자료 서브시스템 |
| [P-04](#p-04) 직접/위임 | 직접 대화와 외부 업무를 같은 대화로 연결 | 좁은 S2S admission + Core handling + Task command | 응답 경로와 업무 인계의 실행 및 권위 구조 |
| [P-05](#p-05) 복합 요청 | 독립, 순서, 조건, 데이터 관계 유지 | durable Request Graph + version별 후속 해제 | 대기, 조건, 결과 의존성을 실행하고 재개하는 구조 |
| [P-06](#p-06) 업무, 질문 연결 | 정확한 Request, Task, 실행, 질문에 반영 | 별도 identity + Pending User Interaction + owner 상태 | 여러 수명의 상태와 질문 재개를 소유하는 구조 |
| [P-07](#p-07) Agent 연동 | 지원 기능, 실제 상태, 결과를 정확히 연결 | capability profile + adapter + inbox + Task projection | 실행 계약 변환, 상태 관측, 동기화 구조 |
| [P-08](#p-08) 중단, 정정과 취소 | 재생 중단과 업무 제어를 구별하고 실제 효력 표시 | local stop + hold/epoch + conditional dispatch | 입력 제어 경로와 외부 전송 확정 경계 |
| [P-09](#p-09) 실제 전달 | Text 상세, Voice 핵심, 실제 전달 범위를 구별 | publication outbox + output lease + delivery receipt | 채널별 출력 생산, 게시, 전달 사실 보존 구조 |
| [P-10](#p-10) 연속성 | 채널 종료가 대화, 업무 종료가 되지 않음 | 내구 Conversation/Task + 별도 Voice session/KV | 대화 원본, 모델 작업 이력, 접속 수명 연결 |
| [P-11](#p-11) 정보, 승인 | 현재 허용한 읽기, 제공, Action만 반영 | Policy Manager + Use Envelope + 실제 사용 port 검사 | 정책 권위와 강제, 철회 전파 경계 |
| [P-12](#p-12) 사용자 기억 | 허용 기억의 조회, 변경, 삭제와 현재 지시 우선 | typed memory + tombstone + 파생 view 무효화 | 기억 원본, 검색/요약, 모델 상태의 수명 구조 |
| [P-13](#p-13) 복구 | 확인 가능한 관계 복원과 중복 실행 방지 | current records + pending ledgers + Agent 조회 | 복구 권위 원본, 재구성, 외부 상태 조정 구조 |
| [P-14](#p-14) PC 자원 | 입력, 제어와 정상 요청 모두 진행 | 독립 ASR + 공유 추론 service + 예약, 유한 queue | 모델 실행, 입력 보호, 자원 소유와 장애 경계 |
| [P-15](#p-15) 변화 | 교체, migration 뒤 기능, identity, 권한 유지 | canonical port/adapter + versioned state migration | 외부 계약을 흡수하는 연동, 상태 변환 구조 |
| [P-16](#p-16) 근거 | 과다 수집 없이 원인 추적, 평가 재확인 | owner 기록의 ID/revision 연결 + 제한된 진단 | 업무 원본과 진단, 평가 근거의 생성, 보존 경계 |

## 4. 문제별 상세 분류

각 항목의 ‘다르게 설계할 지점’은 **동등한 기능을 유지한 채 바꿀 수 있는 영역**이다. Component 이름이나 정책만 달리하는 것으로 끝나는지는 4단계에서 다시 걸러낸다. U 번호는 §5의 지원 확인 목록과 연결된다.

<a id="p-01"></a>
### P-01. 불완전한 요청을 어떻게 확정할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 목표, 대상, 완료 조건, 제약을 보존한다. “아까 그 자료”의 부족한 정보를 확인 답변으로 보완하되 이미 확정한 조건을 잃지 않는다. 정정, 새 목표, 철회를 구별한다. |
| R/B — 예외와 경계 | 부정, 숫자, 수신자 등의 입력/해석 충돌을 임의 확정하지 않는다. 조회 후에도 구별 안 되는 후보는 질문한다(B-05). 충분한 근거가 있는데 항상 재입력을 시키는 것도 정상 해결이 아니다. 업무 내부 계획은 Agent 책임이다. |
| T — 현재 처리 | Request Controller가 입력 revision, Context, 예산을 결합 → Request Interpreter가 공유 Omni로 field별 후보, 근거, 추가 읽기 제안 → Request Controller가 Context Manager 조회와 재해석을 제어하고 Semantic Commit 또는 대기 질문을 확정한다. 정규 음성 입력과 Omni 해석 불일치는 별도 제안으로 남긴다. |
| T — 구체 선택 | input revision당 semantic 총 2회, 추가 읽기 한 묶음, Pending User Interaction, immutable commit와 관련 revision 검증. 동일 모델 두 역할의 일치가 독립적인 진실 검증은 아니다. |
| 다르게 설계할 지점 | 해석에 필요한 근거를 누가 언제 확보하고, 보완 중간 상태와 확정 권한을 어떤 실행 구조가 유지할 것인가. 모델 호출을 함수 하나로 옮기는 차이를 넘어서는지 확인한다. |
| U / 근거 | U-01, 03, 04. [UC-06, 01.3, 09.5](../../05-representative-use-cases.md), [구조 §6~7](../target-architecture/architecture.md#6-요청-이해와-처리-확정), [제어 §3~4](../target-architecture/control-and-lifecycle.md#3-core-해석읽기응답-예산) |

<a id="p-02"></a>
### P-02. 말한 당시의 화면 대상을 어떻게 남길 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 발화 전 선택, 말하면서 가리킨 여러 대상, 집합, 범위, 보존/변경 역할을 당시 문서, 화면과 연결한다. 나중에 전사가 도착해도 마지막 화면 하나로 바꿔 해석하지 않는다. |
| R/B — 예외와 경계 | 포인터 이동이 의도적 선택인지 불명확하거나 시간 오차 구간에 여러 후보가 있으면 모호함을 유지한다. 유실, 구조 정보 없음, 좌표 변환 실패를 정답 ID로 채우지 않는다. 화면 관측과 현재 Action 적용 가능성은 다르다. |
| T — 현재 처리 | Interaction Manager가 OS/UI 사건, 유한 sampling, pre-roll, 발화 구간 pin으로 RAM timeline을 만든다. Model Access가 연결한 독립 Streaming ASR의 시각/revision을 결합한다. Context Manager가 원문, crop, UI 자료를 준비하고 Request Interpreter가 의미 후보를 제안한다. |
| T — 구체 선택 | producer sequence, clock uncertainty, watermark/gap, typed referent, 늦은 근거에 영향받은 field만 무효화. capture는 전사 token마다 시작하지 않는다. 재시작에 필요한 최소 crop만 허용 범위에서 저장한다. |
| 다르게 설계할 지점 | 음성, 화면 근거의 생산자가 무엇을 언제 함께 기록하고, 어떤 중간 자료를 보존하여 과거 지칭을 재구성할 것인가. sampling 숫자나 시간 매칭 알고리즘만 바꾸는 문제로 좁히지 않는다. |
| U / 근거 | U-01, 02, 03, 07. [UC-03, 04와 §5.5](../../05-representative-use-cases.md), [구조 §9](../target-architecture/architecture.md#9-contextcachestale-처리), [기억 §2, 4](../target-architecture/memory-and-context-lifecycle.md) |

<a id="p-03"></a>
### P-03. 현재 보이지 않는 자료까지 어떻게 확보할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 현재 화면, 숨은/닫힌 자료, 과거 대화, 업무 결과를 의도한 범위로 읽고 출처, 확인 시점을 남긴다. 이름 검색 성공과 본문 확보를 구별한다. |
| R/B — 예외와 경계 | 없음, 거부, 미지원, 후보 다수, 부분 조회를 구별한다. 검색 몇 건을 전체 확인이나 유일한 대상으로 간주하지 않는다. 전체 웹/모든 파일의 완전 탐색을 약속하지 않는다. 열린 조사 계획은 Agent 책임이다. |
| T — 현재 처리 | Context Manager가 owner read port와 source adapter, metadata/keyword index, revision cache로 기본 근거를 준비한다. Request Controller가 허용한 한정 추가 조회를 수행하고 Evidence Query Receipt와 원문/요약 view를 반환한다. |
| T — 구체 선택 | 기본 embedding 모델, vector DB 없음. read set, source revision, truncation/실패/누락 기록, source 변경 시 cache 무효화, 관련 의존성만 재검증. 현재 target에도 검색, 요약, cache가 있다. |
| 다르게 설계할 지점 | 요청 시 읽기와 미리 가공한 자료의 비중, 후보 생산 서브시스템, 지속 중간 자료, 갱신 책임을 어떻게 구성할 것인가. 의미 검색 추가를 미리 채택하지 않으며 모든 추가 자료의 권한, 삭제 비용을 따진다. |
| U / 근거 | U-02, 03, 04. [UC-02, 05, Context 6종](../../05-representative-use-cases.md), [기억 §1~3](../target-architecture/memory-and-context-lifecycle.md#1-선택한-기억-구조), [구조 §9](../target-architecture/architecture.md#9-contextcachestale-처리) |

<a id="p-04"></a>
### P-04. 직접 답변을 실제 업무로 어떻게 이어갈 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 자체 지식 질문에 직접 답하고, 자료 기반 설명에서 “이걸로 발표자료 만들어줘”라는 업무로 이어진다. Task 없이 한 응답도 후속 업무의 근거다. 사용자가 Agent 대화창으로 옮겨 갈 필요가 없다. |
| R/B — 예외와 경계 | B-01, 03, 04를 함께 지킨다. Task 관계와 처리 위치는 별개다. direct와 위임 경로가 같은 요청에 모순된 답/중복 업무를 만들지 않는다. S2S 미지원의 Core-only 제한 모드는 전체 S2S 요구의 성공이 아니다. |
| T — 현재 처리 | S2S VoiceProposal을 Request Controller가 좁게 admission하고 Response Manager가 보류된 generation을 release한다. 나머지는 같은 Request를 Core로 인계한다. Request Interpreter의 handling 제안을 검증한 뒤 직접 구성 또는 Task Manager → Agent Gateway로 이어진다. |
| T — 구체 선택 | current-Turn-only S2S, 단일 route owner, speculative buffer, direct admission/semantic commit의 서로 다른 근거, 게시된 Response 참조. S2S에 임의 Context 탐색, 업무 Tool 권한을 주지 않는다. |
| 다르게 설계할 지점 | 직접 응답과 업무 처리 경로가 어떤 실행체, 중간 상태, 인계 계약으로 연결되는가. B-03의 좁은 S2S 행동을 유지하면서 그 허용, 인계, 게시 구조를 바꿀 수 있는지 살핀다. |
| U / 근거 | U-03, 05, 06. [UC-01, 02, 07, 08](../../05-representative-use-cases.md), [구조 §8, 12, 13](../target-architecture/architecture.md#8-s2s와-직접-응답), [제어 §2, 5](../target-architecture/control-and-lifecycle.md) |

<a id="p-05"></a>
<a id="p-05-여러-부탁의-대기선후-관계를-어떻게-실행할-것인가"></a>
### P-05. 여러 부탁의 대기와 선후 관계를 어떻게 실행할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 독립, 순차, 데이터 의존, 조건, 혼합 관계와 각각의 완료 범위를 유지한다. 실제 선행 결과를 필요한 후속 요청에 연결하고, 다른 독립 요청은 진행한다. |
| R/B — 예외와 경계 | 선행 실패, 조건 불명, 결과 version 변경을 완료/참으로 꾸미지 않는다. 한 업무의 내부 도구 단계는 Agent가 계획한다(B-05). 조건 판단에 업무 분석이 필요하면 Agent에 맡긴다. 여러 Agent Action을 VIA가 원자 rollback한다고 약속하지 않는다. |
| T — 현재 처리 | Request Controller의 durable Request Graph가 독립 목표 node와 관계 edge를 소유한다. WAIT_DEPENDENCY에서 선행 결과, 조건, 현재 권한을 확인하고 후속 admission을 만든다. Task Manager는 연결된 업무의 lifecycle을 관리한다. |
| T — 구체 선택 | graph revision + node ID + result version으로 한 번만 해제, 정정 후 옛 결과 무효화, 참/거짓/불명 구별. 한 업무의 “요약 후 발송”을 VIA 내부 두 Tool 단계로 분해하지 않는다. |
| 다르게 설계할 지점 | 사용자 명시 관계의 실행 상태, 대기, 재개, 타이머를 어떤 실행 구조와 저장 원본이 유지할 것인가. 전체 업무를 불투명하게 합쳐 독립 제어를 잃는 것은 대체 수단이 아니다. |
| U / 근거 | U-05, 07. [UC-09 전체, 14](../../05-representative-use-cases.md), [구조 §5, 12](../target-architecture/architecture.md#12-장기-업무복합-요청agent-event), [제어 §4~5](../target-architecture/control-and-lifecycle.md) |

<a id="p-06"></a>
<a id="p-06-짧은-답변을-어느-업무질문에-연결할-것인가"></a>
### P-06. 짧은 답변을 어느 업무나 질문에 연결할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | Task 없는 clarification도 원래 Request에 연결한다. 여러 Task, Execution, 질문 중 답변, 정정, 취소, 결과가 가리키는 대상을 유지한다. 같은 결과물 수정은 기존 업무, 이전 결과를 재료로 한 별도 목표는 새 업무로 구별한다. |
| R/B — 예외와 경계 | 가장 최근 생성된 질문에 “응”을 자동 적용하지 않는다. 실제 제시된 질문과 유일한 유효 대상을 확인하고 모호하면 묻는다. 끝난 실행의 늦은 승인, 답변을 다른 실행에 재사용하지 않는다. 한 질문 대기가 전체 대화를 멈추지 않는다. |
| T — 현재 처리 | Request Controller가 Pending User Interaction과 대화, 요청 관계, Task Manager가 Task, Execution 관계를 소유한다. Context Manager의 후보 조회와 Request Interpreter의 의미 제안을 결합하여 Request Controller가 답변 대상을 확정한다. |
| T — 구체 선택 | interaction ID, 선택적 Task/Execution/question, 실제 질문 focus, 허용 답변, revision, expiry. 답변 확정과 질문 종료는 같은 owner transaction. phase, control, verification 상태를 분리한다. |
| 다르게 설계할 지점 | 여러 수명의 identity와 대기 상태를 어떤 상태 모델, 지속 실행 구조가 유지하고, 사용자 응답으로 정확히 어느 처리를 재개할 것인가. 관리자를 하나 더 만드는 차이만으로 끝내지 않는다. |
| U / 근거 | U-03, 05, 06, 07. [UC-06.4, 10, 14, 16.5](../../05-representative-use-cases.md), [구조 §5](../target-architecture/architecture.md#5-conversationturnrequesttaskexecution), [제어 §4](../target-architecture/control-and-lifecycle.md#4-request질문task의-상태-전이) |

<a id="p-07"></a>
<a id="p-07-서로-다른-agent의-실제-기능상태를-어떻게-연결할-것인가"></a>
### P-07. 서로 다른 Agent의 실제 기능과 상태를 어떻게 연결할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 기능, 권한, 사용자 지정과 요청 제약에 맞는 Agent를 선택해 목표, 완료 조건과 허용 자료를 전달한다. 접수, 진행, 질문, 완료, 부분 실패, 결과물과 후속 실행을 올바른 VIA 업무에 연결한다. |
| R/B — 예외와 경계 | 취소 요청 접수를 취소 완료로, 연결을 업무 성공으로 표시하지 않는다. capability 없는 Agent의 기능을 VIA가 지어내지 않는다. 적합 후보가 없으면 미지원으로 알리고, 비용, 권한 또는 완료 조건이 다른 대체는 사용자 확인 없이 적용하지 않는다. 같은 Agent의 복수 업무도 독립 identity를 유지한다. 도메인 실행 로직은 외부다. |
| T — 현재 처리 | Agent Gateway가 소유한 capability profile을 바탕으로 Request Interpreter가 요구 기능과 후보를 제안하고 Request Controller가 현재 조건을 검증해 선택한다. Agent Gateway의 adapter와 durable outbox/inbox가 protocol을 변환한다. Task Manager가 event의 중복, 순서, correlation을 확인해 source-confirmed projection을 갱신한다. 필요한 경우 source snapshot을 조회한다. |
| T — 구체 선택 | 의미 후보 제안과 결정적 조건 검사를 결합하고, 기능적으로 동등한 후보에는 설정된 선호와 안정적인 우선순위를 적용한다. Task Manager는 선택용 LLM을 호출하지 않는다. 전송 전 profile 재검사, command/Execution/question/artifact ID, source revision, cursor, UNKNOWN submit의 key 조회. event 순서도 조회도 없으면 UNKNOWN이며 자동 재위임하지 않는다. |
| 다르게 설계할 지점 | 선택 전에 어떤 capability 근거를 확보하고, 후보 제안과 검증을 어떤 실행 의존성으로 연결할 것인가. 선택 순위 설정과 구조 변경을 구별한다. capability와 실행 계약을 흡수하는 연동 계층, push/query 관측과 상태 권위 및 보관 구조도 검토한다. 비교에서는 같은 Agent 지원 수준을 사용한다. |
| U / 근거 | U-05. [UC-08, 10, 12, 13, 18](../../05-representative-use-cases.md), [FA-13](../../06-fixed-assumptions.md), [구조 §12 Agent 선택](../target-architecture/architecture.md#12-장기-업무복합-요청agent-event), [제어 §5 capability 표](../target-architecture/control-and-lifecycle.md#5-전송과-정정의-원자적-경계) |

<a id="p-08"></a>
<a id="p-08-중단정정취소를-어디까지-반영했는가"></a>
### P-08. 중단, 정정과 취소를 어디까지 반영했는가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 새 발화를 받고 옛 음성은 멈춘다. 미전송 요청의 정정, 철회와 실행 중 업무의 수정, 취소를 구별한다. 무관한 Task는 계속한다. |
| R/B — 예외와 경계 | B-06의 같은 대화 미전송 보류를 유지한다. 실제 전송 뒤에는 접수, 실행 여부를 확인한다. 완료/취소 경쟁, 미지원, 미확인을 그대로 알리고, 이미 수행된 외부 변경을 소급 취소했다고 하지 않는다. |
| T — 현재 처리 | Interaction Manager가 추론 밖에서 local stop, 출력 세대 무효화 → Request Controller가 입력 hold, 의미 정정 → Task Manager가 command/epoch 갱신 → Agent Gateway가 State Store 조건부 transaction으로 전송 시작을 확정한다. |
| T — 구체 선택 | PENDING→DISPATCHING CAS에 admission, hold, epoch, policy를 함께 검사한다. network는 transaction 밖이고 crash 시 DISPATCHING은 접수 불명이다. backend 계산 취소와 host의 결과 사용 차단은 별개다. |
| 다르게 설계할 지점 | 입력 제어가 모델/저장 대기에 묶이지 않는 실행 경로, 전송 의도, 허가, 외부 실행의 경계를 어떻게 구성할 것인가. 취소 문구나 queue 우선순위 값만 바꾼 비교는 부족하다. |
| U / 근거 | U-01, 03, 05, 06, 07. [UC-11, 12, 04.6, 06.5](../../05-representative-use-cases.md), [구조 §11](../target-architecture/architecture.md#11-동시성정정취소전송), [제어 §1, 5](../target-architecture/control-and-lifecycle.md) |

<a id="p-09"></a>
### P-09. 생성한 답과 실제 전달한 답을 어떻게 구별할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 상세 Text와 핵심 Voice의 대상, 결론, 실패, 불확실성이 일치한다. 결과, 질문이 VIA에 남고, 끊겨도 Text를 확인할 수 있다. 후속 “방금 말한 것”은 실제 전달 근거에 연결한다. |
| R/B — 예외와 경계 | 생성, UI 표시, audible 범위, 전달 불명을 구별한다. 사용자 발화 중 음성 알림 금지(B-07). Text만 표시했는데 음성도 다 들려줬다고 하지 않는다. 전체 Text를 지우거나 모든 음성 기능을 포기해 전달 문제를 피하지 않는다. |
| T — 현재 처리 | Response Manager가 허용 payload에서 Text/Voice view를 구성하고 publication outbox에 기록한다. Interaction Manager에 release/cancel을 보내고 채널별 delivery receipt를 받는다. 발화 차례와 마지막 확인 audible 범위를 별도로 관리한다. |
| T — 구체 선택 | publication ID의 Text upsert, Voice generation, segment, output epoch, 문장 단위 대응 검사, SuspendedDelivery. crash 전 실제 재생과 durable receipt는 원자적이지 않아 확인 범위 밖은 DELIVERY_UNKNOWN이다. |
| 다르게 설계할 지점 | 상세/음성의 생산, 조정, 실제 전달 원장을 어떤 출력 서브시스템이 맡고, 대화가 어떤 기록을 참조할 것인가. 문장을 짧게 요약하는 알고리즘 차이만으로 끝내지 않는다. |
| U / 근거 | U-03, 06, 07. [UC-07, 11, 13, 15](../../05-representative-use-cases.md), [구조 §13](../target-architecture/architecture.md#13-응답-전달), [제어 §6~7](../target-architecture/control-and-lifecycle.md) |

<a id="p-10"></a>
<a id="p-10-연결이-끊겨도-어떤-대화업무를-이어갈-것인가"></a>
### P-10. 연결이 끊겨도 어떤 대화와 업무를 이어갈 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | Voice↔Text, 연결 종료, 재연결 뒤에도 해당 Conversation의 참조와 Task가 이어진다. 새 Conversation이 기존 Task 취소나 자동 이동이 되지 않는다. |
| R/B — 예외와 경계 | 이전 입력, 승인, 업무를 재실행하지 않는다. 생성만 된 내용과 삭제, 만료한 내용을 기억처럼 복원하지 않는다. 새 대화에 기존 업무를 참조할 수 있지만 모든 대화를 무조건 합치지 않는다. |
| T — 현재 처리 | Request Controller의 내구 Conversation/Turn/Request, Task Manager의 Task/Execution, Response Manager의 전달 기록을 원본으로 유지한다. Model Access의 session/KV는 재구성 가능한 작업 상태다. Voice focus와 connection epoch는 별도 수명이다. |
| T — 구체 선택 | 입력 시작 Conversation 고정, 재연결 때 옛 음성 자동 재생 금지, 새 대화의 기존 업무 선택 시 참조 추가. Context Manager가 owner port에서 필요한 원문, view를 구성한다. |
| 다르게 설계할 지점 | 장기 대화 원본과 모델 입력 이력, 현재 작업집합을 어떻게 연결하고 재구성할 것인가. 대화, Task, model session을 이름만 나누고 실제 수명은 결합한 안은 요구를 만족하지 못한다. |
| U / 근거 | U-03, 05, 06, 07. [UC-07.5, 15](../../05-representative-use-cases.md), [구조 §5, 9](../target-architecture/architecture.md), [제어 §1, 6~7](../target-architecture/control-and-lifecycle.md), [기억 §1, 3](../target-architecture/memory-and-context-lifecycle.md) |

<a id="p-11"></a>
<a id="p-11-지금-허용한-정보행동만-어떻게-통과시킬-것인가"></a>
### P-11. 지금 허용한 정보와 행동만 어떻게 통과시킬 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 읽기 동의, 외부 제공 동의, Agent Action 승인을 구별해 해당 대상, 목적, 실행에만 적용한다. 현재 허용 범위가 줄면 관련 사용도 줄어든다. 외부 문서/Agent 텍스트는 권한 변경 지시가 아니다. |
| R/B — 예외와 경계 | 다른 질문의 “응”, 오래된 승인, 바뀐 Action 내용을 승인으로 재사용하지 않는다. 철회 뒤 신규 사용을 막으며 이미 외부에 나간 정보를 회수했다고 주장하지 않는다. 실제 외부 Action 권한 강제는 Agent 책임이다. |
| T — 현재 처리 | Policy Manager가 현재 정책을 소유한다. Request Controller가 확정한 Use Envelope를 Context Manager, Model Access, Agent Gateway, Response Manager가 실제 읽기/제공/게시 직전에 검사한다. 승인은 Pending User Interaction에 실행, 질문, Action digest와 결합한다. |
| T — 구체 선택 | source/recipient/purpose/scope/policy revision/expiry, epoch fence, 진행 생성의 사용 중단, session 무효화. 사용 port의 검사는 각각 다른 정책 엔진을 갖는다는 뜻이 아니다. |
| 다르게 설계할 지점 | 정책 원본과 접근을 강제하는 실행 경계, 데이터 제공 경로, 철회의 파생 상태 전파를 어떻게 구성할 것인가. 권한 체크를 생략하거나 오래된 cache를 신뢰해 빠르게 만드는 안은 동등하지 않다. |
| U / 근거 | U-04, 05, 07. [UC-16](../../05-representative-use-cases.md), [고정 환경 FA-15](../../06-fixed-assumptions.md), [구조 §9~10](../target-architecture/architecture.md), [기억 §5](../target-architecture/memory-and-context-lifecycle.md#5-기억-변경삭제와-권한-철회) |

<a id="p-12"></a>
### P-12. 기억을 지운 뒤에도 다시 살아나지 않게 할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 허용한 선호, routine, 안정된 사실을 쓰고 사용자가 조회, 등록, 수정 및 삭제할 수 있다. 이번 요청의 지시가 장기 선호보다 우선한다. 일회성 요청을 자동 장기 기억으로 승격하지 않는다. |
| R/B — 예외와 경계 | 삭제된 기억의 새 사용, 파생 자동 복원을 막는다. 기억 삭제와 과거 대화 삭제, 대화 삭제와 업무 취소는 다르다. routine 기억을 무인 예약 실행으로 확대하지 않는다. 실제 정리 미완료와 외부 backup/Agent 삭제 한계를 알린다. |
| T — 현재 처리 | Request Controller가 의미와 대상을 확정하고 Context Manager가 typed User Memory를 변경한다. State Store transaction으로 revision, tombstone, invalidation outbox를 기록하고 dependency index로 summary/cache/index/KV/generation을 무효화한다. |
| T — 구체 선택 | 단기 RAM, 중기 파생 view, 장기 기억, 내구 업무 원본의 구분, 논리 차단 후 purge, PURGE_PENDING 재시도. 현재 기록 방식, 계층, 24시간/7일 기본값은 수단이며 삭제 효과와 혼동하지 않는다. |
| 다르게 설계할 지점 | 기억 원본과 검색/요약/model 상태의 연결, 소유권과 삭제 경계를 어떻게 만들 것인가. 추가 저장 구조가 있으면 원본뿐 아니라 그 구조의 오래된 결과도 사용되지 않도록 설명해야 한다. |
| U / 근거 | U-03, 04, 07. [UC-17, 15, 18](../../05-representative-use-cases.md), [기억 §1, 4~6](../target-architecture/memory-and-context-lifecycle.md) |

<a id="p-13"></a>
### P-13. 다시 켰을 때 무엇을 근거로 같은 업무를 복원할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 로컬 저장소가 정상이고 외부 Agent 실행/완료가 확인되는 FA-14 조건에서는 같은 Task, Execution, 결과, 대기 질문, 허용 제어를 실제 복원한다. 외부에 완료된 업무를 다시 실행하지 않는다. |
| R/B — 예외와 경계 | 로컬, 외부의 복구 근거가 소실되거나 불충분한 범위는 미확인/불명으로 남기고 다음 행동을 안내한다. 저장소 정상, 외부 조회 가능한 FA-14 조건까지 “모르니 다시 요청”으로 끝내지는 않는다. 로컬 저장 손상, migration 실패나 전달 receipt 불명도 확인 한계를 드러내되 이를 기본 복구 성공과 혼동하지 않는다. 프로세스 재시작 자체는 복구 완료가 아니다. 저장 장애 시 새 외부 실행 의도를 기록한 척하지 않는다. |
| T — 현재 처리 | State Store의 권위 current records와 command/event/domain/publication 미완료 원장을 복원한다. 새 incarnation으로 오래된 전송을 막고 owner 상태, inbox를 반영한 뒤 Agent source와 조정, 질문, 결과, 전달을 다시 연결한다. |
| T — 구체 선택 | embedded DB + managed blob manifest, outbox/inbox, cursor, owner loader, Task별 recovery deadline. Audit log나 모델 KV가 업무 상태 원본은 아니다. DB와 실제 외부 실행, 실제 audio 전달 전체를 한 transaction이라고 하지 않는다. |
| 다르게 설계할 지점 | 무엇을 권위 원본으로 저장하고 어떤 실행/재구성 장치로 현재 상태를 회복할 것인가. 외부 source 지원은 동일하게 두며 어떤 로컬 저장 구조도 혼자 외부 exactly-once를 보장한다고 하지 않는다. |
| U / 근거 | U-05, 06, 07. [UC-18](../../05-representative-use-cases.md), [FA-14](../../06-fixed-assumptions.md), [구조 §14](../target-architecture/architecture.md#14-프로세스-배치fault-boundary), [제어 §7](../target-architecture/control-and-lifecycle.md#7-장애재시작버전-변경), [기억 §6](../target-architecture/memory-and-context-lifecycle.md#6-파일과-db의-crash-일관성) |

<a id="p-14"></a>
### P-14. 같은 PC 자원을 쓰면서 입력과 정상 처리를 함께 살릴 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 한 명의 사용자가 semantic 처리 중 말해도 capture와 recognition, 중단 제어가 계속된다. Omni 음성 처리와 foreground semantic도 진행해야 한다. 입력만 받고 정상 요청을 영원히 미루는 구조는 성공이 아니다. |
| R/B — 예외와 경계 | B-02의 weight 공유와 역할 격리를 유지한다. 유한한 지원 부하와 메모리 안에서 동작해야 하며 무한 queue나 녹음 backlog를 정상 동시 처리라 하지 않는다. 포화, gap, deadline 실패를 드러낸다. 평가 장비와 Task 수를 제품 보장으로 확대하지 않는다. |
| T — 현재 처리 | Model Access의 weight owner와 단일 scheduler를 Shared Inference Service에 둔다. 독립 Speech Input Worker가 CPU 예산으로 ASR을 처리한다. UI, Voice, Core와 위험한 connector를 분리하며 capture/local stop은 추론 scheduler 밖이다. |
| T — 구체 선택 | Voice/semantic 각각 최소 연산 및 KV 예약, chunk/safe-point 취소, 유한 quantum, queue, background 회수. 실제 비선점 kernel이 짧다는 가정은 아직 입증되지 않았다. process 분리가 열, 전력, 대역폭까지 격리하지 않는다. |
| 다르게 설계할 지점 | 입력 근거 생산과 공유 모델 추론을 어떤 실행체, 메모리 소유, 자원 admission, 장애 경계로 구성할 것인가. scheduler 순위나 thread 수만 바꾼 DP는 충분하지 않다. |
| U / 근거 | U-01, 03, 07, 08. [Mission §1.3](../../01-system-mission-and-boundary.md), [FA-02, 08, 10](../../06-fixed-assumptions.md), [Omni §3~8](../target-architecture/shared-omni-runtime.md) |

<a id="p-15"></a>
<a id="p-15-제공자계약상태-형식이-바뀌면-어디까지-바꿀-것인가"></a>
### P-15. 제공자, 계약 또는 상태 형식이 바뀌면 어디까지 바꿀 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | Model, Agent, Context source, 저장 schema, 관측 계약의 해당 변화 뒤에도 필요한 기능과 기존 대화/업무 identity, 권한, 삭제 효과를 유지한다. 바뀐 지원 범위와 migration 한계를 드러낸다. |
| R/B — 예외와 경계 | Agent 교체가 기존 불명 실행을 중복 start하는 이유가 되지 않는다. 모델 입력 이력, 시간 근거, 출력 계약 변화를 단순 endpoint 변경으로 숨기지 않는다. 모든 변화의 무중단 교체나 외부 실행 자동 이관은 요구하지 않는다. M의 배치 변화는 B-02를 몰래 해제하지 않는다. |
| T — 현재 처리 | Model Access, Agent Gateway, Context Manager의 provider/source adapter가 canonical 계약으로 변환한다. 각 상태 owner와 State Store가 schema version, reader compatibility, migration을 관리한다. |
| T — 구체 선택 | admission drain, outbox fence 후 migration, 진행 실행의 기존 binding 보존/명시 migration, 새 incarnation. 공통 contract가 변하면 다른 Component 영향도 공개한다. adapter가 모든 변경을 항상 한 곳에 가둔다는 주장은 없다. |
| 다르게 설계할 지점 | 외부 계약의 의미를 흡수할 연동 서브시스템, model 이력, durable 상태 변환 및 호환 실행 경계를 어떻게 구성할 것인가. 파일 수나 설정 줄 수로 구조 차이를 판단하지 않는다. |
| U / 근거 | U-01~08 중 해당 변화의 지원 계약. [변화 M/A/C](../../07-intentional-variables.md), [변화 E](../../08-quality-attributes/evidence/change-locality-rationale.md#5-qa-23-experimentlogging-change-pack), [구조 §4](../target-architecture/architecture.md#4-component와-상태-소유권), [제어 §7](../target-architecture/control-and-lifecycle.md#7-장애재시작버전-변경) |

<a id="p-16"></a>
### P-16. 과다 기록 없이 원인과 품질 주장을 어떻게 확인할 것인가

| 구분 | 내용 |
| --- | --- |
| R — 정상 결과 | 어느 입력, 근거, revision, 업무, 전달이 오류에 연결됐는지 추적한다. 후속 평가에서는 채택한 metric의 원시 판정과 실패를 남겨 결과를 독립 재계산할 수 있어야 한다. 운영 원인 분석과 평가 재현은 목적이 다르다. |
| R/B — 예외와 경계 | 원음, 화면, 전체 prompt 상시 저장을 당연시하지 않는다. 삭제 및 보관 권한을 지키며 없는 자료를 재현 가능하다고 하지 않는다. 시험 정답은 후보 입력과 분리하고 실패와 timeout을 제거하지 않는다. 지금 새 trace schema, 측정 계약을 freeze하지 않는다. |
| T — 현재 처리 | 각 owner의 입력/해석, command/event, publication 기록을 ID, revision, 원인 참조로 연결한다. 진단은 payload 없는 상태, 오류 중심의 유한 로그이고 확대 기록은 별도 동의와 수명을 따른다. |
| T — 구체 선택 | 진단 기본 7일, 최소 evidence 보관과 업무 원본 분리. target의 provenance 설계는 선택되어 있지만 target-derived 평가 harness, 원시 결과는 아직 없다. 기존 QA-61/62 계약을 현재 구현 완료로 읽지 않는다. |
| 다르게 설계할 지점 | 업무 경로에서 생성할 근거와 별도 관측 및 평가 경로에서 모을 근거, 그 저장, 접근, 삭제 경계를 어떻게 구성할 것인가. 로그 양을 늘리는 설정만으로 주요 DP를 만들지 않는다. |
| U / 근거 | U-01~08의 관측 가능성, 특히 U-06, 07. [QC/QA 추적](./01-02-problem-coverage-crosscutting.md), [변화 E-01~05](../../08-quality-attributes/evidence/change-locality-rationale.md#5-qa-23-experimentlogging-change-pack), [Core ASR §2, 9](../../08-quality-attributes/core-asr-contract.md), [기억 §4](../target-architecture/memory-and-context-lifecycle.md#4-보관-기본값) |

## 5. 지원이 필요하다는 것과 이미 확보했다는 것은 다르다

아래는 기능 의존성 확인 목록이다. **확보 완료 항목이나 이번에 실행할 시험 목록이 아니다.** 다음 대안을 만들 때 같은 외부 조건으로 비교하고, 추가 dependency가 필요하면 비용과 미지원 동작을 공개하기 위해 남긴다. 특정 조건의 실패 처리가 정의되어 있다는 사실만으로 정상 기능 충족을 주장할 수 없다. VoiceProposal 형식이나 파일/DB crash 계약처럼 target에 종속된 T+U는 대안의 동등한 계약으로 치환할 수 있다. 공통 사용자 결과에 필요한 capability를 유지하는 것이지 모든 안에 같은 schema나 DB를 강제하는 목록은 아니다.

| ID | 실제 확보 여부를 확인해야 할 지원 | 연결 문제 | 미지원이면 무엇이 제한되는가 |
| --- | --- | --- | --- |
| U-01 음성 근거 | 한국어 partial/final, revision, 원음 sample/시간 대응, 오차, gap, semantic 중 지속 인식. 현재 생산자는 ASR이며 정확성과 진행 기한은 미측정 | P-01, 02, 08, 14, 15, 16 | 당시 화면 지칭, 정정의 필수 근거가 없어지면 해당 확정을 보류한다. 재녹음 backlog만으로 동시 입력 성공이라 하지 않는다. |
| U-02 화면, source 식별 | OS/UI 접근, selection, 좌표계, window/document identity, 시각, revision, 이미지 자료, 구조 정보가 없을 때 시각 근거 | P-02, 03, 15, 16 | 지원되지 않은 source는 없는 ID를 만들어내지 않는다. 현재 화면으로 과거를 대체하지 않고 재지칭/자료 선택을 요청한다. |
| U-03 모델과 runtime | VoiceProposal, 구조화 의미 제안, 필요한 이미지 처리, Text/audio 대응, 동시 session, Context/KV 격리, chunk/cancel, 재시작 계약 | P-01~04, 06, 08~10, 12, 14~16 | 해당 경로는 제한/미지원이다. S2S 미지원의 Core-only, SpeechRender 미지원의 Text-only는 전체 기능 성공이 아니다. helper나 cloud를 숨겨 보충하지 않는다. |
| U-04 자료와 권한 | source별 읽기/열거, pagination, revision/freshness, 접근/보관/제공 권한, 철회 확인과 파생 사용 차단 | P-01, 03, 11, 12, 15, 16 | 결과 총수 불명은 별도로 기록한다. bounded scope의 exact identity 확인이나 source가 보증하는 열거/검색 완료 근거도 없으면 coverage는 unknown이다. 현재 권한 확인 불가 시 보호정보를 새로 사용하지 않는다. 외부 완전 소거를 가정하지 않는다. |
| U-05 Agent 계약 | 실행 correlation, 질문/승인, 결과, artifact version, 상태 조회, event 순서, 중복 방지, 취소, precondition의 실제 지원 | P-04~08, 10, 11, 13, 15, 16 | 필요한 기능 없는 Agent는 해당 업무를 받지 못하거나 정확한 제한을 알린다. 상태 확인 불가 시 자동 재전송과 재위임 금지. 지원되는 FA-14 복구 조건에서는 실제 재연결한다. |
| U-06 전달 관측 | UI 표시 receipt, 음성 segment/sample 대응, local stop, buffer 상태와 실제 audible 범위의 관측 가능성 | P-04, 06, 08~10, 13, 15, 16 | 생성 완료나 sink 도착을 실제 청취로 간주하지 않는다. crash와 receipt 사이 구간은 불명으로 유지한다. 실제 물리 endpoint 계측은 후속 검증이다. |
| U-07 저장과 플랫폼 | transaction, 조건부 갱신, durability, 파일/DB crash 규칙, 삭제/purge, process, IPC, sleep/lock, migration 동작 | P-02, 05, 06, 08~16 | 코드/장비 구현으로 확인할 부분이다. 쓰기 실패를 접수 성공으로 처리하거나 cache를 복구 원본처럼 쓰지 않는다. |
| U-08 자원 적합성 | 전체 weights, KV, activation, buffer, DB/OS 여유, CPU/accelerator/열, 전력, 비선점 구간, 동시 부하의 실제 한계 | P-14, 15, 16 | profile 선언만으로 서비스 기한과 메모리 적합성을 입증하지 못한다. 입력 또는 정상 foreground가 지속 정지하면 그 배치/profile이 부적합하다. |

근거: [모델/runtime 요구 §6, 8](../target-architecture/shared-omni-runtime.md), [capability와 미지원 동작](../target-architecture/control-and-lifecycle.md#5-전송과-정정의-원자적-경계), [후속 구현 및 검증 범위](../target-architecture/design-completeness.md#5-설계-완료-밖에-남는-일). 기존 reference 공개 기능이나 사용자 제공 모델 경험은 이 표의 통합 기능 확보 증거로 대체하지 않는다.

## 6. 문제별로 따로 설계하면 놓치는 연결

| 함께 확인할 문제 | 한쪽만 해결했을 때 놓치는 것 | 다음 단계로 넘길 확인 질문 |
| --- | --- | --- |
| P-01, 02, 03, 14 | 빠른 의미 해석도 당시 화면 근거가 사라졌으면 잘못된 대상을 고른다. | 입력 근거 생산, 조회, 추론이 같은 자원에서 실제로 함께 진행하는가? |
| P-04, 06, 09, 10 | 직접 답변이 기록되지 않거나 미전달 내용을 기억하면 이후 업무 인계가 틀린다. | 어떤 응답, 대상, 전달 상태를 다음 Request의 근거로 삼는가? |
| P-05, 06, 07, 13 | 대기 graph를 복원해도 외부 실행과 질문을 잃으면 잘못 재개하거나 중복 실행한다. | 로컬 재개와 source-confirmed 업무 재연결을 어떻게 맞추는가? |
| P-08, 11, 13 | 정정/철회 기록이 있어도 이미 나간 외부 Action을 되돌릴 수 있는 것은 아니다. | 로컬 차단, 외부 확인, 사용자 안내의 효력 경계를 어디에 두는가? |
| P-03, 10, 11, 12, 16 | 자료 원본만 지워도 검색 index, summary, KV, 진단 사본이 정보를 되살릴 수 있다. | 파생 자료와 진단 근거까지 현재 사용권한과 삭제를 어떻게 이어 적용하는가? |
| P-14, 15, 16 | 정상 시연만으로는 runtime 교체, 포화 때 기능 유지나 지연 원인을 확인할 수 없다. | 지원 조건과 관측 가능한 실패를 어떤 경계에서 보존하는가? |

이 연결은 DP 묶음의 결정이 아니다. 여러 문제가 같은 구조 선택으로 해결될 수 있고, 같은 문제가 여러 선택에 영향을 받을 수 있다. P-01~16에 하나씩 DP를 배정하지 않는다.

## 7. 이번 사용자 리뷰와 다음 단계

| 리뷰할 내용 | 의견을 남기는 예 |
| --- | --- |
| 고정해야 할 요구를 T/자유도로 잘못 풀었는가? | “P-xx의 ○○는 변경 가능한 수단이 아니라 필수 행동이다.” |
| 현재 target 수단을 R/B로 과도하게 고정했는가? | “B-xx의 ○○는 다른 구조로도 만족할 수 있으니 결과와 수단을 나눠라.” |
| 정상 및 실패 시 완료 조건이나 dependency 한계가 빠졌는가? | “P-xx는 △△ 상황에서도 복원/확인/중단이 필요하다.” |

다음 **3단계**에서는 이 결과와 경계를 바탕으로 ISO/IEC 25010:2023 품질 시나리오와 충돌을 작성한다. 그다음 **4단계에서 여러 구조를 탐색**하고, 5단계에서 실제 target과 가장 강한 대안을 비교한 후 6단계에서 DP를 선발한다. 이번에는 품질 기호, 우열, 대안 수, DP 개수를 먼저 정하지 않았다.

분류 대조와 보완 범위는 [2단계 검토 기록](./02-01-requirements-review.md)에 남긴다. 1단계의 94개 UC 변형, 운영/변화 추적은 [UC 표](./01-01-problem-coverage-use-cases.md)와 [횡단 표](./01-02-problem-coverage-crosscutting.md)를 그대로 사용하며, 이번에는 **16개 문제 모두에 요구, 예외, 선택, 자유도, 지원 조건을 연결했는지** 확인한다.
