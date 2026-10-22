# VIA 품질 요구사항 표와 발표용 요약

2026-10-23 / 현재 정의 목록 / 시간 QA 정의와 측정방법 합의 / 새 평가 갱신 요청 승인

[QA 목록](README.md)과 [시간 측정 계약](timing-measurement-contract.md)을 따른다. 이전 화면 QA01/완료 QA03의 표와 점수는 [변경 전 원고](history/timing-before-voice-20261022/quality-requirements-table.md)에 보존한다. 아래는 요구사항 확인용 표이며 실제 발표 파일의 반영을 뜻하지 않는다.

<a id="detailed-table"></a>
## 현재 요구사항

| ID | 요구사항명 | 대표 지표 | 목표와 상태 |
| --- | --- | --- | --- |
| [QA01](qa01-first-response-latency.md) | 첫 응답 지연시간 | 첫 실질적 음성의 실제 시작까지 평균 경과시간. 차례 대기 포함과 별도 진단 | 정의 합의. [평균 2초 이하 및 점수 제안](research-notes/qa01-voice-target-proposal-20261023.md), 채택 전 |
| [QA02](qa02-referent-accuracy.md) | 지칭 대상 식별 정확도 | 지칭별 최초 대상 집합 완전 일치율 | 93% 이상 제안 |
| [QA03](qa03-via-processing-time.md) | VIA 처리시간 | 목표의 상태/연동/최종 응답 준비 완료까지 VIA 귀속 경로 시간의 평균 | 정의 합의. [평균 5초 이하 및 점수 제안](research-notes/qa03-completion-target-proposal-20261023.md), 채택 전 |
| [QA04](qa04-change-scope.md) | 변경 범위 | 변경 과제당 실제 운영 Rust 모듈 변경 수의 평균 | 3개 이하 제안 |
| [QA05](qa05-model-cost.md) | 모델 사용 비용 | 지정 활동 묶음의 전체 모델 사용 비용 | COMMON-28 묶음 $2.55 이하 제안. 다른 묶음에 자동 적용하지 않음 |
| [QA06](qa06-local-memory.md) | 최대 RAM 사용량 | VIA 전체 RAM 관측 최고량 | 사용자 지정 512 MB 이하. DP별 적용 상태는 상세 계약 참조 |
| [QA07](qa07-request-meaning-accuracy.md) | 요청 의미 해석 정확도 | 입력별 최초 필수 의미 전체 일치율 | 90% 이상 제안 |
| [QA08](qa08-on-device-model-memory.md) | 온디바이스 모델 추가 메모리 최소화 | 추가 모델 실행에 필요한 동시 메모리 관측 최고량 | 사용자 지정 1 GB 미만 |

<a id="presentation-table"></a>
## 발표 반영 상태

QA01/QA03의 새 목표와 점수를 합의한 뒤 원고를 작성한다. QA09 독립 행과 점수는 사용하지 않는다. 차례 대기와 음성 준비/전달 상태는 QA01 내부 진단이다. 기존 v4 수치, 발표 v10과 예전 점수표를 새 계약으로 재명명하지 않는다.

기존 선택 순서는 QA01→QA02→QA03→QA04→QA05→QA06이며 QA07/QA08의 삽입 순위는 합의 전이다. QA01/QA03이 실제 대표 사례에서 중복되는지는 향후 같은 사례의 결과와 처리 경로로 확인한다. 사용자가 수치 담당 세션의 갱신을 요청했으며 실제 전달 상태는 [공통 계약](timing-measurement-contract.md)을 따른다.
