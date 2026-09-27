# VIA-DP-09 official reference evaluation

> Campaign: `via-dp-09-evaluation-v1-20260927`
> Evidence: `MEASURED_REFERENCE_HARNESS`
> Frozen repetitions: 25 per case and candidate

## 1. 결론

동일 native lifecycle corpus에서 두 후보의 기능 보존, Text control disposition, Task 연결·수렴과 9개 Agent 변화의 실제 handler 영향 범위를 비교했다.

이 결과는 target Mac의 executable reference path에 한정된다. `BLOCKED`와 `N/A`를 수치로 대체하지 않았고 Architecture winner나 ASR을 확정하지 않는다.

## 2. A/B 구조

- A: Agent 경계 semantic normalization
- B: typed contract + Core lifecycle handler
- Mutually exclusive discriminator: Task-level 수명 의미의 최종 해석자가 edge adapter면 A, Core lifecycle handler면 B

## 3. 19-QA complete table

| QA | A | B |
| --- | --- | --- |
| QA-01 | BLOCKED — Agent 실행 전후 경계는 참여하지만 Voice input·Agent interval·audible result endpoint를 실행하지 않음 | BLOCKED — Agent 실행 전후 경계는 참여하지만 Voice input·Agent interval·audible result endpoint를 실행하지 않음 |
| QA-02 | N/A — direct response에는 Agent lifecycle 의미 경계가 참여하지 않음 | N/A — direct response에는 Agent lifecycle 의미 경계가 참여하지 않음 |
| QA-03 | BLOCKED — Agent status 의미는 실행했지만 audible Voice presentation endpoint가 없음 | BLOCKED — Agent status 의미는 실행했지만 audible Voice presentation endpoint가 없음 |
| QA-04 | N/A — 음성 playback 중단은 Agent lifecycle 의미 해석을 통과하지 않음 | N/A — 음성 playback 중단은 Agent lifecycle 의미 해석을 통과하지 않음 |
| QA-05 | p95 0.001 ms / mean 0.001 ms / n=75 | p95 0.001 ms / mean 0.001 ms / n=75 |
| QA-11 | 225/225 (100.0%) | 225/225 (100.0%) |
| QA-12 | N/A — 사용자 요청 의미는 고정되어 Agent native lifecycle 해석만 비교함 | N/A — 사용자 요청 의미는 고정되어 Agent native lifecycle 해석만 비교함 |
| QA-13 | 225/225 (100.0%) | 225/225 (100.0%) |
| QA-14 | 225/225 (100.0%) | 225/225 (100.0%) |
| QA-15 | N/A — 채널 재연결·Conversation continuity scenario가 없음 | N/A — 채널 재연결·Conversation continuity scenario가 없음 |
| QA-21 | average 1.44 elements / 9 changes | average 2.00 elements / 9 changes |
| QA-22 | N/A — Model·Context·State 변화는 고정 Agent lifecycle boundary를 변경하지 않음 | N/A — Model·Context·State 변화는 고정 Agent lifecycle boundary를 변경하지 않음 |
| QA-23 | BLOCKED — E-01~05 전체 logging change exercise 미실행 | BLOCKED — E-01~05 전체 logging change exercise 미실행 |
| QA-31 | BLOCKED — native execution mapping을 포함한 restart recovery campaign 미실행 | BLOCKED — native execution mapping을 포함한 restart recovery campaign 미실행 |
| QA-32 | N/A — 두 후보는 같은 Process·fault boundary에 배치됨 | N/A — 두 후보는 같은 Process·fault boundary에 배치됨 |
| QA-41 | BLOCKED — 전체 VIA와 공유 모델을 포함한 peak committed memory boundary 미실행 | BLOCKED — 전체 VIA와 공유 모델을 포함한 peak committed memory boundary 미실행 |
| QA-51 | max 0 / total 0 | max 0 / total 0 |
| QA-61 | 225/225 (100.0%) | 225/225 (100.0%) |
| QA-62 | 100.0% | 100.0% |

## 4. 보조 진단

| Candidate | 전체 reference path | strict case success | 추가 진단 |
| --- | --- | --- | --- |
| A | p95 0.002 ms / mean 0.001 ms / n=225 | 225/225 (100.0%) | 없음 |
| B | p95 0.002 ms / mean 0.001 ms / n=225 | 225/225 (100.0%) | 없음 |

## 5. 한계

- 외부 Reference Agent의 native event를 deterministic fixture로 실행했으며 실제 OpenClaw/Hermes 제품은 사용하지 않았다.
- Voice status playback과 전체 재시작 recovery는 실행하지 않았다.
- QA-21은 동결된 9개 native change case가 실제로 통과한 reference element registry의 평균이다.

## 6. 재현

`raw/trials.jsonl`에서 `summary.json`과 동일한 core digest를 독립 process로 재계산했다. contract·raw·summary digest는 `manifest.json`, 검증 결과는 `replay-receipt.json`에 있다.
