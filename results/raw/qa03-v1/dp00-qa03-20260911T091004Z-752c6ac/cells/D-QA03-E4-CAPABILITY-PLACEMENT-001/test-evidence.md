# D-QA03-E4-CAPABILITY-PLACEMENT-001

- Campaign: `dp00-qa03-20260911T091004Z-752c6ac`
- Contract source: `752c6ac66fd0a77d7d6efac00d46ad7277bc5247`
- Architecture baseline: `d4059eccd2883b3029b1e8c94fc86d092a5041f8`
- Acceptance oracle commit: `a458b5694b20d6a88500dd239bab8b3588b56342`
- Result commit: `ba9f823edc859beee920a854d5f9c3e3ab1a02fd`
- Initial oracle outcome: FAIL at LOCAL_DIRECT assertion
- Final acceptance: PASS
- Regression: PASS; no new or worsened failures; P12 baseline-equivalent
- Rust workspace tests/check/fmt/clippy: PASS for the frozen regression set
- Python architecture-baseline suite: 95/95 PASS
- Pre-existing defects retained: `qa02:P04`, `qa02:P07`, `qa02:P11`, `qa04:P04_QA02_EXCLUDED`, `qa04:P07_QA02_EXCLUDED`, `qa04:P11_FORBIDDEN_ROUTE_COMMITTED`
- Iteration note: an uncommitted early D change reduced the P11 model-call signature and was rejected; the committed result preserves the selector call.
