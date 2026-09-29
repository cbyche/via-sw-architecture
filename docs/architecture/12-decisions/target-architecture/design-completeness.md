# Target Architecture 주요 설계 완결성 점검

> 상태: **MAJOR_DESIGN_COMPLETE / 주요 설계 완성안 / 사용자 최종 검토 전**
> 점검일: 2026-09-29 · **구현·모델 실행·성능 측정 없음**
> [전체 구조](./architecture.md) · [판단·제어](./control-and-lifecycle.md) · [기억·Context](./memory-and-context-lifecycle.md) · [Omni 실행](./shared-omni-runtime.md)

## 1. 완료의 의미와 범위

사용자는 주요 설계를 스스로 구체화하여 완성하도록 위임했고, 구현·성능 측정은 요구하지 않았다. 이 문서의 설계 완료는 **고정된 UC·필수 변형에 대해 책임, 정상 흐름, 상태·계약, 동시성, 실패/보류/복구와 사용자 결과를 모두 정의했다**는 뜻이다. 구현 성공·모델 품질·실시간 성능 보장이나 사용자의 최종 구조 승인과 구분한다.

설계 우선순위는 QA-19 accuracy → QA-09 responsiveness → QA-29 modifiability → QA-39 reliability/recoverability다. 이 네 ASR은 현재 설계 작업 기준이며 이후 비교의 고정 목록이 아니다. Decision Point와 steelman 후보를 구체화할 때 ASR의 추가·변경과 평가 기준을 함께 정의한다. 이번 수정에서는 기존 QA 정의·점수·측정 조건 자체를 바꾸지 않는다. 이번에 새 Decision Package나 측정 freeze를 만들지 않았다.

완결성 기준은 다음과 같다.

- UC 18개와 필수 변형이 실제 runtime 경로에 연결된다.
- 모든 의미 상태에는 owner, 입력 사건, 허용 전이와 종료/보류 동작이 있다.
- 중요한 race에는 revision·transaction·fence와 외부 한계를 명시한다.
- 정보 부족·dependency 미지원·장애도 사용자에게 전달할 상태와 다음 행동이 있다.
- 주요 선택을 여러 대안 중 미정으로 남기지 않는다. 배포별 숫자·코드/학습·실측은 별도 후속 항목으로 분리한다.

## 2. 필수 기능과 변형의 설계 대응

기준은 [Representative Use Cases](../../05-representative-use-cases.md)다. 아래 “설계됨”은 문서 대조 결과이며 실행 시험 PASS가 아니다. 미지원/실패 안내는 요구된 예외 행동으로 포함하되 정상 업무 성공으로 세지 않는다.

| UC / 필수 변형 | 선택한 정상 경로 | 예외·관계·상태를 닫은 계약 | 설계 판정 |
| --- | --- | --- | --- |
| UC-01.1~4 일반 질문·후속·주제 변경·Text | 좁은 S2S 직접 경로; 후속 지칭은 Core; Text는 Core에서 직접 구성 | direct owner 단일 확정, 실제 Response 참조, 늦은 직접 후보 폐기. [제어 §2](./control-and-lifecycle.md#2-최소-s2s-직접-응답-계약) | 설계됨 |
| UC-02.1~6 파일/폴더·메일·일정·Browser·Web·기록/북마크 | Context adapter의 bounded query → 원문/근거 기반 설명 | 없음/권한 없음/미지원 구분, source 기준 시각·coverage·한도. [기억 §2~3](./memory-and-context-lifecycle.md#2-context-획득과-지칭의-정확성) | 설계됨 |
| UC-03.1~6 사전 단일·복수·연속·분리 선택·pointer·caret | 선택 유효 구간·display/window/document·typed referent와 발화 결합 | 선택 충돌, 다중 모니터/좌표 변환, scroll, 구조 없는 이미지. [기억 §2](./memory-and-context-lifecycle.md#2-context-획득과-지칭의-정확성)·그림04 | 설계됨 |
| UC-04.1~7 발화 중 지시·순차·원형·drag·집합/단일·정정·창 이동 | 표현별 acoustic interval·당시 evidence·최종 correction 관계 | 단순 이동은 후보, clock uncertainty/watermark/gap, late revision, 현재 화면 대체 금지. 제어 §1·기억 §2 | 설계됨 |
| UC-05.1~4 background·닫힌 자료·이전 설명·업무 결과 | source identity/read, Response 전달 범위와 Task artifact 조회 | 열린 자료와 실제 보이는 화면 구분, 동명이인/동명 자료·권한·삭제 처리. 기억 §2~4 | 설계됨 |
| UC-06.1~5 생략·후보 선택·형식/수신자·다중 질문·철회 | semantic 예산 안에서 보완, Pending Interaction ID에 새 Turn 결합 | 질문 focus·유일성, 기존 확정 제약 유지, 만료/철회/새 목표. [제어 §4](./control-and-lifecycle.md#4-request질문task의-상태-전이) | 설계됨 |
| UC-07.1~5 S2S/Core/복수/끼어든 대화/Voice 종료 뒤 업무 | 게시된 답변·원래 자료를 참조해 새 Task·Agent 위임 | Task 없는 답변도 durable Response, 미청취 구간 구분, 다른 주제 뒤 검색·Text 지속. 본문 §5·12·13 | 설계됨 |
| UC-08.1~5 조사·파일·메일/일정·앱·bounded 위임 | semantic 목표/완료 조건 → Controller 검증 → Task command → Gateway | 수행 가능한 Agent, 범위·권한 차이, 접수/완료 구분, 얇은 VIA 위임 시에도 같은 상태 계약. [제어 §5](./control-and-lifecycle.md#5-전송과-정정의-원자적-경계) | 설계됨 |
| UC-09.1~5 독립·순차·데이터·조건·혼합 | 한 업무 내부 단계는 통째 Agent; 독립 목표만 Request Graph·Task 분리 | true/false/unknown, 선행 실패·버전 수정·부분 결과·중복 해제 방지. 본문 §12·그림09 | 설계됨 |
| UC-10.1~5 최근/과거/동일 Agent 복수/완료 후 수정/별도 목표 | 기존 Task read/control, 같은 목표면 후속 Execution, 다른 목표는 새 Task | latest query와 마지막 확인 상태 구분, 실행 ID 재사용 금지. 제어 §4~5 | 설계됨 |
| UC-11.1~5 S2S/Core 중단·발화 정정·위임 전/후 정정 | IM local stop + input hold → 새 의미 해석 | suspended delivery·output epoch, 재개/축약/폐기, 실제 전송 후 rollback 불가. 제어 §1·5·6 | 설계됨 |
| UC-12.1~5 위임 전/중/완료 경쟁/이미 완료/취소 미지원 | WITHDRAWN 또는 cancel command와 source 확인 | 접수·완료·불가·불명 구분, 다른 Task 지속. 제어 §4~5 | 설계됨 |
| UC-13.1~6 progress·질문·완료·부분 실패·다른 앱·다른 응답 중 | inbox → Task/domain outbox → Controller → publication | UI 즉시/Voice 차례, progress 병합, terminal/question 보존, OS 알림 연결. [제어 §6](./control-and-lifecycle.md#6-출력과-대화-차례) | 설계됨 |
| UC-14.1~5 다른/같은 Agent 복수·다중 질문·역순 결과·모호한 지칭 | Task별 identity·execution correlation, Controller 질문 binding | Agent 동시 실행 미지원은 admission 직렬화, 역순 hint→query, 모호함 확인. 제어 §4~5 | 설계됨 |
| UC-15.1~5 Voice→Text/Text→Voice/재연결/업무 중/새 대화 | Conversation/Voice/Task 별도 lifecycle, focus lease | 입력 시작 대화 고정, 옛 음성 재생 금지, 다른 대화의 상세 자동 발화 제한. 제어 §1·6 | 설계됨 |
| UC-16.1~5 접근/제공/Action 승인/거부·축소/다중 대기 | 현재 Policy State와 실제 사용 port gate, typed Pending Interaction | question/action digest·Execution·revision, 철회 fence, 오래된 “응” 거절. 제어 §4~5·기억 §5 | 설계됨 |
| UC-17.1~5 선호 사용/조회/등록·수정/삭제/현재 요청 우선 | Context Manager의 명시적 기억 변경, typed state | 자동 승격 없음, tombstone·파생 view/KV 무효화, 원문 삭제와 구분. [기억 §4~6](./memory-and-context-lifecycle.md#4-보관-기본값) | 설계됨 |
| UC-18.1~6 source/모델/Agent 미지원/실패/불명/재시작 | source별 실패, runtime 상태, Task 재조회·내구 원장 복구 | capability fallback·무조건 재전송 금지·Store 실패·UNKNOWN Voice. [제어 §7](./control-and-lifecycle.md#7-장애재시작버전-변경)·Omni §8 | 설계됨 |

## 3. 이번에 닫은 주요 설계 선택

| 이전의 열린 항목 | 이번에 선택한 구조·정책 | 남는 구현/검증 구분 |
| --- | --- | --- |
| 최소 S2S admission | current-Turn-only VoiceProposal + 허용 분류·의존성·전사·질문/정정·revision host gate; Core로 같은 Request 인계 | 모델 분류의 실제 오류율·text/audio 의미 보존 |
| 형식 repair와 refinement 예산 | 모든 semantic generation 총 2회, 추가 읽기 한 묶음, 별도 구성 1회; 예산 소진 종료 명시 | tokenizer·직렬화·기기별 deadline |
| 입력 종료·시간 근거 | endpoint 후보 → ASR final + producer watermark/gap → SEALED; 늦은 근거는 새 revision | 실제 clock 오차·endpoint 정확도 |
| 질문·승인·재개 | 실제 제시된 질문 focus, ID·Task·Execution·action digest·revision 검사, 침묵 미승인 | UI/adapter 구현 |
| state transition과 race | Request/Task/Execution/Command/질문/Response 상태, owner별 Unit of Work·CAS·epoch | DB transaction과 crash 구현 시험 |
| Agent capability별 보장 | 미지원 start/answer/precondition 거절, UNKNOWN submit 재전송 금지, 순서 없는 event 조회 | 실제 Agent adapter 기능 적합성 |
| 공유 Omni scheduling | Voice+semantic 각각 최소 예산/KV, Voice deadline·foreground round-robin, 유한 quantum·safe-point cancel | Resource Profile 값·장비·열·실제 처리량 |
| 기억·Context 계층 | embedded DB + evidence files, 원본/파생 view 분리, metadata/keyword read, no automatic memory promotion | DB engine·schema·source별 binding |
| 보관·삭제 정책 | raw RAM, evidence cache 24시간, 진단 7일, 대화/Task 명시 삭제; epoch 차단 후 purge·복구 | 용량 상한·OS 보호 연결·물리 purge 관측 |
| 음성 차례·재연결 | 입력 관계/제어 먼저 → 현재 답변 → 중요 결과 → progress; 다른 대화 상세는 선택 후 | endpoint 시간·음성 rendering 품질 |
| process/Store 배치 | UI·Voice·Core·Omni·ASR·connector 경계, embedded Store + managed blobs | OS process/IPC 구현과 장비 배치 |
| sleep·lock·migration | 새 epoch·근거 gap·출력 정지·Task 재조회, schema fence·진행 실행 binding 보존 | 플랫폼별 API·migration code |

정확도 우선으로 direct 범위·candidate coverage·삭제 후 사용·정정 경쟁을 보수적으로 닫았다. Responsiveness는 입력/해석/Agent event의 동시 진행, 사전 Context, 최소 호출과 template, UI/Voice 분리로 다뤘다. Modifiability는 adapter/canonical contract와 owner migration에, reliability는 내구 command/event/publication과 불명 상태 복구에 반영했다. 이 인과 설명은 비교 결과가 아니다.

## 4. 상태와 기능을 가로지른 자체 검토

새 sub-agent나 모델을 실행한 검토가 아니다. 작성한 계약을 서로 대조하여 다음 사건 순서에서 owner와 결과가 정의되는지 점검했다.

| 사건 조합 | 계약상 결과 |
| --- | --- |
| semantic 수행 중 새 발화 | capture·ASR·Omni Voice 진행; 옛 generation 무효화; 해당 대화 미전송만 hold |
| 직접 후보 생성 중 ASR final 수정 | input_echo/revision 불일치로 폐기·같은 Request Core 인계; 중복 응답 없음 |
| 새 질문이 기존 승인 “응”일 수 있음 | direct 제외·질문 focus 검사; 유일하지 않으면 질문 대상 clarification |
| action DISPATCHING 직후 취소·crash | UNKNOWN 기록·같은 command key 조회, 확인 전 새 start 금지 |
| Task 결과 저장 직후 Controller 종료 | domain event outbox 재적용으로 publication intent 복원 |
| Agent terminal과 늦은 approval | 질문/Execution revision 검사로 오래된 승인 전송 차단 |
| 사용자가 말하는 중 다른 업무 실패 | 상세 UI·내구 알림 유지, 음성 대기; 다음 유효 차례에 실패 포함 요약 |
| 재생 중 crash·Voice 재연결 | 마지막 확인 범위 밖 DELIVERY_UNKNOWN, Text 복원·옛 음성 자동 재생 금지 |
| memory 삭제 중 summary/Omni 완료 | data/policy epoch mismatch로 결과 사용 차단, purge 재시도 |
| blob rename 뒤 DB commit 전 crash | 참조 없는 orphan 회수; 거짓 evidence 생성 안 함 |
| evidence 만료·화면 객체 삭제 뒤 follow-up | locator/source revision 재조회 또는 재지칭 요청; 마지막 화면으로 대체 금지 |
| Store full·Omni crash·sleep resume | 새 내구 admission 중지 또는 해당 모델 경로 불가 표시; local stop/확인 가능한 UI 유지·재조회 |

설계 문서 대조에서 확인한 주요 미결 동작은 남겨두지 않았다. 이것이 미발견 결함이 없다는 수학적 증명이나 실제 동작 시험은 아니다. 추후 발견한 결함은 이 원장의 새 revision으로 수정한다.

## 5. 설계 완료 밖에 남는 일

| 후속 항목 | 왜 주요 설계의 빈칸과 다른가 |
| --- | --- |
| 사용자 최종 리뷰 | 완성한 설계의 제품 적합성·구조 수용 여부를 확인하는 단계이며 결정을 사용자에게 떠넘긴 미정 목록이 아님 |
| schema/adapter/DB/IPC 구현 | 필수 field·상태·원자성·실패 행동은 문서에 선택되어 있음; 이를 코드로 구현하는 일 |
| 모델 학습·runtime 연동 | 요구 capability와 미지원 동작은 정해짐; 실제 build의 기능·품질 확보 |
| Resource Profile 수치와 장비 적합성 | 유한 필드·예약·포화 처리는 정해짐; PC별 값과 실제 동시 진행 검증 |
| 성능·오류율·복구 측정 | 아직 실행하지 않았으며 목표 설계 완료 요건에 포함하지 않음 |
| Decision Package·steelman·ASR 재정의 | 이후 후보 구체화 때 ASR 추가·변경과 비교 기준을 함께 정의; 지금 비교나 새 ASR 확정 없음 |

별도의 승인된 ADR·이전 비교 조건·QA 정의를 이 설계 완료 표기로 변경하지 않는다. 주요 설계 완성안의 읽기 시작점은 [architecture.md](./architecture.md)다.
