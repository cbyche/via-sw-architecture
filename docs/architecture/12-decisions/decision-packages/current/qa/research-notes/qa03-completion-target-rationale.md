# QA03 VIA 처리시간 목표와 점수 근거

2026-10-23 / 사용자 합의 / [정의와 점수 정본](../qa03-via-processing-time.md)

## 1. VIA 자체 처리 예산 5초

[Microsoft, Responsive interactions and Windows Application Performance](https://learn.microsoft.com/en-us/windows/apps/develop/performance/responsive)의 상호작용 분류를 참조한다. 원문 갱신일은 2026-03-18이며 2026-10-23에 확인했다.

| 원문 분류 | 지연 범위 | VIA에 적용하는 판단 |
| --- | --- | --- |
| Wait | 1~3초 | 전체 처리가 더 빠르게 끝나는 개선 수준의 참고 |
| Long wait | 2~5초 | **상단 5초를 VIA 자체 처리 예산으로 채택** |
| Captive | 5~10초 | VIA 자체 처리가 더 긴 대기를 추가하지 않도록 관리 |

VIA는 입력 해석, 근거 취득, 업무 연결, 상태 확정과 최종 응답 준비를 수행한다. 외부 Agent 업무를 제외한 이 처리 경로의 평균 T를 **5초 이하**로 유지한다. 필수 채널과 상태의 완료 의무, 포함/제외 및 병렬 처리 산식은 정의 문서를 따른다.

## 2. 우수선과 균등 구간

5초가 목표 충족선이고 1초는 처리 예산의 20%를 사용하는 우수 수준이다. 1초 이하를 6점으로 두고 `(5−1)/5 = 0.8초`씩 균등 분할한다. 0점은 5초 초과, 1점 이상은 목표 충족이다. 사용자와 합의한 정확한 구간은 정의 문서의 점수표를 따른다.

## 3. 적용

새 완료 계약으로 계산한 목표별 T의 평균에 같은 점수표를 적용한다. 필수 응답의 내용과 채널, 상태 및 연동 의무를 양안에 동일하게 둔다. QA01의 NORMAL/WAIT 분리로 QA03의 목표 집합을 임의로 줄이지 않는다. 과거 평가 원값과 점수는 당시 계약으로 보존한다.

[이전 목표 제안](../history/timing-before-normal-profile-20261023/qa03-completion-target-proposal-20261023.md)과 [과거 조사](qa03-duration-research.md)는 이력이다. 이번 합의에서는 5초 목표와 1초 우수선 및 균등 구간을 채택했다.
