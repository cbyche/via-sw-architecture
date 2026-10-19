# Final input assembly source v2: IA1-01 static closure

2026-10-19. Independent reviewer: GPT-6 Astra Ultra, `/root/quant_adapter_astra`.

**PASS_SOURCE_REPAIR_REVIEW_ONLY. IA1-01 is closed for the exact source below. No new blocking finding in this narrow repair review.** This is source acceptance, not actual analyzer/import execution evidence, quantitative-result acceptance or execution authorization.

## Exact scope

| Artifact | SHA256 |
| --- | --- |
| Source-v2 checkpoint | `5c1a4a22165bd803d837ce55240f744a3106aa643889401798912723dfd11322` |
| Copied recorder source | `6c97d90f3d736497ee9bd29ed219a28a9edd1eb7b5ba1d859701175aea0266a0` |
| Copied synthetic checks | `2f10fbe9cf02d7fdac1701de15cc76482a62d469f4bb8e8f6ac3303430dc080b` |
| Copied README | `ed6b68d97ef92e94bdcc9aac41f1027401d16bc5c1d0c0111ddb70181f8d3d30` |
| Preserved v1-to-v2 diff | `16b4bbdf1aa77d376bdec46507e7d24084f0a152dbc2f5ad4702e997f0fb1339` |
| Saved split batch | `83ded07f069f136ac430e6d88c4f3b1dd94051e8b2bab0c282a161b4311b416e` |
| Saved failure batch | `f88e1ecabeebaf8829140faaa746019748d36ac090f49d8bf654a21f16a75eea` |
| Preserved source-v1 review | `76ed0f70e8c3a6147d57f9f2e15149d5477877aeb9cccb7103d736d03360bba2` |
| Unchanged accepted stage03 adapter | `c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0` |

All listed hashes were checked. Review read the recorded diff, independently compared the two copied recorder sources, read changed test/README text and both saved test records, and inspected the remaining named frozen relative-import declarations. The broader assembly review remains source-v1's preserved review; this closure covers its sole requested repair and relevant integration.

## Repair accepted

The child command now explicitly contains `-E -S -B` before the exact frozen analyzer script. The actual argv and `child_startup_policy` are recorded in the execution receipt. The policy distinguishes ignored Python environment configuration, skipped site initialization, disabled bytecode writes and retained script-directory behavior. It records no environment values. The outer interpreter's flags are no longer relied upon to configure the child.

The flags address the reported defect: ambient `PYTHONPATH`/Python environment configuration and site initialization no longer take precedence over the frozen COMMON package, and the child suppresses bytecode writes. Script mode is retained; `-I`, which would remove the needed script-directory import path, is absent. These flags are interpreter arguments, so the frozen analyzer still receives its unchanged `--root` and `--output` arguments without `--offline`.

The static import chain is compatible with this startup policy. `analyze_joint.py` imports its sibling supplement evaluator, controller helpers and mechanism module. Supplement evaluation appends its frozen COMMON path, imports `evaluation_v3`, and that package uses relative endpoints/deadlines/publications imports. Its wrapper inserts the frozen COMMON path and loads the exact preserved legacy evaluator file; the legacy evaluator imports frozen `causal_timing`, which uses standard-library `dataclasses`. The named chain uses frozen modules and the standard library, with no dependency on site initialization or ambient `PYTHONPATH`. Source-v1's bounded import reads were retained; this closure additionally inspected the relative declarations in `evaluation_v3/__init__.py`, `endpoints.py`, `deadlines.py` and `publications.py`. No frozen module was imported or executed.

The actual recorder-source delta is limited to the child-flags/policy constants, their insertion in argv and policy recording. Test changes assert the literal command flags and receipt fields. README changes document the same policy and source-only limits. Frozen source hashes/paths, analyzer CLI semantics, quiescence and snapshot checks, result/trace/oracle associations, fragmentation, lifecycle handling, financial values, failure preservation and exclusive final-manifest behavior are unchanged from the prior reviewed source.

## Evidence limits and handoff

The two saved fake-runner batches report six passing checks: split2 and failures4, maximum0.1250774587970227 seconds. The records bind the accepted source/test hashes above. The updated checks assert `['-E','-S','-B']` in both the fake command and receipt, the individual startup settings and absence of recorded environment values. The records were read and hashed, not rerun. They prove their synthetic assertions only; they do not establish real child imports or actual campaign analysis.

No actual corpus, trace, current run result or live ledger was read. No test, analyzer, adapter, evaluator, model, parser, compiler, network request or process-control action ran. The only new file from this review is this closure. Source-v1's finding/review remains preserved; the accepted adapter and frozen sources were not edited.

The implementation is ready for the already defined later integration stage once root separately establishes quiescence and supplies exact immutable snapshots, final runs hash/size, QA04/oracle bindings and execution GO. Actual file/cumulative read limits, recorded runtime provenance and quantitative/report acceptance remain future evidence. No further source redesign or audit campaign is required by IA1-01 closure.
