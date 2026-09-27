# VIA-DP-05 official reference evaluation

> Campaign: `via-dp-05-evaluation-v1-20260927`
> Evidence: `MEASURED_REFERENCE_HARNESS`
> Frozen repetitions: 20 per case and candidate

## 1. 결론

두 후보는 동일한 source version과 권한 oracle을 만족했다. 측정값은 manifest 재확정과 capability별 조회 검증의 실제 reference 비용, correctness, continuity, privacy와 trace 차이를 보여 준다.

이 결과는 target Mac의 executable reference path에 한정된다. `BLOCKED`와 `N/A`를 수치로 대체하지 않았고 Architecture winner나 ASR을 확정하지 않는다.

## 2. A/B 구조

- A: 불변 Context manifest + generation 전환
- B: scoped Context capability + 점진적 read-set
- Mutually exclusive discriminator: 같은 Context generation에서 최초 집합 밖 ref를 읽을 수 있으면 B, 새 generation 확정이 필수면 A

## 3. 19-QA complete table

| QA | A | B |
| --- | --- | --- |
| QA-01 | BLOCKED — 구조는 위임 준비에 참여하지만 Agent ingress와 audible Voice 결과 endpoint를 실행하지 않음 | BLOCKED — 구조는 위임 준비에 참여하지만 Agent ingress와 audible Voice 결과 endpoint를 실행하지 않음 |
| QA-02 | BLOCKED — 구조는 direct Context 경로에 참여하지만 Voice input과 audible onset을 실행하지 않음 | BLOCKED — 구조는 direct Context 경로에 참여하지만 Voice input과 audible onset을 실행하지 않음 |
| QA-03 | N/A — 동결 case의 Agent status 전달 경로는 새 Context 조회를 수행하지 않음 | N/A — 동결 case의 Agent status 전달 경로는 새 Context 조회를 수행하지 않음 |
| QA-04 | N/A — 음성 playback 중단 경로가 Context read-set 확정에 참여하지 않음 | N/A — 음성 playback 중단 경로가 Context read-set 확정에 참여하지 않음 |
| QA-05 | BLOCKED — Context가 필요한 Task control corpus와 presentation endpoint를 실행하지 않음 | BLOCKED — Context가 필요한 Task control corpus와 presentation endpoint를 실행하지 않음 |
| QA-11 | 100/100 (100.0%) | 100/100 (100.0%) |
| QA-12 | 100/100 (100.0%) | 100/100 (100.0%) |
| QA-13 | N/A — Request와 Task 연결은 고정되어 후보 경로에 참여하지 않음 | N/A — Request와 Task 연결은 고정되어 후보 경로에 참여하지 않음 |
| QA-14 | N/A — 비동기 Task state event가 동결 Context case에 없음 | N/A — 비동기 Task state event가 동결 Context case에 없음 |
| QA-15 | 100/100 (100.0%) | 100/100 (100.0%) |
| QA-21 | N/A — Agent 계약 변화가 Context read-set 후보를 통과하지 않음 | N/A — Agent 계약 변화가 Context read-set 후보를 통과하지 않음 |
| QA-22 | BLOCKED — 15개 전체 Model·Context·State change pack의 실제 변경 exercise 미실행 | BLOCKED — 15개 전체 Model·Context·State change pack의 실제 변경 exercise 미실행 |
| QA-23 | BLOCKED — 5개 전체 실험·로그 change pack의 실제 변경 exercise 미실행 | BLOCKED — 5개 전체 실험·로그 change pack의 실제 변경 exercise 미실행 |
| QA-31 | BLOCKED — 진행 요청의 Context generation/read-set recovery fault pack 미실행 | BLOCKED — 진행 요청의 Context generation/read-set recovery fault pack 미실행 |
| QA-32 | N/A — 이번 정상 Context corpus에는 fault injection이 없음 | N/A — 이번 정상 Context corpus에는 fault injection이 없음 |
| QA-41 | BLOCKED — 공유 semantic model을 포함한 target-device peak committed memory boundary 미실행 | BLOCKED — 공유 semantic model을 포함한 target-device peak committed memory boundary 미실행 |
| QA-51 | max 0 / total 0 | max 0 / total 0 |
| QA-61 | 100/100 (100.0%) | 100/100 (100.0%) |
| QA-62 | 100.0% | 100.0% |

## 4. 보조 진단

| Candidate | 전체 reference path | strict case success | 추가 진단 |
| --- | --- | --- | --- |
| A | p95 0.027 ms / mean 0.016 ms / n=100 | 100/100 (100.0%) | 없음 |
| B | p95 0.039 ms / mean 0.028 ms / n=100 | 100/100 (100.0%) | 없음 |

## 5. 한계

- semantic LLM과 Voice acoustic endpoint를 호출하지 않은 deterministic Context-consumer reference path다.
- 전체 C/M change pack과 recovery fault pack은 실행하지 않았다.
- 절대시간은 Python reference 구현의 target-Mac 결과이며 제품 latency가 아니다.

## 6. 재현

`raw/trials.jsonl`에서 `summary.json`과 동일한 core digest를 독립 process로 재계산했다. contract·raw·summary digest는 `manifest.json`, 검증 결과는 `replay-receipt.json`에 있다.
