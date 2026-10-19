# S20/r1 method qualification review v1

Date: 2026-10-19. Reviewer: independent Astra Ultra method review.

## Decision and scope

**No additional existing mandatory technical-stop trigger is established by this finding.** The observed `make_pdf` failure is a shared candidate/synthetic-receiver capability mismatch with a material interpretation limit. It must not be described as only a model error or as successful PDF completion. This is a bounded source-and-reported-evidence judgment, not technical certification of the campaign, permission to override its guards, or a new stop policy.

The paid controller remains under the root agent's authority. This review neither launched nor stopped it, inspected live stop markers, nor modified frozen sources, criteria, evidence, or the presentation document.

## Inputs and exact references

Paths below are repository-relative. `J` means `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1`; `F` means `J/frozen/source`. SHA-256 values for the four named source files were checked during this review. Only the listed source slices and relevant contract search matches were read; file hashing is not a runtime check.

| Frozen source | Read scope | SHA-256 |
| --- | --- | --- |
| `F/benchmark/architecture/common9_poc_20261019/harness.py` | lines 1–36 and 285–315 | `d30750622e97978f769da8366a8c12510e91cbd482ead32d1111ed9cc7e079d9` |
| `F/prototypes/candidates/common9_poc_20261019/src/runtime.rs` | lines 448–487 and 858–885 | `d96f61406119d870bf65bb9045a8fe2bb8eb1aa68ae8319a65a09845a4437402` |
| `F/docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md` | relevant eligibility, meaning, timing and preservation matches; sections 4.1–4.3, 5 and 8, lines 58–82, 88–111 and 163–183 | `7b3db7b8f358ee6cd441ca37ef13f9b83d1f697322729a0f7bde802bbdc162e7` |
| `F/benchmark/architecture/common_supplement6_20261019/joint_campaign.py` | lines 153–194 | `9314f4f36f58ae2254e0f2d0cb33d4fe5debba5c43f450b696154068745c8d50` |

The following evidence identifiers and facts were supplied by the root and the S20 semantic reviewer (`/root/method_astra_review`). Their files were **not opened or independently rehashed here**:

- Extracted bundle: `J/postanalysis/blocks/COMMON9-S20-r1-v3-r2/evidence.json`, 2,751,956 bytes, SHA-256 `a9a8b6a44c676fed81da2a4feb80a3c6429cdd0957f84ddb3085943e7b8490cc`; AABA is evidence list index 4.
- Bound trace identity: `AABA-S20-r1-540754`, 4,924,590 bytes, SHA-256 `cac5141a7590200c4653f7c66c499b8b13629d3f39b21634fd520c7263000ba4`. Exact reported selectors: `/events/81` (`event-93`, U02 semantic adoption), `/events/83` (`event-97`, handling), `/events/88` (`event-102`, `CONTROL_INGRESS`), `/events/97` (`event-113`, unsupported TA control plus TB non-send result), and `/events/106` (`event-122`, U02 `HOST_COMPLETE`).
- Copied AABA terminal: `J/postanalysis/block-indexes/COMMON9-S20-r1-v3-r2/terminal-4.jsonl`, 732 bytes, SHA-256 `38b0ec8747c785c8e86fca38a4d9ed317dae4d7d7e58053ba2e7f9619e6935b8`. Reviewer reports return code 0, `technical_failure=false`, `budget_stop=false`, with an S20-U01 `RESPONSE_ANCHOR_TIMEOUT`. The reviewer reports the same terminal flags for all seven configurations; this review does not extend that report into independent terminal certification.

## Existing contract and implementation judgment

1. **The mismatch crosses an implementation boundary.** The reported actual TA state permits `make_pdf`. Runtime lines 858–885 accept string action/value fields with the actual Task, optional matching question and one of `APPROVE`, `DENY`, or `UPDATE`. Lines 448–487 commit and emit control ingress and an accepted receipt. Those slices do not check the receiver's action vocabulary. In the common harness, `DENY` is supported; approved send actions are supported; `revise_report` and `set_format` are supported for update/approval. `make_pdf/APPROVE` falls into the ordinary unsupported-result branch. Admission and receipt therefore do not establish the requested effect.

2. **Unsupported action text is not an existing technical-failure classification.** The inspected branch returns a result describing the unsupported state; it neither raises a host failure nor claims a PDF was produced. The technical classifier near harness line 295 counts recorded failures except `RESPONSE_ANCHOR_TIMEOUT` and `MISSING_INGRESS_OR_AGENT_ANCHOR`, and separately considers the technical abort reason. The campaign's inspected guards stop on existing hold/stop marker files, budget conditions, nonzero process return, or the recorded technical/budget flags; they also reject unauthorized freeze/job identity and automatic continuation after a previous flagged terminal. No new occurrence of one of those conditions is established here.

3. **The failure is not silently replaced by a successful external effect.** Contract section 4.2 requires actual ingress/dependencies and forbids replacing failed ingress with a success result. Here the reported external result explicitly says the TA control is unsupported and TB was not sent. This path alone does not establish a fabricated successful-PDF result. It does establish that this frozen synthetic receiver cannot validate the intended PDF effect through the accepted action. Contract section 4.3 already limits the PoC's external-product scope; that general limit must not hide this specific missing capability.

4. **No new measurement-identity breach is established.** The four checked frozen source hashes match the supplied identities. The facts describe behavior of that frozen implementation, not demonstrated source replacement, a different job, an oracle supplied as a candidate answer, or altered timing criteria. This statement is limited to the inspected material and supplied evidence; the normal provenance/lifecycle certification still applies later.

## Required interpretation of the preserved results

- **QA03:** Section 5 defines the endpoint as final-revision host completion and expressly separates it from body correctness and functional fulfillment. Preserve an actual host-completion timing under the unchanged eligibility, goal/revision, causal-path and certification rules. Do not relabel the AABA U02 host completion as elapsed time to a completed PDF. This review does not independently certify that event's final quantitative inclusion. Missing endpoints remain missing, and no repair estimate or invented terminal time is substituted.
- **Full goal and safety:** AABA's reported U02 control correctly distinguishes TA approval from TB denial, but the requested PDF effect is unfulfilled. The TB denial result is separate evidence, not proof of TA completion. Earlier ambiguity handling remains independently assessable. The root-reported four other configurations' premature TB approval and later delegation to a wrong new Task are separate defects; the missing PDF operation does not excuse them.
- **DP conclusions:** Distinguish interpretation/routing evidence, control admission, the shared receiver limitation and realized effects. The shared limitation constrains an end-to-end fulfillment comparison; it does not establish a structural winner, a configuration-specific PDF incapability, or the counterfactual outcomes of a repaired receiver. Do not pool these causes into a model-only error explanation.
- **Fixed evidence:** Retain every affected episode, failure, callback, host endpoint and cost under the original populations and formulas. Do not remove this case, correctness-filter the timing metric, change the denominator, or replace frozen failures with repaired results. Any later implementation repair would require the existing section 8 preservation/new-freeze procedure; this review requests no repair or replay during the current campaign.

## Disposition and limits

Record and disclose the capability mismatch as a scoped method limitation alongside the semantic diagnostic. The finding alone supplies no basis to demand a new technical stop under the inspected existing rules. Root retains authority over any manual hold and must honor actual existing guards if independently triggered. No live-marker absence, whole-run validity, full-goal success, causal structural advantage, or final numerical result is certified here.

Executed work: bounded source reads, four source hashes, and this new review file only. No raw-trace or evidence-bundle read, corpus scan, evaluator/analyzer/parser/model invocation, test, build, API call, quantitative recomputation, or frozen/document edit was performed.
