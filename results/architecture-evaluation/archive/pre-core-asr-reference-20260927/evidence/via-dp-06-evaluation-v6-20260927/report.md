# VIA-DP-06 통합 평가 보고서 v6

> 기존 v5의 응답성·정확성 측정 252회는 다시 실행하지 않았다. 이 보고서는 그 원시 증거를 `source_execution_key`와 digest로 참조하고, 누락됐던 QA-21~23의 29개 변경을 A/B/B′ 각각에 독립 적용한 87개 구조 평가를 추가한다.

## 1. 평가 범위와 증거 수준

- 의사결정: `VIA-DP-06` — 요청 의미의 최종 확정 권한
- 결과 묶음: `via-dp-06-evaluation-v6-20260927`
- QA-21~23을 제외한 기존 축: v5 `MEASURED_REFERENCE_HARNESS` 증거를 참조 재사용
- QA-21~23 신규 축: `HYBRID_REFERENCE_ESTIMATE` — source anchor가 있는 실행 가능한 Architecture Element 원장
- 비교안: A=통합 의미 권한, B=단계별 의미 권한, B′=B+fast-path tactic
- B′의 fast path는 Architecture 구조를 바꾸지 않으므로 QA-21~23 원장은 B와 같다.

## 2. 핵심 결론

- **QA-21:** 세 안 모두 2.78개/변화로 같다. Agent 연동 변화는 대부분 공통 Agent 경계에서 끝나며 DP-06의 의미 권한 배치가 우열을 만들지 않았다.
- **QA-22:** 세 안 모두 1.93개/변화로 같다. 화면 계약 변경은 A의 통합 Coordinator 또는 B의 Grounding Resolver 한 곳까지만 전파되므로 단계 수 전체를 억지로 세지 않았다.
- **QA-23:** A는 2.80개/변화, B/B′는 3.60개/변화다. timing·correlation 변경이 A의 semantic producer 1곳과 B의 stage producer 3곳에 각각 닿기 때문이다. A가 평균 0.80개, 약 22.2% 적다.
- 따라서 DP-06의 변경용이성 trade-off는 QA-21/22가 아니라 **QA-23에서만 명확하게 나타났다.**
- 현재 초안 target을 대입하면 QA-21의 2.78은 2.0 이하를 충족하지 못하고, QA-22의 1.93은 3.0 이하를 충족한다. QA-23 target은 아직 `PENDING`이다.

## 3. 19개 QA 통합표

| QA | A | B | B′ |
| --- | --- | --- | --- |
| QA-01 | 23714.1 ms p95 (평균 22076.9, n=20) (v5 재사용) | 42338.2 ms p95 (평균 40460.9, n=20) (v5 재사용) | 15320.7 ms p95 (평균 14770.3, n=20) (v5 재사용) |
| QA-02 | 22380.9 ms p95 (평균 21647.3, n=20) (v5 재사용) | 43863.1 ms p95 (평균 40234.7, n=20) (v5 재사용) | 15058.3 ms p95 (평균 14463.2, n=20) (v5 재사용) |
| QA-03 | N/A | N/A | N/A |
| QA-04 | N/A | N/A | N/A |
| QA-05 | 24026.2 ms p95 (평균 22587.1, n=20) (v5 재사용) | 15196.8 ms p95 (평균 14444.6, n=20) (v5 재사용) | 13811.0 ms p95 (평균 13144.1, n=20) (v5 재사용) |
| QA-11 | 33.3% strict (8/24) (v5 재사용) | 25.0% strict (6/24) (v5 재사용) | 25.0% strict (6/24) (v5 재사용) |
| QA-12 | 33.3% strict / 78.6% field (v5 재사용) | 25.0% strict / 66.5% field (v5 재사용) | 25.0% strict / 67.3% field (v5 재사용) |
| QA-13 | 70.8% strict (17/24) (v5 재사용) | 54.2% strict (13/24) (v5 재사용) | 58.3% strict (14/24) (v5 재사용) |
| QA-14 | N/A | N/A | N/A |
| QA-15 | BLOCKED | BLOCKED | BLOCKED |
| QA-21 | 2.78개/변화 (25/9, 최대 4) | 2.78개/변화 (25/9, 최대 4) | 2.78개/변화 (25/9, 최대 4) |
| QA-22 | 1.93개/변화 (29/15, 최대 4) | 1.93개/변화 (29/15, 최대 4) | 1.93개/변화 (29/15, 최대 4) |
| QA-23 | 2.80개/변화 (14/5, 최대 4) | 3.60개/변화 (18/5, 최대 6) | 3.60개/변화 (18/5, 최대 6) |
| QA-31 | BLOCKED | BLOCKED | BLOCKED |
| QA-32 | N/A | N/A | N/A |
| QA-41 | BLOCKED | BLOCKED | BLOCKED |
| QA-51 | N/A | N/A | N/A |
| QA-61 | 100.0% (v5 재사용) | 100.0% (v5 재사용) | 100.0% (v5 재사용) |
| QA-62 | 100.0% (v5 재사용) | 100.0% (v5 재사용) | 100.0% (v5 재사용) |

`N/A`는 DP-06 구조가 해당 경로에 참여하지 않는다는 뜻이다. `BLOCKED`는 관련성은 있지만 승인된 전체 endpoint 또는 반복 측정이 아직 없다는 뜻이다. 이번 보완 범위는 QA-21~23이며, 기존 `BLOCKED` 축을 숫자로 대체하지 않았다.

## 4. QA-21~23 상세 결과

| QA | 후보 | 평균 변경 요소 | 합계/변화 수 | 최대 | C | I | S | D |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| QA-21 | A | 2.78 | 25/9 | 4 | 13 | 7 | 5 | 0 |
| QA-21 | B | 2.78 | 25/9 | 4 | 13 | 7 | 5 | 0 |
| QA-21 | B′ | 2.78 | 25/9 | 4 | 13 | 7 | 5 | 0 |
| QA-22 | A | 1.93 | 29/15 | 4 | 17 | 4 | 5 | 3 |
| QA-22 | B | 1.93 | 29/15 | 4 | 17 | 4 | 5 | 3 |
| QA-22 | B′ | 1.93 | 29/15 | 4 | 17 | 4 | 5 | 3 |
| QA-23 | A | 2.80 | 14/5 | 4 | 7 | 4 | 2 | 1 |
| QA-23 | B | 3.60 | 18/5 | 6 | 11 | 4 | 2 | 1 |
| QA-23 | B′ | 3.60 | 18/5 | 6 | 11 | 4 | 2 | 1 |

### QA-21 — Agent 변화 국소성

A-01~A-09를 각각 같은 baseline에서 시작했다. capability 추가·재구성처럼 의미 처리와 닿는 변화도 A의 Coordinator 한 곳 또는 B의 Handling Selector 한 곳에만 닿는다. 나머지는 공통 Agent adapter, lifecycle contract, execution link에 국소화된다. 그러므로 세 후보의 평균이 같은 것이 정상 결과다.

### QA-22 — Model·Context·State 변화 국소성

M-01~M-09와 C-01~C-06을 독립 적용했다. 특히 C-03 화면 연동 계약 변경에서 B의 Task Associator와 Handling Selector는 화면 원천 계약을 직접 소비하지 않고 GroundingResult를 소비한다. 따라서 실제 소비자인 Grounding Resolver만 변경 대상으로 세었다. 후보별 단계 상자 수를 그대로 변경 수로 바꾸지 않았다.

### QA-23 — 실험·로그 변화 국소성

E-01 timing span과 E-02 correlation dimension은 semantic event를 실제 생산하는 authority에 닿는다. A는 producer가 Coordinator 1곳이고, B/B′는 Grounding·Association·Handling 3곳이다. E-03 trace schema와 E-04 export 변경은 공통 observability 경계에만 닿고, E-05 새 실험 추가는 기존 generic assignment/exposure 계약 안의 configuration이므로 0개다.

## 5. 해석과 한계

- 이 수치는 코드 파일 수, 수정 line 수, 개발 일수가 아니다. 책임·계약·상태·배치라는 Architecture Element의 변경 범위다.
- QA-21~23은 실제 제품 migration을 구현한 실측이 아니다. 동결된 전체 요소 원장과 source anchor를 검증하고 변경을 기계적으로 적용한 구조 추정이다.
- 각 변경은 누적하지 않았다. 항상 동일 후보 baseline에서 시작했으며, 결과 원장에 수정·추가·제거 ID와 C/I/S/D breakdown을 남겼다.
- A/B의 최종 선택은 이 보고서만으로 하지 않는다. 응답성·정확성 결과와 QA 우선순위를 함께 검토해야 한다.

## 6. 재현 자료

- 신규 원시 자료: `raw/change-locality.jsonl` (87개 record)
- 기존 증거 연결: `source-evidence-map.json`
- 통합 기계 판독 결과: `summary.json`
- 입력·원시 자료 digest: `manifest.json`
- 독립 재생성 확인: `replay-receipt.json`
