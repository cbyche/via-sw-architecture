# 재시작 후 현재 상태를 복원할 것인가, 확정 이력에서 상태를 재구성할 것인가

> **상세 검토 후보 / 사용자 선정 전** · [목록](./README.md)
> 업무·대화·전달의 복구 원본을 비교한다. DB 제품·process 수·외부 Agent의 exactly-once를 고르는 문서가 아니며 구현·측정은 없다.

## 발표용 2페이지

**배경 — 이 과제에서 왜 어려운가**

![복구 상태 원본: 사용자 사건·근거 변화·구조적 난점](./diagrams/recovery-state-source-background.svg)

[배경 SVG 크게 보기](./diagrams/recovery-state-source-background.svg) · [배경 draw.io 편집 원본](./diagrams/recovery-state-source-background.drawio)

**설계 비교 — 같은 완료 조건을 만드는 두 실행 구조**

![복구 상태 원본: 두 대안의 내부 모듈·상태·호출·예외 경로 비교](./diagrams/recovery-state-source-comparison.svg)

[비교 SVG 크게 보기](./diagrams/recovery-state-source-comparison.svg) · [비교 draw.io 편집 원본](./diagrams/recovery-state-source-comparison.drawio)

검정은 양안 공통, 파랑은 **양안 각각에서 달라지는 모듈·상태·계약**이다. 큰 테두리는 논리 책임 묶음이며 모든 상자가 별도 process라는 뜻이 아니다. 같은 Component를 여러 위치에 확대 표기해도 instance·모델 가중치를 복제하지 않는다. 그림의 내부 모듈과 아래 계약은 대안을 검토하기 위한 구체 설계이며 target 기준선 변경·구현·측정 결과가 아니다.

## ASR·추가 QA 관점의 장단점과 예상 차이

아래는 **동일 기능·완료 조건에서의 구조적 예상**이며 측정 결과나 승자 선정이 아니다. `PRIMARY`는 구조 차이가 개선·악화에 직접 영향을 주는 축, `REGRESSION_ONLY`는 직접 바꾸지 않는 공통 경로의 기능 유지를 확인할 축이라는 **적용 제안**이다. 인과 범위를 정하지 못한 경우에는 `UNRESOLVED`로 남긴다. 정식 모집단·수치·역할은 아직 동결하지 않았다. [현재 ASR 정의](../../08-quality-attributes/core-asr-contract.md)와 [상세 QA 의미](../../08-quality-attributes/README.md)를 유지한다.

**직관적인 핵심:** 1안은 현재 장부와 미완료 전송 목록을 읽어 이어가고, 2안은 확정된 변경 이력으로 현재 장부를 다시 만든다. 이력은 관계 재구성에 도움이 되지만 과거 schema·삭제·재생 비용을 계속 유지해야 하며 원격 실행의 잃어버린 ACK까지 알려 주지는 않는다.

| 관점 · 적용 제안 | 방안 1의 장단점 | 방안 2의 장단점 | 차이가 나는 조건·주의점 |
| --- | --- | --- | --- |
| QA-19 의미·처리 정확성 · REGRESSION_ONLY | 정상 입력의 의미 추론을 바꾸지 않는다. 복원한 Task·질문·결과 관계가 틀리면 후속 요청 처리는 틀릴 수 있다. | 같은 의미 모델을 유지한다. replay가 확정된 관계를 복원할 뿐 과거 모델 판단의 사실 오류를 바로잡지는 않는다. | 정상 의미 정확도 향상을 주장하지 않는다. 복구 후 잘못된 Task·artifact 연결은 QA-39와 관련 binding 회귀에서 확인하고 QA-19 field는 해당하는 것만 본다. |
| QA-09 정상 응답성 · PRIMARY | **장점:** current records와 필요한 intent만 갱신하는 짧은 write 경로를 구성할 수 있다.<br>**단점:** 복잡한 관계 갱신·index 비용은 남는다. | **장점:** append 중심 기록을 활용할 수 있다.<br>**단점:** event + projection + effect intent를 같은 transaction에 쓰므로 append만의 비용이 아니다. | 기준선도 durable write를 한다. 같은 DB·내구성에서 사용자 경로의 commit 대기를 비교하며 event 방식이 무조건 빠르거나 느리다고 단정하지 않는다. |
| QA-29 변경 용이성 · PRIMARY | **장점:** 현재 row schema와 migration을 중심으로 진화한다.<br>**단점:** 오래된 상태를 새 관계로 옮기는 repair가 복잡할 수 있다. | **장점:** 새 projection을 확정 이력에서 다시 만들 수 있다.<br>**단점:** 지원 기간의 event schema·version reader·reducer·checkpoint·redaction을 유지해야 한다. | 새 현재 view 추가와 과거 domain event 의미 변경은 다른 비용이다. 이력 기반이라는 이유만으로 유지보수가 쉬워지지는 않는다. |
| QA-39 신뢰성·복구 · PRIMARY | **장점:** 현재 상태·미완료 원장을 직접 읽어 짧게 복구할 수 있다.<br>**단점:** current state 관계 손상은 이력만으로 임의 복원하지 못한다. | **장점:** projection 손상은 authoritative batch에서 재구성할 수 있다.<br>**단점:** 긴 tail·호환 reader 실패·권위 이력 손상은 복구를 지연·차단한다. | projection 손상과 권위 저장소 손상을 구별한다. 외부 ACK 불명·실제 청취 여부는 양안 공통 한계이며 자동 재실행으로 성공 처리하지 않는다. |
| QA-31 복구시간 / QA-32 장애 범위 · 상세 진단 | row load·migration·source reconcile 시간과 손상 scope를 본다. | checkpoint 검증·tail replay·source reconcile 시간과 reader 실패 scope를 본다. | QA-39를 설명하는 입력이다. 재시작 화면이 뜨는 시간이나 replay 종료 시각이 아니라 올바른 업무·허용 control의 복구가 끝점이다. |
| QA-41 메모리 / 저장·I/O 비용 · 추가 진단 | current/index cache·migration·pending scan buffer가 필요하다. | replay window·projection·checkpoint buffer가 겹쳐 peak가 커질 수 있다. | 긴 event log의 디스크 byte는 QA-41 메모리와 다르다. 저장공간·write amplification·삭제 처리량은 별도 원인 분석 항목이며 새 QA 점수로 발명하지 않는다. |
| QA-61 추적 / QA-62 재현 / QA-51 노출 · 추가 qualification | audit·identity·clock을 별도로 완전하게 남겨야 하며 current row만으로 trace 완전성을 보장하지 않는다. | domain 이력이 전이를 설명하지만 실제 audible 시각·모델 입력 provenance·평가 manifest를 대신하지 못한다. 삭제 payload와 옛 checkpoint도 관리해야 한다. | event replay와 평가 결과 재계산은 다르다. QA-62는 보존된 raw evidence로 같은 평가 결론을 만드는 능력이며 운영 이력이 많다고 자동 향상하지 않는다. |

추가 QA는 기존 지위 그대로 진단·회귀·qualification으로 다룬다. 더 빠른 응답으로 잘못된 대상 실행·중복 실행·권한 위반을 상쇄하지 않는다. 메모리의 core ASR 승격이나 새 QA 정의는 이번 정성 비교에서 확정하지 않는다.

## 그림을 따라 설명할 실행 계약

상단 세 owner가 검증한 변경을 왼쪽 **State Store**의 동일 embedded transaction으로 결합한다. 오른쪽은 재시작 후 각 owner가 수행하는 복구 절차다. 두 안은 같은 DB 내구성·교차 변경의 원자성을 유지하고, **current view가 손상되거나 원본과 불일치할 때 무엇으로 재구성하는가**만 바꾼다.

| 경계 / 상태 | 방안 1의 구체 동작 | 방안 2의 구체 동작 |
| --- | --- | --- |
| commit 단위 | owner current row·expected revision·관계와 command/publication intent를 한 transaction으로 갱신 | `DomainBatch(batch_id, owner revisions, schema_version, events[], effect_intents[])`를 append하고 현재 projection·적용 위치도 같은 transaction으로 갱신 |
| 권위 원본 | current records + 미완료 송수신·전달 원장 | commit된 domain batch; checkpoint·projection은 가속용 파생 상태 |
| 재시작 읽기 | row schema migration·관계 무결성·pending index 읽기 | 검증된 checkpoint 위치 뒤 committed tail을 versioned reducer로 재생; checkpoint 불일치면 사용 가능한 이전 위치부터 재구성 |
| replay 범위 | replay로 운영 의미를 재결정하지 않음 | 확정된 상태 전이만 적용; 모델 재추론·메일 재발송·과거 음성 재생을 reducer에서 호출하지 않음 |
| 외부 효과 | command key·attempt·source ACK/불명을 확인해 reconcile | 같은 effect ledger·source 확인; event를 replay했다고 외부 실행 여부를 안다고 가정하지 않음 |
| 복구 개방 | owner 관계·미전달 질문·publication·현재 권한을 확인한 scope부터 개방 | 동일; replay 완료만으로 새로운 dispatch를 허용하지 않음 |

**전송 직후 crash:** commit된 command intent를 dispatcher가 읽어 전송한 뒤 ACK를 기록하기 전에 죽으면 두 안 모두 `UNKNOWN`이다. Agent Gateway는 같은 command key로 source 상태를 조회한다. Source가 확인·중복 방지 capability를 제공하지 않으면 새 Agent에 다시 start해 해결하지 않고 불명을 보존한다. 이력 원본은 로컬 관계 복원에는 도움을 줄 수 있지만 외부의 잃어버린 ACK를 복구해 주지는 않는다.

**삭제와 schema 진화:** 복구 시작 시 현재 tombstone·deletion epoch를 먼저 적재해 옛 payload 사용을 막는다. 방안 2는 event metadata와 삭제 가능한 payload를 분리하고 payload redaction 뒤에도 동작하는 reader·checkpoint 무효화 계약을 유지한다. 호환 불가능한 schema나 권위 원본 손상은 해당 scope를 복구 실패로 남기며 조용한 best-effort 성공으로 처리하지 않는다.

**수명·성능 비용:** 방안 1은 row migration·원장 repair가, 방안 2는 event schema·reader·checkpoint·tail retention이 추가 유지 대상이다. Snapshot 주기 하나를 바꾼 비교가 아니다. 이 대안에서 checkpoint는 권위 원본이 아니므로 지원 중인 scope를 재구성하는 데 필요한 이력 prefix를 checkpoint만 믿고 제거하지 않는다. 현재 deletion epoch에 따라 payload를 redaction하되 재구성에 필요한 비민감 전이·관계 metadata는 보존한다. 저장 상한에 도달하면 기존 저장소 장애·admission 제한 계약으로 새 변경을 보류하며, 복구 원본을 몰래 폐기하지 않는다. 종료 scope의 보관 만료는 참조·삭제·복구 범위 계약에 따라 처리한다. Prefix compaction을 도입하려면 내구 base generation의 권위·검증·삭제 변환·원자 교체를 별도 설계해야 하며 이번 대안에 포함하지 않는다. 구체 용량·복구 deadline은 아직 측정 freeze가 아니다.

**심사 질문 — “로그는 두 안 모두 있는데 차이가 뭔가?”** Audit·outbox가 존재하는지로 구별하지 않는다. 1안의 current row와 audit가 다르면 audit를 임의 replay하여 운영 상태를 덮지 않는다. 2안의 projection과 committed event batch가 다르면 event 쪽에서 다시 만든다. 권위 원본과 복원 알고리즘의 차이다.

## 1. 배경 — 다시 켜졌다는 것과 업무를 올바르게 이어간다는 것은 다르다

사용자가 “메일은 취소해줘”라고 말한 뒤 VIA가 종료됐다. Agent는 취소를 받았을 수도, 메일을 이미 보냈을 수도 있다. 다른 보고서 Task의 결과는 VIA에 저장됐지만 사용자에게 아직 표시되지 않았을 수 있다. 재시작 후 Task 이름만 복원하면 충분하지 않다. 확정된 의도·전송 시도·확인된 외부 사실·질문·사용자 전달 상태를 다시 연결해야 한다.

현재 상태를 직접 복원하면 빠르게 필요한 관계를 읽을 수 있지만, 상태 migration과 미완료 전달 원장을 함께 일관되게 유지해야 한다. 확정된 변경 이력을 기준으로 재구성하면 상태가 만들어진 경로를 다시 적용할 수 있지만, 재생 규칙·checkpoint·과거 schema·삭제된 payload의 수명을 관리해야 한다. **현재 행과 변경 이력이 불일치할 때 무엇을 신뢰해 운영 상태를 복원할 것인가**가 중심 질문이다.

현재 선택은 [전체 구조 §11~14](../target-architecture/architecture.md#14-프로세스-배치fault-boundary), [제어 계약 §5·7](../target-architecture/control-and-lifecycle.md#7-장애재시작버전-변경), [DB·파일 crash 계약](../target-architecture/memory-and-context-lifecycle.md#6-파일과-db의-crash-일관성)에 있다. [UC-12](../../05-representative-use-cases.md#uc-12), [UC-13](../../05-representative-use-cases.md#uc-13), [UC-16](../../05-representative-use-cases.md#uc-16), [UC-18.6](../../05-representative-use-cases.md#uc-18)의 중복 없는 업무·질문·결과 복구를 다룬다.

## 2. 비교 범위와 공통 조건

대상은 VIA가 확정한 Request·Task·command·질문·publication 관계와 확인된 delivery 상태다. Raw microphone·화면 전체를 event log로 영구 기록하지 않는다. 원본 자료는 현재 보관·삭제 정책을 따르며 Agent 내부 checkpoint·tool 실행은 양쪽 모두 외부 책임이다.

같은 local embedded transactional DB와 evidence 파일, 같은 durability 수준, owner별 쓰기 권한·교차 Unit of Work, Agent capability, process 배치를 유지한다. 양쪽 모두 audit·outbox·inbox·빠른 현재 view를 가질 수 있다. **현재 설계에도 domain event가 있다. 이벤트의 존재와 event sourcing은 다르다.** 현재 DB 상태를 복구 원본으로 삼는지, 확정 domain 이력이 운영 상태를 재구성하는 원본인지로 구별한다.

## 3. 두 가지 구현 방식

### 방안 1 — 현재 owner 상태 + 미완료 송수신·전달 기록

1. 각 owner가 현재 revision·상태와 관련 command/domain event/publication intent를 하나의 transaction으로 확정한다. 여러 owner 관계는 각각의 변경을 Unit of Work로 묶는다.
2. 전송 worker는 현재 admission·hold·policy·command epoch를 검사해 DISPATCHING을 commit한 뒤 외부 호출한다. 접수 불명은 UNKNOWN으로 남긴다.
3. 재시작 시 incarnation·전송 fence를 세우고 owner 상태·schema·inbox·outbox·cursor를 복원한다. 미적용 event와 미완료 전달을 멱등 처리하고 Agent 상태를 조회한다.
4. 질문·publication 관계를 다시 연결한 뒤 해당 경로의 admission을 연다. 감사 이력만 보고 현재 업무 상태를 임의로 다시 쓰지 않는다.

DB의 WAL과 backup은 저장 엔진의 내구성 수단이며 업무의 domain event 이력을 진실로 삼는다는 뜻은 아니다. 현재 설계도 충분한 cause ID와 revision을 남길 수 있으므로 ‘변경 이유를 전혀 알 수 없다’는 약한 안으로 만들지 않는다.

### 방안 2 — 확정 domain 이력 + checkpoint·현재 projection

1. 같은 owner가 입력 명령과 expected revision을 검사한 뒤 확정된 상태 전이와 effect identity를 domain 이력에 기록한다. 여러 owner의 연결은 동일 transaction의 event batch와 관련 outbox로 확정하여 반쯤 연결된 관계를 노출하지 않는다.
2. 정상 읽기는 빠른 현재 projection을 사용한다. 외부 전송·승인처럼 즉시 일관성이 필요한 projection은 이력 append와 같은 transaction에서 갱신할 수 있다. 단지 eventual consistency를 강제해 대안을 약하게 만들지 않는다.
3. Checkpoint는 적용한 이력 위치·owner revision·schema version을 가진다. 재시작 시 검증한 checkpoint와 이후 tail을 side-effect 없는 reducer로 적용한다. Projection이 다르면 유효 이력과 삭제 제약을 기준으로 다시 만든다.
4. 저장된 effect identity·전송 시도·외부 확인을 연결해 미완료 상태를 찾고, 공통 Agent query로 실제 사실을 조정한다. 새 admission은 관련 scope의 복구와 현재 policy 검증 뒤 연다.

재생은 **과거 모델 판단·외부 읽기·메일 발송을 다시 실행하는 것이 아니다.** 당시 확정된 의미 결과·상태 전이·참조를 적용한다. 모델 출력 재생성이나 현재 외부 자료를 읽어 과거 판단을 고치면 복구 의미가 바뀐다. 재구성에 필요한 version reader가 없으면 해당 경로를 보류하며 이전 승인·새 command를 임의 생성하지 않는다.

## 4. 삭제·전송·물리 출력의 한계를 양쪽에서 닫는다

**외부 전달:** 양쪽 모두 DISPATCHING 이후 ACK 소실을 UNKNOWN으로 다룬다. 이력에 ‘보냈다’가 있어도 Agent의 실행 완료를 증명하지 못한다. 같은 command key로 조회·중복 방지를 지원할 때만 계약에 따라 재전송하며, 미지원이면 불명을 유지한다. 재생한 오래된 outbox를 무조건 다시 보내지 않는다.

**사용자 전달:** Text는 같은 publication ID로 upsert한다. Voice의 실제 재생과 receipt 저장은 원자적이지 않다. 마지막 확인 범위 밖은 DELIVERY_UNKNOWN이고 음성을 자동 재생하지 않는다. 더 긴 이력을 남겨도 관측하지 못한 청취 사실은 복원할 수 없다.

**삭제:** 방안 2의 이력은 삭제 불가능한 개인정보 원문 저장소가 아니다. Payload는 삭제 가능한 참조로 분리하고 event·checkpoint에 불필요한 원문·선호 값을 중복 저장하지 않는다. 삭제 tombstone·현재 data/policy epoch를 먼저 적용해 과거 event가 기억·권한·payload를 다시 살리지 못하게 한다. 영향받는 checkpoint·파생 view도 무효화·purge한다. 삭제 후에는 삭제된 값의 과거 상태까지 재현할 수 있다고 주장하지 않으며, 허용된 최소 identity·삭제 상태·미확인 command 관계만 복원한다. Event schema가 민감한 값을 포함한다면 그 부분의 삭제·redaction과 replay 처리를 계약에 포함해야 한다.

Checkpoint나 이전 export가 현재 삭제 ledger보다 오래됐거나 검증할 수 없으면 자동 병합하지 않는다. 이는 현재 [기억 삭제 계약](../target-architecture/memory-and-context-lifecycle.md#5-기억-변경삭제와-권한-철회)을 대안에서도 지키기 위한 비용이다. Append-only라는 편의 때문에 사용자 삭제 기능을 포기하지 않는다.

## 5. 같은 상황을 따라가 보면

| 사건 | 현재 상태 복구 | 이력 재구성 |
| --- | --- | --- |
| 전송 직전 DISPATCHING commit 후 crash | 현재 command·attempt에서 UNKNOWN 복원 | 확정 dispatch 전이·attempt에서 UNKNOWN 재구성; 외부 query는 동일 |
| Task 결과 확정 후 publication 인계 전 crash | domain outbox의 미완료 인계를 찾아 publication 생성/복원 | 이력과 소비 위치·동일 publication identity로 미완료 인계 복원 |
| 현재 projection 일부만 손상, 원본 저장은 정상 | 현재 authoritative 상태라면 복구 가능한 backup·정해진 repair가 필요; audit로 임의 덮어쓰지 않음 | 권위 이력과 유효 checkpoint가 온전하면 해당 projection 재생성 가능 |
| DB 매체·권위 이력 자체 손상 | 확인 가능한 backup/기록 밖은 불명 | 이력도 손상되면 재생만으로 복구 불가; 별도 장애 마법은 없음 |
| schema 업데이트 후 재시작 | 현재 row·pending command migration | event reader/reducer·checkpoint 호환, 필요 시 projection 재구성 |
| 선호 삭제 후 옛 checkpoint를 읽음 | 현재 tombstone·epoch로 삭제 상태 유지 | 현재 삭제 ledger로 checkpoint를 차단/정리하고 삭제된 payload 복원 금지 |

세 번째 사건은 대안의 조건부 장점이다. 같은 물리 손상에서 양쪽의 무엇이 남았는지 공개해야 한다. 방안 1에도 backup·무결성 검사·복구 도구를 허용하며, 방안 2의 이력만 무조건 온전하다고 가정해 전체 장애 우세를 만들지 않는다.

## 6. 실제 구조 차이와 선택의 대가

| 위치 | 방안 1 | 방안 2 |
| --- | --- | --- |
| State Store | authoritative 현재 row·revision·미완료 원장 | authoritative event batch·position·checkpoint·재생 가능한 projection |
| 상태 owner | 현재 전이·repository·row migration | append 계약·versioned event·순수 reducer·checkpoint 호환 추가; 단일 writer 유지 |
| 복구 경로 | 현재 state load → pending 재처리 → 외부 조정 | checkpoint 검증 → tail replay → effect 조정 → 외부 조정 |
| 정상 경로 비용 | 현재 상태와 내구 intent 쓰기 | event append·projection·checkpoint 유지; 동기 쓰기 범위 공개 |
| 보관·삭제 | 현재 데이터·audit·파생 cache 수명 | 이력 payload·여러 checkpoint·projection까지 삭제 dependency 확대 |

**현재 방식이 설득력 있는 조건:** 현재 상태·미완료 동작만으로 필요한 재연결을 설명할 수 있고, 상태 schema가 비교적 안정적이며, 광범위한 과거 replay가 제품 가치의 중심이 아닌 경우다. 별도 event schema의 장기 호환 책임을 줄일 수 있다.

**왜 대안을 선택할 수 있는가:** 여러 owner의 전이·전달 인과를 재구성해야 하는 문제가 잦고, 확인된 상태 view를 다시 만들거나 변경된 projection을 구성할 필요가 큰 경우다. 확정 이력과 재생 계약을 공통 복구 경로로 사용할 수 있다. 단순 audit만 필요하다면 현재 안에도 audit를 넣을 수 있으므로 그것만으로 대안을 정당화하지 않는다.

**현재 선택을 다시 볼 조건:** 현재 기록과 전달 원장의 불일치를 복구하는 별도 조정 로직이 반복적으로 커지고, 확정 이력·검증된 replay로 같은 사용자 관계를 더 명확하게 복구할 수 있는 경우다. 반대로 event migration·삭제·긴 tail 비용이 더 크거나 실제 필요한 복구가 현재 상태 조회로 충분하면 대안의 장점이 작다. 정상 응답 속도나 의미 정확성을 event sourcing 자체가 높인다고 주장하지 않는다.

## 7. 다른 선택과 섞지 않는 범위

[Agent 상태 관측](./agent-state-observation.md)은 외부 상태를 이벤트 또는 snapshot에서 얻는 선택이고, 이 후보는 **이미 VIA가 확정한 상태를 무엇에서 복원하는가**다. Event projection 방식도 current-state 저장을 쓸 수 있고, snapshot 관측도 확정 관측 사실을 이력에 남길 수 있다. 두 축은 독립적으로 고정한다.

[기억 working view](./conversation-context-maintenance.md)는 원본이 아니라 파생 입력이다. 그 view 손실은 양쪽 모두 재구성할 수 있으며, 여기서는 command·질문·전달 같은 운영 원본의 계약을 비교한다. DB 제품·process 격리·교차 transaction·Task writer·Agent 의미 정규화는 공통이다. 기존 ADR의 accepted/deferred 상태도 변경하지 않는다.

영향받는 target 계약은 owner repository·Unit of Work payload, state/event schema와 migration, incarnation fence 이후 복구 절차, checkpoint·삭제 전파다. 외부 정확히 한 번 실행 보장이나 새 보관 기간을 추가하지 않는다. ASR 적용·failure population·deadline은 후속 공동 검토이며 과거 결과를 새 근거로 쓰지 않는다.

## 8. 발표 페이지의 핵심

**배경 1장:** ‘취소 요청 → 전송 시작 → ACK 미수신 → VIA 종료’와 ‘다른 Task 완료 → 사용자 전달 전 종료’의 두 갈래를 그린다. 재시작 화면에 필요한 사실과 모르는 사실을 분리한다. 질문은 **“중단된 interaction을 무엇을 근거로, 어디서부터 복원할 것인가?”**다.

**비교 1장:** 같은 Agent query·owner·효과 ID·출력은 검정. 1안의 current state·pending ledger·loader, 2안의 event store·checkpoint·reducer·projection은 양쪽 파랑. Commit 전후 crash 위치, replay에서 외부 호출로 바로 나갈 수 없다는 경계, 양쪽 UNKNOWN 처리와 삭제 fence를 표시한다. 상단의 배경·비교 SVG와 편집 가능한 draw.io 원본에 이 경로를 반영했다.

## 9. 바로 사용할 발표 요약

**배경 5줄**

1. VIA 재시작은 Task 목록뿐 아니라 미확인 전송·대기 질문·아직 전달하지 않은 결과를 복원해야 한다.
2. 잘못 복구하면 중복 업무 실행이나 오래된 승인의 재사용으로 이어질 수 있다.
3. 현재 상태를 읽는 방식은 미완료 원장과의 일관성·migration을 유지해야 한다.
4. 변경 이력에서 재구성하는 방식은 replay·checkpoint·과거 schema·삭제 수명을 관리해야 한다.
5. 핵심은 로그 유무가 아니라 충돌할 때 무엇을 운영 상태의 복구 원본으로 삼는가다.

**설계 비교 8줄**

1. 방안 1은 현재 owner 상태와 command·inbox·publication의 미완료 기록을 읽어 복원한다.
2. 방안 2는 검증된 checkpoint와 확정 domain 이력을 재생해 현재 projection을 만든다.
3. 두 안 모두 같은 DB 내구성·owner 권한·교차 transaction·Agent 기능을 유지한다.
4. 재생은 저장된 판단·전이를 적용하는 것이며 모델 추론이나 외부 Action의 재실행이 아니다.
5. 외부 ACK와 실제 청취가 불명인 사실은 어느 방식도 이력만으로 성공으로 바꿀 수 없다.
6. 현재 상태 중심이면 복구가 단순할 수 있고, projection 재구성이 중요하면 이력안이 합리적이다.
7. 이력안의 추가 쓰기·schema 호환·checkpoint·삭제 전파 비용을 함께 설명해야 한다.
8. 필요한 복구가 현재 상태로 충분하거나 이력 자체가 손상되면 event sourcing의 이점을 과장하지 않는다.
