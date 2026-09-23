# Gate 2 Evidence

이 디렉터리는 Gate 2 campaign evidence의 상태 경계를 명확히 한다.

| Path | Meaning |
| --- | --- |
| [current](./current/README.md) | 현재 frozen contract와 source revision으로 생성한 evidence만 허용 |
| [archive](./archive/README.md) | superseded campaign 결과와 review provenance |

결과는 contract보다 먼저 존재할 수 없다. Raw trace, failure/timeout, environment, source SHA, fixture fingerprint, approval, aggregation method를 함께 보존해야 하며 summary만 단독으로 current evidence에 넣지 않는다.

Archive 결과는 새 W-01~W-03 값, live model run 또는 product latency로 인용하지 않는다.
