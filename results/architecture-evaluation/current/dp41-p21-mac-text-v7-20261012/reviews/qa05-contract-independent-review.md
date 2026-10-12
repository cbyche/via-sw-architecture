# V7 QA05 선택 변경 작업의 독립 기능 책임 검토

검토자: `dp41-contract-independent-reviewer`. 변경 patch 저자는 아니다. 검토 중 source, frozen evidence와 원래 result를 수정하거나 테스트 및 모델 API를 실행하지 않았다. 모든 U01~U18의 변경 여부, 전후 계약, source 위치와 증거를 [독립 reviewer JSON](qa05-contract-independent-review.json)에 기록했다.

고정한 V7 53파일의 digest는 `44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce`다. 선택 작업의 baseline snapshot digest `935fab8816a88ca514572ffe83c61015b291a343eaf71304126f3c9862845f5c`는 같은 파일의 SHA256에 bytes/mode도 포함한 별도 계산이다. 모든 파일 SHA256, 각 작업의 실제 after source와 manifest, changed paths 및 patch SHA256을 직접 대조하여 일치를 확인했다.

이 독립 판정에서는 양안의 변경 책임 수가 같다. 최종 결과 집계에는 root의 별도 독립 검토 및 hash로 고정한 선택 manifest가 필요하다.

| 과제 | A 변경 책임 | B 변경 책임 | A 수 | B 수 |
| --- | --- | --- | ---: | ---: |
| M01 모델 provider wire/usage 변경 | U15 | U15 | 1 | 1 |
| M02 외부 Agent protocol 변경 | U16 | U16 | 1 | 1 |
| M03 일정 원본 구조 변경 | U09 | U09 | 1 | 1 |
| M04 보고서 실패 후에도 원문으로 초안 실행 | U01, U03, U07, U08 | U01, U03, U07, U08 | 4 | 4 |
| M05 실제 제시 질문의 허용 scope token | U04, U07 | U04, U07 | 2 | 2 |
| M06 원본 선호 등록 및 사용 fence 감사 | 없음 | 없음 | 0 | 0 |
| 합계 / 평균 / 최대 | 9 / 1.5 / 4 | 9 / 1.5 / 4 | | |

M01, M02, M03, M05, M06의 실제 A/B patch는 각각 byte 동일하다. 파일 수를 세지 않고 [고정 책임 registry](../frozen-source/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md:635)의 기능 경계를 적용했다. 모든 12개 작업의 recorded P21 21사례 재생 및 명명된 요구 회귀가 통과했다. 재생은 A1 또는 B1의 기존 입력, 읽기, 모델 출력과 실패를 보존하며 새 모델 정확성을 측정하지 않는다. S5-01과 S5-02는 의미 호출이 없는 외부 사건 실행 경로로 재생한다.

M01의 [provider encode/decode](../changes/CH-DP41-A-M01/source/prototypes/candidates/dp41_45_poc/shared/adapters.py:23)는 바뀐 wire를 기존 상위 port로 정규화한다. 실제 어댑터 관측에 Text/image 입력, 원래 잘못된 text, cached token, 늦은 usage와 usage unknown 보존이 있다. 공통 permission 철회 후 늦은 모델 반환 거절도 실행됐다. 이 증거는 API 없는 wire/실행 회귀다. 새 외부 provider에서 실제 원격 cancellation이나 결제 행동을 검증했다는 주장은 하지 않는다.

M04 A는 [현재 producer predicate](../changes-alternatives/ORIGINAL-A-LINEAGE-v1/CH-DP41-A-M04/source/prototypes/candidates/dp41_45_poc/shared/runtime.py:653)를 실제 Request ID/revision과 producer Goal에서 실제 Task/Execution/Task revision에 연결한다. B의 [유한 tagged reference validation](../changes-alternatives/TYPED-GOALREF-v2/CH-DP41-B-M04/source/prototypes/candidates/dp41_45_poc/dp41/contract.py:143)은 기존 owner string과 exact singleton `{producer_goal_id: from_goal}`을 구별한다. [기존 B binder](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/b.py:60)는 이 object를 보존하며 byte가 바뀌지 않았다. 따라서 U02 추가 변경을 세지 않는다. 실제 Frame 검증, owner read, binding, 성공/실패의 초안 각 1회, 실패의 AFTER/USE_RESULT 차단, SEND 0회, 기존 `GOAL:report` owner string 및 서로 다른 Request revision의 producer 구분을 관측했다. 원래 M04 A/B의 lineage 충돌 실패는 [별도 원시 증거](../changes-supplemental/original-M04-lineage-collision-v1/summary.json)로 보존하고 집계에서 제외한다.

M04의 실제 [FAILURE ingress](../changes-alternatives/TYPED-GOALREF-v2/CH-DP41-B-M04/source/prototypes/candidates/dp41_45_poc/shared/runtime.py:1367)는 U07, producer 연결과 후속 readiness는 U08이다. 기존 응답 port로 실패 사실을 보내는 호출만으로 U06을 추가하지 않았다. 정규화 대화/작업 port가 유지돼 U13도 추가하지 않았다.

M05의 [Task 질문 token 발급과 검사](../changes/CH-DP41-A-M05/source/prototypes/candidates/dp41_45_poc/shared/adapters.py:162)는 U07, [실제 presented token 반영](../changes/CH-DP41-A-M05/source/prototypes/candidates/dp41_45_poc/shared/adapters.py:235)은 U04다. [snapshot/restore](../changes/CH-DP41-A-M05/source/prototypes/candidates/dp41_45_poc/shared/adapters.py:253)는 질문/receipt 전체를 보존하는 기존 코드다. 별도 schema 이행이나 restore 알고리즘의 diff가 없으므로 U14를 더하지 않는다. A의 실제 publication PID 94201과 restore PID 94204, B의 PID 94205와 94206에서 같은 token과 snapshot hash를 보존하고 corrupt token 거절, ANSWER 명령 1회, 중복 거절을 확인했다. [A 보조 기록](../changes-supplemental/CH-DP41-A-M05-PUBLISHED-TOKEN-RESTART-v1/result.json), [B 보조 기록](../changes-supplemental/CH-DP41-B-M05-PUBLISHED-TOKEN-RESTART-v1/result.json).

M06은 빈 diff다. V7 기준에서 실제 원본 `M언어`, current-user 지속 선호, 권한, 공통 read와 receipt가 이미 지원된다. [원본 S1-03 supplemental](../changes-supplemental/M06-EXACT-S103-v3/summary.json)의 A/B before와 after 모두 C1~C6 및 사용자 구분, owner ref/revision, 파생 삭제, 정직한 receipt와 다른 선호 보존의 10조건을 통과했다. `response=None`에서 실제 등록 facts를 공통 응답 port에 전달한 뒤 receipt가 생겼으며 공통 read는 실제 owner source를 반환했다. 지정 semantic output을 쓴 기능 증거이며 실제 모델의 선호 해석 정확성은 아니다. V7 고정 전의 기준 누락 보완을 M06 새 변경량으로 가산하지 않는다.

기능 책임 수용 비용과 실제 모델의 새 자연어 성공은 구별해야 한다. [M04 A actual 모델 실행](../changes-semantic/M04-A-attempt1/change-semantic-validation.jsonl)은 실제 보고서 FAILURE와 D1 원문을 처리했지만 잘못된 owner 참조 및 preservation을 생산하여 초안 0회로 실패했다. [M04 B actual 모델 실행](../changes-semantic/M04-B-attempt1/change-semantic-validation.jsonl)은 잘못된 방향/참조 Frame을 거절한 뒤 확인 답 없이 초안 0회로 실패했다. 두 실패와 알려진 usage 비용을 그대로 보존했다. 이 독립 검토의 4개 책임 판정은 유한 실행 계약의 지정 출력 기능 검증에 관한 것이며 새 자연어 요구가 실제로 성공했다는 결론이 아니다. 두 actual 관측은 P21의 정확성/시간/비용 모집단에 추가하지 않는다.

선택 12개 실제 변경과 위 증거 범위에서 미해결 P1/P2는 발견하지 않았다. 제품 전체 유지보수 비용, Windows 동작과 실제 음성 모델은 이 검토의 관측 범위 밖이다.
