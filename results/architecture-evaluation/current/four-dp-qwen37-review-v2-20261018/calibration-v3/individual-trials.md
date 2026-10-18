# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP04 A · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|NOT_OBSERVED (0/1)|NOT_OBSERVED (0/1)|0.0032836|0.0000000|2|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v3/raw/runs/calibration_v3-dp04-free-region-A-r1-25c0b8/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|
|qa03_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|

실제 실패 callback의 입력→실패 시간: n=1, median=14427.258, min=14427.258, max=14427.258. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|18134.287 (1/1)|18134.432 (1/1)|0.0025896|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/calibration-v3/raw/runs/calibration_v3-dp04-free-region-B-r1-f0906a/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|1|18134.287|18134.287|18134.287|
|qa03_mean_ms|1|18134.432|18134.432|18134.432|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.
