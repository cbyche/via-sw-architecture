# VIA Architecture Baseline

> **Status: active and authoritative**

이 디렉터리는 VIA의 시스템 정의, 공통 범위, 대표 Use Case, 변화 시나리오, 품질 속성, 측정 계약, Decision Point를 연결한 현재 Architecture 기준선이다. 이 기준선이 답하려는 질문은 다음과 같다.

> 사용자 PC에서 연속적인 Voice interaction을 유지하면서 direct response와 여러 Downstream Agent 위임을 일관되게 연결하려면, VIA 내부의 책임·상태·계약·process boundary를 어떻게 나누어야 하는가?

과거 v1.1, vNext/DP-00, W12-G1 문서는 [archive](../archive/README.md)에 있으며 현재 기준이 아니다.

## Baseline at a glance

- VIA는 사용자 interaction과 orchestration을 책임진다.
- Downstream Agent는 업무 reasoning, planning, tool 선택·실행을 책임진다.
- Voice Runtime은 VIA 안에 있고, S2S와 VIA semantic model runtime은 local/remote dependency가 될 수 있다.
- 모든 후보는 동일한 기능 범위와 Use Case를 충족해야 한다.
- Architecture 선택은 IR, TASK, AGENT, EXEC DP별 A/B 직접 비교로 수행한다.
- QC-01~QC-10은 상위 품질 관심사다. QA catalog는 category range와 하나의 QA당 하나의 대표 metric 원칙으로 재구성된 사용자 검토 초안이며 active ID는 15개다.
- ASR은 아직 선정하지 않았으며 Architecture 영향이 확인된 QA에 별도 번호 없이 표시한다.
- QA-01~QA-04는 Voice responsiveness, QA-11~15는 correctness/continuity, QA-21/22는 modifiability, QA-31/32는 reliability/availability, QA-41은 resource, QA-51은 privacy를 다룬다. 새 harness와 결과는 아직 없다.

모든 QA의 실제 stimulus/terminal event, software 인식 event와 component 포함 규칙은 [Measurement Event & Boundary Contract](./11-measurement/event-boundary-contract.md)의 공통 형식으로 관리한다.

## Required reading order

| 순서 | 문서 | 답하는 질문 |
| --- | --- | --- |
| 1 | [System Mission & Boundary](./01-system-mission-and-boundary.md) | VIA가 무엇을 책임지고 무엇을 Agent에 맡기는가? |
| 2 | [Terms](./02-terms.md) | 같은 용어를 어떤 뜻으로 쓰는가? |
| 3 | [Fixed Architecture Scope](./03-fixed-architecture-scope.md) | 모든 후보가 공통으로 제공해야 할 기능은 무엇인가? |
| 4 | [Canonical Interaction Flow](./04-canonical-interaction-flow.md) | direct/delegated interaction이 어떤 lifecycle을 따르는가? |
| 5 | [Representative Use Cases](./05-representative-use-cases.md) | 어떤 사용자 상황으로 Architecture를 검증하는가? |
| 6 | [Fixed Assumptions](./06-fixed-assumptions.md) | 후보 비교에서 무엇을 동일하게 유지하는가? |
| 7 | [Intentional Variables](./07-intentional-variables.md) | 어떤 모델·Agent·계약 변화를 견뎌야 하는가? |
| 8 | [Quality Attributes](./08-quality-attributes/README.md) | 어떤 품질을 어떤 지표로 관찰하는가? |
| 9 | [Traceability](./09-traceability.md) | Scope→UC→QC→QA가 어떻게 연결되는가? |
| 10 | [Architecture Element Definition](./10-element-definition.md) | 후보의 component/contract/state를 어떤 단위로 비교하는가? |
| 11 | [Measurement](./11-measurement/README.md) | 결과 전에 무엇을 동결하고 어떤 evidence를 생성하는가? |
| 12 | [Decisions](./12-decisions/README.md) | DP별 A/B를 어떻게 비교하고 결정하는가? |

문서를 부분 검색으로 먼저 읽으면 역사적 이름과 현재 정의를 섞기 쉽다. 새 참여자는 최소한 01, 03, 05, 08, 11, 12 순서를 유지한다.

## Current status

| Area | State | Meaning |
| --- | --- | --- |
| Mission, boundary, common scope | Current | 후보가 바꿀 수 없는 출발점 |
| Representative Use Cases and change scenarios | Current | 평가 모집단과 변화 범위 |
| QC taxonomy | Current | QC-01~QC-10 상위 관심사 |
| QA catalog | User-review draft | category range와 15개 active single-metric QA; 범위 안 번호 공백은 의도적 |
| ASR classification | Pending | QA별 Architecture 영향 검토 전 |
| QA semantic definitions | Current draft | active QA별 단일 metric과 의미 경계 정의됨 |
| QA-01~QA-04 event-boundary contract | Draft | 실제 source 사건과 software 진단 event를 구분함 |
| Machine contract, pending targets, harness | Pending | 구현·실행 전에 먼저 동결해야 함 |
| Current evidence | `NOT_RUN` | archive 결과로 대체하지 않음 |
| DP inventory | Current | IR/TASK/AGENT/EXEC A/B alternatives |
| ADRs | Mixed | 세 DP accepted with caveats; IR deferred |

## How the baseline becomes a decision

```text
Mission and fixed scope
  → representative Use Cases and changes
  → quality attribute and metric definition
  → pre-result measurement freeze
  → one-DP-at-a-time A/B execution
  → raw evidence and aggregation
  → trade-off analysis
  → ADR
```

같은 실행 결과를 TASK/AGENT 등 non-applicable 축으로 복제해 표본 수를 늘리지 않는다. 여러 DP 조합은 interaction 확인에 사용할 수 있지만 primary decision은 각 DP의 paired contrast다.

## Active authoring rules

- 사용자 목표와 완료 조건을 특정 후보에 맞춰 바꾸지 않는다.
- 결과를 보기 전에 fixture, repetition, percentile, target, score, failure treatment를 동결한다.
- 계산, mock/reference 실행, 실제 모델 실행, 제품 실행의 evidence level을 구분한다.
- `NOT_IMPLEMENTED`나 `NOT_RUN`을 과거 수치로 채우지 않는다.
- 현재 문서가 archive를 normative source로 인용하지 않도록 한다.
- 정의 변경 시 관련 traceability, measurement guide, result status, ADR caveat를 함께 확인한다.
