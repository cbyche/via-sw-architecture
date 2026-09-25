# 핵심 Architecture 재선정 논의 — 현행 정리

> 2026-09-25 · 과거 선정 제안을 전수 inventory로 대체

현재 목적은 주요 DP 몇 개를 먼저 고르는 것이 아니라 **가능한 DP 전체를 같은 깊이로 구체화하고 누락·편향을 막는 것**이다. 이 페이지의 이전 ‘우선 6개 ASR’·‘Process 격리 최우선’ 추천은 현재 추천이 아니다.

## 지금 읽을 문서

- [전체 요약 보고서](./dp-executive-summary.md): 최신 18개 후보와 구현 구조 설명.
- [전체 후보 지도](./core-dp-discovery.md): 주제별 독립 질문과 A/B.
- [이전 번호 매핑·누락 점검](./legacy-dp-mapping.md): 이전 계열 9개·25개 주제의 연결.
- [정리·검증 기록](./dp-review-synthesis.md): 이번 반영 범위와 미실행 항목.

## 이전 4개 절의 정확한 대응

| 이전 절 | 현행 문서 | 정리된 의미 |
| --- | --- | --- |
| 4.1 요청 이해 | [VIA-DP-06](./via-dp-06-semantic-authority.md) | 같은 semantic LLM을 통합 또는 단계별 authority가 사용 |
| 4.2 Task 관리 | [VIA-DP-14](./via-dp-14-task-state-authority.md) | 공유 TaskService vs Task별 supervisor. VIA-DP-02와 별개 |
| 4.3 Agent 통합 | [VIA-DP-09](./via-dp-09-agent-semantics.md) | 외부 Agent의 수명 의미를 경계 또는 Core handler에서 확정 |
| 4.4 실행 경계 | [VIA-DP-11](./via-dp-11-process-isolation.md) | VIA Client 코드의 Process 격리. 외부 Agent embed 선택 아님 |

## 철회·유지한 판단

S2S 1개·semantic LLM 1개를 고정하며 모델 복제로 trade-off를 만들지 않는다. QA-41은 카탈로그에 남기되 메모리 요구·유의미한 차이 근거 없이 주요 ASR로 권하지 않는다. VIA Client의 실제 fatal 위험 확인 전에는 VIA-DP-11을 최우선 핵심으로 권하지 않으며 QA-32도 자동 선정하지 않는다.

이전 4개 영역만으로 충분한 최종 DP 4~5개를 확보했다고 말하지 않는다. 반대로 이전에 ‘보조’로 분류됐다는 이유만으로 후보를 지우지도 않는다. 최종 선정은 전체 보고서 검토 이후 별도로 한다. 기존 accepted/deferred ADR과 현행 19개 QA의 metric은 유지한다.

이전 원문은 docs/archive/dp-review-pre-inventory-2026-09-25에 historical provenance로 보존했고, 작업 시작 전 사용자 수정도 그 사본에 남겼다. 과거 예상·선정 문구는 현행 후보의 검증 결과가 아니다.
