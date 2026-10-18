# 같은 사례의 기본안과 보완 비교

각 행은 동일 사례의 BASIC 3회와 별도 보완 3회다. 기본안 전체12회 평균과 보완3회를 섞지 않는다. 관측값만의 평균은 조건부이며 누락·오답·계약 실패를 함께 읽는다. B4는 직렬/병행 정책 기여분 진단이며 기본 B를 대체하지 않는다. QA04 보완 후 변경과제는 별도 실행하지 않았으므로 이 표에 기본 QA04를 전이하지 않는다.

|보완|사례|구분|첫 응답 ms (관측/예정)|완료 ms (관측/예정)|집합 정답/예정|호출|VIA USD|수신자 USD|최대 동시 MB|계약/완료 실패|
|---|---|---|---|---|---:|---:|---:|---:|---:|---:|
|dp04-A4|dp04-dense|BASIC|17788.669 (3/3)|17788.926 (3/3)|6/6|6|0.0110124|0.0000000|45.220|0/3|
|dp04-A4|dp04-dense|dp04-A4|31372.113 (3/3)|31372.397 (3/3)|6/6|6|0.0154072|0.0000000|43.860|0/3|
|dp04-A4|dp04-free-region|BASIC|NOT_OBSERVED (0/3)|NOT_OBSERVED (0/3)|0/3|6|0.0092508|0.0000000|44.089|3/3|
|dp04-A4|dp04-free-region|dp04-A4|20483.486 (3/3)|20483.679 (3/3)|3/3|6|0.0117408|0.0000000|44.253|0/3|
|dp04-A4|dp04-page-change|BASIC|10011.640 (3/3)|10011.850 (3/3)|3/3|6|0.0105408|0.0000000|44.859|0/3|
|dp04-A4|dp04-page-change|dp04-A4|18808.005 (3/3)|18808.175 (3/3)|3/3|6|0.0133444|0.0000000|45.122|0/3|
|dp04-A4|dp04-partial-correction|BASIC|7089.272 (4/6)|19070.772 (3/3)|8/12|8|0.0107788|0.0000000|44.253|2/3|
|dp04-A4|dp04-partial-correction|dp04-A4|14161.084 (6/6)|28322.990 (3/3)|12/12|12|0.0156724|0.0000000|44.466|0/3|
|dp04-B4|dp04-dense|BASIC|27580.467 (3/3)|27580.712 (3/3)|6/6|6|0.0108684|0.0000000|43.844|0/3|
|dp04-B4|dp04-dense|dp04-B4|22341.768 (3/3)|22341.959 (3/3)|6/6|6|0.0108780|0.0000000|46.072|0/3|
|dp04-B4|dp04-free-region|BASIC|16842.570 (3/3)|16842.797 (3/3)|3/3|6|0.0077416|0.0000000|43.663|0/3|
|dp04-B4|dp04-free-region|dp04-B4|13367.003 (3/3)|13367.302 (3/3)|3/3|6|0.0077384|0.0000000|45.351|0/3|
|dp04-B4|dp04-page-change|BASIC|16475.281 (3/3)|16475.479 (3/3)|3/3|8|0.0094708|0.0000000|44.745|0/3|
|dp04-B4|dp04-page-change|dp04-B4|15607.886 (3/3)|15608.081 (3/3)|3/3|7|0.0097828|0.0000000|46.203|0/3|
|dp04-B4|dp04-partial-correction|BASIC|15410.978 (6/6)|30822.855 (3/3)|12/12|18|0.0150364|0.0000000|45.351|0/3|
|dp04-B4|dp04-partial-correction|dp04-B4|10511.288 (6/6)|21023.434 (3/3)|12/12|18|0.0151800|0.0000000|46.973|0/3|
|dp01-A2|dp01-e3|BASIC|6888.312 (6/6)|21822.022 (6/6)|6/6|12|0.0148424|0.0000000|43.663|0/3|
|dp01-A2|dp01-e3|dp01-A2|6394.130 (6/6)|21432.660 (6/6)|6/6|12|0.0145860|0.0000000|43.926|0/3|

## 개별 반복과 실제 기전

같은 키의 endpoint 관측 pairing과 전체 개별3회 원값은 tactic-comparison.json에 보존한다. 각 phase의 individual-trials.md에는 각3회·median/min/max·raw 링크가 있다. 의미 내용의 literal/equivalence/저자 검토는 구조 QA02 및 계측 완료와 분리한다.

### dp04-A4 · dp04-dense

실제 SOURCE_READ 호출: BASIC 3 → 보완 3. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.joint_grounding": 3, "dp04.joint_source_reread": 3}`

보완 기전: `{"dp04.screen_producer": 3, "dp04.joint_grounding": 3}`

### dp04-A4 · dp04-free-region

실제 SOURCE_READ 호출: BASIC 3 → 보완 3. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.joint_grounding": 3, "dp04.joint_source_reread": 3}`

보완 기전: `{"dp04.screen_producer": 3, "dp04.joint_grounding": 3}`

### dp04-A4 · dp04-page-change

실제 SOURCE_READ 호출: BASIC 6 → 보완 6. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.joint_grounding": 3, "dp04.joint_source_reread": 3}`

보완 기전: `{"dp04.screen_producer": 3, "dp04.joint_grounding": 3}`

### dp04-A4 · dp04-partial-correction

실제 SOURCE_READ 호출: BASIC 6 → 보완 6. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.joint_grounding": 6, "dp04.joint_source_reread": 2}`

보완 기전: `{"dp04.screen_producer": 6, "dp04.joint_grounding": 6}`

### dp04-B4 · dp04-dense

실제 SOURCE_READ 호출: BASIC 3 → 보완 3. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.screen_producer": 3, "dp04.speech_producer": 3}`

보완 기전: `{"dp04.speech_producer": 3, "dp04.screen_producer": 3}`

### dp04-B4 · dp04-free-region

실제 SOURCE_READ 호출: BASIC 3 → 보완 3. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.screen_producer": 3, "dp04.speech_producer": 3}`

보완 기전: `{"dp04.speech_producer": 3, "dp04.screen_producer": 3}`

### dp04-B4 · dp04-page-change

실제 SOURCE_READ 호출: BASIC 6 → 보완 6. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.screen_producer": 3, "dp04.speech_producer": 3, "dp04.candidate_binding": 2}`

보완 기전: `{"dp04.speech_producer": 3, "dp04.screen_producer": 3, "dp04.candidate_binding": 1}`

### dp04-B4 · dp04-partial-correction

실제 SOURCE_READ 호출: BASIC 6 → 보완 6. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp04.screen_producer": 6, "dp04.speech_producer": 6, "dp04.candidate_binding": 6}`

보완 기전: `{"dp04.speech_producer": 6, "dp04.screen_producer": 6, "dp04.candidate_binding": 6}`

### dp01-A2 · dp01-e3

실제 SOURCE_READ 호출: BASIC 12 → 보완 9. 이는 파일 읽기 호출 수이며 physical disk I/O 수가 아니다.

BASIC 기전: `{"dp01_select_adopt_or_complete": 6, "dp01_source_backed_answer": 6}`

보완 기전: `{"dp01_select_adopt_or_complete": 6, "dp01_source_backed_answer": 6}`
