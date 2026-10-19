# Final input assembly source v1: bounded static review

2026-10-19. Independent reviewer: GPT-6 Astra Ultra, `/root/quant_adapter_astra`.

**CHANGES_REQUIRED_SOURCE_REVIEW_ONLY: one P2 execution-provenance finding, IA1-01.** The result-fragment and accepted-adapter integration is otherwise consistent within this bounded review. This review accepts no actual analysis, corpus ingestion, quantitative result or reporting execution and grants no execution GO.

## Exact reviewed artifacts

| Artifact | SHA256 |
| --- | --- |
| Source-v1 checkpoint | `d1b2113519ec20eed733b3acbf708b8a8d5931ee5b9137e49900e6ed321cb178` |
| Copied recorder source | `a19cfafccee6ebef51121e4bc4c1e6d7a777f25632a55f03239ecd217d4b739c` |
| Copied synthetic checks | `155146b219249419f5c6a6d78506ff792a3c11492ff1d3407da1712bd34c1bc9` |
| Copied README | `383341b160df7562d81fee480a130e46aa4535e33a652b24ee6d54fa30f2f432` |
| Saved split batch | `cf3e882227008fae0347232ee333e4aa75885e6bd3ff38ccdbbfb213458b17cc` |
| Saved failure batch | `8152da966cafcfdae302f8dfd7c95b309a6d202baec80a4bec55fc30a5700031` |
| Creation diff | `491e05a1518bccd924cb0f30233ed7687969d7a941a42066813364dc03a63235` |
| Accepted stage03 adapter | `c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0` |
| Stage03 closure and five-step integration plan | `59c797d8bb6b73bcdf9873c171335d16c5bf74b6caf1cc7b703a71ee2b8ab3f4` |
| Governing quantitative specification | `b4bb8735e67ec84dbef1cb193ac3a36f6ea70411094515a7f5d616b0ee82a09e` |

The first nine listed artifact hashes were checked. The specification is the governing version retained from the adapter review. The entire copied recorder and synthetic-check source, README and saved test records were read. Additional bounded reads covered accepted adapter loading/association functions and named frozen analyzer/import entry points. No actual trace, runs corpus, live ledger or current measurement result was read.

No test, analyzer, adapter, evaluator, model, parser, build, network request, process-control action or directory/corpus scan ran. The only new output from this review is this Markdown file. The creation diff was hashed; the copied full source, rather than a repeated full diff read, was the implementation review basis.

## IA1-01 / P2: constrain the recorded child interpreter's imports

**Location:** copied `record_joint_report_inputs.py:172–185`, especially the unqualified child interpreter command at line180 and inherited subprocess environment at line175.

The recorder verifies all112 frozen source files, but its child command is simply the current Python executable followed by the analyzer path. It inherits Python environment and site initialization. In the actual frozen ABI, supplement `evaluate.py:13–15` appends the frozen COMMON directory to `sys.path` and then imports `evaluation_v3`. An existing `PYTHONPATH` entry or site customization can therefore provide that package before the verified frozen copy. The subsequent receipt and result associations still claim the frozen112 dependency mapping. Hashing the intended files before and after execution does not establish that this import used them.

The README's outer `python3 -B` also does not propagate to a newly launched interpreter. The current child may create or update `__pycache__` beneath the frozen source tree. This is avoidable in the recorder and does not require changing the frozen analyzer.

**Bounded repair:** explicitly constrain Python startup/import configuration in the recorded child command while retaining the analyzer's script-directory imports and unchanged `--root`, `--output` and no-`--offline` semantics. For this standard-library/frozen-module path, `-E -S -B` is a suitable narrow direction: ignore Python environment configuration, skip site initialization and suppress bytecode writes. Confirm the named frozen import path statically and record the selected policy/actual argv. Do not use `-I` without handling its removal of the script directory, because the frozen CLI uses sibling imports. No environment contents or credentials need to be dumped. Preserve failure evidence as already implemented.

The existing fake-runner command assertion should include the exact chosen child flags and policy. A real analyzer run is not needed to repair or source-review this finding. This finding concerns actual import resolution and cache-write behavior; it does not claim that the current environment has already substituted a package or that any result has been affected. No actual recorder execution has occurred.

## Integration judged consistent in this scope

- **Quiescence and lifecycle:** the recorder consumes independent STOPPED/COMPLETED evidence and bound immutable progress/status/ledger snapshots. It does not discover or stop processes, settle calls, infer execution from result absence, or grant episode certification. All441 ordered jobs remain in the output; the accepted adapter retains responsibility for exact STARTED→DISPATCHED→TERMINATED reconciliation and missing/unexecuted/conflicting partitions.
- **Frozen execution ABI:** the script and source SHA, exact formal root, new output, FORMAL strict mode and no-offline operation match the frozen analyzer. The helper has one analysis subprocess path and no episode/model/parser/build or report-CLI path. Its receipt distinguishes fake execution, actual attempted invocation, integer exit result and an exception with unknown process creation.
- **Result associations:** separate315/126 populations, planned identity, integer repetition, FORMAL phase, strict evaluator version, unique run ID and exact trace-provenance path/hash are reconciled before publication. Each result remains an unchanged data-object copy linked to the original output hash and array selector, its preserved trace, population oracle, source112 map, freeze/binary and execution receipt. Primary results may omit population under the supported frozen ABI; supplementary results may state it. Neither is guessed from topology.
- **Failure preservation:** nonzero exit, runner exception, absent/oversized output, malformed successful JSON, duplicate/conflicting identity and changed source/input are retained with failure records and available original/log/receipt artifacts. They do not publish adapter-input. Ordinary functional failures and NOT_CERTIFIED results survive valid assembly without numerical or semantic filtering.
- **Fragmentation and reporting:** the large joint output is fully read once. Results, per-configuration summaries, supplement activation objects and the two financial field values have explicit original selectors. The existing adapter input ABI is emitted without invoking that adapter. Raw pairs are absent because the frozen joint output has none; no replacement pair or semantic judgment is generated. The five accepted QA04 role bindings and the two frozen oracles use accepted validation helpers.
- **Immutability and scope:** output creation is exclusive under the exact formal postanalysis root, output payloads are hash-bound and the manifest is written last. The source's accepted-adapter hash pin and before/after input checks preserve the intended source generation, subject to IA1-01's import-resolution gap. Accepted adapter, frozen sources, QA04 and semantic extractors were not edited by this review.

## Saved checks and remaining runtime boundary

The saved records report six passing temporary fake-runner checks in two batches: split2 and failures4, maximum0.12163529195822775 seconds. They bind the exact recorder/test/accepted-adapter hashes. They exercise unchanged object/selector copying, both populations, missing jobs, retained failures, adapter loader compatibility, duplicate/phase/job/trace rejection, source mismatch, exclusive output, failed invocation, partial/oversized output and invocation exception. They were read and hashed, not rerun. They do not exercise a real Python child or its import environment; IA1-01 is consequently outside their present evidence.

After the narrow source repair and closure, the remaining inputs are already identified by the plan: independently established quiescence; exact immutable snapshot references; the final runs hash/size; approved QA04/oracle bindings; and a separate root GO. Actual per-trace sizes and the accepted adapter's cumulative read budget remain to be checked at that authorized stage. The assembler streams traces under64MiB while the accepted adapter defaults to16MiB per file; root must choose explicit existing limits appropriate to the preserved files, and unsupported sizes must remain visible failures. No present corpus-size assumption or automatic cap increase is accepted here.

The unchanged frozen analyzer itself still owns its full-corpus parsing and memory behavior. The recorder's bounded hashing and single-read output packaging do not establish its actual runtime memory use. Actual output provenance, budget reconciliation and final-report acceptance remain future evidence. No additional audit campaign, frozen-source change, metric redesign or measurement rerun is requested by this review.
