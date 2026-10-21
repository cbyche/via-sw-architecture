# DP2, DP3, DP5 공통 QA 층별 산출물

`COMMON-QA-SYN-2.0` / **SYNTHETIC_AUTHORED_NOT_MEASURED**

[통합 문서](../../architecture/12-decisions/decision-packages/qa-common-synthetic-v2-20261021/README.md)는 같은 QA 의미와 서로 다른 Text/Voice profile의 비교 및 판단을 설명한다. 원자료를 새 실험 결과로 바꾸거나 두 층의 절대값을 합산하지 않는다.

| 파일 | 내용 |
| --- | --- |
| [Excel 계산표](outputs/VIA-QA-stratified-v2.xlsx) | 9개 시트. 두 층의 비교, 최초 채택, 목표, 비용과 RAM 계산 |
| [HTML 사례 탐색기](outputs/guide.html) | 층과 구성, 입력과 오류 필터, 정답과 최초 의미, 처리 단계 |
| [normalized-data.json](normalized-data.json) | 7구성의 공통 시나리오, 응답, 지칭, 목표, DAG, 비용, RAM과 변경 원장 |
| [summary.json](summary.json) / [comparison.csv](comparison.csv) | 같은 profile 안의 49개 QA 값과 단일 축 차이 |
| [manifest.json](manifest.json) | 부모 snapshot, 생성기와 출력의 hash 및 판본 |
| [validation.json](validation.json) | 독립 재집계와 연결 검사 범위 |
| [Text 부모 사본](parents/text.json) / [Voice 부모 사본](parents/voice.json) | 변경하지 않는 정확한 원본 JSON 사본 |
| [수식 검증](workbook-validation.json) / [저장 캐시 확인](workbook-cache-check.json) / [표시 함수 검사](presentation-check.json) | 산출물별 확인 범위와 제한 |
| [최종 확인](publication-validation.json) | 부모 원본과 산출물 연결, 검토 범위 및 전체 링크 검사 제한 |

Text에는 240입력, 250표현, 180목표가 구성당 있고 Voice에는 120평가입력, 120표현, 120목표가 구성당 있다. Voice 준비 60활동은 비용과 RAM에 포함한다. Text의 BASE는 두 DP 표에서 같은 출처로 재사용하므로 한 번만 저장한다. 원장의 1,200응답행은 실제 실행 횟수가 아니다.

## 재생성

저장소 루트에서 `.venv/bin/python scripts/presentations/build_common_qa_stratified.py`를 실행한다. 표준 라이브러리만 사용하는 오프라인 자료 adapter다. 기존 DP3 생성기를 실행하거나 원본 폴더를 수정하지 않는다. 생성기 [소스](../../../scripts/presentations/build_common_qa_stratified.py)는 처음에 부모 사본이 없으면 현재 source를 복사하고, 이후에는 그 사본을 사용한다. 원본의 변경을 반영할 때에는 새 dataset과 출력 경로를 만들어야 한다.

현재 생성기는 QA01~07의 49개 부모 값을 유지하는지 확인한다. 시나리오에서 입력, 목표와 지칭 ID로 각 평가 행까지 연결되고, 정확도는 최초 채택, 시간은 인과 DAG, 비용은 사용량×단가, RAM은 같은 상태의 성분 합에서 다시 계산한다. 실제 모델 실행, 화면/음성 관측 및 RAM 계측은 수행하지 않는다.

Excel과 HTML은 [표시 생성기](../../../scripts/presentations/render_common_qa_stratified.mjs)로 만든다. 현재 세션의 bundled Node와 `@oai/artifact-tool`을 확인하고, 그 모듈을 찾을 수 있는 디렉터리를 `VIA_ARTIFACT_MODULE_BASE`로 지정한 뒤 실행한다. 원장의 근거를 보존한 표시 작업이며 과거 `/private/tmp` 경로가 계속 존재한다고 가정하지 않는다.

## 산출물 검증

Excel의 49개 QA 수식값과 42개 비용 분해값이 원장과 일치했다. 단가 변경 뒤 비용 재계산과 복원을 확인했고 저장된 xlsx의 49개 요약 캐시도 일치했다. 수식 오류는 0개다. 9개 시트의 12개 렌더 미리보기를 검토했다. 비용은 같은 구성/구간/역할/bucket/단가의 사용량만 합치고, RAM은 사례별 동일 시점 peak를 보존해 전역 최대를 다시 계산한다.

HTML은 7개 구성의 전체/오류 필터와 상세 표시 함수를 오프라인에서 확인했다. IAB 연결이 제공되지 않았고 Chrome의 로컬 파일 접근도 보안 정책으로 거부되어 브라우저 시각 검증은 수행하지 않았다. 우회하지 않았으며 Windows Excel의 실제 렌더도 확인하지 않았다. 이 한계를 계산 검증과 구분한다.
