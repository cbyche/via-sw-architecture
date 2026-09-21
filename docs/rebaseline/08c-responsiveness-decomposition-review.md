# 08-C. Responsiveness Decomposition Review

> 작성일: 2026-09-21
> 상태: **방법론 검토 초안 — 기존 QA/ASR catalog를 아직 변경하지 않음**
> 목적: VIA의 interface/orchestration product identity에 맞게 Responsiveness를 여러 architecture-significant performance scenario로 분해할 타당성을 검토한다.

## 1. 사용자 가설에 대한 비판적 결론

### 동의하는 부분

VIA의 핵심 제품 역할은 사용자와 Model/Context/Downstream Agent 사이의 Interaction & Orchestration이다. 따라서 사용자가 요청하고, VIA가 전달하고, Agent/event를 다시 사용자에게 전달하는 timing은 Architecture가 직접 지배하는 핵심 품질이다.

또한 하나의 broad Macro-p95는 서로 다른 경로의 개선/악화를 평균으로 상쇄할 수 있다. 예를 들어 foreground interaction을 우선 스케줄링하면 대화 응답은 빨라지지만 background Agent event 전달은 늦어질 수 있다.

SEI performance quality model은 performance response를 하나의 latency 숫자로 한정하지 않고 latency, throughput, jitter, precedence 등 서로 다른 response measure로 다룬다. Architecture tactic은 이 response를 바꾸며 trade-off를 만들 수 있다.

근거:
- https://www.sei.cmu.edu/library/illuminating-the-fundamental-contributors-to-software-architecture-quality/
- https://www.sei.cmu.edu/documents/1965/2000_004_001_13694.pdf

### 동의하지 않는 부분

`Task 성공은 Model/Agent 성능에 주로 달려 있으므로 VIA에서 중요도가 낮다`는 표현은 사용하지 않는다.

VIA가 잘못된 referent, Task, Agent, Context를 연결하면 Downstream Agent가 완벽해도 사용자 목표는 실패한다. 따라서 Task Completion의 **product importance는 여전히 높다.**

다만 fixed strong Model/Agent와 정상적인 Architecture 대안을 비교할 때 ASR-02가 **Architecture discriminator로서 약할 수 있다**고 표현하는 것이 정확하다.

## 2. ASR granularity에 대한 권고

Responsiveness를 별도의 top-level QA 여러 개로 억지 분리하기보다, **QA-01 User-Experienced Responsiveness / Performance 아래에 여러 Architecture-Significant Performance Scenario를 두는 방식**이 더 자연스럽다.

즉 기존의 `QA 하나 ↔ ASR 하나` 규칙은 project convenience였으며, responsiveness rebaseline 시에는 `한 QA가 서로 다른 stimulus/response를 가진 복수 ASR scenario를 낳을 수 있음`으로 수정할 것을 권고한다.

단, 같은 parent QA의 child ASR을 최종 weighted sum에 그대로 중복 가산하지 않는다. 필요하면 parent QA weight를 child ASR 사이에 분배한다.

## 3. 권고 분해 — 3개

### P-ASR-01 Conversational Reaction Latency

**질문:** 사용자가 Voice/Text로 말한 뒤 VIA가 얼마나 빨리 의미 있는 대화 반응을 시작하는가?

대표 scenario:
- S2S Direct Response
- VIA Core Direct Response
- clarification이 필요한 turn

대표 Metric 후보:

**p95 Foreground Conversational Response-Start Latency**

    User input end
    → first meaningful user-visible Voice/Text response start

Agent의 open-ended 업무시간은 들어오지 않는다.

주 구조 영향:
- S2S fast path
- semantic Model call count / pipeline topology
- Context acquisition/materialization
- streaming/buffering
- foreground scheduling

### P-ASR-02 Task Handoff Responsiveness

**질문:** 실제 업무 요청을 VIA가 얼마나 빨리 올바른 Downstream Agent 실행 경계까지 전달·확정하는가?

대표 Metric 후보:

**p95 Request-to-Durable-Handoff Latency**

    User input end
    → Agent acceptance 또는 재시작 후에도 유실되지 않을 durable handoff 확정

단순 UI의 '처리할게요' 문구는 종료점이 아니다.

주 구조 영향:
- Request/Task semantic pipeline
- Agent selection/routing
- Context package construction
- policy/consent gate
- synchronous RPC vs durable queue/outbox
- Task persistence / handoff commit boundary

### P-ASR-03 Task Feedback Responsiveness

**질문:** Agent 쪽에서 새로운 상태·질문·결과가 생겼을 때 사용자가 얼마나 빨리 알 수 있는가?

대표 Metric 후보:

**p95 Agent-Event-to-User Latency**

    Agent progress/question/result available
    → valid user-visible Text/Voice/notification start

주 구조 영향:
- push event vs polling
- event bus/fan-in
- batching/coalescing
- task-local vs central supervisor
- notification scheduling
- foreground/background priority

AWS와 Azure architecture guidance는 polling이 update synchronization lag를 만들 수 있고, event-driven 구조는 near-real-time response/decoupling에 유리하지만 network/eventual-consistency/variable-latency trade-off를 갖는다고 설명한다.

근거:
- https://docs.aws.amazon.com/en_gb/lambda/latest/dg/concepts-event-driven-architectures.html
- https://learn.microsoft.com/ko-kr/azure/architecture/guide/architecture-styles/event-driven

## 4. 왜 세 축은 같은 방향으로 움직이지 않을 수 있는가

### Foreground priority

foreground Voice/Semantic processing에 높은 우선순위를 주면 P-ASR-01은 좋아질 수 있지만 background Agent event queue의 P-ASR-03은 나빠질 수 있다.

### Durable handoff

Agent call 전에 Task/outbox를 durable commit하면 P-ASR-02가 느려질 수 있지만 recovery correctness는 좋아질 수 있다. 반대로 optimistic asynchronous handoff는 빠를 수 있지만 recovery design burden이 커진다.

### Polling vs push

polling은 implementation/contract가 단순할 수 있지만 P-ASR-03 propagation delay가 polling interval에 종속된다. push/event bus는 feedback latency를 줄일 수 있지만 extra hop, ordering, duplicate handling, variable latency를 만든다.

### Voice buffering

작은 audio buffer는 P-ASR-01 first-audio latency를 줄일 수 있지만 playback jitter/underrun risk를 높일 수 있다. 큰 buffer는 반대다.

즉 하나의 평균 Macro-p95만 사용하면 이러한 Architecture trade-off를 숨길 수 있다.

## 5. 지금은 ASR로 승격하지 않을 responsiveness 항목

### Voice Interruption / Barge-in Latency

Voice UX에는 중요하지만 현재 사용자 리뷰에서 ASR-01 대표값에서 제외하기로 이미 결정했고, 일반 request responsiveness보다 product driver 우선순위가 낮다고 판단했다. secondary regression metric 유지가 적절하다.

### Voice Streaming Jitter / Underrun

Architecture sensitivity는 높고 SEI performance response의 jitter와도 잘 맞는다. 특히 buffering과 first-response latency가 trade-off할 수 있다.

그러나 현재 UC가 'smooth audio playback'에 대한 명시적 quantitative product requirement를 충분히 제공하지 않는다. **Reserve performance scenario**로 보관하고, 실제 Voice UX를 핵심 driver로 더 명시할 경우 승격한다.

### Throughput

개인 PC interaction 제품에서 requests/sec 자체는 핵심 사용자 가치가 아니다. 대신 이미 Reserve QA인 Concurrent Task Scalability / Performance Isolation이 더 적절하다.

## 6. 심사 리스크

다음 방식이면 공격받기 쉽다.

- Direct / Voice / Text / Agent 등 구현 경로마다 latency ASR 하나씩 추가
- 모두 ms 단위라는 이유로 서로 거의 같은 metric을 복제
- 같은 responsiveness child ASR들을 독립 가중하여 Performance를 과대평가
- latency 차이를 만들기 위해 artificial workload를 설계

방어 가능한 논리는 다음이다.

- stimulus와 response endpoint가 다름
- 구조 mechanism이 다름
- 동일 Architecture에서 한 response가 개선되고 다른 response가 악화될 수 있음
- 사용자 관점에서 각각 독립적인 의미가 있음
- 후보 결과를 보기 전에 metric/target을 고정

## 7. 현재 권고

현 ASR-01 `6 request-class Macro-p95`를 그대로 확정하는 것보다 다음 rebaseline을 검토하는 것을 권고한다.

1. QA-01 User-Experienced Responsiveness는 parent QA로 유지
2. P-ASR-01 Conversational Reaction Latency
3. P-ASR-02 Task Handoff Responsiveness
4. P-ASR-03 Task Feedback Responsiveness
5. Voice interruption/jitter는 secondary/reserve
6. Concurrent Task Scalability는 별도 Reserve QA로 유지

이 분해는 VIA가 단순 LLM application이 아니라 **Interaction & Orchestration control plane**이라는 제품 정체성을 더 직접적으로 Architecture evaluation에 반영한다.
