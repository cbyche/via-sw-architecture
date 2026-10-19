# Helper v2: bounded repair handoff

V1 is NOT READY for actual extraction. Its six exact original files and14-check PASS record remain unchanged in preparation/source-v1/. Snapshot manifest SHA256:251340643634f015450bb7fb93cdf3b2292a64bdb4b71264957dfd90a3b06468. A synthetic PASS is not independent source acceptance; v1's later static findings are preserved here.

| V1 finding | V2 repair | Synthetic evidence |
| --- | --- | --- |
| A matching display with unknown clock sorted after known clocks could yield a false raw first | Any matching clock that is missing, non-integer or boolean makes raw-first ambiguous; no selected endpoint is manufactured | test_any_missing_or_noninteger_callback_clock_is_ambiguous |
| Substring CLOSED accepted NOT_CLOSED or unsupported summaries | Exact initial S02 status, root-relative file, population/episode/rep and count/rows shape only; missing rows remain explicit UNREVIEWED | test_closed_substring_and_unsupported_status_rejected; test_supported_status_with_wrong_shape_rejected; test_supported_status_with_wrong_file_rejected |
| Identity/hash-only saved evaluation lacked trace/evaluator provenance | Reject every saved-evaluation input as unsupported; initial extraction has no certified endpoint or primary eligibility, and no new strict-envelope parser or BG same-stimulus rule | test_saved_evaluation_provenance_is_unsupported; test_missing_saved_evaluation_explicit |
| Root JSON Pointer was slash instead of empty string | Empty root pointer and normal escaping | test_root_json_pointer_is_empty |
| review.md missing from output hashes | Hash exact UTF-8 review bytes and size before final manifest write | test_no_base64_copy_and_immutable_output |
| Failure/effect facts were absent from short row timeline | Known WORKFLOW_FAILED and STATE_EFFECT_COMMIT nested typed ID fields exposed; all run-level failure/effect/cancel/rejection references remain visible, unmatched events stay unassociated context | test_typed_failure_effect_links_and_unmatched_context |

Actual initial user/source context and frozen oracle expectation remain separate. Recorded model request/response files are hash references; the helper does not infer common prompt text or actual-request success from oracle strings. Candidate input is only the initial object; later streamed ingress remains in trace/raw events. All new semantic dimensions remain UNREVIEWED. No actual corpus, evaluator, model, parser, candidate, compiler, network or block extraction was executed for this repair. Actual extraction requires subsequent Astra review and root GO.
