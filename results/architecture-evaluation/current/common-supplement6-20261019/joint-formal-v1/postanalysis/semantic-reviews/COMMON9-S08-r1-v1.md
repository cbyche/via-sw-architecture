# S08 r1 bounded semantic review

- Reviewer: GPT-6 Astra Ultra, `/root/method_astra_review`.
- Block: `PRIMARY-r1-S08`, frozen ordinal 2, order BAAA, CAAA, ABAA, AABA, AAAB, AAAC, AAAA.
- Review JSON: [COMMON9-S08-r1-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON9-S08-r1-v1.json); SHA-256 `24645051ef2fe3dc391ef88b4832249cb47d14b2ed6b83150089486ff310d300`.
- Accepted bundle manifest SHA-256 `988bd1a901188e90d83f13101757b7ee6805fb1af79f30fa5cd4f5491f3f47b9`.
- Frozen campaign SHA-256 `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd`.

## Scope and coverage

This is a postlaunch manual diagnostic of actual content, request association, observed authority and handoff. It does not recompute QA01, QA02 or QA03 and supplies no semantic filter for their frozen timings or denominators. No saved evaluation was used. The evidence bundle remains UNREVIEWED measurement evidence.

All **21 response obligations and 7 goal relations** are present. Seven handling bodies were observed. Their core target and constraint content passes in seven cases; full-body consistency is **5 PASS, 2 FAIL, 14 UNREVIEWED**. The 14 missing bodies also remain `NOT_OBSERVED` delivery obligations. The seven required final correction relations are unmet. No actual S08-A01 input or completed edit body was observed. Approval is not applicable, never counted as PASS.

## Actual requirement and input

U01 says `이 표들은 그대로 두고 이 문단 아니 저 문단만 고쳐줘.` The frozen PNG shows native selection T1/T2 and separate P1/P2 paragraphs. The later U01 point is (2080,1400), inside P2. U02 says `보존할 표는 첫째와 이 표로 바꿔. 문단 수정은 유지해.` Its point-current is (2370,450), inside T3, while native selection still exports T1/T2. Thus the correction is T1/T3 preservation with P2 editing retained. This interpretation is supported by the actual utterances, image and cues, not an oracle-only added field.

The six rejected joint paths received the actual point-current cue in both raw model requests. Both attempts chose `이 표` = T2 and claimed a one-item native selection, which the frozen exact-selection validator rejected. The raw target is also wrong independently of that validator. No impossible reference schema or missing pointer input was established.

## All planned response rows

| Config | Obligation | Raw display | Core content | Full body | Detail |
| --- | --- | --- | --- | --- | --- |
| BAAA | S08-RU01 | event-45 /events/36 | PASS | FAIL | TA differs from adopted Task |
| BAAA | S08-RU02 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | No adopted response body |
| BAAA | S08-RA01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | Actual Agent input absent |
| CAAA | S08-RU01 | event-64 /events/52 | PASS | PASS | Requested handling scope retained |
| CAAA | S08-RU02 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | No adopted response body |
| CAAA | S08-RA01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | Actual Agent input absent |
| ABAA | S08-RU01 | event-48 /events/39 | PASS | PASS | Requested handling scope retained |
| ABAA | S08-RU02 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | No adopted response body |
| ABAA | S08-RA01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | Actual Agent input absent |
| AABA | S08-RU01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | No adopted response body |
| AABA | S08-RU02 | event-100 /events/84 | PASS | FAIL | TA differs from adopted Task |
| AABA | S08-RA01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | Actual Agent input absent |
| AAAB | S08-RU01 | event-65 /events/54 | PASS | PASS | Requested handling scope retained |
| AAAB | S08-RU02 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | No adopted response body |
| AAAB | S08-RA01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | Actual Agent input absent |
| AAAC | S08-RU01 | event-45 /events/37 | PASS | PASS | Requested handling scope retained |
| AAAC | S08-RU02 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | No adopted response body |
| AAAC | S08-RA01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | Actual Agent input absent |
| AAAA | S08-RU01 | event-46 /events/38 | PASS | PASS | Requested handling scope retained |
| AAAA | S08-RU02 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | No adopted response body |
| AAAA | S08-RA01 | NOT_OBSERVED | UNREVIEWED | UNREVIEWED | Actual Agent input absent |

## Goal relation and handoff

| Config | Adopted request | Required correction relation | Actual ingress | A01 |
| --- | --- | --- | --- | --- |
| BAAA | goal:S08-U01, r1, TASK-1 | FAIL | S08-U01, event-92 | Not provided |
| CAAA | goal:S08-U01, r1, TASK-1 | FAIL | S08-U01, event-112 | Not provided |
| ABAA | goal:S08-U01, r1, TASK-1 | FAIL | S08-U01, event-95 | Not provided |
| AABA | goal:S08-U02, r1, TASK-2 | FAIL | S08-U02, event-116 | Not provided |
| AAAB | goal:S08-U01, r1, TASK-1 | FAIL | S08-U01, event-117 | Not provided |
| AAAC | goal:S08-U01, r1, TASK-1 | FAIL | S08-U01, event-56 | Not provided |
| AAAA | goal:S08-U01, r1, TASK-1 | FAIL | S08-U01, event-98 | Not provided |

Six configurations retain U01 revision 1 and its T1/T2 preservation after U02 grounding fails. Their receiver acceptance records concern that U01 contract; they are not A01 completion facts. AABA adopts U02 with the correct T1/T3/P2 content but a self-targeted FOLLOWUP, new `goal:S08-U02`, revision 1 and TASK-2. U01 never adopts. Its failure is the missing retained association, not the number 1 by itself.

AABA records the A01 missing-anchor boundary at `1980975395614625` ns. U02 ingress is observed at `1980994456981791` ns. The later ingress does not revive the missing event. The receiver then records `OBSERVATION_DEADLINE_CANCELLED_NO_STATE_EFFECT`, and U01 is cancelled with the same reason. The full raw delay and failure remain.

## Literal body and state discrepancies

- BAAA displays: `DOC-A의 P2 문단을 두 문장으로 명확하게 수정하는 작업을 TA 태스크에 위임합니다. 표 T1과 T2는 변경하지 않습니다.` Accepted proposal event-38 names TA, but adoption event-41 and ingress event-92 use TASK-1. The target content is correct; the literal Task statement is inconsistent with the recorded action.
- AABA displays: `Updating task TA to preserve Cost Table (T1) and Resource Table (T3) while applying paragraph modification to P2 as instructed.` Accepted proposal event-93 names TA, but adoption event-96 and ingress event-116 use TASK-2. Core content is correct; the Task statement and retained request association are not established.
- These are observed output/state discrepancies. This review does not assign their cause solely to the model or solely to runtime remapping.

## Authority and limits

DOC-A revision 1 is explicitly allowed for local and external use. The observed contracts request revision work, with no send recipient, completed action, control operation or permission elevation. The six non-AAAA handoffs carry `revise_report`; AAAA carries an empty allowed-actions list, retained as an unresolved handoff permission-completeness diagnostic rather than an invented external action. No actual business edit, unauthorized send, approval decision or final Agent artifact is inferred.

No completed result exists with which to exercise stale-result rejection; this is `NOT_EXERCISED`, not PASS. The primary endpoint, technical eligibility, QA02 scores and full QA03 result remain for the frozen final evaluator. All exact event IDs, trace hashes, source references, raw request/response pairs and per-row explanations are in the JSON.

No new technical defect was established in this bounded review. There is no recommendation to stop, retune, change a contract or rerun based on these failures. No runtime, frozen file, parser, evaluator, analyzer, build, test or API was executed or changed.

## Next bounded review

Ordinal 3, S83, is the next suggested block. Advancing requires a new explicit root authorization and exact index/bundle. This review does not advance the cursor.
