# Task Control Responsiveness Measurement Contract

> 상태: **PRE-IMPLEMENTATION CONTRACT DRAFT / fixture·target·score 미확정 / 결과 NOT_RUN**
>
> 목적: QA-05가 사용자의 Task 제어 요청에 VIA가 얼마나 빨리 올바른 상태로 답하는지를 하나의 직관적인 시간 metric으로 정의한다.

## 1. QA-05 — Task Control Responsiveness

### 질문

사용자가 취소·정정·후속 제어를 말하거나 입력한 뒤, VIA가 올바른 Task에 제어를 연결하고 현재 사실에 맞는 처리 상태를 보여주기까지 얼마나 걸리는가?

### 단일 metric

```text
worst canonical-control-case p95 Task control response time (ms)
```

machine field:

```text
worst_case_case_p95_task_control_response_ms
```

### 시간 경계

```text
t0 = 유효한 사용자 제어 입력이 끝난 실제 시각
t1 = 올바른 Task에 대한 사실에 맞는 제어 처리 상태가 처음 보이거나 들린 시각
sample = t1 - t0
```

Voice 입력은 실제 발화 종료, Text 입력은 사용자가 submit한 시각을 사용한다. `t1`은 다음 중 해당 상황의 진실을 정확히 표현해야 한다.

- 위임 전 요청이 실제로 중단됨
- 실행 중 Task에 제어 요청이 올바르게 기록·전달됨
- 외부 source가 취소·완료 상태를 확인함
- 이미 완료됨, 제어 미지원 또는 현재 확인 불가임을 정확히 알림

단순 버튼 click handler 실행, local queue 삽입 또는 근거 없는 “취소됐습니다”는 종료점이 아니다.

## 2. QA-04와의 구분

- QA-04는 barge-in 뒤 **재생 중인 음성이 멈추는 시간**이다.
- QA-05는 사용자 제어가 **올바른 Task의 사실에 맞는 처리 상태로 나타나는 시간**이다.

사용자가 음성을 끊었다는 이유만으로 Agent Task를 자동 취소하지 않는다. 따라서 두 QA는 합산하거나 같은 endpoint를 사용하지 않는다.

## 3. Canonical control case 후보

- 위임 전 요청 취소
- 실행 중 특정 Task 취소
- 완료와 취소가 교차한 경우
- 이미 완료된 Task 제어
- 취소 미지원 또는 확인 불가
- 여러 Task 중 하나만 취소·정정
- 같은 Agent의 여러 실행 중 하나만 제어

각 case의 trial p95를 계산하고 그중 가장 느린 case p95를 대표값으로 사용한다. 지원하지 않는 기능을 성공처럼 응답하거나 잘못된 Task를 제어한 trial은 latency 성공 표본에서 제외하지 않고 실패로 기록한다.

## 4. Freeze 전에 남은 작업

- canonical case와 Voice/Text 비중
- `t1` disposition schema와 source confirmation rule
- timeout과 실패 처리
- 반복 수와 percentile algorithm
- target과 0~5 score band
- DP별 component path와 applicability
