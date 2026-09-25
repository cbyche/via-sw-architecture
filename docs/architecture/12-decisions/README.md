# Architecture Decisions — 전체 후보 18개

**현재 목적은 가능한 Architecture 의사결정을 빠짐없이 구체화하는 것이다.** 최종 발표용 4~5개 DP나 4~6개 ASR을 지금 선정하지 않는다. 과거의 ‘우선 핵심·보조’ 분류는 후보 누락·자동 제외의 근거가 아니다.

## 읽는 순서

1. [전체 요약 보고서](./dp-executive-summary.md): 시스템 구조와 DP의 실제 구현 차이.
2. [후보 지도](./core-dp-discovery.md): 서로 다른 질문·겹치기 쉬운 경계.
3. 아래 VIA-DP 독립 보고서: 배경 → A/B 구현도·동작 → 배타성 → 사고실험 → 전체 19개 QA → 판단.
4. [이전 번호 매핑·누락 점검](./legacy-dp-mapping.md): 기존 9개 질문·25개 주제와 현행 책임 연결.
5. [정리·검증 기록](./dp-review-synthesis.md): 완료 범위·미확인 사항·검증 상태.

## 공통 제약

- S2S 모델 1개와 semantic LLM 1개. Component·Task·단계별 추가 적재 없음. 역할별 프롬프트·세션은 공유 모델 사용.
- 외부 Agent Runtime은 자체 Process/원격 dependency. VIA Client를 별도 worker에 두는 것과 Agent를 embed하는 것은 다름.
- 현재 QA 번호만 사용: 01~05, 11~15, 21~23, 31/32, 41, 51, 61/62. 단일 metric 원칙 유지.
- QA-41은 자원 확인 대상으로 유지하되 메모리 상한·유의미한 구조 차이 근거 없이 핵심 ASR로 추천하지 않음.
- 모든 현재 QA 결과는 `NOT_RUN`. 후보 명세·기존 부분 코드·제품 검증을 구별함.

## Detailed review reports

ID 순서는 우선순위가 아니다. 같은 이름의 공통 Component, 명시된 Process 경계, queue/buffer·영속 기록과 메시지 흐름을 A/B에서 대응해 읽는다.

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

## Existing ADR status

새 번호로 설명을 완결하되 과거 승인 기록을 삭제하거나 재승인하지 않는다.

| 현행 DP | 기존 결정 상태 | 이력 |
| --- | --- | --- |
| VIA-DP-06 | Deferred; A는 interim reference | ADR-004 / IR-DP01 |
| VIA-DP-09 | A accepted; 현행 change pack 근거 재확인 | ADR-001 / AGENT-DP01 |
| VIA-DP-11 | 현행 A 방향에 해당하는 기존 격리안 accepted; 현행 제품 범위·QA 재검증 필요 | ADR-003 / EXEC-DP01의 B |
| VIA-DP-14 | B accepted; 현행 Voice·상태·복구 QA 재검증 필요 | ADR-002 / TASK-DP01 |

핵심 이해를 위해 옛 후보를 다시 읽을 필요는 없다. 정확한 이력은 [매핑](./legacy-dp-mapping.md), 승인 원문은 [ADR index](../../adr/README.md)에 남긴다. accepted 상태는 현재 실측 승자를 뜻하지 않는다.

## 공통 작성·평가 계약

[Review protocol](./dp-review-protocol.md), [그림 기준](./dp-diagram-guide.md), [평가 방법](./evaluation-method.md)을 적용한다. 동일 범위에서 mutually exclusive한 steelman A/B를 구성하며 합리적 hybrid를 먼저 반영한다. 다른 DP·기능·dependency 조건을 고정하고 실제 참여 QA만 비교한다. 충분한 trade-off 미입증도 정상적인 결론이다.
