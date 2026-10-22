# Independent mechanism review for the whole-deck v10 figures

Read-only review of work-in-progress fragments, 2026-10-22. This is not measurement or a new architecture decision. The QA values, sample populations and selection status are outside the figure edits.

## Sources read

- `docs/architecture/12-decisions/decision-packages/seven-problem-review/01-evidence-supply.md`, especially §3 and §4.1–4.3.
- `.../02-interpretation-execution.md`, especially common contracts and §4.
- `.../05-agent-handoff.md`, especially §3, §4 and §6.
- `.../dp5-delegation-memory-20261020.md`, especially §3–5.1.
- Existing v8 page4,8,19 native text and geometry, plus the 29-page package.
- Work-in-progress `legacy_dp_structure_scenes.mjs` and `dp5_v10_grounding_loops.mjs`.

## Findings communicated to the root author

1. **DP1 A initial working context must be visible.** The first draft sent the initial request directly to interpretation and showed Context Manager only after a missing-evidence proposal. The source A first retrieves the maintained valid working context before interpretation. This is its basic reuse advantage. Use existing permit/query/return paths on initial entry as well, explicitly label current input plus valid context, and retain the later reauthorization loop. Do not add an extra preliminary LLM call.
2. **DP1 internal source read numbering must not imply return before read.** The draft numbered bundle return4 and inner source read5/return6. Unnumber the inner I/O or reorder the numbered actions so the diagram can be followed without an invented temporal ordering.
3. **DP4 B requires a real raw fallback path.** The first draft's extraction module was named “목적별 추출 및 원본 보완기” but only read the published record. Add extraction↔original-source query/result paths. A background producer-only read is not the request's fallback and must not force new publication before a valid raw read.
4. **DP4 C access-state durability should not be newly fixed by a figure.** The first draft called the lease/read-receipt state “RAM / 접근 수명”. The source describes restoration of the access ledger. Prefer “접근 원장 / 읽기 영수증 (접근 수명)” unless the document explicitly separates a RAM view from its durable ledger. Do not delete the original recovery contract in a figure cleanup.
5. **DP4 B event direction.** A producer→source arrow was labelled “변경”. That reads as the producer changing the source. Distinguish source change event→producer from producer read request→source.

These were communicated while other agents were editing; the latest scene revision must be rechecked for their correction before a final all-clear.

## Confirmed responsibilities

- DP1 Context Manager owns composed/derived working views, while original source authority remains at the source owner; Request Controller keeps current permission and final adoption.
- DP2 uses the same Model Access call budget and cloud model in each alternative. R1/R2 are logical executions inside Request Interpreter, not separate processes/model copies. B1 shares immutable input only; mutable per-request state remains separate. No mandatory router-plus-worker call was introduced for short answers.
- DP4's proposed preparation→candidate return→Request Controller permission→Task Manager command→Agent Gateway transmission sequence follows the source's distinction between semantic adoption, execution admission and actual sending. C READ crosses Agent Gateway and Context Manager's range/currentness checks; the source does not authorize arbitrary receiver access.
- DP5's event-triggered production and derived persistent relations remain owned by Context Manager, not a new Component/service. Request Interpreter applies relations to the current request, and Request Controller adopts the meaning. Conditional semantic production, original authority and raw fallback are preserved. Producer reads use the same authorized source-read contract; they must not inherit an unrelated current user's last read permission.

## Supplementary p4/p8/p19 fragments

`legacy_tactic_detail_fragments.mjs` supplies only the existing left-hand mechanism diagrams and records explicit old-shape removal IDs. Existing headings, data/QA tables, numerical diagnostics, conditions and selection caveats remain in the source slide.

- p4: Context Manager's valid working view, Request Controller permission/adoption and Request Interpreter meaning are separate. A2 is an expanded internal path of the same Context Manager, uses changed-source spans, checks the full dependency state and updates the existing view. The two Context Manager boxes are the same logical responsibility shown in baseline and tactic detail, not two instances. Existing fallback and non-adoption captions remain.
- p8: both before and after have one Request Interpreter responsibility containing R1/R2 logical execution modules. Input copies and shared input are cylinders, as are per-request mutable states. B1 changes only immutable input sharing; current QA limits are unchanged.
- p19: Context Manager's C1 prefetch/attachment, Request Controller admission, Task Manager command and Agent Gateway transport have distinct Modules. Existing conditional source references and detailed source material are different read ranges, not new authority/stores. The detailed receiver READ request/result loop remains distinct from initial sending. No model call or durable state was added by splitting these responsibilities.

Scene validator passes for all three fragments: p4 11 nodes/10 edges, p8 11 nodes/8 edges, p19 14 nodes/15 edges. Native final PPTX rendering, line-label clearance and package integration still need checking by the root author. Fragment SVG previews were rasterized and visually inspected, but are not proof of native PowerPoint rendering.
