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
| 12개 전수 metric 및 24 change raw template | GENERATABLE | `catalog_check.py --output`으로 생성. metric은 null/NOT_RUN, template change row는 분석 입력용 |
| C/I/S/D·조합·분모 static 검사 | RUN_BY_REVIEW_TOOL | 구조 명세 검사이며 VIA 기능/성능 시험이 아님 |
| Rust structural A/B prototype / Windows integration | **PARTIAL / WINDOWS_NOT_IMPLEMENTED** | TASK/AGENT/EXEC deterministic prototype과 IR dry-run/structural harness, W-04/W-09/W-10 smoke는 구현됨. Windows-native target integration, 전체 candidate system, 제품 성능 실측은 아직 미완료 |
| Hosted Qwen3-8B reference inference | RUNNER_READY / NOT_RUN | OpenRouter `qwen/qwen3-8b`, provider `alibaba`, fallback off. JSON object + client schema validation/repair. target-PC/local absolute latency 주장은 금지 |
| S2S input transcript/word timing·control binding | UNVERIFIED | native 제공과 helper 필요 여부의 실제 계약 확인 필요 |
| runtime/OS/hardware/artifact digest | NOT_CAPTURED | 같은 장비로 실제 trial 시작할 때 기록 |
| W-10 28 cells의 구체 대상 | **MATERIALIZED / CANDIDATE-ENDPOINT_SMOKE** | `benchmark/rebaseline/gate2/w10-containment-cells.json`의 6×4 external + 4 integration-fatal cell을 유지한다. external fault는 candidate host가 connection-refused/no-reply socket call을 실제 시작한 상태에서 같은 host의 unaffected capability를 probe하고, Agent capability는 shared backend 또는 isolated worker를 실제 경유한다. integration-fatal은 shared whole-host restart와 isolated worker restart를 같은 deadline으로 비교한다. smoke는 metric-ineligible이며 frozen 대표값은 NOT_RUN |
| W-07/W-08 24-change raw design analysis | **MATERIALIZED / DESIGN_ANALYSIS** | `w07-w08-change-rules.json`에 M 9 / A 9 / C 6의 M/A/R 근거를 고정하고 `change_analysis.py`가 8 pair-label × 24 = 192 row를 현재 `source_revision`+`catalog_fingerprint`에 대해 검증한다. 동일 configuration 중복 label은 동일 결과를 강제하며 평균·score·winner는 Measurement Freeze 전 계산하지 않음 |
| W-11 20 protected-unit corpus | **MATERIALIZED / VALIDATED_ORACLE** | `benchmark/rebaseline/gate2/w11-protected-units.json`에 8개 frozen workload와 합성 fixture의 20 semantic fact/relationship unit을 고정. `w11_exposure.py`는 whole-workload union·handle reachable scope·기능 유지 guard를 검증하며 대표 exposure 값은 아직 NOT_RUN |
| W-12 24 safety opportunity oracle | **MATERIALIZED / VALIDATED_ORACLE** | `benchmark/rebaseline/gate2/w12-safety-opportunities.json`에 6 family×4 condition, allow 6 / block 18, stale 10→50→100ms timeline과 resource/target wrong-scope 축을 고정. `w12_safety.py`는 complete 24-set·dedupe·positive-control 분리·V/24 score band를 검증하며 candidate enforcement 결과는 아직 NOT_RUN |
| 기존 94 TC obligation의 terminal-outcome 완전성 | **AUDIT_IMPLEMENTED / PRE-MEASUREMENT** | `terminal_outcome_audit.py`가 94 TC, ASR-02/03/06의 48/30/27 membership, oracle↔flat obligation catalog 일치, scripted clarification TC-06.2/06.3/14.5의 CLARIFICATION+OUTCOME 동시 존재를 검사한다. clarification 질문만으로 completion 만점 처리하는 것을 금지하며 candidate 실행·score는 아직 NOT_RUN |
| measured score·winner | NOT_RUN | Gate 3 전 없음 |

실행 준비가 되지 않은 항목은 숫자 0이 아니다. 이번 Gate 2는 **설계 후보와 비교 계획의 리뷰**를 위한 것이며 executable benchmark freeze를 허위로 선언하지 않는다. 제품 장비·S2S 지원을 확인하지 않고 모든 12개 empirical metric을 당장 계산할 수 있다고 약속하지 않는다.

## 4. 실행 전 통과 조건 — 작업 담당은 설계/구현 측

1. 각 후보의 full element 목록·구성·문서 hash를 Gate 2 승인본으로 고정한다. 본문/표/JSON mismatch는 수정하고 원장 버전을 올린다.
2. 실제 schema/prompt/request body를 직렬화하고 hosted model id/provider/sampling/profile과 request digest를 남긴다. IR B의 실제 stage/bypass/repair도 같은 ledger에 기록한다.
3. S2S의 input transcript, final/partial, timestamps, output cancel 기능을 실제로 확인한다. 필요한 helper를 양 후보 동일하게 넣고 call graph·지연·resource를 갱신한다.
4. Agent 기능 profile을 baseline/full-capability/limited로 분리한다. AF-v1의 revision/idempotency 보장과 실제 native 보장을 혼동하지 않는다.
5. W-10 cell 명세는 `benchmark/rebaseline/gate2/w10-containment-cells.json`, W-11 protected-unit oracle은 `benchmark/rebaseline/gate2/w11-protected-units.json`, W-12 safety opportunity oracle은 `benchmark/rebaseline/gate2/w12-safety-opportunities.json`으로 materialize했다. W-11은 8개 frozen workload의 기능 성공을 전제로 remote endpoint에 직접 전달되거나 handle로 reachable한 20 unit의 whole-workload union을 센다. W-12는 reviewed 6 family×4 condition의 24개 opportunity를 정확히 한 번씩 기록하고, expected BLOCK인데 ALLOW된 opportunity만 V에 포함하며 valid allow 6개 차단은 positive-control failure로 별도 공개한다. W-11/W-12 모두 대표 candidate 측정값은 아직 NOT_RUN이고, 요구 의미가 달라지면 조용히 메우지 않고 별도 rebaseline한다.
6. W-09 whole/fatal controller와 W-10 strict 28-cell controller를 구현했다. W-10 external smoke는 candidate host 자체가 connection-refused/no-reply dependency call을 시작하고 같은 candidate endpoint에서 unaffected capability를 probe한다. Agent Task probe는 shared DeterministicAgent 또는 isolated ProcessBridge를 실제 경유하며, integration-fatal은 shared 후보에도 whole-host restart를 허용해 isolated worker restart와 동일 deadline에서 판정한다. CI smoke timing은 process probe 안정성을 위해 별도 metric-ineligible profile을 쓰고 frozen 30s·5회·2/10/20s·5s 계약은 변경하지 않았다. 대표 W-10 metric은 Measurement Freeze 이후 frozen profile 실행 전까지 NOT_RUN이다.
7. actual-model, deterministic structural replay, design-analysis 결과를 분리한다. W-07/W-08의 24-change raw M/A/R 원장은 `DESIGN_ANALYSIS`로 materialize하고 현재 baseline fingerprint와 기능 유지 논증을 함께 남긴다. 이 raw count는 Measurement Freeze 전 대표 평균·score로 집계하지 않는다.
8. W-05/W-06 실행 전 `build_assets.py` 산출물에 `terminal_outcome_audit.py`를 적용한다. scripted clarification이 있는 canonical ASR-02 TC는 clarification binding과 별도의 OUTCOME obligation을 모두 가져야 하며, 11-B source row와 generated obligation ledger의 terminal 의미 marker가 일치해야 한다.

이들은 사용자가 모델 서버/API를 지금 준비하라는 요청이 아니다. 사용자 리뷰는 후보의 공정성·경계·업무상 타당성에 집중하고, 이 기술적 준비는 다음 구현/검증 산출물의 명시적 완료 조건으로 관리한다.
