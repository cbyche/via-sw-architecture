# Measurement Event & Boundary Contract

> 상태: **W-01~W-03 EVENT CONTRACT DRAFT / machine freeze 전**
>
> 목적: 모든 W metric의 시작·종료 event를 사용자 또는 외부 source의 실제 사건에 고정하고, software가 그 사건을 뒤늦게 인식한 시간을 숨기지 않는다.
>
> 범위: 이 문서는 공통 event 의미와 관측 원칙의 source of truth다. W별 모집단·집계·target·score는 `scoring-contract.md`, 기능 fixture와 oracle은 `test-case-catalog.md`에서 관리한다.

## 1. 공통 원칙

1. **실제 사건과 software 인식 event를 구분한다.** 사용자 발화 종료, Agent ingress, result/status source availability, audible output이 metric 경계다. VAD 판단, local enqueue, VIA receive, audio enqueue는 진단 event다.
2. **Architecture가 만든 경계 비용을 숨기지 않는다.** 실제 critical path에 참여한 Context, IR, Task, Agent adapter, serialization, queue, IPC/RPC, network, validation, Model, speech와 playback 비용을 포함한다.
3. **실행 위치를 바꿔 시간을 제외하지 않는다.** 같은 책임을 helper, worker 또는 child process로 옮겨도 해당 W의 의미 경로에 필요하면 포함한다.
4. **유효한 결과만 endpoint로 인정한다.** 무음, earcon, filler, 접수 인사, spinner, invalid payload와 잘못 연결된 result/status는 종료점이 아니다.
5. **사용자 관점 endpoint와 harness proxy를 구분한다.** 제품 측정은 acoustic/audio loopback을 사용한다. instrumented device callback 또는 mock playback sink는 reference-harness proxy이며 실제 speaker 실측으로 부르지 않는다.
6. 모든 timestamp는 한 trial 안에서 monotonic clock domain으로 비교한다. 외부 source 사건은 source-controlled harness event, clock synchronization 또는 오차가 명시된 correlation으로 연결한다. VIA 수신시각으로 source event를 대체하지 않는다.

## 2. Event registry

### `user_input_end`

사용자가 실제 발화를 마친 acoustic 시점이다. 입력 capture timeline에서 마지막 의미 있는 speech sample의 끝으로 정의한다.

- VAD가 end-of-speech를 선언한 시각이 아니다.
- ASR final transcript, S2S turn-end event 또는 WAV read 완료 시각이 아니다.
- fixture WAV는 `speech_end_frame`을 사전 annotation하고 `capture_start_monotonic + speech_end_frame / sample_rate`로 event를 복원한다. 파일 끝의 trailing silence는 시작점을 늦추지 않는다.
- `vad_end_detected`, `transcript_finalized`, `input_read_complete`는 별도 진단 timestamp다.

따라서 실제 발화 종료 이후의 VAD, turn detection, transcription finalization과 남은 streaming inference는 responsiveness에 포함된다. 발화 종료 전에 수행된 streaming work를 종료 후 비용으로 다시 더하지 않는다.

### `agent_request_available_at_agent_ingress`

검증된 Agent 요청이 선언된 Agent ingress에서 소비 가능해진 최초 시점이다. W-01의 outbound VIA 구간 종료점이다.

이 event 전에는 다음이 모두 포함된다.

- IR 판단과 필요한 VIA LLM 호출
- Task/ExecutionLink/outbox/correlation commit
- Agent 계약 변환과 payload validation
- serialization, local queue, EXEC bridge, IPC/RPC와 outbound transport

`local_dispatch_committed`와 `ipc_write_complete`는 진단 event일 뿐 이 endpoint를 대신하지 않는다. Agent ingress 이후의 Agent 내부 acceptance queue, scheduling, reasoning, tool execution과 artifact 생성은 제외 구간이다. 실제 remote Agent를 직접 계측할 수 없는 reference campaign은 같은 payload를 받는 source-controlled ingress fixture를 사용하며 그 범위를 명시한다.

### `agent_result_available_at_source`

terminal result가 Agent source interface에서 조회 또는 stream 가능한 최초 시점이다.

- VIA가 polling으로 발견하거나 event를 수신한 시점이 아니다.
- 이 시점 이후의 polling cadence, stream delivery, network, IPC, Agent 계약 normalization, Task/run correlation과 validation은 W-01에 포함된다.
- terminal result의 Task/run/artifact identity와 revision이 oracle을 만족해야 한다.

### `agent_status_available_at_source`

비terminal progress, clarification/question 또는 partial/failure status가 Agent source interface에서 조회 또는 stream 가능한 최초 시점이다.

- VIA receive/handler start가 아니다.
- 이 시점 이후의 polling/stream 발견과 모든 inbound software 경계는 W-03에 포함된다.
- source status가 실제 Task/run과 연결되고 새로운 확인 상태를 나타내야 한다. 생성되지 않은 진행률이나 반복 filler는 유효하지 않다.

### `first_meaningful_audible_result_audio`

유효한 terminal result의 의미를 전달하는 첫 audio sample이 사용자의 출력 경로에서 실제 재생되기 시작한 시점이다.

- text 생성, audio packet 생성, sink enqueue 또는 playback API return 시점이 아니다.
- result validation, 필요한 summary/composition, speech 생성, playback queue, buffer와 device/renderer start를 포함한다.
- 제품 evidence는 audio loopback onset을 사용한다. reference harness는 검증된 payload의 instrumented render callback/onset을 사용하고 `MEASURED_REFERENCE_HARNESS`로 제한한다.

### `first_meaningful_audible_direct_response`

downstream Agent를 사용하지 않는 유효한 direct response의 의미 있는 첫 audio sample이 실제 재생되기 시작한 시점이다. filler나 generic acknowledgement는 endpoint가 아니다.

### `first_meaningful_audible_status_audio`

유효한 Agent status의 의미 있는 첫 audio sample이 실제 재생되기 시작한 시점이다. primary reference stratum은 사용자가 발화 중이지 않고 audio lane이 비어 있는 조건을 고정한다. 사용자 발화·우선순위 정책 때문에 의도적으로 보류된 시간은 별도 policy stratum으로 기록한다.

## 3. W-01~W-03 formula

### W-01 — Delegated Task Result Responsiveness

```text
t0 = user_input_end
t1 = agent_request_available_at_agent_ingress
t2 = agent_result_available_at_source
t3 = first_meaningful_audible_result_audio

W-01 sample = (t1 - t0) + (t3 - t2)
excluded_agent_interval = t2 - t1
full_user_wait_secondary = t3 - t0
```

W-01에서 명시적으로 제외하는 것은 Agent ingress 이후 result source availability 이전의 downstream Agent 내부 interval뿐이다. VIA가 소유하거나 Architecture 후보가 변경하는 outbound/inbound 경계는 제외하지 않는다.

### W-02 — VIA Direct Voice Response Responsiveness

```text
t0 = user_input_end
t1 = first_meaningful_audible_direct_response

W-02 sample = t1 - t0
```

S2S-native direct response와 VIA LLM direct response는 서로 다른 case stratum으로 보존한다. 실제 route에 참여한 component만 포함하되, route 변경으로 필요한 비용을 누락하지 않는다.

### W-03 — Agent Progress Voice Feedback Responsiveness

```text
t0 = agent_status_available_at_source
t1 = first_meaningful_audible_status_audio

W-03 sample = t1 - t0
```

Agent가 status를 만들기 전 시간은 제외한다. source availability 이후 VIA가 이를 발견하고 사용자에게 들려주기까지의 모든 비용은 포함한다.

## 4. Component participation

| 경로 | W-01 | W-02 | W-03 |
|---|---|---|---|
| acoustic input end 이후 VAD/turn detection | 포함 | 포함 | 해당 없음 |
| ASR/S2S input finalization | 실제 경로면 포함 | 실제 경로면 포함 | 해당 없음 |
| Conversation/Context materialization | 실제 경로면 포함 | 실제 경로면 포함 | status composition에 필요하면 포함 |
| IR/VIA LLM | 위임·결과 composition에 필요하면 포함 | 실제 direct route에 필요하면 포함 | status composition에 필요하면 포함 |
| Task state/correlation | 포함 | 실제 direct route에 필요할 때만 포함 | 포함 |
| Agent contract normalization | outbound와 inbound 모두 포함 | Agent가 없으므로 원칙상 해당 없음 | inbound 포함 |
| EXEC queue/IPC/RPC/network | 실제 경로의 모든 경계 포함 | 실제 경로의 모든 경계 포함 | 실제 경로의 모든 경계 포함 |
| downstream Agent 내부 실행 | 제외 | 해당 없음 | status 생성 전은 제외 |
| validation/policy/repair | 포함 | 포함 | 포함 |
| speech generation/playback/device | 포함 | 포함 | 포함 |

DP applicability는 DP 이름만으로 정하지 않고 이 실제 component path를 기준으로 고정한다. 어떤 case에서 DP 후보 A/B가 같은 경로를 실행하면 동점 또는 non-applicable로 기록하며 임의 지연을 추가하지 않는다.

## 5. 모든 W가 갖춰야 할 measurement card

W-04~W-12를 포함한 각 W는 결과와 구현 전에 다음을 한 곳에서 고정한다.

1. 사용자·외부 source·fault 등 실제 stimulus event
2. 사용자 또는 system 관점의 terminal observable
3. 포함·제외 interval과 그 근거
4. canonical case/stratum membership과 correctness oracle
5. 실제 component path와 DP별 applicability
6. raw diagnostic event와 metric endpoint의 구분
7. clock domain, source timestamp와 관측 방법
8. timeout/censoring/failure 처리
9. warm-up, scored trial, percentile와 macro aggregation
10. evidence label과 주장 가능한 범위

문서 승인이 끝난 뒤 `benchmark/architecture/`에 같은 event ID와 formula를 갖는 machine-readable contract를 만든다. 문서와 machine contract가 다르면 실행하지 않고 freeze를 실패시킨다.
