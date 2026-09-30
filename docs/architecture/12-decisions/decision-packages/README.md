# Architecture Decision Packages

> 상태: **Decision Reconstruction / 구조 후보 검토 중 / 정식 package·구현·측정 결과 없음**
> 기준: [검토 완료 Target Architecture](../target-architecture/README.md) · 후보 작성일: 2026-09-30

이 디렉터리는 VIA의 목표 Architecture에서 중요한 구조적 선택을 찾고, 강한 대안·비용·반증 조건을 갖춘 Decision Package로 발전시키는 작업 공간이다. **대안에 따라 Component의 책임·구성·상태 소유권·연결이 실질적으로 달라지는 선택을 우선한다.**

현재는 일곱 후보를 사용자와 검토하는 단계다. 각 문서는 **배경 1장 + 설계 비교 1장**의 발표 자료로 옮길 수 있도록 구성했다. 배경에서는 VIA에서의 중요성·서로 충돌하는 요구·구조적 난점과 결정할 질문을, 비교에서는 두 방안의 그림·상태·계약·이점과 비용을 설명한다. 방안 1은 검토 완료 기준선의 구조, 방안 2는 그 선택을 설명하고 재검토하기 위한 steelman 초안이다. 후보 채택과 우열은 확정하지 않았다.

사용자가 요청한 순서에 따라 **구조 후보를 먼저 논의하고 ASR은 후속으로 검토한다.** 기존 DP에 매핑하거나 정식 번호·개수를 미리 정하지 않는다. 현재 문서화는 기준선 변경, ASR 재정의, 구현·모델 실행·성능 측정·freeze의 승인이 아니다.

## 먼저 논의할 구조 후보

아래 다섯 항목은 **선발 원칙에 따른 우선 검토 제안**이다. 채택 개수나 순서는 확정하지 않았다. 방안 1은 기준선의 구조, 방안 2는 이를 설명·재검토하기 위한 대안이다.

| 문제와 후보 | 방안 1 — 기준선 | 방안 2 — steelman 초안 | 그림에서 보이는 실제 변화 |
| --- | --- | --- | --- |
| [요청의 해석과 확정을 어떻게 나눌 것인가](./request-resolution-boundary.md) | Request Interpreter가 의미 제안, Request Controller가 흐름·확정 소유 | Request Controller에 의미 해석 orchestration을 통합하고 내부 검증 경계를 유지 | Request Interpreter의 독립 Component 경계 제거, proposal 계약의 내부화 |
| [여러 곳에서 필요한 Context를 누가 준비할 것인가](./context-preparation-ownership.md) | Context Manager가 요청 목적에 맞는 공통 근거 package 구성 | 소비 Component별 Context 준비, Context Manager는 공통 source 접근·기억 관리 | 공통 구성 책임이 소비자 쪽으로 이동하고 조회 연결이 분기 |
| [대화와 오래가는 업무 상태를 어디서 관리할 것인가](./conversation-task-ownership.md) | Request Controller와 Task Manager가 별도 상태 권위 | Request Controller에 Task Manager의 상태 관리 책임 통합 | Task Manager 경계 제거, 두 owner의 인계가 한 owner의 상태 전이로 변경 |
| [응답 전달을 한곳에서 관리할 것인가](./response-publication-ownership.md) | Response Manager가 직접 답변·업무 알림의 전달을 공통 관리 | 직접 답변과 업무 알림을 별도 응답 Component가 관리 | Response Manager가 둘로 분리, Interaction Manager의 공통 발화 조정 계약 확대 |
| [서로 다른 Agent 연동을 어디서 흡수할 것인가](./agent-integration-boundary.md) | Agent Gateway가 공통 전송·수신 lifecycle과 adapter 관리 | Agent별 독립 연동 Component가 전송·수신 lifecycle 소유 | Agent Gateway의 공통 실행 경계가 Agent별 경계로 분리 |

## 조건을 더 확인할 후보

| 후보 | 남겨두는 이유 | 우선 후보와 구분한 이유 |
| --- | --- | --- |
| [음성 입력 근거 경로를 독립시킬 것인가](./speech-evidence-boundary.md) | 별도 ASR dependency와 입력 worker의 유무가 실제 구조를 바꿈 | VIA 최상위 Component 증감보다 dependency·하위 모듈 경계의 변화다. 대안의 Omni 입력 기능 확보 여부도 미확인 |
| [실시간 입출력과 상태 처리를 프로세스로 격리할 것인가](./runtime-isolation.md) | blocking·process crash를 어디까지 함께 겪는지 달라짐 | 논리 Component는 유지되고 runtime 배치가 달라진다. Component 변화 우선 원칙에서는 후순위 |

스케줄링 우선순위·호출 횟수·보관 기간은 이 표의 독립 후보가 아니다. Agent 선택 알고리즘과 DB 제품 선택도 같은 이유로 별도 후보로 올리지 않았다. 공유 transaction 대 owner별 journal은 구조 차이가 있으나 현재는 업무 상태 소유권 후보와의 중복을 먼저 정리하도록 보류했다. 상세 이유는 [검토 기록](./review-notes.md)에 있다.

## 문서 구성과 읽는 순서

후보 문서와 이후 package는 `decision-packages/` 바로 아래에서 관리한다. 논의 상태는 각 문서에 표시하고, 파일명은 해결 문제를 따른다.

| 문서 | 역할 |
| --- | --- |
| `README.md` | 현재 상태, 후보 목록, 검토 순서와 후속 package 작성 기준 |
| [selection-principles.md](./selection-principles.md) | 구조적 선택의 선발 원칙과 발표 그림의 필수 내용 |
| 위 표의 후보별 문서 | 배경 페이지의 중요성·난점·슬라이드 문구·도식 구성·Challenge, 설계 비교 그림과 양쪽 페이지의 발표 설명 |
| [review-notes.md](./review-notes.md) | 약한 대안의 보완, 후보 간 중복, 보류·제외 이유와 자체 검토 기록 |
| [diagrams/](./diagrams/README.md) | 일곱 비교 그림의 SVG·편집 가능한 draw.io 원본과 표기 규칙 |

처음에는 후보 표와 관심 후보를 읽고, 선발 근거가 궁금할 때 선발 원칙·검토 기록을 확인한다. 기준선과 대조할 때는 [전체 Architecture](../target-architecture/architecture.md)와 후보 문서 끝의 관련 계약을 읽는다. 새 세션은 AGENTS.md와 [기준선 안내](../target-architecture/README.md)의 확정 범위를 먼저 확인한다.

발표 자료를 만들 때는 후보별 **「배경 슬라이드 1장에 사용할 내용」의 제목·세 영역·하단 Challenge**를 첫 장의 원고로 사용한다. 「이 과제에서 왜 중요한가」와 「Architecture적으로 어려운 이유」는 설명과 질의응답의 근거다. 다음 장은 기존 비교 그림과 「설계 비교 페이지 발표 설명」을 사용한다. 배경의 도식 구성은 제작 지침이며 새 배경 그림·발표 파일은 아직 만들지 않았다.

본문의 `owner`는 상태를 최종 변경할 책임자, `admission`은 실행·게시의 허용 판단, `publication`은 사용자에게 게시하는 응답 단위, `revision`은 변경 버전이다. Component는 기준선의 정식 명칭을 사용한다.

## 그림을 읽는 방법

- **검정은 공통 부분, 파랑은 양쪽에서 달라지는 부분**이다. 방안 1에도 같은 기준을 적용하며 색으로 우열을 표시하지 않는다.
- 파란 Component 경계 안의 검정 내부 블록은 기능은 유지되지만 소속·책임 경계가 바뀐다는 뜻이다. 내부 책임 블록을 새 Component로 세지 않는다.
- Component·process/runtime·외부 dependency·저장 기록을 구분한다. 번호는 핵심 흐름, 실선은 호출·event, 점선 화살표는 상태 접근이다.
- 각 그림에는 사용자 상황, 주요 계약·상태, 공통 조건, 양쪽의 이점·비용·검토 질문을 함께 둔다. 생략한 공통 연결은 그림 주석과 본문에 밝힌다.
- SVG와 같은 이름의 `.drawio`를 함께 갱신한다. 상세 범례와 생성·검사 방법은 [그림 작성 규칙](./diagrams/README.md)을 따른다.

## 공동 검토와 정식 package로 발전시키는 순서

1. **후보 검토:** 문제의 중요성, 실제 Component 차이, 대안의 설득력, 후보 간 독립성을 확인해 채택·통합·보류한다. 현재 adapter·검증 순서·저장 방식을 대안에도 그대로 강제하지 않으며, 같은 사용자 목표·제품 경계는 유지한다.
2. **ASR 검토:** 남긴 구조적 선택과 steelman에 대해 관련 ASR의 추가·변경, 의미·우선순위·적용 범위·평가 기준을 논의한다. 직접 효과·조건부 효과·차이 없음·미해결을 구분하며 현재 네 ASR 밖의 품질도 검토한다.
3. **Rationale & Falsification:** 구조 차이의 인과 경로, 선택 구조의 비용·약점, 대안이 유리한 조건, 선택을 기각하거나 재검토할 반증 조건을 정식 package로 작성한다.
4. **Measurement & Revalidation:** 별도 승인 후 정의·모집단·실패 처리·측정 경계·target·score를 결과 전에 동결하고 검증한다. 현재 후보 검토만으로 시작하지 않는다.

Model Access scheduling, Agent 선택, 응답 제어, 프로세스 배치는 발견의 출발점이며 각각 package가 되는 것은 아니다. 모든 Component를 package로 만들지 않는다. 프로세스 수만으로 accuracy 우위를 주장하지 않고 단일 process 대안에도 충분한 thread·비동기·자원 제어를 허용한다.

메모리 사용량은 후속 ASR 검토의 중요 후보다. 공유 weights를 중복 집계하지 않고 역할별 KV·작업 공간·capture/pin·Context cache·IPC·queue·helper 비용과 동시 부하를 함께 고려한다. 최대 사용량·증가 상한·예산 부족 시 기능·다른 ASR과의 trade-off를 어떻게 정의할지 논의한다. QA-41의 diagnostic 지위나 기존 QA 정의·결과는 명시적 변경 전까지 유지한다.

정식 package에는 최소한 다음을 포함한다.

- 목표 구조와 구체적인 해결 문제, 관련 기준선 절·계약
- 같은 목표를 충족하는 가장 강한 현실적 대안
- Component·상태 소유권·계약·호출·배치·영속성·장애 경계의 실제 차이
- 비교 ASR의 정의·선정 또는 변경 근거, 인과 경로와 applicability
- 선택 구조의 비용·약점과 대안이 더 유리해지는 조건
- 검증 방법과 선택 구조를 기각·재검토할 반증 조건

그 과정에서 정의한 ASR 중 하나 이상에 실질적 영향을 주는 구조적 선택만 정식 package로 남긴다. 현재 네 ASR은 출발점이며 최종 목록·개수·정의를 고정하지 않는다. 기존 VIA-DP-01~18은 이전 작업의 reference이며 새 package의 분해 기준이나 번호 체계로 사용하지 않는다.

## 비교 원칙

- `accuracy → responsiveness → modifiability → reliability/recoverability`는 목표 Architecture를 선택·설계할 때의 우선순위다.
- 모든 package는 후보 구체화 과정에서 정한 ASR 전체를 `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED` 중 하나로 명시한다.
- 선택안과 steelman은 동일하게 동결한 모집단·실패 처리·측정 경계로 모든 applicable ASR을 각각 측정한다.
- 한 축의 결과로 후보를 먼저 제외한 뒤 다른 축을 측정하지 않는 방식은 사용하지 않는다.
- 선택안이 더 정확하지만 느리거나, 변경 범위가 작지만 복구율이 낮은 결과도 정상적인 trade-off로 보고한다.
- 비교 축을 하나의 가중 점수로 합치지 않는다. 최종 rationale에서 어떤 손해를 감수하고 왜 선택했는지 명시한다.
- Qualification 위반이 최종 선택을 막더라도 해당 후보의 측정값과 실패 evidence는 보존한다.

## 현재 근거의 범위

현재 검토는 기준선·상세 계약과의 문서 대조, 동일 사용자 행동 유지 여부, Component 변화의 실질성, 강한 대안과 비용, 후보 간 중복에 대한 자체 검토다. 독립 심사나 구현·성능 실험의 결과가 아니다. 목표 Architecture를 처음부터 재설계하지 않으며, 기존 accepted/deferred ADR의 상태와 caveat를 보존한다.
