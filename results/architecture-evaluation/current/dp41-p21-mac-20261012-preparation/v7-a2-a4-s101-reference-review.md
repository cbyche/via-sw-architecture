# V7 A2/A4 S1-01의 전달 미발생 원인

상태: READ_ONLY_NARROW_ACTUAL_TRACE_REVIEW. A2/A4의 완료된 S1-01 trial과 두 actual Context/output, 동결 prompt/contract/adoption port만 읽었다. 코드/정답/fixture 변경, 테스트/API 실행 및 전수 검사는 하지 않았다. QA 점수나 A/B 우열은 판정하지 않는다.

| worker | actual Request/Input | read 및 제안 | 최초 실행 거절 |
| --- | --- | --- | --- |
| A2 | R:e6449098-165f-4bf8-98c5-d6aa45520bbb / I:b38f7d79-1e9a-4e14-8544-11aa228601ec | E100 EXPLICIT_SOURCE D1, read=d5fd40fc-db84-4f0b-a2f3-e0659beab72b/OK; E158 PROPOSE | E160 REQUEST_FAILED: Binding reference was not provided: R:e6449098-165f-4bf8-98c5-d6aa45520bbb |
| A4 | R:1705fd05-4aa3-4d90-88d5-ce9573da05d4 / I:449a3c98-56ca-4c9a-8a57-4287f5efa972 | E108 SOURCE_AT_REV D1v1, read=3158c522-ad5b-425c-bd08-c122bf16cc69/OK; E167 PROPOSE | E169 REQUEST_FAILED: Binding reference was not provided: I:449a3c98-56ca-4c9a-8a57-4287f5efa972 |

두 모델은 실제 D1@v1 원문 “제출 마감일은 2026년 10월 15일입니다. 담당자는 민수입니다.”를 받았다. 첫 호출은 지원된 원천 read이며 두 번째 호출의 answer 문자열도 그 날짜를 포함한다. 그러나 직접 응답 대신 kind/operation=ANSWER_QUESTION을 선택하고 question_ref에 각각 현재 Request ID와 Input ID를 넣었다. actual metadata.questions=[]였으므로 둘 다 owner Question handle이 아니다. 모델은 target=null 및 consent.read=true도 냈다.

[동결 validate_adoption](../dp41-p21-mac-text-v7-20261012/frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py)의 source_refs/recipient_ref/question_ref provided 검증에서 거절됐다. 두 trial의 adopted meanings=[], effects=[], deliveries=[]이며 CRESPONSE 호출도 없다. 뒤의 Existing Task target/Question authority/consent 검사에 도달하지 않았으므로 target와 consent를 최초 거절 원인으로 앞당기지 않는다. 모델 내부 answer에 맞는 날짜가 있어도 실제 전달로 만들지 않는다.

실제 전송 SYSTEM에는 DIRECT_RESPONSE, Goal={id,kind,operation,target,arguments,instruction}, 기존 Task target 및 new/direct null 규칙이 있다. READ_FIELD의 `source_refs:[ref]`와 finite field=DEADLINE/AMOUNT/VALUE/STATUS, ANSWER_QUESTION의 `question_ref,question_revision,answer,properties,consent`도 명시돼 있다. actual Context에는 D1의 ID/revision/권한 metadata, 실제 read receipt의 ref/content/read_id가 공급됐다. [동결 validate_document](../dp41-p21-mac-text-v7-20261012/frozen-source/prototypes/candidates/dp41_45_poc/dp41/contract.py)는 같은 READ_FIELD enum을 받는다. 이번 모델은 READ_FIELD payload의 필수 field를 잘못 채운 것이 아니라 다른 operation 및 유효하지 않은 Question binding을 선택했다.

따라서 현재 관측은 후보의 operation 의미 선택 및 wire reference 오류로 분류한다. READ_FIELD 지원 한계나 필수 스키마 안내 누락으로 확정할 증거는 없다. response 필드 자체의 계약 오류도 이번 최초 거절이 아니며 실제 output은 response=null이었다. 추가 프롬프트 설명이 모델 선택에 도움이 될 가능성과 공통 구현 계약 결함은 구별한다. 정상 실패를 INVALID로 바꾸거나 source/response를 정답으로 보정하지 않는다.

아래 artifact SHA는 실제 원장 값을 인용했고 timing 중 재계산하지 않았다.

| worker/호출 | Context artifact SHA | output artifact SHA |
| --- | --- | --- |
| A2 read 3f8fa085-9847-475b-8c1a-3e8a8b8df42d | e1d5d77f4e769fb3300819be8d35965e2cd569ca1ba813cd1c4ed17f6ab2ab65 | 31ddbcc8292269cdc993782f97520a26b367cb805befdd60301679aa99e90e3d |
| A2 propose d94c46f9-8e64-4713-be46-767c489351a9 | ca2b061b95ae96baf8fa18f46b6484a2c14db8041f89eed554970dcbdc93e0c6 | 97a3468b466b84bbf4ae022a944efad38cef2f5a0371ac3941afde631446e5f7 |
| A4 read cd44db4e-f3fb-4737-b0ed-6e58f6e320b7 | b1a3672b5400857d941030b7edb2168a32e057a41c3e82c5505df08fd345706c | 93c08c8d31c6b1a0bc06daabd7db94c4556f0050ad60e676167c76a85841df0f |
| A4 propose d03eee79-56fb-4843-99c8-6e12cbe9bde8 | 1c2593803a7903db538f755c8ea1bbfd20ef9339d8d78113a87f5b8db6c1bbb4 | 0e8ed9e4e9c9b0be86c2eeb3e70943ba0e851e942ce5b9e506b476d01691b1ab |

원시 근거: [A2 trial](../dp41-p21-mac-text-v7-20261012/workers/A2/trials-unjudged.jsonl), [A4 trial](../dp41-p21-mac-text-v7-20261012/workers/A4/trials-unjudged.jsonl), [A2 propose Context](../dp41-p21-mac-text-v7-20261012/workers/A2/model-contexts/d94c46f9-8e64-4713-be46-767c489351a9.json), [A2 propose output](../dp41-p21-mac-text-v7-20261012/workers/A2/model-outputs/d94c46f9-8e64-4713-be46-767c489351a9.json), [A4 propose Context](../dp41-p21-mac-text-v7-20261012/workers/A4/model-contexts/d03eee79-56fb-4843-99c8-6e12cbe9bde8.json), [A4 propose output](../dp41-p21-mac-text-v7-20261012/workers/A4/model-outputs/d03eee79-56fb-4843-99c8-6e12cbe9bde8.json), [동결 prompt](../dp41-p21-mac-text-v7-20261012/frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py).

이 좁은 대조에서 새로 확정된 공통 P1/P2는 없다.
