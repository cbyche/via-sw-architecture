# QA02 adopted referent regrade v1

2026-10-20. Authorized reanalysis of saved outputs. No candidate, provider, input, deadline, oracle population, repetition or paid model execution changes.

## Population and endpoint

- Keep the selected 441 technically certified identities from the recovery pool analysis. Preserve the original excluded failed identity and both evidence generations.
- Score every original QA02 row: COMMON9 15 expressions times 5 repeats times 7 configurations (525); COMMON_SUPPLEMENT6 6 expressions times 3 repeats times 7 configurations (126). Report the populations separately.
- Use the same first pre-deadline SEMANTIC_ADOPTION event already selected by the frozen evaluator. A missing adoption is failure. Never use a later correction, an unadopted model proposal, an intermediate grounding proposal, a later answer or recipient result to repair that decision.

## Recover the adopted prediction before comparing the answer

1. Associate a plan binding with the fixed expression through the actual input and the binding's expression meaning. Whitespace, particles, added temporal words and a longer enclosing utterance do not change its identity. Do not associate a binding because its targets equal the expected answer.
2. Distinct references in one utterance remain separate. An enclosing expression covering two references cannot supply the same union as both predictions. Missing or conflicting attribution is failure.
3. When the input has exactly one scored referent and no binding can be associated, the adopted plan's top-level targets may supply the prediction if they unambiguously name the object used by that request. Preserve this as ADOPTED_PLAN_TARGETS, not as an invented binding. For multiple referents or ancillary sources, explicit adopted purpose/constraint fields may establish separate attribution, but require a row-specific author annotation pointing to that field. A conflicting associated binding takes precedence and is not repaired using other fields. Evidence citations or source_claims alone do not prove a referent decision. This clarification was recorded before corrected aggregate scoring; its examples are T1/T2 explicitly preserved in adopted constraints and H10/H11 explicitly assigned cost/schedule roles in adopted purpose.
4. Associate opaque/generated source IDs only through actual source registration, delivery, input event and lineage available before adoption. Preserve source version and extent. Resolve whole-source and whole-source named-extent spelling when they describe identical line ranges. Never merge a delivered explanation with its underlying document, an old report with its revision, or different source versions when the input distinguishes them.
5. A nominal summary/delivery ID must actually name the intended delivered kind of content. An acknowledgement stored under a summary-shaped ID is not a summary. Preserve a missing intended artifact as failure, rather than inventing that artifact from its name.
6. Compare the recovered canonical source/version/extent set with the unchanged intended oracle set. Exact set equality is success. Missing, additional or wrong objects/ranges/required versions, ambiguous attribution and no adopted target are failure. Rejected proposals never earn credit.

## Review and outputs

- First extract source-bound adopted inputs, bindings and plan targets with a stable anonymous row key and no expected target set. Record the expression associations and source aliases before producing aggregate scores. Keep automated and manually resolved associations distinct.
- Finite association rules depend on input/expression meanings, not configuration or an answer-set match. Every exception requires a row key, source pointer and explanation. Do not silently broaden a rule after seeing aggregate rankings.
- Preserve all 651 rows with original and corrected decisions, actual original predictions, canonical predictions, oracle, trace SHA-256, adoption event and reason. Reconcile every change with the original score and validate population uniqueness and coverage.
- No independent second reviewer is claimed. Ambiguous records without sufficient adopted evidence are failures with an explicit reason. Mechanical checks and the author's evidence review are reported as such.
- The corrected rate is successful rows / fixed planned rows times 100 for each configuration, with per-scenario and failure-reason tables. This is adopted referent identification in this synthetic text/callback PoC, not general model accuracy or product performance.
