# First-adoption referent failures

These are actual adopted sets, not repaired final answers. Source versions and available/read paths are shown as observed. No model reruns or oracle-driven tuning. Explanations below classify set/version differences, not architectural causes.

## SMOKE dp04-dense A r1 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D01"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-1 at t=300ms (x=375, y=315) falls directly within the bounding box of the card labeled 'D01 / Orion' (top-left card). The transcript '이 항목' (this item) aligns with this specific pointing action."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/smoke-dp04-dense-A-r1-c17814/trace.json`.

## SMOKE dp04-dense A r1 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D06"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-2 at t=1100ms (x=630, y=480) falls directly within the bounding box of the card labeled 'D06 / Orion' (middle row, second column). The transcript '이 항목' (this item) aligns with this second pointing action."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/smoke-dp04-dense-A-r1-c17814/trace.json`.

## BASIC dp01-e4 C r1 — e4-q1-ref rev1

- Expected: `["old-research", "old-price"]`; first adopted: `["old-explanation", "old-price", "old-research"]`.
- Adopted source versions: `[{"revision": 1, "source_id": "old-explanation"}, {"revision": 1, "source_id": "old-price"}, {"revision": 1, "source_id": "old-research"}]`; classification: TARGET_SET_MISMATCH.
- Actual path: ANSWER_FROM_AVAILABLE_EVIDENCE; model requested mode: ANSWER; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "e4-q1", "input": {"conversation": "history-c", "expression": "연구한 전력 결과와 받아둔 구매 견적", "expression_id": "e4-q1-ref", "goal_id": "e4-q1", "kind": "request", "revision": 1, "stimulus_id": "e4-q1", "utterance": "9월에 연구한 전력 결과와 받아둔 구매 견적을 같이 설명해줘."}, "stimulus_id": "e4-q1"}`.
- Model conditions/evidence: `[]`.
- Relevant published records: 10; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp01-e4-C-r1-f090b5/trace.json`.

## BASIC dp01-e4 C r1 — e4-q3-ref rev1

- Expected: `["old-research", "old-price"]`; first adopted: `["old-explanation"]`.
- Adopted source versions: `[{"revision": 1, "source_id": "old-explanation"}]`; classification: TARGET_SET_MISMATCH.
- Actual path: ANSWER_FROM_AVAILABLE_EVIDENCE; model requested mode: ANSWER; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "e4-q3", "input": {"conversation": "fresh-c", "expression": "같은 두 원본", "expression_id": "e4-q3-ref", "goal_id": "e4-q3", "kind": "request", "revision": 1, "stimulus_id": "e4-q3", "utterance": "이 새 대화에서 같은 두 원본의 미확인 사항만 다시 알려줘."}, "stimulus_id": "e4-q3"}`.
- Model conditions/evidence: `["expression '같은 두 원본' refers to the two sources discussed in the recent conversation: old-research and old-price, but the question asks for unconfirmed items *in this new conversation* relative to the full context or original explanation which links them.", "The recent answer covers details of old-research and old-price individually but omits the synthesis/limitation found in old-explanation."]`.
- Relevant published records: 10; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp01-e4-C-r1-f090b5/trace.json`.

## BASIC dp01-e4 C r2 — e4-q1-ref rev1

- Expected: `["old-research", "old-price"]`; first adopted: `["old-explanation", "old-price", "old-research"]`.
- Adopted source versions: `[{"revision": 1, "source_id": "old-explanation"}, {"revision": 1, "source_id": "old-price"}, {"revision": 1, "source_id": "old-research"}]`; classification: TARGET_SET_MISMATCH.
- Actual path: ANSWER_FROM_AVAILABLE_EVIDENCE; model requested mode: ANSWER; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "e4-q1", "input": {"conversation": "history-c", "expression": "연구한 전력 결과와 받아둔 구매 견적", "expression_id": "e4-q1-ref", "goal_id": "e4-q1", "kind": "request", "revision": 1, "stimulus_id": "e4-q1", "utterance": "9월에 연구한 전력 결과와 받아둔 구매 견적을 같이 설명해줘."}, "stimulus_id": "e4-q1"}`.
- Model conditions/evidence: `[]`.
- Relevant published records: 10; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp01-e4-C-r2-62a4bb/trace.json`.

## BASIC dp01-e4 C r3 — e4-q1-ref rev1

- Expected: `["old-research", "old-price"]`; first adopted: `["old-explanation", "old-price", "old-research"]`.
- Adopted source versions: `[{"revision": 1, "source_id": "old-explanation"}, {"revision": 1, "source_id": "old-price"}, {"revision": 1, "source_id": "old-research"}]`; classification: TARGET_SET_MISMATCH.
- Actual path: ANSWER_FROM_AVAILABLE_EVIDENCE; model requested mode: ANSWER; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "e4-q1", "input": {"conversation": "history-c", "expression": "연구한 전력 결과와 받아둔 구매 견적", "expression_id": "e4-q1-ref", "goal_id": "e4-q1", "kind": "request", "revision": 1, "stimulus_id": "e4-q1", "utterance": "9월에 연구한 전력 결과와 받아둔 구매 견적을 같이 설명해줘."}, "stimulus_id": "e4-q1"}`.
- Model conditions/evidence: `[]`.
- Relevant published records: 10; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp01-e4-C-r3-b3350c/trace.json`.

## BASIC dp04-dense A r1 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D01"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-1 at t=300ms (within expression window 100-500ms) is located at (375, 315). Visual inspection of frame 'dense' confirms this coordinate falls squarely within the bounding box of the card labeled 'D01 / Orion' (top-left card in the grid)."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-A-r1-f6c431/trace.json`.

## BASIC dp04-dense A r1 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D06"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-2 at t=1100ms (within expression window 900-1300ms) is located at (630, 480). Visual inspection of frame 'dense' confirms this coordinate falls squarely within the bounding box of the card labeled 'D06 / Orion' (second card in the second row)."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-A-r1-f6c431/trace.json`.

## BASIC dp04-dense B r1 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D03"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-B-r1-eed867/trace.json`.

## BASIC dp04-dense B r1 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D07"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-B-r1-eed867/trace.json`.

## BASIC dp04-free-region A r1 — graph-region rev1

- Expected: `["P3", "P4", "P5"]`; first adopted: `["P2", "P3", "P5"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-free-region:r1", "revision": 1, "stimulus_id": "dp04-free-region:r1", "transcript": "이렇게 둘러싼 세 점만 평균을 내줘."}`.
- Model conditions/evidence: `"The pointer path (loop-1) traces a closed polygon enclosing exactly three labeled points: P2, P3, and P5. The transcript '이렇게 둘러싼 세 점' (these three enclosed points) explicitly references the set defined by this spatial enclosure."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-free-region-A-r1-20c071/trace.json`.

## BASIC dp04-free-region B r1 — graph-region rev1

- Expected: `["P3", "P4", "P5"]`; first adopted: `["P2", "P3", "P4", "P5", "P6", "P7"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-free-region:r1", "revision": 1, "stimulus_id": "dp04-free-region:r1", "transcript": "이렇게 둘러싼 세 점만 평균을 내줘."}`.
- Model conditions/evidence: `"The expression 'graph-region' with meaning 'The three points enclosed or defined by the pointer path loop' is linked to cue 'loop-1'. The screen provider explicitly maps 'loop-1' to the target_ids ['P2', 'P3', 'P4', 'P5', 'P6', 'P7'] via the cue_relations field. Although the speech mentions 'three points', the cue relation provided by the screen producer defines the specific set of candidates associated with that loop gesture. We adopt the candidates defined by the cue relation as the intended targets for this region selection."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-free-region-B-r1-4dc1da/trace.json`.

## BASIC dp04-dense B r2 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D03"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-B-r2-d07234/trace.json`.

## BASIC dp04-dense B r2 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D08"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-B-r2-d07234/trace.json`.

## BASIC dp04-dense A r2 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D01"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-1 at t=300ms (within expression window 100-500ms) is located at (375, 315). Visual inspection of frame 'dense' confirms this coordinate falls squarely within the bounding box of the card labeled 'D01 / Orion' (top-left card in the grid)."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-A-r2-5948ce/trace.json`.

## BASIC dp04-dense A r2 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D06"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-2 at t=1100ms (within expression window 900-1300ms) is located at (630, 480). Visual inspection of frame 'dense' confirms this coordinate falls squarely within the bounding box of the card labeled 'D06 / Orion' (second card in the second row)."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-A-r2-5948ce/trace.json`.

## BASIC dp04-free-region B r2 — graph-region rev1

- Expected: `["P3", "P4", "P5"]`; first adopted: `["P2", "P3", "P4", "P5", "P6", "P7"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-free-region:r1", "revision": 1, "stimulus_id": "dp04-free-region:r1", "transcript": "이렇게 둘러싼 세 점만 평균을 내줘."}`.
- Model conditions/evidence: `"The expression 'graph-region' with meaning 'The three points enclosed or defined by the pointer path loop' is linked to cue_id 'loop-1'. The screen provider's cue_relations explicitly map 'loop-1' to the target_ids ['P2', 'P3', 'P4', 'P5', 'P6', 'P7'] via a 'within_region' relation. Although the speech mentions 'three points', the cue relation provided by the screen producer defines the specific set of candidates associated with that loop gesture. We adopt the candidates defined by the structural cue relation as the most accurate resolution of the pointing intent."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-free-region-B-r2-735b07/trace.json`.

## BASIC dp04-free-region A r2 — graph-region rev1

- Expected: `["P3", "P4", "P5"]`; first adopted: `["P2", "P3", "P5"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-free-region:r1", "revision": 1, "stimulus_id": "dp04-free-region:r1", "transcript": "이렇게 둘러싼 세 점만 평균을 내줘."}`.
- Model conditions/evidence: `"The pointer path (loop-1) traces a closed polygon enclosing exactly three data points: P2, P3, and P5. The transcript '이렇게 둘러싼 세 점' (these three enclosed points) explicitly refers to the items inside this gesture."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-free-region-A-r2-b5d6b5/trace.json`.

## BASIC dp04-dense A r3 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D01"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-1 at t=300ms (within expression window 100-500ms) is located at (375, 315). Visual inspection of frame 'dense' confirms this coordinate falls squarely within the bounding box of the card labeled 'D01 / Orion' (top-left card in the grid)."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-A-r3-2b9a3f/trace.json`.

## BASIC dp04-dense A r3 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D06"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event pointer-2 at t=1100ms (within expression window 900-1300ms) is located at (630, 480). Visual inspection of frame 'dense' confirms this coordinate falls squarely within the bounding box of the card labeled 'D06 / Orion' (second card in the second row)."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-A-r3-2b9a3f/trace.json`.

## BASIC dp04-dense B r3 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D05"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-B-r3-e81f99/trace.json`.

## BASIC dp04-dense B r3 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D08"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-dense-B-r3-e81f99/trace.json`.

## BASIC dp04-free-region A r3 — graph-region rev1

- Expected: `["P3", "P4", "P5"]`; first adopted: `["P2", "P3", "P5"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-free-region:r1", "revision": 1, "stimulus_id": "dp04-free-region:r1", "transcript": "이렇게 둘러싼 세 점만 평균을 내줘."}`.
- Model conditions/evidence: `"The pointer path (loop-1) traces a triangular region enclosing exactly three data points. Visually, the loop starts near P2, moves right to encompass P3, drops down to encompass P5, and returns to the start. The transcript '이렇게 둘러싼 세 점' (these three enclosed points) confirms the intent to select the points inside this specific gesture."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-free-region-A-r3-502307/trace.json`.

## BASIC dp04-free-region B r3 — graph-region rev1

- Expected: `["P3", "P4", "P5"]`; first adopted: `["P2", "P3", "P4", "P5", "P6", "P7"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-free-region:r1", "revision": 1, "stimulus_id": "dp04-free-region:r1", "transcript": "이렇게 둘러싼 세 점만 평균을 내줘."}`.
- Model conditions/evidence: `"The expression 'graph-region' with meaning 'The three points enclosed or defined by the pointer path loop' is explicitly linked to cue_id 'loop-1'. The screen provider's cue_relations map 'loop-1' directly to the target_ids ['P2', 'P3', 'P4', 'P5', 'P6', 'P7'] via a 'within_region' relation. Although the speech mentions 'three points', the cue relation provided by the screen producer defines the specific set of candidates associated with that loop gesture. I adopt the targets defined by the cue relation as they represent the system's interpretation of the pointer path."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `results/architecture-evaluation/current/four-dp-qwen37-20261018/raw/runs/basic-dp04-free-region-B-r3-616b00/trace.json`.

## TACTIC dp04-dense A4 r1 — dense-first rev1

- Expected: `["D06"]`; first adopted: `[]`.
- Adopted source versions: `[]`; classification: MISSING_ADOPTION.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `null`.
- Model conditions/evidence: `null`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-A4/raw/runs/tactic-dp04-dense-A4-r1-eab22d/trace.json`.

## TACTIC dp04-dense A4 r1 — dense-second rev1

- Expected: `["D11"]`; first adopted: `[]`.
- Adopted source versions: `[]`; classification: MISSING_ADOPTION.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `null`.
- Model conditions/evidence: `null`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-A4/raw/runs/tactic-dp04-dense-A4-r1-eab22d/trace.json`.

## TACTIC dp04-dense A4 r2 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D01"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event 'pointer-1' at t=300ms (within expression window 100-500ms) is located at (375, 315). This coordinate falls inside the bounding box of card D05 ([275, 330, 505, 490]). Although the auxiliary parser suggested D01, the pixel coordinates definitively place the pointer on D05. The transcript '이 항목' (this item) combined with the pointing gesture confirms D05 as the target."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-A4/raw/runs/tactic-dp04-dense-A4-r2-36471d/trace.json`.

## TACTIC dp04-dense A4 r2 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D08"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Pointer event 'pointer-2' at t=1100ms (within expression window 900-1300ms) is located at (630, 480). This coordinate falls inside the bounding box of card D08 ([765, 330, 995, 490] - wait, checking X: 630 is between 520 and 750? No. D07 is [520, 750]. D08 is [765, 995]. Let's re-evaluate X=630. 520 < 630 < 750. So X=630 is in column 3 (D03/D07/D11). Y=480 is in row 2 (330-490). Intersection is D07. \n\nCorrection on D08 analysis:\nPointer-2: x=630, y=480.\nRow 2 Y-range: 330 to 490. (480 is inside).\nCol 1 X: 30-260.\nCol 2 X: 275-505.\nCol 3 X: 520-750. (630 is inside).\nCol 4 X: 765-995.\nIntersection is D07 (Orion South / Q3).\nThe auxiliary parser claimed D08, but x=630 is clearly left of the D08 start x=765. The target is D07."`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-A4/raw/runs/tactic-dp04-dense-A4-r2-36471d/trace.json`.

## TACTIC dp04-dense A4 r3 — dense-first rev1

- Expected: `["D06"]`; first adopted: `[]`.
- Adopted source versions: `[]`; classification: MISSING_ADOPTION.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `null`.
- Model conditions/evidence: `null`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-A4/raw/runs/tactic-dp04-dense-A4-r3-8160b9/trace.json`.

## TACTIC dp04-dense A4 r3 — dense-second rev1

- Expected: `["D11"]`; first adopted: `[]`.
- Adopted source versions: `[]`; classification: MISSING_ADOPTION.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `null`.
- Model conditions/evidence: `null`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-A4/raw/runs/tactic-dp04-dense-A4-r3-8160b9/trace.json`.

## DIAGNOSTIC dp04-dense B4 r1 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D08"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-B4/raw/runs/diagnostic-dp04-dense-B4-r1-c8cbc6/trace.json`.

## DIAGNOSTIC dp04-dense B4 r2 — dense-first rev1

- Expected: `["D06"]`; first adopted: `["D03"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-B4/raw/runs/diagnostic-dp04-dense-B4-r2-fbee36/trace.json`.

## DIAGNOSTIC dp04-dense B4 r2 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D08"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-B4/raw/runs/diagnostic-dp04-dense-B4-r2-fbee36/trace.json`.

## DIAGNOSTIC dp04-dense B4 r3 — dense-second rev1

- Expected: `["D11"]`; first adopted: `["D08"]`.
- Adopted source versions: `[]`; classification: TARGET_SET_MISMATCH.
- Actual path: not observed / screen path; model requested mode: N/A; original reads: `[]`.
- Actual expression/utterance: `{"goal_id": "dp04-dense:r1", "revision": 1, "stimulus_id": "dp04-dense:r1", "transcript": "이 항목과 이 항목의 분기 실적을 비교해줘."}`.
- Model conditions/evidence: `"Unique complete typed candidate relation; unresolved/competition/navigation alternatives absent"`.
- Relevant published records: 0; full exact records in referent-failures.json.
- Raw trace: `/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-20261018/tactics/dp04-B4/raw/runs/diagnostic-dp04-dense-B4-r3-d3050e/trace.json`.

