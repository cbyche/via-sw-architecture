# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|계약/완료 실패 episode|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|CALIBRATION|01|A|1|6579.120|100.000|23229.091|N/A|0.0070008|0.0000000|44.073|0|N/A|N/A|1|0/1|
|CALIBRATION|01|B|1|6650.857|100.000|21586.553|N/A|0.0047528|0.0000000|43.385|0|N/A|N/A|0|0/0|
|CALIBRATION|01|C|1|18536.097|100.000|33674.298|N/A|0.0181580|0.0000000|50.627|0|N/A|N/A|1|0/1|
|CALIBRATION|02|A|1|18864.505|N/A|34168.283|N/A|0.0111876|0.0000000|45.089|0|66.667|N/A|0|0/0|
|CALIBRATION|02|B|1|7728.117|N/A|32833.801|N/A|0.0129104|0.0000000|45.711|0|100.000|N/A|1|0/0|
|CALIBRATION|04|A|2|11995.161|100.000|11995.351|N/A|0.0048508|0.0000000|44.696|0|N/A|N/A|0|0/0|
|CALIBRATION|04|B|2|22403.604|100.000|22403.870|N/A|0.0059852|0.0000000|43.434|0|N/A|N/A|0|0/0|
|CALIBRATION|05|A|1|3986.666|N/A|11681.982|N/A|0.0051268|0.0197348|53.166|0|N/A|100.000|1|0/1|
|CALIBRATION|05|B|1|4110.955|N/A|15357.318|N/A|0.0161392|0.0141280|55.640|0|N/A|100.000|0|0/0|
|CALIBRATION|05|C|1|4268.974|N/A|4434.536|N/A|0.0016632|0.0169288|49.742|0|N/A|100.000|1|0/2|

QA01 preserves wrong-answer time; contract/completion failures and actual process exits are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 0. All raw traces and every billed/unknown call remain retained.
