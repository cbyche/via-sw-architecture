# Architecture Candidate Prototype

이 Rust workspace는 IR, TASK, AGENT, EXEC Decision Point 후보의 책임 배치, contract, state ownership, process boundary와 불변조건을 executable form으로 검증한다.

## What this prototype is

- Architecture 대안의 structural behavior를 비교하기 위한 candidate implementation
- deterministic fixture로 contract·recovery·isolation behavior를 검증하는 공간
- 향후 frozen measurement harness가 호출할 수 있는 implementation surface

## What it is not

- 현재 W-01~W-03 measurement harness
- 실제 S2S/VIA LLM 실행 또는 제품 latency evidence
- `fixtures/s2s-delay-smoke.json`의 delay를 production profile로 주장하는 근거
- archived full-factorial 결과를 다시 현재화하는 도구

대안의 정의와 현재 decision state는 [Architecture Candidate Decision Points](../../docs/architecture/12-decisions/candidates/README.md), 측정 규칙은 [Measurement Guide](../../docs/architecture/11-measurement/README.md)를 따른다.

## Verification

Run from repository root:

```bash
cargo +1.98.1 fmt --manifest-path prototypes/candidates/Cargo.toml --all -- --check
cargo +1.98.1 clippy --locked --manifest-path prototypes/candidates/Cargo.toml --workspace --all-targets -- -D warnings
cargo +1.98.1 test --locked --manifest-path prototypes/candidates/Cargo.toml --workspace --all-targets
```

Timed runs require a separately frozen contract and campaign approval; passing these checks is not a performance result.
