# 11-A/B 진행 및 근거 재검토 기록

> 검토일: 2026-09-21
> 상태: **09/10 및 11-A/B 승인 완료. Working ASR 12개 유지. 12 Gate 1 DP catalog 최종 확정, Gate 2 상세 candidate 설계는 미진행.**

## 1. 현재 준비 상태

| 항목 | 현재 상태 |
| --- | --- |
| 09 ASR↔UC/Change mapping | 사용자 승인 완료 상태로 metadata 정합화 |
| 10 Architecture Element counting | 사용자 승인 완료 상태로 metadata 정합화 |
| ASR-01 latency boundary | VIA 직접 Model/Context/orchestration/delivery 포함. Agent open-ended domain research/reasoning/planning/tool execution만 제외 |
| ASR-01 representative metric | 서로 다른 TC raw sample의 pooled p95 폐기. 6개 request class별 p95의 동일가중 Macro-p95로 보정 |
| Voice interruption | audio-stop latency는 secondary/regression observation. ASR-01 대표 score 제외 |
| ASR-02/03/06 scoring | ASR-02/03은 ASR-tagged atomic obligation degree metric으로 보정. ASR-02 48 / ASR-03 30 TC. strict PASS는 secondary. ASR-06은 27 TC strict pass-rate 유지 |
| Clarification | scripted follow-up을 거쳐 terminal success까지 완료해야 ASR-02 PASS. 질문만 맞게 한 HOLD는 completion PASS 아님 |
| Grounding oracle | identity/hit-test 우선, raw region fallback IoU≥0.50. 고정 temporal tolerance/4초 requirement 제거; source event timestamp/order로 판정 |
| Restart/race/recovery | deterministic Agent simulator + fault/event injection. restart는 실제 VIA process memory loss 후 persisted state와 살아 있는 Agent query로 복구 |
| Safety | 24 fixed opportunities 유지. wrong-scope를 destination/pending Action identity mismatch까지 강화. 11-C에서 score로 평가 |
| ASR-04/05 target | 사용자 승인: ASR-04≤2.0 / ASR-05≤3.0 elements/change. 11-C의 score-band 입력으로 사용 |
| Semantic Model evidence | Qwen3-8B Q4_K_M / Windows RTX 4060 planning profile 승인 |
| S2S Model evidence | Qwen3-Omni-30B-A3B-Instruct. concurrency=1 official theoretical audio first-packet 234ms, Thinker 75 tok/s, Talker 140 tok/s, RTF 0.47. remote/high-memory dependency reference |
| 공식 tokenizer/model snapshot | Qwen3-8B tokenizer/template/GGUF revision·hash 기록. 실제 prompt tokenization은 아직 NOT_RUN |
| 실제 VIA/model benchmark | NOT_RUN |
| 11-C | 7개 ASR target·0~5점·반복·집계 초안 작성. 사용자 리뷰 대기 |
| 12 | DP 도출·후보 비교·결정 미진행 |

## 2. ASR-01 Windows Consumer PC reference

주 planning source는 Windows 11 + NVIDIA RTX 4060 8GB에서 Qwen3-8B Q4_K_M을 full GPU offload한 공개 관측이다.

- prompt: 2,957 tokens
- output: 1,225 tokens
- prompt processing: 2,103.19 token/s
- generation: 40.58 token/s
- 약 7.2 GB VRAM / 16k context setting

Intel Core Ultra 9 185H + RTX 4070 Laptop / Ollama CUDA의 별도 5-run 공개 benchmark도 512/128 token에서 평균 TTFT 약 0.19s, prefill 약 2,614 token/s, decode 약 44.92 token/s를 보여 sanity check로 사용한다.

이 값은 VIA 실측 p95가 아니다. frozen prompt token count와 Architecture candidate의 call graph를 이용한 `ESTIMATED_MODEL_ONLY` planning subtotal에만 사용한다.

```text
T_model_complete_hat
= Nin / 2103.19
+ Nout / 40.58
```

Model load, Context access, queue, serialization, IPC/RPC, validation, Voice/Text delivery는 실제 구조에 맞게 별도 span으로 추가한다.

상세: [ASR-01 Qwen evidence](./asr01-qwen-evidence.md).

## 3. Grounding review

작은 ±100/500ms synchronization tolerance를 임의로 두지 않는다. Interaction evidence는 source timestamp와 ordering을 보존하고 object identity/hit-test로 판정한다.

1997년 Oviatt 연구는 historical background로만 유지한다. 당시 pen+speech/click-to-speak interface의 최대 lag 관찰값을 VIA의 4초 requirement로 직접 쓰지 않는다. 2023·2024의 최근 speech-gesture 연구는 referential gesture가 speech와 긴밀하게 정렬되고 대체로 동시 또는 선행한다는 방향을 재확인하지만, desktop pointer/selection system에 적용할 보편적인 시간 threshold를 제공하지 않는다.

따라서 **고정 temporal threshold는 없음**으로 결정한다. canonical fixture의 source event timestamp/order 자체가 oracle이며, current test에 필요한 pre-turn event는 fixture timeline에 명시적으로 포함한다.

Object identity가 없는 raw region만의 fallback에는 IoU≥0.50을 사용한다. 이는 computer-vision localization의 최소 overlap 관행을 빌린 fallback이며 UI identity 기반 판정보다 우선하지 않는다.

상세: [Grounding Oracle evidence](./grounding-oracle-evidence.md).

## 4. Safety review

24 opportunities는 다음 6개 enforcement boundary를 고정 분모로 유지한다.

- READ
- EGRESS
- APPROVAL
- REVOCATION
- ACTION_REVISION
- MEMORY

각각 valid allow / deny / stale / wrong scope 조건을 갖는다. wrong scope는 target이 존재하는 경우 다른 remote destination 또는 다른 pending Action identity를 직접 사용하도록 보완했다.

대표 raw data는 `V/24`이며 retry/guard/log 개수로 분모를 변경하지 않는다. 현재 사용자 결정에 따라 별도 hard gate는 두지 않고 11-C에서 0~5 score를 정의한다.

상세: [ASR-07 safety review](./asr07-safety-opportunity-review.md).

## 5. ASR-04/05 target rationale

외부 standard/report는 숫자 cutoff를 제공하지 않는다. SEI modifiability tactics 등의 핵심 원칙인 **change localization / ripple-effect control**을 근거로 하고, 숫자는 VIA의 product boundary에서 제안한다.

- ASR-04 proposal: Agent change당 평균 changed architecture elements ≤ **2.0**
  - Agent-specific adapter/binding + 필요 시 공통 registry/contract 한 곳까지
  - 3개 이상이 반복되면 Agent change가 orchestration core로 전파되는 coupling을 의심
- ASR-05 proposal: non-Agent change당 평균 ≤ **3.0**
  - Model/Context 변화는 보통 0~2
  - persistent schema evolution은 State + Repository + Migration의 3개 변경이 정당할 수 있음

이는 아직 11-C target approval 전의 proposal이다.

상세: [Change Locality rationale](./asr04-asr05-change-locality-rationale.md).

## 6. 승인된 S2S reference

S2S Model은 **Qwen3-Omni-30B-A3B-Instruct**로 고정한다. 공식 Technical Report의 concurrency=1 theoretical audio first-packet 234ms, Thinker 75 tok/s, Talker 140 tok/s, Generation RTF 0.47을 planning evidence로 사용한다. 공식 weight는 약 70.5GB이며 BF16 Instruct deployment memory도 일반 단일 consumer GPU 범위를 넘으므로, 현재 baseline은 S2S를 remote/high-memory GPU dependency로 보는 것을 허용한다. 원격이면 network/transport도 ASR-01에 포함한다.

상세: [Qwen3-Omni S2S evidence](./asr01-qwen3-omni-s2s-evidence.md).

## 7. 사용자가 지금 준비할 필요가 없는 것

현재 단계에서는 사용자가 local model server, API key, 특정 GPU 또는 VIA candidate 실행환경을 준비할 필요가 없다.

추후 actual measurement 단계에서는 하나의 reference dependency를 고정한다.

- Windows 11
- Qwen3-8B Q4_K_M frozen artifact
- non-thinking
- frozen llama.cpp release/commit + CUDA
- local OpenAI-compatible `llama-server`
- concurrency 1
- canonical context/workload
- warm/cold 상태 분리

지금은 공식 tokenizer snapshot + 공개 Windows consumer GPU profile + candidate별 exact prompt/call graph로 planning estimate를 만든다.

## 8. 리뷰 종료

사용자가 다음 세 항목을 모두 승인했다.

1. ASR-01의 6개 latency request class와 Macro-p95 방식
2. ASR-02 Task Completion Obligation Satisfaction Rate / ASR-03 Continuity Obligation Preservation Rate
3. ASR-04 target 2.0 및 ASR-05 target 3.0 elements/change
4. 11-A/B measurement/test definition의 review-complete 처리

따라서 11-A/B는 종료했다. 11-C 초안을 작성했으며 사용자 승인 전에는 12로 넘어가지 않는다.


## 9. 11-C 초안

[11-C Target & Scoring Baseline](../11c-target-and-scoring-baseline.md)에 다음 초안을 작성했다.

- ASR-01 target ≤2.0s, score 5≤1.0s … score 0>4.0s
- ASR-02/03 target ≥95%, obligation degree score
- ASR-04 target ≤2.0 elements/change
- ASR-05 target ≤3.0 elements/change
- ASR-06 target 100% deterministic scenario pass
- ASR-07 target 0/24 safety violation
- measured/estimated/replay/design-analysis evidence level 분리

Machine-readable scoring baseline과 helper도 `benchmark/rebaseline/`에 연결했다.


## 10. Reserve QA 정식 재평가

[08-B Reserve QA Formal Reassessment](../08b-reserve-qa-formal-reassessment.md)에서 기존 방법론으로 재평가했다.

- H/H 신규 승격 권고: Concurrent Task Scalability & Performance Isolation
- H/H metric 교체 권고: ASR-06 → Recovery Timeliness (strict recovery pass는 regression 유지)
- H/H 신규 승격 권고: Privacy Exposure Minimization
- H/H 신규 승격 권고: Dependency Failure Containment & Graceful Degradation
- 신규 ASR 비권고: Compute Efficiency, Diagnosability, Task State Freshness, Deployment Flexibility

사용자 승인 전에는 08/09/11의 확정 catalog를 변경하지 않는다.


## 11. 12 Gate 1 최종 확정

사용자 피드백을 반영해 2026-09-21 Gate 1을 최종 확정했다.

- **Core DP 6개:** INT-DP01, IR-DP01, TASK-DP01, AGENT-DP01, TASK-DP02, EXEC-DP01
- **Supporting DP 3개:** CTX-DP01, CTX-DP02, SEC-DP01
- MODEL-DP01은 on-device 기본 범위를 유지하기 위해 추가하지 않음.
- STATE-DP01은 별도 DP로 승격하지 않고 TASK-DP01 persistence tactic으로 유지.
- ORCH-DP01은 Core/Agent 혼합이 자연스러워 상호배타적 architecture decision이 아니므로 독립 DP에서 제거.
- 각 DP 대안은 authoritative owner / canonical contract / primary state path / process fault boundary 중 하나를 서로 다르게 정해야 한다.
- CTX-DP02→W-11은 on-device baseline에서 primary 인과가 아니므로 regression으로 내림.
- SEC-DP01의 W-11/W-12도 주요 discriminator가 아니라 secondary/regression/constraint로 처리.
- EXEC-DP01의 process-isolation 차이를 실제로 자극하도록 W-09/W-10에 integration-host fatal fault를 후보 결과 전에 추가.

다음 단계는 Gate 2이며 Core 6개부터 “잘 만든 A vs 잘 만든 B”의 C/I/S/D 및 공통 tactic을 동결한다.


## 12. Gate 2 자체 리뷰 보정

2026-09-22 Gate 2 상세화 후 사용자 지적과 외부 protocol/runtime 근거를 바탕으로 Core DP를 다시 검토했다. **후보 실행·score 산출 전 보정**이다.

- INT-DP01은 S2S Direct Response의 자연스러운 제품 흐름에서 Core 재허가 B가 강한 독립 이점을 갖지 못해 **FP-INT01 S2S Direct Fast Path 고정 원칙**으로 내림.
- TASK-DP02는 polling/streaming/query가 실제 protocol에서 보완적으로 공존하므로 **TASK-T01 Event-first + Query Reconciliation tactic**으로 내림.
- Gate 2 score pair는 **IR-DP01, TASK-DP01, AGENT-DP01, EXEC-DP01 4개**로 압축.
- IR-DP01의 W-05 Completion 차이는 Model-dependent empirical evidence로만 인정.
- TASK-DP01은 Agent harness의 표준 이분법이라고 주장하지 않고 shared-state service vs per-execution durable owner라는 일반 SW architecture family로 설명.
- AGENT-DP01은 wire protocol 선택과 분리된 semantic contract-boundary decision으로 명시.
- EXEC-DP01은 Rust/Tokio single-process async와 child process + Windows named-pipe IPC 모두 구현 가능하되 실제 isolation은 OS process boundary임을 명시.
- Gate 2 review guide에 W-01~W-12 전체 ASR 이름과 짧은 의미를 병기.


## 13. Gate 2 승인

2026-09-22 사용자가 Gate 2 구조를 승인했다.

- **승인된 비교 DP 4개:** IR-DP01, TASK-DP01, AGENT-DP01, EXEC-DP01
- **고정 원칙:** FP-INT01 S2S Direct Fast Path
- **공통 tactic:** TASK-T01 Event-first + Query Reconciliation
- CTX-DP01/CTX-DP02/SEC-DP01은 Supporting DP로 보존하되 현재 구현 우선순위에서는 제외.
- 다음 단계는 executable prototype/fixture와 측정 자산을 만들고 12-A sensitivity sweep을 수행하는 것.
- 후보 결과를 보기 전에 Gate 2 C/I/S/D, common contract, Working-ASR metric/target/score 계약을 유지한다.


## 13. Gate 2 v1.1 승인

2026-09-22 사용자가 자체 리뷰 후 Gate 2 구조를 승인했다.

- **승인된 score DP 4개:** IR-DP01, TASK-DP01, AGENT-DP01, EXEC-DP01
- **고정 Interaction 원칙:** FP-INT01 S2S Direct Fast Path
- **공통 synchronization tactic:** TASK-T01 Event-first + Query Reconciliation
- 다음 단계는 [12-03 Implementation & Test Readiness Plan](../12-03-implementation-and-test-readiness.md)에 따라 deterministic structural harness → non-semantic DP prototype → IR actual-model 실험 순으로 진행.
- 실제 후보 결과를 보기 전에 Gate 2 candidate structure와 Working-ASR metric/target/score 계약을 유지.
