# Common9 QA04 independent ledger review

2026-10-19. Final v2 review: **no unresolved P1/P2 found**. The reviewer performed no candidate edit, compiler execution, helper inference or model API call. This review follows the selected final verification pointers; the earlier preliminary count review is preserved separately.

## Verified result

All seven configurations independently apply the same six changes to the exact frozen baseline `38b92206e6ecfc10eedb839c97f18ddec4c898e850f5c0db5f55d721178501b1`. All42 accepted changes have count vector **[1, 1, 3, 2, 1, 2]**, mean **10/6 = 1.667 operational Rust modules**.

| Change | Independently checked operational module set | Count |
| --- | --- | ---: |
| C01 model wire format | `crate::model_wire` | 1 |
| C02 Agent wire format | `crate::agent_wire` | 1 |
| C03 source/frame input format | `crate`, `crate::context`, `crate::input_format` | 3 |
| C04 partial delivered-result revision | `crate::runtime`, `crate::sources` | 2 |
| C05 delivered provenance query | `crate::runtime` | 1 |
| C06 arbitrary source-line withdrawal | `crate::runtime`, `crate::sources` | 2 |

The module mapping was checked against the crate's actual declarations. C03's new module registration changes operational `main.rs`, so the crate root is counted. Inline test modules, comments and whitespace do not create additional operational modules. This is an explicit mapping for this crate, not a general equivalence between files and modules.

## Evidence checks

The final independent result and per-change references are in [qa04-v2-independent-final-review.json](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/evaluator-offline/qa04-v2-independent-final-review.json). The reproducible read-only review script is [qa04-v2-independent-review.py](../../../../../../results/architecture-evaluation/current/common9-poc-20261019/evaluator-offline/qa04-v2-independent-review.py).

- Verified the baseline freeze digest, all42 candidate source manifests and tool hashes, actual binary hashes against retained trajectories, and selected verification/status/acceptance linkage.
- Compared actual Rust changes with the six sets above. Final patches match each attempt's patch; each task has exactly the same patch across all seven configurations. The changed shared adapters/source/publication paths are live in each configuration; no dormant alternative or unrelated leaf edits were counted.
- Inspected actual task changes and finite acceptance conditions: migrated real wire adapters; normalized both initial and later source/frame ingress; revised the actual delivered section and registered source; queried actual delivered provenance without mutation; blocked withdrawn ranges while preserving permitted disjoint raw fragments.
- Verified successful actual `cargo test`, `cargo build` and task-driver exit records in every selected verification. Re-evaluated all378 retained common9 trajectories with the frozen evaluator: **1,806 substantive callbacks, 966 final goals, 630 exact referent sets**, all PASS. Every accepted change has its own9-case regression and finite task checks. Actual provider calls are0.
- Excluded all v1 `attempt-01` preparations. Preserved AAAA/C01 v2 `verify-01` sandbox `ps` failure; selected `verify-02` passes with the same candidate and source hashes. The failure is not a functional or memory success.

The preliminary `qa04-v2-independent-count-precheck.json` used an incomplete status lookup and deliberately left acceptance pending. It is retained as a count precheck; this final review reads actual `latest-attempt.json` and selected verification directories.

## Interpretation limits

These six changes exercise common adapters, source authority and publication functionality. They do not exercise alternative-specific producer or semantic-execution propagation sufficiently to establish a general modifiability ranking. The observed ties are valid for this change population.

The378 regressions use deterministic semantic/vision doubles and scaled offline timing. They establish scoped implementation acceptance, not semantic-model accuracy, comparative latency, QA05 model cost or full-parser memory performance. Pending audio, production Windows integration and the remainder of COMMON28 are outside this acceptance scope. No benchmark/candidate source was changed by the review.
