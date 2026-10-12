# DP41 QA05 추가 helper 준비 기록

이 경로는 별도 변경 실험 helper의 준비와 API 없는 구조 검증이다. 공식 P21 측정 결과나 책임 수를 확정하지 않는다. 후보, frozen-source, evaluator-tools 및 원래 12개 변경 job은 수정하지 않는다.

## 현재 실행 순서

1. V5 timing 210건이 끝나 `ALL_WORKERS_COMPLETE`가 기록될 때까지 original12 driver가 대기한다. `invalidated.json`이 있으면 종료한다.
2. V5의 immutable evaluator-tools에서 원래 12개 job을 실행하고 원시 diff, source, baseline audit, runtime probe와 21개 recorded semantic replay를 보존한다.
3. 추가 helper를 독립 immutable generation으로 복사하고 파일 SHA와 generation SHA를 고정한다. `run_additional_v2.py`는 그 `helper-manifest.json`을 요구한다. 이 준비 경로 자체를 검증 완료 generation으로 취급하지 않는다.
4. 원래 12개 job의 source에서 같은 `producer_lineage_probe.py`로 A/B 동일 Request 두 revision의 collision과 stale owner 소비를 검증하고 원시 결함 증거를 보존한다. 원래 A producer 표현을 유지한 lineage 보완과 typed-v2 B M04를 같은 frozen baseline의 새 copy에서 각각 실행한다. 원래 patch를 생략하거나 기존 change source를 수정하지 않는다.
5. 원래 A/B M05 change source를 읽기 전용으로 새 프로세스 두 번 실행해, 새 질문의 실제 token 발급과 presentation에서 저장/복원/실제 답변까지 추가 검증한다.
6. 실제 새 M04 자연어는 root가 별도 Qwen 및 judge 원장으로 검증한다. 이 agent와 추가 실행 framework는 API를 호출하지 않는다.
7. root와 contract reviewer가 실제 source/diff/회귀/evidence를 독립 검토한다. root가 `selection-manifest`로 최종 QA05 집계에 사용할 job을 명시한다. author는 책임 수를 확정하지 않는다.

원래 12 job과 대안은 같은 job 이름이더라도 다른 evidence 경로를 사용한다. 기본 대안 결과 경로는 `changes-alternatives/TYPED-GOALREF-v2/CH-DP41-B-M04/`이며 M05 보조 증거는 `changes-supplemental/CH-DP41-{A,B}-M05-PUBLISHED-TOKEN-RESTART-v1/`이다. 기존 `reviewed-result.json` 검색으로 두 M04를 함께 합산하지 않도록 root의 선택 manifest가 필요하다.

## 보존된 첫 번째 대안

`goalref-v1/`는 문자열 `GOAL:<from_goal>` namespace를 검증하고 Request별 actual Task를 해석하는 대안이다. B binder byte는 그대로이며 준비 검증 16개가 통과했다. 그러나 기존 합법적인 owner ID가 `GOAL:report`일 때 producer namespace와 충돌할 수 있다. 이 제약 때문에 최종 최소안으로 미리 채택하지 않는다. source, helper와 준비 증거를 그대로 보존한다.

## typed-v2의 실제 계약

`goalref-v2/`는 기존 owner string과 `reference_id: {producer_goal_id: from_goal}`를 구별한다. producer tag는 정확히 하나의 key만 갖고, 값은 이 relation의 현재 producer goal ID 문자열이어야 한다. producer predicate는 `field=STATE`, `EQ FAILED|COMPLETED` 또는 중복 없는 길이 1~2의 terminal enum `IN`만 허용한다. 일반 owner string의 기존 STATE/KIND/YEAR 및 EQ/NE/IN은 유지한다.

기존 B `_replace_refs`는 tag의 문자열을 재귀 처리하지만 Frame goal ID와 reference ID가 disjoint이므로 producer goal을 owner binding으로 바꾸지 않는다. 실제 job은 B binder의 before/after byte SHA 동일을 확인한다. B가 마지막에 전체 의미를 새 모델로 만드는 hybrid를 추가하지 않는다.

공통 continuation은 `(Request id, VIA Request revision, goal id)`를 실제 dispatch의 Task ID, Execution ID와 Task request revision에 연결한다. 해당 producer가 없거나 owner state가 UNKNOWN이거나 다른 Task Execution/revision이 되면 UNKNOWN을 유지한다. 실제 실패는 owner의 FAILURE ingress/currentness 검사에서 확정하며 임의 성공 artifact를 주입하지 않는다.

검증 범위는 실제 `CREATE_REPORT` Task producer의 두 terminal 분기다. direct/core goal의 일반적 terminal 상태 지원이나 실제 Agent의 보고서 작성 품질은 주장하지 않는다. AFTER/USE_RESULT는 성공 producer 결과가 필요한 기존 계약을 유지한다.

`semantic_change_probe_typed.py`는 frozen actual semantic validator를 복사한 파일이다. strict singleton producer tag의 판정과 helper provenance logging만 추가했다. 실제 후보 출력이나 실행 payload를 보정하지 않는다. root가 timing 뒤 실행할 수 있으며 이 준비 과정에서는 호출하지 않았다.

## M05 fresh process 복원

`m05-restart-v1/m05_restart_probe.py`의 첫 프로세스는 빈 질문 상태에서 실제 QUESTION 사건을 받는다. Task owner가 새 token을 발급하고 실제 Text presentation 및 delivered receipt가 생긴 질문과 binding을 JSON으로 저장한다. 둘째 프로세스는 그 owner 상태를 새 LocalVIA에 로드하고 기존 generic `PresentedQuestionBinding.restore`로 실제 publication receipt를 복원한다.

재시작 전후 token과 실제 publication ID가 동일한지, 재제시를 만들어내지 않았는지, 손상된 dialogue token이 actual Task gate에서 거절되는지, 같은 token의 실제 ANSWER 명령이 한 번만 접수되는지 확인한다. 기존 M05 source는 읽기 전용이며 추가 schema migration 책임을 자동으로 세지 않는다. legacy 질문에 새 token을 덧붙인 초기화와 post-change 실제 제시 token의 복원은 별도 증거다.

## 원래 A 표현을 유지하는 lineage 보완

독립 reviewer는 원래 M04의 `(Request id, goal id)` mapping과 기존 명령의 `(Request id, VIA Request revision, goal id)` idempotency key가 다르다는 결함을 코드로 확인했다. 새 revision의 producer Task가 이전 revision의 producer 슬롯을 덮을 수 있다. 이는 DP41 A/B의 의미 판단 차이로 점수화할 구조적 손실이 아니라 변경 실험 구현 결함이며 원래 결과를 보존한 보완이 필요하다.

`original-a-lineage-v1/lineage_patch.py`는 원래 A의 `producer_goal_id` 표현과 enum/prompt를 유지한다. common runtime의 mapping key, 호출 scope와 실제 Task/Execution/Task revision guard만 typed-v2 B와 동등하게 보완한다. A.py byte SHA는 유지하며 원래 A M04와의 추가 보완은 기존 U08 producer/continuation association 책임으로 기록한다. 실제 FAILURE 수용의 U07과 같은 association을 중복 세지 않는다.

`producer_lineage_probe.py`는 original A, original B, corrected A와 typed B에 같은 기대 조건을 적용한다. 같은 Request의 revision 1은 실제 report 실패 후 draft 한 번, revision 2는 실제 report 성공 후 failure-only draft 0을 요구한다. 실제 wrong Execution/Task revision FAILURE는 거절돼야 하고, actual SET_PROPERTY로 owner Task revision이 바뀐 뒤 이전 producer tag는 UNKNOWN이어야 한다. original source에서는 실패가 예상되더라도 PASS로 뒤집지 않고 FAIL 원시 행을 보존한다. A는 scripted Meaning으로 실행 상태를 검증하고 B는 scripted Frame을 실제 B resolver/read/binding에 통과시킨다. 이 보조 probe는 실제 모델 정확성을 주장하지 않는다.

`run_corrected_a.py`는 timing과 원래 12 job 완료를 확인한 후 original collision 증거를 보존하고 `changes-alternatives/ORIGINAL-A-LINEAGE-v1/CH-DP41-A-M04/`를 생성한다. 같은 frozen baseline의 새 copy에 전체 M04 feature를 적용하므로 원래 A job에서 기능을 선주입받지 않는다. full P21 replay와 기존 M04 runtime checks 및 추가 lineage 검사에 통과해야 하며 두 독립 리뷰 전에 책임 수를 확정하지 않는다. 자연어 A 검증은 root가 원래 semantic validator에 corrected A source를 제공해 별도 실행한다.

A 보완 helper와 공통 lineage probe는 V5 formal timing 중 코드로만 작성했다. V5가 공통 질문 follow-up 결함으로 정지한 뒤 AST와 실제 준비 검증을 수행했다. 독립 helper generation의 고정, 새 캠페인의 실제 job/full P21/API 및 책임 수는 아직 완료하지 않았다. 후보, 원래 12 source, frozen-source와 evaluator-tools는 변경하지 않았다. B typed 최종 job도 A와 같은 `producer_lineage_probe.py`를 추가 required regression으로 실행한다.

## 준비 검증 증거

- `preflight-3708773f/summary.json`: V4 immutable source의 독립 copy로 v1의 16 runtime checks와 M05 A/B fresh process 검사 통과. V4 공식 캠페인은 중단됐으며 이 검증은 helper 준비 증거일 뿐이다.
- `v5-compatibility-37bd1842/summary.json`: V5 원래 12개 patch 적용 및 AST parse 성공, M05 A/B actual published token restart와 v1 16 checks 통과. V5 original/evaluator source는 불변이다.
- `v5-typed-reference-preflight-c9c89e85/`: helper의 f-string brace 결함으로 import가 실패한 원시 증거. 후보 결함이나 최종 QA 실패로 바꾸지 않는다.
- `v5-typed-reference-preflight-3a5c9abe/summary.json`: brace 수정 뒤 typed-v2 16 checks가 통과했다. strict tag와 Frame disjointness, Request 분리, 같은 Request의 두 revision, 실제 잘못된 FAILURE lineage, 실제 SET_PROPERTY 뒤 stale producer UNKNOWN, 실제 owner `GOAL:report`의 B resolve/binding/execution 보존을 포함한다.
- `v5-lineage-preflight-f98f4eee/summary.json`: V5가 정지한 뒤 같은 실제 producer-lineage probe를 적용했다. Original A/B는 같은 Request 새 revision이 기존 producer 슬롯을 덮어 실패했고, actual SET_PROPERTY 후 old producer가 달라진 Task revision을 소비해 실패했다. Corrected A/typed B는 동일 검사와 source 불변 검사를 통과했다. Corrected A는 기존 M04 12 named runtime checks도 통과했다. 원래 실패를 기대 결과라는 이유로 PASS로 뒤집지 않았다.
- `v6-active-compatibility-545498a3/summary.json`: V6 공통 수정 검수 중 active source를 읽기 전용으로 복사했다. Corrected A patch/AST 및 기존 M04 12 checks, typed B patch/AST 및 16 checks와 양안의 동일 lineage 검사가 통과했다. M05 A/B의 실제 제시 token fresh process 복원도 통과했다. baseline copy와 active source는 검사 중 불변이었다. 이후 예정된 callback reservation 해제 수정과 최종 V6 freeze의 검증을 대신하지 않는다.

모든 준비 검증은 API 0이며 공식 timing 시작 전에 실행했다. 공식 timing 동안에는 CPU probe와 API를 실행하지 않는다. 원래 12 job의 `(Request id, goal id)` mapping collision은 코드 독립 리뷰로 확인했고, timing 뒤 동일 보조 probe로 실제 결함 증거를 기록한다. 최종 변화량과 QA05 점수는 실제 V5 job, full P21 replay와 두 리뷰 이후에만 산출한다.


## V6 동결 호환성과 명시적 선호 보존 감사

`v6-frozen-compatibility-60c522c1/summary.json`은 최종 V6의 51파일 digest `bcf3802bfad401059554d288b5f7d303ece0e16743fb43b62eb57cebbc075636`에서 corrected A 12검사와 lineage, typed B 16검사와 lineage, M05 A/B fresh process 복원을 확인한 API 0 준비 증거다. 원래 12개 patch의 anchor와 AST도 통과했다.

V6의 별도 `qa05-additional-helper-v1`은 10파일 digest `9c3c5d8c590a3d8407aec598a535d48da0a418c4fe9c373cdc3a1c94bbfba8cb`로 고정했다. 대기 driver는 원래 12개 다음 corrected A와 typed B 및 M05를 순서대로 실행하도록 연결했지만, V6이 공통 User Memory 구현 누락으로 공식 실행 전에 제외되어 `WAIT_ABORTED_CAMPAIGN_INVALIDATED`로 종료했다. 원래 12개 job과 추가 job 실행은 모두 0이다. 이 세대의 driver와 helper는 보존한다.

QA §6.5가 보존한 S1-03의 정확한 입력은 “앞으로 비교 결과는 표로 보여주는 걸 선호한다고 기억해줘.”다. `m06-exact-preference-v1/`의 첫 검사는 실제 private 저장 이후 common owner 조회가 빈 결과여서 A/B 모두 C5에 실패했다. B의 파생 등록 회귀에는 declared reference가 없는 scripted Frame fixture 오류가 있었으며 그 오류를 후보 결함으로 세지 않는다. `v6-exact-preference-missing-common-port-ae14dc31/`에 원시 실패를 보존했다.

`m06-exact-preference-v2/`는 B의 실제 declared-reference/read/bind와 native `response=None`의 실제 CRESPONSE 전달까지 검사한다. 응답 모사는 실제 `registered=True` 사실과 실제 owner Memory 값이 주어진 경우에만 지정 등록 문장을 반환한다. evaluator가 빈 owner port를 private dict로 투사하거나 정답 key/value/scope를 후보에 덧씌우지 않는다. 현재 사용자, 지속 범위, 실제 고유 ref/revision, M언어 및 무관 선호 보존, 다른 사용자 Memory 차단, 실제 파생 등록 이후 철회와 정직한 처리 receipt를 관측한다.

`v6-exact-preference-v2-before-368f4312/summary.json`은 동일 V6 baseline에서 A/B의 C1~4와 실제 철회 회귀가 통과하고, C5 공통 조회와 C6 실제 응답 전달, 사용자 partition 및 owner ref/revision이 실패한 원시 증거다. 두안의 동결 소스는 불변이다. 이 공통 baseline 누락은 M06의 신규 기능 변경량으로 세지 않으며, root가 승인한 공통 보정 후 새 동결본을 준비한다.

`run_m06_preservation.py`는 공식 timing과 원래 12 job 완료 뒤 원래 baseline과 각 M06 독립 source에서 같은 원본 oracle를 각각 실행한다. before/after 기록은 별도 source hash와 raw로 저장하며 P21 정확성 분모에는 추가하지 않는다. 전체 기능에 통과한 증거와 두 독립 책임 리뷰가 있기 전에는 0변경 수를 확정하지 않는다.

위의 이전 V5 최종 실행 계획은 후속 새 캠페인 계획으로 이월됐으며 V1~V6의 공식 QA05 job은 아직 실행되지 않았다. 실제 새 캠페인의 완료, 별도 자연어 모델 검증과 두 리뷰 뒤에만 최종 변화량을 산출한다.
