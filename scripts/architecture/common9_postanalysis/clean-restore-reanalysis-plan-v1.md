# Clean restore and API-free reanalysis: bounded handoff plan

2026-10-19. GPT-6 Astra Ultra, `/root/quant_adapter_astra`.

**Source-only planning verdict:** the accepted package can preserve and verify bytes at a new path. A plain new-path invocation of the accepted analysis/reporting tools is not supported. Conditional replay at the same logical paths inside an existing isolated runtime is the smallest option that preserves the accepted sources and evidence. No replay, archive, restore, runtime discovery or corpus read was performed for this plan.

## Exact source basis

| Source | SHA256 |
| --- | --- |
| `scripts/architecture/package_common9_evidence.py` | `1a29db2e389daec11d7a7b1d7b84029744c9215a1559893444eda81d43ce6d34` |
| Packager scope-extension-v4 checkpoint | `893a96bc9e8ad3d31505d11fda502400a88970990ea4f74ffb97e2e614f527d9` |
| Packager v4 independent review | `0036a259d5e6504812c4e6a9d6d16667d748942b717912eb8f394d3f0a92ad5f` |
| Accepted copied assembler, `input_assembly_preparation/source-v2/record_joint_report_inputs.py` | `6c97d90f3d736497ee9bd29ed219a28a9edd1eb7b5ba1d859701175aea0266a0` |
| Accepted copied adapter, `adapter_preparation/stage-03/report_adapter.py` | `c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0` |
| Joint formal frozen `benchmark/architecture/common_supplement6_20261019/analyze_joint.py` | `469a5c8a57ee1a3e6812b8c85182cc0e4b2fbd59b69fbca8eeb8591a0fcecb49` |

All listed hashes were checked. Packager checkpoint/review are under `results/architecture-evaluation/current/common9-poc-20261019/packaging-preparation-v2/scope-extension-v4/`. Accepted copied helpers are under `scripts/architecture/common9_postanalysis/`. The frozen analyzer is under `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/`. Only these named sources and their relevant static sections were inspected; the bundle and final provenance specification do not yet exist.

## What is and is not reproducible by the current commands

The packager's `verify` and `restore` validate ordered part/archive hashes and included member bytes, modes and supported links. Its six final roles must name included regular files with matching hashes/sizes. `reproduction_argv` is caller-declared text: the packager neither executes it nor certifies that it is API-free or usable after relocation. Six populated roles therefore do not establish reanalysis.

The relocation gap is concrete:

- Frozen `analyze_joint.py:27–30` derives each preserved trace path from the supplied root, then compares its resolved parent with the unchanged absolute `candidate_input_file` parent in the trace. Passing a different root fails this check. It also compares the combined row with the preserved per-run trace; changing one copy is not a solution.
- The assembler requires every `freeze.sources[].frozen_path` and oracle reference to equal the supplied formal root's exact paths. Its accepted freeze hash would change if those declarations were rewritten.
- The adapter requires the input manifest to reside within its declared absolute root, exact source/trace/result/oracle associations, and no symlink leaf or ancestor. Its reader explicitly says restore mapping is unsupported (`report_adapter.py:506`). A symlink from the original root to the restored directory does not solve these checks.

Do not edit frozen source, substitute paths inside saved traces/manifests, regenerate the approved freeze, or relabel restored evidence as a new measurement. The packager README's statement about locator adaptation/new freeze is a warning about relocation limits; this plan supplies no new measurement freeze and changes no accepted evidence.

**Binary requirement, checked explicitly:** frozen analysis compares stored trace/freeze binary hashes (`analyze_joint.py:23`); it does not open or execute `frozen/bin`. The exact accepted assembler likewise only copies the stored binary SHA into receipt/association (`record_joint_report_inputs.py:227,290`); it does not hash the executable. The adapter compares stored association/trace hashes (`report_adapter.py:534,872`). This source path therefore does not directly require the excluded candidate executable. The final plan has not yet been supplied, however: explicitly supplied quiescence `evidence_refs` or other final references must still be checked against the included inventory. If one requires an excluded file, full assembler replay remains pending even when direct frozen analysis is possible. Preserve that dependency honestly; a newly rebuilt binary is not a substitute for the recorded original binary hash.

## Honest commands available for the final handoff

After the bundle and trusted tool hash have been supplied, these existing commands verify and restore without API calls. Placeholders must be replaced with concrete paths; do not use Python `-O`, because the packager uses integrity assertions.

```sh
python3 -E -S -B /VERIFIED/TOOL/package_common9_evidence.py verify --bundle /BUNDLE
python3 -E -S -B /VERIFIED/TOOL/package_common9_evidence.py restore --bundle /BUNDLE --out /NEW_EMPTY_RESTORE
```

The tool must match the packager hash above and the final bundle's declared source. Restore verifies internally and refuses a nonempty destination. A fresh clone containing files is not an empty restore target: restore into a separate absent/empty directory. Retain the concrete command, output and final bundle/inventory hashes. Label this evidence **verified restoration**, without calling it an analysis rerun.

## Conditional API-free replay after quiescence

1. Complete original-root final analysis and reporting through the already accepted pipeline after independent quiescence/root GO. Bind the actual six role files, generation mapping, source hashes, immutable snapshots, analysis/adapter inputs and resulting payloads in the final handoff. Keep the populations, failures and prior generations separate.

2. If a suitable isolated filesystem runtime is already available, map **only the newly restored copy** read-only at the original logical repository path inside that runtime. Map a separate fresh host output directory read/write to a new replay subtree under the logical formal root's `postanalysis/`. The source of the read-only mapping must be `/NEW_EMPTY_RESTORE`, never the original host repository. There must be no host mount, symlink or write targeting the original repository. Preserve input hashes before/after replay.

3. Record this explicit mapping from package-relative member and original logical path to restored physical path, with bundle/member hashes; record the isolated runtime/Python version and chosen output mapping. This is replay location metadata, not a replacement freeze. Use an available compatible Python runtime with the standard library and required Unix `fcntl` support. No model weights, provider credentials or model call is needed for this analysis path; candidate-executable and final explicit-reference limits are stated above. Disable external network access in the isolated replay if the runtime supports it. Do not build new virtualization infrastructure merely to complete this handoff.

4. Inside that mapping, the unchanged accepted assembler can use the preserved original logical plan/root references and write a fresh assembly; the unchanged adapter can consume its newly recorded input hash and write a fresh report. The packaged executable helper files must first match the accepted hashes above and be co-located as their imports require. The accepted assembler already records child `-E -S -B` and unchanged analyzer `--root`/`--output` arguments; no `--offline` is appropriate for the preserved FORMAL traces. Use the actual final plan path/hash, not an invented plan or an old lightweight evaluation.

   The existing command shapes, conditional on the mapping and concrete final references, are:

   ```sh
   python3 -E -S -B /LOGICAL_REPO/scripts/architecture/common9_postanalysis/record_joint_report_inputs.py --plan /LOGICAL_FINAL_PLAN --plan-sha256 FINAL_PLAN_SHA256 --output /LOGICAL_FORMAL/postanalysis/REPLAY/assembly --execute-authorized-frozen-analysis
   python3 -E -S -B /LOGICAL_REPO/scripts/architecture/common9_postanalysis/report_adapter.py --input-manifest /LOGICAL_FORMAL/postanalysis/REPLAY/assembly/adapter-input.json --input-sha256 NEW_INPUT_SHA256 --output /LOGICAL_FORMAL/postanalysis/REPLAY/report
   ```

   `REPLAY` is the fresh mapped output parent; `assembly` and `report` must not already exist. Record concrete commands and results only after actual validation. These templates are not a claim that an isolated runtime is present or that relocated replay has passed.

5. Compare replayed quantitative/diagnostic payloads against the preserved final analysis and reporting values under the same frozen inputs. Keep execution timestamps, command receipts, fresh paths and their resulting hashes distinct. Do not demand equal bytes for newly dated receipts/manifests or normalize the original evidence to obtain equality. Preserve any mismatch or failed replay and state its scope. Respect the existing file and cumulative read caps; do not silently raise them.

If this isolated mapping is unavailable or impractical, deliver the verified raw/source restoration and independently recorded original-root final analysis, and explicitly state **relocated API-free reanalysis unverified**. Do not publish a bare `analyze_joint.py --root NEW_ROOT` as a working command. A future portable logical-to-physical reader would need separate authorization and review; no locator helper is requested or implemented here.

The bundle excludes runtimes, weights and compiled binaries by design. Candidate rebuild recipes remain useful separate documentation; neither an identical binary across hosts nor a paid experiment replay is required to recompute analysis from preserved raw observations. Analysis replay also does not reproduce provider outputs, original latency, RSS or the paid experiment itself.

## Boundary

This is a bounded handoff plan, not another audit campaign. No current corpus/ledger was read, no directory scan or test was run, and no analyzer, adapter, evaluator, model, parser, build, archive, restore, container or network operation was executed. Accepted source-v2 closure remains valid for its original-root scope. The only new artifact is this plan.
