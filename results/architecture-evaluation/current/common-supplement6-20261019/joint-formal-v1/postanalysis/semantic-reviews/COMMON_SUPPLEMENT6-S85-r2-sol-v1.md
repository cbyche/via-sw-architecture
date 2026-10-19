# S85 r2 bounded semantic review

Reviewer: GPT-6.1 Sol High (`/root/sol_high_continuation/sidecar_closure_sol`). Independent postlaunch review of ordinal22, `SUPPLEMENT-r2-S85`, jobs154–160, all seven configurations. This is a semantic/mechanism diagnostic, not endpoint or metric certification. S85 r1 Markdown was a method reference only; its outcomes were not inherited.

[Final JSON with all14 response and7 goal rows](COMMON_SUPPLEMENT6-S85-r2-sol-v1.json), SHA256 `f96a2f0b0d6042e5750e7ad4d6b992047c8b853b57c73c3decfd14a4a2294d1b`.

## Coverage and result

| Dimension | Observed result |
| --- | --- |
| Actual supplied/captured user inputs | 14 |
| Unique actual displayed bodies | 13 |
| Actual-request body judgment | 13 PASS,0 FAIL,1 UNREVIEWED because absent |
| Current corrected goal relation | 6 PASS,0 FAIL,1 UNREVIEWED because unadopted |
| Required corrected goal unmet | 1, AABA |
| Raw first response callback refs | 13, naming13 distinct events |
| All matching response refs | 19, naming the same13 events |
| Additional actual duplicate displays | 0 |
| Recorded final HOST_COMPLETE | 7, including AABA old U01 only |
| Actual U02 workflow failure | 1, AABA |

U01's six extra matching references are later U02 displays linked to the same origin. They are contextual references, not six new bodies or duplicate physical displays. U01 is answered before U02 is supplied in each run. Its correct original answer is preserved; it does not fulfill the later correction. AABA's missing U02 body is UNREVIEWED for body content, while required response/goal fulfillment remains explicitly unmet. Relation PASS rows are not extra independent response successes.

## Actual current selection and source

U01 asks `표시한 두 버튼의 행 이름을 알려줘.` All seven actual input packets contain selected-only native items B03/B04 and the matching native cue. U02 asks `직전 요청을 정정할게. 지금 표시한 두 버튼의 행 이름을 알려줘.` All seven actual U02 packets change the native selection/cue to B03/B05. The source stays SUP-SCREEN revision1; it is the user's selected set and request revision that change.

The actual source lines are:

```text
B03 | 설계 | 설계 문서 열기
B04 | 검토 | 검토 문서 열기
B05 | 구현 | 구현 문서 열기
```

All seven U01 bodies correctly give B03 설계 and B04 검토. Six U02 bodies correctly give B03 설계 and B05 구현. AAAC also copies the source's document-open label after each row name; it does not claim or perform opening. The actual requested row names are correct.

Thirteen actual meaning requests were independently inspected. U01 receives original SUP-SCREEN lines3–4 in every run. Successful U02 inputs contain actual B03/line3 and B05/line5 originals. Some B03 originals are reused from the earlier acquisition; CAAA's two snippets and AAAB's B05 snippet retain recorded producer provenance. Those are actual input exposures, not an inference from catalog availability and not proof of saved selection calls or cost. AABA acquires current B05 for U02 but fails before a U02 meaning request.

Every native input identifies the authored dense frame `sup-dense-v1`,2880×1800, image hash `d785df6196acde445dc93d6b300af041fa72bfdfe4d984fd3fb0bf784ac07d27`. Its adapter is `AUTHORED_SELECTION_ADAPTER_OF_RENDERED_SOURCE`, with `selected_object_ranges_only` and `complete_scene_objects=false`. This is authored rendered source, not captured Windows application/API evidence. No image or base64 content was decoded. The two selected items do not constitute a full scene inventory.

## Exact per-configuration outcomes

Pointers below are original events carried by the accepted bundle. Original trace files were not opened.

| Configuration / suffix | U01 capture / display | U02 capture / display | Final last input / request revision | Corrected goal |
| --- | --- | --- | --- | --- |
| AAAC /425e41 | `/events/6` /35 | `/events/47` /78 | U02 /2 | PASS |
| AAAA /81bbf8 | `/events/6` /35 | `/events/47` /78 | U02 /2 | PASS |
| BAAA /f3f0ae | `/events/6` /32 | `/events/44` /74 | U02 /2 | PASS |
| CAAA /1de83c | `/events/15` /48 | `/events/72` /105 | U02 /2 | PASS |
| ABAA /48806a | `/events/6` /35 | `/events/47` /78 | U02 /2 | PASS |
| AABA /2100e6 | `/events/6` /37 | `/events/49` /absent | U01 /1 | UNREVIEWED, required goal unmet |
| AAAB /fe7fed | `/events/17` /52 | `/events/78` /113 | U02 /2 | PASS |

All six actual U02 adoptions are CORRECTION of `goal:S85-U01`; final state keeps `last_event_id=S85-U02`,revision2, with the corrected B03/B05 body and matching host receipt/GOAL_COMPLETE. AABA retains only U01/revision1 HOST_COMPLETE. Its old goal does not certify the required correction. Source revision1 and Request revision2 remain different concepts.

Ancillary proposal labels are retained. AAAC, AAAA, CAAA, AABA and AAAB label initial U01 ANSWER targeting self-U01; no prior VIA clarification is invented from that label. Actual state creates the U01 goal. AAAC and CAAA U02 propose self-U02 as target_event while explicitly naming existing `goal:S85-U01`; actual adoption correctly updates that goal. Scoped body/currentness PASS does not assert that every raw label is ideal.

## AABA: correct native evidence, wrong prior-reference interpretation

AABA U01 records GROUNDING_NATIVE_CANDIDATES at `/events/21` with B03/B04 and a successful native-selection GROUNDING_TYPED_BIND at `/events/26`. Its U01 answer is correct.

U02 native candidates at `/events/66` are correctly B03/B05. The actual speech-producer request contains the current correction transcript and native cue, plus prior delivered B03/B04 context. Its actual response at `/calls/4/response_file` interprets the phrase as `prior_reference` to the previously identified B03/B04. This is a wrong proposed interpretation of the current request, not defective actual native selection data.

The two actual binder requests at `/calls/5/request_file` and `/calls/6/request_file` both contain the current native cue item_ids B03/B05, current producer candidates B03/B05, and the speech interpretation/prior delivered B03/B04. Both responses return old B03/B04 with `relation=prior_reference`. `/events/75` records `Binder may only use exactly one producer candidate per target`, then owner expansion repeats the correct native candidates at `/events/76`. The second binder output still selects B04, which is not in the current producer set. `/events/81` ends the U02 workflow with the same validation failure after expansion.

All three exact raw responses have `finish_reason=stop`; the speech/binder/binder completion-token counts are121/144/144. They are not truncated outputs. There is no U02 typed binding, meaning call, adoption, display or final correction. Its proposed old targets remain raw semantic/contract failure evidence; they are not a displayed wrong answer or a relation PASS. This actual r2 failure differs from the previous repetition's failed configurations and extent-ID errors.

## Observed native path and remaining skip-diagnostic limits

The mechanism claim is grounded in actual candidates, typed binding, operation events and exact linked request payloads. AABA's accepted event records cover indexes0–84 without an internal gap and retain seven model request/response reference pairs. The seven observed OPERATION_START stages are two `evidence_select_ranges`, two `grounding.speech_producer`, one `meaning_interpret_and_respond` and two `grounding.candidate_binding`. Its three local operation starts are two `source_range_read` and one `publication_host_receipt_wait`.

No screen-producer or parser dispatch is observed in that accepted event/linked-request scope, including the native owner expansion. That supports a bounded observation of the native-candidate path, independent of the generic mechanism label. It does not establish that every helper process spawn in the full execution is absent. Full helper-process lifecycle metadata and parser call-body fields `local_spawn_admitted_ns` and `local_parse_dispatch_admitted_ns` are not supplied here. Their absence-based formal `native_skip_observed` diagnostic remains UNRESOLVED and was not calculated. Frozen mechanism-output executions by this review are0; a later authorized formal diagnostic is separate.

U01 succeeds through typed native binding. U02 still performs speech and binder work and fails, so this episode does not establish a universally successful native path or a causal parser-cost/latency saving. The six other configurations' actual meaning inputs retain joint original-evidence grounding and correct current selection; no architecture winner is inferred.

## Authority, preparation and stop limits

All thirteen adoptions use DIRECT with Task null. Allowed/completed actions, permission operations and control operations are empty, and every final ingress map is empty. Source registry permission does not authorize executing a document-open label from the quoted row. No opening, modification, sending, external Agent work or approval is observed; approval and handoff are not applicable, without an unexercised safety PASS.

CAAA and AAAB each retain three preparation starts/completions and their producer/read work. AABA's failed speech/binder calls and owner expansion remain. No preparation or downstream work is dropped from cost evidence and no avoided-work estimate is computed.

The seven recorded technical states have returncode0, empty technical_failures, no technical_abort_reason and no unprovided mandatory events. No new mandatory technical-stop condition is established by this bounded review; AABA's workflow failure remains explicit. Clock, controller and certified selected-endpoint correctness were not independently audited. HOST_COMPLETE is not physical product or audio verification.

## Provenance and bounds

| Artifact | SHA256 |
| --- | --- |
| Formal freeze | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| Ordinal22 accepted index | `cf45d5c5d34b9c719e62d63cef3680560a762af4aa943e09c2bebfd41ecf73e7` |
| Accepted bundle manifest | `1c94e7ba092bbb68f1668bbaf60858d818e42b66c375989b4b9c494fe1257e41` |
| Accepted evidence | `d1933e1e06af1ecc1c8e7e98d9a1ce264e39601fec7b3a22cee065dff434c8fc` |
| Accepted worklist | `8a2d34bbde97d58fbd692359efe9d382faf20e609f205c787e1d300472d3916b` |
| Root-bound extraction receipt | `9c2e79189eff9ce85f24eccd82bdd5d0e1f8ea6fe1e531c5d9a379505e0f678b` |

Named request files total1,568,425bytes per named pass; the selected three raw responses total3,002bytes. Request projection finished in0.01363s, response projection in0.00738s and evidence serialization in0.02869s. Every operation was below1s, every file below16MiB, and total repeated reads remain below a conservative64MiB within the128MiB bound. A broad grounding display reached text-output truncation only; narrower exact inputs/proposals were subsequently inspected. No byte/time cap was exceeded and no extraction repeated.

Only accepted block evidence/worklist/scaffold, the prior method Markdown and exact linked raw model files were read. No original trace, corpus, active `runs.jsonl` or saved evaluator was opened. No evaluator/analyzer, domain parser, candidate, compiler/build, test, model/API/network or extractor was executed. No image/base64 decode, source/prompt/contract/metric/score/denominator/eligibility change, presentation edit or controller action occurred. No staging, commit or push occurred. Next ordinal23 requires a separate exact root GO after these final artifacts are fully read; this review does not advance a cursor.
