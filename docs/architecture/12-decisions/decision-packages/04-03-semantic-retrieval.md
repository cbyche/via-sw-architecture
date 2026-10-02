# S-03. 자료 검색 정확성을 위한 자료 찾기 설계 — 이름과 키워드로 검색할 것인가, 내용의 의미로도 검색할 것인가?

> 상태: **STAGE_4_REVISED / 사용자 재검토 대기** / 2026-10-02
> [04 전체 지도](./04-00-structural-alternatives.md#s-03) / [설명과 그림의 공통 원칙](./04-00-structural-alternatives.md#12-처음-읽는-사람을-위한-설명과-그림-원칙) / [품질의 공통 의미](./03-00-quality-scenarios.md#2-어떤-품질을-보고-있는가) / [검토 기록](./04-09-structural-review.md)
> 연결 문제: **P-03; P-10~12, P-15 연결**. S는 탐색 질문이며 DP 선정이 아니다. T는 실제 REVIEWED_BASELINE, A/B는 미채택 탐색안이다.

## 1. 어떤 상황에서 필요한 선택인가

> “전에 비용 줄이자고 이야기했던 자료 찾아줘.” 파일명에는 ‘운영 개선’만 있고 비슷한 문서도 있다.

**T의 원본 조회와 기존 index/cache에, A의 지속 의미 후보 생산 서브시스템을 추가할 가치가 있는지 본다.**

### 1.1 사용하는 자료와 용어

| 이름 | 이 비교에서 뜻하는 것 |
| --- | --- |
| 자료 단서 | 사용자가 말한 주제, 파일 이름, 기간, 관련 Task. Request Controller가 조회 범위와 함께 Context Manager에 보냄 |
| 기존 검색 색인과 캐시 | 이름, 단어, 자료 정보와 최근 읽기 결과를 찾는 target의 자료. T에도 검색 기능이 있음 |
| 의미 색인 | A가 허용된 문서 내용에서 만든 추가 검색 자료. 비슷한 뜻의 표현으로 후보를 찾도록 돕지만 원본은 아님 |
| 원본 버전 목록 | 의미 색인의 각 후보가 어느 원본의 어느 버전에서 왔는지 적은 자료. 아래 `Source Manifest`와 같은 뜻 |
| 원문 확인 | 후보로 찾은 자료를 현재 자료 소유자에게 다시 읽고 접근 권한, 내용과 버전을 확인하는 단계. T와 A에 공통 |

A의 추가 색인은 저장과 갱신을 허용한 유한한 자료에서만 만든다. 색인에 없는 자료를 존재하지 않는 자료로 단정하지 않는다.

## 2. 먼저 볼 차이

| 비교 지점 | T: 실제 target | 대안 A |
| --- | --- | --- |
| 후보 생산 | Context Manager의 owner 조회와 metadata/keyword index | Semantic Retrieval Service의 의미 후보 + 기존 조회 fallback |
| 추가 상태 | 기존 revision cache와 summary | Vector Index, Source Manifest, embedding build와 generation |
| 가장 큰 교환 | 사전 의미 색인 비용 없음, 표현 차이에 따른 후보 누락 가능 | 주제 지칭에 유리할 여지, 추가 helper와 갱신/삭제 비용 |

## 3. 구조를 나란히 보기

![S-03 target와 대안의 설계 비교](./diagrams/stage4-s03-comparison.svg)

[그림 크게 보기](./diagrams/stage4-s03-comparison.svg) / [편집 가능한 draw.io](./diagrams/stage4-s03-comparison.drawio)

검정은 공통, 파랑은 T 전용, 초록은 A/B 전용 구성과 경로다. 제목 칸이 있는 상자는 Component이며 내부의 둥근 상자는 그 Component의 Module이다. 원통은 저장소, 접힌 종이는 자료, 육각형은 내부를 펼치지 않은 연동 대상이나 모델이다. 실선은 요청과 전달, 점선 화살표는 결과와 복원이다. 색과 모양의 뜻은 [설계도 작성 기준](./diagram-design-guide.md)을 따른다. 메인 그림의 위쪽 공통 입력은 사용자 요청이 Interaction Manager와 Request Controller를 거쳐 Context Manager의 조회로 들어가는 시작을 보여준다. 아래쪽에는 원본 변경으로 색인을 만드는 A의 경로와, 요청 시 후보를 찾고 원문을 확인해 Context를 돌려주는 경로가 함께 있다. 이 그림은 비교할 책임을 펼친 것이며 전체 배치도나 구현 확정이 아니다.

## 4. 같은 요청을 따라가 보기

![S-03 색인 생산과 요청 처리의 실행 순서](./diagrams/stage4-s03-execution-flow.svg)

[크게 보기](./diagrams/stage4-s03-execution-flow.svg) / [draw.io 원본](./diagrams/stage4-s03-execution-flow.drawio)

사용자 발화가 확정되면 Interaction Manager가 Request Controller에 입력을 전달한다. Request Controller는 허용 범위와 자료 단서를 정해 Context Manager에 조회를 요청한다. 아래 세 흐름은 각각 1부터 읽는다. A의 색인 생산은 원본 변경으로 시작하며 사용자 요청을 기다리는 작업이 아니다. 두 요청 흐름은 같은 사용자 입력, 원본 접근 권한과 자료 범위를 사용한다.

**A의 추가 자료 준비: 원본이 바뀌거나 삭제될 때**

1. 허용된 자료의 소유자가 변경 또는 삭제와 원본 버전을 Indexing Worker에 알린다. 변경 알림이 없으면 Indexing Worker가 허용된 범위에서 버전을 확인한다.
2. Indexing Worker가 자료 소유자에게 현재 원문, 접근 권한과 버전을 요청한다. 삭제된 자료라면 검색 결과에 다시 나타나지 않도록 삭제 상태를 확인한다.
3. 자료 소유자가 읽을 수 있는 원문과 버전을 Indexing Worker에 반환한다.
4. Indexing Worker가 Embedding Runtime에 문서의 검색 표현을 요청한다. 검색 표현은 비슷한 뜻의 문서를 찾기 위해 만든 값이다.
5. Embedding Runtime이 추가 의미 표현 모델에 문서를 보내 검색 표현을 요청한다.
6. 추가 의미 표현 모델이 검색 표현과 모델 버전을 Embedding Runtime에 반환한다.
7. Embedding Runtime이 받은 표현과 모델 버전을 Indexing Worker에 반환한다.
8. Indexing Worker가 완성한 의미 색인과 원본 버전 목록을 함께 게시한다. 일부만 만들어진 색인은 검색에 사용하지 않는다.

**T의 요청 처리: 사용자 입력이 확정됐을 때**

1. Request Controller가 허용 범위와 자료 단서를 Context Manager에 보낸다.
2. Context Manager가 기존 검색 색인과 캐시에서 이름, 기간, 단어와 관련 Task의 후보를 찾는다.
3. Context Manager가 후보 원본의 소유자에게 현재 내용, 버전과 접근 권한을 요청한다.
4. 원본 소유자가 읽을 수 있는 원문, 버전과 누락 범위를 Context Manager에 반환한다.
5. Context Manager가 현재 권한과 실제 읽은 범위를 확인한 Context를 Request Controller에 반환한다.
6. Request Controller가 확정된 사용자 입력과 Context를 Request Interpreter에 보내 의미 해석을 요청한다.
7. Request Interpreter가 Model Access에 모델 호출을 요청한다.
8. Model Access가 공유 Omni 모델에 의미 해석을 요청한다.
9. 공유 Omni 모델이 해석 결과를 Model Access에 반환한다.
10. Model Access가 결과를 Request Interpreter에 반환한다.
11. Request Interpreter가 의미 제안을 Request Controller에 반환한다. Request Controller는 사용한 원문과 현재 권한을 대조한다. 후보가 모호하거나 원문이 없으면 대상 확정을 보류하고 사용자에게 확인을 요청한다.

**A의 요청 처리: 사용자 입력이 확정됐을 때**

1. Request Controller가 T와 같은 허용 범위와 자료 단서를 Context Manager에 보낸다.
2. Context Manager가 Semantic Retrieval Service에 해당 범위의 의미 후보를 요청한다.
3. Semantic Retrieval Service가 Embedding Runtime에 사용자 질문의 검색 표현을 요청한다.
4. Embedding Runtime이 추가 의미 표현 모델에 질문을 보내 검색 표현을 요청한다. 공유 Omni 모델과는 별도의 모델이다.
5. 추가 의미 표현 모델이 질문의 검색 표현을 Embedding Runtime에 반환한다.
6. Embedding Runtime이 검색 표현을 Semantic Retrieval Service에 반환한다.
7. Semantic Retrieval Service가 의미 색인에서 비슷한 문서 후보를 요청한다. 기존 이름과 단어 조회도 후보 탐색에 사용한다.
8. 의미 색인이 후보와 원본 버전을 Semantic Retrieval Service에 반환한다.
9. Semantic Retrieval Service가 후보, 색인 준비 상태와 조회하지 못한 범위를 Context Manager에 반환한다.
10. Context Manager가 후보 원본의 소유자에게 현재 원문, 버전과 권한을 요청한다.
11. 원본 소유자가 원문, 버전과 권한을 Context Manager에 반환한다.
12. Context Manager가 검증된 원문으로 만든 Context를 Request Controller에 반환한다.
13. Request Controller가 현재 입력과 Context를 Request Interpreter에 보내 의미 해석을 요청한다.
14. Request Interpreter가 Model Access에 모델 호출을 요청한다.
15. Model Access가 공유 Omni 모델에 의미 해석을 요청한다.
16. 공유 Omni 모델이 해석 결과를 Model Access에 반환한다.
17. Model Access가 결과를 Request Interpreter에 반환한다.
18. Request Interpreter가 의미 제안을 Request Controller에 반환한다. Request Controller가 현재 원문과 권한을 대조한다. 색인이 준비되지 않았거나 후보가 불확실하면 기존 조회를 사용하거나 사용자에게 확인한다. 후보 1위도 확정된 대상이나 실행 허가는 아니다.

## 5. 누가 무엇을 소유하고 어떻게 실패하는가

### T — 실제 reviewed target

 P-03의 “전에 이야기한 고객 이탈 분석 자료”는 정확한 파일명이 아닐 수 있다. Target의 Context Manager는 owner read port, source adapter, metadata/keyword index, revision cache와 파생 summary를 사용한다. 기본 embedding 모델과 vector DB는 없다. source의 내용, 권한, 부분 coverage를 receipt로 남긴다. 따라서 이 비교는 ‘검색 없음 대 검색 있음’이 아니다. [기억 §1~3](../target-architecture/memory-and-context-lifecycle.md), [02 P-03](./02-00-requirements-and-choices.md#p-03).

### 대안 A — 지속 의미 검색 서브시스템

Indexing Worker가 지속 가공과 보관이 허용된 유한 collection의 자료와 revision을 읽고 Embedding Runtime이 의미 표현을 만든다. Vector Index와 source manifest가 파생 자료를 보관하며 Semantic Retrieval Service가 요청의 후보를 생산한다. Context Manager는 후보의 현재 source, 권한, identity와 필요한 본문을 다시 확인한다. 의미 검색 결과는 확정 대상이나 전체 coverage 증명이 아니다. 한정 조회 뒤 여러 후보가 남으면 질문한다.

추가 embedding helper는 역할, weights, runtime, CPU/accelerator, 갱신과 설치 비용을 모두 공개할 별도 dependency다. 이를 shared Omni의 무료 기능으로 계산하지 않는다. 색인이 없는 최초 요청과 갱신 중 요청에는 target의 metadata/keyword 및 source 조회 경로를 fallback으로 유지한다. 따라서 구성 단순화가 아니라 **발견 기능과 준비 비용의 교환**이다.

색인 commit은 source revision과 자료 사용 epoch에 묶는다. 늦은 갱신이 삭제 자료를 되살리지 못하게 tombstone과 purge 작업을 적용한다. index 손상은 원본 복구와 구별해 재구축하고 그동안 fallback의 지원 한계를 알린다. 외부 source가 revision이나 삭제 확인을 제공하지 않는 범위는 freshness와 coverage 미확인으로 남긴다. helper의 검색 품질, source connector와 삭제 전파의 실제 동작은 U-03/04/07/08로 미확인이다.

### 참고 자료를 대조하며 구체화한 경계

| 대안 A의 추가 계약 | 구체 처리 |
| --- | --- |
| 색인 게시 | source ID, revision, chunk span, policy epoch, embedding build와 generation을 결합. 새 generation이 완성된 뒤 manifest pointer와 원자 게시. 부분 생성은 검색에 노출하지 않음 |
| query와 문서 | 같은 embedding build로 query와 document vector를 생성. build 교체 중 서로 다른 표현 공간을 섞지 않고 재색인 또는 이전 generation 사용의 한계를 표시 |
| 변경 발견 | source change feed가 있으면 사용하고 없으면 유한 revision scan. 변경 발견 전 지연은 남으며 소비 직전 원문 검증을 대체하지 못함 |
| 후보 한 건 또는 없음 | top-1은 대상 유일성 증명이 아니며 결과 없음도 원본 없음이 아님. 허용된 범위의 owner 조회나 사용자 확인으로 보완 |
| 삭제와 늦은 job | 검색 gate와 소비 gate에서 현재 policy/data epoch 검사. 늦은 index commit은 CAS로 거절하고 vector/chunk/cache purge 추적. epoch 확인 불명이면 해당 scope의 새 사용 차단 |
| 손상과 배치 | Vector Index는 원본에서 재생성할 파생 자산. local worker와 helper를 같은 worker process로 배치할 수 있으며 별도 DB server를 전제하지 않음. query와 원본 확인 경로는 유지 |

## 6. 품질 차이는 어디에서 생기는가

아래는 T에 대한 대안의 조건부 인과다. V-01~13의 의미와 우선순위는 03을 따르며 새 지표나 측정 결과가 아니다. V-02는 목표 달성을 돕는 적절성과 불필요한 사용자 수고를 함께 보며 되묻기 횟수만을 뜻하지 않는다. 필요한 확인과 승인은 단순 감점하지 않는다. V-04/05는 VIA 귀속 시간이며 Agent 내부 실행과 사람의 대기는 외부 조건으로 구별한다. 기능 손실과 미확인은 평가에서 지우거나 동등으로 취급하지 않는다.

| 관점 | A에서 확인할 품질 경로와 조건 |
| --- | --- |
| V-01 기능 정확성 | 주제 지칭에서 후보 발견을 도울 수 있지만 유사한 다른 자료와 오래된 vector가 오확정을 늘릴 수 있다. 원문 및 identity 확인이 필요하다. |
| V-02 기능 적절성 | 파일명을 기억하거나 자료를 다시 고르는 일을 줄일 여지가 있다. 부정확한 후보 목록이 오히려 선택 부담을 늘릴 수 있다. |
| V-03 기능 완전성 | P-03의 자료 유형과 예외 지원 유지 목표. 색인 불가능 source는 fallback 범위를 밝히며 ‘검색 안 됨’을 ‘자료 없음’으로 바꾸지 않는다. |
| V-04 상호작용 반응성 | 준비된 index에서 후보 응답이 빨라질 수 있으나 cold start, 갱신과 원문 재조회에서는 다르다. |
| V-05 VIA 귀속 요청 완료 시간 | 후보 탐색 단축과 본문 확인, 잘못된 후보 재처리, background 자원 경합을 함께 본다. 결과 몇 건이 빨리 나왔다는 사실만으로 전체 완료 이익을 주장하지 않는다. |
| V-06 자원 활용성과 수용량 | helper weights, vector와 manifest, 초기 색인, 재색인, 삭제와 fallback 유지 비용이 추가된다. 요청 밖 비용도 포함한다. |
| V-07 결함 허용성과 복구성 | 원본과 index의 불일치, 부분 갱신과 손상을 처리해야 한다. fallback이 있더라도 의미 검색의 동일 기능 복구는 아니다. |
| V-08 변경 용이성과 모듈성 | 새 source, embedding 차원 또는 청크 표현 변경이 adapter뿐 아니라 재색인, manifest와 검증에 영향을 준다. 검색 계약으로 호출부를 가릴 수 있는 범위와 구별한다. |
| V-09 분석 및 시험 용이성 | 후보 목록, index/source version과 확정 근거가 연결되어야 발견 실패와 해석 실패를 구분할 수 있다. 전체 원문 로그는 필수가 아니다. Worker 갱신을 멈추거나 삭제 뒤 늦은 색인 commit을 전달해 결과 사용 차단을 확인할 시험 경계도 필요하다. |
| V-10 기밀성 | vector와 snippet도 파생 보호자료다. 읽기 허용과 지속 보관 허용을 구별하고 철회 이후 신규 사용 차단과 purge를 수행한다. |
| V-11 상호운용성과 공존성 | source 열거 및 revision 지원과 embedding runtime 호환에 의존한다. background 색인이 다른 앱의 자원을 사용할 수 있다. |
| V-12 조작 용이성과 사용자 오류 방지 | 의미 유사도를 유일 정답처럼 표시하지 않는다. 후보 출처와 구별 단서를 사용자에게 제시해야 한다. |
| V-13 설치 용이성 | helper와 index 초기화, 버전 호환, 제거 후 파생 자료 정리 단계가 늘어난다. |

## 7. 어떤 조건에서 더 살펴볼 만한가

반복되는 주제 조회와 안정된 자료에서는 준비 비용을 감수할 이유가 있다. exact ID 위주의 조회, 자주 바뀌는 자료, 지속 보관이 허용되지 않는 환경에서는 기존 index/cache보다 추가 효익이 작을 수 있다. [기존 의미 검색안](./semantic-retrieval-subsystem.md)은 이 질문의 입력이지만 이전 QA 표와 선정 판단을 그대로 승계하지 않는다.

같은 QA는 모든 DP와 모든 방안에서 동일한 정의, 지표와 측정 방법을 사용한다. 이 문서의 시험 경계는 후속 검토 대상이며 구현이나 측정 freeze가 아니다. 05의 정식 비교와 강한 방안 2 선정은 사용자 리뷰 후에 진행한다.

## 8. 기존 자료에서 무엇을 참고했는가

| 참고 자료 | 가져온 검토와 이번 적용 | 그대로 가져오지 않은 것 |
| --- | --- | --- |
| [의미 검색 구조](./semantic-retrieval-subsystem.md) | 원자 generation 게시, query/document build 일치, top-1과 coverage 구별, 삭제 후 늦은 job 차단을 §5에 구체화 | 이전 QA 번호와 상대 기호, 완전 지원을 후보 진입 조건으로 쓰는 규칙 |
| [복구 원본 구조](./recovery-state-source.md) | 파생 index 손상과 원본 손상을 구별하며 현재 삭제 fence를 먼저 적용 | 검색 index를 업무의 권위 journal로 승격하지 않음 |
| [지속 요청 구조](./durable-request-orchestration.md) | job 재시도 identity와 현재 revision 검사를 참조 | 색인 관리에 Interaction Workflow Runtime을 필수로 두지 않음 |
| [음성 근거 구조](./speech-evidence-source.md) | 추가 helper 전체 비용과 공유 PC 부하를 드러내는 방식 참고 | 음성 producer 변경과 검색 이익을 결합하지 않음 |
