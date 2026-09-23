# Current Architecture Measurement Harness

> **Status: W-01~W-03 `NOT_IMPLEMENTED / NOT_RUN`**

이 디렉터리는 현재 Architecture baseline에 맞는 machine-readable contract, fixture, executable harness, raw-trace validator, aggregation code를 구현할 active 위치다.

구현은 다음 source를 순서대로 따라야 한다.

1. [Voice Responsiveness](../../docs/architecture/08-quality-attributes/voice-responsiveness.md)
2. [Measurement Guide](../../docs/architecture/11-measurement/README.md)
3. [Evaluation Method](../../docs/architecture/12-decisions/evaluation-method.md)
4. 결과 전에 승인된 machine contract와 Measurement Freeze

## Expected layout

새 구현이 시작되면 목적별로 contract/fixture, runner, trace schema, analyzer, tests를 분리한다. Raw result나 generated summary는 이 디렉터리가 아니라 [results/architecture-evaluation/current](../../results/architecture-evaluation/current/README.md)의 새 freeze directory에 저장한다.

## Non-goals

- 실제 실행 없이 model/reference 상수를 합산해 E2E 측정이라고 부르기
- generated WAV를 읽지 않고 input evidence로 기록하기
- integrated trace를 사후 JSON 생성으로 대체하기
- DP와 무관한 축으로 동일 sample을 복제하기
- instrumented sink를 physical audible onset으로 부르기

기존 formula/reference campaign, timing adapter, freeze, result는 [W12-G1 benchmark archive](../archive/w12-g1/README.md)에 있으며 새 evidence를 생성하지 않는다.
