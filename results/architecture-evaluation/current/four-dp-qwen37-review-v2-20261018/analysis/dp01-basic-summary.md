# DP01 BASIC 36개 전체 관측

고정 분모는 안별 QA01/QA03/QA02 각각 36입니다. 실제 첫 구체 안내와 상세출력 계약 terminal을 측정했으며, 사실 정답은 별도입니다.

|안|QA01 ms (관측)|QA02 정확집합|QA03 ms (관측)|QA04 모듈|VIA USD|호출|RSS peak MB|
|---|---:|---:|---:|---:|---:|---:|---:|
|A|6609.115 (36/36)|36/36|13617.335 (36/36)|1.75|0.0674956|63|44.253184|
|B|6266.887 (36/36)|36/36|14241.594 (35/36)|1.75|0.0699160|70|45.318144|
|C|11995.458 (36/36)|34/36|12364.438 (36/36)|2.00|0.1630788|71|50.364416|

B의 QA03은35/36개 관측에 조건부인 INCOMPLETE 값입니다. E1 q2 rep1에서 원문 2/3행을 3/4행으로 인용해 제한된 복구 후 실패했습니다. 실제 입력→실패 callback은 21053.636917ms이며 성공 완료시간에 넣지 않았습니다.

C의 정확집합 오류 2건은 E4 q3 rep1 / E4 q2 rep2에서 old-research/old-price 두 원본에 old-explanation을 추가 채택한 것입니다. 실제 first-adopt set과 오답 시간은 그대로입니다.

## 실제 경로

|안|기확보 근거 직접완료|원본 보완경로|SOURCE_READ 사건|사전 생산호출|사전 생산 USD|사전 생산 ms|
|---|---:|---:|---:|---:|---:|---:|
|A|9|27|39|0|0.0000000|0.000|
|B|3|33|46|0|0.0000000|0.000|
|C|34|2|68|33|0.0729348|677040.875|

C는34/36개 요청에서 select/adopt call이 이미 상세답을 완성해 progress/final callback이 이어집니다. QA01≈QA03은 자연스러운 directcomplete 경로입니다. 33회 생산 호출의 677040.875 ms는 요청 전에 완료됐으며 요청과 겹친 무료작업이 아닙니다. Source활동→다음 응답 진단의 겹치는 구간들을 합산하지 않습니다.

## 같은 시각 메모리

|안|peak Python MB|peak Rust MB|동일run 초기 동시MB|기록보관 논리bytes|Rust 직렬화 논리bytes|
|---|---:|---:|---:|---:|---:|
|A|41.074688|3.178496|34.668544|30326|5526|
|B|42.024960|3.293184|34.455552|51157|5832|
|C|46.661632|3.702784|34.570240|129475|23867|

안별 12 trial의 최대 동시표본입니다. 역할별 최대를 따로 더하지 않았고, 관측 불완전 trial은 각 안 0/12입니다. Python에는 HTTP/직렬화와 실험용events/calls 기록 보관이 함께 포함됩니다. 논리 bytes는 중복 가능한 직렬화 크기이며 RSS 할당 분해가 아닙니다. C의 더 큰 peak를 단일 설계 상태나 로그만의 효과로 분리할 수 없습니다. shared page 중복 계수는 미확인이고 cloud/UI는 제외됩니다. 명목 50 ms와 실제 gap은 별도입니다.

## 내용과 재현

Literal A30/B29/C27 of36, 사전 동결 unit/date동치 A30/B29/C32 of36입니다. Marker가 있어도 추가 허위주장/조건확대가 있을 수 있어 전체 108개 본문의 source 대조를 별도로 제공합니다. QA02와 섞지 않습니다.

- [사례별 평균과 coverage](../basic-v1/case-report.md)
- [3개 개별값·median/min/max·raw links](../basic-v1/individual-trials.md)
- [단계별 call/token/cost](../basic-v1/stage-calls.csv)
- [고정목표 완료와 실패](../basic-v1/goal-endpoints.csv)
- [실제 제안 거부/복구](../basic-v1/proposal-rejections.csv)
- [같은 endpoint의 paired coverage](../basic-v1/pairwise-endpoint-coverage.json)
- [동시 RSS·초기값·차이·논리분류](../basic-v1/memory-breakdown.csv)
