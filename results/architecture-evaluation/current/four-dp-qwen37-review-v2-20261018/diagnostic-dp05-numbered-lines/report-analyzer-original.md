# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|DIAGNOSTIC|05|A|3|4917.844|N/A|14363.581|N/A|0.0158160|0.0271924|703.955|0|N/A|100.000|0|0/0|
|DIAGNOSTIC|05|B|3|5504.059|N/A|13255.524|N/A|0.1174348|0.0173556|702.562|0|N/A|100.000|0|0/0|
|DIAGNOSTIC|05|C|3|5191.650|N/A|5243.135|N/A|0.0039904|0.0233524|703.906|0|N/A|100.000|0|0/0|

QA01 preserves wrong-answer time; errors are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 0. All raw traces and every billed/unknown call remain retained.
