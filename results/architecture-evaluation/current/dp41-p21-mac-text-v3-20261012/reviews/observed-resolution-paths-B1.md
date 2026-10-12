# v3 B1의 두 실제 Request 해결 경로와 A1 대조

상태: **READ_ONLY_OBSERVATION / NO_QA_VERDICT**. 완료된 `(B, 1, S2-02)`와 `(B, 1, S2-05)`를 [동일 v3 A1 관찰](./observed-resolution-paths-A1.md)과 대조했다. 두 trial의 실제 Context/Output/owner read/의미/효과/전달만 읽었다. API, 후보/테스트 실행, 재채점과 후보/fixture/정답 수정은 수행하지 않았다. B1의 나머지 반복과 최종 judge 결과를 포함한 결론은 아니다.

기준은 [manifest](../manifest.json)의 `DP41-P21-MAC-TEXT-v3`, frozen source digest `f521d6bea196342dab6c0f66268134e9c33f25dfa86a0aac874afc160cb6a20a`이다. 실제 Qwen 의미 호출과 공통 전사/Voice 모형의 혼합 증거다. 실제 음성 품질이나 Windows 실측을 주장하지 않는다. source 제공 시각 이후의 실제 handle만 모델 Context에 있었다.

동결된 [ASR-QA-v6 명세](../frozen-source/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md) S2-02/S2-05와 §8.2가 요구 기준이다. 접수와 최종 tail은 구분한다. `OBSERVED`나 전달 receipt를 의미 정답으로 해석하지 않는다.

## 1. 구조와 판독 원칙

B의 실제 [resolve 코드](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/b.py)는 모델의 finite frame을 검증하고 reference별 `UNRESOLVED/READ_PENDING/PARTIAL_PENDING/BOUND/NO_MATCH/AMBIGUOUS`를 갱신한다. 여러 read는 `asyncio.gather`로 요청한다. 필요한 후보 선택만 별도 모델에 맡기며 불확실하면 확인 질문 뒤 새 **unbound frame**을 받는다. 최종 참조 결합은 `_build_meaning`이 수행한다. 부분 모델이 전체 목표/조건을 다시 만드는 경로는 아니다.

이 두 실행에는 B_FRAME/부분 호출/확인 질문이 실제 여러 번 발생했다. B를 무조건 한 번의 모델 호출로 끝나는 구조로 해석할 수 없다. 부분 후보 선택의 `uncertain=true`는 정상적인 불확실성 출력이며, 코드의 합쳐진 reason `AMBIGUOUS_OR_INVALID_PARTIAL`만 보고 malformed JSON이나 API 오류로 분류하면 안 된다.

요구 표현 가능성과 이번 경로의 성공 가능성을 구분한다. 공통 [prompt 계약](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py)에 NEW_TASK/CREATE_PROPOSAL, PAST_DELIVERED, KNOWN_REF와 명시 Task/source locator가 존재한다. 이 두 목표를 나타내는 데 새 연산이 필요하지 않다. 다만 부분 모델의 지원 범위는 **참조 후보 선택**이며 목표/selector/조건 instruction 전체를 교정하지 않는다. 모델이 잘못 만든 frame을 코드가 자동 정답으로 바꾸지 않는다는 실제 한계가 있다. 이 관찰만으로 들은 범위 처리나 모든 연산 조합의 완전한 지원을 증명하지 않는다.

## 2. `(B, 1, S2-02)` — 과거 설명을 찾았으나 잘못 만든 Task locator와 입력 ID로 중단

[trial 원장](../workers/B1/trials-unjudged.jsonl)의 line 4, `E3787`; Request `R:589156a6-6909-4509-a8fe-66feeda8f498`, 최초 input `I:7ec8634e-ac62-4280-b444-8fd621c9cd42`, Conversation `C현재`다. 상태 `OBSERVED`, raw JSONL 행(LF 포함) SHA-256 `85bbbec2f1228a2153268d3dcdac3a6293f6ed0f5dc9c60bdbe237d1dc494f3f`이다.

### 2.1 공통 원천과 첫 frame

원문 요구는 실제 들은 E1 첫 두 문장(유지비/설치 기간, 가격 평가 제외)과 현재 D2@v2/D3@v1로 새 제안서를 위임하는 것이다. E1 미재생 보안 문장을 들은 조건으로 사용하면 안 된다. [첫 Context](../workers/B1/model-contexts/59089d54-9c5d-4378-a42d-075f3cba5b38.json)에 E1/D2/D3 source handles와 T비교/T견적의 현재 result handles가 실제 있었다. 배경 HT193~HT200도 같은 bounded recent Task 목록에 있었다. 원문/들은 문장 body는 초기 Context에 없었다.

E2771 / 6.804초의 frame은 `G-create-proposal`, `NEW_TASK/CREATE_PROPOSAL`을 생성했다. 그러나 instruction은 “두 업무 결과”로 첫 두 완료 READ_FIELD Task HT193/HT194를 대표 선택했다고 명시했다. 실제 제품 비교/견적 Task가 있었음에도 배경 Task를 선택한 모델 출력이다. reference selector는 다음과 같았다.

| Frame reference | 실제 모델 selector | 코드가 보낸 read |
| --- | --- | --- |
| `r-criteria` | `RECENT_INTERACTION`, selector.query=`criteria` | interaction_retrieve; outer query=`아까 들은 기준`, limit=10 |
| `r-task-HT193` | `TASK_RESULT`, source_ref=`THT193` | 실제 없는 Task locator |
| `r-task-HT194` | `TASK_RESULT`, source_ref=`THT194` | 실제 없는 Task locator |

제공된 ID는 HT193/HT194이고 `THT193/THT194`는 제공되지 않았다. 실제 T비교/T견적을 code가 무료로 선택하거나 ID 접두사를 정답으로 고치지 않았다.

### 2.2 항목 해결과 실제 부분 모델

| Event / scene seconds | 실제 receipt/항목 상태 |
| --- | --- |
| E2776 / 6.824 | read `15cb963c-10d6-48ca-9590-f1a0356e03d5`: E1@v1 + H0001~H0009, 총 후보 2001, 반환 10건 |
| E2782 / 6.824 | read `1e2a65fd-25a9-40fa-b069-3f45b6006b73`: THT193 `NO_MATCH`, 후보 0 |
| E2788 / 6.825 | read `d3d44e3d-1809-4264-b645-57306018b159`: THT194 `NO_MATCH`, 후보 0 |
| E2791 / 6.825 | r-criteria `PARTIAL_PENDING` |
| E2822/E2823 / 7.963 | 실제 부분 모델이 `candidate_ref=E1@v1`, `uncertain=false`와 해당 read ID를 출력; 코드가 BOUND로 갱신 |
| E2824/E2825 / 7.963 | 두 Task reference는 NO_MATCH 유지 |

E1 read content는 `text` 전체와 `delivered_voice_text` 첫 두 문장을 별도 필드로 제공했다. partial Context의 후보 evidence에도 두 범위가 함께 있었다. 부분 모델이 E1의 identity를 선택한 것이지 유지비/가격 제외 의미를 별도 typed 값으로 생산한 것은 아니다. 이후 명령이 없으므로 실제 제안서 payload에 그 범위가 보존되는지 이번 경로는 검증하지 않았다.

A1은 selector.query=`criteria`, limit=5의 RECENT_INTERACTION으로 H0001~H0005만 받았다. B1의 실제 read query는 `아까 들은 기준`, limit=10으로 달랐다. 이는 [reference_spec](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/contract.py)의 outer query 기본값 `ref.query`, 없으면 `ref.mention`, limit 기본값 10과 [공통 owner](../frozen-source/prototypes/candidates/dp41_45_poc/shared/owners.py)의 outer query 우선 규칙 때문이다. 이 한 번의 E1 반환을 B가 본질적으로 과거 의미를 더 정확히 찾는다는 증거로 일반화할 수 없다. Task exact locator와 lexical query, 실제 검색어/limit를 비교 원장에 남겨야 한다.

### 2.3 코드 질문과 후속 frame identity

E2826의 확인 질문은 “다음 후보 중 어떤 대상이고 무엇을 변경할까요? E1@v1; H0001@v1; H0002@v1; H0003@v1; H0004@v1; H0005@v1; H0006@v1; H0007@v1”이었다. 실제 scope.field_refs는 미해결 두 Task인데, 일반 템플릿은 이미 선택된 criterion의 후보까지 모아 첫 8개 label을 나열했다. 질문 내용이 두 미해결 Task를 구체적으로 설명하지 않는 **현재 code 질문 생성의 제한**이다. 이는 지원되지 않는 CREATE_PROPOSAL 연산과 다르며, 질문을 의미 정답으로 보강하지 않고 원본 그대로 보존했다.

question `eccbd731-cd7c-475f-8d6c-64cf6d53a723` revision 1, publication `P:1:6ec79adb-33ed-4c57-8caa-eedfe9cdff1c`는 Text 7.972827초, Voice 8.254117~25.257847초로 전체 전달됐다. E3281에서 실제 QUESTION_PRESENTED가 확인됐다. 1초 사용자 대기와 3초 발화 뒤 E3368/E3371에 “새로운 조건이나 권한은 추가하지 않았어. 처음 요청대로 처리해줘.”가 들어왔다. 답변 input ID는 `input:2f13c40f-a76b-40b8-baef-4ad92d184d14`다. 추가 기준/정답 Task ID/권한을 공급하지 않았다.

[후속 frame Context](../workers/B1/model-contexts/a052615b-7bb3-4035-90d5-6b424aec904e.json)는 최초 root input ID와 이전 frame, 실제 답변 envelope의 input ID를 함께 제공했다. E3442 / 32.381초 출력은 같은 잘못된 두 Task 참조를 유지하고 `frame.input_ref`를 답변 input ID로 변경했다. resolve는 최초 Request 해석 input ID를 유지하며 E3443에서 `frame input_ref differs from current input`으로 중단했다. code가 실제 input ID를 fabricated 값으로 바꾸거나 사용자의 새로운 목표를 생성한 증거는 없다.

이 frame identity 검사는 B 표현의 추가 계약이다. A의 whole meaning에는 같은 frame input_ref 필드가 없고 공통 Request/currentness fence는 별도로 적용된다. 이번 관측은 **모델의 frame/답변 input ID 혼동과 현재 B guard/recovery 정책**의 효과다. 입력 ID 불일치는 frame schema validation 후 outer ContractError 경로로 끝나며, 재생성 feedback을 한 번 더 주는 경로는 발생하지 않았다. prompt의 “current input_id” 문구가 답변 ID와 혼동될 여지는 분석 제한으로 남기되, 원본 root ID와 실제 답변 ID의 차이를 지우거나 결과를 정답으로 고치지 않는다.

### 2.4 실패 의미와 실제 전달

E3446 / 32.392초에 goals=[]/unresolved=`ContractError: frame input_ref differs from current input`의 실패 의미가 채택됐다. canonical meaning SHA-256 `097291b262e14618229fe2e50f363637c92dfccba650cb1eac167916da8f817b`다. `effects=[]`, 새 Task/Agent command/제안서 artifact가 없다. T비교/T견적은 COMPLETED, T보호는 RUNNING을 유지했다.

실패 안내 publication `P:2:384f7417-cff1-4a0b-84d1-e4ee22b1fad2`는 Text 33.263262초, Voice 33.515612~41.349107초로 “계약 오류로 인해 프레임 input_ref가 현재 입력과 달라 작업이 거부되었습니다.” 전체가 전달됐다. receipt=true는 실패 안내가 실제 전달된 사실이며 제안서 작성이나 최종 tail 완료를 뜻하지 않는다. 점수는 여기서 판정하지 않는다.

### 2.5 실제 호출

| 원장 line / role | 실제 호출 / 출력 종류 | Context / Output |
| --- | --- | --- |
| 14 / CINTERPRET | `59089d54-9c5d-4378-a42d-075f3cba5b38`<br>B_FRAME; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/59089d54-9c5d-4378-a42d-075f3cba5b38.json)<br>[Output](../workers/B1/model-outputs/59089d54-9c5d-4378-a42d-075f3cba5b38.json) |
| 15 / CINTERPRET | `3c0f7b3e-7a3b-48d3-baf8-d74d66d2398f`<br>B_PARTIAL_SELECT / uncertain=False; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/3c0f7b3e-7a3b-48d3-baf8-d74d66d2398f.json)<br>[Output](../workers/B1/model-outputs/3c0f7b3e-7a3b-48d3-baf8-d74d66d2398f.json) |
| 16 / CINTERPRET | `a052615b-7bb3-4035-90d5-6b424aec904e`<br>B_FRAME; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/a052615b-7bb3-4035-90d5-6b424aec904e.json)<br>[Output](../workers/B1/model-outputs/a052615b-7bb3-4035-90d5-6b424aec904e.json) |
| 17 / CRESPONSE | `bf8908d4-0349-474a-9c7f-11507f6d4393`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/bf8908d4-0349-474a-9c7f-11507f6d4393.json)<br>[Output](../workers/B1/model-outputs/bf8908d4-0349-474a-9c7f-11507f6d4393.json) |

CINTERPRET 3회는 frame 2회+부분 선택 1회다. CRESPONSE 1회, 모두 HTTP 200/attempt 1이다. 부분 호출 통합이 성공한 상태에서 후보의 잘못된 참조/identity가 실제 실패를 만들었다. v2의 고정 API 400 결함을 이 v3 결과에 재사용하지 않는다.

## 3. `(B, 1, S2-05)` — 과거 설명을 Task에서 찾고 새 결과를 기존 Task 결과로 조회

[trial 원장](../workers/B1/trials-unjudged.jsonl)의 line 7, `E6803`; Request `R:dcc13b20-c481-4134-872d-7ee417f7e43f`, input `I:eacc5691-1309-4c06-8990-cd82078a16da`다. 상태 `OBSERVED`, raw JSONL 행(LF 포함) SHA-256 `b3561c1e0a4dba11f038d0b9914105f8ebf9db0537d2e03c191d1034085ce733`다.

### 3.1 실제 원천 도착과 첫 Context

E새/D새비교/D새견적의 canonical 도착은 2.97/2.98/2.99초이며 실제 availability E5140/E5144/E5149는 2.971379/2.980771/2.992080초다. 최초 speech end/전사 확정은 3.000613/3.000651초로 별개다. [첫 Context](../workers/B1/model-contexts/115d3965-d24b-4397-abdf-6215a64e8c07.json)에 세 source handle과 E새의 실제 전달 metadata가 있었다. 원천 body나 미래 결과/정답 Task 연결을 첫 Context에 선주입하지 않았다.

원문은 납기 2주 이내 우선/가격 평가 제외 기준과 두 새 문서를 사용한 제안서 위임이다. 가격 견적 인용은 허용하고 가격 우열 평가를 제외한다.

### 3.2 모델 frame과 세 번 반복한 read/부분 선택

E5248 / 7.135초 frame은 두 NEW_TASK를 만들었다. G-compare COMPARE와 G-proposal CREATE_PROPOSAL을 USE_RESULT로 연결하고 가격 평가 제외 instruction을 적었다. reference 구성은 다음과 같다.

| Reference / selector | 실제 문제와 최초 반환 |
| --- | --- |
| r-baseline / `RECENT_TASK`, selector.query=`criteria or baseline document`, outer query=`전에 들은 기준` | 전달된 설명을 Task 목록에서 조회; E5252는 실제 배경 HT001~HT010, 후보 201/반환 10 |
| r-new-product / `EXPLICIT_SOURCE(DOCUMENT, D새비교)` | E5258 실제 D새비교@v1의 납기 10/20일, 후보 1 |
| r-comparison-result / `TASK_RESULT`, selector.query=`comparison between baseline and new product`, outer query=`비교 결과` | 아직 실행하지 않은 새 G-compare 결과를 기존 Task 결과에서 찾음; E5264 NO_MATCH, 후보 0 |

D새견적 참조는 frame에 없었고 E새도 실제 조회하지 않았다. 현재 producer 결과의 symbolic 참조와 원천으로 이미 도착한 비교 문서, 과거 Task 결과는 서로 다른 개념이다. 공통 계약은 symbolic current result를 구분하지만 이 모델 출력은 과거 TASK_RESULT 조회를 선택했다. 여기서 올바른 symbolic 대체나 새 goal 실행을 code가 대신 생성하지 않았다. 위 두-goal 계획의 모든 symbolic source 결합 지원 여부는 이 중단 경로로 검증되지 않으며, canonical 목표에는 두 goal보다 단일 CREATE_PROPOSAL과 세 실제 source 참조로 표현하는 경로가 있다.

| frame iteration | baseline read / partial | 비교 문서 read | 기존 결과 read | 실제 질문/답변 |
| --- | --- | --- | --- | --- |
| 1 | E5252: `aac01787-7ac3-4db0-a405-ad6ee459ea3b`; E5296: candidate_ref=null, uncertain=true | E5258: `1a3d69e5-5fa0-484a-a322-393df94343b7` | E5264: `1eff6c79-2d70-4384-9664-0a3a48448bbf`, NO_MATCH | E5300 질문; E5752 답변 |
| 2 | E5841: `06d68fc0-0add-4d90-9d2a-74eb24f8d90b`; E5885: candidate_ref=null, uncertain=true | E5847: `fa27b4b0-838e-4ff8-9ec5-a1d1f59d37ca` | E5853: `8b5623a5-7974-4bb7-8969-5ed912ad7936`, NO_MATCH | E5889 질문; E6342 답변 |
| 3 | E6442: `61d35cf9-e0c5-4ae5-9fe2-6a55c983dcd8`; E6489: candidate_ref=null, uncertain=true | E6448: `989c615c-30b1-4943-ac38-0ca2077029d7` | E6455: `b8efe083-a323-4d16-be67-6fa5ebd6112d`, NO_MATCH | 질문 한도 후 중단 |

두 번째/세 번째의 HT001~010과 D새비교 read는 actual cache hit다. 원문 lookup/read 자체가 생략된 무료 정답은 아니다. 두 추가 frame E5837/E6438은 root input ID를 유지했지만 같은 selector와 두 goal을 반복했다. 두 번의 사용자 답변에는 새 기준/정답 ID가 없었다.

부분 선택 모델은 입력으로 주어진 배경 Tasks에서 실제 들은 기준을 식별할 수 없다며 세 번 모두 null/uncertain을 반환했다. 이는 정확한 정답 후보가 조회 집합에 없다는 관측과 일치한다. 부분 모델의 정상적 불확실성 표시를 “모델 API 실패”나 malformed output으로 판정하지 않는다. 부분 모델은 다른 selector/goal로 바꿀 권한이 없으며 이 제한은 순수 B의 현재 구현 범위다. 개선된 조회를 위한 frame 수정은 답변 이후 frame 생산 모델에게 남았고 이번에는 수정되지 않았다.

### 3.3 코드 질문, 중단과 전달

두 질문은 동일하게 “다음 후보 중 어떤 대상이고 무엇을 변경할까요? HT001; HT002; HT003; HT004; HT005; HT006; HT007; HT008”이었다. 최근의 들은 기준을 확인하는 질문으로 구체화되지 않은 일반 template다. 이미 제공된 E새 handle이나 견적 문서를 code가 정답 질문/selector로 추가하지 않았다.

| Publication / 실제 input | Text/Voice 및 actual receipt |
| --- | --- |
| 첫 질문 `P:1:3f70d2dd-49dd-4d54-896c-fa5392c3eb10`; question `e28e9be6-33cb-4296-96c6-a99fdd2e9a2e` | Text 8.149019초; Voice 8.406922~21.912781초 전체; receipt=true. 답변 input `input:88d3d356-7cbd-4f29-927a-86768270118e`, speech end 25.921256초. |
| 둘째 질문 `P:2:a5b77542-2914-4d91-a55f-06f49dce4cec`; question `df4e3b8d-4345-4550-8e9e-e9e39c105c71` | Text 30.725409초; Voice 30.981830~44.487729초 전체; receipt=true. 답변 input `input:468b5d50-9a82-482b-8977-63e257b8534f`, speech end 48.497901초. |
| 실패 안내 `P:3:22c67586-e5c6-48b7-8aca-ad06f865d827` | Text 54.909295초; Voice 55.161834~61.662922초 전체; “해당 업무는 UNRESOLVED_FIELDS로 인해 미해결 상태입니다.”; receipt=true. |

각 답변은 “새로운 조건이나 권한은 추가하지 않았어. 처음 요청대로 처리해줘.”이며 actual question 전달 뒤 1초 대기와 3초 발화가 적용됐다. 질문 내용이 달라진 것을 정답 답변으로 보정하지 않았다.

E6493 / 54.003초의 코드 해결 중단은 r-baseline `AMBIGUOUS_OR_INVALID_PARTIAL`, r-comparison-result NO_MATCH와 frame의 진단 unresolved를 남겼다. E6496 / 54.005초에 goals=[]/unresolved=UNRESOLVED_FIELDS의 실패 의미가 채택됐다. canonical meaning SHA-256 `693255f85bc1a716936b793eef6ffd6bc68225017ccc203a647623ba2ef45c48`다. `effects=[]`, 새 Task/Agent command/제안서 artifact가 없다. 실제 유한 연산을 code가 다른 연산으로 실행한 증거도 없다.

### 3.4 실제 호출

| 원장 line / role | 실제 호출 / 출력 종류 | Context / Output |
| --- | --- | --- |
| 29 / CINTERPRET | `115d3965-d24b-4397-abdf-6215a64e8c07`<br>B_FRAME; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/115d3965-d24b-4397-abdf-6215a64e8c07.json)<br>[Output](../workers/B1/model-outputs/115d3965-d24b-4397-abdf-6215a64e8c07.json) |
| 30 / CINTERPRET | `e7e0e0c4-c32a-46c2-bb52-a8bb77e7ee93`<br>B_PARTIAL_SELECT / uncertain=True; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/e7e0e0c4-c32a-46c2-bb52-a8bb77e7ee93.json)<br>[Output](../workers/B1/model-outputs/e7e0e0c4-c32a-46c2-bb52-a8bb77e7ee93.json) |
| 31 / CINTERPRET | `9455bb01-0ce4-4425-9979-c5645864212e`<br>B_FRAME; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/9455bb01-0ce4-4425-9979-c5645864212e.json)<br>[Output](../workers/B1/model-outputs/9455bb01-0ce4-4425-9979-c5645864212e.json) |
| 32 / CINTERPRET | `0b606c69-ca46-4764-861f-dcf0f075d455`<br>B_PARTIAL_SELECT / uncertain=True; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/0b606c69-ca46-4764-861f-dcf0f075d455.json)<br>[Output](../workers/B1/model-outputs/0b606c69-ca46-4764-861f-dcf0f075d455.json) |
| 33 / CINTERPRET | `75a84df2-3f94-4626-8352-23b6e6e4c5a3`<br>B_FRAME; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/75a84df2-3f94-4626-8352-23b6e6e4c5a3.json)<br>[Output](../workers/B1/model-outputs/75a84df2-3f94-4626-8352-23b6e6e4c5a3.json) |
| 34 / CINTERPRET | `c7d579e8-45b7-4860-a5c3-18b6b21ce499`<br>B_PARTIAL_SELECT / uncertain=True; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/c7d579e8-45b7-4860-a5c3-18b6b21ce499.json)<br>[Output](../workers/B1/model-outputs/c7d579e8-45b7-4860-a5c3-18b6b21ce499.json) |
| 35 / CRESPONSE | `80b7864c-324c-49cf-8431-a669e95d1909`<br>CRESPONSE; HTTP 200, attempt 1 | [Context](../workers/B1/model-contexts/80b7864c-324c-49cf-8431-a669e95d1909.json)<br>[Output](../workers/B1/model-outputs/80b7864c-324c-49cf-8431-a669e95d1909.json) |

CINTERPRET 6회는 frame 3회+부분 선택 3회다. CRESPONSE 1회, 모두 HTTP 200/attempt 1이다. 실제 모델/조회/확인 질문의 추가 사용량이 보존돼 있으며 일괄 추정 호출 수로 대체하지 않는다.

## 4. A1/B1 인과 차이와 공통 한계

| 관점 | A1 S2-02 | B1 S2-02 | A1 S2-05 | B1 S2-05 |
| --- | --- | --- | --- | --- |
| 모델 목표 | 새 CREATE_PROPOSAL | 새 CREATE_PROPOSAL이나 잘못된 배경 Task 참조 | DIRECT_RESPONSE/COMPARE | NEW_TASK COMPARE→CREATE_PROPOSAL |
| 실제 기준 확보 | E1 미조회 | E1 Text/실제 Voice 범위 확보 후 identity 선택 | E새 미조회 | E새 미조회; RECENT_TASK로 잘못 조회 |
| 문서/결과 확보 | 모델 read는 D2 두 번; 실제 payload는 제공된 D2/D3 handles에서 원문 공급 | THT193/THT194 NO_MATCH | D새비교/D새견적 정확 read | D새비교만; 아직 없는 새 결과를 TASK_RESULT에서 조회 |
| 후속 의미 처리 | 모델 전체 의미 재판단; version 오류 후 재호출 | 부분 선택+code state; clarification 후 input_ref 거부 | 모델 전체 의미; version 오류 후 재호출 | 부분 선택 정상 uncertain; 추가 unbound frame 반복 |
| 실제 효과 | 새 작성 명령 1회, 조건 누락 payload 유지 | 명령 0/실패 의미 | 명령 0/직접 응답 | 명령 0/실패 의미 |
| 실제 전달 | 접수 안내 전체, final Text 전체/Voice 일부 중단 | 질문/실패 안내 전체 | 직접 비교 전체 | 질문 2회/실패 안내 전체 |

- **모델/selector 선택:** 실제 주어진 T비교/T견적/E새/D새견적 handles를 놓친 경우, 없는 locator 생성, 직접 응답으로 목표 변경, 과거 Task 결과와 현재 goal의 미래 결과 혼동을 실제 model output에 연결한다.
- **공통 lexical owner 제한:** 한국어/영어 의미 검색을 보장하지 않는다. exact locator는 별도이며 query 기본값/limit/검색 pool의 차이를 남긴다. 배경 후보가 잘못 선택되는 것을 정답 Task/source로 치환하지 않는다.
- **유한 계약의 지원 한계:** 현재 B partial은 identity 후보 선택만 하고 목표/조건 값 전체를 재해석하지 않는다. code 확인 질문은 label template이며 domain 근거를 잘 묻는다는 보장이 없다. 두 사례는 새 연산 부재가 직접 중단 원인은 아니지만 전체 조건 해결 능력이나 모든 연산 조합의 완전 지원을 입증하지도 않는다.
- **코드 결함 판정 제한:** 실제 없는 locator NO_MATCH, null/uncertain 후보 유지, frame identity fence 및 unresolved 중단은 코드의 기록된 규칙에 부합한다. 유효한 CREATE_PROPOSAL을 다른 목표로 바꾸거나 정답 payload를 보정한 증거는 없다. 다만 일반 질문의 의미 부족과 query 전달/default 규칙은 후보 코드/공통 구현의 설명 가능한 제한으로 공개해야 한다. 이를 LLM의 오답 하나로만 묶어 숨기지 않는다.

전달 부족과 정상 실패 안내를 포함해 각 condition의 실제 facts는 이후 평가자가 판정해야 한다. 이 표는 정확성/시간 점수표가 아니고 한 반복의 실제 경로 대조다. 두 실패 경로 때문에 B가 항상 실패하거나 A가 본질적으로 우월하다는 결론은 나오지 않는다.

## 5. 증거 hash와 검수 범위

관련 11개 실제 호출의 Context/Output artifact 22개 canonical hash, 두 stable trial raw 행과 meaning hash를 검증했다. 명령이 없으므로 payload hash를 만들어 채우지 않았다. artifact hash는 JSON의 `ensure_ascii=False`, `sort_keys=True`, `separators=(',', ':')` 직렬화를 사용했다. `context.sha256` artifact hash와 별도 caller fingerprint `context_sha256`는 다른 필드다. append 중인 원장 전체의 최종 hash를 주장하지 않는다.

```text
S2-02 CINTERPRET 59089d54-9c5d-4378-a42d-075f3cba5b38
  context_artifact_sha256=9b8a6783e3ce83af283505228d465420749df30ff237b6b829aa602683485da4
  output_artifact_sha256=29899cc4c6ee45784834e7270f4ea19997553f7de48a5e19a2f04d3cc860e076
S2-02 CINTERPRET 3c0f7b3e-7a3b-48d3-baf8-d74d66d2398f
  context_artifact_sha256=7f836cac4c5e24b22cdc9a0b561b0b64f496e1b4441e745b11a39abaebdc6eee
  output_artifact_sha256=0ea74f9d76736a3328f53ccca0e0a416bd8ecd54177d61dc8699a8b92d3c4024
S2-02 CINTERPRET a052615b-7bb3-4035-90d5-6b424aec904e
  context_artifact_sha256=4eb78d50265cb032ab432c23fda69e74a5655423876a05c286e3ae5d656c1da2
  output_artifact_sha256=65e201c077b3dae09fe00b4773db94349a85f13e5a53ccf7ec030b4fc56638e2
S2-02 CRESPONSE bf8908d4-0349-474a-9c7f-11507f6d4393
  context_artifact_sha256=f0f786e1c1b8bff3539c465a2fa74b93905f2d800fc00c948734d04b66631910
  output_artifact_sha256=f9e4c4287c3476116d99c9d93ebc1cfb77039077053341d9315fe8c16b4b3fec
S2-05 CINTERPRET 115d3965-d24b-4397-abdf-6215a64e8c07
  context_artifact_sha256=f863cc41387f2b8a2ccda6665aac37ced6a76d7d479e960b791e0eed19e50e08
  output_artifact_sha256=e74a09a30c2eb31df834f46623eb1c0ff5f310ab12f425fc65d8dd3ff525bf82
S2-05 CINTERPRET e7e0e0c4-c32a-46c2-bb52-a8bb77e7ee93
  context_artifact_sha256=3a4dda793a7e63369cd1885b46d6748a84c1505bb72a7f30ff64855bb8d0b34d
  output_artifact_sha256=c0b0b7244a1100ef22640299deaace6a88d2a87c617d0ede7da32a93724621cf
S2-05 CINTERPRET 9455bb01-0ce4-4425-9979-c5645864212e
  context_artifact_sha256=3957750e2902c8bb47c75be3237ed8b88c68bfa6e0e9accdd57ec1ba60721473
  output_artifact_sha256=977882e545c7b7c944d865777b6fa55a73b355e38e7b32afb4fbf209ab704def
S2-05 CINTERPRET 0b606c69-ca46-4764-861f-dcf0f075d455
  context_artifact_sha256=e51bfce21006649a6055764eab0b28e5ffd07c2aa7dc358780f367fd3cf5869f
  output_artifact_sha256=8406895adf1ff2969b6c8f26a7f9c5e8a502bb9f26096f81da2ffb46cef31094
S2-05 CINTERPRET 75a84df2-3f94-4626-8352-23b6e6e4c5a3
  context_artifact_sha256=dc0e6d90ec640b6be4e5eb94b5da88df632edb5364e61653377890a8fe773b87
  output_artifact_sha256=5a20684a3f85fcac1615db14488359729af231f92c31fe318acca80599896105
S2-05 CINTERPRET c7d579e8-45b7-4860-a5c3-18b6b21ce499
  context_artifact_sha256=2773757f0ddd0e0476bf7fd412fc3ae3ed98684238a661dd93481f38dac03c79
  output_artifact_sha256=f2b33cc751949f956505209a2aa9e8ed4ac5680dd5f4220a9e1eecc50fbab0ce
S2-05 CRESPONSE 80b7864c-324c-49cf-8431-a669e95d1909
  context_artifact_sha256=4526cbe68e0dbb634a7adb8d140f50ca0a0a113bfeabc2ff376fbb1ea8e88f18
  output_artifact_sha256=34dc9cfe47e6732f846a59d643a431e85e2c5a40d766860139cee236ebb83259
```

[events](../workers/B1/events.jsonl), [inputs/source](../workers/B1/inputs-and-source.jsonl), [deliveries](../workers/B1/deliveries.jsonl), [model calls](../workers/B1/model-calls.jsonl)는 future schedule나 oracle로 후보 출력을 보정하지 않은 실제 기록이다. OS memory 원장과 큰 원문 buffer는 이 검토에서 읽지 않았다. QA 점수, 재실행, 개선안 적용과 B 나머지 반복의 일반화는 별도 작업이다.
