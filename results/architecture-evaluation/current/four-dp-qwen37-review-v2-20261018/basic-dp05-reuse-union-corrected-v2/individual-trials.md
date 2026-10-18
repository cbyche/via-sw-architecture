# 사례별 개별 반복과 기술 통계

단위 ms. NOT_OBSERVED는 실제 callback/완료를 관측하지 못한 값이며 0/timeout으로 대체하지 않습니다. 평균·median/min/max는 관측된 값에 조건부입니다.

## DP05 A · dp05-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|3104.616 (1/1)|3104.740 (1/1)|0.0003780|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-A-r1-8e78f8/trace.json)|
|2|2877.149 (1/1)|2877.284 (1/1)|0.0003892|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-A-r2-c8a27c/trace.json)|
|3|2381.554 (1/1)|2381.674 (1/1)|0.0003588|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-A-r3-d91cd5/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|2877.149|2381.554|3104.616|
|qa03_mean_ms|3|2877.284|2381.674|3104.740|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4458.969 (1/1)|13455.026 (1/1)|0.0017224|0.0022100|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-A-r1-462049/trace.json)|
|2|4820.571 (1/1)|12966.498 (1/1)|0.0017256|0.0060484|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-A-r2-03071d/trace.json)|
|3|4077.019 (1/1)|12232.049 (1/1)|0.0016872|0.0061432|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-A-r3-6c97d3/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4458.969|4077.019|4820.571|
|qa03_mean_ms|3|12966.498|12232.049|13455.026|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4271.945 (3/3)|11376.976 (3/3)|0.0048912|0.0090596|11|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-A-r1-4ba071/trace.json)|
|2|4620.143 (2/3)|12239.514 (2/3)|0.0048148|0.0055436|9|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-A-r2-a33c38/trace.json)|
|3|4267.714 (3/3)|11037.782 (3/3)|0.0048692|0.0086416|11|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-A-r3-0028b1/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4271.945|4267.714|4620.143|
|qa03_mean_ms|3|11376.976|11037.782|12239.514|

실제 실패 callback의 입력→실패 시간: n=1, median=8572.417, min=8572.417, max=8572.417. 성공 완료 시간에 섞지 않았습니다.

## DP05 A · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4845.735 (2/2)|12962.339 (2/2)|0.0043532|0.0101648|10|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-A-r1-0e3a88/trace.json)|
|2|4785.937 (2/2)|12704.663 (2/2)|0.0043764|0.0105088|10|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-A-r2-391937/trace.json)|
|3|5146.790 (2/2)|15151.068 (2/2)|0.0043276|0.0096272|10|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-A-r3-2ae48d/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4845.735|4785.937|5146.790|
|qa03_mean_ms|3|12962.339|12704.663|15151.068|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|2683.814 (1/1)|2683.936 (1/1)|0.0063648|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-B-r1-3fea21/trace.json)|
|2|2967.216 (1/1)|2967.344 (1/1)|0.0059276|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-B-r2-ce285b/trace.json)|
|3|3374.672 (1/1)|3374.759 (1/1)|0.0066260|0.0000000|4|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-B-r3-a99a24/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|2967.216|2683.814|3374.672|
|qa03_mean_ms|3|2967.344|2683.936|3374.759|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4341.091 (1/1)|14831.571 (1/1)|0.0113112|0.0023384|7|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-B-r1-8655ef/trace.json)|
|2|3770.462 (1/1)|14812.733 (1/1)|0.0110568|0.0025472|7|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-B-r2-b8bd03/trace.json)|
|3|4090.061 (1/1)|14864.967 (1/1)|0.0113124|0.0024452|7|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-B-r3-590d54/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4090.061|3770.462|4341.091|
|qa03_mean_ms|3|14831.571|14812.733|14864.967|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4443.013 (3/3)|15554.857 (3/3)|0.0167012|0.0091952|13|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-B-r1-211c9c/trace.json)|
|2|4360.037 (3/3)|14022.574 (3/3)|0.0161300|0.0102048|14|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-B-r2-6cc27c/trace.json)|
|3|4523.310 (3/3)|14612.517 (3/3)|0.0160392|0.0100808|13|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-B-r3-15540f/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4443.013|4360.037|4523.310|
|qa03_mean_ms|3|14612.517|14022.574|15554.857|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 B · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|5256.736 (2/2)|12094.524 (1/2)|0.0369704|0.0124152|14|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-B-r1-3747bb/trace.json)|
|2|5229.683 (2/2)|12825.853 (1/2)|0.0378772|0.0187940|15|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-B-r2-8c2cfd/trace.json)|
|3|5335.850 (2/2)|12653.818 (2/2)|0.0328460|0.0114992|14|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-B-r3-934bac/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|5256.736|5229.683|5335.850|
|qa03_mean_ms|3|12653.818|12094.524|12825.853|

실제 실패 callback의 입력→실패 시간: n=2, median=73044.455, min=63337.799, max=82751.110. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e1

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|3225.748 (1/1)|3225.858 (1/1)|0.0003780|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-C-r1-5c0613/trace.json)|
|2|2940.012 (1/1)|2940.073 (1/1)|0.0003684|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-C-r2-f9c633/trace.json)|
|3|3215.522 (1/1)|3215.636 (1/1)|0.0003796|0.0000000|1|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e1-C-r3-4e8eb3/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|3215.522|2940.012|3225.748|
|qa03_mean_ms|3|3215.636|2940.073|3225.858|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e2

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4387.007 (1/1)|4419.964 (1/1)|0.0005468|0.0026660|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-C-r1-084da4/trace.json)|
|2|3828.498 (1/1)|3860.919 (1/1)|0.0005340|0.0029388|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-C-r2-a03698/trace.json)|
|3|5092.721 (1/1)|5129.376 (1/1)|0.0005468|0.0029028|3|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e2-C-r3-714612/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4387.007|3828.498|5092.721|
|qa03_mean_ms|3|4419.964|3860.919|5129.376|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e3

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4460.099 (3/3)|4506.834 (3/3)|0.0016728|0.0099444|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-C-r1-1b13b7/trace.json)|
|2|4088.186 (3/3)|4137.218 (3/3)|0.0016184|0.0101700|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-C-r2-788e21/trace.json)|
|3|4213.762 (3/3)|4262.081 (3/3)|0.0015960|0.0098636|9|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e3-C-r3-791797/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4213.762|4088.186|4460.099|
|qa03_mean_ms|3|4262.081|4137.218|4506.834|

실제 실패 callback의 입력→실패 시간: n=0, median=NOT_OBSERVED, min=NOT_OBSERVED, max=NOT_OBSERVED. 성공 완료 시간에 섞지 않았습니다.

## DP05 C · dp05-e4

|반복|QA01 평균 (관측/예정)|QA03 평균 (관측/예정)|VIA $|수신자 $|호출|계약/완료 실패|원시 trace|
|---:|---|---|---:|---:|---:|---|---|
|1|4905.732 (2/2)|5079.423 (1/2)|0.0012336|0.0102052|7|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-C-r1-12f21f/trace.json)|
|2|4950.636 (2/2)|5531.554 (1/2)|0.0012752|0.0090544|7|True|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-C-r2-b0009c/trace.json)|
|3|5021.207 (2/2)|5065.861 (2/2)|0.0012432|0.0066548|6|False|[trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-union-corrected-v2/raw/runs/basic_corrected-dp05-e4-C-r3-48f5ab/trace.json)|

|반복별 평균의 분포|관측 반복 수|median|min|max|
|---|---:|---:|---:|---:|
|qa01_mean_ms|3|4950.636|4905.732|5021.207|
|qa03_mean_ms|3|5079.423|5065.861|5531.554|

실제 실패 callback의 입력→실패 시간: n=2, median=39072.402, min=38173.409, max=39971.395. 성공 완료 시간에 섞지 않았습니다.
