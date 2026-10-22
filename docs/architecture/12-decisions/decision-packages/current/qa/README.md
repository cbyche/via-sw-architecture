# 공통 QA 정의, 목표와 점수의 현재 편집 기준

2026-10-22 이관 후 QA07 상세화 / QA01~06 유지 / QA07 목표와 점수 제안 추가, 우선순위 변경 없음

DP2, DP3, DP5와 DP44가 함께 참조하는 QA의 정의와 목표 도출, 점수표를 이 폴더에서 관리한다. 이전 `qa-candidate-package`의 내용은 보존하고 이관한 문서를 앞으로의 편집 기준으로 삼는다. 현재 동일 조건의 계산은 [공통 QA v4](../evaluation/common-qa-v4/README.md), 이전 수치는 [v3 보존본](../evaluation/common-qa-v3/README.md), 입력과 관측 모집단은 [시나리오 안내](../scenarios/README.md)에서 확인한다. 정의 문서에 남은 2026-10-15의 DP별 예상표와 검토 상태는 해당 세대의 이력이다.

| 현재 정의 문서 | 대표 지표 | 유지한 목표, 상태 | 도출 자료 |
| --- | --- | --- | --- |
| [QA01 첫 응답 지연시간](qa01-first-response-latency.md) | 첫 구체적 내용의 실제 화면 표시까지 평균 시간 | 1초 이하 목표 제안, 기존 0~6점 유지 | [지연 조사](research-notes/qa01-latency-research.md) |
| [QA02 지칭 대상 식별 정확도](qa02-referent-accuracy.md) | 표현/revision별 최초 채택 대상 집합의 완전 일치율 | 93% 이상 목표 제안, 기존 0~6점 유지 | [정확도 조사](research-notes/qa02-accuracy-research.md) |
| [QA03 VIA 처리시간](qa03-via-processing-time.md) | 외부 시간을 제거한 목표별 VIA 귀속 처리시간 평균 | 5초 이하 목표 제안, 기존 0~6점 유지 | [처리시간 조사](research-notes/qa03-duration-research.md) |
| [QA04 변경 범위](qa04-change-scope.md) | 변경 과제당 평균 실제 운영 Rust 모듈 수 | 3개 이하 목표 제안, 기존 0~6점 유지 | [ARGO 145건과 산식](targets-source-qa04-argo.md), [조사](research-notes/qa04-change-research.md) |
| [QA05 모델 사용 비용](qa05-model-cost.md) | COMMON-28 전체 묶음당 평균 모델 비용 | $2.55 이하 목표 제안, 기존 0~6점 유지 | [공통 견적](common-model-cost-quote.md), [조사](research-notes/qa05-cost-research.md) |
| [QA06 최대 RAM 사용량](qa06-local-memory.md) | 평가 범위의 VIA 전체 RAM 관측 최고량 | 사용자 지정 512 MB, RAM-v3-MB 점수 유지 | [시스템 요구 출처](targets-source-qa06-platform.md), [조사](research-notes/qa06-memory-research.md) |
| [QA07 요청 의미 해석 정확도](qa07-request-meaning-accuracy.md) | 입력/revision별 최초 확정 필수 의미의 전체 일치율 | **90% 이상과 0~6점 제안**, 채택과 우선순위 미합의 | [의미 해석 조사](research-notes/qa07-meaning-research.md), [입력과 평가 계획](qa07-evaluation-plan.md) |

선택 우선순위는 기존 **QA01 → QA02 → QA03 → QA04 → QA05 → QA06**을 유지한다. 새 QA07을 임의로 끼워 넣거나 QA02를 대체하지 않는다. 가중 총점으로 합산하지 않으며 원값, 분모, 단위, 실패와 근거 종류를 함께 남긴다.

## 보조 후보와 구판 QA07

| 자료 | 현재 역할 |
| --- | --- |
| [QA08 병행 수정 중복](qa08-parallel-change-overlap.md) | 독립 변경 쌍의 평균 겹치는 모듈 수. 기존 예비 후보와 점수 보존 |
| [QA09 음성 전달 지연](qa09-audio-delivery-lag.md) | 응답 채택 후 청취 가능한 시간에서 음성 시작까지의 필수 회귀 진단. 기존 목표, 점수 보존 |
| [구판 QA07 부하 추가 지연](legacy/qa07-load-interference.md) | 다른 세대의 보조 지표. 새 QA07의 정의, 점수, 우선순위로 계승하지 않음 |

현재 공통 QA01은 **화면 표시**다. v3 DP44의 **첫 음성 지연**은 입력 종료부터 장치 출력까지의 별도 진단이다. QA09는 채택 후 청취 가능 시간이라는 다른 시작점과 시간 규칙을 갖는다. 세 값을 같은 지표로 부르거나 평균하지 않는다.

## 정의, 적용 계약과 증거의 역할

현재 동일 사례 비교는 [v4 공통 계약](../evaluation/common-qa-v4/01-common-contract.md)을 따른다. QA 의미는 유지하고 24개 입력에 같은 조건을 적용하며 QA02/QA07은 관측 출력이 없어 미산출이다. QA04는 설계 추정이고 기존 목표·점수나 새 QA07 평가 제안을 자동 적용하지 않는다. 아래 v3는 보존 판본의 역할이다.

1. 이 폴더의 명세는 QA의 의미와 원래 목표, 점수, 출처를 관리한다.
2. [v3 공통 계약](../evaluation/common-qa-v3/01-common-contract.md)은 같은 의미를 Text, Voice/Screen, Delivery의 입력, 목표, 시간, 비용, 메모리 가정에 적용한다. 서로 다른 profile의 값은 합산하거나 DP 간 절대 순위로 읽지 않는다.
3. [v3 원장과 산출물](../evaluation/common-qa-v3/artifacts/README.md)은 모두 `SYNTHETIC_AUTHORED_NOT_MEASURED`다. QA04는 예상 `concept::` 모듈 수이고 실제 Rust diff가 아니다. 비용과 RAM도 작성한 가정의 값이다.
4. COMMON-28의 목표와 점수는 그 단위와 모집단을 가진다. v3의 서로 다른 활동 묶음에 $2.55와 기존 점수표를 자동 적용하지 않는다. 신규 목표, 점수, 모집단 또는 새 실측은 별도 판본과 근거가 필요하다.
5. DP44의 QA02, QA07 100%는 동일 정답을 공급한 제어 fixture다. 정확도 향상이나 차별화 coverage를 주장하지 않는다.

## 목표 근거와 발표에 사용할 자료

- [목표 도출 원칙](target-rationale.md), [조사 상세 목록](research-notes/README.md), [검토와 정정 이력](review-and-revision-log.md).
- [품질 요구사항과 발표용 표](quality-requirements-table.md), [QA 한 장 설명 원고](slide-page-plan.md). QA01~06 및 보조 QA08/09를 편집할 때 정의, 점수 원문과 함께 갱신한다. 이 두 자료의 QA07 행과 페이지는 **구판 부하 추가 지연**으로 표시되어 있으며 새 QA07의 완성된 발표 페이지로 사용하지 않는다.
- [COMMON-28 시나리오 원전](../scenarios/common-28-v1/common-scenarios.md), [JSON](../scenarios/common-28-v1/common-scenarios.json)은 목표와 기존 회귀 명세의 출처다.
- [이전 72개 사고 실험](history/qa-dp-thought-experiment.md), [이전 패키지 개요](history/source-package-overview-20261015.md)는 기존 DP44/47/45/42 세대의 이력이다. 현재 네 DP의 수치와 결론으로 재사용하지 않는다.

## 편집과 검증

새 QA 편집은 이 폴더에서 수행하고 정의, 목표, 점수의 변경 이유와 버전을 기록한다. 이관 전 원본과 과거 동결 corpus는 수정하지 않는다. 이관 작업에서는 숫자를 유지했으며 후속 사용자 요청에 따라 QA07의 90% 목표, 점수와 세부 평가 절차를 `QA07-REQ-v1` 제안으로 추가했다. 실제 측정과 기존 결과의 재채점은 수행하지 않았다.

[이관 원장](../qa-migration.json)은 원래 경로, 현재 경로와 원본 SHA256을 기록한다. 원래 문서에 적힌 `check_qa_candidate_package.py` 및 `qa_candidate_population.py`의 명령은 **기존 패키지와 원전 생성 규칙의 검증/재생성 명령**이다. 현재 파일을 갱신하는 명령으로 실행하지 않는다. 현재 패키지의 이관, 링크, 점수 일관성 검사는 `python3 scripts/architecture/check_current_package.py`로 수행하며 제품, 모델, 장치 측정은 아니다. 시나리오나 출력의 후속 편집은 [평가 보존과 편집 규칙](../evaluation/common-qa-v3/migration-and-editing.md)을 따른다.
