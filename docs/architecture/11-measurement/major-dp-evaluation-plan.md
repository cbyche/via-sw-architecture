# 주요 VIA-DP 연속 평가 실행 계획

> 상태: **ACTIVE PLAN — 공식 campaign 실행 전**
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

## 2. DP별 공통 lifecycle

1. **Decision audit:** A/B 상호 배타성, steelman, Architecture 차이와 tactic 구분
2. **19-QA audit:** 19개 각각의 endpoint와 A/B 차이 참여 경로 작성
3. **Candidate implementation:** component/process/message/state 수준 A/B 구현
4. **Fixture/oracle freeze:** 정상 workload 우선, 필요한 fault/load/change pack만 추가
5. **Qualification:** 정상 trace PASS, mutation sentinel은 예상 failure code로 FAIL
6. **Official campaign:** 새 result directory, 실패 포함 raw 즉시 append
7. **Independent analysis:** 별도 process가 raw summary digest를 재현
8. **Report/audit:** 19-QA 표, 보조 지표, 원인·한계, tests/link/terminology/diff 검사

## 3. 공통 QA 19개

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

## 4. 실행 순서

### 4.1 1차 핵심 DP

사용자와 합의한 순서로 먼저 처리한다.

1. VIA-DP-06 — semantic 최종 확정 권한
2. VIA-DP-11 — 외부 연동 Process 장애 경계
3. VIA-DP-14 — Task 상태 writer
4. VIA-DP-09 — Agent 의미 해석 위치
5. VIA-DP-02 — Conversation–Task 확정 경계
6. VIA-DP-08 — 복구 기준 기록
7. VIA-DP-13 — control 자원 예약

### 4.2 후속 전체 DP

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

## 5. 공식 campaign 이름과 현재 준비 상태

| DP | 공식 결과 경로 | 현재 상태 | 다음 작업 |
| --- | --- | --- | --- |
| DP-06 | `dp06-evaluation-v4-20260927/` | COMPLETE | 결과 감사·요약 반영 완료 |
| DP-11 | `dp11-evaluation-v4-20260927/` | COMPLETE | 결과 감사·요약 반영 완료 |
| DP-14 | `dp14-evaluation-v1-20260927/` | PROTOTYPE_PRIMITIVES_ONLY | 두 Task writer는 있으나 frozen race/load campaign 필요 |
| DP-09 | `dp09-evaluation-v1-20260927/` | PROTOTYPE_PRIMITIVES_ONLY | 두 adapter는 있으나 full Agent change pack 실행 필요 |
| DP-02 | `dp02-evaluation-v1-20260927/` | PROTOTYPE_PRIMITIVES_ONLY | handoff invariant는 있으나 공동 commit/reconciliation 후보가 불완전 |
| DP-08 | `dp08-evaluation-v1-20260927/` | PROTOTYPE_PRIMITIVES_ONLY | recovery primitive는 있으나 두 복구 원본 후보 campaign 미구현 |
| DP-13 | `dp13-evaluation-v1-20260927/` | PROTOTYPE_PRIMITIVES_ONLY | load driver는 있으나 두 자원 예약 scheduler 후보 미구현 |
| DP-01,03~05,07,10,12,15~18 | `dpNN-evaluation-v1-20260927/` | AUDIT_REQUIRED | 1차 DP 후 importance audit |

`IMPLEMENTATION_REQUIRED`는 blocker나 결론이 아니다. 문서 후보를 실제 executable
candidate와 evaluator로 옮겨야 한다는 현재 상태다.

## 6. DP-06 동결 실행 카드

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
- Repetition: case당 1회 breadth. production p95라고 부르지 않음
- 예상 실행: 약 30~40분

DP-06도 19개 QA 전부를 표에 포함한다. contract의 applicability는 qualification에서
실제 참여 경로를 재확인하며, 비참여가 입증된 QA만 `N/A`로 확정한다.

## 7. DP별 산출물 완결 조건

- `manifest.json`: contract/fixture/oracle/source digest와 환경
- `raw/trials.jsonl`: 성공·실패·timeout 포함 원본
- `raw/change-exercises.json`: 실제 QA-21~23 change observation
- `summary.json`: raw 재계산 결과
- `replay-receipt.json`: 독립 analyzer digest 검증
- `report.md`: 19-QA complete table, 보조 지표, 원인과 evidence 한계
- `STATUS.md`: `COMPLETE`, `BLOCKED` 또는 `INVALID`와 근거

값이 없는 QA도 행을 삭제하지 않는다. `N/A` 또는 `BLOCKED`와 구체 사유를 쓴다.

## 8. 사용자 개입 정책

밤사이 사용자 개입은 요구하지 않는다. 다음 경우에만 해당 DP를 멈추고 아침 보고서에
명시한 뒤 독립적인 다음 DP로 진행한다.

- 새 외부 계정·비용·credential이 필요함
- 실제 사용자 데이터나 비가역 외부 동작이 필요함
- A/B 정의를 바꾸는 사업적 선택이 필요함
- 동결 계약 결함으로 공식 campaign 재실행이 필요함

그 외 candidate 구현, synthetic fixture, Reference Agent, fault/load injection, 측정,
분석과 보고서는 자동 진행한다. 승자와 ASR 사업 우선순위는 사용자가 결정한다.

## 9. 진행표

| DP | Decision audit | 19-QA audit | Implementation | Qualification | Campaign | Report | 상태 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| DP-06 | 완료 | 완료 | 완료 | 완료 | 완료 | 완료 | COMPLETE |
| DP-11 | 완료 | 완료 | targeted v4 완료 | 완료 | 완료 | 완료 | COMPLETE |
| DP-14 | 완료 | 사고실험 있음 | 두 writer primitive | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| DP-09 | 완료 | 사고실험 있음 | 두 adapter primitive | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| DP-02 | 완료 | 사고실험 있음 | handoff primitive | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| DP-08 | 완료 | 사고실험 있음 | recovery primitive | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| DP-13 | 완료 | 사고실험 있음 | load driver만 있음 | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| DP-01,03~05,07,10,12,15~18 | 완료 | 사고실험 있음 | 미구현 | 대기 | 대기 | 대기 | AUDIT_REQUIRED |

## 10. 실행 로그

| 시각 (KST) | 항목 | 결과 | 다음 작업 |
| --- | --- | --- | --- |
| 2026-09-27 | 중간 DP-06 파일·결과 정리 | 완료; v4 단일 source set만 유지 | 전체 실행 계획 작성 |
| 2026-09-27 | VIA-DP-01~18 연속 평가 계획 작성 | 완료 | DP-06 qualification |
| 2026-09-27 | DP-06 v4 static/Rust/실경로 qualification | 통과; 3 후보의 model·Agent·audio endpoint와 독립 replay 확인 | 공식 72-trial campaign |
| 2026-09-27 | DP-06 19-QA 참여 감사 | 측정 8, 물리 비참여 N/A 5, 유효 계약 미비 BLOCKED 6 | 거짓 proxy 없이 상태를 보고서에 고정 |
| 2026-09-27 | DP-11 v4 fail-closed 재구성 | 완료; 기존 proxy/부분 predicate를 QA 값에서 제외 | DP-06 뒤 qualification·공식 실행 |
| 2026-09-27 | DP-11 Process/IPC/fatal qualification | 통과; lifecycle transport, worker restart, symmetric blast-radius sentinel 확인 | DP-06 뒤 공식 campaign |
| 2026-09-27 | DP-06 v4 공식 72-trial campaign | 완료; raw 72건, trace 누락 0, 독립 replay 100% | 19-QA report 감사 |
| 2026-09-27 | DP-06 failure-time 분석 규칙 교정 | audible response가 있는 wrong-route case의 실제 시간을 raw에서 재계산; correctness 실패는 유지 | 실행 재수행 없이 contract/manifest에 old/new digest와 사유 기록 |
| 2026-09-27 | DP-11 v4 첫 공식 시도 | INVALID; 정상 runtime 뒤 fault phase 첫 submit이 timeout, QA 결과 생성 전 중단 | normal/fault phase의 Agent process 수명 분리 후 동일 contract로 1회 재실행 |
| 2026-09-27 | DP-11 v4 두 번째 시도 | INVALID; fault query timeout이 campaign을 중단시켜 실패 raw가 소실됨 | recovery 미완료를 30초 timeout raw로 남기고 다음 trial을 계속하도록 fail-closed 수정 |
| 2026-09-27 | DP-11 v4 세 번째 시도 | INVALID; normal 169-run persistent Agent state의 전체 저장 비용이 fault submit 5초 timeout을 초과 | 독립 fault stratum을 같은 Agent 계약의 fresh state로 분리하여 history-size 오염 제거 |
| 2026-09-27 | DP-11 v4 네 번째 시도 | INVALID; 두 후보 fault run을 한 state에 누적해 150 run에서 reference JSON 저장 timeout | 후보별 동일한 fresh Agent state로 분리하고 phase raw 즉시 저장 적용 |
| 2026-09-27 | DP-11 v4 공식 campaign | 완료; QA-31/32/41/61/62 측정, 독립 replay와 19-QA package 검사 통과 | 나머지 DP readiness와 최종 요약 정리 |
| 2026-09-27 | DP-02/08/09/13/14 구현 자산 감사 | 공통 primitive는 존재하나 DP별 동결 후보·전체 oracle 없음 | smoke를 공식 QA로 승격하지 않고 후보 구현부터 진행 |

## 11. 다음 단일 작업

DP-14의 두 Task writer primitive를 같은 외부 계약으로 감싸고, race/load fixture와
machine oracle을 먼저 동결한다. sentinel qualification을 통과하기 전에는 공식 결과
directory를 만들지 않는다. 이후 순서는 DP-09 → DP-02 → DP-08 → DP-13이며, 각 DP는
후보·oracle 미구현 상태를 unit-test 숫자로 대신하지 않는다.
