# DP3 발표 그림의 Component, 상태, 실행 계약

2026-10-22 / 발표 v8의 그림과 본문 계약 / 설계 구체화, 미구현 및 미측정

**A, B-R1, B-R2, B-R3의 기능과 평가 조건은 그대로 두고, 누가 후보를 만들고 누가 모델을 실행하며 누가 최종 의미를 채택하는지를 그림에서 구별한다.** 이 문서는 [CPU runtime 설계](cpu-runtime-architecture.md) §4의 책임을 논리 Component와 내부 Module, 실제 process에 대응시킨다. Reference Architecture나 accepted ADR를 변경하지 않으며 새로운 모델, 기본안, 병렬화 또는 QA 개선을 추가하지 않는다.

[현재 네 대안](cpu-runtime-architecture.md#11-a와-b-r1b-r2b-r3의-네-대안-비교), [로컬 생산기의 입력, 출력, 확장 계약](local-producer.md), [동일 시나리오 합성 평가](synthetic-comparison/README.md)가 내용의 기준이다. `DP3-GROUNDING-CPU-SYN-1.0` / `VOICE-SCREEN-CPU-SYN-1.0`의 데이터셋, 시나리오, 분모, 가정 수치와 QA01~07 값은 이번 그림 수정에서 동결한다. 실측으로 재분류하지 않는다.

## 1. 책임 매핑의 판정

`Screen Target Producer`(화면 대상 생산기)를 B의 시각 후보 생산 Component로, 실행 제어, adapter, session을 `Model Access` 내부 책임으로 나누는 것은 타당하다. 후보의 경계, 관계, 판본을 생산하는 일과 모델의 load, 실행, 회수를 관리하는 일은 다른 계약이다. 두 책임은 이미 CPU runtime 설계에 있으며 이번에 소유자를 명확히 표시한다.

[Reference Architecture의 Component 계약](../../../target-architecture/architecture.md#4-component와-상태-소유권)은 `Model Access`를 논리적인 연동, 자원 관리 책임으로 정의하고 client, scheduler, weight owner, ASR adapter가 서로 다른 process에 배치될 수 있음을 명시한다. 따라서 B-R3에서 host client와 두 시각 worker의 실행 부분을 **동일한 논리 Component인 Model Access의 분산 배치**로 그리는 것은 그 의미와 맞는다. UI/SAM worker가 별도 process라는 사실을 새 최상위 Component 두 개로 바꾸지 않는다.

이번 비교는 사용자 지정에 따라 cloud S2S, cloud 최종 의미 모델, local ASR, local CPU 시각 모델을 사용한다. 참조 구조의 on-device Omni 배치와 이번 비교의 배치를 혼합하지 않는다. `Screen Target Producer`와 아래 내부 Module 명칭은 이번 비교를 설명하는 설계 이름이며, 참조 구조의 정식 Component 목록이나 실제 Rust 모듈 경로로 소급하지 않는다.

| Component의 그림 표기 | 내부 Module 또는 처리의 그림 표기 | 소유 책임 | 소유하지 않는 판단 |
| --- | --- | --- | --- |
| **Interaction Manager** | Evidence Capture / Timeline & Buffer | frame, pointer, selection, window, scroll, 입력 시각, revision, gap과 유한 원본 버퍼. Model Access에서 받은 ASR 전사와 S2S 정제 텍스트의 연결 | 지칭 의도, 최종 요청 의미, 후보 정답 여부 |
| **Request Controller** | 요청 검증 및 채택기 | request/input revision, 실행 attempt, 허용 범위, 최종 의미 commit. 새 범위의 원본 조회와 생산, 확장 요청을 허가. 요청 검증 및 채택기는 현재 입력과 접근 허가를 확인하고 반환의 현재성 및 의존을 검증하는 내부 처리 | 검출, 분할, 추적 연산, 의미 모델의 내부 추론 |
| **Context Manager** | 근거 조회기 | 허가된 범위의 history/frame/crop/native/이전 binding 조회, read receipt, coverage, source revision | 원본 기록의 의미상 소유권, 모델 실행 수명, 최종 지칭 채택 |
| **Request Interpreter** — A | 원본 해석기 | 같은 의미 해석 세션에서 원본을 읽어 대상, 집합, 관계와 요청 의미를 제안. 원본 보완 요청과 잠정 해석 상태 | 요청 commit, 사용자 게시, 후보를 정답으로 강제하는 권한 |
| **Request Interpreter** — B | 후보 결합기 | 같은 발화와 후보 판본, 원본 crop을 연결하여 BindingProposal을 생성. 필요한 후보 확장, 분할, 재추적을 제안 | worker queue, weight 수명, 생산 기록을 우회한 최종 영역 확정 |
| **Screen Target Producer** — B | 후보 구성기 | 허가된 원본에서 native 후보 등록, 시각 모델 작업 구성, 공간 관계 계산, SceneEpisodeRevision의 생성, 변경, coverage. 파생 후보의 단일 writer. 기존 생산 제어와 공간 관계 계산을 이 내부 Module의 책임으로 표현 | 사용자의 최종 의도, 요청 commit, CPU 실행 자원의 직접 소유 |
| **Model Access** — 공통 cloud 경로 | 클라우드 호출기 | 동일 S2S, 의미 모델 연동, 호출, 응답, 사용량, 오류 계약. 발표 확대도는 cloud 의미 모델 client를 표시 | 모델의 제안을 최종 의미로 채택하는 권한 |
| **Model Access** — B local 경로 | 로컬 실행 제어기 / IPC 수신기 / UI 어댑터 / SAM 어댑터 / CPU 추론 런타임 | worker 기동, 건강, generation, 전체 시각 CPU admission, queue, load, release, cancel, 전후처리, model session과 tensor episode의 실행 수명 | 후보 기록의 의미상 권위, 어떤 후보를 사용자가 뜻했는지의 최종 판단 |
| **Response Manager** / **Interaction Manager** | 기존 응답 준비, 실제 표시 처리 | Request Controller가 허가한 내용의 준비, 전달과 실제 표시 기록 | 모델 또는 worker 반환만으로 임의 게시 |

### 1.1 그림 이름과 이전 기술 이름의 대응

Component의 정식 이름은 유지한다. 내부 Module은 읽기 쉬운 이름으로 바꾸며 책임을 추가하거나 새 process를 뜻하지 않는다. 이전 계약, 메시지명과 구현 수준 논의에는 오른쪽 기술 이름을 함께 사용할 수 있다.

| v8 그림 이름 | v7 및 상세 설계의 기술 이름 | 이름의 의미 |
| --- | --- | --- |
| 요청 검증 및 채택기 | Request Revision Guard 및 기존 조회/생산 허가와 최종 채택 처리 | 입력 판본, 접근 범위와 반환 의존을 확인하고 해석의 실행 및 채택을 통제 |
| 근거 조회기 | Evidence Reader | 허가된 이력, crop, native 자료와 이전 채택 결과를 읽음 |
| 원본 해석기 | Joint Grounding Interpreter | A의 원본 기반 지칭과 요청 의미 제안 |
| 후보 결합기 | Multimodal Binding Resolver | B의 후보 판본과 발화를 연결하고 필요 확장을 제안 |
| 후보 구성기 | Candidate Assembler | 후보 생산 작업과 기하 관계를 구성하고 후보 기록 갱신 |
| 클라우드 호출기 | Cloud Model Client | cloud 의미 모델의 요청과 반환 및 사용량 관리 |
| 로컬 실행 제어기 | Local Model Execution Controller | worker 수명, CPU 실행권, 작업 취소와 실행 세대 관리 |
| IPC 수신기 | IPC Endpoint | worker 명령 수신과 결과 및 오류 반환 |
| UI 어댑터 / SAM 어댑터 | UI Adapter / SAM Adapter | 모델별 전후처리와 CPU 추론 연동 |
| CPU 추론 런타임 | PyTorch Runtime | 로딩된 모델 객체를 실행하는 공통 엔진 |
| CPU 시각 실행기 | UI Adapter, SAM Adapter 및 PyTorch Runtime의 p10 묶음 표기 | p10에서 세 내부 처리의 실행 관계를 요약하고 p12에서 펼침. 새 책임이나 별도 실행 파일이 아님 |

`receipt`는 읽은 자료와 판본의 기록, `rev`는 해당 자료나 입력의 판본, `generation`은 worker가 재시작되어 바뀌는 실행 세대다. 그림의 쉬운 표현을 바꾸면서 원본 판본, 입력 판본과 worker 실행 세대를 같은 값으로 합치지 않는다.

`Screen Target Producer`의 후보 권위는 **파생 후보 기록을 누가 갱신하는가**라는 뜻이다. 원본의 사실 권위나 사용자가 가리킨 정답의 권위를 갖는다는 뜻이 아니다. worker가 반환한 bbox, mask, track은 후보 생산의 재료이며, 기록 판본과 좌표, 출처, coverage를 갖춰 게시하는 주체는 Screen Target Producer다.

## 2. 상태의 소유자와 실제 위치

작은 후보 기록과 큰 영상 tensor를 한 원통에 합치지 않는다. 특히 후보 기록이 host에 남는 것과 worker의 embedding이 살아 있는 것은 다른 사건이다.

| 그림에 표시할 자료, 상태 | 의미상 소유자 / 실제 위치 | 수명, 회수 및 장애 계약 |
| --- | --- | --- |
| **Input Evidence Buffer** | Interaction Manager / VIA host | 공통 원본 보존 기간. 모델 unload와 별개. frame 시각, 좌표 변환, gap을 유지. Context Manager 내부의 원본 참조/cache는 이 버퍼의 소유권을 이전한 것이 아님 |
| **Request Revision & Read Set** | Request Controller / VIA host | 현재 입력 판본과 실제 읽은 원본, 후보 판본. 옛 입력 결과의 채택을 차단 |
| **Interpretation State** | Request Interpreter / 해당 해석 실행 | A의 잠정 직접 해석 또는 B의 잠정 binding. A도 유효한 앞선 채택 결과를 재사용 가능. Context Manager가 반환하는 확정 binding은 Request Controller가 채택한 기록의 읽기용 사본/참조이며 최종 의미 writer가 바뀌지 않음 |
| **Target & Interaction Records** | Screen Target Producer / VIA host | candidate ID, bbox/필요 polygon, 관측 시각, 잠정 track, 공간 관계, 출처, coverage, revision. raw crop은 보존 원본의 참조. full tensor를 이 기록에 무조건 복제하지 않음 |
| **Model Sessions** | Model Access / 해당 worker RAM | 로딩된 weights, 모델 객체, 사용 중 참조. R1은 필요 stage 중심으로 교체, R2/R3는 최초 사용 후 유지. 모델 보존과 episode 보존은 별개 |
| **Episode Tensor State** | Model Access의 실행 상태 / 해당 worker RAM | frame, embedding, mask, tracking memory와 source/model 판본. CloseEpisode로 별도 종료. R1/R2/R3의 보존 조건은 동일 |
| **Job Queue & Worker Generation** | Model Access / host 제어 상태와 worker 큐 | job, request, revision, deadline, cancel, generation 연결. R3의 두 큐도 하나의 전체 시각 CPU 예산을 적용 |
| **Shared Frame Buffer** | 제공 측의 buffer 소유권 / IPC 공유 메모리 | handle로 전달하고 읽기 완료 acknowledgement 뒤 회수. 소비자가 사용 중인 자료를 먼저 해제하지 않음 |
| **Model Files & Manifest** | 설치, 모델 배포 자료 / local disk | UI detector, OCR, SAM weights와 model/adapter 판본, hash. 요청마다 다운로드하지 않으며 runtime RAM 원통과 구별 |

후보 기록이 host에 남아 있어도 worker crash로 사라진 episode tensor가 남아 있는 것으로 취급하지 않는다. 반대로 worker crash만으로 이미 생성되어 유효한 후보 기록을 정답 오류로 바꾸지 않는다. 추가 추론이 필요한 경우 원본, seed, 판본을 확인해 tensor를 재생산한다. 필요한 원본이나 권한이 없으면 성공으로 가장하지 않는다.

## 3. Component 사이의 요청, 반환 계약

아래 이름은 그림과 설명을 연결하는 설계 수준의 메시지다. SDK, Rust crate 또는 구현된 API를 뜻하지 않는다. 각 명령과 그 결과의 방향을 별도 화살표로 그린다.

| 경로 | 요청 또는 사건 | 반환, 상태 및 검사 |
| --- | --- | --- |
| Interaction Manager → Request Controller | InputFinal(input_revision, transcript_refs, evidence_range_refs) | 화면 필요 여부를 판단한 현재 요청으로 이어짐. 입력 보존은 이 판단을 기다리지 않음 |
| Request Controller → Request Interpreter | Interpret(request_id, input_revision, permitted_evidence) | SemanticProposal 또는 EvidenceNeed. A/B 모두 최종 채택 전 제안 |
| Request Interpreter → Request Controller → Context Manager | ReadEvidence(time_range/frame/crop/native/previous_binding) | 허가된 원본, read receipt, gap, source revision을 Request Controller 경유로 해석 실행에 전달 |
| Request Controller → Screen Target Producer | ProduceTargets 또는 ExpandTargets(input_revision, source_refs, requested_extent, capability) | SceneEpisodeRevision 또는 coverage/실패. 의미 해석 실행이 낸 확장 제안도 현재 허가를 확인한 뒤 수행 |
| Screen Target Producer → Request Controller → Context Manager | 생산, 확장에 부족한 역사 범위, 원본 crop 조회 | producer가 원본 owner 저장소를 임의로 우회하지 않음. 이미 허가해 전달한 동일 원본은 재사용 가능 |
| Screen Target Producer → Model Access | RunUI / RunSegment / RunTrack(job_id, frame_refs, transforms, seed_or_prompt, revisions) | bbox/OCR/mask/track 관측, uncertainty, model version, worker generation, error. 결과만으로 최종 지칭을 채택하지 않음 |
| Model Access host 부분 ↔ worker 부분 | EnsureModel / RunStage / ReleaseModel / CloseEpisode / Cancel / Shutdown | named pipe로 명령, 완료, 오류, 공유 버퍼로 frame. load 완료, 추론 완료, 실제 RAM 회수는 서로 다른 관측 |
| Screen Target Producer → Request Controller → Request Interpreter | Target & Interaction Records + backing crops | 후보 결합기가 읽은 후보 revision과 source read set을 BindingProposal에 기록 |
| Request Interpreter ↔ Model Access ↔ cloud 의미 모델 | A: 전사+당시 원본 / B: 같은 전사+후보 기록+원본 crop | 의미 제안 또는 보완 필요. 원음 재전송을 B의 경로로 추가하지 않음 |
| Request Interpreter → Request Controller | SemanticProposal / BindingProposal | input revision, source 권한, 후보 판본, schema, 의존을 검사하여 채택 또는 재해석. 옛 generation의 실행 결과가 신규 commit을 덮지 않음 |
| Request Controller → Response Manager → Interaction Manager | 채택한 구체적 응답과 게시 허가 | 실제 UI 표시 기록. cloud LLM 또는 Screen Target Producer가 직접 표시하지 않음 |

**새로운 원본 읽기는 Request Controller가 허가하고 Context Manager가 수행한다.** Request Interpreter 또는 Screen Target Producer에서 Context Manager로 바로 연결되는 조회선을 두지 않는다. 이전 v7의 직접 Read/Crop 선을 각주로 제한하던 설명은 폐기한다. 이미 전달받은 원본을 자신의 실행 상태에서 다시 사용하는 것은 새 원본 조회가 아니며 별도 권한 우회를 뜻하지 않는다.

A의 `EvidenceNeed`는 Request Interpreter → Request Controller → Context Manager로 전달된다. 조회 결과는 Context Manager → Request Controller → Request Interpreter로 돌아간다. B의 `ExpandNeed`는 Request Interpreter → Request Controller로 전달되고, Request Controller가 필요 원본을 Context Manager에서 확보한 뒤 Screen Target Producer에 생산 범위와 함께 전달한다. 생산 중 원본이 부족한 경우에도 Screen Target Producer → Request Controller → Context Manager의 같은 허가 경로를 이용한다. 새 후보 판본은 Screen Target Producer → Request Controller → Request Interpreter로 반환된다.

여기서 재허가는 Request Controller가 현재 요청 판본과 원본 접근 범위를 다시 검사한다는 뜻이다. 모든 보완 조회마다 사용자 승인 창을 띄우는 별도 정책을 추가하지 않는다. 기존 허용 범위를 넘는 경우에만 공통 접근 정책의 확인 절차를 따른다.

허가가 거절되거나 원본이 이미 회수되어 확보할 수 없으면 실패 사유와 부족 범위를 호출자에게 반환한다. 빈 원본이나 빈 후보를 충분한 근거처럼 처리하지 않고, 최종 의미 채택을 보류한 뒤 공통 확인 응답 경로를 사용한다. 이 분기는 성공 경로로 합치지 않는다.

공통 제어를 반복 표시하면 같은 Component의 단계별 표시임을 범례에 명시한다. IPC도 논리 호출의 양끝이 되는 내부 Module까지 연결한다. 요청은 host의 로컬 실행 제어기에서 worker의 IPC 수신기로 전달되며, worker → host의 완료와 오류는 독립된 반환 경로다. process 경계에서 선을 멈추고 각주로 실제 수신자를 지정하지 않는다.

## 4. 같은 사건을 처리하는 네 구성

화면 필요 판단을 마친 요청을 다룬다. 발화 중 수집과 ASR의 배경 실행은 요청 해석의 사전 실행이 아니며 아래 순서에 직렬 단계로 더하지 않는다. 그림 10의 정수 번호는 최초 요청의 실행 순서를 따른다. 유효한 원본이나 후보가 이미 있으면 해당 생산 단계를 생략하고, 조건부 보완에서는 표시한 경로로 돌아간다.

### A: 원본 직접 해석

| 그림 10 번호 | 실제 메시지 경로와 조건 |
| --- | --- |
| 1 | Request Controller의 요청 검증 및 채택기가 현재 입력과 접근 범위를 확인하고 Context Manager의 근거 조회기에 원본 읽기를 허가한다. |
| 2 | 근거 조회기가 원본, 읽은 자료의 판본과 누락 범위를 요청 검증 및 채택기로 반환한다. 원본을 확보할 수 없으면 해당 부족 사유를 보존한다. |
| 3 | 요청 검증 및 채택기가 현재 입력과 확보한 원본을 Request Interpreter의 원본 해석기에 전달하여 시작하거나 재개한다. |
| 4 | 원본 해석기가 Model Access의 클라우드 호출기를 통해 전사, 단어 시각, 당시 화면과 조작 이력으로 의미 모델을 호출한다. |
| 5 | 모델 결과가 클라우드 호출기를 거쳐 원본 해석기로 반환된다. 결과는 채택된 의미가 아니다. |
| 6 | 원본 해석기가 의미 제안 또는 부족 원본의 범위 `EvidenceNeed`를 요청 검증 및 채택기로 반환한다. 의미 제안이면 현재 입력과 읽은 원본 판본 및 권한을 확인해 채택한다. 원본이 부족하면 1번으로 돌아가 조회를 다시 허가한다. |

최초 실행은 **1 → 2 → 3 → 4 → 5 → 6**, 원본 보완은 **6 → 1 → 2 → 3** 이후 해석 재개다. 이미 유효한 원본이 전달되어 있으면 1과 2를 생략할 수 있다. 원본 확보가 불가능하면 부족 사유를 반환하고 공통 확인 응답으로 연결한다. 원본의 경계, 동일성과 집합을 생산하는 책임은 계속 같은 의미 해석 세션에 있으며, 한 세션을 한 번의 LLM 호출로 제한하지 않는다.

### B 공통: 후보 생산과 지칭 결합

| 그림 10 번호 | 실제 메시지 경로와 조건 |
| --- | --- |
| 1 | Request Controller의 요청 검증 및 채택기가 현재 입력과 권한을 확인해 Context Manager의 근거 조회기에 원본 읽기를 허가한다. |
| 2 | 근거 조회기가 원본, 판본과 미확보 범위를 요청 검증 및 채택기로 반환한다. 원본이 없거나 접근할 수 없으면 해당 부족 사유를 보존한다. |
| 3 | 요청 검증 및 채택기가 허가한 범위와 원본으로 Screen Target Producer의 후보 구성기에 최초 생산 또는 확장을 요청한다. native 정보가 충분하면 시각 모델을 생략한다. 부족하면 Model Access의 로컬 실행 제어기를 거쳐 UI를 실행하고, 그 결과 또는 native seed가 있어야 필요한 SAM을 실행한다. |
| 4 | 후보 구성기가 새 후보 판본 또는 생산에 부족한 원본 범위를 요청 검증 및 채택기로 반환한다. 후보는 5번으로 전달한다. 원본이 더 필요하면 1 → 2 → 3번 경로로 보완한다. |
| 5 | 요청 검증 및 채택기가 같은 발화와 확보한 후보 판본, 원본 crop을 Request Interpreter의 후보 결합기에 전달하여 시작하거나 재개한다. |
| 6 | 후보 결합기가 Model Access의 클라우드 호출기를 통해 의미 모델을 호출한다. |
| 7 | 의미 모델의 결과가 클라우드 호출기를 거쳐 후보 결합기로 반환된다. |
| 8 | 후보 결합기가 의미 제안 또는 필요한 후보 범위 `ExpandNeed`를 요청 검증 및 채택기로 반환한다. 의미 제안이면 입력, 원본과 후보 판본을 확인한 뒤 채택한다. 후보가 부족하면 3번으로 돌아가 확장을 허가하고, 새로운 원본이 필요하면 1번으로 돌아가 조회부터 허가한다. |

새 episode에 후보가 없는 최초 실행은 **1 → 2 → 3 → 4 → 5 → 6 → 7 → 8**이다. 유효한 원본이 이미 확보되어 있으면 1과 2를, 유효한 후보까지 있으면 3과 4도 재사용으로 생략할 수 있다. 후보 생산 필요를 확인하기 위해 의미 LLM을 한 번 더 호출하는 단계를 추가하지 않는다.

후보 확장은 **8 → 3 → 4 → 5**, 새 원본이 필요한 확장은 **8 → 1 → 2 → 3 → 4 → 5**로 재결합한다. 생산기 자체에 원본이 부족하면 **4 → 1 → 2 → 3**으로 보완한다. Binder가 생산 기록에 없는 최종 영역을 원본에서 직접 확정하는 우회 경로는 이번 B에 추가하지 않는다.

### B-R1 / B-R2 / B-R3의 실행 배치

| 실행 관점 | B-R1 | B-R2 | B-R3 |
| --- | --- | --- | --- |
| 논리 Component | 같은 Screen Target Producer와 Model Access | R1과 같음 | R1/R2와 같음. Model Access의 배치 부분이 분리됨 |
| 실제 process | VIA host + 시각 worker 1개 | VIA host + 시각 worker 1개 | VIA host + UI worker + SAM worker |
| Model Access의 worker 부분 | 하나의 큐, runtime, UI/SAM adapter | 같은 큐, runtime, adapter | worker마다 IPC endpoint, 큐, adapter, runtime. host가 전체 admission을 제어 |
| 모델 수명 | 필요한 모델을 load하고 stage/요청 조건에 따라 회수, 교체. runtime은 유지 | 최초 필요 시 load, 이후 사용한 모델 session을 유지 | 각 worker에서 해당 모델 session을 최초 사용 후 유지 |
| 실제 CPU 실행 | 한 번에 heavy visual stage 하나 | 같음. 두 모델 상주가 동시 추론을 뜻하지 않음 | 같음. 독립 process가 자동 병렬 허가를 뜻하지 않음 |
| episode | model unload와 별개로 유효 상태 유지, 복원 | 같은 보존 조건 | 같은 보존 조건, 해당 worker가 tensor 소유 |
| worker crash 영향 | 해당 worker의 두 adapter/session과 tensor 상태 손실 | 같은 process에 있던 두 모델 session과 tensor 상태 손실 | 실패한 worker의 session/tensor 손실. 다른 worker의 주소 공간, 상주 session은 유지 |
| 재시작 | Model Access가 generation 갱신, 원본/seed 재실행을 관리 | 같음 | 실패한 worker만 재시작. 그 결과가 필요한 의존 작업은 계속 영향을 받음 |

R1/R2를 다른 Component 목록으로 그리지 않는다. 차이는 같은 실행 제어기의 모델 수명 정책과 그에 따른 load, release 전제다. R3는 실제 process, queue, runtime, 장애 경계가 추가된다. 세 안의 공통 episode 보존 정책을 달리 그려 R2/R3에 별도 정확도나 재사용 이익을 부여하지 않는다.

`CloseEpisode`와 `ReleaseModel`을 다른 명령으로 표시한다. R1의 유효 episode를 재사용하려면 source/model/전처리 판본 호환성을 확인한다. 모델 객체를 참조한 tensor 때문에 실제 메모리가 회수되지 않는 경우도 관측 대상이며, `ReleaseModel` 반환만으로 QA06 절감을 보장하지 않는다.

## 5. 발표 그림의 표기와 페이지 역할

[그림 작성 기준](../../diagram-design-guide.md)과 [INTENT §12](../../../../../../INTENT.md)를 적용한다. DP1, DP2, DP4의 공통 문법은 굵은 제목이 있는 Component 경계, 소유 내부 처리, 원통 상태, 읽기, 갱신, 반환 관계다. 기존 그림의 오래된 약칭이나 동작 문장까지 무조건 복사하지 않는다.

검토한 사례는 통합 발표의 **DP1 비교 p2, DP2 비교 p6, DP4 비교 p17**이다. [기존 발표 체크리스트](../../../../../presentations_files/dp1-dp4-reference-style-20261019/reference-checklist-and-review.md)의 R12~14는 Component, 내부 처리, 상태 계층, 실행, 중간 상태 차이, 공통 검증과 소유권 보존을 요구한다. 기존 DP2의 논리 실행 경계와 이번 DP3의 OS process 경계를 같은 의미로 읽지 않는다.

| 도형, 표기 | DP3 적용 |
| --- | --- |
| 제목 칸이 있는 옅은 노란 Component 상자 | Request Controller, Context Manager, Request Interpreter, Screen Target Producer, Model Access. 상자에는 이름, 책임의 동작, 조건은 연결 라벨 |
| Component 안의 둥근 내부 Module | 원본 해석기, 후보 결합기, 후보 구성기, 요청 검증 및 채택기, 클라우드 호출기, 로컬 실행 제어기, IPC 수신기, UI 어댑터, SAM 어댑터, CPU 추론 런타임. 이름만으로 별도 process를 주장하지 않음 |
| 이름이 있는 점선 process 경계 | VIA Host Process / Vision Worker Process / UI Worker Process / SAM Worker Process. 실제 주소 공간과 장애 범위를 표시 |
| 원통 | 원본 버퍼, host 후보 기록, worker model session, worker episode tensor. RAM/disk와 소유자를 구별 |
| 접힌 종이 또는 자료 표기 | 직렬화한 candidate package, manifest, frame handle처럼 전달되는 자료. 실행 주체처럼 그리지 않음 |
| 육각형 연동 대상 | cloud 의미 모델 및 외부 의존. 모델 연동 코드 Model Access와 구분 |
| 실선, 점선 화살표 | 실선은 요청, 전달, 점선은 결과, 복원으로 통일하고 범례 명시. 경로 옆에 데이터, 동작, 조건, 순서 표시 |
| 색 | DP1, DP2, DP4의 연녹색 대안 표 머리와 옅은 노란 Component 표면은 유지한다. 공통 구성과 경로는 검정, A의 원본 해석 고유 Module과 경로는 파랑, B의 생산 및 local 실행 고유 Module과 경로는 초록이다. OS process 경계는 회색 점선으로 별도 구별한다. B 실행을 확대한 11~12쪽에서는 R 구성에 공통인 요소를 검정으로 표시하고 실제 process 경계를 회색으로 펼친다. 각 페이지 범례가 색의 적용 범위를 밝히며 색을 QA 우위 표시로 사용하지 않는다 |

### 그림 10: A와 B-R1의 지칭 생산 책임

발표 10쪽. 좌측 A, 우측 B-R1의 두 열을 유지한다. 입력 시작점, 원본 조회 권한, 대상 생산 책임, 추가 원본/후보 확장 루프, cloud 연동, 최종 proposal 반환, 채택을 각 열에서 추적할 수 있어야 한다. R1의 worker에는 IPC 수신기와 CPU 시각 실행기, 모델 및 추적 상태를 연결한다. CPU 시각 실행기는 §1.1의 adapter와 runtime을 묶은 표기이며 12쪽에서 실제 내부 구성과 적재 및 회수 경로를 펼친다. 모델 session과 episode 상태를 구별한다. 공통 입력 띠에는 같은 전사, 단어 시각, 당시 화면과 조작 이력을 표시한다. 수집과 보존의 소유 책임은 §1의 Interaction Manager 및 Model Access 계약을 따른다.

[현재 그림 SVG](diagrams/dp3-v8-ab-responsibilities.svg) / [draw.io 원본](diagrams/dp3-v8-ab-responsibilities.drawio) / [PPTX 렌더](diagrams/dp3-v8-ab-responsibilities.png). 공통 scene 모델에서 PPTX, SVG와 draw.io를 생성하며, 원본 소유권과 반환 경로, process 의미를 함께 검토한다.

### 그림 11: B-R2와 B-R3의 배치, 자원, 장애 경계

발표 11쪽. host의 Screen Target Producer, 후보 기록과 Model Access 실행 제어를 양쪽에 정렬한다. 지칭 결합과 최종 채택은 그림 10의 공통 계약을 유지하며, 이 실행 배치 확대도에는 반복하지 않는다. Model Access의 host 부분과 worker 부분을 명시한다. R2의 하나의 process 안에 두 session을, R3의 두 process 안에 해당 adapter/runtime/session/episode를 펼친다. UI 결과가 host로 돌아온 뒤 필요한 SAM 작업을 발행하는 선으로 실제 의존성을 표현한다. UI/SAM으로 동시에 분기하고 합치는 모양을 사용하지 않는다. 명령선은 host의 로컬 실행 제어기와 worker의 IPC 수신기를 직접 연결하며, 별도 반환선도 같은 계약의 양끝으로 돌아온다.

[현재 그림 SVG](diagrams/dp3-v8-resident-processes.svg) / [draw.io 원본](diagrams/dp3-v8-resident-processes.drawio) / [PPTX 렌더](diagrams/dp3-v8-resident-processes.png). 공통 scene 모델에서 PPTX, SVG와 draw.io를 생성하며, 원본 소유권과 반환 경로, process 의미를 함께 검토한다.

| 그림 11 번호 | 요청과 반환 |
| --- | --- |
| 1 | Screen Target Producer의 후보 구성기가 Model Access host의 로컬 실행 제어기에 UI 처리를 요청한다. |
| 2 | 로컬 실행 제어기가 worker의 UI 처리용 IPC 수신기에 실행 명령을 보낸다. R2는 공유 worker, R3는 UI worker다. |
| 3 | CPU 추론 런타임의 UI 결과가 UI 어댑터로 돌아오고, 후처리한 결과가 IPC 수신기 → host의 로컬 실행 제어기 → 후보 구성기 순서로 반환된다. |
| 4 | 정밀 경계나 시간 대응이 필요한 경우에만 후보 구성기가 반환된 UI 결과 또는 native 정보에서 seed를 정해 로컬 실행 제어기에 SAM 처리를 요청한다. |
| 5 | 로컬 실행 제어기가 SAM 처리용 IPC 수신기에 실행 명령을 보낸다. R2는 같은 공유 worker, R3는 SAM worker다. |
| 6 | CPU 추론 런타임의 SAM 결과가 SAM 어댑터로 돌아오고, 후처리한 결과가 IPC 수신기 → host의 로컬 실행 제어기 → 후보 구성기 순서로 반환된다. |

UI 결과에 의존하는 SAM 실행은 3번이 반환되기 전에 4번을 발행하지 않는다. 내부 큐, adapter와 runtime 사이의 선은 실행 구조를 나타내며 별도의 사건 번호를 붙이지 않는다. 후보와 crop을 Request Interpreter에 공급하는 후속 흐름은 그림 10의 4 → 5 경로를 따른다.

### 그림 12: 실행 프로그램 내부와 상태 수명

발표 12쪽의 실행 구조 영역. VIA host에서 bundled Python worker를 기동하고 named pipe로 명령, 완료를 주고받는 관계, worker 내부의 IPC 수신기, adapter, CPU 추론 런타임, model session과 episode tensor를 펼친다. 그림 11에 표시한 worker 대기열은 수명 확대도에서 반복하지 않으며 별도 구조를 제거한 것이 아니다. 디스크의 model files/manifest에서 load하는 경로를 별도로 둔다. model session을 worker 뒤에 놓인 독립 실행 프로그램처럼 그리지 않는다.

[현재 그림 SVG](diagrams/dp3-v8-runtime-lifetimes.svg) / [draw.io 원본](diagrams/dp3-v8-runtime-lifetimes.drawio) / [PPTX 렌더](diagrams/dp3-v8-runtime-lifetimes.png). 공통 scene 모델에서 PPTX, SVG와 draw.io를 생성하며, 원본 소유권과 반환 경로, process 의미를 함께 검토한다.

| 그림 12 번호 | 실제 실행 |
| --- | --- |
| 1 | host의 로컬 실행 제어기가 worker IPC 수신기에 작업 또는 수명 명령을 보낸다. worker가 없으면 먼저 실행 프로그램을 기동하고 준비 완료를 확인한다. |
| 2 | IPC 수신기가 작업을 UI 어댑터 또는 SAM 어댑터에 분배한다. |
| 3 | 해당 어댑터가 CPU 추론 런타임을 호출한다. 런타임과 adapter는 로딩된 model session과 현재 episode 상태를 사용한다. |
| 4 | 런타임 결과를 돌려받은 어댑터가 후처리를 완료하고 IPC 수신기에 결과 또는 오류를 반환한다. 이어 IPC 수신기가 host의 로컬 실행 제어기로 반환한다. 실패와 이전 실행 세대의 결과는 정상 채택 경로에 섞지 않는다. |

`CloseEpisode`는 어댑터가 관리하는 episode 상태를 종료하는 명령이므로 CPU 추론 런타임을 호출할 필요가 없다. 이 경우에도 어댑터 → IPC 수신기 → host의 로컬 실행 제어기로 완료 또는 오류를 반환한다. 런타임의 추론 반환과 어댑터의 작업 완료를 같은 사건으로 합치지 않는다.

상태 수명선은 `EnsureModel`의 모델 적재, `ReleaseModel`의 model session 회수, `RunStage`의 episode 사용과 갱신, `CloseEpisode`의 episode 종료를 구별한다. session 해제 완료는 실제 물리 RAM 반환을 입증하는 사건이 아니며, 사용 중인 참조를 먼저 해제하지 않는다. R2/R3도 최초 사용 때에는 적재 비용이 남고 사용 이후 상주한 session에 한해 재사용한다.

## 6. v7 지적 사항과 v8의 수용 기준

| v7 지적 사항 | v8에서 확인할 실제 표현 |
| --- | --- |
| IPC 호출이 process 경계에서 끝남 | host의 로컬 실행 제어기에서 worker의 IPC 수신기로 명령을 전달하고 결과를 반환하는 별도 연결선 |
| A 원본 보완과 B 후보 확장, 새 범위의 재허가가 각주에만 존재 | 해석기 또는 생산기 → Request Controller → Context Manager의 허가와 조회, Controller를 거친 결과 공급 및 재생산과 재결합 루프 |
| R1 worker의 내부를 설명문으로 요약 | Component 안의 내부 Module과 runtime, session 및 episode 상태와 실제 연결 |
| 상자 내부에 책임 문장과 조건을 적음 | 상자에는 이름, 동작과 조건은 해당 연결선의 라벨 |
| receipt, rev, generation 등의 용어와 가운뎃점이 남음 | 읽은 자료와 판본, 현재 입력 판본, worker 세대 등 쉬운 이름을 먼저 표시하고 §1의 기술 계약에 대응 |
| 자동 검사 통과를 설계 완결성으로 보고 | 그림 경로의 수동 추적, 문서와 도식의 계약 대조, 렌더 점검과 패키지 검사를 각각 기록 |

## 7. 재작성 검수 기준과 수정 범위

1. 그림만 보고 각 Component의 이름과 Module 소유자를 찾을 수 있다. 논리 Component와 OS process 경계를 구별할 수 있다.
2. 최초 실행 허가, 부족한 원본, 후보의 추가 요청, 결과 반환과 최종 채택이 Request Controller의 계약으로 연결된다. cloud 모델의 출력이 직접 최종 상태가 되지 않는다.
3. A의 추가 읽기, crop, native, 유효 이력 재사용과 B의 후보 확장, 원본 보완을 모두 보존한다.
4. host의 후보 기록과 worker의 model session, episode tensor가 별도 상태로 표시된다. R1/R2/R3의 같은 episode 정책과 서로 다른 모델 수명이 구별된다.
5. B-R3의 UI seed→SAM 의존, 동일 총 CPU 예산, 순차 heavy stage를 화살표와 경계가 그대로 표현한다. 따로 적힌 주석으로 반대 모양의 선을 해명하지 않는다.
6. 최초 사용의 load, 이후 재사용, 긴 유휴, memory pressure 회수, worker crash 재시작을 설명할 수 있다. 모델 상주를 항상 사전 로딩된 상태로 그리지 않는다.
7. 오류, 취소, 옛 revision/generation 결과는 정상 채택으로 연결하지 않는다. 모델 반환과 실제 UI 표시를 다른 사건으로 둔다.
8. 문자열 넘침, 선/글자 겹침, 상자 관통, 화살표 출발/도착을 실제 렌더에서 검토한다. native shape 수나 자동 레이아웃 검사만으로 구조적 검수를 대신하지 않는다. 연결선의 각 끝점이 실제 송수신 Module에 닿는지, 재허가와 확장 경로가 닫힌 왕복 경로인지 별도로 확인한다.
9. QA01~07 수치, 데이터셋, 기존 DP1/DP2/DP4/DP5 및 appendix의 역사 DP3는 이번 책임 표기 수정으로 바꾸지 않는다.

이번 문서는 발표용 책임 매핑과 그림 검수 계약이다. 코드 구현, 모델 설치, 실행, Windows 제품 패키징, 실제 메모리 회수, 성능 검증을 수행했다는 근거로 사용하지 않는다.
