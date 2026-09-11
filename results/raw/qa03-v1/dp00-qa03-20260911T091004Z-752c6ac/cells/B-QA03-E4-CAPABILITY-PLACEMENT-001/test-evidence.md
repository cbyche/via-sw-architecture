# B-QA03-E4-CAPABILITY-PLACEMENT-001

- Campaign: `dp00-qa03-20260911T091004Z-752c6ac`
- Contract source: `752c6ac66fd0a77d7d6efac00d46ad7277bc5247`
- Architecture baseline: `d4059eccd2883b3029b1e8c94fc86d092a5041f8`
- Acceptance oracle commit: `fb551adb53049fce0dac492308dc7b87f4f5520f`
- Result commit: `36e43db12acfbcad2fb0e26d27c5f260d69e8415`
- Initial oracle outcome: FAIL at LOCAL_DIRECT assertion
- Final acceptance: FAIL (`ACC-E4-LOCAL`)
- Regression: PASS; no new or worsened failures; P12 baseline-equivalent
- Rust workspace tests/check/fmt/clippy: PASS for the frozen regression set
- Python architecture-baseline suite: 95/95 PASS
- Pre-existing defects retained: `qa02:P04`, `qa02:P06`, `qa02:P07`, `qa02:P11`, `qa04:P04_REQUIRED_ROUTE_NOT_COMMITTED`, `qa04:P06_REQUIRED_ROUTE_NOT_COMMITTED`, `qa04:P07_REQUIRED_ROUTE_NOT_COMMITTED`, `qa04:P11_FORBIDDEN_ROUTE_COMMITTED`
