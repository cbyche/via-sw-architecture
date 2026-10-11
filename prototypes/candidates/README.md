# Current Architecture Candidates

> **Status: no active candidate implementation.** 현재 DP41/44/45/42의 PoC 범위와 평가 방법을 논의 중이다.

이 디렉터리는 새 비교의 후보 구현 위치다. 현재 Cargo workspace나 executable candidate는
없으며 구현, 모델 호출과 측정은 시작하지 않았다.

[공통 실행 계약](../../docs/architecture/12-decisions/decision-packages/04-40-common-execution-contract.md)과
[QA 정의](../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)를 기준으로
대안의 실제 처리 경로를 비교한다. 목표값 달성이나 이전 A/B 우열 재현을 구현 목표로 삼지 않는다.

새 구현 위치와 순서는 [PoC 수행 계획](../../docs/architecture/12-decisions/decision-packages/poc-implementation-plan.md)을 따른다.
현재 계획은 검토용이며 후보 코드나 모델 API 호출을 시작한 상태가 아니다.

이전 Rust/Python 후보와 미완성 source는
[이전 실험 전체 보관 안내](../../scripts/archive/pre-dp41-45-experiment-tools-20261011/README.md)에 연결된
세대별 archive에 보존한다. 이전 후보를 새 DP 번호로 바꾸거나 기존 oracle/QA를 자동 승계하지 않는다.
