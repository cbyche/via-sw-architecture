# VIA-DP-03 Candidate Contract

> **Status: ACTIVE DRAFT — semantic contract defined / machine freeze and product retention approval pending**
>
> 이 문서는 [VIA-DP-03](../12-decisions/via-dp-03-voice-evidence.md)의 A/B가 같은 의미의
> candidate를 만들도록 입력 event, clock, UI evidence, 지칭 pattern과 lifecycle을 정의한다.
> Candidate implementation이나 PoC 결과가 아니며, 연결된 JSON 파일도 freeze 전 초안이다.

## 1. Candidate가 의미하는 것

Candidate는 **전사에서 발견한 하나의 지칭 span과, 그 span의 audio source-time에 조회할 수 있는
UI evidence reference를 묶은 것**이다. Candidate는 `graph-A` 같은 최종 referent도, 사용자의 goal도,
요청 처리 경로도 아니다. 그 판단은 이후 Grounding과 semantic LLM이 한다.

다음 두 종류의 철회를 구분한다.

| 종류 | DP-03 candidate 처리 | 최종 의미 처리 |
| --- | --- | --- |
| ASR revision 철회 | partial에 있던 text span이 후속 revision에서 사라지면 candidate를 `retracted`로 변경 | 철회된 candidate는 Grounding 입력에서 제외하되 provenance는 보존 |
| 사용자 의미 정정 | “이 부분, 아니 여기”의 두 지칭 span을 모두 final candidate로 보존 | semantic Grounding이 correction cue와 순서를 해석해 앞 referent를 철회 |

따라서 correction cue가 있다는 이유만으로 deterministic detector가 candidate를 지우지 않는다.
`retracted`는 **speech recognizer revision lifecycle** 용어이지 사용자 의도 판정이 아니다.

## 2. A/B 공통 입력 event

Provider별 wire format은 adapter가 다음 공통 event로 정규화한다.

| Field | 필수 의미 |
| --- | --- |
| `source_epoch` | 연결 재시작·provider session 교체를 구별하는 값 |
| `source_event_id` | provider event identity. 없으면 VIA가 수신 시 생성 |
| `source_event_seq` | 같은 epoch에서 VIA가 부여하는 단조 증가 수신 순서 |
| `source_segment_id` | provider가 같은 audio 구간이라고 표시한 identity. 없으면 adapter가 구간으로 구성 |
| `text` | 해당 event가 주장하는 transcript text |
| `alignment_units[]` | text range와 audio source-time start/end를 연결하는 단위 |
| `is_final` | 해당 audio 구간에 더 이상 transcript revision이 오지 않는다는 주장 |
| `observed_at_monotonic_ms` | VIA가 event를 관측한 시각. speech source-time을 대신하지 않음 |

`source_event_seq`는 provider의 의미 revision 번호가 아니다. Provider가 명시적 revision을 주지 않아도
late·duplicate·epoch 혼동을 검출하기 위한 VIA 관측 순서다. Candidate provenance에는 provider 원본
identity와 이 순서를 모두 남긴다.

### 2.1 A 입력

A adapter는 Streaming ASR의 partial과 final을 모두 위 event로 변환한다. `alignment_units[]`에는
partial 단계의 word 또는 lexical-unit source timestamp가 있어야 한다. Provider의 stable flag는
보존하지만 stable하지 않다는 이유로 candidate 생성을 금지하지 않는다.

### 2.2 B 입력

B adapter는 S2S의 turn-final time-aligned text event 한 번을 변환한다. 기본 B에서는
`is_final=true`만 허용하며 별도 ASR·forced aligner·post-hoc alignment service를 호출하지 않는다.
S2S가 내보내는 unit은 엄격한 언어학적 “단어”일 필요는 없지만, final transcript의 text range와
audio source interval을 결정적으로 연결해야 한다.

## 3. Machine contract

초안 schema는
[`contracts/via-dp-03-candidate.schema.json`](./contracts/via-dp-03-candidate.schema.json)에 둔다.
공통 envelope의 핵심은 다음과 같다.

```json
{
  "schema_version": "via.voice-evidence-candidate.v0.1",
  "candidate_id": "candidate-1",
  "utterance_id": "utterance-1",
  "span_id": "span-1",
  "candidate_revision": 3,
  "text": "여기",
  "text_start_char": 8,
  "text_end_char": 10,
  "pattern_id": "ko.locative.here",
  "speech_clock_id": "audio-input-epoch-7",
  "clock_mapping_id": "clock-map-7",
  "speech_start_ms": 1120,
  "speech_end_ms": 1340,
  "ui_evidence": {
    "timeline_slice_id": "ui-slice-88",
    "ui_clock_id": "ui-monotonic-1",
    "screen_revision_id": "screen-104",
    "pointer_samples": ["pointer-310", "pointer-311"],
    "selection_state_id": "selection-42",
    "focus_state_id": "focus-18",
    "window_id": "window-7",
    "document_id": "document-3",
    "viewport_id": "viewport-91",
    "state_transition_ids": []
  },
  "provenance": {
    "alternative": "A",
    "provider": "provider-name",
    "profile": "profile-name",
    "source_epoch": "asr-epoch-7",
    "source_event_id": "provider-event-21",
    "source_event_seq": 21,
    "source_segment_id": "provider-segment-4",
    "source_revision": "partial-3",
    "source_final": false,
    "observed_at_monotonic_ms": 1842
  },
  "status": "provisional"
}
```

`text_start_char`와 `text_end_char`는 provider token index가 아니라 NFC-normalized transcript의
Unicode code-point half-open range다. Provider tokenization이 바뀌어도 같은 detector와 oracle을
사용하기 위한 경계다.

## 4. Clock mapping

Audio capture clock을 speech source-time의 기준으로 사용한다.

```text
provider audio offset
→ provider에 실제 전송된 audio frame range
→ local capture audio frame range
→ capture monotonic clock
```

필수 receipt는 다음과 같다.

- candidate가 가리키는 `clock_mapping_id`, `speech_clock_id`, UI timeline의 `ui_clock_id`와
  `source_epoch`
- capture sample rate와 stream 시작 frame/monotonic time
- provider에 전송한 chunk별 local frame start/end
- resample·drop·duplicate·reconnect가 있으면 그 mapping과 error bound

단순히 `provider_offset + websocket_open_time`으로 계산하지 않는다. Network queue는 audio media-time이
아니며, reconnect 뒤 같은 offset 0이 다시 등장할 수 있다. Mapping gap 또는 서로 다른 epoch를 하나로
연결할 수 없는 경우 candidate는 final evidence로 사용할 수 없다.

## 5. UI evidence anchor policy

UI timeline은 transcript 도착 전에 이미 기록된다. Candidate 생성 시 다음 reference를 pin한다.

1. `timeline_slice_id`: fixture 또는 제품 retention policy가 허용한 과거 UI event slice
2. `screen_revision_id`, `selection_state_id`, `focus_state_id`, `window_id`, `document_id`,
   `viewport_id`: speech interval midpoint에 유효한 persistent state
3. `pointer_samples`: span start 직전의 마지막 sample, span 내부 sample 전부, span end 직후의 첫 sample
4. `state_transition_ids`: speech interval 안에서 발생한 window/document/viewport/selection/focus 변화

이 reference 집합은 referent를 미리 선택하지 않는다. Pointer sample이 없거나 UI 행동이 없다는 이유로
candidate를 버리지 않는다. Grounding은 timeline slice와 persistent state를 함께 사용하고 근거가
부족하면 clarification한다.

Screenshot image, OCR/Vision 결과와 evaluator target은 이 계약에 넣지 않는다. `screen_revision_id`는
나중에 Context가 해당 revision을 복원할 수 있는 identity일 뿐 screenshot 생성 요구가 아니다.

## 6. Retention contract

Product retention의 숫자는 아직 승인하지 않았지만 최소 수명 규칙은 다음과 같이 정의한다.

```text
required UI history
>= configured pre-turn history
 + maximum accepted utterance duration
 + evidence finalization timeout
 + Grounding handoff timeout
```

- A는 provisional candidate가 생기면 관련 slice를 즉시 pin할 수 있다.
- B는 turn-final event가 도착할 때까지 candidate가 없으므로 UI ring buffer가 위 전체 구간을 보존해야 한다.
- `final` candidate가 Grounding에 durable handoff되거나 request가 취소·만료될 때까지 pin을 해제하지 않는다.
- `retracted` candidate의 UI payload는 release할 수 있지만 revision provenance와 digest는 QA-61 evidence로 남긴다.

현재 canonical fixture의 pre-turn event는 최대 약 600 ms 전이므로 measurement fixture에는 1,000 ms의
pre-turn timeline을 제공한다. 이 1,000 ms는 제품 correctness tolerance나 보편적인 gesture-speech
window가 아니다. 제품 retention budget은 별도 사용자 승인이 필요하다.

## 7. Deterministic deictic detector

초안 registry는
[`contracts/via-dp-03-deictic-patterns.json`](./contracts/via-dp-03-deictic-patterns.json)에 둔다.
Detector는 LLM, pointer movement 또는 evaluator target을 사용하지 않는다.

처리 순서는 다음과 같다.

1. Unicode NFC normalization을 적용하되 원문 code-point offset mapping을 보존한다.
2. whitespace·문장부호로 eojeol range를 만들고 frozen 조사 suffix를 최대 한 번 제거한다.
3. standalone form과 `이/저 + frozen UI noun` form을 exact match한다.
4. bare `이`, `저`, anaphoric `그/거기`와 일반 단어 내부 substring은 match하지 않는다.
5. match된 text range를 덮는 `alignment_units[]`의 최소 start와 최대 end를 speech interval로 사용한다.
6. 완성되지 않은 `이/저` partial은 후속 unit을 기다린다.

초기 registry는 `여기`, `저기`, `이거`, `저거`, `이것`, `저것`, `이쪽`, `저쪽`, `이곳`,
`저곳`, `이것들`, `저것들`과 `이/저 + 부분·영역·표·그래프·차트·문단·문장·셀·행·열·범위·파일·문서·창·화면·항목·버튼·링크·이미지·사진·둘`을 포함한다.

Registry 밖 표현은 detector가 임의 확장하지 않는다. Fixture에서 필수 표현이 누락되면 결과를 본 뒤
추가하지 않고 Measurement Freeze version을 올려 다시 시작한다.

## 8. Candidate identity와 revision algorithm

A는 새 transcript event마다 현재 provider segment의 지칭 match를 다시 계산한다.

1. 이전 candidate와 새 match의 provider alignment-unit identity가 같으면 같은 `span_id`를 유지한다.
2. Unit identity가 없으면 같은 `source_segment_id` 안에서 audio interval이 겹치는 match 중 overlap이
   가장 큰 하나를 같은 span으로 본다. 동률이면 text order가 앞선 match를 사용한다.
3. 같은 span의 text·range·timestamp·UI reference가 바뀌면 `candidate_revision`을 증가시킨다.
4. 이전 provisional match가 새 revision에 없으면 `retracted` event를 남긴다.
5. Final transcript에 남은 match만 `final`로 확정한다.
6. 이미 final/retracted된 revision보다 오래된 `source_event_seq`와 이전 `source_epoch` event는 적용하지 않는다.

B는 turn-final event에서 match마다 `candidate_revision=1`, `status=final` candidate를 만든다.

동일한 발화에 같은 표면형이 여러 번 나오면 text order와 서로 다른 audio interval로 분리한다. 최신
pointer snapshot 하나를 모든 match에 복사하지 않는다.

## 9. Required trace events

| Event | A | B | 의미 |
| --- | --- | --- | --- |
| `speech_source_event_observed` | partial/final마다 | final 한 번 | 원본 provider event와 수신 시각 |
| `deictic_span_detected` | 새/변경 match | final match | pattern과 text/audio range |
| `candidate_created` | provisional | final | 최초 candidate |
| `candidate_revised` | 가능 | 없음 | 같은 span의 새 revision |
| `candidate_retracted` | 가능 | 없음 | ASR revision에서 span 제거 |
| `candidate_finalized` | 가능 | 생성과 동일 가능 | final transcript에 남은 candidate |
| `ui_evidence_pinned` | candidate 생성/변경 | candidate 생성 | pin한 reference와 lifetime |
| `candidate_handoff_completed` | 공통 | 공통 | Grounding consumer가 accepted한 시각 |

각 event에는 `conversation_id`, `utterance_id`, `candidate_id`, `source_epoch`, `source_event_seq`,
monotonic observed time과 causal parent를 남긴다.

## 10. Measurement Freeze에 남은 값

이번 PoC에서 상용 A provider나 완성된 B build는 필요하지 않다. 다음 contract 값만 결과 전에
동결한다.

1. A/B mock revision, nominal·slow·error/fault schedule과 deterministic seed
2. Shared Spine의 DP-03 event patch, fixture/oracle와 input-separation digest
3. PoC UI timeline retention과 pin/release timeout
4. field/change/fault registry와 common run-default adoption receipt

QA-09/19/29/39 target·score band proposal과 evidence label은 공통 Scoring/Mock Contract를 사용하고
DP-03 Measurement Freeze에서 함께 승인한다. 실제 provider·팀 S2S build 연결은 같은 contract를
재사용하는 후속 revalidation이다.
