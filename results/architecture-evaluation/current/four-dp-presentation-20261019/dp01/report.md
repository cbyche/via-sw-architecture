# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|OUTPUT_POLICY_SENSITIVITY|01|A|3|1905.169|100.000|23117.772|N/A|0.0157364|0.0000000|40.124|0|N/A|N/A|0|0/0|
|OUTPUT_POLICY_SENSITIVITY|01|A|3|7092.283|100.000|21508.024|N/A|0.0146552|0.0000000|41.189|0|N/A|N/A|0|0/0|
|OUTPUT_POLICY_SENSITIVITY|01|B|3|1828.871|100.000|23421.999|N/A|0.0159652|0.0000000|40.157|0|N/A|N/A|0|0/0|
|OUTPUT_POLICY_SENSITIVITY|01|B|3|6724.036|100.000|22171.558|N/A|0.0150744|0.0000000|39.993|0|N/A|N/A|0|0/0|
|OUTPUT_POLICY_SENSITIVITY|01|C|3|2322.050|100.000|20349.887|N/A|0.0426752|0.0000000|43.074|0|N/A|N/A|0|0/0|
|OUTPUT_POLICY_SENSITIVITY|01|C|3|17730.378|100.000|17731.196|N/A|0.0348104|0.0000000|42.254|0|N/A|N/A|0|0/0|

QA01 preserves wrong-answer time; errors are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 0. All raw traces and every billed/unknown call remain retained.
