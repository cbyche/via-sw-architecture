# QA 조사 상세 기록

> **2026-10-23 QA04 개정 안내:** 현재 정의는 [QA04-CHANGE-v4](../qa04-change-scope.md), 적용 과제는 [Q4-CHANGES-v4](../change-scenarios.md)다. 8개 변경 과제에 Rust/Python 등 실제 운영 모듈을 센다. 아래 이전 6과제/Rust 기준과 ARGO 통계는 당시 도출 이력이며 새 모집단의 측정 결과가 아니다. 평균 3개 이하와 점수 간격은 제안으로 유지하고, ARGO와의 단위/범위 비교 가능성은 별도 확인한다. 발표 파일은 변경하지 않았다.

> **현재 QA 자료의 이관본 (2026-10-22):** 출처, 원값, 목표 도출과 기존 검토 내용을 유지한다. 문서 안의 이전 DP, 모집단, 검토 상태는 해당 작성 세대에 한정한다. [현재 QA 기준과 적용 범위](../README.md).


> 2026-10-14 조사 재검토 / QA02와 QA04 및 QA05 목표 개정 / QA06의 새 512 MB 요구사항 반영 / 모델 실행과 측정 없음

사용자가 각 QA를 한 장씩 설명하는 발표자료를 만들 수 있도록 조사 과정의 세부 사실과 수치 및 적용 판단을 보존한다. 단순한 링크 목록 대신 **출처의 위치 → 확인한 내용 → VIA 적용 → 사용할 수 없는 주장 → 발표와 심사에 쓸 설명**을 기록한다.

**정의와 지표 및 점수의 현재 기준은 상위 폴더의 QA 상세 명세다.** 이 부록은 조사 당시의 근거 기록이다. 목표를 바꿀 때 이전 판본을 archive에 보존하고 현재 기록에 조사 날짜와 변경 이유를 남긴다. 문헌 수치와 작성자의 제품 판단 및 설명용 가상값을 구별한다. 논문이나 웹페이지 전문은 복제하지 않고 필요한 사실과 해석 및 원문 위치를 남긴다.

## 읽을 문서

| QA | 조사 상세 부록 | 현재 명세 | 발표에 사용할 핵심 자료 |
| --- | --- | --- | --- |
| 01 | [음성 목표와 점수 근거](qa01-voice-target-rationale.md), [이전 화면 조사](qa01-latency-research.md) | [QA01](../qa01-first-response-latency.md) | 일반 상황 평균 2초, 4초 대비 50% 단축. 차례 대기는 별도 표시 |
| 02 | [대상 지정 정확도 조사](qa02-accuracy-research.md) | [QA02](../qa02-referent-accuracy.md) | ReALM의 집합 완전 일치율 93%와 VIA 과제의 공통점 및 차이 |
| 03 | [완료 목표와 점수 근거](qa03-completion-target-rationale.md), [이전 조사](qa03-duration-research.md) | [QA03](../qa03-via-processing-time.md) | Windows 지침에 따른 5초 처리 예산과 1초 우수선 |
| 04 | [변경 범위 조사](qa04-change-research.md) | [QA04](../qa04-change-scope.md) | ARGO 145건 집계와 40% 개선 요구 및 Rust 모듈 집계 단위 |
| 05 | [모델 비용 조사](qa05-cost-research.md) | [QA05](../qa05-model-cost.md) | gpt-realtime 공개 단가와 공통 28개 묶음의 상세 견적 |
| 06 | [RAM 사용량 조사](qa06-memory-research.md) | [QA06](../qa06-local-memory.md) | 사용자 지정 512 MB 요구사항과 기존 조사 이력 및 Windows 계측 한계 |
| 07 | [업무 순서 및 연결 정확도 근거](qa07-meaning-research.md) | [QA07 업무 순서 및 연결 정확도](../qa07-request-meaning-accuracy.md) | 85% 목표 제안. TaskBench의 그래프 정확도와 SMCalFlow의 전체 의미 구조 정확도, 공통 요청 28개와 5회 반복의 출처 |
| 08 | [Galaxy Book 시스템 요구](../targets-source-qa08-platform.md) | [QA08](../qa08-on-device-model-memory.md) | 1 GB 미만의 사용자 지정 상한과 추가 모델 메모리 측정 조건 |

QA07은 사용자 지정 업무 순서와 신규/기존 업무 연결로 범위를 확정하고 목표 85% 및 평가 절차와 점수를 제안했으며 QA08은 사용자 지정 시스템 요구다. QA01/QA03의 기존 조사 수치는 개정 계약에 자동 적용하지 않는다. 발표자료는 별도 작업 범위다.

## 발표 제작의 진입점

- [QA별 한 장 설명 페이지 구성](../slide-page-plan.md): 현재 정의와 발표 원고의 반영 상태
- [목표 수치 도출 논리](../target-rationale.md): 핵심 여섯 QA의 요구에서 수치까지 이어지는 설명
- [발표용 축약 표](../quality-requirements-table.md#presentation-table): 전체 QA를 한 표에 넣는 문구
- [이관 전 QA와 DP 대응](../history/source-package-overview-20261015.md#4-qadp-대응과-예상-우열): 당시 구조 차이와 예상 우열 및 커버리지
- [독립 교차 리뷰](../review-and-revision-log.md#10-qa0106-목표-근거-재작성과-독립-교차-리뷰): 출처 대조와 수정 이력
- [이번 조사 기록과 페이지 구성안 리뷰](../reviews/research-records-review.md): 원문 대조와 QA05 예시의 평균 조건 보완

## 보존과 재사용 원칙

1. 공개 단가와 장비 사양의 확인일은 2026-10-14다. 이후 발표나 실제 견적에서는 동일 판본을 사용했는지 확인한다.
2. 목표 합격선의 출처와 0~6점 중간 간격을 구별한다. 점수표는 상세 명세에서 가져오며 문헌이 모든 경계를 정했다고 설명하지 않는다.
3. 그림 예시의 수치는 가상값으로 표시한다. 실제 VIA PoC 결과로 바꾸지 않는다.
4. 연구와 다른 과제로 수치를 옮긴 판단 및 제품 상한의 선택을 발표자 노트에 남긴다.
5. 실제 구현 및 계측과 사용자 검증은 문서 검토와 구별한다. 준비되지 않은 측정 방법을 완성된 절차로 표현하지 않는다.

현재 QA04의 직접 수치 근거는 사용자 제공 ARGO 집계다. QA06은 [사용자 지정 512 MB 시스템 요구사항](../targets-source-qa06-platform.md)을 적용한다. 기존 공개 자료에서 그 수치를 도출했다고 표현하지 않는다. [공통 대표 시나리오 28개](../../scenarios/common-28-v1/common-scenarios.md)에서 비용과 RAM의 입력 조건을 확인한다.
