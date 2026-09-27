# DP-12 official reference evaluation

> Campaign: `dp12-evaluation-v1-20260927`
> Evidence: `MEASURED_REFERENCE_HARNESS`
> Frozen repetitions: 20 per case and candidate

## 1. 결론

동일 local spool과 record를 사용해 정상·flush 전 crash·writer failure에서 Text disposition latency, 사용자 요청 완료, logging change 범위와 trace completeness를 비교했다.

이 결과는 target Mac의 executable reference path에 한정된다. `BLOCKED`와 `N/A`를 수치로 대체하지 않았고 Architecture winner나 ASR을 확정하지 않는다.

## 2. A/B 구조

- A: 최소 evidence durable ACK 후 게시
- B: 게시와 evidence background flush 분리
- Mutually exclusive discriminator: 같은 response가 해당 최소 evidence의 durable ACK 전에 게시될 수 없으면 A, 게시될 수 있으면 B

## 3. 19-QA complete table

| QA | A | B |
| --- | --- | --- |
| QA-01 | BLOCKED — publish ordering은 참여하지만 Voice input·Agent result·audible endpoint를 실행하지 않음 | BLOCKED — publish ordering은 참여하지만 Voice input·Agent result·audible endpoint를 실행하지 않음 |
| QA-02 | BLOCKED — direct publish ordering은 참여하지만 audible Voice onset이 없음 | BLOCKED — direct publish ordering은 참여하지만 audible Voice onset이 없음 |
| QA-03 | BLOCKED — status publish ordering은 참여하지만 audible Voice status endpoint가 없음 | BLOCKED — status publish ordering은 참여하지만 audible Voice status endpoint가 없음 |
| QA-04 | N/A — 음성 stop은 diagnostic evidence gate를 기다리지 않는 공통 안전 경로 | N/A — 음성 stop은 diagnostic evidence gate를 기다리지 않는 공통 안전 경로 |
| QA-05 | p95 100.000 ms / mean 33.386 ms / n=60 | p95 0.085 ms / mean 0.028 ms / n=60 |
| QA-11 | 80/100 (80.0%) | 100/100 (100.0%) |
| QA-12 | N/A — semantic interpretation은 게시·기록 ordering 전에 동일하게 확정됨 | N/A — semantic interpretation은 게시·기록 ordering 전에 동일하게 확정됨 |
| QA-13 | N/A — Task binding business durability는 공통이고 후보 차이에 참여하지 않음 | N/A — Task binding business durability는 공통이고 후보 차이에 참여하지 않음 |
| QA-14 | N/A — Task state convergence는 공통 business state 경로에 있음 | N/A — Task state convergence는 공통 business state 경로에 있음 |
| QA-15 | N/A — Conversation·Task continuity는 diagnostic evidence ordering을 통과하지 않음 | N/A — Conversation·Task continuity는 diagnostic evidence ordering을 통과하지 않음 |
| QA-21 | N/A — Agent contract 변화는 evidence publish ordering을 변경하지 않음 | N/A — Agent contract 변화는 evidence publish ordering을 변경하지 않음 |
| QA-22 | BLOCKED — 전체 Model·Context·State change pack에서 required evidence 변경 exercise 미실행 | BLOCKED — 전체 Model·Context·State change pack에서 required evidence 변경 exercise 미실행 |
| QA-23 | average 3.00 elements / 5 changes | average 2.00 elements / 5 changes |
| QA-31 | N/A — Task recovery는 공통 business state이고 diagnostic spool recovery를 QA-31로 세지 않음 | N/A — Task recovery는 공통 business state이고 diagnostic spool recovery를 QA-31로 세지 않음 |
| QA-32 | BLOCKED — writer fault의 승인된 necessary dependency closure와 user-visible unit registry 미동결 | BLOCKED — writer fault의 승인된 necessary dependency closure와 user-visible unit registry 미동결 |
| QA-41 | BLOCKED — pending release와 flush backlog의 target-device peak committed memory campaign 미실행 | BLOCKED — pending release와 flush backlog의 target-device peak committed memory campaign 미실행 |
| QA-51 | max 0 / total 0 | max 0 / total 0 |
| QA-61 | 80/100 (80.0%) | 60/100 (60.0%) |
| QA-62 | 100.0% | 100.0% |

## 4. 보조 진단

| Candidate | 전체 reference path | strict case success | 추가 진단 |
| --- | --- | --- | --- |
| A | p95 0.090 ms / mean 0.060 ms / n=100 | 80/100 (80.0%) | 없음 |
| B | p95 0.086 ms / mean 0.045 ms / n=100 | 100/100 (100.0%) | 없음 |

## 5. 한계

- 실제 Voice playback 후발 event는 실행하지 않아 QA-01~03을 내부 publish 시간으로 대체하지 않았다.
- Business state/outbox durability는 공통 fixture로 고정하고 diagnostic evidence ordering만 비교했다.
- QA-23은 E-01~05를 대표하는 동결 schema-change exercise에서 실제 후보 element registry를 통과한 값이다.

## 6. 재현

`raw/trials.jsonl`에서 `summary.json`과 동일한 core digest를 독립 process로 재계산했다. contract·raw·summary digest는 `manifest.json`, 검증 결과는 `replay-receipt.json`에 있다.
