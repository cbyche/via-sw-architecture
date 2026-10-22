# DP2, DP3, DP5, DP44 현재 작업 패키지

2026-10-22. 사용자가 확인한 설계, 발표 v10, 공통 QA v3와 QA 목표 및 점수의 원전을 한곳에 모은 **현재 편집 기준**이다. 앞으로 이 폴더에서 네 DP와 관련 QA를 수정한다. 이전 위치는 출처와 과거 평가의 재현을 위해 보존한다. 이관으로 설계 대안, QA 정의와 목표, 점수, 평가 수치 또는 사용자 채택 상태를 바꾸지 않았다.

## 바로 열기

**2026-10-23 QA 정의 합의:** QA01은 NORMAL 첫 음성 평균 2초와 별점, WAIT의 별도 보고로 구분했다. QA03은 전체 목표의 VIA 처리 완료 평균 5초다. 두 목표와 점수 구간을 확정했다. QA09는 제거했다. 사용자가 담당 세션의 평가 전체 갱신을 요청했다. [측정 계약과 전달 상태](qa/timing-measurement-contract.md)를 따른다. 아래 v4 숫자와 발표는 이 변경을 반영하지 않는다.

**네 DP의 숫자를 함께 보려면 [공통 QA v4](evaluation/common-qa-v4/README.md)의 Excel `전체비교` 또는 HTML을 연다.** 같은 24개 입력과 공통 환경에서 한 DP씩 바꾼 분석 모델이다. v3는 보존했으며 실제 관측 없는 QA02/QA07은 미산출이다.

| 작업 | 현재 기준 |
| --- | --- |
| DP2 의미 해석 실행 | [설계와 최신 보정](dp02/README.md) |
| DP3 화면과 음성 지칭 | [설계와 CPU 실행 대안](dp03/README.md) |
| DP5 위임용 메모리 | [설계와 관계 생산 및 조회](dp05/README.md) |
| DP44 음성 실행과 전달 | [설계와 입력, 정정 및 재개](dp44/README.md) |
| QA 정의, 목표, 점수와 출처 수정 | [QA 작업 기준](qa/README.md) |
| 사례와 적용 모집단 확인 | [시나리오 안내](scenarios/README.md) |
| 공통 QA v4 수치, Excel과 HTML 및 v3 보존본 | [평가 자료](evaluation/README.md) |
| PPTX와 36쪽 미리보기 | [현재 발표 v10](presentation/README.md) |
| PPTX 제작 | [PPTX-GUIDE.md](presentation/PPTX-GUIDE.md) |
| 설계 그림 작성 | [그림 지침](guidelines/diagram-guide.md) |
| 작업 규칙과 변경 영향 | [AGENTS.md](AGENTS.md), [동기화 규칙](guidelines/change-and-sync.md) |
| 원본과 새 경로 및 보존 검증 | [이관 기록](history/migration.md), [판본 원장](package-manifest.json) |

## 지금의 판본과 반영 상태

| 자료 | 가져온 판본 | 의미와 현재 한계 |
| --- | --- | --- |
| 네 DP 설계 | 사용자 확인 파일과 v8/v9/v10의 상세 계약 및 그림 보정 | 현재 설계 설명. 최종 대안의 사용자 채택과 구별한다. |
| QA01/QA03 | [현재 QA](qa/README.md), 음성 첫 응답/전체 처리 완료 계약 | NORMAL/WAIT 분리, 목표 2초/5초와 점수 합의 완료. 담당 세션 평가 갱신 대상, v4 미반영 |
| QA02/QA04~06 | 기존 정의, 목표와 0~6점 기준 | 이번 시간 지표 개정으로 변경하지 않음 |
| 새 QA07 | [요청 의미 해석 정확도 상세안](qa/qa07-request-meaning-accuracy.md), `QA07-REQ-v1` | 90% 목표와 0~6점, 측정 방법과 근거를 제안했다. 채택과 우선순위는 미합의. |
| 새 QA08 | [온디바이스 모델 추가 메모리](qa/qa08-on-device-model-memory.md), `QA08-LOCAL-MEM-v1` | 사용자 지정 1 GB 미만 요구. DP 공통 측정 조건과 0~6점은 제안, 미측정. |
| 현재 공통 계산 | [COMMON-QA-MODEL-4.0](evaluation/common-qa-v4/README.md) | 같은 24입력과 환경, 7구성. 시간·비용·RAM 조건부 계산, QA04 설계 추정, QA02/07 관측 없음. 실측 아님. |
| 보존 공통 평가 | COMMON-QA-SYN-3.0 / REVIEW-2 | 3개 profile, 9구성, 26개 사례군의 합성 평가. 서로 다른 profile의 수치를 합산하지 않는다. |
| 발표 | 36쪽 reference-style v10 | 확인된 PPTX와 미리보기를 그대로 보존했다. 최신 공통 QA v3 전체를 반영한 발표본은 아니다. |

**남아 있는 내용 동기화:** 발표 v10의 기존 PoC, 합성 예시와 미평가 표시는 유지했다. 공통 QA v3를 발표에 어떤 범위로 반영할지는 별도 내용 개정으로 처리한다. QA 목표나 점수가 바뀌어도 기존 원시 결과와 발표 숫자를 자동으로 새 정의의 결과로 바꾸지 않는다. DP44의 과거 음성 QA01과 v4의 화면 QA01은 당시 계약이며 새 QA01-VOICE 결과로 재명명하지 않는다.

v4도 발표 v10에는 미반영이다. v4 전체 비교를 발표할 때는 네 DP의 수치·모집단·근거 표기를 함께 개정해야 한다. 이번 변경은 설계와 그림, 기존 점수표를 바꾸지 않았다.

## 폴더 책임

**2026-10-22 QA08 신설:** [Galaxy Book 기본 탑재 제약](qa/targets-source-qa08-platform.md)을 근거로 추가 로컬 모델의 메모리 최고량을 평가한다. QA06의 전체 RAM 범위와 기존 적용 상태는 유지한다. 새 QA08은 v3/v4 계산과 발표 v10에 미반영이다. 현재 수치를 새 QA08로 재채점하지 않는다.

**2026-10-22 QA07 상세화:** [출처 조사](qa/research-notes/qa07-meaning-research.md)와 [80입력×5회 평가 계획](qa/qa07-evaluation-plan.md)을 추가했다. 새 목표와 점수는 검토용 제안이며 기존 평가를 재채점하지 않았다. 사용자 요청으로 발표자료와 발표용 표는 수정하지 않았으며 최신 QA07 발표 페이지는 별도 작성이 필요하다.

설계는 `dp02/`, `dp03/`, `dp05/`, `dp44/`에, 후속 공통 그림 보정은 `design-review/`에 있다. QA의 정의와 목표 및 점수는 `qa/`, 사례와 모집단은 `scenarios/`, 판본이 고정된 평가는 `evaluation/`, 발표와 제작 안내는 `presentation/`에서 관리한다. `history/`와 각 하위 이력은 현재 설계나 수치의 대체 정본이 아니다.

상위 [INTENT.md](../../../../../INTENT.md)와 [저장소 AGENTS.md](../../../../../AGENTS.md)는 저장소 전체에 적용하는 정본으로 유지한다. 이 패키지의 [AGENTS.md](AGENTS.md)는 두 문서를 읽은 뒤 이 범위에서 작업할 순서를 정한다. Reference Architecture, accepted ADR, 다른 DP와 실측 원시는 기존 위치와 상태를 유지한다.

## 완료 전 확인

저장소 루트에서 다음 검사를 수행한다. 모두 정적 문서 및 보존 검사이며 모델 실행이나 새 성능 측정이 아니다.

```sh
.venv/bin/python scripts/architecture/check_current_package.py
.venv/bin/python scripts/architecture/check_common_qa_v4.py
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
git diff --check
```

발표 내용을 바꿀 때는 [제작 지침](presentation/PPTX-GUIDE.md)에 따라 새 PPTX를 검증하고 렌더를 확인한다. 원본과 같은 해시를 가진 이번 이관은 기존 검증의 대상을 그대로 보존한 것이며 새 Windows 실행 결과를 주장하지 않는다.
