# DP41 첫 세 사례의 실행 계약 초안

> **PREIMPLEMENTATION_REVIEW / NOT_IMPLEMENTED / NOT_RUN — 2026-10-11.**
> 사용자 검토 ①을 위한 계약이다. 아래 자료형과 처리 표는 실행 코드, 확정 API 또는 측정 결과가 아니다.

이 초안은 [수행 계획의 1단계](../poc-implementation-plan.md#1단계-구현-전-검토-묶음),
[공통 실행 계약](../04-40-common-execution-contract.md),
[DP41](../04-41-request-resolution-control.md)과
[ASR-QA-v6/P21](../03-02-quality-attribute-definitions.md)을 연결한다.
첫 구현 범위는 S1-01, S3-02, S4-01이다. VIA와 상태 owner는 Mac에서 실행하며 의미와 음성 모델은
클라우드 의존성이다. 기존 on-device Omni 참조 구조를 이 PoC의 배치 조건으로 사용하지 않는다.

## 1. 먼저 확인할 결론

기존 [RequestFrame v1 schema](../contracts/dp41-request-frame-v1.schema.json)는 **설계 검토용**으로
보존한다. 현재 schema의 JSON을 통과시키는 것만으로 첫 세 사례를 실제 처리할 수는 없다.

| 기존 계약의 빈틈 | 첫 구현에서 필요한 닫힌 처리 |
| --- | --- |
| `KNOWN_REF`는 이미 제공하거나 조회한 참조만 받는다. “D1”이라고 말한 사실이 문서 내용 제공은 아니다. | 명시 source handle로 원본을 읽는 `EXPLICIT_SOURCE` selector와 `READ_FIELD(DEADLINE)` |
| `DIRECT_RESPONSE/NEW_TASK`와 자유 `instruction`만으로는 실제 행위, 메일 수신자와 본문 원천을 코드가 구별하지 못한다. | goal별 `operation`과 operation에 맞는 유한 argument. 도메인 작성 지시는 별도 문자열로 유지 |
| `USE_RESULT`에는 어느 실행의 어느 산출물인지와 준비 전 참조의 의미가 부족하다. | 현재 request/goal/revision을 가진 symbolic producer/result 참조. 실제 산출물 ID는 생산 후 결합 |
| `KEEP_PROPERTY`는 속성 유지이며 “문장에서 표로 변경” 연산이 아니다. | 기존 Task에 대한 `SET_PROPERTY(FORMAT,TABLE)`과 현재 Task baseline에서 다른 속성을 보존하는 patch |
| 목표가 아닌 메일에 “그대로”를 적용하는 표현이 없다. | 별도 `preservations`로 해당 Task의 기존 지시 유지와 정정으로 인한 신규 명령 없음 표시 |
| `source_text` 일치만으로 현재 원문, 이전 조건과 조회 근거의 출처를 구별하기 어렵다. | host가 소유하는 원문 span/기존 채택 기록/read receipt 연결. 모델이 source revision을 발명하지 못함 |

새 구현용 초안 이름은 **`dp41-pilot-frame-v1-draft`**로 제안한다. 기존 `schema_version: 1`과
호환된다고 표시하지 않는다. 채택 시 기존 schema의 top-level enum/추가 필드 금지 검사를 우회하지 않고,
별도 schema 및 dispatcher version을 함께 만든다. 기존 schema와 출력 예시는 이 단계에서 수정하지 않는다.

이 작은 계약의 목적은 세 사례를 처리할 수 있는 code branch를 정의하는 것이다. P21 전체나 임의의
자연어를 표현한다고 주장하지 않는다. pilot 결과를 보고 목표와 점수에 맞춰 연산을 보태지 않는다.

## 2. 공통 경계와 후보가 소유할 차이

| 공통 기능 | DP41 A 내부 | DP41 B 내부 |
| --- | --- | --- |
| 동일 `InterpretationInput`, 최근 대화 범위, 읽기 scope, source/Task/질문 port, model adapter | 모델 주도 해석 제어기가 허용 `READ/CLARIFY/PROPOSE`를 실행 | 요청 구조화기가 frame을 만들고 Engine이 미해결 항목을 처리 |
| 동일 타입/권한/현재성 검사, command/receipt port, Publication/Delivery receipt | read 결과는 모델의 해석 맥락으로 돌아감. 다음 read, 질문과 전체 MeaningProposal은 모델이 선택 | read 결과는 항목 상태로 돌아감. 다음 read, 부분 호출, 질문과 전체 결합은 Engine 코드가 선택 |
| 동일 유한 실행 예산과 cloud 동시 호출 상한. 동일 실제 source와 모델 자원 조건 | 구조화 출력, 독립 read의 병렬 실행, 기존 의미의 검증 가능한 부분 patch 허용 | 초기 frame 이후 부분 모델 호출, frame 형식 재생성, 실제 사용자 확인 허용 |
| Request Controller가 공통 MeaningProposal을 검증하고 저장 후 채택 | validator가 의미를 정답으로 수정하거나 code가 미리 다음 read를 선택하지 않음 | 마지막 전체 Task/대상/관계 binding을 모델이 다시 만드는 경로는 순수 B로 보고하지 않음 |

공통 runner는 입력과 외부 사건을 제공하고 관측한다. 후보 대신 read 전략, 항목 전이, continuation,
현재성에 따른 의미 재구성 또는 재시도를 선택하지 않는다. 공통 read executor는 요청 인자를 검사하고
원본 port를 호출할 뿐, “D1이면 정답 날짜를 내라” 같은 case별 정답 처리기를 갖지 않는다.

P21은 S1-01의 직접/한정 위임과 S3-02의 직접 요약/위임 요약, 한 Agent에 전체 의존 graph를 맡기는
경로도 허용한다. 첫 구현의 추천 경로는 **S1-01 직접 조회, S3-02 VIA의 두 목표 연결**이다.
실험 route 정책을 양안에 같게 적용하며 다른 허용 경로가 나온 것을 오류로 바꾸지는 않는다.
한 Agent에 전체 graph를 맡기는 경로를 지원하지 않으면 pilot 구현 범위로 명시하고 P21 전체 지원을
선언하지 않는다. Agent가 graph를 실제 수행하는 별도 stub 계약 없이 그 경로의 C5를 통과시킬 수 없다.

## 3. Host envelope와 frame 본문

`InterpretationInput`의 host envelope에는 `conversation_id`, `request_id`, `input_id/input_revision`,
`job_id`, 시작 source/policy epoch, final transcript, 이전 채택 의미/조건, 실제 제시된 질문 snapshot,
허용 읽기 scope와 예산을 둔다. 입력 시작, 발화 종료와 final 전사 도착은 서로 다른 사건이다.
첫 mock 단계의 전사 문자열은 후보 공통 입력이며 실제 ASR 의미 정확성을 평가하지 않는다.

모델은 이 envelope의 권한, revision과 예산을 늘릴 수 없다. envelope는 Model Access가 반환을
실제 call/job에 연결할 때 붙이며 모델이 echo한 식별자를 권위로 사용하지 않는다.

새 frame 본문은 기존 `goals/references/relations/constraints/unparsed_spans`를 유지하고 다음을 추가한다.

| 필드 | 닫힌 의미 |
| --- | --- |
| `frame_version` | 정확히 `dp41-pilot-frame-v1-draft`. 기존 검토 schema와 별도 버전 |
| goal의 `operation/arguments` | 아래 §4 지원표의 discriminated object. 다른 operation의 인자는 거절 |
| `preservations` | `{reference_id, mode:"NO_NEW_COMMAND", source_span_id}` 배열. “그대로”인 Task의 기존 지시를 유지하며 이번 정정으로 신규 수정/취소/재전송하지 않음 |
| goal/reference/relation/constraint의 `source_span_ids` | host가 준 현재 원문 span ID 또는 제공한 기존 채택 조건의 origin ID만 사용 |
| `unparsed_spans` | 미해석 원문과 `UNSUPPORTED_OPERATION/UNSUPPORTED_RELATION/UNCERTAIN_ROLE`의 구분. 비어 있지 않은 목표를 완료 처리하지 않음 |

span은 transcript의 Unicode code point `[start,end)`와 input revision, 원문 hash에 연결한다.
read content의 span은 source locator/hash/revision과 QueryReceipt에 연결한다. 원음 sample-clock과
전사 문자열 offset을 같은 단위라고 가정하지 않는다. 같은 표현이 두 번 나오면 offset으로 구별한다.
단순 substring 일치, 전 문장 coverage 또는 유효 JSON은 의미 정답을 보증하지 않는다.

Frame에는 실제 Task binding, 임의 실행 식, SQL, 정규식 프로그램 또는 명령 ID를 넣지 않는다.
`KNOWN_REF`는 host가 실제 제공한 opaque handle만 허용하고, 모델이 출력한 ID가 우연히 DB에
존재한다는 이유로 binding을 허용하지 않는다. Goal ID와 reference ID는 frame 내부의 잠정 이름이다.

다음은 S3-02의 **문서 데이터 예시**다. 모델이 실제로 반환한 기록은 아니다. 각 배열 object는
추가 key를 금지한다. argument의 reference 값은 같은 frame의 reference ID, goal-result 값은 같은
frame의 producer goal ID만 가리킨다. source span ID는 host가 같은 입력에 제공하는 별도 span 표의 ID다.

```json
{
  "frame_version": "dp41-pilot-frame-v1-draft",
  "input_ref": "input-s3-02",
  "goals": [
    {
      "id": "g-summary",
      "kind": "NEW_TASK",
      "operation": "SUMMARIZE",
      "target_ref": null,
      "arguments": {"source_reference": "r-document"},
      "instruction": "지정 문서 요약",
      "source_span_ids": ["span-summary"]
    },
    {
      "id": "g-mail",
      "kind": "NEW_TASK",
      "operation": "SEND_MAIL",
      "target_ref": null,
      "arguments": {
        "recipient_reference": "r-recipient",
        "body_result_reference": {"kind": "GOAL_RESULT", "producer_goal_id": "g-summary"},
        "action": "SEND"
      },
      "instruction": "요약을 메일 본문으로 전달",
      "source_span_ids": ["span-send"]
    }
  ],
  "references": [
    {
      "id": "r-document",
      "source": "CONTEXT",
      "mention": "D1",
      "selector": {"op": "EXPLICIT_SOURCE", "source_kind": "DOCUMENT", "name": "D1"},
      "source_span_ids": ["span-document"]
    },
    {
      "id": "r-recipient",
      "source": "CONTEXT",
      "mention": "민수",
      "selector": {"op": "EXPLICIT_SOURCE", "source_kind": "CONTACT", "name": "민수"},
      "source_span_ids": ["span-recipient"]
    }
  ],
  "relations": [
    {
      "op": "USE_RESULT",
      "from_goal": "g-summary",
      "to_goal": "g-mail",
      "source_span_ids": ["span-result-use"]
    }
  ],
  "constraints": [],
  "preservations": [],
  "unparsed_spans": []
}
```

`GOAL_RESULT`에는 K현재의 본문, 기존 Task ID나 과거 결과 ID가 없다. Engine이 현재 host envelope의
request/producer revision을 붙여 symbolic ref를 구성한다. frame의 SEND 의도는 owner가 확인한
실제 제공/실행 권한과 별개이며 Engine이 권한을 만들지 않는다.

## 4. 첫 세 사례의 실제 dispatch 범위

### 4.1 Selector

| 연산 | Precondition과 port | Postcondition / 실패 |
| --- | --- | --- |
| `EXPLICIT_SOURCE(source_kind,name)` | kind=`DOCUMENT/CONTACT`, name은 현재 원문에 있는 “D1”/“민수” 같은 문자열. host의 허용 namespace에서 metadata lookup 후 `context_retrieve` | locator와 실제 read receipt를 얻음. 같은 이름의 복수 source면 AMBIGUOUS. 모델이 임의 opaque handle을 만들어 binding하지 않음 |
| `RECENT_TASK(query,limit)` | 해당 Conversation 범위, 확인된 Task 상태에 `task_retrieve`. query는 검색 문자열이며 실행 코드가 아님 | 후보 ID/revision와 검색 coverage. 최신 또는 유일 후보라는 이유만으로 의미 정답을 주장하지 않음 |
| `RECENT_INTERACTION(query,limit)` | 실제 대화/질문/응답 전달 범위에 `interaction_retrieve` | 원문/채택 의미/질문/실제 전달 receipt. 준비만 된 질문은 제시된 질문으로 반환하지 않음 |
| `KNOWN_REF(handle)` | 같은 job/read set 또는 유효하게 제공된 이전 채택/질문 snapshot의 handle | owner/type/revision/권한 확인 후 후보로 사용. 임의 ID, 범위 밖 참조와 stale handle은 거절 |
| `INPUT_SELECTION` | 기존 검토 schema의 연산이지만 첫 세 사례에는 선택 근거 없음 | 첫 pilot은 호출 거절 `NOT_SUPPORTED_IN_PILOT`. S2-01 확장 전에 시간/선택 근거 계약을 구현 |

pilot에서 무의미한 화면 선택을 가짜로 만들어 조회하지 않는다. 후속 P21 확장에서는 각 selector의
source kind와 reference의 `CONTEXT/TASK/INTERACTION`이 맞는지 검사하고 pagination/coverage를 유지한다.

### 4.2 Goal과 typed arguments

| Goal kind / operation | 코드 분기와 인자 | 인계 조건 |
| --- | --- | --- |
| `DIRECT_RESPONSE / READ_FIELD` | `source_reference`, `field:"DEADLINE"`. 허용 원문 읽기 후 모델/응답 작성 경로가 날짜를 해석 | read 사실에 근거한 답변. 날짜의 자유문 의미 판정은 별도 oracle/judge이며 코드가 D1의 정답을 hardcode하지 않음 |
| `DIRECT_RESPONSE / SUMMARIZE` | `source_reference`. 실제 원문으로 내용 생산 job을 만들고 immutable artifact를 등록 | 내용 생산은 실제 모델 또는 지정 반환 mock. 비어 있는 “요약 예정”을 실제 요약으로 등록하지 않음 |
| `NEW_TASK / SUMMARIZE` | 같은 `source_reference`와 domain instruction. 현재 goal의 외부 요약 요청 | Agent capability/제공 scope 검사를 거쳐 위임. stub은 실제 payload와 원문을 사용 |
| `NEW_TASK / SEND_MAIL` | `recipient_reference`, `body_result_reference`, `action:"SEND"` | 모델/frame은 SEND 의도와 권한 근거를 전달. 실제 SEND gate/permission/currentness/idempotency는 Task/Policy/채택 실행 책임 |
| `UPDATE_TASK / SET_PROPERTY` | `target_reference`, `property:"FORMAT"`, `value:"TABLE"`. 실제 Task ID는 해결 후 결합 | 기존 Task baseline에 FORMAT만 교체. 기간/자료/수신자/다른 Task는 변경하지 않음. 외부 변경은 Agent 명령 |
| `UNRESOLVED` | 미해석 원문과 이유를 보존 | 완성 위임 금지. 확인/지원불가 안내가 필요한 기능 성공을 대신하지 않음 |

`READ_FIELD`와 `SUMMARIZE`는 의미 모델의 도구 선택/코드 결합을 구별하기 위한 처리 방향이다.
Engine이 원문에서 “마감일은 10월 15일”을 정답으로 해석하거나 요약 문장을 code로 작성하는 연산은 아니다.
`instruction`에는 요약/보고서 작성의 도메인 내용만 남긴다. 수신자, 기존 Task 연결, SEND 의도,
current-result 의존과 수정 범위를 자유문에만 두면 frame 미완성으로 판정한다.

### 4.3 Relation, 조건과 보존

| 연산 | Precondition / 실제 의미 | 결과와 금지 |
| --- | --- | --- |
| `USE_RESULT(G1,G2)` | 동일 request의 선언된 producer goal G1과 consumer G2. cycle 없음 | Engine이 `request_id/producer_goal_id/producer_goal_revision`의 symbolic result ref를 만든다. 과거 result ID를 대체 사용하지 않음 |
| `AFTER(G1,G2)` | 선언된 목표 사이의 선후 관계 | 성공한 선행 목표 뒤 consumer 실행. baseline 실패 처리=BLOCK/실패 안내. M-04의 실패 뒤 계속 의미는 별도 변경 과제 |
| `INDEPENDENT(G1,G2)` | 두 목표에 서로의 필수 입력/조건 없음 | 코드가 잘못된 의존 edge를 추가하지 않음. 실제 queue 순서는 독립 관계를 바꾸지 않음 |
| `FORBID_ACTION(scope,action)` | scope=선언된 goal ID. 검토 schema의 `SEND/DELETE/OVERWRITE` | 금지 조건을 채택과 인계까지 유지. 허용 권한을 새로 생성하지 않음 |
| `KEEP_PROPERTY(scope,property,value)` | 이미 확인된 goal/Task 속성. `FORMAT/LANGUAGE/AUDIENCE`의 현재 typed 값 | 보존 조건. `SET_PROPERTY`로 지정한 같은 속성과 모순이면 CONFLICT, 임의 우선순위로 해결하지 않음 |
| `preservations(ref,NO_NEW_COMMAND)` | 기존 Task의 원래 지시를 읽고 현재 입력의 “그대로” span에 연결 | 정정으로 해당 Task의 신규 명령/취소/재전송 없음. 원래 허용한 업무의 계속 실행은 차단하지 않음 |
| `WHEN`과 다른 predicate | 기존 schema에는 `STATE/KIND/YEAR`와 `EQ/NE/IN`이 있으나 첫 세 사례에는 없음 | pilot dispatcher 미지원. 실제 조건 event/UNKNOWN/else/현재성 계약을 구현하기 전 유효 JSON만으로 지원을 선언하지 않음 |

`USE_RESULT`가 해결되어 **전체 의미를 채택할 수 있는 시점**과 산출물이 준비되어 **SEND를 실행할
수 있는 시점**은 다르다. 의미 확정은 source/수신자/producer 관계가 충분하면 symbolic result ref로 끝난다.
K현재의 정답 내용이나 artifact ID를 처음 frame에 넣지 않고 실제 산출물 준비를 의미 확정의 필수
대기로 만들지도 않는다. Task의 결과 gate가 현재 producer의 성공/실제 artifact/허용 제공 범위를
확인한 뒤 immutable artifact ref를 SEND payload에 연결한다. 중복 event ID는 동일 효과를 다시 열지 않는다.

A의 공통 MeaningProposal도 같은 positive SEND 의도, symbolic result ref, 부분 patch와 보존을
표현할 수 있다. B만 더 풍부한 실행 권한을 가지는 비교가 아니다.

## 5. B의 항목 상태와 반환 계약

각 항목은 `field_id`, frame/input revision, source span, status, read IDs, 후보 집합, dependency refs,
model call IDs, uncertainty/reason과 terminal disposition을 가진다. 이 상태는 Request Interpreter의
잠정 상태다. Engine의 별도 영속 원본이나 두 번째 채택 transaction을 만들지 않는다.

| 상태 | 다음 코드 처리 |
| --- | --- |
| `UNRESOLVED` | 지원 selector/operation 검사 후 read를 예약. 충분한 supplied handle이면 현재성 확인 |
| `READ_PENDING` | 일치하는 read ID 대기. unrelated 반환은 상태를 바꾸지 않음 |
| `CANDIDATES_READY` | 지원 typed 조건을 적용. 의미 구별이 남으면 해당 부분 모델 호출 또는 확인 |
| `PARTIAL_PENDING` | 허용 field와 후보 범위의 반환만 대기. 전체 graph 반환은 거절 |
| `BOUND` | 충분한 근거에서 참조 연결. 의미 정답 보증은 아니며 공통 검증/채택 필요 |
| `AMBIGUOUS` | 후보를 임의 선택하지 않고 구별할 근거/부분 해석/실제 확인 질문 제안 |
| `CLARIFICATION_PENDING` | 당시 후보 snapshot과 질문 version 저장. 실제 전달 전 답을 받은 것으로 처리하지 않음 |
| `STALE / CONFLICT` | 영향받는 항목과 의존 결합만 무효화/재조회. 무관 조건 유지 |
| `UNSUPPORTED / DENIED / FAILED` | 원인과 원문을 남겨 지원불가/유보/실패 반환. 시간 표본과 청구에서 제거하지 않음 |

`read` 반환은 `status`, `read_id`, 검색 scope/limit/cursor, 실제 후보와 source locator/revision,
허용 발췌, source vector, policy epoch, `coverage/gaps`와 QueryReceipt를 포함한다.
`OK`, 검색 범위의 `NO_MATCH`, `NOT_COVERED`, `DENIED`, `UNAVAILABLE`, `STALE`을 구별한다.
`NO_MATCH`를 전체 이력에 해당 Task가 없다는 증거로 사용하지 않는다. `NOT_COVERED`는 B frame의
표현 불가와 별개다. 이 pilot의 Context 기준은 45A의 원본 조회이며 45B의 무료 정답 게시를 가정하지 않는다.

부분 모델의 **허용 payload 초안**은 다음처럼 한 가지 field의 후보 선택만 반환한다.
host의 call/job/input/frame/질문 revision 연결은 이 payload 바깥에 둔다.

```json
{
  "partial_version": "dp41-pilot-partial-v1-draft",
  "kind": "SELECT_CANDIDATE",
  "field_ref": "report_reference",
  "candidate_ref": "candidate-2",
  "evidence_refs": ["read-7"],
  "uncertain": false
}
```

`field_ref`는 Engine이 요청한 하나의 항목, `candidate_ref`는 제공한 유한 snapshot의 handle 또는
`null`, `evidence_refs`는 실제 제공한 read ID만 허용한다. 추가 key와 새 후보, 다른 field, 전체
goals/relations/constraints, 임의 patch 식은 거절한다. `uncertain:true`나 후보 밖 반환은 binding하지
않는다. 선택이 의미 정답이라는 검증은 oracle가 하며 코드가 정답 candidate로 고치지 않는다.

S4-01의 표 형식은 초기 frame의 `SET_PROPERTY` typed 값으로 추출한다. 부분 호출이 필요하면
별도 discriminated `TYPED_VALUE` 계약으로 요청한 FORMAT의 enum만 반환하도록 확장한다.
이 확장을 첫 구현에서 쓰려면 schema와 실제 소비 code를 함께 준비하며 위 후보 선택 schema로
임의 속성 값을 받아 처리하지 않는다. 새 사용자 정정 전체는 새 input의 frame 또는 A 제안으로
해석한다. 부분 모델 반환으로 요청 전체를 몰래 교체하지 않는다.

형식 오류는 오류 목록과 같은 원문을 모델에 주어 초기 frame을 재생성할 수 있다. 이는 실제 Task
binding을 모델에 맡기는 fallback이 아니다. 모호한 후보는 부분 호출/질문으로 처리하며 지원하지
않는 operation은 호출을 반복해서 지원된 것으로 바꾸지 않는다. 재시도/폐기된 호출도 모두 남긴다.

## 6. 공통 MeaningProposal과 채택 검증

완성 MeaningProposal은 양안 모두 `input/job/revision`, 각 goal의 kind/operation/처리 방향,
확인된 source/Task/recipient binding, symbolic result refs, 관계, typed patch, 금지/보존 조건,
read set/source vector와 미해결 원문을 가진다. 모델 출력 형식이나 내부 frame을 정확성 정답으로 강제하지 않는다.

1. 형식/참조: 미지원 enum, 추가 권한, dangling reference, 관계 cycle과 goal/operation 불일치 거절.
2. 출처: source span/origin/read ID가 실제 제공 범위에 있고 source/Task/질문 owner가 맞는지 검사.
3. 현재성: 시작 input과 현재 revision, Task 상태, 질문 snapshot/실제 제시, 필요한 source와 정책 확인.
4. 조건: 확인된 이전 조건, typed 변경 범위, action 금지와 전달 범위의 일관성 검사.
5. 채택: 필요한 의미/질문 레코드 저장 성공 뒤 Request Controller가 채택. 저장 실패는 성공 인계 금지.
6. 실행: Task/Policy gate가 실제 명령 접수/전송에서 현재 권한, result 준비, hold와 command ID를 검사.

이 검증은 **공통 안전/일관성 검사**다. 잘못된 유효 Task, 잘못 추출한 마감일 또는 빠뜨린 자연어
절을 항상 잡아내는 정답 oracle가 아니다. QA-01은 채택 의미, 실제 payload/효과/상태와 전달을
평가 전용 정답에 연결하여 C1~C6를 판정한다. 틀린 의미를 검증기가 정답으로 수정하지 않는다.

S4-01에서 새 입력 onset은 관련 미전송 후보의 hold를 먼저 유발한다. 확정 정정의 해석이 기존
요청과 연결되면 effective revision을 갱신한다. t=2.1초에 돌아온 R옛 job/read는 시작 revision과
dependency를 유지한다. 현재 의미를 덮거나 새 명령을 만들지 못하며 유효한 source read만 다시
검증하여 재사용한다. 실제로 이전 job이 없는 구조에 가짜 late job을 만들어 같은 비용을 강제하지 않는다.
원격 cancel 실패로 계속 청구된 이전 호출은 폐기 여부와 별개로 원장에 남긴다.

## 7. 세 사례의 끝까지 연결과 P21 확장

| 사례 | 첫 구현에서 필요한 경로 | 채택/실행을 혼동하지 않을 조건 |
| --- | --- | --- |
| S1-01 | D1 handle 찾기 → 허용 D1@v1 read → 마감일 의미/답변 준비 → 실제 Text/Voice 전달 | 직접 경로는 No Tracked Task. 위임이면 해당 신규 조회 Task이며 외부 변경 효과 0. 정답 날짜는 후보가 생산 |
| S3-02 | D1/연락처 read → 요약/메일 goal과 `USE_RESULT` 채택 → 현재 summary artifact 생산 → SEND gate → 접수/완료 각각 전달 | R옛/K옛@v0 event가 현재 producer를 채우지 않음. 중복 현재 event는 SEND 1회. 접수+0.2초와 SEND 접수+8초 final tail 구별 |
| S4-01 | 기존 두 Task/원래 조건 read → 보고서 FORMAT patch/메일 보존 → r2 채택 → 변경 명령/receipt → final 전달 | 변경 접수 1회, 메일 재위임 없음. 늦은 R옛 채택 금지. 접수+0.2초와 접수+8초 ReportRevised@v2 tail 구별 |

다음 표는 **추가할 처리 책임의 목록**이며 현재 지원 선언이 아니다. 기존 v1 enum이 있어도 실제
port/dispatcher/소비자가 없으면 `NOT_IMPLEMENTED`다.

| P21 사례 | 첫 세 사례 뒤 필요한 확장 |
| --- | --- |
| S1-02 | 보고서 작성 `NEW_TASK` 도메인 계약, 해당 input/constraint 인계와 final tail |
| S2-01 | 발화 span의 두 지칭/관측시각 selector, 두 근거의 비교/동일 시점 연결. Text-to-Text Qwen에 raw 구조 사실을 제공하며 지칭 정답은 후보가 만듦 |
| S2-02/04/05 | 실제 과거 전달/Task 결과의 versioned selector, 다중 source/예외 조건, 새 제안서 goal, 유효 reuse와 coverage gap. 45A/B의 생산/조회 port 구현과 별도 연결 |
| S2-03 | 과거 source revision 지정과 당시 값 보존. “최신이면 정답” 규칙 금지 |
| S3-01 | 일정 시간 범위 selector와 `INDEPENDENT`, 두 직접/위임 목표 모두 전달 |
| S3-03 | 일정 owner의 typed FREE/BUSY/UNKNOWN, WHEN의 조건/대체 안내 branch, guarded 생성의 version 충돌. 임의 predicate expression 금지 |
| S4-02 | 공통 local stop와 별도의 지정 수치 설명 goal. stop를 의미 응답 완료로 세지 않음 |
| S4-03 | `CANCEL_TASK`의 대상 binding, 접수/UNKNOWN/외부 확인 상태와 사실에 맞는 안내 |
| S5-01/02 | 외부 질문/결과의 Task/질문/실행/전달 현재성 port. 사용자 입력이 없는데 가짜 frame/model call을 만들지 않음 |
| S5-03 | Voice 연결 수명과 Conversation/Task 수명 분리, 결과 실제 전달/재연결 입력 연결 |
| S5-04 | S4-01의 patch를 네 외부 사건과 함께 처리. 추가 frame 문법보다 44/42의 실제 동시 제어와 공통 port 검증이 필요 |
| S6-01 | 실제 제시 질문/후속 답변, 유한 후보 선택 partial, 질문의 원래 조건 보존 |
| S6-02 | 철회된 질문 답과 제공 범위/발송 승인 구별. 새 허용을 SEND 승인으로 확대하지 않음 |
| S6-03 | source DENIED의 지원/유보 응답과 우회 조회/성공 주장 금지 |
| S6-04 | `ANSWER_QUESTION`, 실제 제시한 question ID/revision의 답 payload와 접수 연결. M-05의 신규 답변 범위 token은 아직 추가하지 않음 |

도메인 감성 조건, 임의 사용자 프로그램, 새로운 VIA 소유 관계, operation을 frame에 자유 문자열로
넣는 경로는 미지원이다. 실제 Agent capability가 의미를 보존하는 하나의 도메인 업무로 위임 가능한
경우와 VIA가 여러 Task 사이의 새 관계를 직접 구현해야 하는 경우를 구별한다. 전체 binding 모델
fallback을 추가하면 `B_HYBRID`로 별도 구성/trace를 만들며 순수 B 결과에 합치지 않는다.

## 8. 예산, 오류 분류와 완료 기준

첫 mock 구현의 예산은 결과 전에 양안에 동일하게 설정한다. 검토 제안은 goal 최대4개,
reference 최대8개, read의 limit 최대10개와 한 logical Request의 read 최대16회/의미 호출 최대8회다.
이는 작은 지원 범위와 runaway 방지를 위한 **검토 제안**으로 아직 freeze가 아니다. 모든 부분 호출,
형식 수정/재시도는 같은 호출 예산을 소비하며 확인 답변으로 동일 logical request 예산을 무한히
초기화하지 않는다. 같은 Request의 재시도와 해석 generation을 새 job ID로 바꾸어 예산을 초기화하지
않는다. 첫 세 사례의 의미 호출8회는 해당 Request에 귀속된 C-INTERPRET/C-CONTEXT/
C-RESPONSE/C-DECISION의 합이다. 음성 호출과 이전 Request 준비 호출도 별도 원장에 모두
남기며 의미 호출 카운터 밖이라는 이유로 비용에서 제외하지 않는다. 후속 45B의 Request 비귀속
배경 생산 예산은 별도 계약으로 정하고 전체 활동의 비용/메모리에 포함한다.
P21 전체의 cloud 동시 호출 상한4개와 불필요한 추가 확인 최대2회는 유지한다.
provider timeout과 case 전체 예산은 mock trace 검토와 실제 provider 연결 전에 별도로 확정한다.

QA-02 5초/QA-03 30초 cap은 보고의 실패 처리 값이다. 입력 문자열, 성공 응답 또는 호출 횟수를
이 목표에 맞추는 구현 규칙이 아니며 실제 관측 시간을 cap 값으로 덮지 않는다. 첫 세 사례에는
P21 전체 분모나 0~6점 최종 등급을 붙이지 않는다.

| 오류 원인 | 확인하는 자료 / 구분 |
| --- | --- |
| 모델 의미 오류 | 원문/동일 허용 근거에서 frame/A 제안/부분 선택이 이미 잘못됨. 유효 JSON과 실제 유효 Task라도 오류 가능 |
| 표현/지원 범위의 한계 | 요구 의미를 정확히 추출해도 선언된 selector/op/관계/값으로 표현하거나 처리할 수 없음. 지원 축소 또는 runtime 계약 확장이 필요 |
| 구현 버그 | 올바른 지원 frame/제안과 read 결과를 받았는데 code 전이/binding/currentness/payload가 계약을 위반 |
| 공통 연동/관측 문제 | 올바른 후보 출력 뒤 adapter/receipt/집계가 손상. 후보 timeout/미지원과 수집기 결함을 구별 |
| 인과 불확실 | 모델 선택과 부족한 source/규칙/공통 구현이 함께 영향을 줌. 구조적 원인이나 상대안 우월성을 임의 확정하지 않음 |

지정 모델 반환은 code 경로와 오류 탐지 확인에 쓸 수 있다. 이를 실제 모델 의미 정확성으로
보고하지 않는다. 실제 모델 pilot에서는 출력을 사후 정답으로 수정하지 않는다. 실패/수정 전
trace를 보존하고 source version과 변경 이유를 남겨 양안을 동일 조건에서 재확인한다.

첫 DP41 단계 완료는 ① 같은 입력/권한/원본으로 세 사례를 끝까지 처리 ② A의 read 결과가 실제로
다음 모델 판단에 소비되고 B의 read 결과가 실제 항목 상태/코드 결합에 소비됨 ③ B의 부분 호출과
미지원/모호함을 정직하게 처리 ④ actual payload/효과/전달에서 oracle 판정 ⑤ late/duplicate/wrong
반환이 raw에 남고 현재 결과를 오염시키지 않음 ⑥ 모든 모델 호출과 실제 비용/시간 필드를 재구성할
수 있음이다. 이는 비교 코드의 준비 완료이며 두 후보의 정확성 통과나 품질 차이 발생을 요구하지 않는다.

사용자 검토 ①에서는 **새 runtime draft의 추가 연산과 지원 범위, 첫 추천 route, 미지원 P21 확장,
원문/현재 producer/부분 patch의 의미**를 확인한다. 세 사례만으로 B의 유한 표현이 VIA 전반을 충분히
지원한다는 결론은 낼 수 없고, 동일값이나 A의 작은 보완으로 차이가 사라지는 결과도 인정한다.
