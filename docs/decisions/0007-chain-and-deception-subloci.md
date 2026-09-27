# 7. Two sub-loci for the doors: the chain, and the rival that was made

Date: 2026-09-27

## Status

Proposed.

No door is added. Two prompt lines gain a sub-locus, and one wording is sharpened. This flips to
Accepted when the two candidate lenses below have source-checked tradition sections, or are dropped
in favour of the prompt lines alone.

## Context

The pass was run, door by door, on about fifteen secondary sources in one week (press, practitioner
posts, law-firm alerts, a vendor's post about a competitor) while preparing a CPD talk, and then
stress-tested against intelligence tradecraft: ODNI **ICD 203** (Analytic Standards; its nine
tradecraft standards) and Heuer & Pherson's structured analytic techniques.

The crosswalk holds up. Every door has a tradecraft counterpart, and two doors ask more than ICD 203:

| Door | Tradecraft counterpart | Note |
|---|---|---|
| 1 Direction | objectivity; cognitive-bias checks | ICD 203 asks for objectivity; Direction adds the cost comparison |
| 2 Words | ICD 203 standard 2 (uncertainty expressed properly); estimative language | Words also catches a term that changes meaning mid-source |
| 3 Ground | ICD 203 standards 1 (source quality) and 3 (intelligence vs assumption); source-reliability vs information-credibility grading | source interest is already capped by the enforced source-class ceiling |
| 4 Rivals | ICD 203 standard 4 (analysis of alternatives); `ACH` | — |
| 5 Breaks | key-assumptions check; indicators; ICD 203 standard 7 (change or consistency) | staleness is already the "after a date" locus |
| 6 Silence | negative evidence | **no ICD 203 standard** — the doors ask more |
| 7 Transfer | ICD 203 standard 5 (relevance and implications) | **a comprehension test**, stronger than "so what" |

Two failures recurred that the pass did not name *at the moment of reading*, and each was caught only
by retrieving the primary:

1. **Retelling drift.** A claim sound at its source changed on the way to the reader, one retelling at a
   time. Lived, all in the same week: a court's passive "was found when the filing was processed" became
   "flagged and blocked" in one secondary and was repeated in a law-firm client alert; a Chinese-language
   thread turned "a blogger's article exhibited to a declaration" into "the lawyer's AI invented two
   cases"; a practitioner post quoted a judicial guidance document in words the document does not use,
   and attributed them to a named judge. **Ground** asks how the source knows; nothing asks how the claim
   reached me. `PEIR-LINT-FALSE-INDEPENDENCE` catches two retellings counted as two sources; it does not
   catch one retelling that changed the words.

2. **The made-to-look-like-this rival.** Hidden text addressed to a machine in an opponent's filing; a
   deepfake exhibit; a vendor's post about a competitor. **Rivals** asks what else would look exactly like
   this, and readers enumerate *innocent* alternatives. Tradecraft treats deliberate deception as a
   standing hypothesis — Heuer & Pherson's Deception Detection checklists: MOM (motive, opportunity,
   means), POP (past opposition practices), MOSES (manipulability of sources), EVE (evaluation of
   evidence) [secondary: read via summaries of the book, not the book].

## Decision (proposed)

Honour "Not more" (`method/anti-summarization.md`, "Why seven"): both failures are sub-loci of existing
motions, so they go on prompt lines.

| Door | Current line | Proposed line |
|---|---|---|
| 3 Ground | How does it know, and can that way of knowing reach this far? | How does it know, can that way of knowing reach this far — **and by what chain did it reach me? Name each retelling between the source and this text, and check the words survived it.** |
| 4 Rivals | What else would look exactly like this, and who got left out of the ring? | What else would look exactly like this — **including that someone made it look like this** — and who got left out of the ring? |
| 5 Breaks | …on its own pages, against the world, after a date, past a case? | …on its own pages, against the world, **as at when (and is it still?)**, past a case? *(wording only)* |

Compact-form lines change to match. Noun-demand for the new sub-loci: a **named retelling** (who, where,
what changed) and a **named maker** (who would gain, with what means).

**Candidate lenses (catalogued, owning no gate — to be source-checked before they enter the catalogue):**

- `CHAIN-OF-TRANSMISSION` — **إسناد** (the chain of transmission) in the science of the prophetic traditions
  (**علم الحديث**), where each transmitter in a chain is weighed and a chain is only as sound as its weakest
  link. Routes from Ground's new sub-locus. *Tradition section not yet written; the Arabic terms and the
  characterisation are from memory and must be verified against academic sources before entry — never
  generate script from memory (the catalogue's source-language rule).*
- `DECEPTION-DETECTION` — Heuer & Pherson's MOM / POP / MOSES / EVE checklists. Routes from Rivals' new
  sub-locus. *Primary (the book) not yet read.*

## Considered and not proposed

- **Independence as its own question** — already routed under Ground (`PEIR-LINT-FALSE-INDEPENDENCE`).
- **The source's own interest** — enforced by the source-class ceiling; Direction stays about the reader,
  where its one comparison lives.
- **Confidence, and what would change my mind** — Breaks past a case (the falsifier) and claim-grading
  already carry it; a door for it would revive the original fifth question, answerable with the source
  closed.

## Consequences

- `method/anti-summarization.md` and `doors.md`: the three table rows and the compact form.
- The inline copy in the `research-method` skill (§0A) must be re-synced from peira, per its own note.
- A lens addition is code (`lens/src/lib.rs` CATALOG) plus a generated page; each needs a source-checked
  tradition section and a worked example, per the catalogue's standard.

## Open questions

1. Is the deception hypothesis better placed under Ground (MOSES — how manipulable is the source?) than
   under Rivals? It has one foot in each; this ADR puts it under Rivals because the reader's failure
   observed was an unlisted alternative, not a mis-weighed source.
2. Does `CHAIN-OF-TRANSMISSION` add enough over Ground's new line to earn a catalogue entry, or does the line suffice?
