# V7 공통 User Memory 등록 포트 독립 검수

검수 시점: 2026-10-12. 상태: READ_ONLY_INDEPENDENT_REVIEW. 구현자의 편집 종료 뒤 최종 세 파일과 지정 모델 기능 회귀의 실제 원장을 읽었다. 독립 후보 실행, 테스트 실행과 API 호출은 0이다. 현재 검수 범위에서 확정된 미해결 P1/P2는 없다. 공통 구현 누락의 수정이며 QA05 변경 책임 수에 포함하지 않는다.

## 원문과 수정 범위

현재 [ASR-QA-v6 §6.5의 M-06 추가 기능 회귀](../../../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)는 S1-03 원본 등록 oracle와 철회/삭제 회귀를 요구한다. 원본 입력은 “앞으로 비교 결과는 표로 보여주는 걸 선호한다고 기억해줘.”이며, 현재 사용자의 향후 비교 결과 형식 선호를 등록하고 M언어 및 다른 선호를 보존해야 한다. S1-03은 P21에서 제외됐으므로 이 기능 회귀를 P21 정확성 분모에 추가하지 않는다.

V6의 REGISTER_MEMORY는 private memory_records에 값만 넣었고 공통 Owner 조회, 실제 사용자 namespace, 초기 Memory 적재와 response=None의 등록 확인 전달 연결이 없었다. V7은 이 공통 포트를 완성한다. [runtime.py](../../../../prototypes/candidates/dp41_45_poc/shared/runtime.py)는 채택된 Goal의 actual key/value/user/scope/source_refs/instruction을 보존하여 [owners.py](../../../../prototypes/candidates/dp41_45_poc/shared/owners.py)에 게시한다. 특정 comparison key나 TABLE 정답을 생산하는 코드가 없다. 잘못된 사용자, 일회성 scope, 충돌한 scope와 외부 target은 거절하며 잘못된 의미를 올바른 값으로 고치지 않는다. scope 선언이 없으면 일반 PERSISTENT namespace를 쓰므로 실제 향후 적용 대상의 의미 적합성은 후보 책임으로 남는다.

## 공통 read, 권한과 수명

새 ref는 사용자와 key를 구분한 via-user-memory namespace 및 revision을 갖는다. 기존 현재 사용자 Memory는 초기 실제 source에서 적재하며 원문은 유지된다. key alias는 현재 사용자의 원본만 선택한다. index metadata는 현재 사용자 source의 identity와 scope를 공개하고 실제 value/text는 OWNER_READ 뒤에 공급한다. 다른 사용자 값과 원문은 공통 lookup/read에서 반환하지 않는다. legacy 같은 key를 다른 사용자가 보유한 경우에도 현재 사용자 선택과 철회만 적용하는 회귀를 확인했다.

실제 등록은 로컬 read/model 권한을 사용한다. source_refs가 있으면 부모 원천의 read/model/agent floor를 각 조회/소비 시점에 적용한다. Agent 제공 권한이 없는 원천을 사용한 로컬 등록은 허용되지만 그 파생 값을 Agent payload로 공급하지 않는다. 초기 M언어와 무관한 선호는 그대로 남는다. 동일 Request revision/Goal의 같은 등록은 한 번만 게시되고 충돌 값은 거절한다. 새 Request의 실제 변경은 새 revision을 게시하여 과거 버전과 구분한다. 기존 Owner identity 충돌 및 공통 source 용량 초과는 private 부분 성공을 만들지 않는다.

## 응답과 철회 연결

등록 성공 facts에는 actual memory, REGISTERED receipt와 completion_confirmed가 들어간다. 후보가 response=None을 반환했을 때만 기존 CRESPONSE 경로가 이 실제 facts를 받아 전달한다. 후보가 직접 response를 제공하면 그 문장을 그대로 보존한다. 잘못된 후보 응답을 공통 코드가 정답 확인 문장으로 대체하지 않는다.

원천 철회는 direct/transitive source_refs를 따라 pending 모델 Context, 파생 artifact와 Memory를 fence한다. 현재 사용자 Memory의 모든 revision에서 실제 value/text를 지우고 역할 권한을 차단하며 cache를 비운다. explicit Memory key 철회도 alias를 원본 identity로 정규화하여 과거 ref의 소비를 차단한다. 원천의 이후 재허용이 철회된 파생 Memory를 부활시키지 않는다. 이미 클라우드로 보낸 사본은 되돌리지 않았다는 실제 receipt를 유지한다. 원시 감사 사본은 별도 제한된 보존 자료이며 이 검수는 저장 매체의 보안 삭제를 입증하지 않는다.

## 동일 원본 oracle의 실제 before/after

[exact helper v3](../dp41-qa05-helper-preparation-20261012/m06-exact-preference-v3/m06_exact_probe.py)는 native A PROPOSE와 B finite Frame을 지정하고 실제 REGISTER_MEMORY, Owner 조회와 전달을 관찰한다. response=None의 지정 CRESPONSE는 facts.memory의 actual key/value/user/scope/persistent와 실제 등록 receipt만 확인한다. Goal arguments에서 예상값을 다시 찾아 누락된 Owner 게시를 숨기지 않는다. evaluator가 공통 read에 정답을 주입하지 않는다. helper SHA-256은 `53b2614e48dd508a2b3539a38a49d1098fc5d8b843725fa0440e878f351688de`이다.

| 원본 oracle 기능 검사 | V6 before A/B | V7 after A/B |
| --- | --- | --- |
| 채택 Goal C1~C4 | 지정 출력이 표현한 조건 확인 | 같은 조건 확인 |
| C5 실제 공통 저장 조회 | 실패 | 실제 Owner 게시/조회 확인 |
| C6 실제 전달 | 실패, 해당 전달 없음 | 실제 Text 전달 receipt 확인 |
| 현재 사용자 partition 및 M언어 common read | 실패 | 확인 |
| actual Owner ref/revision | 실패 | 확인 |
| 파생 철회, 허용 원천과 다른 선호 보존 | 확인 | 확인 |

after A/B의 actual source는 `via-user-memory:current-user:comparison-result-format@v1`, value의 applies_to=COMPARISON_RESULT/format=TABLE/scope=FUTURE_COMPARISON_RESULTS, Owner user=current-user/persistent=true였다. actual Request=preference-request, input=preference-input, Goal=register-comparison-result-format, input SHA=`0412264684c36af2da4f2824fec131669b4194a7c057e13b92f6d72c02aa3e25`로 게시 provenance가 남았다. actual common read는 같은 ref, value 및 Owner scope를 반환했다. response=None 이후 CINTERPRET→CRESPONSE가 실행되고 Voice OFF의 “비교 결과는 표로 보여주는 선호를 기억했다.” Text가 actual delivered_receipt=true로 남았다.

[before A](../dp41-qa05-helper-preparation-20261012/v7-final-memory-helper-v3-compatibility-6690fee9/before-exact-A/raw.json), [before B](../dp41-qa05-helper-preparation-20261012/v7-final-memory-helper-v3-compatibility-6690fee9/before-exact-B/raw.json), [after A](../dp41-qa05-helper-preparation-20261012/v7-final-memory-helper-v3-compatibility-6690fee9/after-exact-A/raw.json), [after B](../dp41-qa05-helper-preparation-20261012/v7-final-memory-helper-v3-compatibility-6690fee9/after-exact-B/raw.json)를 직접 읽었다. after는 10개 지정 기능 조건을 모두 확인했으며 before의 C5/C6 실패가 보존됐다. [compatibility summary](../dp41-qa05-helper-preparation-20261012/v7-final-memory-helper-v3-compatibility-6690fee9/summary.json)는 copied baseline과 active source가 검사 중 변하지 않았음을 기록한다.

[generic 포트 trace](memory-registration-v7-functional-regression/trace.json)의 SHA는 `71ab9cdb51b514a2cc8959dd77e453cc55af4c37b91f1581cceed22da8a956be`이다. 11개 probe의 12 runtime 관찰을 읽었고 actual A/B adoption, explicit response 보존, 중복/버전, 원천별 역할 권한, direct/transitive 철회, pending Context scrub, historical ref, 재허용 후 비부활, 용량/충돌, 같은 key의 사용자 분리를 확인했다. [test_memory_registration.py](../../../../prototypes/candidates/dp41_45_poc/shared/tests/test_memory_registration.py)의 관련 assertion을 대조했다. author 보고 DP41 29/shared 75/benchmark 88 PASS/API0를 독립 테스트 실행 결과로 표시하지 않는다.

이 증거는 SCRIPTED_FUNCTIONAL_REGRESSION/API0이고 실제 모델 의미 정확성, QA 점수, A/B 우열이나 프로세스 재시작 후 저장 내구성을 주장하지 않는다. raw의 책임 수는 None이며 baseline 누락을 M06 변경 비용에 가산하지 않는다.

## 불변 자료와 최종 hash

V6 대비 DP41 A/B, finite contract, prompts, fixture builder, inputs/oracles/manifest 및 QA 정의는 byte-identical이다. 본 검수에서 V1~V6 frozen-source의 각 manifest 파일 hash를 전수 대조했으며 mismatch=0이었다. V6 digest는 `bcf3802bfad401059554d288b5f7d303ece0e16743fb43b62eb57cebbc075636`이다. 과거 모델 실패와 정답은 수정하지 않았다.

| 최종 파일 | SHA-256 |
| --- | --- |
| `prototypes/candidates/dp41_45_poc/shared/runtime.py` | `a1b02a1ebd4ef4ac2cdfadff2d58c56cfee7cb2009ac87774f63551424d1ffab` |
| `prototypes/candidates/dp41_45_poc/shared/owners.py` | `2a35e4370b2444a9855d8e772385dc87494a294cc094044b14d2dd7d02f27aad` |
| `prototypes/candidates/dp41_45_poc/shared/tests/test_memory_registration.py` | `a92d5ac8f7c6960532f20268378b46d46ad76a9ca4ec6e1902703b79bf8d4956` |

| exact raw | SHA-256 |
| --- | --- |
| `before-exact-A/raw.json` | `8ac4e55b684d5006e82ad94fbe1eee9205226f62f90ee136acacdeb0e922ae2d` |
| `before-exact-B/raw.json` | `be45976854e729105f0afb3c0ae7aa408e7b6cdc311bbb288383f33bc00b2c8c` |
| `after-exact-A/raw.json` | `558d49f173eb0e9ba0b344bd8f57039c69e2e7e56854b8544c63740082a1a541` |
| `after-exact-B/raw.json` | `65f26a01d43ccbc7762a52b26d5157e055cbe9f68333537ec985ed08f1d49d6e` |
