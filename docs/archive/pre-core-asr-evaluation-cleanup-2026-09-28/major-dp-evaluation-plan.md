# 주요 VIA-DP 연속 평가 실행 계획

> 상태: **ACTIVE CORE-DP EVALUATION PLAN — Measurement Contract Definition**
> 시작일: 2026-09-27
> 목표: Core DP 6개를 한 번에 하나씩 심층 정의·구현·측정하고, VIA-DP-01~18 전체 inventory의 공통 계약·회귀·추적성을 유지한다.

이 문서는 현재 Core DP 작업의 실행 원장이다. 채팅 기억이 아니라 [Current Architecture Focus](../12-decisions/dp-executive-summary.md), 이 파일의 상태와 체크리스트를 기준으로 진행하며 각 milestone 뒤 진행표와 실행 로그를 갱신한다.

## 1. 불변 규칙

1. **모든 DP의 기본 평가표는 core ASR QA-09·19·29·39 전체다.** 상세 QA input·diagnostic·qualification도 삭제하지 않는다.
2. 각 core ASR은 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED` 중 하나와 근거를 가져야 한다.
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
14. 모든 보고서는 A/B 및 허용된 tactic 후보를 포함한 four-core-ASR complete table과 상세 evidence를 가진다.
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
3. 네 core ASR을 모두 감사하고 각 셀에 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED`와 근거를 남긴다.
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

이 기준을 모두 충족하기 전에는 보고서가 네 core ASR 행을 가진 경우에도 `COMPLETE`가 아니다.

## 3. DP별 공통 lifecycle

1. **Decision audit:** A/B 상호 배타성, steelman, Architecture 차이와 tactic 구분
2. **Core-ASR audit:** 네 core ASR의 모집단과 A/B 차이 참여 경로, 상세 evidence 작성
3. **Candidate implementation:** component/process/message/state 수준 A/B 구현
4. **Fixture/oracle freeze:** 정상 workload 우선, 필요한 fault/load/change pack만 추가
5. **Qualification:** 정상 trace PASS, mutation sentinel은 예상 failure code로 FAIL
6. **Official campaign:** 새 result directory, 실패 포함 raw 즉시 append
7. **Independent analysis:** 별도 process가 raw summary digest를 재현
8. **Report/audit:** 네 core ASR 표, 상세 지표, 원인·한계, tests/link/terminology/diff 검사

## 4. 공통 core ASR 4개

| 핵심 질문 | Core ASR | 상세 input/diagnostic |
| --- | --- | --- |
| 빠른가? | QA-09 | QA-01/02/03/05; QA-04 회귀 |
| 정확한가? | QA-19 | QA-11/12 field; QA-13~15 회귀 |
| 변화에 강한가? | QA-29 | QA-21~23 change ledger |
| 장애에도 살아남는가? | QA-39 | QA-31/32 fault evidence |

QA-41은 memory diagnostic, QA-51과 action/access rule은 safety qualification,
QA-61/62는 evidence qualification이다. core ASR과 합산한 단일 후보 점수는 만들지 않는다.

## 5. 실행 순서

### 5.1 확정한 Core DP set

2026-09-28 사용자 결정으로 다음 여섯 DP를 현재 보고서와 평가의 Core DP set으로 확정했다. 실행은 사용자 interaction 흐름에 맞춘 아래 순서를 기본으로 하며 한 번에 한 DP의 Measurement Freeze를 완성한다.

1. VIA-DP-03 — Voice input evidence의 local 대 S2S-native authority
2. VIA-DP-05 — Plan-first 대 demand-driven Context 획득
3. VIA-DP-17 — canonical 대 consumer-specific Context 표현
4. VIA-DP-06 — semantic 최종 확정 권한
5. VIA-DP-07 — node-level 대 owner-affinity bundle edge authority
6. VIA-DP-15 — continuous Agent progress projection 대 on-demand snapshot

### 5.2 Non-core·supporting DP

- VIA-DP-02·12는 Core DP에서 제외하고 supporting 계약과 기존 evidence를 보존한다.
- VIA-DP-11은 6개 실패 시의 대체 후보가 아니다. 기존 v4 evidence와 v5 runner 작업은 보존하며, 실제 native/plugin/fatal-risk dependency가 제품 범위에 새로 들어올 때만 별도 scope decision으로 다시 검토한다.
- 나머지 VIA-DP는 전체 inventory, fixed context와 regression/qualification 대상으로 유지한다. 핵심에서 빠졌다는 이유로 요구·안전·복구 의무를 삭제하지 않는다.

## 6. 현재 증거 수준과 다음 campaign

| DP | 보존된 결과 경로 | 현재 증거 수준 | 다음 작업 |
| --- | --- | --- | --- |
| VIA-DP-06 | `via-dp-06-evaluation-v6-20260927/` | ANALYSIS_COMPLETE_WITH_NON_QA20_BLOCKED_AXES | QA-21~23 보완 완료; QA-15/31/41 후속 endpoint·반복 필요 |
| VIA-DP-11 | `via-dp-11-evaluation-v4-20260927/` | V5_RUNNER_READY / v4는 TARGETED_PROCESS_EVIDENCE | runner 보존; 실제 A2A/ACP client fatal-risk inventory 전 신규 핵심 campaign 보류 |
| VIA-DP-14 | `via-dp-14-evaluation-v1-20260927/` | INTEGRATED_RUNNER_READY | 실제 Rust Task host, race/reopen, Voice/model/Agent campaign qualification 필요 |
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

VIA-DP-06도 네 core ASR 전부와 상세 evidence를 표에 포함한다. contract의 applicability는 qualification에서
실제 참여 경로를 재확인하며, 비참여가 입증된 모집단만 `NOT_APPLICABLE`로 확정한다.

## 8. DP별 산출물 완결 조건

- `manifest.json`: contract/fixture/oracle/source digest와 환경
- `raw/trials.jsonl`: 성공·실패·timeout 포함 원본
- `raw/change-exercises.json`: 실제 QA-21~23 change observation
- `summary.json`: raw 재계산 결과
- `replay-receipt.json`: 독립 analyzer digest 검증
- `report.md`: four-core-ASR complete table, 상세 지표, 원인과 evidence 한계
- `STATUS.md`: `COMPLETE`, `BLOCKED` 또는 `INVALID`와 근거

값이 없는 QA도 행을 삭제하지 않는다. `N/A` 또는 `BLOCKED`와 구체 사유를 쓴다.

## 9. 사용자 개입 정책

자동 실행 중에는 불필요한 사용자 개입을 요구하지 않는다. 다음 경우에만 해당 DP를 멈추고 상태 보고서에 명시한 뒤 독립적인 다음 작업으로 진행한다.

- 새 외부 계정·비용·credential이 필요함
- 실제 사용자 데이터나 비가역 외부 동작이 필요함
- A/B 정의를 바꾸는 사업적 선택이 필요함
- 동결 계약 결함으로 공식 campaign 재실행이 필요함

그 외 candidate 구현, synthetic fixture, Reference Agent, fault/load injection, 측정,
분석과 보고서는 자동 진행한다. 승자와 ASR 사업 우선순위는 사용자가 결정한다.

## 10. 보존된 구현·evidence 자산 현황

아래 표는 2026-09-27까지 만들어진 자산의 상태이며 현재 Core DP 우선순위가 아니다. 현재 작업 순서는 §5와 §12를 따른다. `완료`는 해당 열의 과거 작업 완료일 뿐 새 Core ASR campaign이나 Architecture winner 완료를 뜻하지 않는다.

| DP | Decision audit | Core-ASR audit | Implementation | Qualification | Campaign | Report | 상태 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| VIA-DP-06 | 완료 | 완료 | v5 통합 reference + v6 변경 원장 | 완료 | breadth 72 + latency 180 + locality 87 | 완료 | ANALYSIS_COMPLETE_WITH_NON_QA20_BLOCKED_AXES |
| VIA-DP-11 | 완료 | 완료 | v5 통합 runner 구현 | 대기 | process/fault v4만 | 예비 보고서 | V5_RUNNER_READY |
| VIA-DP-14 | 완료 | 완료 | 통합 Rust Task host·runner 구현 | 대기 | 대기 | 대기 | INTEGRATED_RUNNER_READY |
| VIA-DP-09 | 완료 | 완료 | Python component | 완료 | preliminary | 예비 보고서 | PRELIMINARY_COMPONENT_ONLY |
| VIA-DP-02 | 완료 | 완료 | Python targeted v2 | 완료 | partial | 예비 보고서 | TARGETED_PARTIAL |
| VIA-DP-08 | 완료 | 사고실험 있음 | recovery primitive | 대기 | 대기 | 대기 | PROTOTYPE_PRIMITIVES_ONLY |
| VIA-DP-13 | 완료 | 완료 | Python targeted | 완료 | partial | 예비 보고서 | TARGETED_PARTIAL |
| VIA-DP-05 | 완료 | 완료 | Python component | 완료 | preliminary | 예비 보고서 | PRELIMINARY_COMPONENT_ONLY |
| VIA-DP-12 | 완료 | 완료 | Python targeted | 완료 | partial | 예비 보고서 | TARGETED_PARTIAL |
| VIA-DP-01,03,04,07,10,15~18 | 완료 | 사고실험 있음 | 미구현 | 대기 | 대기 | 대기 | AUDIT_REQUIRED |

## 11. 실행 로그

이 표의 `다음 작업`은 각 행을 기록하던 당시의 계획이다. 2026-09-28 Core DP 확정 이후의 현재 다음 작업은 §12가 덮어쓴다.

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
| 2026-09-27 | VIA-DP-06 v5 계약·runner 동결 | 24-case correctness + QA-01/02/05 각 20회 physical-loopback 반복, source revision `a5bfadda` | 공식 campaign 실행 중 |
| 2026-09-27 | VIA-DP-11 v5 runner 구현 | local Qwen, generated input WAV, actual Rust shared/isolated host, external Agent, audible loopback, fault/memory/trace 결합 | DP-06 종료 후 qualification |
| 2026-09-27 | VIA-DP-14 executable Task host·runner 구현 | SharedTaskService/PerTaskSupervisors, actual race, DB reopen, model/Agent/audio path 결합 | DP-06·11 뒤 qualification |
| 2026-09-27 | VIA-DP-06 v5 공식 campaign·독립 replay | raw 252건+warm-up 18건, 후보별 84건, failure/trace 누락 0; A correctness 우세, B′ responsiveness 우세 | VIA-DP-11 v5 qualification |
| 2026-09-27 | VIA-DP-06 QA-21~23 v6 보완 | 기존 252건은 `source_execution_key`로 참조만 하고, 29개 변경×3 후보=87개 비누적 원장 실행·독립 replay; QA-23에서 A 우세 | VIA-DP-11 v5 qualification |

## 12. 다음 단일 작업

### 2026-09-28 Core DP 확정 반영

DP-06 v5/v6와 기존 reference package는 보존한다. 다음 작업은 runner를 관성적으로 확장하는 것이 아니라, **VIA-DP-03부터** 심층 A/B 정의와 pre-result Measurement Freeze를 완성하는 것이다. 이후 DP-05→17→06→07→15 순으로 진행한다.

- DP-05와 17은 먼저 source 획득 시점과 Context 표현 권한의 2×2 독립성을 확인한다.
- DP-07은 여러 Agent와 기존 Task owner를 보존하는 bundle partition·edge authority를 명세한다.
- DP-15는 사용자가 보지 않는 동안의 progress 관리까지 포함하고 QA-19 status field registry를 freeze한다.
- DP-03은 S2S capability qualification을 먼저 수행한다.
- DP-11 v5 작업은 보존하되 현재 Core DP campaign 순서에는 넣지 않는다.
- DP-02·12에는 새로운 core campaign을 우선 배정하지 않는다.
