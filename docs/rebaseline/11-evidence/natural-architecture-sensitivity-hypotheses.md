# Natural Architecture Sensitivity Hypotheses — 12 DP 도출 전 점검

> 작성일: 2026-09-21
> 상태: DP 후보를 확정하기 전의 sensitivity hypothesis. 12의 최종 DP/후보/결정이 아니다.

## 1. 원칙

ASR이 System-level로 중요하다는 사실과 특정 DP의 구조 대안을 가르는 Primary QA라는 사실을 구분한다.

Primary QA는 정상적인 기능 요구를 충족하도록 설계한 합리적 대안들 사이에서, 동일한 현실적 constraint 아래 representative metric이 자연스럽게 달라질 구조 인과관계가 있어야 한다.

ASR-01/04/05는 구조 차이가 metric에 직접 반영될 가능성이 높다. ASR-02/03은 특정 구조축에서만 자연스러운 sensitivity가 예상된다. ASR-06/07은 대부분 invariant/constraint 역할일 가능성이 높다.

## 2. ASR-02가 자연스럽게 갈릴 가능성이 높은 구조축

### H-02A Semantic Interpretation / Request Understanding Pipeline

03/04는 Request decomposition, Context/Referent, Refinement, Task Association, Handling, Agent Selection의 논리 책임은 고정하지만 실행 순서와 Component/Model 호출 구조는 고정하지 않는다.

합리적인 후보 예:

- **Integrated semantic decision:** 하나의 Model invocation 또는 하나의 semantic component가 필요한 Context를 함께 보고 Request/Referent/Task/Handling/Agent decision을 통합 생성
- **Staged semantic pipeline:** Grounding/Refinement/Task Association/Handling을 독립 stage와 structured intermediate contract로 연결
- **Hybrid:** deterministic candidate extraction/Context binding 후 하나의 semantic decision, 또는 일부 stage만 분리

이 후보들은 모두 기능 요구를 구현할 수 있다. 그러나 동일 Model과 현실적인 latency/context/token budget을 고정하면 자연스러운 차이가 날 수 있다.

- staged pipeline은 stage별 focused context와 validation 장점이 있지만, intermediate representation이 정보를 압축/누락하고 오류가 다음 stage로 전파될 수 있음
- integrated path는 lossy boundary가 적지만 prompt/schema가 커지고 여러 판단이 강하게 결합됨
- hybrid는 deterministic evidence와 generative judgment의 경계를 어디에 두느냐에 따라 completion 정도와 latency/evolvability가 달라질 수 있음

따라서 이 구조축은 ASR-02 + ASR-01 + ASR-05의 실제 trade-off 후보가 될 가능성이 높다.

### H-02B Context Access & Materialization

03은 Context를 언제 수집하고 어디에 보관할지를 Architecture Decision으로 남겨 두었다.

합리적인 후보 예:

- 요청 처리 전에 관련 Interaction/Conversation/Task Context를 정규화하여 materialize
- source handle/metadata 중심으로 유지하고 필요한 시점에 lazy materialization
- Context 종류별 hybrid materialization

동일한 2초 responsiveness 목표, Model context window, source access latency를 적용하면 어느 정보를 언제 확보하는지가 실제 semantic decision에 사용 가능한 evidence를 바꿀 수 있다.

여기서 Completion 차이가 난다면 '한 후보가 기능을 빼서'가 아니라 **동일한 현실적 latency/context budget 아래 materialization policy가 다르기 때문**이어야 한다.

이 구조축은 ASR-02 + ASR-01 + ASR-05의 후보가 될 수 있다.

## 3. ASR-03가 자연스럽게 갈릴 가능성이 높은 구조축

### H-03A Conversation/Task Context Provisioning to Models

VIA가 Conversation/Task identity를 소유한다는 책임은 고정이다. 그러나 S2S/Semantic Model에 과거 Context를 어떻게 제공할지는 고정하지 않는다.

합리적인 후보 예:

- 필요한 canonical Conversation/Task Context를 매 요청 재구성하여 Model에 제공
- provider conversation/thread state를 cache로 활용하고 VIA canonical state와 동기화
- summary/retrieval 기반으로 오래된 Conversation/Task Context를 축약하여 제공

모두 VIA-owned Conversation/Task identity를 유지할 수 있다. 하지만 동일 context window/latency budget에서 long conversation, multiple active Task, Direct→Agent 전환을 처리할 때 실제 Model에 제공되는 과거 정보의 fidelity가 달라질 수 있다.

따라서 Continuity quality가 자연스럽게 달라질 가능성이 있다. 단, 모든 후보가 동일한 relevant context를 손실 없이 제공할 수 있다면 ASR-03은 동점이며 Primary에서 내려야 한다.

### H-03B Interaction/Conversation Context Retention Representation

pointer/selection/Conversation event를 latest snapshot, bounded timeline, normalized event history 등 어떤 표현으로 유지·제공하는지는 구조 선택이다.

단순 기능 누락이 아니라 representation/compression policy가 실제 과거 referent와 follow-up Context fidelity를 바꾸는 경우에만 ASR-03 sensitivity로 인정한다.

이 구조축은 ASR-03 + ASR-01 + ASR-05 후보가 될 수 있다.

## 4. ASR-06에 대한 현재 판단

현재 대표 Metric은 deterministic reliability/recovery scenario pass rate다.

정상적인 후보가 UC-18과 recovery requirement를 만족하도록 충분한 persistence/idempotency/reconciliation tactic을 포함하면 모두 100%가 될 가능성이 높다.

따라서 **현재 metric 기준 ASR-06은 기본적으로 regression/architecture constraint로 보는 것이 타당하다.**

Task supervision/recovery DP에서 서로 다른 persistence/event-processing 구조를 비교하더라도 모두 100%이면 그 DP의 실제 차이는 다음에서 찾는다.

- recovery mechanism이 만드는 latency/coordination cost → ASR-01
- persistence/state/event contract의 변경 ripple → ASR-05
- Agent lifecycle 변화가 recovery 구조에 미치는 ripple → ASR-04

향후 Recovery Time/RTO 같은 별도 continuous metric을 ASR-06 대표 Metric으로 바꾸지 않는 한, 단지 점수 차이를 만들기 위해 fault를 더 극단적으로 만들지 않는다.

## 5. ASR-07에 대한 현재 판단

정상 후보는 Context read/egress와 approval binding에서 violation 0을 목표로 해야 한다.

centralized policy gateway, distributed enforcement, capability/token 방식 등 서로 다른 구조도 정확히 구현하면 모두 0/24가 가능하다.

따라서 **ASR-07도 기본적으로 non-compensable regression/constraint 역할**이 타당하다. Safety 구조 선택의 실제 trade-off는 다음에서 드러날 가능성이 높다.

- enforcement hop/coordination overhead → ASR-01
- policy/model/connector 변경 ripple → ASR-05
- Agent authentication/approval contract 변화 ripple → ASR-04
- 과도한 차단에 따른 정상 업무 completion 회귀 → ASR-02 secondary

## 6. 12에서의 사용 규칙

DP 후보를 만든 뒤 Primary QA를 먼저 고르지 않는다.

1. 합리적인 구조 후보를 정의한다.
2. Fixed Scope/UC를 모두 만족시키는 데 필요한 tactic까지 포함한다.
3. 각 ASR에 대해 natural causal sensitivity가 있는지 분석한다.
4. 실제 metric 차이가 예상되는 ASR만 Primary로 선정한다.
5. 모두 동일 목표를 만족하는 ASR은 regression/constraint로 유지한다.

특히 ASR-02/03을 Primary로 넣으려면 H-02A/B, H-03A/B와 같은 **실제 information-flow 또는 semantic pipeline 구조 차이**가 representative UC에서 completion/continuity 정도를 바꿀 이유가 있어야 한다.

ASR-06/07은 system-level ASR로 유지하되, 특정 DP의 score discriminator가 아닐 수 있음을 명시한다.
