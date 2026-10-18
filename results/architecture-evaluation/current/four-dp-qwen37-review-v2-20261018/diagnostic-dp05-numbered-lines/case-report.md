# 사례별 실제 측정값

단위 ms, 비용 USD. 평균은 관측된 callback/완료에 조건부입니다. INCOMPLETE를 빠른 승자로 해석하지 않습니다. DP05는 API-boundary receiver shim 내부 구간을 제외한 값과 전체 경과를 함께 표시합니다.

|DP|안|사례|반복|첫 응답 ms (관측/예정)|VIA 완료 ms (관측/예정)|전체 경과 ms|수신자 제외 ms|대상 집합 정답/예정|VIA $|수신자 $|호출|상태|
|---|---|---|---:|---|---|---:|---:|---:|---:|---:|---:|---|
|05|A|dp05-e4|3|4917.844 (6/6)|14363.581 (6/6)|42089.590|27726.008|0/0|0.0158160|0.0271924|26|COMPLETE|
|05|B|dp05-e4|3|5504.059 (6/6)|13255.524 (6/6)|30555.083|17299.559|0/0|0.1174348|0.0173556|36|COMPLETE|
|05|C|dp05-e4|3|5191.650 (6/6)|5243.135 (6/6)|27538.359|22295.224|0/0|0.0039904|0.0233524|18|COMPLETE|

각 3회 개별 값과 raw 경로는 case-summary.json, episode-metrics.csv, response-endpoints.csv, goal-endpoints.csv에 보존합니다. stage-calls.csv는 실제 call/token/cost와 bridge admission+직렬화/기록+slot 대기를 합친 enqueue→send 구간을 표시합니다. 순수 slot 대기는 별도 timestamp가 없어 분리하지 않습니다. policy-waits.csv는 실제 Rust step 대기를 따로 보존하고 preparation.csv는 요청 전 실제 생산 비용/시간을 보존합니다. memory-breakdown.csv는 같은 시각 Python/Rust RSS와 같은 실행 초기값/차이, 논리 버퍼 분류를 표시합니다. retained_evidence는 실험 기록 보관 비용이며 설계 상태와 분리합니다. 논리 직렬화 바이트는 RSS 세부 할당 측정이 아니며 더해서 RSS를 설명할 수 없습니다. 50 ms는 명목 polling이며 실제 표본 간격도 별도 기록했습니다.
