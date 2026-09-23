# Architecture Decisions

Decision Point별 A/B 대안을 다른 DP 조건을 고정한 상태에서 직접 비교한다. 여러 DP 조합의 global winner를 기본 분석 단위로 삼지 않는다.

- [Evaluation Method](./evaluation-method.md)
- [Gate 2 Decision Points](./gate2/README.md)
- [Accepted ADRs](../../adr/)

기존 full-factorial 결과와 W12-G1 mapping은 [`../../archive/w12-g1/`](../../archive/w12-g1/README.md)에 보존되어 있으며, 새 W-01~W-03의 근거로 재사용하지 않는다.
