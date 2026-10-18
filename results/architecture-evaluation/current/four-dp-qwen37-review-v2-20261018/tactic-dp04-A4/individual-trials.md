# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP04 A4 · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|32162.952 (1/1)|32163.287 (1/1)|0.0051368|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-dense-A4-r1-a024a6/trace.json)|
|2|30945.574 (1/1)|30945.833 (1/1)|0.0051336|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-dense-A4-r2-d1e00a/trace.json)|
|3|31007.811 (1/1)|31008.070 (1/1)|0.0051368|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-dense-A4-r3-12de75/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|31007.811|30945.574|32162.952|
|qa03_mean_ms|3|31008.070|30945.833|32163.287|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A4 · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|20313.285 (1/1)|20313.440 (1/1)|0.0039136|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-free-region-A4-r1-d3d75a/trace.json)|
|2|20245.803 (1/1)|20246.009 (1/1)|0.0039136|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-free-region-A4-r2-46ca2c/trace.json)|
|3|20891.368 (1/1)|20891.589 (1/1)|0.0039136|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-free-region-A4-r3-8b6119/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|20313.285|20245.803|20891.368|
|qa03_mean_ms|3|20313.440|20246.009|20891.589|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A4 · dp04-page-change

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|19724.964 (1/1)|19725.110 (1/1)|0.0045652|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-page-change-A4-r1-97f253/trace.json)|
|2|20365.904 (1/1)|20366.089 (1/1)|0.0046240|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-page-change-A4-r2-356f2a/trace.json)|
|3|16333.147 (1/1)|16333.326 (1/1)|0.0041552|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-page-change-A4-r3-3d6fd5/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|19724.964|16333.147|20365.904|
|qa03_mean_ms|3|19725.110|16333.326|20366.089|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A4 · dp04-partial-correction

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|13749.594 (2/2)|27500.090 (1/1)|0.0051396|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-partial-correction-A4-r1-15ab6a/trace.json)|
|2|14275.987 (2/2)|28552.691 (1/1)|0.0052652|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-partial-correction-A4-r2-f21712/trace.json)|
|3|14457.669 (2/2)|28916.190 (1/1)|0.0052676|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-A4/raw/runs/tactic-dp04-partial-correction-A4-r3-bdd885/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|14275.987|13749.594|14457.669|
|qa03_mean_ms|3|28552.691|27500.090|28916.190|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.
