# 음성 입력 근거를 별도 경로에서 만들 것인가

> 상태: **조건부 후보 / 모델·runtime capability 확인 필요** · [목록](./README.md)

**질문:** 공유 Omni와 독립된 입력 인식 경로를 둘 것인가, Omni의 동시 입력 session에서 필요한 음성 근거를 모두 제공할 것인가?

> **발표 구성:** 배경 1장 → 설계 비교 1장. 아래 배경 문구와 도식 구성은 슬라이드 제작용 원고이며 새 배경 그림이나 발표 파일을 생성한 것은 아니다. 사례는 고정 UC를 설명하기 위한 예시이고, 발생 빈도·수치·대안의 우위를 측정한 결과가 아니다.

## 1. 배경 페이지 — 왜 중요한 Architecture 문제인가

### 문제가 드러나는 사용자 상황

화면 자료를 해석하는 중에 사용자가 “아니, 옆의 표야”라고 말한다. 녹음만 쌓아 두었다가 기존 해석이 끝난 뒤 처리하면, 정정 수신과 지칭 시점의 연결이 늦어진다. semantic 추론 중에도 발화 수신·인식과 시간 근거 제공이 계속되어야 한다.

### 이 과제에서 왜 중요한가

VIA는 사용자가 말하는 동안 화면을 가리키거나, 기존 설명을 끊고 대상을 고치는 interaction을 지원한다. 그러려면 최종 문장만 얻는 것에 더해 어느 발화 구간이 어떤 화면·선택에 대응했고 어떤 전사가 수정됐는지 알아야 한다. **말을 받아 두는 것과 현재 요청을 바로잡을 입력 근거를 제때 제공하는 것은 다른 기능**이다.

현재 목표는 하나의 on-device Omni를 음성과 semantic 역할이 공유하는 구조다. 화면 해석이나 긴 결과 요약이 진행 중일 때도 사용자가 다시 말하는 것은 필수 사용 상황이다. 해당 추론이 끝난 뒤 녹음을 처리하면 그동안의 정정과 입력 근거 처리가 밀리므로 녹음 지속만으로 요구를 충족했다고 할 수 없다. 입력 근거 생성의 의존·장애 경계를 별도로 검토할 이유가 여기에 있다.

### Architecture적으로 어려운 이유

| 동시에 만족해야 할 요구 | 구조적으로 부딪히는 지점 | 단순 처리로 남는 문제 |
| --- | --- | --- |
| 가중치는 한 번만 적재한다 / Voice·semantic은 함께 진행한다 | 공유 연산·KV·workspace의 경합과 입력 service 보장 | role별 session을 만들었다는 사실만으로 인식의 지속 진행이 보장되지 않음 |
| 발화와 화면을 연결한다 / 전사·시간 추정에는 오차와 수정이 있다 | sample 범위·span timing·revision·gap을 제공하는 입력 계약 | 최종 transcript 문자열만으로 말하는 동안 바뀐 지칭 대상을 복원할 수 없음 |
| 입력 근거를 독립적으로 만든다 / PC 자원과 중복 처리를 줄인다 | 별도 ASR weights·CPU·buffer와 독립 장애 경계의 교환 | 독립 경로의 전체 비용을 빼거나 모델 하나를 제거한 것만으로 동일 기능을 가정 |
| ASR·Omni 해석이 다를 수 있다 / 업무에는 하나의 확정 입력이 필요하다 | 불일치 보존·원음 재확인·canonical revision 변경의 host 책임 | confidence나 다수결로 대상을 확정하면 핵심 정정을 잘못 적용할 수 있음 |

이 후보에서 비교하는 것은 인식 모델의 내부 알고리즘이 아니다. **입력 근거를 어느 runtime이 생산하고, semantic 부하나 모델 재시작이 그 경로까지 영향을 주는가**이다. 별도 ASR을 없애려면 Omni 경로가 같은 시간·revision·gap·동시 진행 계약을 제공해야 한다. 그 capability는 아직 확인되지 않았으므로 통합 대안을 즉시 구현 가능한 동등 후보로 발표하지 않는다.

### 배경 슬라이드 1장에 사용할 내용

**제목:** 공유 추론이 바빠도 사용자의 정정과 지칭에 필요한 음성 근거는 계속 들어와야 한다.

| 배치 | 슬라이드에 남길 핵심 문구 | 함께 보여줄 도식 |
| --- | --- | --- |
| 왼쪽 — 필수 사용 상황 | “이 문단을 설명해줘” 처리 중 “아니, 옆 표야”라고 정정한다 | semantic 진행 막대 아래 새 음성 입력과 pointer가 옆 표로 이동하는 lane |
| 가운데 — 필요한 입력 | capture + 인식 + 시간 근거·수정 이력이 모두 필요하다 | audio sample → transcript revision·span → 당시 화면 후보의 대응. 처리시간 수치는 넣지 않음 |
| 오른쪽 — 공유의 긴장 | Omni는 한 번 적재하되 입력·semantic의 연산·상태는 별도로 필요하다 | 공유 weights와 역할별 session/KV, runtime 중단 시 영향받는 근거 경로를 표시 |

**하단 Challenge:** 하나의 Omni를 공유하면서도 입력의 지속성과 시간 근거를 확보하려면, 입력 인식 경로를 독립시킬 것인가 공유 runtime에 통합할 것인가?

**발표자 설명:** “VIA의 음성 입력은 나중에 읽을 녹음이 아닙니다. 사용자가 지금 가리킨 표와 정정 발화를 연결할 근거입니다. 독립 ASR은 그 경로를 Omni 부하와 분리하지만 자원과 불일치 처리 비용이 생깁니다. 통합하려면 그 비용을 없앤 만큼 필요한 입력 계약을 Omni가 실제로 제공해야 합니다. 다음 페이지의 통합안은 이 기능 확인을 전제로 한 조건부 대안입니다.”

**배경 근거:** [UC-04 발화·화면 결합](../../05-representative-use-cases.md#uc-04), [UC-11 정정](../../05-representative-use-cases.md#uc-11), [UC-18 모델 장애](../../05-representative-use-cases.md#uc-18), [공유 Omni 계약 §2~7](../target-architecture/shared-omni-runtime.md#3-발화를-받는-경로를-의미-해석과-분리한다). **분류:** dependency·runtime 경계의 조건부 후보이며 Omni 입력 capability와 총자원 비용은 후속 확인 대상이다.

## 2. 설계 비교 페이지 — 음성 입력 근거 생산의 독립 dependency·runtime을 유지하거나 통합한다

**배경에서 이어받는 질문:** 문서 첫머리의 구조적 질문을 같은 사용자 목표·조건에서 비교한다. 먼저 책임 경계, 다음 상태와 계약, 마지막 이점·비용의 순서로 설명한다.

### 두 방안

**방안 1 — 독립 입력 근거 경로.** 기준선은 Speech Input Worker에 경량 Streaming ASR을 두고, Shared Inference Service의 Omni와 별도로 입력 근거를 만든다. Interaction Manager가 revision·시간 근거를 연결하며, Omni 해석과 다르면 원음 참조 또는 clarification으로 조정한다. Model Access는 두 dependency의 계약을 연계한다.

**방안 2 — Omni 입력 경로에 통합.** 별도 ASR과 Speech Input Worker를 제거하고 공유 Omni의 VOICE session이 전사·revision·시간 근거를 제공한다. semantic과 동시 활성화하고 유한 입력 service 계약을 제공해야 한다. capture와 local stop은 Interaction Manager에 남고 Omni 장애 구간은 gap으로 남긴다.

![독립 음성 입력 근거 경로와 Omni 통합 경로 비교](./diagrams/speech-evidence-boundary.svg)

[draw.io 편집 원본](./diagrams/speech-evidence-boundary.drawio) · [표기 규칙](./diagrams/README.md)

### 그림에서 확인할 흐름과 구조 차이

위쪽 Interaction Manager의 capture·timeline·local stop은 공통이다. 가운데 파란 Model Access 계약과 runtime 경계를 비교하면 독립 ASR·입력 buffer·불일치 조정이 없어지는 대신 입력 근거까지 Shared Inference Service가 책임져야 함을 볼 수 있다. 확정 입력은 Interaction Manager의 timeline을 거쳐 Request Controller로 전달한다. 아래 Omni weights는 양쪽 모두 하나이며 session별 복제가 아니다. 오른쪽 필수 입력 capability는 확보된 구현이 아니라 대안 성립 조건이다.

### 구조 차이와 조건

별도 dependency·worker·불일치 조정 계약의 유무가 바뀐다. VIA 최상위 논리 Component 수가 바뀌는 후보는 아니며 **Model Access의 하위 연계 구조와 dependency·runtime 경계**를 비교한다. Streaming ASR이나 Omni 모델 자체를 새 VIA Component라고 세지 않는다.

방안 1은 Omni 추론·재시작과 입력 근거 생성을 분리하지만 ASR weights·CPU·buffer, 중복 인식과 전사 불일치 비용을 감수한다. 방안 2는 이 중복을 줄일 가능성이 있지만 인식·의미 해석이 동일 runtime 장애에 묶이고, 입력 기능과 동시 service를 해당 build에서 확보해야 한다. 단순 transcript 출력만으로 지칭 시각·정정·gap 계약을 충족했다고 할 수 없다.

### 설계 비교 페이지 발표 설명

**그림에서 짚을 순서:** 위쪽 capture·timeline·local stop과 아래 단일 Omni weights가 공통임을 먼저 확인한다. 파란 Speech Input Worker·ASR·불일치 경로가 통합안에서 사라지고 Shared Inference Service의 입력 계약이 확대되는 지점을 짚는다.

**발표자 설명:** “별도 ASR이 있다면 그 weights·CPU·buffer와 중복 인식 비용을 전부 인정해야 합니다. 없앤다면 시간 근거와 수정 이력, semantic 중 입력 진행을 Omni 경로에서 제공해야 합니다. 그 기능은 아직 확인되지 않았습니다. 따라서 이 페이지는 통합안의 성능 우위를 제시하는 것이 아니라, 독립 경로를 재검토할 조건과 비용을 설명합니다.”

## 3. 자체 검토와 남은 질문

- 두 방안 모두 Omni weights는 한 번만 적재한다. whole-call mutex나 무한 backlog를 대안으로 삼지 않는다.
- 방안 2가 semantic 중 수신·인식을 유지할 수 있는지는 미확인이다. 현재 제품과 동등하게 실현 가능하다고 확정하지 않는다.
- Omni 장애 중에도 capture·local stop·Text와 확인 가능한 Task 상태는 유지한다. 별도 인식이 사라져 늘어나는 장애 영향은 대안의 비용으로 드러낸다.
- **조건부로 남긴 이유:** 구조 차이는 선명하지만 모델 내부 알고리즘 비교로 흐를 위험과 capability 공백이 있다. 같은 기능 계약을 제공하는 runtime 대안이 구체화될 때 우선 후보 승격을 검토한다.
- Omni만으로 필수 입력 계약을 더 적은 전체 비용으로 충족한다면 독립 ASR의 필요성이 약해진다. 현재 그런 결과는 없다.
- [프로세스 격리 후보](./runtime-isolation.md)는 ASR을 양쪽에 유지한 채 host 배치만 바꾸므로 이 선택과 구분한다.

기준선 근거: [공유 Omni §1~4·6~8](../target-architecture/shared-omni-runtime.md), [전체 구조 §6·11·14](../target-architecture/architecture.md). 주요 사용자 상황: UC-03·04·11·14·18.
