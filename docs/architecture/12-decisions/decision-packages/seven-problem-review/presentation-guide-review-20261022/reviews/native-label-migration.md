# v8 read-only label audit and safe migration notes

Source SHA256: `0b2a87104b740714a9ced05e35b36407ffca723e172e40c2afc5dc00118ace10`.

`v8-own-label-map.json` contains 299 confirmed native geometry/text pairs across 21 pages. 11 broad-container candidates are explicitly rejected because their text belongs to a smaller inner shape. No unresolved candidate remains in the inspected candidate set. The 130 workflow-header chevron/text pairs are catalogued separately but excluded from the diagram-label repair set. Ordinary flow labels have no paired node and remain separate.

Every confirmed pair records original slide part, visible page, shape ID, label ID, native preset, adjustments, exact node/label coordinates in EMU, original paragraph text, text-body properties and computed offsets. Each selected node and label ID is unique. No selected shape/label is rotated, hyperlinked, a native connector endpoint, or an animation target.

## Existing v7 merge helper needs geometry compensation

`package_dp3_cpu_v7.py::own_shape_labels` adds text-box-to-shape differences to bodyPr insets. This preserves the text rectangle for `rect` but assumes every preset's text rectangle is its full bounding rectangle. That assumption is false for `can`, `roundRect`, `ellipse`, and `hexagon`.

The locally bundled LibreOffice DrawingML preset dictionary is:
`/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/native/libreoffice-headless/libreoffice/LibreOfficeDev.app/Contents/Resources/filter/oox-drawingml-cs-presets`.
Its `Equations` and `TextFrames` sections expose the intrinsic rectangles. This is structural evidence, not a native PowerPoint render test.

Let shape bounds be `(x,y,w,h)`, label bounds `(tx,ty,tw,th)`, old label body insets be `(L,T,R,B)`, and intrinsic preset inward offsets be `(pl,pt,pr,pb)`. The corrected body insets are:

- lIns = tx - x + L - pl
- tIns = ty - y + T - pt
- rIns = x + w - tx - tw + R - pr
- bIns = y + h - ty - th + B - pb

Negative insets can legitimately be required; clamping them changes the intended rectangle. Preserve `anchor`, paragraph styles, wrapping and noAutofit rather than assigning generic text defaults. Render after merging. Shape geometry, stroke, fill, native preset and position stay unchanged; only its own txBody gains the original label and the separate label shape disappears.

For the actual source presets:

- `rect`: all intrinsic offsets are zero.
- `can`: `ss=min(w,h)`, `a=pin(0,adj,50000*h/ss)`, `dy=ss*a/200000`. Offsets `(0,2*dy,0,dy)`.
- `roundRect`: `a=pin(0,adj,50000)`, `ir=ss*a/100000*29289/100000`. Offsets `(ir,ir,ir,ir)`.
- `ellipse`: x inset is `w*(1-1/sqrt(2))/2`; y inset is `h*(1-1/sqrt(2))/2`, symmetric on both sides.

Example: page2 native cylinder id25 is 225×96px, adj16000. Intrinsic top is 15.36px and bottom7.68px. Its old label id26 has bbox offsets left5,top11,right5,bottom6px. Add-only gives body top11/bottom6 and applies the intrinsic offsets in addition. Corrected top is -4.36px and bottom -1.68px. The JSON records all values in EMU, rounded only for final bodyPr integers.

No floating `hexagon` pair occurs in these 299 pairs. If later v9 contains one, do not assume zero. The same dictionary's hexagon TextFrames uses equation indices19/20/21/22, which depend on dimensions and adj. v9 authored own-label nodes should remain untouched.

## Preservation checks after v9 merge

Resolve pages using `ppt/presentation.xml` + relationships; do not assume slide10.xml is visible page10. Existing v8 replacements are slide510.xml,511,512. New v9 insertion may move old appendix from24–29 to30–35 but retained part names should be checked against the package rather than presumed.

Apply a map only when node and label IDs, full text, coordinates, native preset and adjustment values match the source assertions. Original geometry XML must be byte/semantic identical. Copy complete txBody, including font properties. Do not merge section backgrounds, selection outline rectangles, flow labels, or red selection checkmarks. The p4 header pair id17/id19 deliberately leaves its red checkmark id18 separate.

On each changed slide verify all original text runs remain represented once, no model/QA values changed, all tables stay native, original call paths/arrows are unchanged, and final footers use the new total. Assert non-slide package parts unchanged except allowed footer/registry work already made by v9.

ArtifactTool and native PowerPoint may interpret preset text rectangles differently. The compensated insets are structurally correct from DrawingML, but a before/after actual render is still required. SDK schema validation does not prove pixel equality or editing behavior. Do not claim native Windows rendering unless PowerPoint itself was exercised.
