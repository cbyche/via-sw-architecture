# Architecture Decisions

이 디렉터리는 VIA의 구조 질문을 Decision Point로 만들고, 각 DP의 합리적인 A/B 대안을 직접 비교해 선택과 약점을 설명한다.

**먼저 [DP 최종 요약 보고서](./dp-executive-summary.md)를 읽는다.** 전체 시스템의 난제, 13개 후보의 분류, 우선 검증할 세 결정과 추천 순서를 상세 보고서보다 먼저 설명한다. 후보별 A/B·판정·UC/QA coverage·검증 기록은 [전체 검토 종합](./dp-review-synthesis.md)에 있다.

새로운 Decision Point를 처음부터 도출할 때 사용할 시스템 이해 검토 기록은 [VIA System Understanding Review](./system-understanding-review.md)에 있다. 이 기록은 비규범 분석 입력이며 기존 DP나 ADR을 정답으로 전제하지 않는다.

시스템 이해와 active QA에서 백지 도출한 [후보 지도](./core-dp-discovery.md)는 전체 상세 검토 결과를 반영했다. 초기 12개를 재구성하고 DP-13을 추가했다. 새 후보 정의는 사용자 검토 제안이며 기존 ADR의 결정 상태를 변경하지 않는다.

모든 새 후보의 상세 논의에는 [DP 공통 검토 절차](./dp-review-protocol.md)를 적용한다. A/B의 상호 배타성, 양쪽 steelman, hybrid 재구성, 동일 기능·공정한 비교와 완료 기준을 같은 순서로 확인한다.

각 DP 페이지는 처음 읽는 SW Architect 심사관도 이전 대화나 다른 문서 없이 핵심 논리를 따라갈 수 있는 독립된 보고서로 작성한다. 배경부터 현재 판단까지의 보고서 목차와 읽기 검토 기준도 공통 절차에 포함한다.

각 DP의 발표용 배경 인트로와 A/B Mermaid 그림은 [그림 작성 기준](./dp-diagram-guide.md)을 따른다. VIA 내 위치, 두 안의 공통 부분과 구조 차이가 같은 시야에서 드러나야 한다.

첫 적용 사례인 [VIA-DP-01](./via-dp-01-direct-handling.md)에 이어 DP-02~13의 독립 보고서를 작성했다. 모두 배경·A/B 그림, hybrid·상호 배타성·steelman, 전체 19개 QA 사고실험과 자체 검토를 담는다. 구현·측정·대안 선택은 하지 않았다.

## Detailed review reports

| 현재 분류 제안 | 독립 보고서 |
| --- | --- |
| 우선 핵심 검증 | [11 Process 격리](./via-dp-11-process-isolation.md) · [12 실행 근거 확정](./via-dp-12-evidence-commit.md) · [13 제어 자원 예약](./via-dp-13-control-reservation.md) |
| 조건부 핵심 | [02 대화·Task 확정](./via-dp-02-state-consistency.md) · [04 응답 게시](./via-dp-04-response-authority.md) · [05 Context 읽기](./via-dp-05-context-contract.md) · [06 의미 확정](./via-dp-06-semantic-authority.md) |
| 선행 범위·기능 | [01 직접 처리](./via-dp-01-direct-handling.md) · [03 음성 근거](./via-dp-03-voice-evidence.md) · [07 복합 요청](./via-dp-07-compound-orchestration.md) |
| 보조 설계 | [08 복구 기준 기록](./via-dp-08-recovery-source.md) · [09 Agent 의미](./via-dp-09-agent-semantics.md) · [10 Model 세션](./via-dp-10-model-session-authority.md) |

조건부 후보까지 합쳐 핵심 DP 확정으로 해석하지 않는다. 실제 QA 결과는 모두 `NOT_RUN`이다. VIA-DP의 A/B 문자는 아래 기존 ADR의 A/B 문자와 자동 대응하지 않으며 [이력 매핑](./dp-review-synthesis.md#idab-이력과-기존-adr)을 확인한다.

## Decision method

1. 하나의 DP가 바꾸는 authority, state ownership, contract, call graph 또는 fault boundary를 명시한다.
2. hybrid와 유력한 제3안을 검토하고, 동일한 결정 범위에서 상호 배타적인 steelman A/B를 구성한다.
3. 다른 DP와 fixture/dependency 조건을 고정한다.
4. QA catalog 전수를 확인하되, 실제 구조 인과가 있는 QA만 해당 DP의 primary driver로 사용한다.
5. A/B raw metric과 correctness를 paired comparison으로 제시한다.
6. 선택안의 장점뿐 아니라 약점, tactic, 재검증 조건을 ADR에 남긴다.

자세한 규칙은 [Evaluation Method](./evaluation-method.md), 대안 inventory는 [Architecture Candidate Decision Points](./candidates/README.md)를 따른다. 여러 DP 조합의 weighted global winner는 기본 의사결정 방식이 아니다.

## Existing ADR status

| DP | A / B question | Status | Record |
| --- | --- | --- | --- |
| IR-DP01 | integrated vs staged semantic authority | Deferred; A interim reference | [ADR-004](../../adr/ADR-004-semantic-decision-ownership.md) |
| TASK-DP01 | shared transactional service vs durable per-Task supervisor | B accepted; new Voice revalidation required | [ADR-002](../../adr/ADR-002-task-state-authority.md) |
| AGENT-DP01 | edge-normalized canonical contract vs core-visible typed contracts | A accepted | [ADR-001](../../adr/ADR-001-agent-integration-contract-boundary.md) |
| EXEC-DP01 | single-process partition vs process-isolated integration runtime | B accepted; new Voice revalidation required | [ADR-003](../../adr/ADR-003-runtime-fault-isolation-boundary.md) |

“Accepted”는 모든 과거 metric이 현재 정의에도 유효하다는 뜻이 아니다. 각 ADR의 evidence와 revalidation condition을 함께 읽는다.

## What is not current evidence

이전 full-factorial result와 W12-G1 mapping은 [archive](../../archive/w12-g1/README.md)에 있다. 현행 QA catalog에 맞춘 직접 A/B 결과가 아니므로 현재 근거로 재사용하지 않는다. 새 결과가 없으면 `NOT_RUN`으로 남긴다.
