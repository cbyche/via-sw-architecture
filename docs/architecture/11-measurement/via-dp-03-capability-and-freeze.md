# VIA-DP-03 Capability Qualification and Measurement Freeze Draft

> **Status: ACTIVE DRAFT — A/B PoC mock contract selected / behavior profile·spine patch freeze pending**
>
> 이 문서는 [VIA-DP-03](../12-decisions/via-dp-03-voice-evidence.md)의 합의된 A/B를 실제
> candidate로 만들기 전에 capability와 Core-ASR 모집단을 동결하기 위한 원장이다. Candidate,
> runner, scored trial과 winner는 아직 없다.

공통 candidate 의미는 [Candidate Contract](./via-dp-03-candidate-contract.md), 동일 입력·hidden
oracle·fault/change pack은 [Fixture, Oracle, Fault & Change Contract](./via-dp-03-fixture-and-oracle.md)에
분리해 상세화한다.

## 1. 동결할 A/B identity

### A — VIA 중심 근거 융합

```text
Timestamp 없는 S2S
+ Timestamp-capable Streaming ASR
+ VIA가 지칭 span과 당시 UI evidence를 연결
```

A는 발화 중 partial timestamp를 받아 `provisional` candidate를 만들고 ASR revision/final에 따라
갱신·철회·확정한다. 별도 forced aligner와 Vision model은 기본 A에 포함하지 않는다.

### B — 모델 중심 근거 제공

```text
Timestamp-capable S2S
+ VIA가 지칭 span과 당시 UI evidence를 연결
```

B에는 별도 ASR이 없다. 이번 후보는 S2S가 발화 종료 뒤 final transcript와 word/span source
timestamp를 한 번에 제공하는 `turn-final` 계약이다. Streaming timestamp는 기본 B에 포함하지
않는다. 우리 팀의 S2S model 개발은 최종 제품 방향이며, 이번 PoC는 그 native output contract를
deterministic mock으로 실행한다.

## 2. 공통 functional gate

두 후보 모두 같은 final 기능을 만족해야 한다.

| Gate | PASS 조건 |
| --- | --- |
| F-G1 Single deictic | 지칭 span의 source-time과 당시 UI evidence reference를 연결 |
| F-G2 Ordered multiple deictics | 한 utterance의 동일 표현 여러 개를 순서대로 분리 |
| F-G3 Pre-selection | 발화 전 selection/focus/pointer 상태를 관련 span에 연결 |
| F-G4 Concurrent pointing | 발화 중 pointer/selection/viewport 변화를 최신 상태 하나로 덮어쓰지 않음 |
| F-G5 Correction | “이 부분, 아니 여기”의 두 source-time span을 모두 보존하고 이후 Grounding이 앞 referent 철회를 판단할 수 있게 함 |
| F-G6 Window/document change | 각 span을 당시 window/document/viewport reference에 연결 |
| F-G7 Ambiguity honesty | 정보가 부족하면 임의 target 대신 ambiguity/clarification으로 남김 |
| F-G8 Later grounding | DP-03은 target을 만들지 않고 semantic LLM Grounding이 소비할 evidence를 제공 |

Screenshot·Vision model과 evaluator target은 candidate input이 아니다. UI evidence reference의 실제
내용 복원은 공통 Context/Grounding 계약이며 A/B에서 동일해야 한다.

## 3. Candidate contract draft

전체 event, clock, UI anchor, retention, detector와 revision 규칙은
[DP-03 Candidate Contract](./via-dp-03-candidate-contract.md)에 정의한다. Machine schema와 한국어
pattern registry도 그 문서에서 연결하며 모두 freeze 전 `DRAFT`다.

합의한 A provisional 예시는 다음과 같다. 전체 required field와 validation rule은 연결된 machine
schema가 기준이다.

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

Machine schema freeze 때 A/B 공통 envelope와 source-specific provenance를 분리한다.

| 공통 field | 동결할 의미 |
| --- | --- |
| `utterance_id`, `span_id` | Conversation/Request와 별개인 음성 evidence identity |
| `text` | 해당 revision/final이 인식한 지칭 표현 |
| `speech_start_ms`, `speech_end_ms` | local audio source clock으로 변환 가능한 구간 |
| `ui_evidence` | source-time을 포함하는 UI evidence reference 집합 |
| `status` | `provisional`, `final`, `retracted` |
| `provenance` | candidate, model/profile, source event/item과 revision identity |

A의 provenance에는 `source_revision`과 partial/final event identity가, B에는 S2S final
item/evidence revision이 들어간다. B는 기본안에서 `provisional`을 생성하지 않는다. `graph-A` 같은
target은 이 schema에 넣지 않는다.

## 4. A mock contract gate

| Gate | PASS 조건 | 현재 상태 |
| --- | --- | --- |
| A-G1 Timestamp ASR mock profile | mock revision·profile ID와 timestamp granularity 식별 | contract `DRAFT`; timing/error schedule 전 |
| A-G2 Partial timestamp | 발화 중 partial text와 word/span source timestamp 제공 | event schema `DRAFT`; trace fixture 전 |
| A-G3 Revision semantics | partial text·timestamp 변경과 final 안정화 sequence를 재현 | revision sequence `DRAFT`; profile digest 전 |
| A-G4 Clock mapping | ASR source-time, local audio와 UI timeline clock을 변환 가능 | 계약 `DRAFT`; 구현 전 |
| A-G5 Deterministic detector | frozen pattern/diff 규칙으로 새 span만 candidate화 | registry `DRAFT`; fixture review 전 |
| A-G6 Candidate lifecycle | provisional update/retract/final과 late/duplicate 거부 계약 | schema·algorithm `DRAFT`; validator 구현 전 |
| A-G7 No hidden aligner | 기본 A가 별도 forced aligner·Vision·evaluator oracle 없이 기능 충족 | mock dependency graph review 전 |
| A-G8 Same final oracle | B와 같은 final referent/correction oracle 만족 | 실행 전 |

Mock이 timestamp를 final에만 제공하면 합의한 A의 early provisional 경로를 만족하지 않는다. A의
partial arrival·timestamp stability·revision schedule은 공개 ASR contract를 근거로 만들고 결과 전에
동결한다. 특정 상용 ASR 실행은 gate가 아니다. Event-shape 근거는
[DP-03 Speech Evidence Capability 조사](../08-quality-attributes/evidence/via-dp-03-speech-evidence-capability.md)를 따른다.

## 5. B mock contract gate

| Gate | PASS 조건 | 현재 상태 |
| --- | --- | --- |
| B-G1 Timestamp S2S mock profile | 팀 개발 output contract를 재현할 mock revision·profile 식별 | contract `DRAFT`; timing/error schedule 전 |
| B-G2 Turn-final transcript | utterance 종료 후 authoritative final transcript 제공 | event schema `DRAFT`; trace fixture 전 |
| B-G3 Native source timestamp | final word/span별 audio source start/end를 S2S native output으로 제공 | 기술적 가능성 근거 확보; mock profile digest 전 |
| B-G4 No separate ASR | VIA Voice Engine이 별도 ASR/aligner로 timestamp를 재구성하지 않음 | 설계 합의, 실행 전 |
| B-G5 Clock mapping | S2S timestamp를 local audio/UI timeline clock으로 변환 가능 | 설계 합의, 구현 전 |
| B-G6 Final candidate | 공통 detector와 UI evidence 연결로 final candidate 생성 | schema·algorithm `DRAFT`; validator 구현 전 |
| B-G7 Same final oracle | A와 같은 final referent/correction oracle 만족 | 실행 전 |
| B-G8 Oracle separation | mock event와 evaluator-only expected target/field를 분리 | input-separation receipt 전 |

B는 timestamp tool call을 요구하지 않는다. Tool은 모델에 없는 alignment 능력을 만들지 못한다.
Timestamp는 S2S model inference 또는 그 native event contract의 출력이어야 한다. Voice Engine 내부
code가 별도 ASR/aligner 결과를 조합하면 B가 아니라 A다.

완성된 기성 S2S profile은 gate가 아니다. Qwen3-Omni·Whisper·Moshi와 팀 개발 contract는 S2S native
aligned-text event가 기술적으로 가능한지와 어떤 event shape를 mock할지 정하는 근거다. B mock은
별도 ASR/forced aligner를 숨기지 않고 S2S source event로 final alignment를 직접 낸다.

## 6. PoC evidence contract

| Evidence | 이번 PoC에서의 역할 |
| --- | --- |
| 공식 문서·원 논문·팀 output contract | A/B mock event shape·timing/error range의 근거 |
| Deterministic dependency mock + candidate path | QA-09/19/29/39 공식 `MEASURED_MOCK_E2E` 수치·별점·winner 근거 |
| 실제 model revalidation | 같은 spine/evaluator로 향후 dependency assumption 재확인; PoC 선행조건 아님 |

Evaluator annotation을 B output으로 그대로 재생하지 않는다. Mock은 transcript/alignment source
event만 제공하고 candidate가 만든 final semantic field를 hidden oracle과 비교한다. A/B mock profile,
schedule, seed와 근거를 freeze하면 그 실행 결과를 QA-09/19/29/39와 Architecture Decision에 직접 쓴다.

## 7. Pre-result Core-ASR applicability draft

아래 모집단은 `DRAFT`이며 아직 score 분모가 아니다.

### 7.1 QA-09 — `PRIMARY`

| Trial ID | Source case | Interaction class | DP-03 참여 이유 |
| --- | --- | --- | --- |
| DP03-L-01 | SP-01 / TC-04.1 | QA-02 direct | 단일 지칭 span과 당시 target 연결 |
| DP03-L-02 | SP-01 / TC-04.2 | QA-02 direct | 한 발화의 두 지칭 순서 보존 |
| DP03-L-03 | SP-04 / TC-04.6 | QA-02 direct | 발화 중 철회와 최종 target 반영 |
| DP03-L-04 | SP-02 / TC-04.5 | QA-01 delegated | 보존 집합과 수정 대상의 역할 전달 |
| DP03-L-05 | SP-01 / TC-03.5 | QA-02 direct | 발화 전 pointer state를 지칭 span에 연결 |
| DP03-L-06 | SP-02 / TC-04.4 | QA-01 delegated | 발화 중 drag/selection transition과 범위 보존 |
| DP03-L-07 | SP-01 / TC-04.7 | QA-02 direct | 동일 좌표라도 window/document가 다른 두 지칭 분리 |

A에서는 `asr_partial_observed`, `deictic_span_detected`, `candidate_created`, `ui_evidence_pinned`,
`asr_final_observed`, `candidate_finalized`를 기록한다. B에서는 `utterance_end`,
`s2s_timestamped_final_observed`, `candidate_created`를 기록한다. 공통 Grounding 이후 실제 interaction
종료는 Event & Boundary Contract를 따른다. 내부 milestone을 QA-09 종료점으로 대체하지 않는다.

### 7.2 QA-19 — `PRIMARY`

| Field family | Draft field ID | 적용 case |
| --- | --- | --- |
| Input meaning | `input.final_text` | DP03-L-01~07 |
| Deictic order | `referent.span_order` | DP03-L-02 |
| Referent | `referent.target_set` | DP03-L-01~07 |
| Referent role | `referent.role_by_span` | DP03-L-02, DP03-L-04, DP03-L-07 |
| Correction | `referent.retracted_target_set` | DP03-L-03 |
| Goal | `request.goal` | DP03-L-01~07 |
| Constraint | `request.preserved_target_set` | DP03-L-04 |
| Handling | `request.handling` | DP03-L-01~07 |

Source timestamp와 candidate provenance는 위 semantic field verdict의 근거와 QA-61 trace에 남기되
동일 의미 사실을 중복 score하지 않는다.

Recognizer가 실제로 text를 수정·삭제한 lifecycle은 QA-61 provenance 진단이며 semantic field로
중복 score하지 않는다. 사용자가 “이 부분, 아니 여기”라고 말한 경우 두 candidate는 모두 final로
남고, `referent.retracted_target_set`은 correction cue와 순서를 해석한 이후 Grounding의 semantic
oracle이다.

### 7.3 QA-29 — `PRIMARY`

| Change | A에서 볼 직접 영향 | B에서 볼 직접 영향 |
| --- | --- | --- |
| S2S provider/model 교체 | Voice 대화 경로와 ASR 병렬 입력 결합 | Voice 대화와 timestamp evidence를 함께 재검증 |
| Streaming ASR 교체 | timestamp·partial·revision adapter와 profile | `NOT_APPLICABLE` unless B 구조 변경 |
| Timestamp 정보 변경 | S2S timestamp를 소비하지 않으므로 ASR evidence 계약 유지; 기능 회귀만 수행 | S2S output head/token/schema, candidate adapter와 evidence state |
| UI evidence 계약 변경 | 공통 clock/reference mapping | 공통 clock/reference mapping |
| Trace correlation 추가 | S2S+ASR+candidate identity | S2S final+candidate identity |

Closed Change Catalog의 M-01·M-07·C-03·E-01~03 mapping과 candidate별 full Architecture Element
ledger를 freeze 전에 갱신한다. 별도 ASR 추가가 허용된 새 baseline에 맞춰 change ID가 필요한지도
결과 전에 결정한다.

### 7.4 QA-39 — `REGRESSION_ONLY`

| Fault | A 적용 | B 적용 | PASS 핵심 |
| --- | --- | --- | --- |
| Evidence source disconnect | Streaming ASR 중단 | S2S final evidence 누락 | stale candidate를 final로 확정하지 않고 clarification/recovery |
| Late/stale revision | partial/final 역전·duplicate | stale/duplicate final item | accepted candidate와 Request를 되살리거나 중복 dispatch하지 않음 |
| Voice reconnect | S2S와 ASR epoch 재연결 | S2S epoch 재연결 | Conversation과 새 provider epoch를 혼동하지 않음 |

공통 기본값은 fault item당 10회와 5,000 ms recovery deadline이다. DP-03 necessary dependency
closure와 fault injection event digest는 아직 `PENDING`이다.

## 8. Measurement Freeze에서 남은 항목

| 항목 | 현재 상태 |
| --- | --- |
| A Streaming ASR mock revision·timing/error schedule | machine profile `DRAFT`; payload fixture·freeze digest `PENDING` |
| B S2S mock revision·timing/error schedule | machine profile `DRAFT`; payload fixture·freeze digest `PENDING` |
| Candidate machine schema와 detector pattern registry | `DRAFT` |
| UI evidence reference와 retention contract | anchor/lifetime rule `DRAFT`; 제품 budget `PENDING` |
| Shared Spine membership과 DP-03 event patch digest | `DRAFT` |
| Voice/UI fixture와 hidden oracle digest | L-01~07 machine payload `DRAFT`; WAV·digest·input-separation receipt `PENDING` |
| Repetition, run order, timeout | 공통 기본값 채택; campaign digest `PENDING` |
| QA-19 field registry digest | `DRAFT` |
| QA-29 element/change ledger | machine ledger `DRAFT`; change regression evidence·digest `PENDING` |
| QA-39 deadline/fault repetition | machine fault/closure registry `DRAFT`; injection trace·digest `PENDING` |
| Target와 0~5 score band | 공통 Scoring Contract proposal 사용; DP-03 freeze 전 |
| Evidence label과 endpoint 범위 | `MEASURED_MOCK_E2E`; physical Voice endpoint qualification `PENDING` |

따라서 이 문서는 승인된 Measurement Freeze가 아니며 freeze digest도 없다.

Machine-readable 초안은 [DP-03 Spine Patch Registry](./contracts/via-dp-03-spine-patch.json),
[A Streaming ASR Mock Profile](./contracts/via-dp-03-a-streaming-asr-mock-profile.json)과
[B S2S Mock Profile](./contracts/via-dp-03-b-s2s-mock-profile.json),
[Candidate-visible Fixtures](./contracts/via-dp-03-fixture-inputs.json)와
[Evaluator-only Oracles](./contracts/via-dp-03-hidden-oracles.json),
[Change Ledger](./contracts/via-dp-03-change-ledger.json)와
[Fault Registry](./contracts/via-dp-03-fault-registry.json)에 있다. Primary timing은 A/B의
turn-final 도착을 모두 utterance end + 300 ms로 두어 특정 모델 latency를 한쪽에 미리 유리하게
주지 않는다. A만 지칭 audio interval 종료 + 160 ms의 partial을 받으므로, 그 provisional evidence를
실제 candidate path가 활용할 수 있는지가 구조 차이로 남는다. 180/600 ms final profile은 양쪽에 같은
fast/slow sensitivity 조건으로 적용한다.

## 9. 다음 작업

1. DP-03 spine patch, A/B mock profile, L-01~07 fixture/oracle, QA-29 ledger와 QA-39 registry 초안을 review한다.
2. Canonical WAV를 생성하고 Voice/UI fixture·hidden oracle·input-separation digest를 만든다.
3. Fault event payload와 change regression patch를 작성해 현재 static validator에 연결한다.
4. 전체 contract와 fixture digest를 Measurement Freeze manifest로 만든다.
5. freeze digest 전에는 active candidate, runner, scored result 또는 winner를 만들지 않는다.
