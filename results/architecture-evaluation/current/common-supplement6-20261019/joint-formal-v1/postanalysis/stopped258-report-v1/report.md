# Joint441 quantitative evidence report

**COMPLETE_INPUT_REPORT**

Reporting-only projection of bound frozen results. Input coverage is separate from semantic correctness. All24 prediction judgments remain UNREVIEWED.

## COMMON9

| Configuration | QA01 ms (observed/planned) | QA02 correct/planned; complete accuracy | QA03 ms (observed/planned) | QA04 modules | QA05 complete bundle USD; known spent | QA06 MB; status |
| --- | --- | --- | --- | --- | --- | --- |
| AAAA | 8285.436 (92/215) | 8/75; — | 12579.596 (45/115) | 1.833 (11/6) | —; 0.4768216000000000105 | 59.703; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| BAAA | 8265.438 (91/215) | 9/75; — | 12955.852 (46/115) | 1.833 (11/6) | —; 0.4809976000000000169 | 61.325; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| CAAA | 9412.787 (90/215) | 9/75; — | 12662.126 (47/115) | 1.833 (11/6) | —; 1.120037200000000039 | 64.586; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| ABAA | 7962.544 (85/215) | 8/75; — | 11406.765 (41/115) | 1.833 (11/6) | —; 0.4979532000000000197 | 63.242; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| AABA | 8952.418 (86/215) | 9/75; — | 11828.907 (44/115) | 1.833 (11/6) | —; 0.4842316000000000118 | 5918.360; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| AAAB | 8537.903 (85/215) | 8/75; — | 12661.848 (45/115) | 1.833 (11/6) | —; 1.373652000000000040 | 65.798; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| AAAC | 7779.404 (85/215) | 7/75; — | 11030.552 (44/115) | 1.833 (11/6) | —; 0.4611796000000000084 | 62.669; OBSERVED_LOWER_BOUND_DIAGNOSTIC |

### Coverage and failures

| Configuration | Episode states | QA01 partition | QA02 partition | QA03 partition | RSS states |
| --- | --- | --- | --- | --- | --- |
| AAAA | {"CLOSED_CERTIFIED": 22, "UNEXECUTED_VERIFIED": 23} | {"CERTIFIED_NOT_OBSERVED": 10, "CERTIFIED_OBSERVED": 92, "UNEXECUTED_VERIFIED": 113} | {"ADOPTED_INCORRECT": 22, "KNOWN_CORRECT": 8, "NOT_ADOPTED": 10, "UNEXECUTED_VERIFIED": 35} | {"CERTIFIED_NOT_OBSERVED": 13, "CERTIFIED_OBSERVED": 45, "UNEXECUTED_VERIFIED": 57} | {"MISSING": 23, "OBSERVED": 22} |
| BAAA | {"CLOSED_CERTIFIED": 22, "UNEXECUTED_VERIFIED": 23} | {"CERTIFIED_NOT_OBSERVED": 11, "CERTIFIED_OBSERVED": 91, "UNEXECUTED_VERIFIED": 113} | {"ADOPTED_INCORRECT": 19, "KNOWN_CORRECT": 9, "NOT_ADOPTED": 12, "UNEXECUTED_VERIFIED": 35} | {"CERTIFIED_NOT_OBSERVED": 12, "CERTIFIED_OBSERVED": 46, "UNEXECUTED_VERIFIED": 57} | {"MISSING": 23, "OBSERVED": 22} |
| CAAA | {"CLOSED_CERTIFIED": 22, "UNEXECUTED_VERIFIED": 23} | {"CERTIFIED_NOT_OBSERVED": 12, "CERTIFIED_OBSERVED": 90, "UNEXECUTED_VERIFIED": 113} | {"ADOPTED_INCORRECT": 25, "KNOWN_CORRECT": 9, "NOT_ADOPTED": 6, "UNEXECUTED_VERIFIED": 35} | {"CERTIFIED_NOT_OBSERVED": 11, "CERTIFIED_OBSERVED": 47, "UNEXECUTED_VERIFIED": 57} | {"INCOMPLETE": 1, "MISSING": 23, "OBSERVED": 21} |
| ABAA | {"CLOSED_CERTIFIED": 21, "CLOSED_TECHNICALLY_UNCERTIFIED": 1, "UNEXECUTED_VERIFIED": 23} | {"CERTIFIED_NOT_OBSERVED": 15, "CERTIFIED_OBSERVED": 85, "TECHNICALLY_UNCERTIFIED": 2, "UNEXECUTED_VERIFIED": 113} | {"ADOPTED_INCORRECT": 17, "KNOWN_CORRECT": 8, "NOT_ADOPTED": 13, "TECHNICALLY_UNCERTIFIED": 2, "UNEXECUTED_VERIFIED": 35} | {"CERTIFIED_NOT_OBSERVED": 15, "CERTIFIED_OBSERVED": 41, "TECHNICALLY_UNCERTIFIED": 2, "UNEXECUTED_VERIFIED": 57} | {"MISSING": 23, "OBSERVED": 22} |
| AABA | {"CLOSED_CERTIFIED": 21, "UNEXECUTED_VERIFIED": 24} | {"CERTIFIED_NOT_OBSERVED": 14, "CERTIFIED_OBSERVED": 86, "UNEXECUTED_VERIFIED": 115} | {"ADOPTED_INCORRECT": 15, "KNOWN_CORRECT": 9, "NOT_ADOPTED": 14, "UNEXECUTED_VERIFIED": 37} | {"CERTIFIED_NOT_OBSERVED": 12, "CERTIFIED_OBSERVED": 44, "UNEXECUTED_VERIFIED": 59} | {"MISSING": 24, "OBSERVED": 21} |
| AAAB | {"CLOSED_CERTIFIED": 22, "UNEXECUTED_VERIFIED": 23} | {"CERTIFIED_NOT_OBSERVED": 17, "CERTIFIED_OBSERVED": 85, "UNEXECUTED_VERIFIED": 113} | {"ADOPTED_INCORRECT": 21, "KNOWN_CORRECT": 8, "NOT_ADOPTED": 11, "UNEXECUTED_VERIFIED": 35} | {"CERTIFIED_NOT_OBSERVED": 13, "CERTIFIED_OBSERVED": 45, "UNEXECUTED_VERIFIED": 57} | {"MISSING": 23, "OBSERVED": 22} |
| AAAC | {"CLOSED_CERTIFIED": 22, "UNEXECUTED_VERIFIED": 23} | {"CERTIFIED_NOT_OBSERVED": 17, "CERTIFIED_OBSERVED": 85, "UNEXECUTED_VERIFIED": 113} | {"ADOPTED_INCORRECT": 21, "KNOWN_CORRECT": 7, "NOT_ADOPTED": 12, "UNEXECUTED_VERIFIED": 35} | {"CERTIFIED_NOT_OBSERVED": 14, "CERTIFIED_OBSERVED": 44, "UNEXECUTED_VERIFIED": 57} | {"MISSING": 23, "OBSERVED": 22} |

QA01: first substantive response host-callback latency. QA03: frozen causal VIA processing time. Observed-subset means are not whole-population success speed. QA02 incomplete correct/planned is a lower bound, not complete accuracy.

QA05 compares a complete fixed9-case or6-case bundle separately; partial known expenditure is retained. QA06 is an observed same-sample local process-tree maximum; incomplete coverage is a lower bound. Decimal MB and MiB differ.

## COMMON_SUPPLEMENT6

| Configuration | QA01 ms (observed/planned) | QA02 correct/planned; complete accuracy | QA03 ms (observed/planned) | QA04 modules | QA05 complete bundle USD; known spent | QA06 MB; status |
| --- | --- | --- | --- | --- | --- | --- |
| AAAA | 19330.608 (19/36) | 5/18; — | 20678.676 (15/24) | 1.833 (11/6) | —; 0.170118400000000005 | 53.756; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| BAAA | 22440.466 (21/36) | 5/18; — | 22112.569 (15/24) | 1.833 (11/6) | —; 0.167990000000000005 | 53.543; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| CAAA | 21111.706 (19/36) | 4/18; — | 21779.754 (14/24) | 1.833 (11/6) | —; 0.414660000000000034 | 55.755; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| ABAA | 16826.103 (19/36) | 5/18; — | 19063.100 (15/24) | 1.833 (11/6) | —; 0.165920800000000008 | 53.625; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| AABA | 20552.223 (21/36) | 5/18; — | 20566.945 (15/24) | 1.833 (11/6) | —; 0.186115600000000004 | 5951.275; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| AAAB | 22635.547 (19/36) | 5/18; — | 24077.687 (15/24) | 1.833 (11/6) | —; 0.443767600000000018 | 56.050; OBSERVED_LOWER_BOUND_DIAGNOSTIC |
| AAAC | 21686.048 (22/36) | 6/18; — | 22786.129 (16/24) | 1.833 (11/6) | —; 0.168772000000000012 | 53.740; OBSERVED_LOWER_BOUND_DIAGNOSTIC |

### Coverage and failures

| Configuration | Episode states | QA01 partition | QA02 partition | QA03 partition | RSS states |
| --- | --- | --- | --- | --- | --- |
| AAAA | {"CLOSED_CERTIFIED": 15, "UNEXECUTED_VERIFIED": 3} | {"CERTIFIED_NOT_OBSERVED": 11, "CERTIFIED_OBSERVED": 19, "UNEXECUTED_VERIFIED": 6} | {"KNOWN_CORRECT": 5, "NOT_ADOPTED": 9, "UNEXECUTED_VERIFIED": 4} | {"CERTIFIED_NOT_OBSERVED": 5, "CERTIFIED_OBSERVED": 15, "UNEXECUTED_VERIFIED": 4} | {"MISSING": 3, "OBSERVED": 15} |
| BAAA | {"CLOSED_CERTIFIED": 15, "UNEXECUTED_VERIFIED": 3} | {"CERTIFIED_NOT_OBSERVED": 9, "CERTIFIED_OBSERVED": 21, "UNEXECUTED_VERIFIED": 6} | {"KNOWN_CORRECT": 5, "NOT_ADOPTED": 9, "UNEXECUTED_VERIFIED": 4} | {"CERTIFIED_NOT_OBSERVED": 5, "CERTIFIED_OBSERVED": 15, "UNEXECUTED_VERIFIED": 4} | {"MISSING": 3, "OBSERVED": 15} |
| CAAA | {"CLOSED_CERTIFIED": 15, "UNEXECUTED_VERIFIED": 3} | {"CERTIFIED_NOT_OBSERVED": 11, "CERTIFIED_OBSERVED": 19, "UNEXECUTED_VERIFIED": 6} | {"KNOWN_CORRECT": 4, "NOT_ADOPTED": 10, "UNEXECUTED_VERIFIED": 4} | {"CERTIFIED_NOT_OBSERVED": 6, "CERTIFIED_OBSERVED": 14, "UNEXECUTED_VERIFIED": 4} | {"MISSING": 3, "OBSERVED": 15} |
| ABAA | {"CLOSED_CERTIFIED": 15, "UNEXECUTED_VERIFIED": 3} | {"CERTIFIED_NOT_OBSERVED": 11, "CERTIFIED_OBSERVED": 19, "UNEXECUTED_VERIFIED": 6} | {"KNOWN_CORRECT": 5, "NOT_ADOPTED": 9, "UNEXECUTED_VERIFIED": 4} | {"CERTIFIED_NOT_OBSERVED": 5, "CERTIFIED_OBSERVED": 15, "UNEXECUTED_VERIFIED": 4} | {"MISSING": 3, "OBSERVED": 15} |
| AABA | {"CLOSED_CERTIFIED": 15, "UNEXECUTED_VERIFIED": 3} | {"CERTIFIED_NOT_OBSERVED": 9, "CERTIFIED_OBSERVED": 21, "UNEXECUTED_VERIFIED": 6} | {"KNOWN_CORRECT": 5, "NOT_ADOPTED": 9, "UNEXECUTED_VERIFIED": 4} | {"CERTIFIED_NOT_OBSERVED": 5, "CERTIFIED_OBSERVED": 15, "UNEXECUTED_VERIFIED": 4} | {"MISSING": 3, "OBSERVED": 15} |
| AAAB | {"CLOSED_CERTIFIED": 15, "UNEXECUTED_VERIFIED": 3} | {"CERTIFIED_NOT_OBSERVED": 11, "CERTIFIED_OBSERVED": 19, "UNEXECUTED_VERIFIED": 6} | {"KNOWN_CORRECT": 5, "NOT_ADOPTED": 9, "UNEXECUTED_VERIFIED": 4} | {"CERTIFIED_NOT_OBSERVED": 5, "CERTIFIED_OBSERVED": 15, "UNEXECUTED_VERIFIED": 4} | {"MISSING": 3, "OBSERVED": 15} |
| AAAC | {"CLOSED_CERTIFIED": 15, "UNEXECUTED_VERIFIED": 3} | {"CERTIFIED_NOT_OBSERVED": 8, "CERTIFIED_OBSERVED": 22, "UNEXECUTED_VERIFIED": 6} | {"KNOWN_CORRECT": 6, "NOT_ADOPTED": 8, "UNEXECUTED_VERIFIED": 4} | {"CERTIFIED_NOT_OBSERVED": 4, "CERTIFIED_OBSERVED": 16, "UNEXECUTED_VERIFIED": 4} | {"MISSING": 3, "OBSERVED": 15} |

QA01: first substantive response host-callback latency. QA03: frozen causal VIA processing time. Observed-subset means are not whole-population success speed. QA02 incomplete correct/planned is a lower bound, not complete accuracy.

QA05 compares a complete fixed9-case or6-case bundle separately; partial known expenditure is retained. QA06 is an observed same-sample local process-tree maximum; incomplete coverage is a lower bound. Decimal MB and MiB differ.

## Paired and condition evidence

All8 alternative pairs retain all planned same-obligation rows. Only both-certified observed timing intersections carry a delta. B/C is computed directly; AAAA is reused without duplicated spending. Raw analyzer pairs remain separate diagnostics.

All15 cases, all repetitions, all7 configurations, and all6 supplemental conditions remain in case-repetition.jsonl, including missing/failed activation. No receipt proxy becomes semantic adoption or a free-worker proof.

## Financial reconciliation

| Global known USD | RESERVED USD | UNKNOWN USD | Held total USD | Remaining USD | Complete |
| --- | --- | --- | --- | --- | --- |
| 7.32727880000000034646 | 0 | 0 | 7.32727880000000034646 | 32.67272119999999965354 | False |

Recorded call rows: 4229; deduplicated rows: 2124; actual cloud dispatches: 2105; local records: 19.

Full UUID joins, rate classes, other scopes, late settlements and residuals are in report.json/calls.jsonl. UNKNOWN/RESERVED are held amounts, not settled spend. Local API0 is not zero compute cost.

## Input issues

0 input issues; see the exact retained structured records in report.json.

## Interpretation and limitations

- Observed timing means are micro means over certified observed obligations, not whole-population successful speed.
- Semantic review never replaces or filters a frozen QA01 callback or its denominator.
- No provider-internal concurrency or exact free-K2 proof; activation is diagnostic, not hypothesis acceptance.
- RSS is nominal50ms observed same-sample process-tree RSS, not continuous-time/full-product/remote-model memory; shared-page deduplication unverified.
- Local compute monetary cost is NOT_MEASURED. Recorded model comparison prices are not an invoice.
- Prior campaigns and probes remain separate global spending. AAAA observations are reused, not multiplied.
- Every prediction judgment is UNREVIEWED; numeric signs and missing evidence never create a winner.

Semantic annotations, including actual-request/oracle disagreements and NOT_APPLICABLE authority cases, never change a frozen metric or pairing denominator. All24 prediction judgments require independent review.
