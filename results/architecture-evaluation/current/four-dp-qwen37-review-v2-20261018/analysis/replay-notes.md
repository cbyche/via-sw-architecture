> **최종 현재 진입점 (2026-10-19)**: 아래의 `results/analysis/build_replay.py` 명령과 “Final browser review remains pending” 문구는 이전 준비 이력이다. 현재 생성기 정본은 [benchmark reports/build_replay.py](../../../../../benchmark/architecture/four_dp_poc_v2/reports/build_replay.py), 재현 절차는 [FINAL-REPRODUCTION](../../../../../benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md)이다. 최종229 감사·UI 확인은 [final229-review](final-review/final229-review.md)와 [실제 재생](../demo.html)을 읽는다. 로컬 주소는 `http://127.0.0.1:8770/demo.html` (127.0.0.1 전용, 서버 실행 중)에 연결된다. 아래 원래 기록은 보존한다.

# Standalone evidence replay handoff

Owned files: `analysis/build_replay.py`, `analysis/replay_template.html`, `analysis/check_replay_static.js`. No BASE/CRATE/frozen source changed. No candidate or provider runs. Default export includes only actual BASIC/TACTIC/DIAGNOSTIC records found under phase freezes; final all-phase export uses explicit `--include-calibration`. Calibration is labeled separately, and initial run selection plus the visible summary table default to BASIC when present. No separate access smoke exists in this new campaign; all calls from26 calibration episodes remain in the all-phase records/costs.

Final export command (runtime runs after the campaign finishes):

```sh
python3 results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/analysis/build_replay.py results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018 --include-calibration
```

Output `demo.html` plus `demo.build.json`. Main visible inputs, original/overlay PNG bytes, substantive actual events, all actual model call records and provider response JSON, callback/detail/failure/correction/source/recovery timeline, existing derived metrics/coverage, same-sample RSS peak and logical category summaries are embedded. No external scripts/network needed. PNG content is SHA-deduplicated and read from phase-frozen source where available, checked against actual input hash. Existing summaries are displayed by phase; the builder does not invoke scorers or create performance results.

Full 50ms RSS sample arrays and repetitive logical-buffer lifecycle events are deliberately not embedded. Their actual original trace remains linked with SHA; actual RSS values are also written to `analysis/memory-samples/<run-id>.csv`. The embedded projection labels its exact omissions. Standalone UI operation needs only the HTML; full raw/sample links additionally require the companion result directory. Actual provider response JSON and actual input JSON remain expandable inside the HTML. All user/source/model strings use DOM textContent rather than executing HTML.

`summary.errors` is labeled **계약/완료 실패 episode**. Process exit, actual host contract/completion failure, first referent accuracy and finite content conditions are separately displayed. Instrumented GOAL_COMPLETE is not asserted to prove the finite content conditions or domain entailment. QA06 is an actual Python/Rust RSS sample estimate excluding browser/cloud weights; category logical peaks include retained harness evidence and are not summed into RSS or into one another.

Bounded preview evidence:

- `replay-preview-basic.html`: first 2 actual BASIC rows, all11 provider responses, ~333KB with actual model request text (previous full-sample embedding without it was ~2.35MB).
- `replay-preview-screen.html`: the 2 explicitly labeled calibration-v3 screen rows, 2 actual PNGs and all4 provider responses, ~280KB.
- `replay-preview-checks.json`: lightweight static DOM harness PASS for phase/variant selection, raw response expansion, actual failure visibility and PNG data URI wiring. **This is not a rendered browser/visual usability check.** Final browser review remains pending.

Reproduction of bounded static checks (no models):

```sh
python3 results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/analysis/build_replay.py --phase basic-v1 --limit 2 --output results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/analysis/replay-preview-basic.html
python3 results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/analysis/build_replay.py --phase calibration-v3 --include-calibration --limit 2 --output results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/analysis/replay-preview-screen.html
```

The initial static harness incorrectly expected a screen-specific failure reason in a DP01 failure; it was corrected to assert the actual generic GOAL_FAILED event. No candidate, raw event or model response was changed. The exported build metadata records run counts, source JSONL SHA values, missing-file warnings and the HTML SHA. Empty metrics are shown as analysis pending, never as zero/success.

## Actual model-request context and all-phase preview extension

Each actual call now embeds its complete original system/user text and request settings as well as the raw provider response. Only image_url data bytes are replaced by explicit EMBEDDED_ASSET_SHA256 references to the same once-embedded actual pixels. Original request JSON path and SHA256 are retained. The source request body equals the embedded projection after precisely that documented replacement; no server/fetch is required to read model context or view images.

`replay-request-self-contained-checks.json`: bounded8 actual records (2/phase),32 original request bodies equal,32 provider responses,6 image references verified, no RSS arrays embedded. `replay-preview-all-phases.html` is about1.09MB. Static DOM checks verify initial BASIC selection and phase filtering plus raw prompts/responses and actual failure visibility. This remains a static harness rather than browser rendering evidence. Final command uses --include-calibration; the BASIC-first filter does not discard other phases or their costs. --limit-per-phase2 is a preview-only option and must be omitted from the final export.
