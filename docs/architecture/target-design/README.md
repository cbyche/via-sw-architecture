# VIA Target Architecture — 설계 작업 공간

> 상태: **설계 초안 / 전체 구조 합의 전 / 구현·측정 없음**
> 시작일: 2026-09-29
> 목적: responsiveness와 VIA semantic accuracy를 우선하는 완성된 목표 Architecture를 먼저 설계하고, GitHub에서 대화와 함께 수정한다.

## 지금 읽을 문서

| 문서 | 역할 |
| --- | --- |
| [전체 Architecture](./architecture.md) | Component, 상태 소유권, 계약, 호출 흐름, 동시성, Context, 장애 복구와 위험 |
| [검토 기록](./review-log.md) | 사용자가 정한 방향, 제안 상태, 미합의 사항, 다음 검토 주제와 변경 기록 |

이 디렉터리는 현재 논의 중인 **제안**의 source of truth다. 문서에 포함되었다는 이유로 승인된 제품 Architecture나 구현된 기능이 되는 것은 아니다. [기존 Architecture 기준선](../README.md), accepted ADR, 기존 평가 상태를 이 초안이 자동으로 대체하지 않는다.

## 작업 순서

```text
완성된 목표 Architecture 설계·대화·수정
    ↓ 전체 구조 합의 이후
responsiveness / accuracy에 중요한 구조적 선택 추출
    ↓
강한 현실적 대안과 비용·유리한 조건 정리
    ↓
선택 근거와 반증 조건을 갖춘 검증 설계
```

이번 작업은 solution-first 방식이다. 기존 VIA-DP-01~18의 분해·대안·검토 순서에 맞추거나 새 구조를 기존 번호에 매핑하지 않는다. 모든 Component를 Decision Package로 만들지도 않는다.

전체 Architecture 합의 전에는 새 Decision Package, A/B winner, 측정 freeze를 만들지 않는다. 합의 후에도 강한 대안, 선택한 구조의 비용, 대안이 더 유리한 조건과 반증 가능성을 유지한다. 측정 결과에 맞춰 정의를 바꾸거나 결론에 맞게 결과를 고르지 않는다.

## 고정하는 제품 경계

- VIA는 PC의 Voice/Text/screen interaction과 orchestration을 담당한다.
- Downstream Agent는 domain reasoning, planning, tool selection, 실제 업무 수행을 담당한다.
- VIA는 S2S 1개와 공유 semantic LLM 1개를 사용한다. Component·Task별 모델 복제나 임의 helper 모델 추가는 하지 않는다.
- accuracy는 사용자 목표·대상·Task·처리 방향·위임 내용의 정확성이다. Agent가 만든 도메인 결과의 품질과 구분한다.
- responsiveness는 유효한 응답·clarification·위임·진행·중단까지의 VIA 책임 경로를 다룬다. 모델 시간이나 접수 멘트만으로 설명하지 않는다.
- Voice 연결, Conversation, Request, Task, Agent Execution의 수명을 분리한다.

제품 요구의 근거는 [Mission & Boundary](../01-system-mission-and-boundary.md), [Fixed Scope](../03-fixed-architecture-scope.md), [Representative Use Cases](../05-representative-use-cases.md)다. 용어는 [Terms](../02-terms.md), 의미상 정확성의 범위는 [Correctness & Continuity](../08-quality-attributes/correctness-and-continuity.md)를 참고한다.

## GitHub에서 계속 정리하는 방식

1. 설계 내용이 달라지면 `architecture.md`의 현재 제안을 수정한다.
2. 사용자와 명시적으로 합의한 내용, 열린 쟁점과 변경 이유는 `review-log.md`에 남긴다.
3. 문서화·PR 생성·merge를 Architecture의 의미상 승인과 혼동하지 않는다. 합의 상태는 별도로 표시한다.
4. 각 변경은 기능 경계와 문서 링크·용어를 확인한 뒤 scoped commit과 PR로 검토한다.
5. 채팅에만 남은 제안을 구현 완료·성능 확인으로 바꾸지 않는다. 실제 근거가 생기기 전까지 성능은 가설이다.

## 현재 산출물의 한계

논리 구조와 runtime 동작을 제안한 상태다. 모델 capability 확보, 세부 schema, 수치형 resource/deadline 예산, 구현, 실제 responsiveness·accuracy 검증은 남아 있다. [검토 기록](./review-log.md)의 항목을 순서대로 좁혀가며 전체 구조 합의 여부를 판단한다.
