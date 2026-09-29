# 12. Target Architecture and Architecture Rationale

> 상태: **Target Architecture Definition / 전체 구조 검토 중**

12번은 01~11에서 정의한 제품 경계·사용자 행동·품질 의미를 바탕으로 VIA의 목표 Architecture를 완성하고, 그 구조를 성립시키는 핵심 선택을 역으로 설명하는 단계다.

## 작업 흐름

```text
01~11의 고정 전제
  → 완성된 목표 Architecture 설계·합의
  → 네 core ASR에 중요한 구조적 선택 추출
  → 각 선택의 강한 현실적 대안 구성
  → 선택 구조의 인과·비용·약점·반증 조건 설명
  → 결과 전 검증 계약과 revalidation
```

이 작업은 Decision Package를 조합해 Architecture를 발견하는 중립적 exploration이 아니다. 먼저 가장 타당한 목표 Architecture를 세우고, 이후 그 구조를 방어하는 rationale를 만든다. 그렇더라도 대안을 일부러 약하게 만들거나 측정 결과를 결론에 맞추지 않는다.

## 현재 읽기 순서

| 순서 | 문서 | 목적 |
| --- | --- | --- |
| 1 | [Target Architecture 작업 공간](./target-architecture/README.md) | 접근법, 고정 경계, 현재 산출물의 한계 이해 |
| 2 | [전체 Architecture 제안](./target-architecture/architecture.md) | Component, 상태, 계약, runtime, 동시성, 장애와 위험 검토 |
| 3 | [검토 기록](./target-architecture/review-log.md) | 사용자 지정 방향, 제안·합의·열린 항목 확인 |
| 4 | [Target-derived Decision Packages](./decision-packages/README.md) | 전체 구조 합의 이후의 package 도출 규칙 |

현재는 1~3만 진행한다. 전체 Architecture가 충분히 합의되기 전에는 새 Decision Package나 측정 freeze를 만들지 않는다.

## 목표 Architecture 이후의 Decision Package

새 package는 다음 조건을 모두 만족하는 선택만 다룬다.

- 목표 Architecture 안에 실제로 선택된 구조다.
- responsibility, contract, state ownership, call graph, deployment, persistence 또는 fault boundary의 차이다.
- QA-19 semantic accuracy, QA-09 responsiveness, QA-29 modifiability 또는 QA-39 reliability/recoverability에 실질적인 인과 효과가 있다.
- 같은 문제를 해결하는 강한 현실적 대안을 구성할 수 있다.
- 선택 구조가 기대한 특성을 갖지 못했다고 판단할 반증 조건을 둘 수 있다.

Component가 있다는 이유만으로 package를 만들지 않는다. 새 package는 기존 VIA-DP 번호를 이어받거나 기존 inventory에 다시 매핑하지 않는다.

현재는 Target Architecture 완성에 집중하며 package를 구체화하지 않는다. 이후 package 비교에서는 네 core ASR을 모두 다루고 각 축을 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED`로 분류한다. 선택안과 steelman의 applicable 축은 같은 계약으로 독립 측정하며, 어느 한쪽을 먼저 탈락시켜 다른 축을 생략하지 않는다.

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
- S2S 1개와 공유 semantic LLM 1개라는 기반을 유지한다. 목표 Architecture 문서가 실제 dependency capability 확보를 주장하지 않는다.
- 기존 accepted/deferred ADR의 상태와 caveat는 유지한다. 새 목표 구조와 충돌하면 숨기지 않고 재검토 필요성을 기록한다.
- 현재 새 후보 구현과 Core-ASR 결과는 없다. 기존 reference·archive evidence를 새 Architecture의 결과로 소급하지 않는다.
