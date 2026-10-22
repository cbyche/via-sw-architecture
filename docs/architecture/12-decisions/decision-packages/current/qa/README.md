# 공통 QA 정의, 목표와 점수의 현재 편집 기준

2026-10-23 QA01/QA03 정의, 목표와 점수 합의 / QA01 일반/대기 상황 분리 / QA09 독립 지표 제거 / 담당 세션 평가 갱신 요청 승인

DP2, DP3, DP5와 DP44가 함께 참조하는 QA의 정의와 목표 도출, 점수표를 이 폴더에서 관리한다. 이전 `qa-candidate-package`의 내용은 보존하고 이관한 문서를 앞으로의 편집 기준으로 삼는다. 현재 동일 조건의 계산은 [공통 QA v5](../evaluation/common-qa-v5/README.md), 이전 수치는 [v3 보존본](../evaluation/common-qa-v3/README.md), 입력과 관측 모집단은 [시나리오 안내](../scenarios/README.md)에서 확인한다. 정의 문서에 남은 2026-10-15의 DP별 예상표와 검토 상태는 해당 세대의 이력이다.

| 현재 정의 문서 | 대표 지표 | 유지한 목표, 상태 | 도출 자료 |
| --- | --- | --- | --- |
| [QA01 첫 응답 지연시간](qa01-first-response-latency.md) | NORMAL의 첫 실질적 음성 시작까지 평균. WAIT는 해당 DP에 별도 표시 | **평균 2초 이하, 0~6점 합의** | [4초 대비 50% 단축 근거](research-notes/qa01-voice-target-rationale.md) |
| [QA02 지칭 대상 식별 정확도](qa02-referent-accuracy.md) | 표현/revision별 최초 채택 대상 집합의 완전 일치율 | 93% 이상 목표 제안, 기존 0~6점 유지 | [정확도 조사](research-notes/qa02-accuracy-research.md) |
| [QA03 VIA 처리시간](qa03-via-processing-time.md) | 전체 목표의 필수 상태/연동/최종 응답 준비까지 VIA 경로 평균 | **평균 5초 이하, 0~6점 합의** | [Windows 지침과 처리 예산](research-notes/qa03-completion-target-rationale.md) |
| [QA04 변경 범위](qa04-change-scope.md) | 변경 과제당 평균 실제 운영 Rust 모듈 수 | 3개 이하 목표 제안, 기존 0~6점 유지 | [ARGO 145건과 산식](targets-source-qa04-argo.md), [조사](research-notes/qa04-change-research.md) |
| [QA05 모델 사용 비용](qa05-model-cost.md) | COMMON-28 전체 묶음당 평균 모델 비용 | $2.55 이하 목표 제안, 기존 0~6점 유지 | [공통 견적](common-model-cost-quote.md), [조사](research-notes/qa05-cost-research.md) |
| [QA06 최대 RAM 사용량](qa06-local-memory.md) | 평가 범위의 VIA 전체 RAM 관측 최고량 | 사용자 지정 512 MB, RAM-v3-MB 점수 유지 | [시스템 요구 출처](targets-source-qa06-platform.md), [조사](research-notes/qa06-memory-research.md) |
| [QA07 요청 해석 및 응답 적합성](qa07-request-meaning-accuracy.md) | 요청 처리 사례의 최초 해석/응답 구성/실제 전달 전체 충족률 | **목표/점수 미정**, 상세 단위/종료와 우선순위 검토 | [목표 재검토](research-notes/qa07-meaning-research.md), [사례 평가 계획](qa07-evaluation-plan.md) |
| [QA08 온디바이스(로컬) 모델 추가 메모리 최소화](qa08-on-device-model-memory.md) | 추가 모델 실행 경로의 같은 시점 메모리 합계 중 최고량 | **사용자 지정 1 GB 미만**, 측정 조건과 점수는 제안, 미측정 | [Galaxy Book 시스템 요구 출처](targets-source-qa08-platform.md) |

선택 우선순위는 기존 **QA01 → QA02 → QA03 → QA04 → QA05 → QA06**을 유지한다. 새 QA07/QA08을 임의로 끼워 넣거나 QA02/QA06을 대체하지 않는다. QA08의 호스트 RAM은 QA06 전체 RAM에도 포함되므로 두 값이나 같은 메모리 절감 효과를 합산하지 않는다. 가중 총점으로 합산하지 않으며 원값, 분모, 단위, 실패와 근거 종류를 함께 남긴다.

## 시간 지표의 확정 기준

현재 총 QA는 **QA01~QA08, 8개**다. QA09는 독립 목록에서 제거했다. QA07은 요청 해석부터 응답 구성/실제 전달까지 확장했으며 별도 QA를 추가하지 않았다.

QA01은 사전 지정 NORMAL에 평균 2초와 별점을 적용한다. 4초 참조 대비 50% 단축이 목표의 도출 근거다. 사용자 발화/앞 응답 대기가 있는 WAIT는 해당 DP의 총 실제 지연, 대기와 대기 종료 후 추가 지연을 별도로 제시한다. 내부 큐/모델 지연은 일반 상황에서도 포함한다.

QA03은 전체 필수 처리와 최종 응답 준비의 VIA 경로 평균 5초를 목표로 한다. 양쪽 우수선은 1초이며 QA01은 0.2초, QA03은 0.8초씩 균등 분할한 점수를 적용한다. 정의 문서가 점수표 정본이다.

[공통 측정 계약과 전달 상태](timing-measurement-contract.md)에 사례 분류, 원장과 갱신 범위를 기록한다. 담당 세션의 새 판본 평가에는 확정 목표와 점수를 적용하며 과거 혼합 평균이나 화면 응답 수치를 그대로 재사용하지 않는다.

## 정의, 적용 계약과 증거의 역할

v5는 [새 공통 계약](../evaluation/common-qa-v5/01-common-contract.md)에서 QA01/03의 새 경계·목표·점수와 QA08 자원 귀속 추정을 반영했다. 기존 동일 사례 비교는 [v4 공통 계약](../evaluation/common-qa-v4/01-common-contract.md)에 보존되어 있다. v4에는 QA01/QA03 개정과 QA07 응답 범위 확장이 미반영이며 24입력과 기존 경계의 계산값을 새 QA 값으로 사용하지 않는다. QA02/QA07은 관측 출력이 없어 미산출이다. QA04는 설계 추정이고 기존 목표·점수나 새 QA07 평가 제안을 자동 적용하지 않는다. 아래 v3는 보존 판본의 역할이다.

1. 이 폴더의 명세는 QA의 의미와 원래 목표, 점수, 출처를 관리한다.
2. [v3 공통 계약](../evaluation/common-qa-v3/01-common-contract.md)은 같은 의미를 Text, Voice/Screen, Delivery의 입력, 목표, 시간, 비용, 메모리 가정에 적용한다. 서로 다른 profile의 값은 합산하거나 DP 간 절대 순위로 읽지 않는다.
3. [v3 원장과 산출물](../evaluation/common-qa-v3/artifacts/README.md)은 모두 `SYNTHETIC_AUTHORED_NOT_MEASURED`다. QA04는 예상 `concept::` 모듈 수이고 실제 Rust diff가 아니다. 비용과 RAM도 작성한 가정의 값이다.
4. COMMON-28의 목표와 점수는 그 단위와 모집단을 가진다. v3의 서로 다른 활동 묶음에 $2.55와 기존 점수표를 자동 적용하지 않는다. 신규 목표, 점수, 모집단 또는 새 실측은 별도 판본과 근거가 필요하다.
5. DP44의 QA02, QA07 100%는 동일 정답을 공급한 제어 fixture다. 정확도 향상이나 차별화 coverage를 주장하지 않는다.

## 목표 근거와 발표에 사용할 자료

- [목표 도출 원칙](target-rationale.md), [조사 상세 목록](research-notes/README.md), [검토와 정정 이력](review-and-revision-log.md).
- [품질 요구사항과 발표용 표](quality-requirements-table.md), [QA 한 장 설명 원고](slide-page-plan.md). 사용자 요청으로 발표자료는 이번 신설 범위에서 제외한다. QA01/QA03/QA07/QA08의 최신 정의는 위 목록에서 확인하며 해당 판본의 발표 페이지는 아직 작성하지 않았다.
- [COMMON-28 시나리오 원전](../scenarios/common-28-v1/common-scenarios.md), [JSON](../scenarios/common-28-v1/common-scenarios.json)은 목표와 기존 회귀 명세의 출처다.
- [이전 72개 사고 실험](history/qa-dp-thought-experiment.md), [이전 패키지 개요](history/source-package-overview-20261015.md)는 기존 DP44/47/45/42 세대의 이력이다. 현재 네 DP의 수치와 결론으로 재사용하지 않는다.

## 편집과 검증

새 QA 편집은 이 폴더에서 수행하고 정의, 목표, 점수의 변경 이유와 버전을 기록한다. 이관 전 원본과 과거 동결 corpus는 수정하지 않는다. 이관 작업에서는 숫자를 유지했으며 후속 사용자 요청에 따라 QA07의 90% 목표, 점수와 세부 평가 절차를 `QA07-REQ-v1` 제안으로 추가했다. `v1.1`에서는 TreeDST의 정체, 의미 항목별 판정 범위와 80개 후보의 도출 출처를 보완했다. 80건은 전체 의미가 명확하다고 검증된 집합이 아니며 현재 공통 v4의 24입력과 구별한다. 이후 `QA07-REQUEST-RESPONSE-v2`에서 응답 구성/실제 전달까지 범위를 확장하고 사례별 전체 충족률을 제안했다. 이전 v1.1과 목표 90%/점수는 보존본으로 옮겼으며 새 목표/점수는 미정이다. 실제 측정과 기존 결과의 재채점은 수행하지 않았다.

[이관 원장](../qa-migration.json)은 원래 경로, 현재 경로와 원본 SHA256을 기록한다. 원래 문서에 적힌 `check_qa_candidate_package.py` 및 `qa_candidate_population.py`의 명령은 **기존 패키지와 원전 생성 규칙의 검증/재생성 명령**이다. 현재 파일을 갱신하는 명령으로 실행하지 않는다. 현재 패키지의 이관, 링크, 점수 일관성 검사는 `python3 scripts/architecture/check_current_package.py`로 수행하며 제품, 모델, 장치 측정은 아니다. 시나리오나 출력의 후속 편집은 [평가 보존과 편집 규칙](../evaluation/common-qa-v3/migration-and-editing.md)을 따른다.
