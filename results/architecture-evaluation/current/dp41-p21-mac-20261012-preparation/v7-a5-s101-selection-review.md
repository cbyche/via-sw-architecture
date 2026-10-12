# V7 A5 S1-01의 실제 직접 응답 선택

상태: READ_ONLY_NARROW_ACTUAL_TRACE_REVIEW. 완료된 A5 S1-01 행과 두 actual model Context/output, 이전 A1/A2/A4의 해당 제안 및 동결 response port만 읽었다. 코드/정답/fixture 변경, 테스트/API 실행, 전수 hash 및 CPU/메모리 검사는 하지 않았다. QA 수치/점수나 후보 우열은 판정하지 않는다.

A5 trial E397, Request `R:a390e947-42c0-4739-9b82-eeabae65de6b`, Input `I:d6fc6f24-9026-4d82-b341-d65686391485`의 발화는 “D1에 적힌 마감일 알려줘.”였다. 첫 CINTERPRET은 EXPLICIT_SOURCE D1/DOCUMENT/limit1을 요청했다. E105 owner read `5c46a42b-e220-4d03-a49c-376dd89e88eb`는 actual D1@v1 원문 “제출 마감일은 2026년 10월 15일입니다. 담당자는 민수입니다.”를 OK로 반환했다. 두 번째 CINTERPRET Context에도 이 원문과 read ID가 있었다.

모델의 E154 PROPOSE는 `goals=[]`, `response="D1에 적힌 마감일은 2026년 10월 15일입니다."`, `evidence_refs=[actual read ID]`였다. E157은 이를 채택했다. [동결 runtime L730~735](../dp41-p21-mac-text-v7-20261012/frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py)의 candidate_response 경로가 모델의 실제 문자열을 그대로 사용한다. CRESPONSE 호출은 추가되지 않았다. 최종 publication `P:1:28973a97-9dbd-49f3-8d59-1a052b69ec07`는 같은 Text/Voice 문자열과 delivered_receipt=true를 기록했다. E160 TEXT_PRESENTED 시점의 voice_text 빈 값 및 receipt=false는 초기 snapshot이며 이 최종 receipt와 구별한다.

| 반복 | 실제 read 후 제안 | 실제 채택/전달 경로 |
| --- | --- | --- |
| A1 | goals=[], response=null | 채택 뒤 기존 NO_ACTION response 경로 |
| A2 | ANSWER_QUESTION, question_ref=현재 Request ID | binding provided 검증에서 채택 전 거절, 전달0 |
| A4 | ANSWER_QUESTION, question_ref=현재 Input ID | binding provided 검증에서 채택 전 거절, 전달0 |
| A5 | goals=[], 실제 response 문자열 | 채택 뒤 기존 candidate_response를 그대로 전달 |

선택한 네 actual CINTERPRET SYSTEM 문자열은 같았다. model=qwen3.6-35b-a3b, temperature=0, max_tokens=4096, response_format=json_object, enable_thinking=false도 동일했다. source/freeze/common port가 바뀐 증거는 없으며 관측 차이는 모델의 실제 제안 선택에 있다. 원시 Context의 Request/Input/read ID 및 모델이 고른 query/selector는 다르므로 네 호출이 완전히 같은 wire Context였다고 주장하거나 출력 변동의 통계적 원인을 확정하지 않는다.

아래 SHA는 actual call 원장에 저장된 artifact digest이며 이번 timing 중 다시 계산하지 않았다.

| A5 actual call | Context artifact SHA | output artifact SHA |
| --- | --- | --- |
| read 924a5034-6e6b-42e2-ae3f-e10a93d5993e | c05c06dc2f2b73e5b7b0936a4e58e959988f0147ef026577cf203a8cebe2ca68 | 0e7afbaa202890bfaf8cd0405ee156e4edfda38d8e8d69f6a41c905b1c60e372 |
| propose 951a692d-f436-4f1d-8c01-c9324b8d8466 | 1ccb706078f9debe7d436d4764aeb8a3a58d5ed94d06a9ae9a51576b0ec1ace0 | a6240738ad84d856afec093608c6760900dd62ce06721902f7db36382e1436ec |

원시 근거: [A5 trial](../dp41-p21-mac-text-v7-20261012/workers/A5/trials-unjudged.jsonl), [read Context](../dp41-p21-mac-text-v7-20261012/workers/A5/model-contexts/924a5034-6e6b-42e2-ae3f-e10a93d5993e.json), [read output](../dp41-p21-mac-text-v7-20261012/workers/A5/model-outputs/924a5034-6e6b-42e2-ae3f-e10a93d5993e.json), [propose Context](../dp41-p21-mac-text-v7-20261012/workers/A5/model-contexts/951a692d-f436-4f1d-8c01-c9324b8d8466.json), [propose output](../dp41-p21-mac-text-v7-20261012/workers/A5/model-outputs/951a692d-f436-4f1d-8c01-c9324b8d8466.json), [A1 검토](v7-a1-completed-port-review.md), [A2/A4 참조 오류 검토](v7-a2-a4-s101-reference-review.md).

이 좁은 관측에서 새로 확정된 공통 P1/P2는 없다. 원래 후보의 실패와 실제 직접 응답을 각각 보존하며 QA 판정은 이후 judge/evidence audit에서 수행한다.
