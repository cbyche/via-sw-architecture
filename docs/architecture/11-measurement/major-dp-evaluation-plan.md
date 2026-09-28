# Core DP Evaluation Plan

> **Status: ACTIVE — Measurement Contract Definition**
>
> **Core ASRs:** QA-09, QA-19, QA-29, QA-39
>
> **Core DPs:** VIA-DP-03, VIA-DP-05, VIA-DP-17, VIA-DP-06, VIA-DP-07, VIA-DP-15
>
> **Current next work:** VIA-DP-03 capability qualification and pre-result Measurement Freeze

이 문서는 새 session이 과거 runner의 진행 순서를 이어가지 않고 현재 Core DP 작업을
시작하기 위한 실행 원장이다. Core DP의 의미와 선정 이유는
[Current Architecture Focus](../12-decisions/dp-executive-summary.md), metric은
[Core ASR Contract](../08-quality-attributes/core-asr-contract.md)가 기준이다.

## 1. 실행 순서

| 순서 | DP | 먼저 확정할 질문 | 현재 상태 |
| ---: | --- | --- | --- |
| 1 | VIA-DP-03 | S2S가 native source timestamp·revision evidence를 제공할 수 있는가? 제공할 수 없다면 A/B가 동등 기능을 만족하는가? | **NEXT — contract not frozen** |
| 2 | VIA-DP-05 | Context read-set을 semantic execution 전에 닫을지, 같은 execution에서 bounded 확장할지 | PENDING |
| 3 | VIA-DP-17 | 선택된 Source를 canonical value로 확정할지, consumer별 view로 확정할지 | PENDING |
| 4 | VIA-DP-06 | 최종 semantic decision을 통합 authority가 소유할지, 단계별 authority가 correction과 함께 소유할지 | PENDING |
| 5 | VIA-DP-07 | owner 간 edge는 VIA가 유지하면서 owner-affinity bundle 내부 edge authority를 어디에 둘지 | PENDING |
| 6 | VIA-DP-15 | Agent event를 continuous progress state로 확정할지, hint/cursor와 authoritative snapshot으로 확정할지 | PENDING |

이 순서는 문서·구현 작업 순서이며 DP winner dependency가 아니다. DP-05와 DP-17은
독립 축이다. 한 DP가 약하다고 VIA-DP-11을 자동으로 투입하지 않는다. DP-11은 실제
Process-fatal integration이 제품 범위에 들어오는 별도 scope decision이 있을 때만 다시
검토한다.

## 2. 한 DP를 완료하는 절차

다음 단계를 한 DP 안에서 순서대로 완료한 뒤 다음 DP로 이동한다.

1. **Decision contract** — A/B를 구분하는 authority·state·contract·call graph·deployment
   또는 fault boundary를 하나로 명시한다.
2. **Capability qualification** — 양쪽이 같은 user-visible 기능과 oracle을 제공할 수 있는지
   확인한다. 필수 기능 미지원은 낮은 점수가 아니라 `UNRESOLVED` 또는 부적합이다.
3. **Core-ASR applicability** — QA-09 trial, QA-19 field, QA-29 change, QA-39 fault를
   `PRIMARY`, `REGRESSION_ONLY`, `NOT_APPLICABLE`, `UNRESOLVED`로 동결한다.
4. **Measurement Freeze** — fixture, endpoint, 반복 수, timeout, aggregation, failure
   treatment, target, score band와 digest를 결과 전에 확정한다.
5. **Candidate Implementation** — 다른 DP 조건을 고정하고 현재 DP 축만 다른 A/B를 만든다.
6. **Qualification** — 정상 trace와 endpoint·semantic·binding·recovery를 깨뜨린 sentinel이
   각각 예상대로 PASS/FAIL하는지 검증한다.
7. **Paired A/B campaign** — 실패와 timeout을 포함한 raw evidence를 immutable directory에
   기록한다.
8. **Independent replay** — raw evidence만으로 QA-09/19/29/39 분자·분모와 평균을 다시
   생성한다.
9. **Architecture Decision** — 구조적 원인, 양방향 trade-off, tactic과 재검증 조건을
   ADR에 기록한다.

## 3. 비교 불변조건

- A/B의 user goal, completion condition, fixture, oracle와 external dependency profile은 같다.
- S2S Model 1개와 semantic LLM 1개를 공유하며 Component별 모델을 추가하지 않는다.
- Candidate 입력에 evaluator-only oracle을 넣지 않는다.
- Agent queue/execution처럼 metric에서 제외하는 시간도 secondary wall-clock evidence에서
  삭제하지 않는다.
- 실패·timeout을 성공 sample 선택으로 제거하지 않는다.
- 다른 DP가 물리적으로 참여하지 않으면 인과관계를 만들어 matrix를 채우지 않는다.
- 동일 execution을 재사용하면 `source_execution_key`로 연결하고 독립 sample처럼 복제하지
  않는다.
- 16-configuration full-factorial은 interaction 분석일 뿐 primary winner selection이 아니다.

## 4. 결과 저장과 상태

현재 [active benchmark](../../../benchmark/architecture/README.md),
[active candidates](../../../prototypes/candidates/README.md),
[current results](../../../results/architecture-evaluation/current/README.md)는 새 generation을
위해 reset되어 있다. 여섯 Core DP의 새 결과는 모두 `NOT_RUN`이다.

새 result는 `results/architecture-evaluation/current/dpNN-<campaign>-vN-YYYYMMDD/`에
추가하고 기존 directory를 덮어쓰지 않는다. Contract, fixture/oracle digest, source and
environment manifest, raw trace, replay receipt, `summary.json`과 `report.md`를 함께 둔다.

2026-09-27까지의 runner, candidate, result와 이전 실행 로그는
[pre-Core-ASR cleanup archive](../../archive/pre-core-asr-evaluation-cleanup-2026-09-28/README.md)에
보존했다. 그것을 실행하거나 숫자를 다시 집계해 current evidence로 만들지 않는다.

## 5. VIA-DP-03의 현재 단일 작업

다음 session은 구현부터 시작하지 않고 아래 산출물을 먼저 작성한다.

1. 평가할 실제 S2S product/profile 목록과 version·deployment 조건
2. partial/final transcript, source timestamp, revision/retraction, word/span alignment,
   referent evidence의 capability matrix
3. local capture-time evidence A와 S2S-native evidence B가 같은 user-visible 기능을
   제공하는지에 대한 gate
4. QA-09 trial, QA-19 field, QA-29 change, QA-39 fault 후보 모집단
5. 결과를 보기 전 승인할 VIA-DP-03 Measurement Freeze 초안

이 다섯 항목이 확정되기 전에는 active runner, candidate, result를 만들지 않는다.
