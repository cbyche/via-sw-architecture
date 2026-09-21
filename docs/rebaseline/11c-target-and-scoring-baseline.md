# 11-C. Target & Scoring Baseline — 제품 목표와 0~5 평가 기준

> 상태: **사용자 리뷰용 초안**. Architecture 후보 결과를 보기 전에 작성.
> 입력: 08 확정 ASR, 09 mapping, 10 Architecture Element 기준, 사용자 승인 완료 11-A/B.
> 원칙: 같은 ASR은 모든 DP에서 동일 metric / target / score boundary를 사용한다.

## C.1 목적

11-C는 후보 Architecture의 결과를 보기 전에 다음을 고정한다.

1. 대표 Metric
2. 실행/반복 및 집계 규칙
3. Product Target
4. 0~5 Score Boundary
5. 실측/추정/설계분석 evidence level

`NOT_RUN`, `BLOCKED`, `분석 부족`은 Score 0이 아니다. 평가가 없다는 뜻이며 숫자로 대체하지 않는다.

## C.2 최종 7개 ASR 요약

| ASR | Representative Metric | Product Target | 방향 |
| --- | --- | ---: | --- |
| ASR-01 Responsiveness | 6 request-class Macro-p95 latency | **≤ 2.0 s** | 낮을수록 좋음 |
| ASR-02 Task Completion Effectiveness | Task Completion Obligation Satisfaction Rate | **≥ 95%** | 높을수록 좋음 |
| ASR-03 Interaction & Task Continuity | Continuity Obligation Preservation Rate | **≥ 95%** | 높을수록 좋음 |
| ASR-04 Agent Interoperability & Substitutability | Avg. changed architecture elements / Agent change | **≤ 2.0** | 낮을수록 좋음 |
| ASR-05 Evolvability & Maintainability | Avg. changed architecture elements / non-Agent change | **≤ 3.0** | 낮을수록 좋음 |
| ASR-06 Reliability & Recoverability | Reliability/recovery scenario pass rate | **100%** | 높을수록 좋음 |
| ASR-07 Privacy, Security & Action Safety | Safety violation rate V/24 | **0% (0/24)** | 낮을수록 좋음 |

ASR-01~05는 target을 Score 3 경계로 사용하여 target보다 우수한 구조를 4~5점으로 구분한다. ASR-06/07은 deterministic invariant suite이므로 제품 target이 무결점이며 Score 5만 target을 충족한다. 낮은 점수는 후보 비교를 위한 미달 정도이지 허용 기준이 아니다.

---

## C.3 ASR-01 — User-Experienced Responsiveness

### Metric

6개 request class 각각의 p95를 구한 뒤 동일 가중 평균한다.

    Macro-p95
    = mean(p95(L-01), ..., p95(L-06))

Voice interruption audio-stop latency는 대표값에서 제외한다.

### Measurement protocol

Measured mode:

- candidate별/latency class별 **10 warm-up + 100 scored runs**
- concurrency = 1
- Model Runtime은 scoring run 시작 전에 loaded/warm 상태
- cold start/model load는 secondary raw observation
- 동일 fixture, 동일 Model artifact/runtime profile, 동일 network fixture
- remote S2S primary network fixture는 RTT 50 ms, RTT 0/150 ms는 sensitivity only
- class p95는 nearest-rank empirical p95
- candidate 실행 순서는 가능한 경우 round-robin/interleaved하여 thermal/network drift 영향을 줄임
- class별 p50/p95도 raw evidence로 보존

100 scored runs는 p95 tail에 약 5개 관측이 남도록 하는 실용적인 최소 반복 budget이다. 이를 confidence interval 보장이라고 주장하지 않는다.

Estimated/design mode:

- 실제 prompt를 frozen tokenizer로 tokenize
- candidate의 실제 LLM/S2S call graph와 sequential/parallel 관계 사용
- Qwen3-8B Semantic Model public planning profile과 Qwen3-Omni S2S official reference 사용
- Context/IPC/RPC/network/validation/delivery span을 별도 합산
- 결과는 `ESTIMATED` 또는 `OFFICIAL_THEORETICAL_REFERENCE`로 표시
- estimated 값을 measured runtime p95라고 부르지 않음

### Target rationale

최근 conversational-interface 연구에서는 **4초를 넘는 응답 지연이 quality of experience를 저하시킨다**고 보고한다. 반면 최근 low-latency voice-agent 연구는 sub-second response를 실용적인 stretch goal로 제시한다. Qwen3-Omni 자체도 concurrency=1 theoretical first-audio-packet 234 ms를 보고한다.

따라서:

- **2.0 s**: 사용자와 대화의 흐름을 유지하면서 현재 VIA의 semantic/context/orchestration 비용까지 허용하는 product target
- **1.0 s**: state-of-the-art에 가까운 stretch level
- **4.0 s 초과**: conversational QoE degradation 영역

2.0 s는 외부 표준이 정한 숫자가 아니라 위 UX evidence와 VIA reference model budget 사이에서 정한 product-derived target이다.

근거:
- ACM CUI 2025 / Maslych et al.: https://arxiv.org/abs/2507.22352
- ChipChat low-latency voice agent: https://arxiv.org/abs/2509.00078
- Qwen3-Omni Technical Report: https://arxiv.org/abs/2509.17765

### Score

| Score | Macro-p95 | 해석 |
| ---: | ---: | --- |
| **5** | **≤ 1.0 s** | stretch / near-real-time |
| **4** | >1.0 – **≤1.5 s** | target보다 충분히 우수 |
| **3** | >1.5 – **≤2.0 s** | **target 충족** |
| **2** | >2.0 – ≤3.0 s | 경미~중간 미달 |
| **1** | >3.0 – ≤4.0 s | 사용자 체감 지연 큼 |
| **0** | **>4.0 s** | QoE degradation 영역 |

---

## C.4 ASR-02 — Task Completion Effectiveness

### Metric

48개 canonical TC 각각에서 ASR-02 atomic obligation의 충족률을 계산하고 TC별 값을 동일 가중 평균한다.

    ASR-02 = 100 × mean(TC obligation satisfaction)

Strict TC PASS/FAIL은 secondary evidence다.

### Execution protocol

- 동일 Model/runtime/prompt contract를 모든 후보에 사용
- Semantic Model은 non-thinking / deterministic decoding 설정 사용
- 실제 model evaluation 시 각 TC **5회 반복**
- TC degree = 5회 run degree의 평균
- strict TC PASS는 해당 TC의 모든 obligation이 5회 모두 충족된 경우로 별도 기록
- synthetic/design replay는 반복값을 실제 semantic accuracy로 주장하지 않음

### Target rationale

Task Completion은 VIA의 존재 이유에 직접 연결되는 QA다. 외부 표준에는 VIA와 같은 orchestration system의 universal success-rate target이 없으므로 숫자를 외부 benchmark에서 가져오지 않는다.

**95%**는 평균적으로 architecture-relevant completion obligation의 20개 중 19개 이상을 보존하는 수준을 product target으로 둔 것이다. 95% 미만에서는 grounding/request/task/agent/output 중 architecture 책임의 누락이 반복적으로 사용자-visible failure로 나타날 가능성이 있다고 본다.

Strict pass 원자료를 함께 공개하여 95% 평균이 일부 심각한 TC 실패를 숨기지 않도록 한다.

### Score

| Score | Obligation Satisfaction |
| ---: | ---: |
| **5** | **≥99%** |
| **4** | ≥97% – <99% |
| **3** | **≥95% – <97%** |
| **2** | ≥90% – <95% |
| **1** | ≥80% – <90% |
| **0** | **<80%** |

---

## C.5 ASR-03 — Interaction & Task Continuity

### Metric

30개 canonical TC에서 ASR-03 continuity obligation preservation을 계산하고 TC별 degree를 동일 가중 평균한다.

    ASR-03 = 100 × mean(TC continuity preservation)

### Execution protocol

- ASR-02와 동일한 5-repeat semantic execution rule
- modality/connection/task/run/artifact/pending-interaction의 identity를 trace에서 확인
- 같은 TC가 ASR-02와 겹치더라도 실행은 한 번, obligation은 ASR tag에 따라 별도 판정
- strict continuity PASS를 secondary evidence로 보존

### Target rationale

Continuity failure는 사용자가 말한 '아까 그거', 진행 중인 Task, Agent run, 결과물 등의 identity가 끊기는 현상이다. VIA의 핵심 제품 정의가 Conversation/Task/Agent 실행의 연속성을 담당하므로 **≥95%**를 최소 product target으로 둔다.

ASR-02와 같은 percentage band를 사용하지만 obligation의 의미는 다르다. 구조 대안이 필요한 state/correlation을 모두 보존하면 여러 후보가 100%를 받아도 정상이며, 그 경우 해당 DP에서는 ASR-03이 변별 QA가 아니다.

### Score

| Score | Continuity Preservation |
| ---: | ---: |
| **5** | **≥99%** |
| **4** | ≥97% – <99% |
| **3** | **≥95% – <97%** |
| **2** | ≥90% – <95% |
| **1** | ≥80% – <90% |
| **0** | **<80%** |

---

## C.6 ASR-04 — Agent Ecosystem Interoperability & Substitutability

### Metric / protocol

- A-01~09를 동일 candidate baseline에 각각 독립 적용
- 10의 C/I/S/D element ID와 modified ∪ added ∪ removed unique count 사용
- 기능 유지가 검증되지 않으면 change count는 유효 score가 아님
- 반복 측정 개념은 없으며 9개 change ledger를 모두 작성

### Target

**≤2.0 changed architecture elements / Agent change**

근거는 Agent-neutral VIA에서 provider/protocol-specific 변화가 integration seam에 국소화되어야 한다는 product boundary와 modifiability/change-locality 원칙이다.

### Score

| Score | Avg changed elements/change |
| ---: | ---: |
| **5** | **≤1.0** |
| **4** | >1.0 – ≤1.5 |
| **3** | **>1.5 – ≤2.0** |
| **2** | >2.0 – ≤3.0 |
| **1** | >3.0 – ≤4.0 |
| **0** | **>4.0** |

---

## C.7 ASR-05 — Evolvability & Maintainability

### Metric / protocol

- M-01~09 + C-01~06, 총 15개 change를 동일 candidate baseline에 독립 적용
- C/I/S/D changed elements를 동일 rule로 집계
- Model/Context/Persistent State 분야 raw 내역도 보존

### Target

**≤3.0 changed architecture elements / non-Agent change**

Model/Context provider 변화는 주로 0~2 elements에 국소화될 수 있지만, persistent-state evolution은 State schema + Repository + Migration responsibility의 3개 변경이 정당할 수 있다는 VIA change catalog를 근거로 한다.

### Score

| Score | Avg changed elements/change |
| ---: | ---: |
| **5** | **≤1.0** |
| **4** | >1.0 – ≤2.0 |
| **3** | **>2.0 – ≤3.0** |
| **2** | >3.0 – ≤4.0 |
| **1** | >4.0 – ≤5.0 |
| **0** | **>5.0** |

---

## C.8 ASR-06 — Reliability & Recoverability

### Metric

27개 canonical reliability/recovery TC의 strict scenario pass rate.

### Execution protocol

- deterministic Agent simulator + fault/event injection 사용
- 각 TC **5회 반복**
- 한 TC는 5회 모두 invariant를 만족해야 PASS
- race/out-of-order/duplicate/cancel/result/restart의 event schedule과 external state는 모든 후보에 동일
- restart는 실제 VIA process memory loss 후 persisted state + 살아 있는 Agent query로 복구

### Target rationale

이 27개는 자연환경의 확률적 장애 빈도를 추정하는 표본이 아니라 **알려진 deterministic failure condition에서 지켜야 하는 architecture invariant suite**다. 따라서 제품 target은 **100%**다.

예를 들어 duplicate state-changing Action, 잘못된 terminal state, 살아 있는 실행과의 재연결 실패에 '일부 성공'을 제품 목표로 허용하지 않는다.

### Score

| Score | Scenario pass rate | 27 TC 해석 |
| ---: | ---: | --- |
| **5** | **100%** | 27/27, **target 충족** |
| **4** | ≥96% – <100% | 26/27 |
| **3** | ≥92% – <96% | 25/27 |
| **2** | ≥85% – <92% | 23–24/27 |
| **1** | ≥70% – <85% | 19–22/27 |
| **0** | **<70%** | ≤18/27 |

---

## C.9 ASR-07 — Privacy, Security & Action Safety

### Metric

    Safety Violation Rate = 100 × V / 24

24 fixed opportunity를 사용한다.

### Execution protocol

- opportunity별 **5회 반복**
- 어느 한 반복에서라도 unauthorized/wrong-scope operation을 허용하면 해당 opportunity는 violation 1건
- retry/guard/log 개수로 N을 늘리지 않음
- valid allow 6개를 막은 경우 positive-control failure를 별도로 보존
- V/N과 opportunity ID별 raw result를 항상 공개

### Target rationale

24개 opportunity는 임의 자연어 정확도 표본이 아니라 READ, EGRESS, APPROVAL, REVOCATION, ACTION_REVISION, MEMORY의 명시적인 enforcement decision이다.

따라서 unauthorized read/egress/action 또는 잘못된 approval binding에 제품적으로 허용 가능한 violation budget을 두지 않고 **target=0/24**로 둔다.

사용자 결정에 따라 별도 hard gate는 만들지 않으며 0~5 score로 비교한다. 그러나 Score 0~4는 product target을 충족했다는 뜻이 아니다.

### Score

| Score | Violation |
| ---: | ---: |
| **5** | **0/24 = 0%** — target 충족 |
| **4** | 1/24 = 4.17% |
| **3** | 2/24 = 8.33% |
| **2** | 3/24 = 12.50% |
| **1** | 4/24 = 16.67% |
| **0** | **≥5/24 (>16.67%)** |

---

## C.10 Evidence level과 score 사용

| Evidence | Score 사용 | 표현 |
| --- | --- | --- |
| MEASURED | final verification 가능 | 실측 score |
| ACTUAL_MODEL_REPLAY | ASR-02/03 actual semantic evaluation | 실행 score |
| SYNTHETIC_REPLAY | 구조/state/fault logic 비교 | replay score, 실제 사용자 정확도로 과장 금지 |
| ESTIMATED | Architecture candidate planning 비교 가능 | **estimated score**라고 표시 |
| DESIGN_ANALYSIS | ASR-04/05 element ledger | design-analysis score |
| NOT_RUN / BLOCKED | score 금지 | N/A |

동일 DP 표에서 서로 다른 evidence level의 숫자를 섞을 경우 각 값 옆에 evidence level을 표시한다. Estimated 4점과 Measured 4점이 같은 증거 강도를 갖는다고 주장하지 않는다.

## C.11 12로 넘기는 고정 입력

12 Architecture Decision Point 작업은 다음을 변경하지 않고 사용한다.

- ASR 7개와 위 대표 Metric
- Product Target
- 0~5 Score Boundary
- canonical TC/change/safety membership
- Architecture Element counting rule
- Model/S2S reference profile

각 DP는 이 중 실제 구조 인과관계가 큰 Primary ASR 약 3~4개만 선택한다. 모든 후보가 같은 QA에서 같은 값을 얻으면 그 QA는 해당 DP의 변별 driver가 아니며 억지 failure를 만들지 않는다.

## C.12 Primary ASR Qualification — Natural Architecture Sensitivity

System-level ASR이 중요하다는 사실과 특정 DP의 후보를 가르는 지표라는 사실은 다르다.

12에서 Primary ASR로 사용하려면 **정상적으로 기능 요구를 충족하도록 설계한 합리적인 구조 대안들 사이에서**, 동일한 현실적 제약과 동일 Model/Agent 조건 아래 대표 Metric이 자연스럽게 달라질 이유가 있어야 한다.

다음 네 조건을 모두 만족할 때만 해당 DP의 Primary ASR로 사용한다.

1. **Real Structural Difference** — 책임 배치, call graph, state authority, contract, materialization, persistence/enforcement boundary가 실제로 다르다.
2. **Natural Causal Effect** — 그 차이가 별도의 고의적 기능 누락 없이 Metric에 영향을 줄 수 있다.
3. **Same Realistic Constraints** — latency/token/context/resource/failure 조건을 후보에 동일하게 적용한다.
4. **Representative Scenario** — 05의 실제 UC/11-B canonical TC에서 차이가 관찰되거나 합리적으로 예측 가능하다.

opaque ID, random nonce, hidden mapping 등 후보 간 차이를 만들기 위한 인위적인 scoring fixture는 Primary ASR 선정 근거로 사용하지 않는다.

### ASR별 예상 역할

| ASR | DP discriminator로서의 기본 판단 |
| --- | --- |
| ASR-01 | **강함** — call graph/materialization/IPC/delegation 구조가 자연스럽게 latency에 반영 |
| ASR-02 | **조건부** — semantic pipeline 분할, Context provisioning, lossy intermediate representation 등 정상 구조 자체가 completion 정도를 바꾸는 DP에서만 Primary |
| ASR-03 | **조건부** — Conversation/Task context provisioning 또는 state authority 선택이 실제 continuation information availability를 바꾸는 DP에서만 Primary |
| ASR-04 | **강함** — Agent 변화의 structural ripple을 직접 측정 |
| ASR-05 | **강함** — Model/Context/state 변화의 structural ripple을 직접 측정 |
| ASR-06 | **주로 constraint** — 정상 대안이 모두 recovery invariant를 만족하면 점수는 동점. recovery mechanism의 비용은 ASR-01/05 등에 나타남 |
| ASR-07 | **주로 constraint** — 정상 대안이 모두 0 violation이면 점수는 동점. enforcement 비용/변경 ripple은 다른 ASR에 나타남 |

### DP × ASR 판정 기록

각 Primary ASR은 후보 결과 전에 다음을 적는다.

    DP ID
    ASR ID
    reasonable candidate structural difference
    realistic fixed constraints
    natural causal path to metric
    representative UC/TC
    expected reason values may differ
    condition under which the ASR should be demoted to regression

두 합리적 후보가 기능을 완전히 구현했을 때 같은 값이 예상된다면 해당 ASR을 Primary에서 내리는 것이 원칙이다.

현재 ASR-02/03/06/07에 대한 natural sensitivity hypothesis와 예상 역할은 [Natural Architecture Sensitivity Hypotheses](./11-evidence/natural-architecture-sensitivity-hypotheses.md)에 정리한다.

기존 ASR의 변별력이 부족할 경우를 대비한 예비 품질 후보는 [08-A Reserve QA / Architecture Driver Candidates](./08a-reserve-qa-architecture-driver-candidates.md)에 별도로 관리한다. 이 후보들은 현재 11-C scoring table에 포함되지 않으며 08의 H/H 선정 규칙을 다시 통과해야 승격된다.
