# Closed-block v3 plan v1: bounded progress-prefix addendum

2026-10-19. GPT-6 Astra Ultra. Planning judgment only; no progress file read or extraction was executed for this addendum.

Base plan: `astra-closed-block-v3-plan-v1.md`, SHA256 `1e8d7b36fe70f4b9dfb983f049cac2a16a9e6160696135b000c3951a7657cbcf`. Preserve that document unchanged. This addendum relaxes only its requirement that terminal byte ranges be known before reading any progress metadata.

## Decision

**Allow one bounded prefix snapshot of the exact campaign progress JSONL to locate the explicitly requested frozen block.** This is a small metadata read with deterministic selection. It does not authorize a corpus/trace scan, outcome-based block selection or a hash claim about the entire mutable progress file.

The requested freeze, block ID, population, episode, repetition and next-block cursor must be fixed before the read. The helper must not discover a preferred block, select the latest run, change configurations or skip an unsuccessful block based on the prefix contents.

## Concrete bounds and recorded facts

1. Open only the declared original `joint-formal-v1/campaign-progress.jsonl` under the verified campaign root. Reject symlink escape or an unexpected file type. Perform one read beginning at byte 0 with a hard maximum of **1,048,576 bytes**. Do not loop to EOF, fetch another prefix/window, inspect directories or read active `runs.jsonl` to complete discovery.
2. Preserve the exact captured bytes in a new immutable prefix artifact, with its SHA256 and captured byte count. Record the last complete LF boundary, the number of complete-line bytes and any trailing partial-byte count. Parse only complete newline-terminated records. A cut UTF-8 character or partial JSON line at the boundary is unparsed captured context, not malformed completed evidence. Count this artifact and subsequent selected-range reads toward the 128 MiB command limit; the 16 MiB per-file cap remains.
3. Record the source path, actual capture time and bounded-read method. File identity metadata may help detect replacement during the operation, but the prefix hash describes only the copied bytes. Do not label it a whole-file hash, final controller snapshot or proof that no later records exist. Appends outside the captured prefix remain explicitly out of scope.
4. Locate **only** records matching the already requested frozen block and its seven planned jobs. Require exact TERMINATED state and one terminal record per configuration/run identity within the captured complete prefix. Reject duplicate/conflicting requested-block terminal records, mixed identity or unsupported stdout/run-ID linkage. Retain the original full terminal lines, including ordinary/technical failures and stderr. Neither exit0 nor a success count is an admission criterion.
5. Record each selected line's absolute byte offset, byte length including LF and SHA256. Reread **only those seven exact ranges** from the same declared original file and require identical bytes. Detect file replacement or truncation that invalidates these ranges. Concurrent appends elsewhere are permitted. A mismatch fails index preparation without rewriting the original evidence.
6. If the requested seven records are not fully present in the complete prefix, return an immutable `REQUESTED_BLOCK_NOT_FULLY_IN_CAPTURED_PREFIX` diagnostic. Do not infer that missing jobs never ran, call the block failed, extend the read automatically, select later jobs or run an experiment. A later bounded attempt or an explicit supplied-range index requires root scheduling/authorization.
7. The normalized index binds the prefix artifact/hash, selected exact line copies/ranges, frozen job identities, supported recorded run-ID origins and closed trace hashes. The remaining v1 validation, no-saved-evaluation policy, source/oracle separation, immutable output and one-block limits stay in force. Keep the initial 1-second extraction policy unless root explicitly changes it; never trade away evidence checks to meet it.

## Evidence limit

A bounded prefix cannot prove the absence of contradictory duplicates appended later. The frozen controller's no-duplicate lifecycle and the seven current closed traces support this intermediate review, while final quiescent reconciliation must check complete campaign identities. If a later contradiction is discovered, preserve the earlier index and mark its association disputed; do not silently relabel it certified. This limitation is compatible with incremental semantic review because the bundle binds the exact seven observed runs and never certifies campaign completion or metric eligibility.

## Additional tiny synthetic cases

Before actual use, cover: a valid requested block inside the prefix; unrelated later blocks not selected; a duplicate requested terminal rejected; seven records beyond/truncated at the cap returning the explicit diagnostic; partial LF/UTF-8 tail excluded from parsing; append after capture accepted when selected bytes match; changed selected bytes/file replacement rejected; and requested-block/cursor identity fixed before reading. These are prospective tests for Sol, not executions performed by this review.
