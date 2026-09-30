# 실시간 입출력과 상태 처리를 프로세스로 격리할 것인가

> 상태: **조건부 후보 / 논리 Component 변화보다 배치 변화 중심** · [목록](./README.md)

**질문:** UI·Voice·Core를 별도 process로 둘 것인가, 같은 process 안에서 독립 thread·비동기 작업으로 실행할 것인가?

> **발표 구성:** 배경 1장 → 설계 비교 1장. 아래 배경 문구와 도식 구성은 슬라이드 제작용 원고이며 새 배경 그림이나 발표 파일을 생성한 것은 아니다. 사례는 고정 UC를 설명하기 위한 예시이고, 발생 빈도·수치·대안의 우위를 측정한 결과가 아니다.

## 1. 배경 페이지 — 왜 중요한 Architecture 문제인가

### 문제가 드러나는 사용자 상황

Core가 상태를 복구하거나 UI 연동 코드가 막혔을 때도 사용자가 음성을 끊을 수 있어야 한다. 반면 process를 늘리면 입력 근거·출력 명령·receipt가 IPC와 여러 buffer를 지나간다. 독립 실행과 process crash 격리를 위해 어느 경계까지 분리할지가 문제다.

### 이 과제에서 왜 중요한가

VIA는 사용자 PC에서 화면·음성을 계속 다루면서 모델 호출, Context 조회, Agent event와 내구 상태 갱신을 함께 수행한다. 사용자가 “그만”이라고 말할 때 기존 음성을 멈추는 동작은 긴 추론이나 DB 복구 완료를 기다려서는 안 된다. 장애가 발생했을 때도 무엇이 멈췄고 어떤 기능은 쓸 수 있는지 구분해 안내해야 한다.

이는 평상시 평균 처리 속도만의 문제가 아니다. Core가 중단됐을 때도 로컬 음성 stop을 수행하는 것과, 특정 Task 취소를 내구 접수하고 Agent에 보내는 것은 서로 다른 보장이다. **어떤 실패를 함께 겪고 어떤 최소 기능은 계속 제공할 것인가**를 실행 경계로 정해야 사용자 제어를 정직하게 유지할 수 있다.

### Architecture적으로 어려운 이유

| 동시에 만족해야 할 요구 | 구조적으로 부딪히는 지점 | 단순 처리로 남는 문제 |
| --- | --- | --- |
| 장치 callback·local stop은 진행한다 / Core·UI에는 긴 작업과 장애가 생길 수 있다 | 실행 thread의 독립성과 process-fatal 영향 범위는 다름 | 비동기 API만으로 같은 process의 강제 종료까지 격리할 수 없음 |
| 입력 근거를 여러 경계에서 모은다 / 당시 시점·순서를 보존한다 | IPC·buffer·clock mapping·producer sequence·gap 계약 | 분리 뒤 도착 순서를 실제 발생 순서로 오인하거나 유실을 숨길 수 있음 |
| 장애 후 빠르게 재연결한다 / 옛 출력·명령은 다시 유효해지면 안 된다 | process incarnation·output epoch·lease와 내구 상태의 연결 | process 재기동만으로 복구 완료를 선언하면 오래된 packet을 적용할 위험 |
| 격리를 늘린다 / 유한한 PC 자원을 공유한다 | process별 queue·복사·buffer 비용과 공통 CPU·메모리 경합 | process 수만 늘려도 모든 blocking·자원 경합이 해결된다고 오인 |

논리 Component 분리와 process 배치는 다른 축이다. 충분한 thread·비동기 처리를 갖춘 통합 host도 평상시 입력과 상태 처리를 동시에 수행할 수 있다. 따라서 비교의 핵심은 **동일한 논리 구조에서 process-fatal 영향 범위를 줄이는 가치가 경계 관리 비용을 정당화하는가**다. 추론·ASR·위험 connector는 양쪽에서 격리하여 다른 선택을 섞지 않는다.

### 배경 슬라이드 1장에 사용할 내용

**제목:** 처리 종류가 다른 PC 작업을 함께 수행하면서 장애 때의 사용자 제어를 지켜야 한다.

| 배치 | 슬라이드에 남길 핵심 문구 | 함께 보여줄 도식 |
| --- | --- | --- |
| 왼쪽 — 동시 작업 | audio capture·playback / 화면 수집 / 상태·DB / 외부 호출의 실행 성격이 다르다 | 네 실행 lane과 공유 CPU·memory 자원 표시; process 분할은 다음 페이지에서 비교 |
| 가운데 — 장애 순간 | Core 중단 중 사용자가 발화하거나 stop을 누른다 | Core 실패 시점과 local stop 입력을 겹치고, Task 취소 접수 경로는 별도로 표시 |
| 오른쪽 — 설계 과제 | 장애 격리에는 IPC·순서·세대·재연결의 비용이 따른다 | 입력 근거 전달과 출력 명령·receipt 왕복에 queue·epoch·gap 표기 |

**하단 Challenge:** 실시간 입출력의 최소 동작과 상태 일관성을 함께 지키려면, UI·Voice·Core를 어느 process 경계까지 격리해야 하는가?

**발표자 설명:** “Thread를 나누면 긴 작업을 비동기로 처리할 수 있지만 host process가 종료되면 함께 사라집니다. 반대로 process를 나누면 장애 범위를 줄일 수 있는 대신 입력 시각·출력 세대·재연결을 경계 너머에서 맞춰야 합니다. 다음 페이지는 동일한 Component를 유지하며 이 비용과 장애 범위를 비교합니다. 전체 OS 장애나 물리 자원 경합까지 격리한다는 주장은 아닙니다.”

**배경 근거:** [UC-11 음성 중단](../../05-representative-use-cases.md#uc-11), [UC-15 재연결](../../05-representative-use-cases.md#uc-15), [UC-18 장애](../../05-representative-use-cases.md#uc-18), [전체 구조 §14](../target-architecture/architecture.md#14-프로세스-배치fault-boundary), [제어 계약 §7](../target-architecture/control-and-lifecycle.md#7-장애재시작버전-변경). **분류:** 중요한 실행 배치 문제이나 최상위 Component 변화 우선 원칙에서는 조건부 후순위다.

## 2. 설계 비교 페이지 — 같은 Component를 UI·Voice·Core process로 나누거나 host에 모은다

**배경에서 이어받는 질문:** 문서 첫머리의 구조적 질문을 같은 사용자 목표·조건에서 비교한다. 먼저 책임 경계, 다음 상태와 계약, 마지막 이점·비용의 순서로 설명한다.

### 두 방안

**방안 1 — UI·Voice·Core 분리.** 기준선은 UI·Voice·Core process를 분리한다. Interaction Manager의 입출력·timeline 모듈도 이 배치에 나뉜다. Core와의 연결이 끊기면 Voice는 lease·epoch로 오래된 출력을 거절하고 local stop을 유지한다.

**방안 2 — host process 통합.** UI·Voice·Core를 한 process로 합치되 device callback, 화면 수집, 상태 전이, 비동기 연동에 독립 thread·bounded queue·실행 예산을 둔다. native 추론과 위험한 connector는 두 방안 모두 별도 process에 남긴다. 논리 Component와 상태 owner는 유지한다.

![UI Voice Core 분리와 host process 통합 비교](./diagrams/runtime-isolation.svg)

[draw.io 편집 원본](./diagrams/runtime-isolation.drawio) · [표기 규칙](./diagrams/README.md)

### 그림에서 확인할 흐름과 구조 차이

① 화면 근거, ② 음성 입력·시간 근거, ③ 출력 release/receipt가 파란 process 경계를 어떻게 넘는지 비교한다. 왼쪽의 UI·Voice·Core가 오른쪽 Host process 하나로 합쳐져도 내부의 검정 논리 Component와 Request Interpreter를 통한 semantic 경로는 유지된다. Speech Input Worker·Shared Inference Service·위험 connector도 공통으로 격리한다. 이 그림은 배치와 대표 흐름을 보여주며 모든 논리 Component 호출을 반복하지 않는다. Core process-fatal과 전체 host process-fatal의 영향 차이가 핵심이다.

### 구조 차이와 조건

달라지는 것은 **process crash의 영향 범위와 Component 간 전달 방식**이다. IPC가 process 내부 queue·참조 전달로 바뀌고 host 종료 시 UI·Voice·Core가 함께 중단된다. 논리 Component가 통합된 것으로 설명하지 않는다. 별도 process도 CPU·메모리 대역폭·전력 경합은 공유한다.

방안 1은 Core의 process-fatal 오류와 실시간 입출력의 운명을 분리하기 좋다. 대신 IPC·직렬화·buffer·incarnation·lease·재연결 계약이 필요하다. 방안 2는 내부 전달과 데이터 공유가 단순해질 수 있고, 안정적인 host code·강한 thread 격리 조건에서 유리할 수 있다. 대신 process-fatal 오류는 함께 겪으며 thread만으로 이를 격리할 수 없다.

### 설계 비교 페이지 발표 설명

**그림에서 짚을 순서:** 검정 논리 Component와 아래의 추론·ASR·connector 격리는 공통으로 고정한다. 파란 UI·Voice·Core process 경계 및 IPC와 통합 host의 내부 queue를 비교한 뒤 Core 장애·host 장애의 영향 범위를 설명한다.

**발표자 설명:** “통합안도 충분한 thread와 비동기 처리를 갖습니다. 따라서 평상시 처리 능력을 단일 thread와 비교해서는 안 됩니다. 차이는 Core의 process-fatal 오류를 local stop과 분리하는 가치, 그리고 그 분리를 위해 추가한 IPC·세대·복구 계약의 비용입니다. process 수만으로 의미 판단의 정확도가 높아진다고 주장하지 않습니다.”

## 3. 자체 검토와 남은 질문

- 단일 thread에서 모든 일을 직렬 처리하는 약한 대안을 제외했다. 충분한 thread·비동기·자원 제어를 허용한다.
- 비교 범위는 UI·Voice·Core로 한정한다. Shared Inference Service, Speech Input Worker, 위험 connector 격리까지 동시에 없애지 않는다.
- Task Manager·Request Controller 등 논리 책임은 양쪽에서 같다. [대화·업무 owner 통합](./conversation-task-ownership.md)과 별개 선택이다.
- 동일 입력 근거가 유지되면 process 수만으로 의미 판단이 좋아진다고 할 수 없다. event 유실·지연이 실제 판단에 영향을 주는 경우에만 그 경로를 논의한다.
- **후순위로 남긴 이유:** 설계적으로 중요한 runtime 선택이지만 사용자가 우선한 논리 Component 변화에는 해당하지 않는다. 심사에서 runtime view를 별도로 다룰지 결정한 후 채택한다.
- host 통합이 필수 입출력·복구 요구를 충분히 만족하면서 경계 비용을 줄인다면 분리 범위를 재검토할 이유가 된다. 아직 그러한 결과는 없다.

기준선 근거: [전체 구조 §14](../target-architecture/architecture.md), [제어 계약 §7](../target-architecture/control-and-lifecycle.md). 주요 사용자 상황: UC-11·13~15·18.
