# v5 전 query와 질문 label 연결 및 P21 포트 독립 리뷰

상태: **READ_ONLY_INTEGRATION_REVIEW / NO_NEW_P1_P2**. v4의 확정 query-mapping P2와 실제 candidate label 연결 수정은 선언된 포트 계약을 복구한다. 후보의 의미 선택 전략, finite operation/selector 집합, Goal/value partial 지원 범위와 공통 원천/정답은 바뀌지 않았다. 작성자가 실행한 무상 모사 회귀 코드를 읽었으며 테스트, API와 측정은 이 리뷰에서 실행하지 않았다. 실제 의미 정확성, 검색 relevance의 개선이나 QA 점수는 판정하지 않는다.

## 1. 검토 기준과 최종 source

이 리뷰는 [v4 B1 실제 경로 관찰](../dp41-p21-mac-text-v4-20261012/reviews/observed-paths-B1.md)에서 확인한 query 전달 및 질문 표시 연결을 기준으로 한다. live 후보 파일의 final hash는 작성자 편집 종료 통보와 직접 대조했다. 비교 기준 [v4 manifest](../dp41-p21-mac-text-v4-20261012/manifest.json)의 source digest는 `c2dfc26fa6afbce8df4af3405913031ca440fefa150d6a0739c010b3ed4796a7`다.

| 최종 파일 | SHA-256 |
| --- | --- |
| [prototypes/candidates/dp41_45_poc/dp41/contract.py](../../../../prototypes/candidates/dp41_45_poc/dp41/contract.py) | `b0fff4b090a1128000e434d6150e3aa703468cadf5dea87c8b2c3955ec945ed2` |
| [prototypes/candidates/dp41_45_poc/dp41/b.py](../../../../prototypes/candidates/dp41_45_poc/dp41/b.py) | `4d5ae5fbc7f94692bfbc1eddac391800b0ca4a72808c9d5d265ce7e4874b778f` |
| [prototypes/candidates/dp41_45_poc/dp41/tests/test_dp41.py](../../../../prototypes/candidates/dp41_45_poc/dp41/tests/test_dp41.py) | `fafcc6fb1bb800c626f4c5d7728060b1e583a862cdf95d67649ca010e68064db` |

`contract.py`와 `b.py`의 차이, `test_dp41.py`의 새 5개 회귀를 고정 v4 사본과 대조했다. 원 Frame/Meaning의 구조, 모델 prompt, budget, B의 부분 판단 범위, 조회 병렬화, 코드 binding과 runtime adoption/command는 그대로다. live `campaign.py`는 campaign_contract 문자열을 v4에서 v5로 바꾼 한 줄만 추가로 다르며 후보 처리 변경이 아니다.

v4 `frozen-source` 50개와 `evaluator-tools` 50개를 manifest의 각 SHA-256에 직접 대조했다. 두 사본 모두 missing=0, mismatch=0이다. 이전 trial/model Context/출력은 수정하지 않았다. v5 이후 값으로 v4 원장을 덮어쓰거나 과거 후보 결과를 보정하지 않는다.

## 2. 두 수정의 연결 및 회귀 검사 검토

query 우선순위는 명시된 top-level reference.query → selector.query → mention → 빈 문자열이다. 명시적인 빈 query는 빈 문자열을 유지한다. selector.query의 문자열 타입은 top-level query가 있어도 검사한다. COMMON/B prompt 및 validate_read에서 허용했던 nested query를 실제 read envelope에 보존하는 변경이며 새로운 검색 추론이나 정답 source 선택을 더하지 않는다. Owner는 여전히 전달받은 query를 같은 lexical ranking/limit/권한 규칙으로 처리한다. 검색 coverage와 의미 relevance 개선은 실행 전 미정이다.

질문 label은 기존 read item의 metadata와 top-level item을 같은 방식으로 합치고 title, instruction, name, 실제 content의 순서로 선택한다. 실제 channel content는 읽힌 view의 content.text/delivered_voice_text이며 새 정답 설명을 생산하지 않는다. 빈 label은 후보 번호를 표시하고 내용은 최대 120자 발췌와 생략 표시를 사용한다. opaque ref는 질문 label로 표시하지 않는다. question branch의 기존 동작, states, internal candidate_refs, partial Context의 실제 ref와 binding은 그대로다. 표시를 짧게 만든 이유로 B 구조의 필수 우위나 비용 절감을 주장할 수 없다. 8개까지 후보를 나열하는 방식과 재질문의 품질은 여전히 후보 구현의 관측 대상이다.

작성자는 DP41 25 / shared 64 / benchmark 64 unittest PASS 및 API 없음으로 보고했다. 리뷰어는 실행을 반복하지 않고 다음 새 검사와 assertion을 읽었다.

| 작성자 회귀 | 의미 있는 확인 |
| --- | --- |
| reference query 우선순위 6개 subcase | top/nested/mention 및 명시 빈 문자열 보존 |
| nested query 타입 검사 | None/list/dict/number/bool을 top-level query 유무와 무관하게 거절 |
| B Frame→실제 Owner→binding | RECENT_TASK, TASK_RESULT, RECENT_INTERACTION, PAST_DELIVERED를 실제 OwnerRepository로 읽고 limit=1의 실제 ref를 binding. 의미 모델은 지정 출력이며 의미 정확성 측정이 아님 |
| 실제 metadata/content 질문 label | Source title, Task instruction, PAST 실제 heard content를 표시하고 scope.candidate_refs는 원래 internal ref 유지 |
| 제목 없는 긴 내용 및 빈 Task | 실제 content의 120자 발췌, 후보 번호, 내부 ref 비노출 및 model 호출 추가 없음 |

## 3. P21 실제 read 포트 shape 전수 대조

완료된 v4 A1/B1의 각 21개 raw trial을 줄 단위로 읽고 OWNER_READ의 spec, item, metadata 및 content 형태를 대조했다. 측정 파일 중 memory.jsonl, 압축 background 원문, RGBA buffer는 읽지 않았다. 다음 숫자는 포트 shape의 실제 출현 횟수이며 호출 비용이나 QA 분모가 아니다.

| selector | A1 OWNER_READ | B1 OWNER_READ | 실제 shape 예 |
| --- | ---: | ---: | --- |
| CALENDAR_WINDOW | 2 | 0 | A1/S3-01 E6882; A1/S3-03 E8804 |
| CURRENT_QUESTION | 0 | 1 | B1/S6-02 E18558 |
| EXPLICIT_SOURCE | 5 | 7 | A1/S1-01 E105; A1/S2-01 E1608 |
| INPUT_SELECTION | 0 | 6 | B1/S2-01 E1327; B1/S2-01 E1333 |
| KNOWN_REF | 1 | 2 | A1/S6-03 E15822; B1/S1-02 E534 |
| PAST_DELIVERED | 1 | 2 | A1/S4-02 E10535; B1/S2-03 E4378 |
| RECENT_INTERACTION | 0 | 2 | B1/S2-02 E2766; B1/S2-05 E6300 |
| RECENT_TASK | 0 | 1 | B1/S5-03 E15566 |
| SOURCE_AT_REV | 3 | 1 | A1/S1-02 E621; A1/S3-01 E6878 |
| TASK_RESULT | 11 | 4 | A1/S2-02 E2143; A1/S2-02 E2147 |

실제 read item 공통 필드는 `ref, kind, revision, content, metadata, cache_hit, policy_epoch`다. 모델 metadata에는 source body가 없고 실제 OWNER_READ 이후 item.content 또는 metadata 원문에 존재한다. `read_items()`는 items 배열의 dict를 그대로 유지하고 B의 question 유효성 검사 및 label은 metadata를 합쳐 owner 상태/본문을 소비한다. reference_id는 최상위 실제 ref를 우선하며 opaque view의 내부 identity로 바꿔치기하지 않는다.

| 포트/경계 | producer 형식 및 실제 consumer 대조 | 결론과 한계 |
| --- | --- | --- |
| EXPLICIT_SOURCE, SOURCE_AT_REV | source metadata handle와 문자열 본문 → 실제 ref/revision; SOURCE_AT_REV는 요청 revision을 pin하고 최신 하나로 합치는 단계 제외 | 모델이 과거 revision을 명시 선택한 v4 A1 S3-02는 의미 선택 관측. 올바른 v1 요청을 v0으로 전달하는 shape 결함은 아님 |
| selector.query | 유효 nested 문자열 → 수정된 reference_spec.query → 동일 Owner.read query 소비 | v4의 전달 P2 복구. relevance 개선은 미측정 |
| CURRENT_QUESTION | 실제 Q item의 content=질문 문자열, metadata에 question_id/revision/task_id/execution_id/origin/scope/status/presentation 상태 | partial 필터가 metadata 상태를 확인하고 runtime이 실제 제시/철회/Task scope를 다시 검사. 모델이 질문을 잘 골랐는지는 별도 |
| KNOWN_REF의 supplied metadata | 이미 제공한 Task/질문/publication ID는 HOST_METADATA로 target/scope binding 가능. 원문 Source metadata는 실제 Owner.read 필요 | metadata body 제거와 사실 제공을 구별. Task identity를 알고 있다는 사실은 Task 결과 body를 읽었다는 영수증이 아님. 새 clarification 뒤 질문은 실제 read로 갱신 |
| PAST_DELIVERED | content에 actual Voice와 actual Text를 별도로 보존, metadata에 origin/type/revision/scope/hash/read provenance. source ref는 등록한 opaque view | runtime source_payload는 같은 view를 lookup/materialize하고 read/agent 권한 및 현재 원천 floor를 확인. Voice OFF actual Text와 Voice ON 빈 Voice의 차이는 기존 scope 회귀와 구현이 유지 |
| RECENT_INTERACTION | 전체 Text와 actual Voice 두 channel 및 provenance scope=INTERACTION_CHANNELS | heard-only로 자동 변환하지 않음. PAST 선택 여부는 모델/Frame의 의미 선택 |
| TASK_RESULT | Task.result/result_refs의 actual source/artifact dict를 lookup하여 실제 문자열 본문/metadata 및 Task 권한 floor를 반환 | raw artifact와 actual source_ref 모두 기존 lookup에 연결. Task ID exact locator와 query 검색을 구별. query coverage 한계는 기존 공통 조건 |
| INPUT_SELECTION | 실제 pointer/time 관측 및 원 source body/metadata 반환, read item shape는 동일 | 발화의 어느 span이 어느 원천을 지칭하는지는 partial 모델/Frame이 정함. owner가 정답 binding을 추가하지 않음 |
| 현재 GOAL_RESULT | Frame body.result_of → code가 producer Goal/이번 Request/revision을 pin → runtime이 해당 actual producer ready artifact를 기다려 Agent payload.body_artifact에 넣음 | v4 B1 S3-02의 actual 새 Summary artifact ID와 SEND body ID 일치. 과거 K를 무료로 body에 주입하지 않음 |
| Agent payload와 권한 | source_payload에서 actual lookup 원문을 복사하고 SOURCE_COMPOSITION_READ hash 기록; model read와 agent 소비 역할을 각각 검사 | Source body를 Agent에 실제 전달하는 단계는 model Context 선주입과 다름. 단지 metadata에 ref가 있다는 이유로 model이 내용을 읽었다고 표시하지 않음 |

모델이 metadata의 known Task를 binding한 뒤 Agent가 그 Task 원천을 실제 읽는 것은 합법적인 역할 경계다. source_payload는 제공된 handle, actual owner 원문과 정책으로 조합하며 모델이 읽지 않은 내용의 의미를 대신 정답으로 완성하지 않는다. Agent의 실제 보고서 품질은 이 stub의 대상이 아니다.

## 4. clarification 이후 부분 Context 내용 재확인

초기 key 목록만 보고 question/answer 별도 key가 없다는 이유로 전달 누락 가능성을 제기했으나, 실제 내용 재확인 후 철회했다. v4 B1 S2-01 `E1770`의 실제 clarification answer 이후 부분 call `c2c7a3e7-63e9-4807-bfc7-5a5a6a85e5b0`의 Context transcript는 다음을 포함한다.

```text
이 표와 저 표를 비교해줘.
새로운 조건이나 권한은 추가하지 않았어. 처음 요청대로 처리해줘.
```

Context SHA-256=`5e2d2aa63ec4fbdd54c8767b1c53abf0420a75360cb0ed14f0633b529d2c28d1`, output=`56fcddcbd27b05f2350b204f08e862086ed186d73a770947f30c07fdc40605be`. B 코드가 실제 answer.transcript를 inp.transcript에 append하며 이후 partial에 사용한다. 새 Frame에는 별도 실제 answer와 이전 Frame/read state도 제공한다. 이 원장에서 answer 내용 누락을 새 P2로 단정할 근거는 없다. key 이름 자체와 실제 정보 전달을 혼동하지 않는다.

## 5. 최종 리뷰 판단

두 확정 수정은 지원 query를 원형대로 전송하고 실제 read metadata/content를 사용자 질문 표시로 연결한다. 의미 전략, source 추가, 임의 후보 선택과 Goal/value partial 확장은 없다. 새로 확인된 미해결 P1/P2는 없다. 최종 v5 source digest `2143b39761bd0b74e8d9d973d8b59c1e8ba722ccc24ebafcc0d7e03d85cda4b0`의 frozen-source/evaluator-tools 각각 50개도 manifest hash mismatch=0으로 직접 확인했다. 기존 bounded lexical 검색 coverage, B의 reference-only partial 범위, 모델의 잘못된 연산/원천/조건 선택과 후보 질문 품질은 측정에서 그대로 드러나야 하는 조건이다.

이 검토는 declared port와 실제 소비 형식의 일치 및 회귀 검사 내용에 한정한다. 실제 Qwen의 의미 정확성을 회귀 지정 출력으로 입증하지 않는다. 새로운 official freeze와 실행은 campaign owner의 별도 기록을 따른다.
