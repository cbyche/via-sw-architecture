# 발표용 명시적 120회 선택 비교

새 측정 phase가 아니라 원시 선택과 파생 표다. 기존01/02/04 84회와 DP05 전체 교정36회를 사용한다. 원래 DP05 36회·calibration·보완·입력 진단은 모두 보존하고 전체 비용에 포함한다. 개별 실패를 성공 trial로 치환하지 않았다.

QA01은 실제 구체적 callback latency proxy이며 완전한 첫 의미 정답 검증이 아니다. QA03는 동결 host-contract 완료이며 최종 본문 충족과 별도다. 평균은 관측된 endpoint에 조건부이고 누락/오답/내용 조건과 함께 읽는다.

|DP|안|실제source phase|첫 callback ms (관측/예정)|host 완료 proxy ms (관측/예정)|전체wall ms|receiver 제외 ms|지칭 정답/예정|QA04 모듈|VIA USD|receiver USD|calls|동시RSS 최대MB|계약/완료 실패 episode|
|---|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
|01|A|basic-v1|6609.115 (36/36)|13617.335 (36/36)|13617.335|0.000|36/36|1.75|0.0674956|0.0000000|63|44.253|0/12|
|01|B|basic-v1|6266.887 (36/36)|14241.594 (35/36)|14241.594|0.000|36/36|1.75|0.0699160|0.0000000|70|45.318|1/12|
|01|C|basic-v1|11995.458 (36/36)|12364.438 (36/36)|12364.438|0.000|34/36|2.0|0.1630788|0.0000000|71|50.364|0/12|
|02|A|basic-v1|8677.265 (30/30)|24091.786 (23/24)|24091.786|0.000|0/0|2.0|0.1224560|0.0000000|53|45.351|1/12|
|02|B|basic-v1|6698.499 (30/30)|20982.021 (24/24)|20982.021|0.000|0/0|2.0|0.1125156|0.0000000|49|46.170|0/12|
|04|A|basic-v1|11175.802 (10/15)|15623.849 (9/12)|15623.849|0.000|17/24|2.5|0.0415828|0.0000000|26|45.220|5/12|
|04|B|basic-v1|18344.055 (15/15)|22930.461 (12/12)|22930.461|0.000|24/24|2.5|0.0431172|0.0000000|38|45.351|0/12|
|05|A|basic-dp05-reuse-union-corrected-v2|4306.803 (20/21)|11018.836 (20/21)|33025.170|22006.335|0/0|1.5|0.0338936|0.0679472|80|49.119|1/12|
|05|B|basic-dp05-reuse-union-corrected-v2|4421.473 (21/21)|12438.588 (19/21)|31106.113|18667.525|0/0|1.75|0.2091628|0.0795200|116|57.229|2/12|
|05|C|basic-dp05-reuse-union-corrected-v2|4320.514 (21/21)|4329.101 (19/21)|24073.139|19744.037|0/0|1.5|0.0113928|0.0644000|59|48.153|2/12|

고정 예정 분모: QA01 261, QA03 243. Actual observed callback 255, host completion 233.

selection.json contains each actual source phase/input/trace/freeze/binary hash. Case medians/min/max and individual3 values remain in per-phase individual-trials.md and combined case-summary.json. Memory is sampled concurrent Python+Rust only, initial is startup rather than steady baseline, logical bytes are not RSS, and experiment evidence buffers are separate.
