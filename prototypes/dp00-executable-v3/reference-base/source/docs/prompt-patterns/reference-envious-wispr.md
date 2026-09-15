# Source map: EnviousWispr

Where each mechanism in [patterns.md](patterns.md) actually lives, so a claim can
be checked rather than trusted. Paths are relative to
[saurabhav88/EnviousWispr](https://github.com/saurabhav88/EnviousWispr), read at
`main` on 2026-09-06.

Reminder: that repo is GPLv3 and VIA is Apache-2.0. Read these files; do not
copy from them. See [README.md](README.md).

## The app

macOS on-device dictation. Press a keybind, speak, get clean text pasted into
whatever app has focus:

`record → VAD endpoint → on-device ASR → optional LLM polish → paste`

Polish is the stage that concerns us. Everything below is about that stage.

## Models

**Speech-to-text** — all on-device, delivered through signed per-file manifests
(`Sources/EnviousWispr/Resources/*-delivery-manifest.json`):

| Family | Identity | Variant |
|---|---|---|
| Parakeet (NVIDIA NeMo, via FluidAudio) | `parakeet-tdt-0.6b-v3-coreml` | `int8` |
| WhisperKit | `whisperkit-coreml` | `openai_whisper-large-v3-v20240930_turbo` (default) |
| WhisperKit preview | `whisperkit-coreml` | `openai_whisper-small_216MB` |
| Endpointing | Silero VAD (CoreML), 256 ms @ 16 kHz | — |
| Output safety | in-house CoreML pair-encoder classifier over polish output | — |

**Polish LLMs** — seven providers (`Sources/EnviousWisprCore/LLMResult.swift`,
`LLMProvider`):

| Provider | Model | Notes |
|---|---|---|
| EG-1 (first-party) | `eg-1`, version `eg1-1.2-c003`, 16k ctx, q5_k_m GGUF | QLoRA fine-tune of Qwen3-4B-Instruct-2507; served by a bundled `llama-server`; weights under a custom non-open licence |
| S1-mini (third-party) | `superwhisper/s1-mini`, q4_k_m, 8k ctx | a Qwen3-0.6B fine-tune, pinned to HF revision `34add00a…` |
| Apple Intelligence | on-device AFM via `SystemLanguageModel` | 4,096 tokens shared across instructions + input + output |
| Ollama | user's choice, default `qwen2.5:3b` | hosted free tier snapshot: `gemma4:31b`, `gpt-oss:120b`, `gpt-oss:20b`, `minimax-m3`, `nemotron-3-{nano:30b,super,ultra}` |
| OpenAI (BYO key) | default `gpt-4o-mini` | |
| Gemini (BYO key) | default `gemini-3.7-flash` | chosen over 3.5/3.6 on a 1,462-case bench |
| Claude (BYO key) | default `claude-haiku-4-5` | |

Their eval judge is `azure/gpt-5-6-luna` (previously `claude-sonnet-5`).

## Prompt families

Six prompts, five of them selected by one planner
(`Sources/EnviousWisprLLM/Prompting/DefaultPromptPlanner.swift`) and the sixth
branching earlier. All are modeless — none branches on the transcript.

| Family | Serves | Builder | Text of record |
|---|---|---|---|
| `cloudFixed` | OpenAI, Gemini, Claude, and Ollama models the daemon reports as *hosted* | `CloudFixedPromptBuilder.swift` | `scripts/eval/prompts/cloud-fixed-polish-prompt-v7.txt` |
| `localFixed` | any Ollama model running on the user's Mac | `LocalFixedPromptBuilder.swift` | `scripts/eval/prompts/ollama-local-polish-prompt-L3.txt` |
| `egOneFixed` | EG-1 1.1 weights | `EGOnePromptBuilder.swift` | `scripts/eval/prompts/eg1-polish-prompt-v1.txt` |
| `egOneEnvelope` | EG-1 1.2 weights (current) | `EGOneEnvelopePromptBuilder.swift` | `scripts/eval/prompts/eg1-polish-prompt-v2.txt` |
| `s1ControlLine` | S1-mini, by either transport | `S1ControlLinePromptBuilder.swift` | `scripts/eval/prompts/s1-mini-control-line-v1.txt` |
| (separate path) | Apple Intelligence | `AppleIntelligenceConnector.swift` | `scripts/eval/prompts/single-v38.txt` |

### Shape of each

- **Cloud v7** — long, second-person, prose rather than numbered rules. Opens
  with an unconditional no-translate rule, then self-correction handling with
  five worked examples, a don't-guess-names rule, an explicit restraint
  paragraph, spoken-list formatting, and closes with the content-not-instruction
  framing. User message is the bare transcript under a short lead-in.
  Conditional additions: locked language, app name, a `≤10 words` minimal-edit
  guard, user vocabulary.
- **Local L3** — terse, sectioned (`DELETE` / `KEEP` / `SPEECH REPAIR` /
  `ORTHOGRAPHY` / `SEGMENTATION`), linguistic vocabulary, five worked examples
  including one list case. Deliberately *omits* the unconditional language
  preamble and the short-input guard the cloud builder carries — see pattern 5.
- **EG-1 v1** — one sentence of instruction plus the wrapper rule. Nothing else.
- **EG-1 v2** — the v1 sentence, plus greeting/sign-off layout with a worked
  example, plus three self-correction examples. Both wrap the transcript in
  `<TRANSCRIPT>` with zero-width-non-joiner neutralisation, and both refuse all
  conditional enrichment.
- **S1-mini** — the upstream model card's system prompt transcribed verbatim,
  then a first-line control token `[Styling: …] [Structure: …] [Context: …]`
  followed by a bare transcript. No wrapper: the model was tuned without one.
  Defaults `semi-formal` / `lists` / `general`
  (`Sources/EnviousWisprCore/S1ControlSettings.swift`).
- **Apple Intelligence v38** — 14 numbered rules plus two worked examples, sized
  against a 4k shared context. Prefixed with a named-language clause for
  non-English, suffixed with a short speech-to-text-awareness clause.

## Mechanism → file

| Mechanism | Where |
|---|---|
| Family selection, and the argument for each branch | `Prompting/DefaultPromptPlanner.swift` — `family(for:modelID:ollamaIsRemote:egOneFamily:)` |
| Prompt family read from the model manifest | `DefaultPromptPlanner.bundledEGOneFamily`, `Resources/eg1-manifest.json` (`promptTemplateID`) |
| Activation refuses an unknown template | `EGOne/EGOneRuntime.swift` |
| Byte-identity of app constant ↔ text of record ↔ Python mirror | `scripts/eval/acceptance_gate.py` `_selftest_mirrors()`, plus `LocalFixedPromptTests` |
| Delimiter neutralisation (ZWNJ) | `EGOnePromptBuilder.build`, `EGOneEnvelopePromptBuilder.build` |
| Closed enums for user-tunable prompt tokens | `Core/S1ControlSettings.swift` |
| Vocabulary applied deterministically before polish | `PostProcessing/WordCorrector.swift` |
| Tiered vocabulary policy by language-detection confidence | `DefaultPromptPlanner.applyVocabularyPolicy` |
| Plan carries the decision to builder, validator and telemetry | `PolishPlan`, `Pipeline/LLMPolishStep.swift` |
| Distinct ids for managed vs BYO copies of one model | `LLMProvider.s1MiniModelName` and its comment |
| Prompt bake-off record | `docs/audits/2026-08-16-cloud-polish-prompt-bakeoff.md` |
| Model/candidate registry with provenance | `scripts/eval/model-registry.json` |

## Worth reading even though it is not a prompt pattern

- `Sources/EnviousWisprModelDelivery/ShippedModelNames.swift` — an append-only
  freeze list of every model name ever shipped, existing solely so a cache-key
  prefix match can never delete a different model's bytes. The comment explains
  why removing a retired entry silently reopens the bug.
- `LLMProvider.retiredModelIDs` — provider-withdrawn model ids swept off saved
  settings through both the live-settings seam and the crash-recovery replay,
  keyed by provider so a user's legitimately-named local model is never rewritten.
