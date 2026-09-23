# Candidate Architecture — 공통 구조·계약·비교 기준

> **Current measurement contract notice:** W-01~W-03 endpoint는 [Voice Responsiveness](../../08-quality-attributes/voice-responsiveness.md)로 재정의되었다. 이 문서의 기존 metric 연결은 historical이며 새 DP×W execution mapping이 동결될 때까지 측정 근거로 사용하지 않는다.
> 공통 metric event와 software boundary의 의미는 [Event & Boundary Contract](../../11-measurement/event-boundary-contract.md)를 따른다.
> 상태: Candidate definition current / measurement mapping pending. 후보 결과·승자 없음.
> 01~07 제품 조건, 10 요소 집계와 [Measurement & Scoring Contract](../../11-measurement/scoring-contract.md)를 대체하지 않는다.

## 1. 무엇을 고정하고 무엇을 바꾸는가

이 문서는 **구현 가능한 두 구조의 책임·계약·상태·배치**를 비교 전에 명세한다. Core 6개를 상세화하고 Supporting 3개는 공통 참조 구현으로 둔다. 이 참조 선택은 최종 Architecture Decision이 아니다.

| 비교 변수 | 참조값 | 변경하는 단위 |
|---|---|---|
| IR-DP01 | A | 의미 판단 단위와 중간 계약 |
| TASK-DP01 | A | Task 전이 writer와 command scheduling |
| AGENT-DP01 | A | Agent capability variation을 해석하는 위치 |
| EXEC-DP01 | A | 동일 integration code의 process 경계 |

기본 비교 단위는 해당 DP만 바꾸고 나머지 DP 조건을 고정한 A/B pair다. 각 pair는 같은 request, fixture, model profile과 dependency condition을 사용한다. metric에 참여하지 않는 DP의 결과를 복제해 독립 표본 수를 늘리지 않는다. 여러 DP의 interaction을 확인해야 할 때만 별도 contract와 분석 목적으로 조합 실험을 추가하며, 그 결과를 DP별 direct comparison과 구분한다.

Supporting 참조: CTX-DP01은 versioned/scoped Context Broker, CTX-DP02는 원문 기록에서 요청별 package 재구성, SEC-DP01은 같은 PC의 policy service를 통한 use-time 확인이다. **Interaction은 FP-INT01의 S2S Direct Fast Path로 고정하고, Agent state update는 TASK-T01의 Event-first + Query Reconciliation을 공통 tactic으로 적용한다.** 모두 같은 허용 범위·정보 원천을 갖는다. Supporting 선택에 민감한 결론은 해당 축을 교차 확인하기 전 일반화하지 않는다.

**MODEL-DP01/STATE-DP01/ORCH-DP01을 재개하지 않는다.** Model placement는 후보 간 동일한 dependency 조건이다. 기본 방향은 on-device-first이며 Qwen3-Omni가 일반 소비자 GPU에서 검증됐다고 가정하지 않는다. Core의 bounded Read/Search/Understand와 Agent 위임은 공통 허용 정책이며 외부 상태 변경은 Agent만 수행한다.

## 2. 공통 처리 계약

### 2.1 Identity와 수명

`voice_connection_id`, `conversation_id`, `turn_id`, `request_id`, `task_id`, `agent_execution_id`를 구분한다. 하나의 Turn은 여러 Request를 가질 수 있고 하나의 Task는 여러 Agent Execution에 순차/동시 연결될 수 있다. No Tracked Task는 Conversation 기록을 생략하라는 뜻이 아니다.

모든 메시지는 `schema_version, trace_id, event_id, causation_id, source_time, observed_time, request_revision`을 가진다. 필요한 Task/Execution ID는 명시적으로 추가한다. 원천 시각과 수신 시각을 섞지 않고, foreign provider ID는 VIA ID와 별도 필드로 보존한다. trace ID는 업무 authority가 아니다.

| 계약 | 최소 내용 / 완료 의미 | 오류·취소 규칙 |
|---|---|---|
| TurnEnvelope | 원문 Text/원음 참조, modality, 발화 시각, 전사 revision, 관측 가능한 Interaction event 참조 | partial 전사를 final로 확정하지 않음. Text는 Voice 연결을 요구하지 않음 |
| SemanticDecision | Request 목록, referent/source version, 명시 제약, Independent/Sequential/Data-dependent/Conditional edge, TaskRelation, Handling, Agent capability 요구 | READY / NEED_CONTEXT / CLARIFY / REJECT. 아직 미결인 request만 보류 |
| RouteLease | request 또는 확정 전 turn-scope, generation, route owner, release/dispatch 상태 | 같은 request revision의 중복 응답·위임 금지. 정정은 generation 변경 후 미전달 출력 폐기 |
| TaskCommand/View | command_id, expected_task_revision, decision/source 참조, 허용된 상태 전이 | stale revision은 재조회/재판정. 완료와 취소 요청을 같은 terminal 결과로 보지 않음 |
| ExecutionLink | task_id, provider/context/run identity, submission_key, attempt, active/superseded, 확인 시각 | transport 재시도는 같은 submission_key. 수정된 업무 의도는 새 command identity |
| AgentObservation | source run, schema/mode, 확인된 state, artifact/question/approval identity, 지원되면 source_revision | 수신 순번은 Agent의 진짜 source revision이 아님. 미지원 정보를 생성하지 않음 |
| PendingInteraction | question/approval ID, task/run, action revision, scope, destination, decision 상태 | 일반 대화의 '응'을 임의 승인으로 연결하지 않음. 변경된 action은 재승인 |
| ResponseOutput | request/task/result 참조, text/audio generation, 전달 offset, completed/interrupted | 생성한 내용과 실제 전달한 내용을 구분. S2S 답을 Core가 다시 생성하지 않음 |

SemanticDecision은 실행 권한이 아니다. 유효성·policy·Task revision 검사 후만 실행한다. CompoundCoordinator는 사용자가 명시한 의존만 집행하며 업무의 새 domain plan을 만들지 않는다.

### 2.2 Durable handoff와 복구

1. Task owner가 `intent + submission_key + outbox`를 한 짧은 transaction으로 저장한다.
2. EffectDispatcher가 transaction 밖에서 Agent를 호출한다.
3. Agent의 유효 접수/run identity를 확인한다.
4. 같은 Task owner가 ExecutionLink와 접수 상태를 저장한다. 이 event는 새 W-01의 handoff 진단 trace와 correctness 검증에 보존하지만 더 이상 W-02 종료점이 아니다.

Agent accepted 뒤 4 이전 crash는 외부 Agent의 submission-key 조회/중복 억제 계약이 있을 때만 같은 실행으로 조정한다. 이 기능이 없으면 확인 불가로 표시하고 state-changing Action을 자동 재발행하지 않는다. Outbox는 외부 exactly-once 보장이 아니다. [E2](../../../archive/w12-g1/evidence-and-readiness.md)

두 Task 대안 모두 동일한 durable snapshot + inbox/outbox 저장 계약을 사용한다. journal vs snapshot을 동시에 바꾸지 않는다. 참조 저장 엔진은 SQLite WAL, `synchronous=FULL`, 단일 PC의 같은 DB 파일이다. 네트워크/LLM 응답을 기다리며 transaction/Task lock을 잡지 않는다. SQLite writer 직렬화는 두 후보에 공통이므로 actor가 자동으로 DB write 병렬성을 얻는다고 하지 않는다. [E3](../../../archive/w12-g1/evidence-and-readiness.md)

### 2.3 정상 응답·실패 경로

같은 generation에 `S2S_DIRECT` 또는 `CORE_PROCESSING` release 권한은 한 곳이 소유한다. 한 Turn의 다른 Request는 서로 다른 처리 경로일 수 있다. 음성 interruption은 출력 generation만 중지하며 명시적 cancel command 없이 Agent 업무를 취소하지 않는다.

일시적 transport 실패는 같은 idempotency identity로 재시도한다. semantic correction은 provenance와 이전 판단을 보존하여 재판정한다. 재시도·source 재조회·검증 호출은 실제 call graph와 비용에 남긴다. 예상된 성공 경로만 평가하지 않는다.

## 3. 참조 runtime/API와 자원

| 항목 | 설계 단계에서 정한 값 | 실행 전 증거 조건 |
|---|---|---|
| VIA | Windows 참조, Rust 시제품, 같은 toolchain/release build | toolchain/OS/build hash를 실행 manifest에 기록 |
| Semantic dependency | Qwen3-8B Q4_K_M, non-thinking, local llama-server 계열, schema-constrained final JSON | 실제 GGUF/tokenizer/template/runtime digest와 입출력 token ledger 필요 |
| S2S dependency | Qwen3-Omni-30B-A3B-Instruct, 동일 배치·모드 | 입력 전사/word-time/control event는 native 제공을 가정하지 않음. 필요한 helper를 연결하고 비용 계측 |
| Agent | AF-v1의 query·stream 양쪽과 P/Q native envelope | 시험용 계약. 실제 A2A의 revision/replay/idempotency 보장이라고 주장하지 않음 |
| 외부 망 | FA-11의 RTT 50ms, 0/150ms sensitivity | payload bytes/직렬화/전송은 별도 span |
| CPU 실행 | 같은 총 worker 수, foreground/control/event queue별 bounded admission | 선정 장비에서 동일 CPU budget. 프로세스가 늘어도 총 worker budget 불변 |
| Model 호출 | 역할별 concurrency 1 기본, 동일 cache 정책 | 실제 공유 GPU면 resource ID도 공유. 모델 call의 '병렬' 표기를 실행 병렬로 간주하지 않음 |
| DB | 같은 SQLite/WAL/FULL 설정과 corpus | 실제 SQLite build와 storage latency 기록 |

Architecture candidate review에서 SKU나 runtime hash가 없는데 임의의 수치로 채우지 않는다. **설계 리뷰 가능과 실측 실행 가능은 별도 상태**다. supporting model binding이 확인되지 않은 경로는 actual-model 평가를 BLOCKED로 표시하되 구조 원장·static 분석은 진행한다.

### 3.1 EXEC IPC transport

EXEC-DP01의 Architecture decision은 **OS process fault boundary**다. IPC 종류를 동시에 비교변수로 만들지 않는다.

- smoke prototype: portable child stdin/stdout pipe로 semantic equivalence와 worker-fatal survival만 검증.
- macOS measured prototype: Measurement Freeze Review에서 하나의 local IPC transport를 고정한 뒤 A/B 전체 trial에서 유지.
- Windows target realization: named pipe 등 Windows-native local IPC를 별도 confirmatory implementation으로 둘 수 있다.
- macOS에서 얻은 IPC 절대 latency를 Windows named-pipe latency라고 표기하지 않는다.

## 4. 공통 Architecture Element 원장

아래 ID는 10의 같은 granularity를 적용한다. 내부 helper/메서드, 그림의 상위 VIA 박스, Task 인스턴스 수는 추가 집계하지 않는다. 한 표의 각 행은 독립 책임·계약·상태·배치이고, 상세 필드는 §2와 각 DP 문서에서 정의한다. 후보 전수 요소는 공통 원장 + 4개 Core DP에서 선택한 alternative 원장의 합집합이다.

### COMMON

| Element ID | 책임·계약·데이터·배치 | 소유자 / 소비자 / 수명 | 독립 변경 판정 |
|---|---|---|---|
| VIA-C-VOICE | 원음/전사/audio generation 및 Voice Connection 입출력 | VoiceRuntime / router·UI / 연결 수명 | 음성 수명·취소·전사 연동 행위 변화 |
| VIA-C-UI | Text 입력·Chat/Task 표시·알림 | Presentation / 사용자 / 프로세스 수명 | 표시·입력·전달 확인 행위 변화 |
| VIA-C-CAPTURE | display/window/selection/pointer/focus 원천 수집 | Capture / Context / 관측 수명 | OS capture 계약 적응 행위 변화 |
| VIA-C-CONTEXT | source 읽기·권한·version·scope와 content 제공 | ContextBroker / semantic·bounded / 요청 수명 | Source API/형식 처리 행위 변화 |
| VIA-C-HISTORY | canonical 기록에서 model context 구성 | HistoryProvider / models / 요청 수명 | 이력 선택·복원 행위 변화 |
| VIA-C-RECORD | 실제 입력·응답·referent 대화 기록 | ConversationRecorder / history·UI / 대화 수명 | 기록·삭제·전달 상태 처리 변화 |
| VIA-C-SEMCLIENT | Text Model 호출·stream→final 연결 | SemanticAdapter / IR / 호출 수명 | model protocol/오류/취소 적응 변화 |
| VIA-C-S2SCLIENT | Omni 입출력과 VIA event 연결 | S2SAdapter / Voice / 연결 수명 | model event/음성 계약 적응 변화 |
| VIA-C-BOUNDED | 지정 자료의 제한된 읽기·설명 | BoundedResponder / route·UI / 요청 수명 | bounded 기능·모델 사용 행위 변화 |
| VIA-C-POLICY | consent·scope·action revision 사용 시점 검사 | PolicyAuthority / 모든 외부 사용 경계 / 정책 수명 | 접근·반출·승인 의미 변화 |
| VIA-C-MEMORY | 기억 사용·조회·수정·삭제와 tombstone | MemoryService / history·사용자 / 영속 | Memory 정책·이행 행위 변화 |
| VIA-C-REGISTRY | Agent capability와 연결 설정 관리 | CapabilityRegistry / IR·integration / 연결 수명 | capability 등록·검색 행위 변화 |
| VIA-C-EFFECT | outbox 전송·retry·접수 응답 전달 | EffectDispatcher / TaskOwner·AgentBoundary / 시도 수명 | 외부 effect 전달·불확실성 처리 변화 |
| VIA-C-RESPONSE | 의미 있는 Text/Voice 결과 구성 | ResponseComposer / UI·Voice / 응답 수명 | 응답 구성·Task binding 행위 변화 |
| VIA-C-STORE | transaction·CAS·snapshot·inbox/outbox 저장 | Repository / state owners / 영속 | 읽기·쓰기·migration 행위 변화 |
| VIA-C-TRACE | correlation·span·error 증거 수집 | Telemetry / evaluator / 실행 수명 | 관측 의미·수집 행위 변화 |
| VIA-C-NATIVE | P/Q Agent와 Source transport 호출 | NativeClients / integration / 연결 수명 | native method·인증·transport 행위 변화 |
| VIA-C-AGENTSYNC | Event-first 수집 + query reconciliation·fallback | AgentSync / TaskOwner / active execution | stream/query 지원·gap/reconnect·cadence 행위 변화 |
| VIA-C-COMPOUND | 사용자 명시 Request edge의 readiness 확인·command 제출 | RelationScheduler / TaskOwner / 복합 요청 수명 | 의존·조건 집행 행위 변화. domain planning 아님 |
| VIA-I-TURN | TurnEnvelope·revisioned input | ingress→router | 입력/정정/끝 의미 변화 |
| VIA-I-CONTEXT | versioned scoped source/context 접근 | ContextBroker→consumer | version/scope/error 계약 변화 |
| VIA-I-DECISION | SemanticDecision·clarification·need-context | IR→router·TaskOwner | 판단 산출물·오류·수명 변화 |
| VIA-I-TASK | command/revision/view | TaskOwner→UI·router·effect | 상태 전이·command 완료 의미 변화 |
| VIA-I-NATIVE-P | Agent P native 접수·query·stream·control·result 계약 | NativeClients/해석 경계↔Agent P | native method·payload·오류·지원 의미 변화 |
| VIA-I-NATIVE-Q | Agent Q context/run 분리 native 계약 | NativeClients/해석 경계↔Agent Q | 별도 provider의 식별·follow-up·artifact 의미 변화 |
| VIA-I-EXECID | 최소 execution/submission identity | integration→TaskOwner | 접수·run/context 식별 의미 변화 |
| VIA-I-OBS | 검증된 Observation 제출 | synchronizer→TaskOwner | source/freshness/correlation 의미 변화 |
| VIA-I-OUTPUT | ResponseOutput·delivery receipt | composer→UI·Voice | 유효 전달·중지 계약 변화 |
| VIA-I-AUTH | scope/destination/pending-action 사용 판정 | PolicyAuthority→경계 | 허용·철회·revision 의미 변화 |
| VIA-I-STORE | atomic commit/CAS·recoverable read | Repository→state owners | transaction·복구·conflict 의미 변화 |
| VIA-I-TEXTMODEL | 텍스트/structured 모델 호출·취소 | SemanticAdapter→IR | partial/final/token/error 의미 변화 |
| VIA-I-S2S | S2S 입력/응답·지원 기능 선언 | S2SAdapter→Voice | 오디오·전사·control 제공 의미 변화 |
| VIA-I-HISTORY | 대화·Task·Memory 참조 선택 | HistoryProvider→models | 이력/삭제/provenance 계약 변화 |
| VIA-I-SYNC | observation·snapshot·cursor/reconcile 제출 | AgentSync↔AgentBoundary·TaskOwner | freshness·gap·완료 의미 변화 |
| VIA-S-CONV | 실제 대화·전달 상태·reference 기록 | Recorder writer / history reader / 영속 | 독립 보관·삭제·schema 이행 변화 |
| VIA-S-TASK | 목표·Request graph·local revision·사용자 상태 | 선택 TaskOwner writer / UI·scheduler reader / 영속 | Task 상태와 관계의 schema 이행 변화 |
| VIA-S-LINK | Task↔Agent context/run/submission 상관 | TaskOwner writer / effect·sync reader / 영속 | 실행 재연결·대체 run schema 변화 |
| VIA-S-PENDING | clarification/approval binding과 답변 상태 | TaskOwner·request handler / 정책·UI / 해결까지 영속 | 질문/승인 identity·revision 이행 변화 |
| VIA-S-DELIVERY | outbox·inbox·command dedup·확인 상태 | TaskOwner/Repository / dispatcher·sync / 영속 | 재시도·중복·접수 schema 이행 변화 |
| VIA-S-POLICY | 허용·철회·scope·destination revision | PolicyAuthority / 경계 / 영속 | 동의·권한 저장 이행 변화 |
| VIA-S-MEMORY | 기억 내용·소유·삭제 tombstone | MemoryService / history / 영속 | Memory 저장 이행 변화 |
| VIA-S-ROUTE | request generation·출력/dispatch lease | 선택 router / Voice·Core / 현재 요청 | 중복·정정 조정 state 변화 |
| VIA-S-CAP | capability snapshot와 provider binding | Registry / IR·integration / 연결·설정 수명 | capability 저장/갱신 schema 변화 |
| VIA-S-SYNC | source watermark·subscription/query checkpoint | AgentSync / TaskOwner·recovery / execution 수명 | cursor·관측 완전성·복구 schema 변화 |
| VIA-D-SEMANTIC | Text Model dependency의 시작·연결·복구 배치 | 모델 adapter 소비 / VIA와 논리 분리 | local/remote 호출 결합·배치 변화 |
| VIA-D-S2S | S2S dependency의 시작·연결·복구 배치 | Voice adapter 소비 / VIA와 논리 분리 | 음성 모델 dependency 배치 변화 |

State는 wire DTO를 이름만 바꾼 중복이 아니다. 표에 별도 보관·갱신·삭제·복구 수명이 정의된 것만 S로 등록한다. 변경량은 이 전체 목록에 M/A/C를 적용한 Modified∪Added∪Removed ID 수이며, 기본 요소 수를 유지보수 점수로 사용하지 않는다.

## 5. 전수 sweep과 후보 교차 영향

각 DP 문서의 `H` metadata에는 재동결이 끝난 active hypothesis만 둔다. W-01~W-03은 새 applicability가 확정될 때까지 `H`에서 제외한다. 후보 결과 전 `Primary`, 점수, 개선율을 확정하지 않는다. 새 W-01/02/03은 [Voice Responsiveness](../../08-quality-attributes/voice-responsiveness.md)의 서로 다른 Voice endpoint를 사용하고, query/Poll delay나 LLM processing 평균을 runtime p95로 치환하지 않는다.

W-09는 whole restart 4 + integration fatal 2 strata, W-10은 external 24 + host-fatal 4 cells를 유지한다. **동일한 adapter fatal fault**가 자신의 host를 종료하게 하며 Single-process 후보에만 다른 결함을 심지 않는다. 재시작이 충분히 빠르면 두 안 모두 W-10 100%일 수 있다. 그때 interruption duration만 secondary로 남기며 deadline/분모를 바꿔 차이를 만들지 않는다.

W-11은 외부 endpoint가 실제 얻는 정보의 union이다. 배치가 같더라도 외부로 전달한 정보가 다르면 노출은 달라질 수 있다. 반대로 on-device working context만 다르면 그 자체는 remote 노출이 아니다. Supporting의 W-11을 강한 driver로 사전 승격하지 않는다.

평가 자산 상태와 사전 확인 조건은 [evidence-and-readiness](../../../archive/w12-g1/evidence-and-readiness.md)를 따른다.
