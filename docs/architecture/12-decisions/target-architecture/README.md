# VIA Target Architecture — 검토 완료 기준선

> 상태: **REVIEWED_BASELINE / 사용자 검토 완료·목표 설계 기준선 확정 / 구현·측정 없음**
> 시작일·검토 확정일: 2026-09-29
> 목적: QA-19 semantic accuracy, QA-09 responsiveness, QA-29 modifiability, QA-39 reliability/recoverability 순으로 우선하되 네 ASR을 모두 고려한 목표 Architecture를 먼저 완성한다.

## 무엇을 읽으면 되는가

**먼저 [architecture.md](./architecture.md)만 읽으면 된다.** 전체 구조와 13개 그림, 실제 요청 흐름, 책임·상태·장애 경계와 약점을 한 문서에서 검토할 수 있다. 그림은 본문에 이미 삽입되어 있으므로 `diagrams/`를 따로 순회할 필요가 없다.

그다음 [design-completeness.md](./design-completeness.md)를 보면 요구 기능 18개와 예외가 어디에 반영됐는지, 에이전트가 구체화한 주요 선택은 무엇인지 확인할 수 있다. 세부 계약은 본문을 읽다가 해당 동작을 더 엄밀하게 확인하고 싶을 때만 연다. 모든 파일을 순서대로 읽을 필요는 없다.

| 구분 | 파일 | 목적·읽을 때 |
| --- | --- | --- |
| 안내 | `README.md` | 읽기 순서, 제품 경계, 합의 상태와 문서 관리 규칙 |
| **주 리뷰 문서** | [architecture.md](./architecture.md) | 전체 시스템 구조와 그림·사용자 시나리오를 검토 |
| **누락 확인** | [design-completeness.md](./design-completeness.md) | 기능·예외의 설계 대응과 주요 선택, 설계 완료 범위 확인 |
| 상세 계약 | [control-and-lifecycle.md](./control-and-lifecycle.md) | 입력 확정, S2S 직접 응답, 질문·승인, 정정·취소, Task·전송·출력·복구의 정확한 상태 전이 |
| 상세 계약 | [shared-omni-runtime.md](./shared-omni-runtime.md) | Omni 공유, Voice/semantic 역할 분리, 입력 ASR, 자원 경합·포화와 모델/runtime 요구 |
| 상세 계약 | [memory-and-context-lifecycle.md](./memory-and-context-lifecycle.md) | Context 후보·원본·요약, 기억 계층, 보관·삭제와 DB/파일 복구 |
| 참고 근거 | [model-capability-review.md](./model-capability-review.md) | Qwen3-Omni 규모·공개 기능의 출처와 실제 확인 범위; VIA의 성능 측정 결과가 아님 |
| 그림 자산 | [diagrams/README.md](./diagrams/README.md)와 13쌍의 그림 | README는 색상·화살표 범례와 편집 안내, `.drawio`는 편집 원본, `.svg`는 GitHub 본문 표시용. 둘 다 필요 |

이 디렉터리는 사용자 검토를 마친 **목표 Architecture 기준선**의 source of truth다. 사용자가 전체 문서를 읽고 리뷰한 뒤 “지금까지 리뷰한 것을 바탕으로 목표 Architecture를 정리하고 검토 기준선으로 확정”하도록 명시적으로 요청했다. 따라서 단순 문서 게시나 commit을 승인으로 추정한 상태가 아니다. 확정 범위는 주요 책임·상태·계약·정상/예외 흐름·배치·제어·기억 정책과 13개 그림이다. 구현 성공·모델 기능 적합성·성능 우위·대안 비교 승자·측정 freeze를 승인한 것은 아니다. 기존 accepted/deferred ADR, 제품 요구와 QA 정의는 이 확정으로 자동 변경하지 않는다.

## 작업 순서

```text
완성된 목표 Architecture 설계·대화·수정
    ↓ 전체 구조 합의 이후
중요한 구조적 선택·steelman 후보 구체화와 ASR 재검토
    ↓
강한 현실적 대안과 비용·유리한 조건 정리
    ↓
선택 근거와 반증 조건을 갖춘 검증 설계
```

이번 작업은 solution-first 방식이다. 기존 VIA-DP-01~18의 분해·대안·검토 순서에 맞추거나 새 구조를 기존 번호에 매핑하지 않는다. 모든 Component를 Decision Package로 만들지도 않는다.

목표 구조의 합의를 완료했으며 다음 작업은 Decision Reconstruction이다. 기준선의 구조 선택은 반증·재검토할 수 있다. 강한 대안, 선택한 구조의 비용, 대안이 더 유리한 조건과 반증 가능성을 유지한다. 측정 결과에 맞춰 정의를 바꾸거나 결론에 맞게 결과를 고르지 않는다.

## 고정하는 제품 경계

- VIA는 PC의 Voice/Text/screen interaction과 orchestration을 담당한다.
- Downstream Agent는 domain reasoning, planning, tool selection, 실제 업무 수행을 담당한다.
- VIA는 on-device Omni 1개를 음성·semantic 두 역할이 공유한다. 역할·session·권한은 분리하고 가중치는 복제하지 않는다. Semantic 중에도 발화 수신·인식을 유지하며 경량 Streaming ASR을 주안에 명시한다. 추가 helper는 필요성과 전체 비용을 드러낸다.
- 목표 Architecture의 품질 우선순위는 QA-19 semantic accuracy, QA-09 responsiveness, QA-29 modifiability, QA-39 reliability/recoverability 순이다. 낮은 순위도 설계에서 생략하지 않는다.
- accuracy는 사용자 목표·대상·Task·처리 방향·위임 내용의 정확성이다. Agent가 만든 도메인 결과의 품질과 구분한다.
- responsiveness는 유효한 응답·clarification·위임·진행·중단까지의 VIA 책임 경로를 다룬다. 모델 시간이나 접수 멘트만으로 설명하지 않는다.
- modifiability는 Agent·Model·Context source·계약 변화가 VIA의 책임·interface·state·runtime에 퍼지는 범위를 다룬다.
- reliability/recoverability는 dependency·process·network·State Store 장애를 가두고 중복·오연결 없이 확인 가능한 상태로 복구하는 능력을 다룬다.
- Voice 연결, Conversation, Request, Task, Agent Execution의 수명을 분리한다.

제품 요구의 근거는 [Mission & Boundary](../../01-system-mission-and-boundary.md), [Fixed Scope](../../03-fixed-architecture-scope.md), [Representative Use Cases](../../05-representative-use-cases.md)다. 용어는 [Terms](../../02-terms.md), 의미상 정확성의 범위는 [Correctness & Continuity](../../08-quality-attributes/correctness-and-continuity.md)를 참고한다.

## 저장소에서 계속 정리하는 방식

1. 기준선 이후 설계 내용이 달라지면 변경 이유·영향받는 계약·ASR을 명시하고 관련 본문·상세 계약·그림을 함께 수정한다. 이전 상태는 Git 이력으로 추적하며 superseded generation이 생기면 archive 규칙을 따른다.
2. 사용자 지정·합의와 위임에 따른 설계 선택을 구분한다. 현재 합의 상태는 아래 표, 주요 선택·누락 점검은 `design-completeness.md`, 변경 이력은 Git commit에 남긴다. 별도 누적 검토 문서를 필독 목록으로 늘리지 않는다.
3. 문서화·commit을 Architecture의 의미상 승인과 혼동하지 않는다. 합의 상태는 별도로 표시한다.
   순차 리뷰 중 개별 피드백을 전체 승인으로 확대하지 않았으며, 이번에는 전체 검토 후 사용자의 명시적 기준선 확정 지시를 반영했다.
4. 별도 작업 branch·PR을 매번 만들지 않고 현재 Architecture 흐름 안에서 본문·상세 계약·완결성 점검을 함께 갱신한다.
5. 채팅에만 남은 제안을 구현 완료·성능 확인으로 바꾸지 않는다. 실제 근거가 생기기 전까지 성능은 가설이다.
6. 목표 Architecture 그림은 `.drawio` 편집 원본과 같은 이름의 `.svg` preview를 함께 관리한다. Markdown에는 SVG를 삽입하고 draw.io 원본을 바로 연결한다.
7. Component 참조는 `architecture.md` §4의 정식 명칭을 본문·표·상세 계약·그림에 동일하게 쓴다. 축약명·별칭을 쓰지 않으며, 그림 공간은 정식 명칭을 유지한 채 줄바꿈·배치로 확보한다.

## 현재 산출물의 한계

필수 기능의 정상·예외 경로, owner·상태·제어·저장·대화 정책을 선택하고 사용자 검토를 완료했으며 [완결성 점검](./design-completeness.md)에 대응을 남겼다. 코드용 schema·adapter 구현, 실제 모델 기능·장비 적합성과 성능 검증은 별개다. 구현·측정을 하지 않았다는 이유로 주요 설계를 미완료로 분류하지 않으며, 기준선 확정을 성능 입증으로 표현하지 않는다. 현재 네 ASR은 설계 작업 기준이며 최종 비교 목록이 아니다. 이후 Decision Point와 steelman 후보를 구체화할 때 ASR의 추가·변경과 평가 기준을 함께 정의한다.

## 사용자 지정과 검토 기준선의 구분

아래 제품 행동은 사용자 지정 또는 명시적 합의다. 그 행동을 구현하는 세부 계약은 사용자 위임으로 구체화한 뒤 이번 검토 기준선에 포함했다. 제품 요구와 이를 달성하는 구조는 구분하며, 기준선에 포함된 구조라는 이유로 이후 대안도 같은 수단을 사용하도록 강제하지 않는다.

| 제품 행동 | 상태 |
| --- | --- |
| 제한된 조회 후에도 지칭 후보가 둘 이상이면 질문 | 합의 |
| 새 발화 때 같은 대화의 미전송 요청 보류 | 합의 |
| 한정 자료 설명·요약은 VIA, 조사·업무 계획·실행은 Agent | 합의; semantic이 handling을 제안하고 host가 scope·권한 적용 |
| 화면에는 상세 결과, 음성에는 핵심 요약; 사용자 발화 중 음성 알림 금지 | 사용자 지정; 차례·재연결 세부는 검토 기준선에 포함 |
| 중단 후 재개 의도가 불명확할 때만 확인 | 합의 |
| S2S는 맥락 없는 명백한 자체 지식 질문에만 직접 응답 | 사용자 지정; 첫 단순 질문도 허용, 후속 지칭 fast path 확장 미채택 |
| 한 업무는 통째로 Agent, 독립 업무는 같은 Agent여도 별도 Task | 합의 |
| 공유 on-device Omni, 동시 발화 수신·인식, 필요한 별도 ASR 허용 | 사용자 지정; scheduler·모델팀 계약은 검토 기준선에 포함 |
| 기억 계층·보관·삭제와 나머지 주요 정책 구체화 | 사용자 위임으로 설계한 내용을 전체 검토 후 기준선으로 확정 |

## 이번 리뷰의 반영과 다음 작업

| 리뷰 항목 | 기준선에 반영한 내용 |
| --- | --- |
| invariant와 설계 선택 | 무오류 보장이 아닌 현재 구조의 제어 규칙; 이후 구조 비교의 제외 조건으로 사용하지 않음 |
| ASR 범위와 우선순위 | 네 ASR은 설계 기준; 후속 비교의 목록·개수는 재검토하고 다른 축의 측정을 생략하는 필터로 사용하지 않음 |
| Component 명칭과 호출 | 정식 명칭 통일; 최초 해석·추가 조회 모두 Request Controller 경유; 그림 03과 본문 일치 |
| Task 연결과 Context | Request Interpreter가 의미를 제안하고 Request Controller가 검증·확정; Context Manager가 내부·외부 owner의 근거를 조회하며 다른 owner의 State Store 내부 schema에 직접 결합하지 않음 |
| 관측 근거 수집 | streamed text 조각 대신 OS/UI 사건·유한 capture 예산 사용; 시점·buffer·pin·gap·stop 규칙 명시 |
| Agent 선택 | semantic 제안 + 코드의 capability·권한·제약 검사·동등 후보 우선순위; Task Manager는 확정된 선택으로 업무 구성 |
| 공유 추론·메모리·응답·배치 | 기존 scheduling·응답 제어 유지, 자원 비용과 배치의 조건부 정확성 인과 명시; 메모리는 후속 ASR 재검토의 중요 후보 |

다음 세션은 [Decision Packages 안내](../decision-packages/README.md)를 따라 **구조적 선택 / 가장 강한 현실적 대안 초안 / 상태·계약·호출·배치 차이 / 관련 ASR과 인과관계**의 후보 목록부터 작성한다. 목록을 사용자와 함께 좁힌 뒤 정식 package로 발전시킨다. Model Access scheduling, Agent 선택, 응답 제어, 프로세스 배치는 리뷰에서 관심을 표시한 주제이며 package 채택을 미리 확정한 목록은 아니다. QA-41은 아직 diagnostic이며 메모리 ASR의 승격·정의·비교 조건을 후보와 함께 검토한다. ASR 추가를 메모리 하나로 제한하지 않는다.

다른 세션에서 이어갈 때도 AGENTS.md와 이 기준선을 읽고 시작한다. 이전 목표 설계를 처음부터 재설계하거나 기존 DP에 매핑하지 않으며, 별도 지시 없이 구현·모델 실행·성능 측정·측정 freeze를 시작하지 않는다. 이번 확정의 Git commit이 검토 기준 버전이며 이후 변경도 commit 단위로 추적한다.

현재 기능 대응과 후속 항목은 [완결성 점검](./design-completeness.md)이 기준이다. 이전 독립 검토와 누적 검토 기록은 [역사 기록](../../../archive/target-architecture-review-history-2026-09-29/README.md)으로 옮겼다. 현재 리뷰에 읽을 필요가 없으며 당시 열린 질문을 현재 미결 설계로 취급하지 않는다.
