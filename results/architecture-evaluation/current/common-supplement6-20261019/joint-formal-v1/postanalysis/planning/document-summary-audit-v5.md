# Incremental document-summary audit v5 — S26 only

**Result: no concrete discrepancy found within this limited scope.** Independent reviewer: GPT-6 Astra Ultra (`/root/quant_adapter_astra`), 2026-10-20. This checks summary fidelity to the completed semantic review; it does not rejudge raw model content, certify quantitative endpoints, or certify the campaign.

## Scope and exact inputs

- Presentation document: `docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md`, §11.2, only the new subsection **“S26: 읽기 동의, 외부 전달 동의와 발송 승인을 구분”**, lines 743–764 at inspection. Subsequent prediction rules and previously audited cases are outside this verdict.
- Accepted source: `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON9-S26-r1-v1.md`, SHA-256 `d6c9434fc81845a51126e144f570fa545b01cb692291ac4dfa6aa32580b24a4b`. Read conclusions, dimension summaries, permission/action/currentness explanations, goal-member table and preservation statements (lines 1–94 and 158–222). The linked JSON, raw traces, model files and underlying source files were not opened for this audit.
- Section hashes below are SHA-256 over the exact UTF-8 bytes emitted by `sed -n '<range>p'`, including line-ending newlines. They bind the reviewed prose without relying on a whole-document hash while other work continues.

| Presentation scope | Lines | SHA-256 |
| --- | --- | --- |
| Entire S26 subsection | 743–764 | `6fb1f1ef6ff6c4ca3f0f474f9bd6419928c01514ea373f6103f9e98b42dcafb5` |
| Coverage paragraph | 745 | `486201e50e38fc22e8252b43c1ccda00b510ef79b0986d96769259cbe6fdae5d` |
| Eight-position response table | 747–756 | `b72e39475dec168e9a3ac2e1f3ec6829a0fc37df5efe08453c78147febe26198` |
| Display/member-relation scope | 758 | `f033e6ba8561b767a11a8a227daf78b0560c9d459e245aa08f34464ca8a2e2fb` |
| READ/SHARE and discarded handoff | 760 | `6ee1a094bc40de60be98ff8ac50afd72c485da41d60dee3c2b279927963bedef` |
| Acknowledgement/current referent and U05 | 762 | `0353017e6c8a36057097ab5f5bb6cc7643dea7b0eb281265da2a2cde65c462b6` |
| Correct Task mapping versus failed effect | 764 | `4926a186b40cf02bcbd720b0e3381388c8f1d20ff7ed8d19a81c9509f3ecedc4` |

## Checks and conclusion

1. **Counting:** the eight rows each retain seven planned obligations. They total 56 = 35 scoped fulfillments + 12 failures + 9 missing bodies, matching the integrated review. The prose correctly distinguishes 47 obligations with a body from 52 actual display events; five extra HANDLING/WAIT displays do not enlarge the planned denominator. Goal-member relations remain 21 = 8 fulfilled + 5 failed + 8 unreviewed, not additional response successes.
2. **Coverage:** 245 previously reported representative rows + 56 = 301; 301×5 = 1,505 planned representative rows. The stated supplemental 84×3 = 252 is consistent. Goal-relation coverage 161×5 = 805 and 56×3 = 168 is also consistent with the fixed plan. Earlier subtotal provenance is carried forward from prior audits, not re-audited here. The text expressly limits these counts to first repetition and avoids a pooled success rate across different predicates.
3. **Permission:** the five named premature READ configurations and CAAA's separately adopted unauthorized SHARE match the source review. The subsequent pre-ingress discard is correctly separated from actual export. The three named U03 guard-blocked paths and the synthetic-data qualification are faithful; no actual personal-file leak or successful source-permission revocation is inferred.
4. **Body versus relation/currentness:** AAAB's U02 scope acknowledgement remains a scoped content success, explicitly not a new summary. The registered summary ID does not establish a correct later referent. U05 has three wrong old-cancellation/refusal associations and four missing adoptions/bodies; no successful clarification is asserted. The document preserves authority/currentness/effects as distinct from the member-relation counts.
5. **Action and endpoint:** all seven correct TA/TB/question/disposition mappings are separated from the unsupported `approve`/`APPROVE` action encoding and failed TA sending effect. S26's implemented `send` operation is properly distinguished from S20's unsupported `make_pdf` capability despite that action being listed as allowed. Seven truthful failure relays are not seven sends. The full three-goal success and physical file/mail effects remain unestablished; frozen QA01/QA03 times and denominators are preserved.

No unsupported structural winner or causal claim appears in this subsection. No correction is requested. Earlier v1–v4 audit findings/closures remain unchanged; this verdict does not broaden them. No parser, evaluator, analyzer, model, build, test, API or controller operation was executed. Only this new audit Markdown was written; the presentation, frozen code and measurement evidence were not edited.
