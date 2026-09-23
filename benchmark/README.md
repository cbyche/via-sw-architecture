# VIA Architecture Benchmarks

이 디렉터리는 Architecture 후보를 동일한 frozen contract로 실행하고 raw evidence를 만드는 측정 코드를 관리한다.

| Path | Status | Use |
| --- | --- | --- |
| [architecture](./architecture/README.md) | Active, currently empty of W-01~W-03 implementation | next machine contract, fixtures, runner, analyzer |
| [archive](./archive/README.md) | Historical | vNext/DP-00 and W12-G1 harness provenance only |

Benchmark code is not the definition of a metric. The semantic definition comes from [Architecture measurement](../docs/architecture/11-measurement/README.md), and a campaign must bind to a source revision and frozen machine-readable contract.

현재 W-01~W-03은 문서 정의만 있고 새 harness는 없다. Archive runner를 실행하거나 상수·trace 이름을 복사해 current result로 발표하지 않는다.
