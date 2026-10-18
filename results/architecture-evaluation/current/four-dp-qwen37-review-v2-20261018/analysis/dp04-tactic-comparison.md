# DP04 A/A4/B/B4 same-case comparison

Same four cases ×3 each, phase-separated. Provider send→terminal sums are service spans, not QA01/03 or independently identified structural causality. Actual same helper requests do not force identical remote outputs.

|안|상태|QA01 ms 관측/예정|QA03 ms 완료/예정|QA02|calls|USD|RSS MB|
|---|---|---|---|---|---:|---:|---:|
|A|INCOMPLETE|11175.802 10/15|15623.849 9/12|17/24|26|0.0415828|45.219840|
|B|COMPLETE|18344.055 15/15|22930.461 12/12|24/24|38|0.0431172|45.350912|
|A4|COMPLETE|19797.154 15/15|24746.810 12/12|24/24|30|0.0561648|45.121536|
|B4|COMPLETE|14467.847 15/15|18085.194 12/12|24/24|37|0.0435792|46.972928|

|안·stage|calls|input/output tokens|USD|HTTP service 총/평균ms|실제 전송 image 횟수|
|---|---:|---:|---:|---:|---:|
|A dp04.joint_grounding|15|34886/5344|0.0225048|100922.428/6728.162|27|
|A dp04.joint_source_reread|11|31747/3987|0.0190780|76668.445/6969.859|23|
|B dp04.screen_producer|15|29829/10383|0.0285444|181426.811/12095.121|27|
|B dp04.speech_producer|15|10646/2977|0.0090216|66628.059/4441.871|0|
|B dp04.candidate_binding|8|9750/1032|0.0055512|26691.245/3336.406|0|
|A4 dp04.screen_producer|15|29829/10951|0.0294532|191436.550/12762.437|27|
|A4 dp04.joint_grounding|15|44619/5540|0.0267116|105070.510/7004.701|27|
|B4 dp04.speech_producer|15|10635/2982|0.0090252|66755.576/4450.372|0|
|B4 dp04.screen_producer|15|29829/10989|0.0295140|192014.853/12800.990|27|
|B4 dp04.candidate_binding|7|8792/952|0.0050400|24643.475/3520.496|0|

Actual helper request/response equality: {"pairs": 15, "equal_actual_requests": 15, "equal_actual_responses": 8}

B4 actual provider overlap: {"pairs": 15, "positive_pairs": 15, "sum_ms": 66745.257459, "mean_ms": 4449.6838306}

Detailed case rows, raw UUID/file references and immutable trace hashes: dp04-tactic-comparison.json. No new model/candidate execution.
