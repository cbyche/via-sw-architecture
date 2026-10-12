# V6 고정 전 질문 포트 독립 검수

상태: `FINAL_READ_ONLY_INTEGRATION_REVIEW / NO_UNRESOLVED_CONFIRMED_P1_P2 / API0`.

Root의 최종 편집 종료 신호 뒤 live 수정본, V5 frozen-source와의 diff, 지정 모델 출력의 실제 owner/runtime/후보 연결 trace를 읽었다. 검수자는 API, 테스트, 후보 실행, 메모리 원장 전체 읽기, 코드 또는 입력/정답 수정을 수행하지 않았다. 작성자는 분리된 unittest suite의 DP41 29개, shared 64개, benchmark 81개 PASS와 API0를 보고했다. 이 숫자는 검수자가 재실행한 테스트 수가 아니다.

이번 검수에서 미해결 확정 P1/P2는 없다. V5의 QUESTION_PRESENT 후속 입력 누락, B의 pending 질문을 표시할 때도 presented 조건을 요구한 연결 불일치, 독립 검토에서 확인한 허용 질문 표현 거절 및 once-only 예약 소비 문제의 수정 연결을 확인했다. 이는 실제 모델 의미 정확성, QA 통과/점수 또는 A/B 우열의 판정이 아니다. V1~V5 진단 결과를 V6 공식 결과로 재사용하지 않는다.

근거는 [V5 전수 actual 검토](v5-all-scenes-port-audit.md), [현재 QA v6](../../../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md), [V5 manifest](../dp41-p21-mac-text-v5-20261012/manifest.json)와 아래 API0 trace다.

## 최종 수정본과 trace

마지막 author hash와 직접 계산한 live hash 및 최신 trace의 source_hashes가 모두 일치했다.

| 파일 | 최종 SHA-256 |
| --- | --- |
| benchmark/architecture/dp41_45_poc/worker.py | `edc0169b73a1f0a2dea497723cc52dbe5138881d023a844e36236a5567140c49` |
| benchmark/architecture/dp41_45_poc/tests/test_worker.py | `afe39265e958199a370b325379b7666211b3d8f5bfe6c61164f821fbedecb081` |
| prototypes/candidates/dp41_45_poc/shared/runtime.py | `2a3839a67285128cd177f2450da6985f087a2c6b4754f903f19907e0236c3124` |
| prototypes/candidates/dp41_45_poc/dp41/b.py | `49c69506ef737fb4335ca3649ba25165f689f9f110a3a169fc169c842bed0e19` |
| prototypes/candidates/dp41_45_poc/dp41/tests/test_dp41.py | `23ab8241fb99fd429435363902727658517be8003dab4c705b549202082858fb` |

| 증거 | SHA-256 | 범위 |
| --- | --- | --- |
| [최신 once-only functional trace](question-followup-v6-once-only-functional-regression/trace.json) | `f6c395c01c66f5cdc2091064bad6a8459b07113eb122da4f6aa371dd4078d70e` | 4 probe, 5 observation, 최종 source hash |
| [허용 질문 표현과 literal 수정 trace](question-followup-v6-functional-regression-final/trace.json) | `73c8abb01ab8cf862fcca79bdd676f342c38a814ba4b2ce8f219f9fd4e2362d5` | 이전 checkpoint의 A/B 연결 증거, 이후 once-only 수정 전 hash |
| [최초 연결 trace](question-followup-v6-functional-regression/trace.json) | `fb4538da12f290a27c557618c57f19e779e37e8f9246e992f5cd087cc93487cc` | 이전 checkpoint 보존, 최종 코드 증거로 대체 사용하지 않음 |

세 trace 모두 `SCRIPTED_FUNCTIONAL_REGRESSION`, `formal_population=false`, `api_calls=0`으로 구분한다. wait/speech는 테스트 전용 0.002초이고 Voice도 공통 모형이므로 이 trace의 시간을 QA02/03 측정값으로 사용하지 않는다. 실제 후보 resolve, 실제 OwnerRepository, 실제 LocalVIA와 명령/전달 포트를 사용했지만 provider의 의미 출력은 지정됐다.

## 원문과 실제 입력 공급 경로

QA v6 S6-02의 최초 '응.'은 철회된 Q1v1 또는 미제시 Q1v2의 승인이 아니다. Q1v2의 연락처 조회와 해당 메일 업무 제공 동의를 실제 제시한 뒤에만 지정 3초 후속 답을 공급해야 한다. 실제 ANSWER 채택 뒤 주소를 조회/제공하며 Q2의 SEND 승인은 끝까지 없다.

수정된 공통 `deliver`는 playback lock 해제 및 최종 delivery 원장 저장 뒤 tracked 후속 입력 job을 시작한다. automatic/external 질문과 goal QUESTION_PRESENT 모두 같은 실제 표시 경계를 통과한다. private CLARIFICATION mode는 해당 후보의 기존 Request refinement를 유지하며 공통 새 Request callback을 중복 시작하지 않는다.

| 검사 경계 | 확인 결과 |
| --- | --- |
| 실제 제시와 현재성 | Q identity/revision/kind/scope, 실제 Text, Voice ON의 완결 Voice, current VALID 상태와 owner Task의 Execution/request revision을 확인한다. Voice OFF는 Text receipt로 처리한다. |
| 예정 답 공급 | 실제 Q1v2/T메일/민수 및 READ_CONTACT+PROVIDE_TO_TASK 범위를 대조한다. private 주소나 정답을 callback의 새 의미/payload로 만들지 않는다. |
| 후보 Context | 실제 후속 전사와 실제 presentation에서 얻은 question_focus를 넣는다. Context에 plan, eligible_planned_reply, source release 원문 또는 비공개 주소가 없다. 과거 interaction의 at_seconds=-1은 이미 발생한 기록이다. |
| 원천 release | 주소는 모델 Context에 선주입되지 않는다. 실제 후속 ANSWER_QUESTION의 current scope/consent가 허용된 뒤 actual owner read와 업무 제공 payload에 나타난다. |
| CLARIFY | 같은 Request의 후보 refinement이며 generation을 증가시키거나 별도 Request를 동시에 생성하지 않는다. 반환 answer의 실제 transcript와 Q/revision은 유지한다. |
| Agent 질문 | 원문상 추가 답이 없는 Task/Agent 질문에는 neutral unexpected reply를 공급하지 않는다. S6-01의 명시 planned reply는 원문의 첫 관련 질문 조건을 사용하며 origin만으로 금지하지 않는다. |
| partial/withdrawal | partial Voice, 제시 전 철회, 대기 중 철회, Task Execution/revision 변경, 역순 old question/withdrawal 사건은 후속 입력을 공급하지 않는다. 관련 author 회귀는 actual 포트를 사용한다. |

독립 검토 중 QA C6의 허용 예문인 '발송 승인은 별개예요.'가 초기 finite text gate에서 거절되는 P2를 보고했다. 최종 gate는 별도/부정 SEND 구문을 제거한 뒤 남은 positive SEND 표현을 검사한다. 최신 이전-checkpoint trace 및 최종 once-only trace의 B는 이 허용 문장을 실제 full Voice로 제시하고 지정 답을 받았다. 같은 문장에 '지금 보내도 될까요?'를 덧붙이면 답을 공급하지 않는 author 회귀도 있다. 이 finite gate의 검증을 자유로운 한국어 표현 전반의 의미 판단 능력으로 일반화하지 않는다.

## actual once-only와 B 표시 전용 바인딩

중복 방지의 단위는 실제 후속 입력 1회다. `_question_answer`는 `(question_id, revision)`을 callback 진행 중 예약하고, None 또는 예외면 예약을 해제하며 실제 answer 반환 때 유지한다. 다음 경로를 최종 trace에서 직접 확인했다.

| probe | 실제 관측 |
| --- | --- |
| A/B Owner→후보→QUESTION_PRESENT | A 실제 PROPOSE와 B 실제 finite Frame/코드 binding이 Q1v2를 표시하고 각각 새 Request에서 ANSWER_QUESTION을 채택했다. |
| 부적합 full 제시 뒤 재제시 | '이 메일을 발송할까요?'는 full Text receipt가 있어도 지정 답이 없었다. 같은 Q1v2의 적합 재제시 뒤 답/명령은 각 1회다. 첫 잘못된 문장도 trace에 보존된다. |
| 동시 중복 표시 | callback 진행 중 두 번째 표시가 있어도 callback/실제 후속 입력/ANSWER는 각 1회다. |
| callback 예외 | plan이 소비되기 전 주입된 첫 callback 예외 뒤 예약이 해제됐고, 같은 질문 재제시에서 답이 1회 공급됐다. 예외 원장도 남았다. |

B의 `_presentation_only_reference`는 binder가 소비하는 선언된 위치만 검사한다. goal target, source_refs, recipient_ref, body/body_result_reference.source_ref, WHEN predicate.reference_id 및 preservation.reference_id로 함께 소비되는 ref에는 pending-question 예외를 주지 않는다. QUESTION_PRESENT.question_ref에만 쓰는 VALID pending 질문은 표시를 위해 바인딩할 수 있다. ANSWER나 mixed use, terminal/withdrawn/stale 질문을 표시로 바꾸지 않는다. property value와 body.text의 literal 문자열이 ref ID와 같더라도 이를 실행 참조로 오인하지 않는 actual Owner 기반 회귀를 확인했다.

이 필터는 모델이 정한 Goal/operation을 변경하거나 확인 질문의 답을 추측하지 않는다. 기존 partial selector, 여러 모델 호출, CLARIFY 후 새 finite Frame 및 코드의 최종 의미 결합은 유지한다. 선언된 partial Goal/value 지원 한계와 bounded lexical 검색 한계를 새 연결 결함으로 재분류하지 않았다.

최종 main probe의 A 명령은 `CMD:40fbfe72-d0b4-4d69-8801-8b5f154db83c`, Request는 `R:7c0419cf-ac4b-4a9d-8ecb-1fff1ad03201`, payload hash는 `94e0fd6b713b80da8026e2484eae86d3660d170fde61e84f1fb489221749eee2`다. B 명령은 `CMD:987543c5-f9f1-4e4a-ae59-573e4116c88f`, Request는 `R:129600c7-ed45-461a-8763-a8bf065e7c4a`, payload hash는 `eb86146faa19e9b3a09334c387f0d3ab1b53855b00fb9a815f40b14625200d2f`다. 두 payload의 실제 operation은 ANSWER_QUESTION, target은 T메일/X메일, question은 Q1v2, consent는 read=true/share=true/send=false다. send permission은 false이고 generation은 실제 후속 입력에서 1이 됐다. 이것을 실제 LLM의 정확성 또는 최종 응답 필수 사실 충족으로 채점하지 않는다.

## 계약과 이전 freeze 보존

V5 manifest 50개 파일과 live의 hash 대조에서 43개가 그대로다. 변경된 7개는 campaign의 v5→v6 이름, finalize의 QA05 단일 선택 집계, worker/runtime/B 및 두 전용 테스트다. 새 test_selection.py는 추가 파일이다.

| 보존 대상 | V5와 같은 SHA-256 |
| --- | --- |
| DP41 A | `b351057d152788bcb11b6e7d5f8451f6f3c9ce5df38096d569deb14955b256be` |
| finite 계약/schema/enum | `b0fff4b090a1128000e434d6150e3aa703468cadf5dea87c8b2c3955ec945ed2` |
| A/B/partial 모델 prompt | `be366f35926aebc73ba6ce9f1c9ba526366145518090b99d2aa28eee4ee8590c` |
| fixture builder | `c0bd936b7c1b49559313eaf3afecc09d15fc9df3c1164cba54faf3e67ee643d2` |
| 후보 입력 JSON | `8024baf9ecefdc797bce532268ccb3f789fe788cdfaa418765dc730587728bd4` |
| evaluator oracle JSON | `80461f5ef25f921f2d4e180c9adce29f3fc1fe7b22de22c5a8f70f5325a25690` |
| fixture manifest | `192231fa6a60b4054d4faa8bd0f96004c787a7dd8ca20ca7e321333f04431572` |
| QA v6 원문 | `ab76bfc813e1f5c8bb72894a705cfcc9495f7f92e021d566dfcbf8224d8d7d65` |
| common-workload JSON | `53f2b332fa7a1a197855765d7755465a096b9cecb8426068ae366e9cd9413102` |
| workload-manifest JSON | `26410d9883aa34494131dbb4606dfe862537bf40af246c2c295f2d4191b94e4c` |
| poc-inputs JSON | `734b4d862475344aad25b90fb974af3a49628222fc479ccaf4278ac07153a352` |

이전 freeze와 평가 도구 사본도 각 manifest의 모든 파일 hash와 일치했다. 아래는 frozen-source 검사이며, 큰 memory/raw 원장의 전체 hash 재계산을 했다는 뜻은 아니다.

| 세대 | frozen-source 파일 수 | 고정 digest | hash 불일치 |
| --- | --- | --- | --- |
| V1 | 42 | `08e015a4a6a8371df3f1c2d5de48f66fe21cfb107113205ff6cbdaf5e98dfe17` | 0 |
| V2 | 47 | `119b3344337d1f1fd3732df9e6cd3110cca38c574f73b6ce1c649a032227952c` | 0 |
| V3 | 49 | `f521d6bea196342dab6c0f66268134e9c33f25dfa86a0aac874afc160cb6a20a` | 0 |
| V4 | 50 | `c2dfc26fa6afbce8df4af3405913031ca440fefa150d6a0739c010b3ed4796a7` | 0 |
| V5 | 50 | `2143b39761bd0b74e8d9d973d8b59c1e8ba722ccc24ebafcc0d7e03d85cda4b0` | 0 |

V2 evaluator-tools, evaluator-tools-v2, evaluator-tools-v3는 각각 49개 파일이며 각 manifest 불일치 0이다. V3 evaluator-tools 49개, V4/V5 evaluator-tools 각각 50개도 각 manifest 불일치 0이다. 이전 trace와 검토 문서를 덮어쓰지 않았다.

## 후처리 선택 집계와 검수 한계

Root의 finalize.py(`b12b0df43cc142b2e16f29cdc81458c1ca24190ca9d835b9b4c67ed1f31310a2`) 및 새 test_selection.py(`73fe6f5209463dbf5659232f334aa2db9b085f18396c0d8870d95491873c0ebf`)도 읽었다. selected_change_records는 고정 baseline digest, reviewed result hash, actual changed-source/patch hash와 review 완료 flags를 확인하고 A/B×M01~M06의 한 job씩만 집계한다. 보존된 비선택 시도를 중복 집계하지 않으며 누락/중복/다른 baseline/변경된 source를 거절한다. 이 변경은 측정 후보의 실행을 바꾸지 않는다.

최종 판단 범위는 이번 연결 수정의 계약 정합성과 지정 출력에서의 실제 포트 회귀다. 실제 모델이 좋은 질문/정확한 Frame/올바른 consent를 만들거나 자유 응답의 모든 필수 사실을 전달한다는 증거가 아니다. 실제 모델 오류, 잘못된 빠른 첫 응답, 추가 질문 비용과 미완료는 새 고정 실행과 독립 판정에서 그대로 남겨야 한다. 새로운 V6 freeze와 실제 API preflight/공식 모집단 실행의 결과는 이 문서에 포함하지 않는다.
