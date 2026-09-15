# The patterns

Thirteen mechanisms, ordered roughly by how much they would cost VIA to retrofit
later. Each has: the rule, the evidence upstream offered for it, and what it
means for us.

---

## 1. The prompt is a contract with the weights

A fine-tuned model may only ever be sent the instruction it was fine-tuned on.
Not "roughly", not "plus one helpful extra line" — the exact text.

**Evidence.** Upstream ships *two* prompt builders for the same model, one per
training run, and keeps both forever: a user still holding the 1.1 weights must
keep receiving 1.1's prompt. Their comment is blunt about why the obvious
cleanup is wrong — editing one half without the other "silently serves a model
an instruction it never saw, and nothing fails when that happens."

**For VIA.** Our on-device model is the whole product; a prompt/weights mismatch
is invisible and permanent. The prompt belongs *with* the model artifact, not in
whatever module happens to call it.

---

## 2. Which prompt is data on the model, not a constant in the code

The prompt family is read from the model's own manifest (`promptTemplateID`),
resolved once at load, and injectable so a test can drive a template the running
build does not ship. Activation *refuses* a model whose manifest declares a
template the build does not know.

**For VIA.** Put `prompt_template_id` in the model manifest next to the digest
and the context length. Refuse to activate on an unknown value rather than
falling back to a default — a silent fallback here is pattern 1's failure with
extra steps.

---

## 3. One text of record, mirrored under test

Every prompt exists once as a plain `.txt` file. The Swift constant, the Python
eval harness, and any other copy are asserted **byte-identical** to that file by
tests that run before an eval spends money.

**Evidence.** They found their own gap here: the mirror check originally covered
Python-against-file only, leaving the copy users actually receive "guarded by
nothing but the sentence above."

**For VIA.** `include_str!` the prompt from a `prompts/` directory rather than
writing a Rust string literal, and add a test that the eval fixtures read the
same bytes. This is cheap now and near-impossible to retrofit once three copies
exist.

---

## 4. Kill the per-input prompt router

They had a `PolishMode` that picked a prompt shape per transcript. It sent the
"keep as one paragraph, no formatting" variant on **1,548 of 1,690** corpus
cases. The fix was to delete the router: one fixed prompt per model family, with
formatting decided by rules *inside* the prompt.

**For VIA.** If we are tempted to branch the prompt on transcript length, shape,
or detected intent, measure the branch distribution first. A router that picks
one arm 92% of the time is not a router, it is a constant plus a bug surface.

---

## 5. Enrichments are a per-family decision, argued in the code

Conditional additions — locked-language hint, app-name context, user vocabulary,
a short-input guard — are added to some families and deliberately withheld from
others, with the reasoning written down at the point of the omission.

**Evidence.** Their local-model builder refuses two enrichments the cloud
builder carries, because each would have changed the prompt on cases the quality
numbers came from (the short-input guard alone fires on 24.7% of the measured
corpus). Their fine-tuned-model builder refuses *all* of them: the training
distribution had none, so adding one is off-distribution drift.

**For VIA.** Every conditional line appended to a prompt is a divergence from
the condition we measured. Either it was present during measurement, or adding
it invalidates the measurement. Write which, in the code.

---

## 6. Content-not-instruction framing, and no wrapper the model was not trained on

Two separate mechanisms, often confused:

- **Framing.** The system prompt's closing paragraph states that the transcript
  is content the user is composing, never a directive — including the case where
  the user literally dictates "ignore your instructions."
- **Wrapping.** A `<TRANSCRIPT>` delimiter is used *only* for the model that was
  tuned with it. Embedded copies of the tag are neutralised with a zero-width
  non-joiner so dictated text cannot close and reopen the boundary. Models not
  trained on a wrapper get a bare transcript — adding one makes them echo the
  tags into the user's output, which upstream had to write a cleanup path for.

**For VIA.** Take the framing everywhere. Take the wrapper only where the
training format demands it, and if we do wrap, neutralise the delimiter.

---

## 7. Restraint is the goal, and it needs examples, not adjectives

The strongest line in their cloud prompt is the one telling the model that most
inputs need almost no change, and naming the reason: changing something in every
message damages the good ones, and that is damage the user never sees because
they assume cleanup only helps.

**Evidence.** A terser, rules-only variant of their local prompt scored **0 of
60** on restraint traps. Naming an action without demonstrating its boundary
made small models over-apply it — they bulleted single sentences.

**For VIA.** Budget tokens for worked examples before you budget them for rules.
For small models, an example is the only thing that carries a boundary.

---

## 8. Precise vocabulary beats plain language for small models

Their local prompt deliberately uses linguistic terms — *reparandum*, *discourse
marker*, *orthography* — and the comment says so: that vocabulary is "what lets
a small model apply a rule it cannot infer from prose."

**For VIA.** Do not "simplify" a prompt for a small model by removing the
technical term. The term is the compression.

---

## 9. Route on identity and reported location, never on the name

Model identity (first-party, known third-party) outranks where the model runs.
Local-versus-hosted is decided by the daemon's own report, never by the model's
name or parameter count. A missing answer fails safe to the local assumption.

**Evidence.** A user who pulls a known model themselves gets the same prompt the
managed path sends, from the same builder — otherwise the prompt would exist
twice, in two modules, with only one of them pinned by the identity test.

**For VIA.** One builder per prompt family, reachable from every transport. A
second copy behind a different transport is pattern 3's failure wearing a
different name.

---

## 10. Anything user-tunable inside a prompt is a closed enum

Their third-party normaliser takes a control line — three axes, each a closed
enum whose raw value *is* the wire token. Every trained combination is
reachable; nothing else is representable.

**For VIA.** If we expose prompt knobs, the type system should make an
off-distribution value unwritable. Not validated — unwritable.

---

## 11. Do deterministic work deterministically, before the model

User vocabulary is applied by a corrector pass *before* polish. Emoji formatting
is a separate deterministic step. Very short inputs skip the model entirely.
None of these are asked of the LLM.

**For VIA.** Every rule we can implement exactly is a rule we should not spend
prompt tokens, latency, or reliability on.

---

## 12. Prompt changes are measured, in one judging window, with the caveats kept

Prompts are versioned (v6→v7, v33→v38, L1/L2/L3). A change ships on a sealed
benchmark with keys authored independently of the prompt, all arms graded
together, and the baseline re-captured in the same change.

**Evidence.** What is most worth copying is the *honesty*: they record that
their judge shared a family with three of four arms and mark the fourth's number
"indicative"; they record that the winning model's judge-stability check failed
at 5.2pp against a 5.0 limit; they note the cheapest, fastest candidate was also
the worst and a price-led choice would have shipped a regression.

**For VIA.** A prompt change with no re-captured baseline is a guess. A
benchmark whose caveats are not written down is a guess that will be quoted as a
fact in six months.

---

## 13. Decide once, carry the decision

The plan object carries the resolved family and mode, so the builder, the output
validator, and the telemetry all read one decision. Upstream previously
re-derived it a second time purely to stamp telemetry, and notes that the two
could disagree.

Relatedly: the managed path and the bring-your-own-copy path for the *same*
model get deliberately distinct ids, because telemetry groups by model and a
shared id merges two populations into a number nobody can read.

**For VIA.** Resolve the prompt plan once per turn and pass it down. Give
distinct deployments distinct ids even when the weights are identical.

---

## What this costs us if we skip it

Patterns 1, 2, 3 and 9 are structural — retrofitting them means touching every
call site and re-running every measurement. They are worth doing before the
second model lands. The rest can be adopted incrementally.
