# Frozen joint analysis input recorder v1

`record_joint_report_inputs.py` is a separate reporting glue tool. The accepted report adapter, extractors, frozen experiment and QA04 remain unchanged. **Implementation/source review is not execution authorization.** Actual analyzer/assembly/corpus reads require established quiescence and a separate root GO. Only fake-runner temporary synthetic tests were executed during preparation.

The tool implements the five-step plan in adapter stage03's independent closure (`59c797d8bb6b73bcdf9873c171335d16c5bf74b6caf1cc7b703a71ee2b8ab3f4`). It does not implement another evaluator, compute new QA values, launch an episode/model/parser/compiler, control processes, settle calls, infer semantics, or invoke the report adapter's CLI. It imports only the accepted adapter's pure helpers and verifies its exact source SHA `c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0`.

## Root-provided plan

An immutable, hash-bound `JOINT441_REPORT_INPUT_ASSEMBLY_PLAN_v1` object must be supplied inside the exact formal root. Concrete quiescence/snapshot paths are deliberately not guessed or created by this tool.

Required fields:

- `version`, `formal_root`.
- `artifacts.freeze`: exact `<root>/freeze.json`, hash0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd.
- `artifacts.approval`: exact `<root>/approval/APPROVAL-BINDING.json`, hashd3f78606f7a8bad161ee58a0c0fb5e7f7bf83e287140dbc9a4bbe2c028ce93bd.
- `artifacts.quiescence`: independently established `JOINT441_QUIESCENT_SNAPSHOT_v1`. STOPPED/COMPLETED, no_active_episode_processes=true, boolean progress_complete, evidence_refs and exact progress/status/ledger snapshot hashes. This is consumed evidence, not a PID/time inference.
- `artifacts.progress`, `artifacts.status`, `artifacts.ledger`: immutable refs with `snapshot_of` the exact controller files/freeze.ledger_path; their paths differ from the originals. UNKNOWN/RESERVED and incomplete termination remain unchanged.
- `artifacts.runs`: exact `<root>/runs.jsonl` path/hash/bytes that the frozen analyzer actually reads after quiescence. No alternate corpus or snapshot is substituted for its fixed CLI input.
- `oracles.COMMON9` and `oracles.COMMON_SUPPLEMENT6`: exact frozen source refs matching freeze112. Primary and supplement remain distinct.
- `qa04`: five existing accepted-adapter `{approval_role,ref}` bindings: report, classification, final_review, raw630, checkpoint. Exact role/path/SHA/bytes and final acceptance links are checked by the accepted pure helper.

Every ref has an absolute regular-file path, lowercase SHA256 and exact bytes. Symlink ancestors, traversal, mixed roots and silent restore relocation are rejected. Source paths must equal `<root>/frozen/source/<relativePath>`; all112 entries are pinned before and after analysis. No directories or arbitrary model files are discovered.

Optional `artifacts.global_call_snapshot` is an immutable snapshot with snapshot_of=`<root>/model-calls.jsonl`, never the live log. Optional `semantic_review_index` and `registrations` are linked as supplied bound artifacts under the existing adapter ABI. No semantic/mechanism/ranking judgment is invented; the later adapter validates those optional nested joins. `raw_pairs` is deliberately absent because frozen joint analysis supplies none.

Optional `analysis_max_bytes` defaults to128MiB, with hard cap512MiB. Runs are streaming-hashed (default4GiB, root-supplied runs_max_bytes up to8GiB hard; no runs JSON parse by this helper). Other adapter inputs/fragments use the existing adapter limits (default16MiB/file and8GiB aggregate); an optional explicit adapter_read_limits is passed unchanged to the accepted adapter, within its existing64MiB/16GiB hard limits. No cap is automatically raised. Logs have a64MiB hash cap. Fragment sizes must fit the selected existing adapter file limit before adapter-input publication. Oversize artifacts fail with evidence retained, never silently truncating or raising limits.

## Recorded invocation

After separate actual execution GO, the CLI requires `--execute-authorized-frozen-analysis`. It constructs one fixed command:

```
<current Python executable> -E -S -B <root>/frozen/source/benchmark/architecture/common_supplement6_20261019/analyze_joint.py --root <root> --output <new-assembly-root>/joint-analysis.json
```

The pinned analyzer SHA is `469a5c8a57ee1a3e6812b8c85182cc0e4b2fbd59b69fbca8eeb8591a0fcecb49`. The source accepts these exact flags; no `--offline`, arbitrary command, extra program or substituted evaluator is exposed. No shell is used. The child explicitly uses -E to ignore Python environment configuration, -S to skip site initialization, and -B to suppress bytecode writes. Standard script mode retains the analyzer directory; -I is not used. Its frozen sibling imports and the supplemental evaluator's explicit frozen COMMON path remain available. No environment values are recorded or dumped. The unchanged frozen analyzer is responsible for its original strict metrics and actual raw-trace identity/content checks.

The receipt records actual argv/cwd, source112/oracle/freeze/binary/runs bindings, Python version, UTC/monotonic begin/end, exit/exception and stdout/stderr/output refs. An invocation attempt with an exception does not assert a process was created. Failure and partial output remain preserved; no retry occurs. Logs and oversized outputs that cannot be hashed within bounds are explicitly identified as unavailable/oversized, not certified hashes. No adapter-input manifest is emitted after a failed invocation/provenance check.

## Fragment assembly

The original joint JSON is fully read exactly once under the declared cap, hashed and parsed once. No per-result pointer rereads of that large file occur. Original output stays unchanged. Each result/summary/mechanism data object is serialized unchanged into a small fragment, with exact original file hash and JSON selector; a mapping explicitly distinguishes object copies and the two financial field selectors.

Observed result identities must be planned, FORMAL, strict COMMON9-EVALUATOR-v3, unique in their chosen population and unique by run ID. Each joins its exact recorded trace_provenance path/hash; preserved traces are streamed only for hashing, not parsed for new analysis. The actual pinned analyzer operation establishes the result-to-trace derivation; the later accepted adapter independently checks raw trace identity. Both ordinary failures and NOT_CERTIFIED results are retained, without filters. Duplicate/unplanned/mixed generation or conflicting joins reject assembly rather than choose a preferred record.

Per-result `JOINT441_RESULT_ASSOCIATION_v1` files bind the actual execution receipt, original result selector, unchanged fragment, trace SHA/bytes, chosen population oracle, source112, binary/freeze and actual recorded command. This records a newly performed analysis operation; it never upgrades historical identity-only light results by hashing them later.

All441 frozen ordered jobs remain in `planned-identities.jsonl`. Only actually observed results enter `result-index.jsonl`; absent results are not fabricated. The analyzer's original unexecuted labels are preserved separately and explicitly mean result absence, not proof of never-started execution. Quiescent progress still determines reporting lifecycle.

Existing per-configuration summary objects, supplemental activation objects and two top-level financial totals are copied by selector without recomputing metrics or inferring activation. The tool emits the accepted `JOINT441_QUANTITATIVE_INPUT_v1` manifest and stops. A later root-directed adapter invocation writes a different immutable report directory. These are separate acceptance stages.

## Outputs and tests

The output must be a new directory under `<root>/postanalysis/`. Existing output is immutable. The final hash manifest is written last; it includes fragment/association/index/log/original refs and the plan/tool identities. On failure, the immutable failure record and any completed original/receipt/log artifacts remain. Output creation is not final quantitative acceptance.

Tests inject a clearly marked SYNTHETIC_FAKE_NO_PROCESS runner that writes temporary fake joint JSON. There is no CLI fake-runner option. The synthetic source inventories/oracles/QA04 hashes are patched only inside those temporary tests; no accepted file is changed. Saved records bind source hashes, elapsed duration and their no-process/no-corpus scope. Split equality, source selectors, both populations, failures/missing jobs, duplicate/mismatched identity rejection, immutable output, oversize and failed analysis evidence are covered. Actual execution remains0 until separately authorized.

Future authorized use, with concrete paths supplied later by root:

```
python3 -B scripts/architecture/common9_postanalysis/record_joint_report_inputs.py --plan /EXACT/ROOT/postanalysis/assembly-plan.json --plan-sha256 SHA256 --output /EXACT/ROOT/postanalysis/NEW-INPUT-ASSEMBLY --execute-authorized-frozen-analysis
```


## Source-v2 IA1-01 repair

The preserved source-v1 review (`76ed0f70e8c3a6147d57f9f2e15149d5477877aeb9cccb7103d736d03360bba2`) required the child interpreter policy above. Source-v2 changes only the wrapper command, its recorded startup-policy metadata, the exact fake-runner assertion and this documentation. The frozen analyzer and accepted adapter are unchanged. The receipt now carries child_startup_policy with actual flags, the three individual Python startup/cache settings, retained script-directory policy and environment_values_recorded=false. The outer Python's -B is no longer relied upon to propagate to a child.

Bounded static confirmation followed the frozen analyzer's sibling evaluate/joint_campaign/mechanism imports, supplement evaluate's appended frozen COMMON, evaluation_v3's relative endpoints/deadlines/publications and explicit legacy file load, and legacy causal_timing's standard-library dataclasses. Those named imports require standard-library and frozen modules, not site initialization or ambient PYTHONPATH. No imported frozen module or child interpreter was executed for this confirmation. Tiny synthetic tests assert exact -E/-S/-B argv and receipt metadata; they remain fake-runner tests, not runtime import validation or actual analysis acceptance.
