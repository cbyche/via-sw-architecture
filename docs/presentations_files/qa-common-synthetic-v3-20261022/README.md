# DP2, DP3, DP5, DP44 공통 QA 층별 산출물

`COMMON-QA-SYN-3.0` / **SYNTHETIC_AUTHORED_NOT_MEASURED**

[통합 문서](../../architecture/12-decisions/decision-packages/qa-common-synthetic-v3-20261022/README.md)는 같은 QA 의미를 Text 이력, Voice/Screen과 DELIVERY의 세 profile로 비교한다. 9구성의 공통 QA01~07 대표값 63개와 DP44 첫 음성 진단 2개를 분리한다. 다른 층의 값이나 개별 DP 이익을 합산해 결합 제품 성능으로 제시하지 않는다.

| 파일 | 내용 |
| --- | --- |
| [Excel 계산표](outputs/01a15dfe/VIA-QA-stratified-v3.xlsx) | 층별 QA, 화면과 별도 음성, 최초 의미, 목표, 비용과 RAM |
| [HTML 사례 탐색기](outputs/01a15dfe/guide.html) | 층과 구성, 입력 및 오류/fixture, 처리 단계와 원본 연결 |
| [normalized-data.json](normalized-data.json) | 9구성의 시나리오, 응답, 지칭, 목표, DAG, 비용, RAM과 변경 원장 |
| [summary.json](summary.json) / [comparison.csv](comparison.csv) | 같은 profile의 대표값과 비교쌍, 미모델링 비용 |
| [manifest.json](manifest.json) | 부모 사본, 생성기 및 산출물의 판본과 SHA256 |
| [validation.json](validation.json) | 원장 연결과 재집계 검증의 범위 및 상태 |
| [수식 검증](workbook-validation.json) / [산출물 검증](artifact-validation.json) | Excel 계산, 저장된 값과 시트 렌더 확인 |
| [HTML 검사](html-validation.json) / [교차 대조](cross-artifact-validation.json) | 오프라인 동작, 원장과 XLSX/HTML 연결 |
| [재읽기 검증](data-readback-validation.json) / [발행 검사](publication-validation.json) | 고정본 재생성, 링크와 변경 범위 및 파일 해시 |

TEXT는 구성당 240입력, 250표현, 180목표다. VOICE는 120평가 입력, 120표현, 120목표와 별도 준비 60활동이다. DELIVERY는 80평가 입력, 120표현, 90목표와 별도 준비 80활동이다. 원래 보고서 목표 10개는 시작 -6초를 보존한다. 준비 활동은 비용과 메모리에 포함하지만 평가 입력 분모에 더하지 않는다. 구성별 작성 행은 실제 모델 실행 횟수가 아니다.

DELIVERY의 QA02와 QA07 100%는 동일한 올바른 DP2 결과를 공급하는 고정 fixture다. 정확도 우위나 모델 예측으로 읽지 않는다. 첫 화면은 공통 QA01이고 첫 음성은 별도 진단이며, Agent source 도착부터의 전달 진단도 사용자 입력 평균에 섞지 않는다. S08의 물리 첫 음성 이익 50ms와 외부 대기를 제거한 QA03 차이는 별개다.

DELIVERY의 비용 비교는 VIA 모델 사용 비용이다. 수신 Agent 내부 비용과 시스템 전체 비용은 **미모델링**, JSON에서는 `null`이다. Excel의 빈칸이나 표시 문구를 0으로 읽거나 수식으로 0에 합산하지 않는다. QA04는 concept 모듈 경계의 예상값이며 공통 범위 인터페이스가 충분하면 A도 B와 같은 2.000이 되는 민감도를 유지한다.

## 보존과 재생성

v1, v2와 DP44 원본은 보존하고 새 v3 출력 경로를 사용한다. [오프라인 원장 생성기](../../../scripts/presentations/build_common_qa_delivery_v3.py)는 처음 만든 부모 사본을 이후 재실행에서 다시 읽는다. 새로운 원본 판본을 반영하려면 새 dataset과 출력 경로를 만든다. 부모 생성기를 다시 실행해 기존 값을 덮어쓰지 않는다.

저장소 루트에서 `python3 scripts/presentations/build_common_qa_delivery_v3.py`로 원장을 만든다. [표시 생성기](../../../scripts/presentations/render_common_qa_delivery_v3.mjs)는 같은 원장으로 Excel과 HTML을 만든다. 현재 세션의 bundled Node 및 `@oai/artifact-tool` 경로를 확인하여 사용하며 과거 임시 runtime 경로가 남아 있다고 가정하지 않는다. `VIA_ARTIFACT_MODULE_BASE`에는 해당 `node_modules`를 포함한 작업 디렉터리를 지정한다. 시간 DAG와 메모리 상태 수명은 원장 생성기가 소유하므로 workbook의 임의 셀 편집이 전체 실행 시나리오를 다시 만드는 것은 아니다.

저장된 결과의 대조는 `python3 scripts/presentations/check_common_qa_delivery_v3.py`로 수행한다. 이 [독립 검사기](../../../scripts/presentations/check_common_qa_delivery_v3.py)는 생성기 함수를 가져오지 않고 부모 Git 객체, 직렬화 원장, XLSX XML 캐시와 HTML 내장 데이터를 읽는다. [HTML 검사기](../../../scripts/presentations/check_common_qa_delivery_v3_html.mjs)는 Node로 실행하며 외부 패키지나 브라우저 없이 필터와 이벤트 핸들러를 확인한다. 이 세션의 Node 실행 경로는 `/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node`였으며 다른 환경에서는 해당 환경의 Node를 사용한다.

Hash와 수식 일치는 출처 및 계산 연결에 대한 근거다. 작성한 시간, 비용과 오류 빈도의 현실성, 실제 화면/음성 출력, 장치 RAM과 모델 capability를 검증하지 않는다. 이번 작업의 모든 수치는 이전 실제 모델 및 제품 실행 자료와 분리한다.

## v3 검증 상태

**현재 상태: PASS_WITH_RECORDED_LIMITS.** 아래는 최종 v3 원장과 산출물을 대상으로 수행한 검사다. 부모 v2와 DP44의 검사 통과를 v3 통과로 계승하지 않았다.

| 검사 | 상태 | 구별할 범위 |
| --- | --- | --- |
| 63개 QA 및 별도 음성 2개 계산 | PASS | 별도 검사기의 원장 재집계와 기존 49개 값 보존 |
| Excel 비용 재계산과 저장값 | PASS | 세 층 단가 변경 및 원복, 숫자 비용 50개와 미모델링 빈칸 4개 |
| Excel 수식 오류와 렌더 | PASS | 6,756수식 오류/누락 캐시 0, 9시트의 PNG 15개 확인 |
| Excel 의미 순서 | PASS | DELIVERY 160행 순서 보존. S07 순서 변경 시 QA07 1→79/80→1 |
| Native Windows Excel | NOT_RUN | 생성 엔진의 렌더와 별개 |
| HTML 필터 및 상세 표시 | PASS | 오프라인 최소 DOM 하네스, 9구성의 필터/검색/오류/선택과 source 및 부모 ID |
| HTML 브라우저 시각 확인 | NOT_COMPLETED | in-app browser file URL 보안 정책 차단. 우회하지 않음 |
| 독립 검토와 변경 범위 검사 | PASS | 발견한 P2 4건 수정 후 미해결 P1/P2 없음. 활성 링크와 용어 검사 |
| 실제 모델 및 제품 실행 | NOT_RUN | 가상 원장 생성은 실제 실행이 아님 |

부모 DP44의 HTML 브라우저 검증은 로컬 listener 권한과 file URL 보안 정책으로 미완료였고 native Windows Excel도 미실행이었다. 부모의 전체 링크 PASS에는 다른 세션의 미커밋 검사기 변경이 포함됐으므로 현재 v3 검사 결과와 구별한다. 세부 이력과 이후 통합 규칙은 [이관 및 검토 기록](../../architecture/12-decisions/decision-packages/qa-common-synthetic-v3-20261022/04-migration-and-review.md)에 둔다.
