# DP2, DP3, DP5, DP44 현재 작업 패키지

> **2026-10-23 QA 구체화:** [대표값/부록 계약](qa/measurement-appendix.md)과 [QA04의 8개 변경 과제](qa/change-scenarios.md)를 적용한다. QA01은 P95 없이 평균 2초와 NORMAL/WAIT 분리, QA02/QA07은 각각 단일 비율로 별도 유지한다. QA06은 전체 제품 예산이다. QA04-CHANGE-v4는 Rust/Python 등 실제 운영 모듈을 세며 v6에 보존된 기존 6과제의 추정값과 호환되지 않는다. 새 QA04 값은 미산출이며 기존 평가 파일과 PPTX는 보존한다. 후속 담당 작업은 [구체화 전달 계약](qa/refinement-handoff.md)을 따른다.

2026-10-22. 사용자가 확인한 설계, 발표 v10, 공통 QA v3와 QA 목표 및 점수의 원전을 한곳에 모은 **현재 편집 기준**이다. 앞으로 이 폴더에서 네 DP와 관련 QA를 수정한다. 이전 위치는 출처와 과거 평가의 재현을 위해 보존한다. 이관으로 설계 대안, QA 정의와 목표, 점수, 평가 수치 또는 사용자 채택 상태를 바꾸지 않았다.

## 바로 열기

**2026-10-23 QA 정의 합의:** QA01은 NORMAL 첫 음성 평균 2초와 별점, WAIT의 별도 보고로 구분했다. QA03은 전체 목표의 VIA 처리 완료 평균 5초다. 두 목표와 점수 구간을 확정했다. QA09는 제거했다. 사용자가 담당 세션의 평가 전체 갱신을 요청했다. [측정 계약과 전달 상태](qa/timing-measurement-contract.md)를 따른다. 새 공통 v5는 이 변경을 반영하며, v4 숫자와 발표는 당시 계약으로 보존한다.

**네 DP의 숫자를 함께 보려면 [공통 QA v6](evaluation/common-qa-v6/README.md)의 Excel `전체비교` 또는 HTML을 연다.** v6는 QA07 사례, 응답 의무와 판정 원장을 추가했다. 24개 검토용 묶음의 분모/반복/종료 정책은 미동결이고 실제 관측은 없다. 시간과 자원은 v5 원값이다. 같은24입력과7구성에 QA01 NORMAL36/WAIT6예정, QA03 전체24목표와 QA08 추가 모델 메모리를 적용했다. 조건부 분석값과 점수이며 실측이 아니다. v4/v3는 보존했고 QA02/QA07은 관측이 없어 미산출이다.

| 작업 | 현재 기준 |
| --- | --- |
| DP2 의미 해석 실행 | [설계와 최신 보정](dp02/README.md) |
| DP3 화면과 음성 지칭 | [설계와 CPU 실행 대안](dp03/README.md) |
| DP5 위임용 메모리 | [설계와 관계 생산 및 조회](dp05/README.md) |
| DP44 음성 실행과 전달 | [설계와 입력, 정정 및 재개](dp44/README.md) |
| QA 정의, 목표, 점수와 출처 수정 | [QA 작업 기준](qa/README.md) |
| 사례와 적용 모집단 확인 | [시나리오 안내](scenarios/README.md) |
| 공통 QA v6 수치, Excel과 HTML 및 v5/v4/v3 보존본 | [평가 자료](evaluation/README.md) |
| PPTX와 36쪽 미리보기 | [현재 발표 v10](presentation/README.md) |
| PPTX 제작 | [PPTX-GUIDE.md](presentation/PPTX-GUIDE.md) |
| 설계 그림 작성 | [그림 지침](guidelines/diagram-guide.md) |
| 작업 규칙과 변경 영향 | [AGENTS.md](AGENTS.md), [동기화 규칙](guidelines/change-and-sync.md) |
| 원본과 새 경로 및 보존 검증 | [이관 기록](history/migration.md), [판본 원장](package-manifest.json) |

## 지금의 판본과 반영 상태

| 자료 | 가져온 판본 | 의미와 현재 한계 |
| --- | --- | --- |
| 네 DP 설계 | 사용자 확인 파일과 v8/v9/v10의 상세 계약 및 그림 보정 | 현재 설계 설명. 최종 대안의 사용자 채택과 구별한다. |
| QA01/QA03 | [현재 QA](qa/README.md), 음성 첫 응답/전체 처리 완료 계약 | NORMAL/WAIT, 목표2초/5초와 점수 합의 완료. v5 새 경계에 반영, v4는 이전 계약 보존 |
| QA02/QA05~06 | 기존 지표, 목표와 0~6점 기준 | 단일 대표값/부록과 전체 제품 RAM 범위 명확화 |
| QA04 | QA04-CHANGE-v4 / Q4-CHANGES-v4 | 8개 구체 변경 과제와 실제 운영 모듈 기준. 기존 6과제 추정과 구별, 새 결과 미산출 |
| 확장 QA07 | [요청 해석 및 응답 적합성](qa/qa07-request-meaning-accuracy.md), `QA07-REQUEST-RESPONSE-v2` | 최초 해석과 후속 응답 구성/실제 전달의 사례별 전체 충족률. 목표/점수 미정, 상세 단위/종료와 우선순위 검토. |
| 새 QA08 | [온디바이스 모델 추가 메모리](qa/qa08-on-device-model-memory.md), `QA08-LOCAL-MEM-v1` | 사용자 지정 1 GB 미만 요구. v5에 자원 귀속 분석 추정 추가. 실제 계측과 점수는 미확정. |
| QA07 확장 평가표 | [COMMON-QA-MODEL-6.0](evaluation/common-qa-v6/README.md) | 같은 7구성에 해석/구성/실제 전달 의무와 사례 집계 적용. 24묶음은 검토안, 전체 충족률과 점수 미산출. |
| 보존 공통 계산 v5 | [COMMON-QA-MODEL-5.0](evaluation/common-qa-v5/README.md) | 24입력, 7구성. NORMAL/WAIT 음성 사건과 전체 목표 완료, 추가 모델 메모리 추정. QA04 설계 추정, QA02/07 관측 없음. 실측 아님. |
| 보존 공통 계산 | [COMMON-QA-MODEL-4.0](evaluation/common-qa-v4/README.md) | 이전 화면 QA01 계약과 당시 원값·산출물을 그대로 보존 |
| 보존 공통 평가 | COMMON-QA-SYN-3.0 / REVIEW-2 | 3개 profile, 9구성, 26개 사례군의 합성 평가. 서로 다른 profile의 수치를 합산하지 않는다. |
| 발표 | 36쪽 reference-style v10 | 확인된 PPTX와 미리보기를 그대로 보존했다. 최신 공통 QA v3 전체를 반영한 발표본은 아니다. |

**남아 있는 내용 동기화:** 발표 v10의 기존 PoC, 합성 예시와 미평가 표시는 유지했다. 공통 QA v3를 발표에 어떤 범위로 반영할지는 별도 내용 개정으로 처리한다. QA 목표나 점수가 바뀌어도 기존 원시 결과와 발표 숫자를 자동으로 새 정의의 결과로 바꾸지 않는다. DP44의 과거 음성 QA01과 v4의 화면 QA01은 당시 계약이며 새 QA01-VOICE 결과로 재명명하지 않는다.

v4/v5/v6는 발표 v10에 미반영이다. 최신 전체 비교를 발표할 때는 네 DP의 수치·모집단·근거 표기를 함께 개정해야 한다. 이번 변경은 설계와 그림, 기존 점수표를 바꾸지 않았다.

## 폴더 책임

**2026-10-22 QA08 신설:** [Galaxy Book 기본 탑재 제약](qa/targets-source-qa08-platform.md)을 근거로 추가 로컬 모델의 메모리 최고량을 평가한다. QA06의 전체 RAM 범위와 기존 적용 상태는 유지한다. 새 QA08은 v5에서 자원 귀속을 별도로 계산했다. v3/v4와 발표 v10은 보존하며 현재 추정은 제품 적합성 판정이 아니다.

**2026-10-23 QA07 범위 확장:** [현재 명세](qa/qa07-request-meaning-accuracy.md)는 요청 해석과 후속 응답 구성/실제 전달의 적합성을 통합한다. 별도 QA는 추가하지 않았다. 기존 90% 목표/점수와 입력 단위 정의는 [보존본](qa/history/qa07-before-response-scope-20261023/README.md)으로 구별한다. 현재 설계/평가 자료의 이전 QA07 참조를 새 정의의 효과나 결과로 읽지 않는다. 이번 작업은 DP44 설계 변경, 모델 실행이나 재채점을 포함하지 않는다.

설계는 `dp02/`, `dp03/`, `dp05/`, `dp44/`에, 후속 공통 그림 보정은 `design-review/`에 있다. QA의 정의와 목표 및 점수는 `qa/`, 사례와 모집단은 `scenarios/`, 판본이 고정된 평가는 `evaluation/`, 발표와 제작 안내는 `presentation/`에서 관리한다. `history/`와 각 하위 이력은 현재 설계나 수치의 대체 정본이 아니다.

상위 [INTENT.md](../../../../../INTENT.md)와 [저장소 AGENTS.md](../../../../../AGENTS.md)는 저장소 전체에 적용하는 정본으로 유지한다. 이 패키지의 [AGENTS.md](AGENTS.md)는 두 문서를 읽은 뒤 이 범위에서 작업할 순서를 정한다. Reference Architecture, accepted ADR, 다른 DP와 실측 원시는 기존 위치와 상태를 유지한다.

## 완료 전 확인

저장소 루트에서 다음 검사를 수행한다. 모두 정적 문서 및 보존 검사이며 모델 실행이나 새 성능 측정이 아니다.

```sh
.venv/bin/python scripts/architecture/check_current_package.py
.venv/bin/python scripts/architecture/check_common_qa_v4.py
.venv/bin/python scripts/architecture/check_common_qa_v5.py
.venv/bin/python scripts/architecture/check_common_qa_v6.py
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
git diff --check
```

발표 내용을 바꿀 때는 [제작 지침](presentation/PPTX-GUIDE.md)에 따라 새 PPTX를 검증하고 렌더를 확인한다. 원본과 같은 해시를 가진 이번 이관은 기존 검증의 대상을 그대로 보존한 것이며 새 Windows 실행 결과를 주장하지 않는다.
