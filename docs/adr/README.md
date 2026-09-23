# Architecture Decision Records

ADR은 현재 Architecture baseline에서 내린 선택, 근거, 약점, 재검증 조건을 보존한다. ADR은 단순 결론 목록이 아니며, status와 evidence limitation을 함께 읽어야 한다.

| ADR | Decision | Status summary |
| --- | --- | --- |
| [ADR-001](./ADR-001-agent-integration-contract-boundary.md) | AGENT-DP01 canonical boundary | A accepted |
| [ADR-002](./ADR-002-task-state-authority.md) | TASK-DP01 state authority | B accepted; new Voice metrics require revalidation |
| [ADR-003](./ADR-003-runtime-fault-isolation-boundary.md) | EXEC-DP01 fault isolation | B accepted; new Voice latency requires remeasurement |
| [ADR-004](./ADR-004-semantic-decision-ownership.md) | IR-DP01 semantic ownership | Deferred; A is interim reference only |

새 ADR은 [template](./TEMPLATE.md)을 사용한다. Architecture 대안과 평가 방법은 [Decisions](../architecture/12-decisions/README.md), 현재 metric 상태는 [Measurement Guide](../architecture/11-measurement/README.md)를 따른다.

ADR을 변경할 때는 결론만 고치지 말고 superseded evidence, new evidence, consequences, revalidation condition을 함께 갱신한다.
