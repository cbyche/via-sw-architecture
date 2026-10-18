# Independent final audit criteria

`../audit_campaign.py` is a read-only audit of retained evidence and frozen inputs. It accesses no credentials, sends no API requests and changes no candidates, fixtures, oracles or raw records. Only derived audit JSON is written under this directory. Timestamped snapshots are preserved; `latest.json` points to the most recent result.

Run preflight during the campaign:

```sh
python3 results/architecture-evaluation/current/four-dp-qwen37-20261018/audit_campaign.py
```

After root confirms completion of 120 BASIC rows and the final selected supplement plan:

```sh
python3 results/architecture-evaluation/current/four-dp-qwen37-20261018/audit_campaign.py --final
```

The audit enumerates the exact 10 alternatives × four episodes × three repetitions (120 keys), rejects duplicates/unexpected keys, verifies every raw input/trace and the original84/v2-36 split, source and model/usage/price settings, per-run freeze and binary hashes, UUID linkage to the one shared $5 ledger, and presence-only secret/header patterns without exposing values. It independently recomputes derived rows and checks fixed denominators, observed null outputs, known retained referent failures and missing RSS observations. The selected plan supplies exact non-BASIC keys; expected final scope is 12 TACTIC rows and three B4 DIAGNOSTIC rows, separated from BASIC. Prepared conditional tactics are never inferred as executed.

`PENDING` is permitted only in preflight for incomplete populations, current unmerged/in-flight budget entries and a not-yet-final selected tactic plan. `--final` converts those conditions to failures. Missing or unavailable evidence is recorded rather than inferred as a pass. Runtime errors and semantic/oracle mismatches remain distinct.

The original freeze independently pins source but has no binary-SHA field. The audit checks recorded original-run binary equality and the current original binary separately; it cannot claim an independent freeze-time binary pin. DP05 v2 and tactic freezes do pin their binaries. QA06 remains sampled simultaneous Python+Rust RSS, with short-lived missing simultaneous observations explicitly incomplete; shared pages, UI and cloud weights are outside its complete physical-memory claim. Standard-price provider usage is not a provider invoice, and DISPLAY_PROXY is not physical display/audio observation.

Final offline HTML usability and selected tactic before/after interpretation require the separate browser/data review after execution. This script does not turn a structurally valid evidence chain into a product-quality or measured-superiority claim.

The final audit also includes the separately reported, previously frozen semantic-content conditions. A valid evidence chain does not make their failures pass. DP01 E3's twelve applicable displayed answers (nine BASIC, three A2) omit all 36 required fact-marker instances. QA02 identity/version success and a `GOAL_COMPLETE` callback therefore do not establish full summary-content completion. This is an explicit functional-content limitation in audit conclusions, not a pending population or a changed primary QA score.
