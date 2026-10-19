# Independent Sol diagnosis of the stopped formal controller v1

- Reviewer: GPT-6.1 Sol, High, independent bounded source and evidence review.
- Date: 2026-10-20 KST.
- Run: `ABAA-S12-r3-90923b`, COMMON9, configuration ABAA, S12, repetition 3.
- Status: diagnosis complete; no repair, certification, continuation or rerun performed.
- Immutable report: new file, preserving the frozen sources and original failure evidence.

## 1. Finding and actual scope

The observed technical failure is a shutdown protocol race between the Python harness and Rust candidate. The harness sends `cancel_work` and then `end_input` through two separately flushed writes. The frozen runtime treats either message as sufficient to end its input loop. In this run the candidate acknowledged cancellation, returned its episode result and closed stdout before the harness's second control write reported `BROKEN_PIPE`. The second write is the recorded `CANCELLATION_CONTROL_PIPE_CLOSED` failure. This independently supports the owner's preliminary explanation.

The trace explicitly records `technical_certification: NOT_CERTIFIED`. Rust return code 0, completed model calls, a result event and host completion records do not override that failure. This diagnosis establishes the concrete teardown failure for this one run; it does not certify its semantic responses or other campaign runs.

The saved quiescence attestation records 441 planned, 258 started, 258 terminated and 183 unexecuted, with controller state STOPPED. **258 ended is not 258 certified.** Certification count is explicitly `NOT_ANALYZED_DO_NOT_INFER_FROM_TERMINATION`. No campaign-wide certification or quantitative result was calculated here. Process absence and sole session 71219 exit 0 were preserved and separately checked by the coordinator; this reviewer did not repeat a process scan.

## 2. Direct evidence

### 2.1 Exact trace, top-level facts

The reviewed trace is exactly 2,938,455 bytes, SHA-256 `fc20fd3820dbc53ed269fa932ee3daeaed73e08939bced3f2b30b315d72e952a`.

- `/host_failures/0` and `/technical_failures/0`: `CANCELLATION_CONTROL_PIPE_CLOSED`, control `{type: end_input}`. These are the sole entries of each list.
- `/returncode`: 0; `/stderr`: empty string; `/technical_abort_reason`: null.
- `/control_sends/0`: `cancel_work`, reason `FINAL_ACTUAL_MANDATORY_SOURCE_PLUS_60_SECONDS`, with a successful send bracket.
- `/control_sends/1`: `end_input`, `send_error: BROKEN_PIPE`.
- `/unprovided_mandatory_events`: empty. This is a recorded provision fact, not semantic certification.
- Four recorded calls have status COMPLETED, `actual_api_sent: true`, and terminal timestamps before cancellation. No discarded-after-candidate-end flag is present in those four call records.
- The 86 event entries end in cancellation acknowledgement, episode result, then the host's stdout-close marker. No EPISODE_ERROR event is present in those 86 kinds.
- `/events/84`, event-95 EPISODE_RESULT: `active: 0`, `pending_inputs: []`, `failures: []`, and both requests have recorded status HOST_COMPLETE. These are candidate and host lifecycle observations, not physical product completion or independently judged content correctness.

### 2.2 Recorded order on the host monotonic clock

The host observation fields below establish the recorded sequence. Candidate event `host_monotonic_ns` denotes event assembly before stdout locking, not delivery. The table uses host `observed_ns` for event receipt and the harness's send bracket for writes.

| Observation | Timestamp ns | Exact reference |
| --- | ---: | --- |
| Effective admission and observation cutoff | 2007419116071208 | `/effective_admission_stop_ns`, `/observation_deadline_ns` |
| Host cancellation requested | 2007419120896000 | `/cancellation_requested_ns` |
| First control send begins | 2007419120952166 | `/control_sends/0/send_begin_ns` |
| WORK_CANCELLATION_ACK observed | 2007419124144041 | `/events/83/observed_ns`, event-94 |
| EPISODE_RESULT observed | 2007419124422666 | `/events/84/observed_ns`, event-95 |
| PROCESS_STDOUT_CLOSED observed | 2007419124443875 | `/events/85/observed_ns` |
| First control send returns | 2007419124701625 | `/control_sends/0/send_end_ns` |
| Second control send reports BROKEN_PIPE | 2007419124897333 | `/control_sends/1/at_ns` |
| Trace record ends | 2007419136067583 | `/end_ns` |

Cancellation was requested about 4.825 ms after the recorded cutoff. The second control failure was recorded about 0.453 ms after stdout closure. ACK and stdout closure occurring within the first send's bracket is possible under concurrent host/candidate scheduling: a successful flush does not require the candidate to remain alive until the caller resumes. It does not show that `end_input` was successfully delivered.

The settlement deadline was 2007539116071208 ns. There is no recorded settlement-timeout, forced-kill or unfinished-model-thread failure in this trace. The four model terminal records precede cancellation; the last is 2007370441973416 ns. These facts narrow this occurrence to teardown after the recorded work, rather than an observed unsettled provider request.

### 2.3 Exact saved terminal and stderr

`failed-terminal.json` records state TERMINATED, harness PID 28191, returncode 0, `technical_failure: true`, `budget_stop: false`, and a stdout summary with four calls, two sources and the end_input pipe failure. Thus process success and the harness's explicit technical-failure signal coexist.

`failed-controller-stderr.json` preserves Python's `Exception ignored while finalizing file <_io.TextIOWrapper name=4 encoding='UTF-8'>`, followed by `BrokenPipeError: [Errno 32] Broken pipe`. The traceback points to the harness module's main invocation at line 304. The candidate's own trace stderr is empty. The Python finalizer traceback is consistent with the failed pipe write/flush during teardown; its line 304 location alone does not identify an independent error in the CLI summary or a Rust panic.

## 3. Frozen source mechanism

All source references below are under `frozen/source/` in this result root. Only static text was read.

| Frozen source | Lines | Relevant behavior |
| --- | --- | --- |
| `benchmark/architecture/common9_poc_20261019/harness.py` | 59–63 | Spawn candidate with piped text stdin/stdout/stderr. `send` holds the write lock and separately writes and flushes each value. |
| Same harness | 175–190 | Cancel the host observation gate, preserve effective cutoff and settlement deadline, then iterate over `[cancel_work, end_input]`; catch each BrokenPipeError and retain CANCELLATION_CONTROL_PIPE_CLOSED. |
| Same harness | 203–204 | At the observation deadline, invoke cancellation with the normal FINAL_ACTUAL_MANDATORY_SOURCE_PLUS_60_SECONDS reason. |
| Same harness | 100–128, 256–259 | Reader records PROCESS_STDOUT_CLOSED when stdout iteration ends; main loop exits on that marker. Only pre-cancellation closure adds CANDIDATE_EXIT_BEFORE_OBSERVATION_CUTOFF. |
| Same harness | 278–300 | Preserve settlement and forced-kill failures; collect candidate stderr/returncode; include the pipe failure among technical_failures and mark NOT_CERTIFIED. |
| `prototypes/candidates/common9_poc_20261019/src/context.rs` | 103–140 | Stdin reader cancels effects, emits WORK_CANCELLATION_ACK and forwards cancel_work to runtime. end_input is separately forwarded. The stdin reader itself continues after cancel_work. |
| `prototypes/candidates/common9_poc_20261019/src/runtime.rs` | 1003–1010 | On either end_input or cancel_work, set `ended = true`, notify and break the runtime input loop. It does not wait for both controls. |
| Same runtime | 1077–1092 | Join scheduler/background work, then return the final state as Ok. |
| `prototypes/candidates/common9_poc_20261019/src/main.rs` | 13–22 | Successful runtime return emits EPISODE_RESULT and main returns. An outstanding stdin-reader thread is not joined here. |

The source therefore permits candidate main to finish after the first control while the harness still intends to flush the second one. In this run active work had settled, so the documented join path need not delay exit. Source behavior and observed event order agree without requiring a candidate crash, budget block or provider failure.

The trace's binary SHA-256 is `5aea60533c5b2a46c0e22cf8a7e4d86d51b4ba2c3860f6006341f0f073dd2e1d`. No build or binary execution was performed to re-establish source-to-binary equivalence in this review; the diagnosis uses the preserved frozen-source association and its agreement with the actual trace.

## 4. Inference, uncertainty and limits

### Supported causal inference

The immediate technical-stop trigger in the preserved terminal is the harness's explicit pipe-failure flag for the second control. The best-supported root cause is the producer/consumer teardown mismatch: the producer expects a second flushed control to be writable, while the consumer may complete on the first. Concurrency determines whether that second write succeeds. A single exposed failure is sufficient to reject an assumption that both sends are always safe.

### Not established here

- The precise kernel scheduling interleaving or the exact instant the candidate's stdin read end closed is not recorded. Stdout closure is an observed adjacent lifecycle marker, not a direct stdin-close timestamp.
- The finalizer traceback does not establish whether the second message's `write` or its explicit `flush` first raised. Both occur in the same `send` statement; the saved control receipt establishes the failed operation, not the substatement.
- Only this exact trace and its saved terminal were inspected. Frequency, the number of other exposed runs, certification of the preceding 257 terminated jobs and any effect on aggregate comparison remain unreviewed.
- No semantic, physical-delivery or product-completion verdict follows from this diagnosis. Recorded HOST_COMPLETE and returncode 0 are retained at their actual scope.
- The 183 unexecuted planned jobs are not observed failures or successful samples. They remain unexecuted after this controller stop.

## 5. Recovery requirements, without changing the observed result

1. Preserve this failed run, terminal/stderr, original cutoff, failure classification, hashes, costs and all previously ended rows. Do not retroactively erase or waive the end_input error to turn this run into a certified observation.
2. Before any separately authorized continuation, define and review a coherent shutdown handshake: decide when cancel_work allows process exit, whether end_input remains mandatory, how acknowledgement and pending work settlement interact, and what evidence proves the chosen endpoint. The requirement is a consistent protocol; this report does not select or implement a repair.
3. Preserve observation/admission cutoff and settlement semantics. A teardown correction must not extend accepted work, suppress genuine premature termination or provider-settlement failure, change semantic requirements or adjust the evaluation contract to favor observed outcomes.
4. Any code correction needs separately authorized implementation and validation, new source/binary provenance and an explicit freeze/compatibility disposition before new paid work. No repair, test, build, API call or rerun is authorized or performed by this report.
5. Account separately for terminated, technically certifiable, semantically reviewed and unexecuted jobs. Decide explicitly how a future authorized run replaces or supplements this failed slot; retain the original failure and preparation costs. Do not silently resume the stopped controller or treat all 258 terminated rows as valid final comparison results.

## 6. Read bounds and provenance

This diagnosis used bounded local reads of the exact 2.94 MB failed trace, saved quiescence/manifest/terminal/stderr, and the four named frozen source files. Standard JSON projections exposed lifecycle fields, event kinds, the last 20 events and four call terminal fields. No campaign corpus, original aggregate runs.jsonl, ledger/model-call corpus, candidate-initial payload, semantic evaluator, analyzer, domain parser, compiler, test, model, API, network, candidate or actual extractor was run. No source was edited. Each read command returned within one second; every input file was below 16 MiB and total content read including repeats was below 10 MiB, within the 128 MiB cap. The only new artifact is this report.

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| `postanalysis/quiescence-stop-v1/quiescence.json` | 7356 | `e4c5d1401c329d333bc3bf5338dd30a45016d61c3111d41026f96904cfdc815e` |
| `postanalysis/quiescence-stop-v1/snapshot-manifest.json` | 6543 | `23d4fdbca633e9b796e2d57380ff72f09bc160f9a14395aaaeb166b5a4ebed9d` |
| `postanalysis/quiescence-stop-v1/failed-terminal.json` | 1927 | `cc750f41807105b9a6e4031225ec51c13047b563abaf3ac34d2e83c29971d29e` |
| `postanalysis/quiescence-stop-v1/failed-controller-stderr.json` | 1201 | `88ff13894960d802601a9e1c0878b66753780620e02e0f34f38f12c8e7bebc78` |
| `raw/runs/ABAA-S12-r3-90923b/trace.json` | 2938455 | `fc20fd3820dbc53ed269fa932ee3daeaed73e08939bced3f2b30b315d72e952a` |
| `frozen/source/benchmark/architecture/common9_poc_20261019/harness.py` | 30809 | `d30750622e97978f769da8366a8c12510e91cbd482ead32d1111ed9cc7e079d9` |
| `frozen/source/prototypes/candidates/common9_poc_20261019/src/context.rs` | 21488 | `e6ad3d6a2d0cf62e2d9994820cfd906748623f1cab7189f13b7748a60af60de2` |
| `frozen/source/prototypes/candidates/common9_poc_20261019/src/runtime.rs` | 69074 | `d96f61406119d870bf65bb9045a8fe2bb8eb1aa68ae8319a65a09845a4437402` |
| `frozen/source/prototypes/candidates/common9_poc_20261019/src/main.rs` | 588 | `b4a56e4357270c0d14447ee1e7fd2a8b60d6d9374a1d92f66787c07ace47ad75` |

Result root for every relative reference: `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/`.
