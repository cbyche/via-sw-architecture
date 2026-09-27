# VIA-DP-13 official reference evaluation

> Campaign: `via-dp-13-evaluation-v1-20260927`
> Evidence: `MEASURED_REFERENCE_HARNESS`
> Frozen repetitions: 10 per case and candidate

## 1. 결론

같은 총 worker 수와 동일한 non-preemptive work를 사용해 포화 시 Text control disposition과 일반 작업 makespan의 구조적 맞교환을 실제 thread pool에서 측정했다.

이 결과는 target Mac의 executable reference path에 한정된다. `BLOCKED`와 `N/A`를 수치로 대체하지 않았고 Architecture winner나 ASR을 확정하지 않는다.

## 2. A/B 구조

- A: control slot reservation + work borrowing 제한
- B: 전체 공유 pool + pending control 우선
- Mutually exclusive discriminator: 마지막 실행 여력을 회수 불가능한 일반 작업에 내줄 수 없으면 A, 내줄 수 있으면 B

## 3. 19-QA complete table

| QA | A | B |
| --- | --- | --- |
| QA-01 | BLOCKED — scheduler는 위임 경로에 참여하지만 Voice·Agent interval·audible result endpoint를 실행하지 않음 | BLOCKED — scheduler는 위임 경로에 참여하지만 Voice·Agent interval·audible result endpoint를 실행하지 않음 |
| QA-02 | BLOCKED — direct Voice 경로에 구조가 참여하지만 acoustic input/output endpoint가 없음 | BLOCKED — direct Voice 경로에 구조가 참여하지만 acoustic input/output endpoint가 없음 |
| QA-03 | BLOCKED — status delivery가 공유 자원을 쓸 수 있으나 audible status endpoint를 실행하지 않음 | BLOCKED — status delivery가 공유 자원을 쓸 수 있으나 audible status endpoint를 실행하지 않음 |
| QA-04 | BLOCKED — 실제 barge-in onset과 interrupted response의 last audible sample을 관측하지 않음 | BLOCKED — 실제 barge-in onset과 interrupted response의 last audible sample을 관측하지 않음 |
| QA-05 | p95 0.234 ms / mean 0.105 ms / n=30 | p95 41.021 ms / mean 19.866 ms / n=30 |
| QA-11 | 30/30 (100.0%) | 30/30 (100.0%) |
| QA-12 | N/A — 동일한 확정 control 의미를 실행하며 semantic inference는 후보 차이에 참여하지 않음 | N/A — 동일한 확정 control 의미를 실행하며 semantic inference는 후보 차이에 참여하지 않음 |
| QA-13 | 30/30 (100.0%) | 30/30 (100.0%) |
| QA-14 | 30/30 (100.0%) | 30/30 (100.0%) |
| QA-15 | BLOCKED — 재연결·채널 전환 continuity scenario를 실행하지 않음 | BLOCKED — 재연결·채널 전환 continuity scenario를 실행하지 않음 |
| QA-21 | N/A — Agent contract 변화는 scheduler reservation 경계를 변경하지 않음 | N/A — Agent contract 변화는 scheduler reservation 경계를 변경하지 않음 |
| QA-22 | BLOCKED — 전체 M/C change pack에서 reservation dependency 변경 exercise 미실행 | BLOCKED — 전체 M/C change pack에서 reservation dependency 변경 exercise 미실행 |
| QA-23 | BLOCKED — E-01~05 queue instrumentation change exercise 미실행 | BLOCKED — E-01~05 queue instrumentation change exercise 미실행 |
| QA-31 | N/A — 정상 과부하 campaign이며 process/storage fault를 주입하지 않음 | N/A — 정상 과부하 campaign이며 process/storage fault를 주입하지 않음 |
| QA-32 | N/A — 정상 자원 포화는 fault blast radius가 아님 | N/A — 정상 자원 포화는 fault blast radius가 아님 |
| QA-41 | BLOCKED — 전체 VIA와 공유 모델을 포함한 peak committed memory boundary 미실행 | BLOCKED — 전체 VIA와 공유 모델을 포함한 peak committed memory boundary 미실행 |
| QA-51 | N/A — scheduler는 보호정보 recipient나 전달 범위를 변경하지 않음 | N/A — scheduler는 보호정보 recipient나 전달 범위를 변경하지 않음 |
| QA-61 | 30/30 (100.0%) | 30/30 (100.0%) |
| QA-62 | 100.0% | 100.0% |

## 4. 보조 진단

| Candidate | 전체 reference path | strict case success | 추가 진단 |
| --- | --- | --- | --- |
| A | p95 129.537 ms / mean 80.678 ms / n=30 | 30/30 (100.0%) | p95 129.539 ms / mean 80.679 ms / n=30 |
| B | p95 90.222 ms / mean 59.725 ms / n=30 | 30/30 (100.0%) | p95 90.225 ms / mean 59.727 ms / n=30 |

## 5. 한계

- QA-04는 실제 acoustic barge-in/last-audible-sample을 실행하지 않아 내부 control 시간으로 대체하지 않았다.
- 외부 semantic model과 Agent의 비선점 구간은 이 targeted scheduler path에 포함하지 않았다.
- work makespan은 trade-off 원인 진단이며 활성 QA를 새로 만들거나 QA-01/02를 대신하지 않는다.

## 6. 재현

`raw/trials.jsonl`에서 `summary.json`과 동일한 core digest를 독립 process로 재계산했다. contract·raw·summary digest는 `manifest.json`, 검증 결과는 `replay-receipt.json`에 있다.
