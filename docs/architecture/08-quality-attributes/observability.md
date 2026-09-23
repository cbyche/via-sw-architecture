# Observability Measurement Contract

> 상태: **PRE-IMPLEMENTATION CONTRACT DRAFT / fixture·target·score 미확정 / 결과 NOT_RUN**
>
> 목적: 연구 조직이 VIA 기능을 시험하고 개선할 때 필요한 실행 기록의 완전성과 평가 결과의 재현성을 서로 다른 단일 metric으로 정의한다.

## 1. QA-61 — Execution Trace Completeness

### 질문

시험 실행이 끝난 뒤, 저장된 로그만으로 그 요청이 어떤 경로와 상태를 거쳐 어떤 결과가 됐는지 빠짐없이 재구성할 수 있는가?

### 단일 metric

```text
complete execution-trace run rate (%)
```

machine field:

```text
complete_execution_trace_run_rate_pct
```

한 run은 다음 중 그 실행에 적용되는 관계가 모두 연결될 때만 complete다. 적용되지 않는 관계는 누락이 아니라 명시적인 `N/A`로 판정한다.

- 입력, 실제 응답과 최종 성공·실패·timeout
- Conversation, Request, Task와 Agent execution identity
- 실제 처리 경로와 참여 Component/process/dependency
- source event revision, retry, correction, cancel과 상태 전이
- candidate, build, configuration, fixture와 model/prompt version
- metric endpoint에 사용한 원시 event와 clock provenance

```text
QA-61 = 100 × complete run 수 / 전체 scored run 수
```

로그 줄 수나 저장 byte 수는 이 QA의 metric이 아니다. 보호정보 원문을 더 많이 기록해 completeness를 높여서도 안 된다. 필요한 관계는 opaque ID, digest, 범주화된 값과 redaction provenance로 보존할 수 있어야 한다.

## 2. QA-62 — Evidence Reproducibility

### 질문

보존된 raw trace와 freeze manifest만으로 이전에 보고한 평가 결과를 다시 정확히 만들 수 있는가?

### 단일 metric

```text
exactly reproduced evaluation-result rate (%)
```

machine field:

```text
exactly_reproduced_evaluation_result_rate_pct
```

독립된 clean evaluator가 보존된 evidence package를 입력으로 사용해 다음 값을 다시 계산한다.

- trial PASS/FAIL과 failure reason
- raw metric
- case aggregation과 percentile
- target pass/fail과 score
- evidence label과 limitation

이 값들이 원래 보고값과 모두 일치할 때 그 evaluation result가 reproduced다.

```text
QA-62 = 100 × exactly reproduced result 수 / 전체 sampled reported result 수
```

Model이나 외부 Agent를 다시 실행해 같은 자연어 출력을 만들라는 뜻이 아니다. 이미 수집한 raw evidence에서 같은 평가 결론을 다시 계산할 수 있는지를 본다.
검증할 reported result 표본과 표본 추출 규칙은 candidate 결과를 보기 전에 고정하며, 재현에 성공한 결과만 사후 선택하지 않는다.

## 3. 두 QA의 차이

- QA-61은 **실행 자체를 로그에서 재구성할 수 있는가**를 묻는다.
- QA-62는 **그 로그에서 보고된 평가 숫자를 다시 만들 수 있는가**를 묻는다.

완전한 trace가 있어도 analyzer·manifest가 없으면 QA-62는 실패할 수 있다. 반대로 불완전한 trace에서 계산한 잘못된 숫자를 반복 생성할 수 있으므로 QA-62가 QA-61을 대신하지 않는다.

## 4. Architecture sensitivity

이 QA들은 다음 구조 선택에 영향을 받는다.

- component별 자유 형식 로그와 versioned 공통 event envelope
- correlation identity의 발급·전파 책임
- source timestamp와 local observation의 구분
- synchronous logging, buffered collector와 process-local spool
- trace schema version과 backward reader 책임
- immutable raw evidence, manifest와 analyzer version의 소유권
- local/remote export와 privacy filtering boundary

## 5. Freeze 전에 남은 작업

- QA별 canonical run과 사전 고정된 evaluation-result 표본 추출 규칙
- complete trace의 required relation schema
- evidence package와 freeze manifest schema
- privacy-preserving field와 금지 필드
- missing/duplicate/out-of-order event 처리
- target과 0~5 score band
- DP별 applicability와 qualification 여부
