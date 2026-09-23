# Gate 2 Candidate Prototype

IR, TASK, AGENT, EXEC Decision Point 후보의 구조와 불변조건을 검증하는 Rust workspace다.

이 prototype은 Architecture 후보 구현이며 현재 W-01~W-03 측정 harness가 아니다. `fixtures/s2s-delay-smoke.json`과 smoke command는 구조 검증용으로만 사용하며 실제 S2S, 실제 모델 또는 제품 latency를 나타내지 않는다.

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
```
