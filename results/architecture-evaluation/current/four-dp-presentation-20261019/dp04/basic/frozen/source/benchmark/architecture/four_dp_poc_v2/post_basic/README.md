# Corrected PoC post-BASIC execution

These are the canonical runners/adapters for the separately authorized post-BASIC phases. Existing BASIC files and binary are unchanged. Preparation copies under the campaign's `analysis/` directory are provenance only.

- `tactics_control.py`: refuses preparation/compile/run until the exact120 BASIC trials have terminal records. A4 and B4 each use the same four cases×three repetitions and actual variant A4/B4. A2 uses the exact BASIC crate with only the approved `dp01.rs` patch, a separately compiled binary and E3×three repetitions. A4 is TACTIC; B4 is a parallelism DIAGNOSTIC; A2 is TACTIC.
- `line_number_diagnostic.py`: refuses preparation/run until BASIC120 and all27 preceding comparison trials are recorded. Runs DP05 E4×ABC×three repetitions with the same BASIC Rust binary/host/source/guards/oracles/model/caps/recovery. The model-input copy gets compact original line numbers plus one shared explanation. Every original payload and transmitted model request is retained with hashes.
- `compact_source_lines.py`: deep-copy transform only for actual contiguous original source passages. It does not number delivered summaries/statements or mutate host evidence. Invalid line counts/ranges are errors, never padded/truncated. Quotes must exclude display prefixes.
- `check_compact_source_lines.py`: pure checks and read-only replay of all32 original E4r1 requests; no provider calls.

All actual trials use fresh Python/Rust processes and the same campaign-root atomic $4 ledger. Each new phase snapshots and hashes these canonical Python sources. Its phase authorization, exact jobs, candidate binary and effective source/harness are immutable. Outcomes—including errors, wrong targets, failed quote contracts and missing completion—remain separate from BASIC and do not replace it.

Run the canonical scripts from the repository root. `tactics_control.py` takes `prepare|run dp04-A4|dp04-B4|dp01-A2`; `line_number_diagnostic.py` takes `prepare|run --root <absolute corrected campaign path>`. Provider execution is permitted only by the exact per-phase authorization and global budget. No additional tuning or quiet performance reruns are authorized.
