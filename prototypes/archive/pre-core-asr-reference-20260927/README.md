# Pre-Core-ASR Candidate Prototype — 2026-09-27

> **Historical only — this is not the active candidate workspace.**

이 디렉터리는 Core ASR 4개와 Core DP 6개가 확정되기 전에 사용한 Rust/Python candidate
workspace를 `candidates/` 아래 보존한다. IR/TASK/AGENT/EXEC 계열과
VIA-DP-02/05/06/09/11/12/13/14 reference 실험을 지원하던 source다.

Cleanup 당시 미완성이던 DP-11 v5 변경도 버리지 않고 함께 보존했다.

- `via-agent-boundary-host` crate 초안
- Reference Agent의 P/Q shape 선택 지원
- Cargo workspace membership과 lockfile 변경
- 대응 runner의 namespace 수정은 benchmark archive에 보존

이 변경은 완성된 campaign이나 현재 Architecture 결론이 아니다. 재생성 가능한 Cargo
`target/` build cache 2.3 GiB와 Python cache는 source archive에 포함하지 않았다.

현재 active candidate 위치는 [prototypes/candidates](../../candidates/README.md)이며, 새
Measurement Freeze 없이 이 archived workspace를 복사하거나 실행해 current evidence를
만들지 않는다.
