# DP05 공통 원문 재사용 교정의 실행 소스

이 crate는 원래 BASIC 전체 소스에서 승인된 DP05 세 파일만 바꾼 실제 교정 source의 영구 복사본이다. DP01/02/04의 발표 수치는 원래 BASIC 동결본을 사용한다. 이 복사본을 새 전체120 결과로 해석하지 않는다.

정확한 source/binary/freeze SHA와 원래 baseline 관계는 EFFECTIVE-SOURCE.json에 있다. 기존 BASIC 소스·원시 결과·보완 binary는 덮지 않았다. build/cache는 복사하지 않았다.

`cargo build --offline --manifest-path prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/Cargo.toml`로 재빌드할 수 있다. 실제 측정 binary는 결과 freeze에 별도로 보존돼 있으며 새 빌드의 경로/메타데이터에 따라 binary SHA는 달라질 수 있다. API 재실행은 기존 결과를 덮지 않는 새 승인 phase가 필요하다. 전체 재현은 [FINAL-REPRODUCTION.md](../../../benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md)에 있다.
