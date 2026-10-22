# v3 이관과 검토 기록

2026-10-22 / `COMMON-QA-SYN-3.0` / **SYNTHETIC_AUTHORED_NOT_MEASURED**

## 1. 허가된 작업과 보존 범위

사용자는 DP44를 별도 DELIVERY 층으로 공통 QA v3에 통합하고, 독립 검토와 검증을 거쳐 해당 작업 변경만 commit/push하도록 승인했다. 이 승인은 가상 원장과 문서, Excel 및 HTML의 통합 작업이다. 실제 모델 실행, 장치 계측이나 제품 구현 결과를 만들었다는 뜻은 아니다.

[v2](../../../qa-common-synthetic-v2-20261021/README.md)의 Text 3구성과 Voice/Screen 4구성을 유지하고, [DP44 부모](../../../seven-problem-review/dp44-local-voice-execution-20261021/synthetic-comparison-20261022/README.md)의 A/B를 D-A/D-B로 추가한다. v1, v2, DP3와 DP44 원본 문서 및 원장은 보존한다. 다른 세션의 진행 중인 파일은 통합 준비 완료로 추정하거나 이 작업 변경에 포함하지 않는다.

| 항목 | 부모 상태 | v3 처리 |
| --- | --- | --- |
| TEXT | v2의 3구성, 12사례군, 21개 대표값 | 같은 원본과 분모를 보존하고 재집계 |
| VOICE | v2의 4구성, 6사례군, 28개 대표값 | 같은 profile과 제약을 유지하고 재집계 |
| DELIVERY | DP44의 2구성, 8사례군 | D-A/D-B와 별도 profile 추가 |
| QA01 | v2는 화면, DP44는 화면과 음성 병기 | 화면 QA01을 포함한 공통 QA01~07 총 63개 값, 음성 2개 값은 별도 진단 |
| QA02와 QA07 | DP44는 동일 정답 고정 fixture | 수치를 보존하되 정확도 차별화 coverage에서 제외 |
| QA03 | DP44는 90목표, 원래 보고서 시작 -6초 | 원래 목표와 새 전달 목표를 분리하고 DAG 의존 보존 |
| Source 전달 | DP44 `deliveries`의 별도 시작과 분모 | 사용자 입력 QA01과 합치지 않는 진단 원장 |
| 비용 | DP44 수신 Agent 내부 비용 미모델링 | receiver와 total은 `null`, 0으로 채우지 않음 |
| QA04 | A 2.333, B 2.000의 concept 모듈 가정 | 공통 범위 인터페이스에서 A도 2.000인 민감도 병기 |
| RAM | 각 부모의 동일 시점 소유 성분 | 층별 peak 보존, 다른 층이나 개별 peak의 합산 금지 |

## 2. 부모 사본과 새 판본 규칙

[manifest](artifacts/manifest.json)는 부모 사본, 원본 경로, 생성 소스와 파생물의 SHA256을 기록한다. Text와 Voice는 v2가 보존한 부모 세대를 이어받고, v2 요약과 DP44 원장을 함께 식별한다. 첫 생성 뒤에는 기존 부모 사본을 덮어쓰지 않고 그 사본으로만 재생성한다.

원본 세대가 바뀌면 새 dataset과 출력 경로를 만든다. v2와 DP44 파일을 갱신하여 v3 값에 맞추거나 이전 결과를 새 결과로 교체하지 않는다. Hash는 사용한 파일 세대와 계산 연결을 식별할 뿐 시간, 비용과 오류 빈도의 현실성 또는 모델 capability를 입증하지 않는다.

새 데이터는 `COMMON-QA-SYN-3.0`, 공통 계약은 `QA-COMMON-SYN-CONTRACT-v3-STRATIFIED`다. 같은 원본의 값을 다시 계산한 행은 새로운 독립 표본이 아니다. 부모의 오류, 미관측 상태와 한계를 보존하며 모든 근거 종류는 `SYNTHETIC_AUTHORED_NOT_MEASURED`로 유지한다.

## 3. 부모 자료에서 확인된 검증과 한계

아래는 저장된 부모 기록의 범위다. v3의 새 검증 완료를 대신하지 않는다.

| 부모 | 기록된 완료 범위 | 남은 한계 |
| --- | --- | --- |
| v2 | 49개 대표값, 시나리오 연결, DAG, 사용량과 동시 RAM 재집계. 검토 범위 내 미해결 P1/P2 없음. Excel 수식과 캐시 및 9개 시트 렌더 확인 | HTML 브라우저 시각 검증과 native Windows Excel 미완료. 실제 모델 및 제품 실행 없음. S3 시각 명세와 신규 목표 범위의 한계 |
| DP44 | 원장 재계산, 목표 시작점, 동일 의미 fixture, 구판 미재생, 80ms 중단, 겹치지 않는 재생. Excel 수식과 요율 변경 및 9개 시트 렌더 확인 | 별도 독립 검토 미수행. HTML 브라우저 시각 검증 미완료. Native Windows Excel 미실행. 모델, 마이크, 장치와 제품 실행 없음 |

v2의 전체 링크 기록은 보존 복사본의 기존 14건 누락으로 실패했다. DP44 부모의 [발행 기록](../../../../../../presentations_files/dp44-synthetic-backdata-20261022/publication-validation.json)은 926개 활성 Markdown의 전체 링크 PASS를 남겼지만, 당시 검사기에 다른 세션의 커밋되지 않은 변경이 있었고 전체 저장소도 동시 작업을 포함했다. 따라서 이 PASS만으로 v2의 14건이 이 작업에서 해결됐다거나 DP44만으로 전체 통과를 확보했다고 설명하지 않는다. v3에서는 실제 검사 상태와 변경 범위 검사 결과를 구분해 기록한다.

## 4. 최초 v3의 독립 검토와 검증 이력

**최초 v3 상태: PASS_WITH_RECORDED_LIMITS.** 데이터 작성자와 다른 검토자가 원장 및 생성기를 검토했고, 문서와 표시 생성기는 해당 작성과 분리한 검토자가 확인했다. 아래는 최초 v3 `8b2f555`의 검증 이력이며 이후 재검토 완료를 대신하지 않는다. 당시 JSON 보고서는 [validation-history](artifacts/validation-history/8b2f555/README.md)에 그대로 보존했다.

| 항목 | v3 상태 | 확인한 범위 |
| --- | --- | --- |
| 공통 계약과 분모 독립 검토 | PASS | 세 profile, 화면/음성/source 분리, 고정 정확도 fixture, 미모델링 비용 |
| DAG와 원장 독립 검토 | PASS | 1,200목표의 투영 재계산, 원래 목표 -6초, S08 wall/투영 차이, 생성 및 동시 상태 |
| QA04와 결론 독립 검토 | PASS | concept 모듈과 인터페이스 민감도, 기본안/tactic, 개별 권고와 결합 성능 구별 |
| 원장 산술 및 연결 검증 | PASS | 공통 63개 및 음성 2개, 기존 49개 유지, 80개 Delivery 사례의 280개 구성별 source 연결 |
| Excel 수식 및 저장값 | PASS | 공통 63개, 음성 2개, 비용 50개 캐시와 결측 4칸. 6,756수식 오류/누락 캐시 0. 세 층 요율 변경/원복 |
| Excel 의미 순서 검사 | PASS | DELIVERY 160행의 순서 보존. S07 순서만 바꿨을 때 QA07 1→79/80→1 확인 |
| Excel 표시 | PASS / native 미수행 | 생성 엔진의 9시트, PNG 15개 확인. Native Windows Excel 미실행 |
| HTML 함수 및 상세 표시 | PASS | 오프라인 최소 DOM 하네스로 9구성 필터, 검색, 오류 모드, 선택, 부모 노드와 구성별 source 연결 확인 |
| HTML 브라우저 시각 확인 | NOT_COMPLETED | in-app browser가 file URL을 보안 정책으로 거부. 다른 브라우저나 listener로 우회하지 않음 |
| 변경 범위와 저장소 검사 | PASS | 활성 Markdown 링크, 용어, QA catalog/candidate 및 diff 검사. 이전 자료와 무관한 변경은 발행 대상에서 제외 |

중복을 제거한 수정 사항은 아래와 같다. P2 4건은 모두 수정하고 재확인했으며 검토 범위에 미해결 P1/P2는 없다.

[문서와 산출물 독립 검토](reviews/01-independent-document-artifact-review.md), [코드와 원장 독립 검토](reviews/02-independent-code-data-review.md)에 검토자별 범위와 재확인 근거를 남겼다.

| 발견 | 영향 | 수정과 재확인 |
| --- | --- | --- |
| P2: Delivery 시나리오의 source ID namespace 누락 | 정규화 source 원장에 직접 연결할 수 없음 | `DELIVERY:` ID와 부모 ID를 분리하고 80사례의 140공유 참조를 양안 280source에 연결 |
| P2: HTML이 `parents` 대신 없는 `deps`를 읽음 | 처리 단계의 인과 의존이 빈칸으로 표시됨 | 부모 ID를 직렬화하고 표시. 원장 및 HTML 전체 단계 대조와 S03 상세 확인 |
| P2: source 상세의 구성 필터 누락 | 반대 안의 source가 함께 보임 | 사례와 구성으로 필터. D-A/D-B 각각 반대 구성 부재 확인 |
| P2: Excel의 의미 정규화가 순서 배열도 정렬 | 읽기 순서만 틀린 의미를 구별할 수 없음 | DELIVERY 의미 순서를 보존하고 순서 변경/재계산/원복 확인. QA02 집합 정규화는 유지 |
| P3: 계약/corpus ID 불일치와 “화면 63개” 표현 | 판본과 값 개수의 의미 혼동 | ID 통일, 화면 QA01을 포함한 총 63개 QA 값으로 명확화 |

원장 자체 검증과 별도 검사기의 교차 대조를 구분한다. 최초 검증 보고서는 위 보존 폴더에, 재검토 판본의 보고서는 산출물 루트에 둔다. 이전 검토 문서는 당시 확인 범위와 hash를 유지한다. 계산 검사, 독립 의미 검토, 표시 검사, 실제 모델 및 장치 성능은 각각 다른 근거다. 한 종류의 PASS로 다른 종류를 완료 처리하지 않는다.

## 5. 일관성과 수치 범위 재검토

사용자의 추가 전체 검토 요청에 따라 같은 dataset에 `COMMON-QA-SYN-3.0-REVIEW-2`를 기록했다. 원본과 원값은 바꾸지 않고 판정 규칙, 추적 metadata, 표시 및 검증을 보완했다. 기존 63개 공통 QA와 음성 2개 원값은 최초 v3와 정확히 같다. v2, DP44 부모와 기존 점수 및 목표는 보존한다.

| 보완 | 적용과 확인 |
| --- | --- |
| QA07 의미 비교 | 세 층의 집합/순서 규칙과 JSON scalar 범주 통일. 대소문자, missing/false/null/0, 순서 변경을 구별하고 미등록 배열 거부 |
| 표시와 확정의 경계 | 1,360행의 표시/확정 node와 시각 또는 구간 확인. T-EXEC 부분 표시 70행 보존, 전체 의미를 부분 표시 시점으로 소급하지 않음 |
| Excel 판정 | `EXACT` 비교, 순서와 대소문자 변경 후 요약 재계산 및 원복. 8,122수식과 65개 QA 및 비용 저장값 대조 |
| 범위와 분모 | 9구성의 입력/목표/과제 범위, 사례별 비용과 동시 RAM peak 및 분모를 독립 재집계 |
| 수치 해석 | 정확도 `%p`, USD 6자리, 시간/모듈 평균/RAM 3자리 통일. QA01 화면과 첫 음성 진단 구분 |
| RAM 요구 | 560 MB 기반과 512 MB 요구의 충돌 및 전 구성 초과/여유 표시. 두 안의 peak가 다른 사례와 시각임을 명시 |
| 산출물 | 범위검토를 포함한 Excel 10시트와 PNG 22개, HTML 범위/경계 상세를 재생성 |

코드 및 원장과 export 내용은 작성자와 다른 검토자가 확인했다. 문서는 데이터 작성자가 원장과 독립 대조했으며 이는 자신의 생성기에 대한 독립 검토와 구별한다. 현재 발견과 수정 및 검증 범위는 [재검토 기록](reviews/03-consistency-and-range-review.md)에 둔다.

현재 [원장 자체 검증](artifacts/validation.json), [재읽기](artifacts/data-readback-validation.json), [산출물 교차 대조](artifacts/cross-artifact-validation.json), [발행 전 검사](artifacts/publication-validation.json)를 따로 남긴다. Excel/HTML의 실제 확인 범위는 [산출물 안내](artifacts/README.md)에서 확인한다. 브라우저 시각 검증과 native Windows Excel은 여전히 미완료이며 실제 모델/제품 실행은 0회다.

## 6. 다음 DP 또는 결합안을 추가할 때

1. 새 DP의 최신 구조와 사용자 합의를 확인하고 실행, 상태 및 근거 생산에서 실제 바꾸는 축을 등록한다. 원본 파일이 있다는 사실만으로 준비 완료를 추정하지 않는다.
2. 현재 층과 원본, 정답, 자원 및 완료 조건이 같으면 그 층의 새 구성을 작성한다. 다르면 새 profile과 예정 분모를 먼저 기록한다. 미적용이나 미모델링 값은 0이나 동점으로 채우지 않는다.
3. 원본을 보존한 층별 통합인지, 하나의 제품 구성을 모든 workload에서 비교하는지 명시한다. 후자라면 `FIXED_UNMAPPED` 축을 실제 구조로 지정하고 기존 DP도 동일 새 조건에서 계산한다.
4. 같은 작업의 사용량, 공통 모델과 상태 메모리, 현재성 guard를 중복 계산하지 않는다. 개별 이익과 peak를 더해 결합 수치를 만들지 않으며 interaction은 필요한 네 교차 구성이 있을 때만 계산한다.
5. 의미 해석과 전달을 결합하면 입력, 최초 의미 채택, 기존 목표 revision, source 이용 가능 시점과 실제 전달 범위를 새 corpus에 연결한다. 현재 DP44의 고정 정답 fixture를 해석 성능 증거로 계승하지 않는다.
6. 새 판본의 문서, JSON, Excel과 HTML을 같은 원장에서 만들고 독립 검토 및 계산과 표시 검증을 수행한다. 새 실측을 시작한다면 별도 실행 dataset과 evidence label을 사용한다.

이 작업은 새 QA07의 우선순위와 점수, 기존 accepted ADR, reference Architecture 또는 이전 PoC 결과를 변경하지 않는다. 조건부 개별 권고와 사용자의 결합 제품 채택도 구별한다.
