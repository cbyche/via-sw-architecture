# Document summary audit v4: S19/S20 additions

Date: 2026-10-20. Reviewer: independent GPT-6 Astra Ultra, `/root/quant_adapter_astra`.

**Result: no findings within the bounded documentation scope.** The S19 and S20 summaries, updated representative subtotal and separate coverage additions are faithful to the named completed reviews. No correction to document 17 is requested by this audit.

## Scope and evidence

Reviewed document: `docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md`, section 11.2. The content scope is the new S19 paragraph, partial-table scope and counts, the S18 coverage sentence only, and the new S20 prose/category table. S26 is excluded. Earlier S18/S85 semantic conclusions remain under audit v3; this audit does not reopen them or the other existing table rows.

`J` means `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1`.

| Named comparison input | SHA-256 | Read scope |
| --- | --- | --- |
| `J/postanalysis/semantic-reviews/COMMON9-S19-r1-v1.md` | `ca3bcad0f5efab49c6ab8a926ea832d0af91ecc5aa131b2a8aaa05be30215f5b` | Entire short Markdown review, lines 1–164 |
| `J/postanalysis/semantic-reviews/COMMON9-S20-r1-v1.md` | `0c5222b871f09969fc42917083258b7c30a3d1ca6b63c50dfdf374c75195b162` | Conclusion, all response and goal-relation rows, configuration explanations, method and limits, lines 1–233; heading locator only for later provenance sections |
| `J/postanalysis/planning/S20-method-qualification-review-v1.md` | `3ce6a5d999a076ab4c5db00128860f97e831898d5154a9d9da9e963e1d45c325` | Entire 49-line method review |

The following hashes identify the exact presentation text audited. Each digest is SHA-256 of the original UTF-8 bytes returned by `sed -n` for the stated inclusive lines, including their existing terminal newlines, with no normalization. Line numbers describe the snapshot read for this audit; the hashes identify the content if later additions move it.

| Presentation segment | Lines | SHA-256 |
| --- | --- | --- |
| S19 summary paragraph | 681 | `574a8ffd380607e1003e66815022871b6ad421f90e0ffe9fe166e310945d55d4` |
| Partial-table scope paragraph | 683 | `396668e61f797148bcb8cc0093b689822eef7e6df5442e7619b8daaaea8eb39c` |
| Partial table, including S19 and both subtotals | 685–700 | `72d865125c4cbfe46413bb71915c202066f9e358150a716d3220068d6ee01549` |
| S18 coverage sentence only | 704 | `43d86673db24e7b70cdb6fa04bb02bd4752f8f99a2102a3c9d1c1ff2d1a0c284` |
| S20 coverage and display-reference paragraph | 724 | `4615d57e625ec323a8498cd011b3597ed5a843f5fd3f0553e0f4a1ab87a6fb2b` |
| S20 response-category table | 726–735 | `1bc61cd2c02062affc4e63a2ae50ef3daecd37fcf31b9d1498b842834190c4f2` |
| S20 actual questions, adopted routes and goal relations | 737 | `52401bf2c80d31337fbe61ebbcb9735b141105d7faa507b4a7b65fa737343610` |
| S20 receiver mismatch and physical-effect limits | 739 | `14a01f0920fc1aaca5f62de49c3b007010f8d6f230ec6c6a058dc3c86eb57f21` |
| S20 late result, stop-condition and QA03 limits | 741 | `966345b7c44eec9e75e600d1171209654c48e105143b60c8fe9b8eff8b989c0a` |

## Checks and conclusions

### S19 meaning and scope

The presentation correctly distinguishes seven short TCP/UDP explanations, fourteen relays of authored BG progress/result facts, and seven user-goal relationships. The 21 unique displays do not become 21 independent user goals or fourteen physically verified file completions. The source review's Voice OFF limit, unresolved applicability of the common source instruction to general knowledge, and caveat on simplified TCP/UDP speed wording are retained. No new citation, length, numeric performance or physical-product requirement is added.

### Arithmetic and partial coverage

- S19 contributes 21 scoped response PASS rows: the previous representative subtotal of 140 becomes **161 = 139 PASS + 2 FAIL + 20 missing**. The table's representative rows also sum directly to those values.
- The supplemental subtotal remains separately reported as **84 = 38 PASS + 8 FAIL + 31 missing + 7 superseded purposes**. It is not pooled with the representative population.
- Adding S18's 49 separately reviewed response obligations gives **161 + 49 = 210**. Adding S20's 35 gives **210 + 35 = 245**. These are response-review coverage counts, not episode counts or semantic-success totals.
- S20's categories sum to **14 + 4 + 3 + 5 + 2 + 4 + 3 = 35**. Eight absent bodies leave **27 actual displays**. Its 32 actual inputs and four shared result-to-intention references agree with the completed review; those references do not inflate the display count.

The explicit separation of S18 and S20 from the main PASS subtotal prevents their heterogeneous judgments from being presented as a single success rate. No earlier row is reclassified here.

### S20 interpretation and method

The summary follows the completed review on all material distinctions:

- The actual TB question is an approval question; its mismatching expectation label does not change the user's intent or authorize sending.
- Four ambiguous U01 approvals and four later new-Task delegation paths are distinct observed failures. A raw model proposal naming TA is not the same as adoption into actual `TASK-5`; a side control in a DELEGATE plan is not an executed CONTROL operation.
- The seven goal-relation judgments remain four FAIL and three UNREVIEWED. AABA's narrower correct U02 Task/question correspondence is preserved without becoming full-scenario success.
- AABA's advertised `make_pdf` operation reaches the shared receiver but is unsupported. The presentation correctly calls this an implementation mismatch at a shared interface and does not explain it solely as a model error.
- Faithful relay, unsupported effect, unsupported PDF claims, unrelated result text, raw `HOST_COMPLETE`, and physical product verification remain distinct. The summary says real external sending/file creation is unverified; it does not assert either a verified effect or proven physical nonexistence.
- CAAA's late actual result is preserved alongside the earlier missing-anchor flag, keeping the final absent-result count at three.
- The independent method conclusion is stated within the inspected frozen-rule scope. It supplies no additional mandatory technical-stop trigger and does not redefine QA03 as PDF completion time, delete observations, or change planned denominators, failures or costs.

No unsupported structural superiority, DP winner, completed product, full-campaign certification or endpoint redefinition was found in these additions.

## Limits and preservation

This is a prose-to-completed-review consistency audit. It does not independently rejudge model output, inspect raw evidence or certify quantitative eligibility. Only the named Markdown inputs, bounded presentation slices and their hashes were used. A bounded section locator also returned surrounding existing table/headings; those were not used to expand the semantic audit scope.

No raw trace, JSON evidence bundle, corpus, model, image parser, evaluator, analyzer, test, build or network/API operation was run or read for this audit. No frozen code, presentation text, previous audit, controller state or next review cursor was modified. The only new file is this immutable v4 audit; earlier findings and closures remain preserved.
