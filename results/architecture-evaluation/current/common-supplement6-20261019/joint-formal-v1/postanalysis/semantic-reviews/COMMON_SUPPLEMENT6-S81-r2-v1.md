# COMMON_SUPPLEMENT6 S81 r2 bounded semantic review v1

## Scope and binding

GPT-6 Astra Ultra, `/root/method_astra_review`. Fixed ordinal 16, `SUPPLEMENT-r2-S81`, jobs 112–118. Seven already closed runs, fourteen response obligations and fourteen goal relations were reviewed independently from their actual r2 records. Earlier S80/S81 interpretation guidance was retained; no earlier outcome was inherited.

No raw trace was reread, corpus extracted, evaluator/analyzer run, model/parser/API/test/build executed, or frozen artifact changed. Event pointers below refer to the accepted bundle’s exact original trace identities.

| Artifact | SHA-256 |
| --- | --- |
| [bundle_manifest](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S81-r2-v3-r2/manifest.json) | `77de17e074c1d4923e720741ad2cf8db5f0df339b5894191837cb24458632ee7` |
| [evidence](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S81-r2-v3-r2/evidence.json) | `7a460a59b0c6bb5c567129a21f2841cec3afa240caa30d24395689fa54be1953` |
| [worklist](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON_SUPPLEMENT6-S81-r2-v3-r2/worklist.json) | `21db10349708f06f2fd43750f7d681646ce87eb092bb2ddc8d4988d13646a36d` |
| [index](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON_SUPPLEMENT6-S81-r2-v3-r2/index.json) | `bb57899b81499cdfff5eeaa8f0d077bfcd182855cf3b87d0e3687e318f7b2ac8` |
| [execution_receipt](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON_SUPPLEMENT6-S81-r2-v3-r2/EXECUTION-RECORD.json) | `34552ab8e487a384530cbe06343cb16ccf6d3620c68bbf5cbf54edf800f1de2e` |
| [freeze](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json) | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| [fixture](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/fixtures.json) | `d53502887c7305b99428eab8f4d391821ba2c6d5d4b77c20d7fa1176f1c92e97` |
| [oracle](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/oracles.json) | `39372c8c56febc390753204ef50c199d479c8d06ecb3ede6191788e88d1bb833` |
| [functional_oracle](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/functional-oracles.json) | `484d1f00c708bea7dc010c54e37d98831b77ee77c188d1c6be888da89484ffaa` |

Full immutable row evidence: [COMMON_SUPPLEMENT6-S81-r2-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON_SUPPLEMENT6-S81-r2-v1.json), SHA-256 `e896e4981e051ec8d6a47bec5a78bac68c46596d7882978bbee38c01418145df`. All 28 planned keys occur exactly once; no missing or duplicate key.

## Scoped counts

| Dimension | Count |
| --- | --- |
| Actual requested full body | 11 PASS, 3 FAIL, 0 UNREVIEWED |
| Distinct observed display events | 14 |
| Goal member/current revision association | 14 PASS; not additional independent body successes |
| Recorded HOST_COMPLETE | 14; includes the 3 incomplete bodies |
| U01 release blockers and redaction/review next step | 7 preserved |
| U01 support check and release-time adjustment | 4 preserved, 3 omitted |
| Literal Monday12 detail | Absent in all 7; not by itself the full-body failure criterion |
| U02 Wednesday14 and room B | 7 PASS |
| Actual-final sequential anchor | 4 |
| Missing-anchor fallback, sequential arrival unrealized | 3 |
| First-proposal support rejection | 3, preserved with bounded within-run repair |
| Self-target ANSWER plan labels | 4, separate from actual correct goal mapping |
| Ancillary exact-spoken binding deviation | 14 adopted plans |
| Missing / superseded / duplicate actual bodies | 0 / 0 / 0 |

## Actual source and criterion

Actual U01: “공개 검토 문서의 여덟 항목을 대조해 월요일10시 외부 공개안을 진행할 수 있는지, 막는 조건과 다음 조치를 네 문장 이내로 설명해줘. 실제 수정이나 발송은 하지 마.” Actual U02 asks the Wednesday inspection meeting’s place and start time. Every run’s actual meaning input contains all eight SUP-RELEASE lines for U01, plus the meeting source; U02 receives the meeting original. Both sources have registry revision 1, local access allowed and external sharing disallowed. “초안 r2” is document content, not registry revision 2.

P02 states that personal email remains and no external cleaned version has been prepared. P03/P04 require both approvals, with security approval complete but cleaned-version review incomplete. P05 says support begins Monday12 despite the proposed Monday10 release. P07 requires email removal, review, support-availability checking and release-time adjustment. These facts and the request to compare all eight items support the timing-condition criterion prospectively used for S80/S81.

Literal `월요일12시` need not appear when the answer preserves the support-availability condition and corresponding scheduling adjustment. CAAA, AAAB, AAAC and BAAA do preserve those conditions. ABAA, AABA and AAAA omit both, so their otherwise correct blocker/redaction/review answer is only partial fulfillment and receives full-body FAIL. This is not an oracle-only invented extra and not a new numeric score.

## Per-configuration bodies and member relations

| Configuration / run | U01 full body | U02 body | Member relations | Arrival rule actually realized |
| --- | --- | --- | --- | --- |
| CAAA / `CAAA-S81-r2-fc5497` | PASS | PASS | 2 PASS | U01 actual final + 2s |
| ABAA / `ABAA-S81-r2-a03036` | FAIL: timing condition omitted | PASS | 2 PASS | U01 actual final + 2s |
| AABA / `AABA-S81-r2-965394` | FAIL: timing condition omitted | PASS | 2 PASS | 30s fallback; U02 displayed first |
| AAAB / `AAAB-S81-r2-6a6392` | PASS | PASS | 2 PASS | 30s fallback; U02 displayed first |
| AAAC / `AAAC-S81-r2-b86a73` | PASS | PASS | 2 PASS | 30s fallback; U02 displayed first |
| AAAA / `AAAA-S81-r2-70d622` | FAIL: timing condition omitted | PASS | 2 PASS | U01 actual final + 2s |
| BAAA / `BAAA-S81-r2-553332` | PASS | PASS | 2 PASS | U01 actual final + 2s |

### CAAA

**U01 — PASS**. Display `/events/50` / `event-61`; adoption `/events/48`; actual input `/events/18`.

> 월요일 10시 외부 공개는 진행할 수 없다. 보안 승인은 완료되었으나 정리본 검토가 미완료이며, 초안에는 개인 이메일 주소가 남아 있어 P03과 P04 조건을 충족하지 못한다. 다음 조치는 개인 이메일 주소를 제거한 외부 공개용 정리본을 작성하고 검토를 요청하는 것이다. 외부 공개 시각은 검토 완료와 배포 지원 가능 시간을 확인한 후 조정해야 한다.

The answer preserves no-release decision, email removal, pending review, support-availability check and release-time adjustment within four sentences. Literal Monday12 is absent but omission alone does not override the preserved timing condition.

**U02 — PASS**. Display `/events/98` / `event-118`; adoption `/events/96`; actual input `/events/77`.

> 수요일 점검 회의는 14시에 3층 회의실 B에서 시작한다.

Wednesday 14:00 and third-floor meeting room B match the actually supplied meeting notice.

### ABAA

**U01 — FAIL**. Display `/events/34` / `event-41`; adoption `/events/32`; actual input `/events/6`.

> 월요일 10시 외부 공개는 진행할 수 없다. 개인 이메일 주소가 포함된 초안만 존재하며 외부 공개용 정리본이 아직 작성되지 않았기 때문이다. 또한 보안 승인은 완료되었으나 정리본 검토가 미완료되어 공개 조건을 충족하지 못한다. 다음 조치는 개인 이메일 주소를 제거한 외부 공개용 정리본을 작성하고 검토를 요청하는 것이다.

Correct no-release decision and redaction/review conditions are only partial fulfillment: the displayed answer omits deployment-support availability and corresponding release-time adjustment, despite actual P05/P07 and the request to compare the eight items.

**U02 — PASS**. Display `/events/70` / `event-81`; adoption `/events/68`; actual input `/events/46`.

> 수요일 점검 회의는 14시에 3층 회의실 B에서 시작한다.

Wednesday 14:00 and third-floor meeting room B match the actually supplied meeting notice.

### AABA

**U01 — FAIL**. Display `/events/77` / `event-89`; adoption `/events/74`; actual input `/events/6`.

> 월요일 10시 외부 공개는 진행할 수 없다. 보안 승인은 완료되었으나 개인 이메일이 제거된 외부 공개용 정리본의 검토가 미완료되어 P03의 승인 조건을 충족하지 못한다. 또한 현재 초안 r2에는 개인 이메일이 남아 있어 P01 원칙에 위배된다. 다음 조치는 개인 이메일을 제거한 외부 공개용 정리본을 작성하고 검토를 요청하는 것이다.

Correct no-release decision and redaction/review conditions are only partial fulfillment: the displayed answer omits deployment-support availability and corresponding release-time adjustment, despite actual P05/P07 and the request to compare the eight items.

**U02 — PASS**. Display `/events/61` / `event-71`; adoption `/events/59`; actual input `/events/34`.

> 수요일 점검 회의 장소는 3층 회의실 B이며, 시작 시각은 14시입니다.

Wednesday 14:00 and third-floor meeting room B match the actually supplied meeting notice.

### AAAB

**U01 — PASS**. Display `/events/98` / `event-112`; adoption `/events/94`; actual input `/events/20`.

> 월요일 10시 외부 공개는 진행할 수 없다. 개인 이메일 주소가 포함된 초안 r2를 그대로 사용할 수 없으며, 외부 공개용 정리본이 아직 작성되지 않았기 때문이다. 보안 승인은 완료되었으나 정리본 검토가 미완료되어 승인 조건을 충족하지 못한다. 다음 조치는 이메일을 제거한 정리본을 작성하고 검토를 요청한 후, 지원 가능 시각과 함께 일정을 재조정하는 것이다.

The answer preserves no-release decision, email removal, pending review, support-availability check and release-time adjustment within four sentences. Literal Monday12 is absent but omission alone does not override the preserved timing condition.

**U02 — PASS**. Display `/events/81` / `event-95`; adoption `/events/79`; actual input `/events/54`.

> 수요일 점검 회의 장소는 3층 회의실B이며, 시작 시각은 14시입니다.

Wednesday 14:00 and third-floor meeting room B match the actually supplied meeting notice.

### AAAC

**U01 — PASS**. Display `/events/77` / `event-89`; adoption `/events/74`; actual input `/events/6`.

> 월요일 10시 외부 공개는 진행할 수 없다. 보안 승인은 완료되었으나, 개인 이메일이 포함된 초안 대신 정리본 검토가 필수이며 이는 아직 미완료 상태다. 따라서 먼저 개인 이메일을 제거한 외부 공개용 정리본을 작성하고 검토를 요청해야 한다. 외부 공개 시각은 정리본 검토 완료와 배포 지원 가능 시간을 확인한 후 조정해야 한다.

The answer preserves no-release decision, email removal, pending review, support-availability check and release-time adjustment within four sentences. Literal Monday12 is absent but omission alone does not override the preserved timing condition.

**U02 — PASS**. Display `/events/61` / `event-71`; adoption `/events/59`; actual input `/events/34`.

> 수요일 점검 회의 장소는 3층 회의실 B이며, 시작 시각은 14시입니다.

Wednesday 14:00 and third-floor meeting room B match the actually supplied meeting notice.

### AAAA

**U01 — FAIL**. Display `/events/34` / `event-41`; adoption `/events/32`; actual input `/events/6`.

> 월요일 10시 외부 공개는 진행할 수 없다. 현재 초안(r2)에 개인 이메일 주소가 남아 있고(P02), 외부 공개용 정리본 검토가 미완료이기 때문이다(P04). 보안 승인만으로는 부족하며 정리본 검토 완료와 이메일 제거가 필수 조건이다(P03, P07). 다음 조치는 개인 이메일을 제거한 외부 공개용 정리본을 작성하고 검토를 요청하는 것이다(P07).

Correct no-release decision and redaction/review conditions are only partial fulfillment: the displayed answer omits deployment-support availability and corresponding release-time adjustment, despite actual P05/P07 and the request to compare the eight items.

**U02 — PASS**. Display `/events/70` / `event-81`; adoption `/events/68`; actual input `/events/46`.

> 수요일 점검 회의 장소는 3층 회의실 B이며, 시작 시각은 14시다.

Wednesday 14:00 and third-floor meeting room B match the actually supplied meeting notice.

### BAAA

**U01 — PASS**. Display `/events/31` / `event-38`; adoption `/events/29`; actual input `/events/6`.

> 월요일 10시 외부 공개는 진행할 수 없다. 보안 승인은 완료되었으나 정리본 검토가 미완료이며, 초안에는 개인 이메일 주소가 남아 있어 P03과 P04 조건을 충족하지 못한다. 다음 조치로 개인 이메일을 제거한 외부 공개용 정리본을 작성하고 검토를 요청해야 한다. 외부 공개 시각은 검토 완료와 배포 지원 가능 시간을 확인한 후 조정한다.

The answer preserves no-release decision, email removal, pending review, support-availability check and release-time adjustment within four sentences. Literal Monday12 is absent but omission alone does not override the preserved timing condition.

**U02 — PASS**. Display `/events/66` / `event-78`; adoption `/events/64`; actual input `/events/43`.

> 수요일 점검 회의는 3층 회의실 B에서 14시에 시작한다.

Wednesday 14:00 and third-floor meeting room B match the actually supplied meeting notice.

## Sequential anchor and late final are distinct

The selected anchor is not inferred from body correctness. In four runs it exactly equals the recorded U01 `RESPONSE_FINAL.observed_ns`. In AABA, AAAB and AAAC it instead equals U01’s actual successful provision time plus 30 seconds, matching the copied terminal’s `RESPONSE_ANCHOR_TIMEOUT`. U02 is due two seconds after that chosen fallback boundary. The late U01 final does not replace the chosen fallback or retrospectively make the sequential condition true. Both eventual bodies remain observed and all obligations remain in the population.

| Config | Selected U02 anchor ns | Actual U01 final ns | U01 final pointer | U02 actual provided_at ns | Display order |
| --- | ---: | ---: | --- | ---: | --- |
| CAAA | 1991943202943166 | 1991943202943166 | `/events/56` | 1991945213166791 | S81-U01 → S81-U02 |
| ABAA | 1992032165913458 | 1992032165913458 | `/events/40` | 1992034167890500 | S81-U01 → S81-U02 |
| AABA | 1992129372159250 | 1992148040814583 | `/events/83` | 1992131383595750 | S81-U02 → S81-U01 |
| AAAB | 1992226575540250 | 1992251951117083 | `/events/108` | 1992228579870333 | S81-U02 → S81-U01 |
| AAAC | 1992323825242416 | 1992345296656375 | `/events/83` | 1992325831089708 | S81-U02 → S81-U01 |
| AAAA | 1992408992066000 | 1992408992066000 | `/events/40` | 1992410993062916 | S81-U01 → S81-U02 |
| BAAA | 1992494456462833 | 1992494456462833 | `/events/37` | 1992496461948166 | S81-U01 → S81-U02 |

The timeout records’ `late_actual_final_ns=null` describes the observation when the fallback was recorded. Later actual final events are retained separately in this review; null is not asserted to mean no final ever occurred. No call overlap, free K2 capacity or provider compute concurrency is inferred from this display ordering.

## Proposal, adoption and authority diagnostics

AABA `/events/29`, AAAB `/events/49` and AAAC `/events/29` reject the initial U01 proposal with `DIRECT_ACTUAL_SUPPORT_VALIDATION_FAILED`. Their raw actual model requests contain both originals, and the quoted release claims inspected are present. The precise mismatch is that the proposals declare both `SUP-MEETING` and `SUP-RELEASE` in `source_revisions` but provide claims only for SUP-RELEASE. Frozen `runtime.rs` lines 922–944 require a matching supplied quote claim for each declared source revision. This explains the rejection through the existing support predicate. The successful later proposals narrow the revision map. These are actual frozen bounded repairs within the runs, not new attempts requested by this review.

ABAA, AABA, AAAB and AAAC adopt U01 with `relation=ANSWER`, `target_event_id=S81-U01`; the actual resulting goal is still its own `goal:S81-U01`, origin/last event U01 and revision 1. The label is retained explicitly and is not presented as evidence of an earlier VIA clarification. Every U02 adopts NEW into its own distinct goal. No cross-request merge or stale revision is observed here.

All fourteen adopted plans contain at least one binding expression that is source wording or a resolved value rather than an exact substring of the actual current utterance. This is a separate deviation from the meaning instruction. Source-target and factual-body correctness do not erase it; S81 has no frozen QA02 set obligations, so no new set denominator or QA01 semantic gate is added.

All actual adopted routes are DIRECT, Task IDs null, permission/control operation arrays empty. Final ingress maps are empty and no actual permission application, control ingress or Agent ingress event is observed. Registry local access is not external export consent. The user’s no-modification/no-send boundary is preserved in the observed path; real external work, approvals or physical product execution are NOT_APPLICABLE/NOT_EXERCISED.

## Stop-rule assessment and limits

All seven copied terminal records have return code 0, technical_failure false and budget_stop false. Their three anchor failures are preserved. Frozen supplemental harness line 295 excludes RESPONSE_ANCHOR_TIMEOUT from technical_failures; contract20 lines 48–50 explicitly retain missing-anchor fallback as a failed condition, not a fabricated final. The shared controller lines 190–194 stop on nonzero exit, technical failure or budget stop. This bounded review establishes no new existing mandatory technical-stop condition and does not independently certify clocks or endpoint eligibility.

The 3 full-body failures, 3 proposal rejections, 3 fallback paths and ancillary binding deviations remain visible. HOST_COMPLETE and goal-association PASS do not convert those bodies into full fulfillment. No frozen endpoint, denominator, raw failure, cost or memory value is filtered or changed. Voice is OFF and text endpoints are host callback proxies. No tuning, rerun, controller action or next cursor is authorized by this note.
