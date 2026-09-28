# VIA Architecture Decisions

VIA-DP-01~18은 전체 decision inventory이며 번호는 우선순위가 아니다. 2026-09-28 사용자 결정으로 **VIA-DP-03·05·06·07·15·17을 핵심 DP shortlist**로 선정했다. QA-09·19·29·39는 네 core ASR로 확정했지만, 여섯 DP의 상세 A/B Measurement Freeze와 새 계약에 따른 실측·승자 선정은 완료하지 않았다. VIA-DP-02·12는 supporting decision, VIA-DP-11은 실제 fatal-risk integration inventory가 확인될 때 재승격하는 조건부 보조 DP다.

## 어디부터 읽을까?

[전체 요약 보고서](./dp-executive-summary.md)를 먼저 읽고 필요한 개별 DP로 이동한다. 요약에는 책임 지도, 후보 간 경계, 미완료 사항과 이전 번호의 추적성 부록이 포함돼 있다.

| 문서 | 목적 | 언제 읽는가 |
| --- | --- | --- |
| [전체 요약](./dp-executive-summary.md) | 18개 DP와 시스템 구조 이해 | 처음 읽을 때 |
| 아래 개별 VIA-DP | 배경·A/B 구현·사고실험·네 core ASR 적용과 상세 진단 | 해당 결정을 검토할 때 |
| [검토 protocol](./dp-review-protocol.md) | 배타성·steelman·보고서·그림 작성 기준 | DP를 작성·수정·검토할 때 |
| [평가 방법](./evaluation-method.md) | A/B 측정과 QA 판별·평가 왜곡 방지 | 실제 비교·평가를 준비할 때 |
| [공통 계약](./common-contract.md) | 동일 모델·identity·메시지·상태·실패 처리 조건 | 대안을 구체화할 때 |

## 개별 보고서

- [VIA-DP-01 — 범위가 정해진 정보 처리의 책임 경계](./via-dp-01-direct-handling.md)
- [VIA-DP-02 — 대화와 Task 관계의 확정 경계](./via-dp-02-state-consistency.md)
- [VIA-DP-03 — 음성 입력 근거의 최종 기준](./via-dp-03-voice-evidence.md)
- [VIA-DP-04 — S2S 직접 응답의 게시 권한](./via-dp-04-response-authority.md)
- [VIA-DP-05 — 요청 Context의 획득 계획 확정 계약](./via-dp-05-context-contract.md)
- [VIA-DP-06 — 요청 의미의 최종 확정 권한](./via-dp-06-semantic-authority.md)
- [VIA-DP-07 — 복합 요청 graph의 dependency 실행 권한](./via-dp-07-compound-orchestration.md)
- [VIA-DP-08 — 재시작 후 상태의 기준 기록](./via-dp-08-recovery-source.md)
- [VIA-DP-09 — Agent 수명 계약의 의미 해석 위치](./via-dp-09-agent-semantics.md)
- [VIA-DP-10 — Model 세션·연결 수명의 관리 권한](./via-dp-10-model-session-authority.md)
- [VIA-DP-11 — 외부 연동 코드의 Process 장애 경계](./via-dp-11-process-isolation.md)
- [VIA-DP-12 — 응답 게시와 실행 근거의 영속 확정 순서](./via-dp-12-evidence-commit.md)
- [VIA-DP-13 — 사용자 제어를 위한 실행 자원을 예약할 것인가](./via-dp-13-control-reservation.md)
- [VIA-DP-14 — Task 상태 전이의 소유권](./via-dp-14-task-state-authority.md)
- [VIA-DP-15 — Agent progress state의 유지 방식](./via-dp-15-agent-observation-authority.md)
- [VIA-DP-16 — 모델 입력 이력의 구성·유지 책임](./via-dp-16-model-context-state.md)
- [VIA-DP-17 — Context 표현의 최종 의미 소유권](./via-dp-17-context-materialization-authority.md)
- [VIA-DP-18 — 보호정보·Action 사용 시 권한을 확인하는 위치](./via-dp-18-authorization-enforcement.md)

## 현재 기준과 이력

S2S 1개·semantic LLM 1개를 유지하며 Component·Task별 모델을 추가 적재하지 않는다. VIA Client와 외부 Agent Runtime을 구별한다. QA는 [현행 catalog](../08-quality-attributes/quality-model.md), 기존 accepted/deferred 결정은 [ADR](../../adr/README.md)을 따른다. 새 보고서 작성은 재승인이나 실측 승자 선정을 뜻하지 않는다.

역할이 끝난 중간 문서·이전 경로 안내는 제거했다. 원문과 당시 검증 기록은 [문서 통합 이력](../../archive/dp-document-consolidation-2026-09-25/README.md)에 보존하며 현재 요구의 근거로 사용하지 않는다.
