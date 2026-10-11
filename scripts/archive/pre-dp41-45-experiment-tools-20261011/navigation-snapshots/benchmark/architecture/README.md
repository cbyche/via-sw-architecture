# Current Architecture Measurement Harness

> **Status: reset for the Core-ASR generation — no active DP harness yet**

이 디렉터리는 현재 Architecture baseline과 승인된 Measurement Freeze를 구현하는 위치다.
QA-09·QA-19·QA-29·QA-39가 확정되기 전에 작성된 runner, contract, fixture, analyzer와
test는 active 경로에서 제거했다.

현재 Core DP의 새 측정 결과는 모두 `NOT_RUN`이다. 다음 active 구현은
[VIA-DP-03](../../docs/architecture/12-decisions/via-dp-03-voice-evidence.md)의 S2S
capability qualification과 A/B Measurement Freeze가 승인된 뒤 시작한다.

## 새 구현의 진입 조건

다음 항목을 결과 전에 동결하지 않았다면 이 디렉터리에 runner를 추가하거나 campaign을
실행하지 않는다.

1. A/B를 구분하는 authority·state·contract·call graph 또는 fault boundary
2. 동일한 user goal, fixture, dependency profile과 fixed DP context
3. QA-09 trial, QA-19 field, QA-29 change, QA-39 fault의 applicability와 모집단
4. source/terminal event, 반복 수, timeout, failure treatment와 단순평균 집계
5. target·score band·evidence label과 source/fixture digest
6. 정상 trace와 의도적으로 깨뜨린 sentinel을 함께 검증하는 qualification test

새 구현은 contract, fixture/oracle, runner, raw validator, analyzer와 test를 분리한다.
Candidate 입력과 evaluator-only oracle도 물리적으로 분리한다. Raw result는 이 디렉터리가
아니라 [current results](../../results/architecture-evaluation/current/README.md)의 새로운
immutable campaign directory에 기록한다.

## 이전 세대

2026-09-27까지 사용한 상세-QA runner와 미완성 VIA-DP-11 v5 작업은
[pre-Core-ASR benchmark archive](../archive/pre-core-asr-reference-20260927/README.md)에
보존했다. 그 코드는 설계 아이디어와 provenance를 확인할 수 있지만, archive에서 실행해
현재 evidence를 만들 수 없다.

현재 규칙은 다음 문서를 따른다.

- [Current Architecture Focus](../../docs/architecture/12-decisions/dp-executive-summary.md)
- [Core ASR Contract](../../docs/architecture/08-quality-attributes/core-asr-contract.md)
- [Measurement Guide](../../docs/architecture/11-measurement/README.md)
- [Core DP Evaluation Plan](../../docs/architecture/11-measurement/major-dp-evaluation-plan.md)
