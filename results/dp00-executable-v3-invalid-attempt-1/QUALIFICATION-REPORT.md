# DP-00 Executable Reference Architecture Qualification v3 Results

## Evidence status

VALID EXECUTABLE QA-v1 EVIDENCE; official QA-07 remains unevaluable.

## QA-v1 results

| QA | R1 raw / score | R3 raw / score | R1+@ raw / score | Target |
| --- | ---: | ---: | ---: | --- |
| QA-01 | 0.10109 seconds / 5 | 0.106207 seconds / 5 | 0.101069 seconds / 5 | p95 <= 3.0 seconds |
| QA-02 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=95% |
| QA-03 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=99% |
| QA-04 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=95% |
| QA-05 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=90% |
| QA-06 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=90% |
| QA-07 | UNEVALUABLE | UNEVALUABLE | UNEVALUABLE | <=1.5x |
| QA-08 | 0.00766108 seconds / 5 | 0.0128492 seconds / 5 | 0.00782663 seconds / 5 | p95 <=10 seconds |
| QA-09 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=98% |
| QA-10 | 100 percent / 5 | 100 percent / 5 | 100 percent / 5 | >=98% |
| QA-11 | 9.175e-05 seconds / 5 | 0.000144208 seconds / 5 | 9.5833e-05 seconds / 5 | p95 <=1.5 seconds |
| QA-12 | 10 milliseconds / 5 | 10 milliseconds / 5 | 10 milliseconds / 5 | p95 <=200 ms |

## Decision

**FULL QUALIFICATION BLOCKED ONLY BY QA-07**

R1 and R3 are not selected while QA-07 lacks the approved physical-memory denominator. R1+@ is evaluated as an R1 tactic; any latency improvement is reported without treating it as a third base architecture.
