# Current Architecture Candidates

> **Status: DP41 A/B PoC candidates implemented and measured — 2026-10-12.**

[최종 측정 보고서](../../results/architecture-evaluation/current/dp41-p21-mac-text-v7-20261012/final/report.md)는
본 실행 전 고정한 후보 source와 실제 210건의 결과를 사용한다. 성공/실패와 별개로
여섯 QA 측정은 완료됐으며, 이 구현의 결과를 전체 VIA의 구조적 우열로 일반화하지 않는다.

현재 후보는 [dp41_45_poc](./dp41_45_poc/)의 Python 구현이다. `dp41/a.py`는 모델이
READ/CLARIFY/PROPOSE를 선택하고 조회 결과를 다시 해석하는 반복 구조다.
`dp41/b.py`는 유한 Frame의 항목별 해결 상태와 조회/부분 판단/질문을 코드가 관리한다.
`shared/`는 공통 owner, 모듈형 Core, 비동기 continuation, 실제 payload를 보존하는
Agent stub, 출력 sink, API·메모리·인과 기록을 제공한다.

[공통 실행 계약](../../docs/architecture/12-decisions/decision-packages/04-40-common-execution-contract.md)과
[QA 정의](../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)를 기준으로
대안의 실제 처리 경로를 비교한다. 목표값 달성이나 이전 A/B 우열 재현을 구현 목표로 삼지 않는다.

새 구현 위치와 순서는 [PoC 수행 계획](../../docs/architecture/12-decisions/decision-packages/poc-implementation-plan.md)을 따른다.
사용자가 DP41 A/B의 여섯 QA 완료와 API 비용을 승인했으며 실제 구현·호출을 수행했다.
[1단계 검토 묶음](../../docs/architecture/12-decisions/decision-packages/poc-preimplementation-review/README.md)은
DP41 첫 세 사례의 유한 실행 계약과 공통/후속 DP의 owner 경계를 구체화한 구현 전 초안이다.

이전 Rust/Python 후보와 미완성 source는
[이전 실험 전체 보관 안내](../../scripts/archive/pre-dp41-45-experiment-tools-20261011/README.md)에 연결된
세대별 archive에 보존한다. 이전 후보를 새 DP 번호로 바꾸거나 기존 oracle/QA를 자동 승계하지 않는다.
