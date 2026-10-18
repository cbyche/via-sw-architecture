# Frozen content-condition diagnostic

Previously frozen `semantic_checks` are now reported separately. Existing QA02 and QA01/03 values remain unchanged. This is not a new formal QA. No source, oracle, raw record, prompt or execution condition changed, and no model judge/call was used.

|Phase / DP / option / tactic|Runs|Frozen condition instances|Exact-marker pass|Whole required-fact omission|Required markers present/total|Forbidden literal hits|Runs without content checks|
|---|---:|---:|---:|---:|---:|---:|---:|
|BASIC / 01 / A / —|12|3|0|3|0/9|0|9|
|BASIC / 01 / B / —|12|3|0|3|0/9|0|9|
|BASIC / 01 / C / —|12|3|0|3|0/9|0|9|
|BASIC / 02 / A / —|12|0|0|0|0/0|0|12|
|BASIC / 02 / B / —|12|0|0|0|0/0|0|12|
|BASIC / 04 / A / —|12|0|0|0|0/0|0|12|
|BASIC / 04 / B / —|12|0|0|0|0/0|0|12|
|BASIC / 05 / A / —|12|0|0|0|0/0|0|12|
|BASIC / 05 / B / —|12|0|0|0|0/0|0|12|
|BASIC / 05 / C / —|12|0|0|0|0/0|0|12|
|DIAGNOSTIC / 04 / B4 / B4|3|0|0|0|0/0|0|3|
|SMOKE / 01 / C / —|1|0|0|0|0/0|0|1|
|SMOKE / 02 / A / —|1|0|0|0|0/0|0|1|
|SMOKE / 04 / A / —|1|0|0|0|0/0|0|1|
|SMOKE / 05 / C / —|1|0|0|0|0/0|0|1|
|TACTIC / 01 / A / A2|3|3|0|3|0/9|0|0|
|TACTIC / 02 / A / A1|3|0|0|0|0/0|0|3|
|TACTIC / 04 / A4 / A4|3|0|0|0|0/0|0|3|
|TACTIC / 05 / C / C2|3|0|0|0|0/0|0|3|

The only nonempty frozen content condition is DP01 E3 q2. It requires 1180000, 36, and 11월 1일, plus current source identities and a forbidden-claim guard. BASIC has nine such displayed answers (ABC×3) and selected A2 adds three. Source/version identity may pass QA02 while the displayed response omits these facts. The preserved answers repeat the referring phrase; they do not contain the frozen summary facts.

This is an omitted evaluation/closure check, not evidence that the whole summary task completed. The plan scopes DP01 through actual evidence production and request interpretation; QA02 specifically measures referent sets/version identity. The separate prewritten content condition means it is also inaccurate to say all summary-content verification was outside the PoC. Keep identity completion and missing content-condition satisfaction separate.

Exact-string checks have representation limits: `1,180,000` or a paraphrase need not contain `1180000`. Forbidden literal absence is not semantic safety proof. Here all three markers are wholly absent from short echo responses, which is preserved as an observable omission rather than a formatting-only conclusion.

## Original source references

- `oracles/dp01.json`, E3 `semantic_checks`: frozen required facts, source versions and forbidden claim.
- `11-minimal-poc-plan.md` §5: DP01 executes actual derived-state production through request interpretation.
- `benchmark/architecture/four_dp_poc/README.md`, frozen QA02: exact adopted target set and required versions; content facts are not QA02.
- `IMPLEMENTATION-dp01-dp05.md`: actual-evidence guard for direct completion and bounded source completion. E3 baseline q2 rejects direct completion but proceeds with no new body read.
- `analyze.py` before this separate diagnostic did not consume `semantic_checks`; this script does not change its primary metrics.

## Preserved actual display text

### basic-dp01-e3-A-r1-e83530

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 최신으로 정정된 견적`.

### basic-dp01-e3-B-r1-aafc55

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### basic-dp01-e3-C-r1-0e2e09

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### basic-dp01-e3-B-r2-714b31

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### basic-dp01-e3-C-r2-18b712

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### basic-dp01-e3-A-r2-8529aa

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### basic-dp01-e3-C-r3-2e2502

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### basic-dp01-e3-A-r3-55a0f5

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### basic-dp01-e3-B-r3-a6beae

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### tactic-dp01-e3-A-r1-cd959a

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### tactic-dp01-e3-A-r2-a18cef

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.

### tactic-dp01-e3-A-r3-6a5e82

Status: ALL_REQUIRED_FACT_MARKERS_ABSENT; displayed text: `그 설명 기준과 이제 최신으로 정정된 견적만 다시 정리해줘.`.
