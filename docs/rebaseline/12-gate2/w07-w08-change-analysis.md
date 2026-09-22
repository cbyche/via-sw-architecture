# Gate 2 — W-07 / W-08 Raw Change Design Analysis

> 상태: **PRE-MEASUREMENT DESIGN_ANALYSIS / RAW LEDGER RULES COMPLETE / AGGREGATE METRIC NOT COMPUTED**  
> 기준: 07 §7.4의 24개 change, 10의 C/I/S/D 및 M/A/R 집계 규칙, 11-D의 W-07/W-08 고정 분모.  
> 목적: Measurement Freeze 전에 후보 결과를 보지 않고 change별 구조 영향 원장을 고정한다.

## 1. 이 문서가 하는 것 / 하지 않는 것

이 분석은 각 change를 같은 baseline candidate configuration에 **독립 적용**하여
`modified ∪ added ∪ removed` Architecture Element 수를 기록한다.

- A-01~09: W-07 raw input
- M-01~09 + C-01~06: W-08 raw input
- 증거 등급: `DESIGN_ANALYSIS`
- 기능 유지: 설계 논증만 기록하며 실제 replay 성공을 주장하지 않는다.
- **W-07/W-08 평균, 0~5 score, ranking, winner는 여기서 계산하지 않는다.**
- `0`은 누락이 아니라 10의 규칙에 따라 **기존 Architecture contract 안에서 configuration/data만 바뀌는 경우**에만 허용한다.

Machine source of truth:

- `benchmark/rebaseline/gate2/w07-w08-change-rules.json`
- `benchmark/rebaseline/gate2/change_analysis.py`

Analyzer는 현재 checkout의 `source_revision`과 C/I/S/D `catalog_fingerprint`를 묶어 8 pair-labelled candidate × 24 change = **192 raw rows**를 생성한다. 같은 5개 complete configuration을 공유하는 pair-label 사이 결과가 다르면 validation failure다.

## 2. Raw change ledger — Model M-01~09

| Change | Baseline M/A/R | Raw count | 설계 근거 |
|---|---|---:|---|
| M-01 S2S 제공자 교체 | M: `G2-C-S2SCLIENT` | 1 | provider별 연결/포장만 S2S adapter에서 변경. canonical S2S/Voice 의미는 유지 |
| M-02 의미 판단 모델 교체 | M: `G2-C-SEMCLIENT` | 1 | provider API/response wrapping을 SemanticAdapter에 국소화 |
| M-03 모델 실행 프로필 변경 | 없음 — CONFIGURATION_ONLY | 0 | 같은 location/API와 요구 기능을 만족하는 profile의 size/limit/sampling 값 변경 |
| M-04 External→Private Cloud | M: `G2-D-SEMANTIC` | 1 | 동일 logical contract의 remote deployment 환경 변경. S2S 대상으로 적용해도 D-S2S 1개로 동수 |
| M-05 Remote→사용자 PC | M: `G2-C-SEMCLIENT`, `G2-D-SEMANTIC` | 2 | connection/lifecycle + deployment 변경. S2S target도 client+D 2개로 동수 |
| M-06 사용자 PC→Remote | M: `G2-C-SEMCLIENT`, `G2-D-SEMANTIC` | 2 | remote 연결/실패 처리 + deployment 변경. 기존 Policy contract 사용은 추가 변경으로 세지 않음 |
| M-07 S2S event 정보 변경 | M: `G2-C-S2SCLIENT`, `G2-I-S2S`, `G2-C-VOICE` | 3 | word timing→segment/revision에 따라 adapter, canonical event, Voice timing binding 변경 |
| M-08 Semantic final→streaming | M: `G2-C-SEMCLIENT` | 1 | common SemanticAdapter가 partial/final/error/cancel을 final semantic contract로 정규화 |
| M-09 Model history 전달 계약 | M: `G2-C-HISTORY`, `G2-C-SEMCLIENT`, `G2-S-CONV` | 3 | provider conversation ID binding, history reconstruction, persisted mapping 변경. S2S target도 동일 count |

M-04~06, M-09는 change catalog가 특정 model role 하나를 강제하지 않으므로 semantic-model instantiation을 canonical row로 기록한다. 같은 change를 S2S role에 적용했을 때 **동일 C/I/S/D count가 되는 대응 요소**를 rule의 `scope_equivalence`에 함께 고정한다. 후보별로 유리한 role을 골라 count를 바꾸지 않는다.

## 3. Raw change ledger — Agent A-01~09

| Change | AGENT-DP01 A — Edge normalized | AGENT-DP01 B — Core-visible typed | Raw count A / B |
|---|---|---|---:|
| A-01 Agent 추가 | CONFIGURATION_ONLY | 동일 | 0 / 0 |
| A-02 동일 protocol Agent 교체 | M: `G2-C-NATIVE` | 동일 | 1 / 1 |
| A-03 다른 Agent protocol 추가 | M: `G2-C-NATIVE`, `G2-C-EDGESEM`; A: `G2-I-NATIVE-R` | M: `G2-C-NATIVE`, `G2-C-TYPEDHANDLER`, `G2-I-TYPEDAGENT`; A: `G2-I-NATIVE-R` | 3 / 4 |
| A-04 push→query status | CONFIGURATION_ONLY — TASK-T01 기존 path 선택 | 동일 | 0 / 0 |
| A-05 handle→thread+run/follow-up | CONFIGURATION_ONLY — baseline P/Q variation envelope | 동일 | 0 / 0 |
| A-06 capability contract 재구성 | M: `G2-C-REGISTRY`, `G2-S-CAP` | 동일 | 2 / 2 |
| A-07 Agent auth contract | M: `G2-C-NATIVE`, `G2-C-POLICY`, `G2-I-AUTH` | 동일 | 3 / 3 |
| A-08 질문/승인 reply lifecycle | M: `G2-C-NATIVE`, `G2-C-EDGESEM` | M: `G2-C-NATIVE`, `G2-C-TYPEDHANDLER`, `G2-I-TYPEDAGENT` | 2 / 3 |
| A-09 result artifact contract | M: `G2-C-NATIVE`, `G2-C-EDGESEM` | M: `G2-C-NATIVE`, `G2-C-TYPEDHANDLER`, `G2-I-TYPEDAGENT` | 2 / 3 |

A-01은 새 Agent **instance/capability record** 추가이며 `G2-C-REGISTRY/G2-S-CAP`의 schema나 처리 책임을 바꾸지 않는다는 현재 baseline 전제를 사용한다.  
A-04는 Event-first + Query Reconciliation을 이미 공통 tactic으로 보유하므로 provider profile 선택으로 처리한다.  
A-05는 baseline 자체가 P-style와 Q-style identity/follow-up variation을 함께 명세하므로 기존 sealed variation을 선택하는 경우로 센다. 이 전제가 달라지면 해당 change만 rebaseline해야 하며 candidate 결과를 본 뒤 count를 조정하지 않는다.

A-03/A-08/A-09에서는 A가 native lifecycle variation을 edge에서 canonicalize하고 B가 typed variation contract와 Core handler까지 노출한다는 **AGENT-DP01의 의도된 구조 차이**가 raw ledger에 나타난다. 이것은 아직 W-07 평균이나 score가 아니다.

## 4. Raw change ledger — Context / Storage C-01~06

| Change | Baseline M/A/R | Raw count | 설계 근거 |
|---|---|---:|---|
| C-01 Source 제공자 교체 | M: `G2-C-NATIVE`, `G2-C-CONTEXT` | 2 | native API/identity/field mapping + provider adaptation |
| C-02 DOCX format 추가 | M: `G2-C-CONTEXT` | 1 | ContextBroker의 source-format handling 확장 |
| C-03 화면 연동 API 변경 | M: `G2-C-CAPTURE` | 1 | handle/좌표/selection 변형을 Capture에서 canonicalize |
| C-04 Memory record format 확장 | M: `G2-C-MEMORY`, `G2-S-MEMORY`, `G2-C-STORE` | 3 | memory schema + service read/write/migration + repository migration |
| C-05 같은 종류 Source 추가 | M: `G2-C-NATIVE`, `G2-C-CONTEXT` | 2 | provider connection + source identity/scope 선택. 새 merge 기능은 추가하지 않음 |
| C-06 Conversation/Task record V1→V2 | M: `G2-C-STORE`, `G2-S-CONV`, `G2-S-TASK`, `G2-S-LINK`, `G2-S-PENDING`, `G2-S-DELIVERY` | 6 | 관계를 구성하는 persistent schemas와 migration behavior 변경 |

현재 Core 4-DP 후보의 M/C raw sets는 동일하다. 즉 이 원장에서는 **W-08에 candidate-specific 구조 차이를 인위적으로 만들지 않는다.** IR/TASK 등의 alternative가 실제 common adapter/Repository contract를 넘어 수정되어야 한다는 근거가 생길 때만 해당 rule을 명시적으로 rebaseline한다.

## 5. Validation invariants

CI는 다음을 실패 조건으로 둔다.

1. 24개 change가 정확히 M 9 / A 9 / C 6인지.
2. 8 pair candidate 모두 24개씩, 총 192 row인지.
3. Modified/Removed ID가 해당 complete configuration의 active element인지.
4. Added ID가 baseline active element가 아니며 C/I/S/D type이 명시됐는지.
5. M/A/R 집합이 서로 겹치지 않는지.
6. `CONFIGURATION_ONLY`만 0개를 가질 수 있는지.
7. 동일 complete configuration을 공유하는 pair-label의 change 결과가 동일한지.
8. A-03/A-08/A-09 외 Agent row에서 AGENT A/B count를 임의로 다르게 만들지 않았는지.
9. M/C raw count가 후보별로 다르지 않은 현재 rule을 유지하는지.
10. representative metric, score, winner가 계속 비어 있는지.

## 6. 다음 단계

이 raw ledger가 CI green이면 Measurement Freeze Review 입력에 포함한다. 리뷰에서는 change 의미와 M/A/R 판정의 타당성을 먼저 승인하고, **그 다음 frozen revision에서만** W-07의 9개 Agent change 평균과 W-08의 15개 non-Agent change 평균을 계산한다.

따라서 현재 단계의 산출물은 “점수”가 아니라 **재현 가능한 변경 근거 원장**이다.
