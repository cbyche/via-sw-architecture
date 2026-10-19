# Fresh QA04 after deadline and protocol-metadata correction

Status: PREPARATION ONLY, awaiting the exact final candidate/benchmark freeze. No candidate copies, builds, tests or acceptance have run in this generation.

The earlier qa04/ and qa04-protocol-v2/ generations remain preserved with their own exact baselines, failures and accepted results. Neither their counts nor their saved regressions are imported here. Formal-v1/v2 actual model failures also remain unchanged.

Apply the identical C01–C06 requirements and operational Rust module-count method to7configurations ×6independent whole-baseline copies, after the execution lead supplies final freeze-ready. Rebase patch anchors only against that immutable source. Include necessary parent module registration and all real operating-code changes; do not force a tie or omit common runtime changes. Tests/comments/whitespace only remain excluded.

The deadline/cancel gate and catalog-derived valid-reference metadata are common baseline obligations. Feature patches must preserve that baseline, including its required effect gates, and must not bypass deadline/authority checks. API calls remain0 for QA04; acceptance uses deterministic doubles and finite task checks plus common9. No execution occurs during any paid timing window.

Results will be written to results/architecture-evaluation/current/common9-poc-20261019/qa04-method-v3/. The initial tools are copies of the previously reviewed purpose-equivalent patches; any new anchor adaptation and its actual source diff/build/acceptance must be measured freshly. Module counts and means are unknown until completion.
