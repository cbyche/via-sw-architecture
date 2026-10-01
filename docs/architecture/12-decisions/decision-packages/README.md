# VIA 구조 선택 비교안

> 현재 상태: **2단계 승인 완료 / 3단계 품질 충돌 사용자 검토 대기** / 기존 4개는 재평가 입력 / 최종 DP 선정, 대안 채택, 성능 우열 미확정
> 방안 1은 검토 완료 target, 방안 2는 같은 VIA 문제를 푸는 다른 SW Architecture다. 구현·모델 실행·성능 측정은 하지 않았다.

## 현재 작업부터 읽기

사용자가 2026-10-01에 02-00을 승인했다. 요청한 순서대로 01-00의 가운데점을 정리하여 `e8a01e4`로 먼저 commit/push했고 CI 성공을 확인했다. 이후 3단계 품질 시나리오와 충돌을 작성했다.

먼저 **[03-00-quality-scenarios.md — 품질 시나리오와 충돌](./03-00-quality-scenarios.md)**을 읽는다. §2의 ISO 품질 기준 → §3의 충돌 요약 → §4의 관심 있는 P 상세 → §5의 추가 품질 제안 순서다. 현재 네 ASR뿐 아니라 메모리, 지속 부하, 사용자 제어, 상호운용성, 분석 및 시험 가능성을 함께 검토한다.

- [3단계 작성자 검토 기록](./03-01-quality-review.md): 요구 보존, 품질 분류, 미검증 지원과 QA 의미 대조. 별도 독립 리뷰를 수행한 기록은 아니다.
- [2단계 요구와 현재 해결 수단](./02-00-requirements-and-choices.md): 사용자 승인 완료. §2 공통 경계와 §3 요약표가 이번 품질 시나리오의 입력이다.
- [2단계 독립 검토 기록](./02-01-requirements-review.md): 분류 오류·요구 약화·capability 가정 점검과 보완.
- [1단계 중요 문제 원장](./01-00-problem-coverage.md): 16개 문제와 VIA 특성. 사용자 검토 후 2단계 진행을 승인받았다. 16개는 DP 개수가 아니다.
- [UC 상세 추적](./01-01-problem-coverage-use-cases.md), [운영·변화·품질 점검](./01-02-problem-coverage-crosscutting.md), [1단계 검토 기록](./01-03-problem-coverage-review.md): 문제 범위의 근거.
- [WORKPLAN의 현재 6단계](./00-workplan.md): 순서·산출물·재개 지점.

**이번 리뷰에서는 다음을 확인한다.**

1. VIA에서 중요한 품질 충돌과 그 원인이 빠졌는가?
2. 필수 사용자 결과를 포기하는 것을 품질 절충으로 잘못 다뤘는가?
3. 차이를 좌우하는 부하, 실패, 지원 조건과 관찰 항목이 충분한가?
4. 메모리를 포함한 추가 품질 검토 제안이 적절한가?

의견은 `P 번호 또는 §5 항목 + 빠진 품질/조건`으로 남기면 된다. 3단계 자료를 검토한 뒤 4단계의 여러 구조 탐색으로 이어간다. 현재 4단계는 미착수이며 양안 우열 기호나 DP 개수를 먼저 정하지 않았다.

문제·요구·품질·여러 구조를 검토한 뒤 DP를 선발한다. 아래 네 문서는 재평가 입력으로 보존하며 새 문제 탐색의 최종 선발 결과가 아니다.

## 단계별 파일 이름과 읽기 순서

파일명은 **단계번호-읽기순서-주제**로 정렬한다. `00-workplan.md`는 전체 계획이고, 각 단계의 `00` 문서부터 읽는다. 이후 번호는 같은 단계의 상세 근거·검토 순서다.

| 단계 | 파일 | 상태 |
| --- | --- | --- |
| 전체 계획 | [00-workplan.md](./00-workplan.md) | 현재 진행·재개 기준 |
| 1. 문제 범위 | [01-00-problem-coverage.md](./01-00-problem-coverage.md) | 사용자 검토 후 2단계 진행 승인 |
| 1. UC 근거 | [01-01-problem-coverage-use-cases.md](./01-01-problem-coverage-use-cases.md) | 작성 완료 |
| 1. 운영·변화·품질 근거 | [01-02-problem-coverage-crosscutting.md](./01-02-problem-coverage-crosscutting.md) | 작성 완료 |
| 1. 독립 리뷰 | [01-03-problem-coverage-review.md](./01-03-problem-coverage-review.md) | 수정·재확인 완료 |
| 2. 요구와 수단 | [02-00-requirements-and-choices.md](./02-00-requirements-and-choices.md) | 사용자 승인 완료 |
| 2. 독립 리뷰 | [02-01-requirements-review.md](./02-01-requirements-review.md) | 검토·검사 기록 |
| 3. 품질 충돌 | [03-00-quality-scenarios.md](./03-00-quality-scenarios.md) | 작성 완료, 사용자 검토 대기 |
| 3. 작성자 리뷰 | [03-01-quality-review.md](./03-01-quality-review.md) | 대조 및 문서 검사 기록 |
| 4. 구조 대안 탐색 | `04-00-structural-alternatives.md` | 예정·미작성 |
| 5. 강한 대안 비교 | `05-00-comparison-and-selection.md` | 예정·미작성 |
| 6. DP 선발·상세화 | `06-00-decision-packages.md`, `06-01-<topic>.md`부터 개별 자료 | 예정·미작성 |

번호 없는 아래 문서들은 기존 비교, 참고 자료다. 새 단계의 완료 산출물로 오인하지 않도록 구별한다. 현재 2단계까지 승인받았으며 아래 네 비교안은 3~6단계의 완료 증거로 취급하지 않는다.

## 기존 비교 자료

아래 네 문서를 순서대로 읽으면 된다. 각 문서는 **배경 1장 + 설계 비교 1장**의 SVG와 편집 가능한 draw.io, 배경 5줄·비교 8줄 원고, Component 변화표, 예외·복구 계약, ASR/QA 장단점 표를 포함한다. 비교표는 **공통 5항목(현재 ASR 4개+메모리)과 DP별 추가 품질 질문**을 `(+)·(-)·(0)`로 먼저 보여주고, 각 판단 조건과 쉬운 이유를 설명한 뒤 상세 인과를 제공한다. 모든 비교 행은 **ISO/IEC 25010:2023의 특성 → 부특성**에 연결했다. 먼저 [품질 대응 근거](../../08-quality-attributes/iso-25010-quality-basis.md)를 보면 자원 지표·분석/시험/설치 용이성과 기존 QA의 관계를 확인할 수 있다. [8페이지 전체 보기](./diagrams/review.html)는 파일을 내려받아 브라우저에서 열 수 있다. GitHub에서는 각 문서의 SVG가 바로 표시된다.

| 순서·문서 | VIA에서 풀 문제 | 방안 1: 실제 target | 방안 2: 다른 구조 | 그림에서 확인할 변화 |
| --- | --- | --- | --- | --- |
| 1. [과거 자료·대화·업무 검색](./semantic-retrieval-subsystem.md) | 이름이 아니라 주제·상황으로 지칭한 자료를 어떻게 찾나? | Context Manager의 owner 조회·metadata/keyword index·cache | Semantic Retrieval Service + 지속 색인 생산 | Indexing Worker·Embedding Runtime·Vector Index 추가, 후보 생산 책임 이동 |
| 2. [요청 대기·재개](./durable-request-orchestration.md) | 답변·선행 결과를 기다리다 정정·재시작이 오면 어디서 이어가나? | Request Controller의 durable domain 상태기계 | Interaction Workflow Runtime이 continuation 소유 | graph·질문 writer 이동, Signal Inbox·Timer Service·Activity Dispatcher·Continuation Store 도입 |
| 3. [음성 입력 근거](./speech-evidence-source.md) | 의미 추론 중에도 새 발화를 인식하고 당시 화면에 연결하려면? | 독립 Speech Input Worker + Streaming ASR, 공유 Omni | Omni native evidence가 인식도 담당 | 독립 recognizer process·helper 제거, Native Evidence Adapter와 공유 장애 경계 |
| 4. [업무·대화 복구](./recovery-state-source.md) | 재시작 뒤 현재 관계를 무엇에서 복원하나? | 권위 current records + 미완료 원장 + loader | 권위 Domain Journal + projection/replay/checkpoint | 저장 원본 지위 변경, Projection Engine·Replay Engine·Checkpoint Manager 추가 |

파랑은 **양안에서 달라지는 구성·책임**, 검정은 공통이다. 2안 추천 표시가 아니다. 큰 경계와 내부 모듈은 실제 수명·상태·호출 계약을 설명한다. 일부 그림은 같은 Component의 기능을 다른 위치에 확대 표시하며 별도 instance를 뜻하지 않는다. 세부 표가 각 요소의 존폐·책임 이동을 명시한다.

## 기존 네 후보의 선발 기록과 한계

당시에는 기존 개수를 유지하지 않고 [발굴·선발 기록](./discovery-and-selection.md)의 gate를 통과한 네 문제를 구체화했다. 기존 8개 중 음성·복구 문제는 살려 구조를 다시 설계했다. 지속 대화 projection과 자료별 Fact View도 가능한 Architecture지만, 현재 VIA 사용 조건에서 별도 주력 비교안으로 주장할 근거는 보류했다. 나머지는 중요한 내부 설계 계약으로 남으며 동일 위상의 DP로 포장하지 않았다. 다만 당시 개별 후보 검토는 제품 문제 전체와 대안 탐색의 충분성을 입증하지 못했다. 해당 제외·보류도 새 6단계에서 재평가한다.

- [선발 원칙](./selection-principles.md): Component가 바뀌는 **이유와 운영 메커니즘**을 함께 요구한다.
- [품질 비교 규칙](./quality-comparison-contract.md): 현재 4 ASR의 적용과 메모리·추가 QA, 과장하면 안 되는 효과.
- [검토·보완 기록](./review-notes.md): 독립 검토의 결함과 수정, 검사 범위.
- [지속 작업 계획](./00-workplan.md): 작업 순서·완료 기준·재개 지점.
- [과거 아이디어 검토](./reference-idea-review.md): 기존 7개 및 VIA-DP 참고 범위.

## 기준선과 보관

규범 근거는 [target 전체 구조](../target-architecture/architecture.md), [제어와 수명](../target-architecture/control-and-lifecycle.md), [기억과 Context](../target-architecture/memory-and-context-lifecycle.md), [공유 Omni](../target-architecture/shared-omni-runtime.md), [설계 완결성](../target-architecture/design-completeness.md)다. 방안 1의 기능을 빼서 대안을 유리하게 만들지 않는다. 방안 2가 target의 책임 배치를 바꾸는 것은 의도된 비교이며 기준선 수정은 아니다.

이전 세대는 역사 기록으로만 보존한다. [처음 7개 Component 경계 후보 archive](../../../archive/decision-reconstruction-component-boundaries-2026-09-30/README.md)는 유지했고, [직전 8개·동일 배치 그림 archive](../../../archive/decision-reconstruction-uniform-layouts-2026-10-01/README.md)에 원본 문서·그림·생성기 48파일의 SHA-256 manifest를 남겼다. 이전 검토의 “완성” 평가는 구조 차이 선발·표현을 충분히 검증하지 못했다는 점에서 철회한다. Archive를 현재 요구·검증 근거로 인용하지 않는다.

현재 ASR·QA의 metric·지위, target, ADR, VIA-DP-01~18 및 과거 측정 evidence는 유지했다. QA 품질 모델에는 ISO 분류 대응 설명을 보충했다. 상세 자료 완성은 모델 capability 확인이나 대안 선정 완료를 뜻하지 않는다.
