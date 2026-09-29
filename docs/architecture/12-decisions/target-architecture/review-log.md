# VIA 목표 Architecture 검토 기록

> 갱신일: 2026-09-29
> [설계 개요](./README.md) · [현재 Architecture 제안](./architecture.md)

## 상태를 읽는 방법

- **사용자 지정:** 사용자가 명시한 목적·범위·진행 방식.
- **제안:** Architecture에 포함했지만 아직 사용자와 구조적으로 합의하지 않은 내용.
- **합의:** 사용자가 명시적으로 수용한 설계 내용. 문서 작성·게시·merge만으로 승격하지 않는다.
- **열림:** 세부 설계 또는 실제 dependency 확인이 필요한 항목.

현재 전체 Architecture는 **제안**이다. 구현·측정·성능 검증은 하지 않았다.

## 사용자가 지정한 방향

| 주제 | 내용 | 상태 |
| --- | --- | --- |
| 접근법 | 완성된 목표 Architecture를 먼저 설계하고 주요 구조 선택과 강한 대안은 그 이후 구성 | 사용자 지정 |
| 품질 우선순위 | 목표 Architecture 설계는 1순위 QA-19 semantic accuracy, 2순위 QA-09 responsiveness. 이후 Decision Package에서는 후보별 두 축을 독립 측정하고 accuracy–responsiveness trade-off를 그대로 비교 | 사용자 지정 |
| 경계 | VIA는 interaction·orchestration, Agent는 업무 추론·계획·도구·실행 | 사용자 지정 |
| 모델 | S2S 1개와 공유 semantic LLM 1개; Component·Task별 복제 금지 | 사용자 지정 |
| 기존 분해 | 기존 결정 번호·대안·순서에 구속되거나 다시 매핑하지 않음 | 사용자 지정 |
| 이름 | 더 직관적인 Component 이름으로 변경 가능 | 사용자 지정 |
| 진행 | 전체 구조부터 대화하며 수정하고, 합의 후에만 Decision Package·측정 근거 설계 | 사용자 지정 |
| 기록 | GitHub에서 문서를 정리하면서 계속 진행 | 사용자 지정 |
| 저장소 위치 | 목표 Architecture와 이후 Decision Package는 12번 작업 안에서 관리 | 사용자 지정 |
| Git 방식 | 이 Architecture 대화마다 새 branch와 PR을 만들지 않음 | 사용자 지정 |

초기에는 저장소 수정 없이 논의했고, 이후 사용자의 GitHub 정리 요청에 따라 이 문서를 추가했다. 그 요청은 전체 제안을 승인했다는 의미가 아니다.

## 현재 주 설계안의 상태

| 제안 | 상태 | 관련 절 |
| --- | --- | --- |
| 실시간 interaction·요청 처리·장기 업무의 동시 진행 | 제안 | [전체 흐름](./architecture.md#2-사용자-관점의-전체-흐름) |
| Request Controller와 Task Manager의 상태 소유권 분리 | 제안 | [Component](./architecture.md#4-component와-상태-소유권) |
| bounded read와 입력 revision당 semantic 최대 2회 | 제안 | [LLM 계약](./architecture.md#7-semantic-llm-계약과-호출-예산) |
| 보수적인 S2S 직접 응답과 host 출력 소유권 | 제안·admission 계약 열림 | [S2S 경로](./architecture.md#8-s2s와-직접-응답) |
| 당시 지칭 근거와 현재 실행 유효성의 구분 | 제안 | [Context](./architecture.md#9-contextcachestale-처리) |
| revision 검증·outbox·전송 경계에 따른 정정·취소 | 제안·원자성 세부 열림 | [동시성과 전송](./architecture.md#11-동시성정정취소전송) |
| event 기반 Task 상태와 필요 시 재조회 | 제안 | [장기 업무](./architecture.md#12-장기-업무복합-요청agent-event) |
| UI·Voice·Core·연동 worker 배치 | 제안 | [프로세스와 장애](./architecture.md#14-프로세스-배치fault-boundary) |
| host 계산 semantic commit과 bounded-completeness gate | 독립 검토 후 보강·세부 schema 열림 | [요청 확정](./architecture.md#6-요청-이해와-처리-확정) |
| S2S speculative 생성과 Controller direct admission | 독립 검토 후 보강·admission 기준 열림 | [S2S 경로](./architecture.md#8-s2s와-직접-응답) |
| command epoch 기반 dispatch 선형화 | 독립 검토 후 보강·Agent capability 열림 | [동시성과 전송](./architecture.md#11-동시성정정취소전송) |
| Agent inbox·Task projection·response publication 복구 | 독립 검토 후 보강·상태 전이 열림 | [장기 업무](./architecture.md#12-장기-업무복합-요청agent-event) |

## 다음 검토 주제

전체 Component와 흐름을 먼저 확인한 뒤, 아래 순서로 구체화한다. 이는 대화의 순서이며 구조 선택의 승패 의존 관계가 아니다.

| 순서 | 주제 | 확인할 질문 |
| --- | --- | --- |
| 1 | 전체 구조·책임 | Component가 과하거나 빠진 곳은 없는가? Controller와 Task Manager 경계가 자연스러운가? |
| 2 | Voice·S2S 직접 경로 | 직접 응답을 언제 허용하며 잘못된 routing을 어떻게 드러내는가? 시간·출력 제어 capability가 있는가? |
| 3 | Context·지칭 | 기본 근거와 추가 조회가 충분한가? 원문·이미지·시각·후보 누락을 어떻게 표현하는가? |
| 4 | 의미 해석 | 통합 해석과 최대 2회 예산이 사용자 시나리오를 충족하는가? 직접 답변 생성 비용은 적절한가? |
| 5 | 정정·취소·확정 | 전송 시작과 새 입력의 순서를 어디서 확정하는가? source version 확인은 Agent와 어떻게 나누는가? |
| 6 | 장기 업무·복구 | Agent event·조회 capability별로 무엇을 보장할 수 있는가? 중복 실행 불명 상태는 어떻게 처리하는가? |
| 7 | 배치·자원 | local/remote 모델 배치, queue·memory·증거 보관·deadline 예산을 어떻게 정할 것인가? |

## 아직 하지 않은 일

- 전체 목표 Architecture에 대한 사용자 합의
- 필수 S2S·semantic LLM·Agent capability 확인
- machine-readable schema, 전송·제어 event ordering의 상세 계약
- 수치형 timeout·자원·보관 예산의 확정
- 후보 구현, 성능 측정, 실제 모델·제품 경로 검증
- 새 Decision Package와 강한 대안·구체 실험 구성

## 변경 기록

| 날짜 | 변경 | 의미상 합의 상태 |
| --- | --- | --- |
| 2026-09-29 | 대화의 전체 설계안을 개요·Architecture·검토 기록으로 문서화 | 사용자 지정 작업 방식 기록; 세부 구조는 제안 상태 유지 |
| 2026-09-29 | 목표 Architecture를 12번 안으로 이동하고, 합의 후 구조적 선택을 역으로 추출하는 저장소 흐름으로 변경 | 작업 흐름은 사용자 지정; 세부 구조는 제안 상태 유지 |
| 2026-09-29 | 목표 Architecture 품질 우선순위를 QA-19 semantic accuracy 1순위, QA-09 responsiveness 2순위로 명시 | 사용자 지정; 세부 구조는 제안 상태 유지 |
| 2026-09-29 | 세 독립 관점의 사전 검토를 기록하고, 공통으로 확인된 S2S admission·evidence coverage·dispatch 선형화·event/response 복구 공백을 본문에 보강 | reviewer 지적 반영; 보강 구조는 사용자 검토 전 제안 상태 |
| 2026-09-29 | accuracy 우선순위를 목표 Architecture 설계 원칙과 후보 비교 방법으로 분리하고, Decision Package에서는 accuracy·responsiveness를 독립 측정하도록 정정 | 사용자 지정; 기존 blanket gate 철회 |

이후 수정 때는 바뀐 구조·이유·합의 상태를 이 표에 남긴다. 과거 문구의 전체 이력은 Git으로 보존한다.
