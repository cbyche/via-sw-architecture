# DP1/DP4 mechanism sidecar preparation, stage 01

Status: **PREPARED_FOR_INDEPENDENT_STATIC_REVIEW; REAL EXECUTION NOT AUTHORIZED**.

The standalone source is `../../extract_dp1_dp4_mechanism_evidence.py`. This stage
holds an exact source copy, the synthetic test source, immutable results for three
preparation attempts, named fake inputs and a provenance checkpoint. The accepted
adapter, assembler, frozen candidate/harness/contracts and actual campaign evidence
were not changed or executed. No Git stage, commit or push was performed.

The controlling plan is
`results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/planning/mechanism-evidence-readiness-v1.md`,
SHA256 `071f5872010214aca45f168c585ac1e323f7c303187693f69c45d8e0796e85ea`.

## Interface and scope

The prospective CLI takes `--input-manifest`, `--input-sha256`,
`--accepted-report-manifest`, `--accepted-report-sha256` and a fresh `--output`.
It consumes the accepted quantitative-input ABI and freeze's exact441 jobs,
the pinned result index and one explicitly referenced trace at a time. It validates
the accepted report manifest's exact `calls.jsonl` role, size and hash, without
reading or recalculating the call ledger. The linkage is explicitly reference
only. No code from the candidate, evaluator, parser, adapter or assembler is imported.

Rows retain all 315 COMMON9 and 126 COMMON_SUPPLEMENT6 identities. Generation joins
require a unique producer-backed committed payload. Configuration letters never
establish source ownership. Built capsules, actual ingress and dispatched receiver
request exposure have distinct phases; same record ID alone cannot join generations.
Request text supports both recorded bridge envelopes and uses separate RFC6901
outer-message and decoded-input pointers. Prose is represented by hashes/references;
no images/base64 or response bodies are decoded or interpreted.

The prospective caps remain 16MiB per trace, 1MiB per request, 100 distinct selected
request-file references per run, 16GiB aggregate input and 2MiB projected row per run.
Metadata additionally has a 2MiB file cap. Missing, conflicting, unsupported and
cap-limited extraction is preserved. Extraction completeness is never certification,
semantic use, avoided cost, causal savings, a new metric or a population filter.

## Synthetic checks actually executed

There are **20 unique final check cases**. They cover exact plan membership,
integer repetitions, pointer escaping, prose hashing, read-derived ownership,
ambiguous acquisition, unique versus repeated record generation, built-but-stale
handoff, incomplete traces, both request text envelopes, failed receiver supply,
duplicate-call consumer counting, unsent input, hash/file/aggregate/request-count
limits, explicit receipt/ingress association, path traversal, duplicate JSON keys,
and uninspected nonselected request/response references. The boundary check uses
fake pinned metadata and temporary test-only freeze/approval constant substitution.
It does not establish actual campaign readiness or independently verify accounting.

Three successive short processes ran 20 cases each: **60 passing check executions**.
These are preparation checks, not 60 unique cases or measurement trials.

| Saved result | Checks | Failures/errors | Test runtime | Named fake input bytes |
| --- | ---: | --- | ---: | ---: |
| `fake-results.json` | 20 | 0 / 0 | 0.0237630829s | 63,865 |
| `fake-results-attempt-02.json` | 20 | 0 / 0 | 0.0236372091s | 65,075 |
| `fake-results-attempt-03.json` | 20 | 0 / 0 | 0.0165520830s | 65,865 |

Each process remained below the one-second test target and each named fake-file
batch stayed below 1MiB. Final check command was `python3 -B
scripts/architecture/common9_postanalysis/mechanism_sidecar_preparation/stage-01/test_fake.py
attempt-03`. No actual all441 extraction CLI was run.

## Review limits and outstanding concern

Independent Astra static review is pending. Synthetic success does not establish
real trace schema coverage, complete lineage, causal savings, semantic correctness,
measurement eligibility or product success. No actual trace, model request, response,
result corpus, paid controller or process tree was read by the checks.

One source concern was reported to root before review: execution preflight rereads
the input manifest, and accepted-report metadata is read by `pinned_file` before
addition to `Reader.total`. Individual metadata reads are bounded, but the aggregate
counter omits that preliminary duplicate read and does not precheck the report
addition. Strict all-input aggregate-cap conformance needs independent review and
any correction must be captured in a new preparation stage. Source is held unchanged
at the checkpoint while that review occurs.

Actual use requires the root to confirm paid-controller quiescence, accept the
source after independent review and issue an exact separate GO. This stage does
not authorize candidate/model/API/evaluator/analyzer/build execution or real-corpus
inspection. Prior preparation attempts remain preserved.
