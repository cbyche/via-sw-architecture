# Architecture Decisions

이 디렉터리는 VIA의 구조 질문을 Decision Point로 만들고, 각 DP의 합리적인 A/B 대안을 직접 비교해 선택과 약점을 설명한다.

새로운 Decision Point를 처음부터 도출할 때 사용할 시스템 이해 검토 기록은 [VIA System Understanding Review](./system-understanding-review.md)에 있다. 이 기록은 비규범 분석 입력이며 기존 DP나 ADR을 정답으로 전제하지 않는다.

시스템 이해와 active QA에서 전체 Core DP 후보를 백지 도출한 결과와 구조 강도·QA coverage·독립성 검토는 [Core Decision Point Discovery](./core-dp-discovery.md)에 있다. 이 문서는 사용자 검토 초안이며 기존 후보 정의나 ADR을 변경하지 않는다.

모든 새 후보의 상세 논의에는 [DP 공통 검토 절차](./dp-review-protocol.md)를 적용한다. A/B의 상호 배타성, 양쪽 steelman, hybrid 재구성, 동일 기능·공정한 비교와 완료 기준을 같은 순서로 확인한다.

각 DP 페이지는 처음 읽는 SW Architect 심사관도 이전 대화나 다른 문서 없이 핵심 논리를 따라갈 수 있는 독립된 보고서로 작성한다. 배경부터 현재 판단까지의 보고서 목차와 읽기 검토 기준도 공통 절차에 포함한다.

각 DP의 발표용 배경 인트로와 A/B Mermaid 그림은 [그림 작성 기준](./dp-diagram-guide.md)을 따른다. VIA 내 위치, 두 안의 공통 부분과 구조 차이가 같은 시야에서 드러나야 한다.

## Decision method

1. 하나의 DP가 바꾸는 authority, state ownership, contract, call graph 또는 fault boundary를 명시한다.
2. hybrid와 유력한 제3안을 검토하고, 동일한 결정 범위에서 상호 배타적인 steelman A/B를 구성한다.
3. 다른 DP와 fixture/dependency 조건을 고정한다.
4. QA catalog 전수를 확인하되, 실제 구조 인과가 있는 QA만 해당 DP의 primary driver로 사용한다.
5. A/B raw metric과 correctness를 paired comparison으로 제시한다.
6. 선택안의 장점뿐 아니라 약점, tactic, 재검증 조건을 ADR에 남긴다.

자세한 규칙은 [Evaluation Method](./evaluation-method.md), 대안 inventory는 [Architecture Candidate Decision Points](./candidates/README.md)를 따른다. 여러 DP 조합의 weighted global winner는 기본 의사결정 방식이 아니다.

## Current DP status

| DP | A / B question | Status | Record |
| --- | --- | --- | --- |
| IR-DP01 | integrated vs staged semantic authority | Deferred; A interim reference | [ADR-004](../../adr/ADR-004-semantic-decision-ownership.md) |
| TASK-DP01 | shared transactional service vs durable per-Task supervisor | B accepted; new Voice revalidation required | [ADR-002](../../adr/ADR-002-task-state-authority.md) |
| AGENT-DP01 | edge-normalized canonical contract vs core-visible typed contracts | A accepted | [ADR-001](../../adr/ADR-001-agent-integration-contract-boundary.md) |
| EXEC-DP01 | single-process partition vs process-isolated integration runtime | B accepted; new Voice revalidation required | [ADR-003](../../adr/ADR-003-runtime-fault-isolation-boundary.md) |

“Accepted”는 모든 과거 metric이 현재 정의에도 유효하다는 뜻이 아니다. 각 ADR의 evidence와 revalidation condition을 함께 읽는다.

## What is not current evidence

이전 full-factorial result와 W12-G1 mapping은 [archive](../../archive/w12-g1/README.md)에 있다. 현행 QA catalog에 맞춘 직접 A/B 결과가 아니므로 현재 근거로 재사용하지 않는다. 새 결과가 없으면 `NOT_RUN`으로 남긴다.
