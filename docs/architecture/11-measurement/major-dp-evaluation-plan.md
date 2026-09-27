# 주요 VIA-DP 연속 평가 실행 계획

> 상태: **ACTIVE RECOVERY PLAN — 통합 campaign 재구축 중**
> 시작일: 2026-09-27
> 목표: VIA-DP-01~18 전체를 동일한 Architecture 평가 절차로 검토·구현·측정한다.

이 문서는 밤사이 작업의 실행 원장이다. 채팅 기억이 아니라 이 파일의 상태와
체크리스트를 기준으로 진행하며, 각 milestone 뒤 진행표와 실행 로그를 갱신한다.

## 1. 불변 규칙

1. **모든 DP의 기본 평가표는 active QA 19개 전체다.** 일부 QA만 선택해 시작하지 않는다.
2. 각 QA는 `MEASURED`, `N/A`, `BLOCKED` 중 하나와 근거를 가져야 한다.
3. `N/A`는 A/B 구조 차이가 해당 QA의 실제 dependency graph에 물리적으로 참여하지
   않을 때만 허용한다. 차이가 작을 것 같다는 이유는 `N/A`가 아니다.
4. `BLOCKED`는 QA가 관련되지만 유효한 endpoint, oracle, 반복 또는 candidate 구현이
   없을 때 사용한다. proxy, 고정 PASS, hand-authored 결과로 숫자를 채우지 않는다.
5. A/B의 사용자 목표, completion condition, fixture, dependency, model, Agent와 자원
   한도는 동일하게 유지하며 한 번에 DP 하나만 바꾼다.
6. 후보 입력과 evaluator-only oracle을 분리한다.
7. correctness 실패와 latency를 분리한다. 응답 endpoint가 관측되면 실제 시간을
   보존하고, 물리 endpoint 자체가 없거나 무효일 때만 동결 timeout을 적용한다.
8. QA-01~04는 Voice input과 audible loopback endpoint를 사용한다. payload, renderer
   callback 또는 queue operation을 audible endpoint로 대체하지 않는다.
9. QA-01은 Agent 실행시간을 제외하지만 실제 Agent ingress/result source event를 쓴다.
10. 실패·timeout을 사후 삭제하지 않고 raw evidence를 summary보다 먼저 기록한다.
11. contract, fixture, repetition, 집계와 failure treatment는 결과 전에 동결한다.
12. 결과를 본 뒤 contract를 바꾸거나 자동으로 새 버전을 만들어 재실행하지 않는다.
    결함 발견 시 그 campaign을 `INVALID`로 중단하고 이 문서에 기록한다.
13. preflight는 `/private/tmp`에서 수행하고 성공 후 삭제한다. `current/`에는 공식
    immutable campaign만 둔다.
14. 모든 보고서는 A/B 및 허용된 tactic 후보를 포함한 19-QA complete table을 가진다.
15. 한 DP가 `BLOCKED`/`INVALID`여도 원인이 독립적인 다음 DP는 계속 진행한다.
16. result package가 완결됐다는 이유만으로 `COMPLETE`라고 하지 않는다. 관련 QA의
    실제 endpoint, 반복 수, candidate 구조 참여와 독립 replay가 모두 충족되어야 한다.
17. 아래 수준보다 낮은 기존 결과는 원본을 보존하되 Architecture 선택 근거가 아닌
    `PRELIMINARY_COMPONENT_ONLY`, `TARGETED_PARTIAL`,
    `TARGETED_PROCESS_EVIDENCE`, `INTEGRATED_REFERENCE_BREADTH_ONLY`로 표시한다.

## 2. 완료 판정 기준

DP-06 v4의 장점인 실제 local semantic model, 외부 Reference Agent process, generated
Voice input, BlackHole audible loopback과 raw field-level oracle을 공통 최저선으로 삼는다.
그보다 높은 최종 campaign은 다음을 모두 만족해야 한다.

1. A/B가 문서 설명이 아니라 component/process/message/state 차이로 실행된다.
2. 두 후보에서 해당 DP 외의 model, fixture, Agent, Context, 자원 한도는 같다.
3. 19개 QA를 모두 감사하고 각 셀을 `MEASURED`, `N/A`, `BLOCKED`로 근거와 함께 남긴다.
4. Responsiveness 대표값은 warm-up 뒤 case별 20회 이상의 scored repetition에서 계산한다.
5. correctness는 strict case success와 field-level predicate 원본을 함께 보존한다.
6. fault/recovery가 관련되면 독립 fresh state fault trial을 조건별 10회 이상 실행한다.
7. QA-21~23은 실제 change exercise와 frozen element ledger로 측정한다.
8. QA-41은 후보 process와 shared local model을 포함한 동일 accounting boundary를 쓴다.
9. QA-51은 evaluator-only protected-data oracle로 실제 exposure를 판정한다.
10. QA-61/62는 raw trace completeness와 clean replay digest로 측정한다.
11. physical endpoint가 필요한 QA는 실제 input/loopback 사건을 사용한다. 관련 endpoint가
    없으면 숫자를 만들지 않고 `BLOCKED`로 남긴다.
12. 모든 raw trial, model call/token ledger, repair 위치, failure, timeout과 environment를
    immutable result directory에 보존한다.

이 기준을 모두 충족하기 전에는 보고서가 19행 표를 가진 경우에도 `COMPLETE`가 아니다.

## 3. DP별 공통 lifecycle

1. **Decision audit:** A/B 상호 배타성, steelman, Architecture 차이와 tactic 구분
2. **19-QA audit:** 19개 각각의 endpoint와 A/B 차이 참여 경로 작성
3. **Candidate implementation:** component/process/message/state 수준 A/B 구현
4. **Fixture/oracle freeze:** 정상 workload 우선, 필요한 fault/load/change pack만 추가
5. **Qualification:** 정상 trace PASS, mutation sentinel은 예상 failure code로 FAIL
6. **Official campaign:** 새 result directory, 실패 포함 raw 즉시 append
7. **Independent analysis:** 별도 process가 raw summary digest를 재현
8. **Report/audit:** 19-QA 표, 보조 지표, 원인·한계, tests/link/terminology/diff 검사

## 4. 공통 QA 19개

| 범주 | QA |
| --- | --- |
| Responsiveness | QA-01, QA-02, QA-03, QA-04, QA-05 |
| Correctness / continuity | QA-11, QA-12, QA-13, QA-14, QA-15 |
| Modifiability | QA-21, QA-22, QA-23 |
| Reliability | QA-31, QA-32 |
| Resource | QA-41 |
| Privacy | QA-51 |
| Observability | QA-61, QA-62 |

QA-11은 통합 결과이고 QA-12~15는 비가산 원인 지표다. 모두 보고하지만 단순 합산한
후보 점수는 만들지 않는다.

## 5. 실행 순서

### 5.1 1차 핵심 DP

사용자와 합의한 순서로 먼저 처리한다.

1. VIA-DP-06 — semantic 최종 확정 권한
2. VIA-DP-11 — 외부 연동 Process 장애 경계
3. VIA-DP-14 — Task 상태 writer
4. VIA-DP-09 — Agent 의미 해석 위치
5. VIA-DP-02 — Conversation–Task 확정 경계
6. VIA-DP-08 — 복구 기준 기록
7. VIA-DP-13 — control 자원 예약

### 5.2 후속 전체 DP

1차 완료 후 다음 우선순위로 나머지를 처리한다. 우선순위는 핵심 사용자 경로 참여,
상태·권한 소유권, A/B의 19-QA trade-off 가능성, 변경 비용 순이다.

1. VIA-DP-04 — S2S 직접 응답 게시 권한
2. VIA-DP-05 — 요청 Context 읽기 집합 계약
3. VIA-DP-12 — 응답 게시와 실행 근거 확정 순서
4. VIA-DP-15 — Agent 상태 관측 확정 경로
5. VIA-DP-18 — 보호정보·Action 권한 확인 위치
6. VIA-DP-10 — Model 세션·연결 수명
7. VIA-DP-16 — Model 입력 이력 유지 책임
8. VIA-DP-17 — Context materialization 책임
9. VIA-DP-07 — 복합 요청 관계 실행 책임
10. VIA-DP-01 — 범위가 정해진 정보 처리 책임 경계
11. VIA-DP-03 — 음성 입력 근거의 최종 기준

각 DP의 decision audit 결과 순서를 바꿔야 하면 이유를 실행 로그에 먼저 기록한다.

## 6. 현재 증거 수준과 다음 campaign

| DP | 보존된 결과 경로 | 현재 증거 수준 | 다음 작업 |
| --- | --- | --- | --- |
| VIA-DP-06 | `via-dp-06-evaluation-v4-20260927/` | INTEGRATED_REFERENCE_BREADTH_ONLY | v5 공통 통합 하네스, case별 20회 반복 |
| VIA-DP-11 | `via-dp-11-evaluation-v4-20260927/` | TARGETED_PROCESS_EVIDENCE | 통합 Voice/semantic/Task 경로에 실제 process 후보 결합 |
| VIA-DP-14 | `via-dp-14-evaluation-v1-20260927/` | PROTOTYPE_PRIMITIVES_ONLY | 두 Task writer는 있으나 frozen race/load campaign 필요 |
| VIA-DP-09 | `via-dp-09-evaluation-v1-20260927/` | PRELIMINARY_COMPONENT_ONLY | Rust lifecycle path와 shared model/Voice/Agent에 결합 |
| VIA-DP-02 | `via-dp-02-evaluation-v2-20260927/` | TARGETED_PARTIAL | Rust Task/Conversation store와 통합 interaction path 결합 |
| VIA-DP-08 | `via-dp-08-evaluation-v1-20260927/` | PROTOTYPE_PRIMITIVES_ONLY | recovery primitive는 있으나 두 복구 원본 후보 campaign 미구현 |
| VIA-DP-13 | `via-dp-13-evaluation-v1-20260927/` | TARGETED_PARTIAL | 실제 scheduler/queue와 정상·포화 통합 workload 결합 |
| VIA-DP-05 | `via-dp-05-evaluation-v1-20260927/` | PRELIMINARY_COMPONENT_ONLY | 실제 Context adapter/read-set과 semantic path 결합 |
| VIA-DP-12 | `via-dp-12-evaluation-v1-20260927/` | TARGETED_PARTIAL | Rust evidence store와 response/Agent dispatch 경로 결합 |
| VIA-DP-01,03,04,07,10,15~18 | `via-dp-NN-evaluation-v1-20260927/` | AUDIT_REQUIRED | executable candidate·oracle 구현 필요 |

`IMPLEMENTATION_REQUIRED`는 blocker나 결론이 아니다. 문서 후보를 실제 executable
candidate와 evaluator로 옮겨야 한다는 현재 상태다.

## 7. VIA-DP-06 v4 보존 카드와 v5 기준

- Contract: `benchmark/architecture/contracts/via-dp-06-evaluation-v4.json`
- Candidates: A, B, B′
- Cases: 24 × 3 = 72 trial
- Model: shared local Qwen3-8B Q4_K_M 1개
- Agent: external Reference Agent TCP process
- Voice: generated request WAV + Yuna renderer + BlackHole loopback
- Context: 동일 frozen Context output replay; Context Engine은 이 DP의 변수가 아님
- S2S tool calling: 없음
- Correctness: QA-11~13 strict, QA-12 field-level 보조
- Responsiveness: QA-01/02/05 endpoint와 full wall-clock
- Physical endpoint failure score: 60초; semantic 오답에는 적용하지 않음
- Repetition: case당 1회 breadth. production p95도 최종 선택 근거도 아님
- v5: 동일 24개 case를 유지하되 warm-up 2회 + scored 20회, candidate process와
  shared dependency 수명을 고정하고 trial-level raw를 즉시 기록

VIA-DP-06도 19개 QA 전부를 표에 포함한다. contract의 applicability는 qualification에서
실제 참여 경로를 재확인하며, 비참여가 입증된 QA만 `N/A`로 확정한다.

## 8. DP별 산출물 완결 조건

- `manifest.json`: contract/fixture/oracle/source digest와 환경
- `raw/trials.jsonl`: 성공·실패·timeout 포함 원본
- `raw/change-exercises.json`: 실제 QA-21~23 change observation
- `summary.json`: raw 재계산 결과
- `replay-receipt.json`: 독립 analyzer digest 검증
- `report.md`: 19-QA complete table, 보조 지표, 원인과 evidence 한계
- `STATUS.md`: `COMPLETE`, `BLOCKED` 또는 `INVALID`와 근거

값이 없는 QA도 행을 삭제하지 않는다. `N/A` 또는 `BLOCKED`와 구체 사유를 쓴다.

## 9. 사용자 개입 정책

밤사이 사용자 개입은 요구하지 않는다. 다음 경우에만 해당 DP를 멈추고 아침 보고서에
명시한 뒤 독립적인 다음 DP로 진행한다.

- 새 외부 계정·비용·credential이 필요함
- 실제 사용자 데이터나 비가역 외부 동작이 필요함
- A/B 정의를 바꾸는 사업적 선택이 필요함
- 동결 계약 결함으로 공식 campaign 재실행이 필요함

그 외 candidate 구현, synthetic fixture, Reference Agent, fault/load injection, 측정,
분석과 보고서는 자동 진행한다. 승자와 ASR 사업 우선순위는 사용자가 결정한다.

## 10. 진행표

| DP | Decision audit | 19-QA audit | Implementation | Qualification | Campaign | Report | 상태 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| VIA-DP-06 | 완료 | 완료 | v4 breadth | 완료 | 1회 breadth | 예비 보고서 | INTEGRATED_REFERENCE_BREADTH_ONLY |
| VIA-DP-11 | 완료 | 완료 | targeted v4 | 완료 | process/fault 일부 | 예비 보고서 | TARGETED_PROCESS_EVIDENCE |
| VIA-DP-14 | 완료 | 사고실험 있음 | 두 writer primitive | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| VIA-DP-09 | 완료 | 완료 | Python component | 완료 | preliminary | 예비 보고서 | PRELIMINARY_COMPONENT_ONLY |
| VIA-DP-02 | 완료 | 완료 | Python targeted v2 | 완료 | partial | 예비 보고서 | TARGETED_PARTIAL |
| VIA-DP-08 | 완료 | 사고실험 있음 | recovery primitive | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| VIA-DP-13 | 완료 | 완료 | Python targeted | 완료 | partial | 예비 보고서 | TARGETED_PARTIAL |
| VIA-DP-05 | 완료 | 완료 | Python component | 완료 | preliminary | 예비 보고서 | PRELIMINARY_COMPONENT_ONLY |
| VIA-DP-12 | 완료 | 완료 | Python targeted | 완료 | partial | 예비 보고서 | TARGETED_PARTIAL |
| VIA-DP-01,03,04,07,10,15~18 | 완료 | 사고실험 있음 | 미구현 | 대기 | 대기 | 대기 | AUDIT_REQUIRED |

## 11. 실행 로그

| 시각 (KST) | 항목 | 결과 | 다음 작업 |
| --- | --- | --- | --- |
| 2026-09-27 | 중간 VIA-DP-06 파일·결과 정리 | 완료; v4 단일 source set만 유지 | 전체 실행 계획 작성 |
| 2026-09-27 | VIA-DP-01~18 연속 평가 계획 작성 | 완료 | VIA-DP-06 qualification |
| 2026-09-27 | VIA-DP-06 v4 static/Rust/실경로 qualification | 통과; 3 후보의 model·Agent·audio endpoint와 독립 replay 확인 | 공식 72-trial campaign |
| 2026-09-27 | VIA-DP-06 19-QA 참여 감사 | 측정 8, 물리 비참여 N/A 5, 유효 계약 미비 BLOCKED 6 | 거짓 proxy 없이 상태를 보고서에 고정 |
| 2026-09-27 | VIA-DP-11 v4 fail-closed 재구성 | 완료; 기존 proxy/부분 predicate를 QA 값에서 제외 | VIA-DP-06 뒤 qualification·공식 실행 |
| 2026-09-27 | VIA-DP-11 Process/IPC/fatal qualification | 통과; lifecycle transport, worker restart, symmetric blast-radius sentinel 확인 | VIA-DP-06 뒤 공식 campaign |
| 2026-09-27 | VIA-DP-06 v4 공식 72-trial campaign | 완료; raw 72건, trace 누락 0, 독립 replay 100% | 19-QA report 감사 |
| 2026-09-27 | VIA-DP-06 failure-time 분석 규칙 교정 | audible response가 있는 wrong-route case의 실제 시간을 raw에서 재계산; correctness 실패는 유지 | 실행 재수행 없이 contract/manifest에 old/new digest와 사유 기록 |
| 2026-09-27 | VIA-DP-11 v4 첫 공식 시도 | INVALID; 정상 runtime 뒤 fault phase 첫 submit이 timeout, QA 결과 생성 전 중단 | normal/fault phase의 Agent process 수명 분리 후 동일 contract로 1회 재실행 |
| 2026-09-27 | VIA-DP-11 v4 두 번째 시도 | INVALID; fault query timeout이 campaign을 중단시켜 실패 raw가 소실됨 | recovery 미완료를 30초 timeout raw로 남기고 다음 trial을 계속하도록 fail-closed 수정 |
| 2026-09-27 | VIA-DP-11 v4 세 번째 시도 | INVALID; normal 169-run persistent Agent state의 전체 저장 비용이 fault submit 5초 timeout을 초과 | 독립 fault stratum을 같은 Agent 계약의 fresh state로 분리하여 history-size 오염 제거 |
| 2026-09-27 | VIA-DP-11 v4 네 번째 시도 | INVALID; 두 후보 fault run을 한 state에 누적해 150 run에서 reference JSON 저장 timeout | 후보별 동일한 fresh Agent state로 분리하고 phase raw 즉시 저장 적용 |
| 2026-09-27 | VIA-DP-11 v4 공식 campaign | 완료; QA-31/32/41/61/62 측정, 독립 replay와 19-QA package 검사 통과 | 나머지 DP readiness와 최종 요약 정리 |
| 2026-09-27 | VIA-DP-02/08/09/13/14 구현 자산 감사 | 공통 primitive는 존재하나 DP별 동결 후보·전체 oracle 없음 | smoke를 공식 QA로 승격하지 않고 후보 구현부터 진행 |
| 2026-09-27 | VIA-DP-05/09/02/13/12 executable reference 후보·contract 구현 | qualification 23 tests PASS; 19개 QA map·배타 trace 확인 | official campaign 실행 |
| 2026-09-27 | VIA-DP-05/09/13/12 v1 official campaign | 각각 200/450/60/200 raw trial, 독립 replay·package 검사 PASS | VIA-DP-02 endpoint 감사 |
| 2026-09-27 | VIA-DP-02 v1 endpoint 감사 | INVALID; QA-05에 DB 준비 포함, QA-31 reopen 누락 | v1 archive 후 v2 contract·candidate 수정 |
| 2026-09-27 | VIA-DP-02 v2 official campaign | 120 raw trial, 실제 reopen recovery, 독립 replay·package 검사 PASS | 전체 결과 문서 반영 |
| 2026-09-27 | 전 결과 품질 재감사 | 7개 결과 모두 최종 `COMPLETE` 기준 미달. package 완결성과 Architecture 의사결정 증거를 혼동한 상태를 정정 | 공통 통합 하네스와 DP-06 v5 구축 |

## 12. 다음 단일 작업

### 2026-09-27 사용자 지시 정정

준비도 분석에서 멈추지 않는다. 먼저 DP-06 v5를 공통 통합 campaign의 reference
implementation으로 만든다. 이후 같은 runner, trace schema와 evaluator를 재사용하여
DP-11 → 14 → 09 → 02 → 08 → 13 순서로 한 축씩 교체한다.

1. VIA-DP-05 — 닫힌 Context manifest 대 확장 가능한 scoped read-set
2. VIA-DP-09 — edge semantic normalization 대 Core lifecycle handler
3. VIA-DP-02 — 공동 관계 commit 대 독립 commit+reconciliation
4. VIA-DP-13 — 회수 가능한 control reservation 대 완전 공유 priority pool
5. VIA-DP-12 — 최소 evidence durable ACK 선행 대 비동기 flush

DP-06 v5가 qualification과 독립 replay를 통과하기 전에는 다른 DP의 공식 campaign을
시작하지 않는다. 다만 candidate 구현과 applicability audit은 병행할 수 있다. v5가
통과하면 기존 component experiment의 숫자를 재사용하지 않고 통합 경로에서 다시 측정한다.
