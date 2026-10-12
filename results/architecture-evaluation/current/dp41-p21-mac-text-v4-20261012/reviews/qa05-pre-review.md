# QA05 독립 책임 검토의 사전 질문

상태: `READ_ONLY_PRE_REVIEW / ACTUAL_JOB_DIFF_NOT_YET_REVIEWED`.

이 문서는 M01~M06 요구와 현재 evaluator의 패치 생성 규칙을 읽고 실제 독립 변경 job에서 확인할 질문을 정리한 것이다. 변경 job의 완료, 변경 책임 수, 평균이나 점수를 확정하지 않는다. 테스트, 후보 실행, 모델/API 호출을 하지 않았으며 frozen source, evaluator tools와 raw를 수정하지 않았다.

## 고정한 검토 대상과 관측 범위

캠페인: `DP41-P21-MAC-TEXT-v4`. `manifest.json:198`과 `evaluation-tools-current.json`의 source digest는 `c2dfc26fa6afbce8df4af3405913031ca440fefa150d6a0739c010b3ed4796a7`이다. 아래 경로는 이 캠페인 디렉터리를 기준으로 한다.

| 읽은 대상 | SHA-256 |
| --- | --- |
| `manifest.json` | `ceb58edcbcfb79206e5a2d5999532cfd65617e701b10f70f7cb70c39c1209946` |
| `evaluation-tool-manifest.json` | `52ab2e9433d6da8ef96859f52b7a4aadc2fbc7b12e3322647bb36cce3f0d8356` |
| `evaluator-tools/benchmark/architecture/dp41_45_poc/change_experiments.py` | `ace1893fad7318bd8ff5d0677f96a1fd0b0b18b36a314b7f6e084ea29224021a` |
| 같은 폴더 `change_patches.py` | `a2f36fd73a7737b34c650c3e01c1b61b56956af6dd356d82e256a858ac9db918` |
| 같은 폴더 `change_runtime_checks.py` | `18ac7a8bd03d8bd042978347bceba3156134b8f9c1131dcc760fd5a0c8472818` |
| `frozen-source/prototypes/candidates/dp41_45_poc/dp41/b.py` | `4637588c85e745c290334c3bf7471e752d2c1946bd2c1baee649c6f60b2b73e2` |
| 같은 폴더 `contract.py` | `5415d2ed95b55dccb62740d3dc7b5502be3567c31add4f9a14a7820938bb392f` |
| `frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py` | `4bd5a5b3d3c802cbc0249fe195ebadef34fcccfeb9c389859fe2c4f2a48d35ec` |
| 같은 폴더 `adapters.py` | `9c1247328c6c33082efa13012e4cbad0aba55d1d72a2e9b1449698edd11ca187` |

frozen QA `03-02-quality-attribute-definitions.md:614~685`의 정의, registry와 여섯 과제를 읽었다. 현재 구현 전 검토 문서 `03-qa-readiness-and-change-tasks.md:119~212`의 기존 지원/신규 delta 분리 규칙도 확인했다. `change_experiments.py`의 독립 사본 생성, 실제 diff/hash 보존, 완료/회귀와 두 검토자 gate를 읽었고, 관련 frozen B binder/검증/continuation/질문 owner 및 복원 코드와 `change_runtime_checks.py`의 M04/M05/M06 probe를 대조했다. `replay.py:1~160`의 증거 수준과 recorded port 입력/출력 읽기 경계도 확인했다.

아직 12개 실제 `patch.diff`, job별 원본/변경 source manifest, 실제 probe 출력, P21 replay 결과와 두 검토자의 판정을 읽지 않았다. 패치 생성 코드가 의도하는 동작을 실제 성공으로 표현하지 않는다.

## M04: B의 U02 변경이 필요한 최소안인지

현재 생성 규칙은 `change_patches.py:63~80`에서 WHEN에 `producer_goal_id`라는 새 필드를 추가하고, `:92~97`에서 B binder가 `reference_id`를 가진 경우에만 결합하도록 바꾼다. `:109~111`은 이를 U02 변경으로 주장한다. 이 구체 패치에서 B의 결합 책임을 바꾸는 것은 실제 delta이지만, 요구를 만족하는 최소 변경안에서도 필수인지 별도로 검토해야 한다. frozen QA `:680`은 불필요한 재설계를 세지 말고 최소한의 강한 수정안을 요구한다.

정적 대안 후보는 기존 `predicate.reference_id`에 유한한 producer 참조 namespace를 추가하는 것이다. 예를 들어 `GOAL:report`를 해당 WHEN의 `from_goal=report`와 일치하는 현재 Request의 producer 참조로만 인정한다. frozen `b.py:21~32`의 `_replace_refs`는 결합표에 없는 문자열을 그대로 보존하고 `:102~105`의 WHEN 경로가 이미 이 함수를 사용한다. frozen `contract.py:280~281`의 frame declared-reference 검사를 유한 producer 참조일 때만 면제하고, 공통 continuation이 이를 현재 Request의 실제 생성 Task로 해석하는 대안은 B binder 무변경으로 구현할 가능성이 있다. 이는 읽기에서 도출한 후보이며 실행으로 검증하지 않았다. 기존 baseline이 이미 M04를 지원한다는 주장은 아니다.

실제 변경 job 검토 질문은 다음과 같다.

1. 새 필드 방식과 namespace 방식에 동일한 자연어 요구, 성공/실패 두 분기, SEND 금지와 원본 `doc@v1` payload를 적용할 수 있는가? 실패한 보고서의 성공 artifact를 선주입하거나 실패 의미를 자유 domain instruction에 숨기지 않는가?
2. namespace는 임의 문자열이나 실행식이 아닌 명시적인 유한 문법인가? 존재하는 producer goal, 동일 `from_goal`, 현재 Request 범위, `STATE`와 제한된 terminal enum만 허용하는가? 기존 source/frame reference ID가 예약 namespace와 충돌하면 명시 거절하며 일반 owner 참조 의미를 바꾸지 않는가?
3. 실제 Task/Execution/request revision은 명령 접수 뒤 얻은 owner 값인가? 모델이 미래 Task ID를 만들어 내거나 코드가 의미 해석을 대신하지 않는가? frozen `runtime.py:267~279`의 adoption 제공근거 검사와도 일치하는가?
4. B의 기존 binder, 항목 해결과 partial call을 실제로 수정하지 않고 공통 U03/U08 변경만으로 끝나는가? 다른 위치로 옮긴 새 B 결합 책임을 파일 이름 때문에 누락하지 않는가?
5. `change_runtime_checks.py:197~310`의 실제 report command→FAILURE event→owner FAILED→draft 한 번, 중복/late success 차단, 기존 AFTER/USE_RESULT와 성공 분기 회귀를 동일하게 통과하는가? 이 probe가 요구하는 새 필드 형식에만 결합되어 대안의 동등 기능을 거절하지 않는가?

실제 최소안 검증이 끝나기 전에는 U02를 구조상 필수 비용 또는 불필요한 비용으로 확정하지 않는다. 원래 12개 job이 새 필드 패치로 실행되면 해당 실제 diff와 증거는 보존하고, 더 작은 대안의 별도 검증 증거를 원래 실행으로 바꾸어 표기하지 않는다.

## M05: U14를 추가할 독립 영속 변경이 있는지

현재 생성 규칙 `change_experiments.py:415~459`는 Task owner의 발급/검사(U07), 실제 제시된 질문에서 token을 반영하는 대화 책임(U04)을 바꾸며 U14를 주장하지 않는다. frozen `adapters.py:217~229`는 질문 객체 전체를 실제 receipt와 함께 복사한다. `:246~255`의 snapshot/restore도 질문 객체 전체를 복사하고 원래 receipt로 재구성한다. 따라서 token 필드의 보존만으로 추가 저장 schema 코드를 반드시 바꿔야 한다는 근거는 없다. frozen QA `:654`는 별도의 실제 영속 schema 이행/복원이 바뀔 때만 U14를 센다.

실제 변경 job에서 다음을 확인한다.

1. post-change Task owner의 token과 actual-presentation binding의 token을 저장한 뒤 복원해도 같은 값인가? 새로운 Runtime/owner와 대화 binding을 복원하는 과정까지 검사하는가? adapter의 in-memory deepcopy 왕복만으로 제품의 영속 재시작을 검증했다고 확대하지 않는가?
2. `change_experiments.py:434~438`의 초기 질문 재발급은 이미 token이 있는 질문에서 같은 token을 유지하는가? token 없는 과거 질문에 새 token을 발급할 때, 과거 presentation receipt에 없던 token을 실제 제시된 token이라고 소급해 만들지 않는가? 새 token을 제시해야 하는 경우의 처리와 token 보존 복원을 구별해야 한다.
3. `change_runtime_checks.py:153~172`의 `restored_initial_question_with_token`은 처음부터 token 없는 legacy 질문을 입력한다. 이것과 `change_experiments.py:530~533`의 binding snapshot 왕복이 각각 검증하는 범위는 무엇인가? 둘을 합쳐도 실제 owner와 actual binding의 동일 token 복원 근거가 남는가?
4. 실제 DB/serialized schema version, field allowlist, migration 또는 독립 restore 규칙을 바꾼 diff가 있는가? 있다면 U07/U04의 token 생성/반영과 별개인 U14 계약 delta를 제시해야 한다. 없다면 복원 test 추가만으로 U14를 늘리지 않는다.

## 나머지 과제의 실제 diff 확인 질문

| 과제 | 확인할 질문 |
| --- | --- |
| M01 | `change_experiments.py:351~365`의 provider envelope/usage 변경이 U15에 국소화되는가? 실제 변경 source의 encode/decode를 candidate port가 사용하며 원래 잘못된 JSON 출력도 그대로 유지하는가? 취소/late usage와 capability 보존을 wire 모사 회귀 수준으로 표시하는가? |
| M02 | `:366~397`의 새 wire가 실제 Agent command/receipt 경로에서 사용되는가? command ID, Task/Execution/revision, payload와 UNKNOWN 의미를 유지하며 duplicate가 effect뿐 아니라 orphan Task도 만들지 않는가? |
| M03 | `:398~414`의 정규화가 실제 source ingress에서 사용되는가? 일정의 current revision, 권한, 실제 원문/UTC→local time이 유지되는가? 외부에서 미리 정규화한 입력만 주고 새 raw port의 연결까지 통과했다고 표시하지 않는가? |
| M06 | `:346~347`의 0변경은 `:239~247`의 전체 baseline capability audit gate를 실제로 충족하는가? `change_runtime_checks.py:311~368`의 원본/파생/cache/pending model/late return/publication fence와 honest receipt, 다른 자료 유지가 모두 원본 baseline에서 성공하는가? baseline 필수 기능 결함을 신규 변화로 세거나 45B 미구현 경로의 검증을 주장하지 않는가? |

## U01~U18 동일 기능 책임 판정의 질문

파일 하나에 여러 책임이 있어도 실제 독립 계약 delta를 각각 확인한다. 여러 파일이 같은 계약을 구현하면 같은 U-ID 한 번이다. 아래 표는 실제 diff를 볼 때 사용할 질문이며 현재 changed set 판정이 아니다.

| 책임 | 실제 diff에서 분리할 경계 |
| --- | --- |
| U01 | 자연어 의미 producer에 새 의미 규칙이 필요한가? 단순 prompt 재출력과 구별한다. |
| U02 | B의 실제 항목 해결/결합/dispatch 계약이 바뀌는가? 새 필드 선택의 비용과 유한 표현 지원 구조의 필수 비용을 구별한다. |
| U03 | 공통 Meaning/Frame/WHEN 허용 문법과 검증이 바뀌는가? U02의 동일 검사 이동을 중복 세지 않는다. |
| U04 | actual-presentation focus에서 답을 반영/채택하는 계약이 바뀌는가? Task owner의 검사와 별개인가? |
| U05 | 재생/hold/시각 제어 자체가 바뀌는가? token 객체가 기존 경로를 지나간다는 이유만으로 추가하지 않는다. |
| U06 | 게시 내용/전달 원장 계약 자체가 바뀌는가? 기존 원장이 확장 객체를 그대로 보존하면 별도 변경으로 추정하지 않는다. |
| U07 | actual Task/Execution의 terminal 상태 또는 질문 token 발급/현재성 검사가 바뀌는가? |
| U08 | 실패 producer readiness, 조건 대기와 후속 실행 규칙이 바뀌는가? U07의 상태 전이와 별개 기능을 확인한다. |
| U09 | raw source를 같은 정규 원본 port로 대응하는 계약만 바뀌는가? |
| U10 | 실제 45A 검색/조합/cache의 의미가 바뀌는가? 일반 source port 변화만으로 추가하지 않는다. |
| U11 | 실제 관계 생산/coverage/무효화 책임의 신규 delta가 있는가? 이번 두 구성에서 존재 여부를 명시한다. |
| U12 | 실제 관계 Reader/부분 범위 보완 책임의 신규 delta가 있는가? 미구현 45B의 변경 수를 추정하지 않는다. |
| U13 | dialogue↔work command/acceptance/reflection의 상위 계약이 바뀌는가? 내부 token을 기존 확장 객체가 운반하면 wire 변경을 자동 추정하지 않는다. |
| U14 | 독립 영속 schema/migration/restore 계약 diff가 있는가? producer의 private 필드와 snapshot 보존을 중복 세지 않는다. |
| U15 | cloud wire/usage/cancellation 정규화 변경인가? 상위 의미가 같은지 확인한다. |
| U16 | Agent wire/capability 정규화 변경인가? normalized command/payload 의미가 같은지 확인한다. |
| U17 | 허용/동의의 정책 판정 기준 자체가 바뀌는가? 기존 질문 currentness token과 정책 범위 확대를 구별한다. |
| U18 | 여러 owner 사이의 삭제 전달/완료 계약이 별도로 바뀌는가? U10~12 내부 무효화와 같은 변경을 중복 세지 않는다. |

## 실제 검토를 시작할 때 필요한 증거

각 job의 원본 baseline hash, after-source hash, `patch.diff`, 완료/회귀 결과, probe imported source path와 관측 trace를 서로 대조한다. M04/M06은 변경 전 audit도 읽는다. recorded output replay는 같은 실제 모델 오류를 유지하는 구조 회귀이고 실제 모델 의미 정확성 재측정과 구별한다. 이후 독립 검토자는 저자의 주장 집합과 별개로 U01~U18의 필요/유지 이유를 작성하고, 최소 대안 이견을 증거로 조정한다. 이 문서는 그 전 단계의 질문만 기록한다.
