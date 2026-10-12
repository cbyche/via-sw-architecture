# M04 producer reference 및 Request revision 독립 검토

검토자: `/root/poc_dp41_contract`. 상태: `READ_ONLY_PREPARATION_REVIEW`. V5 공식 timing 진행 중에 소스와 보존된 준비 실행 evidence만 읽었다. 이 검토에서 테스트, 모델 API, 측정 실행 또는 후보/evaluator 소스 수정은 하지 않았다. 이 파일만 새로 작성했다.

V5 기준 source digest는 `2143b39761bd0b74e8d9d973d8b59c1e8ba722ccc24ebafcc0d7e03d85cda4b0`이다. 준비 helper는 공식 후보와 별도의 source copy에 적용됐으며 공식 QA05 job의 완료 또는 점수로 취급하지 않는다.

## 1. 검토 대상과 고정된 식별자

경로 기준은 `/Users/changbae/Workspace/via-sw-architecture/`이다.

| 대상 | SHA-256 |
| --- | --- |
| `results/architecture-evaluation/current/dp41-qa05-helper-preparation-20261012/goalref-v2/goalref_patch.py` | `f5abaf7c0a42f4905e8f1c14aa8e55d065dc53a8ef1e40183ca363efd938ab82` |
| 같은 폴더 `goalref_runtime_checks.py` | `08b59d8d88650cf2c275dcbd45046fef224b359a82481b5df59d107e661dbc76` |
| 같은 폴더 `goalref_extras.py` | `22f5fe07589437df63840d70ace251c1e8ad489a5ad10b0c9092ab0fb6624ce3` |
| 같은 폴더 `semantic_change_probe_typed.py` | `73af8b0912be98ec2639b518a60caa015893ad21f9164a6c0b1dcfcf78ffbb25` |
| V5 `evaluator-tools/benchmark/architecture/dp41_45_poc/change_patches.py` | `a2f36fd73a7737b34c650c3e01c1b61b56956af6dd356d82e256a858ac9db918` |
| 준비 `v5-typed-reference-preflight-3a5c9abe/claims.json` | `e94ace663c2772a3e3c021d768798913401253e93096d35e080ef6b382147789` |
| 같은 준비 실행 `runtime-checks.json` | `0338f3d6cb5e747a1cead7bb8ef88f6b4b486d10c8872d8c572856e21f09e77e` |
| 같은 준비 실행 `summary.json` | `d32cadf5443e3f7fe3e0fe83a10a446d07f870f51c084a76b5c1ee4c322dd5d0` |

`goalref_patch.py`, `goalref_extras.py`, 원래 M04 patch 전체와 runtime checker의 M04 기능 경로를 읽었다. semantic API probe의 source/import/failure/check 경로도 읽었지만 그 probe의 실제 모델 실행 결과는 아직 관찰하지 않았다. V5 frozen 후보는 관련 dispatch, B 결합, 공통 검증 구간만 읽었다. 준비 raw는 아래에 적은 producer, owner lineage 및 기존 owner string 경로를 읽었으며 전체 P21 또는 전체 QA05 job을 검토했다고 주장하지 않는다.

## 2. 확정 P2: 원래 M04 producer map은 Request revision을 잃는다

V5 `frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py:839`의 명령 fingerprint는 `(ports.request_id, ports.revision, goal["id"])`이다. 같은 Request의 revision 1과 2가 같은 goal ID를 쓰면 서로 다른 실제 명령과 Task를 생성할 수 있다.

반면 원래 `evaluator-tools/benchmark/architecture/dp41_45_poc/change_patches.py:20`부터 22의 `_predicate` 추가 코드는 `(request_id, producer)`로 producer를 찾는다. 같은 파일 25부터 26은 `(ports.request_id, goal["id"])`에 실제 Task ID를 저장한다. 명령의 revision은 map에 없다.

따라서 다음 경로는 표현의 한계나 모델 오류가 아니라 추가 M04 구현의 association 결함이다.

1. Request R revision 1의 goal `report`가 실제 Task T1로 접수된다.
2. R revision 2의 동일 goal이 실제 Task T2로 접수되면 같은 producer map 슬롯을 T2가 덮는다.
3. revision 1의 `WHEN FAILED`가 T1의 실패를 기다린 뒤 predicate를 확인할 때 T2의 상태를 읽을 수 있다.
4. T2가 RUNNING 또는 COMPLETED이면 T1 실패 continuation을 빠뜨리거나 다른 revision 상태에 따라 결정한다.

이 판정은 코드 경로로 확정했다. 원래 A patch에서 이 경로를 실제 재실행하지는 않았다. 공식 P21 후보 또는 공식 timing 소스의 변경 사유로 확대하지 않는다.

## 3. typed-v2 보완의 실제 범위

`goalref_patch.py:32`부터 46은 map key를 `(Request ID, VIA Request revision, producer goal ID)`로 바꾸고 값에 실제 `task_id`, `execution_id`, Task의 `request_revision`을 저장한다. `_predicate` 호출은 같은 파일 40부터 41에서 `(ports.request_id, ports.revision)`을 전달한다. 조회한 Task의 Execution 또는 Task revision이 저장한 dispatch lineage와 다르면 UNKNOWN에 해당하는 `None`을 반환한다.

VIA Request revision과 Task `request_revision`은 다른 필드다. 준비 코드가 둘을 같은 값으로 대입하지 않는다. 새 Task가 VIA revision 2에서 생성되더라도 Task 자체 revision은 1일 수 있으며, stored binding은 실제 생성된 Task 값을 사용한다.

준비 copy의 `shared/runtime.py:558`부터 575에서 이 검사가 실제 predicate 평가에 있다. 같은 copy의 `shared/owners.py:205`부터 224는 실제 Questions/Tasks를 Source보다 먼저 lookup한다. 따라서 여기서 실제 Task ID를 조회할 때 Source를 우선해 잘못된 source 상태를 읽는 경로는 확인하지 못했다.

`goalref_extras.py:127`부터 177과 보존된 `runtime-checks.json`의 `actual_request_revision_and_owner_execution_lineage` 관찰에는 다음 사실이 있다.

- 같은 Request의 VIA revision 1과 2가 별도 실제 report Task 및 Execution으로 접수됐다.
- 잘못된 Execution 및 Task revision의 FAILURE는 수용되지 않았다.
- revision 1의 실제 실패와 revision 2의 실제 성공을 구별해 실패 continuation draft가 하나만 실행됐다.
- 실제 SET_PROPERTY로 첫 Task의 owner revision을 변경한 뒤 이전 producer predicate는 UNKNOWN이 됐다.
- 저장한 dispatch Execution을 명시적으로 변조한 검증은 `EVALUATOR_MAPPING_FAULT`로 기록됐다. 실제 외부 사건이 Execution을 변경한 것으로 설명하지 않는다.

서로 다른 Request 검사는 같은 파일 91부터 125, 기존 owner ID `GOAL:report` 보존은 179부터 197에서 실제 B resolve/read/binding/execute 경로를 사용한다. 준비 `runtime-checks.json:47362`부터 47367은 B variant, source copy의 import 경로, API 0회를 선언한다. 준비 summary의 통과 항목은 작성자의 scripted 실행 결과를 읽은 것이다. 이 reviewer가 새로 실행한 결과가 아니다.

이 근거는 실제 CREATE_REPORT producer에 한정한다. 모든 직접 응답 또는 core producer의 terminal event를 지원한다는 결론, 모든 결과 future가 Request revision별로 분리됐다는 결론은 내리지 않는다.

## 4. 순수 B 및 기존 owner reference의 보존

tagged producer reference는 `WHEN.predicate.reference_id = {"producer_goal_id": relation.from_goal}`이라는 exact singleton object다. `goalref_patch.py:52`부터 75는 own finite goal, STATE, EQ terminal 또는 IN의 고유 terminal subset만 허용한다. 임의 표현식이나 owner string을 producer로 다시 해석하지 않는다. 기존 owner string branch는 유지한다.

V5 frozen `dp41/b.py:21`부터 32는 일반 dict를 재귀적으로 보존한다. 같은 파일 111부터 114는 WHEN의 reference_id에 그 기존 치환을 적용한다. V5 frozen `dp41/contract.py:251`부터 252는 Frame reference ID와 goal ID의 겹침을 거절한다. 따라서 tag 안의 goal ID가 reference binding으로 대체되지 않는다. B는 여전히 코드가 완성 Meaning을 결합하며, 마지막 전체 Meaning 모델 재생성 또는 자유 instruction을 통한 VIA 조건 은닉을 추가하지 않는다.

준비 claims의 B binder before/after SHA는 모두 `4d5ae5fbc7f94692bfbc1eddac391800b0ca4a72808c9d5d265ce7e4874b778f`이다. 이는 원래 별도 `producer_goal_id` predicate 필드를 도입해서 B U02를 변경한 경로보다 작은 대안이 있다는 실제 구현 근거다. binder byte 동일성만으로 전체 QA05 회귀 완료를 주장하지 않는다.

## 5. A의 최소 동등 보완 및 기능 책임

원래 A의 top-level `predicate.producer_goal_id` 표현을 유지하더라도 revision collision을 막는 데 필요한 최소 runtime 보완은 동일하다.

1. `_predicate`에 `(ports.request_id, ports.revision)`을 전달한다.
2. producer map key를 기존 명령 fingerprint와 같은 triple로 만든다.
3. actual Task ID뿐 아니라 실제 Execution 및 Task revision을 map value에 저장한다.
4. absent 또는 superseded lineage는 UNKNOWN으로 두고 다른 Task의 terminal state를 사용하지 않는다.

이 보완은 A의 model-led 의미 loop 또는 `a.py`의 새 의미 알고리즘을 요구하지 않는다. tagged 표현을 A/B 공통으로 채택한다면 공통 prompt와 표현 검증도 같은 기능 계약을 사용하면 된다.

현재 registry에서 producer와 Request revision의 association 및 predicate continuation은 기존 U08 흐름 책임에 속한다. 실제 FAILURE를 Task/Execution owner 상태로 수용하고 현재성/중복/late success를 제어하는 변화는 U07이다. 같은 map key 정정을 별도 U-ID로 만들거나 U07과 U08에서 다시 중복 세지 않는다. 표현 전달은 U01, 유한 tag 검증은 U03에 대응한다. A/B의 실제 독립 diff와 검증이 나오기 전에는 개수 및 최종 QA05 점수를 확정하지 않는다.

원래 방식의 B U02 변경은 실제 diff이지만, typed-v2는 기존 B 결합 책임을 변경하지 않고 요구를 구현할 수 있다는 준비 근거가 있다. 이를 DP41B 구조 자체가 반드시 부담하는 추가 책임으로 단정하면 안 된다. 최종 선택된 변경안과 원래 안은 별도 evidence 경로로 남기고 합산 manifest에서 하나의 M04를 명시해야 한다.

## 6. 실제 job에서 확인해야 할 경계

- 동일 frozen baseline에서 원래 A의 동등 revision/lineage 보완 또는 공통 typed 표현을 적용한 actual job diff를 확인한다.
- producer terminal을 원문 허용 source와 연결한 실제 draft, 성공/실패 양분기, duplicate FAILURE, late success, AFTER/USE_RESULT 유지 및 SEND 0 evidence를 확인한다.
- 실제 자연어 model probe와 full P21 회귀를 확인한다. 지금 읽은 준비 mock 통과를 모델 의미 정확성으로 사용하지 않는다.
- 변경 responsibility ledger의 U01~U18 각각을 actual diff와 대응한다. 이 문서는 전체 18책임 판정 또는 원래 12 job review를 대신하지 않는다.
- M05의 post-change 발행 token을 동일하게 restore하는 증거와 U14 실제 delta 여부는 별도 검토 대상이다. 이번 M04 revision 판정으로 U14를 추가하지 않는다.

이 검토에서 확인한 P2는 원래 M04 독립 변경 patch의 revision 없는 producer map이다. typed-v2의 읽은 범위에서 이를 그대로 남기는 결함은 확인하지 않았다. 아직 실행되지 않은 actual QA05 jobs의 기능/회귀/의미 정확성은 미확정이다.
