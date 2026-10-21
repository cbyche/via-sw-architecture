# DP3 CPU 네 대안 발표판

> 이 문서는 보존된 v6의 기록이다. 사용자 후속 요청으로 10~12쪽 그림을 재작성한 [v7](dp3-v7-review.md)이 현재 발표판이며 요청한 기존 경로는 v7로 갱신했다.

2026-10-21. 사용자가 지정한 v5-r2 PPTX의 DP3을 새로 구성하고 이전 DP3 다섯 장을 appendix로 보존했다.

- 현재 29장 버전: [VIA-DP1-DP5-reference-style-29slides-v6-dp3-cpu-windows.pptx](VIA-DP1-DP5-reference-style-29slides-v6-dp3-cpu-windows.pptx)
- 요청한 기존 경로도 같은 29장 내용으로 갱신했다. 그 파일명의 `21slides-v5-r2`는 이전 이름이다.
- 수정 전 21장 원본: [VIA-DP1-DP5-reference-style-21slides-v5-r2-before-dp3-cpu.pptx](VIA-DP1-DP5-reference-style-21slides-v5-r2-before-dp3-cpu.pptx). 기존 v5-r2 검토 기록과 hash, `previews-v5-r2/`는 이 수정 전 원본에 해당한다.

| 현행 페이지 | 내용 |
| --- | --- |
| 1~8 | 기존 DP1·DP2, 본문과 노트 보존 |
| 9 | DP3 두 시점 화면 지칭의 문제 |
| 10 | A와 B-R1 구조 비교 |
| 11 | B-R2와 B-R3 구조 비교 |
| 12 | CPU worker 실행 구조와 동시 메모리 |
| 13 | 동일 6유형의 첫 응답과 S3 임계 경로 |
| 14 | QA01~07 네 대안 비교 |
| 15 | 현재 가정하의 선택과 대안 선택 조건 |
| 16~23 | 기존 DP4·DP5, 본문과 노트 보존 |
| 24 | 이전 DP3와 현행 비교 범위를 구분하는 appendix 표지 |
| 25~29 | 원본 9~13쪽의 이전 DP3, 본문과 노트 보존 |

새 DP3의 근거는 [CPU 실행 구조](../../architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/cpu-runtime-architecture.md)와 [합성 비교 원장](../../architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/synthetic-comparison/README.md)이다. `DP3-GROUNDING-CPU-SYN-1.0`, `VOICE-SCREEN-CPU-SYN-1.0`, `SYNTHETIC_AUTHORED_NOT_MEASURED`를 따른다. 모델 실행·실측·실제 청구액이 아니다. 준비 활동 60개는 비용과 동시 RAM에 포함하며, 평가 입력 120개와 분모를 구분한다. 기존 Text 공통 corpus의 절대값과 합산하지 않는다.

QA01은 일반 접수 안내를 제외한 첫 구체적 채택 내용 표시이며 오답의 첫 표시도 유지한다. QA04는 가정한 Rust 변경 범위이며 실제 diff가 아니다. QA02와 QA07의 같은 대상 오류 개선을 두 개의 독립 이익으로 합산하지 않는다. R3의 worker 격리를 자동 병렬 속도 향상이나 QA04 개선으로 취급하지 않는다. 현재 CPU 가정과 QA01 우선순위에서 A, B를 택할 조건에서는 R2를 기본 검토 후보로 제시한다.

제작은 `scripts/presentations/build_dp3_cpu_v6.mjs`의 Artifact Tool native text·shape·table과 `scripts/presentations/package_dp3_cpu_v6.py`의 의존 관계 보존 패키징을 사용했다. 1920×1080, Calibri, ko-KR, 정수 pt 정책을 유지했다. 재생성 시 수정 전 원본을 독립 build 디렉터리의 `source-v5-r2.pptx`로 복사해 사용한다. 이미 갱신된 기존 경로를 원본으로 다시 넣지 않는다.

29장 모두 렌더링했다. 기존 21장은 페이지 번호를 제외한 본문 픽셀이 수정 전 preview와 동일하다. 원본 notes·slide relationships·master·layout·theme·media도 보존했다. 새 7장과 appendix 표지를 개별 시각 검토하고, 독립 검토에서 나온 R3 연결선 분기·합류를 수정했다. SAM 조건, warm 모델 범위, UI 모델 객체 메모리 표현을 명확히 했다. package integrity, 레이아웃, native table, 폰트, Artifact Tool 재import 검사가 통과했다. Windows PowerPoint 앱에서의 실제 열기 검증이나 모델 CPU 실행 검증은 수행하지 않았다.

검증 자료: [보존 기록](dp3-v6-preservation.json), [렌더 보존](dp3-v6-render-preservation.json), [파일 검사](dp3-v6-validation.json), [페이지 preview](previews-v6-dp3-cpu/).

사용자가 지정한 **DP2·DP5 QA 시나리오 정리 계획** 세션(`01a15d51-8178-7f13-a727-754050b93bee`)에는 최신 구조, 원장 경로, QA 정의·수치·준비 활동, 구버전과의 차이, 합성 근거 및 모집단 구분을 전달했다. 해당 세션의 공통 QA 확장은 별도 작업이며 이 PPTX의 DP2·DP5 본문을 수정하지 않는다.

이 작업에서 모델 실행·측정·commit·push는 수행하지 않았다.
