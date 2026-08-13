# Reporting to a tribunal

peira's Court Mode exists because of what is in this file. If you are not producing evidence for a
tribunal you can skip it — but the three-layer rule is worth reading anyway, because collapsing the
layers is the most common way a technical report overstates.

---

## Three epistemic layers — never collapse them

```mermaid
flowchart TB
    L1["1 — OBSERVED FACTS<br/>what the evidence directly shows"]
    L2["2 — FORENSIC INFERENCE<br/>'consistent with' · 'strongly consistent with'<br/>'not consistent with'"]
    L3["3 — LEGAL CONCLUSION<br/>crime · fraud · breach · ownership"]

    L1 -->|"state as findings"| OUT["the report"]
    L2 -->|"state as hedged inference"| OUT
    L3 -->|"HAND BACK to the tribunal"| T["'the Court may draw<br/>its own conclusions'"]

    L3:::forbidden
    classDef forbidden fill:#5a1a1a,stroke:#d33,color:#fff,stroke-width:2px
```

Layer 3 is **never** the expert's. Layer-3 overreach concentrates in **callout boxes, opinion
summaries and section conclusions** — review those first, because they are where a hedge gets
dropped for punchiness.

peira mechanises the layer-1/layer-2 boundary as the causal ladder gate
(`PEIR-CAUSAL-RUNG-UNREACHED`) and the substance/function gate (`PEIR-FUNCTION-AS-SUBSTANCE`) —
noting that each reaches a verdict only when the claim declares `causal_rung:` or `aspect:`; a
claim that declares neither is not examined by them. It does **not** detect layer-3 overreach —
that remains a human obligation.

---

## The substitution table

| Overstatement | Correct form |
|---|---|
| "confirms X" | "is consistent with X" |
| "proves X" | "establishes / provides evidence of X" |
| "is consistent only with X" | "is strongly consistent with X" |
| "contradicted by" | "is not consistent with" |
| "establishes that P was victimised" | "is consistent with P having been victimised" |
| "does not belong to D1" | "has no established relationship with D1's known identifiers" |
| "this is not a matter of dispute" | remove — that is for the tribunal |
| "X is confirmed active and targeted" | "X activity confirmed; the pattern is consistent with targeting" |

**Never make a definitive negative ownership finding from absence of a connection.** *"No
relationship found"*, not *"is not D1's"*. And before any negative finding at all, ask whether the
source it rests on has passed the controls in
[`source-register.md`](source-register.md) — a silent failure upstream becomes a confident
negative here. **Never state a party's witness-statement fact as your own finding** — attribute it.

---

## The principle of least disclosure

This principle governs **correspondence about your own process** — a conflict-check confirmation,
a scope or fee letter, a reply to an instructing solicitor. It does not govern findings: the scope
of a search a finding rests on is part of the finding, and the hard limit below takes over there.

Answer with the **bare minimum information that fully and honestly answers the question**. Every
additional word, category, qualifier or volunteered detail is surface a cross-examiner can probe, or
use to define a limit you did not intend.

- **In a conflict-check confirmation, do not enumerate the categories you searched.** *"I checked
  clients, employers and relationships"* invites *"so you did not check X?"* — and the enumeration
  asserts **less** than the broad form, not more: any list omits a category. *"I have checked the
  names provided against my records"* is the wider claim, and complete.
- **Do not assert more than the question asks.** A conflict check asks whether you have a conflict.
  It does not ask you to certify *"no prior relationship of any kind"* — a different, broader and
  falsifiable claim.
- **Do not explain reasoning you were not asked for.** Volunteered rationale becomes a target.
- **State each thing once.** Duplicated assertions drift apart across drafts and hand a
  cross-examiner two phrasings of one fact to compare for discrepancy.

### The hard limit: least disclosure is not concealment

It governs *words about your own process in correspondence*. It is overridden, absolutely, for two
things that **MUST** be disclosed fully and early:

1. **Any conflict or connection that is not obviously immaterial.** Do not decide materiality
   yourself — disclose it and let the instructing party assess. **Non-disclosure is what destroys an
   expert, not the connection itself.**
2. **Any fact, limitation or assumption bearing on the opinion.** Candour to the tribunal is the
   overriding duty. **The scope of a search underlying a finding is always in this category.** A
   negative finding — *"no relationship found"*, *"nothing located"* — carries exactly the weight of
   the search behind it, so what was searched, over what period, with what instrument, is material
   to that weight and stays in the report ([`claim-grading.md` §9](claim-grading.md): scope limits
   stay in regardless). Cutting a search boundary does not tighten the prose; it converts a bounded
   check into a broader implication — overstatement by omission.

Minimal words about *how* you work; hide nothing about *what affects the opinion*.

---

## Conflicts

A prior relationship does **not** automatically disqualify an expert. The test is the independence of
the opinion, judged by materiality — see *Toth v Jarman* [2006] EWCA Civ 1028. **Non-disclosure is
the killer**: an undisclosed past association surfacing in cross-examination gets the evidence given
greatly reduced weight or ruled inadmissible — see *EXP v Barker* [2017] EWCA Civ 63. (English
authorities; the principles travel to other common-law jurisdictions and to arbitration.)

Answering a conflict-check request:

- **Anchor on the real test** — no conflict *"actual or apparent, that would affect my independence
  or impartiality"*. Not *"no prior relationship"*, which is the wrong test and a hostage to fortune.
- **Show a real check without itemising it.**
- **Affirm the overriding duty** to the court or tribunal.
- **Make it as-at-date, with a continuing duty** to disclose anything arising later.
- **Keep the artefact.** The defensible thing later is the dated record, not the bare assertion.

---

## Why the packet is generated, not written

The obvious design is to let someone write the courtroom sentence and check it against the evidence.
That check is impossible to do well: natural language overstates in ways no matcher catches, and the
person who wrote the sentence is the last one able to see it.

peira inverts it. The safe statement is **rendered from the graph** in the 金剛經 three-moment form —
*what is called X*, *X is not the thing itself*, *it is named X under these conditions*. Nobody
writes the sentence, so nobody can overstate it. This is one instance of the wider rule that a
deliverable is **compiled from the record**, never authored beside it — stated in
[`claim-grading.md` §8](claim-grading.md) and mapped onto the machinery in
[`../architecture.md` §1](../architecture.md). `peira verify` then re-derives the packet from the
vault and compares digests, so drift that reaches the rendered body announces itself — within two
limits established by audit: fields the packet does not render (grades, graders, instrument links)
change no digest, and a packet whose `Packet format:` line was edited returns no verdict rather
than a mismatch. See [`../architecture.md`](../architecture.md), defects 6 and 8.

**A known limit, found by an adversarial audit and confirmed in the source:** the rendered
statement quotes the author-written term fields — `as_used`, `not_essence`, `stipulated` —
verbatim, and the forbidden-verb lint scans only a node's title and body, never those fields. An
overstatement placed in a term's stipulation therefore reaches the packet unaltered. See
[`../architecture.md`](../architecture.md), defect 5. The generation is a structural defence, not a
complete one: read the rendered statement as your own prose before it leaves your hand, because in
those three fields it is.

---

## Before anything leaves your hand

- Does it answer the exact question and nothing wider?
- In correspondence: any enumerated categories where the broad formulation says more? Prefer the
  broad form. In a finding: the scope of the search stays — a negative finding stripped of its
  search scope overclaims.
- Any volunteered reasoning not asked for? Cut it.
- Any assertion made twice in different words? Reduce to one.
- Any not-obviously-immaterial connection, or opinion-bearing fact or limitation? Disclose in full.
- Inferences hedged, legal conclusions handed to the tribunal?
- Any negative finding resting on a source that never passed a positive control?
- Every identifier written in full, never elided?
- Dated, and filed in the record?
