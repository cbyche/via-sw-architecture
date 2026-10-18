# 이번 PoC 조건의 기본안별 측정 결과

Actual Rust alternatives and Qwen3.7-Plus calls. Voice OFF; DISPLAY_PROXY. Authored finite fixtures.

|Phase|DP|Variant|N|QA01 ms|QA02 %|QA03 ms|QA04 modules|VIA $|Receiver $|QA06 RSS MB|RSS incomplete trials|N-R %|N-H %|Runtime errors|Missing response / completion|
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|BASIC|01|A|12|7226.126|100.000|7226.173|1.500|0.0368480|0.0000000|39.240|0|N/A|N/A|0|0/0|
|BASIC|01|B|12|7589.081|100.000|7589.136|1.500|0.0355520|0.0000000|39.174|0|N/A|N/A|0|0/0|
|BASIC|01|C|12|6957.523|88.889|6957.586|1.500|0.1224032|0.0000000|39.977|0|N/A|N/A|0|0/0|
|BASIC|02|A|12|11791.961|N/A|11792.043|1.500|0.0528276|0.0000000|39.092|0|77.778|N/A|0|0/0|
|BASIC|02|B|12|9383.159|N/A|9383.291|1.500|0.0535072|0.0000000|39.191|0|77.778|N/A|0|6/6|
|BASIC|04|A|12|5710.382|62.500|5710.427|1.500|0.0155436|0.0000000|38.928|0|N/A|N/A|0|0/0|
|BASIC|04|B|12|23785.960|62.500|23785.997|1.500|0.0458792|0.0000000|39.633|0|N/A|N/A|0|0/0|
|BASIC|05|A|12|0.019|N/A|8825.955|1.500|0.0256812|0.0298096|39.502|2|N/A|100.000|0|0/0|
|BASIC|05|B|12|0.024|N/A|12545.004|1.500|0.1629320|0.0310880|40.878|0|N/A|100.000|0|0/0|
|BASIC|05|C|12|0.020|N/A|18.246|1.500|0.0000000|0.0383716|39.092|3|N/A|100.000|0|0/0|
|DIAGNOSTIC|04|B4|3|22965.194|33.333|22965.227|N/A|0.0096588|0.0000000|39.944|0|N/A|N/A|0|0/0|
|SMOKE|01|C|1|5422.097|100.000|5422.142|N/A|0.0054772|0.0000000|38.666|0|N/A|N/A|0|0/0|
|SMOKE|02|A|1|4270.843|N/A|4270.945|N/A|0.0027716|0.0000000|37.929|0|0.000|N/A|0|0/0|
|SMOKE|04|A|1|6218.492|0.000|6218.527|N/A|0.0010252|0.0000000|38.355|0|N/A|N/A|0|0/0|
|SMOKE|05|C|1|0.015|N/A|8.800|N/A|0.0000000|0.0017724|38.093|0|N/A|100.000|0|0/0|
|TACTIC|01|A|3|7800.453|100.000|7800.503|N/A|0.0061700|0.0000000|38.371|0|N/A|N/A|0|0/0|
|TACTIC|02|A|3|7553.355|N/A|7553.523|N/A|0.0078872|0.0000000|38.240|0|100.000|N/A|0|0/0|
|TACTIC|04|A4|3|35066.846|0.000|35066.884|N/A|0.0173976|0.0000000|38.765|0|N/A|N/A|2|2/2|
|TACTIC|05|C|3|0.022|N/A|24.372|N/A|0.0000000|0.0181268|39.207|0|N/A|100.000|0|0/0|

QA01 preserves wrong-answer time; errors are separate. QA03 excludes only actual receiver/user wait intervals. QA06 is the maximum simultaneous sampled RSS across repetitions; shared physical pages may be counted twice. Browser UI is outside scope. QA05 includes preparation/background/unused calls within episode windows; access smoke is campaign overhead. No COMMON-28 scaling or product scores.

Basic episode rows: 120. All raw traces and every billed/unknown call remain retained.

## Separate frozen content conditions

DP01 E3 q2: BASIC ABC×3 has9 required-content omissions; A2×3 has3. All36 frozen fact-marker checks are absent in the displayed answers. Source/set identity QA02 and original timing values remain unchanged. GOAL_COMPLETE is a candidate marker, not proof that summary-content conditions were met. Other cases without frozen semantic_checks are NOT_DEFINED, never a pass. See semantic-conditions.md/json for original text and exact-marker limits.

## Final finite-PoC selections

Basic choices:01A /02A /04A /05C (read-capable receiver profile). Adopt02 A1 only for bounded short tool-free independent side questions.01 A2 and04 A4 are not adopted;05 C2 basic application is deferred because the observed overall VIA time did not improve and serving savings were tiny.04 B4 remains diagnostic only. See poc-selections.json for exact reasons and tactic-comparison.md for all individual repetitions. No accepted ADR change.
