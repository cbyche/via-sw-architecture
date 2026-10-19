# Common9 newer-completion endpoint addendum

Status: **ROOT-APPROVED STATIC DESIGN; IMPLEMENTATION PENDING**. Approval recorded at **2026-10-19 06:36:02 UTC** from the coordinating root agent’s explicit current-task instruction. Recorded during formal-v2, before any BG episode was closed. No frozen evaluator, candidate, input, score or paid execution was changed. No synthetic test or corrected evaluator was run. Implementation and checks are deferred until the measurement window is released.

## Contract and finding

[Contract19 §5](../19-common9-evaluation-contract.md#5-qa별-같은-측정과-집계) measures the first actual substantive text callback; content correctness remains separate. [The inherited population rule](../../qa-candidate-package/common-scenarios.md#32-사건의-시각과-완료-의존) permits a newer completion card of the same Task to satisfy a preceding progress-response obligation. It does not permit omitting a user request, approval question or another Task's result.

The frozen `evaluate.py:218–230` implements that rule for three progress labels containing `더 최신 완료`: S11-A01, S18-A03 and S19-A01. It checks only a matching callback Task ID, substantive text, the substring `완료`, and display time after the progress source. It does not prove an actual newer RESULT source. A handling intention mentioning completion, or an older source displayed late, can therefore qualify statically. Conversely it considers that replacement only when no own progress callback exists, so it can miss an earlier valid completion card when a progress card appears later.

These are endpoint-eligibility defects, distinct from judging whether a model's ordinary answer is semantically correct. The closed formal-v2 traces inspected for this finding were S02 only; they contain no eligible BG obligations and demonstrate no fallback miscount. Other literal Task IDs are already excluded by equality. No candidate producer or model output is changed by the correction.

## Corrected predicate for this bounded runtime

Retain the ordinary substantive-text filter and the original progress source time. Scope the extra rule to the three frozen BG progress obligations above. Do not infer candidate intent from an evaluator answer.

1. The original obligation must have an actual `INPUT_CAPTURED` record with `input_class:agent`, `event_class:PROGRESS`, a nonempty Task ID, and an actual `provided_sources` entry. Preserve its source time as the latency root.
2. Join a proposed replacement callback's `stimulus_id` to its actual capture and provided source. The capture must be `input_class:agent` and `event_class:RESULT`. The progress capture, RESULT capture and callback must carry the same nonempty Task ID. No string aliasing or guessed Task association is allowed.
3. The RESULT source must be strictly newer than the progress source. Required clocks must be present and mutually consistent. The RESULT capture must be observed no later than its callback, and the callback no earlier than the RESULT source.
4. The callback phase must be `ACTUAL_AGENT_FACT` or `AGENT_RESULT`, the two actual result-emission paths in the frozen runtime. `HANDLING`, `DIRECT`, question and waiting phases cannot substitute through this rule. The misleading shared `content_type:TASK_QUESTION` field is not used as proof of event class.
5. For this implementation both result paths emit `checked_text(event.text)`, which only trims the actual received source text. Require the callback text to equal that captured RESULT text after outer trimming only. Do not normalize inner whitespace, rewrite a quote, compare to expected answer prose, or depend on the word `완료`. This proves source-to-card consistency in this bounded proxy; it does not prove an external Agent's domain claim is true. A future partial/translated/streamed renderer would require a separately specified provenance rule.
6. Confirm that this RESULT is the latest actually available same-Task PROGRESS/RESULT status as of the card. Use actual source/capture records and their clocks; a known newer status before the card excludes an older result. A status whose source is after the card cannot retroactively invalidate it. Provision-order uncertainty is handled below.
7. Collect the ordinary eligible progress callbacks and qualifying replacement cards together, then select the earliest eligible observed callback. Do not prefer an own callback merely because it exists. Its text may still be semantically wrong under the ordinary QA01 policy; that remains a separate body/Task diagnostic.

Current actual BG pairs are S11-A01→S11-A02, S18-A03→S18-A04 and S19-A01→S19-A02. These identify the frozen population; eligibility must still be proved by actual captures, provided sources, Task IDs and timestamps. Missing or wrong metadata must not be filled from expected values.

## Provision time and firstness uncertainty

`provided_sources.source_ns` is the authored source due time (`harness.py:140–143`), not a separately observed stdin send timestamp. A capture observed before the card proves receipt. A capture observed after the card, with a newer source timestamp already before that card, does not establish whether the source was actually provided before or after display. Do not invent `provided_at`, use future knowledge to silently invalidate a card, or claim latestness has been proved.

Record such a potentially superseding same-Task status as `UNRESOLVED_PROVISION_ORDER`. Do not admit the ambiguous replacement as a certified exact endpoint. If a certainly eligible own progress callback exists later, retain its time as an **upper bound** on the first eligible response, not an exact first-response observation. The earlier ambiguous card supplies a possible lower bound. Report `UNRESOLVED_EARLIEST_BOUNDARY`, both raw card IDs/times and the reason; keep exact latency null and the original planned obligation. If a certain callback already precedes every ambiguous candidate, that earlier callback remains a certified first endpoint. With no certain callback, do not manufacture an upper bound or a numeric penalty.

This uncertainty affects the endpoint's firstness, not whether raw text was displayed. Keep observed publication facts and semantic diagnoses separately. The primary observed-endpoint count includes only certified first endpoints; unresolved obligations remain visible alongside `NOT_OBSERVED` under the fixed215 denominator. Root explicitly approved this uncertainty policy before any BG episode was closed; corrected code and tests remain deferred until the paid measurement window is released.

## Static boundary cases — not executed

Times below share the host monotonic domain. P is a BG progress source at100. Unless stated otherwise all required metadata is present, the result text matches its actual capture, and phases are valid.

| Case | Observed records | Required decision |
| --- | --- | --- |
| Valid newer RESULT | RESULT source110, capture111, card120, same BG | Replacement eligible at120 |
| Older source displayed late | RESULT source90, capture115, card120 | Reject: source is not newer than P |
| Handling wording | User/HANDLING card120 says “BG 작업의 완료 여부를 확인하겠습니다” | Reject replacement despite `완료` substring |
| Different Task | RESULT source110/card120 belongs to another Task | Reject replacement |
| Not provided | No actual provided-source entry for proposed RESULT | Reject; retain missing evidence, do not synthesize a source |
| RESULT with wrong phase | Actual RESULT capture exists, but card phase is HANDLING or DIRECT | Reject replacement |
| Text differs from actual source | Card is a question, negation or altered text instead of actual RESULT text | Reject replacement; retain body diagnostic |
| Own progress displayed later | Valid result card120 and own progress card130 | Choose120; original algorithm could choose130 |
| Own progress displayed first | Own progress card105 and valid result card120 | Choose105 |
| Known newer status | RESULT source110/card140; newer BG PROGRESS source130/capture131 | Older RESULT cannot substitute as latest status |
| Future status | Valid result card120; another status source130/capture132 | Do not retroactively invalidate120 |
| Ambiguous provision, later own card | RESULT card120; later capture135 of status source115; own progress card130 | Exact firstness unresolved; possible120, certain upper bound130 |
| Ambiguous candidate after certain own card | Own progress card105, ambiguous result card120 | First endpoint105 remains certified |
| Card before actual capture/source | Card precedes its claimed RESULT receipt or source clock | Reject/inconsistent-source diagnostic |

## Evidence preservation and later execution

After the paid window, preserve the frozen original evaluator and all original outputs. Create a separately versioned corrected evaluator/addendum with its own source hash, finite boundary checks and exact policy above as approved. Re-evaluate the same retained raw315 plan under identical rules for every configuration. Do not change a source, trigger, candidate response, callback timestamp, cost or DAG to obtain the expected result.

Publish original versus corrected eligibility, selected card IDs, reason codes, observed/planned counts and timing differences for every affected row. Include rejected and unresolved candidates. Keep the original315 identities and215/75/115 denominators. QA02, QA03, QA04, QA05 and QA06 are unchanged by this endpoint correction. No paid rerun follows from re-evaluation. Corrected reporting must not hide a functional stale-progress redisplay merely because an earlier completion supplies a timing endpoint.
