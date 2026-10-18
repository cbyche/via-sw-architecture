# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP05 A · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4589.933 (2/2)|13768.910 (2/2)|0.0047536|0.0129164|10|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-A-r1-5782bb/trace.json)|
|2|5101.452 (2/2)|13990.696 (2/2)|0.0057292|0.0070792|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-A-r2-f822bf/trace.json)|
|3|5062.146 (2/2)|15331.138 (2/2)|0.0053332|0.0071968|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-A-r3-ed39eb/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5062.146|4589.933|5101.452|
|qa03_mean_ms|3|13990.696|13768.910|15331.138|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4933.616 (2/2)|12931.354 (2/2)|0.0380224|0.0060172|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-B-r1-c38950/trace.json)|
|2|6110.357 (2/2)|13211.732 (2/2)|0.0379648|0.0055864|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-B-r2-0af110/trace.json)|
|3|5468.203 (2/2)|13623.485 (2/2)|0.0414476|0.0057520|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-B-r3-180a4e/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5468.203|4933.616|6110.357|
|qa03_mean_ms|3|13211.732|12931.354|13623.485|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|5482.435 (2/2)|5526.532 (2/2)|0.0013696|0.0084432|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-C-r1-57e2ba/trace.json)|
|2|5228.884 (2/2)|5282.511 (2/2)|0.0013648|0.0073984|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-C-r2-771370/trace.json)|
|3|4863.631 (2/2)|4920.363 (2/2)|0.0012560|0.0075108|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/diagnostic-dp05-numbered-lines/raw/runs/diagnostic-dp05-e4-C-r3-dc39c3/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5228.884|4863.631|5482.435|
|qa03_mean_ms|3|5282.511|4920.363|5526.532|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.
