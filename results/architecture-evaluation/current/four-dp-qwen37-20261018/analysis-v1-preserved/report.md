# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|N-R %|N-H %|Errors|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|SMOKE|01|C|1|5422.097|100.000|5422.142|N/A|0.005|0|38.666|N/A|N/A|0|
|SMOKE|02|A|1|4270.843|N/A|4270.945|N/A|0.003|0|37.929|0.000|N/A|0|
|SMOKE|04|A|1|6218.492|0.000|6218.527|N/A|0.001|0|38.355|N/A|N/A|0|
|SMOKE|05|C|1|0.015|N/A|8.800|N/A|0|0.002|38.093|N/A|100.000|0|

QA01 preserves wrong-answer time; errors are separate. QA03 excludes only actual receiver/user wait intervals. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 0. All raw traces and every billed/unknown call remain retained.
