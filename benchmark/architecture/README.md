# Current Architecture Measurement Harness

> **Status: QA-01~15 common evaluator foundation implemented; official DP campaign NOT_RUN**

이 디렉터리는 현재 Architecture baseline에 맞는 machine-readable contract, fixture, executable harness, raw-trace validator, aggregation code를 구현할 active 위치다.

구현은 다음 source를 순서대로 따라야 한다.

1. [Voice Responsiveness](../../docs/architecture/08-quality-attributes/voice-responsiveness.md)
2. [Task Control Responsiveness](../../docs/architecture/08-quality-attributes/interaction-control-responsiveness.md)
3. [Correctness & Continuity](../../docs/architecture/08-quality-attributes/correctness-and-continuity.md)
4. [Reliability & Resource](../../docs/architecture/08-quality-attributes/reliability-and-resource.md)
5. [Observability](../../docs/architecture/08-quality-attributes/observability.md)
6. [Measurement Guide](../../docs/architecture/11-measurement/README.md)
7. [Evaluation Method](../../docs/architecture/12-decisions/evaluation-method.md)
8. 결과 전에 승인된 machine contract와 Measurement Freeze

## Expected layout

새 구현이 시작되면 목적별로 contract/fixture, runner, trace schema, analyzer, tests를 분리한다. Raw result나 generated summary는 이 디렉터리가 아니라 [results/architecture-evaluation/current](../../results/architecture-evaluation/current/README.md)의 새 freeze directory에 저장한다.

공식 DP 실행 전에 `contracts/qa01-15-foundation-v1.json`과
`qa01_15_evaluator.py`를 공통 기반으로 사용한다. 이 evaluator는 실제 event
provenance, QA-05 전체 path stage, QA-11~15의 모든 required predicate group과
명시적 `N/A`를 강제한다. 누락을 default PASS로 처리하지 않는다.

다음 qualification은 정상 trace PASS뿐 아니라 의도적으로 endpoint·semantic·binding·
state·continuity를 깨뜨린 sentinel이 정확한 failure code로 FAIL하는지 확인한다.

```bash
.venv/bin/python -m unittest \
  benchmark/architecture/tests/test_qa01_15_foundation.py -v
```

`run_dp11_complete.py`와 v1~v3 결과는 preliminary harness provenance다. 특히 v3는
19행 표와 raw replay 형식은 검증했지만 QA-01~15 전체 active contract를 만족하지
못했으므로 공식 DP/ASR 결과로 사용하지 않는다.

`run_dp06_evaluation.py`는 VIA-DP-06의 통합 의미 권한 A, 단계별 의미 권한 B와 B의
결정적 단계 생략 tactic B′를 같은 local Qwen3-8B와 Context Evidence로 실행한다.
후보 입력 `dp06-semantic-input-v4.json`과 evaluator-only
`dp06-semantic-oracle-v4.json`은 물리적으로 분리한다. Context Engine은 실행하지
않고 모든 후보에 같은 canonical synthetic Evidence를 재생한다. 24건 breadth
workload는 UC 출처를 기록하며 `D/W/C/P/G/Q/T/A/E/X`는 Test Case 접두사이지 DP나
QA 번호가 아니다.

중간 pilot/reference runner, contract와 결과는 삭제했다. 현재 DP-06 source of truth는
`via-dp-06-evaluation-v4.json`, v4 fixture 3종, `run_dp06_evaluation.py`,
`analyze_dp06_evaluation.py`뿐이다. 공식 결과는
[`dp06-evaluation-v4-20260927`](../../results/architecture-evaluation/current/dp06-evaluation-v4-20260927/report.md)에 있다.

DP-11의 active runner는 `via-dp-11-evaluation-v4.json`을 사용한다. 기존 v1~v3
결과와 analyzer는 preliminary provenance이며, v4는 proxy를 QA 값으로 승격하지 않고
실제 실행 가능한 fault recovery, blast radius, target-Mac memory와 trace만 점수화한다.
공식 결과는 [`dp11-evaluation-v4-20260927`](../../results/architecture-evaluation/current/dp11-evaluation-v4-20260927/report.md)에 있다.

```bash
cargo +1.98.1 build --locked --manifest-path prototypes/candidates/Cargo.toml \
  -p via-host -p via-worker -p via-reference-agent

# 공통 qualification, frozen fixture pack과 audio loopback adapter가 모두 준비된 뒤에만
# 새 DP runner를 실행한다.
```

Runner는 기존 output directory를 덮어쓰지 않는다. Raw JSONL과 derived summary를
분리하고 contract, binary, Git SHA와 exact command를 manifest에 기록한다.

저장된 raw evidence만으로 summary가 정확히 재현되는지는 다음처럼 확인한다.

```bash
.venv/bin/python benchmark/architecture/analyze_dp11_complete.py \
  --evidence results/architecture-evaluation/archive/dp11-preliminary-20260926/dp11-complete-v3-20260926 --check

.venv/bin/python benchmark/architecture/analyze_dp06_evaluation.py \
  --result-dir results/architecture-evaluation/current/dp06-evaluation-v4-YYYYMMDD
```

## Non-goals

- 실제 실행 없이 model/reference 상수를 합산해 E2E 측정이라고 부르기
- generated WAV를 읽지 않고 input evidence로 기록하기
- integrated trace를 사후 JSON 생성으로 대체하기
- DP와 무관한 축으로 동일 sample을 복제하기
- instrumented sink를 physical audible onset으로 부르기

기존 formula/reference campaign, timing adapter, freeze, result는 [W12-G1 benchmark archive](../archive/w12-g1/README.md)에 있으며 새 evidence를 생성하지 않는다.
