# 41 → 44 → 45 → 42와 46의 조합 검토

> COMPOSITION_DOCUMENT_REVIEW / 2026-10-10 / 미선정·미구현·미측정. [공통 실행 계약 40](./04-40-common-execution-contract.md)과 각 DP의 재구체화 계약을 연결한 검토 기록이다. 문서상 조합 조건과 실제 실행 인증을 구별한다.

## 1. 서로 다른 네 선택을 하나의 프로그램에서 연결한다

- **41:** Request Interpreter에서 다음 조회와 전체 의미 완성을 누가 주도하는가. A는 모델의 bounded tool 조회와 의미 제안, B는 모델의 유한 요청 틀/부분 의미와 코드의 전체 resolution이다.
- **44:** 입력·업무 알림·작업 완료·실제 전달 사건 뒤 후속 실행을 누가 이어가는가. A는 Request Controller의 Dialogue Dispatcher와 중앙 continuation, B는 연결된 Stage와 각각의 window 및 Response Manager의 Publication Join이다.
- **45:** 과거 근거를 어떤 계약으로 공급하는가. A는 원본 owner의 읽기 결과를 현재 요청에 조합, B는 생산자가 유지하는 공통 관계/coverage의 게시 계약을 읽는다.
- **42:** 대화/업무 상태를 어디서 확정하고 어떤 실행 수명을 유지하는가. A는 모듈형 Core의 관련 owner 변경 공동 확정, B는 독립 대화/업무 서비스의 별도 확정과 command/receipt/control 계약이다.

41A의 모델 주도는 프로그램 전체의 권한·저장·대기·전송을 모델이 소유한다는 뜻이 아니다. 모델은 읽기와 의미를 제안하고 코드가 권한, 예산, 채택, 전송 시작과 실제 전달을 검증한다. 45는 일부 tool의 근거 공급 기제를 바꾸지만 42와 44를 tool 내부 구현만으로 축소할 수 없다. 이들은 모델 호출 전후에도 계속 실행되는 상태/사건 계약이다.

## 2. 네 선택이 바뀌어도 유지하는 연결

| 처리·상태 | 공통 owner | 조합의 필수 조건 |
| --- | --- | --- |
| 원음·화면/행동 관측·발화 경계·실제 장치 출력 | Interaction Manager | Turn-Taking Control에 로컬 VAD. 지속 capture와 즉시 stop는 cloud call·근거 생산·dispatcher 완료를 기다리지 않는다. 발화 종료와 전사 완료는 별개다. |
| provider session·전사 대응·cloud call/usage | Model Access | 음성/전사와 의미 모델은 cloud. PC에는 client/buffer/context만 있으며 모델 서비스에 영속 업무 권위를 맡기지 않는다. |
| 현재 의미 제안 | Request Interpreter | 41A/B는 같은 read 권한과 현재 입력/source를 사용한다. `task_retrieve`는 Task Manager에 직접, `context_retrieve`/`interaction_retrieve`는 Context Manager의 근거 계약으로 조회할 수 있다. |
| 현재 의미 채택·Request·질문 focus·미전송 admission | Request Controller | 제안과 채택은 별개. 새 입력의 의미가 미해결이면 관련 미전송 효과를 hold한다. Stage/Memory hit가 이를 생략하지 않는다. |
| 과거 근거 공급·User Memory·파생 사용 검사 | Context Manager | 45A/B의 output은 Bundle/Receipt와 실제 읽은 범위·source vector·누락이다. 과거 기억은 현재 Task binding이나 실행 원장이 아니다. |
| Task·질문·확인된 result/version·로컬 명령 원본 | Task Manager | 업무 상태와 조건은 owner가 재확인한다. Context cache, 대화 receipt, model confidence가 현재 업무 권위를 대신하지 않는다. |
| 외부 전송·receipt·Agent event | Agent Gateway | 채택 후 Request Controller → Task Manager → Agent Gateway 경로. 로컬 접수/전송 시작/외부 접수는 서로 다른 상태다. |
| 내용 준비·게시 제어·내구 실제 전달 원장 | Response Manager | 준비, release, 실제 표시/재생을 구별한다. Publication Outbox는 내구 원본이고 Interaction Manager의 Playback State는 장치 상태/제한 receipt buffer다. |
| 현재 권한·철회 및 검증 변경 저장 | Policy Manager / State Store | 모델과 네트워크 호출은 transaction 밖. epoch 확인과 신규 사용 승인/queue 등록의 선후를 실제 배치의 게이트에서 설명한다. |

## 3. 조합에 적용할 네 가지 조건

**T — 41B 유한 표현 조건.** 모델이 내는 field/연산은 versioned finite representation이다. 추가 모델 호출로 부분 의미를 갱신할 수 있지만 코드가 전체 binding을 완성해야 순수 B다. 모델이 전체 binding을 다시 만들면 명시적 혼합안이다. 45B에 관계가 있다는 사실만으로 41B의 지원 연산이 늘지 않는다. 지원 밖 VIA-owned 조건을 free-text domain instruction에 숨기지 않는다.

**D — 44B 연결 실행 조건.** 각 Stage와 Publication Join이 자기 입력/currentness/대기/취소/credit을 관리한다. 중요 입력·질문·실패/완료·receipt는 durable owner에 보존한다. 내용 후보는 InputSettled와 채택/admission 등을 기다리지만 현재 모호함을 해결할 확인 질문은 별도 clarification admission을 쓴다. 자기 후보의 아직 생성되지 않은 실제 receipt를 게시 선행조건으로 기다리지 않는다. 단순히 중앙 queue에 이름만 바꾼 subscriber를 붙이면 A/B 기제 차이가 사라진다.

**M — 45B 게시 근거 조건.** 정상 조회는 지원하는 공통 관계/coverage를 읽고 현재 source/권한을 확인한다. 지원 종류의 미생산은 NOT_COVERED → 생산 요청 → 조건부 게시 결과 → 재조회, 지원 밖 종류는 UNSUPPORTED_RELATION이다. 원문 dereference는 허용되지만 Reader가 매번 원본 교차 관계를 재구성하면 명시적 B+ 또는 A 확장이다. 의미 추출에만 조건부 C-CONTEXT를 쓰고 명시 ID/version 관계는 코드로 생산할 수 있다. 과거 게시 완료는 현재 의미 채택이나 publication 허가가 아니다.

**H — 42B 독립 확정/전송 조건.** 대화 서비스는 의도/접수 대기를, 업무 서비스는 자신의 gate·명령·전송 시작을 확정한다. Hold/HoldAck의 control session/sequence와 gate revision, command expected revisions, transmission CAS, reconnect reconciliation을 유지한다. ACK 없는 대화 hold 의도를 업무 전송 차단 완료로 표시하지 않는다. 현재 권한 및 source 조회가 불가하면 신규 효과를 보류한다.

이 조건은 문서에 명시한 설계 연결이며 실행 가능한 protocol, 처리 시간 상한, 모델의 의미 정답이나 모든 조합의 성공률을 증명하지 않는다.

## 4. 16개 조합의 문서상 연결 경로

아래 표의 순서는 **41 / 44 / 45 / 42**다. 공통 owner 계약과 위 조건을 충족하는 범위에서 연결 경로를 설명할 수 있다는 뜻이다. 모든 조합을 구현해 통과시켰다는 뜻이 아니다.

| 41 | 44 | 45 | 42 | 현재 문서에서 연결할 경로와 추가 조건 |
| --- | --- | --- | --- | --- |
| A | A | A | A | 모델 read → owner 조합 Bundle → 중앙 완료 회수/후속 지시 → 관련 로컬 owner 공동 확정. |
| A | A | A | B | 위 read/중앙 실행을 유지하고 업무 의도·접수·hold ACK를 별도 완료로 회수. **H** 적용. |
| A | A | B | A | 모델 read → 게시 Reader Bundle. 미생산 대기는 중앙 continuation이 회수하고 후속 재조회를 지시. **M** 적용. |
| A | A | B | B | 게시 생산/조회와 중앙 continuation, 독립 업무 접수·control ACK를 구별. **M·H** 적용. |
| A | B | A | A | 모델 read/owner 조합의 완료가 typed Stage 입력이 되고 관련 owner 공동 확정 결과와 Publication Join을 연결. **D** 적용. |
| A | B | A | B | Stage 완료와 독립 업무 ACK/접수 결과를 typed completion으로 연결. stream 도착은 업무/게시 허가가 아님. **D·H** 적용. |
| A | B | B | A | 게시 Reader/생산 완료를 Stage 대기에 연결. 생산 완료와 현재 InputSettled/meaning adoption을 구별. **D·M** 적용. |
| A | B | B | B | Stage 대기·게시 revision·업무 gate ACK의 세 조건을 구별하며 각각 owner에서 재확인. **D·M·H** 적용. |
| B | A | A | A | 유한 틀/부분 의미 → 코드 read/전체 결합 → 중앙 완료 회수 → 관련 owner 공동 확정. **T** 적용. |
| B | A | A | B | 코드 전체 resolution 완료와 독립 업무 접수/hold ACK를 중앙에서 별도 회수. **T·H** 적용. |
| B | A | B | A | 코드 Engine이 게시 근거를 읽지만 자신의 연산 지원 한계를 유지. 생산 대기는 중앙 continuation. **T·M** 적용. |
| B | A | B | B | 유한 resolution·게시 근거·독립 업무 gate를 혼동하지 않고 각각 completion/receipt로 연결. **T·M·H** 적용. |
| B | B | A | A | 코드 resolution/owner 조합의 결과가 Stage 입력. Stage가 의미 값을 대신 생산하지 않으며 owner 공동 확정 유지. **T·D** 적용. |
| B | B | A | B | 코드 resolution과 Stage 실행, 독립 업무 접수·ACK를 구별. **T·D·H** 적용. |
| B | B | B | A | 코드 resolution이 게시 근거와 지원 범위를 읽고, NOT_COVERED 대기는 Stage/Join 계약으로 이어감. **T·D·M** 적용. |
| B | B | B | B | 유한 의미 결합·단계 실행·게시 근거·독립 확정의 모든 경계를 유지해야 연결 가능. **T·D·M·H** 적용. 가장 많은 계약 경계를 갖지만 우열이나 부적합을 단정하지 않음. |

42A에도 capture→hold 적용→로컬 전송 시작까지의 경합이 있다. 42B는 RC 의도 확정과 업무 gate 적용 사이가 별도 저장/전달 경계로 더 명시적으로 드러난다. A의 비동기 worker·모듈성·cache/부분 갱신과 B의 유한 대기·재조회·현재성 확인을 모두 허용한다.

## 5. 같은 경합을 관찰할 지점

대표 경합은 사용자 Input1의 첫 답변 K1이 준비된 뒤 새 Input2가 이를 정정하는 동안, 별도 Agent 알림과 이전 응답 준비 완료가 들어오는 경우다.

1. Interaction Manager는 로컬 발화 시작과 즉시 stop/output epoch 변경을 기록한다. Task 취소를 의미하지 않는다.
2. Request Controller는 InputStarted/hold 의도를 기록한다. 44A는 중앙 continuation, 44B는 input/control 경로를 통해 후속 내용과 notice의 대기를 관리한다.
3. 42A는 관련 owner의 gate/command/CAS 변경을 같은 로컬 확정 순서로 조합한다. 42B는 업무 서비스가 Hold를 적용하고 HoldAck를 반환한 사실과 RC의 요청 사실을 구별한다.
4. **Hold가 K1 transmission CAS보다 먼저면** K1을 HELD로 남긴다. Input2 의미 채택과 현재 질문/Task 검사를 거쳐 reconciliation으로 K1을 REVOKED하고 별도 K2를 준비할 수 있다. ASR final이나 timeout만으로 release하지 않는다.
5. **K1 CAS가 먼저면** DISPATCHING/UNKNOWN을 숨기지 않는다. 네트워크 call 전 crash도 가능하므로 외부 접수는 별도 receipt/조회로 확인하고 같은 command/attempt에 기록한다. 이미 전송을 시작한 효과는 capability 범위에서 정정/취소 효력을 확인한다.
6. 45A/B는 새 query/유효 source를 반환한다. 재사용한 과거 근거로 이전 현재 binding을 복사하지 않는다. source/permission 변화는 Bundle/cache/게시 revision의 사용부터 fence한다.
7. 늦은 이전 Candidate0 준비 완료는 시작 input/generation/source 조건에서 폐기한다. 인증된 Task 알림은 새 입력이 왔다는 이유로 지우거나 기존 Task 취소로 해석하지 않는다.
8. 실제 표시·재생 receipt는 Interaction Manager → Response Manager의 내구 원장으로 반영하고 Request Controller가 실제 질문 focus/답변 연결에 참조한다.

관측할 시각은 capture onset, RC hold 요청/저장, 업무 gate 적용/ACK, transmission CAS, 외부 send/receipt, 의미 채택, publication release와 실제 전달이다. **물리적 발화 시작부터 전역 무지연 차단은 어떤 조합에서도 주장하지 않는다.** 42B의 전달 lag 및 heartbeat/lease 단절 감지 구간은 아직 수치가 동결되지 않았다. 감지/적용 전에 시작한 dispatch와 감지/적용 후 보호 상태를 구별한다. ACK를 받지 못한 대화 측은 관련 후속 효과를 보류하고 실제 업무 gate 적용 상태를 조회한다. 업무 재시작·새 control session에서는 gate를 보호 상태로 시작해 대조하며 명시적 reconciliation 전 자동 release하지 않는다.

## 6. 모델 호출과 비용의 겹침을 한 번만 계산한다

현재 의미는 41의 C-INTERPRET, 자유문 과거 관계 가공은 필요한 경우에만 45의 C-CONTEXT, 응답 준비는 필요한 C-RESPONSE, 지속 음성/전사와 출력은 C-VOICE-IN/OUT이다. 44 Dispatcher/Stage/Join, 42 intent/receipt/hold/CAS와 명시 ID/version join은 코드다.

같은 call을 여러 DP의 이익/비용으로 두 번 합산하지 않는다. B producer의 초기·갱신·backfill·retry·미사용·stale 호출 및 A optional cache 생산·재검증 호출을 동일 workload 기간에서 기록한다. 호출 횟수는 비용과 지연을 설명하는 보조값이고 role별 usage/청구가 비용이다. cloud weights/KV를 PC RAM으로 계상하지 않는다. 실제 model portfolio·단가·한도·QA freeze/측정은 후속이다.

## 7. 46과의 조합은 전체 A/B와 하위 선택을 구별한다

46은 현재 시간 근거를 구성하는 원리까지 45의 과거 근거 공급 원리를 확장한다. 시간 관계 owner는 Interaction Manager, 과거 관계 owner는 Context Manager다. 시간 overlap/순서/오차는 코드 후보이며 현재 referent/Task 정답은 Request Interpreter가 제안하고 Request Controller가 채택한다.

| 시간 근거 선택 | 과거 근거 선택 | 의미 |
| --- | --- | --- |
| 요청별 Temporal Evidence Resolver | 45A Context Composer | **46 전체 A**에 해당하는 묶음. |
| 요청별 Temporal Evidence Resolver | 45B Memory Publisher / Evidence Reader | 교차 조합. 시간 query 결과와 게시된 과거 근거를 묶는 반환 계약을 명시해야 하며 46 전체 A라고 부르지 않는다. |
| 게시 Temporal Evidence Publisher / Reader | 45A Context Composer | 교차 조합. Composer가 게시 시간 근거와 과거 owner read를 묶으며 46 전체 B라고 부르지 않는다. |
| 게시 Temporal Evidence Publisher / Reader | 45B Memory Publisher / Evidence Reader | **46 전체 B**에 해당하는 묶음. |

따라서 46 전체 A/B는 이 2×2의 두 묶음을 보여준다. 독립 시간 선택과 45 선택을 모두 검증한 결과가 아니다. 45B와 46 전체 A를 동시에 선택했다고 쓰면 과거 처리의 서로 다른 기제를 동시에 지정하게 된다. 교차 조합은 별도 조건부 설계 기록으로 설명한다.

46B의 temporal 게시 완료는 InputFinal의 새 필수 선행조건이 아니다. 같은 원본 취득·정규화·watermark/gap을 유지하고 실제 의미에 필요한 시간 자료를 조회할 때만 NOT_COVERED 생산/재조회 대기가 생긴다. 단어 timing이 없거나 clock 오차/gap이 크면 관계 후보의 UNKNOWN_TIMING/SOURCE_GAP을 유지한다. 게시 revision이 VALID라는 것과 시간 정보가 충분하다는 것은 별도 축이다. 현재 화면 capture로 과거 관측을 복구하지 않는다.

46의 과거 관계 비용/이익은 45와 한 경로로 계산하고 시간 관계의 추가 손익만 따로 설명한다. 46의 게시된 전체 MAIN/계약과 고정 temporal JSON, native 발표를 인수 검토했다. 46의 최종 독립 DP 선정이나 모든 교차 조합의 구현 인증은 하지 않았다.

## 8. 확인 범위와 후속 작업

현재 고정 JSON 44/45/42에 대해 scripts/architecture/check_redetailing_contracts.py의 기본 실행과 --dp별 실행이 통과했다. 검사는 문서 예시의 key/type/reference, 실제 원문 구간, 전달 포함 관계, source vector·게시 revision 및 주석으로 표현한 hold/CAS 선후를 대조한다. 실제 schedule, protocol failure 복원, LLM 의미 정답과 품질/비용을 실행 검증하지 않는다.

04-40 §6·§9·§13은 최신 42의 Hold/HoldAck/CAS/reconciliation 문서와 연결하여 문서 상세화와 실행 미검증을 구별해야 한다. 2026-10-10 개별 게시와 조합 검토에서 이 연결을 최신 파일에 갱신했다. 이전 UNRESOLVED 문구를 그대로 최종 결론으로 인용하지 않는다.

후속 구현/측정을 별도 승인받으면 같은 입력·권한·원본·Agent capability·외부 조건 아래 capture/hold/CAS 경합, late result, receipt 불명, source 정정/철회, 미게시/미지원, 재연결과 clarification 대기를 검증한다. 예시 validator나 그림이 통과했다는 이유로 이 결과를 생략하지 않는다. 지금 단계는 검토 가능한 문서/도식의 재구체화이며 기존 reference/QA/ADR 전체 동기화와 A/B 최종 선택은 별도 작업이다.


## 9. 검토 근거와 읽기 순서

[41](./04-41-request-resolution-control.md) → [44](./04-44-continuous-interaction.md) → [45](./04-45-memory-and-context.md) → [42](./04-42-lifecycle-ownership.md)를 읽은 뒤 [46](./04-46-input-and-context-evidence.md)의 시간 확장 범위를 확인한다. 구체 예시는 [44 실행 계약](./contracts/dp44-execution-examples.json), [45 근거 계약](./contracts/dp45-evidence-examples.json), [42 인계 계약](./contracts/dp42-handoff-examples.json), [46 시간 계약](./contracts/dp46-temporal-evidence-examples.json)이다.

독립 검토에서 §5의 ACK 미수신을 실제 HOLD 적용과 혼동할 수 있는 P2 한 건을 수정했다. 대화 측 후속 효과 보류/실제 업무 gate 조회와 새 업무 control 세대의 보호 gate를 구별한다. 그 외 공통 owner/tool port·code/model 지도·16개 조건부 조합·46의 전체 AA/BB 및 교차 계약을 대조했다. 최종 문서/도식/발표의 검수 범위와 단계별 Git 이력은 [지속 작업 기록](./redetailing-workplan.md)과 각 DP review에 있다.

별도 QA 세션의 다섯 묶음을 보며 영향 경로를 설명하지만 새 QA ID/분모/목표/우선순위를 이 문서에서 확정하지 않는다. 기존 ASR/QA/ADR/참조 Architecture와 43은 보존한다. 실제 전체 프로그램 검증과 성능 우열은 승인된 동일 조건의 후속 구현/측정에서 판단할 일이다.
