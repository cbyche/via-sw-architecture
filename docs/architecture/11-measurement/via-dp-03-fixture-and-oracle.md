# VIA-DP-03 Fixture, Oracle, Fault & Change Contract

> **Status: ACTIVE DRAFT — Shared Spine mapping and case semantics defined / mock schedule·fixture digest pending**
>
> 이 문서는 DP-03 A/B가 같은 사용자 사건을 처리하도록 pre-result fixture와 evaluator 경계를
> 정의한다. Audio/UI fixture나 runner를 구현한 문서가 아니며, 모든 결과는 아직 `NOT_RUN`이다.
> 공통 사용자 여정과 실행 기본값은
> [Core DP PoC Mock & Shared Spine Contract](./core-poc-mock-and-spine-contract.md)를 따른다.

## 1. 비교 단위와 입력 분리

각 paired trial은 동일한 다음 입력을 사용한다.

- 사용자가 실제로 말한 하나의 audio file 또는 capture stream과 그 digest
- local capture frame과 monotonic clock mapping
- 발화 전·중·후의 동일 UI source-event timeline과 digest
- 동일 Conversation/Request/Task 초기 상태
- 동일 deterministic deictic pattern registry
- 동일 candidate consumer, Grounding, semantic LLM, response endpoint

A만 같은 audio를 timestamp-capable Streaming ASR에 병렬 전송한다. B는 별도 ASR 없이 B용 S2S가
같은 audio를 처리한다. 이 speech evidence topology와 그 실제 비용·지연·장애만 DP-03 독립 변수다.

### Candidate-visible input

- audio와 동결된 A/B dependency mock이 반환한 transcript/alignment event
- candidate contract가 허용한 UI event와 persistent-state reference
- clock mapping receipt, source epoch·event·segment identity
- 공통 pattern registry

### Evaluator-only oracle

- 사람이 확정한 final transcript와 지칭 span의 audio source interval
- 지칭 span별 expected UI target·집합·역할과 semantic correction relation
- expected final request fields와 금지 output/action
- trial별 fault schedule, timing endpoint와 pass/fail 판정

Oracle transcript, 정답 target identity와 semantic correction label을 dependency mock이나 candidate
input에 주지 않는다. Mock은 동결된 speech-evidence event만 제공하며 candidate가 만든 final field를
hidden oracle과 비교해 QA-19 `MEASURED_MOCK_E2E` 점수를 만든다.

Machine-readable 초안은 두 파일로 물리적으로 분리한다.

- [Candidate-visible Fixtures](./contracts/via-dp-03-fixture-inputs.json): 동일 audio reference, UI timeline과
  A/B mock이 실제로 전달할 transcript/alignment event만 포함한다.
- [Evaluator-only Oracles](./contracts/via-dp-03-hidden-oracles.json): expected span target·role, 최종
  QA-19 field와 금지 결과만 포함하며 candidate process에는 mount하거나 전달하지 않는다.

현재 audio asset path와 source timing은 정의했지만 WAV와 digest는 아직 생성하지 않았으므로 두 파일은
`DRAFT_FOR_FREEZE`다.

## 2. Functional qualification pack

이 pack은 scored Core-ASR run보다 먼저 양쪽이 같은 기능을 만들 수 있는지 확인한다. Existing Test
Case의 사용자 의미를 재사용하되 DP-03에 필요한 source timeline과 hidden oracle을 별도로 고정한다.

| Qualification ID | Source | 발화·UI 사건 | candidate 단계의 필수 결과 | Grounding 이후 oracle |
| --- | --- | --- | --- | --- |
| DP03-QF-01 | SP-01 / TC-04.1 | chart-A를 가리키며 “여기 표시된 값” | 한 final span과 chart-A 시점 UI reference | chart-A 하나 |
| DP03-QF-02 | SP-01 / TC-04.2 | chart-A에서 첫 “여기”, chart-B에서 둘째 “여기” | 서로 다른 source interval과 UI slice를 가진 두 candidate | 순서대로 chart-A, chart-B |
| DP03-QF-03 | SP-01 / TC-03.1/03.5 | 발화 전에 selection 또는 pointer를 둔 뒤 지칭 | pre-turn state를 포함한 timeline slice | 선택/가리킨 대상, 발화 후 최신 snapshot 금지 |
| DP03-QF-04 | SP-02 / TC-04.4 | “이 문장들”이라고 말하며 drag | 지칭 interval의 selection transition reference | 완성된 para-A range |
| DP03-QF-05 | SP-04 / TC-04.6 | “이 부분 아니 여기만”과 para-A→para-B pointing | 두 final candidate와 correction cue가 있는 final transcript 보존 | para-A 철회, para-B 유효 |
| DP03-QF-06 | SP-01 / TC-04.7 | 두 창/문서에서 같은 표현과 좌표 사용 | span별 window/document/viewport reference 분리 | 서로 다른 두 document object |
| DP03-QF-07 | 신규 ambiguity variant | 지칭 표현은 있으나 pointer·selection·focus 근거 없음 | candidate를 버리지 않고 empty/null evidence를 정직하게 보존 | 임의 target 금지, clarification |
| DP03-QF-08 | A lifecycle qualification | partial에 지칭 span이 나타났다가 recognizer revision에서 text 자체가 삭제 | provisional→retracted provenance 보존, final handoff 제외 | semantic correction case와 혼동 금지 |

DP03-QF-08은 A의 revision mechanism만 검증하므로 B는 `NOT_APPLICABLE`이다. 이 case를 B의 실패로
세거나 A/B final correctness 분모에 넣지 않는다. DP03-QF-05에서는 두 표현이 final transcript에
남으므로 어느 후보도 DP-03 단계에서 첫 candidate를 `retracted`로 만들지 않는다.

## 3. UI timeline fixture 규칙

각 UI event는 최소한 다음을 가진다.

- `ui_event_id`, `ui_clock_id`, source monotonic time, event sequence
- display/window/document/viewport identity
- pointer이면 coordinate, button/drag phase와 source가 제공하는 hit-test identity
- selection이면 set/range identity와 revision
- focus/caret이면 focused object identity와 revision
- screen revision이 바뀌면 그 identity와 이전 revision relation

Candidate는 [DP-03 Candidate Contract](./via-dp-03-candidate-contract.md)의 anchor policy에 따라
reference만 pin한다. Evaluator가 아는 expected target을 `ui_candidates`처럼 미리 계산해 주지 않는다.
Object identity가 없는 fixture는 coordinate와 해당 시각의 topmost hit-test oracle을 evaluator에만 둔다.

Canonical qualification fixture에는 발화 시작 전 1,000 ms를 포함한다. 이는 현재 Test Case의 최대
약 600 ms pre-turn 사건을 재현하기 위한 fixture 범위이며 제품 retention 목표가 아니다. 제품
retention은 승인된 최대 발화·finalization·handoff timeout을 포함해 별도 산정한다.

## 4. Speech fixture 규칙

동일 script를 A/B에 사용하되 다음을 manifest에 고정한다.

- speaker/recording source, locale `ko-KR`, sample rate, channel, PCM encoding과 audio digest
- speech start/end와 utterance end를 판정하는 physical/source event
- 사람 annotation의 final transcript, code-point span, audio start/end와 annotation revision
- 각 지칭 span 사이의 pointer/selection/window event 순서
- correction cue, 조사·붙여쓰기와 중복 표현을 포함한 pattern coverage
- 무음, 발화 속도와 background-noise profile

PoC는 deterministic synthetic speech를 공통 기본값으로 사용하고 generator/profile을 기록한다.
A/B에는 같은 audio bytes를 사용한다. Mock adapter format에 맞추기 위한 resample은 공유 canonical
audio에서 결정적으로 만들고 mapping receipt와 digest를 남긴다.

## 5. Core-ASR 측정 계약 초안

### QA-09 — PRIMARY

대표값은 DP-03 내부 milestone이 아니라 applicable QA-01/02/05 interaction trial의 실제
`VIA-attributable milliseconds` 산술평균이다. Voice input의 start/end와 의미 있는 audible response
onset은 [Event & Boundary Contract](./event-boundary-contract.md)를 따른다.

내부 진단으로 다음을 함께 기록한다.

- A: first relevant partial, provisional candidate, UI pin, ASR final, final candidate, Grounding handoff
- B: utterance end, S2S timestamped final, final candidate, Grounding handoff
- 공통: Grounding/semantic completion, response generation, playback queue와 meaningful audible onset

A의 provisional candidate가 final response 전에 준비 비용을 줄였는지는 이 span으로 설명하되 내부
milestone을 QA-09 종료점으로 바꾸지 않는다. 공통 기본값은 warm-up 5회, spine+patch당 A/B 각
30회, missing/incorrect terminal 5,000 ms다. Target과 score는 Scoring Contract의 공통 system band를
사용한다.

### QA-19 — PRIMARY

Scored field는 final semantic output의 중복 없는 QA-11/12 oracle field다. DP-03 candidate의
timestamp error, revision count와 pin count는 원인 진단이고 같은 사실을 다시 score하지 않는다.

- DP03-QF-01~07의 applicable `input.final_text`, `referent.span_order`, `referent.target_set`,
  `referent.role_by_span`, `referent.retracted_target_set`, `request.goal`, constraint/task relation/handling
- 누락 field, 잘못된 추가 target, 순서 뒤집힘과 임의 target은 field FAIL
- ASR/S2S가 final transcript나 alignment를 내지 못해 final semantic output이 없으면 성공 trial만
  골라내지 않고 applicable field를 FAIL로 남김
- DP03-QF-08은 lifecycle qualification이며 QA-19 A/B score 분모 밖

### QA-29 — PRIMARY

DP-03에 직접 적용할 pre-result change pack은 Core-ASR 원장의 `M-01`, `M-07`, `C-03`,
`E-01`, `E-02`, `E-03`으로 고정한다. 각 변화는 동일 baseline에서 독립 적용하고 누적하지 않는다.

| Change | 고정 자극 | A에서 추적할 경계 | B에서 추적할 경계 |
| --- | --- | --- | --- |
| M-01 S2S provider 교체 | 동등 Voice 기능의 S2S adapter/profile 교체 | S2S 대화 경로; ASR authority는 유지 | Voice와 timestamp final contract를 함께 제공할 새 S2S adapter/build |
| M-07 S2S event 정보 변경 | word timestamp→segment timestamp/revision 계약 | S2S timestamp를 소비하지 않으므로 ASR evidence 계약은 유지; 기능 회귀만 수행 | S2S native alignment unit·candidate adapter·evidence state 변경 |
| C-03 화면 연동 계약 변경 | handle/coordinate/selection 표현 변경 | 공통 UI timeline normalizer와 evidence reference | 공통 UI timeline normalizer와 evidence reference |
| E-01 timing span 추가 | candidate lifecycle에 span 하나 추가 | partial/final/candidate producer와 trace contract | final/candidate producer와 trace contract |
| E-02 correlation 추가 | 새 correlation dimension 전파 | S2S+ASR+candidate causal chain | S2S final+candidate causal chain |
| E-03 trace schema 확장 | backward-readable event field 추가 | 공통/versioned event writers와 readers | 공통/versioned event writers와 readers |

Architecture Element registry와 실제 changed-element ledger가 없으면 구조 설명만 남기고 숫자를 만들지
않는다. 공통 library 하나의 변경을 호출 지점 수만큼 중복 세지 않으며, B model training 내부는 VIA
Architecture Element가 아니지만 model output/API·adapter·deployment contract 변화는 센다.

Machine-readable pre-result baseline과 touched-element set은
[DP-03 Change Ledger](./contracts/via-dp-03-change-ledger.json)에 둔다. 이 원장은 결과를 본 뒤 요소
경계를 바꾸지 않도록 C/I/S/D identity와 여섯 독립 change patch를 먼저 고정한 것이다. 최종 QA-29
수치는 Candidate Implementation에서 각 change patch와 SP-01/02/04 기능 회귀를 실제로 통과한 뒤
같은 ID 집합으로 집계한다.

### QA-39 — REGRESSION_ONLY

| Fault ID | 주입 | 공통 PASS 조건 |
| --- | --- | --- |
| DP03-F-01 | evidence source가 final 전에 disconnect | provisional/stale candidate를 final로 사용하지 않고 truthful clarification/recovery; 중복 Action 없음 |
| DP03-F-02 | old epoch event가 reconnect 뒤 도착 | 새 epoch candidate를 되살리거나 덮어쓰지 않음 |
| DP03-F-03 | 같은 partial/final event duplicate | candidate/handoff/dispatch가 idempotent |
| DP03-F-04 | final보다 늦은 partial 또는 sequence inversion | terminal revision을 rollback하지 않음 |
| DP03-F-05 | audio↔monotonic clock mapping gap | 잘못된 UI slice를 추정하지 않고 evidence invalid/clarification |
| DP03-F-06 | 필요한 UI slice가 retention에서 evict | 현재 snapshot으로 대체하지 않고 evidence unavailable 처리 |

A의 F-01은 Streaming ASR, B의 F-01은 S2S timestamped-final evidence에 주입한다. 후보마다 물리적으로
불가능한 event는 억지로 복제하지 않고 source-specific fault로 기록하되, 최종 containment·정확한
recovery·no-duplicate·evidence 조건은 동일하다. 공통 기본값은 fault item당 A/B 각 10회와
5,000 ms recovery deadline이다.

Fault injection 의미, user-visible unit과 candidate별 necessary dependency closure는
[DP-03 Fault Registry](./contracts/via-dp-03-fault-registry.json)에 둔다. F-01은 현재 turn의 speech
evidence session disconnect이며 S2S 전체 서비스 중단으로 확대하지 않는다. 따라서 A/B 모두 현재
UI-grounded Voice request만 necessary closure에 넣고 unrelated Voice/Text/Task/Agent/Context path는
계속 사용 가능해야 한다.

## 6. 실행 전 qualification 순서

1. JSON schema와 pattern registry의 static validation
2. SP-01/02/04 fixture에 A/B event patch와 hidden oracle을 작성
3. A의 partial/revision/final nominal·slow·error/fault mock trace 동결
4. B의 turn-final aligned-text nominal·slow·error/fault mock trace 동결
5. fixture/oracle/mock-profile digest와 DP-03 change/fault registry freeze
6. 그 뒤에만 Candidate Implementation과 A/B Measurement & Evaluation로 이동

Candidate Implementation 뒤 이 mock trace로 실행한 결과는 `MEASURED_MOCK_E2E` QA-09/19/29/39
공식 PoC evidence다. 실제 provider/model trace는 별도 revalidation이며 이 순서의 선행조건이 아니다.

## 7. Measurement Freeze에 남은 값

| 값 | 현재 상태 |
| --- | --- |
| SP-01/02/04 fixture·oracle digest | L-01~07 machine payload `DRAFT`; WAV·digest·input-separation receipt `PENDING` |
| A/B mock profile ID, schedule와 deterministic seed | machine profile `DRAFT`; payload fixture·freeze digest `PENDING` |
| DP-03 event patch·field/change/fault registry digest | machine registries `DRAFT`; reviewed digest `PENDING` |
| PoC UI timeline retention | fixture pre-turn 1,000 ms + 발화·finalization·handoff schedule로 계산 예정 |
| 공통 run order·repetition·timeout·recovery deadline | Shared contract 기본값 채택; freeze digest 전 |
| QA-09/19/29/39 target·score band | Scoring Contract proposal 사용; DP-03 freeze 전 |
| evidence label | `MEASURED_MOCK_E2E` |

상용 ASR 선택, cloud audio 전송 승인과 팀 S2S build milestone은 이번 PoC Measurement Freeze의 blocker가
아니다.

Pre-freeze machine 초안은 [DP-03 Spine Patch Registry](./contracts/via-dp-03-spine-patch.json),
[A Streaming ASR Mock Profile](./contracts/via-dp-03-a-streaming-asr-mock-profile.json)과
[B S2S Mock Profile](./contracts/via-dp-03-b-s2s-mock-profile.json),
[Candidate-visible Fixtures](./contracts/via-dp-03-fixture-inputs.json)와
[Evaluator-only Oracles](./contracts/via-dp-03-hidden-oracles.json)에 둔다. 아직 `PENDING`인 것은 초안의
존재가 아니라 이 값들을 fixture payload·oracle digest와 묶어 변경 불가능한 Measurement Freeze로
승인하는 일이다.
