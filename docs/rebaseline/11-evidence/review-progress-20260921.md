# 11-A/B 진행 및 근거 재검토 기록

> 검토일: 2026-09-21
> 상태: **09/10 승인 정합화 완료 / 11-A·11-B 사용자 리뷰 반영 중. 11-C·12는 미진행.**

## 1. 현재 준비 상태

| 항목 | 현재 상태 |
| --- | --- |
| 09 ASR↔UC/Change mapping | 사용자 승인 완료 상태로 metadata 정합화 |
| 10 Architecture Element counting | 사용자 승인 완료 상태로 metadata 정합화 |
| ASR-01 latency boundary | VIA 직접 Model/Context/orchestration/delivery 포함. Agent open-ended domain research/reasoning/planning/tool execution만 제외 |
| ASR-01 representative metric | 서로 다른 TC raw sample의 pooled p95 폐기. 6개 request class별 p95의 동일가중 Macro-p95로 보정 |
| Voice interruption | audio-stop latency는 secondary/regression observation. ASR-01 대표 score 제외 |
| ASR-02/03/06 scoring membership | Primary owner 기준으로 ASR-02 47 / ASR-03 23 / ASR-06 17 TC 고정 |
| Clarification | scripted follow-up을 거쳐 terminal success까지 완료해야 ASR-02 PASS. 질문만 맞게 한 HOLD는 completion PASS 아님 |
| Grounding oracle | identity/hit-test 우선, raw region fallback IoU≥0.50, 최소 4초 pre-turn interaction history 보존 |
| Restart/race/recovery | deterministic Agent simulator + fault/event injection. restart는 실제 VIA process memory loss 후 persisted state와 살아 있는 Agent query로 복구 |
| Safety | 24 fixed opportunities 유지. wrong-scope를 destination/pending Action identity mismatch까지 강화. 11-C에서 score로 평가 |
| ASR-04/05 rationale | change-locality 원칙과 VIA boundary로 target proposal ASR-04≤2.0 / ASR-05≤3.0 elements/change 작성. 아직 11-C 승인 전 |
| Qwen planning evidence | Qualcomm을 주 profile에서 제외. Windows 11 + RTX 4060 + Qwen3-8B Q4_K_M 공개 관측으로 교체 |
| 공식 tokenizer/model snapshot | Qwen3-8B tokenizer/template/GGUF revision·hash 기록. 실제 prompt tokenization은 아직 NOT_RUN |
| 실제 VIA/model benchmark | NOT_RUN |
| 11-C | target·0~5점·반복 수·최종 집계 미진행 |
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

Oviatt et al.의 multimodal interaction 연구에서 sequential pen→speech lag는 평균 1.4초, 관측치 전체가 4초 이내였으므로 **User Turn 시작 전 최소 4초 interaction history를 보존**한다. 4초는 matching tolerance가 아니라 evidence retention minimum이다.

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

## 6. 사용자가 지금 준비할 필요가 없는 것

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

## 7. 아직 닫지 않은 것

11-C로 넘어가기 전 사용자 승인 또는 추가 리뷰가 필요한 핵심은 다음이다.

1. ASR-01의 6개 latency request class와 Macro-p95 방식
2. ASR-04 target proposal 2.0 및 ASR-05 target proposal 3.0
3. 11-A/B의 현재 measurement/test definition을 review-complete로 볼지 여부

그 외 canonical TC membership, clarification rule, grounding oracle, restart/race stub 방식, safety 24 denominator, Windows Qwen planning profile 선택은 이번 리뷰 반영안으로 문서화했다.
