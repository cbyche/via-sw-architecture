# PoC 구현 전 검토: 여섯 QA의 수집, 판정과 변경 실험

> **REVIEW_DRAFT / NOT_IMPLEMENTED / NOT_RUN — 2026-10-11.**
> 구현 전 사용자 검토 ①의 상세 부록이다. 이 문서의 작업 ID, 판정 예시와 비용 원장은 실행 결과가 아니다. 코드 작성, API 호출과 유료 측정을 시작하지 않았다.

현재 기준은 [ASR-QA-v6 정의](../03-02-quality-attribute-definitions.md), [P21 원천 입력](../03-02-common-workload.json), [관측 manifest](../03-02-workload-manifest.json), [계산 입력](../03-02-poc-inputs.json)과 [검산 보고](../03-02-poc-calculation-report.md)다. [사용자 지정 보고 방식](../../../../../benchmark/architecture/README.md#poc-결과-보고-요구사항--2026-10-11-사용자-지정)과 [단계별 수행 계획](../poc-implementation-plan.md)을 함께 적용한다. 과거 QA-09/19/29/39와 on-device Omni 조건을 이번 평가로 가져오지 않는다.

## 1. 준비 여부와 첫 세 사례의 한계

**여섯 QA의 평가 설계는 준비할 수 있지만 지금 실행 가능한 수집기와 후보는 없다.** 첫 단계의 S1-01 한정 조회, S3-02 결과 의존 메일, S4-01 부분 정정과 늦은 반환으로 먼저 raw에서 판정과 집계까지 연결할 경로를 검토한다. 이 세 사례는 45의 과거 관계 재사용, 44의 네 사건 집중 도착, 42의 정상 유효 질문 답변을 충분히 활성화하지 않는다. 세 사례의 성공으로 네 DP의 평가 준비 완료나 최종 QA 점수를 선언하지 않는다.

| QA | 지금 정해진 평가 단위 | 반드시 수집할 raw | 자동 판정 | judge와 수동 확인 | 세 사례로 검토 가능한 것 | 전체 수치에 추가로 필요한 것 |
| --- | --- | --- | --- | --- | --- | --- |
| 01 요청 처리 정확성 | 구성별 P21 21사례 × 5회 × C1~C6 = 630조건 | 입력과 원천 version, 읽은 범위, 모델 출력, 채택 의미, 실제 인계 payload, 명령/효과/상태, Text와 실제 재생 범위, checkpoint와 final tail | source/ID/revision/Task 대응, 관계 graph, 접수/효과/중복, 금지 접근과 필수 증거를 evaluator-only oracle로 검사 | 자유문 필수 사실/금지 주장 판정. 불확실이나 이견은 별도 평가자가 evidence와 원문을 보고 조정 | C1~C6 판정 경로, 잘못된 payload를 stub이 고치지 않는지, late/stale/duplicate 실패가 남는지 | P21 전체, 실제 Qwen의 의미 출력, 실제 음성 내용/전달. mock 모델은 실제 의미 정확성 근거가 아님 |
| 02 상호작용 응답시간 | 구성별 27반응 × 5회 = 135관측 | 실제 발화 종료 또는 source availability, 최초 실제 핵심 반응, 최초 유효 반응, Text 표시와 음성 onset, stop 마지막 sample, 오답/누락/정정, causal DAG | §2의 세 평균, 관측별 유효 종료와 cap, 일반/집중/local stop 보호 조건 | 의미 유효성은 QA01과 연결. audio 핵심 시작점의 annotation과 sample-clock mapping을 소수 수동 확인 | 세 지정 반응의 cap/부분집합 산출과 사건 연결. mock 음성은 proxy 시간만 검증 | 실제 Voice 입출력과 물리 출력 관측, S4-02/S5-04 stop, S5-03 OFF/재연결, 전체 135관측 |
| 03 VIA 요청 처리시간 | 구성별 19목표 × 5회 = 95관측 | 최초 입력 종료, 명세된 최종 전달, 원래 wall-clock, 외부 구간, DAG와 race 결과, 늦은 완료/미완료 | 같은 DAG의 외부 Agent 업무/사용자 대기/사용자 발화 node를 0으로 두고 재계산. §3의 두 평균과 cap | 완료의 사실/응답 의미 확인. 복잡한 경합에서 parent edge와 외부 구간 분류를 검토 | 직접 답변, 결과 의존 SEND tail, 정정 결과 tail을 서로 다른 완료 조건으로 연결 | S5-01/02의 사전 N/A, 확인 질문/답변/권한 유보 등 전체 19목표, 실제 모델/음성 경계 |
| 04 VIA 모델 사용 비용 | 구성별 P21 5활동의 USD/P21 평균 | 모든 call/context/usage/price, 초기 준비/배경 생산/미사용/재시도/취소/폐기/철회/drain/idle, 청구 정정 | call ID와 provider response ID 중복 제거, modality별 실제 usage × 단가, 활동/role/phase별 합 | 누락 usage와 청구 내역 대조, 분류되지 않은 별도 전사/helper 청구 확인 | mock call 원장과 합산 누락/이중 계산 검증. 실제 비용은 측정 불가 | 실제 provider usage와 요금 버전, 45 생산/갱신 전체 원장. 평가 judge와 외부 Agent 비용은 별도 |
| 05 VIA 변경 용이성 | 구성별 M01~M06 고유 수정 책임 수의 평균. 다섯 구성 총 30개 독립 job | 변경 전 baseline hash, 요구 delta, 실제 patch, U01~U18의 변경/유지 이유, 완료/회귀 evidence, 두 검토자의 판정 | 변경 job과 baseline 동일성, 회귀 실행 결과와 산출물 누락 검사 | 책임의 의미 대응과 중복 제거는 두 검토자가 확인. 파일 수로 자동 채점하지 않음 | 과제 요구와 30개 job 정의, registry 대응, 설계 patch의 검토 기준 | 완성된 다섯 후보 baseline, 각 독립 실제 변경과 회귀. 설계 patch는 DESIGN_CHANGE_REVIEW로 별도 |
| 06 로컬 VIA 메모리 사용량 | Mac의 phys_footprint와 resident size/RSS를 각각 보고 | 포함 process, 50ms sample 시각/두 값/오차/누락, allocation의 크기/수명/공유/copy, phase, restart generation | 회차별 process 합과 그 합의 관측 최대, phase/반복별 peak와 idle 변화 | SDK/OS metric 의미, 공유 중복과 짧은 peak 누락, process 포함 범위 확인 | Mac 계측 계약과 allocation provenance 설계. 세 사례의 실행 크기는 전체 P21 peak가 아님 | 실제 process와 memory collector, P21 준비/동시 부하/생산/철회/drain/idle. Windows commit 점수는 이 측정으로 산출하지 않음 |

P21이 실제로 실행되기 전에는 전체 630/135/95 분모를 적용한 값을 만들지 않는다. 세 사례 pilot은 실제 실행한 사례/반응/목표 수를 분모로 공개하고 **최종 등급 없음**으로 표시한다. 원지표, 보호 조건, 필수 위반과 evidence level은 분리한다. 낮은 시간/비용/메모리로 정확성 실패를 상쇄하지 않는다.

## 2. QA02의 세 평균과 구현 전 경계 확인

반응 ID마다 `start`, `first_substantive_response`, `first_valid_response`, `initial_response_valid`, `valid_response_observed`, `wrong_response_refs`, `correction_refs`, `cap_reason`, `missing_timestamp_reason`을 구별한다. 실제 응답은 해당 사건을 다루는 핵심 답변/동작이다. 접수 인사, filler, earcon, model 첫 packet와 outbox 등록은 제외한다. 오답이어도 어떤 목표/상태에 반응했는지 식별할 수 있는 실제 핵심 응답은 raw timing 진단에 남긴다.

Voice ON의 각 응답 종료는 실제 Text 표시와 해당 핵심 음성 onset 중 늦은 시각이다. Voice OFF는 Text만, local stop는 중단 대상의 마지막 실제 sample을 사용한다. Text와 음성의 내용이 다르면 두 원문/시각과 불일치를 보존한다. 음성 핵심 사실의 시작 sample을 생성 packet 시각으로 대신하지 않는다.

| 값 | 포함 집합과 분모 | 사용할 시간 | 누락 처리 |
| --- | --- | --- | --- |
| cap 적용 평균 | 전체 지정 반응 N. P21에서는 N=135 | 현재 QA의 유효 반응 시간 또는 실패 대체값 5초 | 해당 실패를 분모에 남기고 reason/오답/미전달을 표시 |
| 정답 반응만의 실측 평균 | 최초 핵심 반응이 유효 정답이며 timestamp가 있는 집합 V, 분모 \|V\|라는 사용자 검토 추천 | 최초 유효 반응 시간 | 최초 핵심 반응이 오답인 관측과 미전달을 제외. 뒤 정정 시간은 별도 진단 |
| 오답 포함 실측 평균 | 최초 실제 핵심 반응과 timestamp가 있는 집합 O, 분모 \|O\| | 최초 실제 핵심 반응 시간 | 미전달/시간 누락은 제외하며 0초나 5초를 넣지 않음 |

집합이 비면 `N/A (n=0)`다. `initial_wrong`, `eventually_valid`, `late_valid`, `never_valid`, `timestamp_missing`은 별도 flag다. 오답 뒤 정정은 `initial_wrong`과 `eventually_valid`가 동시에 참일 수 있으므로 이를 서로 배타적인 성공/실패 개수로 억지로 합치지 않는다. 분자, 분모, 포함/제외 관측 ID와 다섯 paired 값은 저장한다.

**선행 검토가 필요한 모호함:** QA 원문은 “잘못된 반응이나 미전달은 5초”라고 정의하지만 오답 후 유효 정정이 있는 관측의 주 지표와 정답 실측 평균 membership을 완전히 닫지 않았다. README는 최초 실제 반응과 최초 유효 반응을 모두 남기고 정답 평균에 유효 반응 시간을 사용하므로 오답 뒤 정정 관측을 정답 부분집합에 포함할 수 있는 해석이 남는다. 사용자의 “오답 제외” 요구를 명확히 하기 위해 아래 기준을 검토 ①의 추천으로 제시한다. 원본 QA/README를 이 부록에서 자동 개정하지 않는다.

- 첫 핵심 반응이 오답이면 뒤에 정정돼도 **주 지표는 5초 대체**, 최초 오답과 정정 시간을 모두 보존하는 방안을 제안한다. 잘못된 빠른 응답의 주 점수 이익을 막으면서 정정의 실제 지연은 진단에서 볼 수 있다.
- 최초 핵심 반응이 오답인 관측은 정답 반응만의 실측 평균에서 제외하고, 최초 오답 시간은 오답 포함 평균에 넣는다. 뒤 정정은 first_valid_response와 corrected 진단으로 남긴다.
- 최초 핵심 반응이 유효 정답이면 5초보다 늦어도 실제 시간을 사용한다. 예를 들어 6초이면 세 평균에 각각 실제 6초가 들어가고 해당 시간 보호 조건은 별도로 실패한다. `min(t,5)` clipping이나 새 공통 5초 응답 deadline을 만들지 않는다.
- 관측 종료까지 실제 핵심 반응이 미전달이면 주 지표 5초 대체, 두 실측 평균에서 제외한다. 수집기 결함의 무효 여부는 공통 INVALID 규칙으로 따로 처리한다. cap은 실행 중단 시각이 아니다.

예를 들어 첫 오답 0.2초, 정정 유효 반응 1.4초라면 위 추천에서는 주 지표 5초, 정답 실측에서 해당 관측 제외, 오답 포함 실측 0.2초와 별도 정정 지연1.4초다. 이것은 집계 규칙 예시이며 실제 실행 결과가 아니다. 정정됐다고 앞선 QA01 실패/금지 효과를 지우지 않는다. 반대로 다른 QA01 조건 실패를 관련 없는 모든 반응의 cap으로 자동 확대하지도 않는다.

평균 외에 일반 반응 100/105가 2초 이내, 집중 30개 모두 5초 이내, local stop 10개 모두 0.1초 이내, 미전달 0건의 현재 보호 조건을 함께 보고한다. cap 수와 late/never-valid 수를 공개해 낮은 평균으로 실패를 숨기지 않는다.

## 3. QA03의 두 평균과 완료 판정

각 목표의 완료 조건은 P21 oracle에서 가져온다. S1-01은 날짜 전달, S3-02는 현재 요약에 의존한 메일의 source-confirmed SENT와 전달, S4-01은 유효 정정의 ReportRevised@v2 전달이다. 접수 안내는 뒤의 두 목표의 final endpoint가 아니다. 거부/유보/취소 접수 미확인은 해당 사례가 그것을 최종 처리로 요구할 때만 완료다. 성공을 요구하는 작성 요청에서 임의 포기를 완료로 세지 않는다.

| 값 | 포함 집합과 분모 | 산출 |
| --- | --- | --- |
| cap 적용 평균 | 전체 지정 목표 N. P21에서는 N=95 | 유효 완료의 VIA 귀속 시간. 귀속 30초 안에 필수 endpoint가 없으면 30초 대체 |
| 완료 관측만의 실측 평균 | 관측 종료까지 명세상 완료되고 실제 시간을 재계산할 evidence가 있는 집합 C | 실제 VIA 귀속 시간 평균. 30초 뒤 완료도 포함하되 late_completed 표기 |

S5-01/02의 N/A는 모집단을 정할 때 제외한 것이다. 다른 목표의 실패/미완료를 뒤에 N/A로 바꾸지 않는다. `within_cap_completed`, `late_completed`, `never_completed`, `completion_timestamp_missing`과 포함/제외 ID를 남긴다. 완료 표본이 없으면 `N/A (n=0)`다. 실행 관측 상한과 귀속 30초 cap은 서로 다르다.

원래 wall-clock과 causal DAG는 보존한다. 각 node에는 owner, 시작/끝, kind, parents, request/effect/delivery ref와 evidence를 붙인다. EXTERNAL_AGENT_WORK, EXTERNAL_USER_WAIT, EXTERNAL_USER_SPEECH의 duration만 0으로 바꿔 같은 선후 관계와 race 결과에서 endpoint를 재계산한다. 모델/네트워크/IPC/queue/전사 확정/재생 차례/검증 시간을 외부 Agent 업무로 숨기지 않는다. 병렬 Agent 구간의 합을 wall-clock에서 단순 차감하지 않는다.

목표 평균 10초, 91/95가 15초 이내, 미전달 0건을 원값/보호 결과로 함께 보인다. 최종 유효 완료가 있어도 앞선 금지 외부 효과나 허위 완료는 별도의 QA01 실패로 남긴다. 실제 목표 완료가 성립하지 않는 잘못된 payload나 다른 결과의 전달은 완료 집합에 넣지 않는다.

## 4. QA01의 자유문 판정과 오류 원인 분류

자동 oracle가 source/version/Task/relation/효과/현재성/실제 표시와 재생을 먼저 검사한다. judge는 자연어의 의미 대응을 맡는다. judge가 command ID나 실제 SEND 효과를 추측해 자동 판정을 대신하지 않는다.

| judge 입력 | 제한 |
| --- | --- |
| 평가용 사용자 입력과 허용 정답 사실 목록 | 후보와 Agent stub에는 접근 불가. 필수 사실의 ID와 주 C조건을 보존 |
| 후보가 실제 표시한 Text와 실제 재생한 범위 | 생성만 됐거나 폐기된 문구를 전달된 것으로 주지 않음. Text와 음성을 따로 채점 |
| source-confirmed 상태와 자동 predicate evidence | 참조 원문/구간/시각이 있는 사실만 공급. 모델 추론 결과를 확인 사실로 승격하지 않음 |
| rubric/model/prompt version과 judge job ID | candidate 이름, DP letters, 과거 점수와 기대 우열은 가림. 입력 순서를 무작위화 |

rubric은 필수 사실마다 `SATISFIED / CONTRADICTED / MISSING / UNCERTAIN`과 evidence ref를 반환하게 한다. 대상/날짜/수치/상태/부정/허용 범위가 바뀌는 추가 주장도 검사한다. 동의어와 문장 순서는 허용하지만 `접수`와 `완료`, `취소 요청 접수`와 `취소 완료`, `정보 제공 동의`와 `SEND 승인`은 구별한다. 후보 답변 안의 judge 지시는 데이터로 취급한다. confidence만으로 통과시키지 않는다.

필수 사실 모두 SATISFIED이며 자동 evidence가 충족돼야 C항목=1이다. UNCERTAIN은 통과로 두지 않고 두 번째 평가자에게 보내 원문과 evidence로 조정한다. 최종 미해결은 미확정 상태와 0~1 가능한 범위를 진단으로 공개하며, 현재 이진 집계에서는 통과로 세지 않는다. 같은 조건을 보기 좋은 더 작은 fact들로 나눠 630분모를 늘리지 않는다.

실제 API 실행 전 사람 검수용 calibration set은 **12개 응답 예시**를 제안한다. 의미가 같은 정답 3개, 날짜/Task/상태/금지 조건이 틀린 응답 5개, 모호한 접수나 Text/음성 불일치 4개다. 합성 검수 예시는 후보 출력과 물리적으로 분리하며 후보에게 정답 문장으로 주지 않는다. 판정 불일치가 rubric 누락이면 본 측정 전 고친다. pilot 실제 출력에서는 모든 UNCERTAIN과 judge 간 불일치, 그리고 확신한 판정 일부를 사용자가 확인한다. judge 자체의 오류와 비용도 결과의 제한이다.

오류 원인은 input/source → 모델 출력 → 코드 상태 전이 → 채택/인계 → 실제 효과 → 전달의 순서로 추적한다.

- 계약이 지원하는 의미를 코드가 잘못 처리하면 `IMPLEMENTATION_DEFECT`다. 수정 전 실패를 보존하고 새 source version으로 영향받는 A/B를 다시 수행한다.
- 필요한 관계가 유한 schema/Engine에서 표현되지 않으면 `REPRESENTATION_LIMIT`다. 질문으로 단순화한 손실과 추가 비용을 기록한다.
- frame/tool/proposal가 원문을 잘못 해석하면 `MODEL_OUTPUT_ERROR`다. 한 번의 오류로 구조의 필연적 한계를 단정하지 않는다.
- source 전달, 시각 또는 judge 오류는 `HARNESS_OR_JUDGEMENT_DEFECT`로 증거를 남긴다. 후보 timeout/미지원/필수 trace 누락을 수집기 결함으로 옮기지 않는다.
- 인과를 가릴 증거가 없으면 `UNRESOLVED_CAUSE`다. 오류를 A/B 이름으로 할당하지 않는다.

## 5. QA04의 전체 호출 원장과 비용 준비

호출마다 activity/case/phase, call ID, parent call/attempt, role, foreground/background, provider/model/region/revision/mode, schema/prompt version, Context artifact/hash와 읽은 source vector를 남긴다. enqueue/send/first/terminal/cancel/requested-discard 시각, provider response ID, 실제 input/output/cached token, audio/Text/전사별 usage, retry 이유, 취소 뒤 늦은 청구, rate/price version과 usage completeness도 기록한다. API key와 authorization header는 저장하지 않는다. 내부 사고 기록의 제공을 요구하지 않는다.

Context가 길거나 재사용되면 실제 직렬화해 보낸 Context와 modality별 청구 이력을 연결한다. Text/audio cache를 자기 추정으로 할인하지 않는다. usage가 없으면 비용 0이 아니라 `USAGE_MISSING`으로 남기고 invoice/usage export로 조정한다. 같은 billed response를 call log, response.done과 invoice에서 세 번 합산하지 않는다. 아직 청구가 도착하지 않은 작업이 있으면 비용 결과도 잠정 상태다.

| 비용 원장 | QA04 포함 여부 |
| --- | --- |
| VIA 현재 의미, 부분 해석, 응답 구성, 입력 전사/음성 생성 | 포함 |
| 45A 실제 summary/cache 생산과 45B 초기/갱신/backfill/미사용 생산 | 포함 |
| VIA 실패/재시도/취소 불가 폐기/배경/drain/idle 청구 | 포함 |
| 외부 Agent 내부 reasoning/tool/모델 | 별도. stub은 모델 비용이 없더라도 외부 역할 비용을 VIA 절감으로 주장하지 않음 |
| 평가 LLM-as-a-judge | 평가비로 별도. 전체 지출 한도에는 포함하되 후보 QA04에는 제외 |
| 평가 runner와 데이터 검수 비용 | 별도. 기능 수행하는 VIA helper로 위장해 제외하지 않음 |

현재 QA 원장의 62의미 호출/2,000입력/200출력, 음성 입력50초/출력179초, 누적 Text10k/2k는 **계산 예산**이다. 실행을 그 횟수로 고정하지 않는다. reference 소계 $0.351840/P21을 다섯 구성 × 5활동에 단순 곱하면 $8.796000이지만, 이는 후보별 추가 호출/재시도/audio 이력/초기 생산/judge/변경 회귀가 빠진 reference 산술이다. 실제 전체 예상 지출이나 사용자가 승인한 한도가 아니다.

유료 단계 전에는 pilot별 call-plan과 실측 usage로 모델/음성/전사/judge/재실행의 예상 범위와 총 비용 한도를 제시한다. 지금 API 계정과 provider 선택 reference가 있다는 이유로 유료 준비 실행을 시작하지 않는다. 사용자 비용 상한은 미정이다.

## 6. QA06의 Mac 수집과 공유 메모리 보고

Mac에서는 `phys_footprint_bytes`와 `resident_size_bytes`를 같은 sampling 회차에 수집한다. RSS는 resident size 계열 이름이며 세 번째 지표가 아니다. phys_footprint는 OS 귀속 회계이고 압축 메모리 등도 포함한다. resident size는 현재 resident page 크기다. 두 값을 기존 Windows PrivateUsage/고유 공유 anonymous commit으로 환산하지 않는다. OS 버전, Mac hardware와 실제 실행 environment를 기록한다.

process identity, restart generation, 포함/제외 이유와 회차 내 scan skew를 남긴다. VIA helper, 42B의 대화/업무 process, DB/queue/직렬화 buffer와 생산 job은 포함한다. cloud weights/KV, 외부 Agent와 평가 runner/judge는 제외한다. 하나의 process에 후보와 runner가 섞여 있으면 실제 VIA 부분을 OS 지표에서 분리할 수 없으므로 계측 구조를 먼저 분리하거나 그 제한을 공개한다.

각 회차의 process 합을 구한 뒤 그 시계열의 관측 최대를 취한다. process별 다른 시각 peak를 더하지 않는다. 누락 process를 0으로 채우지 않는다. 50ms sampling과 allocation 사건을 함께 남겨 짧은 peak의 미관측 가능성을 표시한다. RSS 합에 공유 page 중복이 있을 수 있으므로 `process_sum_rss`를 고유 물리 메모리라고 부르지 않는다. phys_footprint process 합도 고유 system RAM 합으로 단정하지 않는다.

후보가 관리한 allocation은 ID, 크기, 수명, 보유 process, shared mapping와 copy parent를 별도 원장에 남긴다. 알려진 공유와 확인하지 못한 공유를 구별한다. allocation 원장의 byte를 OS process 합에 다시 더하지 않는다. 이를 임의 padding을 할당해 1,792MiB 예산을 재현하는 도구로 사용하지 않는다.

P21의 2,000turn/200완료 Task/30source/20MiB 원천 준비, 관련 활성 Task 상한10/cloud call 상한4/capture 상한15frame, source4갱신/maintenance2회/철회/drain30초/idle10초를 포함한다. 상한만큼 job/frame을 만들도록 강제하지 않는다. 반복별/phase별 두 metric peak와 idle 시작/끝/최대값, sampling completeness를 공개한다. Windows의 4,096MiB 목표와 0~6점, working-set 보호 조건은 Mac으로 검증하지 않는다. Mac 점수가 필요하면 본 측정 전 근거와 경계를 따로 승인해야 한다.

## 7. QA05 책임 수를 세는 규칙

채점 registry는 [현재 QA §6.2](../03-02-quality-attribute-definitions.md#62-공통-변경-책임-registry)의 U01~U18이다. 같은 책임을 같은 ID에 대응하고 구현 파일, Module 또는 process 개수로 분할하지 않는다.

| ID | 공통 책임 |
| --- | --- |
| U01 | 자연어 의미 생산 prompt/해석 규칙 |
| U02 | 유한 표현의 dispatch/항목 해결/코드 결합 |
| U03 | 의미/frame/조건 표현의 공통 검증 계약 |
| U04 | 요청 채택/질문 focus/입력 hold 규칙 |
| U05 | 입력/전달 시각과 실제 재생 제어 |
| U06 | 응답 내용/게시/실제 전달 원장 |
| U07 | Task/Execution/업무 명령/질문 현재성 |
| U08 | 중앙 continuation 또는 Stage/window 흐름 |
| U09 | source adapter와 정규 원본 port |
| U10 | 요청별 원본 조합/검색/cache 의미 |
| U11 | 공통 과거 관계 생산/coverage/무효화 |
| U12 | 게시 관계 조회/부분 범위 보완 |
| U13 | 대화와 업무의 명령/접수/반영 계약 |
| U14 | 저장 schema/이행/복원 |
| U15 | cloud provider wire/usage/중단 adapter |
| U16 | 외부 Agent protocol/capability adapter |
| U17 | 허용/동의/삭제 정책의 판정 |
| U18 | 삭제/정정의 소비자 파생물 처리 계약 |

U18은 여러 owner 사이의 삭제 전달/완료 계약 자체가 바뀔 때만 별도로 센다. U10~12 내부의 동일 무효화 수정을 U18로 중복 세지 않는다. U14도 producer의 private cache 필드와 중복 세지 않고 실제 영속 schema 이행/복원이 별도로 바뀔 때만 센다. 같은 책임의 prompt와 코드가 같이 수정돼도 고유 ID 한 번이다. 테스트/문서 수정, 동일 코드 generator 재출력은 추가 책임 수가 아니며 검증 부담으로 별도 기록한다.

모든 job에서 U01~U18 각각 `CHANGED / UNCHANGED / NOT_PRESENT`와 이유를 채운다. NOT_PRESENT는 기능이 후보에 없어 평가 요구를 면제한다는 뜻이 아니다. 과제가 해당 책임을 새로 요구하면 신설/변경으로 센다. 새 책임이 필요하면 양안 공통 registry version을 올려 합의한 뒤 적용한다.

`u_job = |실제 변경 고유 U-ID 집합|`, 구성별 값은 해당 구성의 M01~M06 `u_job` 합/6이다. 평균/최대/집합과 회귀 결과를 보고한다. 불완전 patch는 0변경 성공으로 채점하지 않는다. 같은 baseline의 여섯 변경을 순차 누적하지 않으며 job마다 원래 snapshot의 별도 disposable 작업 공간에서 시작한다.

### 7.1 변경 전 capability audit가 필요한 이유

M04의 실패 후 continuation은 현재 Frame v1의 `AFTER/USE_RESULT/INDEPENDENT/WHEN`에 별도 실패 continuation opcode가 없다는 점을 확인했다. 하지만 완성 baseline이 명시 terminal-outcome 조건으로 이미 동일 기능을 지원한다면 추가 patch 없이 완료될 수 있다. 성공/실패/발송 금지 회귀 evidence를 남기면 **정당한 0변경**이다. 반대로 첫 세 사례에 아직 구현하지 않은 기본 순서/의존 실행을 채우는 작업은 baseline 완성 작업이며 M04 변경량으로 세지 않는다. 평가 변경을 부풀리려 baseline을 의도적으로 좁히지 않는다.

M06은 더 주의가 필요하다. [공통 계약](../04-40-common-execution-contract.md), [DP41](../04-41-request-resolution-control.md), [DP45](../04-45-memory-and-context.md)와 [DP42](../04-42-lifecycle-ownership.md)는 이미 철회된 source/cache/파생/모델 Context의 신규 사용 차단과 현재성 검사를 요구한다. 이를 baseline에서 빼고 M06의 신규 기능으로 다시 넣으면 공정한 비교가 아니다. 구현 전/후 **실제 추가 delta**, 예를 들어 처리 receipt의 범위/완료 집합이 기존 계약보다 확장되는지 audit한다. baseline이 전체 M06을 이미 충족하면 0변경과 검증 결과를 허용한다. 필수 baseline 기능의 누락은 defect 수정으로 분리한다. 실제 신규 delta가 없는데 구조 차이를 내기 위해 추가 책임을 만든다면 과제를 수정해야 하므로 QA 계약 검토에서 닫는다. 원본 M06을 이 문서에서 자동 변경하지 않는다.

M01~03은 adapter 경계로 끝날 수 있는 대조군이다. provider envelope 교체가 U15 하나, Agent protocol 교체가 U16 하나, source 교체가 U09 하나로 양안 동점이어도 정상 결과다. M05는 기존 ID/revision에 추가되는 Task owner 발급/검사 token과 실제 제시 token의 반환을 확인하므로 단순 schema field 추가와 구별한다.

### 7.2 공통 여섯 과제의 상세 delta와 완료

| 과제 | 실제 변경 입력과 보존하는 계약 | 완료 조건 | 필수 회귀 | 산출물 |
| --- | --- | --- | --- | --- |
| M01 | 동등 기능 의미 provider의 request/response/usage wire envelope를 다른 형식으로 교체. 상위 의미 input/tool/frame/text/취소/usage port는 유지. provider stub 두 형식으로 회귀 가능하며 실제 두 번째 유료 모델 정확성 비교를 요구하지 않음 | 새 envelope의 입력/반환/stream종료/취소/usage를 정규 port로 대응 | P21 C1~C6, schema 거절, stale/취소/usage 누락 처리. 역할 model capability 의미 변경 없음 | wire before/after, adapter patch, U ledger, protocol/회귀 evidence |
| M02 | 메일 Agent submit/cancel wire envelope와 receipt format 교체. 상위 command identity/접수/미확인/완료 의미는 유지 | 같은 command와 payload의 접수/취소/조회 대응 | duplicate command/event, 역순 receipt, UNKNOWN, S3-02/S4-03/S5-02와 P21 정답 | protocol before/after, Gateway patch, U ledger, 실제 받은 payload/receipt/effect |
| M03 | 일정 source field 이름과 timezone envelope 변경. 정규 interval/timezone/version/permission port는 유지 | 새 raw envelope를 같은 정규 시간/원본/현재성으로 제공 | S3-03 최신 BUSY, 대체16~17시, 생성0, cache stale/접근 거부와 P21 | source samples, adapter patch, U ledger, normalized read/회귀 evidence |
| M04 | 보고서 실패에도 메일 초안 작성 계속이라는 VIA 실패 후 실행 관계 추가. 성공/실패 두 분기 모두 검사, SEND는 금지. 어떤 자료를 초안에 쓰는지 명시하고 보고서 성공 결과가 없는 경우를 정답으로 채워 주지 않음 | report FAILED 이후 초안 목표를 한 번 실제 접수/수행하고 조건/근거를 보존. 성공 분기 역시 의도대로 실행 | 기존 AFTER/USE_RESULT, 실패 중복 event, late success, 무관 Task 유지, SEND0와 P21 | capability audit, relation before/after, parser/schema/engine/continuation patch 중 실제 변경, U ledger, 양분기 effect trace |
| M05 | 기존 question ID/revision에 허용 답변 범위 token 추가. Task owner 발급/검사, 대화가 실제 제시 token을 보관하고 답변 때 반환 | 실제 제시된 현재 token 답변 한 번 접수. stale/다른 Task token은 거절하며 복원 뒤 같은 대응 | S6-04 정상 답변, S6-02 철회/미제시, 중복/역순, 재시작, 입력 hold와 P21 | token 계약 before/after, producer/consumer/storage patch, U ledger, receipt/ANSWER/reject/복원 trace |
| M06 | 특정 문서 제공 동의 철회를 관련 원문/요약/관계/model Context 신규 사용 차단과 처리 receipt까지 적용. §7.1에서 실제 baseline 대비 delta 먼저 확정 | 원본/파생/대기/늦은 생산 사용 차단, 미완료 처리 receipt를 완료로 위장하지 않음, 다른 허용 자료 유지 | 45A cache와 45B 생산/게시/Reader, 미사용 생산/권한epoch/late 결과, 명시 선호 등록 원본 oracle, 철회/삭제와 P21 | capability audit, consent/receipt before/after, 실제 신규 patch 또는0변경 근거, U ledger, 소비차단/처리결과/회귀 trace |

M04에서 실패 뒤 초안에 사용할 공통 payload는 과제 fixture로 지정하되 후보에게 성공 보고서 산출물을 선주입하지 않는다. 입력 원문과 허용 source로 초안 목표를 위임할 수 있고 원인/실패 상태를 보존해야 한다. 도메인 초안 작성의 품질은 Agent 범위다. 변경 과제는 요구 oracle를 후보와 분리한다.

### 7.3 다섯 구성과 30개 독립 change job

구성 ID는 이 평가 job을 연결하는 표식이며 대안 이름이나 최종 선택을 바꾸지 않는다. `C0=41A+44A+45A+42A`, `C41=41B+44A+45A+42A`, `C44=41A+44B+45A+42A`, `C45=41A+44A+45B+42A`, `C42=41A+44A+45A+42B`다.

각 행의 변경/완료/회귀는 §7.2의 해당 M과제가 원본이다. 구성별 관심 경로는 예상 책임 수나 강제 patch가 아니다. 실제 diff와 evidence가 변경 집합을 결정한다.

| Job ID | 시작 baseline | 실제 변경 과제 | 이 구성에서 확인할 경로 | 완료/회귀/산출물 계약 |
| --- | --- | --- | --- | --- |
| CH-C0-M01 | C0 snapshot | 의미 provider wire/usage 교체 | 강한 A의 Model Access 경계 | M01 전체 |
| CH-C0-M02 | C0 snapshot | 메일 Agent wire/receipt 교체 | 공통 Agent Gateway 정규화 | M02 전체 |
| CH-C0-M03 | C0 snapshot | 일정 source/timezone 교체 | 원본 port의 시간/version 유지 | M03 전체 |
| CH-C0-M04 | C0 snapshot | 실패 뒤 초안 실행 관계 | 모델 제안과 중앙 continuation의 실제 지원 audit | M04 전체 |
| CH-C0-M05 | C0 snapshot | 질문 답변 범위 token | 모듈형 공동 확정과 실제 제시 token 반환 | M05 전체 |
| CH-C0-M06 | C0 snapshot | 철회 파생 범위/receipt | 원본 조합/cache/Context fence와 기존 지원 audit | M06 전체 |
| CH-C41-M01 | C41 snapshot | 의미 provider wire/usage 교체 | 초기 frame/부분 반환의 상위 계약 유지 | M01 전체 |
| CH-C41-M02 | C41 snapshot | 메일 Agent wire/receipt 교체 | frame를 Gateway wire에 결합하지 않는지 | M02 전체 |
| CH-C41-M03 | C41 snapshot | 일정 source/timezone 교체 | Engine은 같은 정규 evidence 소비 | M03 전체 |
| CH-C41-M04 | C41 snapshot | 실패 뒤 초안 실행 관계 | 유한 schema/코드 결합과 실제 continuation의 확장 또는0변경 | M04 전체 |
| CH-C41-M05 | C41 snapshot | 질문 답변 범위 token | frame/부분 답변과 actual-presentation token 연결 | M05 전체 |
| CH-C41-M06 | C41 snapshot | 철회 파생 범위/receipt | frame 항목/후보/부분 모델 Context 무효화 | M06 전체 |
| CH-C44-M01 | C44 snapshot | 의미 provider wire/usage 교체 | Stage가 provider wire를 직접 소비하는지 확인 | M01 전체 |
| CH-C44-M02 | C44 snapshot | 메일 Agent wire/receipt 교체 | typed notice의 정규 상위 의미 유지 | M02 전체 |
| CH-C44-M03 | C44 snapshot | 일정 source/timezone 교체 | Stage가 같은 source port 소비 | M03 전체 |
| CH-C44-M04 | C44 snapshot | 실패 뒤 초안 실행 관계 | Stage 대기/실패 입력과 Publication Join | M04 전체 |
| CH-C44-M05 | C44 snapshot | 질문 답변 범위 token | Notice/Publication Window와 실제 제시의 token 범위 | M05 전체 |
| CH-C44-M06 | C44 snapshot | 철회 파생 범위/receipt | Window의 대기 후보/late 준비 결과와 use fence | M06 전체 |
| CH-C45-M01 | C45 snapshot | 의미 provider wire/usage 교체 | 배경 생산도 같은 Model Access 경계 소비 | M01 전체 |
| CH-C45-M02 | C45 snapshot | 메일 Agent wire/receipt 교체 | 정규 owner event와 relation 생산 영향 | M02 전체 |
| CH-C45-M03 | C45 snapshot | 일정 source/timezone 교체 | 생산 source vector와 Reader 정규 port 유지 | M03 전체 |
| CH-C45-M04 | C45 snapshot | 실패 뒤 초안 실행 관계 | 과거 관계 생산과 현재 실패 continuation 혼동 금지 | M04 전체 |
| CH-C45-M05 | C45 snapshot | 질문 답변 범위 token | 파생 게시가 Task token의 현재 권위 대체하지 않음 | M05 전체 |
| CH-C45-M06 | C45 snapshot | 철회 파생 범위/receipt | Publisher/Repository/Coverage/Reader/late publication | M06 전체 |
| CH-C42-M01 | C42 snapshot | 의미 provider wire/usage 교체 | 서비스가 동일 provider 정규 adapter 계약 유지 | M01 전체 |
| CH-C42-M02 | C42 snapshot | 메일 Agent wire/receipt 교체 | 업무 authority/Gateway 정규화와 대화 receipt | M02 전체 |
| CH-C42-M03 | C42 snapshot | 일정 source/timezone 교체 | 독립 owner 간 동일 시간/version 의미 | M03 전체 |
| CH-C42-M04 | C42 snapshot | 실패 뒤 초안 실행 관계 | 대화 소유 의존 해제와 업무 명령/접수/반영 | M04 전체 |
| CH-C42-M05 | C42 snapshot | 질문 답변 범위 token | 업무 token 생산/검사, 대화 보관/반환, IPC/복원 | M05 전체 |
| CH-C42-M06 | C42 snapshot | 철회 파생 범위/receipt | owner별 차단 적용/ACK/처리 receipt의 실제 범위 | M06 전체 |

C0의 여섯 job은 한 번의 증거 자산으로 네 비교에서 참조하며 독립 표본으로 복제하지 않는다. 각 job은 `job_id`, `baseline_source_hash`, `registry_version`, `requirement_delta_hash`, `patch_ref`, `changed_unit_set`, U별 이유/evidence, `completion_result`, `regression_result`, `reviewer_findings`, `evidence_level`을 갖는다. job source를 다른 구성으로 재사용하면 source_execution_key와 실제 동일 범위를 표시한다.

최종 QA05는 실제 독립 patch와 회귀가 끝난 뒤 산출한다. 변경 책임 수의 목표/0~6점은 원본 QA를 따른다. 두 검토자가 같은 registry와 patch를 독립 검토하고 실제 changed set의 이견을 근거로 조정한다. 자동 diff가 특정 파일에 대응된 U-ID를 제안할 수는 있지만 그 제안만으로 공식 개수를 확정하지 않는다.

## 8. 45 cache와 생산을 공평하게 관측할 준비

cache는 구조를 선택한 뒤 적용할 수 있는 tactic이다. 45 비교의 본체는 **요청 경로가 owner 원본을 조합하고 누락을 해결하는 A**와 **공통 관계를 실제 생산/검증/게시한 뒤 정상 조회가 그 게시를 소비하는 B**다. 이름이 cache인지 repository인지로 구별하지 않는다.

주 비교에서 강한 A의 유효 관계 cache와 부분 갱신을 금지하지 않는다. 두 안의 원천 준비/시간창/변경/허용을 같게 주고 local 파생 상태는 비운다. A의 cache와 B의 관계는 실제 후보 경로로 생산하며 생산 지연/청구/할당을 P21에 포함한다. 구조 인과 설명용 cache-off 진단을 추가하면 본 비교와 분리해 같은 source와 정확성 계약으로 기록한다. cache-off 값만으로 A/B 승자를 정하지 않는다.

S2-02 초기 과거 결합 → S2-04 같은 자료 재사용 → S2-05 새 자료 첫 사용을 묶어 보고한다. 준비 중 B의 미완료 생산, NOT_COVERED backfill/재조회, PARTIAL coverage/지원 한계, 원본 확인, B01~B04 미사용 갱신, maintenance와 마지막 철회까지 원장에 포함한다. B+의 원본 조합 우회는 별도 혼합 경로다. 한 번의 hit 지연만으로 초기 생산을 공짜로 만들거나 B에 완성 정답 관계를 미리 제공하지 않는다.

## 9. 사용자 검토 ①에서 닫을 항목과 다음 완료 기준

이 부록이 정한 것은 관측/판정/변경 작업의 설계다. 실행 준비 완료가 아니다. 코드 작성 전에 다음 항목을 검토 묶음에서 확인한다.

1. 첫 세 사례의 final endpoint, 핵심 Text/음성 사실과 Agent stub의 payload 보존 범위.
2. QA02의 선행 오답 뒤 정정에 대한 주 지표와 정답 부분집합 규칙. 유효 정답 반응의 실제 시간이5초보다 늦어도 clipping하지 않으며 진단 raw/세 평균은 모두 보존.
3. M04/M06의 baseline capability audit와 실제 requirement delta. baseline 결함 수정, 정당한0변경과 신규 기능 변경의 분리.
4. QA05 U-ID 대응/중복 제거와 두 검토자, 독립 baseline의 변경 job 운영.
5. 모델 mock/실모델, 출력 sink proxy/실제 물리 음성 경계의 증거 수준 구분. Mac 두 지표의 원값 보고와 Windows 점수 미환산.

세 사례의 mock 실행 단계 완료는 입력부터 후보 출력/효과/실제 sink receipt/판정/집계까지 raw로 재계산할 수 있고, 오답/중복/stale/미완료가 그대로 실패로 남는 것이다. 실제 의미 정확성/음성 지연/청구는 다음 유료 pilot 승인과 실제 실행 후에만 보고한다. 전체 P21과 30변경 완료 전에는 여섯 QA의 최종 점수와 네 DP의 우열을 주장하지 않는다.
