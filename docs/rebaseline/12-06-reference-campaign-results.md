# 12-06. Reference Campaign Results

> 상태: W-01~W-12 중 W-05 제외 측정 완료

## 결과 요약

| Metric | 결과 | Evidence scope |
|---|---|---|
| W-01 | AAAA 303.3ms, IR-B 327.3ms, EXEC-B 308.3ms; 모두 5점 | SIMULATED_REFERENCE |
| W-02 | 70~85ms; 모두 5점 | Agent P/Q acceptance stub + durable link |
| W-03 | AGENT A 29ms, B 32ms; 모두 5점 | instrumented reference UI |
| W-04 | A reference 1.06/4점, TASK-B·EXEC-B 1.05/5점 | SIMULATED_REFERENCE |
| W-05 | NOT_RUN | OpenRouter key 대기 |
| W-06 | 30 TC × 5회, 100%/5점 | fixture replay |
| W-07 | AGENT A 4점, B 3점 | DESIGN_ANALYSIS |
| W-08 | applicable DP 모두 4점 동점 | DESIGN_ANALYSIS |
| W-09 | TASK 동점 5점, EXEC B raw recovery 개선·동일 5점 | MEASURED_STRUCTURAL |
| W-10 | EXEC A/B 28/28·5점 | MEASURED_STRUCTURAL |
| W-11 | 모든 configuration 5/20=25%·3점 | synthetic protected-unit corpus |
| W-12 | 모든 configuration 0/24 violation·5점 | complete policy fixture |

W-01 입력 WAV는 `results/rebaseline/gate2-freeze-a4400f90/reference-campaign/w01-reference-input.wav`다. 이 campaign은 제품 latency 실측이 아니라 사전 동결 mockup/reference 모델이며 absolute latency claim을 하지 않는다.

## Combined Differentiation Gate

| DP | 결정 | Primary evidence |
|---|---|---|
| IR-DP01 | Deferred | W-01/W-02/W-08 score 동점, W-05 대기 |
| TASK-DP01 | **B Accepted** | W-04 reference 4→5점 |
| AGENT-DP01 | **A Accepted** | W-07 3→4점 |
| EXEC-DP01 | **B Accepted** | W-04 reference 4→5점; W-09 raw recovery도 B 방향 |

W-06/W-11/W-12는 모든 configuration이 동점이므로 system regression/constraint로 유지한다. Weighted total과 global winner는 만들지 않았다.

Machine result는 `results/rebaseline/gate2-freeze-a4400f90/combined-sweep.json`이다.
