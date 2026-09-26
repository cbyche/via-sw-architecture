# QA-01~QA-04 근거 원장 — Alibaba Qwen3-Omni Realtime S2S profile

> 기준일: 2026-09-26
> 상태: **Hosted dependency connection smoke-tested / 공식 QA benchmark NOT_RUN**
> 목적: VIA Voice Runtime이 사용할 S2S 제품·snapshot·region·transport를 고정한다.

## 1. Frozen dependency

| 항목 | 고정값 |
| --- | --- |
| Provider | Alibaba Cloud Model Studio |
| Region | Singapore |
| Model | `qwen3-omni-flash-realtime-2025-12-01` |
| Transport | WebSocket |
| Input | streaming audio/text/image/video |
| Output | streaming text/audio |
| Context | 65,536 tokens |

```text
wss://{WorkspaceId}.ap-southeast-1.maas.aliyuncs.com/api-ws/v1/realtime?model=qwen3-omni-flash-realtime-2025-12-01
Authorization: Bearer ${DASHSCOPE_API_KEY}
```

Official sources:

- [Model information](https://www.alibabacloud.com/help/en/model-studio/qwen3-omni-flash-realtime)
- [Realtime WebSocket guide](https://www.alibabacloud.com/help/en/model-studio/realtime)
- [WebSocket overview](https://www.alibabacloud.com/help/en/model-studio/realtime-websocket-overview)

Alibaba는 `qwen3-omni-flash-realtime`이 이 snapshot과 기능적으로 동등하다고 설명한다. 재현 가능한 campaign은 alias가 아니라 위 snapshot 이름을 사용한다.

## 2. Credential 준비

1. Model Studio를 활성화한다.
2. Singapore region에서 API key를 만든다.
3. 같은 workspace의 Workspace ID를 복사한다.
4. 실행 환경에 `DASHSCOPE_API_KEY`, `DASHSCOPE_WORKSPACE_ID`를 설정한다.

Key 값은 저장소·문서·fixture·trace에 기록하지 않는다. Key와 workspace region이 다르면 연결에 실패한다.

## 3. QA 측정 경계

Hosted S2S이므로 다음을 모두 별도 span으로 관찰한다.

```text
user input end / acoustic barge-in onset
→ local Voice Runtime
→ WebSocket send and remote queue
→ S2S input/turn event
→ first meaningful output audio packet
→ local playback queue
→ first meaningful audible sample or interrupted audio stop
```

Provider가 보고한 payload 도착만으로 QA-01~04를 종료하지 않는다. QA-01~03은 first meaningful audible audio onset, QA-04는 interrupted response의 last audible sample을 사용한다.

## 4. 234 ms reference의 한계

Qwen3-Omni technical report의 concurrency=1 first-audio-packet 234 ms는 S2S runtime 내부의 theoretical reference다. Network, VIA orchestration, playback과 audible onset을 포함하지 않으므로 target이나 실제 측정값으로 사용하지 않는다. 실제 Alibaba endpoint가 같은 숫자를 보장한다고 가정하지도 않는다.

Mock-only 구조 시험에서 필요할 때만 `ESTIMATED_MODEL_ONLY` scheduled span으로 사용할 수 있다. 최종 QA 비교는 선택한 hosted snapshot의 실제 request/response와 physical audio endpoint를 관측한다.

## 5. Evidence 상태

### 2026-09-26 connection smoke test

Target Mac에서 macOS `Yuna` voice로 만든 3.510초 한국어 합성 발화를 16 kHz·mono·PCM16 WAV로 변환하고, 3,200-byte chunk를 실제 시간 간격으로 snapshot endpoint에 전송했다.

| 항목 | 관측값 |
| --- | ---: |
| Response status | `completed` |
| Input audio | 112,322 bytes |
| Output audio | 579,840 bytes, 24 kHz PCM16 |
| Input end → first text delta | 365.312 ms |
| Input end → first audio packet | 729.152 ms |

Evidence label은 `MEASURED_MODEL`이다. 한 번의 provider 연결 시험이며 physical speaker의 first meaningful audible onset, 반복 p95, QA target 또는 DP A/B 결과가 아니다.

| 주장 | 상태 |
| --- | --- |
| Model Studio가 snapshot과 Singapore endpoint를 제공한다 | official provider documentation |
| API key·Workspace ID 인증 연결 | `MEASURED_MODEL` smoke evidence |
| 실제 audio input/output | `MEASURED_MODEL` smoke evidence |
| S2S model/network first-packet | single-run smoke evidence |
| QA-01~04 user-experienced p95 | `NOT_RUN` |

다음 단계는 승인된 고정 Voice fixture 반복, physical playback trace와 QA별 endpoint 검증이다.
