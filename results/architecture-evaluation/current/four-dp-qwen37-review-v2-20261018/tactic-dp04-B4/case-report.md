# 사례별 실제 측정값

단위 ms, 비용 USD. 평균은 관측된 callback/완료에 조건부입니다. INCOMPLETE를 빠른 승자로 해석하지 않습니다. DP05는 API-boundary receiver shim 내부 구간을 제외한 값과 전체 경과를 함께 표시합니다.

|DP|안|사례|반복|첫 응답 ms (관측/예정)|VIA 완료 ms (관측/예정)|전체 경과 ms|수신자 제외 ms|대상 집합 정답/예정|VIA $|수신자 $|호출|상태|
|---|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---|
|04|B4|dp04-dense|3|22341.768 (3/3)|22341.959 (3/3)|22341.959|0.000|6/6|0.0108780|0.0000000|6|COMPLETE|
|04|B4|dp04-free-region|3|13367.003 (3/3)|13367.302 (3/3)|13367.302|0.000|3/3|0.0077384|0.0000000|6|COMPLETE|
|04|B4|dp04-page-change|3|15607.886 (3/3)|15608.081 (3/3)|15608.081|0.000|3/3|0.0097828|0.0000000|7|COMPLETE|
|04|B4|dp04-partial-correction|3|10511.288 (6/6)|21023.434 (3/3)|21023.434|0.000|12/12|0.0151800|0.0000000|18|COMPLETE|

각 3회 개별 값과 raw 경로는 case-summary.json, episode-metrics.csv, response-endpoints.csv, goal-endpoints.csv에 보존합니다. stage-calls.csv는 실제 call/token/cost와 bridge admission+직렬화/기록+slot 대기를 합친 enqueue→send 구간을 표시합니다. 순수 slot 대기는 별도 timestamp가 없어 분리하지 않습니다. policy-waits.csv는 실제 Rust step 대기를 따로 보존하고 preparation.csv는 요청 전 실제 생산 비용/시간을 보존합니다. memory-breakdown.csv는 같은 시각 Python/Rust RSS와 같은 실행 초기값/차이, 논리 버퍼 분류를 표시합니다. retained_evidence는 실험 기록 보관 비용이며 설계 상태와 분리합니다. 논리 직렬화 바이트는 RSS 세부 할당 측정이 아니며 더해서 RSS를 설명할 수 없습니다. 50 ms는 명목 polling이며 실제 표본 간격도 별도 기록했습니다.
