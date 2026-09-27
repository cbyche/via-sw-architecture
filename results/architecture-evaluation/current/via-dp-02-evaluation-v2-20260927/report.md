# VIA-DP-02 official reference evaluation

> Campaign: `via-dp-02-evaluation-v2-20260927`
> Evidence: `MEASURED_REFERENCE_HARNESS`
> Frozen repetitions: 12 per case and candidate

## 1. 결론

실제 SQLite FULL durability를 사용해 정상 교차 관계, Task control, 완료 교차와 중단 후 복구를 실행했다. 후보별 commit 수와 reconciliation이 Text disposition·관계 정확성·복구시간에 미치는 영향을 비교한다.

이 결과는 target Mac의 executable reference path에 한정된다. `BLOCKED`와 `N/A`를 수치로 대체하지 않았고 Architecture winner나 ASR을 확정하지 않는다.

## 2. A/B 구조

- A: 교차 관계 공동 원자 commit
- B: 독립 owner commit + relation reconciliation
- Mutually exclusive discriminator: Conversation·Task 교차 관계를 하나의 최종 transaction으로 확정하면 A, 독립 commit의 확인으로 확정하면 B

## 3. 19-QA complete table

| QA | A | B |
| --- | --- | --- |
| QA-01 | BLOCKED — 교차 commit은 위임 경로에 참여하지만 Voice·Agent ingress/result·audible endpoint를 실행하지 않음 | BLOCKED — 교차 commit은 위임 경로에 참여하지만 Voice·Agent ingress/result·audible endpoint를 실행하지 않음 |
| QA-02 | BLOCKED — 기존 Task를 참조하는 direct Voice case의 acoustic endpoint를 실행하지 않음 | BLOCKED — 기존 Task를 참조하는 direct Voice case의 acoustic endpoint를 실행하지 않음 |
| QA-03 | BLOCKED — Agent status와 Conversation 게시 관계는 관련되지만 audible status endpoint가 없음 | BLOCKED — Agent status와 Conversation 게시 관계는 관련되지만 audible status endpoint가 없음 |
| QA-04 | N/A — 음성 playback 중단은 교차 관계 commit을 기다리지 않음 | N/A — 음성 playback 중단은 교차 관계 commit을 기다리지 않음 |
| QA-05 | p95 0.590 ms / mean 0.307 ms / n=48 | p95 2.083 ms / mean 1.158 ms / n=48 |
| QA-11 | 60/60 (100.0%) | 60/60 (100.0%) |
| QA-12 | N/A — 사용자 의미 해석 결과는 동일한 fixture로 고정됨 | N/A — 사용자 의미 해석 결과는 동일한 fixture로 고정됨 |
| QA-13 | 60/60 (100.0%) | 60/60 (100.0%) |
| QA-14 | 60/60 (100.0%) | 60/60 (100.0%) |
| QA-15 | 60/60 (100.0%) | 60/60 (100.0%) |
| QA-21 | N/A — Agent integration contract는 양쪽에서 고정됨 | N/A — Agent integration contract는 양쪽에서 고정됨 |
| QA-22 | BLOCKED — C-06을 포함한 15개 전체 Model·Context·State change exercise 미실행 | BLOCKED — C-06을 포함한 15개 전체 Model·Context·State change exercise 미실행 |
| QA-23 | BLOCKED — E-01~05 전체 experiment/logging change exercise 미실행 | BLOCKED — E-01~05 전체 experiment/logging change exercise 미실행 |
| QA-31 | p95 0.346 ms / mean 0.273 ms / n=12 | p95 0.952 ms / mean 0.826 ms / n=12 |
| QA-32 | max 0 / n=60 | max 0 / n=60 |
| QA-41 | BLOCKED — 전체 VIA와 공유 모델을 포함한 peak committed memory boundary 미실행 | BLOCKED — 전체 VIA와 공유 모델을 포함한 peak committed memory boundary 미실행 |
| QA-51 | max 0 / total 0 | max 0 / total 0 |
| QA-61 | 60/60 (100.0%) | 60/60 (100.0%) |
| QA-62 | 100.0% | 100.0% |

## 4. 보조 진단

| Candidate | 전체 reference path | strict case success | 추가 진단 |
| --- | --- | --- | --- |
| A | p95 1.117 ms / mean 0.812 ms / n=60 | 60/60 (100.0%) | 없음 |
| B | p95 3.556 ms / mean 2.591 ms / n=60 | 60/60 (100.0%) | 없음 |

## 5. 한계

- 동일 Process 안의 reference SQLite 구현이며 distributed database나 Process isolation을 비교하지 않는다.
- Voice acoustic endpoint와 전체 15개 state change pack은 실행하지 않았다.
- 절대시간은 target Mac filesystem과 Python sqlite3에 한정된다.

## 6. 재현

`raw/trials.jsonl`에서 `summary.json`과 동일한 core digest를 독립 process로 재계산했다. contract·raw·summary digest는 `manifest.json`, 검증 결과는 `replay-receipt.json`에 있다.
