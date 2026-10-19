# COMMON_SUPPLEMENT6 S85 r1 독립 의미 검토 v1

검토자: GPT-6 Astra Ultra. 정식 실행 후 진단이며 사전 승인이나 QA 재평가가 아니다. 승인된 ordinal 11, job indexes 77–83의 일곱 구성만 검토했다. 후보, 계약, 입력, 동결 소스, 점수 및 분모를 변경하지 않았다.

기계 판독 기록: [COMMON_SUPPLEMENT6-S85-r1-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON_SUPPLEMENT6-S85-r1-v1.json) — SHA-256 `e7962d8e07c6c5e2b3c8b92fd9a66395b4207a82f8688b8eb2cda340a4a65398`, 765897 bytes. 아래 각 행의 원본 파일 해시, 크기, JSON pointer, 실제 입력 및 모델 request/response 파일 연결은 이 JSON에 보존했다.

## 1. 결과와 집계 단위

14개 응답 의무 중 실제 표시 본문 11개는 해당 시점의 선택 및 요청과 일치했다. 나머지 3개는 U02 본문이 없어 `UNREVIEWED`이다. 없는 본문을 의미 오답으로 만들지 않는다. 그러나 해당 세 구성의 최종 정정 목표는 충족되지 않았다.

| 항목 | 수 | 해석 |
| --- | ---: | --- |
| 계획된 응답 의무 | 14 | 각 구성 U01과 U02 |
| 관측 본문 PASS | 11 | U01 일곱 개와 U02 네 개 |
| 관측 본문 FAIL | 0 | 표시된 본문에 한정 |
| 본문 없음 / UNREVIEWED | 3 | AAAB, AAAA, CAAA의 U02 |
| 고유 실제 DISPLAY | 11 | 중복 실제 표시 0 |
| 모든 matching callback 참조 | 15 | U02 네 개가 U01 origin에도 연결되어 생긴 추가 참조 4개 |
| 계획된 목표 관계 | 7 | 추가 독립 응답 성공으로 합산하지 않음 |
| 실제 정정 관계 및 최종 요구 충족 | 4 | AAAC, BAAA, ABAA, AABA |
| 정정 미채택 / 관계 UNREVIEWED / 최종 요구 미충족 | 3 | AAAB, AAAA, CAAA |
| 원시 HOST_COMPLETE | 7 | 그중 3개는 이전 U01 revision 1만 완료 |
| 실제 제공된 입력 | 14 | 입력 미제공으로 대체 해석하지 않음 |

U01은 각 구성에서 자체 답변이 표시된 뒤 U02가 제공되었다. U01 본문을 나중에 폐기된 목적 때문에 실패로 바꾸지 않는다. 동시에 U01의 완료를 U02의 정정된 최종 목표 충족으로 재사용하지 않는다. 공유 callback 참조 네 개는 추가 본문이나 중복 표시가 아니다.

## 2. 실제 요청, 선택 데이터와 권한

- U01: `표시한 두 버튼의 행 이름을 알려줘.` 실제 native 선택은 B03과 B04이다.
- U02: `직전 요청을 정정할게. 지금 표시한 두 버튼의 행 이름을 알려줘.` 실제 native 선택은 B03과 B05이다.
- 실제 SUP-SCREEN revision 1의 행은 `B03 | 설계 | 설계 문서 열기`, `B04 | 검토 | 검토 문서 열기`, `B05 | 구현 | 구현 문서 열기`이다. 행 이름을 요청했으므로 뒤의 문서 열기 문구는 실행 권한이 아니다.
- 모든 U01의 실제 사용 원문은 3–4행이며, 채택된 U02는 3행과 5행을 사용했다. 표시 본문과 채택된 source claim을 실제 제공 원문에 대조했다.
- 관측된 채택 계획의 allowed_actions, completed_actions, permission_operations 및 control_operations는 빈 배열이다. 문서 열기, 수정, 외부 전송, Agent handoff 및 승인 절차는 수행되지 않았다. 승인과 handoff는 `NOT_APPLICABLE`이며 PASS로 합산하지 않는다.

정답 원문과 실제 요청은 이 사례에서 일치한다. 동결 functional oracle은 실제 입력과 분리해 대조했으며, oracle을 모델 입력으로 간주하지 않았다.

### Native 관측 범위

이 입력은 `AUTHORED_SELECTION_ADAPTER_OF_RENDERED_SOURCE`가 만든 `AUTHORED_RENDERED_SCREEN_NOT_CAPTURED_WINDOWS_APP`이다. 각 입력에는 두 개의 selected-only item과 native cue가 있고 `complete_scene_objects=false`이다. pointer path, move, drag는 없다. 실제 Windows 또는 상용 앱 API의 구현이나 동작 검증을 뜻하지 않는다.

프레임은 sup-dense-v1, 2880×1800이다. B03/B04/B05의 authored pixel bbox는 각각 [1400,412,1416,428], [1400,436,1416,452], [1400,460,1416,476]이다. S83과 동일한 dense PNG SHA-256은 `d785df6196acde445dc93d6b300af041fa72bfdfe4d984fd3fb0bf784ac07d27`이다. 이번 검토에서는 이미지 bytes/hash를 확인했고 새 렌더링이나 base64 decode를 하지 않았다.

전체 native-object inventory가 없다는 사실과 원래 전체 프레임 PNG 또는 원문이 없다는 주장은 다르다. 여섯 joint 경로의 실제 request에는 원본 PNG와 텍스트가 포함되어 있다. 선택 데이터는 원래 화면 내용 전체를 객체 목록으로 제공한 것이 아니다.

## 3. 모든 응답 의무

`/events/N`은 각 행의 run trace 안의 원본 pointer이다. 이 검토는 승인된 bundle을 사용했으며 원본 trace를 새로 읽거나 평가하지 않았다. 원본 trace 해시는 §7 및 JSON의 각 raw_ref에 있다.

| 구성 | 의무 | 실제 capture | 실제 adoption | 표시 / raw first | 본문 판정 |
| --- | --- | --- | --- | --- | --- |
| AAAB | S85-RU01 | `/events/17` / `event-21` | `/events/50` / `event-58` | `/events/52` / `event-62` | PASS / OBSERVED |
| AAAB | S85-RU02 | `/events/78` / `event-92` | 없음 | 없음 | UNREVIEWED / NOT_OBSERVED |
| AAAC | S85-RU01 | `/events/6` / `event-7` | `/events/33` / `event-38` | `/events/35` / `event-42` | PASS / OBSERVED |
| AAAC | S85-RU02 | `/events/47` / `event-54` | `/events/76` / `event-87` | `/events/78` / `event-91` | PASS / OBSERVED |
| AAAA | S85-RU01 | `/events/6` / `event-7` | `/events/33` / `event-38` | `/events/35` / `event-42` | PASS / OBSERVED |
| AAAA | S85-RU02 | `/events/47` / `event-54` | 없음 | 없음 | UNREVIEWED / NOT_OBSERVED |
| BAAA | S85-RU01 | `/events/6` / `event-7` | `/events/30` / `event-35` | `/events/32` / `event-39` | PASS / OBSERVED |
| BAAA | S85-RU02 | `/events/44` / `event-51` | `/events/72` / `event-84` | `/events/74` / `event-88` | PASS / OBSERVED |
| CAAA | S85-RU01 | `/events/15` / `event-19` | `/events/46` / `event-54` | `/events/48` / `event-58` | PASS / OBSERVED |
| CAAA | S85-RU02 | `/events/72` / `event-86` | 없음 | 없음 | UNREVIEWED / NOT_OBSERVED |
| ABAA | S85-RU01 | `/events/6` / `event-7` | `/events/33` / `event-38` | `/events/35` / `event-42` | PASS / OBSERVED |
| ABAA | S85-RU02 | `/events/47` / `event-54` | `/events/76` / `event-87` | `/events/78` / `event-91` | PASS / OBSERVED |
| AABA | S85-RU01 | `/events/6` / `event-7` | `/events/35` / `event-40` | `/events/37` / `event-44` | PASS / OBSERVED |
| AABA | S85-RU02 | `/events/49` / `event-56` | `/events/80` / `event-91` | `/events/82` / `event-95` | PASS / OBSERVED |

관측된 U01 일곱 본문은 모두 다음과 같다.

```text
B03: 설계
B04: 검토
```

관측된 U02 네 본문은 모두 다음과 같다.

```text
B03: 설계
B05: 구현
```

## 4. 최종 정정 목표와 현재 관계

동결 목표 조건은 `Final corrected request revision; old U01 completion alone is insufficient; no external action`이다. 다음 표는 원시 HOST_COMPLETE와 이 조건을 구분한다.

| 구성 | 실제 최종 goal / revision / last event | 최종 request 원본 | 정정 관계 | 최종 요구 |
| --- | --- | --- | --- | --- |
| AAAB | `goal:S85-U01` / 1 / `S85-U01` | `/events/110/data/requests/goal:S85-U01` / `event-127` | UNREVIEWED_CORRECTION_NOT_ADOPTED | UNMET_OLD_U01_ONLY |
| AAAC | `goal:S85-U01` / 2 / `S85-U02` | `/events/89/data/requests/goal:S85-U01` / `event-102` | PASS | OBSERVED_CURRENT_CORRECTED_GOAL |
| AAAA | `goal:S85-U01` / 1 / `S85-U01` | `/events/76/data/requests/goal:S85-U01` / `event-87` | UNREVIEWED_CORRECTION_NOT_ADOPTED | UNMET_OLD_U01_ONLY |
| BAAA | `goal:S85-U01` / 2 / `S85-U02` | `/events/85/data/requests/goal:S85-U01` / `event-99` | PASS | OBSERVED_CURRENT_CORRECTED_GOAL |
| CAAA | `goal:S85-U01` / 1 / `S85-U01` | `/events/104/data/requests/goal:S85-U01` / `event-121` | UNREVIEWED_CORRECTION_NOT_ADOPTED | UNMET_OLD_U01_ONLY |
| ABAA | `goal:S85-U01` / 2 / `S85-U02` | `/events/89/data/requests/goal:S85-U01` / `event-102` | PASS | OBSERVED_CURRENT_CORRECTED_GOAL |
| AABA | `goal:S85-U01` / 2 / `S85-U02` | `/events/93/data/requests/goal:S85-U01` / `event-106` | PASS | OBSERVED_CURRENT_CORRECTED_GOAL |

AAAC, BAAA, ABAA, AABA는 실제 U02 채택이 원래 goal:S85-U01의 revision 2를 만들고 정정된 본문을 표시했다. AAAB, AAAA, CAAA는 U02 채택 자체가 없고 revision 1, last_event S85-U01의 HOST_COMPLETE만 남았다. 이들은 최종 요구가 미충족이며, 미관측 정정 관계를 올바르다고 가정하지 않는다.

부수적인 원시 relation 필드도 보존한다. AAAB, AAAC, AAAA, CAAA, ABAA의 U01은 새 질문인데 ANSWER와 자기 U01 target_event를 제출했다. 실제 상태는 U01 goal을 만들었으므로 이 라벨만으로 다른 목표가 채택되었다고 판단하지 않는다. BAAA의 U02는 target_event_id=S85-U02와 target_goal_id=goal:S85-U01을 함께 제출했다. 이벤트 필드는 불일치하지만 실제 채택은 올바른 기존 목표의 revision 2이다. 본문과 실제 관계의 제한된 PASS가 모든 원시 필드가 완벽하다는 뜻은 아니다.

## 5. 세 U02 실패의 구체적 원인

AAAB, AAAA, CAAA의 U02 raw joint proposal은 세 실행 모두 의도된 B03과 B05를 식별했다. 실패는 선택된 native 데이터나 행 이름의 오인식으로 분류하지 않는다. 각 실행의 두 번 시도, 총 여섯 번은 extent.item_id에 bare visible ID 대신 `SUP-SCREEN:v1:B03` 및 `SUP-SCREEN:v1:B05`를 넣어 거절되었다.

실제 request는 visible item_id와 host canonicalization을 명시한다. 두 번째 시도에는 `Visible item_id must be nonempty without canonical prefix` 오류도 제공되었지만 같은 오류가 반복되었다. 동결 wire.rs 3–14행의 canonical 함수는 비어 있거나 colon을 포함한 extent item_id를 거절한 뒤 source_id:vREV:item_id를 구성한다. 성공한 다른 U02 joint proposal은 canonical target 문자열을 유지하더라도 extent.item_id는 bare B03/B05로 제출했다. 따라서 정확한 실패 필드는 extent.item_id이며, 일반적인 모든 canonical reference가 금지된다는 뜻이 아니다.

| 구성 | rejudgment 원본 | 최종 workflow 실패 | 관측 결과 |
| --- | --- | --- | --- |
| AAAB | /events/102 event-118, /events/107 event-124 | /events/108 event-125 | U02 미채택, 미표시, 기존 U01만 완료 |
| AAAA | /events/68 event-78, /events/73 event-84 | /events/74 event-85 | U02 미채택, 미표시, 기존 U01만 완료 |
| CAAA | /events/96 event-112, /events/101 event-118 | /events/102 event-119 | U02 미채택, 미표시, 기존 U01만 완료 |

각 실제 request/response 파일의 SHA, 크기, /calls/N 연결 및 text payload는 JSON의 configuration_reviews[].model_facts에 있다. 해당 U02는 AAAB calls 6/7, AAAA calls 4/5, CAAA calls 6/7이다. 첫 거절 뒤 나머지 모든 기하 필드가 유효했는지는 UNREVIEWED이다. 올바른 ID 의도만으로 전체 binding, 실제 채택 또는 완료를 인증하지 않는다.

세 실행의 RESPONSE_ANCHOR_TIMEOUT도 그대로 유지한다. 실제 입력은 제공되었으나 U02 응답이 없었고 late_actual_final_ns는 null이다. 이를 입력 누락이나 뒤늦은 실제 표시로 바꾸지 않는다.

## 6. 실제 호출 구조와 중단 조건의 범위

AABA에서는 U01의 GROUNDING_NATIVE_CANDIDATES /events/21 event-24와 GROUNDING_TYPED_BIND /events/26 event-30, U02의 /events/66 event-75와 /events/71 event-81을 확인했다. 실제 Speech Reference Producer request 두 개와 meaning request도 확인했다. 이는 관측된 native 후보 및 typed binding 경로의 근거이다. 전체 calls/process 목록을 새로 감사하지 않았으므로 local parser 또는 helper의 완전한 비호출을 독립 인증하지 않는다. parser 비용 절감이나 지연 수치를 이 기록에서 산출하지 않는다.

여섯 다른 구성은 실제 joint text+PNG 경로를 사용했다. S83과 S85가 같은 dense PNG를 쓰더라도 cue 제공 방식이 다르고, 이번 기록은 한 조건의 첫 반복이다. 밀도만의 인과 효과, 실제 Windows 기능, provider 계산 동시성 또는 DP 우열을 결론내리지 않는다.

새 기술 중단 사유는 이 제한된 검토에서 확인되지 않았다. 일곱 copied terminal record의 해시와 크기를 확인했으며 모두 returncode=0, technical_failure=false, budget_stop=false이다. 동결 supplemental harness 295–310행은 기술 abort/control 실패와 보존할 ordinary RESPONSE_ANCHOR_TIMEOUT을 구분한다. joint_campaign.py 153–194행은 동결 검증, authorization/HOLD, 예산 및 terminal stop 조건을 적용한다. 여기서 관측한 세 실패는 실제 제공 입력에 대한 후보/model joint-wire 실패이다.

이는 새로운 clock 또는 endpoint 인증이 아니다. 동결 contract19 99–111행의 의미/기능 진단과 시간 관측의 분리를 유지한다. QA01, QA02, QA03의 기존 endpoint와 분모를 바꾸거나 본문 PASS만 남기지 않는다. 코드나 프롬프트 수정, 취소, 튜닝, 재시도 또는 다음 cursor 진행을 요청하지 않는다.

## 7. 정확한 증거 연결

- Freeze: [freeze.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json), SHA-256 `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd`.
- Accepted manifest: [manifest.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S85-r1-v3-r2/manifest.json), SHA-256 `4b8dbaf716cf0193809d9cbaf4e47af2d337ed41cca8d8569304ae5956591f54`.
- Evidence: [evidence.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S85-r1-v3-r2/evidence.json), SHA-256 `cefd72a9e7e69401607bc65e2c8a48f2d51ceeedfd49bd06a3ec879168a72135`.
- Worklist: [worklist.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S85-r1-v3-r2/worklist.json), SHA-256 `4e2c321862d3a074f8e0bcc837125f6d24967c60de6ea5465cc3f8d6ae9ed0d1`.
- Index: [index.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON_SUPPLEMENT6-S85-r1-v3-r2/index.json), SHA-256 `5abf8df47703374c0d9a4edee55d4a0ec003687edcfb4651c6c5d99dd917ed52`.
- Execution receipt: [EXECUTION-RECORD.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON_SUPPLEMENT6-S85-r1-v3-r2/EXECUTION-RECORD.json), SHA-256 `273527125ca8b02e0bdd078c63151d24df2e2edf49a3ae00c48f0f4098a28bf8`.

### 원본 trace 식별자

아래 해시는 승인된 bundle의 바인딩이며, 이 검토에서 원본 trace를 재추출하지 않았다.

| Run | 원본 trace SHA-256 | bytes |
| --- | --- | ---: |
| [AAAB-S85-r1-f2f2c3](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAB-S85-r1-f2f2c3/trace.json) | `b02c91afd24763991723fa8c82c39b3e0b471e16919b1347bab15104f8a2dc97` | 3677899 |
| [AAAC-S85-r1-908fc4](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAC-S85-r1-908fc4/trace.json) | `1549623024f798382ab7f1a499a1bd37000c11e80cd7bbc6c41f38de55c6f133` | 3183599 |
| [AAAA-S85-r1-917c40](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAA-S85-r1-917c40/trace.json) | `91bf54eb42ef4a6281d163fde2b4bea2b511c030f3932098a9231c5a48f179b9` | 3144211 |
| [BAAA-S85-r1-17787a](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/BAAA-S85-r1-17787a/trace.json) | `6ca534124fde17ecccaa97d5fbdb1ebdf083dafa5692dd2f6de15231e9ef4c6c` | 3231715 |
| [CAAA-S85-r1-816e8e](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/CAAA-S85-r1-816e8e/trace.json) | `e77763d641a0651ee582f770370fb1d70e80adbcec1eddeea4cd3be09149f9e4` | 3523026 |
| [ABAA-S85-r1-de6ba9](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/ABAA-S85-r1-de6ba9/trace.json) | `7e4e448b0ef952e7aec8360e49b270445a56a8a32013c3f0c6093e4cac97491b` | 3274300 |
| [AABA-S85-r1-0782c0](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AABA-S85-r1-0782c0/trace.json) | `ad0453c0a728b03046b815af2fbdfaef9f7fbd2a94f8dc0658c0e3a0bdff2394` | 3251920 |

### 동결 source 연결

| 역할 | 파일 | SHA-256 |
| --- | --- | --- |
| fixtures | [fixtures.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/fixtures.json) | `d53502887c7305b99428eab8f4d391821ba2c6d5d4b77c20d7fa1176f1c92e97` |
| oracles | [oracles.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/oracles.json) | `39372c8c56febc390753204ef50c199d479c8d06ecb3ede6191788e88d1bb833` |
| functional_oracles | [functional-oracles.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/functional-oracles.json) | `484d1f00c708bea7dc010c54e37d98831b77ee77c188d1c6be888da89484ffaa` |
| selection_actions | [selection-actions.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/assets/selection-actions.json) | `de3b7d9e0a8c7be8202d960dc7446ebf80cd58c0e53c69b8fbca632a022d54cb` |
| observations | [observations.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/assets/observations.json) | `7116f2c068e127bc555832e95e90dc22d200fe862914eecc65b9426e429cbd63` |
| dense_png | [sup-dense-v1.png](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/assets/sup-dense-v1.png) | `d785df6196acde445dc93d6b300af041fa72bfdfe4d984fd3fb0bf784ac07d27` |
| contract19 | [19-common9-evaluation-contract.md](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md) | `7b3db7b8f358ee6cd441ca37ef13f9b83d1f697322729a0f7bde802bbdc162e7` |
| contract20 | [20-common-supplement-evaluation-contract.md](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/docs/architecture/12-decisions/decision-packages/seven-problem-review/20-common-supplement-evaluation-contract.md) | `4be703679b161af9312b136f4056d121e4f09465bc39e0d87391cd5b068878d3` |
| runtime | [runtime.rs](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/prototypes/candidates/common9_poc_20261019/src/runtime.rs) | `d96f61406119d870bf65bb9045a8fe2bb8eb1aa68ae8319a65a09845a4437402` |
| wire | [wire.rs](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/prototypes/candidates/common9_poc_20261019/src/grounding/wire.rs) | `913158227067ba4743d84a40a75cb8b7eddc5faca2fa322d80d41b557821d62a` |
| joint | [joint.rs](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/prototypes/candidates/common9_poc_20261019/src/grounding/joint.rs) | `390e75fdfbcac0d8e62f2afc103c5e560b42e579a40baad8e038b4aa1cf35674` |
| harness | [harness.py](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/harness.py) | `21e799cf933b479414c33b2ff7ec1d53461746988b3338461116e69f86676ce6` |
| joint_campaign | [joint_campaign.py](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/joint_campaign.py) | `9314f4f36f58ae2254e0f2d0cb33d4fe5debba5c43f450b696154068745c8d50` |

## 8. 남는 제한

- 14개 응답 의무와 7개 목표 관계를 모두 기록했지만, 미표시 본문 3개와 미채택 정정 관계 3개는 UNREVIEWED로 남는다. 최종 요구 미충족은 별도 기록이다.
- 승인된 raw-first bundle에는 saved certified endpoint가 없다. 이 검토의 raw callback은 인증된 QA01 선택값이 아니다.
- 권한 취소, 실제 외부 실행, Agent handoff, 승인, 무관한 동시 source 변경 및 post-cutoff commit 경합은 이 사례에서 검증하지 않았다.
- 모델의 올바른 대상 의도, 채택된 현재 집합, 표시 본문 및 최종 정정 목표를 별도 증거로 취급했다.
- 새 전체 trace 읽기, evaluator/analyzer, model/parser, test/build, 네트워크 또는 API 호출은 0이다. frozen 파일 변경과 cursor 진행도 없다.
