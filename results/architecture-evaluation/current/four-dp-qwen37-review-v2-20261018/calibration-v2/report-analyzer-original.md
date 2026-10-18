# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|CALIBRATION_V2|01|A|1|7318.102|100.000|20698.114|N/A|0.0047536|0.0000000|43.549|0|N/A|N/A|0|0/0|
|CALIBRATION_V2|01|B|1|7266.164|100.000|22863.390|N/A|0.0051560|0.0000000|43.467|0|N/A|N/A|0|0/0|
|CALIBRATION_V2|01|C|1|17954.830|100.000|17955.360|N/A|0.0119304|0.0000000|50.102|0|N/A|N/A|0|0/0|
|CALIBRATION_V2|02|A|1|8846.014|N/A|19444.265|N/A|0.0075836|0.0000000|44.024|0|100.000|N/A|0|0/0|
|CALIBRATION_V2|02|B|1|4499.268|N/A|16991.491|N/A|0.0099552|0.0000000|44.728|0|100.000|N/A|0|0/0|
|CALIBRATION_V2|04|A|2|N/A|0.000|N/A|N/A|0.0065508|0.0000000|44.745|0|N/A|N/A|2|2/2|
|CALIBRATION_V2|04|B|2|22562.266|100.000|22562.594|N/A|0.0059852|0.0000000|44.171|0|N/A|N/A|0|0/0|
|CALIBRATION_V2|05|A|1|4440.677|N/A|13293.903|N/A|0.0052788|0.0082544|48.267|0|N/A|100.000|0|0/0|
|CALIBRATION_V2|05|B|1|4518.747|N/A|15191.961|N/A|0.0155812|0.0165744|56.426|0|N/A|100.000|0|0/0|
|CALIBRATION_V2|05|C|1|4084.448|N/A|4117.102|N/A|0.0016296|0.0095180|46.383|0|N/A|100.000|0|0/0|

QA01 preserves wrong-answer time; errors are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 0. All raw traces and every billed/unknown call remain retained.
