# DP04: fresh BASIC and bounded coordinate repair

All48 actual outcomes; fixed15 inputs/12 goals/24 exact sets per configuration. Conditional means remain conditional.

|Configuration|QA01 seconds observed/15|QA02 correct/24|QA03 seconds complete/12|Calls|USD|Peak RSS MB|Common endpoints eligible|
|---|---|---|---|---:|---:|---:|---|
|A|9.268066 12/15|19/24|12.559165 10/12|22|known $0.0339156 + UNKNOWN 1 (reservation upper $0.0729404)|41.451520|False|
|B|18.921755 14/15|22/24|22.622273 11/12|38|known $0.0446804|41.336832|False|
|A2A5|11.035942 15/15|24/24|13.795300 12/12|23|known $0.0413400|50.036736|True|
|B4|13.881438 13/15|20/24|17.603567 11/12|40|known $0.0459164|42.762240|False|

A2+A5 combines affected-expression patching and new coordinate presentation. It does not isolate their individual causal effects. Shared contract, source revisions, count/type/geometry checks and first-adoption scoring remain unchanged. QA04 was not newly exercised. RSS is sampled local Python/Rust, excluding cloud weights/UI.

Checks: 133/133. Known usage at fixed comparison rates: $0.1658524; unknown usage calls: 1, retained upper-bound reservations: $0.0729404. Unknown is not zero cost.

Actual provider requests: 123 = 122 usage-confirmed + 1 usage-unknown. These are distinct from 133 read-only evidence checks.
