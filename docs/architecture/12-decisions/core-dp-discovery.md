# VIA Architecture Decision 후보 지도

> 2026-09-25 · 전수 inventory · 선정 순위가 아님

VIA는 사용자 입력의 의미를 이해하고 직접 응답하거나 외부 Agent에 위임한 뒤, 비동기 상태와 결과를 같은 대화·Task로 잇는다. 이 책임을 실제로 구성하는 독립 질문을 18개 보고서로 관리한다. 문서 수가 확정 핵심 DP 수는 아니다.

## 시스템 책임에서 출발하는 읽기 지도

| 책임 영역 | 질문과 담당 VIA-DP |
| --- | --- |
| 입력·화면 근거 | 03 입력 revision의 의미 계약 |
| Context | 05 읽기 집합의 확장 권한, 17 source 값 변환 owner, 16 모델 입력 이력 유지 |
| 요청 이해·실행 범위 | 06 의미 확정, 01 bounded 직접 실행, 07 명시된 복합 관계 실행 |
| 대화·Task 상태 | 02 교차 관계 commit, 14 Task writer, 08 복구 원본 |
| 외부 Agent | 09 수명 의미 해석, 15 상태 확정 근거 |
| 모델·사용자 응답 | 10 세션 수명, 04 게시 승인 |
| 권한·실행 경계 | 18 protected use 승인, 11 VIA Client Process, 13 제어 여력 |
| 연구 기록 | 12 게시 전 실행 근거의 영속 확인 |

## 전체 후보와 대안

| 독립 보고서 | 대안 A | 대안 B |
| --- | --- | --- |
| [VIA-DP-01 범위가 정해진 정보 처리의 책임 경계](./via-dp-01-direct-handling.md) | 선택적 직접 처리 + Agent 위임 | 정보 처리 실행의 Agent 일원화 |
| [VIA-DP-02 대화와 Task 관계의 확정 경계](./via-dp-02-state-consistency.md) | 분리된 처리 책임 + 공동 원자 커밋 | 독립 상태 확정 + 관계 조정 |
| [VIA-DP-03 음성 입력 근거의 최종 기준](./via-dp-03-voice-evidence.md) | VIA 입력 계약 + 비모델 정렬·정규화 | S2S 입력 계약을 기준으로 사용 |
| [VIA-DP-04 S2S 직접 응답의 게시 권한](./via-dp-04-response-authority.md) | 한정 권한 위임 + 범위 밖 Core 승인 | 모든 직접 응답에 Core 요청별 승인 |
| [VIA-DP-05 요청 Context의 읽기 집합 확정 계약](./via-dp-05-context-contract.md) | 불변 입력 명세 + 필요한 값만 지연 적재 | 범위 제한 조회 권한 + 처리 중 입력 확장 |
| [VIA-DP-06 요청 의미의 최종 확정 권한](./via-dp-06-semantic-authority.md) | 단계별 보조 처리 + 통합 최종 확정 | 단계별 의미 권한 + 명시적 정정 계약 |
| [VIA-DP-07 복합 요청 관계의 실행 책임](./via-dp-07-compound-orchestration.md) | VIA 관계 조정 + 가능한 부분의 묶음 위임 | 복합 업무 전체의 Agent 조정 |
| [VIA-DP-08 재시작 후 상태의 기준 기록](./via-dp-08-recovery-source.md) | 상태 변경 이력 + 검증된 checkpoint | 현재 상태 + 미완료 동작 + 감사 이력 |
| [VIA-DP-09 Agent 수명 계약의 의미 해석 위치](./via-dp-09-agent-semantics.md) | 공통 의미 정규화 + 손실 없는 확장 | 공통 전송·타입 계약 + Core 유형별 의미 확정 |
| [VIA-DP-10 Model 세션·연결 수명의 관리 권한](./via-dp-10-model-session-authority.md) | 공통 세션 관리자 + 역할별 직접 stream | 역할별 세션 소유 + 공통 adapter library |
| [VIA-DP-11 외부 연동 코드의 Process 장애 경계](./via-dp-11-process-isolation.md) | 위험 연동 격리 + 얇은 Core 연결부 | 같은 Process + 제한된 queue·실패 처리 |
| [VIA-DP-12 응답 게시와 실행 근거의 영속 확정 순서](./via-dp-12-evidence-commit.md) | 최소 근거 선확정 + 상세 자료 비동기 수집 | 게시와 영속 기록의 비동기 분리 |
| [VIA-DP-13 사용자 제어를 위한 실행 자원을 예약할 것인가](./via-dp-13-control-reservation.md) | 제어 여력 예약 + 회수 가능한 유휴 자원 공유 | 전체 자원 공유 + 우선순위 기반 제어 우대 |
| [VIA-DP-14 Task 상태 전이의 소유권](./via-dp-14-task-state-authority.md) | 공유 transactional Task 서비스 | Task별 단일 writer supervisor |
| [VIA-DP-15 Agent 상태를 확정하는 관측 경로](./via-dp-15-agent-observation-authority.md) | 유효 event 확정 + query 복구 | Event 알림 + query 확인 후 확정 |
| [VIA-DP-16 모델 입력 이력의 구성·유지 책임](./via-dp-16-model-context-state.md) | 요청별 재구성 + version 검증 cache | 증분 working context + 필요 시 재구성 |
| [VIA-DP-17 Source를 소비 가능한 Context로 만드는 책임](./via-dp-17-context-materialization-authority.md) | 공통 materializer + 소비자별 projection | 공통 접근 handle + 소비자 소유 변환 |
| [VIA-DP-18 보호정보·Action 사용 시 권한을 확인하는 위치](./via-dp-18-authorization-enforcement.md) | 사용마다 중앙 승인 + 사전 준비 cache | 철회 가능한 capability + 로컬 use gate |

## 겹쳐 보이지만 다른 결정

- **02 vs 14:** 여러 상태의 관계를 함께 commit하는가 vs 한 Task에 누가 쓰는가.
- **05 vs 17 vs 16:** 어떤 source를 읽을 수 있는가 vs 값을 누가 만드는가 vs 대화 이력을 어떻게 유지하는가.
- **09 vs 15:** Agent 사건의 의미를 어디서 해석하는가 vs event 자체로 상태를 확정할 수 있는가.
- **04 vs 18:** 응답 게시 권한 vs 보호정보·Action의 사용 권한.
- **10 vs 11 vs 13:** 모델 세션 권한 vs VIA Client의 Process 경계 vs 일반 작업의 제어 자원 점유 권한.
- **08 vs 12:** 운영 상태 복구의 원본 vs 사용자 응답 전 연구 근거의 기록 완료 의무.

같은 제품에서 이 축들은 조합할 수 있다. 한 DP의 A와 B를 같은 조건의 최종 권한으로 동시에 채택할 수 있다는 뜻은 아니다. [이력·누락 점검](./legacy-dp-mapping.md)은 기존 계열과 주제가 어디로 갔는지 설명한다. [요약](./dp-executive-summary.md)은 실제 구조 수준으로 읽는 출발점이다.
