# 통합 v5의 최신 QA 비교

현재 발표와 수정의 기준은 [전체 통합 PPTX, 32장](../../VIA-architecture-presentation.pptx)이다. 이 폴더와 [DP v5, 14장](../../VIA-DP-background-and-comparison-41-45_v5.pptx)는 최초 통합 시점의 비교/QA 참고본이다. 네 DP는 전체 통합본의 21~32장에 포함됐다. 이후 발표 수정은 [부분 편집 안내](../../full-presentation/README.md)를 따른다. 아래 데이터/PNG/재생성 절차는 최초 이관 당시의 이력이며 비교 전용 v5 PPTX는 만들지 않는다.

[ASR-QA-v6](../../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 동일 P21 전체를 5회 수행하는 계약을 사용한다. 총 105개 시나리오, 정확성 630조건, 반응 135개, 요청 처리 95개다. 비용과 메모리는 초기화, 준비, 재사용, 갱신, 배경 생산, 폐기, drain과 idle을 포함한 전체 활동을 평가한다.

| QA | 목표 | A/B 원값과 등급의 현재 상태 |
| --- | --- | --- |
| 01 요청 처리 정확성 | ≥98%, 정답 618/630 이상. 전체/S/C ≥95%, 필수 위반 0건 | 새 P21 실제 출력이 없어 평가 전 |
| 02 상호작용 응답시간 | 평균 ≤1초, 모든 local stop ≤0.1초. 일반 ≥100/105 ≤2초, 집중 30/30 ≤5초 | 135개 endpoint/trace가 없어 전체 평균 평가 전 |
| 03 VIA 요청 처리시간 | 평균 ≤10초. 91/95 ≤15초, 미전달 0건 | 95개 causal DAG가 없어 전체 평균 평가 전 |
| 04 VIA 모델 사용 비용 | 평균 ≤$0.50/P21, 전체 청구 누락 없음 | 공통 예산 reference만 계산. 후보별 청구는 평가 전 |
| 05 VIA 변경 용이성 | 여섯 과제 평균 ≤3개, 과제별 ≤5개, 기능 회귀 충족 | 실제 변경 patch/책임 단위 증거가 없어 평가 전 |
| 06 로컬 VIA 메모리 사용량 | commit 동시 최대 ≤4,096MiB, working-set/idle 보호 조건 별도 | 자원 예산 가정만 계산. 실제 peak/목표 판정은 평가 전 |

미실측은 0점이 아니다. 이전 v4의 오류 2/8개, W100 비용, 일부 사례의 전체 시간 환산과 원형 점수는 현재본에서 제거했다. 이전 v4 파일과 JSON은 이력으로 보존한다. 최신 등급 경계는 `qa-review.json`과 QA 목표 근거 PPTX에 있으며 전체 원값과 보호 조건을 확인한 뒤 적용한다. 지역 계산이나 예산 가정에 후보 전체 등급을 부여하지 않는다.

## 계산한 최신 reference

음성은 `gpt-realtime-2025-08-28`, 의미는 Alibaba Cloud Model Studio Singapore/International의 `qwen3-30b-a3b-instruct-2507` non-thinking이다. 날짜별 PoC 비용 reference이며 제품 모델이나 DP 승자 선택이 아니다. 단가·usage 범위와 출처는 [공통 계산 원본](../../../architecture/12-decisions/decision-packages/03-02-poc-calculation-report.md)과 발표 노트를 따른다.

| P21 공통 예산 | USD/P21 |
| --- | --- |
| 의미 62호출, 입력 124,000/출력 12,400 token | 0.034720 |
| 신규 audio 입력 50초/출력 예산 179초 | 0.245120 |
| 음성 모델의 예산 Text/context 입력 10,000/출력 2,000 token | 0.072000 |
| 기본 소계 | **0.351840** |
| $0.50 목표까지 잔여 | **0.148160** |

신규 audio 길이와 예산 Text만의 소계다. 이력 audio 재입력, 별도 전사/vision/helper, 추가 생산, 재시도와 폐기는 실제 청구로 더한다. 62호출은 양안의 호출수를 강제하지 않는다. 지역 과거 처리 비용과 공통 호출이 겹치면 교체하고, 별도이면 추가하여 중복 계산하지 않는다.

| 지역 예시 | 가정 아래 계산 | 해석 한계 |
| --- | --- | --- |
| 41 S2-01, 충분한 typed 근거 | 기준 profile A 4.196923초 / B 2.398462초. 빠른/기준/느린 profile은 데이터에 보존 | A 2호출/B 1호출이라는 지역 가정. B 추가 해석이나 A cache로 차이가 줄며 전체 평균이 아님 |
| 44 S5-04, 전이 6회 | A 0.060초 / B 0.050초 | 실제 queue, credit, Join와 음성 차례를 포함한 전체 반응/처리시간이 아님 |
| 45 과거 근거 처리 | A 3miss $0.002880, 강한 cache 1miss $0.000960 / B 4생산 $0.006080 | 초기/새 근거/갱신을 포함한 지역 예산. 생산 빈도와 재사용에 따라 변하며 P21 총액이 아님 |
| 42 S6-04 정상 답변 한 번 | A 0.010초 / B 0.025초 | 지역 확정/receipt 가정. 모델과 외부 접수는 공통이며 전체 평균이 아님 |

동시 메모리 예산 예시는 A 1,792MiB와 B 41:1,760 / 44:1,840 / 45:1,920 / 42:2,080MiB를 쓴다. 실제 heap/copy/회수와 peak를 확인한 결과가 아니다. 구조 차이와 가정은 데이터와 발표 노트에 남긴다.

## 미리보기

| DP | 구조 비교와 QA 요약 | QA 상세 |
| --- | --- | --- |
| 41 | [PNG](./dp41-comparison_v5.png) | [PNG](./dp41-qa-detail_v5.png) |
| 44 | [PNG](./dp44-comparison_v5.png) | [PNG](./dp44-qa-detail_v5.png) |
| 45 | [PNG](./dp45-comparison_v5.png) | [PNG](./dp45-qa-detail_v5.png) |
| 42 | [PNG](./dp42-comparison_v5.png) | [PNG](./dp42-qa-detail_v5.png) |

## 재생성

번들 runtime을 사용하며 새 private build에 `node_modules`를 연결하고 [갱신기](../../../../scripts/presentations/refresh_dp_v5_qa.mjs)를 복사한다. `RUNTIME_NODE_MODULES`, `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`은 번들 절대 경로다.

```text
<bundled-node> <build>/refresh_dp_v5_qa.mjs <repo> <build>
```

[scene 생성기](../../../../scripts/presentations/build_dp_v5_qa_scenes.py)는 현행 QA 및 계산 원본의 SHA를 대조하고 비용을 독립 검산한다. [package 편집기](../../../../scripts/presentations/dp_v5_qa_package.py)는 QA 영역과 해당 노트만 바꾸고 비교의 다른 native 개체와 모든 비대상 part를 bytes로 보존한다. 현재 v5가 없으면 최초 43 제거용 생성기를 실행한 뒤 이 갱신기를 별도 새 build에서 실행한다.
