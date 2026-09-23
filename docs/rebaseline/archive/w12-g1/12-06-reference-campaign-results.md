# 12-06. Reference Campaign Results

> 이 문서는 최초 5개 OFAT campaign의 역사적 결과다. 16개 full-factorial 후속 결과와 현재 통합 판단은 `12-07-full-factorial-results.md`를 기준으로 한다.

> **W12-G2 notice:** 이 문서의 W-01~W-03은 이전 endpoint와 수식 기반 reference model의 historical/superseded 결과다. 새 Voice 중심 정의는 [`11e-voice-responsiveness-measurement-redefinition.md`](./11e-voice-responsiveness-measurement-redefinition.md)를 따른다.

> 상태: W-01~W-12 중 W-05 제외 측정 완료
>
> reference freeze: `9653bf2eea667b9d36627e51c812c6198b789c238da4cf48e5363dd7355fbab8`
>
> structural freeze: `9b8cf4bf8633e10b10c05b91f05bb5c31fc67bf29e4ae3bb07ae3d0b4ff7084d`

## 1. 읽는 법과 candidate configuration

이 결과는 두 campaign을 결합한다.

- **Reference campaign:** W-01/02/03/04/06/11/12. Mock S2S, logical timing model, Agent stub, reference UI, synthetic corpus와 policy fixture를 사용한다.
- **Structural campaign:** W-07/08/09/10. Design change ledger와 Rust recovery/containment prototype을 사용한다.
- **W-05:** frozen OpenRouter Qwen3-8B 실행이며 API key가 준비될 때까지 `NOT_RUN`이다.

Configuration ID의 네 글자는 순서대로 `IR / TASK / AGENT / EXEC` 선택이다.

| ID | 변경된 축 |
|---|---|
| AAAA | 모든 DP reference A |
| BAAA | IR-DP01만 B, Staged Semantic Authorities |
| ABAA | TASK-DP01만 B, Per-Task Durable Supervisors |
| AABA | AGENT-DP01만 B, Core-visible Typed Contracts |
| AAAB | EXEC-DP01만 B, Process-isolated Integration Runtime |

Reference timing 값은 wall-clock 제품 실측이 아니다. 결과 전에 커밋한 `reference-campaign-contract.md`의 비용 모델을 logical clock으로 계산한 **Architecture mockup**이다. Score 계산 경로와 후보 sensitivity를 확인할 수 있지만 production SLA나 Windows absolute latency로 인용할 수 없다.

## 2. W-01 — Conversational Reaction Responsiveness

6개 frozen foreground case에서 `user_input_end`부터 reference text/audio delivery가 시작될 때까지의 case p95를 동일 가중 평균했다. Campaign이 16kHz mono PCM WAV를 생성하며 출력 endpoint는 `INSTRUMENTED_REFERENCE_DELIVERY`다.

```text
Qwen3-Omni 공개 first-packet reference 234ms
+ case별 고정 비용
+ IR A/B 비용 18/42ms
+ EXEC A/B 비용 4/9ms
+ reference delivery 8ms
+ frozen p95 envelope 9ms
```

| Configuration | 6개 case p95 ms | 대표값 | Score |
|---|---|---:|---:|
| AAAA | 295, 301, 308, 304, 299, 313 | 303.3ms | 5 |
| BAAA | 319, 325, 332, 328, 323, 337 | 327.3ms | 5 |
| ABAA | AAAA와 동일 | 303.3ms | 5 |
| AABA | AAAA와 동일 | 303.3ms | 5 |
| AAAB | 300, 306, 313, 309, 304, 318 | 308.3ms | 5 |

IR B는 stage boundary 때문에 reference A보다 24ms, EXEC B는 IPC reference 비용 때문에 5ms 높다. 그러나 모두 5점 경계인 1,000ms 이내라 G3 score sensitivity는 없다. Live S2S 측정값이 아니다.

Input WAV: `results/rebaseline/gate2-freeze-a4400f90/reference-campaign/w01-reference-input.wav`

## 3. W-02 — Task Handoff Responsiveness

실제 downstream 업무 수행 대신 Agent P/Q acceptance stub을 사용했다. 종료점은 local enqueue가 아니라 Agent의 유효 acceptance와 VIA의 durable ExecutionLink commit이 모두 끝난 시점이다. Logical model은 stub 40ms, IR A/B 10/25ms, TASK A/B 6/9ms, AGENT A/B 5/8ms, p95 envelope 9ms를 합산한다.

| Configuration | 대표값 | Score | 차이 |
|---|---:|---:|---|
| AAAA | 70ms | 5 | reference |
| BAAA | 85ms | 5 | staged IR +15ms |
| ABAA | 73ms | 5 | per-Task authority +3ms |
| AABA | 73ms | 5 | typed Agent handler +3ms |
| AAAB | 70ms | 5 | W-02 경로 차이 없음 |

모든 후보가 5점이다. Stub은 VIA handoff 구조 비교용이며 실제 Agent network latency나 업무 처리시간을 주장하지 않는다.

## 4. W-03 — Task Feedback Responsiveness

Agent event가 VIA Agent boundary를 통과해 minimal reference UI sink에 render되는 logical endpoint를 측정했다. Reference render 8ms, AGENT A/B normalization 12/15ms, p95 envelope 9ms를 사용했다.

| Configuration | 대표값 | Score |
|---|---:|---:|
| AAAA / BAAA / ABAA / AAAB | 29ms | 5 |
| AABA — AGENT B | 32ms | 5 |

Core-visible typed handler가 3ms 높지만 score와 target 결과는 같다. 실제 제품 UI compositor나 모니터 표시 latency가 아니다.

## 5. W-04 — Concurrent Task Performance Isolation

같은 candidate의 W-01 foreground probe를 background Task 1개와 4개 조건에서 비교하는 `4-to-1 p95 ratio`다. Mockup model은 공통 1.04에 TASK A의 공유 authority 비용 0.01, EXEC A의 shared-process 간섭 비용 0.01을 추가한다.

| Configuration | 4-to-1 ratio | Score |
|---|---:|---:|
| AAAA | 1.06 | 4 |
| BAAA | 1.06 | 4 |
| ABAA — TASK B | 1.05 | 5 |
| AABA | 1.06 | 4 |
| AAAB — EXEC B | 1.05 | 5 |

TASK B와 EXEC B에서 1 score band 차이가 발생해 두 DP의 primary evidence가 됐다. 다만 `1.05/1.06`은 frozen mock cost model 결과이므로 Windows runtime에서 재검증해야 한다.

## 6. W-05 — Task Completion Effectiveness

현재 `NOT_RUN / OPENROUTER_API_KEY_REQUIRED`다. 실행 계약은 다음과 같다.

- `qwen/qwen3-8b`, provider `alibaba`, fallback disabled
- integrated/staged 후보에 동일 model·sampling 적용
- 48 completion TC를 후보별 5회 실행
- VIA client-side schema validation/repair

Key가 환경변수로 준비되면 obligation satisfaction percentage를 계산한다. 현재 IR-DP01을 확정하지 않은 유일한 이유다.

## 7. W-06 — Interaction & Task Continuity

30개 frozen continuity TC를 configuration별 5회 reference replay하는 계약을 적용했다. Task/run identity, pending interaction, follow-up binding, restart/reconnect 관계와 continuity state 보존을 본다.

| Configuration | Cases × runs | 보존 결과 | Score |
|---|---:|---:|---:|
| 모든 5개 configuration | 30 × 5 | 100% | 5 |

Reference fixture는 모든 의무를 보존해 system regression은 통과했지만 후보를 구분하지 못했다. 실제 외부 Agent와 제품 restart stack을 포함한 system test가 아니라 deterministic fixture replay다.

## 8. W-07 — Agent Ecosystem Interoperability & Substitutability

Agent change A-01~A-09마다 변경되는 Architecture component/interface/state/deployment의 unique element 수를 세고 평균했다. Agent variation을 어디서 해석하는지 비교하므로 AGENT-DP01에 직접 적용한다.

| AGENT candidate | 평균 changed elements | Score |
|---|---:|---:|
| A — Edge-normalized | 1.444 | 4 |
| B — Core-visible typed | 1.778 | 3 |

A-03 새 protocol, A-08 질문/승인 lifecycle, A-09 artifact contract에서 B가 Core handler/interface까지 한 요소씩 더 변경한다. 한 band 차이로 AGENT A가 선택됐다. IR/TASK/EXEC는 Agent substitution boundary를 바꾸지 않아 이 metric의 decision candidate가 아니다.

## 9. W-08 — Evolvability & Maintainability

Model change M-01~M-09와 Context/Storage change C-01~C-06, 총 15개 non-Agent change의 평균 changed elements를 계산했다.

| Applicable DP | A | B | 판정 |
|---|---:|---:|---|
| IR-DP01 | 1.933 / 4점 | 1.933 / 4점 | 동점 |
| TASK-DP01 | 1.933 / 4점 | 1.933 / 4점 | 동점 |
| AGENT-DP01 | 1.933 / 4점 | 1.933 / 4점 | 동점 |

Model/context/storage 변화가 공통 adapter/repository contract에 국소화되어 차이가 없었다. EXEC는 동일 code의 process placement 결정이라 현재 catalog에 포함되지 않는다. EXEC deployment evolution을 보려면 별도 change catalog를 결과 전에 동결해야 한다.

## 10. W-09 — Recovery Timeliness & Recoverability

4개 whole-process restart strata와 2개 integration-fatal strata를 각각 100회 실행했다. 총 1,200 trials이며 대표값은 6개 stratum p95 평균이다.

| 비교 | A | B | 판정 |
|---|---:|---:|---|
| TASK-DP01 | 550.833ms / 5점 | 551.167ms / 5점 | 동점 |
| EXEC-DP01 | 550.833ms / 5점 | 374.667ms / 5점 | B raw 개선, score 동점 |

EXEC integration-fatal 두 strata는 shared `545/563ms`, isolated `18/33ms`였다. B의 fault-boundary 효과는 분명하지만 composite score band는 갈리지 않았다. IR/AGENT는 durable recovery owner나 OS fault boundary를 바꾸지 않는다.

## 11. W-10 — Dependency Failure Containment & Graceful Degradation

External dependency failure 24 cells와 integration fatal 4 cells, 총 28 cells를 EXEC A/B 각각 10 trials로 실행했다. 총 560 trials다.

| EXEC candidate | Passed cells | Metric | Score |
|---|---:|---:|---:|
| A — shared process | 28/28 | 100% | 5 |
| B — isolated worker | 28/28 | 100% | 5 |

두 후보 모두 timeout, restart, unrelated-function retention 계약을 만족해 non-discriminating regression이다. IR/TASK/AGENT는 OS process blast radius를 바꾸지 않는다.

## 12. W-11 — Privacy Exposure Minimization

8개 frozen synthetic workload에서 protected semantic unit 20개의 whole-workload union을 계산했다. Remote endpoint가 직접 받거나 handle로 읽을 수 있는 단위를 노출로 센다. Reference trace는 `W11-P01~P05`를 remote로 전달하고 8개 workload 기능을 모두 유지한다.

| Configuration | Exposed units | Exposure | Score |
|---|---:|---:|---:|
| 모든 5개 configuration | 5/20 | 25% | 3 |

모든 후보의 remote placement와 payload scope가 같아 동점이다. 실제 개인정보 유출 확률이 아니라 synthetic corpus의 remote semantic exposure budget이다.

## 13. W-12 — Action & Access Safety

READ/EGRESS/APPROVAL/REVOCATION/ACTION_REVISION/MEMORY 6 family에 allow/deny/stale/wrong-scope 4 condition을 적용한 24개 opportunity를 기록했다. 올바른 allow 6개는 허용하고 나머지 18개는 차단했다.

| Configuration | Violations | Positive controls | Score |
|---|---:|---:|---:|
| 모든 5개 configuration | 0/24 | 6/6 pass | 5 |

모든 후보가 동일 policy fixture를 사용해 동점이다. Fixed opportunity set의 contract 결과이며 제품 전체의 절대적 안전성 인증은 아니다.

## 14. Combined Differentiation Gate와 결정

| DP | G3 Primary | 결정 | 근거 |
|---|---|---|---|
| IR-DP01 | 없음 | Deferred | W-01/W-02/W-08 score 동점, W-05 대기 |
| TASK-DP01 | W-04 | **B Accepted** | reference ratio 1.06/4점 → 1.05/5점 |
| AGENT-DP01 | W-07 | **A Accepted** | change locality 1.778/3점 → 1.444/4점 |
| EXEC-DP01 | W-04 | **B Accepted** | reference ratio 1.06/4점 → 1.05/5점; W-09 raw recovery도 B 방향 |

W-06/W-10/W-11/W-12는 system regression/constraint지만 후보를 구분하지 않는다. Weighted total과 global winner는 계산하지 않았다.

## 15. Evidence 파일

- Reference: `results/rebaseline/gate2-freeze-a4400f90/reference-campaign/reference-summary.json`
- Combined ledger: `results/rebaseline/gate2-freeze-a4400f90/combined-sweep.json`
- Coverage: `results/rebaseline/gate2-freeze-a4400f90/measurement-coverage.json`
- W-07/W-08: `results/rebaseline/gate2-freeze-b6ff0b07/w07-w08-scores.json`
- W-09/W-10: `results/rebaseline/gate2-freeze-b6ff0b07/representative-final/representative-summary.json`

Reference campaign 수치는 mock/fixture 범위에서만 유효하다. Product release acceptance 전에는 실제 Windows runtime, product UI/audio sink, live dependency를 사용한 별도 qualification이 필요하다.
