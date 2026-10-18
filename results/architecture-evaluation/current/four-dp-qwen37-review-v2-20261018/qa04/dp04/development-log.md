# QA04 DP04 exercise development log

No provider calls. All changes are in independent calibration-v1 scratch copies; live baseline was untouched.

- C05 compiled and passed actual delivered/failed-proposal provenance checks for A/B.
- First C06 compile exposed an editing-script error: a broad text replacement added policy_epoch to begin() parameters as well as the struct. Corrected the generated scratch signature; the intended separate policy setter remained. Regression then passed.
- First C06 acceptance attempt: A passed; B failed because the test-only vision double incorrectly applied withdrawal filtering before the external authorization event and used visible candidate count/order for the word middle. Corrected the TEST HARNESS to apply filtering only after actual redaction metadata and to interpret middle by actual frame-center geometry. No production C06 Rust change was required for that correction. Repeated A/B acceptance then passed. Initial tool failure summaries are transcribed here; initial failed probe files were replaced during harness development and are not claimed as preserved raw model failures.
- Final C05:2 operating Rust modules; C06:5. Added real operation modules are counted. New tests/harness are excluded. Both alternatives tie; no B locality benefit manufactured.
- Actual PNG checks decode the produced masked image: a sample inside withdrawal is black and a disjoint pixel equals the original. Subsequent actual Rust model messages carry redacted view paths. The external mock still stands in for visual semantics and is not an actual model-accuracy result.
- These exercises are anchored to calibration-v1 freeze1ee6c0517894d1f69a080dd37b55341f21b3f3087a35ada31fcf4709a0144ba0. If formal baseline changes, apply relationship/acceptance must be checked against that generation and old evidence retained.
