# 실제 resolution 경로 관찰: A1/B1의 S2-02와 S2-05

상태: **DIAGNOSTIC_OBSERVATION_NO_SCORE**. 2026-10-12의 완료된 A1/B1 원시 기록만 읽었다. 후보 재실행, 모델/API 호출, 시간 worker 변경 또는 C1~C6/QA 점수 판정은 하지 않았다. 신규 파일은 이 리뷰 문서 하나다.

Frozen source digest: `119b3344337d1f1fd3732df9e6cd3110cca38c574f73b6ce1c649a032227952c`.

정적 검토에서 찾지 못했던 실제 B 부분 호출의 JSON-mode 통합 결함을 원시 provider 응답에서 확인하여 부모 작업에 즉시 전달했다. 아래는 모델 선택, 구현 결함, 실제 효과/전달을 분리하는 원인 검토다. 기록을 보존하며 결함 수정 후 결과를 소급 생성하지 않는다.

## 요구와 표현 범위

S2-02는 실제 들은 E1 첫 두 문장, 현재 D2@v2/D3@v1, 유지비/설치 기간 기준과 가격 평가 제외를 새 제안서 위임에 연결해야 한다. 미재생 보안 문장을 들은 요구로 추가하지 않는다. 원문 receipt checkpoint에 v6 §8.2의 실제 유효 접수+8초 final tail이 함께 적용된다.

S2-05는 t=2.97/2.98/2.99초에 도착한 E새/비교/견적을 실제 확보하여 납기 2주 우선과 가격 평가 제외를 새 제안서에 연결해야 한다. 양안 모두 최초 모델 호출은 발화 종료 뒤이며 도착한 source handle을 볼 수 있다. source 본문은 실제 read가 필요하다.

표현 가능성은 실행 성공 주장과 구분한다. 두 요구 모두 기존 CREATE_PROPOSAL, 신규 Task, bounded 원문/전달 read, source_refs와 도메인 instruction으로 표현할 수 있다. 가격 평가 제외/납기 우선은 Downstream Agent가 수행할 도메인 기준이므로 instruction에 놓을 수 있다. VIA가 소유하는 Task/권한/의존 관계를 자유문만으로 대신 표현하는 허용을 의미하지 않는다.

근거: 동결한 [QA v6 원문](../frozen-source/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md) §S2-02, §S2-05, §8.2. 후보 계약은 [contract.py](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/contract.py), [prompts.py](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py), [b.py](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/b.py), [공통 runtime](../frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py)이다.

## A1 / S2-02 / repeat 1

Tuple: `(A,1,S2-02)`, trial evidence `E2980`, request `R:3e22ed0c-4556-408b-b1f4-d214587df878`. 원본 trial: [trials-unjudged.jsonl](../workers/A1/trials-unjudged.jsonl) line 4. 이 trial의 canonical JSON SHA256: `66d482a1e844bd7ff08b74867e83b95fd51d05145a34ad66e271634dc4763dba`.

입력: “아까 들은 기준과 두 업무 결과로 새 제안서를 만들어줘.”

조회 후 모델이 `CREATE_PROPOSAL` 하나를 제안했다. 실제 `TASK_RESULT(T비교)`는 D2@v2, `TASK_RESULT(T견적)`는 D3@v1을 반환했다. E1 조회는 Text 전체와 실제 Voice 전달 첫 두 문장을 서로 다른 필드로 반환했다. 모델의 instruction에는 유지비/설치 기간과 가격 제외가 들어갔지만, 미재생 보안 문장에서 가져온 “Optional factor: Security level ... can be added as a top priority”도 포함됐다. 가격은 원문 100/120백만원인데 instruction에서는 10/12 million KRW로 전환됐다. 실제 sources 본문은 100/120백만원으로 그대로 전달됐다.

채택 뒤 신규 제안서 명령 1개를 보냈고 실제 receipt를 받았다. 공통 stub은 전달받은 D2/D3 본문을 연결한 실제 artifact를 만들었다. 접수 안내와 최종 결과가 각각 Text와 Voice로 전달됐다. 접수 안내 문장은 일반적인 operation/ACCEPTED 안내이며 “들은 기준/비교/견적/가격 제외”를 전부 직접 설명하지 않았다.

원인 구분: E1의 실제 들은 범위, 현재 두 결과, 새 제안서 위임은 기존 계약으로 표현 가능하다. 따라서 보안 tail의 사용 가능성, 잘못된 금액 환산과 안내의 구체성은 모델 출력/응답 및 판정할 의미 사실의 문제로 추적해야 한다. 코드가 source를 바꾸거나 보안 조건을 생성한 경로는 없다. Text에 있던 미재생 문장을 어떤 의미로 사용했는지의 C2/C4 최종 판정과 C6 판정은 judge 뒤 확정한다. stub의 제안서 작성 품질은 실물 제안서의 품질 측정이 아니다.

### 실제 사건 연결

| evidence ID | scene seconds | kind | 실제 근거 |
| --- | ---: | --- | --- |
| E2015 | 3.003547 | dp41_model_requested | A_NEXT_ACTION; ordinal=1 |
| E2070 | 5.344400 | dp41_a_model_action | action=READ |
| E2072 | 5.345877 | OWNER_READ | read_id=47fda6e6-c2b6-4465-a5bc-c00a9490bf7f; OK; candidate_count=1; returned=D2@v2 |
| E2076 | 5.347164 | OWNER_READ | read_id=b71816db-d306-48f4-8f35-52a6dd4e53d0; OK; candidate_count=1; returned=D3@v1 |
| E2082 | 5.454651 | OWNER_READ | read_id=5b6f734e-59de-48af-a212-3e16fd2b22ae; OK; candidate_count=2001; returned=E1@v1,H0001@v1,H0020@v1,H0021@v1,H0040@v1 |
| E2085 | 5.454858 | dp41_model_requested | A_NEXT_ACTION; ordinal=2 |
| E2170 | 9.290902 | dp41_a_model_action | action=PROPOSE |
| E2174 | 9.299762 | MEANING_ADOPTED | goals=CREATE_PROPOSAL; unresolved=[] |
| E2180 | 9.307504 | COMMAND_SENT | CMD:227a63d5-dd67-4e53-96ae-7166255fbe7f |
| E2188 | 9.510184 | COMMAND_ACCEPTED | CMD:227a63d5-dd67-4e53-96ae-7166255fbe7f |
| E2215 | 10.499572 | TEXT_PRESENTED | publication_id=P:1:62397ca5-7f21-4d95-bffd-995414b0b3da |
| E2224 | 10.767412 | VOICE_ONSET | publication_id=P:1:62397ca5-7f21-4d95-bffd-995414b0b3da |
| E2397 | 17.315951 | ARTIFACT_PRODUCED | A:4b673708-bbca-42e9-9ff0-1915446a9041; payload_sha256=4ee78c6f823c30f1b02d6fb9953d9dc44b677e79c87b4c4806b5ad02f8e5bc32 |
| E2427 | 18.379399 | TEXT_PRESENTED | publication_id=P:2:90d91474-15e9-4675-a50c-4b19789ac515 |
| E2435 | 18.649366 | VOICE_ONSET | publication_id=P:2:90d91474-15e9-4675-a50c-4b19789ac515 |

### 실제 효과와 최종 delivery

- command `CMD:227a63d5-dd67-4e53-96ae-7166255fbe7f`, goal `G-create-proposal`, operation `CREATE_PROPOSAL`, Task `T:2854884d-effe-43fe-ab37-09b9af464a45`, Execution `X:137a3b55-e130-4352-8b93-c848833e7f25`, payload SHA256 `4ee78c6f823c30f1b02d6fb9953d9dc44b677e79c87b4c4806b5ad02f8e5bc32`.
- publication `P:1:62397ca5-7f21-4d95-bffd-995414b0b3da`, kind `RESPONSE`, delivered_receipt `True`, 실제 Voice: `CREATE_PROPOSAL 업무가 접수(ACCEPTED)되었습니다.`. 원본 Text/Voice/문자별 시각은 trial의 delivery에 남아 있다.
- publication `P:2:90d91474-15e9-4675-a50c-4b19789ac515`, kind `RESULT`, delivered_receipt `True`, 실제 Voice: `CREATE_PROPOSAL 업무가 완료되었습니다. 결과: A: 설치 2일, B: 설치 4일 / A: 가격 100백만원/연간 유지비 3백만원, B: 가격 120백만원/연간 유지비 5백만원`. 원본 Text/Voice/문자별 시각은 trial의 delivery에 남아 있다.

### 실제 model context와 output

SHA256는 원시 ledger가 참조하는 canonical JSON hash이며 이 리뷰에서 파일을 읽어 일치 여부를 확인했다. JSON 파일의 provider 출력과 error를 바꾸지 않았다.

| call ID / role / HTTP | context와 SHA256 | output과 SHA256 |
| --- | --- | --- |
| `1d9c7d39-ec63-44d1-9264-54a23740fe06` / CINTERPRET / 200 | [context](../workers/A1/model-contexts/1d9c7d39-ec63-44d1-9264-54a23740fe06.json) `fa91caa3715d269ecd883fd49109ae15670412a6c1c223e43fd6cdcef420ee24` | [output](../workers/A1/model-outputs/1d9c7d39-ec63-44d1-9264-54a23740fe06.json) `830e2b18de127f955e24f6e6fb9a14725a802fda8bd89d87e7a2afa5a333516e` |
| `bfc54d00-76f1-4f33-828a-fa2edf19ee1c` / CINTERPRET / 200 | [context](../workers/A1/model-contexts/bfc54d00-76f1-4f33-828a-fa2edf19ee1c.json) `55d443ca9e255824b71210a2b337cdcb2bc9ff154ba0a37fd659120f1564e98e` | [output](../workers/A1/model-outputs/bfc54d00-76f1-4f33-828a-fa2edf19ee1c.json) `40af107b1c51c9dc7249ee107d12f96ac76f90b10cd06bad3591ed0d18678348` |
| `96f956de-1f8f-4789-83be-02d043caf083` / CRESPONSE / 200 | [context](../workers/A1/model-contexts/96f956de-1f8f-4789-83be-02d043caf083.json) `221d70221feb3b640927c0002d8c2e240a32bd8367a3136820e42d871be828b4` | [output](../workers/A1/model-outputs/96f956de-1f8f-4789-83be-02d043caf083.json) `f3a5104ea821f369f2aaeadd59df25b6316f5b830681e07e7fd2d897b5bd255d` |
| `cd7c30bc-e351-498a-b0ca-e407b6f59d4a` / CRESPONSE / 200 | [context](../workers/A1/model-contexts/cd7c30bc-e351-498a-b0ca-e407b6f59d4a.json) `887959f03a14ce40fc9892fa15d8222337455ab7d39bf0fbaca66e6bea536a79` | [output](../workers/A1/model-outputs/cd7c30bc-e351-498a-b0ca-e407b6f59d4a.json) `ffe2da1271b3d513dfdf0b66d326a952e79cec4c9026346357c27da21dd3bb45` |

## A1 / S2-05 / repeat 1

Tuple: `(A,1,S2-05)`, trial evidence `E6082`, request `R:f9bf2400-dbcc-41d5-be22-b2b442c597c3`. 원본 trial: [trials-unjudged.jsonl](../workers/A1/trials-unjudged.jsonl) line 7. 이 trial의 canonical JSON SHA256: `b9bc9c418529370c01c3c345a3b42fbf5f4d38052d8993c3389c674d4816ceca`.

입력: “전에 들은 기준과 방금 도착한 제품 비교, 견적으로 제안서 만들어줘. 가격은 평가하지 마.”

모델은 새 D새비교@v1과 D새견적@v1을 실제 조회했다. E새/과거 실제 전달 기준을 읽는 추가 조회는 없다. 첫 조회의 query는 D새비교를 “previous standards/criteria”를 찾는 문서라고 기술했다. 최초 PROPOSE는 meaning version 누락으로 거절되고 다음 모델 호출에서 형식을 보완했다. 최종 instruction에는 두 새 source의 납기/가격과 가격 평가 제외가 있지만 E새의 “납기 2주 이내 우선”은 없다.

채택 뒤 신규 CREATE_PROPOSAL 명령 1개와 실제 receipt, 새 두 source의 본문에서 생성된 artifact가 기록됐다. 접수와 완료의 두 응답은 실제 전달 receipt와 전체 Voice 기록이 있다. 원래 E새 source는 발화 종료 전 이미 owner에 도착해 초기 모델 metadata에 handle로 보였다. 후보가 그 원문을 읽거나 전달 근거를 확보한 기록은 없다.

원인 구분: E새는 `KNOWN_REF` 또는 `PAST_DELIVERED`로 읽을 수 있고, 두 새 문서는 기존 selector로 읽을 수 있다. 새 제안서와 납기 기준/가격 제외의 도메인 지시는 계약 안에서 표현 가능하다. 이전 기준을 D새비교 내용으로 대체하고 E새를 조회하지 않은 선택은 원시 모델 출력에서 관찰된다. 정상 코드가 적법하게 공급된 E새를 버리는 경로는 실행되지 않았다. version 보완은 형식 오류에 대한 모델 재호출이며 별도 호출/시간/비용을 이미 남겼다. C1~C6 최종 점수는 이 문서에서 정하지 않는다.

### 실제 사건 연결

| evidence ID | scene seconds | kind | 실제 근거 |
| --- | ---: | --- | --- |
| E5135 | 3.004616 | dp41_model_requested | A_NEXT_ACTION; ordinal=1 |
| E5184 | 5.077063 | dp41_a_model_action | action=READ |
| E5186 | 5.077640 | OWNER_READ | read_id=6f77b2de-a341-4616-b43a-bb75689864ae; OK; candidate_count=1; returned=D새비교@v1 |
| E5190 | 5.078169 | OWNER_READ | read_id=3d6dd6a6-014e-4488-81bf-60e362cae2ef; OK; candidate_count=1; returned=D새견적@v1 |
| E5193 | 5.078455 | dp41_model_requested | A_NEXT_ACTION; ordinal=2 |
| E5258 | 7.952279 | dp41_a_model_action | action=PROPOSE |
| E5259 | 7.952363 | dp41_output_rejected | version must be dp41-meaning-runtime-1 |
| E5260 | 7.952421 | dp41_model_requested | A_NEXT_ACTION; ordinal=3 |
| E5311 | 10.103519 | dp41_a_model_action | action=PROPOSE |
| E5316 | 10.107074 | MEANING_ADOPTED | goals=CREATE_PROPOSAL; unresolved=[] |
| E5322 | 10.114928 | COMMAND_SENT | CMD:6d98edae-64d5-44e6-b3b8-0044842da340 |
| E5329 | 10.317266 | COMMAND_ACCEPTED | CMD:6d98edae-64d5-44e6-b3b8-0044842da340 |
| E5359 | 11.440599 | TEXT_PRESENTED | publication_id=P:1:c7406295-1262-430b-a5fe-8c75aa84176a |
| E5368 | 11.706871 | VOICE_ONSET | publication_id=P:1:c7406295-1262-430b-a5fe-8c75aa84176a |
| E5537 | 18.117371 | ARTIFACT_PRODUCED | A:dde8ba8a-52c2-430e-8048-86ecfff26d26; payload_sha256=87a8a145ab0cf2dbbc56e4a05b07ef0077c435308eb7375554079b91ede0680b |
| E5577 | 19.319471 | TEXT_PRESENTED | publication_id=P:2:54fa5c8d-44f0-4364-b2d1-c49b73027b69 |
| E5606 | 20.323581 | VOICE_ONSET | publication_id=P:2:54fa5c8d-44f0-4364-b2d1-c49b73027b69 |

### 실제 효과와 최종 delivery

- command `CMD:6d98edae-64d5-44e6-b3b8-0044842da340`, goal `G-create-proposal`, operation `CREATE_PROPOSAL`, Task `T:332b1a42-cb37-4a49-ba65-e71498ec716f`, Execution `X:de093265-7fd1-4054-b7a8-e416cd4e3918`, payload SHA256 `87a8a145ab0cf2dbbc56e4a05b07ef0077c435308eb7375554079b91ede0680b`.
- publication `P:1:c7406295-1262-430b-a5fe-8c75aa84176a`, kind `RESPONSE`, delivered_receipt `True`, 실제 Voice: `CREATE_PROPOSAL 업무가 ACCEPTED(접수)되었습니다. 완료되지 않았습니다.`. 원본 Text/Voice/문자별 시각은 trial의 delivery에 남아 있다.
- publication `P:2:54fa5c8d-44f0-4364-b2d1-c49b73027b69`, kind `RESULT`, delivered_receipt `True`, 실제 Voice: `CREATE_PROPOSAL 작업이 완료되었습니다. 결과: 제품 A의 납기는 10일, 제품 B의 납기는 20일 / 제품 A는 30백만원, 제품 B는 40백만원`. 원본 Text/Voice/문자별 시각은 trial의 delivery에 남아 있다.

### 실제 model context와 output

SHA256는 원시 ledger가 참조하는 canonical JSON hash이며 이 리뷰에서 파일을 읽어 일치 여부를 확인했다. JSON 파일의 provider 출력과 error를 바꾸지 않았다.

| call ID / role / HTTP | context와 SHA256 | output과 SHA256 |
| --- | --- | --- |
| `2a321542-bd00-4d31-8a01-61be90921178` / CINTERPRET / 200 | [context](../workers/A1/model-contexts/2a321542-bd00-4d31-8a01-61be90921178.json) `08115096f07b435f63f5f3cbb87d29e5965a026d7063aed54a2a93acd7756553` | [output](../workers/A1/model-outputs/2a321542-bd00-4d31-8a01-61be90921178.json) `0bf329d55d678453bef13d669b48d2c61a6ab2be4f0118a1ac1920b141e170ce` |
| `29d5fe17-065d-40d4-8e92-43f110eabe74` / CINTERPRET / 200 | [context](../workers/A1/model-contexts/29d5fe17-065d-40d4-8e92-43f110eabe74.json) `9fbbb8f90b33a4662463ae95332d6591f2af124d93bb9688a40ddcd6fec813a8` | [output](../workers/A1/model-outputs/29d5fe17-065d-40d4-8e92-43f110eabe74.json) `20364993c8c6eb70e7a68adc26d46181adc2af98267804a78855852503791859` |
| `74caf908-aee5-4485-b673-0cb1bdaef8de` / CINTERPRET / 200 | [context](../workers/A1/model-contexts/74caf908-aee5-4485-b673-0cb1bdaef8de.json) `1da391a043e9fd17b4bf8f4dac21940cb13760384e488e0fc7c2f3b7237c4756` | [output](../workers/A1/model-outputs/74caf908-aee5-4485-b673-0cb1bdaef8de.json) `fbca31ad08e9bd8e52e0edbb435b0fb3a1bf2ef01e5834f7a6401f6cfcf56eb1` |
| `60fff59f-8b4a-4994-b726-56de4e761a9d` / CRESPONSE / 200 | [context](../workers/A1/model-contexts/60fff59f-8b4a-4994-b726-56de4e761a9d.json) `de82ead20447a305f0e13a9d6893891e120498d48ba7e67d4d543806beb0e34d` | [output](../workers/A1/model-outputs/60fff59f-8b4a-4994-b726-56de4e761a9d.json) `631f05713d30de88e93b4077f027b12d9d9a5e9960a24e2ce53c40e0b2e9e207` |
| `16064116-65f8-4659-8c63-cbe402a2a9cc` / CRESPONSE / 200 | [context](../workers/A1/model-contexts/16064116-65f8-4659-8c63-cbe402a2a9cc.json) `2d40ef1caf1f41fba0c8f382be21639c632822fa4987fafe63e36d4ec9a20f5f` | [output](../workers/A1/model-outputs/16064116-65f8-4659-8c63-cbe402a2a9cc.json) `81474cf1670ce857f8d0b295cfb9005fd2784f658a6d0af7581cdb44c87a46eb` |

## B1 / S2-02 / repeat 1

Tuple: `(B,1,S2-02)`, trial evidence `E2272`, request `R:987195f6-7494-4468-989d-4053dfe45ce6`. 원본 trial: [trials-unjudged.jsonl](../workers/B1/trials-unjudged.jsonl) line 4. 이 trial의 canonical JSON SHA256: `57896011af8efcdeff319f9f70a9db4b19c0d62f77ab74aafeecad313572f3cf`.

입력: “아까 들은 기준과 두 업무 결과로 새 제안서를 만들어줘.”

초기 frame은 CREATE_PROPOSAL 1개와 기준/두 결과 reference 3개를 만들었다. 두 `TASK_RESULT` selector는 같은 넓은 query를 사용했고, 각각 D2@v2와 D3@v1의 두 후보를 반환했다. 기준 조회는 E1과 배경 이력 후보들을 반환했다. Engine은 r-criteria를 PARTIAL_PENDING으로 바꾸고 허용된 reference 한 항목의 부분 모델 호출을 요청했다.

이 부분 호출은 의미 모델 답을 받기 전에 HTTP 400으로 거절됐다. provider의 실제 error는 `messages`에 `json`이라는 단어가 없는데 `response_format=json_object`를 썼다는 요청 정합성 오류다. SELECT_SYSTEM과 이 실제 partial input에는 해당 단어가 없다. 이어 Engine은 goals=[]와 HTTP400 unresolved를 가진 failed meaning을 반환했고 공통 경계는 이를 채택했다. 명령/Task 생성 효과는 0개다. 별도 CRESPONSE가 오류 안내를 생성해 실제로 전달했다.

원인 구분: 실제 E1/두 결과로 제안서를 만드는 요구는 유한 계약으로 표현 가능하다. 이 실행의 첫 terminal 원인은 B의 부분 호출과 공통 JSON-mode adapter 간 통합 버그다. 모델의 후보 선택 능력이나 B의 표현 한계로 바꾸어 설명할 수 없다. 초기 frame의 넓고 중복된 result selector는 모델이 만든 선택이며 부분 판단을 더 필요하게 만들었지만, 합법적인 부분 판단 경로 자체가 HTTP400으로 차단됐다. 해당 defect가 B 품질/시간/비용 비교를 왜곡할 수 있으므로 원본은 진단 증거로 보존하고 공식 비교에 사용할 수 있는지는 별도 campaign 판단이 필요하다.

잠재 문제는 별도로 남긴다. 초기 frame unresolved에는 문자열 두 개가 들어 있다. validator는 list 여부만 확인하고 B의 후속 recoverable 처리에서는 dict의 `.get`을 기대한다. 현재는 HTTP400에서 먼저 종료됐으므로 이 잠재 AttributeError를 실제 관측 실패로 계산하거나, 수정 후 성공을 예측하지 않는다.

### 실제 사건 연결

| evidence ID | scene seconds | kind | 실제 근거 |
| --- | ---: | --- | --- |
| E1849 | 3.004370 | dp41_model_requested | B_FRAME; ordinal=1 |
| E1927 | 6.513909 | dp41_b_frame_produced | goals=CREATE_PROPOSAL |
| E1932 | 6.545783 | OWNER_READ | read_id=87d910b1-036b-464c-8c64-c17018e8bcb6; OK; candidate_count=2001; returned=E1@v1,H0001@v1,H0002@v1,H0003@v1,H0004@v1,H0005@v1,H0006@v1,H0007@v1,H0008@v1,H0009@v1 |
| E1938 | 6.547574 | OWNER_READ | read_id=652f86d4-fee0-431e-9830-58f0b231dbc4; OK; candidate_count=2; returned=D2@v2,D3@v1 |
| E1944 | 6.548845 | OWNER_READ | read_id=fd3b4c75-0b90-4f50-b6c6-5aa5a86c1209; OK; candidate_count=2; returned=D2@v2,D3@v1 |
| E1948 | 6.549051 | dp41_model_requested | B_PARTIAL_SELECT; ordinal=2 |
| E1973 | 7.435510 | dp41_resolution_failed | qwen HTTP 400; saved response model-outputs/7be8f202-a36b-4097-ae7a-fd89eed99d3f.json |
| E1976 | 7.456166 | MEANING_ADOPTED | goals=; unresolved=[{'reason': 'RuntimeError: qwen HTTP 400; saved response model-outputs/7be8f202-a36b-4097-ae7a-fd89eed99d3f.json'}] |
| E2005 | 8.461962 | TEXT_PRESENTED | publication_id=P:1:e0494413-cb85-4317-abcd-907dea317094 |
| E2012 | 8.713749 | VOICE_ONSET | publication_id=P:1:e0494413-cb85-4317-abcd-907dea317094 |

### 실제 효과와 최종 delivery

- 실제 effect/command: 0개. 이 관측은 C1~C6 점수 0을 의미하는 표기가 아니다.
- publication `P:1:e0494413-cb85-4317-abcd-907dea317094`, kind `RESPONSE`, delivered_receipt `True`, 실제 Voice: `qwen HTTP 400 오류로 인해 응답이 생성되지 않았습니다.`. 원본 Text/Voice/문자별 시각은 trial의 delivery에 남아 있다.

### 실제 model context와 output

SHA256는 원시 ledger가 참조하는 canonical JSON hash이며 이 리뷰에서 파일을 읽어 일치 여부를 확인했다. JSON 파일의 provider 출력과 error를 바꾸지 않았다.

| call ID / role / HTTP | context와 SHA256 | output과 SHA256 |
| --- | --- | --- |
| `d31f01b4-ecec-4a78-a4ab-d2929902dc56` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/d31f01b4-ecec-4a78-a4ab-d2929902dc56.json) `581df8306d1eab94c4de3288511b5bc504bfd1be9adb29e287d27db0536ff8af` | [output](../workers/B1/model-outputs/d31f01b4-ecec-4a78-a4ab-d2929902dc56.json) `2eb00c44d4d9038cb3a6dce8a731afc7d8948cd49496e215d3f5e253fcd484ef` |
| `7be8f202-a36b-4097-ae7a-fd89eed99d3f` / CINTERPRET / 400 | [context](../workers/B1/model-contexts/7be8f202-a36b-4097-ae7a-fd89eed99d3f.json) `76056cafa9fb2d1509106f7cd1281c9364991265ae72bd7d740a01d649689a1f` | [output](../workers/B1/model-outputs/7be8f202-a36b-4097-ae7a-fd89eed99d3f.json) `91ca69d1b163fa3322612b6b936b0331def70c7c078d7d60c57d1b5cec958e4f` |
| `f570239a-2fb0-42b3-b6cc-49ccdc53745b` / CRESPONSE / 200 | [context](../workers/B1/model-contexts/f570239a-2fb0-42b3-b6cc-49ccdc53745b.json) `f16dbd97805b5ebbfb44cc0121bf9d8f48d54420c21ae6f83b7585e6b22bf91e` | [output](../workers/B1/model-outputs/f570239a-2fb0-42b3-b6cc-49ccdc53745b.json) `e261a1d6c5d870f2f65e455525070296bfd4e0cf7d5e2380855b6bd5430c32e7` |

## B1 / S2-05 / repeat 1

Tuple: `(B,1,S2-05)`, trial evidence `E4897`, request `R:7ae7edb0-8c73-437f-84b2-0d5789ccd35e`. 원본 trial: [trials-unjudged.jsonl](../workers/B1/trials-unjudged.jsonl) line 7. 이 trial의 canonical JSON SHA256: `b5f316e617e16cadefef0ddb8b6f096fa5b4e18739effb3abab601ed6764afeb`.

입력: “전에 들은 기준과 방금 도착한 제품 비교, 견적으로 제안서 만들어줘. 가격은 평가하지 마.”

모델은 COMPARE 후 CREATE_PROPOSAL이라는 두 목표를 만들었다. 최초 여섯 frame에서 제안서의 target_ref를 `G-compare`라는 goal ID로 놓아 `frame target must be an unbound reference ID`로 거절됐다. 일곱 번째 frame은 target_ref를 null로 바꿨지만, 미래 현재 비교 결과를 `TASK_RESULT(source_ref=G-compare)`라는 실제 owner Task 조회로 표현한 reference를 유지했다. Engine의 실제 read는 이 reference에 NO_MATCH를 반환했다. 새 견적 D새견적 reference도 frame에는 없다.

기준의 `RECENT_INTERACTION` read에는 배경 이력 10개가 반환되고 E새가 이 bounded 결과에 없다. 해당 실제 read의 외부 query는 mention “전에 들은 기준”이다. reference_spec은 top-level query가 없으면 mention을 넣고 OwnerRepository는 selector.query보다 외부 query를 먼저 사용한다. 이 공통 조회는 의미 정답을 찾지 않고 lexical 후보를 반환한다. Engine은 r-baseline에 부분 판단을 요청했지만 이 호출도 동일 JSON-mode HTTP400으로 거절됐다.

failed meaning은 goals=[]로 채택됐고 실제 command/effect와 사용자 delivery는 0개다. 일곱 frame 호출과 부분 호출이 logical Request의 물리 호출 8회 한도를 소비해 오류 응답 모델 호출도 budget으로 거절됐다.

원인 구분: 기존 계약은 E새@v1의 실제 source/전달 근거와 두 새 문서의 참조를 연결해 신규 CREATE_PROPOSAL을 표현할 수 있다. goal ID를 기존 Task target와 TASK_RESULT로 사용하는 선택, 새 견적 reference 누락은 모델 출력에서 관찰된다. 실제 현재 producer 의존을 원한다면 계약의 symbolic result가 따로 있지만 본 실행은 그 형식을 선택하지 않았다. 이 모델 오류들과 형식 재생성 비용을 통합 버그 수정이라는 이유로 성공으로 바꾸면 안 된다. 다만 뒤의 부분 판단과 오류 응답 실패는 HTTP400/예산과 구분해야 하며, JSON 요청 통합 버그로 정상 경로가 차단된 결과를 B 구조의 필연으로 일반화할 수 없다.

### 실제 사건 연결

| evidence ID | scene seconds | kind | 실제 근거 |
| --- | ---: | --- | --- |
| E4285 | 3.004055 | dp41_model_requested | B_FRAME; ordinal=1 |
| E4367 | 6.717983 | dp41_b_frame_produced | goals=COMPARE,CREATE_PROPOSAL |
| E4368 | 6.718095 | dp41_output_rejected | frame target must be an unbound reference ID |
| E4369 | 6.718152 | dp41_model_requested | B_FRAME; ordinal=2 |
| E4444 | 10.084029 | dp41_b_frame_produced | goals=COMPARE,CREATE_PROPOSAL |
| E4445 | 10.084098 | dp41_output_rejected | frame target must be an unbound reference ID |
| E4446 | 10.084142 | dp41_model_requested | B_FRAME; ordinal=3 |
| E4517 | 13.227670 | dp41_b_frame_produced | goals=COMPARE,CREATE_PROPOSAL |
| E4518 | 13.227746 | dp41_output_rejected | frame target must be an unbound reference ID |
| E4519 | 13.227790 | dp41_model_requested | B_FRAME; ordinal=4 |
| E4593 | 16.516313 | dp41_b_frame_produced | goals=COMPARE,CREATE_PROPOSAL |
| E4594 | 16.516539 | dp41_output_rejected | frame target must be an unbound reference ID |
| E4595 | 16.516673 | dp41_model_requested | B_FRAME; ordinal=5 |
| E4644 | 18.582766 | dp41_b_frame_produced | goals=COMPARE,CREATE_PROPOSAL |
| E4645 | 18.582957 | dp41_output_rejected | frame target must be an unbound reference ID |
| E4646 | 18.583236 | dp41_model_requested | B_FRAME; ordinal=6 |
| E4699 | 20.836774 | dp41_b_frame_produced | goals=COMPARE,CREATE_PROPOSAL |
| E4700 | 20.836975 | dp41_output_rejected | frame target must be an unbound reference ID |
| E4701 | 20.837091 | dp41_model_requested | B_FRAME; ordinal=7 |
| E4750 | 22.885122 | dp41_b_frame_produced | goals=COMPARE,CREATE_PROPOSAL |
| E4755 | 22.904759 | OWNER_READ | read_id=bb1baaf8-c436-4c31-a02e-41f628e2917a; OK; candidate_count=2001; returned=H0001@v1,H0002@v1,H0003@v1,H0004@v1,H0005@v1,H0006@v1,H0007@v1,H0008@v1,H0009@v1,H0010@v1 |
| E4761 | 22.905500 | OWNER_READ | read_id=27d00ea5-9814-4e25-b86a-e2e22542320e; OK; candidate_count=1; returned=D새비교@v1 |
| E4767 | 22.905962 | OWNER_READ | read_id=1c5c2658-ac09-4246-8831-85a874a3ff86; NO_MATCH; candidate_count=0; returned= |
| E4771 | 22.906221 | dp41_model_requested | B_PARTIAL_SELECT; ordinal=8 |
| E4791 | 23.599536 | dp41_resolution_failed | qwen HTTP 400; saved response model-outputs/be26a961-a51f-44ea-ba23-7e260499319f.json |
| E4794 | 23.612499 | MEANING_ADOPTED | goals=; unresolved=[{'reason': 'RuntimeError: qwen HTTP 400; saved response model-outputs/be26a961-a51f-44ea-ba23-7e260499319f.json'}] |
| E4796 | 23.612654 | REQUEST_FAILED | Logical Request model-call budget exhausted |

### 실제 효과와 최종 delivery

- 실제 effect/command: 0개. 이 관측은 C1~C6 점수 0을 의미하는 표기가 아니다.
- 실제 사용자 delivery: 0개. 시간 endpoint/미완료 처리는 별도 QA 계약에 따른다.

### 실제 model context와 output

SHA256는 원시 ledger가 참조하는 canonical JSON hash이며 이 리뷰에서 파일을 읽어 일치 여부를 확인했다. JSON 파일의 provider 출력과 error를 바꾸지 않았다.

| call ID / role / HTTP | context와 SHA256 | output과 SHA256 |
| --- | --- | --- |
| `f326908c-aab6-4751-bcda-448016de8244` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/f326908c-aab6-4751-bcda-448016de8244.json) `c1428b464831660839e8656270721562a24de1feb3cdaf0de5722ed54fc99a72` | [output](../workers/B1/model-outputs/f326908c-aab6-4751-bcda-448016de8244.json) `3fc248581b958d57ba952d32b35777074c39d6b13afecaebb95ff960fcee5244` |
| `ee9307ec-92df-4915-a55d-bc93e6b3d849` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/ee9307ec-92df-4915-a55d-bc93e6b3d849.json) `a094a9dda60d7d7f95e53c4aa0d17ed61ead99b24fcd8650bb5c4bd6f27f5da7` | [output](../workers/B1/model-outputs/ee9307ec-92df-4915-a55d-bc93e6b3d849.json) `b5a7ec0e3d656f7da5c0e4afefbfd77a2b582d6b29a6cd9e3df09c76f479980c` |
| `68564873-8324-4d57-9628-38409b13ffc3` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/68564873-8324-4d57-9628-38409b13ffc3.json) `9c4cfa4e8c487dfa7951a4f48605171413949d8eb4712554855e17b8de806a5b` | [output](../workers/B1/model-outputs/68564873-8324-4d57-9628-38409b13ffc3.json) `ab200ac89e0eb8288f0010a433f0ffacdde81a810ef38815b17a5eefd65e32b6` |
| `bc571c3f-7694-448f-b8ac-a186681bc501` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/bc571c3f-7694-448f-b8ac-a186681bc501.json) `263a2499a238a060b597b818eaf6b2f5d3aa958b3a50fdaa0e88018899da58f5` | [output](../workers/B1/model-outputs/bc571c3f-7694-448f-b8ac-a186681bc501.json) `2175432a3ad8b2ae98bfda5939939c9dc88890dce4ac867a668fe6638f2c9509` |
| `65896359-4aaa-42ce-9b7e-40dcf8ce57bb` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/65896359-4aaa-42ce-9b7e-40dcf8ce57bb.json) `7396de452588053c562b7e496292be07eb68d3f76225ee5017248901f4e738b4` | [output](../workers/B1/model-outputs/65896359-4aaa-42ce-9b7e-40dcf8ce57bb.json) `2d4a5111c11c642e01ff3edc36a15cee0f3ef781bc926d350431f3622a7b73e2` |
| `0d64ea21-8900-4941-811b-3147fdf4a8f5` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/0d64ea21-8900-4941-811b-3147fdf4a8f5.json) `dd4af530b2e2ea3b21c7c6fd9a9a5b32c486b3b7c56d5a4c310c46186820bae5` | [output](../workers/B1/model-outputs/0d64ea21-8900-4941-811b-3147fdf4a8f5.json) `6b1730bd5262ac3dce30b264a780fc49b4d7f892623b061c9600e0b0ce90f59c` |
| `f9ee04c1-9e20-41f8-b263-6343627c8f22` / CINTERPRET / 200 | [context](../workers/B1/model-contexts/f9ee04c1-9e20-41f8-b263-6343627c8f22.json) `221e1faa71064f1224bca5afac431f9a71f24027dc162fb9ee3d0db7434c31b1` | [output](../workers/B1/model-outputs/f9ee04c1-9e20-41f8-b263-6343627c8f22.json) `d30d61f4f1cb58f5ef661473b79206aa2491c01ffc30b4189588546ed4847326` |
| `be26a961-a51f-44ea-ba23-7e260499319f` / CINTERPRET / 400 | [context](../workers/B1/model-contexts/be26a961-a51f-44ea-ba23-7e260499319f.json) `635330dfda7d2d613a8f36592e322ee7807b38f3b8ca0fc5e714378db8249ef7` | [output](../workers/B1/model-outputs/be26a961-a51f-44ea-ba23-7e260499319f.json) `7def64a448b9f7528b8f2f3e80ec1aff0293ada9adcda184eefb4fffd0270049` |

## 구현 결함과 일반화 한계

**P1: B_PARTIAL_SELECT JSON-mode 통합 결함.** 실제 두 HTTP400 응답은 동일하게 `messages must contain the word json ... response_format ... json_object`를 보고한다. B SELECT_SYSTEM에는 JSON 단어가 없고, 실제 partial context도 없었다. 반면 초기 A/B 시스템에는 `Return one JSON object`가 있다. 이는 유한 reference 판단이 실행 가능한 모델 요청으로 전송되지 않은 차이이며 구조의 의미 정확성 한계로 보고할 수 없다. `prompts.py:82-88`, `b.py:184-188`, provider 공통 wire의 `response_format=json_object`를 함께 검토해야 한다.

**잠재 계약 불일치: unresolved 원소 타입.** B1/S2-02 frame의 자유 문자열 두 개는 `validate_document`의 list 타입 검사에 통과하지만 `b.py:209`는 `.get`을 요구한다. 실제 이 실행의 terminal은 앞선 HTTP400이다. 별도의 실행 없이 잠재 오류를 실제 발생 오류로 집계하지 않는다.

B1/S2-05의 goal/reference 오류는 통합 결함과 함께 원시 기록에 보존한다. 결함 수정은 모델이 선택한 잘못된 목표 연결, 필요한 근거 누락과 과도한 형식 재생성을 정답으로 보정할 권한을 주지 않는다. 또한 수정 후 결과는 이 기록에서 추정하거나 소급 만들지 않는다.

A1/S2-02/S2-05의 접수/완료 receipt는 의미 전체의 정답을 보증하지 않는다. C2/C4에 필요한 실제 들은 범위와 기준, C6 전달 사실은 개별 조건과 실제 delivery를 대조해야 한다. 한 내부 오류를 모든 조건 실패로 전파하거나 실제 전체 Voice 전달을 초기 event snapshot의 빈 Voice로 바꾸어 판정하지 않는다.

이 리뷰는 네 실제 실행에서 관찰된 경로 설명이다. A/B 우열, QA 최종 수치, 전체 자연어 지원 능력, 실제 제안서 품질과 실제 음성 모델 성능을 주장하지 않는다.

## 원시 stream hash

| worker | trials-unjudged.jsonl file SHA256 | model-calls.jsonl file SHA256 |
| --- | --- | --- |
| A1 | `8553ee7e0c2b89f2d1af36a3cb1008a9613ea6ee3eb3f1f99bbfced110aa9796` | `76601ed71ef78706c774478d2a4296a86d06ae2058a9ee7194a54d2b04737f0c` |
| B1 | `4ff2c6684234f3d75175e537982b98affbde623aaeefa8b15316cce540890d9a` | `6e8320877faad79e4fec1d19c904ef1a4327c6f0977cca8bee563381b4b682c5` |

이 리뷰 생성 과정은 JSON/Markdown 파일 읽기와 hash 계산, 이 Markdown 하나 저장뿐이다. 모델 client/candidate/runtime 모듈을 import하거나 실행하지 않았다.
