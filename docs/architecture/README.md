# VIA Architecture Baseline

> **Status: active and authoritative**

> **현재 12번 작업:** [Target Architecture](./12-decisions/target-architecture/README.md)를 먼저 설계·합의한 뒤,
> 그 구조에서 중요한 품질 차이를 만드는 핵심 선택을 역으로 추출하고, steelman 후보 구체화와 함께 ASR을 재검토한다.
> 기존 VIA-DP 분해에 맞추거나 다시 매핑하지 않으며, 전체 구조 합의 전에는 새 Decision Package를 만들지 않는다.

이 디렉터리는 VIA의 시스템 정의, 공통 범위, 대표 Use Case, 변화 시나리오, 품질 속성, 측정 계약, Decision Point를 연결한 현재 Architecture 기준선이다. 이 기준선이 답하려는 질문은 다음과 같다.

> 사용자 PC에서 연속적인 Voice interaction을 유지하면서 direct response와 여러 Downstream Agent 위임을 일관되게 연결하려면, VIA 내부의 책임·상태·계약·process boundary를 어떻게 나누어야 하는가?

과거 v1.1, vNext/DP-00, W12-G1 문서는 [archive](../archive/README.md)에 있으며 현재 기준이 아니다.

## Baseline at a glance

- VIA는 사용자 interaction과 orchestration을 책임진다.
- Downstream Agent는 업무 reasoning, planning, tool 선택·실행을 책임진다.
- Voice interaction은 VIA 안에 있으며, 현재 목표 배치는 on-device Omni 1개를 S2S·semantic 두 역할이 공유한다. 입력 보호용 Streaming ASR을 포함한 배치는 12번 주 설계이며, local/remote 교체 시나리오와 이전 DP 조건은 별도로 보존한다.
- 모든 후보는 동일한 기능 범위와 Use Case를 충족해야 한다.
- 01~11의 전제 위에서 목표 Architecture를 먼저 설계한다. 이후 중요한 품질 차이를 만드는 구조적 선택만 Decision Package로 추출한다.
- QC-01~QC-10은 상위 품질 관심사다. QA catalog는 category range와 하나의 QA당 하나의 대표 metric 원칙을 유지하며, 네 통합 core QA와 상세 measurement·diagnostic QA를 함께 관리한다.
- **QA-09·19·29·39를 핵심 `CONFIRMED_ASR`로 확정했다.** 정의와 VIA-DP-01~18 적용 원장은 [Core ASR Contract](./08-quality-attributes/core-asr-contract.md)에 있다. 공통 system target·score band proposal은 작성했고 DP별 Shared Spine patch·mock profile·모집단과 함께 freeze 전이다.
- 현재 목표 Architecture를 설계하는 품질 우선순위는 **QA-19 semantic accuracy 1순위, QA-09 responsiveness 2순위, QA-29 modifiability 3순위, QA-39 reliability/recoverability 4순위**다. 낮은 순위도 생략하지 않는다. 현재 네 ASR은 이후 비교의 고정 목록이 아니다. Decision Point와 steelman 후보를 구체화할 때 ASR의 추가·변경과 비교 기준을 함께 정의한다. 이후 Decision Package는 그때 정한 ASR 전체를 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED` 중 하나로 보고하고 applicable 축을 독립적으로 비교한다.
- 기존 VIA-DP-01~18과 Core DP 6개는 이전 decision-first 작업의 reference로 보존한다. 새 목표 Architecture의 구성요소나 선택을 이 번호에 맞추지 않는다.
- QA-01/02/03/05는 QA-09 input, QA-11/12는 QA-19 field source, QA-21~23은 QA-29 change source, QA-31/32는 QA-39 fault evidence다. QA-04·13~15·41·51·61/62는 회귀·진단·qualification으로 유지한다. 이전 일곱 DP reference generation은 archive했으며 새 core ASR 결과가 아니다.

모든 QA의 실제 stimulus/terminal event, software 인식 event와 component 포함 규칙은 [Measurement Event & Boundary Contract](./11-measurement/event-boundary-contract.md)의 공통 형식으로 관리한다.

12번 작업은 [Architecture Decisions](./12-decisions/README.md)에서 시작한다. 현재 목표 구조와 합의 상태를 먼저 읽고, 기존 Core DP와 전체 inventory는 이전 작업의 reference가 필요할 때만 확인한다.

## Minimum orientation path

처음 접속한 session은 아래 다섯 단계만 먼저 읽는다. 목적·기능 요구사항을 이해하기 전에 QA나 DP부터 해석하지 않는다.

| 순서 | 문서 | 답하는 질문 |
| --- | --- | --- |
| 1 | [System Mission & Boundary](./01-system-mission-and-boundary.md) | VIA는 왜 존재하며 무엇을 책임지는가? |
| 2 | [Fixed Architecture Scope](./03-fixed-architecture-scope.md)와 [Representative Use Cases](./05-representative-use-cases.md) | 모든 후보가 제공해야 할 기능과 사용자 완료 조건은 무엇인가? |
| 3 | [Core ASR Contract](./08-quality-attributes/core-asr-contract.md) | responsiveness·accuracy 등 품질을 어떻게 해석하는가? |
| 4 | [Target Architecture](./12-decisions/target-architecture/README.md) | 현재 제안 구조와 열린 검토 항목은 무엇인가? |
| 5 | [Architecture Decisions](./12-decisions/README.md) | 목표 구조에서 rationale와 검증을 어떤 순서로 만드는가? |

세부 문서는 작업 목적에 따라 연다.

| 필요 | 문서 |
| --- | --- |
| 용어와 요청 lifecycle | [Terms](./02-terms.md), [Canonical Interaction Flow](./04-canonical-interaction-flow.md) |
| A/B 고정 조건과 변화 시나리오 | [Fixed Assumptions](./06-fixed-assumptions.md), [Intentional Variables](./07-intentional-variables.md) |
| 상세 QA와 요구 추적 | [Quality Attributes](./08-quality-attributes/README.md), [Traceability](./09-traceability.md), [Architecture Element Definition](./10-element-definition.md) |
| 측정 endpoint와 evidence | [Measurement](./11-measurement/README.md) |
| 목표 Architecture와 새 rationale | [Architecture Decisions](./12-decisions/README.md) |
| 기존 DP inventory와 결정 | [Previous Decision Focus](./12-decisions/dp-executive-summary.md), [ADRs](../adr/README.md) |

부분 검색, 전체 18개 DP 순회, archive 열람으로 시작하지 않는다. Archive는 현재 요구·계약·결과가 아니라 필요할 때만 확인하는 provenance다.

## Current status

| Area | State | Meaning |
| --- | --- | --- |
| Mission, boundary, common scope | Current | 후보가 바꿀 수 없는 출발점 |
| Representative Use Cases and change scenarios | Current | 평가 모집단과 변화 범위 |
| QC taxonomy | Current | QC-01~QC-10 상위 관심사 |
| QA catalog | Active core-ASR baseline | 네 core QA와 상세 measurement·diagnostic QA; 범위 안 번호 공백은 의도적 |
| ASR classification | Confirmed | QA-09·19·29·39; 측정 freeze와 결과는 pending |
| QA semantic definitions | Current | [Core ASR Contract](./08-quality-attributes/core-asr-contract.md)와 상세 QA 의미 경계 |
| QA-01~QA-05 event-boundary contract | Draft | 실제 source 사건과 software 진단 event를 구분함 |
| Mock/spine machine contract and harness | Mixed | 공통 target·score band proposal과 mock/spine 방법은 작성; freeze·active harness·full campaign은 미완료 |
| Current evidence | Empty / `NOT_RUN` | 이전 VIA-DP-02·05·06·09·11·12·13 reference generation은 archive; current 결과로 대체하지 않음 |
| Target Architecture | Major design complete / final review pending | 필수 기능·예외·주요 계약 완성안; 사용자 최종 수용과 구현·측정은 별도 |
| Previous Core DP set | Preserved reference | VIA-DP-03·05·06·07·15·17; 새 구조의 reading order나 매핑 대상이 아님 |
| Previous full DP inventory | Preserved reference | VIA-DP-01~18 독립 보고서; 새 package 도출 시 그대로 계승하지 않음 |
| ADRs | Mixed | 세 DP accepted with caveats; IR deferred |

## How the baseline becomes an Architecture rationale

```text
Mission and fixed scope
  → representative Use Cases and changes
  → quality attribute and metric definition
  → complete target Architecture
  → structural choices, steelman candidates and ASR review
  → strong alternatives and trade-offs
  → falsification conditions and measurement
  → rationale / ADR
```

Decision Package는 Architecture를 발견하기 위한 중립적 탐색 단위가 아니다. 이미 선택한 목표 구조의 핵심 선택을 설명하고 반증 가능한 방식으로 방어한다. 대안은 약하게 만들지 않고, 선택 구조의 비용과 대안이 더 유리한 조건을 함께 기록한다.

## Active authoring rules

- 사용자 목표와 완료 조건을 특정 후보에 맞춰 바꾸지 않는다.
- 결과를 보기 전에 fixture, repetition, percentile, target, score, failure treatment를 동결한다.
- 계산, mock/reference 실행, 실제 모델 실행, 제품 실행의 evidence level을 구분한다.
- `NOT_IMPLEMENTED`나 `NOT_RUN`을 과거 수치로 채우지 않는다.
- 현재 문서가 archive를 normative source로 인용하지 않도록 한다.
- 정의 변경 시 관련 traceability, measurement guide, result status, ADR caveat를 함께 확인한다.
