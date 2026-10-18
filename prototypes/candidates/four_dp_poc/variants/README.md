# Frozen campaign implementation variants

`dp05-receiver-v2.rs` is the pre-DP05 fairness correction authorized before the first DP05 BASIC trial. It replaces `src/dp05.rs` only in a separately built scratch copy of the common crate. The original crate/binary remains the frozen DP01/02/04 implementation.

All ABC share receiver select-or-complete. A/B can complete from actually supplied valid source-backed capsule evidence; otherwise the receiver chooses allowed original reads and then completes. C's manifest alone cannot supply facts. Rejected shortcut/repair calls remain recorded and charged. This is a receiver protocol correction, not oracle-based tuning or an accepted architecture decision.

The DP05 v2 campaign freeze records this exact source, shared baseline source hashes, fixture/oracle hashes, model/settings and binary. Rebuild by copying the baseline crate without its `target`/`variants` directories, replacing `src/dp05.rs` with this file, then running Cargo with that scratch manifest.

## Selected tactic implementations

- `dp01-A2-fragment-invalidation.rs`: replaces DP01 only. The `A2` tactic retains valid working evidence fragments from unaffected sources; common recent delivered-answer history remains common.
- `dp02-A1-bounded-side-question.rs`: replaces DP02 only. The model decides whether a short tool-free independent side answer is suitable; otherwise the original A boundary handles the input. Original lifecycle and K are retained.
- DP04 A4/B4 use the original crate's dormant `A4`/`B4` dispatch. A4 supplies the same screen producer to the joint final writer; B4 parallelizes initial producers under K2 and is a separate diagnostic.
- `dp05-C2-exact-read-cache.rs`: speculative v2-based candidate until selected. Exact source/revision/range/access-epoch/recipient cache, current rights checked on every hit. Counters refer to source-file reads, not physical disk I/O.

Rebuild a preserved variant into a new directory (no model calls):

```sh
python3 benchmark/architecture/four_dp_poc/rebuild_variant.py dp05-v2 --output /private/tmp/via-dp05-v2-rebuild
```

The script also supports `dp01-A2`, `dp02-A1`, `dp04-tactics`, and `dp05-C2`. Never replace the original crate/binary or prior evidence. Campaign freezes under `results/architecture-evaluation/current/four-dp-qwen37-20261018/` record the exact selected source and binary hashes, affected case and execution settings.
