# 최종 effective120 QA04 근거·독립 검토

QA04 for effective main120: original DP01/02/04 plus corrected DP05 only. Four predeclared equal-weight tasks. No tactic/diagnostic QA04 inheritance and no architecture-wide/semantic/performance claim.

|DP/안|C01|C03|새 기능1|새 기능2|네 과제 주평균|
|---|---:|---:|---:|---:|---:|
|01/A|1|2|1 C05|3 C06|1.75|
|01/B|1|2|1 C05|3 C06|1.75|
|01/C|1|2|1 C05|4 C06|2.00|
|02/A|1|2|2 C04|3 C06|2.00|
|02/B|1|2|2 C04|3 C06|2.00|
|04/A|1|2|2 C05|5 C06|2.50|
|04/B|1|2|2 C05|5 C06|2.50|
|05/A|1|2|1 C04|2 C06|1.50|
|05/B|1|2|1 C04|3 C06|1.75|
|05/C|1|2|1 C04|2 C06|1.50|

## 실제 baseline 관계

- 원래01/02/04: basic-v1 freeze `50e4116e1fa86807c19abecac6586110e88968087542eececcfc4e379a7cecf2`.
- 교정05: basic-dp05-reuse-union-corrected-v2 freeze `ed3fb4ad408541ab1acc7522ec036cf1541ac3d8e4a65ec7f34ab65a29d10981`.
- 교정 crate와 원래 crate의 Rust 차이는 dp05.rs/dp05_handoff.rs/dp05_sources.rs뿐이며, controls의 실제 고정 baseline inventory도 교정freeze의 effective_candidate_source와 일치했다. manifest.source는 그대로인 원래 baseline guard이며 실제 교정 코드는 별도 effective map으로 고정된다.
- DP04는 최종 relation 계약 위 독립 C05/C06 A/B scratch를 재base·실행했고, 기본freeze 동일성은 applicability JSON으로 보존했다.
- DP01은 guard-v2 scratch와 최종 운영모듈 동일성/전이 근거를 연결한다. DP02 common adapter/feature 전이 근거도 별도 보존한다.
- 교정05는 C01/C03 actual6 offline, C04/C06 actual6 offline가 별도 source/diff/원시로 보존된다. 초기 ps 차단은 후보 시작 전 인프라 실패로 별도 보존했으며 수용 PASS로 바꾸지 않았다.

## 독립 검토 범위

DP04 independently implemented/executed functional scratch changes; corrected C01/C03 actual six patched Rust runs independently executed. DP01/05 initial ancillary-text issue independently read/rechecked then owner rebased; owner feature acceptance results are linked, not claimed as newly independently rerun here. Final corrected source3 was independently read/hash-checked.

개별안에 실제 필요한 변경 Rust 모듈만 세었고 새 test-only probe/harness는 제외했다. 초기 cal-v1 ancillary 문구 재주입 finding과 그 수정/검사 이력은 삭제하지 않는다. 새 기능만의 pair 평균이 네 과제 주평균을 대신하지 않는다. 보완 A2/A4/B4 및 번호 진단에는 이 QA04를 자동 상속하지 않는다.

## 상세 근거

- [qa04/summary.json](summary.json)
- [qa04/reuse-union-corrected-dp05-four-task-summary.json](reuse-union-corrected-dp05-four-task-summary.json)
- [qa04/controls/acceptance-summary.json](controls/acceptance-summary.json)
- [qa04/common-dp02-final-baseline-transition.json](common-dp02-final-baseline-transition.json)
- [qa04/baseline-v2/dp01-C05/summary.json](baseline-v2/dp01-C05/summary.json)
- [qa04/baseline-v2/dp01-C06/summary.json](baseline-v2/dp01-C06/summary.json)
- [qa04/dp04-rebased-calibration-v3/summary.json](dp04-rebased-calibration-v3/summary.json)
- [qa04/dp04-rebased-calibration-v3/basic-v1-applicability.json](dp04-rebased-calibration-v3/basic-v1-applicability.json)
- [qa04/controls-reuse-union-corrected-v2/acceptance-summary.json](controls-reuse-union-corrected-v2/acceptance-summary.json)
- [qa04/baseline-dp05-reuse-union-corrected-v2/dp05-C04/summary.json](baseline-dp05-reuse-union-corrected-v2/dp05-C04/summary.json)
- [qa04/baseline-dp05-reuse-union-corrected-v2/dp05-C06/summary.json](baseline-dp05-reuse-union-corrected-v2/dp05-C06/summary.json)
- [analysis/qa04-cross-review-payload-checks.json](../analysis/qa04-cross-review-payload-checks.json)
- [analysis/final-review/dp05-common-retention-final-prefix-source/review.json](../analysis/final-review/dp05-common-retention-final-prefix-source/review.json)

구체적 source SHA·task별모듈·수용 링크는 final-effective-cross-review.json. 기존수치/원시/모델호출 변경0.
