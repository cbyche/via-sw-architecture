# Pre-Core-ASR Reference Benchmark — 2026-09-27

> **Historical only — do not run this tree to produce current evidence.**

이 디렉터리는 QA-09·QA-19·QA-29·QA-39와 Core DP 6개가 확정되기 전에 active
`benchmark/architecture/`에 있던 상세-QA measurement generation을 보존한다.

보존 범위는 다음과 같다.

- VIA-DP-02/05/06/09/11/12/13/14 contract, fixture, runner, analyzer와 tests
- QA-01~15 predecessor foundation evaluator
- DP-06 v4/v5/v6 reference integration·change-locality 도구
- DP-11 v4와 **실행 완료 전이던 v5** 도구
- target-Mac audio loopback probe와 Reference Agent 연결 도구

`architecture/` 아래에는 cleanup 직전 source가 그대로 있다. 특히 DP-11 v5에는
trial namespace 충돌을 피하기 위한 runner 수정이 포함돼 있으나, 새 Core ASR 계약의
campaign으로 실행·승인된 상태는 아니다.

이 generation의 결과는
[pre-Core-ASR evidence archive](../../../results/architecture-evaluation/archive/pre-core-asr-reference-20260927/README.md),
candidate source는
[pre-Core-ASR prototype archive](../../../prototypes/archive/pre-core-asr-reference-20260927/README.md)에
있다. 경로, QA 의미, p95 집계, fixture와 command는 당시 세대에만 유효하다.

현재 active 구현 위치는 [benchmark/architecture](../../architecture/README.md)다.
