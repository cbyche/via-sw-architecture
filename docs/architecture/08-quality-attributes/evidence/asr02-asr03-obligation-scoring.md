# ASR-02 / ASR-03 Obligation Scoring — 정도 기반 평가 기준

> 작성일: 2026-09-21
> 상태: **사용자 승인된 measurement/oracle 보정**. 실제 Architecture 후보 결과는 NOT_RUN.
> 목적: Task Completion과 Continuity를 단순 "했다/못했다"가 아니라 architecture-relevant obligation을 얼마나 보존했는지로 측정한다.

## 1. 대표 Metric

### ASR-02 Task Completion Effectiveness

**Task Completion Obligation Satisfaction Rate (%)**

TC i에 적용되는 ASR-02 obligation 집합을 O(i,02)라 한다.

    TC02_i
    = satisfied(O(i,02)) / |O(i,02)|

    ASR-02
    = 100 × mean(TC02_i)
      for canonical 48 TCs

### ASR-03 Interaction & Task Continuity

**Continuity Obligation Preservation Rate (%)**

    TC03_i
    = preserved(O(i,03)) / |O(i,03)|

    ASR-03
    = 100 × mean(TC03_i)
      for canonical 30 TCs

TC를 동일 가중한다. 복잡한 TC에 obligation이 많다는 이유로 전체 QA에서 더 큰 비중을 주지 않는다.

## 2. Atomic obligation

하나의 obligation은 trace에서 독립적으로 참/거짓을 판정할 수 있는 architecture-relevant 요구 하나다.

- `REQUIRED_PRESENT`: 반드시 관찰되어야 함
- `FORBIDDEN_ABSENT`: 발생하지 않아야 함

후보 결과를 본 뒤 obligation을 쪼개거나 합치지 않는다.

### ASR-02 대표 category

- GOAL: 현재 사용자 목표/의미 보존
- REFERENT: 대상/source binding
- CONSTRAINT: 명시 제약 보존
- REQUEST_STRUCTURE: compound decomposition 및 relation
- TASK_ASSOCIATION: No Tracked/New/Existing과 정확한 Task
- HANDLING_AGENT: 처리 위치/capability/Agent 선택
- CLARIFICATION: 질문 및 사용자 답의 원 Request binding
- OUTCOME: terminal result/outcome binding

### ASR-03 대표 category

- CONVERSATION: 앞 대화/설명 관계 보존
- REFERENT_HISTORY: 과거 referent/result 재참조
- TASK_IDENTITY: 동일 VIA Task identity
- EXECUTION_CORRELATION: Task ↔ Agent run/thread correlation
- RESULT_ARTIFACT: 결과물 relation
- PENDING_INTERACTION: 질문/승인 relation
- MODALITY_CONNECTION: Voice/Text/Voice Connection 변화
- MULTI_TASK_ISOLATION: 여러 Task 사이의 분리와 switching

Category는 obligation granularity를 검토하기 위한 label이며 별도 가중치가 아니다.

## 3. Multi-ASR TC

같은 TC가 여러 ASR을 검증해도 실행은 한 번이다. 다만 obligation마다 ASR tag를 둔다.

예:

    TC-01.3 "새 질문인데 피타고라스 정리를 설명해줘"

    O1 [ASR-02, GOAL]
      새 주제 질문에 맞는 응답을 만든다.

    O2 [ASR-03, TASK_IDENTITY]
      기존 T-PPT에 이 요청을 잘못 연결하지 않는다.

    O3 [ASR-03, MULTI_TASK_ISOLATION]
      T-PPT를 수정/취소하지 않는다.

ASR-02 O1이 실패했다고 ASR-03 O2/O3을 자동 FAIL 처리하지 않으며 그 반대도 마찬가지다.

## 4. Strict PASS는 버리지 않는다

degree metric만 보면 심각한 한 조건 실패가 평균에 묻힐 수 있다. 따라서 각 TC에 다음 두 값을 함께 남긴다.

    degree = satisfied obligations / applicable obligations
    strict PASS = all applicable obligations satisfied

대표 Metric은 degree이고 strict pass rate는 secondary evidence다.

예:

    TC-09.3 Data-dependent Compound Request

    ASR-02 obligations:
      O1 두 Request 보존       PASS
      O2 data dependency 보존  PASS
      O3 실제 summary binding  PASS
      O4 recipient binding     PASS
      O5 잘못된 이전 결과 금지 FAIL

    degree = 4/5 = 80%
    strict = FAIL

## 5. 모든 후보가 100%가 나올 수 있는가

**그럴 수 있으며 그 자체는 문제가 아니다.**

ASR-02/03은 Architecture가 반드시 만족해야 할 중요한 품질이지만, 모든 DP에서 반드시 후보를 가르는 metric일 필요는 없다.

같은 모델과 같은 Context가 실제 판단 경계까지 전달되고, 필요한 Conversation/Task/Execution state를 모든 후보가 올바르게 유지한다면 여러 후보가 모두 100%를 받을 수 있다.

그 경우의 해석은 `이 DP에서 ASR-02/03이 차이를 만들지 않았다`이지, 시험을 더 어렵게 만들어 억지로 실패를 만들어야 한다는 뜻이 아니다.

DP에서 ASR-02 또는 ASR-03을 Primary QA로 선정하려면 후보 구조 차이가 다음 causal chain을 가져야 한다.

    responsibility / contract / state authority 차이
    → 사용할 수 있는 evidence 또는 유지 가능한 identity/correlation 차이
    → 동일 obligation 결과 차이

이 인과관계가 없다면 해당 QA는 그 DP에서 regression constraint 또는 secondary observation이다.

## 6. 구조 차이로 실제 degree 차이가 날 수 있는 예

다음은 실제 candidate 결과가 아니라 **변별 가능성의 예**다.

### Interaction Grounding

- 구조 A: pointer latest snapshot만 전달
- 구조 B: timestamped interaction timeline 전달

TC-04.2에서 구조 A는 두 "여기"를 마지막 pointer로 묶을 수 있고, 구조 B는 각각의 source event에 연결할 수 있다.

이때 referent obligation 결과가 달라질 수 있다.

### S2S → Agent continuity

- 구조 A: S2S direct response가 provider-local history에만 존재
- 구조 B: S2S response가 VIA Conversation record에도 binding

TC-07.1에서 "방금 설명으로 발표자료 만들어줘"의 history obligation 결과가 달라질 수 있다.

### Existing Task correlation

- 구조 A: Agent ID만 유지
- 구조 B: VIA Task ↔ run/thread correlation을 유지

동일 Agent에서 복수 run이 존재하는 TC-10.3/14.2의 continuity obligation 결과가 달라질 수 있다.

반대로 두 후보 모두 필요한 state와 contract를 제공하면 둘 다 100%가 맞다.

## 7. Anti-gaming rule

- 후보 결과를 본 뒤 obligation 추가/삭제 금지
- 후보별 obligation granularity 변경 금지
- 단어/문장 수로 obligation 개수 증가 금지
- 하나의 의미상 요구를 여러 동의어 조건으로 중복 가산 금지
- TC별 동일 가중 유지
- strict 결과와 obligation 원자료 항상 공개

이 규칙으로 partial credit을 주되 점수를 임의로 조작하는 것을 막는다.
