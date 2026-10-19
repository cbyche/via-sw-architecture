# 공통 보충6 입력과 실행 방법 독립 검토

2026-10-19 / **보충 source와 fresh42의 제한된 수용 종료; QA04 offline 연결 진행 가능, 정식 유료 GO 대기**

검토자: 사용자 지정 GPT-6 Astra Ultra. 이 검토자는 [계약20](../20-common-supplement-evaluation-contract.md)의 사전 명세 작성자이며, GPT-6.1 Sol High가 구현한 source와 증거를 독립적으로 읽는다. 구현, 테스트, renderer, evaluator, parser 또는 API를 실행하지 않았다. 실제 PNG 두 개는 `view_image`로 직접 보았다. 아래 판단은 범위가 명시된 source/input 검토이며 정식 실행 GO가 아니다.

## 1. 고정된 검토 대상

[Asset/fixture checkpoint1975046036436708](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/asset-fixture-draft-1975046036436708/checkpoint.json)은18개 snapshot 파일을 보존한다. checkpoint SHA256은 `99ea7cef9e7e1abd7b5c3f84222c3f5d03fbd2f31f1e11ebf1a354b5e05d776c`, 기준 계약20 SHA256은 `4be703679b161af9312b136f4056d121e4f09465bc39e0d87391cd5b068878d3`다. 이 checkpoint는 작성 중인 supplement harness, offline double, aggregation/controller의 수용을 포함하지 않는다.

검토자는 checkpoint, asset/fixture builder, population과 oracle 전체, 생성된 fixture의 초기 상태와 사건/schedule 필드, 원본 및 관측 metadata를 읽었다. 아래 파일은 실제 snapshot bytes를 독립적으로 hash 확인했다.

| Snapshot | SHA256 |
| --- | --- |
| fixtures.json | `93b504c676c1525426165a7e186d852a3c77c0d18aad0b655dbfcc0cd44e1a66` |
| oracles.json | `39372c8c56febc390753204ef50c199d479c8d06ecb3ede6191788e88d1bb833` |
| sup-dense-v1.png | `d785df6196acde445dc93d6b300af041fa72bfdfe4d984fd3fb0bf784ac07d27` |
| sup-sparse-v1.png | `9b9b1a37cf7285443855708d571f8b0b7039da078fe5e4e79f5b39aef8d9e3c0` |

전체18개에 대한 재검사 프로그램을 실행한 것은 아니다. 최종 freeze는 전체 source/asset 및 실제 실행 binary와의 mapping을 다시 확인해야 한다.

## 2. 입력과 asset 판단

### 실제 화면 확인

고정 [dense PNG](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/asset-fixture-draft-1975046036436708/source/assets/sup-dense-v1.png)와 [sparse PNG](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/asset-fixture-draft-1975046036436708/source/assets/sup-sparse-v1.png)를 실제 열어 보았다. 두 화면에 요구사항부터 보관까지12개 한국어 행 이름과 B01–B12가 올바른 순서로 보이며, 겹친 글자나 잘린 ID를 발견하지 못했다. 같은 arrow 버튼과 흰 배경을 사용한다. Dense는16px 버튼/24px 행 간격, sparse는32px 버튼/96px 행 간격이며 명세의 baseline과 위치 계산이 builder/authoring-layout에 일치한다.

글꼴은 Apple SD Gothic Neo Regular collection index0,20px다. 실제 font 파일 SHA256 `e33989af92c53dd2b80efd88f50c404094a046658d0e7a7692619587570e616c`와 Pillow12.3.0이 render manifest에 기록돼 있다. PNG는 고정 font의 glyph를 직접 그렸고, SVG에는 같은 geometry/text가 남는다. SVG를 다른 font/renderer에서 열면 raster 결과가 바뀔 수 있으므로 정확한 PNG 재현에는 기록된 font와 renderer script가 필요하다.

이 판단은 작성 화면의 가독성과 계약 일치다. 실제 parser나 원격 모델이 작은 요소를 잘 읽는다는 결과가 아니며, 화면이 실사용 UI의 대표 분포라는 판단도 아니다. 실제 모델 결과를 보고 크기나 간격을 바꾸는 것을 허용하지 않는다.

### Raw 관측과 native 범위

- S83/S84의17점 closed pointer paths는 명세의 중심, 반지름11/22px,800–1600ms 및3200–4000ms,50ms 간격을 사용한다. Cue에는 item ID/정답 bbox가 없고 원시 좌표와 시각만 있다. 원형 경로를 단일 point나 native 선택으로 바꾸지 않는다.
- S85는 S83과 같은 dense PNG hash를 사용한다. 별도로 작성된 selection actions에서 U01 B03/B04, U02 B03/B05를 선택하며,800ms의 실제 작성 native 관측에 선택된 항목의 source/revision/line range/bbox만 포함한다. 전체 scene object 목록은 입력하지 않는다.
- Builder는 evaluator oracle을 읽지 않는다. 작성된 사용자 행동과 geometry에서 관측을 만든다. `authoring-layout.json`의 전체 scene map은 renderer/검사용 파일로 남고 후보 사건에는 포함되지 않는다.
- Frame은 작성된 rendered screen이라는 evidence class를 유지한다. 상용 앱의 실제 API 또는 Windows capture를 검증했다고 표기할 수 없다. Native에서 원본 pixels가 같은 것은 이 명세에 명시된 조건이며, 별도의 선택 metadata가 actual 작성 관측이다.

### 원본, 권한과 기대값

SUP-RELEASE의8개 문단, SUP-MEETING 한 줄, SUP-SCREEN12개 line/extent 및 정확한 여섯 종류 요청 문구가 계약20과 일치하는 builder를 확인했다. 원본과 conversation log는 local read true/external work sharing false다. 공통 합성200턴과 완료20Task 및5개 기존 슬롯을 보존하고, 초기 historical-delivered는 빈 배열이다. 후보에게 새로운 의미 요약이나 미래 결과를 넣은 흔적을 발견하지 못했다.

Fixture는 candidate용 initial/current event와 host용 전체 schedule을 구분한다. 검토한 initial/event keys에는 expected set, expected goal, 기능 정답 또는 전체 미래 schedule이 없다. 이 구조적 분리는 실제 harness가 initial과 current event만 전송할 때 성립하며, 그 최종 source/trace 확인은 별도 항목이다.

Oracle의 S80/S81 독립 목표2개, S82–S85의 U01/U02를 가진 정정 목표1개, S83–S85의 최초 지칭2개가 사전 명세와 일치한다. Expected sets는 U01 B03/B04, U02 B03/B05이며 전체 set exact match다. 예정 분모는 한 묶음12응답/8목표/6판정, 구성당3반복36/24/18이다. 모든7구성에 같은 조건을 적용한다.

## 3. 발견한 보완 사항

| ID | 발견과 영향 | 필요한 최소 조치 | 상태 |
| --- | --- | --- | --- |
| SUP-01, P3 | `build_fixtures.py`의 S80/S82 U02 `original_trigger` 설명은 `final response + 2s`이지만 typed trigger는 올바르게 `after_actual_provided`다. 첫 입력 설명도 `t=0`으로 남아 있어 Tinit+5초의 실제 정의와 provenance가 어긋난다. | 설명 필드도 actual successful U01 provided_at+2초와 Tinit+5초를 정확히 명시한다. typed timing이나 사전 입력 조건은 변경하지 않는다. 새 fixture checkpoint에 기록한다. | 담당자에게 전달, 수정 확인 전. 다른 구현 작업을 막는 source/geometry 결함은 아니다. |

기능 진단의 수용 확인도 남는다. 현재 oracle `required_content`는 모든 응답에 일반 문구만 기록한다. 계약20 §4의 외부 공개 불가 조건, 내부 링크 공유 제한 및 회의 시각/장소 등 구체적 답변 조건은 별도의 host-only 기능 진단으로 보존해야 한다. 이것을 QA01/QA03의 기존 endpoint 의미나 QA02 분모를 바꾸는 검증으로 사용하지 않는다. 아직 작성 중인 evaluator/분석기를 읽지 않았으므로 이 항목은 최종 mapping 확인이며 실행 실패 발견이 아니다.

## 4. 현재 결론과 남은 수용

**검토한 작성 원본, geometry, raw cue, native provenance, 예정 집합 및 입력/oracle 분리는 계약20과 일치한다.** SUP-01의 작은 설명 수정 외에 이 checkpoint의 입력을 다시 설계해야 할 문제는 발견하지 못했다. 이 판단은 asset draft 범위이며 구현 전체 또는126 정식 실행의 GO가 아니다.

다음 고정 source checkpoint에서는 actual-provided+2초 scheduler, final/fallback 구분, Text0초와 전사6초 source 경계, 공통 clock/receipt/cancel 보존, 명시적인 보충 oracle과 독립 aggregation 분모, source/fixture/asset mapping을 검토한다. 실제42개 API-free 연결 회귀, finite boundary 검증, QA04 각 복사본의 추가6사례, 공동315+126 비용 projection,441행 고정 순서 및 최종 freeze/GO가 아직 필요하다. 이전 common9/probe의 성공을 이 수용 결과로 상속하지 않는다.

## 5. 별도 harness와 집계 source 검토

§1–4는 최초 asset 검토 당시 상태다. 후속 고정 대상은 [source checkpoint1975473700232250](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/source-checkpoint-1975473700232250/checkpoint.json), SHA256 `3d4133f829feb75d73bd1945f343a41021e30a5870f74094f04d9f3461f8be71`의25개 snapshot이다. 이 source 세대의 harness SHA는 `f6f39960c7796f911144a59d6701099413c0d9c2fc4f20d423e6e8001f4795d9`, 집계 adapter SHA는 `fbf42b62ff755cb1f903e025a06dbef1ebd7e642aef7c94319fb5ce4970335ef`다. 후보 binary `5aea60533c5b2a46c0e22cf8a7e4d86d51b4ba2c3860f6006341f0f073dd2e1d`와 공통9 core는 변경하지 않았다.

다음 source 경로는 계약20에 맞게 구현됐음을 확인했다.

- **SUP-01 설명 수정 확인.** 첫 입력은 initial actual provided+5초, S80/S82의 U02는 성공한 U01 stdin flush의 actual provided+2초로 typed trigger와 설명이 일치한다. source due와 actual 제공 시각을 구분하며 Text0초는 begin을 capture보다 먼저 enqueue한다. S83–85의6초 전사 입력도 별도 보존한다.
- Candidate에는 initial과 현재 event만 전송한다. 전체 schedule, expected sets, 기능 정답은 host에 남는다. 별도 `functional-oracles.json`은 계약의 외부 공개 조건, 내부 검토 링크 제한, 회의 시각/장소와 행 이름을 보존하며 `MANUAL_REVIEW_REQUIRED_NOT_QA01_FILTER`로 표시한다. 이 수동 기능 진단을 offline PASS나 QA01 정답으로 자동 인정하지 않는다.
- 별도 harness는 공통 method-v3의 clock proof, 실제 source-send bracket, publication receipt, cutoff, 실제 취소와120초 정산, shared K2와 전역 reservation ledger를 유지한다. 실제 제공 trigger만 별도 확장한다. QA04의 crate 지정과 C03 입력 변환은 offline 경로에 한정된다.
- 별도 evaluator는 변경하지 않은 method-v3 per-trace 평가기에 명시적 보충 oracle을 전달한다. 집계는 자체12/8/6과3반복, 구성당36/24/18 예정 분모를 사용한다. duplicate/unexpected identity는 정상 평균을 무효화하고 missing 행을 남긴다. 기술 미증명 trace의 timing은 raw 진단으로 남으며 certified primary에 합치지 않는다. QA02 ordinary wrong과 technical unknown을 구분하고 known-correct의 고정 분모 lower bound를 표시한다. 실제 비용과 unknown reservation은 유지하고, incomplete 비용의 대표 평균과 RSS의 완전성 주장을 제한한다.
- QA04는 동일42개 실험 참조로 표시한다. 새126개의 모듈 변경 측정으로 세지 않는다. Offline double은 작성 geometry와 현재 cue를 사용하는 결정적 대체물이며 실제 parser/model 성능 증거가 아니다.

의도된 고정 모집단의 집계 공식에서 추가 blocker를 발견하지 못했다. 이 판단은 별도 controller, activation 분석, 최종 source mapping과 전체 patched-copy 수용을 대신하지 않는다.

## 6. 실제 final/fallback 경계 발견과 수정 추적

| ID | 발견과 영향 | 최소 조치와 증거 | 상태 |
| --- | --- | --- | --- |
| SUP-02, P2 | 위 checkpoint harness는 final의 실제 reader 관측이 U01 actual provided+30초보다 늦어도, 다음 main poll 전에 dictionary에 들어오면 missing-at30을 억제했다. 계약의 fallback 경계와 다음 입력의 latency root가 달라질 수 있다. | 실제 final 관측이 경계 이하이면 그 시각, 그렇지 않으면 정확한30초 경계를 선택하고 late actual final을 별도 원시 사실로 보존한다. | 순수 경계 함수와 실제 늦은/큐 대기 검사를 읽어 조건식 수정 확인. 아래 동시성 잔여 항목 때문에 최종 수용 대기. |
| SUP-02 동시성 잔여 | reader의 `observed_ns` 채집, final timestamp dictionary 저장, parent 저장이 서로 분리돼 있고 main의 fallback 선택과 공통 잠금이 없다. 제때 관측한 final이 shared record에 아직 게시되지 않은 사이 timeout을 선택하거나 parent가 빠질 수 있다. | final 관측 시각과 timestamp/parent record를 같은 짧은 임계 구역에서 기록하고, main의 snapshot/anchor 선택도 같은 잠금으로 보호한다. IO나 wait는 포함하지 않는다. 관측 직후 게시 전 pause의 결정적 경계 증거를 남긴다. | 구현 담당과 root에 전달. 추가 후보 정책이나 새 조건을 요구하지 않는다. |

후속 실제 읽기 대상 [contract20-checks-v2 summary](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/contract20-checks-v2/summary.json)의 SHA256은 `03248832ecd9bf0109d099c9f86e20d5fa94c5898440cf1f0f45727e34710c8d`다. 기록된65 finite checks와4 actual harness cases는 모두 PASS다. 이 세대 harness SHA `80d3291bb610e826f789c70a6f6fe09e8557ef05ef1932b9bd4e9e6a7d031475`, `trigger_rules.py` SHA `c49fb68e9fc53f188dde7c6e23797a761d6552fb1b603b5817c022b520129b7c`를 실제 파일 hash와 대조했다. 검토자가 검사를 다시 실행한 것은 아니다.

네 실제 검사는 missing final,31초를 축소한 늦은 final, 제때 수신했지만 main queue에서31초 대기한 final, 잘못된 DELEGATE다. 모두 API0이며 인위적 wait는 기술 검사 전용으로 명시돼 있다. 늦은 사례의 실제 raw에서 fallback `1975994496488750`과 늦은 actual final `1975994559067791`을 각각 보존함을 확인했다. fallback 선택 때 final이 없었다면 당시 `late_actual_final_ns:null`은 맞으며, 나중 실제 final은 `actual_response_final_ns`에 남는다. 실제 검사의 늦은 사례는 fallback 이후 final 도착을 검증하고, 늦은 final이 다음 poll 전에 도착한 분기는 finite helper 검사가 담당한다. 모든 thread interleaving을 검증했다는 주장은 하지 않는다.

기존 offline-baseline-v1의42/42 PASS는 보존된 중간 수용 증거다. 수정된 scheduler의 fresh42와 위 동시성 경계 수용이 아직 필요하다. 잘못된 import로 공통 harness가 선택된 첫 checker 실패도 보존됐으며, v2 checker의 명시적인 보충 harness 파일 import를 확인했다.

## 7. 공동441 비용 판단

[공동 projection v1](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/preflight/joint441-cost-projection-v1.json)의 SHA256은 `e99645f5582c9ce802b36786a18d2977205d929257aa82d8ed1602dcfae91f2a`다. source 계산과 실제 기록을 읽었으며 새 가격 조회나 모델 호출을 수행하지 않았다. 이 projection은 당시 offline source traces에 기초한 계획 자료다. 최종 scheduler evidence와 freeze의 provenance mapping은 다시 고정해야 한다.

| 가정 | 이전 확정 지출을 포함한 총액 |
| --- | ---: |
| ordinary, 입력3bytes/token 및 output cap와 image byte proxy | $30.3569025 |
| 모든 의미 workflow의 MAX3 slot 사용, 같은 token proxy | $47.7324109 |
| full-byte reservation 및 output cap/image byte의 보수적 계획 proxy | $100.7697944 |

확정 이전 지출은 $0.7150616, unknown/inflight는0이며 남은 상한은 $39.2849384다.3UTF-8bytes/token과 image byte proxy는 provider의 검증된 tokenizer나 보편적 상한이 아니다. Offline double은 실제 model의 repair 횟수나 경로를 예측하지 않는다. Reservation proxy를 전체441에 합친 값은 settlement 후에도 그 액수가 동시에 묶여 있다는 뜻이 아니다.

**현재 정보에서는 기존441과 $40 승인 및 계약20의 고정 block 중단 규칙을 유지하는 것이 합리적이다.** Ordinary 가정의 전체 여유는 약 $9.6431이고, 일곱 구성 한 block의 가장 큰 보수적 계획액은 S02의 약 $3.3561이다. 따라서 ordinary 시나리오에서는 block 계획 확인 자체가 완주를 강제로 막지 않는다. 높은 stress projection만으로 즉시 사례 축소나 추가 예산 승인을 요구할 근거는 부족하다. 이 판단은 실제 완주 확률이나 완주 보장이 아니다.

Paid GO 전에441회 완료가 불확실하고 hard cap 또는 block check에서 고정 분모를 유지한 INCOMPLETE가 될 수 있음을 명시해야 한다. 순서, 구성별 선택, repair MAX3,100call cap, output cap, 모델, 사례와 반복 수를 그대로 두며 결과를 보고 싼 구성을 채워 넣지 않는다. 이 상태에서 추가 사용자 예산 선택은 필수 선행 조건이 아니다. 수정된 현실적 projection이 $40과 합리적으로 양립하지 않거나 실제 cap 도달 후 완주가 필요하면 그 시점에 구체적 예산/범위 선택을 받아야 한다.

새 모델, repair 축소, output cap 또는 반복 수 변경은 모든7구성에 적용하더라도 측정할 실험을 바꾼다. 동등한 과학적 조건을 보존하는 개선은 입력과 호출 정책을 바꾸지 않는 projection 정확화, provenance 및 accounting 보완이다. 현재 후보 정책을 낮추는 조치는 필요하지 않다. 최종 paid GO는 scheduler 수용, QA04, 공동 controller와 activation 검토 및 정확한 freeze의 별도 판단을 기다린다.

## 8. Atomic scheduler 수용과 실행 증거의 해석 범위

### SUP-02 source 및 경계 수용 종료

[Source closure checkpoint1976298543605583](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/source-closure-checkpoint-1976298543605583/checkpoint.json)의 SHA256은 `c6f6c20aaa10fd2eba1487ed5e6ab45ecd1f3076d7e194280814ede4243ff121`다. 이 세대의 harness SHA는 `21e799cf933b479414c33b2ff7ec1d53461746988b3338461116e69f86676ce6`, trigger registry SHA는 `9285b3b176c5e51cd614f3c738f598a75009ec76750d44dcd1b4284756bbd1a3`다.

`FinalAnchorRegistry.observe`는 공통 잠금을 얻은 뒤 실제 reader 시각과 parent를 함께 기록한다. `choose`는 같은 잠금 안에서 관측값을 읽고 final 또는 missing-at30 경계를 한 번만 저장한다. Production 임계 구역 안에는 IO, wait 또는 외부 callback이 없다. 따라서 이미 찍었으나 게시하지 않은 시각을 timeout 선택이 추월하는 SUP-02 잔여 race를 닫는다. 이 시각은 짧은 registry 잠금 뒤의 실제 host 수신 관측이며, socket 또는 Rust 발신 시각이라고 주장하지 않는다.

[contract20-checks-v3](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/contract20-checks-v3/summary.json), SHA256 `4dfb1e6291d5c025cbcea40d4da2f3c635acd8344c366263d06526e859e4b259`는68 finite와4 actual harness cases PASS를 기록한다. 추가 finite는 실제 thread 두 개로 timestamp 이후 dictionary 게시를 의도적으로 멈추고, 경계를 지난 chooser가 동일 mutex를 기다린 뒤 제때의 timestamp와 parent를 보존하는지 확인한다. 이런 wait는 검사 안에만 있다. 기존 missing/late/timely-queued/wrong-delegate 실제 cases도 새 source로 PASS다. 검토자는 source, manifest와 실제 결과를 읽었다. **SUP-02는 이 source와 경계 검사 범위에서 CLOSED다.** 최종 fresh42와 정확한 전체 mapping은 별도 완료 항목이다.

### 공동 controller와 모집단 분리

`joint_campaign.py` SHA `9314f4f36f58ae2254e0f2d0cb33d4fe5debba5c43f450b696154068745c8d50`의 순서, block budget 검사, shared ledger, root/global controller locks, 예외 해제, orphan/duplicate 탐지 및 technical/cap 중단 경로를 읽었다. 기존315 상대 순서와126 교차 삽입 규칙을 유지하며, 정확한 authorization manifest가 없으면 dispatch하지 않는다. 실제 시작했으나 terminal 기록이 없는 identity를 자동 재실행하지 않는다. 검토한 새 공동 controller 경로에서 추가 blocker를 발견하지 못했다. 이 판단은 임의의 외부 수동 실행까지 공통 잠금이 통제한다는 주장이 아니다.

`analyze_joint.py` SHA `469a5c8a57ee1a3e6812b8c85182cc0e4b2fbd59b69fbca8eeb8591a0fcecb49`는 단일 freeze/root의 binary, phase, 고유 예정 identity와 실제 per-run raw 일치를 확인하고315와126을 각각5반복과3반복으로 집계한다. 이전 probe나 raw generation을 합치지 않으며 미실행 행을 남긴다. Shared AAAA의 상관된 재사용, activation이 모집단 filter가 아니라는 제한과 동일 QA04 참조를 명시한다. 최종 freeze와 raw가 생기기 전이므로 이 분석기가441개의 실제 결과를 확인했다는 뜻은 아니다.

### DP2 조건을 검증할 수 있는 범위

아래 판단은 유료126 결과 전에 root가 채택했다. 입력, 예측 또는 분모를 변경하지 않는다.

- `INPUT_CAPTURED.observed_ns`는 원시 입력 instrumentation을 host가 받은 시각이다. `STATE_EFFECT_COMMIT / INPUT_CAPTURE / linearization_ns`는 의미 입력 상태의 실제 commit 경계다. 두 시각을 따로 기록하며 commit을 원래 raw 입력 도착 시각으로 부르지 않는다. 실제 clock/commit proof가 일관된 trace만 조건 판정에 사용한다.
- U02의 확인된 semantic input commit이 U01에 속한 실제 cloud-call interval 안에 있으면, U01 호출이 진행 중일 때 U02가 의미 입력 상태에 수용됐음을 보일 수 있다. U01/U02의 올바른 요청 대응과 양의 actual call overlap은 **두 요청의 의미 실행 경로가 실제 model call을 동시에 진행했다**는 좁은 기제 증거다. 이 interval은 host admission부터 terminal까지이며 transport를 포함한다. Provider 내부 compute 병행을 증명하지 않는다.
- 동일 조건, 정확한 의미 결과와 QA01/03 차이가 함께 관측되면 해당 fixture에서 실현된 경로의 지지로 해석할 수 있다. 입력 도착 순간의 정확한 빈 K2 슬롯, worker 종료 시각 또는 경합이 없는 조건의 순수 latency 효과는 확인하지 못한다. Host operation start/end envelope는 worker 전체 수명 증거가 아니다.
- Overlap이 없고 당시 capacity/worker 상태가 unresolved라면 비발동과 미확인 원인을 남긴다. 유리한 조건에서 병행이 도움이 될 수 있다는 가설을 그 행만으로 반증하지 않는다. S82의 같은 목표 정정을 잘못 독립 목표로 해석한 overlap도 독립 병행 이익으로 세지 않는다. 모든 행은 원래126 분모에 남는다.

현재 목적에는 이 제한을 가진 실제 호출 겹침과 요청 대응 증거가 충분하다. 정확한 초기 free capacity나 worker lifetime에 대한 더 강한 주장이 필요하면 slot-release/worker-terminal의 새로운 사전 계측과 수용이 필요하다. 현재 core freeze를 넓히는 변경은 요구하지 않는다.

### Activation 진단의 보완 원장

초기 `mechanism.py`의 dispatch-to-terminal call 수는 전체 K2 점유 수로 표현하면 안 된다. 실제 slot은 dispatch 전에도 보유되며 release clock이 별도로 없다. 또한 raw capture receipt와 semantic commit을 구분해야 한다. 후속 source는 이 경계를 명명하고 정확한 free capacity를 UNRESOLVED로 남기며, native skip은 helper spawn과 parser dispatch가 모두 없는지 확인한다. [joint-guard-v2](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/joint-guard-v2/summary.json)의18 finite PASS를 읽었다. 이 증거는 synthetic extraction 및 OS lock 검사이며 실제 model activation 결과가 아니다.

마지막 남은 진단 항목은 failed/cancelled call의 slot interval이다. 공통 bridge는 일부 예외의 `terminal_ns`를 slot 해제 뒤 outer handler에서 기록하므로, 모든 `slot_acquired_ns…terminal_ns`를 점유 lower bound로 세면 과장될 수 있다. 최소 수정은 실제 slot 안에서 terminal을 기록한 것이 확인되는 completed call만 held-slot 증거로 사용하고 나머지는 unresolved로 남기는 것이다. 이 항목과 최종 진단 source hash/finite 확인은 진행 중이며 core/기본 정책 변경을 요구하지 않는다.

## 9. 보충 source 최종 수용: QA04 offline 연결 범위

최종 읽기 대상은 [final-v2-source-checkpoint1976508465529083](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/final-v2-source-checkpoint-1976508465529083/checkpoint.json), SHA256 `4f41d17a5df73bc915f93a0d2ac854af25eacc185c452aecb502c2e99e4a026e`다.34개 보충 source와22개 QA04 runtime dependency, 기존 core84개 및 binary `5aea605…d2e1d` mapping을 기록한다. 계약20 hash는 `4be703…878d3` 그대로다. Fresh42 뒤 변경은 진단 전용 `mechanism.py`와 그 검사이며 runtime, harness, fixture 및 evaluator는 바꾸지 않았다.

§8의 마지막 항목은 수정 확인 후 **CLOSED**다. 최종 `mechanism.py` SHA256 `9eba155a9de36a2b098e11b92abf0fe17cff692e6bfe66ee69163ea8c55c6fe1`은 completed call만 확정 slot/dispatch interval 증거로 삼고 실패/취소는 unresolved로 보존한다. [joint-guard-v3-final](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/development/joint-guard-v3-final/summary.json), SHA256 `d62c0029ca3b978be6be9816cf5a2c517db0e2a1846b78c3d78df2e28210cb87`의19 finite PASS에는 slot 획득10, 실제 해제12, outer terminal40, capture15의 합성 취소 사례를 점유 증거로 세지 않는 확인이 포함된다. 긍정적 activation 판정에도 이 보수적 완료 조건을 적용하며, 제외한 호출의 실제 비용이나 원래 모집단 행을 삭제하지 않는다.

| 최종 실제 증거 | 읽기 확인 결과 |
| --- | --- |
| [offline-baseline-v3-final summary](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/offline-baseline-v3-final/summary.json), SHA `26374a58be6d1e70dc9f94b3fc89fbd14bbe709b92b1ab17c141b4e788103d5d` | 새42개 모두 exit0, regression PASS와 technical CONSISTENT. 응답84, 목표56, exact sets42. API0. |
| [최종 trace 수용](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/offline-baseline-v3-final/contract20-trace-acceptance.json), SHA `f86273e92bd85df49c6464f15acab1abe19f979a3c65679a71e1554d2fca1768` |70 checks PASS. Source/trigger 경계와 모든 구성의12/8/6, 누락/중복/technical-unknown 집계 수용을 기록한다. |
| 최종 checkpoint의 실행 mapping | 실행 전 closure snapshot과 runtime source 일치, core84 unchanged,42 RSS OBSERVED를 기록한다. |
| [공동 projection v3-final](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/preflight/joint441-cost-projection-v3-final.json), SHA `772022548d75ad1e2b53d09c1357b4c3b1ffe0401d80e83b3291366049fe897b` | 최종 보충42와 기존 최종 공통9 offline63을 provenance로 사용한다. §7의 ordinary $30.3569025, MAX3 $47.7324109, reservation proxy $100.7697944와 이전 지출/unknown 수치는 동일하다. |

검토자는 프로그램을 실행하거나 전체 raw를 새로 재평가하지 않았다. 실제 source, 고정 manifest, 체크 구현 및 기록된 수용 결과를 읽은 독립 판단이다. 이42개는 축소된 시각과 결정적 double을 사용하는 연결 검사이며 실제 모델 의미 정확도, 지연시간, token, learned parser 또는126회의 예측 성립 결과가 아니다.

**이 고정 source에서 검토한 입력, scheduler, 집계, controller 및 activation 진단의 남은 blocker는 없다. QA04 dependency freeze와42개 실제 변경 복사본의 offline matrix를 진행할 수 있다.** Root는 이 제한된 판단을 채택했다. 이것이 QA04 변경 측정 자체 또는315/126 유료 정식 실행의 GO는 아니다.

다음 단계에는 실제 QA04의42개 변경 복사본 및630개 회귀 수용, 정확한 공동 formal source/112개 review snapshot과441행 manifest 검토, 최신 ledger와 egress/provenance, 독립 paid GO가 남는다. Prepared review snapshot만으로 API를 허용하지 않는다. 과거 source/checker 실패, intermediate42와 모든 raw는 보존하며 정식 결과에 합치지 않는다.

## 10. 공동441 review snapshot의 최종 방법 검토

대상은 [joint441-readiness-snapshot-v1 freeze](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/preflight/joint441-readiness-snapshot-v1/freeze.json), SHA256 `779f811570da28e0503d4398c0d459379199f94124a8c8d2b394e370fd7816fd`다. 이 root는 review 전용이며 `MODEL-CALLS-AUTHORIZED.json`이 없다. 검토자는 후보, 검사, 평가 프로그램 또는 API를 실행하지 않았고 source/JSON 읽기와 독립 hash 대조만 수행했다.

### 독립 mapping 확인

- 112개 source 모두 live와 frozen 사본의 bytes가 manifest hash와 일치한다. 기존 probe와 공통인77개 source는 변경이 없으며, 최종 보충 checkpoint의34개 source도 모두 포함되고 hash가 같다.112개에 probe84개 전부가 들어 있다는 뜻은 아니다.
- Frozen binary는 `5aea60533c5b2a46c0e22cf8a7e4d86d51b4ba2c3860f6006341f0f073dd2e1d`, 복사된 공동 projection은 `772022548d75ad1e2b53d09c1357b4c3b1ffe0401d80e83b3291366049fe897b`와 실제 일치한다.
- External dependency22개의 크기/mtime은 manifest와 일치한다. 기존21개 dependency record는 그대로이고 추가1개는 고정 rendering font다. 이번 검토에서 큰 weight bytes를 다시 읽어 hash하지 않았다. 실제 launch의 전체 dependency 검증은 기존 controller 경로에서 수행해야 한다.
- 441개 identity는 모두 고유하고63개 block 각각에7구성이 있다. Primary315의 상대 순서는 보존된 기존315 plan과 같으며, 보충 반복별 조건 순서는 §8 사전 명세의0/2/4칸 회전과 일치한다. K2,100call cap, MAX3 및 모델/output 정책은 그대로다.
- [Empty-plan acceptance](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/preflight/joint441-readiness-snapshot-v1/empty-plan-analysis-acceptance.json), SHA `6791fa45ed4a56799a7d35992f2eb379850b6ea93053750ff15645cff1a204aa`는441개가 모두 미실행이어도 공통9의 구성별215응답/115목표/75판정과 보충36/24/18이 따로 남음을 확인한다. 실제 결과가 없으므로 이것은 모집단/분모 연결 수용이다.

실제 final source, clock/receipt/cancel, technical stop과120초 settlement, budget hard gate, 모든 raw 비용/unknown 보존 및 두 모집단별 집계는 앞서 닫은 source와 같다. Ordinary 모델 오류는 원래 outcome으로 유지하고 technical failure/cap은 controller를 중단한다. 이미 시작한 identity의 자동 재시도, 저렴한 구성의 선택 완주 또는 이전 probe pooling을 추가하지 않는다. §7의 budget 판단과 §8의 DP2 관측 한계도 동일하다.

### 인간의 전송 동의와 실제 자료 범위

[최종 egress proof](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/preflight/joint441-egress-provenance-v2-final.json)의 SHA256은 `5a6334d229e133ebe684024168b8aa50762dec0d97be7ef4f24878dc50da7964`다. 연결된 인간 동의 원문과 addendum을 직접 읽었으며, 동의/fixture/builder/oracle/bridge/harness의 기록된 reference hash를 실제 파일과 대조해 불일치가 없음을 확인했다.

인간에게 제시된 범위는 합성 대화와 Task 기록, 작성한 시험 문서/화면, 실험 지시문을 Alibaba Cloud DashScope Singapore의 Qwen으로 보내는 고정2회, 대표315회와 보완126회이며 총 $40이다. 실제 답변은 **“해당 시험 자료의 DashScope 전송 승인”**이다. 원문 기록 SHA는 `0f5693693cabe033c6b8a9e2c201a36661009e9a0b24c79e5e72f21b60a05c9b`, 초기 addendum SHA는 `ed2dcadd7deb48b10547a9638ad51f56b7032e1f2020babcc15cf18e3895e991`다. 현재9+6 authored fixtures, 동일 Singapore endpoint/model과 같은 상한은 이 동의 범위에 들어간다. 새 목적지 동의를 다시 받을 필요가 없다. 전송 동의는 formal 실행 GO와 구분한다.

현재 builder/fixture와 payload 경로에서 실제 개인 파일, 실제 메일/일정, 저장소 구현 source text, Git 내용 또는 비밀값을 model input으로 수집하는 경로를 발견하지 못했다.15사례의 원본은 inline authored text이며 초기 source에 file-backed `path`가 없다. 화면은 공통9의 작성 PNG2개와 보충 dense/sparse PNG2개의 고정 assets다. 합성 과거200턴과 Task 상태, 현재 event, 실제 model-derived 해석/표시/합성 receiver 결과는 입력될 수 있다. Oracle, 미래 schedule과 전체 renderer geometry는 평가/host에 남는다. Fixture에 `PRIVATE_ATTACHMENT_DO_NOT_DISCLOSE` 같은 합성 권한 시험 문구가 있다는 사실을 실제 개인 파일 수집으로 오해하지 않는다.

Bridge는 `payload.input`, 실험 instruction과 명시된 fixture image bytes로 request body를 만든다. Credential은 의도한 provider 인증 header에서만 사용하고 prompt/body evidence로 직렬화하지 않는다. 검토자는 credential이나 실제 routing 값을 읽지 않았다. 이 판단은 고정된 작성 자료와 현재 source 경로에 한정된다. 인간의 과거 동의를 새 실제 개인 자료나 다른 목적지에 확대하지 않는다.

### 최종 formal artifact에 남겨야 하는 방법 계약

112개 review snapshot에서 빠진 기존 probe7개 중5개는 probe 전용 계획/도구/이전 승인 자료다. 다음2개는 현재 정식 방법에 계속 적용되므로 최종 formal manifest 또는 immutable approval artifact가 정확한 hash로 함께 묶어야 한다.

| 현재 적용되는 계약 | SHA256 |
| --- | --- |
| `docs/architecture/12-decisions/decision-packages/seven-problem-review/reviews/common9-newer-completion-endpoint-addendum-20261019.md` | `fce69c1b86cd8a7002a77b07265b6b7d83660b2135ff42d46f5dcc8af27d950f` |
| `results/architecture-evaluation/current/common9-poc-20261019/preflight/deadline-correction-design-proposal.md` | `238f7472c9d2c452680f977483d4c997870aa65f6ea13071b86c3fb5a4450369` |

Deadline proposal은 최초 설계 문서이며 후속 receipt/technical-abort 세부 수정은 실제 수용된 method closure와 함께 해석한다. 최종 묶음은 두 계약, 현재 method/supplement closure의 immutable 사본, 실제 QA04 수용 및 독립 검토, 최종 egress proof와 위 consent records, 정확한 formal freeze/projection을 연결해야 한다. 계속 편집되는 review 파일을 나중에도 같은 hash인 것처럼 참조하지 않는다. 새 formal root를 만들며 이 review snapshot은 보존한다. 이 보완은 provenance 묶음 작업이며 runtime 변경, 새 회귀 또는 새 인간 전송 허가를 요구하지 않는다.

**최종 제한 판단:** 검토한112개 source와441 plan, 전송 동의 적용, budget/분모/실패 보존 방법에는 남은 source blocker가 없다. 실제 QA0442/630 및 독립 수용을 완료하고 위 방법/승인 artifact를 정확한 새 formal hash에 묶으면 root의 paid GO 판단으로 진행할 수 있다. 현재 판단은 review snapshot 수용이며 유료 실행을 승인하지 않는다. Root는 알려진 QA04와 artifact binding을 남긴 이 조건부 판단을 채택했다.

## 11. 새 formal root의 계약 staging 검토

새 [joint-formal-v1 freeze](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json)의 SHA256은 `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd`다. [contract-staging-v1](../../../../../../results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/approval/contract-staging-v1.json)의 SHA256은 `761a0cb5b2963fd64345e705ebbff3c0219e9279662589f96e536ab251005af8`다. 검토자는 두 파일,18개 계약/증거 사본과 staging source를 읽고 bytes/hash를 대조했다. 코드, 검사, 후보 또는 API를 실행하지 않았다.

확인 시점에18개 사본과 기록된 원본의 hash/크기는 모두 staging 기록과 일치했다.112개 live/frozen source에도 불일치가 없었으며 source map,441행 jobs, block 계획액, binary, projection 및22개 external dependency records가 기존779f review snapshot과 같았다. 이번에도 큰 weight bytes는 다시 hash하지 않았다. Formal raw trace는0개이고 `MODEL-CALLS-AUTHORIZED.json`과 `approval/APPROVAL-BINDING.json`은 존재하지 않았다. 실제 ledger는 상한 $40, 확정 $0.7150616, unknown/inflight0이었다.

§10의 두 operative contract는 정확한 immutable 사본으로 보존돼 **provenance 누락이 해결됐다.** Deadline proposal의 최초 설계 status, 후속 receipt/technical-abort closure의 적용, 과거 consent addendum의 fixed2 실행 한계도 staging에서 구분한다. 기존 동의를 새441 실행 승인으로 잘못 바꾸지 않는다.19/20, method/supplement closure, egress/consent, 이전 $40 승인, 최종63/42와 projection이 함께 묶였다.

Count-v2 결과 `55f87e66f4b511ed74686f6a5e66e0f42f3287febddb28cadbb3ecd5f5841130`의 staging 상태가 `PENDING_INDEPENDENT_REVIEW`인 것은 생성 당시 기록이다. 이후 전담 QA04 Astra reviewer가 수정된42개 source classification을 제한적으로 수용한 사실은 새로운 최종 binding에서 해당 review hash와 함께 명시한다. 원래 classification 파일이나 staging의 과거 status를 소급 변경하지 않는다. 이 검토자는 QA04의 복사본별 module/630 raw 감사를 중복 수행하지 않았다.

### 유료 GO 전에 남은 정확한 순서

1. 실제42개 QA04 변경 복사본의 feature/build 결과와630개 strict 기능 회귀를 모두 닫고, corrected count-v2를 사용하는 최종 report/checkpoint를 만든다. 전담 Grounding의 전체 raw 감사와 QA04 Astra의 최종 독립 수용을 연결한다. 부분 완료를 전체 성공으로 올리지 않는다.
2. RSS는 별도 상태로 보존한다. 현재 확인된 `AAAA/C05/AAAA-S02-r1-ce7376`, `BAAA/C04/BAAA-S82-r1-92f834`의 teardown invalid samples는 frozen QA06 `INCOMPLETE`와 lower-bound로 남는다. QA04의 사전 functional gate가 통과했다는 사실과 양립하며, 관측 성공으로 재분류하거나 재실행하지 않는다. 최종630 전체 RSS 상태 수와 affected rows를 기능 수용 수와 나란히 기록한다. Formal QA06 population으로 합치지 않는다.
3. 새 immutable `APPROVAL-BINDING.json`에 이 formal freeze, staging, 기존18개 정확한 사본, 완료된 QA04 report/checkpoint/count 수용, 독립 raw 감사와 QA04 최종 review, 이 staging 방법 검토의 고정 사본을 hash로 묶는다. 필요한 최신 review는 별도 사본으로 추가하고 기존 사본은 보존한다. 원본 review가 이후 편집되더라도 이미 고정한 사본의 bytes를 바꾸지 않는다.
4. Root의 paid GO와 authorization이 **formal freeze hash와 최종 binding hash 둘 다** 명시하도록 한다. 현재 controller는 freeze/source와 authorization/jobs를 검사하지만 새 `final_approval_binding_requirement` 필드 자체를 실행 코드에서 읽어 검증하지 않는다. 따라서 최종 binding과 모든 연결 사본의 hash 검증은 root의 실제 prelaunch 승인 절차에서 수행하고 증거를 남겨야 한다. Descriptor가 있다는 사실만으로 검증됐다고 선언하지 않는다. 이를 위해 현재 runtime을 수정할 필요는 없다.
5. 그 후 기존 전체 dependency validation, 최신 reservation-aware ledger, 가벼운 host load/isolation 확인과 단일 controller 경로를 사용한다. API는 exact GO 이전에 실행하지 않으며 실패/예산 중단과 원래 분모 유지 규칙을 바꾸지 않는다.

**판정: 계약 staging 수용.** 현재 추가 source 또는 전송 동의 blocker는 없다. 남은 항목은 실제 QA04 전체 수용과 그 결과를 포함한 최종 immutable binding/GO다. 새 인간 예산/목적지 허가, runtime 변경 또는 이 staging만을 위한 회귀 재실행은 요구하지 않는다. 이 판단 자체는 유료 실행 승인이 아니다.
