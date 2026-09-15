# Phase 6 deviations — `via-wake-word`

Layer 1 — on-device wake-word detection: the verified model installer, the
keyword configuration, the detector seam and its streaming PCM front end.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-wake-word`

*158 tests (default) · 163 + 1 `#[ignore]`d live test with
`--features sherpa,http` · clippy clean · 65/65 mutants killed · 23 deviations*

Ported from upstream `server/src/voice/wake-word/{model-manager,sherpa-detector}.mjs`
and `server/test/wake-word-detector.test.mjs`.


### The build constraint that shapes the crate

`docs/adr/0001-placement.md` records that VIA's native dependencies —
`sherpa-onnx`, `llama-cpp-2`, `cpal` — are exactly what keeps VIA out of ARGO's
workspace, because ARGO requires every crate to cross-compile clean to four
targets. A native toolchain in every VIA CI lane, for a feature most builds
never exercise, is that same decision made badly at a smaller scale.

So the crate is split at a trait:

| Half | Feature | What |
| --- | --- | --- |
| the pipeline | *default* | `ModelManager`, `ModelArtifact`, `KeywordSet`, `TokenInventory`, `WakeWordSettings`, `DetectionConfig`, `WakeWordDetector`, `WakeWordStream`, `ScriptedDetector`, `ScriptedFetch` |
| the engine | `sherpa` | `SherpaDetector` |
| the fetcher | `http` | `HttpModelFetch` |

`cargo build -p via-wake-word` needs no C toolchain, no prebuilt native library
and no network. `cargo test -p via-wake-word` exercises download, digest,
extraction, install, keyword resolution, resampling and detection against
`ScriptedFetch` and `ScriptedDetector`, with no bytes leaving the machine.
`tests/sherpa.rs` is `#![cfg(feature = "sherpa")]`, so without the feature it is
an empty test binary rather than a compile error.


### What it is built on rather than restating

| Owned by | What |
| --- | --- |
| `via-audio` | `SampleRate::HZ_16000`, the resampler, the frame buffer, every PCM16 ↔ `f32` conversion |
| `via-core` | `Config` — the enable flag, the phrase, the install root, the locale — and `InstallPaths::wake_word_model_directory` |
| `via-i18n` | every sentence a person reads, and `Locale` |
| `via-store` | `write_atomic`, `FILE_MODE` |

**The feature extractor's 16 000 Hz is read, not retyped.** The catalogued
`sherpa KWS detection config` contract pins eleven values.
[`phase-0.md`](phase-0.md) records that `samplingRate` belongs to `via-audio`,
which asserts it against the same row; `DetectionConfig::sample_rate` *is*
`SampleRate::HZ_16000` and `tests/contracts.rs` asserts that identity rather
than the number. The other nine — `featureDim`, `numThreads`, `provider`,
`debug`, `modelingUnit`, `maxActivePaths`, `numTrailingBlanks`,
`keywordsScore`, `keywordsThreshold` — are asserted here, each parsed out of
the catalogue.


### Corrections to upstream, found by running it

Three of the deviations below were found by installing the real artifact and
opening the real engine (`tests/sherpa.rs::live_install_detects_the_vendors_own_keyword`,
`#[ignore]`d and gated on `VIA_WAKE_WORD_LIVE_MODEL=1`). All three are latent in
upstream and invisible until a phrase changes — which is exactly what
`docs/architecture.md` §16 makes possible. The first was found the hard way: a
first attempt at the live test exited the test process with status 255 and no
panic and no backtrace, because that is what the library does.

- **A keyword file the model cannot encode ends the process; it does not
  return an error.** `sherpa-onnx/csrc/utils.cc`'s `EncodeBase` logs *"Cannot find ID for
  token …"*, `InitKeywords` logs *"Encode keywords failed."*, and the library
  exits. Upstream can live with that: its token line is a hard-coded literal
  that was correct when it was written. VIA's is configuration, so a mistyped
  token would take the Gateway down rather than disable a feature. **Added:**
  `TokenInventory`, which reads the model's own `tokens.txt`, applies the same
  rule `EncodeBase` applies — *inventory first, marker second, otherwise out of
  vocabulary* — and answers `WakeWordError::UnknownTokens`. It runs at
  **install**, so a keyword file that would kill the Gateway is never written
  next to a model, and again at **open**, because a caller can assemble a
  `DetectionConfig` by hand.

- **The same function has a second way in.** `EncodeBase` parses a `:score` or
  `#threshold` payload with `std::stof` and **no `try`**, so `:later` is an
  uncaught `std::invalid_argument` rather than a rejected value. **Added:**
  `TokenInventory::malformed_markers` and `WakeWordError::MalformedMarker`. The
  numeric check is deliberately *lenient* where `stof` is lenient — `stof`
  converts a valid prefix and ignores the rest, so `:2.0dB` is a score of 2.0 to
  the library and must be one to the guard, because rejecting a file the engine
  would accept turns a working wake word off. It over-rejects by exactly three
  spellings `stof` would take (`inf`, `nan`, hex floats), and over-rejection
  fails safe: it names the word, where under-rejection ends the process.

- **The keyword file's display column is one whitespace-free word.**
  `EncodeBase` splits the line on whitespace and takes the `@`-prefixed *word*
  as the label; a second word is read as another token and fails to encode. The
  model's own fixtures show the convention — `keywords_raw.txt`'s
  `LIGHT UP @LIGHT_UP` becomes `keywords.txt`'s `L AY1 T AH1 P @LIGHT_UP`.
  Upstream never meets this because its phrase is a single CJK word. **Added:**
  `WakePhrase::label()`, which joins the phrase's words with `_` exactly as the
  vendor does, and a `Keyword` whose *text* (what a person is told to say) and
  *label* (what the engine reports) are separate. `KeywordSet::locale_of`
  matches the label. For a single-word phrase the two coincide, which is why the
  catalogued upstream line still round-trips byte for byte.


### The phrase

- **No default, anywhere.** Upstream hard-codes `config.wakeWord` with no
  environment override (`config.mjs:503`). `docs/architecture.md` §16 makes the
  phrase "a configuration value, never a literal" and
  [`rebrand.md`](../rebrand.md) rows 147-149 mark upstream's phrase RENAME,
  noting that changing it requires a regenerated keyword model. **VIA has not
  chosen a phrase**, so this crate ships no constant, no `Default` and no
  fallback. An unconfigured phrase resolves to
  `DisabledReason::NoPhraseConfigured` — a state the caller handles, not a
  panic. Upstream's phrase appears in this repository only where
  `tests/contracts.rs` parses it out of `contracts.json` at run time to prove
  the keyword-line format round-trips; it is never written into a source file.

- **The token line is configuration too, beside the phrase.** It cannot be
  derived: with `modelingUnit: cjkchar` the catalogued model spells English in
  ARPAbet with stress digits (`L AY1 T AH1 P`) and Chinese in tone-marked pinyin
  (`n ǐ h ǎo`), and which spelling a phrase takes is a property of that model's
  263-symbol inventory. `WakePhrase` therefore carries the pair and refuses to
  guess one from the other.

- **Per-locale is a `BTreeMap<Locale, Keyword>`.** §16 says "three locales may
  mean three models", so the cap is structural rather than checked —
  `MAX_KEYWORD_MODELS` is `Locale::ALL.len()`, and there is no fourth locale to
  insert. Iteration is `en`, `zh`, `ko` whatever order the table was built in,
  which is the order keyword lines reach disk. Two locales may share one
  artifact (`en` and `zh` do, on the zh-en model), and then they share one
  download and one install directory; `KeywordSet::artifacts` deduplicates by
  artifact id.

- **Added: the announced phrase is cross-checked against the keyword model.**
  `VIA_WAKE_WORD` is what the sleep message says out loud; the keyword model is
  what the microphone actually listens for. If they disagree, nothing errors and
  nothing logs — the Gateway simply tells the user to say one phrase while
  listening for another, forever. `WakeWordSettings::from_config` records the
  configured phrase and `resolve` reports the disagreement as
  `DisabledReason::PhraseMismatch`, turning a silent product failure into a
  log line. Upstream has no counterpart because it has nothing to disagree with.

- **`realtime.asleep` is filled from the resolved phrase.** The key already
  carries a `{wake_word}` placeholder where upstream interpolated its own
  literal (recorded in `via-i18n`'s crate documentation).
  `WakeWordSettings::asleep_message` fills it with the phrase *that locale
  actually resolves to*, and answers `None` when the locale resolves disabled —
  so the sentence can never name a phrase that would not wake anything.


### The installer

- **The archive never touches the disk.** Upstream streams the body to a temp
  file, re-reads it to hash it, re-reads it again to extract it, and cleans the
  file up in a `finally` (`model-manager.mjs:56-72,79-108`). VIA hashes and
  extracts the same buffer it downloaded. It removes a temp file, its cleanup
  path, and the window between hashing a file and reading it back. The cost is
  holding roughly 33 MB during a first run, once.

- **`keywords.txt` is rewritten on every call.** Upstream's completeness check
  includes the keyword file (`complete()`, `model-manager.mjs:37-39`), so once a
  model is installed the keyword file is never regenerated. That is correct for
  a hard-coded phrase and wrong for a configured one: an operator changing
  `VIA_WAKE_WORD` would keep listening for the previous phrase, with no error
  anywhere. VIA's short circuit therefore tests the four **downloaded** members,
  and writes the keyword file afterwards either way — validated first, then
  atomically through `via_store::write_atomic`, since it is being replaced under
  a detector that may be reading it. `is_installed` still reports the five-file
  check upstream's `complete()` does.

- **Concurrent installs serialize instead of sharing a promise.** Upstream keys
  a module-level `Map` on the target directory so two callers await one download
  (`model-manager.mjs:36,109-119`). VIA holds a per-target async mutex: the
  second caller waits, then finds the model installed and takes the short
  circuit. Identical for the success case — one download — and better for the
  failure case, where upstream hands the second caller the first caller's error
  and VIA lets it retry.

- **Only a basename is ever joined.** Upstream takes `basename(header.name)`
  for its own reasons (`model-manager.mjs:57`). VIA keeps that and states the
  consequence: a `../../` member in a hostile archive cannot escape the staging
  directory, because `Path::file_name` is `None` for `..` and can never contain
  a separator. It is tested against an archive built with the traversal written
  straight into the tar header, which `tar-rs`'s writer refuses to produce
  through its normal API.

- **The four downloaded members are checked before anything reads them.**
  Upstream's single completeness check runs after the keyword file is written.
  VIA checks the four first, so an archive missing `tokens.txt` is reported as
  the incomplete archive it is rather than as an I/O error from the token guard
  trying to read it.

- **An empty keyword file is refused before the download.** A model installed
  with no keywords loads, runs, consumes CPU and never matches anything, which
  is the hardest wake-word failure to notice. Upstream cannot reach the state;
  VIA can, because the phrase is configuration.

- **The digest comparison ignores hex case.** Upstream compares
  `digest !== WAKE_WORD_MODEL_SHA256` against a lowercase literal. VIA's
  artifacts are constructible, so a digest configured in upper case still
  verifies. The value is public, so there is nothing here for a timing side
  channel to leak.

- **The HTTP client is an injected trait.** `ModelFetch`, with `ScriptedFetch`
  in the default build and `HttpModelFetch` behind `http`. Every branch — a 404,
  a 2xx with no body, a transport failure, a truncated archive, a digest
  mismatch, a hostile member name — is reachable in a test with no network and
  no fixture server. `FetchResponse::body` is an `Option` specifically so
  upstream's `!response.ok || !response.body` keeps both of its halves.


### The detector and the front end

- **`keywordsBufSize` is not a field VIA carries.** Upstream must pass it
  because the Node binding takes a length beside the pointer
  (`sherpa-detector.mjs:49`); the Rust binding derives it from the buffer.
  `DetectionConfig::keywords_buf_size()` exists so the catalogued value is
  asserted — in bytes, not characters, which is the confusion the contract
  exists to prevent — rather than assumed. Likewise `keywords: ''` is
  `keywords_file: None` and `debug: 0` is `debug: false`; both equivalences are
  written down at the call site.

- **The keyword buffer, not the keyword path.** Upstream's comment gives the
  WASM reason. VIA has no WASM filesystem and keeps the buffer for a different
  one: the phrase is configuration, so the keyword text can change between two
  runs against one installed model, and a buffer cannot go stale the way a path
  can.

- **Added: a resampling front end.** Upstream's host always captured at exactly
  the extractor's rate, so `sherpa-detector.mjs:66-81` passes samples straight
  through. VIA's does not. `WakeWordStream` puts `via-audio`'s frame buffer,
  downmix and band-limited resampler between the socket and the detector, and
  builds the resampler from `WakeWordDetector::sample_rate()` rather than
  assuming 16 kHz.

- **A torn PCM frame is carried, not dropped.** The catalogued
  `pcm16 -> f32 conversion` says an odd trailing byte is dropped, and it is —
  by `via_audio::pcm16le_to_f32`, the stateless decoder upstream uses on one
  self-contained base64 payload at a time. A streaming front end is handed
  arbitrary socket-sized chunks, where dropping the byte shifts every following
  sample and turns the rest of the chunk into noise, so `WakeWordStream` uses
  `FrameBuffer::append_pcm16le`, which carries it. Both behaviours are asserted
  side by side so the difference cannot be mistaken for an accident.

- **An empty chunk never reaches the detector.** Upstream returns early on
  `!samples.length` (`sherpa-detector.mjs:70`) and so does every
  `WakeWordDetector`; `WakeWordStream` additionally does not make the call,
  because an FFT resampler legitimately produces nothing until it has a whole
  chunk.

- **`ScriptedDetector` and `ScriptedFetch` ship in the default build**, not
  behind `cfg(test)`. `via-voice` and `via-app` cannot exercise "the user said
  the phrase" without a detector that can be told to say so, and a double behind
  `cfg(test)` is invisible to them. The doubles are also where the
  `WakeWordDetector` contract — an empty slice is a no-op, a match resets the
  stream — is pinned in a form this crate's tests can assert with no ONNX
  runtime.


### Workspace

- **`bzip2` and `tar` added to the root manifest.** The artifact is one
  `.tar.bz2`, and a verified download that cannot open the archive it verified
  is not a model manager. `bzip2`'s default backend is `libbz2-rs-sys` — pure
  Rust — so the default build stays toolchain-free; the `bzip2-sys` feature is
  deliberately not enabled. `tar` takes `default-features = false` to drop
  `xattr`: the four members are extracted by name and written with an explicit
  mode.


### Not claimed

- **A real detection is not in CI.** `tests/sherpa.rs` covers the guards in
  front of the engine with no model files at all. The one test that installs the
  catalogued artifact and detects a real spoken keyword is `#[ignore]`d *and*
  gated on `VIA_WAKE_WORD_LIVE_MODEL=1`, because CI should not depend on a third
  party's release page staying up. It has been run: it verifies that
  `WAKE_WORD_MODEL_URL` still resolves, that `WAKE_WORD_MODEL_SHA256` still
  matches the bytes it serves, that `WAKE_WORD_MODEL_FILES` names four members
  that are really in the archive, that the catalogued configuration opens a
  spotter, and that the vendor's own `LIGHT_UP` keyword is detected in the
  vendor's own recording through `WakeWordStream` — twice: once at the
  recording's own 16 kHz mono, and once with the same audio upsampled to 48 kHz
  stereo, so the band-limited resample and the downmix are exercised against
  real weights rather than only against a scripted detector.

- **`via-conformance`'s registry is untouched.** The eight rows it carries
  against `Crate::ViaWakeWord` are this crate's to claim, and claiming them is a
  `via-conformance` edit rather than a `via-wake-word` one.
