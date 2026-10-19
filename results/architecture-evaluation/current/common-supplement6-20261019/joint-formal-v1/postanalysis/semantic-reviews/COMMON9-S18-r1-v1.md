# S18/r1 bounded semantic review v1

## Scope and reading rule

Exactly `PRIMARY-r1-S18`, ordinal10, job indexes70–76, seven configurations. All49 response obligations and21 goal-relation rows are retained. This is a postlaunch semantic diagnostic. It does not recalculate QA endpoints, filter wrong content out of timings, change denominators, or approve a new run. No raw trace was reread and no evaluator, parser, model, build, test or network call was executed. Frozen code, inputs and contracts remain unchanged.

- JSON: [COMMON9-S18-r1-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON9-S18-r1-v1.json); SHA `d05b548e0d4170a97cf3b26cfb75025d86f62025db31e7935d77f9b55d912622`.
- Bundle manifest: [manifest.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S18-r1-v3-r2/manifest.json); SHA `3002fa5c087b43a391b22f321d6b3289da9eee38ced5522c318f41d41192beb7`.
- Index: [index.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON9-S18-r1-v3-r2/index.json); SHA `9c2bb15eaf88541f928c1af2b16102f76973d65128ce5725744fb2f8d8ec0ed7`.
- Receipt: [EXECUTION-RECORD.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON9-S18-r1-v3-r2/EXECUTION-RECORD.json); SHA `3232caecf97537cb3b18e92cdf229647a6f069e6cdcc5601c248ecdd1cce1a6e`.
- Formal freeze SHA `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd`.

**Do not flatten this case into “41 successful responses.”** The categories below distinguish supported status, correct handling intent, faithful relay, missing body, actual source fulfillment and verified work product. An admitted handoff, accepted source-fact receiver, synthetic RESULT label and HOST_COMPLETE each mean less than a verified two-page report or presentation file.

## Actual requests and source facts

| Input | Literal actual request |
| --- | --- |
| S18-U01 | 보고서 작업은 어디까지 됐어? |
| S18-U02 | 그 보고서는 결론을 먼저 놓고 두 장으로 줄여줘. |
| S18-U03 | 그 수정본을 자료로 별도의 발표자료 파일을 만들어줘. |

All seven initial TA records identify a completed report task, current_request “보고서 두 장 작성,” and artifact `report-v1.txt`. Its actual source `TA:current-request` revision1 is:

> TA 보고서 요청: 결론을 포함한 보고서 두 장. 현재 완료 확인됨.
> 결과: report-v1.txt

`DOC-A` revision1 supplies five short factual lines about costs, schedule, resources, ten prepared files/conclusion-first, and a paragraph to edit. Both sources permit local and downstream reads. Neither is the physical `report-v1.txt` report body. The filename is not automatically a registered source or a read grant.

This supports all seven U01 completed-status answers. AABA’s extra “결론을 포함한 두 장” detail already exists in the initial source, so it is not evidence of future U02 leakage. U02 still explicitly requires conclusion-first and two pages; both constraints must survive the handling/handoff. U03 requires a separate presentation using the revised report, which cannot be established merely by reusing the old request log.

The prior fixture-label review is used only for interpretation after current input comparison: TA/TC are descriptive aliases, not mandatory literal new Task names. U02 must target the existing report, and U03 must be a distinct work request with the correct revised source. No prior outcomes are inherited. Legacy voice-summary wording is NOT_APPLICABLE under frozen Voice OFF.

## Coverage without incompatible success arithmetic

| Category | Count | Reviewed fact and limit |
| --- | ---: | --- |
| Supported current-report status bodies | 7 | Actual TA source supports completed state and report-v1.txt |
| Report-revision handling intents | 7 | Conclusion-first and two-page constraints retained; admission/result may still fail |
| Separate-presentation handling intents | 6 | Distinct intent observed; adopted revised-source requirement FAIL in all6 |
| Actual bounded receiver relays | 7 | Three A01 plus four A02; faithful quotation is not verified work |
| Actual BG fact relays | 14 | Separate progress and completion callbacks, no user-goal substitution |
| Absent response bodies | 8 | Four A01, three A02, one U03; all remain UNREVIEWED |
| Total planned response rows | 49 | Above categories preserve the entire block |
| Goal rows with actual adopted request | 20 | One U03 is unadopted; this is not a success count |
| Runtime HOST_COMPLETE goal rows | 14/21 | Seven status, three revision-result and four presentation-result states |
| Verified full revised-report/presentation products | 0 | No physical product is certified by this bounded review |

The JSON records literal status/intent/relay fidelity as PASS for the41 observed bodies and UNREVIEWED for8 absent bodies. This is deliberately not a full semantic-success measure. Required-source failures and product limits below remain separate. For cross-case reporting, keep this case as a scoped49-row table rather than adding41 to a differently scoped “correct answer” subtotal.

## All49 response obligations

Cells show the actual display event and trace index. S = supported status, I = handling intent, R = faithful bounded receiver relay, B = actual BG fact. “Absent” is UNREVIEWED body content, not an invented wrong body. Each exact original trace path/hash and all row pointers are in the JSON.

| Config | RU01 | RU02 | RA01 | RU03 | RA02 | RA03 | RA04 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| AAAB | S `event-85` (`/events/71`) | I `event-153` (`/events/130`) | R `event-234` (`/events/201`) | I `event-230` (`/events/199`) | R `event-326` (`/events/279`) | B `event-31` (`/events/24`) | B `event-51` (`/events/41`) |
| AAAC | S `event-67` (`/events/56`) | I `event-106` (`/events/91`) | Absent | I `event-142` (`/events/123`) | R `event-182` (`/events/158`) | B `event-13` (`/events/10`) | B `event-33` (`/events/27`) |
| AAAA | S `event-61` (`/events/51`) | I `event-100` (`/events/86`) | Absent | I `event-136` (`/events/118`) | R `event-181` (`/events/157`) | B `event-13` (`/events/10`) | B `event-33` (`/events/27`) |
| BAAA | S `event-58` (`/events/48`) | I `event-97` (`/events/82`) | Absent | I `event-133` (`/events/113`) | R `event-178` (`/events/152`) | B `event-13` (`/events/10`) | B `event-33` (`/events/27`) |
| CAAA | S `event-81` (`/events/67`) | I `event-148` (`/events/124`) | R `event-192` (`/events/162`) | I `event-270` (`/events/228`) | Absent | B `event-29` (`/events/22`) | B `event-49` (`/events/39`) |
| ABAA | S `event-61` (`/events/51`) | I `event-100` (`/events/86`) | Absent | I `event-173` (`/events/150`) | Absent | B `event-13` (`/events/10`) | B `event-33` (`/events/27`) |
| AABA | S `event-61` (`/events/51`) | I `event-100` (`/events/86`) | R `event-154` (`/events/132`) | Absent | Absent | B `event-13` (`/events/10`) | B `event-33` (`/events/27`) |

There are41 distinct raw-first display events for41 observed rows. No repeated display event or superseded input-purpose row was found. `all_matching_display_refs` contains48 references because each of the3 A01 callbacks also matches its U02 origin, and each of the4 A02 callbacks also matches its U03 origin. These7 repeated references do not create additional bodies or independent answers. The first U02/U03 callback is its handling, not the later result.

Both BG PROGRESS and BG RESULT were separately provided and displayed in all seven runs. The newer-completion substitution rule was not needed. Their `ACTUAL_AGENT_FACT` receipt/state records explicitly name Task BG and task_fact_state_recorded; no goal or completion for the user’s TA revision/presentation is invented.

## Goal association, admission and result table

| Config | U01 goal | U02 revision work | U03 separate work | Actual receiver/product status |
| --- | --- | --- | --- | --- |
| AAAB | `goal:S18-U01` / `TA`; ingress0; HOST_COMPLETE | `goal:S18-U02` / `TASK-5`; ingress1; HOST_COMPLETE | `goal:S18-U03` / `TASK-8`; ingress1; HOST_COMPLETE | A01: REQUEST_RESTATED_NOT_DEMONSTRATED; A02: CREATION_CLAIM_UNVERIFIED_AND_REVISED_SOURCE_UNSUPPORTED |
| AAAC | `goal:S18-U01` / `TA`; ingress0; HOST_COMPLETE | `goal:S18-U02` / `TASK-5`; ingress0; not complete | `goal:S18-U03` / `TASK-7`; ingress1; HOST_COMPLETE | A01: NOT_OBSERVED; A02: EXPLICIT_NOT_CREATED |
| AAAA | `goal:S18-U01` / `TA`; ingress0; HOST_COMPLETE | `goal:S18-U02` / `TASK-5`; ingress0; not complete | `goal:S18-U03` / `TASK-7`; ingress1; HOST_COMPLETE | A01: NOT_OBSERVED; A02: EXPLICIT_UNVERIFIABLE_MISSING_CONTENT |
| BAAA | `goal:S18-U01` / `TA`; ingress0; HOST_COMPLETE | `goal:S18-U02` / `TASK-5`; ingress0; not complete | `goal:S18-U03` / `TASK-7`; ingress1; HOST_COMPLETE | A01: NOT_OBSERVED; A02: EXPLICIT_UNABLE_TO_PRODUCE_BOUNDED_DRAFT |
| CAAA | `goal:S18-U01` / `TA`; ingress0; HOST_COMPLETE | `goal:S18-U02` / `TASK-5`; ingress1; HOST_COMPLETE | `goal:S18-U03` / `TASK-9`; ingress1; not complete | A01: MODIFICATION_CLAIM_UNVERIFIED; A02: NOT_OBSERVED |
| ABAA | `goal:S18-U01` / `TA`; ingress0; HOST_COMPLETE | `goal:S18-U02` / `TASK-5`; ingress1; not complete | `goal:S18-U03` / `TASK-7`; ingress0; not complete | A01: NOT_OBSERVED; A02: NOT_OBSERVED |
| AABA | `goal:S18-U01` / `TA`; ingress0; HOST_COMPLETE | `goal:S18-U02` / `TASK-5`; ingress1; HOST_COMPLETE | Unadopted | A01: DESCRIPTIVE_DRAFT_ONLY_FULL_PRODUCT_UNVERIFIED; A02: NOT_OBSERVED |

U01 is a status question; its completed answer does not make a later revision or presentation complete. U02 creates a new revision-work request referencing the original TA report, with actual derived Task TASK-5. U03, when adopted, creates a separate goal and a distinct Task (TASK-7, TASK-8 or TASK-9), despite some raw FOLLOWUP proposals carrying target_goal_id of U02. Actual adoption, not the unused proposed field or a literal TC alias, determines the recorded mapping.

All20 adopted request identities have the intended status/revision/separate-work association. This narrow association PASS does not repair the six adopted U03 revised-source failures. AABA’s unadopted U03 relation remains UNREVIEWED. No correct product is inferred from canonical Task state changes or the14 HOST_COMPLETE markers.

## Per-configuration evidence and limits

### AAAB — `AAAB-S18-r1-ba6fb9`

- **A01:** Accepted bounded receiver restates that the completed original report must be revised; it does not supply a two-page revised report. Actual synthetic result relay exists.
- **U03:** U03 capture and handling precede A01 display. Its public U02 request remains DELEGATE. The proposed/adopted source set is old TA:current-request plus DOC-A; using these as revised report data is not justified.
- **A02:** Accepted bounded receiver claims a presentation draft from DOC-A and TA:current-request, but acquired spans contain the original request log and DOC-A, not actual revised-result content. Physical presentation and correct revised-source use are not established.

Receiver S18-U02, accepted=true, `event-196` (`/events/168`):

> 보고서 'report-v1.txt'는 작업이 완료되었으며, 결론을 먼저 제시하고 두 장 이내로 제한하도록 수정해야 합니다.

Receiver S18-U03, accepted=true, `event-318` (`/events/273`):

> Presentation draft created based on revised report data from DOC-A and TA:current-request.

### AAAC — `AAAC-S18-r1-06eaed`

- **A01:** No U02 ingress: completion_condition is null and handoff build fails. No A01 source or body is fabricated.
- **U03:** U03 proceeds after missing-A01 fallback. Only original TA request log is acquired; TASK-5 is not a confirmed revised report.
- **A02:** Actual bounded receiver explicitly says No presentation file was created and explains missing report contents. The outer synthetic RESULT/file label and HOST_COMPLETE do not reverse that statement.

Receiver S18-U03, accepted=true, `event-174` (`/events/152`):

> No presentation file was created. The provided source 'TA:current-request' contains only a request confirmation and a filename reference ('report-v1.txt'), but does not contain the actual content of the revised report required to generate a presentation. Additionally, the allowed action 'create_presentation' cannot be executed as no real external actions are performed.

Recorded unknowns: `["Content of report-v1.txt is not provided in actual_evidence", "Cannot create presentation without source content"]`.

Workflow failures:

- `event-112` (`/events/97`): `HANDOFF_ADOPTION_completion_condition_MISSING`.

### AAAA — `AAAA-S18-r1-ff292a`

- **A01:** No U02 ingress: completion_condition is null and handoff build fails. No A01 source or body is fabricated.
- **U03:** U03 proceeds after missing-A01 fallback. Only original TA request log is acquired; TASK-5 is not a confirmed revised report.
- **A02:** Actual bounded receiver says presentation creation cannot be verified and revised report contents are missing. This is not evidence that a physical presentation was created, nor proof of its physical nonexistence.

Receiver S18-U03, accepted=true, `event-173` (`/events/151`):

> Based on the provided evidence, the source document 'TA:current-request' confirms the completion of a report request, resulting in the file 'report-v1.txt'. The mandatory contract requires creating a separate presentation file using this revised report as the source. However, the actual evidence provided only contains the text confirming the report's existence and does not contain the content of 'report-v1.txt' itself, nor does it provide evidence that a presentation file has been created. Therefore, while the source is identified, the specific content required to generate the presentation is missing from the supplied originals, and the completion condition (file creation) cannot be verified from the text alone.

Recorded unknowns: `["Content of report-v1.txt is not provided in actual_evidence", "Verification of presentation file creation is not possible from text evidence alone"]`.

Workflow failures:

- `event-106` (`/events/92`): `HANDOFF_ADOPTION_completion_condition_MISSING`.

### BAAA — `BAAA-S18-r1-048bf7`

- **A01:** No U02 ingress: completion_condition is null and handoff build fails. No A01 source or body is fabricated.
- **U03:** U03 proceeds after missing-A01 fallback. Only original TA request log is acquired; TASK-5 is not a confirmed revised report.
- **A02:** Actual bounded receiver explicitly says it is unable to produce the bounded draft or handoff verification because revised report contents are absent. An accepted source-fact result is not a fulfilled presentation request.

Receiver S18-U03, accepted=true, `event-170` (`/events/146`):

> Unable to produce the bounded draft or handoff verification. The mandatory contract requires the creation of a presentation file using 'report-v1.txt' as source material. However, the actual supplied evidence only contains a request log stating that 'report-v1.txt' is the result, but does not contain the actual content of 'report-v1.txt'. Without the text content of the revised report, no facts can be extracted to form the presentation, and the completion condition cannot be met based on the provided originals.

Recorded unknowns: `["Content of report-v1.txt is missing from actual_evidence."]`.

Workflow failures:

- `event-103` (`/events/88`): `HANDOFF_ADOPTION_completion_condition_MISSING`.

### CAAA — `CAAA-S18-r1-f5af1f`

- **A01:** Accepted bounded receiver claims the report was modified, but supplies only a short textual claim; full two-page artifact is not inspected or certified.
- **U03:** Initial proposal names actual TASK-5:result but did not acquire that original span. Repair switches to TA:current-request and DOC-A. Actual source/currentness requirement remains unmet even though the handling says TASK-5 modification.
- **A02:** Actual receiver repair returns text:null with missing modified-report evidence and required-source errors; receipt accepted=false. No A02 source/body follows.

Receiver S18-U02, accepted=true, `event-185` (`/events/157`):

> 보고서 'report-v1.txt'는 결론을 먼저 제시하고 두 페이지로 제한하도록 수정되었습니다.

Receiver S18-U03, accepted=false, `event-318` (`/events/270`):

> `text:null`

Recorded errors: `["EMPTY_OR_OVERSIZED_RESULT", "REQUIRED_SOURCE_FACT_MISSING \"DOC-A\"", "REQUIRED_SOURCE_FACT_MISSING \"TA:current-request\""]`.

Recorded unknowns: `["The mandatory condition requires using 'modified report content' from 'artifact-S18-U02.txt', but this artifact is not provided in the actual_evidence.", "The actual_evidence contains raw source data (DOC-A) and a request log (TA:current-request), but does not contain the specific 'modified report' text required by the completion condition.", "Cannot create the presentation file as the primary source material (the modified report) is missing from the supplied evidence."]`.

### ABAA — `ABAA-S18-r1-b9481a`

- **A01:** U02 ingress exists, but receiver serves only DOC-A and rejects missing required TA:current-request. Receipt accepted=false; no A01 source/body.
- **U03:** U03 proceeds after absent revised result, names TASK-5 and old request source, then fails an unsupported original-source read. Distinct presentation intent is observed; actual revised-source provision is not.
- **A02:** U03 handoff tries report-v1.txt lines1-200, a filename reference without an authorized registered source/span. It fails HANDOFF_NO_AUTHORIZED_DETAILS before ingress; no A02 source/body.

Receiver S18-U02, accepted=false, `event-153` (`/events/133`):

> `text:null`

Recorded errors: `["EMPTY_OR_OVERSIZED_RESULT", "REQUIRED_SOURCE_FACT_MISSING \"TA:current-request\""]`.

Recorded unknowns: `["REQUIRED_SOURCE_FACT_MISSING \"TA:current-request\"", "Source 'TA:current-request' is required but not provided in actual_evidence", "Cannot fabricate content for missing source 'TA:current-request'"]`.

Workflow failures:

- `event-195` (`/events/169`): `HANDOFF_NO_AUTHORIZED_DETAILS [{"consumer":"VIA_HANDOFF_COMPOSER","error":"RANGE_OR_PROVENANCE_NOT_AUTHORIZED","external":true,"request":{"end_line":200,"revision":1,"source_id":"report-v1.txt","start_line":1}}]`.

### AABA — `AABA-S18-r1-626fd3`

- **A01:** Accepted bounded receiver supplies a short descriptive revised-draft statement (conclusion first, two pages), not a verified two-page work product. Actual result relay exists.
- **U03:** All three raw proposals name actual TASK-5:result and quote the actual delivered result, but that source was not included in actually_supplied_originals. This is a correct intended ID in an unadopted unsupported-span proposal; it is not delivered source adoption or verified work.
- **A02:** U03 has no adopted plan or ingress after three quote-validation failures. No A02 source/body is fabricated.

Receiver S18-U02, accepted=true, `event-147` (`/events/127`):

> 보고서 수정 초안: 결론을 먼저 제시하며 두 페이지 분량으로 구성됨.

Workflow failures:

- `event-202` (`/events/176`): `MEANING_TOTAL_LIMIT_EXHAUSTED: claim quote is not an exact substring of actually supplied span after JSON decoding`.

### Revised-source failures are observable, not a filename convention

All six observed U03 handlings speak of using the revised report, but their actual acquired/adopted source set is `TA:current-request` revision1, with DOC-A added for AAAB and CAAA. No acquired revised-result content appears in these adopted source sets. This is FAIL for the actual revised-source requirement. It does not mean their generic create-a-presentation intention was misheard.

CAAA initially proposes the actual `TASK-5:result:v1:result` and quotes its actual result text, but that original span was not acquired; the validator rejects it. Its repair adopts old TA/DOC-A sources, and the receiver later returns null because the modified report is missing. AABA likewise proposes the real revised-result ID and actual quote in all3 attempts but never acquires the required span, so no plan/handling is adopted. Correct raw IDs are not proof of successful adoption or source use.

AAAB receives A01 as an actual input before U03, but U03 is captured at event201/index173 and its handling appears at event230/index199 before A01 is displayed at event234/index201. Its U03 model sees U02 as DELEGATE. This order is retained; a future or pending result must not be treated as already delivered source. CAAA/AABA do display A01 before U03, yet their current evidence acquisition still misses that original result span.

### Product realization and outer RESULT wording

The exact frozen harness constructs the outer text “업무 결과 파일 artifact-…txt” and an “Actual synthetic downstream artifact” from the accepted receiver response (harness.py:29–31). Its actual Agent event is supplied only after real ingress and an accepted receiver receipt (:248–255). This is an actual synthetic test event, not a claim that a physical presentation was produced.

The receiver is explicitly a bounded draft/verification role with actions_performed empty. Its `accepted` predicate checks nonempty/limited text, exact authorized source facts, every required source and no claimed real actions (handoff.rs:85–117,366–377). It does not mechanically verify all prose claims, a two-page layout or a physical presentation file.

- AAAB A01 says the report **must be revised**, so it does not demonstrate the requested revision even though a result wrapper exists.
- CAAA A01 claims modification; AABA A01 supplies a short descriptive draft. Full revised artifacts remain unverified.
- AAAB A02 claims a draft based on revised-report data but supplies only DOC-A and the original request log. The revised-source condition fails and the product claim is unverified.
- AAAC A02 explicitly says **no presentation file was created**.
- AAAA A02 says creation **cannot be verified** from the available content. Do not turn that into either verified creation or proven physical nonexistence.
- BAAA A02 explicitly says it is **unable to produce even the bounded draft/verification**.

Faithful relay of these nested bodies is not successful work realization. The outer RESULT prefix, actual Task result registration and HOST_COMPLETE cannot override an explicit limitation or supply a missing revised report.

## Anchor flags versus final supplied bodies

| Config | A01 anchor flag | A01 finally supplied/displayed | A02 anchor flag | A02 finally supplied/displayed |
| --- | --- | --- | --- | --- |
| AAAB | Yes | Yes, late after flag | Yes | Yes, late after flag |
| AAAC | Yes | No | No | Yes |
| AAAA | Yes | No | No | Yes |
| BAAA | Yes | No | No | Yes |
| CAAA | No | Yes | Yes | No |
| ABAA | Yes | No | Yes | No |
| AABA | No | Yes | Yes | No |

The flagged-run counts are A01=5 and A02=4. The finally supplied/displayed counts are A01=3 and A02=4. Both statements are true because AAAB later supplies the actual sources:

- A01 missing-anchor flag at `1986268262754125`; actual successful send end at `1986270920972708`.
- A02 missing-anchor flag at `1986306262754125`; actual successful send end at `1986311126785583`.

The original flags remain; late actual inputs and displays are also retained. A01 is finally nonprovided in four runs; A02 in three. AABA additionally captures U03 but produces no adopted body after quote-validation exhaustion. These account for8 absent response bodies. No missing Agent fact is fabricated and no late source is silently erased.

## Authority and stop-rule assessment

TA:current-request and DOC-A have actual local/downstream read authority. Model/handoff requirements retain user-authorized revise_report or create_presentation actions, with no adopted permission operation or claimed completed external action. Receiver actions_performed is empty. `report-v1.txt` is only a filename in a log, so ABAA’s request to read it as an authorized registered source at lines1–200 is correctly retained as a failed source/provenance request. Approval is NOT_APPLICABLE; real external actions are not exercised by the bounded receiver.

**This bounded review establishes no new campaign technical-stop trigger.** Missing mandatory completion conditions, unsupported quote spans, missing receiver facts and current-source failures are real candidate/receiver outcomes. The observed outer synthetic result behavior is in the frozen method and must be disclosed, not upgraded into product success.

The seven copied terminal records were hash/size checked and all report returncode0, technical_failure=false and budget_stop=false. Frozen harness.py:296 explicitly preserves RESPONSE_ANCHOR_TIMEOUT and MISSING_INGRESS_OR_AGENT_ANCHOR separately from host/control technical failures. The joint controller honors explicit HOLD, technical and budget markers and stops on process/technical/budget failure (:153–194). This review does not run the strict technical evaluator or claim fresh clock/endpoint certification.

Contract19:99 requires separate Task, approval and body diagnostics and preserves wrong-content timing observations. These findings therefore do not authorize a source/prompt/fixture change, a stopped-and-tuned replacement run, a retrospective QA filter, or a new denominator. Failures and all associated costs remain.

## Exact source bindings

| Source | SHA-256 |
| --- | --- |
| [fixtures](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/fixtures.json) | `529d8f41cd72bd861e84475697c5af3cd64e070f559801e71de1d715c83c6d79` |
| [oracles](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/oracles.json) | `74b434872444c2a303e0f1bcee1ba162e519386a82620c1a74c2666142f5da52` |
| [contract19](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md) | `7b3db7b8f358ee6cd441ca37ef13f9b83d1f697322729a0f7bde802bbdc162e7` |
| [runtime](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/prototypes/candidates/common9_poc_20261019/src/runtime.rs) | `d96f61406119d870bf65bb9045a8fe2bb8eb1aa68ae8319a65a09845a4437402` |
| [handoff](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/prototypes/candidates/common9_poc_20261019/src/handoff.rs) | `22290f705503ccb85c7ace2ae5a41f55f162abd054e6fa917748c1d57e7e2b1d` |
| [harness](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/harness.py) | `d30750622e97978f769da8366a8c12510e91cbd482ead32d1111ed9cc7e079d9` |
| [joint_campaign](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/joint_campaign.py) | `9314f4f36f58ae2254e0f2d0cb33d4fe5debba5c43f450b696154068745c8d50` |
| [historical_label_guidance](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common9-poc-20261019/independent-formal-audit/v2-semantic/fixture-label-review.md) | `9ba2eb427cf792ba46b1cd924fbb1cd8f26b08a1f82388acb4f6fcf0e31cb4c4` |

## Limits and handoff

All49 response and21 goal rows have explicit original references and scoped status dimensions in the JSON.41 host receipt records link to actual display events; user-goal and BG Task-fact paths remain distinct. No saved certified endpoint is available in this bundle. No physical artifact inspection, full product execution, timing aggregation or DP comparison was performed. This is one case and repetition.

Root may report prior compatible semantic subtotals plus this separately reviewed49-row table. It should not combine differently scoped PASS labels into a new score. There is no automatic cursor advance.
