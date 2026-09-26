# VIA Architecture evaluation — 실행 요약 보고서

> 2026-09-27 · target Mac reference harness · Architecture winner/ASR 최종 확정 전

## 1. 결론

이번 실행에서 **공식 package까지 완결된 DP는 VIA-DP-06과 VIA-DP-11 두 개**다.
두 package 모두 후보별 active QA 19행을 빠짐없이 가지며, 숫자를 만들 수 없는 행을
임의 PASS나 proxy로 채우지 않고 `N/A` 또는 `BLOCKED`로 분리했다.

- **VIA-DP-06:** A 통합 semantic authority가 correctness와 QA-01에서 우세했다.
  B′ tactic은 B의 모델 호출과 지연을 줄여 QA-02/05 frozen-case 최대시간에서 가장
  짧았지만 A의 correctness 수준을 회복하지 못했다.
- **VIA-DP-11:** 별도 worker A는 integration-client fatal에서 무관한 기능의 초과
  중단을 **0개**로 제한했고, 같은 Process B는 **4개**를 중단시켰다. B는 target-Mac
  peak memory p95가 약 **3.7 MiB** 작았다. 복구시간 차이는 0.3 ms 미만이었다.

따라서 실제 양방향 trade-off가 가장 명확한 현재 조합은 **DP-11 × QA-32/QA-41**이다.
DP-06에서는 A와 B′가 correctness 대 responsiveness에서 갈렸지만 case당 1회라
분산과 p95를 주장할 수 없다. 두 결과만으로 Architecture winner나 ASR을 확정하지 않는다.

## 2. VIA-DP-06 핵심 결과

Evidence: `MEASURED_REFERENCE_HARNESS`. 24 case × A/B/B′ = 72 trial이며 같은 local
Qwen3-8B 1개, synthetic Context output, 외부 Reference Agent와 BlackHole loopback을
사용했다. 최대시간은 case당 1회인 frozen corpus maximum이지 p95가 아니다.

| QA | A 통합 authority | B 단계별 authority | B′ B+tactic | 읽는 법 |
| --- | ---: | ---: | ---: | --- |
| QA-01 최대 / 평균 | 17.8s / 13.4s | 27.7s / 22.4s | 24.3s / 15.3s | A 우세 |
| QA-02 최대 / 평균 | 19.7s / 10.5s | 25.5s / 17.6s | 15.9s / 11.5s | 최대는 B′, 평균은 A |
| QA-05 최대 / 평균 | 38.4s / 16.1s | 27.5s / 16.0s | 24.3s / 14.6s | B′ 우세 |
| QA-11 strict | 8/24 (33.3%) | 6/24 (25.0%) | 6/24 (25.0%) | A 우세 |
| QA-12 strict / field | 33.3% / 78.6% | 25.0% / 66.5% | 25.0% / 67.3% | A 우세 |
| QA-13 strict | 17/24 (70.8%) | 13/24 (54.2%) | 14/24 (58.3%) | A 우세 |
| QA-61 / QA-62 | 100% / 100% | 100% / 100% | 100% / 100% | 동률 |
| Model calls | 26 | 70 | 52 | B′가 B 약점 완화 |

정식 보고서: [DP-06 v4](./dp06-evaluation-v4-20260927/report.md)

## 3. VIA-DP-11 핵심 결과

Evidence: `MEASURED_REFERENCE_HARNESS`. 동일한 VIA-owned Agent Client를 A는 별도
supervised worker, B는 Core Process에 배치했다. 정상 workload와 fault workload는
동일 Reference Agent 구현·계약의 독립 state strata로 실행했다.

| QA | A 별도 worker | B 같은 Process | 읽는 법 |
| --- | ---: | ---: | --- |
| QA-31 최악 fault p95 | 11.5 ms | 11.3 ms | 실질 차이 없음 |
| QA-32 최대 초과 중단 | **0 units** | **4 units** | A의 명확한 fault containment 우세 |
| QA-41 peak memory p95 | 7,813.2 MiB | 7,809.5 MiB | B 약 3.7 MiB 우세; shared model이 대부분 |
| QA-61 trace complete | 100% | 100% | 동률 |
| QA-62 replay | 100% | 100% | 동률 |

QA-01/03/05의 renderer proxy, QA-11/13/14의 부분 predicate와 QA-21/23의 design
ledger는 공식 값으로 승격하지 않고 `BLOCKED`로 남겼다.

정식 보고서: [DP-11 v4](./dp11-evaluation-v4-20260927/report.md)

## 4. 18개 DP 전체 진행 상태

| DP | 현재 증거 상태 | 이번에 확인한 것 | 공식 QA 값이 아직 없는 이유 |
| --- | --- | --- | --- |
| 01 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | executable direct/Agent scope 후보 없음 |
| 02 | PROTOTYPE_PRIMITIVES_ONLY | handoff invariant tests PASS | 공동 commit 대 reconciliation 전체 후보·oracle 미구현 |
| 03 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | 실제 Voice evidence 후보 없음 |
| 04 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | S2S 게시 authority 후보·audible corpus 없음 |
| 05 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | Context read-set 두 후보와 source fixture 미구현 |
| 06 | **COMPLETE_REFERENCE_CAMPAIGN** | 72 raw trial, 19-QA report, replay PASS | 반복 p95·6개 관련 QA contract는 후속 필요 |
| 07 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | compound execution 후보·oracle 미구현 |
| 08 | PROTOTYPE_PRIMITIVES_ONLY | restart/recovery primitive tests PASS | event+checkpoint 대 snapshot+outbox 전체 후보 미구현 |
| 09 | PROTOTYPE_PRIMITIVES_ONLY | edge/core adapter invariant tests PASS | A-01~09 실제 source-change pack 미구현 |
| 10 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | session lifecycle 후보·stream fixture 미구현 |
| 11 | **COMPLETE_TARGETED_CAMPAIGN** | fault/recovery/memory/trace, 19-QA report, replay PASS | Voice/correctness/change-locality 일부 BLOCKED |
| 12 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | publish/commit crash 후보·evidence oracle 미구현 |
| 13 | PROTOTYPE_PRIMITIVES_ONLY | fixed load driver smoke PASS | reserved-control 대 shared-priority scheduler 미구현 |
| 14 | PROTOTYPE_PRIMITIVES_ONLY | shared writer/per-Task writer invariant tests PASS | frozen race/load campaign과 19-QA endpoint 미구현 |
| 15 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | push-confirm/query-confirm 후보·event source fixture 미구현 |
| 16 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | model-history state 후보·reconnect fixture 미구현 |
| 17 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | materializer/projection 후보·source change pack 미구현 |
| 18 | DOCUMENT_ONLY | A/B 보고서·19-QA 사고실험 | authorization use-gate 후보·revocation fixture 미구현 |

`PROTOTYPE_PRIMITIVES_ONLY`와 unit/smoke PASS를 공식 A/B 측정으로 부르지 않는다. 특히
아직 없는 후보에 손으로 숫자를 넣거나 관련성이 있다는 이유만으로 19개 QA를 모두
측정값처럼 보이게 하지 않았다.

## 5. 다음 우선순위

다음 implementation 순서는 상태·권한 소유권과 현재 primitive 재사용 가능성을 함께
고려해 **DP-14 → DP-09 → DP-02 → DP-08 → DP-13**이다. 각 DP는 같은 절차를 따른다.

1. A/B mutually exclusive·steelman 재감사
2. 19개 QA별 실제 참여 graph와 `MEASURED/N/A/BLOCKED` 사전 동결
3. 두 executable candidate와 machine oracle 구현
4. sentinel qualification 뒤 단 한 번의 official campaign
5. raw append, 독립 replay, 19-QA package 검사

DP-06/11에서 확인한 가장 큰 교훈은 **“19행을 채우는 것”과 “19개를 측정한 척하는
것”은 다르다**는 점이다. 현재 결과는 실제 endpoint가 있는 축만 숫자로 남겼기 때문에
후속 DP와 비교 가능한 기준선으로 사용할 수 있다.
