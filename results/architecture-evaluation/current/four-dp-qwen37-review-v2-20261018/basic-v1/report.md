# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|계약/완료 실패 episode|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|BASIC|01|A|12|6609.115|100.000|13617.335|1.750|0.0674956|0.0000000|44.253|0|N/A|N/A|0|0/0|
|BASIC|01|B|12|6266.887|100.000|14241.594|1.750|0.0699160|0.0000000|45.318|0|N/A|N/A|1|0/1|
|BASIC|01|C|12|11995.458|94.444|12364.438|2.000|0.1630788|0.0000000|50.364|0|N/A|N/A|0|0/0|
|BASIC|02|A|12|8677.265|N/A|24091.786|2.000|0.1224560|0.0000000|45.351|0|100.000|N/A|1|0/1|
|BASIC|02|B|12|6698.499|N/A|20982.021|2.000|0.1125156|0.0000000|46.170|0|96.296|N/A|0|0/0|
|BASIC|04|A|12|11175.802|70.833|15623.849|2.500|0.0415828|0.0000000|45.220|0|N/A|N/A|5|5/3|
|BASIC|04|B|12|18344.055|100.000|22930.461|2.500|0.0431172|0.0000000|45.351|0|N/A|N/A|0|0/0|
|BASIC|05|A|12|4298.005|N/A|11150.302|1.500|0.0359644|0.0982780|52.101|0|N/A|100.000|3|0/4|
|BASIC|05|B|12|4426.724|N/A|13183.894|1.750|0.2240744|0.0781776|59.130|0|N/A|100.000|0|0/0|
|BASIC|05|C|12|4459.561|N/A|4426.841|1.500|0.0115320|0.0626724|48.038|0|N/A|100.000|2|0/2|

QA01 preserves wrong-answer time; contract/completion failures and actual process exits are separate. QA03 uses fixed user-goal mappings; receiver internal intervals are explicitly scoped shim proxies. Full elapsed accompanies them. INCOMPLETE conditional means are not winner evidence. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 120. All raw traces and every billed/unknown call remain retained.
