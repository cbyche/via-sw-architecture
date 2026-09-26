# Voice Responsiveness Measurement Contract

> 상태: **PRE-IMPLEMENTATION CONTRACT DRAFT / 결과 미확인**
>
> 목적: QA Catalog의 QA-01~QA-04를 Voice 중심 사용자 경험과 DP별 A/B 직접 비교에 맞게 정의한다. 측정 코드는 아직 변경하지 않는다.
>
> 효력: 이 문서는 QA-01~QA-04의 current semantic definition이다. [`scoring-contract.md`](../11-measurement/scoring-contract.md)는 이 정의를 따르며 기존 코드·freeze·결과는 새 정의의 결과로 재사용하지 않는다.
>
> 공통 event의 정확한 의미와 관측 규칙은 [`event-boundary-contract.md`](../11-measurement/event-boundary-contract.md)를 따른다.

## 1. 공통 원칙

- 대표 경로는 **Voice input → Voice output**이다. Text-only latency는 secondary/regression이며 대표값에 넣지 않는다.
- 출력 종료점은 payload가 sink에 도착한 시각이 아니라 **첫 audible audio sample이 실제 재생되기 시작한 시각**이다. speech 생성, playback queue, audio buffer와 device/renderer start 지연을 포함한다.
- 실제 제품 측정은 audio loopback 등으로 audible onset을 관측한다. instrumented playback sink는 reference-harness evidence일 뿐 실제 speaker 출력이라고 주장하지 않는다.
- QA-01은 Agent delegation, QA-02는 downstream Agent가 없는 direct response, QA-03은 Agent progress/status 전달, QA-04는 Voice interruption을 각각 측정한다. 서로 평균내지 않는다.
- 각 DP는 다른 DP 조건을 고정한 A/B paired run으로 직접 비교한다. 여러 DP 조합의 global winner를 이 세 지표의 기본 분석 단위로 삼지 않는다.
- 잘못된 결과·status·Task binding은 빠른 성공으로 세지 않는다. timeout과 correctness failure를 raw trace에 남기며 성공 표본만 선택해 p95를 만들지 않는다.
- 기존 제안인 stratum별 10 warm-up + 100 scored trials, nearest-rank p95, case p95의 동일가중 Macro-p95는 새 fixture freeze에서 재승인하기 전까지 **proposed** 상태다.

## 2. QA-01 — Delegated Path VIA Responsiveness

### 의미

Voice로 요청한 Agent delegation 업무에 대해 VIA가 요청을 해석·위임하고, Agent 결과가 준비된 뒤 이를 다시 Voice로 사용자에게 전달하는 데 소비한 **VIA 책임시간**을 측정한다. Agent 외부의 접수 대기·queue·실제 실행·완료 대기는 제외하지만 전체 wall-clock은 secondary로 보존한다.

### 시간 경계

```text
t0 = user_input_end
t1 = agent_request_available_at_agent_ingress
t2 = agent_result_available_at_source
t3 = first_meaningful_audible_result_audio

QA-01 sample = (t1 - t0) + (t3 - t2)
full_user_wait_secondary = t3 - t0
excluded_agent_interval = t2 - t1
```

`agent_request_available_at_agent_ingress`는 검증된 위임 요청이 Agent ingress에서 소비 가능해진 최초 시각이다. local dispatch commit, Agent 계약 변환, serialization, EXEC queue/IPC와 outbound transport를 이 event 전에 포함한다. downstream Agent 내부의 acceptance queue와 실행은 QA-01에 포함하지 않는다. `agent_result_available_at_source`는 결과가 VIA 밖의 Agent source에서 조회 또는 stream 가능한 최초 시각이며, 중간 broker가 늦게 읽었다고 시작점을 뒤로 옮기지 않는다.

여러 Agent가 겹치는 compound request는 외부 interval을 단순 합산하지 않는다. representative fixture에 포함하려면 결과에 필요한 Agent interval의 union과 first-result policy를 별도 동결해야 한다. 그 전에는 single-delegation case만 대표값에 사용한다.

### 포함 경로

```text
Voice 발화 종료
→ turn/input 확정
→ Conversation·Task·Context materialization
→ VIA LLM 요청 정리·Task 연관·Agent 선택·위임 판단
→ structured output 완성·검증·repair
→ durable local dispatch/correlation commit
→ Agent 계약 변환·serialization·EXEC queue/IPC·outbound transport
→ Agent ingress에서 요청 사용 가능
→ [downstream Agent interval 제외]
→ Agent result source event 수신
→ Task/run correlation 및 결과 검증
→ 필요한 VIA LLM 결과 요약·Voice response composition
→ speech 생성
→ playback queue/buffer/device
→ first meaningful audible result audio
```

첫 접수 멘트나 “처리 중” status는 QA-01 종료점이 아니다. 실제 Agent 결과의 의미 있는 Voice 전달이 종료점이다.

### 대표 Metric

```text
worst_case_case_p95_delegated_via_active_time_ms
```

잠정 canonical 후보는 `TC-07.1`, `TC-08.1`, `TC-08.2`, `TC-10.4`다. 각 case에는 deterministic Agent result fixture와 `agent_request_available_at_agent_ingress`/`agent_result_available_at_source` event가 추가되어야 하며, 이 membership은 새 Measurement Freeze 전에 확정한다.

## 3. QA-02 — VIA Direct Voice Response Responsiveness

### 의미

downstream Agent delegation 없이 VIA가 직접 답하는 Voice request의 전체 반응시간을 측정한다.

### 시간 경계

```text
t0 = user_input_end
t1 = first_meaningful_audible_direct_response

QA-02 sample = t1 - t0
```

### 포함 경로

```text
Voice 발화 종료
→ turn/input 확정
→ 필요한 transcription/S2S input event 확정
→ Conversation·interaction·bounded Context materialization
→ VIA route/semantic decision
→ VIA LLM 또는 명시된 direct-response component
→ output validation·policy·repair
→ Voice response composition
→ speech 생성
→ playback queue/buffer/device
→ first audible direct-response audio
```

bounded Read/Search/Understand는 실행 위치가 Core인지 helper인지와 무관하게 direct response 결과에 필요하면 포함한다. open-ended downstream Agent를 시작하는 case는 QA-02 모집단에 넣지 않는다.

S2S-native direct response와 VIA LLM direct response는 별도 case stratum으로 보존한다. 무음, earcon, generic acknowledgement와 filler는 유효 direct response endpoint가 아니다.

### 대표 Metric

```text
worst_case_case_p95_direct_voice_response_ms
```

잠정 canonical 후보는 `TC-01.1`, `TC-02.1`, `TC-04.2`, `TC-06.2`다. `TC-01.4` Text-only는 secondary/regression으로 내린다. 각 case의 Voice fixture, Context 크기, validity oracle과 output contract는 새 freeze 전에 고정한다.

## 4. QA-03 — Agent Progress Voice Feedback Responsiveness

### 의미

Agent의 progress/status가 source에서 제공 가능해진 뒤 VIA가 이를 올바른 Task/run에 연결하고 사용자에게 Voice로 알려주는 Architecture 지연을 측정한다. Agent가 status를 생성하기까지의 시간은 제외한다.

### 시간 경계

```text
t0 = agent_status_available_at_source
t1 = first_meaningful_audible_status_audio

QA-03 sample = t1 - t0
```

### 포함 경로

```text
Agent status source event available
→ stream 수신 또는 polling 발견
→ Task/run correlation
→ revision·중복·최신성 검증
→ 필요한 state commit
→ 사용자용 status 문구 생성
→ speech 생성
→ priority playback queue/buffer/device
→ first audible status audio
```

유효 status는 실제 Request/Task/run과 연결되고 확인된 상태나 처리 단계에 근거해야 한다. 완료·접수·진행률을 사실보다 먼저 주장하거나, 동일한 무의미 문구를 반복하여 latency를 만족시켜서는 안 된다. 사용자가 발화 중이면 audio를 겹쳐 재생하지 않으며, 정책상 보류된 interval과 Text fallback은 raw trace에 별도로 남긴다.

primary reference stratum은 사용자가 발화 중이지 않고 audio lane이 비어 있는 조건을 고정한다. 사용자 발화나 우선순위 정책으로 의도적으로 보류된 시간은 별도 policy stratum으로 기록한다.

### 대표 Metric

```text
worst_case_case_p95_status_to_voice_ms
```

잠정 canonical 후보는 `TC-13.1` progress, `TC-13.2` Agent clarification/question, `TC-13.4` partial/failure status다. `TC-13.3` terminal result는 QA-01 결과 전달 fixture로 사용하고 QA-03 대표값에 중복 포함하지 않는다.

## 5. QA-04 — Voice Interruption Responsiveness

### 의미

VIA가 Voice Response를 재생하는 동안 사용자가 새로 발화했을 때, 현재 음성 출력이 실제로 멈추기까지의 시간을 측정한다. Voice interruption은 진행 중인 Agent 업무의 cancel과 다르다.

### 시간 경계

```text
t0 = barge_in_speech_onset
t1 = interrupted_response_last_audible_sample

QA-04 sample = t1 - t0
```

`barge_in_speech_onset`은 사용자의 새 발화가 microphone capture timeline에서 실제 시작된 acoustic 시점이다. VAD가 speech를 선언하거나 semantic intent를 확정한 시각이 아니다. `interrupted_response_last_audible_sample`은 중단 대상 audio가 실제 출력 경로에서 마지막으로 재생된 시각이다. cancel API return, queue clear 또는 renderer callback만으로 대체하지 않는다.

### 포함 경로

```text
실제 barge-in speech onset
→ microphone capture / echo handling
→ activity·turn detection
→ interruption routing
→ speech generation/stream cancellation
→ playback queue와 buffer 폐기
→ device/renderer stop
→ interrupted response의 last audible sample
```

새 User Turn의 semantic 처리나 새 응답 생성은 QA-04 종료점 뒤의 별도 경로다. Task cancel·correction·approval delivery는 endpoint가 다르므로 QA-04에 합치지 않는다.

### 대표 Metric

```text
worst_case_case_p95_barge_in_to_audio_stop_ms
```

잠정 canonical 후보는 `TC-11.1` S2S direct response interruption과 `TC-11.2` Core/Agent result Voice interruption이다. echo, buffered audio, streaming generation과 이미 종료 직전인 response 조건은 fixture freeze 전에 구분한다.

## 6. S2S 234ms의 사용 범위

Qwen3-Omni 공개 234ms는 concurrency=1에서 tail-packet preprocessing, Thinker/Talker TTFT, MTP와 codec decode를 포함한 **theoretical first-audio-packet runtime reference**다.

- QA metric의 시작점이 아니다.
- 사용자 발화 종료 감지, VIA orchestration, network, playback queue/device를 포함하지 않는다.
- VIA LLM이 생성한 Text를 음성으로 바꾸는 일반 TTS 지연으로 사용할 수 없다.
- 실제 candidate 경로가 동일한 full S2S first-packet 의미를 가질 때만 scheduled mock dependency span으로 사용할 수 있다.
- QA-01/QA-02/QA-03에 일괄 상수로 더하지 않는다.

## 7. VIA LLM latency planning contract

QA-01~QA-03은 target Mac의 local Qwen3-8B를 실제 호출한다. QA-04는 LLM token-rate가 아니라 실제 interruption path를 측정한다. 후보별 actual serialized prompt와 frozen tokenizer로 token 수를 기록하고, queue·prompt processing·first meaningful output·structured completion을 직접 계측한다.

각 LLM call은 다음을 결과 전에 고정한다.

- system prompt, output schema와 chat template revision
- Conversation·Task·Context와 user request
- serialized prompt SHA-256와 exact input token count
- endpoint에 필요한 output token count
- call dependency와 sequential/parallel 관계
- streaming first-meaningful-output인지 structured completion인지
- validation, retry와 repair policy

순차 call은 합산하고 실제로 독립 resource에서 병렬 실행 가능한 call만 critical-path max를 사용한다. network, IPC/RPC, Context access, validation, speech generation과 playback은 model subtotal에 섞지 않고 별도 실제 또는 frozen reference span으로 둔다.

실제 local model span은 `MEASURED_MODEL`로 기록할 수 있지만 physical audio endpoint가 없는 결과를 `PRODUCT_E2E`로 부르지 않는다. Smoke-test throughput을 이후 모든 prompt의 고정 속도로 환산하지 않는다.

## 8. Raw trace 최소 항목

QA-01~QA-03은 다음을 보존한다.

- metric/DP/candidate/case/trial identity
- Voice input SHA-256와 audio format
- monotonic clock identity와 모든 원시 event timestamp
- actual route와 component/call graph
- prompt/input/output token ledger
- Context/payload byte 크기와 digest
- source event, local dispatch commit, Agent ingress, result/status availability event
- speech generation start/end
- playback enqueue, device callback과 first audible audio
- correctness, timeout, retry/repair와 evidence scope
- QA-01의 included segments, excluded Agent interval과 full wall-clock

대표 sample은 component p95의 합이 아니라 각 trial의 endpoint 또는 QA-01 segmented interval에서 계산한다.

QA-04는 추가로 barge-in audio fixture, acoustic onset, VAD/activity detection, cancel signal, playback queue clear, renderer/device stop과 last audible sample을 보존한다.

## 9. 기존 자료와 후속 작업

- 기존 `reference_campaign.py`의 QA-01~QA-03 수식 결과는 W12-G1 historical/superseded evidence다.
- `benchmark/archive/w12-g1/working12/baseline.json`, `w01-foreground-strata.json`과 archived runner는 아직 이 문서와 일치하지 않는다. 이 문서 승인 후 별도 구현 작업에서 변경한다.
- 동시 Task는 독립 QA ratio가 아니라 QA-01~QA-04의 workload condition으로 둔다. 지원할 동시 Task 수는 제품 workload 근거와 함께 결과 전에 고정한다.
- QA-01~QA-04의 target과 0~5 score band는 기존 값을 이관하지 않는다. 새 representative fixture와 product budget을 결과 전에 별도로 승인한다.
