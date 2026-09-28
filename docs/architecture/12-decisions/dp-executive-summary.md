# VIA Current Architecture Focus — Core ASRs and Core DPs

> **Status: active orientation document · 2026-09-28**
>
> **Core ASRs:** QA-09, QA-19, QA-29, QA-39
>
> **Core DPs:** VIA-DP-03, VIA-DP-05, VIA-DP-06, VIA-DP-07, VIA-DP-15, VIA-DP-17
>
> 이 문서는 새 참여자와 새 LLM session이 현재 Architecture 작업의 초점을 빠르게 이해하기 위한 기준 문서다. Core QA와 Core DP의 선정은 확정했지만, 여섯 DP의 A/B winner와 새 core-ASR 결과는 아직 확정하지 않았다.

## 1. 지금 확정된 결론

현재 보고서와 평가의 핵심 범위는 다음 **6개 DP로 고정**한다.

```text
VIA-DP-03 → Voice input evidence authority
VIA-DP-05 → Context acquisition plan authority
VIA-DP-17 → Context representation authority
VIA-DP-06 → semantic decision authority
VIA-DP-07 → compound-request dependency authority
VIA-DP-15 → Agent progress-state maintenance
```

- 이 6개는 임시 shortlist나 예비 후보 목록이 아니다. **현재 Architecture 보고서가 중점적으로 심층 정의·구현·측정할 Core DP set**이다.
- VIA-DP-01~18 전체 inventory는 누락 점검, 공통 계약과 회귀 검증을 위해 유지한다. Inventory에 남는 것과 Core DP로 평가 자원을 우선 배정하는 것은 다르다.
- Core DP 선정은 대안 A/B winner 선정이 아니다. 여섯 DP 모두 새 core-ASR 계약에 따른 winner는 아직 없다.
- Core DP set을 바꾸려면 단순히 한 후보의 결과가 약하다는 이유가 아니라, 사용자 승인과 active baseline 갱신을 포함한 별도 scope decision이 필요하다.
- VIA-DP-11은 여섯 DP가 약할 때 자동으로 투입하는 7번째 후보가 아니다. 실제 native SDK·plugin·browser/OS bridge 등 VIA-side integration의 Process-fatal 위험이 제품 범위에 새로 들어오면 그때 별도 scope change로 재검토한다.

## 2. 바뀌지 않는 시스템 경계

VIA는 사용자의 Voice·Text·화면 interaction을 받고, Context와 요청 의미를 이해하며, 직접 답하거나 Downstream Agent에 위임하고, 진행·질문·제어·결과를 같은 Conversation과 Task로 다시 연결한다.

| 영역 | 고정 책임 |
| --- | --- |
| VIA | Voice/Text interaction, Context, semantic decision, Conversation/Task, direct response, Agent routing, progress/control/result 연결 |
| Downstream Agent | domain reasoning, planning, Tool 선택·실행, open-ended research, 외부 업무 Action |
| S2S Model | Voice inference dependency. Voice Engine이 integration과 observable evidence 계약을 소유함 |
| Semantic LLM | VIA 의미 판단 dependency. Component별 별도 모델이 아니라 공유 모델 1개 |

VIA의 공통 기반은 **S2S Model 1개와 semantic LLM 1개**이며 Component·Task·semantic stage별로 모델을 복제하지 않는다. 역할별 prompt·call·session·buffer는 달라질 수 있다. VIA-DP-03은 evidence source topology 자체가 비교 축이므로 A에 timestamp-capable Streaming ASR 1개를 추가하고, B에는 별도 ASR 없이 S2S의 turn-final time-aligned text capability를 요구한다. 이 한정 예외의 지연·자원·변경·장애 비용은 숨기지 않는다.

### 2.1 모든 후보가 유지해야 하는 기능

Core DP는 기능을 고르는 목록이 아니다. A/B 어느 쪽이든 다음 사용자 경험을 모두 제공해야 하며, 한쪽이 제공하지 못하면 낮은 점수가 아니라 기능 부적합 또는 `UNRESOLVED`다.

- Voice·Text·화면 interaction과 “이거”, “아까 그 파일” 같은 지칭을 같은 Conversation에서 이어간다.
- 일반 지식과 범위가 명확한 read-only 요청에는 직접 응답할 수 있고, 조사·문서 작성·Application 조작 같은 실제 업무는 적절한 Agent에 위임한다.
- 직접 응답 뒤 실제 업무를 이어 맡기고, 새 업무와 기존 Task의 후속 요청·정정·취소를 구분한다.
- 독립·순차·조건·데이터 의존 복합 요청의 관계를 보존하되, domain planning과 Tool 선택은 Agent에 맡긴다.
- 여러 Agent 업무의 진행·질문·승인·결과를 올바른 Conversation과 Task에 연결하고 Voice/Text 채널을 바꿔도 이어간다.
- 권한·Consent·User Memory 제어를 지키고, 장애·재시작 뒤 확인되지 않은 완료나 중복 Action을 만들지 않는다.

정식 기능 범위는 [Fixed Architecture Scope](../03-fixed-architecture-scope.md), 사용자 목표와 성공·실패 조건은 [Representative Use Cases](../05-representative-use-cases.md)가 기준이다.

## 3. 네 개의 Core ASR

중요도는 **QA-09와 QA-19가 가장 높고, 그다음이 QA-29와 QA-39**다. 네 QA는 서로 다른 질문에 답하므로 하나의 가중 총점으로 합치지 않는다.

| Core ASR | 처음 읽는 사람을 위한 질문 | 대표 metric | 방향 |
| --- | --- | --- | --- |
| **QA-09** Average VIA-attributable Interaction Responsiveness | 사용자가 요청·상태·제어 결과를 기다리는 동안 VIA가 실제로 소비한 시간은 평균 얼마인가? | applicable delegation·direct response·Agent progress·Task control trial 전체의 산술평균 ms | 작을수록 좋음 |
| **QA-19** VIA Request Handling Field Accuracy | VIA가 목표·대상·관계·처리 경로·진행 상태 같은 필요한 field를 얼마나 정확히 결정했는가? | applicable QA-11/12 oracle field 전체의 단순 accuracy | 클수록 좋음 |
| **QA-29** Average Architecture Change Locality | Agent·Model·Context·State·실험 계약 한 건이 바뀔 때 Architecture Element가 평균 몇 개 바뀌는가? | applicable QA-21/22/23 change당 changed element 수의 산술평균 | 작을수록 좋음 |
| **QA-39** Fault Containment & Recovery Success Rate | 장애가 불필요하게 퍼지지 않고 제한 시간 안에 올바른 상태로 복구된 비율은 얼마인가? | applicable fault trial 전체의 단순 PASS 비율 | 클수록 좋음 |

집계 원칙은 다음과 같다.

- QA-09는 p95가 아니라 **전체 applicable trial의 평균**이다. QA-04 Voice interruption은 core 평균에서 제외하고 회귀로 본다.
- QA-19는 strict whole-run pass나 family별 macro-average가 아니라 **중복 없는 applicable field 전체의 accuracy**다. QA-13~15는 별도 회귀 진단이다.
- QA-29는 QA-21/22/23의 applicable change를 하나의 모집단으로 만들고 **change당 평균 changed element 수**를 계산한다.
- QA-39는 recovery time과 blast radius를 더하지 않는다. 두 조건과 correctness를 모두 만족한 fault trial의 **단순 성공률**이다.
- QA-41 memory는 target-device diagnostic이다. QA-51과 action/access rule은 safety qualification, QA-61/62는 evidence qualification이다.
- 서로 다른 DP는 applicable case·field·change·fault 모집단이 다를 수 있으므로 DP 사이 절대값을 합산해 순위를 만들지 않는다. 같은 DP의 A/B만 동일 모집단으로 직접 비교한다.
- 여섯 Core DP의 공식 PoC는 같은 Shared Evaluation Spine과 근거가 동결된 deterministic dependency
  mock을 사용한다. DP 차이는 spine의 최소 event/state patch로 표현하고 `MEASURED_MOCK_E2E`
  결과를 네 Core ASR 수치와 공통 별점에 직접 사용한다.

정확한 수식·실패 처리·field/change/fault 범위는 [Core ASR Contract](../08-quality-attributes/core-asr-contract.md)가 유일한 상세 기준이다.

<a id="decision-map"></a>

## 4. 여섯 Core DP

| Core DP | 주 Component | 서로 배타적인 A/B 질문 | Core ASR 역할 | 현재 상태 |
| --- | --- | --- | --- | --- |
| [VIA-DP-03](./via-dp-03-voice-evidence.md) | Voice Engine, Context/Intent 입력 경계 | A의 timestamp-capable Streaming ASR partial로 provisional evidence를 만들 것인가, B의 S2S turn-final timestamped transcript로 final evidence를 만들 것인가? | QA-09·19·29 primary, QA-39 regression | A/B identity·candidate 계약 `DRAFT`; mock behavior profile·spine patch와 Measurement Freeze는 `PENDING` |
| [VIA-DP-05](./via-dp-05-context-contract.md) | Context Engine, Intent Refiner | semantic execution 전에 Context read-set을 닫을 것인가, 같은 execution에서 bounded source를 demand-driven으로 확장할 것인가? | QA-09·19·29·39 primary | 새 A/B 정의 완료, measurement freeze 미작성 |
| [VIA-DP-17](./via-dp-17-context-materialization-authority.md) | Context Engine과 Context consumer | 선택된 Source를 공통 canonical ContextValue로 확정할 것인가, consumer가 목적별 Context view를 확정할 것인가? | QA-09·19·29·39 primary | 새 A/B 정의 완료, measurement freeze 미작성 |
| [VIA-DP-06](./via-dp-06-semantic-authority.md) | Intent Refiner, Orchestrator, Task association, Agent Router | 하나의 통합 authority가 최종 의미를 확정할 것인가, 단계별 authority가 versioned correction으로 협력할 것인가? | QA-09·19·29 primary, QA-39 regression | reference evidence는 있으나 새 core-ASR 결과와 product E2E는 아님 |
| [VIA-DP-07](./via-dp-07-compound-orchestration.md) | Orchestrator, Task Manager, Agent Router | VIA가 모든 user-declared dependency edge를 release할 것인가, owner-affinity bundle 내부 edge는 Agent가 실행할 것인가? | QA-09·19·29·39 primary | 다중 Agent·기존 Task owner를 보존하도록 재정의, capability/fixture 미동결 |
| [VIA-DP-15](./via-dp-15-agent-observation-authority.md) | Agent Router, Task Manager, Orchestrator, Voice/Text Publisher | 유효 Agent event를 continuous progress projection에 반영할 것인가, hint/cursor만 유지하고 필요 시 authoritative snapshot을 조회할 것인가? | QA-09·19·29·39 primary | 진행 상태 범위 강화, field/fault/notification 계약 미동결 |

### 4.1 DP-05와 DP-17은 왜 다른가

- DP-05는 **어떤 Source를 언제 읽을 수 있는가**를 결정한다.
- DP-17은 **이미 선택된 Source를 누가 어떤 의미 표현으로 확정하는가**를 결정한다.

따라서 `plan-first + canonical`, `plan-first + consumer view`, `demand-driven + canonical`, `demand-driven + consumer view` 네 조합이 모두 가능하다. 검토 순서는 DP-05 다음 DP-17로 두지만, DP-05의 winner가 정해져야만 DP-17을 비교할 수 있는 것은 아니다.

### 4.2 DP-07은 모든 복합 요청을 한 Agent에 맡기는 문제가 아니다

먼저 capability와 기존 Task/Agent execution owner에 따라 node를 partition한다. 서로 다른 owner 사이 edge는 양쪽 모두 VIA가 관리한다. A/B가 갈리는 지점은 같은 owner가 처리할 수 있고 동등한 node status·artifact·부분 control을 제공하는 bundle 내부 edge의 readiness authority다.

### 4.3 DP-15는 push 대 polling 문제가 아니다

양쪽 모두 event와 query를 사용할 수 있다. A는 유효 event를 VIA progress state의 확정 근거로 쓰고 gap에서 query로 조정한다. B는 event를 dirty/hint로 사용하고 사용자 status·notification·terminal·gap·restart 시 coalesced snapshot query로 확정한다. Event마다 query하는 약한 B는 비교 대상이 아니다.

## 5. 심층 검토 순서

사용자 interaction이 실제로 흐르는 순서대로 다음 순서를 기본으로 한다.

1. **DP-03 — Voice evidence**: A의 Streaming ASR mock partial timestamp와 B의 S2S mock native final
   timestamp를 같은 spine에 적용할 contract를 먼저 동결한다.
2. **DP-05 — Context acquisition**: 요청 처리 중 Source 획득 범위를 언제 닫는지 확정한다.
3. **DP-17 — Context representation**: 선택된 Source를 어떤 표현 계약으로 소비할지 확정한다.
4. **DP-06 — Semantic authority**: Voice·Context evidence를 바탕으로 목표·대상·Task relation·handling을 누가 최종 확정하는지 비교한다.
5. **DP-07 — Compound orchestration**: 복합 요청의 node/bundle edge와 여러 Agent·기존 Task 관계를 확정한다.
6. **DP-15 — Agent progress state**: 위임 후 진행·질문·완료를 VIA가 어떻게 유지·복구·통지할지 확정한다.

이 순서는 문서 작업 순서이지 DP 사이 winner dependency나 전역 pipeline 고정을 뜻하지 않는다. 심층 검토에서 독립성이 무너지거나 동등 기능의 A/B를 만들 수 없으면 결과를 숨기지 않고 Core DP 정의 자체를 다시 검토한다.

## 6. Core DP마다 해야 할 일

현재 lifecycle 단계는 **Measurement Contract Definition**이다. 각 Core DP를 한 번에 하나씩 다음 순서로 처리한다.

1. **A/B 식별 규칙 확정** — authority·state·contract·call graph·fault boundary 중 실제로 달라지는 한 축을 명시한다.
2. **동등 기능과 capability gate 확정** — 한쪽이 필수 기능을 제공하지 못하면 낮은 점수가 아니라 `UNRESOLVED` 또는 기능 부적합으로 기록한다.
3. **Core-ASR applicability freeze** — QA-09 trial, QA-19 field, QA-29 change, QA-39 fault를 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED`로 고정한다.
4. **Measurement Freeze 작성** — fixture, source event, terminal event, 반복 수, timeout, aggregation, failure treatment, target과 score band를 결과 전에 승인한다.
5. **Candidate A/B 구현** — 다른 DP 조건과 모델·Agent·Context 조건을 고정하고 해당 DP만 바꾼다.
6. **직접 A/B 실행과 독립 replay** — 실패·timeout을 포함한 raw evidence에서 네 core ASR을 재계산한다.
7. **Architecture Decision** — trade-off, 약점, tactic과 재검증 조건을 ADR에 남긴다.

16-configuration full-factorial은 필요한 경우 interaction 분석으로만 사용한다. Primary winner selection은 한 DP씩 다른 조건을 고정한 paired A/B 비교다.

<a id="review-status"></a>

## 7. 완료 상태와 남은 일

### 완료

- Core ASR 4개와 대표 metric 선정
- Core DP 6개 선정과 비핵심 inventory 분리
- 여섯 DP의 현재 A/B 구조 질문과 Component 경계 초안
- DP-03의 A/B evidence-source topology, candidate 의미, provisional/final lifecycle과 Grounding 경계 합의
- DP-05/17의 독립성, DP-07의 multi-Agent/owner partition, DP-15의 progress 관리 범위 정리

### 아직 완료되지 않음

- 여섯 DP별 machine-readable case/field/change/fault registry
- DP별 Shared Spine patch·mock timing/error/fault profile과 Measurement Freeze
- 공통 system target·score band를 적용할 analyzer와 score schema
- DP-03 candidate schema·detector registry 초안의 freeze와 제품 UI evidence retention budget 승인
- DP-07 bundle과 DP-15 progress/snapshot의 Reference-Agent mock behavior contract freeze
- 새 A/B candidate implementation과 네 core-ASR full campaign
- 여섯 DP의 A/B winner와 이를 반영한 새 ADR

현재 네 core ASR의 새 결과는 모두 `NOT_RUN`이다. 기존 reference campaign과 accepted/deferred ADR은 구현 가능성·역사적 근거로 보존하지만, 새 core QA 값이나 현재 Core DP winner로 소급 변환하지 않는다.

## 8. Core가 아닌 DP를 읽는 법

VIA-DP-01~18 전체 목록은 [Decision index](./README.md)에 유지한다. Core가 아닌 DP도 필수 기능·안전·회귀 책임에서 삭제되지 않는다.

- VIA-DP-02와 VIA-DP-12는 supporting design decision이다.
- VIA-DP-11은 현재 Core DP가 아니며 6개 실패 시의 대체 후보도 아니다. 기존 EXEC-DP01/ADR-003의 accepted 이력과 reference runner는 보존한다.
- VIA-DP-09와 VIA-DP-14에 대응하는 기존 accepted ADR, VIA-DP-06에 대응하는 deferred ADR의 상태는 그대로 유지한다. Core DP 선정이 기존 ADR을 자동 폐기하거나 재승인하지 않는다.
- 나머지 DP는 전체 Architecture inventory, fixed context, regression/qualification과 향후 독립 scope 질문의 근거로 유지한다.

## 9. 새 세션의 읽기 순서

1. [System Mission & Boundary](../01-system-mission-and-boundary.md) — VIA와 Agent의 책임 경계
2. [Fixed Architecture Scope](../03-fixed-architecture-scope.md)와 [Representative Use Cases](../05-representative-use-cases.md) — 고정 기능과 사용자 완료 조건
3. **이 문서** — 현재 Core ASR, Core DP와 다음 작업
4. [Core ASR Contract](../08-quality-attributes/core-asr-contract.md) — 네 QA의 정확한 수식과 실패 처리
5. 현재 차례의 Core DP 개별 보고서 — A/B 구조·사고실험·미확정 사항
6. [Major DP Evaluation Plan](../11-measurement/major-dp-evaluation-plan.md) — pre-result freeze와 실행 절차

전체 inventory, 상세 측정 계약, ADR은 현재 작업에서 필요할 때 [Decision index](./README.md), [Measurement Guide](../11-measurement/README.md), [ADRs](../../adr/README.md)에서 찾는다. 부분 검색이나 archive 문서로 시작하지 않는다. Archive는 당시 판단의 provenance일 뿐 현재 요구·우선순위·결과가 아니다.

<a id="legacy-mapping"></a>

## 10. Legacy provenance

IR/TASK/AGENT/EXEC/CTX/SEC 번호는 현재 작업의 읽기 경로나 Core DP 목록이 아니다. 옛 번호와 VIA-DP-01~18의 상세 매핑은 [2026-09-28 pre-cleanup snapshot](../../archive/core-dp-selection-pre-cleanup-2026-09-28/README.md)에 보존한다. 현재 판단은 이 active 문서와 개별 VIA-DP 보고서를 따른다.
