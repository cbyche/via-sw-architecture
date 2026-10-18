# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP01 A · dp01-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7223.136 (3/3)|10022.233 (3/3)|0.0040652|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-A-r1-fa7d4c/trace.json)|
|2|7047.968 (3/3)|9314.064 (3/3)|0.0037156|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-A-r2-5805b3/trace.json)|
|3|7119.777 (3/3)|9339.898 (3/3)|0.0037076|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-A-r3-59a3a7/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|7119.777|7047.968|7223.136|
|qa03_mean_ms|3|9339.898|9314.064|10022.233|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 A · dp01-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|5263.967 (4/4)|11095.107 (4/4)|0.0080608|0.0000000|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-A-r1-ce936c/trace.json)|
|2|5573.715 (4/4)|11507.416 (4/4)|0.0080720|0.0000000|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-A-r2-a11626/trace.json)|
|3|5460.558 (4/4)|11396.909 (4/4)|0.0081160|0.0000000|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-A-r3-7b9601/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5460.558|5263.967|5573.715|
|qa03_mean_ms|3|11396.909|11095.107|11507.416|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 A · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6876.805 (2/2)|21324.252 (2/2)|0.0048776|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-A-r1-1f39fd/trace.json)|
|2|6786.006 (2/2)|22760.981 (2/2)|0.0051000|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-A-r2-3d7648/trace.json)|
|3|7002.126 (2/2)|21380.834 (2/2)|0.0048648|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-A-r3-ff0b17/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|6876.805|6786.006|7002.126|
|qa03_mean_ms|3|21380.834|21324.252|22760.981|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 A · dp01-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7348.575 (3/3)|15337.992 (3/3)|0.0056408|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-A-r1-3fe011/trace.json)|
|2|7525.941 (3/3)|15200.760 (3/3)|0.0056544|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-A-r2-478ea2/trace.json)|
|3|7536.373 (3/3)|15216.450 (3/3)|0.0056208|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-A-r3-b47f0a/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|7525.941|7348.575|7536.373|
|qa03_mean_ms|3|15216.450|15200.760|15337.992|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 B · dp01-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6210.440 (3/3)|11612.429 (2/3)|0.0058488|0.0000000|7|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-B-r1-f16625/trace.json)|
|2|5794.699 (3/3)|11966.494 (3/3)|0.0050072|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-B-r2-2c1ad8/trace.json)|
|3|5962.024 (3/3)|12034.999 (3/3)|0.0051000|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-B-r3-f3f60c/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5962.024|5794.699|6210.440|
|qa03_mean_ms|3|11966.494|11612.429|12034.999|

실제 실패 callback의 입력→실패 시간: n=1, median=21053.637, min=21053.637, max=21053.637. 성공 완료 시간에 섞지 않았습니다.

## DP01 B · dp01-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|5298.838 (4/4)|10911.472 (4/4)|0.0073872|0.0000000|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-B-r1-862d85/trace.json)|
|2|5330.132 (4/4)|11336.386 (4/4)|0.0073664|0.0000000|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-B-r2-f8bb75/trace.json)|
|3|5310.489 (4/4)|11141.373 (4/4)|0.0074256|0.0000000|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-B-r3-923631/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5310.489|5298.838|5330.132|
|qa03_mean_ms|3|11141.373|10911.472|11336.386|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 B · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7007.987 (2/2)|22062.543 (2/2)|0.0048584|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-B-r1-1599b2/trace.json)|
|2|6836.653 (2/2)|21909.145 (2/2)|0.0048248|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-B-r2-72eabe/trace.json)|
|3|7048.377 (2/2)|21488.016 (2/2)|0.0048472|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-B-r3-d8e938/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|7007.987|6836.653|7048.377|
|qa03_mean_ms|3|21909.145|21488.016|22062.543|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 B · dp01-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7340.385 (3/3)|14655.439 (3/3)|0.0053404|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-B-r1-184e18/trace.json)|
|2|7200.058 (3/3)|16338.103 (3/3)|0.0062816|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-B-r2-d3c20a/trace.json)|
|3|7513.744 (3/3)|15256.499 (3/3)|0.0056284|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-B-r3-36a1f0/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|7340.385|7200.058|7513.744|
|qa03_mean_ms|3|15256.499|14655.439|16338.103|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 C · dp01-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|8983.424 (3/3)|8983.957 (3/3)|0.0085752|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-C-r1-a29c36/trace.json)|
|2|9891.441 (3/3)|9891.954 (3/3)|0.0089384|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-C-r2-0e132a/trace.json)|
|3|9032.146 (3/3)|9032.683 (3/3)|0.0085820|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e1-C-r3-8473ab/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|9032.146|8983.424|9891.441|
|qa03_mean_ms|3|9032.683|8983.957|9891.954|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 C · dp01-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|8358.217 (4/4)|8358.905 (4/4)|0.0193252|0.0000000|8|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-C-r1-1f1019/trace.json)|
|2|8508.714 (4/4)|10169.220 (4/4)|0.0204068|0.0000000|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-C-r2-1c6a08/trace.json)|
|3|8027.527 (4/4)|9682.698 (4/4)|0.0203732|0.0000000|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e2-C-r3-180566/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|8358.217|8027.527|8508.714|
|qa03_mean_ms|3|9682.698|8358.905|10169.220|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 C · dp01-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|18497.782 (2/2)|18498.568 (2/2)|0.0104584|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-C-r1-fa7160/trace.json)|
|2|18166.950 (2/2)|18168.014 (2/2)|0.0127060|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-C-r2-11ac89/trace.json)|
|3|18400.389 (2/2)|18401.223 (2/2)|0.0122512|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e3-C-r3-fed7ee/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|18400.389|18166.950|18497.782|
|qa03_mean_ms|3|18401.223|18168.014|18498.568|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP01 C · dp01-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|16784.336 (3/3)|16785.296 (3/3)|0.0141412|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-C-r1-67c6c1/trace.json)|
|2|15598.888 (3/3)|15599.773 (3/3)|0.0137180|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-C-r2-af3f67/trace.json)|
|3|13752.572 (3/3)|13753.297 (3/3)|0.0136032|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp01-e4-C-r3-8e8dfc/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|15598.888|13752.572|16784.336|
|qa03_mean_ms|3|15599.773|13753.297|16785.296|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 A · 02-E1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6715.601 (2/2)|20365.143 (2/2)|0.0097168|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E1-A-r1-9487db/trace.json)|
|2|6186.647 (2/2)|26732.063 (2/2)|0.0150872|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E1-A-r2-c9495e/trace.json)|
|3|7294.423 (2/2)|20988.575 (1/2)|0.0143528|0.0000000|6|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E1-A-r3-6fd020/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|6715.601|6186.647|7294.423|
|qa03_mean_ms|3|20988.575|20365.143|26732.063|

실제 실패 callback의 입력→실패 시간: n=1, median=41758.672, min=41758.672, max=41758.672. 성공 완료 시간에 섞지 않았습니다.

## DP02 A · 02-E2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|10252.719 (2/2)|25641.642 (2/2)|0.0094468|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E2-A-r1-f8515a/trace.json)|
|2|10478.351 (2/2)|29675.516 (2/2)|0.0102884|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E2-A-r2-52fa71/trace.json)|
|3|10420.641 (2/2)|28150.679 (2/2)|0.0099040|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E2-A-r3-28046a/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|10420.641|10252.719|10478.351|
|qa03_mean_ms|3|28150.679|25641.642|29675.516|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 A · 02-E3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|10254.652 (2/2)|25410.953 (2/2)|0.0093416|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E3-A-r1-744672/trace.json)|
|2|10090.372 (2/2)|26187.658 (2/2)|0.0096420|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E3-A-r2-622e35/trace.json)|
|3|10891.972 (2/2)|27737.975 (2/2)|0.0097508|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E3-A-r3-438891/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|10254.652|10090.372|10891.972|
|qa03_mean_ms|3|26187.658|25410.953|27737.975|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 A · 02-E4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7439.922 (4/4)|20826.228 (2/2)|0.0099476|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E4-A-r1-51b2f4/trace.json)|
|2|7757.314 (4/4)|17967.446 (2/2)|0.0075228|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E4-A-r2-4a59e0/trace.json)|
|3|8589.561 (4/4)|17865.946 (2/2)|0.0074552|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E4-A-r3-8af895/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|7757.314|7439.922|8589.561|
|qa03_mean_ms|3|17967.446|17865.946|20826.228|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 B · 02-E1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6539.347 (2/2)|22250.467 (2/2)|0.0102920|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E1-B-r1-8f71d6/trace.json)|
|2|8194.669 (2/2)|27823.208 (2/2)|0.0134744|0.0000000|5|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E1-B-r2-463be7/trace.json)|
|3|7501.999 (2/2)|24868.524 (2/2)|0.0108092|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E1-B-r3-b1ac09/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|7501.999|6539.347|8194.669|
|qa03_mean_ms|3|24868.524|22250.467|27823.208|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 B · 02-E2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|6459.846 (2/2)|19991.301 (2/2)|0.0092168|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E2-B-r1-1818ea/trace.json)|
|2|6994.364 (2/2)|21272.074 (2/2)|0.0094336|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E2-B-r2-dde872/trace.json)|
|3|7063.363 (2/2)|22483.845 (2/2)|0.0097932|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E2-B-r3-d9033b/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|6994.364|6459.846|7063.363|
|qa03_mean_ms|3|21272.074|19991.301|22483.845|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 B · 02-E3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|10596.783 (2/2)|27889.583 (2/2)|0.0095328|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E3-B-r1-03dac4/trace.json)|
|2|9441.260 (2/2)|24682.519 (2/2)|0.0090376|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E3-B-r2-5335b0/trace.json)|
|3|10143.531 (2/2)|27565.246 (2/2)|0.0096360|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E3-B-r3-c66e84/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|10143.531|9441.260|10596.783|
|qa03_mean_ms|3|27565.246|24682.519|27889.583|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP02 B · 02-E4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4626.238 (4/4)|11607.365 (2/2)|0.0071928|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E4-B-r1-3647ef/trace.json)|
|2|4570.730 (4/4)|11220.697 (2/2)|0.0071300|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E4-B-r2-42df9d/trace.json)|
|3|4574.192 (4/4)|10129.423 (2/2)|0.0069672|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-02-E4-B-r3-899c52/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4574.192|4570.730|4626.238|
|qa03_mean_ms|3|11220.697|10129.423|11607.365|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|17608.337 (1/1)|17608.598 (1/1)|0.0036708|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-dense-A-r1-ca5202/trace.json)|
|2|17324.893 (1/1)|17325.137 (1/1)|0.0036708|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-dense-A-r2-962c0a/trace.json)|
|3|18432.777 (1/1)|18433.043 (1/1)|0.0036708|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-dense-A-r3-e788aa/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|17608.337|17324.893|18432.777|
|qa03_mean_ms|3|17608.598|17325.137|18433.043|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|NOT_OBSERVED (0/1)|NOT_OBSERVED (0/1)|0.0032528|0.0000000|2|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-free-region-A-r1-4a3fae/trace.json)|
|2|NOT_OBSERVED (0/1)|NOT_OBSERVED (0/1)|0.0032548|0.0000000|2|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-free-region-A-r2-1edfa6/trace.json)|
|3|NOT_OBSERVED (0/1)|NOT_OBSERVED (0/1)|0.0027432|0.0000000|2|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-free-region-A-r3-449511/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|
|qa03_mean_ms|0|NOT_OBSERVED|NOT_OBSERVED|NOT_OBSERVED|

실제 실패 callback의 입력→실패 시간: n=3, median=13586.563, min=10040.297, max=13651.190. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-page-change

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|10064.009 (1/1)|10064.229 (1/1)|0.0035148|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-page-change-A-r1-74dd55/trace.json)|
|2|10128.369 (1/1)|10128.587 (1/1)|0.0035148|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-page-change-A-r2-5827f2/trace.json)|
|3|9842.542 (1/1)|9842.733 (1/1)|0.0035112|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-page-change-A-r3-20beaf/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|10064.009|9842.542|10128.369|
|qa03_mean_ms|3|10064.229|9842.733|10128.587|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 A · dp04-partial-correction

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|7086.780 (2/2)|14174.450 (1/1)|0.0026680|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-partial-correction-A-r1-24e4a1/trace.json)|
|2|7070.837 (1/2)|21326.173 (1/1)|0.0040576|0.0000000|3|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-partial-correction-A-r2-fd1aa8/trace.json)|
|3|7112.693 (1/2)|21711.693 (1/1)|0.0040532|0.0000000|3|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-partial-correction-A-r3-8cc650/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|7086.780|7070.837|7112.693|
|qa03_mean_ms|3|21326.173|14174.450|21711.693|

실제 실패 callback의 입력→실패 시간: n=2, median=14426.671, min=14254.866, max=14598.475. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-dense

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|27372.122 (1/1)|27372.395 (1/1)|0.0036228|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-dense-B-r1-d58fd4/trace.json)|
|2|27017.661 (1/1)|27017.935 (1/1)|0.0036228|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-dense-B-r2-b9ae76/trace.json)|
|3|28351.618 (1/1)|28351.807 (1/1)|0.0036228|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-dense-B-r3-beeedd/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|27372.122|27017.661|28351.618|
|qa03_mean_ms|3|27372.395|27017.935|28351.807|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-free-region

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|16790.447 (1/1)|16790.653 (1/1)|0.0025784|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-free-region-B-r1-4e0794/trace.json)|
|2|17081.922 (1/1)|17082.152 (1/1)|0.0025784|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-free-region-B-r2-3c521f/trace.json)|
|3|16655.341 (1/1)|16655.586 (1/1)|0.0025848|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-free-region-B-r3-815fac/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|16790.447|16655.341|17081.922|
|qa03_mean_ms|3|16790.653|16655.586|17082.152|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-page-change

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|17398.476 (1/1)|17398.692 (1/1)|0.0033868|0.0000000|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-page-change-B-r1-d8bbab/trace.json)|
|2|14680.321 (1/1)|14680.512 (1/1)|0.0026972|0.0000000|2|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-page-change-B-r2-14cc73/trace.json)|
|3|17347.047 (1/1)|17347.233 (1/1)|0.0033868|0.0000000|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-page-change-B-r3-8345aa/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|17347.047|14680.321|17398.476|
|qa03_mean_ms|3|17347.233|14680.512|17398.692|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP04 B · dp04-partial-correction

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|14990.107 (2/2)|29981.230 (1/1)|0.0049632|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-partial-correction-B-r1-6053f3/trace.json)|
|2|15588.873 (2/2)|31178.686 (1/1)|0.0050968|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-partial-correction-B-r2-a7e625/trace.json)|
|3|15653.953 (2/2)|31308.649 (1/1)|0.0049764|0.0000000|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp04-partial-correction-B-r3-205ebb/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|15588.873|14990.107|15653.953|
|qa03_mean_ms|3|31178.686|29981.230|31308.649|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|2841.371 (1/1)|2841.498 (1/1)|0.0003748|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-A-r1-3d7ae8/trace.json)|
|2|2647.973 (1/1)|2648.114 (1/1)|0.0003748|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-A-r2-414e2a/trace.json)|
|3|2746.061 (1/1)|2746.198 (1/1)|0.0003828|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-A-r3-0ab83a/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|2746.061|2647.973|2841.371|
|qa03_mean_ms|3|2746.198|2648.114|2841.498|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|3917.776 (1/1)|12877.362 (1/1)|0.0017432|0.0025372|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-A-r1-81b485/trace.json)|
|2|4172.339 (1/1)|13539.817 (1/1)|0.0017304|0.0059032|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-A-r2-7eccf2/trace.json)|
|3|4370.869 (1/1)|12053.768 (1/1)|0.0016624|0.0024336|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-A-r3-994b49/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4172.339|3917.776|4370.869|
|qa03_mean_ms|3|12877.362|12053.768|13539.817|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4085.815 (3/3)|12184.381 (3/3)|0.0050932|0.0089288|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-A-r1-d41830/trace.json)|
|2|4645.669 (3/3)|13482.372 (3/3)|0.0052188|0.0081800|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-A-r2-ff3f37/trace.json)|
|3|4376.988 (3/3)|13171.072 (3/3)|0.0051492|0.0085600|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-A-r3-e1499c/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4376.988|4085.815|4645.669|
|qa03_mean_ms|3|13171.072|12184.381|13482.372|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4786.163 (2/2)|NOT_OBSERVED (0/2)|0.0047772|0.0265536|13|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-A-r1-a59958/trace.json)|
|2|5209.744 (2/2)|13771.387 (1/2)|0.0046828|0.0167568|11|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-A-r2-289c6d/trace.json)|
|3|5122.245 (2/2)|12563.516 (1/2)|0.0047748|0.0184248|11|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-A-r3-d3457d/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5122.245|4786.163|5209.744|
|qa03_mean_ms|2|13167.451|12563.516|13771.387|

실제 실패 callback의 입력→실패 시간: n=4, median=75364.928, min=69081.453, max=84782.970. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|2809.224 (1/1)|2809.378 (1/1)|0.0062116|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-B-r1-82101c/trace.json)|
|2|2731.809 (1/1)|2731.958 (1/1)|0.0062564|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-B-r2-c2285d/trace.json)|
|3|2703.407 (1/1)|2703.589 (1/1)|0.0065136|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-B-r3-fbee72/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|2731.809|2703.407|2809.224|
|qa03_mean_ms|3|2731.958|2703.589|2809.378|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|3990.380 (1/1)|12570.208 (1/1)|0.0105276|0.0022720|7|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-B-r1-39e3fa/trace.json)|
|2|4602.874 (1/1)|15815.459 (1/1)|0.0108576|0.0075948|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-B-r2-611b96/trace.json)|
|3|4622.316 (1/1)|14347.662 (1/1)|0.0107768|0.0022536|7|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-B-r3-fd80e5/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4602.874|3990.380|4622.316|
|qa03_mean_ms|3|14347.662|12570.208|15815.459|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4492.121 (3/3)|16486.595 (3/3)|0.0154996|0.0127012|15|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-B-r1-85cb46/trace.json)|
|2|4293.680 (3/3)|16472.050 (3/3)|0.0163944|0.0155856|14|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-B-r2-7c457d/trace.json)|
|3|4714.125 (3/3)|15803.268 (3/3)|0.0158776|0.0211552|17|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-B-r3-1e3b0a/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4492.121|4293.680|4714.125|
|qa03_mean_ms|3|16472.050|15803.268|16486.595|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|5222.128 (2/2)|13561.469 (2/2)|0.0459712|0.0048132|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-B-r1-af586a/trace.json)|
|2|5236.634 (2/2)|12549.007 (2/2)|0.0374480|0.0043556|12|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-B-r2-5751d9/trace.json)|
|3|5041.944 (2/2)|13688.412 (2/2)|0.0417400|0.0074464|13|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-B-r3-e772fb/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5222.128|5041.944|5236.634|
|qa03_mean_ms|3|13561.469|12549.007|13688.412|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|2705.405 (1/1)|2705.551 (1/1)|0.0003812|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-C-r1-266c16/trace.json)|
|2|3196.059 (1/1)|3196.192 (1/1)|0.0003780|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-C-r2-5f6871/trace.json)|
|3|3133.715 (1/1)|3133.861 (1/1)|0.0003860|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e1-C-r3-905dcc/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|3133.715|2705.405|3196.059|
|qa03_mean_ms|3|3133.861|2705.551|3196.192|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4660.767 (1/1)|4692.710 (1/1)|0.0005468|0.0028900|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-C-r1-822bf5/trace.json)|
|2|4202.598 (1/1)|4233.620 (1/1)|0.0005468|0.0028052|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-C-r2-ddebc1/trace.json)|
|3|4321.966 (1/1)|4351.370 (1/1)|0.0005468|0.0027012|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e2-C-r3-98124c/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4321.966|4202.598|4660.767|
|qa03_mean_ms|3|4351.370|4233.620|4692.710|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4176.928 (3/3)|4212.571 (3/3)|0.0016376|0.0104548|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-C-r1-15c5b1/trace.json)|
|2|4913.284 (3/3)|4953.567 (3/3)|0.0017160|0.0100268|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-C-r2-7687e2/trace.json)|
|3|4573.168 (3/3)|4616.608 (3/3)|0.0017160|0.0099220|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e3-C-r3-a92a04/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4573.168|4176.928|4913.284|
|qa03_mean_ms|3|4616.608|4212.571|4953.567|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|5336.721 (2/2)|5189.340 (1/2)|0.0012352|0.0089008|7|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-C-r1-9c2ddb/trace.json)|
|2|5184.329 (2/2)|5214.066 (2/2)|0.0012368|0.0060848|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-C-r2-740cee/trace.json)|
|3|4699.012 (2/2)|4830.964 (1/2)|0.0012048|0.0088868|7|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-v1/raw/runs/basic-dp05-e4-C-r3-668933/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5184.329|4699.012|5336.721|
|qa03_mean_ms|3|5189.340|4830.964|5214.066|

실제 실패 callback의 입력→실패 시간: n=2, median=39763.836, min=39082.559, max=40445.114. 성공 완료 시간에 섞지 않았습니다.
