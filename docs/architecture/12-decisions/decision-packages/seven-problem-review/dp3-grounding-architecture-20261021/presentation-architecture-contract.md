# DP3 발표 그림의 Component·상태·실행 계약

2026-10-21 / 발표 그림 재작성 기준 / 설계 구체화, 미구현·미측정

**A, B-R1, B-R2, B-R3의 기능과 평가 조건은 그대로 두고, 누가 후보를 만들고 누가 모델을 실행하며 누가 최종 의미를 채택하는지를 그림에서 구별한다.** 이 문서는 [CPU runtime 설계](cpu-runtime-architecture.md) §4의 책임을 논리 Component와 내부 Module, 실제 process에 대응시킨다. Reference Architecture나 accepted ADR를 변경하지 않으며 새로운 모델, 기본안, 병렬화 또는 QA 개선을 추가하지 않는다.

[현재 네 대안](cpu-runtime-architecture.md#11-a와-b-r1b-r2b-r3의-네-대안-비교), [로컬 생산기의 입력·출력·확장 계약](local-producer.md), [동일 시나리오 합성 평가](synthetic-comparison/README.md)가 내용의 기준이다. `DP3-GROUNDING-CPU-SYN-1.0` / `VOICE-SCREEN-CPU-SYN-1.0`의 데이터셋·시나리오·분모·가정 수치와 QA01~07 값은 이번 그림 수정에서 동결한다. 실측으로 재분류하지 않는다.

## 1. 책임 매핑의 판정

`Screen Target Producer`(화면 대상 생산기)를 B의 시각 후보 생산 Component로, 실행 제어·adapter·session을 `Model Access` 내부 책임으로 나누는 것은 타당하다. 후보의 경계·관계·판본을 생산하는 일과 모델의 load·실행·회수를 관리하는 일은 다른 계약이다. 두 책임은 이미 CPU runtime 설계에 있으며 이번에 소유자를 명확히 표시한다.

[Reference Architecture의 Component 계약](../../../target-architecture/architecture.md#4-component와-상태-소유권)은 `Model Access`를 논리적인 연동·자원 관리 책임으로 정의하고 client, scheduler, weight owner, ASR adapter가 서로 다른 process에 배치될 수 있음을 명시한다. 따라서 B-R3에서 host client와 두 시각 worker의 실행 부분을 **동일한 논리 Component인 Model Access의 분산 배치**로 그리는 것은 그 의미와 맞는다. UI/SAM worker가 별도 process라는 사실을 새 최상위 Component 두 개로 바꾸지 않는다.

이번 비교는 사용자 지정에 따라 cloud S2S·cloud 최종 의미 모델·local ASR·local CPU 시각 모델을 사용한다. 참조 구조의 on-device Omni 배치와 이번 비교의 배치를 혼합하지 않는다. `Screen Target Producer` 및 아래 영문 내부 Module 명칭은 이번 비교를 설명하는 설계 이름이며, 참조 구조의 정식 Component 목록이나 실제 Rust 모듈 경로로 소급하지 않는다.

| Component의 그림 표기 | 내부 Module 또는 처리의 그림 표기 | 소유 책임 | 소유하지 않는 판단 |
| --- | --- | --- | --- |
| **Interaction Manager** | Evidence Capture / Timeline & Buffer | frame·pointer·selection·window·scroll, 입력 시각·revision·gap과 유한 원본 버퍼. Model Access에서 받은 ASR 전사와 S2S 정제 텍스트의 연결 | 지칭 의도, 최종 요청 의미, 후보 정답 여부 |
| **Request Controller** | Request Revision Guard / 기존 입력·조회 허가 처리 | request/input revision, 실행 attempt, 허용 범위, 최종 의미 commit. 새 범위의 원본 조회와 생산·확장 요청을 허가. Request Revision Guard는 반환의 현재성·의존을 확인하는 내부 처리 | 검출·분할·추적 연산, 의미 모델의 내부 추론 |
| **Context Manager** | Evidence Reader | 허가된 범위의 history/frame/crop/native/이전 binding 조회, read receipt·coverage·source revision | 원본 기록의 의미상 소유권, 모델 실행 수명, 최종 지칭 채택 |
| **Request Interpreter** — A | Joint Grounding Interpreter | 같은 의미 해석 세션에서 원본을 읽어 대상·집합·관계와 요청 의미를 제안. 원본 보완 요청과 잠정 해석 상태 | 요청 commit, 사용자 게시, 후보를 정답으로 강제하는 권한 |
| **Request Interpreter** — B | Multimodal Binding Resolver | 같은 발화와 후보 판본·원본 crop을 연결하여 BindingProposal을 생성. 필요한 후보 확장·분할·재추적을 제안 | worker queue·weight 수명, 생산 기록을 우회한 최종 영역 확정 |
| **Screen Target Producer** — B | Candidate Assembler | 허가된 원본에서 native 후보 등록, 시각 모델 작업 구성, 공간 관계 계산, SceneEpisodeRevision의 생성·변경·coverage. 파생 후보의 단일 writer. 기존 생산 제어와 공간 관계 계산을 이 내부 Module의 책임으로 표현 | 사용자의 최종 의도·요청 commit, CPU 실행 자원의 직접 소유 |
| **Model Access** — 공통 cloud 경로 | Cloud Model Client | 동일 S2S·의미 모델 연동, 호출·응답·사용량·오류 계약. 발표 확대도는 cloud 의미 모델 client를 표시 | 모델의 제안을 최종 의미로 채택하는 권한 |
| **Model Access** — B local 경로 | Local Model Execution Controller / IPC Endpoint / UI Adapter / SAM Adapter | worker 기동·건강·generation, 전체 시각 CPU admission, queue, load·release·cancel, 전후처리, model session과 tensor episode의 실행 수명 | 후보 기록의 의미상 권위, 어떤 후보를 사용자가 뜻했는지의 최종 판단 |
| **Response Manager** / **Interaction Manager** | 기존 응답 준비·실제 표시 처리 | Request Controller가 허가한 내용의 준비·전달과 실제 표시 기록 | 모델 또는 worker 반환만으로 임의 게시 |

`Screen Target Producer`의 후보 권위는 **파생 후보 기록을 누가 갱신하는가**라는 뜻이다. 원본의 사실 권위나 사용자가 가리킨 정답의 권위를 갖는다는 뜻이 아니다. worker가 반환한 bbox·mask·track은 후보 생산의 재료이며, 기록 판본과 좌표·출처·coverage를 갖춰 게시하는 주체는 Screen Target Producer다.

## 2. 상태의 소유자와 실제 위치

작은 후보 기록과 큰 영상 tensor를 한 원통에 합치지 않는다. 특히 후보 기록이 host에 남는 것과 worker의 embedding이 살아 있는 것은 다른 사건이다.

| 그림에 표시할 자료·상태 | 의미상 소유자 / 실제 위치 | 수명·회수 및 장애 계약 |
| --- | --- | --- |
| **Input Evidence Buffer** | Interaction Manager / VIA host | 공통 원본 보존 기간. 모델 unload와 별개. frame 시각·좌표 변환·gap을 유지. Context Manager 내부의 원본 참조/cache는 이 버퍼의 소유권을 이전한 것이 아님 |
| **Request Revision & Read Set** | Request Controller / VIA host | 현재 입력 판본과 실제 읽은 원본·후보 판본. 옛 입력 결과의 채택을 차단 |
| **Interpretation State** | Request Interpreter / 해당 해석 실행 | A의 잠정 직접 해석 또는 B의 잠정 binding. A도 유효한 앞선 채택 결과를 재사용 가능. Context Manager가 반환하는 확정 binding은 Request Controller가 채택한 기록의 읽기용 사본/참조이며 최종 의미 writer가 바뀌지 않음 |
| **Target & Interaction Records** | Screen Target Producer / VIA host | candidate ID·bbox/필요 polygon·관측 시각·잠정 track·공간 관계·출처·coverage·revision. raw crop은 보존 원본의 참조. full tensor를 이 기록에 무조건 복제하지 않음 |
| **Model Sessions** | Model Access / 해당 worker RAM | 로딩된 weights·모델 객체·사용 중 참조. R1은 필요 stage 중심으로 교체, R2/R3는 최초 사용 후 유지. 모델 보존과 episode 보존은 별개 |
| **Episode Tensor State** | Model Access의 실행 상태 / 해당 worker RAM | frame·embedding·mask·tracking memory와 source/model 판본. CloseEpisode로 별도 종료. R1/R2/R3의 보존 조건은 동일 |
| **Job Queue & Worker Generation** | Model Access / host 제어 상태와 worker 큐 | job·request·revision·deadline·cancel·generation 연결. R3의 두 큐도 하나의 전체 시각 CPU 예산을 적용 |
| **Shared Frame Buffer** | 제공 측의 buffer 소유권 / IPC 공유 메모리 | handle로 전달하고 읽기 완료 acknowledgement 뒤 회수. 소비자가 사용 중인 자료를 먼저 해제하지 않음 |
| **Model Files & Manifest** | 설치·모델 배포 자료 / local disk | UI detector·OCR·SAM weights와 model/adapter 판본·hash. 요청마다 다운로드하지 않으며 runtime RAM 원통과 구별 |

후보 기록이 host에 남아 있어도 worker crash로 사라진 episode tensor가 남아 있는 것으로 취급하지 않는다. 반대로 worker crash만으로 이미 생성되어 유효한 후보 기록을 정답 오류로 바꾸지 않는다. 추가 추론이 필요한 경우 원본·seed·판본을 확인해 tensor를 재생산한다. 필요한 원본이나 권한이 없으면 성공으로 가장하지 않는다.

## 3. Component 사이의 요청·반환 계약

아래 이름은 그림과 설명을 연결하는 설계 수준의 메시지다. SDK·Rust crate 또는 구현된 API를 뜻하지 않는다. 각 명령과 그 결과의 방향을 별도 화살표로 그린다.

| 경로 | 요청 또는 사건 | 반환·상태 및 검사 |
| --- | --- | --- |
| Interaction Manager → Request Controller | InputFinal(input_revision, transcript_refs, evidence_range_refs) | 화면 필요 여부를 판단한 현재 요청으로 이어짐. 입력 보존은 이 판단을 기다리지 않음 |
| Request Controller → Request Interpreter | Interpret(request_id, input_revision, permitted_evidence) | SemanticProposal 또는 EvidenceNeed. A/B 모두 최종 채택 전 제안 |
| Request Interpreter → Request Controller → Context Manager | ReadEvidence(time_range/frame/crop/native/previous_binding) | 허가된 원본·read receipt·gap·source revision을 Request Controller 경유로 해석 실행에 전달 |
| Request Controller → Screen Target Producer | ProduceTargets 또는 ExpandTargets(input_revision, source_refs, requested_extent, capability) | SceneEpisodeRevision 또는 coverage/실패. 의미 해석 실행이 낸 확장 제안도 현재 허가를 확인한 뒤 수행 |
| Screen Target Producer → Request Controller → Context Manager | 생산·확장에 부족한 역사 범위·원본 crop 조회 | producer가 원본 owner 저장소를 임의로 우회하지 않음. 이미 허가해 전달한 동일 원본은 재사용 가능 |
| Screen Target Producer → Model Access | RunUI / RunSegment / RunTrack(job_id, frame_refs, transforms, seed_or_prompt, revisions) | bbox/OCR/mask/track 관측, uncertainty, model version, worker generation, error. 결과만으로 최종 지칭을 채택하지 않음 |
| Model Access host 부분 ↔ worker 부분 | EnsureModel / RunStage / ReleaseModel / CloseEpisode / Cancel / Shutdown | named pipe로 명령·완료·오류, 공유 버퍼로 frame. load 완료·추론 완료·실제 RAM 회수는 서로 다른 관측 |
| Screen Target Producer → Request Controller → Request Interpreter | Target & Interaction Records + backing crops | Multimodal Binding Resolver가 읽은 후보 revision과 source read set을 BindingProposal에 기록 |
| Request Interpreter ↔ Model Access ↔ cloud 의미 모델 | A: 전사+당시 원본 / B: 같은 전사+후보 기록+원본 crop | 의미 제안 또는 보완 필요. 원음 재전송을 B의 경로로 추가하지 않음 |
| Request Interpreter → Request Controller | SemanticProposal / BindingProposal | input revision·source 권한·후보 판본·schema·의존을 검사하여 채택 또는 재해석. 옛 generation의 실행 결과가 신규 commit을 덮지 않음 |
| Request Controller → Response Manager → Interaction Manager | 채택한 구체적 응답과 게시 허가 | 실제 UI 표시 기록. cloud LLM 또는 Screen Target Producer가 직접 표시하지 않음 |

원본 범위의 추가 확보는 Request Controller가 허가하고 Context Manager가 읽는다. 발표의 Request Interpreter 또는 Screen Target Producer에서 Context Manager로 직접 연결한 Read/Crop 선은 **이미 허가되어 전달된 read set 범위의 소비**만 뜻한다. Request Controller의 조회·생산 허가 선과 새 범위의 재허가 루프를 함께 표시해 무제한 원본 읽기로 해석되지 않게 한다. 공통 제어를 반복 표시하면 같은 Component의 단계별 표시라는 범례를 둔다.

## 4. 같은 사건을 처리하는 네 구성

아래 번호는 화면 필요 판단을 마친 요청의 처리 순서다. 발화 중 capture·ASR의 배경 수집은 별도 흐름이며 speculative execution이 아니다.

### A: 원본 직접 해석

1. Request Controller가 현재 입력과 허가된 근거로 Request Interpreter의 Joint Grounding Interpreter를 시작한다.
2. Context Manager가 공급한 당시 frame·pointer·selection·native와 같은 전사·단어 시각을 Model Access의 cloud 경로로 전달한다.
3. 근거가 부족하면 해석기가 필요한 범위를 제안하고 Request Controller의 허가를 거쳐 Context Manager가 원본을 보완한다. 한 의미 세션을 한 번의 LLM 호출로 제한하지 않는다.
4. SemanticProposal을 Request Controller가 검증·채택하고 공통 응답 경로로 전달한다. 원본의 경계·동일성·집합을 제안하는 실행이 cloud 의미 해석에 포함된다.

### B 공통: 후보 생산과 지칭 결합

1. Request Controller가 화면 대상 생산을 허가한다. Screen Target Producer는 이미 확보한 source 구간과 실제 native coverage를 확인한다.
2. native 선택·범위가 충분하면 후보로 등록하고 시각 모델을 생략한다. 시각 처리가 필요하면 Model Access에 UI 작업을 요청한다.
3. UI 결과의 bbox로 충분하면 SAM을 생략한다. 정밀 경계 또는 시간별 대응이 필요하면 실제 UI/native seed와 원본으로 SAM 작업을 요청한다. UI 결과에 의존하는 SAM 작업을 UI와 동시에 시작하지 않는다.
4. Screen Target Producer가 기하 관계와 출처·불확실성을 포함한 후보 revision을 만든다. Request Interpreter의 Multimodal Binding Resolver가 같은 발화와 후보·crop으로 BindingProposal을 만든다.
5. 후보가 부족하면 현재 요청 허가 아래 생산·확장·재결합한다. Binder가 원본을 보고 생산 기록에 없는 최종 영역을 직접 확정하는 경로는 이번 B에 추가하지 않는다.
6. Request Controller가 읽은 input/source/candidate 판본과 권한을 확인해 채택하고 공통 응답 경로로 전달한다.

### B-R1 / B-R2 / B-R3의 실행 배치

| 실행 관점 | B-R1 | B-R2 | B-R3 |
| --- | --- | --- | --- |
| 논리 Component | 같은 Screen Target Producer와 Model Access | R1과 같음 | R1/R2와 같음. Model Access의 배치 부분이 분리됨 |
| 실제 process | VIA host + 시각 worker 1개 | VIA host + 시각 worker 1개 | VIA host + UI worker + SAM worker |
| Model Access의 worker 부분 | 하나의 큐·runtime, UI/SAM adapter | 같은 큐·runtime·adapter | worker마다 IPC endpoint·큐·adapter·runtime. host가 전체 admission을 제어 |
| 모델 수명 | 필요한 모델을 load하고 stage/요청 조건에 따라 회수·교체. runtime은 유지 | 최초 필요 시 load, 이후 사용한 모델 session을 유지 | 각 worker에서 해당 모델 session을 최초 사용 후 유지 |
| 실제 CPU 실행 | 한 번에 heavy visual stage 하나 | 같음. 두 모델 상주가 동시 추론을 뜻하지 않음 | 같음. 독립 process가 자동 병렬 허가를 뜻하지 않음 |
| episode | model unload와 별개로 유효 상태 유지·복원 | 같은 보존 조건 | 같은 보존 조건, 해당 worker가 tensor 소유 |
| worker crash 영향 | 해당 worker의 두 adapter/session과 tensor 상태 손실 | 같은 process에 있던 두 모델 session과 tensor 상태 손실 | 실패한 worker의 session/tensor 손실. 다른 worker의 주소 공간·상주 session은 유지 |
| 재시작 | Model Access가 generation 갱신·원본/seed 재실행을 관리 | 같음 | 실패한 worker만 재시작. 그 결과가 필요한 의존 작업은 계속 영향을 받음 |

R1/R2를 다른 Component 목록으로 그리지 않는다. 차이는 같은 실행 제어기의 모델 수명 정책과 그에 따른 load·release 전제다. R3는 실제 process·queue·runtime·장애 경계가 추가된다. 세 안의 공통 episode 보존 정책을 달리 그려 R2/R3에 별도 정확도나 재사용 이익을 부여하지 않는다.

`CloseEpisode`와 `ReleaseModel`을 다른 명령으로 표시한다. R1의 유효 episode를 재사용하려면 source/model/전처리 판본 호환성을 확인한다. 모델 객체를 참조한 tensor 때문에 실제 메모리가 회수되지 않는 경우도 관측 대상이며, `ReleaseModel` 반환만으로 QA06 절감을 보장하지 않는다.

## 5. 발표 그림의 표기와 페이지 역할

[그림 작성 기준](../../diagram-design-guide.md)과 [INTENT §12](../../../../../../INTENT.md)를 적용한다. DP1·DP2·DP4의 공통 문법은 굵은 제목이 있는 Component 경계, 소유 내부 처리, 원통 상태, 읽기·갱신·반환 관계다. 기존 그림의 오래된 약칭이나 동작 문장까지 무조건 복사하지 않는다.

검토한 사례는 통합 발표의 **DP1 비교 p2, DP2 비교 p6, DP4 비교 p17**이다. [기존 발표 체크리스트](../../../../../presentations_files/dp1-dp4-reference-style-20261019/reference-checklist-and-review.md)의 R12~14는 Component·내부 처리·상태 계층, 실행·중간 상태 차이, 공통 검증과 소유권 보존을 요구한다. 기존 DP2의 논리 실행 경계와 이번 DP3의 OS process 경계를 같은 의미로 읽지 않는다.

| 도형·표기 | DP3 적용 |
| --- | --- |
| 제목 칸이 있는 옅은 노란 Component 상자 | Request Controller, Context Manager, Request Interpreter, Screen Target Producer, Model Access. 상자에는 이름, 책임의 동작·조건은 연결 라벨 |
| Component 안의 둥근 내부 Module | Joint Grounding Interpreter, Multimodal Binding Resolver, Candidate Assembler, Request Revision Guard, Cloud Model Client, Local Model Execution Controller, IPC Endpoint, UI Adapter, SAM Adapter. 이름만으로 별도 process를 주장하지 않음 |
| 이름이 있는 점선 process 경계 | VIA Host Process / Vision Worker Process / UI Worker Process / SAM Worker Process. 실제 주소 공간과 장애 범위를 표시 |
| 원통 | 원본 버퍼, host 후보 기록, worker model session, worker episode tensor. RAM/disk와 소유자를 구별 |
| 접힌 종이 또는 자료 표기 | 직렬화한 candidate package, manifest, frame handle처럼 전달되는 자료. 실행 주체처럼 그리지 않음 |
| 육각형 연동 대상 | cloud 의미 모델 및 외부 의존. 모델 연동 코드 Model Access와 구분 |
| 실선·점선 화살표 | 실선은 요청·전달, 점선은 결과·복원으로 통일하고 범례 명시. 경로 옆에 데이터·동작·조건·순서 표시 |
| 색 | 이번 발표는 DP1·DP2·DP4의 연녹색 대안 표 머리와 옅은 노란 Component 표면을 유지한다. 구조와 일반 요청·반환은 검정, OS process 경계는 파란 점선이다. 파랑은 최종 채택 위치·공유 버퍼 경로 등 설명 강조에도 쓰며 범례와 라벨로 구별한다. A/B 식별이나 QA 우위는 색으로 추정하지 않고 대안 이름과 구조로 읽는다. 기존 문서의 A 파랑/B 초록 비교색 범례를 이번 발표에 소급하지 않는다 |

### 그림 10: A와 B-R1의 지칭 생산 책임

추천 위치: 현재 발표 10쪽. 좌측 A, 우측 B-R1의 두 열을 유지한다. 입력 시작점, 원본 조회 권한, 대상 생산 책임, 추가 원본/후보 확장 루프, cloud 연동, 최종 proposal 반환·채택을 각 열에서 추적할 수 있어야 한다. R1의 worker가 필요 모델을 교대한다는 실제 경계와 모델/episode의 분리를 보여준다. 모든 공통 수집 Components를 크게 반복하느라 핵심 경로를 생략하지 말고, 공통 입력 띠에 Interaction Manager의 보존 책임과 ASR를 명시한다.

[현재 그림 SVG](diagrams/dp3-v7-ab-responsibilities.svg) / [draw.io 원본](diagrams/dp3-v7-ab-responsibilities.drawio) / [PPTX 렌더](diagrams/dp3-v7-ab-responsibilities.png). 동일 scene 좌표에서 생성했으며 원본 소유권·반환 경로와 process 의미를 일치시켰다.

### 그림 11: B-R2와 B-R3의 배치·자원·장애 경계

추천 위치: 현재 발표 11쪽. host의 생산기·후보 기록·지칭 결합은 동일하게 정렬한다. Model Access의 host 부분과 worker 부분을 명시한다. R2의 하나의 process 안에 두 session을, R3의 두 process 안에 해당 adapter/runtime/session/episode를 펼친다. UI 결과가 host로 돌아온 뒤 필요한 SAM 작업을 발행하는 선으로 실제 의존성을 표현한다. UI/SAM으로 동시에 분기하고 합치는 모양을 사용하지 않는다.

[현재 그림 SVG](diagrams/dp3-v7-resident-processes.svg) / [draw.io 원본](diagrams/dp3-v7-resident-processes.drawio) / [PPTX 렌더](diagrams/dp3-v7-resident-processes.png). 동일 scene 좌표에서 생성했으며 원본 소유권·반환 경로와 process 의미를 일치시켰다.

### 그림 12: 실행 프로그램 내부와 상태 수명

추천 위치: 현재 발표 12쪽의 실행 구조 영역. VIA host에서 bundled Python worker를 기동하고 named pipe로 명령·완료를 주고받는 관계, worker 내부의 IPC endpoint·큐·adapter·PyTorch runtime, model session과 episode tensor를 펼친다. 디스크의 model files/manifest에서 load하는 경로를 별도로 둔다. model session을 worker 뒤에 놓인 독립 실행 프로그램처럼 그리지 않는다.

[현재 그림 SVG](diagrams/dp3-v7-runtime-lifetimes.svg) / [draw.io 원본](diagrams/dp3-v7-runtime-lifetimes.drawio) / [PPTX 렌더](diagrams/dp3-v7-runtime-lifetimes.png). 동일 scene 좌표에서 생성했으며 원본 소유권·반환 경로와 process 의미를 일치시켰다.

## 6. 재작성 검수 기준과 수정 범위

1. 그림만 보고 각 Component의 이름과 Module 소유자를 찾을 수 있다. 논리 Component와 OS process 경계를 구별할 수 있다.
2. 최초 실행 허가, 부족한 원본·후보의 추가 요청, 결과 반환과 최종 채택이 Request Controller의 계약으로 연결된다. cloud 모델의 출력이 직접 최종 상태가 되지 않는다.
3. A의 추가 읽기·crop·native·유효 이력 재사용과 B의 후보 확장·원본 보완을 모두 보존한다.
4. host의 후보 기록과 worker의 model session·episode tensor가 별도 상태로 표시된다. R1/R2/R3의 같은 episode 정책과 서로 다른 모델 수명이 구별된다.
5. B-R3의 UI seed→SAM 의존, 동일 총 CPU 예산, 순차 heavy stage를 화살표와 경계가 그대로 표현한다. 따로 적힌 주석으로 반대 모양의 선을 해명하지 않는다.
6. 최초 사용의 load, 이후 재사용, 긴 유휴·memory pressure 회수, worker crash 재시작을 설명할 수 있다. 모델 상주를 항상 사전 로딩된 상태로 그리지 않는다.
7. 오류·취소·옛 revision/generation 결과는 정상 채택으로 연결하지 않는다. 모델 반환과 실제 UI 표시를 다른 사건으로 둔다.
8. 문자열 넘침·선/글자 겹침·상자 관통·화살표 출발/도착을 실제 렌더에서 검토한다. native shape 수나 자동 레이아웃 검사만으로 구조적 검수를 대신하지 않는다.
9. QA01~07 수치·데이터셋·기존 DP1/DP2/DP4/DP5 및 appendix의 역사 DP3는 이번 책임 표기 수정으로 바꾸지 않는다.

이번 문서는 발표용 책임 매핑과 그림 검수 계약이다. 코드 구현·모델 설치·실행·Windows 제품 패키징·실제 메모리 회수·성능 검증을 수행했다는 근거로 사용하지 않는다.
