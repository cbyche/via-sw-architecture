# 12. Reference Architecture and Structural Choices

> 상태: **Reference Architecture를 활용한 Decision Reconstruction / Stage 4 핵심 기능 대안 탐색** / 2026-10-03
> [최신 결과: 여섯 구조 비교](./decision-packages/04-30-comparison-guide.md). 기존 target은 참조 구조이며 두 비교안 중 하나로 고정하지 않는다.

12번은 01~11의 제품 경계와 사용자 행동, 품질 의미를 바탕으로 중요한 문제를 서로 다른 구조로 해결하는 방법을 비교한다. 기존 target은 기능 전반을 살핀 참조 구조다. 아래 초기 작업 흐름과 target 유래 규칙은 설계 이력이며, 현재는 두 안 모두 참조 구조와 달라도 된다.

## 작업 흐름

```text
01~11의 고정 전제
  → 완성된 목표 Architecture 설계·합의
  → 중요한 구조적 선택과 강한 현실적 대안 구체화
  → 후보 구체화와 함께 ASR 추가·변경·비교 기준 정의
  → 선택 구조의 인과·비용·약점·반증 조건 설명
  → 결과 전 검증 계약과 revalidation
```

현재 탐색은 참조 구조를 방어하는 결론을 전제하지 않는다. 두 안의 의미 있는 기능 차이와 비용, 각각을 선택할 조건을 먼저 밝힌다. 대안을 일부러 약하게 만들거나 측정 결과를 결론에 맞추지 않는다.

## 현재 읽기 순서

| 순서 | 문서 | 목적 |
| --- | --- | --- |
| 1 | [Target Architecture 작업 공간](./target-architecture/README.md) | 접근법, 고정 경계, 현재 산출물의 한계 이해 |
| 2 | [전체 Architecture 기준선](./target-architecture/architecture.md) | 합의한 Component, 상태, 계약, runtime, 동시성, 장애와 위험 확인 |
| 3 | [설계 완결성 점검](./target-architecture/design-completeness.md) | 필수 기능·예외 대응과 구체화한 주요 선택 확인 |
| 4 | [Target-derived Decision Packages](./decision-packages/README.md) | 구조적 선택·steelman·ASR 후보 목록과 후속 package 도출 규칙 |

1~3의 주요 설계와 사용자 검토는 완료했다. 다음 작업은 4의 지침에 따라 사용자와 후보 목록을 함께 구체화하는 것이다. 정식 package·ASR 재정의·검증 계약은 후보 검토를 거쳐 발전시키며 이번 기준선 확정에서 측정 freeze나 구현·실험을 시작하지 않는다.

사용자 리뷰에 따라 처음 7개 Component 경계 후보와 직전 8개 동일 배치 중심 후보 세대는 archive로 보존했다. 현재 [상세 비교안 4개](./decision-packages/README.md)는 **의미 검색 서브시스템, 내구 요청 workflow, 음성 입력 근거 실행 구성, 권위 복구 저장**이다. 각 문서는 target을 방안 1로 고정하고, 실제 Component·실행체·저장소의 존재·부재·책임 이동이 다른 방안 2를 제시한다.

배경·비교 그림 총 8장(SVG/draw.io), 실행·정정·삭제·복구 계약, ASR/추가 QA 비교표와 발표 원고를 제공한다. [지속 작업 계획](./decision-packages/00-workplan.md), [발굴·선발 기록](./decision-packages/discovery-and-selection.md), [선발 원칙](./decision-packages/selection-principles.md), [검토 기록](./decision-packages/review-notes.md)에 작업 방법과 판정 근거를 남겼다. 자료 완성은 최종 DP 선정·대안 채택·기준선 변경·성능 우열 확정을 뜻하지 않는다.

## 목표 Architecture 이후의 Decision Package

새 package는 다음 조건을 모두 만족하는 선택만 다룬다.

- 목표 Architecture 안에 실제로 선택된 구조다.
- responsibility, contract, state ownership, call graph, deployment, persistence 또는 fault boundary의 차이다.
- 후보 구체화 과정에서 정의한 ASR에 실질적인 인과 효과가 있다. 현재 네 ASR 밖의 중요한 품질 효과도 검토한다.
- 같은 문제를 해결하는 강한 현실적 대안을 구성할 수 있다.
- 선택 구조가 기대한 특성을 갖지 못했다고 판단할 반증 조건을 둘 수 있다.

Component가 있다는 이유만으로 package를 만들지 않는다. 새 package는 기존 VIA-DP 번호를 이어받거나 기존 inventory에 다시 매핑하지 않는다.

현재 네 ASR은 목표 설계의 작업 기준이다. 다음 Decision Point와 steelman 후보 구체화에서 ASR의 추가·변경 여부, 의미·우선순위·적용 범위·평가 기준을 함께 정의한다. 메모리 사용량은 사용자 리뷰의 중요 후보이며 QA-41의 diagnostic 지위는 아직 변경하지 않았다. 이후 package 비교에서는 그렇게 정한 ASR 전체를 다루고 각 축을 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED`로 분류한다. 선택안과 steelman의 applicable 축은 같은 계약으로 독립 측정하며, 어느 한쪽을 먼저 탈락시켜 다른 축을 생략하지 않는다.

## 기존 VIA-DP-01~18의 위치

기존 문서는 이전 decision-first 작업의 active reference로 보존한다. 당시의 Core DP 선정, A/B 정의, measurement 초안과 ADR caveat를 왜곡하거나 현재 측정 결과인 것처럼 바꾸지 않는다.

이 문서들은 새 목표 Architecture의 구성요소 목록이나 새 작업 순서가 아니다. 목표 구조를 만들기 전에 전체 inventory를 순회하지 않으며, 누락 확인·과거 근거·이미 승인된 ADR 제약을 확인할 때만 참고한다.

- [이전 Current Architecture Focus](./dp-executive-summary.md)
- [이전 검토 protocol](./dp-review-protocol.md)
- [이전 평가 방법](./evaluation-method.md)
- [이전 공통 계약](./common-contract.md)

### 이전 Core DP 보고서

- [VIA-DP-03 — 음성 입력 근거의 최종 기준](./via-dp-03-voice-evidence.md)
- [VIA-DP-05 — 요청 Context의 획득 계획 확정 계약](./via-dp-05-context-contract.md)
- [VIA-DP-17 — Context 표현의 최종 의미 소유권](./via-dp-17-context-materialization-authority.md)
- [VIA-DP-06 — 요청 의미의 최종 확정 권한](./via-dp-06-semantic-authority.md)
- [VIA-DP-07 — 복합 요청 graph의 dependency 실행 권한](./via-dp-07-compound-orchestration.md)
- [VIA-DP-15 — Agent progress state의 유지 방식](./via-dp-15-agent-observation-authority.md)

### 그 밖의 이전 DP inventory

- [VIA-DP-01 — 범위가 정해진 정보 처리의 책임 경계](./via-dp-01-direct-handling.md)
- [VIA-DP-02 — 대화와 Task 관계의 확정 경계](./via-dp-02-state-consistency.md)
- [VIA-DP-04 — S2S 직접 응답의 게시 권한](./via-dp-04-response-authority.md)
- [VIA-DP-08 — 재시작 후 상태의 기준 기록](./via-dp-08-recovery-source.md)
- [VIA-DP-09 — Agent 수명 계약의 의미 해석 위치](./via-dp-09-agent-semantics.md)
- [VIA-DP-10 — Model 세션·연결 수명의 관리 권한](./via-dp-10-model-session-authority.md)
- [VIA-DP-11 — 외부 연동 코드의 Process 장애 경계](./via-dp-11-process-isolation.md)
- [VIA-DP-12 — 응답 게시와 실행 근거의 영속 확정 순서](./via-dp-12-evidence-commit.md)
- [VIA-DP-13 — 사용자 제어를 위한 실행 자원을 예약할 것인가](./via-dp-13-control-reservation.md)
- [VIA-DP-14 — Task 상태 전이의 소유권](./via-dp-14-task-state-authority.md)
- [VIA-DP-16 — 모델 입력 이력의 구성·유지 책임](./via-dp-16-model-context-state.md)
- [VIA-DP-18 — 보호정보·Action 사용 시 권한을 확인하는 위치](./via-dp-18-authorization-enforcement.md)

## 아직 바뀌지 않은 사실

- 제품 경계, 대표 Use Case와 현재 QA 의미는 01~11의 active 문서를 따른다.
- 현재 목표는 on-device Omni 1개를 음성·semantic 두 역할이 공유한다. 입력 보호용 ASR을 포함한 주안은 target 문서에서 발전시키며, 이전 DP의 모델 portfolio는 그대로 보존한다. 실제 capability 확보·성능 확인은 별개다.
- 기존 accepted/deferred ADR의 상태와 caveat는 유지한다. 새 목표 구조와 충돌하면 숨기지 않고 재검토 필요성을 기록한다.
- 현재 새 후보 구현과 Core-ASR 결과는 없다. 기존 reference·archive evidence를 새 Architecture의 결과로 소급하지 않는다.
