# §11.2 semantic-summary documentation audit v1

2026-10-19. Independent reviewer: GPT-6 Astra Ultra, `/root/quant_adapter_astra`.

**Result: one localized wording finding (DS-01/P3). Coverage arithmetic and the seven body-result summaries otherwise match their cited short reviews.** This audits documentation fidelity only; it does not rejudge model outputs or certify quantitative results.

## Exact inputs

Repository root: `/Users/changbae/Workspace/via-sw-architecture`.

Presentation input: `docs/architecture/12-decisions/decision-packages/seven-problem-review/17-presentation-results.md`, **§11.2 only**, lines645–699 at review time. SHA256 of the exact section text, obtained with `sed -n '645,$p'` including its existing newlines: `570587a4e5aa8456e0e85cfc87fdb96108d893f5c8b1ae581b5400e3a124e1a4`. This is a section hash, not a claim to have read or audited the rest of the document.

All seven review paths below are relative to the exact directory `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/` under that repository root. Their entire short Markdown files were read; total65,559 bytes.

| Input file | SHA256 |
| --- | --- |
| `COMMON9-S02-r1-v1.md` | `876656a9d07562c2b3050897cac91da98c9128035ae51783adc7245d8fd62f11` |
| `COMMON_SUPPLEMENT6-S80-r1-v1.md` | `749b9813736afb2c676d3f4df927ba5318a55cc3b50170bbf6a1808a8431006e` |
| `COMMON9-S08-r1-v1.md` | `42061252147c58bceb337c0b24cf2021023daf0435c2a25cd710bc0226a3cafd` |
| `COMMON_SUPPLEMENT6-S83-r1-v1.md` | `eb52e66b1b1ecca7a80a1f36e1e2a2a229db7fd41d8114a222f37ef0fedcf1b8` |
| `COMMON9-S11-r1-v1.md` | `db58954129959511f189fdef21e0bc9d2a80c0891ecc5ecd4986d5ebbd6f6f77` |
| `COMMON_SUPPLEMENT6-S81-r1-v1.md` | `933287daa62837850709f7e7268557835d2f64f6b0ee02d55be036c79f8cbeff` |
| `COMMON9-S12-r1-v1.md` | `0fd50624dcd37f24928ed043b390aa388885833895fd2bc735e666f90dab930b` |

The existing `results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/planning/astra-final-report-plan-v1.md`, SHA256 `bb23355ba50776e1096ccb009e1fd8d170d46a9d55abff8fe8eae273b9234e41`, was consulted only for its fixed denominator lines16/18/68. The separate earlier manual AAAA/S02 reference and linked JSON/raw artifacts were not reopened; their original independent correctness is outside this audit.

## DS-01 / P3: keep authored U02 time distinct from actual provision

Presentation line665 says: “여섯 구성은 실제 첫 최종 응답 뒤2초에 다음 요청을 제공했고”. The cited S81 review §3 establishes an actual U01-final anchor for six configurations and an **authored U02 source time** two seconds after that anchor. It explicitly distinguishes that source time from actual provision. For example, its BAAA record separately gives fallback anchor, authored source and later actual provision. The summary's verb “제공했고” turns the two-second source rule into an exact actual-provision observation that the short review does not establish for those six rows.

Suggested narrow replacement: “여섯 구성은 실제 첫 최종 응답을 기준 시각으로 삼았고, 다음 요청의 authored source 시각은 그2초 뒤였다. 실제 제공 시각은 별도 관측값이다.” Retain the existing BAAA30-second fallback and unrealized sequential-condition statements. No count, body judgment, primary timing or frozen rule needs changing. The presentation was not edited by this audit.

## Checks with no additional finding

- All seven table rows match their cited full-body PASS/FAIL/missing counts. Common subtotal is112 =90 PASS +2 FAIL +20 missing; supplement subtotal is42 =23 PASS +5 FAIL +14 missing. The scopes represent28 common and21 supplement episode records, all at repetition1, rather than154 independent users or154 observed bodies.
- The declared full response obligations1505/252 and goal/relation obligations805/168 match the existing report plan. The partial table keeps populations separate and does not imply campaign completion or extrapolate to unreviewed cases/repetitions.
- S08 distinguishes correct core target content from two literal Task inconsistencies,14 missing bodies and seven unmet correction relations. S83 keeps14 unreviewed bodies,14 missing delivery outcomes and seven unmet final goals separate; unadopted proposal errors are not presented as displayed wrong answers.
- S11 retains29 observed-body PASS and six missing responses. Its source-identity/revision support failure is not reclassified as false literal prose. The14 actual BG notifications are not claimed as user-goal completion. S12 distinguishes correct original/current source use from an unexercised in-flight stale-result race; its wording does not require all seven to display an explicit comparison sentence.
- S80 and S81 preserve the same full-body criterion: omission of the literal Monday12 detail alone is distinct from omission of the required support/schedule next step. Goal association and recorded completion are not substituted for full-body correctness or endpoint certification.
- S80's three ABAA cross-request host-call overlaps are supported by its short review. The presentation retains the network/provider-compute and causal-superiority limits. No new structural winner or general performance claim was found in the seven summaries. S02 preserves actual-request versus added-oracle-detail distinctions and nonapplicable approval/handoff status.

## Limits

This is a limited fidelity audit of existing review conclusions, not independent semantic evidence validation. No large JSON, raw trace, ledger, corpus scan, evaluator/analyzer, parser, model, build, test, package or network operation was used. Frozen sources and criteria were untouched. The only new output is this audit file. DS-01 is a documentation precision issue and does not identify an active-experiment instrumentation defect or justify a rerun.
