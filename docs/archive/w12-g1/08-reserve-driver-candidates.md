# 08-A. Reserve QA / Architecture Driver Candidates

> 작성일: 2026-09-21
> 상태: **예비 후보 원장 — 현재 확정 ASR 7개를 변경하지 않음**
> 목적: 12 DP 분석에서 기존 ASR의 자연스러운 구조 변별력이 부족할 경우를 대비하되, 후보 결과에 맞춰 ASR을 사후 추가하는 것을 방지한다.

## 1. 승격 원칙

예비 후보는 '구조 차이가 잘 나서'가 아니라 다음을 모두 만족할 때만 ASR로 승격한다.

1. 기존 Product Definition / UC / Fixed Assumption에서 독립적인 제품 중요성이 H(8~10)로 설명된다.
2. Architecture Difficulty가 H(8~10)다.
3. 기존 ASR representative metric이 이미 측정하는 품질과 실질적으로 중복되지 않는다.
4. 모든 DP에 동일하게 적용 가능한 대표 Metric 하나를 정의할 수 있다.
5. 정상적인 합리적 Architecture 대안 사이에서 Metric이 자연스럽게 달라질 구조 인과관계가 있다.
6. 후보 결과를 보기 전에 target / score boundary를 고정할 수 있다.

승격 시에는 08 QA catalog를 명시적으로 rebaseline하고 09 mapping, 11 measurement/test/scoring을 함께 갱신한다. 12의 특정 후보에 유리하도록 즉석에서 ASR을 추가하지 않는다.

외부 근거:
- SEI는 performance, reliability/availability, security, modifiability 등의 Quality Attribute가 Architecture를 강하게 좌우하며 QA scenario와 measurable response로 분석해야 한다고 설명한다: https://www.sei.cmu.edu/library/reasoning-about-software-quality-attributes/
- ATAM은 여러 Quality Attribute 사이 trade-off를 Architecture 개발 전에 분석하는 방법이다: https://sei.cmu.edu/library/steps-in-an-architecture-tradeoff-analysis-method-quality-attribute-models-and-analysis/

## 2. RQA-01 Concurrent Task Scalability & Performance Isolation

### 기존 제품 근거

- UC-14: multiple active Task
- UC-13: asynchronous progress/result
- FA-08: 0/1/4 active Task와 복수 referent/request 조건

즉 concurrent workload는 새로운 요구가 아니라 이미 승인된 제품 조건이다.

### 예비 평가

- Importance: **8 (H)**
- Architecture Difficulty: **9 (H)**
- 예비 판정: **H/H 승격 후보**

### Representative Metric 후보

**Foreground Latency Degradation Ratio under 4 Active Tasks**

    FLDR
    = ASR-01 Macro-p95 with 4 active background Tasks
      / ASR-01 Macro-p95 with 1 active Task

낮을수록 좋으며 1.0이면 foreground response가 concurrent Task의 영향을 거의 받지 않는다는 뜻이다.

### 왜 구조에 민감한가

- central queue vs task-local supervision
- shared Model/Context resource scheduling
- progress/result fan-in
- Task state lock/contention
- synchronous coordination vs event-driven processing

모든 후보가 기능적으로 multiple Task를 지원해도 interference 정도는 자연스럽게 달라질 수 있다.

ASR-01이 baseline responsiveness를 보는 반면 이 QA는 **load 증가에 따른 성능 degradation**을 보므로 stimulus가 다르다.

---

## 3. RQA-02 Recovery Timeliness / Availability

### 기존 제품 근거

- UC-18 restart/recovery
- FA-14: VIA restart 후 Agent execution 생존/조회 가능 조건
- UC-12/13/14: cancel/race/async event

### 예비 평가

- Importance: **9 (H)**
- Architecture Difficulty: **9 (H)**
- 예비 판정: **H/H**

### 권고 형태

새 ASR을 하나 더 추가하기보다 **현재 ASR-06의 representative metric 교체 후보**로 우선 검토한다.

현재 ASR-06의 27/27 strict PASS는 recovery invariant/regression constraint로 유지하고, 대표 Metric을 다음으로 바꾸는 방안이다.

**p95 Time to Correct User-Visible Recovery (TCR)**

    fault/restart occurrence
    → VIA가 authoritative Task state를 복원/재확인하고
      사용자가 correct status/result/control을 다시 사용할 수 있는 시점

낮을수록 좋다.

### 왜 구조에 민감한가

- event log replay vs snapshot + reconciliation
- local authoritative state vs Agent query reconstruction
- centralized supervisor vs per-Task supervisor
- eager recovery vs lazy recovery
- persistence/commit boundary

정상적인 후보가 모두 '결국 복구'하더라도 **얼마나 빨리 올바른 상태를 회복하는지**는 자연스럽게 달라질 수 있다.

SEI Availability tactics는 fault detection, recovery, reintroduction 등의 architectural tactic이 availability response에 영향을 준다는 관점을 제공한다: https://www.sei.cmu.edu/library/realizing-and-refining-architectural-tactics-availability/

---

## 4. RQA-03 Privacy Exposure Minimization

### 문제 인식

현재 QA-09 / ASR-07의 `Safety violation V/24`는 unauthorized access/action correctness에는 적합하지만, **모두 정책을 준수하면서도 한 Architecture가 필요 이상의 private Context를 remote Model/Agent에 노출하는 차이**는 측정하지 못한다.

Privacy와 Action Safety를 대표 Metric 하나로 묶은 현재 QA-09의 약점일 수 있다.

### 기존 제품 근거

- Personal Information Context
- User Memory Context
- remote/local Model dependency
- Agent Context provisioning
- UC-16 Consent/Action approval
- M-04/M-05/M-06 deployment 이동

### 예비 평가

- Importance: **9 (H)**
- Architecture Difficulty: **8 (H)**
- 예비 판정: **H/H 승격 후보**

### Representative Metric 후보

**Effective Remote Sensitive-Context Exposure per Canonical Workload (KiB)**

wire bytes만 세지 않는다. remote dependency가 접근할 수 있게 된 sensitive payload/scope의 크기를 센다.

예:
- full document 전달 → 해당 sensitive document scope 전체
- 필요한 field만 전달 → 해당 field payload
- remote가 dereference 가능한 handle을 전달 → handle이 열어주는 sensitive scope 전체
- local-only processing → remote exposure 0

동일 canonical workload와 동일 기능 completion을 전제로 낮을수록 좋다.

### 왜 구조에 민감한가

- local vs remote semantic processing
- eager full-context materialization vs selective provisioning
- redaction/minimization boundary
- Agent별 context package 구성
- handle/capability scope

NIST Privacy Framework는 제품/서비스 설계에서 data processing ecosystem과 privacy risk를 명시적으로 관리하도록 권고한다. 이 Metric의 숫자 target을 NIST가 제공한다는 뜻은 아니다: https://www.nist.gov/privacy-framework/

승격한다면 현재 QA-09를 `Action/Access Safety`와 `Privacy Exposure Minimization`으로 분리할지 검토한다.

---

## 5. Tier-2 Reserve — QA-07 Compute & Resource Efficiency 재검토

현재 QA-07은 Importance 4 / Difficulty 7로 ASR이 아니다.

Architecture sensitivity는 높을 가능성이 있다.
- staged vs integrated Model calls
- duplicated Context processing
- parallel inference
- local vs remote binding

그러나 현재 Product Definition에는 고정된 user-PC CPU/GPU/RAM/power budget이나 cloud compute budget이 없다. 따라서 **구조를 가르기 좋다는 이유만으로 Importance를 H로 올리지 않는다.**

추후 'consumer Windows PC에서 foreground application과 공존하며 local semantic processing budget을 X 이하로 유지' 같은 제품 constraint가 승인되면 QA-07을 다시 평가한다.

Representative Metric 후보는 fixed reference hardware에서의 **Total Reference Model Busy Time per Canonical Workload** 또는 명시적 local resource budget이다. target hardware/budget이 없는 현재는 동결하지 않는다.

---

## 6. Tier-2 Reserve — QA-10 Diagnosability & Operability 재검토

현재 QA-10은 Importance 6 / Difficulty 7로 M/M이다.

multi-Model/multi-Agent orchestration에서는 구조적으로 중요한 품질일 수 있으나, 현재 제품 요구에는 MTTR/OPEX/운영조직 목표가 H 수준으로 고정되어 있지 않다.

추후 운영/OPEX가 핵심 제품 목표가 되면 다음 Metric을 검토할 수 있다.

**Fault Localization Success Rate from Standard Telemetry (%)**

fixed injected incident set에서 debug-only instrumentation 없이 Request/Task/Agent Execution/Model/Context 중 원인 경계를 정확히 식별할 수 있는 비율이다.

현재는 reserve로만 유지한다.

## 7. 우선순위 제안

| 우선 | 후보 | 현재 권고 | 이유 |
| ---: | --- | --- | --- |
| **1** | RQA-01 Concurrent Task Scalability | 승격 검토 가치 높음 | 기존 UC/FA에 이미 있고 자연스러운 구조 차이 가능 |
| **2** | RQA-02 Recovery Timeliness | ASR-06 metric 교체 우선 검토 | binary 100% 문제를 직접 해결 |
| **3** | RQA-03 Privacy Exposure Minimization | QA-09 split/신규 ASR 검토 | Safety 0/24 동점 문제와 privacy 구조 차이를 분리 |
| 4 | QA-07 Compute Efficiency | reserve | 제품 resource budget이 아직 없음 |
| 5 | QA-10 Diagnosability | reserve | 현재 Importance가 H가 아님 |

이 원장은 기존 ASR-01~07을 즉시 변경하지 않는다. 12 DP 후보를 확정하기 전에 RQA-01~03의 H/H 타당성과 기존 QA 중복 여부를 별도로 리뷰한다.
