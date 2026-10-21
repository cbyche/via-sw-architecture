# DP2 개별 가상 원장 보존본

2026-10-21에 작성한 DP2의 200입력, 14시트 합성 자료를 공통판의 선행 근거로 보존한다. 현재 DP 간 비교와 후속 확장은 [DP2와 DP5 공통판](../../architecture/12-decisions/decision-packages/qa-common-synthetic-20261021/README.md)을 사용한다. 개별판과 공통판은 모집단과 계산 가정이 달라 값을 직접 합산하거나 개선율로 비교하지 않는다.

- [개별 설명서](outputs/01a154e2/DP2-backdata-guide.html), [Excel](outputs/01a154e2/DP2-synthetic-backdata.xlsx), [원본 JSON](synthetic-data.json).
- [생성 시 검증](validation.json), [저장 workbook 검증](saved-workbook-check.json). `previews/`의 18개 이미지는 14개 시트의 검토 영역이다.
- [당시 생성기](../../../scripts/presentations/build_dp2_synthetic_backdata.mjs), [합성 일관성 검사](../../../scripts/presentations/validate_dp2_synthetic_backdata.py).

이 자료의 수치와 사례는 작성 가정이다. 실제 후보나 모델 시험 결과가 아니다. 원본 값, workbook과 작성 스크립트는 재평가하거나 공통판 값으로 고치지 않았다. 생성기는 당시 사용자 로컬 경로와 `@oai/artifact-tool` 환경에 의존하며 새 작업의 재생성 기반은 공통판을 따른다. 중복 내용의 큰 `*.xlsx.inspect.ndjson`은 로컬 도구 출력으로 Git에서 제외한다.
