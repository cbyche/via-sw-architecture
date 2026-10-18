# VIA 일곱 설계 문제의 최종 분석

> **증거 보존과 재현:** 작은 최종 보고서는 직접 읽을 수 있다. 전체 raw·실패·동결 snapshot·큰 replay는 [두 캠페인 분할 패키지](../../../../../results/architecture-evaluation/current/four-dp-poc-evidence-20261019/README.md)를 검증·복원한 뒤 원래 링크에서 읽는다. [Clean clone의 API 없는 복원·재분석](../../../../../benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md)을 따른다. 패키징 구성 설명은 Git 발행 완료 선언이나 과거 미발행 이력의 변경이 아니다.

> **현재 읽을 자료:** [15 보완 결과 보고서](15-corrected-poc-results.md)와 [10 발표 원고](10-presentation-outline.md). 최종 기본10개 안120회, A2/A4/B4 보완27회와 공통 원문 표현 진단9회를 완료했다. 모든 QA의 구조적 예상과 실제 호출·시간·비용·변경 모듈·메모리를 연결하며 실패·교정 이력을 보존한다. 현재 권고는01A/02B/04B+B4이고 05는 조건 보존 우선B 또는 수신자 읽기·시간·비용 우선C다. 아래 초기 결과와 설계 판단은 각각의 이력이다.

2026-10-19 / 설계 분석·보완 PoC 실행과 증거 대조 완료

[14 실행 계약](14-corrected-poc-contract.md)에 따라 사용자 입력부터 실제 첫 callback과 목표 완료를 나누고, 실패 분모·복구·현재 원문 사용을 보완했다. 실제 미래 기능 확장의 Rust 변경과 역할별 메모리도 측정했다. 코드 개발·실행은 사용자 지정 GPT-6.1 Sol High 담당들이 수행했다. 최종 기본 비교는 10개 안 120회이며 교정 전·중간 실행을 성공 행만 골라 섞지 않는다.

[실제 기록 재생](../../../../../results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/demo.html) / [수치 정본](../../../../../results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/final-measurement-summary.md) / [재현 방법](../../../../../benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md). 이번 전체 실행은 calibration·교정 이력을 포함해229회/1,274호출/$2.3201984이며 발표용 기본120의 비용은$1.0864784다.

**초기 실행 이력:** [11 초기 계획](11-minimal-poc-plan.md)에 따른 120회와 보완·진단 15회는 [12 초기 결과](12-poc-results.md), 한계와 정정 이유는 [13 사용자 리뷰](13-poc-user-review-and-remeasurement.md)에 보존했다. 그때의 **01 A / 02 A / 04 A / 05 C 및 02 A1**은 현재 최종 선택으로 사용하지 않는다. [초기 재생 화면](../../../../../results/architecture-evaluation/current/four-dp-qwen37-20261018/evidence-browser.html)도 당시 기록이다. 참조 Architecture와 accepted ADR은 유지한다.

## 결론

**최종 주요 DP 세트는 네 개다. 현재 요청의 근거 공급, 후속 입력의 의미 판단 실행, 화면과 발화의 지칭 결합, Agent 간 업무 인계다.** 네 개로 맞춘 결과가 아니라 일곱 문제의 구조와 정상 사용의 품질 선택을 각각 판정한 결과다. DP-44, Agent 선택과 DP-42는 독립 상세 설계로 남기고 기본안과 재검토 조건을 정했다.

| 자료 | VIA에서 선택해야 하는 것 | 판정 | 설계 분석 당시 기본안 권고 |
| --- | --- | --- | --- |
| [01 근거 공급](01-evidence-supply.md) | 이미 읽은 근거를 대화 문맥에 유지할지, 요청 때 구성할지 | **주요 DP**. 문제 1 + DP-45 통합 | **A** 유지 작업 문맥과 필요한 원본 보완 |
| [02 의미 판단 실행](02-interpretation-execution.md) | 앞선 VIA 판단 중 새 입력을 같은 실행에 반영할지, 요청별 실행으로 나눌지 | **주요 DP**. 문제 2 독립 | **B** 공통 대화와 요청별 해석 실행 |
| [03 처리 단계 연결](03-execution-coordination.md) | 비동기 작업의 다음 단계를 중앙 Dispatcher가 시작할지, 소비 Stage가 시작할지 | 독립 상세 설계. DP-44 유지 | **A** 중앙 비동기 Dispatcher |
| [04 화면과 발화의 지칭 결합](04-screen-speech-grounding.md) | 당시 원본을 함께 판단할지, 별도 화면/발화 후보를 생산해 결합할지 | **주요 DP**. 문제 3 + DP-47 통합 | **A** 원본에서 결합 판단 |
| [05 업무 인계](05-agent-handoff.md) | 위임 때 자료를 구성할지, 지속 기록을 유지할지, 수신자가 세부를 읽게 할지 | **주요 DP**. 문제 4 독립 | **A** 위임 시 목적별 인계 묶음 |
| [06 Agent 선택](06-agent-selection.md) | 요청 해석과 함께 고를지, 별도 선택 실행으로 넘길지 | 독립 상세 설계. 문제 5 유지 | **A** 요청 해석과 선택 결합 |
| [07 상태 확정과 서비스 경계](07-lifecycle-boundaries.md) | 관련 대화/업무 변경을 함께 확정할지, 독립 서비스에서 접수와 반영을 나눌지 | 독립 상세 설계. DP-42 유지 | **A** 모듈화한 통합 Core |

01~07은 이 자료의 읽기 순서이며 새 공식 DP 번호가 아니다. 위 권고는 실행 전 설계 분석의 이력이다. **실제 측정 후 권고는 15와 10을 따른다.** 어느 권고도 사용자 채택이나 accepted ADR 변경을 뜻하지 않는다.

## 읽는 순서

1. **[15 보완 PoC 결과와 설계 선택](15-corrected-poc-results.md)**: 현재 수치, 모든 QA의 구조적 설명, 시나리오별 손익·실패와 보완 전후.
2. **[10 발표용 비교와 선택 원고](10-presentation-outline.md)**: 기본 구조 비교 → 선택 이유 → 약한 QA의 보완 순서로 발표할 문장과 그림.
3. **위 표의 01~07 상세 분석과 [설계 그림 안내](diagrams/README.md)**: 문제·처리·상태·정정·참고 구현·대안과 편집 가능한 그림. 본문의 초기 추천과 현재 측정 후 추천은 상단에서 구별한다.
4. **[14 보완 실행 계약](14-corrected-poc-contract.md)**: 실행 전 예상, 같은 조건과 분모, 기능 변경, 수정 이유와 phase 분리.
5. **[08 DP 세트의 선정 이유](08-final-dp-set.md)와 [09 QA 대응·확인 범위](09-qa-tradeoffs.md)**: 일곱 문제에서 네 주요 DP를 가져온 이유와 과정 평가에서 아직 약한 차별 축.
6. **[11 초기 계획](11-minimal-poc-plan.md) → [12 초기 결과](12-poc-results.md) → [13 사용자 리뷰](13-poc-user-review-and-remeasurement.md)**: 첫 실험의 수치와 사용자 리뷰로 확인한 결함·수정 이력. 현재 표의 수치로 재사용하지 않는다.

근거가 궁금하면 [공식 구현 원장](../../../../references/seven-problem-review/README.md), 검토 결과는 [독립 검토와 수정 기록](reviews/README.md)을 읽는다. [분석 기준](00-analysis-contract.md)은 공통 기능과 제외 규칙을 정의한다.

## 이번에 완성한 것과 확인하지 않은 것

일곱 문제 모두에 기능 완료 조건, 복수 시나리오, 실제 구현에 근거한 대안, 상태와 오류/복구, QA01~06 인과, 추천과 반대안 선택 조건, 선택 후 보완을 작성했다. 비교 그림은 각각 SVG, 편집 가능한 draw.io, PNG를 제공한다. OpenClaw, OpenHands, Deep Agents, AutoGen Magentic-One, CrewAI와 관련 원 논문/실행 SW의 근거를 연결했다.

**설계 분석과 수치 산출이 과정의 모든 QA×DP 차별화 조건 충족을 뜻하지는 않는다.** 현재 측정에서는 QA04의 별도 생산 경로 수정 차이가 확인됐지만, QA02의 두 번째 강한 DP와 전체 제품의 QA06 우위는 여전히 신중하게 구별해야 한다. [09](09-qa-tradeoffs.md)는 실행 전 판단과 현재 확인 범위를 나누고 [15](15-corrected-poc-results.md)는 실제로 확인한 메커니즘과 조건을 제공한다.

위 설계 분석 이후 사용자 승인으로 실제 Qwen 비교를 수행했다. 전체 Windows 제품 검증, 기존 0~6 점수의 재산정과 최종 사용자 채택은 이번 PoC로 자동 성립하지 않는다. 기존 QA 정의/점수/COMMON-28과 참조 Architecture, ADR은 유지한다.
