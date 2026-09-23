# VIA Architecture Baseline

> 상태: **현재 Architecture source of truth**

이 디렉터리는 VIA의 시스템 정의에서 출발해 품질 속성, 측정 계약, Decision Point를 도출하는 현재 기준선이다. 과거 v1.1, vNext/DP-00, W12-G1 자료는 [`../archive/`](../archive/README.md)에 격리한다.

## Reading order

| 순서 | 문서 | 상태 |
| --- | --- | --- |
| 1 | [System Mission & Boundary](./01-system-mission-and-boundary.md) | current |
| 2 | [Terms](./02-terms.md) | current |
| 3 | [Fixed Architecture Scope](./03-fixed-architecture-scope.md) | current |
| 4 | [Canonical Interaction Flow](./04-canonical-interaction-flow.md) | current |
| 5 | [Representative Use Cases](./05-representative-use-cases.md) | current |
| 6 | [Fixed Assumptions](./06-fixed-assumptions.md) | current |
| 7 | [Intentional Variables](./07-intentional-variables.md) | current |
| 8 | [Quality Attributes](./08-quality-attributes/README.md) | W-01~W-03 재정의 완료, 구현 전 |
| 9 | [Traceability](./09-traceability.md) | current |
| 10 | [Architecture Element Definition](./10-element-definition.md) | current |
| 11 | [Measurement](./11-measurement/README.md) | 새 W-01~W-03 machine contract 재동결 전 |
| 12 | [Decisions](./12-decisions/README.md) | DP별 A/B 직접 비교 기준 |

## Current review point

W-01~W-03은 Voice 중심으로 재정의됐다. [`voice-responsiveness.md`](./08-quality-attributes/voice-responsiveness.md)가 세 지표의 endpoint, 포함·제외 구간, S2S 234 ms와 VIA LLM latency evidence의 허용 범위를 정의한다.

현재 순서는 다음과 같다.

1. 각 DP의 A/B 대안과 W-01~W-03 applicability를 확정한다.
2. use case별 Voice fixture, prompt/token ledger, model-rate profile, mock dependency와 audio playback 관측 계약을 결과 전에 동결한다.
3. 새 계약을 machine-readable fixture와 harness로 구현한다.
4. 동일 조건의 paired run으로 각 DP의 A/B를 직접 비교한다.

기존 W12-G1 harness와 결과는 새 정의의 측정값이 아니다. 실제 모델·제품 latency로 주장하지 않으며, 새 측정 구현 전까지 W-01~W-03은 `NOT_IMPLEMENTED / NOT_RUN`이다.

## Authoring rules

- 사용자 요구와 완료 조건을 특정 후보에 맞춰 바꾸지 않는다.
- 각 DP는 다른 DP 조건을 고정한 A/B paired comparison으로 평가한다.
- contract와 fixture는 결과를 보기 전에 동결한다.
- 계산값, mock/reference 측정, 실제 모델 측정, 제품 측정을 evidence label로 구분한다.
- active 문서가 archive 문서를 현재 기준으로 가리키지 않도록 한다.
