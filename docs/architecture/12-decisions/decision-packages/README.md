# VIA 구조 선택 비교안

> 현재 상태: **STAGE_4_SIX_COMPARISONS_REVIEW / 여섯 구조 비교, 사용자 검토용** / 2026-10-03
> 기존 target은 참조 구조이며 두 비교안 모두 달라도 된다. 구현, 모델 실행 및 성능 측정은 하지 않았다.

## 현재 읽기: 여섯 구조 비교

**[04-30 가이드](./04-30-comparison-guide.md) → 04-31~36 → [04-37 검토](./04-37-comparison-review.md)** 순서로 읽는다. 각 문서는 VIA에서 중요한 이유와 SW 설계의 어려움, 구조도와 동일 사건의 흐름, 13개 품질 손익 및 가장 싼 전환 반론을 포함한다.

| 번호 | 주제 |
| --- | --- |
| [04-31](./04-31-semantic-construction.md) | 전체 의미 생성과 제약 결합 |
| [04-32](./04-32-screen-grounding.md) | 화면 관측과 source 객체 |
| [04-33](./04-33-dialogue-coordination.md) | 중앙 대화와 업무별 actor 협력 |
| [04-34](./04-34-incremental-voice.md) | 확정 발화와 수정 가능한 연속 처리 |
| [04-35](./04-35-state-authority.md) | 현재 상태와 사건 이력 원본 |
| [04-36](./04-36-capability-isolation.md) | 공유 권한과 제한 처리 |

여섯 개 모두 작성 후 사용자와 리뷰하는 자료이며 최종 DP 선정, 구현 또는 측정 결과가 아니다. 아래는 이전 탐색의 읽기 기록이다.

## 이전 탐색의 읽기 기록

**최신 리뷰 반영:** M-2는 핵심 기능 주제로 우선 검토한다. 복구/격리는 구조와 품질 손익이 충분한 경우의 후순위 후보다. 상위 기능 품질에 직접 연결되는 세 주제를 추가했다.

1. [04-18 — 추가 탐색 결론](./04-18-functional-priority-exploration.md): 새 세 주제와 기존 후보의 우선순위.
2. [04-19 — 화면 지칭](./04-19-temporal-grounding.md): 사건별 관측 해석과 지속 객체 추적.
3. [04-20 — 여러 업무의 대화](./04-20-task-dialogue.md): 전체 대화 해석과 업무별 대화 상태.
4. [04-21 — 음성 처리](./04-21-voice-processing.md): 원음 중심과 Text 중심, 혼합안 및 지원 한계.
5. [04-22 — 품질과 반론](./04-22-functional-priority-review.md): 13개 관점, 실제 적용 조건과 중복/전환 판단.

아래 15방향 탐색은 앞선 체크포인트다. 세 신규 주제는 확정 DP가 아니며 정식 비교 및 최종 선정은 아직 진행하지 않았다.

### 앞선 확대 탐색

**2026-10-02 사용자 리뷰로 선발 기준과 탐색 방법을 수정했다.** 목적은 중요한 VIA 품질을 달성하는 구조 선택의 이유를 설명하는 것이다. 기능 추가나 제거로 쉽게 전환할 수 있는 T/A는 같은 계열로 묶고, 핵심 설계를 상당 부분 바꿔야 전환되는 다른 해결 메커니즘을 찾는다. 전환을 어렵게 만드는 것 자체가 목적은 아니다.

현재는 [선발 원칙](./selection-principles.md)과 [WORKPLAN의 맨 위 실행 계획](./00-workplan.md)을 먼저 읽는다. 원칙을 먼저 GitHub에 게시한 뒤 기존 계열 분류, 새 대안과 저렴한 전환 시도, V-01~13 및 보류 이유까지 작성하여 보고한다. 이전 S-01~06은 재평가 입력이며, 이전 독립 검토가 새 기준 통과를 뜻하지 않는다. Stage 5와 DP 선정은 아직 시작하지 않는다.

**이번 결과: 15개 방향을 검토해 구조 비교 후보 3개를 남겼다.** 11개는 기존 계열 안의 확장으로 제외했고, 엄격한 분산 확정 1개는 구조 차이를 인정하되 현재 PC에서의 선택 가치로 보류했다. 기존 C-대화는 국소 확장으로 판정해 주요 후보에서 제외했다.

1. [04-13 — 전체 탐색과 판정](./04-13-broad-mechanism-discovery.md): 다섯 문제군, 15개 방향, 기존 미결의 결론.
2. [04-14 — 통합 판단과 단계별 의미 결합](./04-14-staged-semantic-resolution.md): 의미 처리 단계와 중간 계약의 교체.
3. [04-15 — 현재 상태와 확정 이력](./04-15-event-authoritative-state.md): 쓰기 원본과 상태 생성/복구의 교체.
4. [04-16 — 공유 처리와 권한별 격리](./04-16-capability-compartments.md): 자료 접근 권한과 처리 코드의 분리.
5. [04-17 — 품질 및 반론 검토](./04-17-broad-mechanism-review.md): 13개 관점, 기능 지원, 싼 전환 반론과 한계.

세 후보의 정상/정정/철회/실패/재시작, 양방향 전환과 조건별 선택 이유를 작성했다. 이는 작성자의 구조 자격 판단이며 최종 DP 선정이나 측정된 우위가 아니다. [첫 재발굴 04-10](./04-10-mechanism-family-discovery.md), [04-11](./04-11-mechanism-family-proposals.md), [04-12](./04-12-mechanism-family-review.md)는 후속 판정 링크와 함께 보존했다.

아래는 이전 Stage 4 자료의 읽기 안내와 이력이다. 자료와 그림을 보존하되 새 후보 수나 구조 계열의 분해 기준으로 삼지 않는다.

사용자가 2026-10-01에 02-00을 승인했다. 요청한 순서대로 01-00의 가운데점을 정리하여 `e8a01e4`로 먼저 commit/push했고 CI 성공을 확인했다. 이후 3단계 품질 시나리오와 충돌을 작성했다.

**[04-00-structural-alternatives.md — 중요한 품질 차이를 만드는 SW 구조 탐색](./04-00-structural-alternatives.md)**부터 읽는다. **§2 P별 탐색 깊이 → §3 구조 전체 표 → §4 관심 있는 S의 개별 문서 → §5 추가 탐색 영역** 순서다. 집중 질문 6개와 대안 7개는 새 DP 수가 아니다.

사용자 리뷰에 따라 S-01~06을 개별 문서로 나눴다. 각 문서는 사용자 상황, 공통 시작, 핵심 차이, 비교 그림과 번호가 맞는 글 흐름을 먼저 제시하고, 뒤에서 상태와 실패, 13개 품질 관점 및 참고 자료의 취사선택을 설명한다. 처음 읽는 사람을 위한 용어와 흐름, 그림 구성은 [04-00 §1.2 공통 원칙](./04-00-structural-alternatives.md#12-처음-읽는-사람을-위한-설명과-그림-원칙)을 따른다.

| 질문 | 개별 문서 | 비교 |
| --- | --- | --- |
| S-01 | [요청 해석 정확성을 위한 추가 자료 확인 설계](./04-01-semantic-execution.md) | T / A |
| S-02 | [화면 지칭 정확성을 위한 화면 정보 처리 설계](./04-02-screen-evidence.md) | T / A, A는 후보 유지 및 보강 방향 미정. 공통 질문과 선택 tactic |
| S-03 | [자료 검색 정확성을 위한 자료 찾기 설계](./04-03-semantic-retrieval.md) | T / A |
| S-04 | [요청 복구성을 위한 대기 및 재개 설계](./04-04-request-continuation.md) | T / A / B |
| S-05 | [응답성을 위한 음성 응답 생성 설계](./04-05-response-path.md) | T / A |
| S-06 | [음성 인식 정확성과 응답성을 위한 음성 입력 처리 설계](./04-06-speech-evidence.md) | T / A |

그림은 각 문서에서 바로 볼 수 있고 SVG 옆에 편집 가능한 draw.io 원본을 연결했다. [구조 비교 6장과 S-01 실행 상세 모아 보기](./diagrams/stage4-review.html)는 내려받아 브라우저에서 열 수 있다. Component 안의 Module, 실제로 옮겨지는 책임과 경로, 서로 다른 자료/저장소 모양, 공통 검정/T 파랑/대안 초록을 적용하는 [설계도 작성 기준](./diagram-design-guide.md)을 함께 관리한다. 대안 수를 맞추려고 새로운 B를 만들지는 않았다.

04는 [작업 계획의 탐색 노력 배분](./00-workplan.md#4단계의-탐색-노력-배분)을 따른다. 상위 품질에 직접 영향을 주는 문제부터 집중 탐색하고 다른 문제는 연계 검토 또는 간략 점검한다. 구체화한 대안의 품질 검토에서는 13개 관점을 유지한다.

품질 기준은 **[03-00-quality-scenarios.md — 품질 시나리오와 충돌](./03-00-quality-scenarios.md)**에서 확인한다. §2의 13개 품질 관점 → §3의 관점별 충돌 → §4의 관심 있는 P 시나리오 → §5의 04~05 후속 검토표 순서다. V-01~13을 문서 전체에서 연결하며 기존 QA 번호나 핵심/추가 구분을 사용하지 않는다.

**후속 합의:** 03에서 품질의 의미와 시나리오, 04~05에서 구조에 따른 품질 차이를 검토하고 실제 측정 전에 정확한 평가 계약을 고정한다. 같은 QA는 **모든 DP와 모든 방안에서 동일한 정의, 지표, 측정 방법**을 사용한다. 04~05에서도 13개 관점을 모두 점검한다. 기존 QA 자료는 03-00 끝의 별도 참고로 두며 자동 채택하지 않는다. [03-00 §6.1](./03-00-quality-scenarios.md#61-모든-dp와-모든-방안에서-같은-qa를-사용한다)에 공통 원본, 적용 및 개정 규칙을 기록했다.

**이번 리뷰 반영:** V-02 기능 적절성, V-03 기능 완전성, V-04 상호작용 반응성, V-05 VIA 귀속 요청 완료 시간을 각각 분리했다. 기능을 일부 포기한 안도 탐색과 비교에 남기고 지원 범위, 사용자 영향과 얻는 이익을 함께 적는다. 두 시간 관점 모두 Downstream Agent 내부 실행 시간을 외부 조건으로 제외한다. 분리 전 우선순위를 이어받은 전체 순위는 [03-00 §2.1](./03-00-quality-scenarios.md#21-이번-비교의-우선순위)에 있다.

- [4단계 검토 기록](./04-09-structural-review.md): 별도 검토자 3명의 target 대조, 탐색 충분성과 품질 인과 검토. 초안 P2 3건과 이후 그림 P2 3건의 수정 및 재확인, 렌더링과 문서 검사 기록.
- [3단계 작성자 검토 기록](./03-01-quality-review.md): 요구 보존, 품질 분류, 미검증 지원과 QA 의미 대조. 별도 독립 리뷰를 수행한 기록은 아니다.
- [2단계 요구와 현재 해결 수단](./02-00-requirements-and-choices.md): 사용자 승인 완료. §2 공통 경계와 §3 요약표가 이번 품질 시나리오의 입력이다.
- [2단계 독립 검토 기록](./02-01-requirements-review.md): 분류 오류·요구 약화·capability 가정 점검과 보완.
- [1단계 중요 문제 원장](./01-00-problem-coverage.md): 16개 문제와 VIA 특성. 사용자 검토 후 2단계 진행을 승인받았다. 16개는 DP 개수가 아니다.
- [UC 상세 추적](./01-01-problem-coverage-use-cases.md), [운영·변화·품질 점검](./01-02-problem-coverage-crosscutting.md), [1단계 검토 기록](./01-03-problem-coverage-review.md): 문제 범위의 근거.
- [WORKPLAN의 현재 6단계](./00-workplan.md): 순서·산출물·재개 지점.

**후속 탐색에서도 다음을 확인한다.**

1. VIA에서 중요한 품질 충돌과 그 원인이 빠졌는가?
2. 일부 기능을 포기한 대안의 사용자 손실과 다른 품질의 이익을 함께 드러내는가?
3. 차이를 좌우하는 부하, 실패, 지원 조건과 관찰 항목이 충분한가?
4. 13개 관점이 구체 시나리오와 후속 구조 검토까지 일관되게 이어지는가?

의견은 `S 번호 또는 P/V 번호 + 구조나 조건에 대한 지적`으로 남기면 된다. 04에는 실제 target와 대안의 실행 및 상태 흐름, 기능 손실과 13개 품질 인과를 기록했다. 독립 검토와 수정 후 재확인을 마쳤으며, 다음은 사용자 리뷰다. 05의 정식 양안 비교와 DP 선발은 아직 남아 있다.

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
| 3. 품질 충돌 | [03-00-quality-scenarios.md](./03-00-quality-scenarios.md) | 사용자 리뷰 후 04 진행 승인 |
| 3. 작성자 리뷰 | [03-01-quality-review.md](./03-01-quality-review.md) | 대조 및 문서 검사 기록 |
| 4. 구조 대안 탐색 | [04-00-structural-alternatives.md](./04-00-structural-alternatives.md) | S-01~06의 용어, 입력 자료, 비교 그림과 번호 순서 정리, 사용자 재검토 대기 |
| 4. 검토 기록 | [04-09-structural-review.md](./04-09-structural-review.md) | 작성자 대조, 독립 지적 반영 및 재확인 기록 |
| 4. 구조 계열 재발굴 | [04-10-mechanism-family-discovery.md](./04-10-mechanism-family-discovery.md) | 기존 여섯 주요 자격 재검토, 첫 탐색 |
| 4. 메커니즘 구체화 | [04-11-mechanism-family-proposals.md](./04-11-mechanism-family-proposals.md) | 두 방향의 사건과 상태, 전환 반론 |
| 4. 재발굴 판정 | [04-12-mechanism-family-review.md](./04-12-mechanism-family-review.md) | 작성자 검토. 주요 후보 미확정 |
| 5. 강한 대안 비교 | `05-00-comparison-and-selection.md` | 예정·미작성 |
| 6. DP 선발·상세화 | `06-00-decision-packages.md`, `06-01-<topic>.md`부터 개별 자료 | 예정·미작성 |

번호 없는 아래 문서들은 기존 비교, 참고 자료다. 새 단계의 완료 산출물로 오인하지 않도록 구별한다. 현재 04의 독립 검토와 수정을 마쳤으며 아래 네 비교안은 새 4~6단계의 완료 증거로 취급하지 않는다.

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

규범 근거는 [target 전체 구조](../target-architecture/architecture.md), [제어와 수명](../target-architecture/control-and-lifecycle.md), [기억과 Context](../target-architecture/memory-and-context-lifecycle.md), [공유 Omni](../target-architecture/shared-omni-runtime.md), [설계 완결성](../target-architecture/design-completeness.md)다. 방안 1의 기능을 빼서 대안을 유리하게 만들지 않는다. 방안 2는 일부 기능을 제한하거나 포기할 수 있으며, 공통 요구 목록 대비 V-03의 지원 차이와 다른 품질의 이익을 함께 비교한다. 이전 자료의 완전 충족 전제를 새 후보의 자동 제외 조건으로 사용하지 않는다. 방안 2가 target의 책임 배치를 바꾸는 것은 의도된 비교이며 기준선 수정은 아니다.

이전 세대는 역사 기록으로만 보존한다. [처음 7개 Component 경계 후보 archive](../../../archive/decision-reconstruction-component-boundaries-2026-09-30/README.md)는 유지했고, [직전 8개·동일 배치 그림 archive](../../../archive/decision-reconstruction-uniform-layouts-2026-10-01/README.md)에 원본 문서·그림·생성기 48파일의 SHA-256 manifest를 남겼다. 이전 검토의 “완성” 평가는 구조 차이 선발·표현을 충분히 검증하지 못했다는 점에서 철회한다. Archive를 현재 요구·검증 근거로 인용하지 않는다.

현재 ASR·QA의 metric·지위, target, ADR, VIA-DP-01~18 및 과거 측정 evidence는 유지했다. QA 품질 모델에는 ISO 분류 대응 설명을 보충했다. 상세 자료 완성은 모델 capability 확인이나 대안 선정 완료를 뜻하지 않는다.
