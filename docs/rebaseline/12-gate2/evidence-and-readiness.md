# Gate 2 — 근거, 제약, 실행 준비 상태

> 2026-09-22 / G2-DESIGN-v1.1. 자체 리뷰 반영. 설계 근거와 실제 후보 측정 결과를 구분한다.

## 1. 외부 1차 근거

| ID | 확인한 1차 자료 | 이 설계에 반영한 사실 | 주장하지 않는 것 |
|---|---|---|---|
| E1 | A2A Specification **v0.3.0**, https://a2a-protocol.org/v0.3.0/specification/ | query·stream·cancel·resubscribe가 별도 계약이며 resubscribe의 누락 event 재생은 구현 의존적이다. | 모든 Agent의 source revision, backfill, exactly-once 또는 submit-key 조회가 보장된다는 주장 |
| E2 | AWS Transactional outbox, https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html | DB 상태와 외부 메시지의 dual-write를 분리하는 outbox·idempotent 처리 필요성을 참고했다. | local outbox commit이 외부 접수나 exactly-once Action을 증명한다는 주장 |
| E3 | SQLite WAL / Isolation, https://www.sqlite.org/wal.html , https://www.sqlite.org/isolation.html | WAL에서 reader/writer 동시성은 가능하지만 writer는 직렬화된다. | Task별 actor를 두면 같은 DB에서 write가 무제한 병렬이라는 주장 |
| E4 | Azure Bulkhead, https://learn.microsoft.com/en-us/azure/architecture/patterns/bulkhead | 실행/자원 경계는 장애 영향 격리 수단이며 비용도 가진다. | 별도 process면 공유 GPU·DB·OS 장애까지 자동 격리된다는 주장 |
| E5 | llama.cpp server README, https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md | chat-completion·schema-constrained JSON·cache/parallel 설정을 제공한다. | 선택 Qwen artifact·실행 build가 본 작업에서 실제 검증됐다는 주장 |
| E6 | Qwen3-Omni official README, https://github.com/QwenLM/Qwen3-Omni | text/audio 등 입력과 streaming text/speech 출력 모델임을 확인했다. | 입력 단어별 timestamp, VIA 전용 routing event, 지정 PC에서의 p95를 제공한다는 주장 |
| E7 | Microsoft Orleans request scheduling, https://learn.microsoft.com/en-us/dotnet/orleans/grains/request-scheduling | actor별 serial execution과 async 대기/reentrancy를 구분하는 참고 사례다. | VIA가 Orleans/.NET으로 구현된다는 주장. VIA prototype은 Rust 조건 유지 |
| E8 | LangGraph Thinking in LangGraph, https://docs.langchain.com/oss/javascript/langgraph/thinking-in-langgraph | thread state/checkpointer 기반 persistent shared state와 node-boundary durable execution 사례 | LangGraph가 VIA의 A 후보와 동일하다는 주장 |
| E9 | Temporal Workflow Execution/Event History, https://github.com/temporalio/documentation/blob/main/docs/encyclopedia/workflow/workflow-execution/workflow-execution.mdx | Workflow Execution별 durable/recoverable execution과 Event History가 per-execution durable-owner family의 실재를 보여줌 | Temporal 자체를 VIA에 도입하거나 B 후보가 Temporal 구현이라는 주장 |
| E10 | Tokio process / Windows named pipes, https://docs.rs/tokio/latest/tokio/process/ , https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/ | Rust/Tokio에서 child process 관리와 Windows async named-pipe IPC가 가능 | Tokio가 OS process isolation을 대신하거나 IPC가 무료라는 주장 |

E1은 고정 공개 버전이다. 나머지 live 문서는 2026-09-22 열람 기준 설계 근거이며 binary/runtime 성능 profile이 아니다. 구현 실행 때 필요한 digest는 별도 manifest에서 잠근다. 논문·표준이 VIA의 후보 수·score band를 정해줬다고 설명하지 않는다.

## 2. 새로 발견한 설계상 주의점

**중앙=느림이라는 가정 금지.** 같은 process의 Core 경유는 inline 함수 호출일 수 있다. INT의 차이는 경유 박스 개수가 아니라 실제 release/queue/semantic 처리 경로로 판단한다.

**Actor=write 병렬이라는 가정 금지.** TASK 두 안에 동일 Repository를 적용한다. DB 병목을 없애려고 B만 별도 DB/sharding을 사용하면 복합 구조 변경으로 재분류한다.

**Query/event는 서로 다른 진실이 아니다.** Agent가 실행 사실을 소유하며, TASK-DP02는 정상 관측 갱신 경로를 선택한다. Event source의 revision/retention 보장은 실제 provider에서 확인한다.

**Hybrid가 존재하면 DP가 무효라는 일반론은 사용하지 않는다.** 비교 범위 안에서 실제 선택이 있고 그 조합에 조정/계약/수명 비용이 있는지가 핵심이다. 조합이 사실상 무료이고 차이가 사라지면 해당 DP의 발표 중요도를 낮춘다.

**On-device-first와 검증 완료는 다르다.** MODEL-DP를 재개하지 않는다. Qwen3-Omni를 임의 GPU에 올렸다고 가정하거나 remote 예외를 사용자 승인 없이 최종 deployment로 확정하지 않는다. 같은 고정 S2S binding을 사용할 수 있을 때만 실제 latency 비교를 진행한다.

**Privacy 관련 Gate 1 문구의 적용 한계.** 배치가 고정됐다는 이유만으로 W-11이 반드시 같아지는 것은 아니다. 실제 외부 전달 scope가 다르면 차이를 기록하되, local model context 차이를 remote exposure로 세지는 않는다. 이 명확화는 W-11 metric/target 변경이 아니다.

**INT-DP01/TASK-DP02 탈락:** Gate 2 자체 리뷰에서 S2S direct fast path는 현재 제품 흐름상 자연스러운 fixed principle이고, Agent status는 query+event가 보완적으로 공존하는 것이 현실적이므로 두 축을 A/B score 대상에서 제외한다. 이는 결과를 보고 불리한 DP를 삭제한 것이 아니라 후보 실행·score 산출 전 design review 결과다.

## 3. Gate 2 산출물 상태

| 자산 | 현재 상태 | 의미 |
|---|---|---|
| Core 6개 × A/B 구조 명세 | REVIEW_READY | component·interface·state·deployment·flow·공통 보완책 작성 |
| 공통/후보 Element 원장 | REVIEW_READY | full configuration 조립 규칙·ID·변경 판정 정의 |
| 12개 전수 평가 및 24 change raw template | GENERATABLE | `catalog_check.py --output`으로 생성. 모든 측정값 null |
| C/I/S/D·조합·분모 static 검사 | RUN_BY_REVIEW_TOOL | 구조 명세 검사이며 VIA 기능/성능 시험이 아님 |
| 실제 Rust A/B 구현·Windows integration | NOT_IMPLEMENTED | diagram을 실행 구현이라고 주장하지 않음 |
| 실제 Qwen prompt token ledger·inference | NOT_RUN | prompt/schema 계약은 정의, 실제 직렬화·모델 실행은 미완료 |
| S2S input transcript/word timing·control binding | UNVERIFIED | native 제공과 helper 필요 여부의 실제 계약 확인 필요 |
| runtime/OS/hardware/artifact digest | NOT_CAPTURED | 같은 장비로 실제 trial 시작할 때 기록 |
| W-10 28 cells의 구체 대상 | MATERIALIZED / STRUCTURAL_SMOKE | `benchmark/rebaseline/gate2/w10-containment-cells.json`에 6×4 external + 4 integration-fatal cell을 materialize. frozen contract를 controller가 검증하지만 대표 metric은 NOT_RUN |
| W-11 20 protected-unit corpus | ASSET_GAP | 기존 11-D가 요구하는 20-unit oracle은 아직 machine-readable asset으로 materialize되지 않음 |
| 기존 94 TC obligation의 terminal-outcome 완전성 | AUDIT_REQUIRED | 예: clarification 질문만으로 completion을 만점 처리하면 안 됨 |
| measured score·winner | NOT_RUN | Gate 3 전 없음 |

실행 준비가 되지 않은 항목은 숫자 0이 아니다. 이번 Gate 2는 **설계 후보와 비교 계획의 리뷰**를 위한 것이며 executable benchmark freeze를 허위로 선언하지 않는다. 제품 장비·S2S 지원을 확인하지 않고 모든 12개 empirical metric을 당장 계산할 수 있다고 약속하지 않는다.

## 4. 실행 전 통과 조건 — 작업 담당은 설계/구현 측

1. 각 후보의 full element 목록·구성·문서 hash를 Gate 2 승인본으로 고정한다. 본문/표/JSON mismatch는 수정하고 원장 버전을 올린다.
2. 실제 schema와 prompt를 직렬화하여 공식 tokenizer로 세고 runtime/template/model digest를 남긴다. IR B의 실제 stage/bypass/repair도 같은 ledger에 기록한다.
3. S2S의 input transcript, final/partial, timestamps, output cancel 기능을 실제로 확인한다. 필요한 helper를 양 후보 동일하게 넣고 call graph·지연·resource를 갱신한다.
4. Agent 기능 profile을 baseline/full-capability/limited로 분리한다. AF-v1의 revision/idempotency 보장과 실제 native 보장을 혼동하지 않는다.
5. W-10 cell 명세는 `benchmark/rebaseline/gate2/w10-containment-cells.json`으로 materialize했다. W-11 protected-unit oracle은 아직 남아 있다. 요구 의미가 달라지면 조용히 메우지 않고 별도 rebaseline한다.
6. W-09 whole/fatal controller와 W-10 strict 28-cell controller를 구현했다. W-10 smoke는 transient integration fatal과 persistent external connection-refused/no-reply를 구분하고 frozen 28-cell 분모·반복·deadline을 검증한다. 단, external capability probe가 아직 deterministic fixture이므로 대표 W-10 metric은 NOT_RUN이며 실제 candidate endpoint adapter 연결 후에만 measurement-eligible하다.
7. actual-model, deterministic structural replay, design-analysis 결과를 분리한다. W-07/08 설계 변경량은 실행 환경 없이도 별도 근거 원장을 작성할 수 있지만 그 경우 DESIGN_ANALYSIS로 표시한다.

이들은 사용자가 모델 서버/API를 지금 준비하라는 요청이 아니다. 사용자 리뷰는 후보의 공정성·경계·업무상 타당성에 집중하고, 이 기술적 준비는 다음 구현/검증 산출물의 명시적 완료 조건으로 관리한다.
