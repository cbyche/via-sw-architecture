# Gate 2 review tooling

> **HISTORICAL / SUPERSEDED:** The commands and W12-G1 contracts below are preserved for audit. They do not implement the current W-01~W-03 definitions and must not generate current evidence. See [`benchmark/architecture`](../../../architecture/README.md).

`catalog_check.py`는 GitHub Markdown을 원본으로 사용하여 **자체 리뷰 후 남은 Core 4개 DP**의 A/B inventory와 complete configuration을 조립한다. FP-INT01(S2S Direct Fast Path)과 TASK-T01(Event-first + Query Reconciliation)은 공통 fixed principle/tactic이며 score pair가 아니다.

```bash
python benchmark/rebaseline/gate2/catalog_check.py --output results/gate2-review
python -m unittest discover -s benchmark/rebaseline/gate2 -p 'test_*.py' -v
python benchmark/rebaseline/build_assets.py
python benchmark/rebaseline/gate2/terminal_outcome_audit.py --root .
python benchmark/rebaseline/gate2/change_analysis.py --output results/gate2-change-analysis
python benchmark/rebaseline/working12/score.py --validate-only
python benchmark/rebaseline/gate2/w04_foreground_contract.py \
  --contract benchmark/rebaseline/gate2/w01-foreground-strata.json \
  --validate-only
python benchmark/rebaseline/gate2/w01_observation_adapter.py \
  --contract benchmark/rebaseline/gate2/w01-foreground-strata.json \
  --trace benchmark/rebaseline/gate2/w01-observation-smoke.json
python benchmark/rebaseline/gate2/w12_safety.py \
  --oracle benchmark/rebaseline/gate2/w12-safety-opportunities.json \
  --validate-only
```

Measurement Freeze 승인 뒤 W-09/W-10 대표 실행은 raw Rust CLI를 직접 발표 근거로 사용하지 않고 approval-gated orchestrator를 통한다.

W-01/02/03/04/06/11/12 reference campaign은 `reference_campaign.py`를 사용한다. 생성 WAV와 instrumented reference delivery, acceptance stub, frozen functional fixtures를 사용하며 `docs/rebaseline/12-gate2/reference-campaign-contract.md`의 사전 고정 logical timing model을 따른다. 이 score는 Architecture mockup/reference evidence이고 production latency가 아니다.

`factorial_analysis.py`는 16개 complete configuration의 W-01~W-04/W-06~W-12 결과를 결합하고, B-minus-A main effect, pairwise difference-of-differences, DP별 8개 matched-context contrast를 산출한다. W-05 누락은 계속 `BLOCKED_OPENROUTER_KEY`이며 weighted total/global winner를 만들지 않는다.

```bash
cargo +1.98.1 build --locked --manifest-path prototype/gate2/Cargo.toml \
  -p gate2-bench -p gate2-host -p gate2-worker
.venv/bin/python benchmark/rebaseline/gate2/representative_runner.py \
  --manifest results/rebaseline/<freeze>/source-manifest.json \
  --approval results/rebaseline/<freeze>/measurement-approval.json \
  --bin-dir prototype/gate2/target/debug \
  --output results/rebaseline/<freeze>/representative \
  --suite all
```

W-09는 frozen 500ms whole/shared restart controller와 100회/stratum을 실행하고 4개 whole-process + 2개 integration-fatal p95의 평균을 configuration별로 산출한다. W-10은 frozen 30초 fault, 2/10/20초 probe, mode별 5회와 fatal 5회를 실행한다. W-10 wall time을 줄이되 cell 간 간섭을 피하기 위해 같은 cell의 독립 10 trials만 병렬 실행하며 이 parallelism도 freeze fingerprint에 포함한다. 두 결과는 deterministic dependency/Agent fixture를 사용한 `MEASURED_STRUCTURAL`이며 Windows absolute performance가 아니다.

`catalog_check.py` 산출물에는 16개 complete configuration, 8개 pair-labelled change-analysis 후보, W-01~W-12의 미측정 pair ledger 96개 row와 24개 change용 192개 raw template row가 들어간다. null은 0점/0개가 아니라 NOT_RUN/NOT_ANALYZED다. `source_revision`은 실행 checkout의 실제 Git HEAD를 기록하고, `catalog_fingerprint`는 C/I/S/D 요소 정의와 Core alternative metadata를 해시한다. `change_analysis.py`는 별도의 reviewed 24-change rule을 이 동일 fingerprint에 적용해 192개 M/A/R raw row를 `DESIGN_ANALYSIS`로 확장하지만 Measurement Freeze 전에는 W-07/W-08 평균·score·winner를 계산하지 않는다.

이 도구는 설계 ID·조합·전수 ledger의 정합성만 검사한다. W-10 CI smoke는 candidate endpoint wiring과 fault-containment controller correctness만 검증하며 대표 metric으로 승격하지 않는다. W-11/W-12의 checked-in oracle과 evaluator도 평가 계약의 분모·집계·guard를 고정하고 검증할 뿐이며, reviewed complete candidate trace가 없으면 대표 metric은 계속 NOT_RUN이다. VIA 전체 구현, Model accuracy, p95, Architecture winner를 증명하지 않는다. 사람용 문서에서는 W-ID만 쓰지 않고 전체 ASR 명칭/설명을 병기한다.


Final Working-12 metric→0~5 변환의 machine source of truth는 `benchmark/rebaseline/working12/baseline.json`과 `working12/score.py`다. 기존 `scoring-baseline.json`/7-ASR helper는 legacy compatibility용이며 Working-12 final scoring에 사용하지 않는다. 실제 semantic reference run은 OpenRouter hosted Qwen3-8B profile을 사용하되 evidence는 `MEASURED_MODEL_REFERENCE`로 표시하고 local/target-PC absolute latency라고 표현하지 않는다.
