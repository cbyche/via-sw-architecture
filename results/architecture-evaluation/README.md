# Architecture Evaluation Evidence

이 디렉터리는 현재 measurement contract와 source revision으로 생성한 A/B measurement evidence를 보관한다.

| Path | Meaning |
| --- | --- |
| [current](./current/README.md) | 현재 계약으로 생성된 evidence만 허용 |
| [historical archive](../gate2/archive/README.md) | 과거 campaign 결과와 review provenance; 현재 결과로 사용하지 않음 |

결과는 measurement contract보다 먼저 존재할 수 없다. Raw trace, failure/timeout, environment, source SHA, fixture fingerprint, approval과 aggregation method를 함께 보존한다.
