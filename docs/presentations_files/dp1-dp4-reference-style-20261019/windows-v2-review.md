# DP1–DP4 Windows PowerPoint version review

## Deliverables

| Deck | File | Slides |
| --- | --- | ---: |
| Combined | `VIA-DP1-DP4-reference-style-16slides-v2-windows.pptx` | 16 |
| DP1 | `../dp1-reference-style-20261019/VIA-DP1-reference-style-v4-windows.pptx` | 4 |
| DP2 | `../dp2-reference-style-20261019/VIA-DP2-reference-style-v3-windows.pptx` | 4 |
| DP3 | `../dp3-reference-style-20261019/VIA-DP3-reference-style-v2-windows.pptx` | 4 |
| DP4 | `../dp4-reference-style-20261019/VIA-DP4-reference-style-v2-windows.pptx` | 4 |

Earlier versions remain in place. This revision keeps the same sixteen-slide content and reference-style structure. To accommodate Calibri metrics, the bottom margin of dense comparison/selection diagrams was enlarged, and two DP4 footnotes were condensed without changing their QA values.

## Validation

- `presentation_windows_policy.py check` passed for all five PPTX files: zero violations. Declared fonts, including theme, master, layout, notes, East Asian and complex-script defaults, are Calibri; sizes are integer PowerPoint points; language and new-text defaults are `ko-KR`.
- The OOXML finalization compared slide text, shape/table/picture counts, positions and media before and after the typography pass. All matched. See each deck's `windows-validation.json` and this folder's `windows-v2-validation.json`.
- The combined deck contains sixteen native PowerPoint slides with editable shapes and native tables. Every slide has native shapes, fourteen have native tables, and none rely on a raster picture for the structure diagram or table.
- The combined deck and all four individual decks rendered successfully with LibreOffice on macOS. Calibri's Korean glyphs used an explicitly configured local fallback for this visual check. All sixteen combined slide previews were inspected; dense comparison/selection slides 6, 7, 10, 11, 14 and 15 received a closer inspection after layout adjustment. The final overview is in `windows-v2-contact-sheet.png`.

## Verification limit

The local visual render was not Microsoft PowerPoint on Windows. Native PowerPoint was not installed or exercised in this environment. The repository's `presentation-windows-check` workflow can validate the package on Windows with Microsoft's Open XML SDK; that validation also does not establish native PowerPoint rendering.
