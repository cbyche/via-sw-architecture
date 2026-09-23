# 12-07. Gate 2 Full-Factorial Results

> **W12-G2 notice:** 이 문서의 W-01~W-03 값·score·contrast는 이전 정의의 historical/superseded evidence다. 새 Voice 중심 정의는 [`11e-voice-responsiveness-measurement-redefinition.md`](./11e-voice-responsiveness-measurement-redefinition.md)를 따르며 아직 측정되지 않았다. 따라서 아래 `ABAB` 권고의 W-01~W-03 근거는 새 정의에 이관되지 않는다.

> 상태: 16개 `2^4` configuration 평가 완료, W-05만 `BLOCKED_OPENROUTER_KEY`
>
> source commit: `c8869c884102d848b1b1c43d6d6de966062a591e`
>
> freeze fingerprint: `793e4c59fe2e1ddf4fe9978cad9e844a4f86b55d97ebba5884e61c440164ac20`

## 1. 결론

`IR / TASK / AGENT / EXEC` 네 축의 A/B를 조합한 16개 configuration을 모두 평가했다. 기존 5개 OFAT 결과에서 직접 확인하지 못했던 `ABAB`와 나머지 10개 결합도 결과에 포함했다.

현재 통합 권고는 `ABAB`다.

- IR A — Integrated Semantic Authority: **interim 유지**. W-01/W-02 reference raw 값은 모든 context에서 A가 낮지만 모두 같은 score band이며 W-05가 없다.
- TASK B — Per-Task Durable Supervisors: **reference evidence로 유지**. W-04 raw 값은 8/8 context에서 B가 낮고 4/8 context에서 score band가 갈린다. W-02는 A가 더 낮지만 score는 동점이다.
- AGENT A — Edge-normalized Canonical Contract: **확정 근거 유지**. W-07이 8/8 context에서 4점 대 3점으로 갈린다.
- EXEC B — Process-isolated Integration Runtime: **structural evidence로 유지**. W-09는 모든 context에서 약 176.3ms 낮고, W-04도 8/8 context에서 raw 개선이다. W-01/W-02/W-03 reference 비용은 소폭 증가하지만 score는 변하지 않는다.

다만 weighted total과 global winner는 계산하지 않았다. W-05가 비어 있고 reference timing은 production wall-clock 실측이 아니기 때문이다.

## 2. 무엇을 실제로 16개로 했는가

Configuration ID의 네 글자는 `IR / TASK / AGENT / EXEC` 순서다.

```text
AAAA AAAB AABA AABB ABAA ABAB ABBA ABBB
BAAA BAAB BABA BABB BBAA BBAB BBBA BBBB
```

- W-01/02/03/04: 16개 모두 frozen logical timing model과 사전 고정 interaction term을 적용했다.
- W-06/11/12: 16개 모두 통합 trace와 deterministic fixture를 실행했다.
- W-07/08: frozen design-change ledger를 각 configuration의 해당 choice에 매핑했다. 비인과 축의 반복값을 독립 표본으로 세지 않았다.
- W-09: TASK A/B whole-process 800 trials와 EXEC A/B integration-fatal 400 trials을 수행하고, 16개 configuration의 TASK×EXEC 선택으로 6-strata metric을 조립했다.
- W-10: EXEC A/B 각각 28 cells × 10 trials, 총 560 trials을 수행했다. IR/TASK/AGENT는 이 metric의 인과 축이 아니므로 동일 EXEC 결과를 사용했다.
- W-05: OpenRouter key가 없어 실행하지 않았고 0점이나 추정값을 넣지 않았다.

따라서 이것은 **16개 architecture configuration의 full-factorial reference/structural 평가**다. 16개 production binary를 각각 실제 UI·live Agent·live S2S로 실행한 제품 qualification은 아니다.

## 3. 전체 configuration 결과

아래 값은 metric value다. Score는 별도 열을 늘리지 않기 위해 4절의 핵심 configuration에 표시한다.

| Config | W-01 ms | W-02 ms | W-03 ms | W-04 ratio | W-06 % | W-07 | W-08 | W-09 ms | W-10 % | W-11 % | W-12 % |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| AAAA | 303.3 | 70 | 29 | 1.06 | 100 | 1.444 | 1.933 | 551.0 | 100 | 25 | 0 |
| AAAB | 308.3 | 70 | 29 | 1.05 | 100 | 1.444 | 1.933 | 374.7 | 100 | 25 | 0 |
| AABA | 303.3 | 73 | 32 | 1.06 | 100 | 1.778 | 1.933 | 551.0 | 100 | 25 | 0 |
| AABB | 308.3 | 77 | 36 | 1.05 | 100 | 1.778 | 1.933 | 374.7 | 100 | 25 | 0 |
| ABAA | 303.3 | 73 | 29 | 1.05 | 100 | 1.444 | 1.933 | 550.8 | 100 | 25 | 0 |
| **ABAB** | **308.3** | **73** | **29** | **1.03** | **100** | **1.444** | **1.933** | **374.5** | **100** | **25** | **0** |
| ABBA | 303.3 | 79 | 32 | 1.05 | 100 | 1.778 | 1.933 | 550.8 | 100 | 25 | 0 |
| ABBB | 308.3 | 83 | 36 | 1.03 | 100 | 1.778 | 1.933 | 374.5 | 100 | 25 | 0 |
| BAAA | 327.3 | 85 | 29 | 1.06 | 100 | 1.444 | 1.933 | 551.0 | 100 | 25 | 0 |
| BAAB | 338.3 | 85 | 29 | 1.05 | 100 | 1.444 | 1.933 | 374.7 | 100 | 25 | 0 |
| BABA | 327.3 | 88 | 32 | 1.06 | 100 | 1.778 | 1.933 | 551.0 | 100 | 25 | 0 |
| BABB | 338.3 | 92 | 36 | 1.05 | 100 | 1.778 | 1.933 | 374.7 | 100 | 25 | 0 |
| BBAA | 327.3 | 92 | 29 | 1.05 | 100 | 1.444 | 1.933 | 550.8 | 100 | 25 | 0 |
| BBAB | 338.3 | 92 | 29 | 1.03 | 100 | 1.444 | 1.933 | 374.5 | 100 | 25 | 0 |
| BBBA | 327.3 | 98 | 32 | 1.05 | 100 | 1.778 | 1.933 | 550.8 | 100 | 25 | 0 |
| BBBB | 338.3 | 102 | 36 | 1.03 | 100 | 1.778 | 1.933 | 374.5 | 100 | 25 | 0 |

## 4. `ABAB` scorecard

| Metric | Value | Score | Evidence |
|---|---:|---:|---|
| W-01 | 308.3ms | 5 | simulated S2S/reference delivery |
| W-02 | 73ms | 5 | representative acceptance stub |
| W-03 | 29ms | 5 | minimal reference UI sink |
| W-04 | 1.03 | 5 | frozen concurrency mock |
| W-05 | NOT_RUN | — | OpenRouter key required |
| W-06 | 100% | 5 | 30 cases × 5 fixture replay |
| W-07 | 1.444 | 4 | design-change analysis |
| W-08 | 1.933 | 4 | design-change analysis |
| W-09 | 374.5ms | 5 | measured structural recovery |
| W-10 | 100%, 28/28 | 5 | measured structural containment |
| W-11 | 25% | 3 | synthetic protected-unit corpus |
| W-12 | 0%, 0/24 | 5 | fixed policy opportunities |

W-11의 3점은 모든 configuration에 공통이다. W-05가 없으므로 점수 합계나 순위를 만들지 않는다.

## 5. Main effects와 interaction

Main effect는 다른 세 축을 모두 평균한 `B - A`다. Lower-is-better metric에서 양수는 A에, 음수는 B에 유리하다.

| Metric | Main effect | Pairwise interaction | 해석 |
|---|---|---|---|
| W-01 | IR +27ms, EXEC +8ms | IR×EXEC +6ms | reference responsiveness는 A/A 방향이 낮지만 전부 5점 |
| W-02 | IR +17ms, TASK +6.5ms, AGENT +6.5ms, EXEC +2ms | IR×TASK +4ms, TASK×AGENT +3ms, AGENT×EXEC +4ms | 결합 boundary 비용이 있으나 전부 5점 |
| W-03 | AGENT +5ms, EXEC +2ms | AGENT×EXEC +4ms | typed-Core + isolated worker 결합 비용, 전부 5점 |
| W-04 | TASK -0.015, EXEC -0.015 | TASK×EXEC -0.01 | TASK B와 EXEC B가 함께일 때 mock contention ratio가 추가 개선 |
| W-09 | TASK 약 -0.17ms, EXEC 약 -176.33ms | TASK×EXEC 약 0 | recovery 차이는 사실상 EXEC fault boundary가 지배 |
| W-07 | AGENT +0.333 | N/A | AGENT B의 변경 범위가 더 큼 |
| W-06/08/10/11/12 | 0 | 0 또는 N/A | 이번 fixture/metric에서는 구분하지 못함 |

W-01~W-04 interaction은 결과를 본 뒤 맞춘 값이 아니라 `reference-campaign-contract.md`에 먼저 고정한 reference-model 항이다. 실제 Windows runtime에서 같은 크기로 나타난다는 뜻은 아니다. W-09의 약 `-0.17ms` TASK 차이는 score가 같고 run-to-run noise 수준으로 해석한다.

## 6. DP별 context robustness

각 DP는 나머지 세 축의 8개 context에서 A/B를 직접 짝지었다.

| DP / Metric | A 우세 | B 우세 | 동점 | Score split | 판단 |
|---|---:|---:|---:|---:|---|
| IR / W-01 | 8 | 0 | 0 | 0/8 | A raw 우세, score 동점 |
| IR / W-02 | 8 | 0 | 0 | 0/8 | A raw 우세, score 동점 |
| TASK / W-02 | 8 | 0 | 0 | 0/8 | A raw 우세, score 동점 |
| TASK / W-04 | 0 | 8 | 0 | 4/8 | B raw 우세, 절반 context에서 score 분리 |
| TASK / W-09 | 0 | 8 | 0 | 0/8 | B 약 0.17ms, 실질 동점 |
| AGENT / W-03 | 8 | 0 | 0 | 0/8 | A raw 우세, score 동점 |
| AGENT / W-07 | 8 | 0 | 0 | 8/8 | A의 결정 근거가 context-independent |
| EXEC / W-01 | 8 | 0 | 0 | 0/8 | A raw 우세, score 동점 |
| EXEC / W-04 | 0 | 8 | 0 | 4/8 | B raw 우세, 절반 context에서 score 분리 |
| EXEC / W-09 | 0 | 8 | 0 | 0/8 | B 약 176.3ms 개선, score 동점 |
| EXEC / W-10 | 0 | 0 | 8 | 0/8 | A/B 모두 28/28 |

AGENT A는 score 기준으로 가장 강하게 robust하다. TASK B와 EXEC B는 raw 방향은 모든 context에서 유지되지만 W-04 score split 자체는 EXEC/TASK counterpart가 A인 절반 context에서만 생긴다. 따라서 두 ADR은 reference/structural evidence에 기반한 선택이며 Windows qualification 전에는 절대 성능 확정으로 해석하지 않는다.

## 7. 증거 파일

- Full-factorial ledger: `results/rebaseline/gate2-factorial-c8869c88/full-factorial.json`
- Reference 16 configurations + integrated traces: `results/rebaseline/gate2-factorial-c8869c88/reference-campaign/reference-summary.json`
- W-09/W-10 combined 16-row summary: `results/rebaseline/gate2-factorial-c8869c88/representative-summary.json`
- W-09 raw: `results/rebaseline/gate2-factorial-c8869c88/representative/w09-whole-process.json`, `w09-integration-fatal.json`
- W-10 raw: `results/rebaseline/gate2-factorial-c8869c88/representative-w10/w10-containment.json`
- W-07/W-08: `results/rebaseline/gate2-factorial-c8869c88/w07-w08-scores.json`
- Freeze: `results/rebaseline/gate2-factorial-c8869c88/source-manifest.json`, `measurement-approval.json`

다음 완료 조건은 W-05 hosted model run이다. Key는 환경변수로만 제공하고 repository에는 저장하지 않는다.
