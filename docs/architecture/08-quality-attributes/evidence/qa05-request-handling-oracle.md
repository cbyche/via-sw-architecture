# QA-05 VIA Request Handling Correctness — Oracle Contract

> 상태: **USER REVIEW DRAFT / fixture와 oracle 미작성 / 결과 NOT_RUN**

## 1. 목적과 범위

QA-05는 Downstream Agent가 업무를 얼마나 잘 수행했는지가 아니라 VIA가 사용자의 요청을 올바르게 구조화하고 처리 경로·Task·Agent·결과를 연결했는지를 측정한다.

정상 network, 정상 dependency와 고정 Agent fixture를 사용한다. network failure와 recovery는 QA-09로 분리한다.

## 2. 왜 정상 조건에서도 실패할 수 있는가

다음은 구현 crash가 없어도 VIA model과 Architecture의 정보·책임 배치에 따라 달라진다.

- 현재 화면·이전 대화·과거 결과 중 referent 선택
- 요청의 조건·순서·의존 관계 보존
- New/Existing/No Tracked Task 판단
- S2S direct response와 Core/Agent delegation 선택
- Agent capability 선택과 필수 Context packaging
- model structured output의 validation·repair
- result를 원래 Conversation·Task·run에 연결

통합 semantic 판단과 staged 판단은 prompt/context/call graph가 다르므로 같은 model family를 사용해도 결과가 달라질 수 있다.

## 3. 사람이 매 실행을 채점하지 않는 방식

사람은 결과를 보기 전에 case와 oracle을 검토·승인한다. 실제 run은 structured trace를 자동 채점한다.

각 case oracle은 필요한 항목만 사용한다.

| Oracle type | 예 |
| --- | --- |
| `EXACT` | `task_mode=EXISTING`, `task_id=T-PPT` |
| `ONE_OF` | direct 또는 bounded helper 둘 다 허용 |
| `SET_EQUAL` | referent 집합이 `{chart-A, chart-B}` |
| `ORDERED_RELATION` | `summarize → send` dependency |
| `REQUIRED_PROPOSITION` | constraint에 `recipient=김대리` 포함 |
| `FORBIDDEN_PROPOSITION` | 철회한 target 또는 다른 Task 포함 금지 |
| `CLARIFICATION_REQUIRED` | 정보 부족 시 임의 처리 대신 질문 |
| `BINDING` | result의 conversation/task/run identity |
| `CARDINALITY` | dispatch와 user response 각각 정확히 1회 |

자연어 답변의 문체·단어 선택·문장 유사도는 QA-05 점수에 넣지 않는다. 의미상 필요한 proposition과 금지 proposition만 response의 structured provenance 또는 validated semantic output에서 확인한다.

## 4. Run-level 판정

```text
run_pass = 모든 applicable assertion PASS
case_rate = pass runs / all scored runs
QA05 = 100 × mean(case_rate)
```

후보에게 evaluator-only oracle을 입력으로 제공하지 않는다. timeout, invalid output, exhausted repair와 missing trace는 실패다. 동일 실패가 여러 assertion을 깨뜨릴 수 있으므로 원인은 한 번 기록하되 실제 깨진 assertion은 숨기지 않는다.

## 5. Corpus 작성 원칙

[Test Case Catalog](../../11-measurement/test-case-catalog.md)의 94개 variation은 source pool이다. 모두를 자동으로 QA-05 분모에 넣지 않는다. 다음 정상조건 family를 포함하는 architecture-sensitive subset을 결과 전에 승인한다.

- visible/historical referent
- compound request와 constraint
- New/Existing Task association
- direct/delegated handling
- Agent/capability selection
- clarification과 answer binding
- result/response binding과 duplicate suppression

각 family의 쉬운 경우와 모호하지만 정답 또는 허용행동을 정의할 수 있는 경우를 포함한다. 정답 합의가 불가능한 case는 점수 분모에서 제외하고 exploratory corpus로 남긴다.

## 6. 아직 필요한 작업

- canonical case membership
- 실제 structured trace schema
- case별 assertion JSON
- actual VIA model profile, temperature/seed와 반복 수
- critical failure 목록과 score cap 여부
- QA-05 0~5 band 최종 승인
