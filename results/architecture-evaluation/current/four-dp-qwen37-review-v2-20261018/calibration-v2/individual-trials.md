# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP01 A · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7318.102 (2/2)|20698.114 (2/2)|0.0047536|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp01-e3-A-r1-34a104/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|7318.102|7318.102|7318.102|
|qa03_mean_ms|1|20698.114|20698.114|20698.114|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 B · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7266.164 (2/2)|22863.390 (2/2)|0.0051560|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp01-e3-B-r1-53f9f9/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|7266.164|7266.164|7266.164|
|qa03_mean_ms|1|22863.390|22863.390|22863.390|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 C · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|17954.830 (2/2)|17955.360 (2/2)|0.0119304|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp01-e3-C-r1-71e558/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|17954.830|17954.830|17954.830|
|qa03_mean_ms|1|17955.360|17955.360|17955.360|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 A · 02-E4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|8846.014 (4/4)|19444.265 (2/2)|0.0075836|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-02-E4-A-r1-8702bc/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|8846.014|8846.014|8846.014|
|qa03_mean_ms|1|19444.265|19444.265|19444.265|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 B · 02-E4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4499.268 (4/4)|16991.491 (2/2)|0.0099552|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-02-E4-B-r1-ba5e95/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|4499.268|4499.268|4499.268|
|qa03_mean_ms|1|16991.491|16991.491|16991.491|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|NOT_OBSERVED (0/1)|NOT_OBSERVED (0/1)|0.0034632|0.0000000|2|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp04-dense-A-r1-cae917/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|
|qa03_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|

실제 실패 callback의 입력→실패 시간: n=1, median=17214.893, min=17214.893, max=17214.893. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|NOT_OBSERVED (0/1)|NOT_OBSERVED (0/1)|0.0030876|0.0000000|2|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp04-free-region-A-r1-a459cb/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|
|qa03_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|

실제 실패 callback의 입력→실패 시간: n=1, median=13820.910, min=13820.910, max=13820.910. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|27165.226 (1/1)|27165.567 (1/1)|0.0034484|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp04-dense-B-r1-b96b45/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|27165.226|27165.226|27165.226|
|qa03_mean_ms|1|27165.567|27165.567|27165.567|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|17959.305 (1/1)|17959.621 (1/1)|0.0025368|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp04-free-region-B-r1-b085db/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|17959.305|17959.305|17959.305|
|qa03_mean_ms|1|17959.621|17959.621|17959.621|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4440.677 (3/3)|13293.903 (3/3)|0.0052788|0.0082544|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp05-e3-A-r1-054d32/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|4440.677|4440.677|4440.677|
|qa03_mean_ms|1|13293.903|13293.903|13293.903|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4518.747 (3/3)|15191.961 (3/3)|0.0155812|0.0165744|15|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp05-e3-B-r1-d2b9b2/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|4518.747|4518.747|4518.747|
|qa03_mean_ms|1|15191.961|15191.961|15191.961|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4084.448 (3/3)|4117.102 (3/3)|0.0016296|0.0095180|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v2/raw/runs/calibration_v2-dp05-e3-C-r1-b3f83e/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|4084.448|4084.448|4084.448|
|qa03_mean_ms|1|4117.102|4117.102|4117.102|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.
