# 구현 전 검토 묶음의 작성과 교차 검수

> **DOCUMENT_REVIEW / NOT_IMPLEMENTED / NOT_RUN — 2026-10-11.**
> [사용자 검토용 요약](./README.md)을 위한 작성 기록이다. 아래 PASS는 문서 정적 검사이며
> 후보 코드의 동작, 의미 모델 품질과 QA 실측을 뜻하지 않는다.

## 1. 분업과 통합

사용자가 첫 단계와 sub-agent 활용을 지시했다. 세 sub-agent가 사례/정답, DP41의
실행 계약, 여섯 QA/변경 과제를 각각 작성했다. 주 작업자는 공통 실행/기록 계약과
요약을 작성하고 중복 계약을 통합했다. 이후 세 sub-agent가 자기 부록 이외의 문서,
원본 계약과 기록 예시를 읽기 전용으로 교차 검토했다.

| 검토 | 확인한 범위 | 결과 |
| --- | --- | --- |
| 사례 담당의 교차 검토 | 요약, DP41와 공통 계약 대조. 실제 old job, checkpoint/tail, route, stub, oracle와 사건 공개 범위 | P1 없음. P2 두 개를 발견하고 수정 문단 재확인 후 해당 범위 잔여 P1/P2 없음 |
| DP41 담당의 교차 검토 | 요약, QA 부록, 공통 계약과 JSON 예시. A/B 의미 소유, binding/부분 반환, 예산과 M04/M06 | 중복 P2 한 개. 수정 예산 재확인 후 잔여 P1/P2 없음 |
| QA 담당의 교차 검토 | 요약, 사례, 공통 계약과 JSON 예시. QA02/03, 비용/증거 수준, Mac 합과 30개 변경 | 잔여 P1/P2 없음. JSON 메모리 합 필드 분리 제안을 반영 |

검토 발견 수는 중복 제거하여 **P2 두 개**다. 이는 문서에 남아 있던 계약 충돌이며
실제 실행 실패를 발견한 것이 아니다.

## 2. 수정한 사항

| 발견 | 왜 수정했는가 | 반영 |
| --- | --- | --- |
| 미래 외부 사건 계획이 후보 가시 입력으로 읽힐 수 있음 | 미래 schedule/보류 반환 payload를 먼저 보면 의미와 현재성 비교가 오염됨 | [공통 계약 §4](./04-common-runtime-and-evidence-contract.md#4-입력과-사건-재현): 입력/허용 source/before-state와 replay-only 계획을 분리. availability 이후 실제 envelope만 후보에게 전달 |
| read16/의미8의 예산 단위가 job과 Request 사이에서 다름 | job 재시작/정정/부분 재시도로 후보별 허용 경로가 달라질 수 있음 | [DP41 §8](./02-dp41-executable-contract.md#8-예산-오류-분류와-완료-기준): logical Request 총예산으로 통일. 새 job ID/generation으로 초기화 금지 |
| JSON의 메모리 합 필드가 어느 지표인지 구별하지 못함 | footprint와 RSS는 각각의 process 합과 peak가 필요함 | [기록 예시](./evidence-record-example.json): 두 지표의 합 필드로 분리. 미수집값은 null |

검수 중 확인한 기존 Frame v1의 실행 빈틈은 [새 runtime draft](./02-dp41-executable-contract.md)에
별도 제안으로 남겼다. 기존 schema와 QA 원문을 개정하지 않았다. M04/M06의 baseline
기능 중복, QA02의 오답 후 정정 membership과 S3-02의 반응 endpoint는 사용자 검토 항목이다.
이를 해소한 것처럼 표시하지 않는다.

## 3. 정적 검사와 남은 한계

다음 기존 검사들이 통과했다.

- `check_active_markdown_links.py`: 활성 Markdown 168개의 local link 확인.
- `check_active_terminology.py`: 활성 용어와 정식 Component 이름 확인.
- `qa_common_workload.py --check`: P21 105/630, 135반응/95사이클과 한 묶음5회 반복 확인.
- `qa_poc_reference.py --check`: 여섯 QA 원본/계산/점수 경계의 정합성 확인.
- `dp41_output_contract.py`: 기존 설계 Frame/부분 반환/읽기 예시 확인. 신규 runtime draft 실행 검사가 아님.
- `check_redetailing_contracts.py`: 기존 44/45/42의 문서 예시 확인. 실제 protocol 실행 검사가 아님.

새 JSON 기록과 Markdown 내 JSON 예시4개의 구문, 30개 job ID와18개 registry ID,
문서 예시의0.2초/5초 분리와 NOT_RUN/null 표시도 정적으로 확인했다. 새 후보/harness
구현 경로가 아직 없는 것도 확인했다. `git diff --check`와 관련 링크를 최종 갱신 뒤
재확인했다. 후보 실행 검사와 API 호출은 수행하지 않았다.

전체 CI의 기존 DP42/44/45 생성물 drift는 [이전 게시 검증 기록](../poc-implementation-plan.md#5-준비-문서-게시-전-검증)에
보존돼 있다. 이번 범위는 해당 도식/생성기의 수정이나 전체 CI 통과 주장이 아니다.
다른 세션의 04-50, 입력 근거 도식/문서, 전체 발표 검토와 `work/` 변경은 보존한다.
