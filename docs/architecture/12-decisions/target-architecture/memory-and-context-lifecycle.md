# Context·기억·증거의 수명과 저장 계약

> 상태: **주요 설계 완성안 / 사용자 최종 검토 전 / 구현·성능 측정 없음**
> [전체 구조](./architecture.md) · [판단·제어](./control-and-lifecycle.md) · [기능 완결성](./design-completeness.md)

## 1. 선택한 기억 구조

**단기 근거, 중기 작업 view, 장기 사용자 기억을 구분하고, 업무 복구의 원본 기록은 별도로 보존한다.** 주 저장 구조는 local embedded transactional DB와 수명 관리되는 evidence 파일이다. 새 embedding 모델·vector DB를 기본 구성에 추가하지 않는다. Context Manager의 metadata/keyword index와 owner read port로 후보를 좁히고, 필요할 때 공유 semantic 역할이 현재 요청과 결합한다.

![기억 계층과 삭제·재구성 경계](./diagrams/13-memory-lifecycle.svg)

[draw.io 편집 원본](./diagrams/13-memory-lifecycle.drawio)

| 계층 | 저장과 소유권 | 승격·소멸 조건 |
| --- | --- | --- |
| 단기 입력 근거 | IM의 bounded RAM audio/screen timeline; Controller workspace; Context cache | 현재 입력·해석·불일치 해소까지 pin. 안정된 참조·최소 근거 확보 후 raw 해제 |
| 중기 작업 view | Context Manager의 최근 대화/Task summary·검색 index·read cache; source owner revision을 포함 | 접근·사용 시 필요한 범위 생성, source 변경·삭제·policy 변경이면 무효화. 원본에서 재구성 |
| 장기 User Memory | Context Manager의 typed preference/routine/stable fact, origin·허용 범위·revision·삭제 상태 | 명시적 저장/수정 요청만 commit. 자동 장기 승격 없음. 현재 요청이 우선 |
| 내구 업무·대화 원본 | Controller의 input/request/question, Task의 상태/결과 참조, Gateway의 command/event, Response의 게시·전달 기록 | 관련 owner lifecycle과 사용자 보관·삭제 정책 적용. 요약 cache 만료로 활성 업무 기록을 삭제하지 않음 |

Memory routine은 참고 정보이며 무인 예약 실행을 만들지 않는다. “항상 표로 답해”는 허용된 기억 변경이지 사용자 문서를 편집하는 Agent Action이 아니다. 반대로 “내 파일에 이 선호를 기록해”는 외부 파일 변경이므로 Agent다.

## 2. Context 획득과 지칭의 정확성

기본 Context는 현재 입력·당시 선택·window/document identity·최근 실제 응답·대기 질문·관련 Task 요약·명시적으로 허용된 선호다. 모든 파일·메일 본문을 미리 읽지 않는다. Controller가 현재 source/recipient/purpose scope를 정하고 Context Manager가 port별로 제한된 evidence를 만든다.

| 근거 유형 | 확정 가능한 범위와 계약 |
| --- | --- |
| stable object / file / artifact ID | 해당 identity와 source revision으로 직접 읽기; 이름만 같은 후보로 바꾸지 않음 |
| 선택·caret·pointer | display/window/document/viewport·좌표계·유효 구간을 결합. 오래된 선택도 교체·해제 전 구간을 보존; focus와 충돌하면 임의 우선순위로 확정하지 않음 |
| 복수 선택·drag·둘러 그리기 | 단일 대상·집합·연속 범위·분리 영역을 별도 typed referent로 유지. 원형 이동은 후보 근거이며 의도적 선택으로 자동 확정하지 않음 |
| UI 구조 없는 그래프·이미지 | 당시 screenshot crop와 좌표 변환·문서 identity로 시각 후보를 semantic 역할에 제공. 현재 화면으로 과거 crop을 대체하지 않음 |
| hidden window / file / mail / calendar / browser / web | source별 허용 read adapter; 현재 보이는 화면과 구분. 시간/검색 조건과 source 기준 시각 기록 |
| 이전 설명·업무 결과 | Response의 실제 게시/청취 범위와 Task artifact revision을 owner port로 조회. 생성만 된 문장을 “방금 말한 내용”으로 사용하지 않음 |

`Evidence Query Receipt`는 source와 query 범위, snapshot/observed revision, pagination, 반환 후보, 제외 이유, 접근 거부·실패·truncation을 담는다. 결과 총수를 알 수 없으면 `unknown`으로 기록한다. 검색 몇 건을 받았다는 사실은 completeness가 아니다.

Coverage는 **명시한 bounded scope에서** exact identity를 확인하거나, source가 지원하는 완결된 열거/검색을 끝냈을 때만 `COMPLETE`다. Public web 전체나 모든 사용자 파일을 완전 탐색했다고 주장하지 않는다. 이름·지칭 후보가 열려 있으면 query scope를 좁히거나 사용자가 자료를 선택하도록 질문한다. 읽기 전용 설명은 확인한 범위·기준 시각을 명시할 수 있지만 외부 변경 대상은 후보와 source version을 확정해야 한다.

발화 구간과 evidence producer는 clock mapping uncertainty·sequence·watermark를 가진다. 모든 필수 producer가 해당 구간의 처리를 마쳤거나 gap을 선언할 때 snapshot을 닫는다. 불확실한 시간 구간과 겹치는 후보는 함께 유지한다. Source timeout은 gap으로 바뀌며 소프트웨어 대기가 무한히 이어지지 않는다. 오래된 이벤트가 도착하면 영향받은 referent와 admission만 무효화한다.

Scroll·window 이동은 당시 identity를 바꾸지 않는다. 내용 변경·삭제는 현재 적용 가능성을 바꾸므로 Context read set과 Agent execution precondition에서 확인한다. VIA가 현재 version을 읽었더라도 Action 직전 경쟁은 Agent가 확인해야 한다.

## 3. 조회·요약·cache 규칙

1. Controller가 확정 입력, 관련 owner snapshot, 허용 scope를 요청한다. Context Manager가 source identity·revision·policy·recipient를 포함한 key로 cache를 확인한다.
2. 질문에 필요한 원문·typed record를 먼저 고른다. 최신 질문·미해결 제약·부정·대상 ID·수치·승인·상태는 요약으로만 전달하지 않는다.
3. 예산에 들어가는 관련 원문과 source receipt를 semantic 역할에 제공한다. 길면 명시적 section/page 범위와 retrieval 결과를 전달하며 누락 범위를 기록한다. 필수 근거를 잘라 전체를 읽은 것처럼 답하지 않는다.
4. 중기 summary는 `source IDs/revisions + covered ranges + omissions + memory/policy epoch`가 있는 파생 view다. 원본이 바뀌면 `DIRTY`, 검증 후 `VALID`, 삭제·철회되면 `INVALID`다. DIRTY/INVALID summary로 실행 대상을 확정하지 않는다.
5. Summary 오류·불확실성이 목표/조건에 영향을 주면 원문으로 복귀한다. 원문이 없으면 다시 읽거나 사용자에게 확인한다. 요약 결과를 다시 요약해 출처 사슬을 무한히 늘리지 않는다.
6. 백그라운드 요약은 유휴 예산에서만 진행한다. 완료 전에는 기존 typed record와 원문 window로 계속 처리하고, 오래된 summary 결과는 source revision CAS 실패로 버린다.

활성 Task·대기 질문은 recency 순서만으로 Context에서 밀어내지 않는다. 전체 Task 목록을 매 호출에 넣지도 않는다. 이름·artifact·명시 대상·최근 사용 관계로 후보 집합을 만들고 후보 누락 가능성을 receipt에 남긴다. 모호한 “그거”를 최근 Task 하나에 자동 연결하지 않는다.

## 4. 보관 기본값

다음은 이번 주요 설계에서 선택한 제품 기본 동작이다. 성능 실측이나 QA target이 아니다.

| 데이터 | 기본 보관 정책 |
| --- | --- |
| 연속 raw audio·전체 화면 | RAM ring만 사용, 상시 디스크 녹화 없음. 입력 비활성·lock·권한 철회 시 수집 중지와 buffer 해제 |
| 요청이 사용 중인 raw 구간 | 해석·정정 확인 중에만 RAM pin; 장기 WAIT_USER 진입 전에 stable referent와 최소 근거로 바꾸고 raw 해제. 원음이 다시 필요하면 재입력 요청 |
| 재시작 후 필요한 시각 근거 | 안정된 source 참조가 없고 요청 처리에 필요한 선택 crop/자료 snapshot만 현재 보관 권한 안에서 managed evidence file로 저장. 전체 화면을 대신 저장하지 않음 |
| 요청 evidence 파일·읽은 자료 cache | 비활성 마지막 사용 뒤 24시간 TTL. 활성 read/dispatch pin은 유한한 Resource Profile 상한 안에서만 유지; 상한 도달 시 보류·재조회/재지칭으로 전환 |
| Conversation Text·typed referent·Task·결과/전달 원장 | 사용자가 삭제하거나 선택한 자동 보관 정책이 만료될 때까지 local 보관. 기본 자동 기간 삭제 없음. 저장 예산 소진 시 오래된 기록을 몰래 삭제하지 않고 정리 안내·새 내구 admission 제한 |
| User Memory | 명시적 변경/삭제까지 유지; 현재 지시 우선. Conversation Text를 장기 기억으로 자동 승격하지 않음 |
| 중기 summary·검색 index | source 존재·권한 범위 안에서만 보관; 삭제/철회 즉시 조회 불가, 재생성 가능. pin 없는 cache는 LRU+TTL로 회수 |
| 진단 로그 | payload 없는 ID·revision·상태·오류 위주, 기본 7일 순환·용량 상한. 원음·화면·전체 prompt 상시 기록 없음 |
| VIA 관리 backup | 기본 cloud sync/자동 backup 없음. 명시적 local export 시 범위와 포함 정보를 보여줌. 외부 OS backup/사용자 복사본은 VIA가 삭제 완료를 보장하지 않음 |

24시간·7일은 변경 가능한 보관 설정이다. TTL이 만료돼도 이미 확인한 Task 상태·응답 이력은 원래 보관 정책을 따른다. 과거 원문/crop이 사라지면 locator로 다시 읽고 source revision을 확인하거나 근거 만료를 알린다. 이 때문에 “아까”의 상세 내용이 복구 불가능해질 수 있으며 기억을 지어내지 않는다.

Read/전달 권한과 evidence 보관 권한은 구분한다. Source가 저장을 허용하지 않으면 transient evidence만 사용하고 재시작 후 재조회한다. Diagnostic 확대는 별도 설정·동의·유한 수명으로만 가능하다. 전제된 single OS user 밖으로 기억·model cache·파일을 공유하지 않는다.

## 5. 기억 변경·삭제와 권한 철회

User Memory는 `PROPOSED → ACTIVE → SUPERSEDED/DELETED`를 가진다. 명시적 “기억해/바꿔/잊어” 요청을 Interpreter가 해석하고 Controller가 대상·scope를 확정한다. Context Manager가 기억 revision, tombstone, invalidation outbox를 하나의 transaction에 기록한다. 여러 기억이 후보면 삭제 대상을 질문한다. 현재 요청의 일회성 스타일 지시는 지속 기억을 수정하지 않는다.

**삭제는 먼저 사용을 차단하고, 물리 정리는 그다음 수행한다.**

1. 삭제/철회를 commit하면서 data/policy epoch를 올린다. Context 조회·Model Access job·Gateway 제공·Response 게시가 동일한 epoch를 검사하므로 이전 권한의 결과는 새로 사용되지 않는다.
2. Dependency index로 summary·cache·검색 index·model session/KV·대기 generation을 찾아 무효화한다. 사용 중 작업을 취소하고 pin도 삭제를 막지 못하게 한다.
3. managed evidence/기억 본문을 정리한다. 쓰기 실패·실행 중 backend 미응답이면 `PURGE_PENDING`으로 남기고 재시작 후 계속한다. 해당 데이터 접근은 이미 차단되어 있다.
4. `LOGICALLY_DELETED`와 `LOCAL_PURGE_CONFIRMED`를 구분해 사용자에게 알린다. Device 매체의 잔존 bit나 외부 backup까지 소거했다고 주장하지 않는다.
5. 삭제된 기억은 과거 Conversation summary나 이전 KV에서 자동 복원하지 않는다. Tombstone은 opaque identity·revision·삭제 시각만 보존하고 원문을 복제하지 않는다. 새로운 명시적 저장 요청이 있어야 새 기억으로 등록한다.

“이 선호를 잊어줘”는 User Memory 및 그 파생 사용을 제거하며 과거 대화 Text까지 지우는 것과 구분해 결과를 알려준다. 이전 대화를 명시적으로 조회할 수는 있지만 삭제된 선호를 현재 지시로 다시 적용하지 않는다. “이 대화도 삭제해”이면 해당 Text와 파생 view까지 제거한다.

Conversation 삭제는 실행 중 Task 취소와 별개다. 기본 UI는 대화 삭제와 업무 취소를 별도 선택으로 설명하고, 활성 Task는 Task view에 “원 대화 삭제됨”으로 남겨 조회·제어를 제공한다. 사용자가 관련 업무 데이터까지 삭제하면 새 command 제공을 차단하고 본문·Context를 지우되, 이미 실행 중인 업무의 상태 조회/취소에 필요한 opaque execution·command key와 미확인 상태만 분리 보존한다. 외부 Agent에 이미 나간 내용을 삭제했다고 표시하지 않는다. 더 이상의 추적도 제거하는 선택에는 외부 실행이 계속될 수 있다는 실제 상태를 알려준다.

권한 철회는 신규 읽기·제공·게시를 막는다. 이미 Agent에 전달된 자료는 지원되는 중단·삭제 command와 확인 결과로만 보고하며 소급 회수를 약속하지 않는다. Consent의 범위 축소는 관련 Request만 재해석/보류하고 무관한 요청은 계속한다.

## 6. 파일과 DB의 crash 일관성

구조화 상태는 한 local DB transaction 경계에 둔다. 큰 evidence blob은 DB 밖 파일이므로 같은 원자 transaction이라고 부르지 않는다.

- 쓰기: 임시 파일 기록·완료 확인 → 최종 blob identity로 원자 rename → DB manifest에 digest·크기·owner·scope·revision·expiry와 참조 commit. DB commit 전 orphan 파일은 source로 사용하지 않는다.
- 읽기: manifest ACTIVE·권한·expiry·digest/존재를 확인한다. 없거나 손상되면 EVIDENCE_GAP, source 재조회 또는 clarification; 빈 내용으로 정상 처리하지 않는다.
- 삭제: DB tombstone·use fence·cleanup intent 먼저 commit → 파일 제거 → purge 확인 기록. Crash 뒤 cleanup intent를 재처리한다.
- 재시작: 미완성 임시/orphan 파일 정리, ACTIVE manifest의 누락 표시, deletion epoch 적용 후 cache 재구성. 삭제된 데이터를 오래된 index에서 되살리지 않는다.

장기 export 복원은 새 Store incarnation과 현재 schema/deletion ledger를 적용한 import다. 오래된 command·approval·Voice delivery를 다시 실행하지 않으며, 외부 Task는 query로 조정한다. 현재 삭제 ledger와 검증할 수 없는 오래된 backup은 자동 병합하지 않고 별도 읽기/선택적 복원으로 취급한다.

## 7. 수용 범위와 비용

이 설계는 UC-02~07의 자료·지칭·대화 참조, UC-15의 채널 연속성, UC-16의 정보 통제, UC-17의 기억 제어, UC-18의 재연결을 연결한다. 수집량 제한, 파생 view 무효화, 추가 DB 기록은 비용이다. 원본을 지우고 모든 과거 문맥을 완벽하게 복원할 수는 없다. 대신 무엇이 남고 무엇을 다시 확인해야 하는지 정한다.

기억 정책의 주요 선택은 여기서 닫았다. 직렬화 schema·DB 제품·기기별 byte 상한·OS 암호화 API 연결은 구현 단계의 binding이며, 새로운 ASR이나 성능 검증 조건을 이번 작업에서 추가하지 않는다.
