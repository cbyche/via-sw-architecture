# Evidence checkpoint restoration

This is a lossless raw-evidence/source checkpoint, not a full executable environment or proof of final results. Inspect PRESERVATION-INVENTORY.json provenance records and every generation's own freeze, planned population, failures, cost and measurement-status ledgers. Old common9, old condition diagnostics, supplement6 and joint441 retain separate provenance. A planned population is not a completed measurement; partial or technically invalid generations are not silently pooled.

Verify all ordered parts, whole archive and every included member without API calls:

    python3 scripts/architecture/package_common9_evidence.py verify --bundle BUNDLE_DIRECTORY

Restore only into a previously absent or clearly empty directory; the command verifies all restored byte hashes, modes and symlink records:

    python3 scripts/architecture/package_common9_evidence.py restore --bundle BUNDLE_DIRECTORY --out NEW_EMPTY_DIRECTORY

Do not use unverified wildcard concatenation or extract over an existing checkout. The manifest records ordered <=20MiB parts, SHA256 values and every included raw member. Git-ignored JSONL, CSV, failed attempts and prior generations are included by filesystem traversal. Weights, secrets/environments, caches, compiled binaries and recursive preservation outputs are excluded. Potential embedded credential content causes packaging to fail closed, without redacting evidence.

Original files are never overwritten or removed. Source stamps are checked across packaging; use a quiescent, settled result generation. A failure preserves partial packaging output for diagnosis and is not a complete bundle. Absolute paths in historical evidence identify their original host. Relocation requires explicit locator adaptation and a newly reviewed freeze; it cannot rewrite the original evidence or authorize paid execution.

Use the inventory's build_recipes with its recorded toolchain/Cargo.lock. Weights, Python runtimes and Cargo caches must be recreated separately from preserved dependency locks, manifests and upstream hashes. Rebuilt binary SHA can differ by host/path/toolchain; keep the original binary hash and label the rebuild. No credential or Keychain material is bundled.

Current evaluation reproduction commands must come from the exact final provenance records (reproduction_argv) and their retained source freeze; the packager never executes them. Pending records mean the final path/command is not yet supplied and final reproduction has not been validated. Run API-free reanalysis into a fresh output directory only. Deterministic doubles/scaled preparation remain diagnostics; extracting/building a bundle grants no model-call authorization.
