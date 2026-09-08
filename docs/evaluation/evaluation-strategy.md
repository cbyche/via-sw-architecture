# Evaluation Strategy

## Principle

Separate three kinds of quality:

1. **Product E2E quality** — what the user experiences.
2. **VIA software quality** — what VIA architecture directly controls.
3. **Model / Downstream Agent quality** — dependencies measured independently.

## Latency reporting

- p50: typical experience
- p95: primary architecture acceptance metric
- p99: diagnostic only during prototype stage

## Architecture isolation

Use deterministic downstream-agent stubs when evaluating VIA overhead.

Example profiles:

| Stub | Behavior |
|---|---|
| D0 | result after 200 ms |
| D1 | result after 2 s |
| D2 | progress + result after 10 s |
| D3 | periodic progress + result after 30 s |
| D-Cancel | runs until cancel |
| D-Fail | configurable timeout/error |

## Baselines

Each DP must define a naive/reference baseline before optimized alternatives are compared.
