# Session 7.1 — Realistic Profile Qualification Evidence

## Scope

This record captures implementation qualification for the prospective R1–R4
profile freeze. It is not an A/B/C/D comparative campaign, does not rank
alternatives, and does not modify Session 6.x or Session 7.0 evidence.

## Static and automated qualification

| Check | Result |
| --- | --- |
| Python `.venv` full pytest | PASS — 95/95 |
| Rust workspace/all-target check | PASS |
| Rust format check | PASS |
| Rust workspace/all-target clippy, warnings denied | PASS |
| Rust qualification build | PASS |
| Rust workspace/all-target qualification tests | PASS — 107/107 |
| Existing Session 7.0 raw strict validation under analyzer v9 | PASS — 0 errors, 0 warnings |

Focused profile coverage verifies:

- contract JSON and Rust R1–R4 constants match exactly;
- Profile Z remains 0/0/0 µs and Profile C remains provisional;
- cost composition is additive and takes no alternative identity;
- R1 changes elapsed timing without changing P01/P12 semantic projections;
- all Model, Agent, and Tool semantic events contribute the expected configured
  budget;
- P12 keeps S1 then S2, common parent, distinct children, both commits before
  execution, and two charged effects across A/B/C/D;
- qualification CLI accepts only Z or frozen R1–R4 and rejects provisional C.

## Small preflight

The optimized runner executed P01 and P12 for A/B/C/D under R1, R2, R3, and R4,
one CAPTURE repetition each: 32 development-only executions. All completed with
no architecture error. This was the permitted implementation qualification run,
not the final campaign; no result ranking was produced.

`dp00-analysis-v9` strict validation returned:

```text
valid: true
errors: 0
warnings: 0
```

Two derivations from the same transient raw root were byte-identical:

```text
SHA-256  71e7b8b6c10e9877e2d84687c2550ad57070d4ac9e5ed63f4088007c716093c0
```

The transient raw and derived files were created outside the repository and are
not campaign evidence. The repeatable contract, tests, and this qualification
summary are the Session 7.1 deliverables.

## Disposition

`REALISTIC EXECUTION PROFILE FROZEN — READY FOR COMPARATIVE CAMPAIGN`

DP-00 remains:

`NOT READY FOR ARCHITECTURE DECISION`
