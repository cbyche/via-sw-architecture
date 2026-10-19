# Stopped 258/441 evidence checkpoint

The campaign stopped on its frozen technical-stop rule after 258 terminated episodes: 257 technically certified, 1 technically uncertified, and 183 verified unexecuted identities. These counts are not semantic success. The fixed 441-row population and all prior generations/failures remain preserved. Global recorded spending including prior campaigns/probes is **USD 7.3272788**; reserved and unknown amounts are zero.

## Contents and limits

- 25,249 files; 4,798,687,758 logical bytes (4.799 GB).
- Compressed archive: 274,636,943 bytes (274.637 MB), split into 14 parts, each at most 20 MiB.
- Whole archive SHA256: `805f257595b7ca512eb0a9d5ef0f1cf3ccc0efbcce571cca5696438c6a8b024f`.
- Raw inputs/responses/failures, frozen source, planned identities, contracts/reviews and final analysis provenance are included within the inventory's explicit scope.
- Weights, targets, caches, environments and compiled frozen binaries are excluded; their inventory/build references are retained. Local originals were not deleted or overwritten.
- This is lossless preservation of selected raw evidence, not a complete execution-environment clone or a completed 441-episode claim.

## Actual verification and restore

Both commands below ran once with exit 0 from `/Users/changbae/Workspace/via-sw-architecture`. Verify checked all 25,249 included files. Restore created a previously absent destination and verified all 25,249 reconstructed files against recorded hashes. No API calls occurred. See `VERIFICATION.json`, `VERIFY-RESULT.json`, and `RESTORE-RESULT.json` for exact command/result hashes and sessions.

```sh
python3 -E -S -B scripts/architecture/package_common9_evidence.py verify --bundle /Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/preservation-final-stopped258-v1
python3 -E -S -B scripts/architecture/package_common9_evidence.py restore --bundle /Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/preservation-final-stopped258-v1 --out /private/tmp/via-common15-stopped258-clean-restore-v1
```

The restore command rejects an existing nonempty destination. Do not overwrite originals. `RESTORE.md` contains reconstruction/build instructions and environment limitations. **Relocated API-free analysis is UNVERIFIED and was not run**; byte reconstruction is the validated scope. This README and verification sidecars are outside the archived selection in the excluded package directory.
