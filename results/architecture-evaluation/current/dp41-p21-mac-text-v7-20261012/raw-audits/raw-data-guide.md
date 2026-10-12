# V7 원시 데이터 재사용 안내

이 안내는 완료된 `DP41-P21-MAC-TEXT-v7`의 원시 기록을 다른 분석에 재사용하기 위한 경로와 필드 설명이다. 본 실행 210개, 원판정 210개, 승인 override가 있는 120개 trial, 선택된 QA05 변경 실험 12개를 연결해 최종 여섯 QA를 집계했다. 정확성 1,260조건, 반응 270관측, 사이클 190관측을 유지하며 판정 오류와 불확실성의 원형 기록 및 근거 disposition도 보존했다. 독립 원시 재계산의 174,750개 검사는 모두 통과했다. 이 검수는 산술과 provenance의 일치 검증이며, 의미 정답에 대한 추가 독립 인간 판정은 아니다.

캠페인 루트 `C`는 다음 절대 경로다. 아래 경로는 모두 `C` 기준이다.

```text
/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/dp41-p21-mac-text-v7-20261012
```

[manifest.json](../manifest.json)은 모집단, 모델/단가, OS, 원시 입력 및 측정 범위를 연결한다. 동결 소스 digest는 `44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce`이며 53개 파일을 포함한다. `frozen-source/`가 실제 실행한 코드이고 `evaluator-tools/`는 첫 판정/변경/집계 도구 사본이다. `evaluation-tool-manifest.json`과 `evaluation-tools-current.json`은 그 첫 세대 provenance를 그대로 유지한다. 재사용 때 활성 저장소 코드나 current라는 파일명만으로 실제 재개 도구를 판단하지 않는다.

실제 판정 재개는 `evaluator-tools-v2/`와 `evaluation-tool-manifest-v2.json`으로 별도 고정됐다. V2 digest는 `2718bbb3dcc2220d23d3ddeca63202543d621cd3789d8da8daba0bde52774fc3`이며 변경은 judge의 publication lookup 두 곳에서 publication_id 없는 실제 LOCAL_STOP receipt를 안전하게 건너뛰는 보정이다. 프롬프트, 정답/경계, 후보와 본 worker 실행 53파일은 바뀌지 않았다. `evaluation-control/stages.jsonl`은 실제 재개/도구 단계의 원장이다. 최종 normalize/metrics 집계는 동일한 원래 `evaluator-tools/`로 수행한다. override schema 검증의 실제 도구도 해당 실행 단계와 manifest를 확인한다.

추가 재시도는 `evaluator-tools-v3/`, `evaluation-tool-manifest-v3.json`으로 고정됐다. V3 digest는 `ac7c59a945d54fdeafeeceb52527a400fc6ca18b98904d7c3ee929eb7821e583`이다. V2 보정에 더해, schema retry feedback에서 잘못 인용한 source/publication/Task/causal-node ID와 실제 supplied enclosing record의 evidence ID를 구별하고 허용 ID를 안내한다. SYSTEM, 첫 pass의 evaluation_view, 모델, 정답/경계와 의미 flags는 같고 자동 의미 판정 변경은 없다. **재시도 feedback Context는 달라졌으므로** V2와 V3의 attempt를 도구 hash/실제 Context artifact로 구별한다. API0 회귀의 `evaluation-control/evidence-feedback-repair-tests.json`은 도구 검증이며 후보 성능 반복이 아니다. 원래/V2/V3의 manifest file_hashes와 changes를 모두 보관한다.

publication ID나 정확한 core quote를 반복해서 잘못 인용한 세 판정의 재개는 `evaluator-tools-v4/`, `evaluation-tool-manifest-v4.json`으로 고정됐다. V4 digest는 `5403b8b40ad049486a7b4b06ebfd7fc7d11f90d3d8cd9d65959b945c92d83d6d`이다. schema retry feedback은 오류가 난 publication/quote 필드를 가리키고 실제 publication ID 및 해당 publication의 실제 표시 Text 또는 실제 방출된 Voice text를 안내한다. 생성됐지만 방출되지 않은 Voice 뒷문장은 허용 인용으로 제공하지 않는다. 첫 SYSTEM/evaluation_view, 정답/경계와 validator는 V3와 같고, 코드가 endpoint나 passed/wrong_first/uncertain을 대신 선택하거나 고치지 않는다. V4의 재시도 Context는 별도 원시 artifact로 추적한다. `evaluation-control/exact-emission-feedback-repair-tests.json`의 250개 API0 검사와 `publication-quote-feedback-actual-raw-probes.json`의 실제 누락 세 판정 오류 검증은 평가 도구의 증거 형식 검증이며 후보 의미 정확성 관측을 추가하지 않는다. V3와 이전 세대 원형 파일은 그대로 보존한다.

## 1. 모집단과 제외 자료

| 범위 | 자료 | 재사용 시 의미 |
| --- | --- | --- |
| V7 본 실행 | `workers/A1`~`A5`, `workers/B1`~`B5` | 21사례 × 5독립 활동 × 2방안 = 210 trials. A는 41A+44A+45A+42A, B는 41B+44A+45A+42A |
| 정확성 | 각 trial의 C1~C6 | 방안당 630조건, 합계 **1,260조건**. checkpoint와 final-tail 조건을 해당 C 안에서 검사하며 새 조건으로 중복 추가하지 않음 |
| 반응시간 | trial의 `reaction_blueprint` | 방안당 135관측, 합계 **270관측**. 사례 수나 조건 수를 시간 분모로 복사하지 않음 |
| 전체 사이클 | trial의 `cycle_blueprint`가 있는 19사례 | 방안당 95관측, 합계 **190관측**. Agent 사건 단독 사례의 null cycle은 모집단에 없음 |
| 변경 용이성 | `qa05-selection-manifest.json`의 `selected` | A/B × M01~M06 = **12 jobs**만 선택. 보존한 원래 M04와 대안을 glob으로 모두 합치지 않음 |
| V7 readiness 준비 | `preflight-readiness.json`의 `actual_preflight[].raw_directory` | 별도 preparation 디렉터리의 실제 Qwen 호출. V7 본 P21 정확성/시간/비용 분모에 더하지 않음 |
| 이전 V1~V6 | 형제 `dp41-p21-mac-text-v1`~`v6` 디렉터리 및 `invalidated.json`/진행 기록 | 중단, 제외 또는 사전 실행의 진단/비용 provenance. V7 실측과 이어 붙이거나 누락 반복을 보충하는 자료로 쓰지 않음 |
| 추가 실제 자연어 M04 | `changes-semantic/M04-A-attempt1`, `M04-B-attempt1` | 별도 새 기능 모델 검증. **A/B 모두 FAIL**이며 `scope=EVALUATION_CHANGE`, `repeat=0`. 본 P21 정확성/시간/비용에 더하지 않음 |
| 판정 비용 | `evaluation/model-calls.jsonl` | OpenAI judge의 평가 비용. VIA 처리 모델 비용에서 제외하고 총 실험 지출 원장에서는 별도 포함 |
| A 별도 판정 재시도 | `evaluation/a-preserved-retry/` | 동일한 V7 A 원시 trial의 판정 재시도/오류/모델 호출 provenance. 새 후보 실행이나 새 성능 반복이 아니며 판정 원장을 trial 키로 합칠 때 출처를 유지 |

`campaign-progress.jsonl`의 `ALL_WORKERS_COMPLETE`는 210개 원시 실행 완료를 뜻한다. 의미 판정 완료와 같은 사건이 아니다. 전체 최종 완료는 `evaluation/evaluation-summary.jsonl`, 판정/override coverage와 `final/completion.json`을 확인해야 한다. 준비 호출을 제외한다는 것은 본 활동 내부의 실제 준비 호출이나 늦은 반환 호출을 빼는 뜻이 아니다. 예를 들어 S4-01의 `R옛` 실제 호출은 본 worker 원장에 남고 VIA 비용에 포함된다.

## 2. 원본 입력, 후보 출력과 연결 키

| 경로 | 주요 필드 | 용도 |
| --- | --- | --- |
| `frozen-source/benchmark/architecture/dp41_45_poc/fixtures/inputs.json` | 사례별 transcript, initial, source 사건, stub profile, 관측 blueprint | 후보에 제공하는 입력과 공통 실행 조건 |
| 같은 폴더 `oracles.json` | `conditions.C1..C6`, mandatory/forbidden facts, checkpoint/final 요구 | 후보와 분리된 정답. 후보 Context에 정답을 다시 넣어 결과를 재현하지 않음 |
| `workers/{variant}{repeat}/activity-baseline.json.gz` | sources/tasks/interactions 및 payload 정보 | 해당 활동에서 실제 생성한 공통 원본 전체. gzip을 풀어 JSON으로 읽음 |
| `inputs-and-source.jsonl` | `kind`, `scene_key`, `event`, 실제 source availability/followup 입력과 시각 | 예정 사건과 실제 수신 사건 구별. source 변경/후속 사용자 입력의 실제 조건 |
| `raw-inputs/observation-buffer-manifest.json` | 15 frames의 width/height/channels/bytes/SHA와 retention | 실제 RGBA 버퍼 보관 증거. 의미 해석 입력은 별도 선언 Text 원천이며 실제 픽셀 인식 품질 증거가 아님 |
| `trials-unjudged.jsonl` | `variant`, `repeat`, `case_id`, `request_id`, `initial`, `fixture_initial_seed`, `meanings`, `effects`, `deliveries`, `final_tasks`, `final_questions`, `model_calls` | 각 실행의 실제 후보 출력과 최종 상태를 묶은 원시 snapshot. 아직 정답 판정이 붙지 않은 자료 |
| `events.jsonl` | `kind`, `evidence_id`, `scene_key`, request/input/command/publication/source 사건 ID, 실제 값 | 모델 선택, owner 조회, 의미 채택과 폐기, 명령, 결과, 질문 제시, 전달과 현재성 흐름 |
| `effects.jsonl` | `command_id`, `task_id`, `execution_id`, `request_revision`, `operation`, `payload`, `payload_sha256`, send/ingress/receipt 시각 | 외부 Agent에 실제 보낸 원본 payload와 접수 사실. 모사 Agent가 잘못된 payload를 정답으로 고친 것으로 해석하지 않음 |

gzip 파일 자체 SHA와 압축 해제한 canonical 원본 SHA는 다르다. `allocations.jsonl`의 `ACTIVITY_ORIGINALS_CREATED.buffer_sha256`는 압축 전 canonical JSON byte의 SHA이고 `actual_serialization_copy_bytes`가 그 길이다. source 원문/권한/revision과 사건은 실제 저장값을 사용한다. 새로운 Task/artifact의 런타임 ID가 문서의 예시 ID와 다르면 실제 Request, producer Task/Execution, source와 revision의 동일 역할로 대응한다.

trial 기본 키는 **`(variant, repeat, case_id)`**다. 그 아래 실제 Request별 분석은 **`(variant, repeat, case_id, request_id)`**로 연결한다. 한 trial 안에 원래 Request와 질문 답변/정정의 새 Request가 있을 수 있으므로 최초 `trial.request_id`만으로 모든 호출을 묶지 않는다. `call_id`는 HTTP attempt, `command_id`는 실제 명령, `publication_id`는 사용자 전달, `evidence_id`는 각 Evidence writer 안의 기록 ID다. `E100` 같은 번호가 다른 worker나 judge 원장에서도 나올 수 있으므로 디렉터리/stream 또는 variant/repeat/case와 함께 사용한다.

## 3. 모든 모델 호출과 비용

`workers/*/model-call-starts.jsonl`과 `model-calls.jsonl`을 `call_id`로 연결한다. `billing.jsonl`은 동일 terminal 호출 기록의 복제이며 두 파일의 금액을 합치면 중복 청구가 된다. 시작됐지만 terminal이 없는 호출은 먼저 pending/누락을 확인하고 usage를 0으로 발명하지 않는다.

| 필드 | 의미 |
| --- | --- |
| `role` | 실제 VIA 의미 처리 `CINTERPRET`, 사용자 응답 문장 `CRESPONSE`. 평가기는 `JUDGE`/`JUDGE_CALIBRATION` |
| `purpose`, `attempt`, `status`, `discard_reason` | 선언 목적, HTTP 시도 번호, 정상/오류/로컬 취소 후 반환/전송 전 취소와 사유. 모든 실제 사용 비용을 유지 |
| `enqueue_ns`, `send_ns`, `terminal_ns` | enqueue, HTTP 전송, 실제 반환 시각. 전체 왕복과 queue를 분리. non-streaming의 `first_token_ns=None`은 TTFT 미관측이지 0초가 아님 |
| `context.path`, `context.sha256`, `context_sha256` | 실제 wire 요청 artifact 위치/본문 hash와 별도 messages hash |
| `output.path`, `output.sha256`, `raw_usage` | 실제 provider 반환 artifact와 원형 token 사용량 |
| `usage.input_tokens`, `usage.output_tokens`, `usage.input_tokens_details.cached_tokens`, `usage.input_tokens_details.cache_write_tokens` | provider가 반환한 정규화 청구 token. reference 예산 200/300회나 추가 70회를 사용량 대신 넣지 않음 |
| `price`, `cost_usd`, `usage_state`, `cost_basis` | 동결 단가, usage × 단가 값, 알려짐/누락 여부. invoice 금액과 구별 |
| `billed_usd`, `invoice_state` | 실제 청구서 대조 여부. 현재 billed 값은 null이며 provider 사용량 기반 표준 단가 금액을 계정 invoice 확인 금액이라고 말하지 않음 |

artifact JSON 파일에는 끝 newline이 붙지만 `context.sha256`/`output.sha256`는 **JSON을 parse한 뒤** `json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()`로 만든 canonical 본문의 SHA다. 파일 byte 그대로의 SHA는 newline 때문에 다를 수 있다. `context_sha256`는 wire envelope 전체가 아니라 messages 배열의 같은 canonical hash다. 판정 수정 제안에서 쓰는 `source_adjudication_sha256`도 원본 JSON 객체의 canonical hash이며 raw JSONL 파일 전체 SHA와 구별한다.

비용은 방안별 5개 worker의 실제 VIA call을 `call_id` 한 번씩 합산하고 **5로 나눠 USD/P21 평균**을 계산한다. 입력 token에서 실제 cached/cache-write token을 구별해 해당 단가를 적용한다. retry/실패/폐기/미사용/로컬 취소 후 반환도 포함한다. unknown usage의 예산 reserve는 지출 상한 관리용이며 실측 청구로 대체하지 않는다. 별도 readiness, judge, 추가 변경 의미 검증의 원장은 계속 보존하고 범위별 비용으로 보고한다. 음성 API 호출은 없으므로 물리 음성 비용을 실측했다고 주장하지 않는다.

현재 formal call의 `purpose`는 모두 FOREGROUND로 기록됐지만 S4-01의 scene onset 이전 `R옛` 준비 호출도 존재한다. 필요하면 enqueue와 trial의 `scene_origin_monotonic_ns`로 phase를 복원한다. `events`의 `dp41_model_requested.purpose`/ordinal은 A_NEXT_ACTION, B frame/부분 해석 등 구조 역할을 더 자세히 보여준다. `MEANING_DISCARDED`는 Request/scene 수준 기록이므로 모든 terminal call에 일대일 semantic discard 라벨이 있다고 가정하지 않는다. `OUTPUT_DISCARDED` 같은 원천 모사 사건이 새 모델 청구를 뜻하지도 않는다.

## 4. 실제 Text/모사 음성 전달과 시각 좌표

`deliveries.jsonl`과 trial의 `deliveries`는 실제 실행한 로컬 출력 backend의 최종 snapshot이다. `publication_id`, `kind`, `request_id`, `source_event_id`, `facts`, `text`, `voice_text`, `text_presented_s`, `voice_onset_s`, `voice_end_s`, `delivered_receipt`, `discard_reason`, `voice_char_events`를 함께 읽는다.

`text`는 생성한 문장이고 `voice_text`는 backend가 실제로 방출한 prefix다. 중단된 문장의 나머지는 사용자에게 발화된 것으로 세지 않는다. `TEXT_PRESENTED`는 음성 재생 전에 생기는 사건이므로 그 사건 snapshot의 빈 voice_text를 최종 음성 실패로 쓰지 않는다. 생성한 문장이 있다고 전송/게시가 있었다고 추정하지 않으며 실제 presentation/receipt 필드를 확인한다. `delivered_receipt`는 로컬 backend 증거이지 사람이 실제 들었다는 증거가 아니다.

`voice_char_events[]`의 `char_index`, `onset_s`, `causal_node_id`는 모사 backend가 실제 tick에서 문자 prefix를 방출한 시각이다. 계획한 `문자 위치 / characters_per_second`를 실측 onset으로 대신 쓰지 않는다. judge가 고른 `first_actual_core_quote`, `valid_core_quote`는 각각 실제 voice_text에 있는 **정확한 substring**이어야 한다. Voice ON endpoint는 quote 시작 문자의 실제 방출 시각과 Text 제시 시각 중 늦은 시각이다. Voice OFF는 실제 Text 제시 시각이다. 잘못된 최초 핵심 반응 시각과 나중 정정 endpoint를 따로 유지한다. LOCAL_OUTPUT_STOP은 자동 관측한 LOCAL_STOP의 시각/DAG node를 사용하며 publication 문장을 만들어내지 않는다.

시각의 좌표를 섞지 않는다.

- JSONL `monotonic_ns`/`at_seconds`는 Evidence writer의 기록 시각이다. 사건 자체의 send/span/onset 필드가 있으면 그 관측 시각을 우선한다.
- delivery의 `*_s`, event의 `scene_seconds`와 실제 입력/source offset은 scene 기준이다.
- trial의 `scene_origin_monotonic_ns`와 `scene_origin_evidence_seconds`는 scene 기준과 worker Evidence 공통 좌표의 연결점이다.
- `causal_timeline.nodes[].start_s/end_s`는 준비 작업을 포함한 **Evidence 공통 좌표**다. scene origin이 바뀌어도 이 좌표는 바뀌지 않는다.

예를 들어 scene 상대 `start_s`를 DAG에 넣을 때는 `scene_origin_evidence_seconds + start_s`를 사용한다. 다른 worker의 monotonic 기준이나 scene 상대 시각을 원천 UTC로 오해하지 않는다.

## 5. DAG, QA02 세 평균과 QA03 두 평균

`causal-nodes.jsonl` 또는 trial의 `causal_timeline`에는 `timeline_id`, `node_id`, `parents`, `kind`, `owner`, `request_id`, `start_s`, `end_s`, `duration`, `evidence`, `metadata`가 있다. 모델 enqueue/send/return, owner read, 의미 채택, 실제 명령 접수, Agent 결과, input hold, Text, voice setup/queue/각 문자/재생이 같은 실제 종속 관계로 연결된다.

귀속시간은 같은 관측 DAG에서 `EXTERNAL_AGENT_WORK`, `EXTERNAL_USER_WAIT`, `EXTERNAL_USER_SPEECH`의 duration만 0으로 두고 부모의 max와 순차 합으로 다시 계산한다. cloud 모델, queue, 전사 확정, 소스 조회, 로컬 commit/검증, 다른 음성 재생 대기와 설명되지 않은 gap은 VIA 시간에 남는다. 외부 구간의 길이를 단순 합산해서 wall-clock에서 빼지 않는다. 먼저 원래 DAG가 실제 endpoint의 wall-clock을 복원하는지 검증한다. 재계산은 같은 실현 경로의 귀속 분석이며, 외부 시간이 달랐다면 모델 판단이나 경합이 실제로 어떻게 달라졌을지의 새 실행 예측은 아니다.

최종 정규 관측에는 반응별 `start_s`, `end_s`, `first_actual_s`, `valid`, `wrong_first`, `attributed_seconds`, `first_actual_attributed_seconds`, endpoint node와 evidence가 있다. QA02는 다음 세 값을 같은 **135관측/방안**에서 보여준다.

| QA02 값 | 오답/미전달 및 시간 처리 |
| --- | --- |
| `variants.{A,B}.qa02.cap_mean` 주지표 | 최초 핵심 응답이 오답이면 5초 대체. 지정 valid endpoint가 없으면 wrong_first=false인 경우에도 5초. 유효 정상 반응이 6초면 실제 6초이며 모든 성공값을 min(t,5)로 자르지 않음 |
| `correct_only_mean` | wrong_first=false **그리고 지정 valid endpoint가 있는** 관측만 실제 귀속시간으로 평균. 오답 뒤 정정과 endpoint 미완료는 제외하며 n/분모를 별도 공개 |
| `actual_including_wrong_mean` | 최초 실제 핵심 반응 시각이 있는 관측을 오답 포함 실제 귀속시간으로 평균. 미전달로 최초 실제 시각도 없으면 제외하며 n/분모 공개 |

같은 `qa02` 객체의 각 평균은 `{value, numerator, n}`이다. value는 초 단위 평균, numerator는 포함된 귀속 초의 합, n은 실제 분모다. cap_mean의 n은 135이며 두 보조 평균의 n은 제외 여부에 따라 작을 수 있다. 원시 관측의 `cap_seconds`, `correct_seconds`, `actual_seconds`와 함께 재계산한다.

**정상 단계 진행과 오답을 구별한다.** 실제 상태에 맞는 접수 안내나 처리 단계 반응은 `wrong_first=false`일 수 있다. 그래도 해당 반응의 지정 valid endpoint가 아직 없으면 주지표 5초이며 정답 첫 평균에서는 제외한다. 실제 최초 반응 시각이 있으면 실측 평균에는 포함한다. 잘못된 payload, 불필요한 확인 질문, 한 번의 종결 답에서 필수 사실을 누락한 반응은 `wrong_first=true`다. 같은 입력/endpoint 규칙을 A/B에 적용한다. 뒤 정정은 `correction_seconds`와 valid endpoint로 기록하되 최초 오답을 소급해서 정답 반응으로 바꾸지 않는다.

QA03는 같은 **95 cycle/방안**에서 `variants.{A,B}.qa03.cap_mean`(30초 대체 주지표)과 `completed_only_mean`을 보여준다. 두 평균도 `{value, numerator, n}`이고 주지표의 n은 95다. 귀속 30초 내 지정 유효 endpoint가 없으면 주지표에 30초를 넣고 분모 95를 유지한다. 미완료/미전달을 분모에서 지워 좋은 평균으로 만들지 않는다. 관측 중 늦게 완료된 경우에도 30초 기준을 초과하면 주지표는 30초이고, 완료 관측만의 평균에는 실제 귀속시간을 남긴다. 늦은 완료와 끝까지 미완료를 구별한다. `wall_clock_seconds`는 보조로 보존하며 전체 외부 Task 모두의 종료를 cycle endpoint로 바꾸지 않는다.

첫 발화의 종료와 전사 확정은 다른 사건이다. reaction blueprint의 start_kind와 `observed_inputs`, `observed_source_availability`에서 실제 시작 경계를 고른다. 예정 input/source 시각을 실제 수신 시각으로 대체하거나 없는 시각을 0으로 채우지 않는다. 필요한 endpoint/DAG의 연결이 끊긴 평가 오류는 후보 미완료와 별도로 해결한다.

## 6. Mac 메모리와 배분 기록

`workers/*/memory.jsonl`은 macOS `TASK_VM_INFO` flavor22의 `phys_footprint_bytes`, `resident_size_bytes`/RSS 실측이다. 각 row는 `sample_id`, `process_id`, `generation`, `included_in_via_sum`, `phase`, `sample_start_ns`, `sample_end_ns`, `success`, `task_info_count`를 갖는다. phase는 PREPARATION/ACTIVITY/DRAIN/IDLE이다. 수집 실패/누락은 0으로 바꾸지 않는다.

같은 `(repeat, sample_id)`의 포함 process들을 PID+generation별 한 번씩 합치고 그 동시 합의 최대를 고른다. 현재 명시한 scope는 worker PID 하나이며 VIA/HTTP와 측정 thread/공통 stub/원본 버퍼/원시 기록기의 RAM을 포함한다. controller, 별도 judge/변경 process, cloud 모델과 외부 Agent 서버, OS desktop은 포함하지 않는다. credential의 짧은 security CLI child는 별도 메모리 표본이 없다. 이 값은 제품 전체 process census가 아니다.

주지표는 **다섯 반복 전체의 최대 footprint**다. 반복 peak의 평균은 보조값이며 별도 반복의 최대들을 동시에 존재한 것처럼 더하지 않는다. RSS 자체 최대와 footprint 최대 시점의 RSS도 구별해 보존한다. idle 보호 조건은 10초 IDLE 구간 첫/마지막의 같은 process footprint 증가이고 감소는 증가값 0으로 둔다. 50ms sampling의 실제 interval/read duration을 확인하며 표본 사이의 순간 peak까지 관측했다고 말하지 않는다.

`allocations.jsonl`은 실제 만진 15 RGBA 버퍼 byte, 각 scene의 20MiB source payload 원본, 그 원본을 포함한 실제 canonical serialization copy byte를 설명한다. payload 길이와 전체 JSON copy 길이는 다르다. 이는 Python 객체/alias의 완전한 배분 census가 아니며 **OS 실측에 배분 byte를 다시 더하지 않는다**. 공통 버퍼/모델 client의 공유와 owner/Context/event snapshot/serialization copy는 OS 표본에서 실제 존재할 때 계상된다. phys_footprint와 RSS는 서로 다른 Mac 원지표이고 Windows commit으로 환산하지 않는다. campaign의 4GiB 참조 예산/MiB band 비교도 Mac PoC 비교척도로 표시하고 Windows 측정이나 Windows 성능 예측이라고 쓰지 않는다.

## 7. QA05 선택과 기능 증거의 재사용

[qa05-selection-manifest.json](../qa05-selection-manifest.json)의 `selected[].reviewed_result_ref`, `reviewed_result_sha256`, `source_snapshot_digest`, `changed_unit_set`, `count`가 선택 원장이다. `selected_change_records`는 hash와 동일 baseline/변경 source/patch/완료 상태를 검사한다. 원래 12개와 M04 대안 2개가 디스크에 있다고 14개 과제로 합산하지 않는다.

각 job에는 독립 `source/`, `baseline-manifest.json`, `after-source-manifest.json`, `patch.diff`, 원시 `result.json`, `regression.json`, `final-reviews.json`, 새 `reviewed-result.json`이 있다. 원시 결과의 author count=None/AWAITING 상태는 그대로 남고 root/contract 두 LLM agent의 책임 리뷰 이후 별도 reviewed 결과를 생성했다. 리뷰는 인간 2명이나 3개의 독립 모델 통계 표본으로 표시하지 않는다.

`baseline_source_hash`/`after_source_hash`는 source 파일별 SHA뿐 아니라 byte 길이와 mode를 포함한 snapshot digest다. campaign의 flat file-SHA digest와 숫자가 다른 것이 바로 파일 변화의 증거는 아니다. `patch_sha256`는 patch 파일의 raw SHA다. 책임 수는 고정 U01~U18에서 수정한 책임 집합이며 파일 수/LOC와 다르다.

M04의 selected A는 `changes-alternatives/ORIGINAL-A-LINEAGE-v1/CH-DP41-A-M04`, B는 `changes-alternatives/TYPED-GOALREF-v2/CH-DP41-B-M04`다. 지정 모델 port/local runtime/producer lineage 회귀 PASS와 `changes-semantic`의 실제 새 자연어 모델 FAIL은 서로 다른 증거다. 실제 자연어 실패를 기능 stub PASS로 바꾸거나 최종 QA01 본집단에 합치지 않는다.

M05의 `changes-supplemental/CH-DP41-{A,B}-M05-PUBLISHED-TOKEN-RESTART-v1`은 실제 게시 token/owner+binding snapshot/다른 PID의 fresh restore/actual ANSWER를 잇는다. M06의 `changes-supplemental/M06-EXACT-S103-v3`는 exact 원문 preference 등록의 C1~C6, 사용자/지속 scope와 M언어 보존/공통 조회/철회 삭제를 baseline와 unchanged job 사본에서 검사했다. 기존 공통 기능 누락을 본 측정 전에 보정한 것은 M06의 새 변경 비용으로 세지 않는다. 이 기능 증거는 실제 의미 모델 정확성 재측정이 아니다.

## 8. 원래 judge, 판정 오류와 override

| 원장 | 핵심 필드와 역할 |
| --- | --- |
| `evaluation/judgements.jsonl` | variant/repeat/case, `pass`, `evaluation_input_sha256`, 원래 `judgement`. 두 blind prompt pass와 필요 시 세 번째 dispute pass의 각각을 보존 |
| `evaluation/adjudications.jsonl` | `judgements`, `disagreement_fields`, `final_judgement`, `violation_review_candidates`, `method`, `same_judge_model_not_statistical_independence` | 
| `evaluation/judge-validation.jsonl`, `model-validation.jsonl` | 잘못된 schema/증거 ID/JSON의 원형 출력과 repair/retry 기록 |
| `evaluation/evaluation-errors.jsonl` | `status=RETRY_REQUIRED`, error와 trial 키. 평가 자체의 오류이며 후보 오류 조건으로 변환하지 않음 |
| `evaluation/a-preserved-retry/{judgements,adjudications,model-calls,evaluation-errors}.jsonl` | A trial의 별도 재시도 원장. 원래 root 판정 원장과 같은 이름이어도 writer/evidence_id 좌표는 별개 |
| `evaluation/evaluation-origin-manifest.json` | 완성된 최종 210행 판정의 결합 provenance. A preserved retry에서 결합한 행의 origin/source SHA, 원래/V2/V3/V4 evaluator manifest, 최종 원장 hash와 method 구분 |
| `reviews/*adjudication-proposals.jsonl` | audit_id, reviewer, trial 키, 원본 canonical SHA, path, old/proposed value, reason/evidence. 제안은 아직 적용된 판정이 아님 |
| `reviews/root-adjudication-decisions.json` | hash-pinned 제안 파일과 audit_id별 ACCEPT/REJECT/WITHDRAWN 및 이유. 생성되면 읽음 |
| `evaluation/trace-adjudication-overrides.jsonl` | 승인된 변경을 별도 row로 보존한 final_judgement와 `root_trace_audits`. 원래 judge JSONL을 편집하지 않음 |
| `reviews/root-adjudication-completion.json` | 제안/승인/overridden trials 수, 결정/도구/override SHA와 source/API 변화 여부. 생성되면 읽음 |

최종 결합 순서는 `(variant,repeat,case_id)`별 원래 adjudication을 먼저 고르고, 같은 키의 승인된 override가 있으면 그 레코드를 사용한다. 원본, 제안, 명시적 결정, 적용 후 레코드를 모두 연결해야 사후 변경이 어떤 근거로 이루어졌는지 재검토할 수 있다. override는 후보 출력을 고치는 기능이 아니라 실제 trace에 대한 판정을 교정한 원장이다. model-call 비용/원시 payload/전달 시각은 그대로 둔다.

A의 preserved retry와 원래 재개 판정 중 어느 행을 채택했는지는 최종 origin manifest에서 확인한다. 병합된 A 행은 `evaluation_retry_origin`이 가리키는 원형 원장과 `source_row_sha256`를 가지며, 이 SHA는 provenance 필드를 덧붙이기 전 원형 JSON 객체의 canonical hash다. 그 필드가 없는 행은 root의 원래 재개 원장 소속이며 단계별 사용 도구는 `evaluation-control/stages.jsonl`과 네 manifest로 확인한다. 같은 trial이 두 writer에서 판정됐다고 두 조건 묶음을 합산하지 않는다. 원래 judge 원장과 보존 재시도의 원형 행/SHA를 따라가며 최종 trial 키는 정확히 210개를 유지한다. judge 비용은 실제 distinct call_id별로 모든 재시도를 포함하되 결합된 원장과 원형 재시도 원장을 다시 더해 중복 지출을 만들지 않는다. origin manifest 생성 전에는 재시도 완료 행을 합친 것을 최종 판정이라고 단정하지 않는다.

**`uncertain=true`나 judge API/schema/증거 오류는 후보 실패가 아니다.** unresolved 판정은 평가 미완료로 보고 명시적 근거 판정/승인 override 또는 보존된 재판정으로 해소한다. 후보 실패로 cap/오류 수를 넣거나 불확실 관측을 조용히 제외해 분모를 줄이지 않는다. 현재 evaluate의 ADJUDICATED 라벨만으로 uncertainty가 없음을 보장하지 않으므로 최종 `final_judgement.uncertain`을 반드시 검사한다. 실제 후보의 미응답/잘못된 payload와 평가기의 불확실성을 구별한다. 210개 모두 해결된 판정을 갖고 원래 오류/재시도가 보존된 이후 집계한다.

uncertainty를 해소한 경우에도 원래 `final_judgement.uncertain=true`, `disagreement_fields`, 각 blind pass의 조건/이유를 삭제하지 않는다. 해당 trial의 uncertainty 변경 제안과 근거 evidence, root의 명시적 disposition, 적용 후 `uncertain=false`를 같은 audit_id/원본 canonical SHA로 추적한다. 예를 들어 동일 Conversation/Task/기존 결과 관계를 trace에서 확인한 해소와, 실제 채택/명령/전달이 없음을 확인한 후보 실패 판정은 서로 다른 근거다. 단순히 uncertainty flag를 false로 바꿔 완료시키는 작업으로 취급하지 않는다.

## 9. 최종 산출물과 재가공 순서

최종 집계 후 `final/qa-results.json`에 여섯 QA의 원값/0~6점/보호 조건/반복 값/paired 비교가 생긴다. 다음 파일도 생성되므로 표를 재사용할 때 원시 링크와 함께 보관한다.

- `completion.json`: 210 trials / 1,260 conditions / 270 reactions / 190 cycles / 12 selected jobs의 coverage 확인.
- `normalized-trials.jsonl`: 실제 trial과 최종 판정을 연결한 C1~C6/reaction/cycle.
- `causal-attribution.jsonl`: observation_id와 purpose별 wall-clock/귀속시간/외부 제외/경로/node 재계산.
- `conditions.csv`, `reactions.csv`, `cycles.csv`, `changes.csv`, `model-calls.csv`, `repeat-results.csv`: 재가공용 표. CSV 안 dict/list는 JSON 문자열로 parse.
- `request-paths.jsonl`, `mechanism-summary.json`: 실제 Request별 호출/조회/반영/폐기 구조의 비교.

재가공은 새 출력 디렉터리에서 수행하고 원시/판정/동결 사본/원래 final 결과를 덮어쓰지 않는다. 권장 순서는 manifest+53파일 hash 검증, 10 worker의 21사례/완료 확인, call 시작-종료-billing/artifact hash 연결, 210 adjudication과 승인 override의 완전성/uncertainty 해결 확인, 실제 publication/DAG endpoint 연결, 각 QA의 분모/비용/동시 메모리/선택 12개 재계산이다. score band와 보호 조건은 같은 동결 QA JSON과 campaign 예외를 적용한다. 새로운 입력/모델/API/규칙으로 실행하면 새 campaign 결과이며 V7의 일부로 취급하지 않는다.

독립 감사 파일은 `raw-audits/qa04-qa06-independent-review.{json,md}`, `qa05-selection-independent-review.{json,md}`다. 원시 SHA/peak 표본/호출 비용/선택 결과를 보존했으며 다시 숫자를 만들 때 참고할 수 있다. 평가 중인 judge 원장에 대한 당시 비용은 시각이 붙은 snapshot이고 최종 평가 지출로 쓰지 않는다. 완결된 판정+override를 기준으로 QA01~QA06을 독립 재계산한 `raw-audits/final-independent-review.json`은 산출값, score, 보호 조건, 다섯 paired repeat, 선정 12개와 네 판정 도구 세대의 불변성을 final 결과와 대조해 통과했다.

완료 상태와 산출물은 [completion.json](../final/completion.json), [여섯 QA 원값과 점수](../final/qa-results.json), [최종 보고서](../final/report.md), [최종 독립 원시 재계산](final-independent-review.json)에서 확인한다. 최초 동결 집계와 후속 보고서 보충은 `final/report-supplement-provenance.json`으로 구분한다. 재가공의 실행 키와 분모는 이 완료 세대에 고정하며, 준비 세대나 추가 M04 의미 FAIL을 본 P21 조건에 섞지 않는다.
