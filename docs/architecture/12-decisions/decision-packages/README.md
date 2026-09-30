# VIA 구조 선택 비교안

> 상태: 상세 비교 자료 작성 완료 · 4개 후보 · 최종 DP 선정·대안 채택·성능 우열은 미확정
> 방안 1은 검토 완료 target, 방안 2는 같은 VIA 문제를 푸는 다른 SW Architecture다. 구현·모델 실행·성능 측정은 하지 않았다.

## 먼저 읽을 자료

아래 네 문서를 순서대로 읽으면 된다. 각 문서는 **배경 1장 + 설계 비교 1장**의 SVG와 편집 가능한 draw.io, 배경 5줄·비교 8줄 원고, Component 변화표, 예외·복구 계약, ASR/QA 장단점 표를 포함한다. 비교표는 **공통 5항목(현재 ASR 4개+메모리)과 DP별 추가 품질 질문**을 `(+)·(-)·(0)`로 먼저 보여주고, 각 판단 조건과 쉬운 이유를 설명한 뒤 상세 인과를 제공한다. 모든 비교 행은 **ISO/IEC 25010:2023의 특성 → 부특성**에 연결했다. 먼저 [품질 대응 근거](../../08-quality-attributes/iso-25010-quality-basis.md)를 보면 자원 지표·분석/시험/설치 용이성과 기존 QA의 관계를 확인할 수 있다. [8페이지 전체 보기](./diagrams/review.html)는 파일을 내려받아 브라우저에서 열 수 있다. GitHub에서는 각 문서의 SVG가 바로 표시된다.

| 순서·문서 | VIA에서 풀 문제 | 방안 1: 실제 target | 방안 2: 다른 구조 | 그림에서 확인할 변화 |
| --- | --- | --- | --- | --- |
| 1. [과거 자료·대화·업무 검색](./semantic-retrieval-subsystem.md) | 이름이 아니라 주제·상황으로 지칭한 자료를 어떻게 찾나? | Context Manager의 owner 조회·metadata/keyword index·cache | Semantic Retrieval Service + 지속 색인 생산 | Indexing Worker·Embedding Runtime·Vector Index 추가, 후보 생산 책임 이동 |
| 2. [요청 대기·재개](./durable-request-orchestration.md) | 답변·선행 결과를 기다리다 정정·재시작이 오면 어디서 이어가나? | Request Controller의 durable domain 상태기계 | Interaction Workflow Runtime이 continuation 소유 | graph·질문 writer 이동, Signal Inbox·Timer Service·Activity Dispatcher·Continuation Store 도입 |
| 3. [음성 입력 근거](./speech-evidence-source.md) | 의미 추론 중에도 새 발화를 인식하고 당시 화면에 연결하려면? | 독립 Speech Input Worker + Streaming ASR, 공유 Omni | Omni native evidence가 인식도 담당 | 독립 recognizer process·helper 제거, Native Evidence Adapter와 공유 장애 경계 |
| 4. [업무·대화 복구](./recovery-state-source.md) | 재시작 뒤 현재 관계를 무엇에서 복원하나? | 권위 current records + 미완료 원장 + loader | 권위 Domain Journal + projection/replay/checkpoint | 저장 원본 지위 변경, Projection Engine·Replay Engine·Checkpoint Manager 추가 |

파랑은 **양안에서 달라지는 구성·책임**, 검정은 공통이다. 2안 추천 표시가 아니다. 큰 경계와 내부 모듈은 실제 수명·상태·호출 계약을 설명한다. 일부 그림은 같은 Component의 기능을 다른 위치에 확대 표시하며 별도 instance를 뜻하지 않는다. 세부 표가 각 요소의 존폐·책임 이동을 명시한다.

## 왜 4개인가

기존 개수를 유지하지 않고 [발굴·선발 기록](./discovery-and-selection.md)의 gate를 통과한 네 문제를 구체화했다. 기존 8개 중 음성·복구 문제는 살려 구조를 다시 설계했다. 지속 대화 projection과 자료별 Fact View도 가능한 Architecture지만, 현재 VIA 사용 조건에서 별도 주력 비교안으로 주장할 근거는 보류했다. 나머지는 중요한 내부 설계 계약으로 남으며 동일 위상의 DP로 포장하지 않았다.

- [선발 원칙](./selection-principles.md): Component가 바뀌는 **이유와 운영 메커니즘**을 함께 요구한다.
- [품질 비교 규칙](./quality-comparison-contract.md): 현재 4 ASR의 적용과 메모리·추가 QA, 과장하면 안 되는 효과.
- [검토·보완 기록](./review-notes.md): 독립 검토의 결함과 수정, 검사 범위.
- [지속 작업 계획](./WORKPLAN.md): 작업 순서·완료 기준·재개 지점.
- [과거 아이디어 검토](./reference-idea-review.md): 기존 7개 및 VIA-DP 참고 범위.

## 기준선과 보관

규범 근거는 [target 전체 구조](../target-architecture/architecture.md), [제어와 수명](../target-architecture/control-and-lifecycle.md), [기억과 Context](../target-architecture/memory-and-context-lifecycle.md), [공유 Omni](../target-architecture/shared-omni-runtime.md), [설계 완결성](../target-architecture/design-completeness.md)다. 방안 1의 기능을 빼서 대안을 유리하게 만들지 않는다. 방안 2가 target의 책임 배치를 바꾸는 것은 의도된 비교이며 기준선 수정은 아니다.

이전 세대는 역사 기록으로만 보존한다. [처음 7개 Component 경계 후보 archive](../../../archive/decision-reconstruction-component-boundaries-2026-09-30/README.md)는 유지했고, [직전 8개·동일 배치 그림 archive](../../../archive/decision-reconstruction-uniform-layouts-2026-10-01/README.md)에 원본 문서·그림·생성기 48파일의 SHA-256 manifest를 남겼다. 이전 검토의 “완성” 평가는 구조 차이 선발·표현을 충분히 검증하지 못했다는 점에서 철회한다. Archive를 현재 요구·검증 근거로 인용하지 않는다.

현재 ASR·QA의 metric·지위, target, ADR, VIA-DP-01~18 및 과거 측정 evidence는 유지했다. QA 품질 모델에는 ISO 분류 대응 설명을 보충했다. 상세 자료 완성은 모델 capability 확인이나 대안 선정 완료를 뜻하지 않는다.
