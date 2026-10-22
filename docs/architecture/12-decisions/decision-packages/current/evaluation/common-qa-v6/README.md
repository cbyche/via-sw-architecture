# 공통 QA v6

2026-10-23 / `COMMON-QA-MODEL-6.0` / QA07 확장 평가표 / 실제 실행 없음

QA07을 **요청 해석과 응답 구성 및 실제 전달의 사례별 전체 충족률**로 갱신했다. 이전 의미 정답만으로 성공률을 계산하지 않는다. QA01~06과 QA08의 분석 원값은 [v5](../common-qa-v5/README.md) 그대로이며 새 실측이나 대안 선정은 없다.

| 보고 싶은 것 | 열 자료 |
| --- | --- |
| 모든 DP와 방안의 QA | [Excel](artifacts/outputs/01a15dfe-v6/VIA-QA-common-v6.xlsx)의 `전체비교` |
| QA07 요약과 사례별 판정 | 같은 Excel의 `QA07사례` |
| 입력별 해석/응답/전달 의무와 관측 입력란 | 같은 Excel의 `QA07의무`, [HTML](artifacts/outputs/01a15dfe-v6/guide.html) |
| 무엇을 맞혀야 하는가 | [사례 매핑](02-qa07-case-mapping.md), [JSON 명세](qa07-case-manifest.json) |
| 판정과 집계 규칙 | [공통 계약](01-qa07-contract.md), [검토 및 검증](03-review-and-validation.md) |
| 계산 원장 | [통합 JSON](artifacts/ledger.json), [QA07 판정 JSON](artifacts/qa07-results.json) |

24개는 기존 U01~U24를 연결한 **검토용 사례 묶음**이다. 확정된 QA07 분모가 아니다. 반복 수, 기능 기한, 후속 관측창, 경합 정책과 일부 선행 요청 원문/최초 해석이 미정 또는 없다. 7구성의 168행은 작성 슬롯이며 실제 실행 수는 0이다. 모두 UNKNOWN이고 전체 충족률, 범위와 점수는 미산출이다.

Excel은 의무별 최종 판정, 증거와 평가자 참조를 입력하면 사례 상태를 집계한다. FAIL 증거가 하나라도 있으면 다른 관측이 없어도 실패를 남긴다. PASS는 모든 의무와 종료 증거 및 사전 계약이 완비돼야 한다. 처음 실패한 뒤 복구한 경우 최종 판정은 FAIL로 유지하고 원시 이력은 별도로 보존한다. 자유문장 의미를 Excel이 자동으로 심사하는 도구는 아니다.

구형 `의미판정` 시트는 최초 해석의 부분 진단으로 보존한다. 새 QA07 총계는 그 시트를 참조하지 않는다. QA02와 같은 대상 오류는 `원인/QA02 공유 ID`로 연결한다. 화면 표시와 실제 가청 출력은 준비 수용과 구별한다.

[v5](../common-qa-v5/README.md), v4/v3/v2와 기존 PPTX를 보존했다. 발표에는 v6가 미반영이다. 명세와 자료의 정합성 검증은 모델/제품 성능 검증과 다르다.

```sh
python3 scripts/architecture/build_common_qa_v6.py
python3 scripts/architecture/check_common_qa_v6.py
```

Excel/HTML은 `scripts/presentations/render_common_qa_v6.mjs`로 기존 v5 workbook을 불러와 QA07 관련 영역만 확장한다. Artifact Tool 런타임을 사용하며 원본 workbook은 덮어쓰지 않는다.
