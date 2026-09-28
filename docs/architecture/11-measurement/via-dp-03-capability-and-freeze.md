# VIA-DP-03 Capability Qualification and Measurement Freeze Draft

> **Status: ACTIVE DRAFT — A/B design contract agreed, candidate profiles `PENDING`, Measurement Freeze `NOT_FROZEN`**
>
> 이 문서는 [VIA-DP-03](../12-decisions/via-dp-03-voice-evidence.md)의 합의된 A/B를 실제
> candidate로 만들기 전에 capability와 Core-ASR 모집단을 동결하기 위한 원장이다. Candidate,
> runner, scored trial과 winner는 아직 없다.

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
않는다. 우리 팀의 S2S model 개발을 기본 방향으로 하되 PoC용 기존 모델·mock/replay는 아래 evidence
제한을 따른다.

## 2. 공통 functional gate

두 후보 모두 같은 final 기능을 만족해야 한다.

| Gate | PASS 조건 |
| --- | --- |
| F-G1 Single deictic | 지칭 span의 source-time과 당시 UI evidence reference를 연결 |
| F-G2 Ordered multiple deictics | 한 utterance의 동일 표현 여러 개를 순서대로 분리 |
| F-G3 Pre-selection | 발화 전 selection/focus/pointer 상태를 관련 span에 연결 |
| F-G4 Concurrent pointing | 발화 중 pointer/selection/viewport 변화를 최신 상태 하나로 덮어쓰지 않음 |
| F-G5 Correction | “이 부분, 아니 여기”에서 철회된 span/candidate와 최종 candidate를 구별 |
| F-G6 Window/document change | 각 span을 당시 window/document/viewport reference에 연결 |
| F-G7 Ambiguity honesty | 정보가 부족하면 임의 target 대신 ambiguity/clarification으로 남김 |
| F-G8 Later grounding | DP-03은 target을 만들지 않고 semantic LLM Grounding이 소비할 evidence를 제공 |

Screenshot·Vision model과 evaluator target은 candidate input이 아니다. UI evidence reference의 실제
내용 복원은 공통 Context/Grounding 계약이며 A/B에서 동일해야 한다.

## 3. Candidate contract draft

합의한 A provisional 예시는 다음과 같다.

```json
{
  "span_id": "span-1",
  "text": "여기",
  "speech_start_ms": 1120,
  "speech_end_ms": 1340,
  "asr_revision": 3,
  "ui_evidence": {
    "screen_revision_id": 104,
    "pointer_samples": ["pointer-310", "pointer-311"],
    "selection_state_id": "selection-42",
    "focus_state_id": "focus-18",
    "window_id": "window-7",
    "document_id": "document-3",
    "viewport_id": "viewport-91"
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

A의 provenance에는 `asr_revision`과 partial/final event가, B에는 S2S final item/evidence revision이
들어간다. B는 기본안에서 `provisional`을 생성하지 않는다. `graph-A` 같은 target은 이 schema에
넣지 않는다.

## 4. A capability gate

| Gate | PASS 조건 | 현재 상태 |
| --- | --- | --- |
| A-G1 Timestamp ASR profile | 실제 Streaming ASR 제품·버전·배치와 timestamp granularity 식별 | `PENDING` |
| A-G2 Partial timestamp | 발화 중 partial text와 word/span source timestamp 제공 | `PENDING` |
| A-G3 Revision semantics | partial text·timestamp 변경과 final 안정화 의미가 문서화·관측됨 | `PENDING` |
| A-G4 Clock mapping | ASR source-time, local audio와 UI timeline clock을 변환 가능 | 설계 합의, 구현 전 |
| A-G5 Deterministic detector | frozen pattern/diff 규칙으로 새 span만 candidate화 | 설계 합의, registry 전 |
| A-G6 Candidate lifecycle | provisional update/retract/final과 late/duplicate 거부 계약 | 설계 합의, machine schema 전 |
| A-G7 No hidden aligner | 기본 A가 별도 forced aligner·Vision·evaluator oracle 없이 기능 충족 | 실행 전 |
| A-G8 Same final oracle | B와 같은 final referent/correction oracle 만족 | 실행 전 |

ASR이 timestamp를 final에만 제공하면 합의한 A의 early provisional 경로를 만족하지 않는다. 그런
profile은 A-G2에서 실패하거나 별도 candidate contract 재승인이 필요하다. Timestamp ASR도 plain
transcript보다 lookahead가 길 수 있으므로 partial arrival와 timestamp stability를 실제로 계측한다.

## 5. B capability gate

| Gate | PASS 조건 | 현재 상태 |
| --- | --- | --- |
| B-G1 Timestamp S2S profile | 개발 또는 선택할 S2S model/profile과 inference contract 식별 | `PENDING` |
| B-G2 Turn-final transcript | utterance 종료 후 authoritative final transcript 제공 | `PENDING` |
| B-G3 Native source timestamp | final word/span별 audio source start/end를 S2S native output으로 제공 | `PENDING` |
| B-G4 No separate ASR | VIA Voice Engine이 별도 ASR/aligner로 timestamp를 재구성하지 않음 | 설계 합의, 실행 전 |
| B-G5 Clock mapping | S2S timestamp를 local audio/UI timeline clock으로 변환 가능 | 설계 합의, 구현 전 |
| B-G6 Final candidate | 공통 detector와 UI evidence 연결로 final candidate 생성 | 설계 합의, machine schema 전 |
| B-G7 Same final oracle | A와 같은 final referent/correction oracle 만족 | 실행 전 |
| B-G8 Model evidence honesty | mock/oracle timestamp를 실제 모델 능력으로 주장하지 않음 | 설계 의무 |

B는 timestamp tool call을 요구하지 않는다. Tool은 모델에 없는 alignment 능력을 만들지 못한다.
Timestamp는 S2S model inference 또는 그 native event contract의 출력이어야 한다. Voice Engine 내부
code가 별도 ASR/aligner 결과를 조합하면 B가 아니라 A다.

## 6. Capability evidence ladder

| Evidence | 허용하는 결론 | 허용하지 않는 결론 |
| --- | --- | --- |
| 공식 문서·원 논문 | timestamp generation의 기술적 가능성, schema 후보 | VIA 후보 성능·지연·정확도 |
| Deterministic mock | schema, call graph, UI reference, revision/fault 처리 | model QA-09/19, winner |
| 실제 model trace replay | 관측된 event shape로 downstream 경로 재현 | live model/network E2E latency |
| 실제 candidate model 실행 | 해당 profile의 model span·정확도·지연 | PRODUCT_E2E unless physical endpoints complete |
| 실제 제품 경로 | frozen endpoint가 포함된 전체 result | 다른 profile로 일반화 |

Evaluator annotation을 B output으로 그대로 재생해 QA-19를 계산하지 않는다. Mock/replay는
`MEASURED_MOCK_E2E`, `MEASURED_REFERENCE_HARNESS` 또는 적용 가능한 제한 label을 사용하고 실제
model observation만 `MEASURED_MODEL`로 기록한다.

## 7. Pre-result Core-ASR applicability draft

아래 모집단은 `DRAFT`이며 아직 score 분모가 아니다.

### 7.1 QA-09 — `PRIMARY`

| Trial ID | Source case | Interaction class | DP-03 참여 이유 |
| --- | --- | --- | --- |
| DP03-L-01 | TC-04.1 | QA-02 direct | 단일 지칭 span과 당시 target 연결 |
| DP03-L-02 | TC-04.2 | QA-02 direct | 한 발화의 두 지칭 순서 보존 |
| DP03-L-03 | TC-04.6 | QA-02 direct | 발화 중 철회와 최종 target 반영 |
| DP03-L-04 | TC-04.5 | QA-01 delegated | 보존 집합과 수정 대상의 역할 전달 |
| DP03-L-05 | TC-11.3 | QA-02 direct | 같은 utterance의 철회 자료와 최종 자료 구별 |
| DP03-L-06 | TC-12.2 Voice variant | QA-05 control | Voice input을 올바른 기존 Task control에 연결 |

A에서는 `asr_partial_observed`, `deictic_span_detected`, `candidate_created`, `ui_evidence_pinned`,
`asr_final_observed`, `candidate_finalized`를 기록한다. B에서는 `utterance_end`,
`s2s_timestamped_final_observed`, `candidate_created`를 기록한다. 공통 Grounding 이후 실제 interaction
종료는 Event & Boundary Contract를 따른다. 내부 milestone을 QA-09 종료점으로 대체하지 않는다.

### 7.2 QA-19 — `PRIMARY`

| Field family | Draft field ID | 적용 case |
| --- | --- | --- |
| Input meaning | `input.final_text` | DP03-L-01~06 |
| Input revision | `input.active_revision` | DP03-L-03, DP03-L-05 |
| Deictic order | `referent.span_order` | DP03-L-02 |
| Referent | `referent.target_set` | DP03-L-01~05 |
| Referent role | `referent.role_by_span` | DP03-L-02, DP03-L-04 |
| Correction | `referent.retracted_target_set` | DP03-L-03, DP03-L-05 |
| Goal | `request.goal` | DP03-L-01~06 |
| Constraint | `request.preserved_target_set` | DP03-L-04 |
| Task relation | `request.task_relation` | DP03-L-06 |
| Handling | `request.handling` | DP03-L-01~06 |

Source timestamp와 candidate provenance는 위 semantic field verdict의 근거와 QA-61 trace에 남기되
동일 의미 사실을 중복 score하지 않는다.

### 7.3 QA-29 — `PRIMARY`

| Change | A에서 볼 직접 영향 | B에서 볼 직접 영향 |
| --- | --- | --- |
| S2S provider/model 교체 | Voice 대화 경로와 ASR 병렬 입력 결합 | Voice 대화와 timestamp evidence를 함께 재검증 |
| Streaming ASR 교체 | timestamp·partial·revision adapter와 profile | `NOT_APPLICABLE` unless B 구조 변경 |
| Timestamp 정보 변경 | ASR granularity·stability와 candidate lifecycle | S2S output head/token/schema와 candidate adapter |
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

Recovery deadline, repetition과 necessary dependency closure는 아직 `PENDING`이다.

## 8. Measurement Freeze에서 남은 항목

| 항목 | 현재 상태 |
| --- | --- |
| A Streaming ASR identity/version/deployment | `PENDING` |
| B S2S model/profile 또는 개발 build identity | `PENDING` |
| Candidate machine schema와 detector pattern registry | `DRAFT` |
| UI evidence reference와 retention contract | `PENDING` |
| Voice fixture와 deictic span annotation digest | `PENDING` |
| UI event fixture와 target oracle digest | `PENDING` |
| Repetition, run order, timeout | `PENDING` |
| QA-19 field registry digest | `DRAFT` |
| QA-29 element/change ledger | `DRAFT` |
| QA-39 deadline/fault repetition | `PENDING` |
| Target와 0~5 score band | `PENDING` |
| Evidence label과 physical endpoint 범위 | `PENDING` |

따라서 이 문서는 승인된 Measurement Freeze가 아니며 freeze digest도 없다.

## 9. 다음 작업

1. A용 timestamp-capable Streaming ASR 후보의 partial timestamp·revision contract를 조사한다.
2. B용 S2S time-aligned final output의 model-development contract와 근거를 정리한다.
3. 공통 candidate machine schema, deterministic pattern registry와 UI reference 수명을 초안화한다.
4. Mock/replay로 schema·lifecycle·fault 처리만 qualification한다면 evidence label을 먼저 고정한다.
5. 양쪽이 같은 final functional gate를 만족할 근거가 생긴 뒤 fixture·반복·timeout·target을 승인한다.
6. 승인 전에는 active candidate, runner, scored result 또는 winner를 만들지 않는다.
