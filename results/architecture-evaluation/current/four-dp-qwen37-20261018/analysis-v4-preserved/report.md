# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|BASIC|01|A|12|7226.126|100.000|7226.173|1.500|0.037|0|39.240|N/A|N/A|0|0/0|
|BASIC|01|B|12|7589.081|100.000|7589.136|1.500|0.036|0|39.174|N/A|N/A|0|0/0|
|BASIC|01|C|12|6957.523|88.889|6957.586|1.500|0.122|0|39.977|N/A|N/A|0|0/0|
|BASIC|02|A|12|11791.961|N/A|11792.043|1.500|0.053|0|39.092|77.778|N/A|0|0/0|
|BASIC|02|B|12|9383.159|N/A|9383.291|1.500|0.054|0|39.191|77.778|N/A|0|6/6|
|BASIC|04|A|2|5587.754|0.000|5587.803|1.500|0.002|0|38.748|N/A|N/A|0|0/0|
|BASIC|04|B|2|26321.606|0.000|26321.651|1.500|0.006|0|38.715|N/A|N/A|0|0/0|
|SMOKE|01|C|1|5422.097|100.000|5422.142|N/A|0.005|0|38.666|N/A|N/A|0|0/0|
|SMOKE|02|A|1|4270.843|N/A|4270.945|N/A|0.003|0|37.929|0.000|N/A|0|0/0|
|SMOKE|04|A|1|6218.492|0.000|6218.527|N/A|0.001|0|38.355|N/A|N/A|0|0/0|
|SMOKE|05|C|1|0.015|N/A|8.800|N/A|0|0.002|38.093|N/A|100.000|0|0/0|

QA01 preserves wrong-answer time; errors are separate. QA03 excludes only actual receiver/user wait intervals. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 64. All raw traces and every billed/unknown call remain retained.
