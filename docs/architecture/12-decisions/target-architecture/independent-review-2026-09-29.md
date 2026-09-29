# VIA Target Architecture — 독립 검토 결과

> 검토일: 2026-09-29  
> 범위: [목표 Architecture](./architecture.md)  
> 우선순위: QA-19 semantic accuracy 1순위, QA-09 responsiveness 2순위  
> 상태: reviewer pass 완료 / 사용자 검토 전 보강 반영 / 구현·측정 없음

## 검토 방식

서로의 의견을 보지 않은 세 reviewer가 같은 초안을 각각 검토했다.

| 관점 | 중점 질문 |
| --- | --- |
| Semantic accuracy | 목표·지칭·Task·handling·Agent 전달을 잘못 확정할 수 있는가? |
| 상태·Component 구조 | 상태 소유자, lifecycle, transaction과 복구 경계가 닫혀 있는가? |
| Runtime·fault | 동시성, 정정·취소, 장기 event, barge-in과 장애에서 계약이 유지되는가? |

세 검토는 기존 결정 번호나 대안에 매핑하지 않았고 문서를 직접 수정하지 않았다. 아래 내용은 세 결과를 합친 뒤 목표 Architecture 본문에 반영한 disposition이다.

## 공통 결론

세 reviewer 모두 현재의 큰 뼈대는 유지할 가치가 있다고 보았다. 특히 Conversation·Turn·Request·Task·Agent Execution 분리, read-only semantic 도구, host의 상태·권한·dispatch 소유, 당시 지칭과 현재 실행 유효성 분리, Task와 Agent Execution identity 분리, outbox와 무조건 재전송 금지, 실제 Voice 전달 상태 기록은 타당한 기반으로 평가했다.

반면 기존 초안은 accuracy 최우선 Architecture로 합의하기에 이르다고 판단했다. 세 검토에서 독립적으로 겹친 치명적 공백은 다음 네 가지였다.

1. **S2S 의미 게시 권한:** S2S가 self-contained 질문을 잘못 분류하면 Core semantic 검증을 우회할 수 있었다.
2. **근거와 후보 coverage:** 연결된 근거가 있다는 사실만 확인하고 정답 후보가 탐색에서 빠졌는지는 판정할 수 없었다.
3. **dispatch 선형화:** 전송 직전 정정·취소와 기존 command 전송 사이 race를 막는 원자적 경계가 없었다.
4. **복구 가능한 수신·게시:** Agent event 수신과 Task projection, 응답 계획과 실제 Text·Voice 전달을 crash 뒤 일관되게 복원할 계약이 없었다.

이 네 항목은 latency 최적화보다 먼저 닫아야 한다. 잘못된 대상·Task·승인·command를 빠르게 처리한 결과는 responsiveness 이점으로 인정하지 않는다.

## 본문에 반영한 보강

| 검토 지적 | 반영한 Architecture 계약 |
| --- | --- |
| 모델이 `READY`를 사실상 결정 | 모델은 Semantic Proposal과 field별 Resolution만 제안하고, Controller가 coverage·conflict·freshness를 포함해 readiness와 Semantic Commit을 계산 |
| S2S 직접 경로의 자기 승인 | S2S 출력은 speculative buffer에 두고 Controller의 Direct Admission Record 후에만 게시; 화면·과거 대화·Task·Action·최신성·복합 관계 가능성이 있으면 Core 경로 사용 |
| 지칭 시간축 부재 | utterance span, transcript revision, acoustic interval, 화면·UI revision, pointer·selection과 clock uncertainty를 잇는 Interaction Evidence Timeline 추가 |
| 후보 누락을 판정할 수 없음 | query 범위, source, 전체·반환 후보, truncation·실패·제외 이유를 가진 Evidence Query Receipt와 bounded-completeness gate 추가 |
| 여러 질문·승인의 결합 권한 분산 | VIA clarification, Agent question, Context consent와 Action approval을 Pending User Interaction으로 통합하고 Controller가 답변 결합을 단일 소유 |
| 복합 요청 graph 소유권 모호 | durable Request Graph는 Controller, Task lifecycle은 Task Manager가 소유하도록 분리 |
| dispatch race | immutable Semantic Commit·command epoch·precondition·`DISPATCHING` 선형화 지점과 `supersedes_command_id` 추가 |
| event crash window | durable Agent event inbox, dedupe, source cursor와 Task projection을 한 transaction으로 적용 |
| 응답 crash·barge-in 이후 맥락 | Canonical Response Payload와 publication outbox, 실제 Text 게시·audible prefix/range receipt 추가 |
| 고정 2회가 accuracy 규칙처럼 보임 | 최대 2회를 측정 가능한 초기 resource policy로 낮추고, 미해결 field를 추측하지 않는 종료 규칙을 invariant로 승격 |

## 아직 닫히지 않은 항목

다음은 Architecture 방향은 정했지만 schema·capability 또는 수치 계약이 더 필요하다.

- 실제 S2S가 transcript 시간 정렬, host-controlled audio와 speculative buffering을 제공하는지 확인
- Direct Admission의 구체적 allowlist 또는 semantic negative-risk gate
- source별 bounded completeness 정의와 화면 evidence watermark·clock 오차 허용치
- Request·Task·Command·Execution projection·Response Publication의 전체 상태 전이표
- Agent capability profile별 지원 가능한 start·query·correct·cancel·idempotency 보장
- 하나의 semantic LLM과 S2S를 공유할 때 queue class, 최대 점유시간과 포화 시 degradation
- 화면·음성 원본과 derived evidence의 보관·삭제 경계
- local/remote model과 durable Store의 실제 process·failure 배치
- direct response 내용의 factual correctness를 QA-19 범위로 넣을지 별도 qualification으로 둘지

이 항목은 곧바로 Decision Package를 만들 이유가 아니다. 먼저 전체 Architecture 검토에서 요구 수준과 주 설계가 맞는지 확인한 뒤 세부 계약을 닫는다.

## 사용자 검토에서 먼저 볼 질문

1. S2S는 admission 전 speculative 생성만 허용하고, 사용자에게 말할 권한은 Controller가 갖는 구조가 맞는가?
2. 외부 Action은 후보 coverage가 불완전하거나 동등 후보가 남으면 항상 추가 조회·clarification·실패 중 하나로 끝내는가?
3. VIA clarification, Agent 질문, Context consent와 Action approval의 답변 결합을 하나의 Pending User Interaction 체계로 통합하는가?
4. 복합 요청의 모든 하위 node와 dependency를 Controller의 durable Request Graph로 관리하는가?

네 질문에 동의해도 전체 Architecture 승인이나 구현·성능 입증을 뜻하지 않는다. 이는 가장 큰 wrong-target·wrong-task 위험을 막는 상위 구조를 먼저 확정하는 검토다.
