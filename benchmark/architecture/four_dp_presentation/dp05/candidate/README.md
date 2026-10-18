# DP05 발표용 인계 보완의 실행 소스

이 crate는 이전 공통 원문 재사용 교정 소스를 기반으로 발표용 C1 조건 원문 제공과 C2 읽기 cache를 비교한 소스다. 이전 소스의 출처는 EFFECTIVE-SOURCE.json에 보존했고, 이번 비교의 실제 실행·동결본은 [발표용 추가 증거](../../../../../results/architecture-evaluation/current/four-dp-presentation-20261019/README.md)를 따른다. 이 소스를 이전 전체120 결과의 새 동결본으로 해석하지 않는다.

이전 source/binary/freeze SHA와 원래 baseline 관계는 EFFECTIVE-SOURCE.json의 당시 기록이다. 이번 source/binary/freeze SHA는 추가 증거의 각 phase에 별도로 보존한다. 기존 BASIC 소스·원시 결과·보완 binary는 덮지 않았다. build/cache는 발행하지 않는다.

Cargo.lock은 포함하고 build/dependency cache는 발행하지 않는다. 필요한 Rust toolchain과 dependency cache가 이미 있는 환경에서는 `cargo build --locked --offline --manifest-path benchmark/architecture/four_dp_presentation/dp05/candidate/Cargo.toml`로 재빌드할 수 있다. 새 checkout에 dependency cache가 없다면 먼저 네트워크가 가능한 환경에서 `cargo fetch --locked --manifest-path benchmark/architecture/four_dp_presentation/dp05/candidate/Cargo.toml`을 실행하거나 같은 manifest 경로의 `cargo build --locked`로 의존성을 받아야 한다. 이 빌드는 Qwen API 호출과 별개이며 저장된 보고서를 읽는 데 Rust 빌드가 필요하지 않다.

실제 측정 binary는 결과 freeze에 별도로 보존돼 있으며 새 빌드의 경로/메타데이터에 따라 binary SHA는 달라질 수 있다. API 재실행은 기존 결과를 덮지 않는 새 승인 phase가 필요하다. 재생 입력이 참조하는 이전 캠페인의 복원 방법은 [FINAL-REPRODUCTION.md](../../../four_dp_poc_v2/FINAL-REPRODUCTION.md)에 있다. 이 이전 캠페인용 명령이 이번 추가 실험 전체를 재실행한다는 뜻은 아니다.
