# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP04 B4 · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|22439.683 (1/1)|22439.883 (1/1)|0.0036292|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-dense-B4-r1-273f94/trace.json)|
|2|22232.243 (1/1)|22232.428 (1/1)|0.0036228|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-dense-B4-r2-d7444f/trace.json)|
|3|22353.379 (1/1)|22353.566 (1/1)|0.0036260|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-dense-B4-r3-eb330b/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|22353.379|22232.243|22439.683|
|qa03_mean_ms|3|22353.566|22232.428|22439.883|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B4 · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|13304.585 (1/1)|13304.800 (1/1)|0.0025800|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-free-region-B4-r1-a26ef2/trace.json)|
|2|13167.430 (1/1)|13167.627 (1/1)|0.0025800|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-free-region-B4-r2-cbfc11/trace.json)|
|3|13628.992 (1/1)|13629.478 (1/1)|0.0025784|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-free-region-B4-r3-008a36/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|13304.585|13167.430|13628.992|
|qa03_mean_ms|3|13304.800|13167.627|13629.478|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B4 · dp04-page-change

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|20652.214 (1/1)|20652.411 (1/1)|0.0040924|0.0000000|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-page-change-B4-r1-4d98b7/trace.json)|
|2|13465.780 (1/1)|13465.972 (1/1)|0.0029660|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-page-change-B4-r2-509168/trace.json)|
|3|12705.665 (1/1)|12705.860 (1/1)|0.0027244|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-page-change-B4-r3-6f0fe3/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|13465.780|12705.665|20652.214|
|qa03_mean_ms|3|13465.972|12705.860|20652.411|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B4 · dp04-partial-correction

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|10750.019 (2/2)|21500.930 (1/1)|0.0050848|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-partial-correction-B4-r1-5cf39d/trace.json)|
|2|10698.719 (2/2)|21398.380 (1/1)|0.0051120|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-partial-correction-B4-r2-7becde/trace.json)|
|3|10085.125 (2/2)|20170.991 (1/1)|0.0049832|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp04-B4/raw/runs/diagnostic-dp04-partial-correction-B4-r3-7c4df7/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|10698.719|10085.125|10750.019|
|qa03_mean_ms|3|21398.380|20170.991|21500.930|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.
