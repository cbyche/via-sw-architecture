# Closed-block reporting helper

This helper only arranges recorded evidence for independent Astra review under frozen19 section5. It creates no endpoint, metric, semantic success numerator, paired aggregate, oracle or model input. It imports no evaluator and runs no candidate, model, parser, build, network request or subprocess. Primary common9 and supplement6 remain separate.

## Scope and input

Root must explicitly name one already closed seven-configuration population/episode/repetition block. The immutable freeze must name all seven planned identities. The supplied closure summary supplies only the seven explicit run IDs. No recursive scan or whole runs.jsonl read occurs. A duplicate identity/run, mixed phase/configuration/binary, wrong-root path, changed frozen source or oversized input is rejected. A missing listed run/trace retains every declared response/goal obligation as missing evidence and UNREVIEWED; absence alone is not evidence that a run never executed.

The freeze contains absolute frozen_path references. Those must resolve exactly under the supplied root's frozen/source tree; active source files or relocated snapshots cannot substitute. The helper checks only explicitly relevant frozen oracle/evaluator/analyze_joint/contract files, never all runtime files or large dependencies. Supplement functional-oracles is additionally referenced when that population is explicitly selected. An unknown functional-oracle shape is retained as an unresolved reference, without normalization.

The CLI requires root, closure, population, episode, rep, freeze-sha256 and a new output path. It must not be launched against actual evidence until root approves a bounded extraction. An optional saved-evaluations JSON index maps only this block's run IDs to exact absolute path/sha256 pairs. Missing saved evaluations are explicit; the helper never executes an evaluator to fill them.

## Resource bounds

Each trace is read and hashed sequentially with a16MiB per-file cap and128MiB total-input cap. No parallel workers or corpus scan is used. Large memory sample arrays are not copied into the readable evidence. Only event records, source/input context, actual host receipts and explicitly recorded model request/response file references are retained. Referenced model files are hashed without copying their content. The output accumulates projected plain-text evidence, not all raw traces. Other strings above65,536 characters, arrays above256 items and image/binary/base64 payloads become explicit omission records containing unchanged raw file/hash/JSON-pointer references. No image decoding occurs. The relevant body remains UNREVIEWED if an omission prevents judgment.

This bounds work; it does not establish that reporting creates no host-load interference. Actual extraction scheduling remains root's decision while measurements run.

## Outputs and association

- `manifest.json`: exact input/source/tool hashes, explicit planned7 identities, declared oracle response/goal counts, separate observed handoff/control event count, missing/omitted records and byte limits.
- `evidence.json`: recorded events in unchanged raw index order with ID/ns/data and exact JSON pointers; source/revision/extent/read/authority facts stay in these records. Recorded candidate input and source plain text remain context, without invented associations or commands. Base64/images are reference-only.
- `worklist.json`: stable population/configuration/episode/rep/run_id/obligation_kind/obligation_id keys, frozen expectations and raw evidence links. Expected oracle obligations and observed ingress events have distinct origins. Automatic QA02 binding obligations are not a new manual denominator.
- `review.md`: readable expectation, original selected endpoint fact when saved, all matching actual callback bodies, adopted plans and stimulus-linked timeline; full preceding/following trace context remains in evidence.json.

Association uses only exact recorded stimulus_id/origin_event_id/request.stimulus_id values. All raw events remain available, including prior/later Task/revision, source/permission changes, control, approval, actual ingress/receipt/result and rejection facts. Unknown wire formats are not reconstructed from the oracle. The helper makes no claim that a linked event or recorded source is semantically correct.

The raw first matching display is ordered by actual observed_ns. A tie, missing clock or duplicate event ID is explicitly ambiguous; the helper does not choose a replacement. Saved frozen QA01 accepted_callback_id and saved QA03 rows are separate, unchanged facts. A later callback never replaces the original selected endpoint. A missing saved evaluation supplies no certified primary eligibility or latency. Missing output is distinct from missing execution/technical evidence; none produces a new automatic semantic FAIL.

All four new review dimensions—body, Task/revision, approval/authority and handoff—are UNREVIEWED. Subsequent Astra PASS/FAIL/UNREVIEWED judgments belong to separate versioned artifacts that bind this bundle's hashes. Existing frozen diagnostic statuses, if present in the saved artifact, are evidence rather than new judgments. Partial semantic review never changes QA01's original timestamp or denominator.

`bundle-schema-v1.json` defines the manifest and named evidence/worklist schemas. Output files use exclusive creation, with the manifest written last. An existing output directory is never overwritten. A failed write may leave an incomplete new directory, which must be preserved and replaced by a fresh output name.

## Synthetic checks

`test_extract_closed_block.py` constructs only temporary tiny synthetic roots and checks wrong root/phase/configuration, duplicate/missing runs, ambiguous callbacks, oversize rejection, payload omission, immutable output, absent saved evaluation, no-output distinction, and raw-first versus saved-selected callback separation. It never opens actual campaign evidence. Test execution requires root's explicit clearance; the authorized synthetic-only result is recorded immutably in synthetic-check-v1.json.

Planning source: joint-formal-v1/postanalysis/planning/astra-final-report-plan-v1.md, SHA256 bb23355ba50776e1096ccb009e1fd8d170d46a9d55abff8fe8eae273b9234e41. This helper is outside the112 frozen runtime sources and changes no running experiment or QA04 artifact.
