# Recovery 184 evidence package

This package preserves the separate 2026-10-20 recovery generation. The original stopped 258-run generation remains in `preservation-final-stopped258-v1`. The recovery consists of one fresh rerun of the original uncertified identity and 183 previously unstarted identities. All 184 finished with controller exit 0 and independent technical certification. The two generations supply one certified result for each of the 441 planned identities.

The [compact summary](summary.json) records the 315-run COMMON9 and separate 126-run COMMON_SUPPLEMENT6 results, generation strata, cost reconciliation, and observed paired intersections. The full analysis, raw traces, frozen source, authorization, call records and progress files are in the archive described by [chunk-manifest.json](chunk-manifest.json) and [inventory.json](inventory.json). [VERIFICATION.json](VERIFICATION.json) records SHA-256 verification and byte-identical restoration of all 3,909 member files to a new empty directory.

Archive SHA-256: `00b82af8b9c5b9a913bb24668e6cf5282c0e98ee62464ce5be60803f4b7ab88d` (46,057,937 bytes, three parts). The old failed trace remains in the earlier archive and its $0.0082976 remains in the $12.6685720 global ledger. It is excluded from the 441-result primary QA sample, where its newly certified rerun supplies that identity. The global $40 cap has $27.3314280 remaining and no unsettled reservations or unknown calls.

To verify or restore from the repository root without calling a model:

```sh
python3 scripts/architecture/package_recovery184_evidence.py verify --bundle results/architecture-evaluation/current/common-supplement6-20261019/preservation-recovery184-v1
python3 scripts/architecture/package_recovery184_evidence.py restore --bundle results/architecture-evaluation/current/common-supplement6-20261019/preservation-recovery184-v1 --out /private/tmp/via-recovery184-restored-new
```

The restore destination must be absent. Byte restoration does not establish that the absolute-path analysis can be replayed from a different directory or that paid model calls are reproducible. The package excludes compiled binaries, model weights, local environments and secrets. The freeze retains their hashes and dependency inventory.
