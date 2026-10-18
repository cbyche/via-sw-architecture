# 문제 4. Agent가 바뀌어도 이어지는 업무 기억과 인계

2026-10-16 / 다섯 문제 심화 조사 / 복수 대안 탐색

[공동 기준](./00-research-contract.md) / [구현 근거 R01~R12](../../../../../references/five-problem-study/evidence-and-handoff.md) / [문제 1](./01-evidence-supply.md) / [Agent 선택과 전환](./05-agent-selection-and-transition.md)

## 1. VIA의 제품 가치를 성립시키는 문제

**VIA가 사용자의 메인 interface라면 Agent가 바뀔 때 사용자가 목표, 수정 조건과 채택한 결과를 다시 설명하지 않아야 한다.** 이를 위해 VIA가 받은 공개 요청과 결과를 보존하고, 다음 Agent가 실제로 사용할 수 있는 업무 문맥으로 연결해야 한다. 단순히 transcript를 오래 보관하는 것만으로는 충분하지 않다.

대표 정상 사용은 ‘A의 제품 비교 결과에서 사용자가 조건을 수정한 뒤 B에게 발표자료를 요청하고, 이후 C에게 수정하는 경우’다. B에게 최초 비교 질문만 보내면 수정된 조건이 빠지고, A의 마지막 답변만 보내면 사용자가 채택한 부분과 제외한 부분이 섞일 수 있다. 결과 URL이 만료되면 참조를 보냈다는 사실만으로 자료를 인계한 것이 되지 않는다. 관련 UC는 05, 07~10, 12~18이다.

이번에 남긴 구조는 다음과 같다.

| 안 | 업무 문맥을 만드는 방식 | 외부 Agent가 받는 것 | 정상 사용의 선택 이유 |
| --- | --- | --- | --- |
| **H-A 위임 시 인계 묶음 구성** | 다음 위임이 확정됐을 때 원본 기록과 현재 조건에서 필요한 문맥을 구성 | 그 위임에 맞는 self-contained capsule과 원본 / 결과물 참조 | 인계가 드물고 업무마다 필요한 자료 범위가 크게 다름 |
| **H-B 공통 업무 기록을 계속 갱신** | 업무 사건과 사용자 수정을 반영한 portable work record를 유지 | 현재 record의 필요한 projection과 원본 / 결과물 참조 | 같은 결과가 여러 후속 업무로 이어지고 조건 재사용이 빈번 |
| **H-C 수신 Agent가 필요한 자료를 읽는 인계** | VIA는 필수 조건 manifest와 source별 접근 가능한 workspace를 준비 | 목표 / 제약 / 결과물의 최소 manifest, bounded read interface와 versioned 자료 참조 | 자료가 크고 다음 작업자만 실제 필요한 세부를 결정할 수 있음 |

**권고:** H-A/H-B는 심화할 주요 후보이고 H-C도 의미 있는 조건부 대안으로 유지한다. H-C는 외부 Agent가 읽기 계약을 지원해야 하므로 전체 Agent 공통 기능으로 약속할 수 없다. 같은 Agent의 native session 재사용은 세 안 모두에 조합할 수 있는 공통 최적화다. 이를 ‘portable 기억의 반대안’으로 세우지 않는다.

## 2. 무엇을 수집하고 무엇을 기억하는가

‘모든 history를 기억한다’는 제품 약속은 **VIA를 통해 관측한 공개 기록을 사용자가 허용한 범위에서 보존하고 다시 사용할 수 있다**로 구체화한다. 외부 Agent의 비공개 추론이나 memory를 읽을 수 있다고 전제하지 않는다.

| 기록 종류 | 수집 출처와 권위 | 다음 Agent에서의 용도 |
| --- | --- | --- |
| 사용자 목표, 수정과 제외 조건 | 실제 입력 및 Request Controller가 채택한 revision | 현재 유효한 수행 조건. 단순한 최신 문장만으로 전체 조건을 대체하지 않음 |
| 위임 요청과 연결 | Task Manager의 VIA Task, Agent Gateway가 전달한 공개 입력과 native 식별자 | 무엇을 누구에게 보냈는지, 후속 결과가 어느 요청에 대한 것인지 |
| 접수 / 진행 / 질문 / 결과 | 실제 외부 관측. ‘전송함’, ‘접수됨’, ‘완료됨’을 구별 | 확인된 업무 범위, 미답 질문과 필요한 후속 처리를 이해 |
| 결과물과 근거 자료 | 관측된 artifact locator, 접근권한, native version이 있으면 해당 version. 없으면 보존한 복사본의 로컬 식별자 | 정확한 자료를 열고 필요한 부분 사용. 존재하지 않는 native revision을 생성하지 않음 |
| 사용자의 채택과 정정 | VIA에 실제로 주어진 선택 / 수정 | Agent의 최초 주장과 사용자가 사용할 결과를 구별 |
| 실제 설명 전달 | Response Manager의 text 표시와 audio delivery extent | ‘들려준 부분으로 자료를 만들어’ 같은 요청의 범위를 결정 |
| 원본에서 만든 요약 / 인계 문서 | Context Manager의 파생물, source dependency와 생성 revision | 다음 업무의 입력을 줄임. 원본 및 현재 조건의 권위를 대체하지 않음 |
| 허용 User Memory | 명시된 보존 목적과 사용자 변경 / 삭제 권리 | Agent가 달라져도 적용할 안정된 선호. 업무의 일시 조건과 분리 |

모든 원본을 하나의 거대한 모델 문맥에 넣는 것은 아니다. State Store와 기존 소유자의 저장 계약을 사용하고, source별 index 및 범위 참조로 접근할 수 있다. 원본 payload와 secret은 권한 / 보존 정책에 따른다. 수집하지 못한 내용은 ‘미지원’, ‘일부만 관측’, ‘접근 불가’ 등으로 기록한다.

### 공통 인계 계약

1. 현재 위임의 목표, 결과물 형식, 유효한 필수 제약과 명시적 제외 조건을 전달한다.
2. 사용할 결과물과 source revision / 관측 복사본, 접근 가능한 extent를 식별한다.
3. 완료한 일, 취소 요청, 확인된 종료, 확인되지 않은 실행을 구별한다.
4. 질문 / 승인 중 현재 작업에 필요한 항목만 전달하며, 이전 승인 범위를 새 Action으로 확대하지 않는다.
5. 수신 Agent의 실제 지원 기능과 권한을 확인한다. text만 받는 Agent에게 shared workspace 읽기를 성공한 것처럼 보고하지 않는다.
6. 전송 당시 위임 revision과 자료 권한을 대조한다. 전송 중 변경이 발생하면 기존 묶음을 현재 것으로 채택하지 않는다.
7. 원본과 이력의 보존 성공, 파생 인계 생산 성공, 외부 Agent 접수 성공을 구별한다. 정형 접수 확인이 없는 Agent에는 전송만 관측했다고 표시한다.

이 공통 계약이 모든 안의 완료 조건이다. 해석, 인계와 공개 업무 사실의 정리는 VIA 책임이고 실제 발표자료의 계획 및 작성은 Downstream Agent 책임이다.

## 3. reference 조사에서 얻은 구조

| source 근거 | 직접 확인한 메커니즘 | VIA에 옮기는 부분과 추가 설계 |
| --- | --- | --- |
| Deep Agents isolated / fork, R07 | 위임 설명만 전달하거나 유효 부모 문맥과 상태를 상속. 일부 state 제외 및 structured result 반환 | H-A의 선택된 공개 문맥 전달. 이종 Agent로 private state fork를 확대하지 않음 |
| CrewAI `_get_context`, R09 | 명시된 이전 Task 출력 또는 이전 출력 묶음을 다음 Task에 전달 | 결과 의존에 따른 인계. 사용자 수정 / source version / 승인 범위는 VIA 계약 추가 |
| OpenClaw consolidation / provenance와 CrewAI memory, R03~04, R09 | 사건에서 기억을 생산하고 원본 계보, 갱신 / 삭제와 조회 대기를 관리 | H-B의 portable 파생 업무 기록. 기억 문장만으로 현재 조건과 실행 사실을 대체하지 않음 |
| Deep Agents StoreBackend / CompositeBackend / 범위 읽기, R02, R08 | 여러 thread가 namespace 안의 자료를 읽고 임시 및 지속 backend를 분리 | H-C의 참조 자료 읽기. 외부 Agent에 허용된 read interface를 제공하는 부분은 VIA integration 설계 |
| OpenHands 이름별 child conversation, R06 | 같은 child에 후속 입력 전달, persistence와 결과 모으기 | 동일 Agent 후속 수정의 native session 재사용 |
| OpenClaw ACP와 transcript lease, R10~11 | 외부 harness 입력 변환, resume ID 전달, 제한된 transcript reader | native 식별자와 VIA Task 분리, 수집 한계와 capability 수명 표현 |
| Magentic-One ledger, R12 | 조정용 명시 상태와 공유 message thread | 업무 facts의 명시성 참고. 업무 내부 계획을 VIA로 이동하지 않음 |

공식 구현에는 실용적인 조각이 있다. 아래 전체 VIA 구조가 어느 제품에 그대로 완성돼 있다는 주장은 하지 않는다. 표에 표시한 추가 계약이 설계 작업의 내용이다.

## 4. 복수 구조의 실제 실행과 상태

### H-A. 위임할 때 원본에서 인계 묶음 구성

**기본 상태:** VIA는 원본 사건, 현재 채택 조건과 artifact index를 유지한다. 미래의 모든 인계를 위한 의미 요약은 매 사건마다 생산하지 않는다.

1. Request Interpreter가 후속 요청의 대상과 관계를 채택 경로에 제안한다. 예를 들어 비교 결과 v2에서 발표자료를 만드는 요청임을 확인한다.
2. Context Manager는 현재 목표에 필요한 원본, 수정과 실제 결과를 bounded read로 취득한다. 기존 유효 capsule과 원본 cache도 재사용한다.
3. 필요한 조건과 출처를 인계 capsule로 구성하고 Task Manager의 현재 위임 revision 및 Policy Manager의 허용 범위를 대조한다.
4. Agent Gateway가 선택된 Agent의 지원 형식으로 전송한다. caption만으로 열 수 없는 자료는 허용된 attachment / 복사본이나 복구 경로를 사용한다.
5. 전송한 capsule의 hash / 로컬 revision, source 목록과 native receipt를 기록한다. 다음 위임에서 그대로 재사용할 수 있는 범위는 cache로 남긴다.

**정상 이익:** 후속 위임이 실제로 생길 때 필요한 자료만 가공한다. 발표자료와 메일 작성처럼 사용하는 범위가 다르면 각 목적에 맞게 작게 구성할 수 있다.

**비용:** 위임 직전에 과거 수정과 결과를 다시 결합하는 대기가 생긴다. 장기간 누적된 여러 기록에서 필수 조건을 찾는 호출과 입력이 늘 수 있다. 잘못 압축한 capsule이 여러 번 재사용되지 않도록 revision별 의존을 관리한다.

**정정 / 실패:** 묶음 생성 중 수정이 들어오면 영향받는 부분만 다시 구성한다. source 접근 실패를 capsule에서 숨기지 않는다. 필요한 자료가 부족하면 허용된 대체 원본을 찾거나 사용자에게 현재 제한을 알리고 위임을 보류한다. memory 생산이 실패했다는 이유로 완료된 원래 업무를 실패로 되돌리지 않는다.

### H-B. Agent에 독립적인 업무 기록을 계속 갱신

**기본 상태:** 공통 원본과 현재 조건 외에, 여러 후속 작업이 읽을 portable work record가 있다. 목표 / 제약, 사용자 채택, 결과물 / 근거 관계와 남은 불확실성을 출처별로 관리한다. native Agent session은 이 기록의 권위가 아니다.

1. 새 결과와 사용자 수정이 들어오면 Context Manager의 생산 역할이 해당 업무 기록의 변경 후보를 만든다. 현재 조건처럼 이미 구조화된 사실은 코드로 반영하고, 의미 정리가 필요한 부분만 모델을 사용한다.
2. source revision 및 현재 위임 / 조건 revision을 대조해 기록의 새 revision을 게시한다. 최신 사건까지 반영한 범위와 아직 반영하지 못한 범위를 함께 둔다.
3. 후속 요청에서는 게시 기록 중 필요한 부분을 선택하고 원본을 보완해 인계 projection을 만든다.
4. Agent Gateway 전송과 receipt 기록은 H-A와 같다. 최신 record가 준비되지 않았으면 기다리는 조건을 명시하거나 원본에서 H-A 방식으로 해당 부분을 구성한다.

**정상 이익:** 같은 결과를 여러 Agent가 후속 사용하고, 서로 다른 시점의 수정이 쌓이는 업무에서 반복 조합을 줄인다. 조건, 채택된 결과와 artifact 관계를 한 번 해석해 여러 인계에 사용할 수 있다.

**비용:** 이후 인계에 쓰이지 않는 업무에도 생산이 일어날 수 있다. schema, source dependency, 갱신 큐와 invalidation을 유지하며 정정이 많으면 재생산이 누적된다. 게시 기록만으로 부족한 새 목적에서는 결국 원본 읽기가 필요하다.

**정정 / 실패:** producer가 오래된 revision에서 만든 결과를 늦게 반환하면 게시를 거절한다. 부분 수정은 dependency가 있는 record 구절만 교체한다. source 철회로 여러 출처의 요약이 영향을 받으면 의미적으로 분리 가능한 범위만 유지하고 나머지는 사용 보류 또는 재생산한다. 게시 실패는 기존 유효 조각과 원본 fallback으로 복구한다.

### H-C. 필수 manifest를 전달하고 수신 Agent가 자료를 읽음

**기본 상태:** VIA는 원본, 현재 조건, artifact와 source별 접근 정책을 가진 read workspace를 제공한다. 수신 Agent에게 전체 문맥을 미리 복사하지 않는다. 다만 **목표, 필수 제약, 금지 / 제외 조건과 사용할 결과의 revision은 manifest에 반드시 포함**한다. 안전한 수행에 필요한 조건을 선택적 검색 뒤에 숨기지 않는다.

1. VIA가 위임 대상을 채택하고 필수 manifest를 만든다. 어떤 source와 범위를 읽을 수 있는지 read lease와 locator를 준비한다.
2. Agent Gateway는 해당 Agent가 이 read interface와 자료 형식을 지원하는지 확인한 뒤 manifest와 접근 방법을 전송한다.
3. 수신 Agent는 업무 계획에 따라 필요한 범위를 읽는다. VIA 측 read 경계는 실제 반환 revision, extent, 권한과 누락 / 만료를 검사하고 기록한다.
4. 필요한 추가 자료가 없으면 Agent는 기존 질문 채널로 보완을 요청한다. VIA가 그 질문과 원래 요청을 연결한다.
5. 결과는 원본과 사용 자료의 공개 참조를 함께 받을 수 있는 만큼 기록한다. source 사용 목록을 외부 Agent가 지원하지 않으면 자동으로 안다고 가정하지 않는다.

**정상 이익:** 큰 결과물 중 어떤 세부가 필요한지 수신 작업자만 알 수 있는 업무에서 VIA가 모든 세부를 미리 요약 / 복사하는 일을 줄인다. 서로 다른 작업자가 같은 원본을 범위별로 읽을 수 있다. read interface가 이미 있는 Agent와 반복 자료 사용에 적합하다.

**비용:** 업무 도중 원격 읽기와 보완 왕복, lease 수명 및 사용 기록이 추가된다. source를 사용할 수 있는 기간이 실행과 연결되고, 텍스트 전용 Agent에는 그대로 적용할 수 없다. 읽기와 선택 비용 일부가 Downstream Agent로 이동하므로 VIA 모델 비용 감소를 전체 업무 비용 감소로 주장하지 않는다.

**정정 / 실패:** 새로운 위임 revision에서 lease를 갱신하거나 폐기한다. 이미 전달한 내용은 read 권한 철회만으로 지워지지 않는다. 실행 중 정정이 수신되었는지 확인 가능한 계약을 사용하고 미지원이면 새 인계 또는 보류로 처리한다. read interface 미지원은 H-A/H-B capsule fallback을 명시적으로 선택한다. 이 fallback을 H-C 순수 구조의 이익으로 세지 않는다.

### 상태 소유와 수명 비교

| 상태 | H-A | H-B | H-C |
| --- | --- | --- | --- |
| 현재 목표 / 조건, 확인된 실행 사실 | 공통 원본 및 채택 상태 | 같음. 파생 work record가 덮어쓰지 않음 | 같음. manifest의 권위는 공통 채택 상태 |
| 인계를 위한 의미 가공 | 위임 시 요청별 capsule | 사건 기반 work record + 위임별 projection | 필수 manifest + 수신 작업자의 필요별 원본 읽기 |
| 지속 파생 상태 | 재사용 가치가 있는 capsule cache와 dependency | 업무 수명의 portable work record와 dependency | source catalog, access lease와 실제 read 관측. 선택적 manifest cache |
| 업무와 Agent 식별자 | VIA Task와 native session / run mapping | 같음 | 같음 |
| 정정이 전파되는 곳 | 실행 중 capsule와 cache | work record, 미완료 생산, 전송 projection과 cache | manifest, source admission과 lease, 이미 진행한 읽기 / 실행의 정정 연결 |
| 종료 후 자원 | 위임 임시 문맥 종료, 필요한 이력 보존 | 기록 보존 정책에 따른 게시 상태, bounded 생산 큐 | lease 종료, 필요한 artifact와 read 이력 보존 |

## 5. 여러 장면을 같은 조건으로 따라가기

EX03-H와 EX04-H는 공통 EX03 / EX04 뒤에 외부 업무 위임을 명시적으로 추가한 사고 실험 변형이다. 원래의 직접 재조회 / 설명만 하는 입력에는 H-C의 수신 Agent를 도입하지 않으며 N-H도 적용하지 않는다. 기존 COMMON-28에 새 위임 입력을 추가한 측정 모집단이 아니다.

| 장면과 공통 완료 조건 | H-A | H-B | H-C | 주요 차이 / 제한 |
| --- | --- | --- | --- | --- |
| **EX11 Agent 간 후속 위임:** A 비교 v2, 유지보수 우선, 제품 Z 제외. B가 그 조건으로 발표자료 작성 | 수정과 결과 v2를 읽어 발표자료용 capsule 생성 | 현재 work record에서 조건 / 결과 관계를 projection. 미반영 수정은 원본 보완 | 조건과 v2는 manifest에 포함. B가 필요한 표 / 근거를 범위 읽기 | H-B의 반복 조합 절약, H-A의 목적별 최소 가공, H-C의 세부 읽기 지연과 capability 의존 |
| **EX12 같은 Agent 후속 수정:** 제목만 변경, native session 사용 가능 | 같은 session에 명시 수정과 revision 전달. 새 전체 capsule 강제 안 함 | 같은 session에 명시 수정, work record 해당 부분 갱신 | 같은 session 재사용. 추가 읽기 불필요하면 생략 | 세 안 모두 빠를 수 있음. native 재사용을 H-A만의 이익으로 세지 않음 |
| **EX04-H 과거 결과 재사용 / 반복 후 위임:** 비용 비교 재조회와 반복 후 ‘그 비용과 일정 자료로 회의자료를 만들어줘’ | 첫 원본 조회 후 capsule / source cache hit. 새 일정 관계만 조합 | 두 결과의 게시 기록을 사용하고 version 확인 | 필수 관계 조건을 전달하고 수신 Agent가 원본 읽기. 반복 read cache 허용 | 큰 새 결합에서 차이, 단순 반복에서 동점 가능 |
| **EX03-H 실제 전달 범위의 위임:** ‘들려준 부분만 발표자료로’ | audio A의 실제 extent를 읽어 capsule에 제한 | record가 modality / extent를 보존했으면 선택. 없으면 원장 보완 | manifest에 A extent만 허용하고 해당 범위 제공 | 전체 text A/B/C를 자동 전달하면 세 안 모두 기능 실패 |
| **EX06 / EX09 부분 정정:** T1/T2를 T1/T3로, P2 수정은 유지 | 진행 중 capsule의 표 조건만 갱신 | 관련 record와 전송 projection을 갱신. P2 dependency는 유지 | manifest 갱신, 해당 read / 실행에 새 revision 전달 | 초기 요약을 완성한 뒤 정정이 왔다는 이유로 무효 조건을 계속 전달하지 않음 |
| **EX14 실행 중 전환:** A가 파일을 만들었음은 확인, 전송은 불명 | 확인된 파일과 불명 상태를 새 capsule에 포함. 전송 재실행 보류 | record의 확인 / 불명 상태를 그대로 인계 | manifest에 전송 금지 / 확인 필요를 명시. 파일 읽기만 허용 | native 내부 실행 재개나 exactly-once migration을 주장하지 않음. 확인 불가 Action을 자동 중복 실행하지 않음 |
| **EX16 철회 / 만료:** 비교 source 일부 사용 철회 또는 URL 만료 | source 사용 필터, cache 제거, capsule 재구성. 허용 복사본 없으면 보류 | source 계보를 따라 record와 projection 차단 / 재생산 | lease 폐기 또는 locator 복구. 이미 내려간 내용의 삭제는 별도 capability | H-C가 이후 읽기를 차단하기는 쉬울 수 있으나 과거 복사본 삭제까지 해결하지 않음 |
| **EX17 재접속:** native session은 사라졌고 result v3가 도착 | 원본 / 현재 조건으로 새 capsule, 새 session 연결 | record의 마지막 반영 지점 확인 후 v3 갱신, 새 session 연결 | workspace 원본과 manifest 재발급. 오래된 lease 재사용 금지 | VIA 이력이 남아도 hidden Agent 상태까지 복구되는 것은 아님 |
| **EX13 적합하지만 text만 받는 Agent 추가** | 허용 크기와 attachment 지원에 맞춰 capsule 전달 | work record projection을 text 계약으로 전달 | 직접 read 미지원이면 명시 H-A/B fallback 또는 후보 제외 | 지원 범위 차이를 Agent 선택 정확도나 성능 차이로 숨기지 않음 |

### EX11의 공통 입력과 실제 인계

원본에는 최초 ‘가격 우선’ 요청, A의 비교 v1, 사용자의 ‘유지보수 우선, Z 제외’ 수정, A의 비교 v2와 사용자 채택이 있다. 새 요청은 ‘그 결과로 발표자료를 만들어줘’다. 후보에게 ‘v2가 정답’이라는 oracle을 제공하지 않는다. VIA가 기록에서 해당 연관성을 판단한다.

공통 인계의 필수 의미는 ‘채택한 비교 결과를 근거로 발표자료 작성’, ‘유지보수 우선’, ‘Z 제외’, ‘관측된 v2 또는 그 정확한 보존 복사본 사용’이다. H-A는 지금 조합하고, H-B는 게시 record를 확인해 뽑고, H-C는 이 조건을 manifest에 넣은 뒤 세부 표와 자료의 읽기를 수신 Agent에 맡긴다. H-C에서 Z 제외 조건을 원본 검색에서 찾아야만 알 수 있게 만들면 강한 대안이 아니다.

## 6. 동일 Agent 재사용과 이종 Agent 인계를 구별

OpenHands와 OpenClaw에는 같은 native session을 이어가는 실제 경로가 있다. Deep Agents에는 부모 문맥 fork가 있다. 세 기능은 서로 다른 권한과 상태를 사용한다.

| 기능 | 얻는 것 | 얻지 못하는 것 |
| --- | --- | --- |
| 같은 native session에 follow-up | 해당 Agent가 보유한 공개 / 내부 문맥을 계속 활용할 가능성 | 다른 제품이 그 session을 읽거나 실행을 이어받을 보장 |
| 같은 framework의 context fork | 유효 message와 허용 state의 초기 복제 | 임의 외부 서비스의 private memory나 실행 중 tool 상태 이전 |
| VIA portable 인계 | VIA가 관측한 목표, 수정, 확인된 결과와 자료 접근을 새 Agent에 제공 | 공개되지 않은 내부 plan과 수행 사실의 복구 |

따라서 세 안 모두 동일 Agent가 적합하면 native continuation을 사용할 수 있다. VIA는 별도로 공개 입력과 현재 조건을 보존하고, correction을 받았는지 확인하는 지원 범위를 적용한다. 단순 ID 재사용으로 옛 조건의 무효화까지 보장하지 않는다. Agent를 바꿀지 판단하는 것은 [문제 5](./05-agent-selection-and-transition.md)의 선택 / 전환 구조이며, 이 문서는 바꿀 때 무엇을 전달할지를 맡는다.

## 7. QA trade-off와 추가 QA 후보

| 축 | H-A / H-B / H-C에서 바뀌는 물리적 경로 | 조건과 제한 |
| --- | --- | --- |
| **QA01 첫 응답** | 대상 결과와 유효 조건을 채택하기 위해 원본을 다시 읽어야 하면 H-A 지연, H-B record hit 이익 가능. H-C manifest에 필요한 최소 조건 판단도 동일하게 필요 | ‘채택한 비교 결과로 발표자료를 만들겠습니다’가 먼저 표시 가능하면 이후 capsule / lease 준비와 Agent 접수는 QA01 밖이다. 보편적 H-B 우위 주장 금지 |
| **QA02 지칭 정확도** | ‘그 결과’, ‘들려준 부분’의 대상 채택에 원본 / record / catalog가 영향을 줌 | 채택 후 인계에서 조건을 빠뜨린 것은 QA02와 다르다. 대부분 explicit result ID 장면이면 동점 가능 |
| **QA03 VIA 처리시간** | H-A 위임 시 원본 조합과 검증, H-B projection 및 필요 시 생산 대기, H-C manifest / read 준비 중 기존 trial의 인과 경로 안에 있는 처리만 반영 | 상세 표시와 전체 음성 요약 준비라는 기존 endpoint를 늘리지 않음. 그 이후 downstream pull에 응답한 VIA serving 비용 / 시간은 별도 진단. 외부로 비용이 이동했다고 전체 경험이 빠르다고 결론 내리지 않음 |
| **QA04 변경 범위** | source extent 철회 / 설명 출처 변경은 H-A capsule dependency, H-B record schema / 갱신, H-C manifest / lease 계약에 영향. 새 read protocol은 H-C에 추가 변경 가능 | ADR-001의 공통 integration edge를 유지. wire-only 변경은 모든 안에서 adapter에 한정될 수 있다. Rust 구현 없이 모듈 수와 우위 미정 |
| **QA05 모델 비용** | H-A 각 위임의 의미 조합, H-B 미사용 record 생산과 갱신 후 작은 projection, H-C 필수 manifest 생산과 VIA의 필요한 보완 호출 | 동일 Agent 단순 수정과 인계가 드문 업무는 H-A에 유리할 수 있음. H-C의 downstream 읽기 / 추론 비용 이동은 별도 기록하고 VIA 비용 절약으로 전체 절약 주장 금지 |
| **QA06 최대 RAM** | H-A 조합 중 source buffers, H-B 생산 큐 / record cache, H-C 동시 read buffers와 lease / catalog | bounded queue, streaming과 disk 저장을 모든 안에 허용. 512 MB 조건의 충족을 측정 없이 선언하지 않음 |
| **신규 후보 N-H: 업무 인계 정확도** | 외부 실행을 허가하며 확정한 인계 계약에서 필수 목표 / 조건 / 제외 / version / 확인 수준과 접근 방식이 정확한지 | H-A의 즉석 누락, H-B stale record, H-C 누락된 manifest / 읽기 권한 실패를 같은 계약 oracle로 비교. 업무 산출물의 우수성과 분리 |
| **신규 후보: Agent 선택 정확도** | 읽기 지원, session 재사용, artifact 지원 등 인계 가능성이 선택의 필수 조건을 바꿈 | 이 문서의 안 자체가 적합 Agent를 고르는 알고리즘은 아님. 문제 5의 선택 대안에 같은 capability 사실로 입력 |

### N-H의 고정 관찰 지점과 판정

[공통 N-H 정의](./06-qa-candidates-and-coverage.md#3-신규-후보-n-h-업무-인계-정확도)를 그대로 사용한다. 평가 단위는 `(Task, request revision, execution attempt, delegation revision)`에 연결된 확정 계약이며, **VIA가 외부 Agent에게 실행을 허가하는 지점**에서 판정한다. 최초 위임, 후속 수정과 Agent 교체에 같은 지점을 적용한다.

독립적으로 정한 oracle은 유효 목표, 완료 조건, 필수 제약과 제외, 결과 source / revision / 범위, 현재 접근 방식, 승인된 수행 범위와 확인 수준을 명시한다. 필수 항목이 빠지거나 모순된 옛 조건이 재도입되면 해당 계약 전체가 실패다. 항목별 누락은 원인 진단으로 남기며 항목 점수를 합쳐 계약 실패를 가리지 않는다. 문장 표면의 완전 일치를 요구하지 않는다.

H-A/B는 실제 확정한 본문과 첨부 / reference를, H-C는 필수 manifest와 실제 접근 가능한 허용 source 계약을 판단한다. 원본 본문을 전부 복사할 필요는 없지만 목표와 필수 수행 제약을 선택적 검색 뒤에 숨기지 않는다. 확정 당시 만료된 locator나 없는 필수 원본은 실패다. 수신 Agent가 유효 자료를 스스로 읽지 않은 오류나 업무 수행 실패는 이 VIA 지표와 구별한다.

외부 native receipt를 제공하는지 별도 관측하되 그 수신 시점으로 N-H 경계를 바꾸지 않는다. 위임을 올바르게 보류해야 하는 장면은 사전에 명시된 보류 계약으로 판정한다. 목표값, deadline, 점수와 최종 모집단은 이번에 정하지 않는다.

문제 1에서도 근거 공급 오류가 **이 동일한 최종 위임 계약**까지 남는 장면에만 N-H를 적용한다. 해석 모델의 중간 입력에 필요한 근거가 있었는지는 별도 원인 진단이다. 직접 응답 장면까지 인계 지표로 넓히지 않는다.

## 8. 강한 보완, 혼합과 반증 조건

| 선택 / 보완 | 처리와 비용 | 판단 |
| --- | --- | --- |
| H-A의 반복 capsule cache | 동일 goal / source / revision에서 인계 재사용. 정정 시 dependency 검사 | 필수 steelman. 첫 인계 비용을 매번 강제하지 않음 |
| H-B의 지연된 부분 생산 | 실제 재사용 가능성이 높은 업무만 생산하고 최근 수정은 on-demand 보완 | 실용적 H-A/B 혼합. 어떤 업무를 생산할지 선택 비용과 miss를 포함 |
| H-A/B에 source 읽기 추가 | capsule에 필수 조건, 큰 자료는 읽기 API로 확장 | H-C의 transport 축을 조합한 혼합. read capability와 lease 비용 추가 |
| H-C의 자주 쓰는 구절 선행 전달 | 필수 manifest 외에 반복 자료 일부를 push해 read 왕복 감소 | 허용. 대부분 자료를 push하게 되면 H-A/B와 수렴하는지 확인 |
| 광범위 공개 history replay | 누락을 줄이려고 허용된 관련 transcript를 추가 전달 | budget과 무관한 이력 / 오래된 조건 혼동 비용. 이종 Agent의 tool 메시지와 private state를 그대로 이식하지 않음 |
| 동일 native session 재사용 | 간단한 수정은 delta 전달. source 및 조건 변경 확인 | 모든 안의 공통 최적화. 한 안만 금지해 차이 제조 금지 |

**H-A 이익의 반증:** 반복 인계에서도 새로운 원본 조합이 대부분 필요하고 해당 비용이 유지 record의 생산 / 갱신보다 커진다. 반대로 작은 cache만으로 H-B의 이익을 대부분 얻는다면 H-B를 별도 major DP로 삼을 근거가 약해진다.

**H-B 이익의 반증:** 다음 목적이 달라 게시 record를 재사용하기 어렵거나, 수정이 잦아 원본 재조합과 게시 비용을 모두 치른다. 구조화된 현재 조건과 artifact catalog만으로 대부분 인계가 완료되면 별도 의미 producer의 정상 사용 이익이 줄어든다.

**H-C 이익의 반증:** 사용할 Agent 다수가 read interface를 지원하지 않거나 작은 자료를 읽기 위해 반복 왕복이 발생한다. 전체 자료 push가 자주 필요해지는 경우 H-C는 범용 주안보다 선택적 대형 자료 경로에 적합하다. downstream 비용 이동만으로 좋아 보이면 해당 성과 주장을 기각한다.

**업무 인계 정확도 차이의 반증:** 공통 필수 manifest와 강한 source 검증을 적용한 뒤 세 안이 같은 의미를 전달하면 이 QA에서는 동점이다. 동점 때문에 강한 안의 검증이나 cache를 제거하지 않는다.

## 9. 문제 1과의 관계와 DP 권고

문제 1은 **현재 사용자 발화를 이해하는 VIA 모델 입력**을 언제 만드는가다. 문제 4는 **업무를 수행할 외부 Agent에 어떤 공개 상태와 source 접근을 넘기는가**다. 둘은 같은 기록을 사용할 수 있지만 입력을 받는 주체, 완료 조건과 수명이 다르다.

| 가능한 조합 | 성립하는 이유 |
| --- | --- |
| E-A 대화 view + H-A 위임 시 capsule | VIA는 연속 대화를 유지하면서 외부 인계만 목적별로 작게 구성 |
| E-B 요청별 해석 + H-B portable work record | 현재 지칭은 원본에서 판단하고 이미 정리된 업무 조건은 여러 Agent에 재사용 |
| E-C 게시 기억 + H-C 수신자 읽기 | VIA는 과거 사건을 회상하고 수신 Agent는 필수 manifest와 원본을 읽어 업무 수행 |

**현재 권고는 문제 4를 독립적인 심화 후보로 유지하는 것이다.** 사용자가 제시한 Agent 교체 후 일관된 경험을 직접 설명하고, native session과 portable context의 경계를 설계하게 한다. H-A/H-B의 가공 시점은 문제 1 및 기존 DP-45와 겹치므로 공통 producer를 사용한다면 하나의 구조 선택으로 합쳐야 한다. 같은 요약 생산 비용을 두 DP의 별도 품질 이익으로 중복 계산하지 않는다.

H-C는 ‘인계 내용을 누가 세부 선택해서 읽는가’라는 추가 축으로 남긴다. H-A/B 중 하나와 조합할 수 있으므로 단일 3자 경쟁으로 점수를 서열화하기보다 **가공 시점 축과 push / bounded pull 축**을 구분한 뒤 실제 구성별 비용을 비교하는 것이 맞다. 사용 가능한 외부 Agent 계약을 확인했을 때만 범용 DP의 주안 자격을 판단한다.

현재 QA01/02는 장면에 따라 동점이고 QA04 우위도 미확인이다. 따라서 새 문제라는 이유만으로 course의 주요 DP 조건을 이미 충족했다고 보고하지 않는다. 업무 인계 정확도와 QA03/05의 실질 차이, H-C의 지원 범위를 함께 검토하는 것이 다음 판단 근거다. 현재 자료에는 공식 코드의 존재와 설계 인과가 있으며 VIA 구현이나 측정 결과는 없다.
