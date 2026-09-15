# Phase 4 deviations

Layer 2 — `via-conversation`: the two Markdown memory documents and their
behavioural-authority split, the flat `memory` and `notes` tools, the frontend
context blocks, and the session-end memory extractor.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-conversation`

*254 tests · clippy clean · 64/64 mutations killed · 18 deviations*

Ported from `server/src/conversation/{conversation-sync, frontend-agent-context,
frontend-memory-service, frontend-notes, markdown-context-store, memory-audit,
memory-extractor}.mjs`, plus the `memory` and `notes` halves of
`server/src/voice/tools/tool-call-handler.mjs`.
`server/src/core/memory-scopes.mjs` already shipped as
`via_core::memory_scopes` and is used, not restated.

### The one thing this phase exists to keep apart

`USER.md` is directive and `MEMORY.md` is data. Everything below is downstream
of that: the extractor's boundary classifier, the two `<user_preferences>` /
`<user_memory>` renderings, the refusal to route a change through anything but
`FrontendMemoryService`, and the fact that `ASSISTANT.md` is not reachable from
any of it. `tests/tools.rs::a_list_item_never_reaches_a_memory_document` and
`tests/extractor.rs::assistant_md_is_unreachable_from_a_proposed_change` are the
two that pin the negative space.

### Structural

- **`get_current_time`'s `local_time` is not ICU prose.** Upstream renders
  `Intl.DateTimeFormat(locale, {dateStyle:'full', timeStyle:'long',
  hour12:false})`, which needs a full ICU; VIA has no ICU4X dependency and the
  catalogue itself records the gap (*"Rust needs icu/ICU4X to reproduce it"*).
  `current_time_snapshot` renders a deterministic 24-hour form carrying the same
  four facts — `2026-07-23 Thursday 12:00:00 +08:00 (Asia/Shanghai)` — with
  English weekday and month spellings in every locale. The **instant**, the
  **zone** and the three sibling fields are exact, and those are what the model
  computes `schedule_reminder`'s `execute_at` from. Asserted in
  `tests/contracts.rs`.

- **Two dependencies were added to the root manifest**, both reported:
  `chrono-tz` (the IANA zone database — `chrono` alone cannot convert an instant
  into a client-supplied `time_zone`) and `iana-time-zone` (the host's own zone
  name, which is upstream's documented fallback and which `chrono` resolves
  internally but does not re-export; it was already in the graph as one of
  `chrono`'s own dependencies).

- **BCP-47 validation is structural, not ICU.** Upstream asks
  `new Intl.DateTimeFormat(locale)` and treats a throw as invalid.
  `is_well_formed_locale` accepts subtags of 1-8 alphanumerics separated by `-`
  with an alphabetic primary subtag. It agrees with ICU on the catalogued case
  (`not_a_locale` carries `_` and is rejected) and on every real tag; it accepts
  a well-formed-but-unknown tag, which ICU also accepts.

- **`ConversationSync` is a state machine plus an owning task.**
  `docs/architecture.md` §11 names four ordering invariants and none of them is
  in this crate, but `seq` is monotonic and the message array *is* conversation
  order, and both are observable in what the model is replayed. So
  `ConversationSyncHandle` is the actor — bounded `mpsc`, per-command `oneshot`,
  shutdown by dropping the last sender, and the state handed back on join — and
  `ConversationSync` underneath it stays synchronous and directly testable.

- **`FrontendNotesStore` and `MarkdownContextStore` use a `Mutex`, not a task.**
  Their ordering contract is the cross-process file transaction, which
  `via_store::with_file_transaction` already owns; within one process the
  requirement is mutual exclusion over a cache, which is exactly what a `Mutex`
  is. Two concurrent `add` calls have no defined order upstream either.

- **The notes document is a `via_store::VersionedJsonStore`.** Upstream
  hand-rolls the atomic write, the `version: 1` envelope, the
  `<path>.corrupt-<ms>` quarantine and the disable-rather-than-clobber rule
  inside `frontend-notes.mjs`; all four are `via-store`'s, and only the
  notes-specific parts (the mtime + content-hash refresh, the reload inside the
  lock, the per-owner retention) live here. `StoreMessages` holds plain `fn`
  pointers, so `notes_store_messages(locale)` bakes the locale into each arm
  through a macro rather than capturing it.

- **`IndexMap`, not a sorted map.** Notes order is observable three times: the
  file's key order, the tie-break of a `lists` result, and the capped candidate
  list an ambiguous name reports. A `BTreeMap` would silently rewrite all three,
  and `drop` uses `shift_remove` for the same reason. Pinned by
  `tests/notes.rs::dropping_a_list_keeps_the_order_of_the_rest`.

- **`MemoryExtractor::maybe_run` is `async` and the caller spawns it.**
  Upstream's is synchronous-gated and fire-and-forget; here the transcript comes
  from an async `TranscriptSource`, so the gates run first (they are still free)
  and the caller is told to `tokio::spawn` rather than await. The four "returned
  `null`" branches are preserved as `ExtractionOutcome::was_gated()`.

- **The debounce is a check-then-claim, not a check-then-set.** Upstream is
  single-threaded and gets check-and-set for free. Two concurrent session closes
  here would otherwise both pass the first check and both call the model, so
  `claim_run` re-checks under the lock. Behaviour is identical for one close;
  `tests/extractor.rs::concurrent_closes_claim_the_window_once` covers the other
  case.

- **`ExtractorLlm` and `TranscriptSource` are traits, and the HTTP client is
  behind a `http` feature.** Upstream's `createExtractorLlmCall` takes a
  `fetchImpl`; the trait is the same seam with a type. `testing` (default-on,
  matching `via-downstream`) supplies `ScriptedLlm` and `VecTranscripts`.

- **`MemoryToolOutcome::changed()` replaces `notifyMemoryChanged()`.** Upstream
  calls back into the realtime session to re-inject the memory blocks. The
  session belongs to `via-voice`; this crate reports the count and lets the
  caller decide.

- **The two tool surfaces live here rather than in `via-voice`.** Upstream keeps
  them in `voice/tools/tool-call-handler.mjs`, but the nine and six failure
  codes are contracts about *memory* and *notes*, and holding them beside the
  stores is what lets the refusal ladder be tested without a realtime session.
  The JSON Schemas and the tool descriptions stay `via-voice`'s (they are
  already `voice.tool.*` keys); this crate exposes the names, the action lists
  and the codes.

### Fidelity

- **CLOSED — `is_js_whitespace` now lives in `via-core`.** It was duplicated
  from `via_downstream::text` because taking the dependency would have added a
  `via-conversation → via-downstream` edge that upstream's own table
  (`server/test/dependency-boundaries.test.mjs`: `conversation → conversation,
  core, shared`) forbids, and this record said the predicate's real home was
  `via-core`. It is now `via_core::text::is_js_whitespace`, and
  `via-downstream`, `via-conversation` and `via-voice` each re-export it under
  their own module path, so no call site changed and no new edge was added —
  all three already depended on `via-core`.

  The move was not cosmetic. There were **four** copies, not two, and the third
  had drifted: `via-voice::text` spelled the predicate
  `char::is_whitespace() || U+FEFF`, which is a *superset* of ECMAScript's `\s`
  — it also matches U+0085 NEXT LINE, which Rust classes as `White_Space` and
  ECMAScript does not. `via-voice`'s `clean` therefore collapsed a character
  upstream leaves alone. The fourth was a private copy in `via-core::env` with
  the same drift, used by the `Number()` port. Both now read the enumerated
  set, and `via_core::text`'s tests assert the two disagreements with Rust in
  **both** directions, so a future "simplification" back to
  `char::is_whitespace` fails rather than passing quietly.

- **Two truncation units, both reproduced.** Upstream truncates with
  `[...value].slice(0, n)` (code points) in most places and `value.slice(0, n)`
  (UTF-16 code units) for `locale` and `workingDirectory`. `text.rs` has both,
  and `bounded_utf16` stops one character short rather than emitting a lone
  surrogate — identical to upstream for all BMP text, and the same call
  `via_downstream::text::bounded` recorded in phase 2.

- **`speech_ngrams` slices by code point where upstream slices by UTF-16 code
  unit.** They agree for every BMP character; astral characters are almost all
  `\p{S}` and are stripped by `speech_key` before they reach the bigram pass.

- **The extractor's classifiers are Chinese-pattern-only, as upstream's are.**
  `USER_PREFERENCE_PATTERNS` and `EXPLICIT_DIRECTIVE_PATTERNS` are reproduced
  pattern for pattern. VIA ships `en` and `ko`, so an English or Korean
  transcript matches neither — which means a `USER.md` write is **refused**
  (`user_directive_not_explicit`) and a `MEMORY.md` write is allowed. Both
  failures are in the conservative direction, and widening the patterns would be
  a behaviour change rather than a port. Same call `via-process` made for
  `backend_failure_code`.

- **`. ` in those patterns excludes only `\n`,** where JavaScript's also
  excludes `\r`, U+2028 and U+2029. The strings they run over have already been
  through `clean`, which collapses all four.

- **`to_lowercase`, not `toLocaleLowerCase`.** Rust's is locale-independent full
  Unicode; JavaScript's without an argument uses the host default. They differ
  only for Turkish dotted/dotless I, and a list key that folded differently per
  host would be worse than one that folds the same everywhere.

- **The notes change-detector hashes with SHA-256, not SHA-1.** The hash never
  leaves the process — it exists to notice that another Gateway wrote the file —
  so reaching for a broken primitive to match an implementation detail would be
  the wrong kind of fidelity.

- **`fileMtimeMs` does not rethrow.** Upstream's `statSync` propagates any
  non-ENOENT error out of `lists()` / `show()`, which would make a `stat`
  failure crash a read. Here an unreadable mtime counts as *changed* and forces
  a reload, which is the conservative answer and cannot panic.

- **The input-summary separators (` · ` and `；`) are constants, not `via-i18n`
  keys.** They are punctuation rather than prose and the catalogue pins them as
  part of the `<recent_conversation>` line format; the prose around them
  (`realtime.referable_inputs_suffix`) is a key. Same call `via-process` made
  for its failure/stderr separator.

### Reproduced rather than "fixed"

Three upstream properties look like defects and are kept, because changing any
of them would change observable behaviour:

- **`FrontendMemoryService::apply` does not hold a file transaction across its
  two persists.** It prepares every change first, so a refusal on the second
  document writes neither — but a second Gateway can still interleave between
  the two writes. Taking two cross-process locks in sequence would introduce a
  deadlock the single-document path does not have. `MarkdownContextStore::edit`
  (the single-document path) *is* transactional.

- **`clear` and `drop` have no code-level confirmation.** The destructive-intent
  requirement is prompt-level, in the catalogued `notes` tool description, and
  the catalogue's own `why` says so: *"There is NO code-level confirmation for
  clear/drop — the store executes them immediately."* A voice product has no
  turn in which to ask.

- **`drop` writes the owner's access timestamp for an owner it may have just
  removed.** Harmless, reproduced, and pinned by the test that asserts the owner
  leaves the file with their last list.
