# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|BASIC_CORRECTED|05|A|4|4506.525|N/A|11383.623|1.500|0.0120716|0.0146216|48.185|0|N/A|100.000|0|0/0|
|BASIC_CORRECTED|05|B|4|4270.872|N/A|12062.730|1.750|0.0785604|0.0298100|57.278|0|N/A|100.000|1|0/1|
|BASIC_CORRECTED|05|C|3|3971.520|N/A|4007.987|1.500|0.0025608|0.0127344|47.858|0|N/A|100.000|0|0/0|

QA01 preserves wrong-answer time; errors are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 0. All raw traces and every billed/unknown call remain retained.
