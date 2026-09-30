# Target Architecture에서 복원한 설계 선택 후보

> 상태: **Decision Reconstruction / 상세 후보 8개 공동 검토 / 우열·정식 DP 미선정**
> 갱신일: 2026-10-01 · 기준: [검토 완료 Target Architecture](../target-architecture/README.md)

**주요 기능을 구현하면서 어떤 방식을 선택했는지, 그때 다른 설계자라면 어떤 합리적인 방식을 선택할 수 있었는지를 복원한다.** 현재 설계에 적힌 처리 방식은 근거이고, 아래 대안과 선택 이유의 해석은 이번에 재구성한 제안이다. 당시 두 안을 실제 비교했거나 현재 안의 우위를 검증했다는 기록은 아니다.

## 먼저 읽을 곳

각 후보 문서 맨 위에 **배경 1장 + 두 대안 SW 구조 비교 1장**을 넣었다. 그다음 **ASR·추가 QA 장단점 비교표 → 그림의 실행 계약 → 상세 근거·반례 → 발표 요약** 순서로 읽는다. SVG는 GitHub에서 바로 보이고 각 그림 아래에 크게 보기와 `.drawio` 편집 원본을 연결했다.

- 전체 그림: [16페이지 목록·범례·편집 방법](./diagrams/README.md)
- 브라우저에서 연속 검토: [review.html](./diagrams/review.html) — 로컬에서 열면 배경·비교 순서로 16페이지를 표시한다.
- 품질 비교표는 **현재 ASR 네 축 + 관련 추가 QA**를 양안 장단점·유불리 조건으로 비교한다. 수치·별점·승자 없이 구조적 예상과 적용 제안을 표시했다.

## 후보별 문서와 발표 그림

| 후보 / 상세 문서 | 배경 페이지 | SW 구조 비교 페이지 |
| --- | --- | --- |
| [요청 의미 해석](./request-interpretation-topology.md) | [배경](./diagrams/request-interpretation-topology-background.svg) | [양안 구조도](./diagrams/request-interpretation-topology-comparison.svg) |
| [Context 획득](./context-acquisition-strategy.md) | [배경](./diagrams/context-acquisition-strategy-background.svg) | [양안 구조도](./diagrams/context-acquisition-strategy-comparison.svg) |
| [Agent 상태 관측](./agent-state-observation.md) | [배경](./diagrams/agent-state-observation-background.svg) | [양안 구조도](./diagrams/agent-state-observation-comparison.svg) |
| [직접 응답 생성](./direct-response-generation.md) | [배경](./diagrams/direct-response-generation-background.svg) | [양안 구조도](./diagrams/direct-response-generation-comparison.svg) |
| [음성 입력 근거](./speech-evidence-source.md) | [배경](./diagrams/speech-evidence-source-background.svg) | [양안 구조도](./diagrams/speech-evidence-source-comparison.svg) |
| [대화 Context 유지](./conversation-context-maintenance.md) | [배경](./diagrams/conversation-context-maintenance-background.svg) | [양안 구조도](./diagrams/conversation-context-maintenance-comparison.svg) |
| [자료 표현·소비](./context-representation-pipeline.md) | [배경](./diagrams/context-representation-pipeline-background.svg) | [양안 구조도](./diagrams/context-representation-pipeline-comparison.svg) |
| [복구 상태 원본](./recovery-state-source.md) | [배경](./diagrams/recovery-state-source-background.svg) | [양안 구조도](./diagrams/recovery-state-source-comparison.svg) |

## 후보 선정 검토표

| 기능을 구현할 때의 질문 | 현재 설계에서 선택한 방식 | 재구성한 합리적 대안 | 그림의 테두리를 지워도 남는 차이 | 제안 상태 |
| --- | --- | --- | --- | --- |
| [대상·목표·업무·처리 경로를 어떻게 함께 이해할까?](./request-interpretation-topology.md) | 여러 의미를 한 semantic 작업에서 통합 판단하고 필요 근거를 추가 | 근거·대상 → 목표·Task 관계 → 처리 경로의 단계별 해석, 중간 후보와 되돌림 계약 | 추론 의존 그래프, 중간 결과 저장, 오류 전파와 재처리 범위 | 우선 검토 |
| [요청을 이해하는 데 필요한 정보를 언제 어떻게 확보할까?](./context-acquisition-strategy.md) | 입력 중 기본 Context 사전 준비 → 통합 해석 → 부족한 정보만 추가 조회 | 최소 입력으로 조회 계획 → 필요한 source 조회 → 해석 | 첫 추론의 입력, 선행 조회, 대기 경로와 불필요한 읽기 | 우선 검토 |
| [여러 Agent의 진행 상태를 어떻게 사용자에게 제공할까?](./agent-state-observation.md) | event를 적용해 로컬 진행 상태를 유지하고 gap·최신성 요구에 재조회 | snapshot 조회로 진행 상태를 구성하고 event는 조회 신호로 사용 | event reducer·진행 projection 유지와 query coordinator·snapshot cache의 차이 | 우선 검토; 같은 Agent의 조회 기능 전제 |
| [S2S 직접 응답의 대기를 줄이기 위해 답변을 먼저 만들어 둘까?](./direct-response-generation.md) | 직접 경로 허용 전 답변을 선생성·보류하고 허용 시 release, 기각 시 폐기 | 짧은 직접/Core 분류 → host 경로 허용 → 선택된 경로에서 답변 생성 | 미승인 generation·보류 buffer·취소 경로와 분류→생성의 순차 의존 | 추가 검토; 적용 범위와 runtime 계약 확인 필요 |
| [음성 입력 근거를 별도 recognizer로 만들까?](./speech-evidence-source.md) | 독립 Streaming ASR과 Omni의 이중 입력 처리·불일치 조정 | 공유 Omni의 native 전사·시각·revision stream | 모델 dependency·worker·증거 생산·장애 범위와 전체 자원 비용 | 확대 검토; native capability·장애 trade-off 명시 |
| [다음 대화에 쓸 Context를 계속 갱신해 둘까?](./conversation-context-maintenance.md) | 요청 시 owner 조회·조합, 유효 cache·summary 재사용 | 활성 대화의 증분 working view + revision read barrier | projector·변경 전달·적용 위치·rebuild와 상시 메모리 | 확대 검토; 기존 cache와 수렴하는지 확인 |
| [같은 자료에서 재사용할 구조화 근거를 먼저 만들까?](./context-representation-pipeline.md) | 원문·이미지·typed record 중심 package로 실제 요청 해석 | 구조화 사실 view materialization 후 여러 소비 경로에서 재사용 | 선행 추출 job·view 저장·원문 복귀·오류 전파 경로 | 확대 검토; 반복 사용 가치·정보 보존 검토 |
| [재시작 후 무엇을 원본으로 상태를 복원할까?](./recovery-state-source.md) | 현재 owner 상태와 미완료 송수신·전달 원장 | 확정 domain 이력 + checkpoint·현재 projection | authoritative 저장·replay·migration·삭제·효과 조정 | 확대 검토; 복구 필요성과 전체 유지 비용 검토 |

**현재 개별 상세 문서는 8개다.** 앞서 작성한 4개에 음성 근거·기억 유지·자료 표현·복구 원본 4개를 추가했다. 앞의 ‘우선 검토’는 최초 제안의 읽기 순서이며 새 후보보다 중요하다는 최종 순위가 아니다. 8개 모두 채택한 것도 아니고 발표 DP 개수를 확정한 것도 아니다. 미확인 능력·수렴 가능성은 후보를 만들지 않을 이유로 삼지 않고, 강한 대안을 작성한 뒤 함께 판단할 조건으로 남겼다.

각 문서에는 배경·비교 SVG와 draw.io 원본, ASR·QA 장단점 비교표, 내부 모듈·입출력 field·중간 상태·정정/취소/복구 계약, 같은 상황의 추적, 이점·대가·유리한 조건과 반증 조건을 적었다. 끝에는 바로 읽을 수 있는 **배경 5줄·설계 비교 8줄**을 두었다. 추가 자료 표현 후보는 target이 공통 의미 변환을 이미 강제한다고 가정하지 않고, 실제 원문 중심 근거 경로와 선행 materialization 대안을 비교하도록 질문을 다듬었다.

**기존 일곱 후보는 현재 추천에서 철회했다.** Component를 묶거나 나누는 차이를 중심에 두어, 실제 작동 방식과 품질 영향의 설명이 부족했다. 다만 문제 상황·정정·근거·전달 계약까지 폐기하지 않았다. [기존 7개에서 살린 내용과 VIA-DP-01~18 아이디어 검토](./reference-idea-review.md)에 반영·유보 이유를 각각 남겼다. [archive의 철회 사유와 원본 목록](../../../archive/decision-reconstruction-component-boundaries-2026-09-30/README.md)은 역사 기록이며 새 후보의 규범 근거가 아니다. 새 목록은 이전 후보의 이름 변경이나 기존 DP 번호의 재배치가 아니다.

## 읽는 순서와 근거

1. 이 표에서 해결할 기능과 두 구현 방식을 비교한다.
2. 관심 후보의 **「같은 상황을 따라가 보면」**과 **「왜 대안을 선택할 수 있는가」**를 읽는다.
3. [선발 원칙](./selection-principles.md)에서 구조적 선택 판정 기준을, [검토 기록](./review-notes.md)에서 제외·보류 이유와 후보 간 중복을 확인한다.
4. [기존 자료 검토](./reference-idea-review.md)에서 이전 7개 재사용 판단, 이전 18개 아이디어·ADR 상태, 보관 경로·방법을 확인한다.
5. 현재 방식의 정확한 계약은 각 문서의 target 절·고정 UC 링크로 확인한다. 대안을 기준선에 반영한 것은 아니다.

사용자 요청에 따라 후보 선정 전에 리뷰할 수 있도록 8개 모두 실제 배경·비교 그림을 작성했다. **공통은 검정, 서로 다른 경로·상태·계약은 양쪽 모두 파랑**이다. 구조도는 Component 내부 처리 모듈·데이터·실행 의존·보완/무효화·확정·복구 경계를 보여준다. 그림이 존재한다는 사실은 설계 채택이나 효과 검증을 뜻하지 않는다.

## 함께 유지할 조건

- 같은 사용자 목표와 완료 조건, VIA/Downstream Agent 책임 경계, 사용자 명시 합의를 유지한다. 한 업무는 통째로 Agent에 맡기고 독립 업무는 별도 Task로 추적한다.
- 한 on-device Omni의 가중치를 공유한다. 여러 단계 호출은 모델 복제가 아니다. 입력 수신·인식, 역할별 권한·session 분리, 좁은 S2S 직접 응답 범위를 유지한다.
- 대상 모호함을 숨기거나 검증·중단·삭제·승인·장애 처리를 제거하여 대안을 빠르게 만들지 않는다. 다만 현재의 조회 순서·중간 표현·저장 수단·호출 횟수까지 대안의 불변 조건으로 강제하지 않는다.
- 의미 해석 후보에서는 입력 근거를, Context 획득 후보에서는 통합 해석 방식을, Agent 상태 후보에서는 provider 기능을 맞춰 비교한다. 세부 범위는 [중복 검토](./review-notes.md#3-후보-간-중복과-고정할-조건)를 따른다.
- Context 관련 세 후보는 각각 **읽기 시작 시점 / 내부 기록의 지속 갱신 / 확보한 자료의 변환·소비 방식**을 다룬다. 하나의 큰 Context 개선안으로 묶어 효과를 중복 주장하지 않는다.
- 음성 근거 후보는 dependency 구성 자체를 바꾸므로 ASR 유무와 비용·오류·장애 범위를 드러낸다. 같은 fault가 두 구조에 같은 기능 손실을 주도록 강제하지 않되, 인식 중단·gap을 성공으로 바꾸지 않는다.

## 현재 검토의 범위

사용자 요청에 따라 QA-19·09·29·39와 추가 QA의 장단점을 표로 비교하고, core 축별 `PRIMARY`/`REGRESSION_ONLY` 적용을 제안했다. 동일 구조가 언제 유리하고 불리한지 조건을 적었으며 정식 모집단·역할·목표·ASR 변경을 동결하지 않았다. QA-41 메모리는 별도 모델·KV·중간 결과·cache·buffer의 전체 비용으로 설명하고 기존 diagnostic 지위를 유지한다. QA-04, QA-13~15, QA-31/32, QA-51, QA-61/62도 관련되는 후보에서 기존 의미에 맞춰 구별했다.

현재 산출물은 문서 대조와 사건 흐름 추적에 의한 자체 검토다. 독립 리뷰·구현·모델 실행·성능 측정·freeze는 수행하지 않았다. 기준선·accepted/deferred ADR·과거 증거도 변경하지 않았다. 후보를 함께 좁힌 뒤 관련 ASR을 검토하고 정식 rationale·반증 조건·검증 계약으로 발전시킨다.
