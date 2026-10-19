# DP1/DP4 mechanism sidecar preparation, stage 02

Status: **REPAIRS_PREPARED_FOR_INDEPENDENT_SOL_HIGH_REVIEW**.
Real extraction remains forbidden before independently supplied controller quiescence
and a separate exact root GO. Stage 01, its three saved attempts and its independent
review remain unchanged. This stage changes only the new standalone sidecar and
adds isolated preparation artifacts. No accepted adapter/assembler, frozen source,
measurement path, contract, presentation or Git staging/commit/push was changed.

The accepted plan remains
`results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/planning/mechanism-evidence-readiness-v1.md`,
SHA256 `071f5872010214aca45f168c585ac1e323f7c303187693f69c45d8e0796e85ea`.
The complete prior review is `../stage-01/static-astra-review-stage01.md`,
SHA256 `fd0600602845fbdfdcf1e99584da9d1510552fc1c7f685270d6856a881a78e11`.
The stage-02 implementation and checks were performed by GPT-6.1 Sol High.
Independent closure of the following repair claims is pending.

| Finding | Prepared repair | Final synthetic cases |
| --- | --- | --- |
| MS1-01 | One retained Reader from source-self/preflight through rejection. Every bounded read passes the shared aggregate gate; read allowance has no extra sentinel byte. Stable full-file versus failed/prefix hashes and zero-byte attempts remain explicit. Saved rejected receipts retain bytes and inventory. | 01–04 |
| MS1-02 | Occurrence tuples carry exact association pointers separately from record/capsule pointers. Outer message pointers and decoded-input pointers remain distinct for both bridge shapes. | 05–07 |
| MS1-03 | Identity fields are scalar-validated before hashing; malformed index entries remain conflicts. Collection processing states are tracked; interrupted/affected collections fail closed. Per-row serialization failures preserve available projected facts with explicit unsupported markers or a bounded failure row. Every accepted planned identity survives individual evidence errors. | 08–11, 22 |
| MS1-04 | Raw call occurrences, recorded dispatch UUIDs, established operation nodes and ingress IDs are distinct. UUID/causal and node/UUID conflicts or missing identities remain raw evidence and unresolved coverage, excluded from established distinct-operation counts. | 12–16, 23–25 |
| MS1-05 | Reuse and producer-support acquisition joins consider only prior recorded read events. Timestamp adjacency is never a substitute. Unique committed producer payload joins do not impose universal commit-publication order on consumption. | 17–19, 26 |
| MS1-06 | Exact selected DP1 producer fields `observed_evidence` and `actual_delivered` are projected with source/span identities and hashed prose. No extra stage discovery or response interpretation. | 20 |

Case 21 additionally preserves the distinction between unsent saved input,
built-but-stale capsules and actual receiver supply. Failure outcomes remain
eligible as observed dispatched supply when canonical identity is established;
these extraction states do not alter measurement eligibility or semantic oracles.

## Prospective CLI and fixed limits

Root explicitly approved adding required `--formal-root`. The full prospective
interface is `--formal-root`, `--input-manifest`, `--input-sha256`,
`--accepted-report-manifest`, `--accepted-report-sha256`, `--output`.
The explicit root confines the fresh exclusive output before any read and permits
an immutable rejection receipt even for a first-boundary failure. It does not
establish live quiescence. Existing manifests are unchanged.

The accepted report manifest's exact `calls.jsonl` role remains pinned by recorded
hash and size and confined beside that manifest. The calls file and call ledger
are not reread, repriced or recalculated. That is reference-only accounting linkage.

The existing caps remain 16MiB per trace, 1MiB per linked request, 100 distinct selected
request-file references per run, 16GiB aggregate input, 2MiB projected row per run and
an additional 2MiB metadata-file cap. The shared aggregate byte counter now includes
the exact source-self read and failed bounded reads. A read that encounters concurrent
file growth cannot consume beyond its predeclared per-file/aggregate allowance.

## Saved synthetic checks

Final source has **26 unique targeted check cases**. There were two separate short
preparation processes, totaling **48 check executions**. Stage 01's 20 unique cases
and 60 executions remain separate historical preparation evidence.

| Saved result | Cases executed | Failures/errors | Test runtime | Named fake file bytes |
| --- | ---: | --- | ---: | ---: |
| `fake-results-attempt-01.json` | 22 | 0 / 0 | 0.02644699998s | 49,640 |
| `fake-results-attempt-02.json` | 26 | 0 / 0 | 0.03092325013s | 52,263 |

Each named fake-file batch is below 1MiB; both test processes are below the one-second
target. The final command was `python3 -B
scripts/architecture/common9_postanalysis/mechanism_sidecar_preparation/stage-02/test_fake.py
attempt-02`. Both full441 synthetic planned-row checks write to bounded in-memory
sinks: final late-boundary rejection is 552,195 bytes and malformed-index/call/record
processing is 562,361 bytes, each with exactly 441 rows. These are synthetic writer
checks, not campaign extraction or measurement trials. The rejected execution receipt
is saved as a real tiny fake file and inspected for exact read accounting. Fake freeze
and approval hashes are substituted only inside the test scope.

## Limits of this preparation

No actual sidecar CLI extraction, campaign trace/result/model request/response read,
candidate/parser/evaluator/analyzer/build invocation, model/API/network operation or
paid-controller operation occurred. Named static plan/review/source files were read.
Synthetic success does not establish real-corpus ABI coverage, controller quiescence,
semantic use, original-read avoidance, causal savings, product completion, a new metric
or any measurement certification. Identity-conflicted exposure can remain unresolved.
The source, copied source, tests, results and checkpoint hashes bind this preparation;
source acceptance still requires independent Sol High review and root authorization.
