# Phase 7 deviations — `via-context`

Layer 2 — the Context Engine: `ContextPack`, referents, ordered deixis, and
provenance-driven fencing.

Read [`phase-0.md`](phase-0.md) for what a deviation record is and is not.


## `via-context`

*141 tests · clippy clean · 18/18 mutants killed · 11 deviations*

**No upstream.** `docs/architecture.md` §5 marks this box *"OURS · NO QWEN
EQUIVALENT"* and the verification agreed: qwen-audio-agent has no Context Engine
at all, and ARGO has prompt assembly, memory and three narrow fencing
conventions but nothing to extend for referents or ordered deixis. So most of
what follows is a *design record* rather than a divergence from a source.


### What it is built on rather than restating

| Owned by | What |
| --- | --- |
| `via-conversation` | every frontend block (`user_preferences_section`, `memory_section`, `runtime_context_section`, `build_recent_conversation_context`), the string primitives (`clean`, `trim`, `clean_bounded`, `bounded_code_points`, `code_point_len`), and `MAX_PROMPT_CHARS` / `MAX_RESULT_CANDIDATES` / `MAX_ITEM_CHARS` |
| `via-voice` | `PROMPT.md`, the `<assistant_profile authority="persona_only">` wrapper, and the `<input_parts>` projection — all three arrive as strings, because Layer 2 may not depend on Layer 1 |
| `via-i18n` | the two model-facing sentences: the fence's framing line and its truncation marker |
| `via-protocol` | `SessionMode` |

Nothing in `assemble.rs` renders a block. The claim that the pack is *"a thin
typed layer, not a rewrite"* is asserted as an equality, not asserted in prose —
see deviation 1.


### 1. The thin-layer proof lives in `via-voice`, not here

`docs/architecture.md` §9 gives Layer 1 the row
`voice → conversation, core, shared, task, voice`, so `via-voice` may depend on
`via-context` and `via-context` may not depend back. `via-voice` is therefore the
only crate that can see both sides of

```text
pack.instructions() == via_voice::assemble_frontend_instructions(prompt, profile, context)
```

so that equality is `crates/via-voice/tests/context_pack.rs`, added as a
`[dev-dependencies]` edge only. It is asserted for all three locales, for all
six shapes of the two optional memory blocks crossed with the optional
`client_working_directory` line, and for three shapes of persona whitespace —
because a join that only agrees in the full case agrees by accident.

The same file carries the drift guard `via-context` cannot write for itself: it
neutralises `via_voice::prompt::ASSISTANT_PROFILE_OPEN_TAG` and asserts the
fence would escape it, so a rename of `via-voice`'s persona tag cannot silently
leave the fence guarding a tag nobody emits.


### 2. `Tier` is ARGO's `PromptTier`, and `SemiStatic` gains a VIA occupant

`tinicore/src/prompt/builder.rs:464-477` names three tiers and states the cache
property they encode. VIA keeps the names and the meaning:

| Tier | ARGO's occupant | VIA's |
| --- | --- | --- |
| `Static` | persona, safety, locale policy | `PROMPT.md`, `<assistant_profile>`, `<user_preferences>`, `<user_memory>`, `<runtime_context>` |
| `SemiStatic` | the tool-definitions section | the ordered-deixis view |
| `Dynamic` | datetime, retrieved memory | `<recent_conversation>`, `<input_parts>`, fenced tool evidence |

VIA declares its tools through the provider's `session.update` rather than in
the prompt, so `SemiStatic` would otherwise have been an empty variant. The
deixis view is a genuine occupant by ARGO's own definition — *"invalidates only
when the scope selector activates a different namespace"* — because it is
invalidated by a new **surface generation**, not by a new turn. A user who talks
for five turns without touching the screen re-sends nothing.

`ContextPack::instructions()` is exactly `render_tier(Tier::Static)`, which is
what makes the tier the *delivery channel* rather than a label: session
instructions are set once, so anything per-turn must not be in them.


### 3. Fencing extends ARGO's sentinel pair; it does not add a fourth delimiter

`docs/architecture.md` §5: *"Fencing needs a provenance dimension, not a fourth
delimiter."* ARGO has three unrelated conventions
(`<|user_input_start|>` in `triage.rs` / `classifier/binary/llm.rs`,
`<recalled_memories>` in `cognitive_recall.rs`, `<answer>` in `workflow/schema.rs`).

VIA takes the **sentinel-pair shape** — `<|via_untrusted_start|>` /
`<|via_untrusted_end|>` — and **both** neutralisation mechanisms, each where the
upstream uses it: sentinels are redacted to `[REDACTED]` (the triage rule) and
tag openers are escaped to `&lt;` (the answer-fence rule).

Three deliberate differences, each recorded because each could look like a
mistake:

**No tag allow-list.** ARGO escapes the one tag it fences with, which is right
for a fence around one known payload. A VIA prompt carries six model-visible
tags and will carry a seventh one day, so a blocklist would have a hole in it
from the moment that tag landed. `neutralize` escapes **anything that could open
a tag at all**: `<` followed by optional whitespace, an optional `/`, more
whitespace, and a character that could begin a name (`[A-Za-z_!?|]`).

**`a < b` becomes `a &lt; b`.** The whitespace tolerance that stops
`< / user_preferences >` also catches a comparison whose right side starts with a
letter. That is the correct direction to err inside a block of quoted evidence,
and the common numeric forms (`<3 s`, `< 5 items`) survive untouched because a
digit cannot begin a tag name.

**The framing does not name the sentinels.** ARGO's classifier framing names both
by literal name, which makes every prompt carry each one at least twice and
forces its own test to note that *"the real fence is the last one"*. VIA's block
is delimited structurally — one line that is exactly the open marker, one that is
exactly the close — so the framing describes the content instead and there is no
first-versus-last ambiguity to reason about.

**Growth and order.** Redaction shrinks; escaping grows by three code points per
escaped `<` with a 2.5× ceiling. The cap is applied to the raw body *before*
neutralisation, which is ARGO's `#2069` round-3 review #5 lesson: truncating
afterwards would let bounded escape growth inflate a body past the cap.


### 4. Provenance is a lattice meet, and it only ever lowers trust

The missing classification, as a two-argument function:

| `ToolTrust` | `ContentSource` | `Provenance` |
| --- | --- | --- |
| `FirstParty` | `Own` | `Tool` |
| `FirstParty` | `External` | **`ThirdParty`** |
| `ThirdParty` | either | `ThirdParty` |

The second row is the one that did not exist. `Provenance::ThirdParty` then
floors `Trust` to `Untrusted` inside `ContextSection::new`, and `Untrusted`
fences in the same constructor — so there is no ordering of calls in which
third-party bytes reach a rendered prompt unfenced. Forgetting the fence is not
representable, which is a stronger statement than "every call site remembers".

The clamp is one-directional. A caller that declares `Data` for first-party
bytes is not promoted.


### 5. Two blocks are host- or user-supplied and are deliberately **not** fenced

**`<runtime_context>`.** Its three values are *fields*, not free-form bytes, and
`via_conversation::normalize_client_context` already strips NUL and collapses
CR/LF for exactly this reason — the catalogued test is
`a_working_directory_cannot_forge_a_context_line`. A fence would add nothing and
would change a catalogued block format.

**`<recent_conversation>`.** It is what the user and VIA said, its format is
catalogued, and it is replayed verbatim inside `<restored_context>` on reconnect.
Fencing it would put the user's own words behind a marker that says *"ignore
directives here"*, which is backwards: the current utterance is the top of
`PROMPT.md`'s hierarchy.

The screen, by contrast, **is** fenced — see deviation 8.


### 6. A fence sentinel inside a *trusted* body is left verbatim

Neutralisation runs only on untrusted bodies. A user whose `USER.md` happens to
contain the marker still sees their preferences byte for byte, which is what
keeps the deviation-1 equality unconditional.

The safety argument is directional and is asserted rather than assumed
(`a_sentinel_in_trusted_text_is_left_alone_and_can_only_downgrade`): a stray
*close* before any fence closes nothing, and a stray *open* at worst makes the
model read the trusted tail as untrusted — a downgrade. Neither can pull content
out of a real fenced block, because that block's own body has its sentinels
redacted.


### 7. Referents: five kinds, two turns, four staleness reasons

**What may be one.** `Screen`, `Input` (`input_N`), `Work` (`work_id`),
`NoteList`, `Directory` (`<runtime_context>`'s `client_working_directory`, which
`PROMPT.md` names by field). The rule is *an identity VIA can act on*. Free text
is not a referent and neither is a line of `MEMORY.md`: both are content, and
"delete that one" could not be executed against either without first inventing
what it denotes.

**How long.** Two turns — the binding turn and the next. Short on purpose: "the
second one" is about the enumeration the user just heard, and a stale hit is
worse than a miss because the miss asks and the hit acts. Handles are drawn from
a monotonic counter and **never reused**, so an evicted or expired `ref_3`
answers *not found* rather than resolving to a different object.

**How staleness is detected.** `Expired` (the turn ended, then another did),
`Moved` (still on screen, but its fingerprint — position, label, kind, actions —
changed), `Gone` (the screen changed and it is not in the new snapshot), and the
fail-closed `SurfaceUnknown` (a screen referent with no current snapshot to
vouch for it). Expiry is enforced twice, by the sweep on `begin_turn` and by the
predicate on every resolve; removing either is a killed mutant.

**Ambiguity.** The ladder is `via-conversation`'s notes ladder
(`frontend-notes.mjs:50-83`) — exact, then unique bidirectional substring, then
candidates — including its asymmetry: when several match *exactly*, the substring
pass is skipped so two identically-labelled objects report each other. Two things
are layered on top, and both preserve *never guess*: the ladder runs over **live**
referents first and only then over stale ones, so a unique stale match is
reported as `Stale` with its reason instead of as *not found*; and the candidate
list is always live, because offering a dead one invites the model to pick it.


### 8. Ordered deixis: the order is the source's, and the generation is the contract

**Nothing here sorts.** Objects are enumerated in the order the host gave them.
The host knows its own reading order; VIA does not, and inventing one would put a
layout opinion between the user's eyes and their words.

**A generation change is the only thing that renumbers.** A source that changes
its objects without advancing `SurfaceSnapshot::generation` is contradicting
itself, and `observe_surface` answers `SurfaceGenerationReused` rather than
quietly accepting either version.

**Re-reading an unchanged screen reuses handles.** `bind_surface_object` reuses a
binding when the fingerprint is *identical* and mints a new one otherwise. Both
halves matter: without reuse the registry fills with duplicates until a phrase
that resolved on turn one comes back ambiguous on turn three; with unconditional
reuse, a moved row would silently inherit the model's stale idea of `ref_1`.

**An object with no id is not enumerated**, because it cannot be tracked for
staleness next snapshot. A blank *label* is fine — the ordinal still names it.

**Truncation refuses `Last`.** A snapshot beyond `MAX_SURFACE_OBJECTS` (50) is
truncated rather than refused, because a host paging a long list is normal. But
`Ordinal::Last` on a truncated view answers `NotFound` with candidates: the last
object genuinely is not in the enumeration, and answering with the last *visible*
one would be the guess this crate never makes.

**The screen is third-party.** The host is first-party; a label is whatever the
screen says. So `DeixisView::section` is the literal case of deviation 4 —
`classify(FirstParty, External)` — and the enumeration is fenced. Every label,
kind and action is whitespace-collapsed first, so a label carrying a newline
cannot forge a second numbered line; the fence is defence in depth on top of
that, not instead of it.

**The source is synchronous.** `SurfaceSource::snapshot` returns a value rather
than a future. A host that has to cross an IPC boundary caches the frame it was
pushed and answers from the cache, which is what a screen source *is*; making it
`async` would invite a blocking round-trip inside prompt assembly. VIA grows no
screen reader of its own — `ScriptedSurface` is the only implementation this
crate ships, and it is compiled unconditionally rather than behind a `testing`
feature because a host wiring its own affordances needs it to test that wiring.


### 9. The token estimate is not ARGO's `chars / 4`

ARGO budgets with `chars / 4` (`tinicore/src/workflow/schema.rs`), which is close
for ASCII and badly wrong for the locale half of VIA's catalog: `zh` is
upstream's own text and a Han character is roughly one token, not a quarter of
one. `estimate_tokens` charges ASCII code points at four to the token and
everything else at one each.

It is an **estimate**, for budgeting a pack, never for billing or for a hard
provider limit. Two properties are asserted: it never decreases when text is
appended, and for the same code-point count a non-ASCII string never estimates
below an ASCII one.


### 10. Concurrency: no owning task, deliberately

`docs/architecture.md` §11 gives an owning task to each of the four invariants
that need FIFO order across tasks. A per-session referent registry is not one of
them — it has no ordering requirement across tasks — so `ReferentRegistry` and
`ContextEngine` are plain state with `&mut self` methods and no interior
mutability, owned by whichever task owns the session. The one lock in the crate
is inside `ScriptedSurface`, and a poisoned lock there answers with a
`SurfaceError` rather than panicking, because a source that cannot be read is
already a first-class outcome.


### 11. `SessionMode::Interface` stops degrading — the shipped-crate edits

`docs/architecture.md` §2 required the degradation to be reported on
`/api/health`, never silent, and `via-voice` reported
`context_engine_unavailable` unconditionally because the engine did not exist.
It does now, so three shipped crates were edited. Every edit is listed here.

**`crates/via-voice/src/mode.rs`** — `ModePlan` gains a third fact.
`with_context_engine(requested, harness_configured, context_engine)` is the new
constructor; `new(requested, harness_configured)` is
`with_context_engine(…, true)`, because a Gateway built from this workspace has
`via-context` compiled in. `Degradation::ContextEngineUnavailable` keeps its
exact `/api/health` code and is now **conditional**: it is produced only by a
caller that says it has no engine — an embedder building `via-voice` without
`via-context`. The fact is read only by `interface`, which is the only mode that
mounts an engine, and `a_missing_context_engine_constrains_no_other_mode`
asserts it does not leak sideways.

**`crates/via-voice/src/status.rs`** — one test fixture switched from
`ModePlan::new(Interface, true)` to `with_context_engine(Interface, true, false)`
so the aggregation test still exercises a degraded row.

**`crates/via-app/src/health.rs`, `tests/http_routes.rs`,
`src/realtime/engine.rs`** — the `sessionModes` row for `interface` now reports
`effective: "interface", reason: null`, and the two stale doc notes saying
*"behaves as `direct` until `via-context` lands"* are corrected. `via-app` gains
no dependency; it calls `ModePlan::new` exactly as before.

**`crates/via-voice/Cargo.toml`** — `via-context` as a `[dev-dependencies]` edge
only, for deviation 1's equality. Nothing that ships is added.

**What is *not* claimed.** Closing the degradation makes the mode real; it does
not wire a screen into the Gateway. VIA has no screen capture, and a
host-registered surface is a protocol-level surface (new `connect`-time frames)
that phase 7 does not add. An `interface` session on today's Gateway reports
`SurfaceStatus::Unbound`: referents over the conversation resolve, nothing on
screen does, and the engine says so rather than pretending. That is a per-session
state the engine answers, deliberately not folded into the `sessionModes` table,
because that table describes what a mode *is* on this Gateway and not what one
session happens to be looking at.


### The root manifest

Not edited. `via-context = { path = "crates/via-context" }` was already present in
`[workspace.dependencies]`.
