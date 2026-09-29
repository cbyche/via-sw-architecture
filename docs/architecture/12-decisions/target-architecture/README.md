# VIA Target Architecture — 설계 작업 공간

> 상태: **MAJOR_DESIGN_COMPLETE / 주요 설계 완성안 / 사용자 최종 검토 전 / 구현·측정 없음**
> 시작일: 2026-09-29
> 목적: QA-19 semantic accuracy, QA-09 responsiveness, QA-29 modifiability, QA-39 reliability/recoverability 순으로 우선하되 네 ASR을 모두 고려한 목표 Architecture를 먼저 완성한다.

## 지금 읽을 문서

| 문서 | 역할 |
| --- | --- |
| [전체 Architecture](./architecture.md) | Component, 상태 소유권, 계약, 호출 흐름, 동시성, Context, 장애 복구와 위험 |
| [판단·제어·대화 계약](./control-and-lifecycle.md) | S2S admission, 입력 확정, 질문·승인·Task·Command·출력 상태, 정정·전송 경쟁, 재연결·복구 |
| [Context·기억 수명 계약](./memory-and-context-lifecycle.md) | 후보 coverage, 원본·요약·cache, 보관 기본값, 기억 삭제·철회, 파일/DB 복구 |
| [Interaction·업무 경계·기억 검토](./interaction-and-memory-design.md) | S2S 조건부 직접 경로, 사용자 합의, 모델 확인 책임, 복합 업무·기억 개요와 상세 계약 연결 |
| [공유 Omni·동시 처리 설계](./shared-omni-runtime.md) | 역할 분리, 입력 ASR, 자원 예약·동시 세션·과부하·모델팀 요구 계약 |
| [모델 기능 확인](./model-capability-review.md) | Qwen3-Omni 30B 표기의 의미, 공개 기능 근거, 실제 통합 확인이 필요한 항목 |
| [Architecture 그림](./diagrams/README.md) | 본문에 삽입한 SVG preview, draw.io 편집 원본과 색상 규칙 |
| [설계 완결성 점검](./design-completeness.md) | 18개 UC·필수 변형, 주요 선택과 사건 순서 점검, 설계 완료와 후속 구현의 구분 |
| [독립 검토 결과](./independent-review-2026-09-29.md) | 사용자 검토 전 세 관점의 공통 결함, 본문 반영 내용과 열린 질문 |
| [검토 기록](./review-log.md) | 사용자가 정한 방향, 제안 상태, 미합의 사항, 다음 검토 주제와 변경 기록 |

이 디렉터리는 12번 작업의 첫 단계인 **목표 Architecture 제안**의 source of truth다. 문서에 포함되었다는 이유로 승인된 제품 Architecture나 구현된 기능이 되는 것은 아니다. [Architecture 기준선](../../README.md), accepted ADR, 기존 평가 상태를 이 설계안이 자동으로 대체하지 않는다.

## 작업 순서

```text
완성된 목표 Architecture 설계·대화·수정
    ↓ 전체 구조 합의 이후
네 core ASR에 중요한 구조적 선택 추출
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
- VIA는 on-device Omni 1개를 음성·semantic 두 역할이 공유한다. 역할·session·권한은 분리하고 가중치는 복제하지 않는다. Semantic 중에도 발화 수신·인식을 유지하며 경량 Streaming ASR을 주안에 명시한다. 추가 helper는 필요성과 전체 비용을 드러낸다.
- 목표 Architecture의 품질 우선순위는 QA-19 semantic accuracy, QA-09 responsiveness, QA-29 modifiability, QA-39 reliability/recoverability 순이다. 낮은 순위도 설계에서 생략하지 않는다.
- accuracy는 사용자 목표·대상·Task·처리 방향·위임 내용의 정확성이다. Agent가 만든 도메인 결과의 품질과 구분한다.
- responsiveness는 유효한 응답·clarification·위임·진행·중단까지의 VIA 책임 경로를 다룬다. 모델 시간이나 접수 멘트만으로 설명하지 않는다.
- modifiability는 Agent·Model·Context source·계약 변화가 VIA의 책임·interface·state·runtime에 퍼지는 범위를 다룬다.
- reliability/recoverability는 dependency·process·network·Store 장애를 가두고 중복·오연결 없이 확인 가능한 상태로 복구하는 능력을 다룬다.
- Voice 연결, Conversation, Request, Task, Agent Execution의 수명을 분리한다.

제품 요구의 근거는 [Mission & Boundary](../../01-system-mission-and-boundary.md), [Fixed Scope](../../03-fixed-architecture-scope.md), [Representative Use Cases](../../05-representative-use-cases.md)다. 용어는 [Terms](../../02-terms.md), 의미상 정확성의 범위는 [Correctness & Continuity](../../08-quality-attributes/correctness-and-continuity.md)를 참고한다.

## 저장소에서 계속 정리하는 방식

1. 설계 내용이 달라지면 `architecture.md`의 현재 제안을 수정한다.
2. 사용자와 명시적으로 합의한 내용, 열린 쟁점과 변경 이유는 `review-log.md`에 남긴다.
3. 문서화·commit을 Architecture의 의미상 승인과 혼동하지 않는다. 합의 상태는 별도로 표시한다.
4. 별도 작업 branch·PR을 매번 만들지 않고 현재 Architecture 흐름 안에서 문서와 검토 기록을 함께 갱신한다.
5. 채팅에만 남은 제안을 구현 완료·성능 확인으로 바꾸지 않는다. 실제 근거가 생기기 전까지 성능은 가설이다.
6. 목표 Architecture 그림은 `.drawio` 편집 원본과 같은 이름의 `.svg` preview를 함께 관리한다. Markdown에는 SVG를 삽입하고 draw.io 원본을 바로 연결한다.

## 현재 산출물의 한계

사용자가 위임한 주요 설계를 완성한 상태다. 필수 기능의 정상·예외 경로, owner·상태·제어·저장·대화 정책을 선택했으며 [완결성 점검](./design-completeness.md)에 대응을 남겼다. 사용자 최종 검토, 코드용 schema·adapter 구현, 실제 모델 기능·장비 적합성과 성능 검증은 별개다. 구현·측정을 하지 않았다는 이유로 주요 설계를 미완료로 분류하지 않으며, 문서 완성을 성능 입증으로 표현하지 않는다. 추가 ASR 후보는 이후 DP 발굴·대안 비교 때 다시 논의한다.
