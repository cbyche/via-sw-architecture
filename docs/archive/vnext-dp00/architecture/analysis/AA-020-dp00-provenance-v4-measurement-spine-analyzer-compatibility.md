# AA-020 — DP-00 Provenance-v4 Measurement Spine Analyzer Compatibility

## Status and scope

**PROSPECTIVE REMEDIATION APPROVED.** Session 6.5 calibration
`dp00-calibration-20260911T050356Z-28aedd0`, frozen at
`28aedd0d42acc74bb5b67a6b95fb6c7d8a268c78`, remains permanently:

`REJECTED — PROVENANCE-V4 ANALYZER QUALIFICATION DEFECT`

Its raw evidence contained 768/768 complete pairs, 768/768 normalized semantic
signature matches, 768/768 shared-provenance matches, and 768/768 QA-04 semantic
matches. `dp00-analysis-v7` nevertheless produced 592 QA-02 pair mismatches and
zero acceptance-qualified pairs. This record diagnoses and fixes analyzer
compatibility only. It does not reinterpret that rejected run, create acceptance
results from it, change an architecture, or run another calibration.

## Root cause

The closed-world support branch in
`benchmark/analysis/dp00_analysis/qa02.py::_closed_world_conditions` treated
`dp00-pilot-provenance-v3` as Measurement Spine-capable in CAPTURE and MINIMAL,
then treated every other provenance version as CAPTURE-only. Session 6.4R added
`dp00-pilot-provenance-v4` solely to carry counterbalanced calibration protocol
identity and measured-execution ordinal; it retained v3 Measurement Spine
semantics. The analyzer's exact-v3 conditional was not extended with that
contract. Consequently every v4 MINIMAL absence condition became open-world,
which was an analyzer compatibility defect rather than architecture/runtime
behavior.

## Decision

Analysis version advances from `dp00-analysis-v7` to `dp00-analysis-v8`.
Measurement Spine support is centralized in
`supports_measurement_spine(provenance_version, instrumentation_mode)` with
these explicit outcomes:

| Provenance | CAPTURE | MINIMAL |
| --- | :---: | :---: |
| `dp00-pilot-provenance-v2` | supported, legacy behavior | unsupported |
| `dp00-pilot-provenance-v3` | supported | supported |
| `dp00-pilot-provenance-v4` | supported | supported |
| unknown | unsupported and rejected by strict provenance validation | unsupported and rejected |

For v3/v4, the declared `measurement_spine_event_count` must still equal the
independently counted non-model-diagnostic canonical stream. Terminal uniqueness,
finality, event-count equality, boundary declaration, and episode integrity
requirements are unchanged. Support is not inferred from a filename, directory,
peer CAPTURE run, or guessed event count.

QA-02 semantics, QA-04 semantics, `canonical-event-v3`, P12 semantics,
correctness qualification, missing-evidence handling, and the existing 88
architecture nonconformances are unchanged. A/B/C/D and all scenario/corpus
assets are unchanged.

`dp00-calibration-acceptance-v1` is unchanged, including M, the 5% margin,
early/late split, Theil–Sen estimator, bootstrap method/seed, outlier policy,
and percentile estimator. `dp00-calibration-protocol-v1` and every schedule,
warm-up, balance, inversion, adjacency, and rotation rule are unchanged.

## Paired diagnostic schema

v8 replaces the ambiguous combined diagnostic with independently named fields:

```text
incomplete_pairs
duplicate_mode_pairs
semantic_mismatch_pairs
qa02_mismatch_pairs
qa04_mismatch_pairs
provenance_mismatch_pairs
rejected_pairs
rejection_reasons_by_pair
```

Each complete pair also records sorted `rejection_reasons`. A multi-cause pair
retains every applicable reason. `semantic_mismatch_pairs` now means only an
actual normalized semantic-signature mismatch. Qualification logic itself is
unchanged.

## Verification boundary

Synthetic v4 QA-01 CAPTURE/MINIMAL evidence is loaded through the real loader,
canonical reconstruction, QA-02, QA-04, and paired analyzer. The MINIMAL member
omits model diagnostic canonical events while retaining a declared Measurement
Spine. A separate missing-terminal case proves that v4 support does not convert
genuinely incomplete evidence into closed-world evidence.

P12 integration copies only the committed cycle-0 P12-A CAPTURE/MINIMAL pair
from the rejected evidence into a temporary test directory. It verifies S1 then
S2 commits, common parent, distinct children, two required effects, final S2
boundary, distinct first S1 boundary, execution after the route-plan barrier,
equal exact QA-02, equal QA-04, and pair qualification. The committed rejected
artifact is read-only, the entire run is never re-derived, and no Session 6.5
acceptance result is generated or promoted.

The remediation commit is only a candidate frozen source for a future session.
A future calibration must start from that clean commit, use a new calibration
ID, and execute W0/W1 plus all 16 measured cycles from the beginning.
