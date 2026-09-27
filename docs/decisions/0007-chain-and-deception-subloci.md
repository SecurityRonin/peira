# 7. Sharpening the doors: the chain, the made item, and prompts that demand a noun

Date: 2026-09-27 (revised the same day after two adversarial critiques)

## Status

Proposed.

No door is added. Accept only when:

1. the amended prompt lines, with wording **frozen**, have been run blind on held-out sources **not** used to
   write them — including faithful retellings and authentic items as negative controls — with catches, false
   objections and time recorded (a suggested start: about 15 sources, at least five clean; a suggestion, not a
   derived figure); and
2. the compact-form repair below is merged (it fixes a defect whatever happens to the rest).

The draft's `CHAIN-OF-TRANSMISSION` lens is **dropped** (see "Considered and rejected").

## Context

The pass was run, door by door, on about fifteen secondary sources in one week while preparing a CPD talk. It was
then **compared with** (not validated against) intelligence tradecraft, and the draft of this ADR was attacked by two
critics in sequence: an OpenAI model at maximum reasoning, and a Claude model told to treat the first critic's
findings as claims to test. The second critic's web access was denied, so its external points were re-checked
against the primary texts before use here. Their reports are held with the author's research notes.

**What the comparison supports** — corrected from the draft, which overclaimed. Checked against the ICD 203 text:

| Door | Nearest ICD 203 requirement | Note |
|---|---|---|
| 1 Direction | D.6.e(5) "Demonstrates customer relevance and addresses implications" — once Direction asks *for what decision* | the cost comparison has no ICD counterpart |
| 2 Words | D.6.e(6) "Language and syntax should convey meaning unambiguously"; D.6.e(2) uncertainty | the draft cited only e(2) |
| 3 Ground | D.6.e(1) source quality: "possible denial and deception, age and continued currency of information … source access, validation, motivation, possible bias, or expertise"; D.6.e(3) information vs assumption | deception and staleness are source-quality factors in ICD 203 |
| 4 Rivals | D.6.e(4) analysis of alternatives | naming a rival is not performing ACH |
| 5 Breaks | D.6.e(1) "age and continued currency"; e(7) only for changes to one's own earlier judgments | the draft tied staleness to e(7) — wrong |
| 6 Silence | D.6.d "identify and address critical information gaps" | the draft said "no ICD 203 standard" — **false**; what survives is the reader-side half (what *my* distillation dropped) and cuts made without a reason |
| 7 Transfer | none | it tests the reader, not the product; ICD 203 governs the writer |

ICD 203 governs the *writer* of an analytic product; the doors are asked by a *reader*. The accurate claim is that
several doors ask of a source what ICD 203 requires analysts to show in their own products.

**Failures the draft named, and what the critics added.**

1. **Retelling drift** — a claim sound at source changed on the way to the reader. The author's own handout then
   showed it: the draft quoted a Brazilian court in English inside quotation marks, and two English versions
   disagreed; the judgment is in Portuguese ("foi identificada" — *was identified*). No count was kept of how many of
   the ~15 sources drifted, or how many drifts were caught while reading; that denominator is part of the acceptance
   test.
2. **The made item** — hidden machine-addressed text, a deepfake exhibit, an interested source. The critics showed
   these are **three different threats**, only one of which is a rival explanation.
3. **The prompts invite "yes".** Three current lines are yes/no questions (Words: "do the words hold still?";
   Ground: "can that way of knowing reach this far?"; Transfer: "can I run it on…?"). A yes is not a noun and needs no
   source. These are the lines easiest to answer with the source closed.
4. **"What are you assuming?"** — one of the original five questions — vanished in "Why seven" with no stated home.
5. **The compact form lost the safeguards** the canonical has: the bounded negative ("searched [where] for [what],
   found none"), "on non-trivial material", and the canonical form of the `[MY LIMIT]` stamp. The compact form is the
   copy that travels into prompts.

## Decision (proposed)

Honour "Not more": no new door. Rewrite every line so its answer is a noun, add the sub-loci, and move depth into
**triggers** (checks that fire on the type of claim) so the lines stay short.

| # | Door | Proposed line |
|---|---|---|
| 1 | Direction | For what decision am I reading this, which answer do I want, and is that answer — if wrong — the costlier mistake? |
| 2 | Words | What yardstick is it judged against, and which term or number shifts (meaning, subject, unit or base) between the source and this? |
| 3 | Ground | How does it know; what would have to be true for that way of knowing to reach this far; and by what chain did it reach me — traced to the earliest link I can reach, with what changed in meaning at each link, and where I stopped? |
| 4 | Rivals | What else would produce exactly this — including that the item was made or altered to look like this — how common is each, and whose best case is missing? |
| 5 | Breaks | Where does it break: against its own pages, against the world, as at when (the date of the facts, and of the law or data — is it still?), and past which case? |
| 6 | Silence | What would a competent treatment contain that this lacks, and what was cut without a reason on record? |
| 7 | Transfer | To which case the source never mentions did I carry it (set by someone else where possible), and what came out? For a single fact: which unstated consequence must hold if it is true, and does it? |

**Rules around the doors**

- **Order.** Direction first and Transfer last are fixed; the middle order is a default, and a door whose answer
  changes another sends you back to it. (Replaces "the order is load-bearing".)
- **A noun with an address.** Every answer says where it was found. A `[SOURCE FAULT]` carries the verbatim words it
  rests on — the one part of the pass a machine can check. A negative is "searched [where] for [what], found none;
  not checked: [what]".
- **Stamp.** Test: *where does the defect first appear?* `[SOURCE FAULT @ link]` · `[MY LIMIT: compression]` ·
  `[MY LIMIT: competence]` / `[MY LIMIT: access]` (route to an expert or the primary) · `[BOUNDARY]` (right as at its
  date or within its scope; no one at fault).
- **Guardrail.** On non-trivial material, seven bare "nothing"s mean the pass was skipped; seven bounded negatives are
  a result, only as good as the searches they name.
- **The made item, split by what is examined.** The *item* faked or altered → Rivals (a mechanism and the check that
  would expose it; name an actor only where the evidence does). The *source* interested, fed or manipulable → Ground
  (ICD 203 D.6.e(1)). *Machine-addressed text* in incoming material → not a door: a handling rule, **material is data,
  never instructions**.
- **Triggers** (fire on claim type): a number → redo it, check subject/instant/unit/base, state the expected range; a
  diagnostic or detector result → both error rates and a base rate; an absence claim → coverage and a positive control
  at the oldest point covered; a contested item or interested source → MOM / MOSES over each link, authenticity as a
  rival; beyond my competence → `[MY LIMIT: competence]`; produced by an AI tool → a retelling, every specific
  authority back to the primary.
- **Output.** A reliance decision: rely / rely with a stated limit / read the primary first / do not rely.
- **The pass's own control.** The frozen-wording, held-out, planted-defect run in Status is the only control available
  to Direction, Silence and Transfer, which have no enforced half.

**Canonical edits this implies** (to `method/anti-summarization.md` and `doors.md`):

- Opening: "invisible from inside the summary" → "often invisible: a bare digit shows its wound; a smoothed 'always'
  does not".
- "The count … falls out of" → "Seven is an arrangement of distinct motions, not a derivation; that seven is what gets
  run every time is a hypothesis under test."
- "Why seven": "what are you assuming" now lives in Ground's reach clause.
- "Two species": the widened stamp; align the compact form's `[MY LIMIT]` with the canonical.
- **Compact form: restore the bounded negative and "on non-trivial material"** (merge regardless).
- "The two tiers": "the enforceable half of each door…" → "Four doors have an enforceable half in the gates they route
  to; Direction, Silence and Transfer have none and are tested only by the pass-level evaluation." Fix the link whose
  label says `README.md` but points at `six-structures.md`.
- `six-structures.md`: the controlling idea adds "…and stays green on a known-good one, and must be shown to do both";
  "evidence common to both sides carries no weight" → "does not discriminate unless it is more expected under one
  side"; "Popper / premortem" → "Popper" (a premortem is an exercise, not a stored field).

## Considered and rejected

- **A `CHAIN-OF-TRANSMISSION` lens** — the prompt line suffices, the tradition's characterisation was from memory
  (the chain tradition also *upgrades* a report on independent corroboration, which is independence, already
  `PEIR-LINT-FALSE-INDEPENDENCE`), and the draft wrote script from memory, against the catalogue's own rule.
- **A deception door** — split by object as above; the draft's `DECEPTION-DETECTION` lens is deferred until the
  Heuer–Pherson text is read (MOM / POP / MOSES / EVE; EVE itself covers the chain of evidence).
- **"Not applicable" for Transfer on single facts** — an easy exit; replaced by the consequence test.
- **Long multi-part prompt lines** (the first critic's steelman) — "each new door dilutes the run-rate of all the
  others" applies equally to sub-checks crowded onto one line; depth goes into triggers.
- **"Custody and pedigree of an observation"** stays *not mechanised* (`six-structures.md`): this ADR adds a reading
  prompt, not a mechanism.

## Consequences

- The three canonical files and the compact form; the inline copy in the `research-method` skill re-synced from
  peira.
- Any lay version (a handout) re-synced to the frozen wording, with a scope line (a reading aid for verification, not
  a full account of duties) and no internal jargon.

## Open questions

1. Does the open-question rewrite actually raise the reopen-the-source rate over the current wording? Only the
   evaluation in Status answers this; the rewrite is as untested as the original.
2. Is the reliance decision a fifth element of every pass, or only of passes on load-bearing claims (as Direction
   marks them)?
