# Independent Sol review of the final partial report v1

**Verdict: no material P1/P2 finding identified within the requested review scope.** Reviewer: GPT-6.1 Sol High, `/root/sol_high_continuation/sidecar_closure_sol`, 2026-10-20 KST. This is a bounded reporting review, not a new campaign certification, semantic review or architecture selection.

## Reviewed scope and bindings

The inspected 17-presentation-results.md snapshot was174,488bytes, SHA-256 `e784d8cf64e2e671b8e4dde4ec09d9a4385574f04011021f9bf7fa8614eb75c3`. Review covered its top status, §9 status correction, §9.11 history, §11 framing and new §12.1–12.4 through line915. §12.5 restoration/handoff is outside scope.

| Input | SHA-256 |
| --- | --- |
| `planning/sol-final-prediction-judgments-v2.json`,14,137bytes | `6672b4145e9c6b9eb8035f3e0b597640bc9b3354b34a443162a6102d9de02697` |
| `stopped258-report-v1/report.md`,11,227bytes | `571837dfc1cce2d2a4be1aca8e533d4fba767c86b06ef911264aec181d61622a` |
| `stopped258-report-v1/case-repetition.jsonl`,2,088,589bytes | `aaf135b3b30c483ff64c86c1109a4a219886b726b1d4199a5b63cb6bf9ab7f05` |
| `stopped258-report-v1/manifest.json`,340,187bytes | `84b92183ff3cf4d7ec199ea9b16283e0b84ea0b07df85f3d2d2ed07ca0680451` |
| `stopped258-mechanism-v2/execution-receipt.json`,1,412,880bytes | `5c9dd39304d54de8680ec2cd6f9bfd302bcc5053f14c41a84c16181937d1241e` |
| Prior independent supplement interpretation | `f84bda268a264652e28ce53fc729dfffdbbe1cc363921d080be80c094404a384` |
| Prior independent technical-stop diagnosis | `09f3fe92f4f9ee7f86119f213e810c0260cf97aeb28671187fd4c16df2b7870f` |

Paths in the table are relative to J/postanalysis, where J is `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/`. The two prior independent reports are the reviewer's retained work and conclusions; they were used without reopening raw campaign traces or model data.

## Checks and result

| Area | Review result |
| --- | --- |
| Termination and certification | Case rows independently sum to COMMON9 152 certified+1 technically uncertified+162 unexecuted, supplement105 certified+21 unexecuted. Thus258 terminated,257 certified,1 uncertified,183 unexecuted. §12.1 preserves the failed ABAA/S12/r3 and unstarted AABA/S12/r3; exit0 is not substituted for certification. Shutdown explanation agrees with the independent diagnosis and retains scheduling/substatement uncertainty. |
| Numerical tables | Simple projection and arithmetic over441 CASE_REPETITION rows reproduce every configuration's QA01/03 observed mean/count/planned denominator, QA02 known-correct/planned count, known expenditure and observed RSS maximum in the two §12.2 tables, to displayed rounding. QA04 remains the separately accepted11/6 observation; it was not rerun here. |
| Exclusive partitions | COMMON9 QA01=614 observed+96 not observed+2 uncertified+793 unexecuted=1505; QA03=312+90+2+401=805; QA02=58 correct+140 adopted incorrect+78 not adopted+2 uncertified+247 unexecuted=525. Supplement QA01=140+70+42=252; QA03=105+35+28=168; QA02=35 correct+63 not adopted+28 unexecuted=126. These match the prose. |
| Missingness and fairness | Last block incompleteness and differing observed case composition are explicit. No completed5/3-repeat representative mean, replacement by the first2 repetitions, zero-time imputation, removed failure denominator or complete QA05 bundle mean is claimed. S81/S83/S84 r3 remain unexecuted for all7 configurations. |
| RSS qualification | Case counters reproduce one invalid sample, zero startup-zero and sampler-failure records, and maximum gaps295.665459ms/156.784708ms for representative/supplement scopes. Same-sample local RSS, lower-bound status, MB/MiB distinction and shared-page/full-product limits remain explicit. The failed ABAA run's observed RSS is not silently removed. |
| 24 cells | JSON has24 unique DP×QA cells and exactly DP1–4×QA01–06 coverage. All24 prose rows agree with the supplied JSON judgments and figures. Automatic adapter UNREVIEWED worklist is distinguished from the separately authored manual judgments. Numeric order is not converted into a preregistered prediction or winner. |
| DP2 and DP3 mechanism limits | S80/S82 ABAA actual request-model overlap, S81 no overlap, unknown exact free-K2/provider concurrency, S82 latest-input failure, dense/sparse noncompletion and S85 native skip/failed r2 binding remain qualified consistently with the independent supplement interpretation. S85's formal activation evidence is stronger than the earlier limited semantic bundle, without relabeling semantic failure or proving arbitrary OS-wide process absence. |
| DP1 and DP4 mechanism claims | Positive reuse/read counts are described as events including producer reads, not independent requests or saved calls. Same-generation ingress/receiver exposure is distinguished from semantic dependence, original-read elimination, physical completion and amortized savings. Three AAAB/S02 cap-limited rows are retained; COMPLETE is not absence or technical certification. |
| Cost and prior scopes | Known expenditure is distinguished from complete fixed-bundle mean, local API0 from local compute cost, reused AAAA evidence from duplicated spend, and prior missing-scope ledger rows from uncertain token charges. DP1 C relation-production cost is corrected to0.3139204USD while preserved v1 is explicitly superseded. No material overstatement was identified in that reporting distinction. |

## Clarified hash concern

During review, the path-less v2 `bindings.report_sha256` was initially compared with report.md and flagged to the coordinator as an apparent mismatch. The coordinator clarified that it denotes report.json. The retained accepted manifest independently confirms report.json is26,077,453bytes with SHA-256 `f00e18d83c8e8a6eddac97363a5a4f477c7820f6b88ea70ba503bbcbe1a99c20`, whereas report.md has the distinct hash shown above. **The apparent hash-mismatch P2 is withdrawn.** It is not a numerical or accepted-output hash defect. The coordinator plans an immutable v3 with explicit path/hash/bytes references for both files. This review does not certify a future v3 or require a metric change.

## Precise limits

The accepted report.md, full v2 judgment JSON, requested presentation ranges and bounded metadata were read. Case-repetition rows were projected for arithmetic checks; no analyzer or evaluator was executed. The sidecar receipt confirms255 COMPLETE_WITHIN_DECLARED_SCOPE,3 LIMIT_EXCEEDED,183 MISSING and output SHA-256 `0ed9bf4566505b08f250953f2374b2d0a7d616fd16a17cc67d2e6652f882f58a`. The124,893,994byte sidecar output was not opened. Consequently its positive reuse/read counts and one repeated-generation claim were reviewed for interpretation and preserved scope, not independently recounted here. No scope-complete row was treated as technical certification.

The exact paired-obligation deltas, role-cost subtotals, global UUID/rate reconciliation and old-ledger scope subtotals are not reconstructible from case-repetition and report.md alone. They were assessed for coherent wording against the supplied accepted figures and metadata, without reopening paired.jsonl, calls.jsonl, ledger/model corpus or the26MB report.json. This verdict does not claim a fresh independent recomputation of those values. Detailed causality, all unreviewed semantics, original sidecar payload association and §12.5 remain outside this bounded review.

No raw trace/model input, analyzer, extractor, tests, compiler/build, API/network, candidate or source edit was performed. No17 edit, contract/denominator change, rerun, restart or new comparison population was made. Input files remained below16MiB, each command returned within1second and total bounded reads remained below128MiB. The only new artifact is this immutable report. No material P1/P2 correction remains requested within the reviewed scope.
