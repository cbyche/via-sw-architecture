# COMMON9 S08 r2 bounded semantic review v1

## Scope and exact binding

Reviewer `/root/method_astra_review`, GPT-6 Astra Ultra. Only ordinal17 `PRIMARY-r2-S08`, jobs119–125. Twenty-one response and seven goal-relation obligations are present once each. No r1 result was inherited. The review used the accepted extracted records, their named raw model JSON, frozen source slices and one exact frozen PNG view. No original trace was reread and no evaluator, parser, model, test, build, network call or frozen edit occurred.

| Artifact | SHA-256 |
| --- | --- |
| [bundle_manifest](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S08-r2-v3-r2/manifest.json) | `ca63bf83c10a93d4760e223fe50090b7dbc7f7a753b1d2d72a013629e7fb8c9c` |
| [evidence](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S08-r2-v3-r2/evidence.json) | `4696db63c230f77c3cb8fba73e72f2401eacdb21775136e156efa35c5d24017e` |
| [worklist](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S08-r2-v3-r2/worklist.json) | `3f1f3033b2a961174672fa0e2f388dc842517d48cdf5b5cb6c49ec8d32514b06` |
| [index](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON9-S08-r2-v3-r2/index.json) | `5aec997090d9b2e7e3a5da9a7a6b3e46746048a128295550c2d01c6b14c35ef0` |
| [execution_receipt](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON9-S08-r2-v3-r2/EXECUTION-RECORD.json) | `5623363e04e55d43fd3bec1ebbbeb4dd0816e331a6d4ca31c7be7ff35b204fe0` |
| [freeze](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json) | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| [fixture](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/fixtures.json) | `529d8f41cd72bd861e84475697c5af3cd64e070f559801e71de1d715c83c6d79` |
| [oracle](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/oracles.json) | `74b434872444c2a303e0f1bcee1ba162e519386a82620c1a74c2666142f5da52` |
| [screen_png](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/assets/s08-doc-a-v1.png) | `c29794fb3fc1f053fc78186ac0ef9021acd78fd283624ac8c3b73b11ef368ba4` |

Complete immutable row evidence: [COMMON9-S08-r2-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON9-S08-r2-v1.json), SHA-256 `e7ae39cfd4244f1eccd065fec0063b90fc7218916d8fe5d39fc7b3f05d6e60ab`.

## Exact response accounting

| Response scope | Count |
| --- | --- |
| Correct initial handling intention for U01 | 6 |
| Incorrect current-correction handling for U02 | 2 |
| Faithful relay of actual A01 text, not product success | 2 |
| No actual body, content UNREVIEWED | 11 |
| Planned obligations | 21 |
| Unique actual display events | 10 |
| Raw response callback references | 13, referring to 10 distinct display events |

The JSON has eight scoped PASS rows: six initial handling intentions and two faithful relays. These must not be presented as eight completed user requests. The two U02 bodies FAIL current-correction semantics. Eleven missing bodies remain UNREVIEWED rather than receiving an invented body-content verdict. Three repeated contextual callback references are preserved: AABA RU02 also links its A01, while AAAA RU01 also links its U02 and A01. Actual bodies are assigned by their own stimulus and are not counted twice.

Goal member/current revision association is **1 PASS (AAAA), 1 FAIL (AABA), 5 UNREVIEWED**. The narrow AAAA association PASS does not repair its wrong preservation set or prove a rewritten P2. No configuration establishes the full corrected T1/T3 plus P2 work product in the reviewed evidence.

## Actual screen and input meaning

Both actual inputs refer to the same authored DOC-A revision1 PNG. The native selected-only inventory contains T1 and T2 on both inputs; it does not silently change to T1/T3. The screenshot visibly places T1/T2/T3 across the top, P1 in the middle and P2 at the bottom.

- U01: `이 표들은 그대로 두고 이 문단 아니 저 문단만 고쳐줘.` The selected T1/T2 are preserved. The earlier point `(480,950)` is inside P1, and later point `(2080,1400)` is inside P2; the within-utterance correction ends at P2.
- U02: `보존할 표는 첫째와 이 표로 바꿔. 문단 수정은 유지해.` The native set is still context T1/T2. “첫째” is T1 and current point `(2370,450)` is inside T3. The required change is T1/T3 preservation with the same P2 revision request.
- DOC-A line5 itself says the paragraph should be revised into two clear sentences, so this detail in the handling is supported by the actual source. It is not introduced only by the descriptive oracle.
- This is an authored rendered screen and authored selected-object adapter, not evidence of a real Windows API implementation or a complete native-object inventory.

## Per-configuration outcome

| Config / run | U01 actual body | U02 actual body | A01 actual body | Member chain |
| --- | --- | --- | --- | --- |
| CAAA / `CAAA-S08-r2-80590f` | Correct handling intention | Missing | Not provided / missing | UNREVIEWED complete chain |
| ABAA / `ABAA-S08-r2-56bf10` | Correct handling intention | Missing | Not provided / missing | UNREVIEWED complete chain |
| AABA / `AABA-S08-r2-211a91` | Missing | Wrong goal / Task-TA claim; correct T1/T3 IDs | Faithful relay only | FAIL |
| AAAB / `AAAB-S08-r2-34d0c1` | Correct handling intention | Missing | Not provided / missing | UNREVIEWED complete chain |
| AAAC / `AAAC-S08-r2-ac890d` | Correct handling intention | Missing | Not provided / missing | UNREVIEWED complete chain |
| AAAA / `AAAA-S08-r2-0567fc` | Correct handling intention | Correct revision2; wrong T1/T2 | Faithful relay only | PASS narrowly |
| BAAA / `BAAA-S08-r2-c44e04` | Correct handling intention | Missing | Not provided / missing | UNREVIEWED complete chain |

### Six initial handling bodies

CAAA `/events/52`, ABAA `/events/39`, AAAB `/events/54`, AAAC `/events/38`, AAAA `/events/38` and BAAA `/events/35` identify P2 and retain T1/T2. Their adopted bindings map `이 표들` to T1/T2 and `저 문단` to P2. They are future handling intentions; no paragraph completion is inferred. Five initial raw meaning proposals use FOLLOWUP/self-U01 or NEW with proposed `task_id=TA`; actual runtime adoption owns newly assigned TASK-1. The raw proposal and actual assigned Task are kept separately in JSON.

### AABA: correct current IDs do not establish correction continuity

No U01 handling is displayed. The U01 binder first and after owner expansion leaves `저 문단` unresolved, requesting original-source expansion; `/events/128` records the eventual workflow failure. Meanwhile U02’s actual binder resolves `첫째` to T1 and `이 표` to T3 using `point-current`. Those correct IDs are not inherited as success for the next stages.

The actual U02 meaning request already includes both captured utterances but its `public_state.requests` is empty. Historical Task TA exists as an IDLE two-page report task. The proposal emits FOLLOWUP/self-U02 and says to update TA. Frozen `resolve_goal` could attach an explicit CORRECTION to captured U01, but this emitted relation instead creates `goal:S08-U02`; adoption `/events/73` assigns TASK-2 revision1. Display `/events/75` says:

> DOC-A:v1:T1, DOC-A:v1:T3: Updating preservation scope for Task TA to include Cost Table (T1) and Resource Table (T3) as specified by selection and pointer, while maintaining the paragraph revision instruction.

The TA identifier is real historical context, not a fabricated ID, but its claimed update is not the actual new TASK-2 assignment or required U01 correction. The actual U02 handoff targets only T1/T3 and retains generic paragraph context, not a concrete P2 target. The receiver returns `mode=COMPLETE` with “Preservation targets updated…” and `actions_performed=[]`; its cited facts are table content. This does not prove TA state modification or P2 revision. Actual A01 `/events/107` and display `/events/131` faithfully relay this synthetic result under TASK-2. The source is real within this harness, but its completion wording is not independent product verification.

### AAAA: correct same-goal revision with wrong referent

U02 adoption `/events/84` is CORRECTION of `goal:S08-U01`, TASK-1 revision2. Its body `/events/86` says:

> DOC-A:v1:T1, DOC-A:v1:T2, DOC-A:v1:P2: Updating task TASK-1 to preserve tables T1 (DOC-A:v1:T1) and T2 (DOC-A:v1:T2) while modifying paragraph P2 (DOC-A:v1:P2) into two clear sentences.

That is the old T1/T2 preservation set, not current T1/T3. The raw joint proposal labels `이 표` as the entire native selection T1/T2, which is truthful about the still-selected native objects but wrong for the current pointer-based user referent. Meaning then adopts `이 표 → T2`. No wire-check PASS can substitute for correct user designation.

The U02 ingress `/events/102` really carries revision2, but its mandatory target/constraints remain T1/T2/P2. The actual receiver requests P2 and returns the original T1/T2/P2 text with `actions_performed=[]`, not a rewritten two-sentence paragraph. A01 capture `/events/124` and display `/events/129` faithfully reproduce that actual synthetic result. Final HOST_COMPLETE is retained as host state; corrected preservation and domain work product are not fulfilled by it.

### Five absent corrections

CAAA, ABAA, AAAB, AAAC and BAAA each make two U02 joint-grounding attempts, ending with `Claimed complete native selection differs from actual selected objects`. Their offending binding claims singleton T2 (or separate singleton native selections) as a complete native selection, while the actual native cue still lists T1 and T2. Frozen `grounding/wire.rs` lines173–193 requires the entire declared native set. This observed rejection is consistent with that predicate. The current pointer on T3 is still separately available; the failed model relation is not a broken native selection input.

Their old U01 ingress and receiver calls do occur, including bounded source reads and model outputs. They must retain costs and raw work. They do not become the missing corrected-U02 ingress and do not supply the expected A01. There is no U02 adoption or actual A01 capture/display in these five runs.

## Timeout flags versus later actual Agent sources

All seven terminal summaries preserve `MISSING_INGRESS_OR_AGENT_ANCHOR` for A01. That historical flag is not the final supplied-source count: AABA and AAAA later actually provide and display A01. The other five remain unprovided. AABA also has U01 and U02 response-anchor timeouts; the five absent-correction runs have U02 response-anchor timeouts. These records are neither removed nor converted into fake Agent results.

| Config | Historical A01 missing-anchor ns | Actual A01 provided_at ns | Actual A01 display | Final A01 state |
| --- | ---: | ---: | --- | --- |
| CAAA | 1992628305645333 | None | None | Not provided |
| ABAA | 1992730496421083 | None | None | Not provided |
| AABA | 1992840909420250 | 1992858254239666 | /events/131 | Actually provided later |
| AAAB | 1992991197522791 | None | None | Not provided |
| AAAC | 1993095316080208 | None | None | Not provided |
| AAAA | 1993198886839375 | 1993214030361875 | /events/129 | Actually provided later |
| BAAA | 1993348034260541 | None | None | Not provided |

The fixture’s authored A01 sentence says corrected P2 completion with T1/T3. The runtime supplies actual receiver-derived text instead; missing source text is not filled from the fixture. The two actual relays therefore receive a narrowly stated fidelity PASS while corrected-product fulfillment remains separate.

## Authority, stop rules and limits

Actual DOC-A is locally and externally permitted in the supplied source registry. Observed routes delegate the requested document work using allowed `revise_report`; no new permission grant or CONTROL ingress is observed. This does not prove a physical document modification, real external sending or action approval; approval is NOT_APPLICABLE here. Preserved model `conditions_respected` claims are not independently verified edits.

All seven copied terminals report return code0, technical_failure=false and budget_stop=false. Source review of the native-set guard and `resolve_goal` explains the observed paths. Frozen harness lines295–299 retain response/missing-ingress anchor failures outside technical_failures; controller lines190–194 apply the existing technical/budget stop predicates. No new existing mandatory technical-stop condition or instrument contradiction was established by this bounded review.

No timing endpoint was selected, no QA02 set result recomputed, no denominator or failure filtered, and no source/prompt repair or rerun follows. Voice is OFF and callback records are host text proxies. Goal-relation PASS, raw correct IDs, faithful relay and HOST_COMPLETE remain separate from full request/product completion. No next cursor is authorized.
