# Natural Architecture Sensitivity Hypotheses

> 작성일: 2026-09-21
> 상태: DP 후보를 확정하기 전의 sensitivity hypothesis. 최종 DP/후보/결정 또는 ASR 판정이 아니다.

## 1. 원칙

제품적으로 중요한 QA라는 사실, 특정 DP의 대안을 가르는 Primary QA라는 사실, 시스템 수준 ASR이라는 판정은 서로 다르다.

Primary QA는 정상 기능을 충족하도록 설계한 합리적 대안 사이에서 동일한 현실적 constraint 아래 metric이 달라질 구조 인과가 있어야 한다. ASR은 점수 차이뿐 아니라 구조 위험·변경 비용·안전 제약까지 포함하여 [Quality Model](../quality-model.md)의 기준으로 별도 판정한다.

현재 가설상 QA-01~QA-04, QA-07, QA-08은 구조 차이가 metric에 직접 반영될 가능성이 높다. QA-05와 QA-06은 특정 정보 흐름 또는 semantic pipeline 구조축에서만 자연스러운 sensitivity가 예상된다. QA-09·QA-10·QA-12는 정상 후보가 모두 목표를 만족하는 invariant/constraint일 수도 있다.

## 2. QA-05가 자연스럽게 갈릴 수 있는 구조축

### Integrated vs staged semantic decision

Request decomposition, Context/Referent, Refinement, Task Association, Handling과 Agent Selection 책임은 필요하지만 실행 순서와 component/model 호출 구조는 고정하지 않는다.

- Integrated path는 필요한 Context를 한 번에 보고 semantic decision을 통합 생성한다.
- Staged path는 focused stage와 structured intermediate contract를 연결한다.
- Hybrid path는 deterministic evidence binding과 generative judgment의 경계를 나눈다.

동일 Model과 latency/context/token budget에서 intermediate representation의 정보 손실, joint prompt/schema 복잡도, validation 위치가 QA-05 결과를 바꿀 수 있다. 관련 trade-off 후보는 QA-01·QA-02와 QA-08이다.

### Context access and materialization

요청 전에 관련 Context를 정규화해 materialize할지, source handle 중심으로 유지하고 필요한 시점에 읽을지, 종류별 hybrid를 사용할지는 구조 선택이다. 동일한 latency와 context-window budget에서 semantic decision에 실제로 제공되는 evidence가 달라질 때만 QA-05 sensitivity로 인정한다.

## 3. QA-06이 자연스럽게 갈릴 수 있는 구조축

VIA가 Conversation/Task identity를 소유한다는 책임은 고정이지만 Model에 과거 Context를 제공하는 방식은 다음처럼 달라질 수 있다.

- canonical Conversation/Task Context를 매 요청 재구성
- provider conversation/thread state를 cache로 활용하고 VIA canonical state와 동기화
- summary/retrieval로 오래된 Context를 축약

Interaction evidence도 latest snapshot, bounded timeline, normalized event history 등으로 표현할 수 있다. representation/compression policy가 실제 referent, follow-up, multiple-Task identity fidelity를 바꿀 때 QA-06 sensitivity가 생긴다. 모든 후보가 같은 relevant context를 보존하면 QA-06은 동점이어야 한다.

## 4. QA-09와 QA-10에 대한 현재 판단

정상 후보가 persistence, idempotency, reconciliation과 containment tactic을 충분히 구현하면 recovery correctness와 unaffected capability retention이 모두 목표를 만족할 수 있다.

따라서 결과 차이를 만들기 위해 fault를 사후 극단화하지 않는다. Task supervision/recovery 구조의 차이는 다음에서도 나타날 수 있다.

- recovery와 foreground coordination 비용 → QA-01~QA-04
- persistence/state/event contract 변경 ripple → QA-08
- Agent lifecycle 변화 ripple → QA-07

QA-09는 recovery time을 직접 측정하므로 raw latency 차이를 보존한다. QA-10은 모든 정상 후보가 100%면 non-discriminating constraint로 기록한다.

## 5. QA-11과 QA-12에 대한 현재 판단

Privacy exposure와 Action/Access safety는 서로 다른 QA다. QA-11은 허용된 기능을 유지하면서 외부에 노출되는 보호 정보 범위를 측정하고, QA-12는 access·egress·approval binding violation을 측정한다.

centralized policy gateway, distributed enforcement, capability/token 방식 모두 정확하게 구현될 수 있다. 관련 trade-off는 다음에서 나타날 수 있다.

- enforcement hop과 coordination overhead → QA-01~QA-04
- policy/model/connector 변경 ripple → QA-08
- Agent authentication/approval contract 변화 ripple → QA-07
- 과도한 차단에 따른 정상 업무 회귀 → QA-05

QA-12에서 모든 후보가 0 violation이면 이를 숨기지 않고 constraint 충족으로 기록한다.

## 6. 평가 사용 규칙

1. 합리적인 구조 후보를 먼저 정의한다.
2. Fixed Scope와 UC를 만족시키는 데 필요한 tactic을 양쪽에 포함한다.
3. QA별 natural causal sensitivity와 구조 위험을 분석한다.
4. 실제 인과가 있는 QA만 해당 DP의 Primary driver로 둔다.
5. 동일 목표를 만족하는 QA는 regression/constraint로 유지한다.
6. 시스템 수준 ASR 여부는 DP별 Primary 여부와 분리하여 판정한다.

QA-05 또는 QA-06을 Primary로 두려면 실제 information-flow, semantic pipeline 또는 state representation 차이가 representative UC의 obligation 결과를 바꿀 이유가 있어야 한다.
