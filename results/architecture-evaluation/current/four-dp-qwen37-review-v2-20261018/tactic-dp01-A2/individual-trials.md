# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP01 A · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6153.474 (2/2)|22208.198 (2/2)|0.0050524|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp01-A2/raw/runs/tactic-dp01-e3-A-r1-f6e6e0/trace.json)|
|2|6249.696 (2/2)|20638.623 (2/2)|0.0047084|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp01-A2/raw/runs/tactic-dp01-e3-A-r2-743ec3/trace.json)|
|3|6779.220 (2/2)|21451.159 (2/2)|0.0048252|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/tactic-dp01-A2/raw/runs/tactic-dp01-e3-A-r3-ecc5c9/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|6249.696|6153.474|6779.220|
|qa03_mean_ms|3|21451.159|20638.623|22208.198|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.
