# Prompt patterns

Field notes on how a shipping on-device speech pipeline manages its prompts, and
what VIA should borrow.

VIA's Layer 2 sends a transcript to a local model and expects clean text back.
That is the same problem shape as a dictation app's "polish" stage, and the
failure modes are the same: a prompt drifts away from the weights it was tuned
on, the eval harness and the app quietly disagree about what was measured, a
router picks a formatting mode that suppresses the model on most inputs, and
dictated content gets read as an instruction. These notes are about the
machinery that prevents those, not about prompt wording.

## Contents

| File | What it covers |
|---|---|
| [patterns.md](patterns.md) | The 13 patterns, each with the evidence behind it and how it maps onto VIA |
| [reference-envious-wispr.md](reference-envious-wispr.md) | The cited source map: which model gets which prompt family, and where each mechanism lives upstream |

## Provenance and licensing

The observations come from reading
[saurabhav88/EnviousWispr](https://github.com/saurabhav88/EnviousWispr) at
`main`, a macOS on-device dictation app (Swift). Reviewed 2026-09-06.

**That project is GPLv3. VIA is Apache-2.0.** These files therefore describe
*patterns and mechanisms* in our own words. They do not reproduce upstream
prompt text, and no upstream code or prompt file may be copied into VIA. Where
a phrase is quoted for illustration it is short, in quotation marks, and
attributed to the file it came from — cite the path, do not vendor the text.

If we ever want their actual prompt wording, that is a licensing decision, not
an engineering one. Ask first.

EnviousWispr's own EG-1 model weights are separately non-open (a custom
"EG-1 Community Model License", on top of a Qwen3-4B-Instruct-2507 base). Not
usable by VIA under any reading.
