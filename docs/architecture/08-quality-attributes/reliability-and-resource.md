# Reliability, Availability & Resource Measurement Contract

> 상태: **PRE-IMPLEMENTATION CONTRACT DRAFT / fixture·target·score 미확정 / 결과 NOT_RUN**
>
> 목적: QA-31 recovery time, QA-32 fault blast radius와 QA-41 target-device memory를 서로 다른 단일 metric으로 정의한다.

## 1. QA-31 — Correct Task Recovery Time

### 질문

장애 뒤 모든 영향 Task의 올바른 identity, state, result와 허용 control이 다시 사용 가능해지기까지 얼마나 걸리는가?

### 시간 경계

```text
t0 = fault가 VIA 기능 또는 Task control을 실제로 사용할 수 없게 만든 최초 시각
t1 = 모든 영향 Task의 올바른 identity·state·result와 허용 control이 다시 사용 가능한 시각
sample = t1 - t0
```

### 단일 metric

```text
worst_fault_stratum_p95_full_task_recovery_ms
```

process·worker·UI가 다시 시작된 시점은 종료점이 아니다. Task 누락, duplicate execution, stale terminal state, 잘못된 result binding 또는 허용 control의 손실이 있으면 recovery failure다.

## 2. QA-32 — Fault Blast Radius

### 질문

한 fault가 그 dependency를 필요로 하지 않는 사용자 기능과 Task까지 얼마나 손상시키는가?

### 단일 metric

```text
worst_fault_excess_affected_user_visible_units
```

각 fault `f`에 대해 다음을 계산한다.

```text
excess_affected(f)
= count(actual unavailable or incorrect user-visible units
        outside the pre-approved necessary dependency closure of f)

QA-32 = max excess_affected(f) over all frozen fault strata
```

`user-visible unit`은 결과 전에 고정한 다음 종류의 workload cell이다.

- active VIA Task
- Direct Voice interaction path
- Text interaction path
- Task 조회·제어 path
- Agent integration path
- Context source read path

Architecture Component 수를 blast-radius unit으로 사용하지 않는다. fault별 necessary dependency closure도 후보 결과를 보기 전에 같은 의미 기준으로 승인한다. 의존성상 반드시 영향받는 unit의 복구 시간은 QA-31에서 관찰하고, 불필요하게 함께 손상된 unit만 QA-32가 센다.

## 3. QA-41 — Target Device Memory Footprint

### 질문

고정 workload에서 후보 Architecture가 target PC에 요구하는 최대 committed memory는 얼마인가?

### 단일 metric

```text
worst_workload_stratum_p95_peak_committed_memory_bytes
```

trial별로 workload window의 전체 peak committed bytes를 기록하고, stratum별 p95 중 가장 큰 값을 대표값으로 사용한다.

### Accounting boundary

포함:

- 모든 VIA-owned process
- 후보가 요구하는 local integration/helper process
- 후보 선택 때문에 target PC에서 실행되는 local S2S/semantic Model Runtime
- VIA가 소유한 in-memory cache와 buffer

제외:

- 다른 후보에서도 동일하게 실행되는 OS baseline
- Downstream Agent 내부가 별도 외부 dependency로 실행되는 경우의 내부 memory
- remote Model/Agent server memory

한 후보의 local dependency를 별도 process라는 이유로 제외하지 않는다. process별 working set, CPU/GPU/VRAM과 energy sample은 원인 분석 evidence이며 QA-41의 추가 대표 metric이 아니다.

Target PC 사양, OS build, power mode, warm/cold condition과 workload를 결과 전에 고정한다. 제품 장비가 미정인 현재는 target과 score band를 만들지 않는다.

## 4. 공통 fault·resource 원칙

- fault injection과 workload는 모든 후보에 동일한 의미로 적용한다.
- 성공한 recovery만 골라 p95를 만들지 않는다.
- 확인 근거가 없는 외부 실행을 정상 복구로 처리하지 않는다.
- 모든 요청을 차단하거나 기능을 제거해 blast radius나 memory를 낮춘 후보는 기능 적합성에 실패한다.
- QA-31과 QA-32는 같은 fault run에서 계산할 수 있지만 서로 다른 metric이며 합산하지 않는다.

## 5. Freeze 전에 남은 작업

- fault strata와 necessary dependency closure
- user-visible unit registry와 expected affected set
- active Task workload와 repetition·timeout
- target PC와 memory sampling method
- target과 0~5 score band
- DP별 applicability와 evidence label
