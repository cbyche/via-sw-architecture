# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|BASIC_CORRECTED|05|A|12|4306.803|N/A|11018.836|1.500|0.0338936|0.0679472|49.119|0|N/A|94.444|1|1/1|
|BASIC_CORRECTED|05|B|12|4421.473|N/A|12438.588|1.750|0.2091628|0.0795200|57.229|0|N/A|100.000|2|0/2|
|BASIC_CORRECTED|05|C|12|4320.514|N/A|4329.101|1.500|0.0113928|0.0644000|48.153|0|N/A|100.000|2|0/2|

QA01 preserves wrong-answer time; errors are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 0. All raw traces and every billed/unknown call remain retained.
