# 공통 QA v5

2026-10-23 / `COMMON-QA-MODEL-5.0` / **ANALYTICAL_SIMULATION_NOT_MEASURED**

**[Excel의 전체비교](artifacts/outputs/01a15dfe-v5/VIA-QA-common-v5.xlsx)에서 네 DP의 모든 방안을 함께 읽는다.** [HTML](artifacts/outputs/01a15dfe-v5/guide.html)에서는 입력을 골라 VIA/Agent 책임, 음성 사건과 계산 경로를 확인할 수 있다.

- QA01: 사전 지정 NORMAL 36사건 평균과 2초 기준 점수. WAIT 6예정 중 5출력·1정상철회를 별도 표시한다.
- QA03: 24목표의 필수 상태·외부 확인·전체 화면/음성 준비 수용까지 VIA 경로, 5초 기준 점수.
- QA08: 추가 로컬 모델 자원의 같은 시각 합계 최고량. 1,000 MB 미만 요구와 분석 추정·제품 미검증을 구분한다.
- QA02/07은 실제 최초 채택 출력이 없어 미산출이다. QA04는 개념 모듈 추정, 비용·시간·메모리는 공개 가정의 계산이다. 실제 관측 수는 0이다.

작업 중 QA07이 `QA07-REQUEST-RESPONSE-v2`로 확장됐다. 응답 구성과 실제 전달까지의 평가는 이 판본에 미반영이며 QA07은 미산출을 유지한다. Excel의 기존 의미 정답은 부분 진단이고 그것만으로 QA07을 계산하지 않는다.

| 확인할 내용 | 자료 |
| --- | --- |
| 같은 정의와 산식, 분모와 실패 처리 | [공통 계약](01-common-contract.md) |
| U01~U24의 VIA/Agent 역할과 음성 사건 | [입력과 책임](02-configurations-and-scenarios.md) |
| 모든 방안의 수치와 대기 종류별 결과 | [수치와 범위](03-results-and-ranges.md) |
| 독립 검토, 재계산과 확인 한계 | [검토와 검증](04-review-and-validation.md) |
| 가정과 정답 | [명세](specification.json), [평가자 정답](oracles.json) |
| 전체 처리·사건·비용·메모리 근거 | [원장](artifacts/ledger.json), [요약](artifacts/summary.json), [CSV](artifacts/comparison.csv) |

7개 구성에 같은 입력과 목표를 적용하고 한 DP만 바꾼다. A 네 행은 같은 BASE 증거의 재표시다. QA01/03의 분석 점수는 실제 성능 점수나 최종 선택이 아니다. QA08 점수는 부여하지 않았다. 원래 [v4](../common-qa-v4/README.md), v3, 기존 v2와 발표 v10은 보존했다. v5는 발표에 미반영이다.

## 재현

저장소 루트에서 다음을 실행한다. 모델/API/제품 호출은 없다.

```sh
python3 scripts/architecture/build_common_qa_v5.py
python3 scripts/architecture/check_common_qa_v5.py
```

Excel/HTML은 `scripts/presentations/render_common_qa_v5.mjs`로 번들 Artifact Tool 환경에서 생성한다. 시간·메모리 가정을 바꾸면 명세부터 원장을 재생성해야 한다. Excel은 원장의 사건 평균, DAG 투영, 비용·메모리 합계를 수식으로 재계산한다. Excel 자체가 자원 scheduler를 다시 실행하는 것은 아니다.
