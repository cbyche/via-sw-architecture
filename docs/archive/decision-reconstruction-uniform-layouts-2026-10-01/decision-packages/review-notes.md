# 설계 선택 복원과 자체 반론 검토

> **작성자의 문서 대조·사건 추적 / 독립 심사·실행 검증 아님 / 사용자 선정 전**
> 2026-09-30 · [후보 표](./README.md) · [선발 원칙](./selection-principles.md)

## 1. 무엇을 다시 검토했는가

출발점은 검토 완료 target의 [전체 구조](../target-architecture/architecture.md), [제어](../target-architecture/control-and-lifecycle.md), [기억](../target-architecture/memory-and-context-lifecycle.md), [공유 Omni](../target-architecture/shared-omni-runtime.md), [설계 완결성](../target-architecture/design-completeness.md)이다. 고정 UC와 사용자 합의를 구분한 뒤, 기능을 제공하는 현재 처리 방식에서 갈림길을 복원했다. 기존 Component별 후보나 이전 DP 번호에서 새 목록을 분해하지 않았다.

후속 사용자 요청에 따라 기존 7개와 VIA-DP-01~18의 설계 아이디어·ADR을 다시 대조했다. [별도 검토 기록](./reference-idea-review.md)에 각 자료의 반영·유보 이유를 남겼다. 특히 의미 정정 계약, read-set 확장과 조회 시점의 구별, 필수 Agent 사건 보존, 생성 순서와 게시 권한의 구별을 네 상세 문서에 보강했다. 이전 자료와 관련이 없거나 전부 새 아이디어라고 주장하지 않는다.

이후 ‘가치가 있다면 작업해야 한다’는 사용자 리뷰에 따라 네 개를 선정 상한으로 삼지 않고 음성 근거·기억 유지·자료 표현·복구 원본을 추가 상세화했다. 아래는 **현재 8개 상세 후보와 나머지 주제의 판정**이다. 최초 4개만 우선 문서화한 상태를 최종 발굴 결과처럼 해석하지 않는다.

| 주요 기능 / 현재 선택 | 검토한 다른 구현 방식 | 이번 판단 |
| --- | --- | --- |
| 요청의 목표·대상·Task·처리 경로를 통합 semantic 작업으로 제안 | 중간 후보·의존 기록을 가진 단계별 해석 | **우선 검토:** 실제 추론·재처리 그래프가 달라짐 |
| 입력 중 기본 Context 준비, 부족한 정보만 추가 읽기 | 최소 입력의 조회 계획 뒤 source 읽기·통합 해석 | **우선 검토:** 첫 판단의 정보와 source 대기가 달라짐 |
| event inbox·순서 검사·진행 projection, gap에 query | snapshot으로 진행 상태 구성, 일반 event는 hint | **우선 검토:** 같은 source의 event·query 기능 전제와 필수 사건 보존 필요 |
| 좁은 S2S 직접 경로의 speculative 생성·host release | 짧은 경로 분류 뒤 허용된 응답만 생성 | **추가 검토:** 순차 대기와 버린 생성 비용은 실재하나 적용 범위·build 계약 확인 필요 |
| OS/UI 사건·유한 sampling·timeline으로 당시 화면 확보 | 발화 구간을 연속 media로 보존하고 나중에 추출 | **보류:** 무조건 현재 화면만 보는 안은 UC 위반. 강한 media 대안은 같은 timestamp·pointer·문서 identity도 필요하며, codec/빈도 차이를 넘는 기능적 이유를 아직 충분히 제시하지 못함 |
| metadata/keyword 후보 검색·원문·typed state·파생 summary | 별도 의미 색인·embedding 검색 경로 | **보류:** 단어 검색 대 의미 검색의 알고리즘 차이만으로 선정하지 않음. 재표현된 과거 자료 탐색의 실제 필요와 색인 생성·갱신·삭제·추가 dependency 비용을 먼저 구체화해야 함 |
| Context 변경 시 의존 read set만 재검증 | 전체 요청 snapshot 폐기·재구성 | **흡수:** 단독 후보보다 해석·Context 후보의 정정 비용/완결성에서 다룸. 무조건 오래된 근거를 사용하는 대안은 제외 |
| 한 업무 전체를 Agent에, 독립 목표는 별도 추적 | VIA가 한 업무 내부 단계를 분해·계획·연결 | **제외:** 한 업무 전체 위임 합의와 domain planning 경계를 변경함 |
| 한정 자료 설명은 VIA, 업무 조사·실행은 Agent | 정보성 요청을 모두 Agent에 위임 | **이번에는 제외:** 01~05의 넓은 범위에서는 가능한 구성이지만 target 리뷰에서 합의한 기본 직접 처리 방향을 다시 여는 문제. 이번 후보에 조용히 포함하지 않음 |
| 직접 응답은 맥락 없는 자체 지식 질문에 제한 | S2S에 대화·화면·Task 도구를 주어 처리 범위 확대 | **제외:** 사용자 지정 범위를 바꿈. 생성 순서 후보는 범위 유지 |
| 하나의 Omni·동시 session·입력 ASR | 역할별 큰 모델을 따로 적재하거나 whole-call 직렬화 | **제외:** 가중치 공유 또는 동시 입력 진행 요구 위반 |
| 독립 Streaming ASR이 입력 시간 근거 생성 | 같은 공유 Omni의 native 전사·시간 근거 | **상세 후보 추가:** [음성 근거](./speech-evidence-source.md). Native stream·동시 인식·SEALED·Omni 장애 때 기능 손실을 명시했다. 실제 build 적합성은 확인 전이며 이전 turn-final-only 제한은 강제하지 않음 |
| 필요 시 owner 기록에서 대화 Context 구성, 유효 cache·summary 사용 | 활성 대화 working view의 증분 갱신·read barrier | **상세 후보 추가:** [기억 유지](./conversation-context-maintenance.md). cache 유무가 아니라 변경 전달·적용 위치·gap rebuild와 정상 소비 계약이 달라짐 |
| 원문·이미지·typed record 중심 Evidence Package | 선행 materialization으로 구조화 사실 view 재사용 | **상세 후보 추가:** [자료 표현](./context-representation-pipeline.md). target에 없는 공통 의미 확정자를 기준선으로 발명하지 않고 변환·소비 파이프라인을 재구성 |
| Voice/semantic 자원 예약·bounded scheduling | priority·quantum·예약량만 조정 | **제외:** 현재는 정책·설정 차이. runtime 기능이 달라지는 대안을 이름만으로 만들지 않음 |
| 현재 owner 상태와 내구 command/inbox/publication | 확정 이력 + checkpoint·현재 projection | **상세 후보 추가:** [복구 원본](./recovery-state-source.md). 교차 transaction은 유지하며 replay·effect identity·삭제·migration을 구체화. owner별 독립 journal/commit은 별도 축으로 남김 |
| UI·Voice·Core process 격리 | thread 기반 통합 host | **우선 목록에서 제외:** 실제 장애 차이는 있으나 이번 사용자 리뷰가 요구한 핵심 기능의 설계 갈림길보다 약함. 장애 격리의 의미 자체를 부정한 것은 아님 |
| Text 상세·Voice 요약·공통 사실 연결 | Text 완성 후 일괄 음성 요약 | **보류:** 공통 사용자 행동은 가능하지만 생성·전달 단계 차이의 중요성을 먼저 좁혀야 함. 현재는 S2S 생성 후보와 별개 대형 후보로 늘리지 않음 |

기준선에 등장하는 모든 선택을 후보로 만들지 않았다. 메모리의 중요성도 전용 후보 개수를 만들 이유가 아니라, 실제로 추가·유지·폐기되는 데이터·KV·buffer 비용을 검토할 이유로 사용했다.

## 2. 강한 대안을 만들기 위해 수정한 것

| 후보 | 처음 떠올리기 쉬운 약한 대안 | 반론을 반영한 현재 대안 | 남은 약점 |
| --- | --- | --- | --- |
| 해석 단계 | 단계마다 결과 하나를 확정하고 뒤로만 진행 | 원문·후보·미해결을 전달, 의존 revision 기록, 필요한 되돌림 허용 | 되돌림·중간 계약이 커지면 통합보다 복잡하고 느릴 수 있음 |
| Context 획득 | 발화가 끝나야 화면 capture까지 시작 | 당시 관측은 양쪽 공통, 해석용 source 조회의 순서만 변경 | 계획을 만들기 위해 기본 근거 전체가 필요하면 구조 차이가 줄어듦 |
| Agent 상태 | 모든 이벤트를 버리고 가끔 polling | 필수 질문·승인·terminal은 내구 보존, 일반 progress만 hint, query 합치기·cache·연결 복구 | snapshot 기능과 필수 사건 보존이 약한 Agent에는 성립하지 않음 |
| 직접 응답 생성 | 생성 전 host keyword 분류만으로 직접 경로 확정 | 같은 Omni의 구조화 제안 + host 조건 검사, generation 뒤 내용 검사 | 분류-only 호출·이어지는 generation의 실제 build 적합성은 확인 필요 |
| 음성 근거 | native 모델이 timestamp를 이미 지원한다고 가정하거나 final-only로 제한 | native streaming·source-time·revision·gap·동시 진행을 요구 계약으로 명시 | 같은 runtime 장애 때 인식까지 멈춤; 실제 native 기능·전체 자원 미확인 |
| 기억 유지 | 매번 전체 이력 재구성 대 무제한 상주 cache | 양쪽 cache·부분 재구성 허용, 대안에 유한 활성 view·revision barrier·gap rebuild | 변경 lag·view 유지 비용이 요청 준비 절약보다 클 수 있음 |
| 자료 표현 | 현재 target이 공통 canonical 의미를 이미 확정한다고 가정 | 실제 원문 중심 package 대 출처·누락·원문 복귀를 가진 선행 사실 view | 별도 추출의 오류·첫 사용 비용, 새 목적마다 view 확장 필요 |
| 복구 원본 | 로그 없음 대 모든 사건을 무조건 replay | 양쪽 audit·현재 view·동일 내구성 허용; 대안은 checkpoint+tail과 side-effect 없는 재생 | event schema·삭제·checkpoint 유지 비용; 외부 미확인은 그대로 남음 |

현재 설계도 최선의 형태로 읽었다. 통합 해석은 field별 근거를 유지하고, 사전 Context는 모든 자료를 읽지 않으며, 이벤트 projection도 필요하면 query한다. 선생성도 이미 Core임을 아는 요청에서 무조건 끝까지 실행하지 않는다. 이런 최적화를 제거해야만 차이가 생긴다면 그 후보는 설득력이 없다.

해석 단계는 호출 수만으로 구별하지 않는다. 대안의 후속 단계가 앞 의미를 바꿀 때 정정 계약·새 version·후속 무효화를 거쳐야 하며 마지막 전체 재판단을 붙이면 통합안으로 수렴한다. Context 계획은 조회 시점의 선택이지 최초 source 집합을 절대로 확장하지 못하게 하는 선택이 아니다. Agent 상태 후보는 일반 progress 공급 방식을 바꾸고 Task writer·event sourcing·Agent 계약까지 동시에 바꾸지 않는다.

## 3. 후보 간 중복과 고정할 조건

| 후보 쌍 | 겹치는 부분 | 분리해서 비교할 축 |
| --- | --- | --- |
| 해석 단계 / Context 획득 | 첫 모델 작업·추가 읽기·호출 수 | 해석 비교는 초기 근거·조회 수단을 같게 둔다. Context 비교는 최종 통합 해석을 유지하고 읽기 trigger·앞뒤 관계만 바꾼다. |
| Context 획득 / Agent 상태 | Task 정보가 Context의 source | Context 비교는 같은 Task 조회 port·상태 공급 방식을 사용한다. 상태 비교는 같은 사용자 최신성 요구 아래 source event/query 적용을 바꾼다. |
| 직접 응답 생성 / 해석 단계 | 같은 Omni의 호출·자원 | 직접 응답 후보는 좁은 S2S 경로에 한정하고 Core 해석을 바꾸지 않는다. Core 후보에서는 direct 처리 구조를 유지한다. |
| 음성 근거 / 해석·직접 응답 | 같은 audio와 Omni 사용 | 음성 근거 비교는 실제 recognizer·input source를 바꾸되 해석·생성 순서는 고정한다. 다른 후보는 동일한 입력 근거를 쓴다. |
| Context 획득 / 기억 유지 | 첫 요청의 Context 준비 | 획득은 읽기 시작 trigger, 기억은 같은 trigger에서 owner 조회·조합 대 이미 유지된 view. 증분안의 백그라운드 갱신 비용을 숨기지 않는다. |
| Context 획득 / 자료 표현 | source 읽기·모델 입력 | 같은 source·revision·구간·읽기 시점으로 맞춘 뒤 변환 산출물만 바꾼다. |
| 기억 유지 / 자료 표현 | 파생 view·cache·삭제 | 기억은 여러 owner의 대화 working set, 표현은 선택된 자료의 사실·구조. 표현 방법은 고정한 채 갱신 방식을 비교하고 반대 비교에서는 유지 방식을 고정한다. |
| 해석 단계 / 자료 표현 | 중간 모델 작업 | 자료 표현은 source의 구조·사실·provenance를 추출하고 Task·handling은 결정하지 않는다. 요청 의미 파이프라인은 동일하게 둔다. |
| Agent 상태 / 기억 유지 | Task 정보 재사용 | 같은 확인된 Task 상태를 기억 view에 반영한다. 외부 진행 상태 공급 방식과 대화용 파생 view 갱신은 별도다. |
| Agent 상태 / 복구 원본 | event·projection·query | 외부 상태를 얻는 방식과 VIA 확정 상태의 복구 원본을 구분한다. Event 관측이어도 current-state 저장이 가능하다. |
| 기억 유지 / 복구 원본 | checkpoint·rebuild | 파생 Context view는 양쪽에서 잃어도 재구성 가능하다. 복구 후보는 command·질문·전달 같은 운영 원본을 바꾼다. |
| 모든 후보 / 모델 scheduling | 공유 queue·KV·입력 보호 | 공통 입력·자원 계약을 유지한다. 새 작업이 늘어난 비용은 인정하되 scheduling 알고리즘을 함께 바꿔 이점으로 세지 않는다. |

두 안의 모델 호출 수를 무조건 같게 만들면 구조 차이를 지울 수 있다. 무관한 모델/build·권한·필수 기능·Agent 능력 차이로 이점을 만들지 않는다. 단, 음성 근거처럼 dependency 구성 자체가 선택이면 ASR·native 기능의 차이와 모든 자원 비용을 공개하고 나머지 Omni 능력·입력·장비 조건을 가능한 한 맞춘다. 전체 자원·deadline 같은 공통 조건은 후속 검증 계약에서 정하며 현재 수치·동결 조건을 만들지 않는다. 정확한 같은 실행 결과를 양쪽에 강제하지 않고 같은 입력·목표·외부 조건에서의 오류와 실패를 모두 남기는 방향이다.

## 4. 문서에서 수행한 사건 추적

| 확인한 반례 | 검토 결과 / 문서에 남긴 조건 |
| --- | --- |
| 지칭 결과가 보고서 Context에 의존했는데 Task만 변경 | 단계화해도 지칭 결과 재사용 불가; 의존 기록과 앞 단계 무효화 필요 |
| 조회 계획 없이 “그거”의 source조차 특정 불가 | 최소 식별 근거와 제한된 후보 조회 허용; 모호함을 자신 있게 추측하는 계획은 불가 |
| 발화 중 화면 이동 뒤 final transcript 도착 | 두 Context 안 모두 당시 capture 유지; 조회 후행을 capture 후행으로 혼동하지 않음 |
| snapshot 응답이 늦는 동안 Agent 질문 도착 | 질문 내구 원본 보존, snapshot revision으로 늦은 상태가 질문을 지우지 않도록 처리 |
| “최신 상태를 다시 확인” 요청 | 이벤트 안도 query; 조회 비용을 대안에만 부과하지 않음 |
| event/query를 모두 못 하는 Agent | 양쪽 모두 지원 한계·불명 처리; 대안의 일반 적용 범위에서 제외 |
| direct 경로 허용 뒤 새 발화 시작 | 두 안 모두 새 revision으로 게시 차단; 생성 시작 전/후의 낭비량만 다름 |
| current direct gate는 audio handle을 요구 | 대안은 경로 허용과 실제 게시 허용의 두 상태로 변경 필요. 기존 schema 그대로 가능하다고 적지 않음 |
| Native 전사가 semantic job 종료 뒤에만 나옴 | 단순 audio backlog는 동시 인식 요구 미충족; 해당 build/profile은 대안의 필수 조건을 충족하지 못함 |
| Shared Inference Service crash 중 새 발화 | native안도 capture·local stop은 남지만 인식은 중단될 수 있음. 독립 ASR의 차이를 인정하며 입력 gap과 미처리를 표시 |
| 기억 삭제 뒤 늦은 projector·summary 결과 | 현재 epoch로 사용 차단; source별 적용 위치가 충분해도 삭제 fence를 우회할 수 없음 |
| Task 상태 변경 알림 하나 유실 | 증분 view의 gap 탐지·DIRTY·owner snapshot rebuild; 다른 owner의 내부 DB 직접 조회로 메우지 않음 |
| 표 각주가 사실 view에서 누락 | 필수 원문·누락·provenance를 보존하고 보완/원문 복귀; 구조화됐다는 이유로 정확하다고 확정하지 않음 |
| View 추출 비용이 있는데 자료를 한 번만 사용 | 선행 호출·저장 비용을 인정; 반복 사용 사례만 골라 대안을 유리하게 하지 않음 |
| DISPATCHING 이력 replay | 과거 외부 호출을 재실행하지 않음. 같은 effect identity·UNKNOWN과 현재 query로 조정 |
| 삭제 전 checkpoint와 삭제된 event payload | 현재 삭제 ledger를 먼저 적용; 해당 checkpoint 정리·deleted 상태 복원, 과거 개인정보 완전 재현을 약속하지 않음 |
| 권위 이력 자체 손상 | event sourcing도 확인 불가·backup/repair 필요. 모든 DB 손상을 replay로 복구한다고 하지 않음 |

이는 문서상 정상·예외 경로를 따라간 검토이며 실행 시험이나 모델의 성공률 확인이 아니다.

## 5. 다음 공동 검토의 판단 기준

먼저 각 후보가 과제의 핵심 기능을 설명하는 데 중요한지, 대안도 실제로 선택할 만한지, 그림에서 작동 차이가 드러나는지 확인한다. 특히 해석 단계가 prompt 분할에 그치지 않는지, Context 조회 계획이 사전 준비와 실질적으로 구분되는지, 상태 snapshot 기능의 전제가 현실적인지 비판적으로 검토해야 한다.

추가 네 후보도 같은 기준을 적용한다. 음성 dependency는 실제 capability와 장애 손실, 기억은 지속 갱신 계약과 cache 수렴, 자료 표현은 반복 소비와 정보 손실, 복구는 사용자 interaction을 다시 연결할 필요와 유지 비용을 본다. 8개를 모두 발표 DP로 선정하지 않으며, 기존 네 개를 자동 우선 채택하지도 않는다. 상세 문서 작성은 후보 검토를 가능하게 한 상태이지 효과 검증이나 설계 채택이 아니다.

후보 채택 후 메모리를 포함한 ASR의 관련성·정의·우선순위·평가 범위를 논의한다. 현재는 ASR·추가 QA의 인과 가설과 적용 제안을 표로 기록했다. ASR을 재정의하거나 모두 PRIMARY로 분류하지 않았다. 별도의 허가 없이 구현·측정으로 넘어가지 않는다.

## 6. 2026-10-01 발표 그림·계약·품질 비교 보강

사용자가 기존 원고 수준을 거절하고 실제 SW 구조도와 구체 대안을 요청했다. 이전의 “선정 뒤 그림 제작” 방침을 폐기하고 8개 후보에 배경·비교 16페이지를 작성했다. 이전 7개 자료는 도식의 상세 수준을 비교하는 역사 참고로만 열었고 원본을 바꾸지 않았다.

| 검토 항목 | 반영한 내용 |
| --- | --- |
| 박스 분할을 넘어선 작동 차이 | 해석 stage version·ReadPlan·snapshot query·held generation·native evidence·working view·FactView·복구 권위 원본을 각각 표시 |
| 승인·소유 경계 | Agent 상태는 Request Controller를 거쳐 게시; Context 반환·semantic 제안도 host 확정과 구별 |
| 동시성·예외 | 선생성의 중첩, 독립 query, correction·stale 결과·late snapshot·gap·삭제·UNKNOWN·replay의 외부 효과 금지를 구체화 |
| 상태 수명 | 미확정 해석 stage는 attempt RAM, query·view·buffer는 유한 수명, 삭제 fence·runtime incarnation을 명시 |
| 품질 비교 | 현재 ASR 네 축과 관련 추가 QA를 양안 장단점·유불리 조건 표로 작성; 적용은 제안이며 수치·승자·freeze 없음 |
| 표현 규칙 | 공통 검정·차이 양안 파랑, 동일 16:9 canvas, 내부 모듈·Component·명시적 runtime 범위를 구분 |
| 편집·재생성 | 16개 `.drawio`와 16개 `.svg`를 공통 scene에서 생성; draw.io에 nested parent·source/target 연결·명시 waypoint 보존 |
| 자체 시각 검토 | 로컬 Chrome으로 16개 SVG 렌더링, 실제 glyph overflow 확인, 구조도 연결과 원고 계약 대조; 선이 무관한 node 내부를 관통하면 생성 검사 실패 |

이 검토는 작성자의 문서·도식 검토다. 독립 Senior Architect 심사, 실제 모델 native capability 확인, 제품 실행·메모리·성능·복구 측정은 아니다. 특히 QA-39와 QA-32의 필요 의존 범위, QA-19와 QA-13~15의 관계, QA-41과 디스크 저장량, QA-62와 운영 replay를 구별했다.

## 2026-10-01 독립 재검토

세 별도 검토 에이전트가 설계 계약·품질 비교·구조도를 독립 검토하고 수정본을 다시 대조했다. [발견사항과 보완 기록](./independent-review-2026-10-01.md)에 구체 반례, 계약·그림 수정, 확인 범위와 남은 조건을 정리했다. 이는 후보의 채택이나 실측 우위 확인이 아니다.
