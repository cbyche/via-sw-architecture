# Reporting helper v3 r2 checkpoint

All changes are outside the frozen campaign. No actual progress prefix, closure index or trace extraction has run.

Stage-01 remains immutable. The three independent findings are corrected: present bool-or-null terminal flags retain null as unknown; ordinary cursors require exact parent advancement authorization joining freeze/ordinal/block/index/bundle/review; all data reads reserve verified size/requested maximum before I/O and retain attempted/returned byte diagnostics. Reader growth no longer causes a limit+1 read.

34 tiny synthetic tests PASS in less than one second. One intermediate growth-test failure was a temporary-path alias in the test fixture (/var versus /private/var); normalizing that synthetic root exercised the intended actual growth path. Exact failed test source/log hashes are reconstructed and verified in failed-test-generation, with all original synthetic records retained. This was not a campaign result or runtime failure.

No actual extraction is authorized by this checkpoint; Astra source review and separate root GO remain required.
