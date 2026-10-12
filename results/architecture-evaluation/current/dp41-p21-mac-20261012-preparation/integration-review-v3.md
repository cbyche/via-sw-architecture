# v3 API 및 계약 연결 수정 독립 검토

상태: **READ_ONLY_STATIC_REVIEW / NO_API / NO_CANDIDATE_EXECUTION / NO_SCORE**. 2026-10-12. v2 동결본과 live 네 파일을 대조했다. 다른 세션의 변경을 보존하고 이 새 Markdown만 저장했다.

결론: 검토한 변경에서 새 P1/P2는 발견하지 못했다. 실제 v2 B 부분 호출의 JSON-mode 요청 결함과 validator/소비자 타입 불일치를 고치는 범위다. 정답 source/Task/목표/조건을 추가하거나 모델의 잘못된 의미를 보정하는 변경은 없다. 이 결론은 정적 검토이며 실제 provider 연결 성공과 전체 P21 통과를 뜻하지 않는다.

## 검토 기준과 결함 근거

비교 기준은 [v2 manifest](../dp41-p21-mac-text-v2-20261012/manifest.json)의 frozen source digest `119b3344337d1f1fd3732df9e6cd3110cca38c574f73b6ce1c649a032227952c`다. v2 raw는 변경하지 않았다. 실제 결함은 [A1/B1 네 경로 관찰](../dp41-p21-mac-text-v2-20261012/reviews/observed-resolution-paths.md)에 call ID/context/output hash와 함께 남겼다.

B1/S2-02와 B1/S2-05의 B_PARTIAL_SELECT는 `response_format=json_object`인데 message에 `json` 단어가 없어 provider HTTP400으로 거절됐다. S2-02 frame의 unresolved 문자열은 후속 B 코드가 기대하는 dict와도 맞지 않았다. 모델의 올바른 부분 의미 판단을 보기도 전에 실패한 통합 결함이므로 이 결과를 B 구조의 필연적인 정확성 한계로 사용할 수 없다.

## 변경과 A/B 동등성

| 파일 | 실제 변경 | 의미 처리 영향 |
| --- | --- | --- |
| prompts.py | PROMPT_VERSION을 dp41-prompts-2로 변경. COMMON에 unresolved 객체의 reason/선택 field_ref/source_ref 형식 안내. SELECT_SYSTEM에 one JSON object 명시 | COMMON 안내는 양안에 동일하게 적용된다. SELECT의 JSON 명시는 API에 필요한 표현 조건이며 새 답/후보를 주지 않는다. |
| contract.py | enum/관계/지원 필드가 문자열인지, predicate/preservation/unresolved가 기대한 객체인지 검사. frame reference 위치의 문자열 검사. reference_spec의 selector guard | 잘못된 중첩 타입을 ContractError로 돌려 A/B 기존 모델 형식 보완 경로를 사용한다. 정상 source/목표를 대신 골라주지 않는다. |
| b.py | 부분 반환 candidate_ref가 문자열인지, evidence_refs가 문자열 배열인지 확인한 뒤 기존 후보/evidence membership 검사 | 잘못된 반환을 기존 AMBIGUOUS/확인 경로로 보낸다. 합법적 부분 선택을 다른 의미로 교체하거나 전체 graph를 부분 모델에 맡기지 않는다. |
| tests/test_dp41.py | 부분 요청의 실제 system 메시지에 JSON 안내가 있는지, malformed nested 타입의 거절/보완, A의 같은 계약 처리, 기존 object source/symbolic result 지원을 확인하는 사례 추가 | generic port 사례를 읽었다. 정답 fixture 또는 측정 시나리오를 유리하게 바꾸는 기대값은 발견하지 못했다. 이 리뷰에서는 테스트를 실행하지 않았다. |

공통 계약의 MEANING_VERSION/FRAME_VERSION, GOAL_KINDS, OPERATIONS, SELECTORS, RELATIONS, TOOLS 및 관련 top-level key 집합이 동일한지 정적 자료 비교로 확인했다. Budget 코드 블록과 A 후보 파일은 v2 bytes와 동일하다.

B의 reference fetch, supplied metadata 처리, 실제 bounded read, singleton/MANY binding, 추가 부분 모델/확인 질문/frame 재생성, 최종 code binding은 변경하지 않았다. Goal 4개/reference 8개/read 16회/model 8회/확인 2회의 한도와 기존 source 접근 범위도 늘리지 않았다.

허용된 의미가 유지되는 근거: 새 검사들은 기존 enum의 문자열, 유효 선언 reference 문자열, 정상 predicate 객체와 reason 객체를 계속 허용한다. 금액/기간/보안 tail/새 견적 누락/goal ID를 Task로 오인하는 의미 오류를 코드가 정답으로 수정하는 분기는 추가되지 않았다.

## 한계와 새 동결의 필요성

- v2와 wire 처리가 완전히 동일하다고 표시하지 않는다. v3는 의도한 unresolved 객체 형식을 명시하고 잘못된 문자열/중첩 타입을 조기 거절하므로 형식 재생성, 호출 수, 대기와 실패 경로가 달라질 수 있다. 양안을 새 공통 prompt/contract 아래 다시 고정해 측정해야 한다.
- 추가 검사는 모든 임의 JSON 필드에 대한 완전한 schema 검증 증명이 아니다. 이번 리뷰는 네 파일의 변경과 합법 의미 보존을 정적으로 확인한 범위다. 실제 provider/API preflight와 전체 후보 회귀는 root의 후속 수행으로 확인해야 한다.
- B의 부분 모델은 reference 후보 선택 범위이고, 임의 속성/관계의 모든 부분 의미 해석을 구현한 것으로 확대하지 않는다. source KNOWN_REF의 실제 read, 여러 부분 판단의 직렬 실행과 bounded lexical retrieval은 기존 구현 선택으로 남는다.
- 새 타입 검사는 정상 후보의 잘못된 source/Task/goal/조건과 필요한 근거 누락을 성공으로 바꾸지 않는다. 수정 후 좋아질 결과를 예측하거나 v2 실패를 소급 삭제하지 않는다.
- M04 변경 실험 helper는 QA 담당 범위다. 이 네 파일 리뷰에서 해당 helper의 정합성 또는 완료를 확인했다고 주장하지 않는다.
- 원본/모델 문맥/출력/hash, 실제 비용과 실패는 보존한다. 이 리뷰는 QA 점수, A/B 우열이나 전체 제품 성능을 산출하지 않는다.

## 검토한 live 파일 hash

아래 SHA256는 이 리뷰에서 읽은 파일 bytes다. 후속 편집이 있으면 동일 리뷰로 승인한 것으로 취급하지 않고 새 hash를 대조한다.

| live 파일 | v2 frozen SHA256 | 검토한 live SHA256 |
| --- | --- | --- |
| [prototypes/candidates/dp41_45_poc/dp41/prompts.py](../../../../prototypes/candidates/dp41_45_poc/dp41/prompts.py) | `d18124ba58709d59301471230828a49e50058f378ca391efa9c58c7297549e54` | `be366f35926aebc73ba6ce9f1c9ba526366145518090b99d2aa28eee4ee8590c` |
| [prototypes/candidates/dp41_45_poc/dp41/contract.py](../../../../prototypes/candidates/dp41_45_poc/dp41/contract.py) | `43e180fdd95c6e57978b58d7f849a6123933144af6f177bf16c3e4fcfd81999b` | `5415d2ed95b55dccb62740d3dc7b5502be3567c31add4f9a14a7820938bb392f` |
| [prototypes/candidates/dp41_45_poc/dp41/b.py](../../../../prototypes/candidates/dp41_45_poc/dp41/b.py) | `d16c3694bc3e94cf451b21fde20ea4ce80562a2292e944236273cefa42d2e5b8` | `4637588c85e745c290334c3bf7471e752d2c1946bd2c1baee649c6f60b2b73e2` |
| [prototypes/candidates/dp41_45_poc/dp41/tests/test_dp41.py](../../../../prototypes/candidates/dp41_45_poc/dp41/tests/test_dp41.py) | `b34b8eb9153d8e3f1a2b2527182279b9f9e31a182db0d8240419e3020db083e5` | `509ad9d7c909c9df8a5cfa4cd22c788d2fdb8e6ba1b028d854f8ccfcdc6984f1` |

A 후보 bytes 불변 SHA256: `b351057d152788bcb11b6e7d5f8451f6f3c9ce5df38096d569deb14955b256be`.

검증 방법: 파일 diff와 전체 관련 validator/호출 경로 및 추가 테스트의 읽기, 정적 enum/Budget 비교, 파일 SHA256 계산. candidate/provider를 import하거나 실행하지 않았다. API 및 유료 실행은 없다.
