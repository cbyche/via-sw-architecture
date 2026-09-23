# Natural Architecture Sensitivity Hypotheses

> 작성일: 2026-09-23
> 상태: **USER REVIEW DRAFT** — 최종 DP, 결과 또는 ASR 판정이 아니다.

## 1. 판정 원칙

제품적으로 중요한 관심사, 특정 DP의 A/B를 가르는 QA, 시스템 수준 ASR은 서로 다르다. 정상 기능을 충족한 합리적 대안 사이에서 같은 현실적 조건 아래 metric이 달라질 구조 인과가 있어야 Primary QA가 된다.

현재 활성 초안은 QA-01/02/03/05/07/08/09/11이다. QA-04/06/10/12의 이전 정의는 현행 QA가 아니다.

## 2. 예상되는 구조 민감도

| QA | 값이 달라질 수 있는 구조 원인 |
| --- | --- |
| QA-01~03 | call graph, process/IPC boundary, state lookup, validation, buffering, playback path |
| QA-05 | integrated/staged semantic pipeline, Context materialization, intermediate contract와 canonical Task binding |
| QA-07 | Agent-specific 차이를 adapter/contract boundary에 국소화하는 정도 |
| QA-08 | Model·Context·state·deployment 변화의 dependency direction과 ripple effect |
| QA-09 | state ownership, persistence, reconciliation, restart boundary와 recovery call graph |
| QA-11 | local/remote placement, Context packaging과 recipient별 filtering boundary |

정상 continuity는 QA-05의 Task/session/referent binding predicate로 측정하고, 장애 후 continuity는 QA-09 recovery correctness로 측정한다. 동시성은 QA-01~03·05·09의 workload condition이다. Failure containment은 QA-09의 원인 설명용 blast-radius trace다. Action/access safety는 모든 후보의 0-violation 필수 회귀다.

## 3. QA-05가 갈릴 수 있는 예

Integrated semantic path와 staged path는 같은 최종 책임을 수행해도 information loss, joint prompt/schema 복잡도, validation 위치가 다를 수 있다. Context를 미리 정규화할지 source handle로 유지할지에 따라서도 실제 모델에 제공되는 evidence가 달라진다. 이 차이가 사전 승인된 predicate 결과에 영향을 줄 때만 QA-05 인과로 인정한다.

단순 구현 버그, 다른 모델 품질, 시험기가 정답 Task ID를 주입한 fixture는 구조 우열의 증거가 아니다. 두 후보가 같은 relevant Context와 판단 결과를 보존하면 QA-05는 동점이어야 한다.

## 4. QA-09와 QA-11의 해석

QA-09는 장애가 어디까지 번졌는지가 아니라 **모든 영향 Task의 올바른 identity·state·result·control이 돌아온 시각**을 점수화한다. 격리 구조도 느리게 복구할 수 있고, 넓게 재시작한 구조도 빠르고 정확하게 복구할 수 있다. blast radius는 그 이유를 설명하는 secondary evidence다.

QA-11은 실제 unauthorized disclosure 허용량이 아니다. 그 값은 항상 0이어야 한다. 정상 기능을 유지하면서 recipient와 목적에 필요한 최소 단위를 넘겨 외부에 접근 가능하게 만든 단위 수가 구조에 따라 달라지는지를 본다. 모든 요청을 차단해 0을 만드는 후보는 기능 유지에 실패한다.

## 5. 사용 규칙

1. 합리적인 후보와 공통 tactic을 먼저 정의한다.
2. QA별 구조 인과와 applicable DP를 결과 전에 기록한다.
3. 실제 인과가 있는 QA만 그 DP의 Primary driver로 둔다.
4. 동점이나 non-discriminating QA를 숨기지 않는다.
5. ASR 여부는 점수 차이만으로 정하지 않고 제품 중요성·구조 위험·evidence를 별도 검토한다.
