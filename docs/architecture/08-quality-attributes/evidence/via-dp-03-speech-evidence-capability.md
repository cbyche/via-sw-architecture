# VIA-DP-03 Speech Evidence Capability 조사

> **상태: ACTIVE SUPPORTING EVIDENCE — 문서 기반 capability 조사 / model 실행 아님**
>
> 이 문서는 DP-03 A/B PoC mock의 event shape와 behavior range가 현실적인지 확인한다.
> 아래 판정은 공식 문서와 원 논문에 근거한 capability 검토 또는 `UNRESOLVED` 상태이며,
> measurement evidence label이 아니다. `MEASURED_MODEL`, QA-09/19 결과나 winner가 아니다.

## 1. 필요한 정확한 능력

### A — Timestamp-capable Streaming ASR

A profile은 한국어 audio에 대해 발화 중 partial transcript, partial 단계의 lexical-unit source
start/end, revision/final 의미와 stable event identity를 제공해야 한다. Final에서만 timestamp를 주는
profile은 A의 early provisional candidate 경로를 충족하지 않는다.

### B — Timestamp-capable S2S

B profile은 별도 ASR·forced aligner 없이 S2S inference의 native output으로 turn-final transcript와
각 text unit의 user-audio source start/end를 제공해야 한다. Streaming 출력은 이번 B 계약에 필요하지
않다.

## 2. A 후보 조사

| 후보 | 문서에서 확인한 능력 | 한국어 | 현재 판정 |
| --- | --- | --- | --- |
| Amazon Transcribe Streaming | streaming result의 `IsPartial`, segment `StartTime`/`EndTime`, item별 `StartTime`/`EndTime`, partial-result stabilization의 `Stable` flag | `ko-KR` batch·streaming 지원 | A mock의 partial/stable/final event shape 근거 |
| Google Cloud Speech-to-Text | streaming interim/final, `stability`, `resultEndOffset`; word time offsets를 streaming recognition에서 지원 | `ko-KR` 지원 | provider-independent A mock semantics의 교차 근거 |
| Deepgram Streaming | live response에 word start/end와 interim/final 제공 | 현재 대표 multilingual profile의 한국어 지원이 제한적 | interim revision과 timestamp behavior의 보조 근거 |

공식 근거:

- Amazon Transcribe의 [streaming partial-result 문서](https://docs.aws.amazon.com/transcribe/latest/dg/streaming-partial-results.html),
  [`Result` API](https://docs.aws.amazon.com/transcribe/latest/APIReference/API_streaming_Result.html),
  [`Item` API](https://docs.aws.amazon.com/transcribe/latest/APIReference/API_streaming_Item.html),
  [지원 언어 표](https://docs.aws.amazon.com/transcribe/latest/dg/supported-languages.html)
- Google Cloud의 [`StreamingRecognitionResult`](https://docs.cloud.google.com/speech-to-text/docs/reference/rest/v2/StreamingRecognitionResult),
  [word time offset](https://docs.cloud.google.com/speech-to-text/docs/v1/async-time-offsets),
  [지원 언어 표](https://docs.cloud.google.com/speech-to-text/docs/speech-to-text-supported-languages)
- Deepgram의 [live streaming response](https://developers.deepgram.com/docs/live-streaming-audio),
  [interim result](https://developers.deepgram.com/docs/interim-results),
  [model/language 표](https://developers.deepgram.com/docs/models-languages-overview/)

### A에 대한 사전 결론

Amazon Transcribe Streaming `ko-KR`의 contract를 A nominal mock event shape의 첫 근거로 사용하고,
Google/Deepgram contract로 streaming revision의 일반성을 교차 확인한다. 실제 cloud 호출은 이번 PoC
gate가 아니다. Mock profile에는 다음 behavior를 명시적으로 넣는다.

- partial event에 지칭 표현을 덮는 item start/end가 실제로 존재하는가
- 같은 result/segment가 어떻게 수정되고 final이 되는가
- `Stable` item과 불안정 item의 timestamp가 어떤 식으로 변하는가
- reconnect·duplicate·late event identity를 adapter가 구별할 수 있는가
- provider media offset을 local capture frame과 오차 범위 내에서 연결할 수 있는가

## 3. B 가능성 조사

| 근거 | 확인한 내용 | DP-03 B에 대한 의미 |
| --- | --- | --- |
| Qwen3-Omni | end-to-end multilingual S2S와 realtime interaction, Korean speech input/output을 지원한다고 문서화 | 팀 B build와 같은 S2S base capability의 근거; 공개 native timestamp contract는 없음 |
| Qwen3-ASR | timestamp mode는 별도 Qwen3 Forced Aligner를 호출하고 streaming ASR은 timestamp를 지원하지 않는다고 구현에 명시 | B mock에 별도 aligner를 숨기면 안 된다는 경계 근거 |
| Whisper | text token과 timestamp token을 함께 예측하는 학습·decode 방식을 제시 | integrated timestamp token이 가능한 근거 |
| Moshi | time-aligned text token을 audio token보다 먼저 예측하는 streaming full-duplex spoken LM 구조를 제시 | S2S 안에서 aligned text를 native output으로 만드는 직접적인 feasibility 근거 |

공식·1차 근거:

- Qwen3-Omni [공식 저장소 README](https://github.com/QwenLM/Qwen3-Omni/blob/main/README.md)
- Qwen3-ASR [공식 inference 구현](https://github.com/QwenLM/Qwen3-ASR/blob/main/qwen_asr/inference/qwen3_asr.py)
- OpenAI [Whisper 논문](https://cdn.openai.com/papers/whisper.pdf)
- Kyutai [Moshi 논문](https://arxiv.org/abs/2410.00037)

### B에 대한 사전 결론

B는 **기술적으로 개발 가능한 대안**이며 팀 개발 S2S가 최종 후보다. 기성 S2S profile의 존재는
이번 PoC 조건이 아니다. Qwen prompt나 tool call로 timestamp JSON을 요청하거나
Qwen3-ASR+Forced Aligner를 Voice Engine에서 조합하는 대신, B mock은 아래 팀 build acceptance
contract를 S2S native output으로 직접 재현한다.

우리 팀이 B용 S2S를 개발한다면 build acceptance contract는 최소한 다음을 요구해야 한다.

1. 발화 종료 뒤 authoritative final transcript 한 번
2. NFC transcript code-point range와 user-audio source start/end를 잇는 alignment unit
3. audio source clock, model build/profile과 inference event identity
4. `is_final=true`와 final 관측 monotonic time
5. “이 부분, 아니 여기”의 두 표현과 correction cue를 transcript에서 보존
6. timestamp 생성을 위해 별도 ASR·forced aligner·tool call을 사용하지 않았다는 dependency manifest

## 4. PoC 적용

이 근거를 바탕으로 A의 partial/revision/final trace와 B의 turn-final aligned-text trace를
deterministic하게 만든다. Correct·slow·missing·timestamp drift·duplicate·late·disconnect behavior와
schedule은 Measurement Freeze에 기록한다.

이 mock과 실제 candidate Architecture 경로를 함께 실행한 결과는 `MEASURED_MOCK_E2E` QA-09/19/29/39
수치와 A/B winner 근거로 사용한다. Evaluator-only final field·referent oracle은 mock output과 분리한다.
실제 Model 실행은 같은 spine/evaluator를 사용하는 후속 revalidation이다.
