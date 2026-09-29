# VIA 목표 Architecture 검토 기록

> 갱신일: 2026-09-29
> [설계 개요](./README.md) · [현재 Architecture 제안](./architecture.md)

## 상태를 읽는 방법

- **사용자 지정:** 사용자가 명시한 목적·범위·진행 방식.
- **제안:** Architecture에 포함했지만 아직 사용자와 구조적으로 합의하지 않은 내용.
- **합의:** 사용자가 명시적으로 수용한 설계 내용. 문서 작성·게시·merge만으로 승격하지 않는다.
- **열림:** 세부 설계 또는 실제 dependency 확인이 필요한 항목.

현재 전체 Architecture는 **제안**이다. 구현·측정·성능 검증은 하지 않았다. 모델 기능의 공식 문서 1차 확인은 [확인 원장](./model-capability-review.md)에 따로 기록한다.

## 사용자가 지정한 방향

| 주제 | 내용 | 상태 |
| --- | --- | --- |
| 접근법 | 완성된 목표 Architecture를 먼저 설계하고 주요 구조 선택과 강한 대안은 그 이후 구성 | 사용자 지정 |
| 품질 우선순위 | 목표 Architecture 설계는 1순위 QA-19 semantic accuracy, 2순위 QA-09 responsiveness, 3순위 QA-29 modifiability, 4순위 QA-39 reliability/recoverability. 네 ASR 모두 필수 | 사용자 지정 |
| 경계 | VIA는 interaction·orchestration, Agent는 업무 추론·계획·도구·실행 | 사용자 지정 |
| 모델 | on-device 공유 Omni 1개, 음성·semantic 역할 분리, Component·Task별 가중치 복제 금지; 필요 ASR 허용 | 사용자 지정·이전 두 모델 전제 대체 |
| 기존 분해 | 기존 결정 번호·대안·순서에 구속되거나 다시 매핑하지 않음 | 사용자 지정 |
| 이름 | 더 직관적인 Component 이름으로 변경 가능 | 사용자 지정 |
| 진행 | 전체 구조부터 대화하며 수정하고, 합의 후에만 Decision Package·측정 근거 설계 | 사용자 지정 |
| 이후 비교 | 모든 Decision Package는 네 core ASR을 모두 applicability 분류하고 applicable 축을 독립 측정 | 사용자 지정·Architecture 합의 이후 적용 |
| 기록 | GitHub에서 문서를 정리하면서 계속 진행 | 사용자 지정 |
| 저장소 위치 | 목표 Architecture와 이후 Decision Package는 12번 작업 안에서 관리 | 사용자 지정 |
| Git 방식 | 이 Architecture 대화마다 새 branch와 PR을 만들지 않음 | 사용자 지정 |
| 그림 | 목표 Architecture의 모든 구조 그림은 draw.io 원본과 GitHub용 SVG를 함께 두고 Markdown에 삽입; Mermaid는 사용하지 않음 | 사용자 지정 |

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
| domain event outbox·dispatch CAS·DELIVERY_UNKNOWN | 그림 기반 점검에서 주 설계 구체화·구현 미검증 | [설계 완결성 점검](./design-completeness.md) |
| Voice/Core client + 공유 Omni service + 독립 ASR worker | 주안 개정·실제 자원/기능 확인 필요 | [프로세스와 장애](./architecture.md#14-프로세스-배치fault-boundary) |
| 최소 S2S 직접 응답 | 명백한 자체 지식 질문만 허용하는 방향 합의; 최소 admission 계약 열림 | [S2S 경로](./architecture.md#8-s2s와-직접-응답) |

## 이번 대화에서 확인한 제품 행동

| 내용 | 상태 |
| --- | --- |
| 제한된 추가 조회 후 지칭 후보가 둘 이상이면 질문 | 합의 |
| 새 발화 때 같은 대화의 미전송 요청을 잠시 보류 | 합의 |
| 한정 자료 설명·요약은 VIA, 조사·업무 계획·실행은 Agent; semantic LLM이 handling 판단 | 합의; host가 고정 scope·권한을 적용하는 구조는 유지 |
| 화면은 상세 결과, 음성은 듣기 좋은 핵심 요약 | 사용자 지정 |
| 사용자 발화 중 음성 알림 금지; 결과 화면 표시 후 발화 종료 때 음성 전달 | 사용자 지정; 차례·새 요청 우선순위·Voice 재연결 세부는 제안 |
| 중단된 답변을 계속할지 사용자 확인 | 불명확할 때만 확인하는 정책 합의; Request Interpreter가 의미 해석 |
| S2S 자체 지식 질문만 직접 응답 | 사용자 지정; 첫 단순 질문도 허용, 대화 지칭·자료·Task 해석은 Core. 후속 대화 fast path 확장안은 미채택 |
| 한 업무는 통째로 Agent, 독립 업무는 별도 Task | 합의; 요약 후 발송은 한 Task. 동일 Agent를 써도 독립 업무 identity는 분리 |
| 기억 설계·모델 기능 확인 | 사용자 요청에 따라 진행; 기억 세부 정책·모델 적합성은 완료 아님 |

## 공유 Omni 전환에서 확인한 것

- 사용자 지정: 하나의 Omni를 두 역할로 사용하며 semantic 중에도 발화 수신·인식과 동시 사용을 지원한다. On-device 약 10B 개발 방향이며 reference는 일부 fine-tuning한 Qwen3-Omni-30B-A3B-Instruct다.
- 사용자 위임: 모델팀 계약을 이 설계에서 정의하고 필요한 ASR을 포함할 수 있다.
- 주안: 독립 Streaming ASR, 단일 shared inference owner, 역할별 KV·권한, Voice 연산 예약·chunked scheduling·semantic 최소 진행량. 세부안은 사용자 검토 전이며 [상세 문서](./shared-omni-runtime.md)에 있다.
- 공식 보고서에서 30B가 Thinker 크기임을 확인했다. 같은 기준의 10B와 전체 모델·메모리 합계를 구분한다. 품질 동등성·PC 성능은 미검증이다.

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
| 7 | 변화 격리 | Model·Agent·Context source·schema 변화가 adapter 밖으로 얼마나 퍼지는가? canonical contract와 migration 경계가 충분한가? |
| 8 | 장애·복구 | command·event·publication crash window와 process·Store 장애에서 중복·오연결 없이 수렴하는가? |
| 9 | 배치·자원 | local/remote 모델 배치, queue·memory·증거 보관·deadline 예산을 어떻게 정할 것인가? |

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
| 2026-09-29 | 목표 Architecture 우선순위를 QA-19 → QA-09 → QA-29 → QA-39로 확장하고 네 ASR을 모두 설계에 반영; Decision Package 작업은 전체 구조 합의 이후로 유지 | 사용자 지정; 목표 Architecture 검토에 집중 |
| 2026-09-29 | `architecture.md`의 Mermaid·ASCII 구조 그림을 7개 draw.io 그림과 SVG preview로 교체·보강하고, 동일한 시각 언어와 생성 절차를 추가 | 사용자 지정 표현 방식 반영; Architecture 내용은 제안 상태 유지 |
| 2026-09-29 | 전체 구조 그림의 핵심 Component 배치와 edge routing을 재구성하고 Policy 판단의 Controller 적용, State Store의 durable infrastructure 위치, Controller가 중재하는 Context·Interpreter 관계를 명시 | 사용자 리뷰 반영; 구조는 계속 제안 상태 |
| 2026-09-29 | Interaction Runtime을 Channel I/O·Evidence Capture·Timeline & Buffer로 펼치고 Model Access의 S2S client 관계와 직접 S2S 응답의 Response Manager publication protocol을 명시; 색상·화살표 legend 추가 | 사용자 리뷰 반영; 내부 모듈과 protocol은 제안 상태 |
| 2026-09-29 | 전체 그림을 동일 수준 Component로 재배치하고 Interaction Manager로 명칭 변경; VIA 경계 안에 공유 Store·Model Access 배치, Task Manager의 Gateway 요청과 Gateway→Task→Controller→Response 결과 경로, raw audio와 요청 이벤트의 구분을 명시; publication admission의 근거와 M3 응답 생성 경로 보완 | 사용자 리뷰 반영; 전체 구조는 제안 상태이며 모델 기능·실제 성능 미검증 |
| 2026-09-29 | 그림 02~07 재구성, 08 응답·중단 / 09 복합 요청 / 10 Context·권한·기억 추가; 알림 유실·dispatch 경쟁·전달 불명·Model Access 배치·실제 사용 시 권한 검사·queue 포화 동작 보강; 18개 UC의 경로와 열린 질문 점검 | 사용자 요청에 따른 자체 설계 검토; 문서 완결성 보강이며 실제 구현·모델·성능 검증 및 전체 구조 합의는 아님 |
| 2026-09-29 | 지칭 clarification·미전송 보류·VIA/Agent 경계·사용자 발화 비중단·화면 상세/음성 요약 원칙 반영; 그림 08에 알림 대기 경로 추가; S2S 조건부 직접 경로·중단 후 재개·복합 실패·기억 계층 검토안 기록 | 명시한 제품 행동만 합의/사용자 지정; 구조 제안과 실제 모델·성능은 열림 |
| 2026-09-29 | S2S를 명백한 독립 자체 지식 질문으로 축소; 후속 대화 fast path 제안 철회; 요약·발송을 한 Task로 위임하도록 본문·그림 09 교정; 재개 정책 합의 기록; 모델 기능 공식 문서 1차 확인 | 사용자 지정 범위 반영; 최소 admission·기억·실제 모델 연동은 계속 설계/확인 중 |
| 2026-09-29 | 사용자 지정에 따라 두 모델 전제를 공유 on-device Omni·역할 분리로 개정; 별도 ASR·동시 입력·자원 예약 주안, Qwen parameter 근거와 모델팀 계약, 그림 01·06·08 개정 및 11 추가 | 공유 방향·동시 입력·필요 ASR 허용은 사용자 지정; 구체 배치·스케줄링·모델 실현은 제안/미검증 |

이후 수정 때는 바뀐 구조·이유·합의 상태를 이 표에 남긴다. 과거 문구의 전체 이력은 Git으로 보존한다.
