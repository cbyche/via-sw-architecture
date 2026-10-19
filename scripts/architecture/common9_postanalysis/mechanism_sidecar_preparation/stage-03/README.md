# DP1/DP4 mechanism sidecar preparation, stage 03

Status: **MS2_01_REPAIR_PREPARED_FOR_INDEPENDENT_SOL_HIGH_REVIEW**.

This new stage addresses only residual P2 MS2-01 from the complete preserved review
`../stage-02/static-sol-review-stage02.md`, SHA256
`dda8f4db1c7dc930ca18424bdf707c54005ce42f8072d831bd85f93969619480`.
Stage 02 and earlier artifacts remain unchanged. GPT-6.1 Sol High performed this
source-only repair and the bounded synthetic checks. Independent closure is pending.

Malformed event/data shape now uses the common interruption helper for every
collection. That helper records `coverage_issue: true`, `status: UNSUPPORTED` and
`processing_state: INTERRUPTED`. Collection finalization preserves an interrupted
state instead of resetting it to complete. Existing projected facts and retained
issues are preserved. The CLI, input/report ABI, fixed caps and all other accepted
tools are unchanged.

One short synthetic process executed **3 unique cases**, all passing with no errors
in **0.009629292180761695s**. Named fake files total **1,750 bytes**. The command was
`python3 -B scripts/architecture/common9_postanalysis/mechanism_sidecar_preparation/stage-03/test_fake.py attempt-01`.

The first two cases use terminal-shaped traces containing a valid source-read fact
followed by either a non-object event or a source-read event with `data: null`.
Each case checks every collection's final status, coverage flag, processing state,
row status, retained issue pointer and preservation of the earlier fact. The third
case writes all 441 synthetic planned rows to a bounded memory sink and checks both
malformed runs again. It produces 637,560 virtual bytes and no actual extraction file.
These are three preparation checks, not campaign trials or measurement evidence.

The accepted plan remains SHA256
`071f5872010214aca45f168c585ac1e323f7c303187693f69c45d8e0796e85ea`.
Actual extraction still requires independently established controller quiescence
and a separate exact root GO. No campaign trace, result or model payload was read;
no actual sidecar CLI, evaluator, parser, model, build, API or controller operation
was executed. No frozen source, adapter, assembler, contract, measurement path,
presentation, prior preparation file or Git staging/commit/push was changed.

The copied source, test source, saved result and checkpoint bind this stage. Synthetic
success does not establish actual-corpus coverage, live quiescence, semantic use,
causal savings, a new metric, eligibility or a winner.
