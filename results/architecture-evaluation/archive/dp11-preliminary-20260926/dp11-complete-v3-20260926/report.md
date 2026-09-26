# VIA-DP-11 전체 19개 QA 평가 결과

> 근거 수준: `MEASURED_MODEL` + `MEASURED_REFERENCE_HARNESS` + `HYBRID_REFERENCE_ESTIMATE` + `ARCHITECTURE_LEDGER`
>
> 방안 A: 별도 Integration Worker process
> 방안 B: VIA Core process 안의 Agent Client

## 결론

동결한 한 campaign에서 active QA 19개를 모두 평가했다. 독립 analyzer가 source에서 계산되는 18개 row를 모두 정확히 재현했으므로 QA-62는 양쪽 모두 100%다.

동결 최소 차이를 넘은 조합은 QA-23 (B 우세), QA-32 (A 우세)뿐이다. 나머지 수치 차이는 최소 차이 미달이거나 공통·회귀 축이다. 따라서 이 보고서는 전체 승자를 선택하지 않는다.

QA-12는 69.57%, 통합 QA-11은 75.00%로 양쪽이 같다. 이는 공통 semantic 경로의 한계이며 DP-11 process placement 차이가 아니다. 결과에 맞춰 없애거나 A/B trade-off로 세지 않고 qualification 미달로 보존한다.

## 전체 DP × QA 표

시간·변경 요소·영향 단위·메모리·노출은 작을수록 좋다. 정확성·완전성·재현성은 클수록 좋다.

| QA | A | B | 동결 기준 판정 | DP-11에서의 역할 |
| --- | ---: | ---: | --- | --- |
| QA-01 Delegated Path VIA Responsiveness | 2168.181 ms | 2168.024 ms | 수치상 B 우세, 최소 차이 미달 | primary_candidate |
| QA-02 VIA Direct Voice Response Responsiveness | 2516.433 ms | 2516.433 ms | 공통/회귀 검증; DP-11 변별용 아님 | non_applicable_regression |
| QA-03 Agent Progress Voice Feedback Responsiveness | 0.476 ms | 0.358 ms | 수치상 B 우세, 최소 차이 미달 | primary_candidate |
| QA-04 Voice Interruption Responsiveness | 0.001 ms | 0.001 ms | 공통/회귀 검증; DP-11 변별용 아님 | non_applicable_regression |
| QA-05 Task Control Responsiveness | 8.444 ms | 8.336 ms | 수치상 B 우세, 최소 차이 미달 | primary_candidate |
| QA-11 VIA Request Handling Correctness | 75.00% | 75.00% | A/B 동일; 공통 qualification | qualification |
| QA-12 Request Semantic Resolution Correctness | 69.57% | 69.57% | 공통/회귀 검증; DP-11 변별용 아님 | non_applicable_regression |
| QA-13 Task & Interaction Binding Correctness | 100.00% | 100.00% | A/B 동일; 공통 qualification | qualification |
| QA-14 Async Task State   Convergence Correctness | 100.00% | 100.00% | A/B 동일; 공통 qualification | qualification |
| QA-15 Interaction & Task Continuity Correctness | 100.00% | 100.00% | A/B 동일; 공통 qualification | qualification |
| QA-21 Agent Change Locality | 1.78 elements | 1.78 elements | A/B 동일; 차이 없음 | candidate_assessment |
| QA-22 Model, Context & State Change Locality | 1.80 elements | 1.80 elements | 공통/회귀 검증; DP-11 변별용 아님 | non_applicable_regression |
| QA-23 Experiment & Logging Change Locality | 2.80 elements | 1.60 elements | 의미 있는 차이: B 우세 | supporting_candidate |
| QA-31 Correct Task Recovery Time | 12.426 ms | 14.829 ms | 수치상 A 우세, 최소 차이 미달 | primary_candidate |
| QA-32 Fault Blast Radius | 0 units | 4 units | 의미 있는 차이: A 우세 | primary_candidate |
| QA-41 Target Device Memory Footprint | 5053.8 MiB | 5050.0 MiB | 수치상 B 우세, 최소 차이 미달 | diagnostic_candidate |
| QA-51 Protected Data Exposure Minimization | 0 units | 0 units | A/B 동일; 공통 qualification | qualification |
| QA-61 Execution Trace Completeness | 100.00% | 100.00% | A/B 동일; 차이 없음 | primary_candidate |
| QA-62 Evidence Reproducibility | 100.00% | 100.00% | 재현 완료 | qualification |

## 실제로 이 DP를 구분하는 QA

- **QA-32:** A는 주입한 Integration Client fatal fault를 worker에 격리했다. B는 Core process와 함께 독립 사용자 기능 네 개를 잃었다. 가장 강한 구조적 trade-off다.
- **QA-23:** B는 worker IPC·supervision 경계가 없어서 frozen 실험·로그 change pack의 변경 Architecture Element가 더 적었다.
- **QA-01/03/05/31/41:** 수치 방향은 갈렸지만 절대 차이가 동결 기준에 못 미쳤다. 측정값은 보존하되 trade-off로 승격하지 않는다.
- **QA-02/04/12/22:** 공통 또는 회귀 축이다. DP-11이 이 경로를 소유하지 않으므로 동률이나 미세한 proxy noise가 정상이다.
- **QA-11/13/14/15/51/61/62:** qualification이다. 더 빠르거나 더 잘 격리된 후보가 부정확·불안전·추적 불가·재현 불가한 상태로 이기는 것을 막는다.

## Evidence package

- semantic model 23건, 정상 runtime 250건, fault 60건, change ledger 58건, memory 50건, 보호정보 노출 110건을 보존한다.
- Raw record는 `raw/`, runner가 만든 표는 `derived/summary.json`에 있다.
- `derived/verified-summary.json`은 독립 analyzer가 동결 contract와 raw evidence만으로 생성한다.
- Process placement는 공통 semantic 경로를 바꾸지 않으므로 같은 semantic-model record를 양쪽에 매핑한다.

## 한계

- Voice endpoint는 microphone-to-speaker acoustic loopback이 아니라 instrumented renderer/audio-buffer proxy다. `PRODUCT_E2E` 값이 아니다.
- Reference Agent는 deterministic하다. VIA Architecture 동작을 격리하며 실제 Agent reasoning·실행 품질을 측정하지 않는다.
- QA-21/22/23은 runtime timing이 아니라 frozen Architecture Element ledger다.
- 주입한 fatal fault는 그 fault가 존재할 때의 구조적 결과를 증명한다. 제품 의사결정에서의 가중치는 실제 Agent Client fault profile에 달려 있다.
