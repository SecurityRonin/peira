---
name: anti-summarization
description: >-
  The anti-summarization pass — seven probing questions for distilling any source (a
  document, a codebase being reverse-engineered, a corpus) without letting compression
  smooth away its contradictions, unsupported claims, ambiguity, and omissions. Use when
  summarising, distilling, auditing, or reviewing a source and rigour must survive the
  compression. Triggers: "summarise", "distil", "what did I miss", "decompress",
  "where does it get murky", "audit this document".
---

# The anti-summarization pass

> 盡信書不如無書 — to wholly trust the book is worse than to have no book. (孟子)

A summary is a compression, and compression is where rigor leaks — and the damage is
**invisible from inside the summary**, because what compression removed is not in the text
you are reading. So each door below is aimed at the **gap** between your summary and the
source (never at the summary), and every answer must be a **noun** — a yardstick, a means
of knowing, a named rival, a break-point, a missing item, a worked case. "Nothing" is not
a noun, so it is not an available answer, and producing one forces you back into the source.

## The seven doors

- **1 · Direction** *(ask first — aims the scrutiny)* — Which way do I *want* this to come out, and which way hurts more if I'm wrong? If they're the same direction, my motivation is pushing me toward the costly mistake.
- **2 · Words** — What is the yardstick (the standard a judgement is made against), and do the words hold still (one term, one meaning)?
- **3 · Ground** — How does each load-bearing claim know, and can that way of knowing reach this far? Testimony dressed as observation; a number with no denominator/unit/date; causation from correlation; an absence "proven" by a search that couldn't have found the thing.
- **4 · Rivals** — What else would look *exactly* like this evidence, and who got left out of the ring (the strongest opponent, the deleted losing side, the unaddressed corner)?
- **5 · Breaks** — Where does it break: on its own pages (contradiction), against the world, after a date (staleness), past a case (the forced universal, the missing limit, the claim nothing could falsify)?
- **6 · Silence** — What should be here and isn't (the omitted argument, dataset, caveat, voice — the dog that didn't bark), and what was cut without a reason on record?
- **7 · Transfer** *(the exit exam)* — Can I run it on a case the source never mentions — predict, act, build — or only restate it? What I can't apply is what I flattened.

## Stamp every finding by species

Set by one test — *is the defect in the source as written?*

- **[SOURCE FAULT]** — yes: the source contradicts itself, asserts without evidence, omits, leaps. A **finding**.
- **[MY LIMIT: compression]** — no: the source had it and my distillation dropped it. A **task**.

The same symptom flips owner ("no rivals named" is a source fault if the source named none, my limit if it named them and I lost them). **"What did I smooth over for fluency?" gets no door** — asked head-on it always returns "nothing"; it surfaces only as the compression stamp above and as failure at Transfer.

## Guardrail

- An **all-empty pass is a tell of a *skipped* pass**, not a flawless source — re-run, don't celebrate.
- A **"none found" is itself an absence claim** — write "searched [where] for [what], found none," never a bare "nothing here."
- **Default to "not established."**

## Canonical source and the checkers

This block is the portable surface. The full framework — the crosswalk from each door to the critical-thinking lens behind it, the seven-versus-five history, and every lens in full — is emitted by the peira binary this plugin ships with:

- **`peira method anti-summarization`** prints the canonical document, behind a line naming the peira version (so a copy made from it never drifts silently).
- **`peira lens <ID>`** shows any lens in full.
- The **peira MCP server** (registered by this plugin) exposes `check_prose` — run it on a draft before it reaches a reader — plus the vault-backed `examine`, `gates`, `freeze` and `verify` for a modelled claim graph.

Change the framework in peira, never only here.
