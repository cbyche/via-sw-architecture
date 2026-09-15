# Phase 5 deviations — `via-realtime-dashscope`

Layer 1 — the two realtime providers `qwen-audio-agent` ships: cloud DashScope
over the beta OpenAI Realtime dialect, and a user-run huggingface/speech-to-speech
endpoint over the GA one.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-realtime-dashscope`

*154 tests · clippy clean · 46/46 mutants killed · 9 deviations*

Ported from `server/src/voice/providers/dashscope.mjs` (133 lines),
`server/src/voice/providers/s2s.mjs` (138 lines) and the registration half of
`server/src/voice/providers/registry.mjs`, against the provider half of
`server/test/realtime-provider.test.mjs`.

`via-catalog` already owns the four model profiles, the provider keys and
aliases, the endpoint defaults and the configuration-signature hash;
`via-realtime` owns the two traits, the registry, the session and both wire
dialects; `via-i18n` owns every sentence; `via-core` owns the environment
reading. None of the four is restated here.


### ⚠ The bug that is not ported

`docs/architecture.md` §7, *"One bug not to port"*, and the reason this crate
has a `preflight()` at all.

Upstream's `dashscope.mjs:84-89` gates `session.input_audio_format` and
`session.turn_detection` on `profile.transportCapabilities.audioInput`. With any
of the four catalogued profiles that gate *is* the contract — all four declare
`audioInput: true`. What makes it dangerous is what sits under it:
`resolveDashScopeRealtimeModelProfile()` answers an unrecognised id with an
**all-capabilities-false** profile, and this code turns that profile into a
session with no input format and no turn detection. It opens successfully and
then never hears the user: no error, no log line, no failed connection.

Upstream's protection is a *separate* check in a different file —
`realtime-provider.mjs:134-140` refuses to connect when
`modelProfile.family === 'unknown'`. It works, and it is one forgotten line away
from not working, because the fallback that makes it necessary lives nowhere
near it. For VIA that matters concretely: a **local** model id is by definition
not in the DashScope table, so on the on-device path the all-false fallback is
on the happy path rather than in the error path.

VIA keeps the gate and removes the fallback:

1. `via_catalog::resolve_dashscope_realtime_model_profile` returns `Err`. There
   is no `unknown` family in VIA at all — which is also why upstream's check
   cannot be transcribed as written, since there is no longer a family to
   compare against.
2. `DashScopeProvider::preflight()` is that check, re-expressed where the
   knowledge lives. It turns the `Err` into `RealtimeError::UnsupportedModel`
   **naming the id**, and `via-realtime` runs `preflight()` before
   `is_configured()` and both before the socket.

`DashScopeProvider::profile()` returns `None` for exactly the ids `preflight`
refuses, so no session that ever opens can be built from one — the deaf payload
is unreachable rather than merely unlikely.

| Test | What it pins |
| --- | --- |
| `tests/dashscope.rs::an_unknown_model_is_refused_before_a_socket_is_opened` | the transport saw **no frame at all** — the equivalent of upstream's `assert.equal(frontend.ws, null)` |
| `tests/dashscope.rs::an_unknown_model_outranks_a_missing_credential` | the ordering: `preflight()` before `is_configured()` |
| `tests/dashscope.rs::capabilities_are_never_inferred_from_the_shape_of_an_id` | `qwen3.5-omni-plus-realtime-future` is an error, not an Omni profile |
| `tests/dashscope.rs::the_refusal_names_the_resolved_id_not_the_padded_one` | the message interpolates the trimmed id, as upstream's `modelProfile.id` did |
| `tests/contracts.rs::an_unknown_model_is_refused_at_connect_as_the_catalogue_says_it_must_be` | the catalogue's own rationale — *"family 'unknown' makes RealtimeFrontend.connect() reject up front"* — still describes what VIA does |

The mutation `turn_detection always null (the deafness bug)` is killed by
`tests/dashscope.rs::the_default_model_configures_smart_turn_only`.


### Structural

- **Both providers ship from one crate.** `docs/architecture.md` §15 lists
  `via-realtime-s2s` as a separate phase-6 crate. `s2s.mjs` is 138 lines, needs
  no dependency `dashscope.mjs` does not already have, and
  `providers/registry.mjs` registers the two as **one built-in pair**
  (`createRealtimeProviderRegistry({ providers: [dashscopeProvider, s2sProvider] })`)
  — which `builtin_provider_registry()` is. Splitting 138 lines into its own
  crate would give the port a seam the implementation VIA follows does not have.

  They are not one provider wearing two hats: the dialects differ (beta vs GA),
  four of five capability flags differ, the response-start budgets differ by 2×,
  and the error corpora share exactly one arm. Each difference is a separate
  contract with its own test.

- **A provider is given the config fields it reads, not the `Config`.**
  Upstream's providers close over the `config` module singleton and re-read it on
  every call. `DashScopeSettings` / `SpeechToSpeechSettings` are the read set,
  named once, with `from_config` / `from_frontend` constructors. Three reasons,
  in order of weight: the credential is a `via_core::Secret` so a provider's
  `Debug` cannot print it (the trait requires `Debug`, and an operator formatting
  a session with `{:?}` is exactly how a key leaks); a provider test should not
  have to build a whole `Config`; and the read set is a checkable statement about
  what a provider depends on.

- **The three response instructions are reached by i18n key, not by import.**
  Upstream's providers import `speakResponseInstructions`,
  `resultResponseInstructions` and `permissionResponseInstructions` from
  `server/src/voice/frontend-tools.mjs`. That file is `via-voice`'s
  (`docs/architecture.md` §9), and `via-voice` sits **above** the provider crates
  — it depends on `via-realtime`, and `via-app` wires the providers into it. A
  provider that imported `via_voice` would invert the crate graph. So
  `crate::prompt` reads the same three `via-i18n` keys `via-voice` reads. The
  text is not duplicated; only a three-line accessor is.

- **A provider carries a `Locale`.** Upstream has one locale and interpolates its
  sentence directly. `RealtimeProvider::{missing_configuration_message,
  connect_timeout_message}` take a locale, but `build_speak_response` and the two
  injections do not — the instructions they compose are model-visible text, so
  the provider holds the session's locale in its settings.

- **`configuration_signature()` is computed, not cached.** Upstream's providers
  define no `configurationSignature`, and `providers/registry.mjs` falls back to
  `config.realtimeConfigSignature` — the signature of whichever provider is
  *active*. `via-realtime`'s trait requires each provider to answer for itself,
  so each builds its own `via_catalog::RealtimeIdentity` under the identity shape
  the catalog declares for it.
  `tests/{dashscope,speech_to_speech}.rs::the_signature_is_the_one_via_core_resolved_for_the_same_environment`
  asserts the two agree for the active provider, which is what keeps
  `/api/health.realtimeConfigurationSignature` reproducible by a client.

- **The provider label is `Qwen-Audio-Realtime`, not `DashScope`.** Upstream
  carries two labels for this provider and they disagree:
  `shared/realtime-provider-catalog.mjs:25` says `'DashScope'` (the
  *configuration* label, `via-catalog`'s row) and `dashscope.mjs:45` says
  `'Qwen-Audio-Realtime'` (the *voice provider* label, which
  `describeActiveRealtime` publishes and which every connection error
  interpolates). `docs/rebrand.md` records the disagreement and keeps both. VIA
  reproduces both, in the same two places.

### Corrected, not copied

- **A GA tool missing a field declares it as `null` rather than dropping it.**
  Upstream's `s2s.mjs:72-77` writes
  `{ type, name: tool.function.name, description: …, parameters: … }`; a missing
  field is `undefined`, which `JSON.stringify` silently omits. VIA writes
  `null`, so the service's own schema validation names the field. A silently
  absent `parameters` is the harder bug to find, and neither shape is valid
  input the service accepts.

- **A tool that is not in the beta shape is passed through, not emptied.**
  Upstream's `tool.function.name` on a flat tool yields `undefined` for all three
  fields, producing `{ type: 'function' }` — a tool that exists and does nothing.
  VIA returns the tool unchanged, so the service reports the schema error and the
  capability does not disappear without a diagnostic.

### Added, which is not a port

- **`patterns_compile()`.** The DashScope `fatal` corpus is four alternation
  patterns with a word boundary, three optional-separator classes and a `.*`
  span; `via-realtime` writes its one pattern by hand and records why, but this
  is not that. The compiled sets are `Option<RegexSet>` rather than an
  `expect()`, and a `None` degrades classification to `Other` — the class that is
  *shown to the user* — so the failure direction is a visible auth error rather
  than a silently suppressed one. `patterns_compile()` is the assertion that it
  never happens, checked by `tests/contracts.rs`.

- **ASCII case folding, stated.** Every upstream pattern carries JavaScript's
  `i` flag with no `u` flag, which is ASCII folding: `/k/i` does not match
  U+212A KELVIN SIGN there, but Rust's Unicode-aware `(?i)` does. The haystack is
  lowercased with `to_ascii_lowercase` and the patterns are written lowercase,
  and `\b` is written `(?-u:\b)` for the same reason. This is an exactness
  measure rather than a change; `src/classify.rs::classification_never_folds_non_ascii_case`
  is the one input where the two spellings differ.

### Testing

154 tests. Every catalogued value is **parsed out of
`docs/reference/contracts.json`**, never retyped — including the two error
corpora, which are catalogued as JavaScript `RegExp` literals:
`tests/common/mod.rs::Contract::js_regexes` compiles the catalogued source and
`tests/contracts.rs` runs both classifiers against a 30-message corpus,
comparing every answer. That is what proves the Rust transcription rather than
the transcription proving itself.

`tests/session.rs` drives both providers through a real `RealtimeSession` over
`via_realtime::testing::test_transport`, because three contracts only meet on
the wire: the provider's payload, the dialect's envelope, and the capability
flag the session branches on. Every test there is
`#[tokio::test(start_paused = true)]`, which is what makes
`s2s_is_ready_the_moment_the_update_is_written` an assertion — a session that
waited for an acknowledgement would burn the 25 s connect budget and fail rather
than hang.

**Mutation-checked**: 46 hand-built mutants across all five modules — the
capability flags, both signature inputs, the deafness gate, every classification
arm and every regex alternative, the key orders, the credential predicates and
the prompt tag. 46 killed, 0 survived, 0 skipped.
