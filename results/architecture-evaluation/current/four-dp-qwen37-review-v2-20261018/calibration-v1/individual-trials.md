# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP01 A · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6579.120 (2/2)|23229.091 (1/2)|0.0070008|0.0000000|5|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp01-e3-A-r1-d50640/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|6579.120|6579.120|6579.120|
|qa03_mean_ms|1|23229.091|23229.091|23229.091|

실제 실패 callback의 입력→실패 시간: n=1, median=36896.316, min=36896.316, max=36896.316. 성공 완료 시간에 섞지 않았습니다.

## DP01 B · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6650.857 (2/2)|21586.553 (2/2)|0.0047528|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp01-e3-B-r1-060335/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|6650.857|6650.857|6650.857|
|qa03_mean_ms|1|21586.553|21586.553|21586.553|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 C · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|18536.097 (2/2)|33674.298 (1/2)|0.0181580|0.0000000|8|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp01-e3-C-r1-35906a/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|18536.097|18536.097|18536.097|
|qa03_mean_ms|1|33674.298|33674.298|33674.298|

실제 실패 callback의 입력→실패 시간: n=1, median=49019.224, min=49019.224, max=49019.224. 성공 완료 시간에 섞지 않았습니다.

## DP02 A · 02-E4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|18864.505 (4/4)|34168.283 (2/2)|0.0111876|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-02-E4-A-r1-16fe21/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|18864.505|18864.505|18864.505|
|qa03_mean_ms|1|34168.283|34168.283|34168.283|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 B · 02-E4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7728.117 (4/4)|32833.801 (2/2)|0.0129104|0.0000000|6|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-02-E4-B-r1-d36439/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|7728.117|7728.117|7728.117|
|qa03_mean_ms|1|32833.801|32833.801|32833.801|

실제 실패 callback의 입력→실패 시간: n=1, median=36629.560, min=36629.560, max=36629.560. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|16999.474 (1/1)|16999.705 (1/1)|0.0033756|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp04-dense-A-r1-3fc9ee/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|16999.474|16999.474|16999.474|
|qa03_mean_ms|1|16999.705|16999.705|16999.705|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6990.847 (1/1)|6990.997 (1/1)|0.0014752|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp04-free-region-A-r1-2578f0/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|6990.847|6990.847|6990.847|
|qa03_mean_ms|1|6990.997|6990.997|6990.997|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|26804.483 (1/1)|26804.788 (1/1)|0.0034628|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp04-dense-B-r1-84387e/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|26804.483|26804.483|26804.483|
|qa03_mean_ms|1|26804.788|26804.788|26804.788|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|18002.725 (1/1)|18002.951 (1/1)|0.0025224|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp04-free-region-B-r1-a7f91e/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|18002.725|18002.725|18002.725|
|qa03_mean_ms|1|18002.951|18002.951|18002.951|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|3986.666 (3/3)|11681.982 (2/3)|0.0051268|0.0197348|17|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp05-e3-A-r1-314e96/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|3986.666|3986.666|3986.666|
|qa03_mean_ms|1|11681.982|11681.982|11681.982|

실제 실패 callback의 입력→실패 시간: n=1, median=84302.006, min=84302.006, max=84302.006. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4110.955 (3/3)|15357.318 (3/3)|0.0161392|0.0141280|15|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp05-e3-B-r1-bc9e24/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|4110.955|4110.955|4110.955|
|qa03_mean_ms|1|15357.318|15357.318|15357.318|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4268.974 (3/3)|4434.536 (1/3)|0.0016632|0.0169288|11|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v1/raw/runs/calibration-dp05-e3-C-r1-dd30ed/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|4268.974|4268.974|4268.974|
|qa03_mean_ms|1|4434.536|4434.536|4434.536|

실제 실패 callback의 입력→실패 시간: n=2, median=59243.595, min=54435.875, max=64051.315. 성공 완료 시간에 섞지 않았습니다.
