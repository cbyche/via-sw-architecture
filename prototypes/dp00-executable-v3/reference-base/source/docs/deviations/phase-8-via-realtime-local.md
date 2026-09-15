# Phase 8 deviations — `via-realtime-local`

Layer 1 — the on-device providers: the componentized pipeline presented as one
realtime session, and a thin client for a local OpenAI-Realtime server.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-realtime-local`

*199 tests (default) · clippy clean under `-D warnings` · 21 deviations*

**No upstream.** `docs/architecture.md` §9 marks this crate with an asterisk:
qwen-audio-agent has no local provider and ARGO has none either. The three rows
`docs/architecture.md` §7 adds — `local-omni:pipeline`, `local-omni:endpoint`,
and the `local` model family — are the specification, and the catalogued
contracts this crate is judged against are the ones it has to **agree with** in
order to be indistinguishable from a cloud provider, not ones it reproduces.


### The three verified facts, and what each one cost

`docs/architecture.md` §7 opens with three findings. Each one removed a design
that would otherwise have looked obvious:

1. **Qwen3.5-Omni has no open weights** — DashScope-API-only. So there is no
   in-process omni model to load, and `DEFAULT_LOCAL_ENDPOINT_MODEL` is
   `Qwen3-Omni-30B-A3B` rather than a `qwen3.5-omni-*` id.
2. **No Rust runtime anywhere runs the Qwen3-Omni Talker** — checked against
   candle, mistral.rs, llama.cpp (whose `mtmd_gen_audio_type` is
   `{NONE, QWEN3TTS, POCKETTTS}`) and mlx-rs. So the path that runs on a laptop
   is a **cascade**, and the crate's hard problem is making a cascade look like
   a duplex session.
3. **`sherpa-onnx` covers VAD, streaming ASR and TTS behind one interface** —
   which is why three of the four stages come from one optional dependency and
   the fourth from another.


### The build constraint that shapes the crate

`docs/adr/0001-placement.md` records that VIA's native dependencies —
`sherpa-onnx`, `llama-cpp-2`, `cpal` — are exactly what keeps VIA out of ARGO's
workspace, because ARGO requires every crate to cross-compile clean to four
targets. A native toolchain in every VIA CI lane, for a feature most builds
never exercise, is that same decision made badly at a smaller scale.
`via-wake-word` draws the line at a trait; so does this crate.

| Half | Feature | What |
| --- | --- | --- |
| the session machine | *default* | `machine`, `stages`, `sentence`, `events`, both providers, `LocalSettings`, `WeightsSet`, `scripted` |
| the sherpa engines | `sherpa` | `SherpaVoiceActivity`, `SherpaTranscriber`, `SherpaSpeaker` |
| the reasoning engine | `llama` | `LlamaResponder` |

`cargo build -p via-realtime-local` needs no C toolchain, no weights and no
network. `cargo test -p via-realtime-local` drives **the whole pipeline** —
handshake, utterance, turn, incremental synthesis, barge-in, cancellation, tool
calls, `dictation`, and every stage failure — through `scripted`, because that
is the only part CI can ever run. `tests/sherpa.rs` and `tests/llama.rs` are
`#![cfg(feature = …)]`, so without the feature they are empty test binaries
rather than compile errors.


### The hard part: making a cascade look full duplex

Everything in this section is new. There is nothing to deviate *from*; what is
recorded is why each choice is the one it is.

- **One owning task, three sources, `biased`, inbound first.**
  `docs/architecture.md` §11 — *"Each gets an owning task, not a mutex."* The
  ordering is load-bearing rather than stylistic: audio is how barge-in arrives,
  and a loop that drained a generation before reading its input would hear the
  interruption after the turn it was meant to interrupt.

- **Cancellation is a `drop`, not a method.** Both stage streams —
  `ResponseStream` and `SpeechStream` — are cancelled by being dropped, and the
  traits carry no `cancel()`. A second way to stop a turn is a second thing that
  can be forgotten. The cost is two obligations on an implementation, both
  written into `stages`: a dropped stream must not block, and must be inert
  afterwards. Both real engines satisfy them the same way — the drop closes a
  channel, and the blocking thread notices at its next chunk or token.

- **Synthesis is per sentence, overlapped with generation.** `sentence`
  exists because waiting for a reasoning turn to finish before speaking adds the
  whole generation to the time-to-first-audio, which on a laptop is seconds.
  Three rules, one of which is a *bound*: an unpunctuated run is cut at
  `MAX_SENTENCE_CHARS`, so "the pipeline eventually speaks" is a property rather
  than a hope.

- **A soft terminator waits one character.** `.`/`!`/`?` end a sentence only
  when whitespace follows, so `3.14` and `example.com` are not split. The known
  false boundary is an abbreviation followed by a space (`e.g. `), and
  `an_abbreviation_followed_by_a_space_is_a_known_false_boundary` states it
  rather than hiding it: the alternative is a per-locale abbreviation dictionary
  that is wrong somewhere else, and the cost of this one is an extra breath.

- **Barge-in publishes the speech edge before the cancellation it caused.**
  `input_audio_buffer.speech_started` first — it is the Gateway's `userSpeaking`
  and the Injection Gate's first blocking input — then both streams are dropped,
  then `response.done` with status `cancelled`. Settling the caller's outcome
  matters: without it the response would sit until the 120 s inactivity
  watchdog.

- **Two `PlaybackCursor`s, doing what the type is for.** One on the input, so
  `audio_start_ms` and `audio_end_ms` are positions in the session's own audio
  rather than wall-clock guesses; one on the live response, so a barge-in can
  report how much speech reached the wire — the number
  `conversation.item.truncate` carries.

- **One response slot, a two-deep queue.** A create that cannot start is queued
  rather than refused, because there is no remote service to say no.
  `MAX_QUEUED_RESPONSES` is 2, which is exactly the depth the one real
  interleaving needs: a `response.create` that arrived while the user was
  speaking, plus the turn that user's speech is about to produce. A third is
  refused with ARGO's **catalogued** conflict vocabulary, so `via-realtime`'s
  bounded busy-retry ladder replays the payload rather than the Gateway
  inventing a retry.

- **The queue is drained in the run loop, not at each site that frees the
  slot.** `start` reached through `drain_queue` would be an `async fn` in its
  own call graph — an infinitely sized future. Draining once per loop iteration
  removes the recursion and makes "the slot is free, so start the next" true in
  one place.

- **The two `error` messages are wire vocabulary and are not localized.**
  `classify_error` matches on them and `via-realtime-openai`'s corpus is what
  matches; translating them would silently stop the busy-retry ladder in two
  locales out of three. Everything a *person* reads still comes from `via-i18n`,
  through `LocalError`.

- **The inbound channel is bounded and the outbound one is not.** The session
  rate-limits itself to what a client sends, so backpressure inbound is real and
  safe. The machine must never block while writing: a blocked machine stops
  reading its inbound channel, the session's writer blocks, the session's state
  task stops draining events, and the two halves deadlock.


### One bug not to port

`docs/architecture.md` §7's own heading. Upstream answers an unknown realtime
model id with an all-capabilities-false profile, and `dashscope.mjs:84-87` gates
`session.turn_detection` on `transportCapabilities.audioInput` — so an unknown
id **opens a session that hears nothing**. A local model id is by definition not
in the DashScope table, so on this path that fallback is reached on the *happy*
path.

`via-catalog` fixed the first half in phase 0: `local_realtime_model_profile`
carries real flags and there is no `unknown` family. This crate is the second
half, in three places:

- `LocalPipelineProvider::model_profile` and `LocalEndpointProvider::model_profile`
  are that profile, so `turn_detection` is `server_vad` and `audio_input` is
  true;
- `preflight` refuses a missing weights file as
  `LocalError::ModelFileMissing` **naming the path**, before a task, a channel or
  a session exists;
- `WeightsSet::verify` distinguishes *nothing installed* from *one file
  missing*, because telling an operator to reinstall when they have nine of ten
  files wastes a download.

`tests/contracts.rs::the_local_family_answers_the_turn_detection_row_that_upstream_leaves_null`
asserts both halves against the catalogued row, including the half that is about
what is **absent**: no `threshold`, no `silence_duration_ms`.


### `local-omni:endpoint` — configured, not forked

`docs/architecture.md` §7 calls this row *thin*, and the crate keeps it thin.
`via-realtime-openai` already owns the dialect, the four-candidate walk, the
first-frame probe, the GA/pre-GA schema repair, the subprotocol, the auth shapes
and the error corpus — all ported from ARGO and all paid for with device runs.
`LocalEndpointProvider` holds one and delegates.

What it does **not** delegate, and why:

- **`is_configured`.** `via-realtime-openai` demands an API key, because for
  OpenAI and Azure a missing key is the whole failure. A loopback server on a
  single-user machine needs none, so this provider asks for an **endpoint**
  instead — and refuses to guess one, because `via-catalog`'s `local-omni` row
  declares no default URL on purpose. `tests/endpoint.rs::the_credential_gate_is_the_local_one_not_the_cloud_one`
  asserts the inner client would have refused the configuration this one accepts.

- **`headers`.** The inner provider sends `api-key:` with an empty value when
  no credential is configured, and an empty credential header is the kind of
  thing a strict local server answers with a 400 that reads like a protocol
  error. Built here instead: a bearer only when there is one, plus the
  subprotocol on every candidate.

- **`model_profile`.** The inner provider answers `None`; this one answers the
  `local` family. See *One bug not to port*.

- **`transcribe_input` defaults to `false`.** `via-realtime-openai` defaults it
  **on** and records why (`dictation` is a mode whose whole output is the user's
  transcript). A local omni server produces the user's transcript itself, and
  `whisper-1` is an OpenAI-hosted model id such a server almost certainly does
  not have — asking for it would fail the whole `session.update`.
  `with_input_transcription(true)` turns it on for a gateway that really does
  host one. **This is a guess about software this repository cannot run**, and
  it is the deviation most likely to be wrong.

- **The connect diagnosis.** `diagnose_connect_failure` turns a refused TCP
  connect into *"no local server is running at X"* and a TLS failure into a
  sentence about the scheme, and leaves a failure the server **answered** — an
  HTTP status — exactly as it was. The subtle part is in `ANSWERED_MARKERS`: the
  walk's own aggregate wrapper is deliberately *not* a marker, because the
  wrapper carries every rejection inside it, so a walk in which any candidate got
  a status still matches on the status. Matching the wrapper would have made
  every walk look answered and thrown the diagnosis away on the one path that
  needs it.

**It is unverified on this machine, and it says so in four places.**
`docs/architecture.md` §7: *"CUDA-only; cannot be verified on this machine."*
`sgl-omni serve --enable-realtime` needs a CUDA host and this repository has
none. So the statement is `LocalMode::is_verified_on_this_machine`,
`LocalHealth::verified`, `LocalHealth::note` (a `via-i18n` key, so an operator
reads it in their own language), and the module documentation — not a comment in
a file nobody reading `/api/health` will open.

What *is* verified here is the provider surface, the configuration gate, and the
refusal diagnosis — the last against a real TCP port bound and released, so the
OS has just confirmed nothing is behind it.


### Configuration, and the one thing `via-core` does not carry

- **The mode is derived, not read from an environment variable of its own.**
  `via-core` owns environment reading and ships no local-mode variable, and this
  crate does not edit it. So the rule is `via-catalog`'s own — *"`pipeline` needs
  none and `endpoint` must be told where to look"* — and an endpoint configured
  selects `Endpoint` while nothing configured selects `Pipeline`, which is also
  `docs/architecture.md` §16's ordering. `LocalSettings::with_mode` overrides it
  and `LocalMode::parse` is there for whoever wires a config surface later.
  **The alternative not taken:** a `VIA_LOCAL_OMNI_MODE` variable on
  `via_core::config::names`. It would be one line in a shipped crate this phase
  is not allowed to edit, and it would still need the derivation as its default.

- **The model root is derived from `InstallPaths`, not from the environment.**
  `<config>/models/local-omni`, beside `via-wake-word`'s
  `<config>/models/wake-word`, with `LocalSettings::with_model_root` as the
  override. Same reason.

- **A `dashscope_realtime_url` equal to the DashScope default reads as
  *unset*.** `resolve_realtime_frontend` records DashScope's default whichever
  provider is active; `via-realtime-openai` reads the same field the same way and
  records the same reasoning. For this provider the consequence is sharper:
  reading it as an address would point the endpoint mode at a cloud host nobody
  chose.

- **The pipeline's model id is the reasoning file's *stem*.** There is no fixed
  table to look a local id up in, so a deployment that wants a legible id in
  `/api/health` renames the GGUF. `WeightsSet::discover` adopts a single `.gguf`
  in `<root>/reasoning` and **refuses to choose between two**, because picking
  would make the answer depend on directory order.

- **The mode is part of the configuration signature.** Switching between an
  in-process pipeline and a remote server changes the sample rates, the voice and
  whether a server is needed at all. A signature that did not move would leave
  every cached client silently wrong, which is the exact failure the signature
  exists to prevent.


### `StageOrigin` — the seam that lets CI run the pipeline

A provider whose engines load files must refuse to open when they are absent; a
provider handed working stages must not. `StageOrigin` makes that a declaration
rather than a guess: `Weights` (the shipping path, `is_configured` is "the files
are on disk") or `Supplied` (the caller's stages — `scripted`, or an embedder
that already owns a recognizer).

Without it, either every pipeline test would have to build a nine-file fixture
tree, or the weights gate would have to be weakened until it stopped being a
gate. `tests/pipeline.rs` uses `Supplied`; `pipeline::tests` and
`tests/contracts.rs` exercise `Weights` against real temporary directories,
including the broken-install case.


### What the two feature modules are, and are not

Both are written against the libraries' **own sources** —
`sherpa-onnx` 1.13.5 and `llama-cpp-2` 0.1.154 — and neither has been run
against real weights in this repository. No model files are checked in and no
lane here builds the native libraries. What each module asserts is the guard
that needs no weights: a missing file is refused **by name, before the library is
asked**, because both C APIs answer a missing model with a null handle and no
reason at all, so "the file is missing" and "the graph is corrupt" would
otherwise arrive identically.

Three mappings in `sherpa` are not straight-through, and each is a place a
naive binding would be wrong:

- **Edges, not levels.** `SherpaOnnxVoiceActivityDetectorDetected` answers *"is
  speech being detected right now"*, so it is true for every voiced block. The
  trait wants one `Started` per utterance, because the machine keys barge-in on
  it — a detector that reported `Started` on every block would cancel the same
  response over and over.
- **A running transcript, not a repeated one.** `get_result` returns the whole
  current hypothesis on every call. Publishing it verbatim would send an
  identical transcription delta to every client for every 20 ms of silence
  inside an utterance.
- **Endpointing is off.** The pipeline's turn detection is the VAD; a recognizer
  that ended turns on its own timer would disagree with it, and two endpoint
  detectors on one microphone is one more than the session can act on.

Two more in `llama`:

- **Partial UTF-8 is buffered.** A token is a byte sequence: a CJK glyph or an
  emoji spans two or three. `from_utf8_lossy` per token would put a `U+FFFD` on
  the wire for each one — visible in the transcript, and fed to the synthesizer.
- **`LlamaBackend::init` is once per process** and answers
  `BackendAlreadyInitialized` afterwards, so a second `LlamaResponder` in the
  same Gateway would refuse to load a model for no reason a user could act on.
  One `OnceLock`, and the first initialization's error is reported to every
  caller rather than to the first one only.

**`LlamaResponder` does not author tool calls.** A GGUF emits them as text in
its own model-specific format, and parsing that is a per-model concern rather
than a transport one — a wrong parser silently turns a tool call into spoken
prose. `ResponseDelta::Tool` is part of the stage *contract* and the machine
publishes it correctly (`tests/pipeline.rs::a_tool_call_is_published_as_an_output_item_and_its_arguments`);
a deployment that wants tool calls from a local model supplies a `Responder`
that wraps this one and parses its model's format. The `local` family's
`function_calling: true` is therefore a statement about the **pipeline**, not
about this particular GGUF wrapper.

**Kokoro selects a voice by integer.** VIA's configuration surface — and
`via-catalog`'s `DEFAULT_LOCAL_REALTIME_VOICE` — carries a name.
`SherpaSpeaker::with_voice` is the table between them, and a voice that is
neither a known name nor a number falls back to speaker 0 rather than failing the
utterance: a session in the wrong voice is recoverable, and a silent one is not.


### Testing

*199 tests: 143 unit, 28 pipeline, 16 endpoint, 10 contract, 2 doctest.*

- **The thesis is one test.**
  `tests/pipeline.rs::the_event_vocabulary_is_one_the_shipped_gateway_already_reads`
  drives a session through an utterance, a tool call, a truncate and a cancel,
  then asserts every event type it saw is in this crate's declared vocabulary and
  that every `response.*` name is in `via-realtime`'s shipped activity table. A
  name invented here would be a name nothing above this crate reads.

- **Every wait has a bound.** `EventLog::wait_for` is a deadline, and every
  wait in the suite goes through it. Phase 6's lesson was that a test whose
  property is "this terminates" needs its own bound, or a mutant that hangs takes
  the suite with it rather than failing it. `scripted`'s own
  `a_stalling_turn_never_yields_on_its_own` is the inverse case, and it is
  asserted with a deadline rather than by waiting.

- **Cancellation is observable.** Dropping a stream *is* the cancellation, which
  leaves a test nothing to assert — a discarded turn and a turn that never
  started look alike from outside. `CancelCount` closes that: both scripted
  streams count their own drops, and the barge-in test asserts the count moved
  for **both** the reasoning turn and the utterance.

- **The refusal is tested against a real socket.** A `TcpListener` bound on port
  0 and dropped leaves a loopback port the OS has just confirmed is free, which
  is the closest a test can get to "the operator has not started the server yet"
  — and it exercises the whole path: the walk, the aggregate it composes, the
  diagnosis and the localized sentence.

- **Contract values are parsed, never retyped.** `tests/common/mod.rs` reads
  `docs/reference/contracts.json`; `tests/contracts.rs` asserts the sample rates,
  the block sizes, the response failure statuses, the response-id lookup order,
  the tool-call round trip, the turn-detection row, the timeouts and the
  classification vocabulary against the shipped constants.

- **Mutation-checked by hand**, and the record is in the next section.


### Mutation check

`cargo-mutants` is not available in this environment, so the check was run by
hand against the predicates whose failure would be silent — the ones where a
wrong answer produces a working-looking session rather than an error. Each
mutation was applied, the suite run, and the mutation reverted.

| Mutation | Killed by |
| --- | --- |
| `SpeechEvent::is_voiced` returns `true` for `Ended` | `the_running_transcript_is_text_plus_stash…` (an extra delta) |
| barge-in emits `response.done` before `speech_started` | `barge_in_cuts_speech_mid_utterance…` (the ordering assertion) |
| `cancel_active` reports `RESPONSE_COMPLETED` | `barge_in_cuts_speech_mid_utterance…` (status) |
| `cancel_active` does not drop the speech stream | `barge_in…` (`cancelled_speech.get()`) |
| `has_model_turn` returns `true` in `dictation` | `dictation_transcribes_and_never_creates_a_response` |
| `finish_if_done` ignores `speech.is_none()` | `synthesis_is_per_sentence…` (a truncated turn) |
| `SentenceSplitter` treats a soft terminator as hard | `a_dot_inside_a_number_or_a_word_is_not_a_boundary` |
| `SentenceSplitter` drops the `MAX_SENTENCE_CHARS` bound | `an_unpunctuated_run_is_cut_at_the_bound…` |
| `WeightsSet::verify` returns `Ok` for one missing file | `one_absent_file_names_that_file_and_its_stage` |
| `RequiredPath::is_present` uses `exists()` for a directory | `a_data_directory_that_is_a_file_is_still_missing` |
| `diagnose_connect_failure` checks `answered` last | `a_walk_in_which_one_candidate_answered_keeps_its_status` |
| `LocalEndpointProvider::is_configured` delegates to the client | `the_credential_gate_is_the_local_one_not_the_cloud_one` |
| `preflight` skips `weights.verify()` | `supplied_stages_are_configured_and_weights_are_not…` |
| `model_profile` answers `None` | `the_local_family_answers_the_turn_detection_row…` |
| `transcription_delta` puts the tail in `text` | `the_running_transcript_is_text_plus_stash…` |
| `active_response_conflict_message` drops the invariant phrase | `the_conflict_refusal_classifies_as_a_busy_slot…` |

Two survivors, both recorded rather than fixed:

- **`LocalMode::is_verified_on_this_machine` returning `true` for both modes**
  is killed by `the_health_note_says_the_mode_is_unverified_on_this_machine`, but
  only because that test reads the note. The *fact* it encodes — that no CUDA
  host exists here — cannot be tested; it can only be stated.
- **`ScriptedSpeaker::chunks_for`'s ramp formula** is asserted for stability and
  length, not for its exact arithmetic. A different deterministic ramp would pass
  the suite. That is correct: the ramp is a fixture, not a contract.


### What is deliberately absent

- **No audio device.** `cpal` is not a dependency of this crate. Capture and
  playback are the host shell's (`docs/architecture.md` §3), and a provider that
  opened a microphone would be a second one.
- **No model downloader.** `via-wake-word` ships one because its artifact is a
  single pinned `.tar.bz2` on a release page. A GGUF and a voice pack are
  operator-chosen, of operator-chosen size, from operator-chosen hosts; a
  downloader with no pinned digest is a supply-chain hole, and one with a pinned
  digest would only work for the models VIA pinned.
- **No `via-app` wiring.** `via-app` is shipped and this phase does not edit it.
  `local_provider`, `register_local_provider` and `local_health` are the three
  functions the Gateway needs; `LocalPipeline::open` takes the `Stages` a
  feature-enabled build constructs.


### Workspace

`sherpa-onnx` and `llama-cpp-2` were already in the root
`[workspace.dependencies]` table, both with the comment that the consuming crate
marks them `optional`. This crate does exactly that. **No root manifest edit was
needed.**

Eight `via-i18n` keys were added in a new asset file,
`crates/via-i18n/assets/i18n/local.json`, under the `realtime.local_*` namespace.
A new file rather than an addition to `realtime.json`, so a concurrent edit to
that file cannot conflict with this one. Every `zh` value carries Han
characters, so **no `NOT_CHINESE_PROSE` entry was needed**.

No new directory was added under `crates/via-realtime-local/src/`, so
`scripts/risky_unwrap.py` needed no `SUBSYSTEM_DIRS` entry. The audit baselines
are unchanged: `risky_unwrap.py` 0, `panic_filter.py` 2, `unreachable_filter.py`
0, `brand_leak.py` clean.
