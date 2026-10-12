# v4 공통 scope/capacity 통합 독립 검토

상태: **READ_ONLY_INDEPENDENT_REVIEW / NO_QA_VERDICT**. 구현 agent가 편집 종료를 통보한 뒤 아래 최종 hash를 다시 확인했다. 코드/fixture 변경, API 호출, 후보/테스트 실행과 QA 재채점을 수행하지 않았다. 정적 검토와 구현 agent가 보고한 무상 회귀 실행 결과를 구분한다.

**결론:** 검토한 최종 버전에서 두 공통 P2의 수정 경로와 필요한 구조 회귀 조건을 확인했다. 재검수 중 발견했던 structured-ref alias 우회와 existing-target를 이용한 Task capacity 우회도 수정됐다. 아래 범위에서 미해결 P1/P2를 찾지 못했다. 실제 모델 pilot/전체 QA의 성공을 뜻하지 않는다.

최종 파일 확인 UTC: 2026-10-11T19:46:03.703687+00:00. 검토 기준은 [전수 계약 감사](./v4-preflight-contract-audit.md), current ASR-QA-v6 P21, local Mac/합성 Text/Voice 입력 범위다. 중단한 v3 결과/리뷰는 보존했다.

## 실제 전달 view

[owners.py](../../../../prototypes/candidates/dp41_45_poc/shared/owners.py)의 owner read는 actual interaction record에 기반한 opaque `via-interaction:<selector>:<publication>@v<revision>:<record_hash>`를 발행한다. 원본 source와 publication의 ID가 같아도 view를 original source로 재조회하지 않는다. `_materialize_interaction_view`는 실제 retained record와 hash를 확인하고 source_payload는 그 actual materialized view를 가져간다. 평가자/oracle의 기준 문장이나 정답 Task를 주입하지 않는다.

| 확인 항목 | 최종 코드의 동작 |
| --- | --- |
| PAST_DELIVERED, Voice ON | actual delivered_voice_text를 executable text로 공급. 미재생 Text tail을 제공하지 않음 |
| PAST_DELIVERED, Voice ON + 음성 0글자 | 빈 actual voice text 유지. 표시한 Text로 fallback하지 않음 |
| PAST_DELIVERED, Voice OFF | actually text_presented인 Text를 공급하고 ACTUAL_TEXT로 기록 |
| RECENT_INTERACTION | prepared/표시 Text와 actual Voice를 별도 field로 보존. INTERACTION_CHANNELS scope는 PAST actual scope와 다른 opaque ref |
| origin/provenance | Response Manager, INTERACTION type, selector/actual scope, publication/revision/record hash, channel hashes, source_refs, 실제 read_ids 보존 |
| namespace/currentness | original source reference는 원문으로 남음. actual view lookup은 발행 registry와 pinned record를 사용. current()도 view namespace의 live 권한 floor를 사용 |
| 권한 | retained/current interaction과 PUBLICATION_ORIGINAL/backing source의 read/model/agent floor를 각각 반영. 새 revision이 생겨도 허용된 pinned history body는 유지하며 현재 권한 철회는 소비 차단 |
| hash/revision 변조 | 미발행 handle/변조 hash/다른 revision의 alias는 view registry 확인 또는 revision 비교에서 거절. original source로 fallback하지 않음 |

재검수에서 처음 발견한 dict source_id/publication_id/artifact_id 경로의 alias 우회는 `_REFERENCE_KEYS`와 `_wire_ref`의 동일 정규화로 수정됐다. 테스트는 11개 alias의 actual/forged/revision mismatch, nested ref mismatch, 같은 revision의 body/range 변조를 포함한다. Publication revision pin과 backing source revision은 별개로 유지하고 모순된 publication revision 요구는 NO_MATCH로 반환한다.

Runtime의 실제 delivery history에도 `voice_on`, actual Text 표시/Voice onset과 interrupted 값이 저장되므로 이후 Voice OFF Text와 Voice ON 빈 Voice를 구분할 근거가 있다. 과거 fixture의 Voice 속성이 생략된 record는 기존 Voice ON 규칙을 적용하며 current P21의 관련 before-state와 일치한다.

**소비 범위:** prescribed output 구조 테스트는 실제 A READ→PROPOSE와 B FRAME→binding을 각각 거쳐 source_payload와 Agent wire encode/decode의 actual received source를 검사한다. PAST의 미재생 tail이 wire source에 없고 original source는 원문을 유지한다. Stub은 그 actual source body를 소비한다. RECENT의 whole Text와 actual Voice를 구분해 전달하는 것은 합법적 정보 공급이며, 모델이 ‘들은 기준’에 부적절한 selector를 고르는 오류를 코드가 올바른 selector로 바꾸지 않는다.

## cache와 명령 idempotency

View의 record hash는 read ID나 현재 permission 변경으로 body identity를 바꾸지 않는다. Cache content key는 evidence_provenance의 read_ids만 제외하고 ref/content/scope/권한과 policy_epoch는 포함한다. 새 actual QueryReceipt는 원장과 provenance에 계속 보존하고 KNOWN_REF view read도 새 read ID를 기록한다.

Runtime `_command_input_value`는 실제 발행된 INTERACTION_EVIDENCE의 read_ids만 업무 명령 fingerprint에서 제외한다. Actual wire payload/hash에는 full read provenance가 남는다. 같은 body/scope를 다시 읽고 같은 logical command를 실행해도 새 Task/효과가 생기지 않는다. Source body/revision 또는 PAST↔RECENT scope가 바뀌면 conflicting payload로 거절한다. arbitrary property에 read_ids가 있다는 이유로 값을 제거하지 않는다.

## 현재 scene의 Task 10개 상한

[runtime.py](../../../../prototypes/candidates/dp41_45_poc/shared/runtime.py)는 current scene_task_ids와 pending_task_ids를 owner 보관 이력과 구분한다. Capacity를 명령/Task 생성 전에 검사한다. 같은 command fingerprint의 중복은 이 검사보다 먼저 억제한다.

| 경계 | 확인한 최종 동작 |
| --- | --- |
| S5-04의 initial 7 + 신규 3 | 현재 활성 10개 허용 |
| 11번째 신규 | 명시 capacity 거절. 신규 Task/Execution/효과/성공 receipt를 생성하지 않음 |
| 신규 operation에 existing target 부착 | actual dispatch operation이 existing operation인지 판단해 신규 슬롯으로 검사. target만으로 guard 우회 불가 |
| existing SET_PROPERTY/CANCEL_TASK/ANSWER_QUESTION | 같은 활성 Task면 새 슬롯을 소비하지 않음. 비활성 보관 Task의 재활성화는 capacity 검사 |
| 접수 ACK 대기 | pending ID로 슬롯을 예약. 같은 logical command 중복은 새 슬롯/Task 생성 없음 |
| 취소 ACK/UNKNOWN | 실제 종료 확인 전 active 유지. ACK만으로 슬롯 해제하지 않음 |
| actual terminal | current Task의 COMPLETED/FAILED/CANCELLED 상태는 active 집합에서 제외. retained owner/state는 삭제하지 않음 |
| history | baseline 완료 200 Task와 현재 scene에 활성화되지 않은 이전 scope의 owner 이력은 capacity에서 제외하며 증거/메모리는 보존 |

P21의 external 사건 Task는 해당 scene 초기 Task 집합에 있고 중복 event가 새 Task를 만들지 않는다. 이 확인을 임의 새로운 external Task 발견/활성화 프로토콜 전체의 검증으로 확장하지 않는다.

## 의미 알고리즘과 평가 범위 보존

DP41 a.py/b.py/contract.py/prompts.py 및 fixture inputs.json/oracles.json의 byte는 v3 frozen-source와 동일하다. A의 read→모델 재판단/최종 의미와 B의 reference-only B_PARTIAL_SELECT/코드 binding/clarification 뒤 새 unbound Frame 경로를 바꾸지 않았다. B의 일반 Goal/condition/value 추출 partial이 새로 생긴 것은 아니다. enum/연산/budget이나 정확성 정답도 변경하지 않았다.

이번 수정은 actual requested delivery view와 source namespace/권한을 보존하고 공통 Task capacity를 집행한다. 이를 목표 수치 달성이나 B의 실패를 성공으로 보정한 것으로 취급하지 않는다. 모델의 잘못된 selector/Task/input_ref/조건/관계, 불필요 질문과 quota 소진은 계속 후보 오류다. Agent domain proposal/report 품질, 실제 SMTP 및 실제 음성 모델 성능은 이 구조 검토의 입증 범위 밖이다.

## 무상 회귀 근거와 최종 hash

구현 agent의 편집 종료 메시지는 DP41 20/shared 64/benchmark 64 unittest PASS와 API 호출 없음, scope/capacity 12개 회귀 포함을 보고했다. **본 독립 검토자는 이를 재실행하지 않았다.** 아래 고정 테스트 파일을 읽어 필요한 긍정/부정 조건이 포함됐는지 검토했다. 위 PASS는 구현 agent의 실행 보고이며 실제 모델 의미 정확성 결과가 아니다.

| 최종 수정/회귀 file | SHA-256 |
| --- | --- |
| [owners.py](../../../../prototypes/candidates/dp41_45_poc/shared/owners.py) | `08d4e5d421c8e19eeafb01b214fd7af2b894aef23a64794f9d9f9666e8fc376e` |
| [runtime.py](../../../../prototypes/candidates/dp41_45_poc/shared/runtime.py) | `4bd5a5b3d3c802cbc0249fe195ebadef34fcccfeb9c389859fe2c4f2a48d35ec` |
| [test_evidence_scope.py](../../../../prototypes/candidates/dp41_45_poc/shared/tests/test_evidence_scope.py) | `ebad290e637f7b1069f4ef9867e870d1f2542c31fd6c9e8ccf201e53e05bae41` |
| [test_fixtures.py](../../../../benchmark/architecture/dp41_45_poc/tests/test_fixtures.py) | `5386a89bd7b377ec56c8eceff46718200a2c3735c29f7b3c0df2c16dc3cf5cca` |

| v3 frozen-source와 byte 동일한 file | SHA-256 |
| --- | --- |
| [a.py](../../../../prototypes/candidates/dp41_45_poc/dp41/a.py) | `b351057d152788bcb11b6e7d5f8451f6f3c9ce5df38096d569deb14955b256be` |
| [b.py](../../../../prototypes/candidates/dp41_45_poc/dp41/b.py) | `4637588c85e745c290334c3bf7471e752d2c1946bd2c1baee649c6f60b2b73e2` |
| [contract.py](../../../../prototypes/candidates/dp41_45_poc/dp41/contract.py) | `5415d2ed95b55dccb62740d3dc7b5502be3567c31add4f9a14a7820938bb392f` |
| [prompts.py](../../../../prototypes/candidates/dp41_45_poc/dp41/prompts.py) | `be366f35926aebc73ba6ce9f1c9ba526366145518090b99d2aa28eee4ee8590c` |
| [inputs.json](../../../../benchmark/architecture/dp41_45_poc/fixtures/inputs.json) | `8024baf9ecefdc797bce532268ccb3f789fe788cdfaa418765dc730587728bd4` |
| [oracles.json](../../../../benchmark/architecture/dp41_45_poc/fixtures/oracles.json) | `80461f5ef25f921f2d4e180c9adce29f3fc1fe7b22de22c5a8f70f5325a25690` |

이 hash가 새 freeze에 포함되어야 본 검토의 대상과 일치한다. 새 실제 실행은 새 campaign에서 수행하고 v3의 무효/중단 증거를 덮어쓰지 않는다.
