# Core ASR 공통 측정 Harness 계약

> 상태: **FOUNDATION REQUIRES CORE-ASR V2 UPDATE / DP 실행 금지 상태**
>
> 목적: DP마다 측정 코드를 다시 만들지 않고, 모든 후보가 같은 event·oracle·집계·실패 처리를 사용하는 공통 기반을 고정한다.

## 1. 이 기반이 통과하기 전에는 DP를 측정하지 않는다

다음 네 조건을 모두 통과해야 DP별 A/B campaign을 시작할 수 있다.

1. **Contract completeness:** QA-09/19/29/39와 그 상세 input의 모든 endpoint·field·change·fault verdict가 machine contract에 있고 `default PASS`가 없다.
2. **Failure sensitivity:** QA별 정상 trace는 PASS하고, 의도적으로 한 의무를 깨뜨린 sentinel trace는 반드시 FAIL한다.
3. **Endpoint fidelity:** Voice 대표값은 annotated Voice input과 audio loopback endpoint를 사용한다. queue clear·payload delivery·renderer 함수 반환은 진단값일 뿐 대표 QA endpoint가 아니다.
4. **Independent replay:** raw evidence만으로 네 core ASR의 원 분자·분모와 산술평균/성공률을 정확히 다시 계산한다.

이 gate를 통과하지 못한 실행은 `PRELIMINARY_HARNESS_DIAGNOSTIC`이며 DP, ASR 또는 Architecture 결론에 사용하지 않는다.

## 2. 한 번 구현해 모든 DP가 공유하는 구조

```mermaid
flowchart LR
  F["Frozen fixture pack<br/>WAV·Context·Agent events"] --> R["Candidate-neutral scenario runner"]
  O["Evaluator-only oracle<br/>QA-11/12 fields + regression predicates"] --> E["Common QA evaluator"]
  R --> A["Candidate A adapter"]
  R --> B["Candidate B adapter"]
  A --> T["Versioned raw event trace"]
  B --> T
  L["Audio loopback observer"] --> T
  G["Reference Agent source clock"] --> T
  T --> E
  E --> S["4-core-ASR table<br/>+ detailed diagnostics"]
  T --> X["Independent replay analyzer"]
  X --> S
```

DP adapter는 Architecture 차이만 구현한다. QA 판정, oracle, audio endpoint, failure 처리와 집계 코드는 바꾸지 않는다. DP와 무관한 공통 실행은 `source_execution_key`로 한 번만 보존하고 양쪽에 매핑한다.

## 3. 공통 실행 단위

모든 trial은 다음 세 파일의 frozen digest에 연결된다.

| 입력 | 내용 | Candidate에 제공 여부 |
| --- | --- | --- |
| Scenario fixture | Voice WAV, Text input, visible Context, 초기 Conversation·Task state, Reference Agent event script | 제공 |
| Evaluator oracle | 허용 semantic graph, identity relation, final state, continuity relation, mandatory cardinality | **미제공** |
| Measurement contract | event 의미, timeout, 반복·집계, evidence requirement | 공통 |

Voice input은 사람이 실시간 발화하지 않는다. 사전 생성한 PCM WAV의 `speech_start_frame`, `speech_end_frame`, barge-in onset과 digest를 고정하고 실제 capture timeline에 재생한다. 사용자는 마이크에 말할 필요가 없다.

Voice output 대표 endpoint는 실제 audio output device를 audio loopback으로 관측한다. 시작 sample과 중단 대상의 마지막 sample을 waveform correlation으로 찾는다. Model packet 도착, buffer enqueue, queue clear와 callback은 원인 분석 event로만 남긴다.

## 4. QA-09 시간 측정과 상세 event

모든 latency trial은 같은 monotonic clock domain의 원시 timestamp에서 계산한다. 물리적 endpoint가 없거나 event provenance가 무효이면 frozen timeout 값을 넣는다. 잘못된 응답의 실제 시각은 raw trace에 보존하지만 올바르고 의미 있는 terminal event가 아니므로 QA-09 trial 값에는 frozen timeout을 적용하고 QA-19 field 실패를 함께 기록한다.

| 상세 stratum | 필수 원시 event | QA-09 trial 계산 | 상세 진단 |
| --- | --- | --- | --- |
| QA-01 | `user_input_end`, `agent_request_available_at_agent_ingress`, `agent_result_available_at_source`, `first_meaningful_audible_result_audio` | `(t1-t0)+(t3-t2)`; Agent 내부 `t2-t1` 제외 | case별 nearest-rank p95 중 최댓값 |
| QA-02 | `user_input_end`, `first_meaningful_audible_direct_response` | `t1-t0` | case별 nearest-rank p95 중 최댓값 |
| QA-03 | `agent_status_available_at_source`, `first_meaningful_audible_status_audio` | `t1-t0` | case별 nearest-rank p95 중 최댓값 |
| QA-04 | `barge_in_speech_onset`, `interrupted_response_last_audible_sample` | `t1-t0` | case별 nearest-rank p95 중 최댓값 |
| QA-05 | `task_control_input_end`, `correct_task_control_disposition_presented` | `t1-t0` | case별 nearest-rank p95 중 최댓값 |

QA-05에는 input finalization, semantic target resolution, Task binding, control 전달, 필요한 source confirmation과 실제 Voice/UI presentation이 모두 포함된다. `cancel RPC + query` 같은 내부 부분시간만 QA-05라고 부르지 않는다.

QA-09의 대표값은 QA-01/02/03/05 applicable scored trial 전체의 **산술평균**이다. 네 stratum의 단위는 모두 VIA-attributable milliseconds이며 A/B에는 같은 case와 repetition을 쓴다. 합계, sample count, class·case별 평균과 timeout 수를 함께 공개한다. 위 표의 p95는 상세 계약과 과거 결과 추적을 위한 tail diagnostic일 뿐 QA-09 target이나 score를 대체하지 않는다. QA-04는 Voice interruption 회귀이므로 QA-09 분모에서 제외한다.

### Voice endpoint 허용 근거

| Provenance | 대표 QA 사용 | Evidence label |
| --- | --- | --- |
| Annotated input WAV가 실제 capture timeline에 주입됨 | 허용 | 실행 범위에 따라 reference/product |
| Audio loopback에서 의미 있는 output onset/last sample 검출 | 허용 | 실행 범위에 따라 reference/product |
| Instrumented renderer callback | 대표값 금지, 진단값만 | `MEASURED_REFERENCE_HARNESS` |
| payload delivery, queue enqueue/clear, API return | 대표값 금지 | diagnostic only |

## 5. QA-19 field accuracy와 QA-13~15 회귀

Evaluator는 case마다 명시된 predicate만 평가하며 누락된 predicate를 `true`로 간주하지 않는다. 적용되지 않으면 oracle에 명시적으로 `N/A`가 있어야 한다.

| QA | 반드시 평가하는 machine predicate | PASS 조건 |
| --- | --- | --- |
| QA-11 | 해당 case의 response/result, QA-12~15 applicable predicate, dispatch/control cardinality, mandatory safety gate | 모든 applicable predicate PASS |
| QA-12 | goal, constraints, deliverables, referents, request decomposition/dependency, clarification, Task relation, direct/Agent handling, Agent/capability | 허용 semantic graph와 일치 |
| QA-13 | Conversation, Request, Task, Agent execution, question, result/artifact의 binding과 delivery cardinality | 모든 required relation·cardinality 일치 |
| QA-14 | revision acceptance, duplicate suppression, stale-event rejection, race resolution, terminality, result reference, pending interaction, allowed control | 전체 event script 뒤 final state oracle와 일치 |
| QA-15 | channel·connection·conversation·Task 전환 전후 identity/referent 관계 | 모든 continuity relation 유지 |

QA-11은 통합 outcome이고 QA-12는 semantic driver다. 두 계약의 atomic predicate를 중복 없는 field ID로 평탄화하여 QA-19의 `correct applicable fields / all applicable fields`를 계산한다. QA-13~15는 Task binding·state convergence·continuity 회귀이며 QA-19 분모에 자동 포함하지 않는다.

QA-19 결과에는 다음 대표값과 보조지표를 함께 보존한다.

- `QA-19 field_accuracy = correct non-duplicate applicable QA-11/12 fields / all such fields`
- `strict_case_success = all applicable predicates가 맞은 case 수 / scored case 수`

Field accuracy가 QA-19의 대표 metric이다. QA-11/12 strict 비율과 QA-13~15 pass rate는 반드시 분자/분모와 함께 진단으로 표시한다. Semantic stage만 실행한 campaign은 integrated QA-11 field를 측정한 것처럼 채우지 않고 `UNRESOLVED / NOT_MEASURED_PRODUCT_INTEGRATED_OUTCOME`으로 둔다. QA-12 값을 QA-11 field로 복제하지 않는다.

### Oracle 연산자

`EXACT`, `ONE_OF`, `SET_EQUAL`, `ORDERED_RELATION`, `REQUIRED_PROPOSITION`, `FORBIDDEN_PROPOSITION`, `CLARIFICATION_REQUIRED`, `BINDING`, `CARDINALITY`, `FINAL_STATE_EQUAL`, `CONTINUITY_RELATION`만 사용한다. 자연어 문체를 채점하지 않고 structured fact와 관계를 채점한다.

## 6. corpus와 반복 freeze

Fixture 수를 임의로 늘려 쉬운 case로 실패를 희석하지 않는다. 아래 v1 수치는 기존 상세 harness의 planning reference이며 새 core-ASR campaign의 승인값이 아니다. QA-09/19/29/39 모집단과 반복은 pilot variance, 요구 precision과 실행비용을 근거로 별도 Measurement Freeze에서 정한다.

| Pack | 최소 구성 | 반복 |
| --- | --- | --- |
| Latency canonical | QA-01 4 case, QA-02 4, QA-03 3, QA-04 2, QA-05 7 | case별 10 warm-up + 100 scored paired trials |
| Semantic correctness | 승인된 N-01~06 최소 28 case; semantic predicate 종류별 최소 3 case가 안 되면 필요한 case만 추가 | case별 3회 |
| Binding | progress/question/result/failure/follow-up/cancel/multiple-run 각각 포함 | deterministic case별 3회 |
| State convergence | stale, duplicate, reorder, cancel-complete, question-terminal, push-query, partial-terminal 7 strata | stratum별 20회 |
| Continuity | 문서의 8개 전환 scenario를 모두 포함 | scenario별 10회 |

Model·S2S가 DP의 변경 경로에 참여하지 않으면 같은 frozen source execution을 A/B에 매핑할 수 있다. 참여하면 후보별 실제 실행이 필요하다. 표본을 복제해 독립 실행 수를 늘리지 않는다.

## 7. Failure-sentinel qualification

공통 evaluator는 다음 mutation을 모두 검출해야 한다.

| QA | 정상 trace | 반드시 실패해야 하는 sentinel |
| --- | --- | --- |
| QA-01 | 네 endpoint와 올바른 result | Agent 실행시간을 VIA 시간에 포함, ingress 누락, wrong result, audio proxy만 존재 |
| QA-02 | direct route와 loopback onset | Agent가 참여, filler를 endpoint로 사용, output correctness 실패 |
| QA-03 | 새 status·올바른 binding·loopback onset | stale/duplicate status, wrong Task, source 시각을 receive 시각으로 대체 |
| QA-04 | barge-in annotation과 loopback last sample | queue-clear/cancel-return을 종료점으로 사용, 새 응답 생성시간 포함 |
| QA-05 | 입력 종료부터 truthful disposition presentation | semantic 시간 제외, wrong Task, 근거 없는 `canceled`, local enqueue를 종료점으로 사용 |
| QA-11 | 모든 applicable predicate PASS | driver·response·cardinality 중 하나라도 FAIL |
| QA-12 | 전체 semantic graph 일치 | route는 같지만 constraint/referent/dependency 중 하나가 틀림 |
| QA-13 | 전체 identity graph와 cardinality 일치 | wrong Task/run/question, duplicate dispatch/control |
| QA-14 | oracle final state 수렴 | stale revision 수락, terminal reopen, cancel-complete race 오판 |
| QA-15 | 모든 transition relation 유지 | provider connection ID를 Conversation ID로 대체, channel 전환 뒤 Task/referent 손실 |

각 sentinel은 `expected_failure_code`까지 일치해야 한다. 단순히 예외가 났다는 이유로 qualification을 통과하지 않는다.

## 8. DP campaign acceptance

DP별 package는 QA-09/19/29/39 각각을 다음 중 하나로 기록하고 상세 진단을 연결한다.

- `PRIMARY`: 구조 차이의 자연 인과를 사전 가정하고 공통 evaluator로 A/B를 비교함
- `REGRESSION_ONLY`: 양쪽이 동일 의무를 만족하는지 실제 실행함
- `COMMON_SOURCE_REGRESSION`: DP가 물리적으로 참여하지 않아 같은 source execution을 명시적으로 매핑함
- `NOT_APPLICABLE`: 해당 QA 모집단에 그 DP 경로가 없고 이유가 component path로 증명됨
- `UNRESOLVED`: 필요한 설계, endpoint, registry 또는 dependency를 실행하지 못함

Proxy 값을 넣어 `PRIMARY` 칸을 채우지 않는다. `UNRESOLVED`는 좋은 점수가 아니며, 모든 core ASR을 억지로 숫자로 만드는 것보다 정직하게 남긴다.

## 9. 현재 상태

VIA-DP-11 v3는 raw format·표·independent replay의 개발 자료다. 위 contract의 endpoint fidelity와 correctness sentinel을 통과하지 않았으므로 공식 campaign이 아니다. 현재 다음 작업은 VIA-DP-03 A의 Streaming ASR와 B의 timestamp-capable S2S capability qualification 및 pre-result Measurement Freeze 초안이다. 그 freeze가 승인된 뒤 candidate나 campaign보다 먼저 이 공통 evaluator와 qualification suite를 Core-ASR v2로 구현한다. VIA-DP-11을 재실행하거나 과거 harness를 active 구현으로 되살리지 않는다.

## 10. 공통 결과 저장 형식

새 DP campaign은 `results/architecture-evaluation/current/dpNN-<campaign>-vN-YYYYMMDD/` 아래의 새 immutable directory에 둔다. 기존 directory를 덮어쓰지 않는다. 최소 구성은 frozen contract/fixture/oracle과 digest, candidate별 raw trace, environment·source manifest, 재계산 가능한 `summary.json`, 사람이 읽는 `report.md`다. A/B 이후 선택안 보완 tactic인 B′를 실행했다면 applicable QA의 같은 complete table에 세 번째 후보로 표시하고, 실행하지 않은 ledger나 endpoint는 N/A로 남긴다.
