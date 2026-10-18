# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|BASIC|04|A|12|9268.066|79.167|12559.165|N/A|0.0339156|0.0000000|41.452|0|N/A|N/A|3|3/2|
|BASIC|04|B|12|18921.755|91.667|22622.273|N/A|0.0446804|0.0000000|41.337|0|N/A|N/A|1|1/1|

QA01 preserves wrong-answer time; errors are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 24. All raw traces and every billed/unknown call remain retained.
