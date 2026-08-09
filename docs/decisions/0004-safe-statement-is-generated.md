# 4. The Court Mode safe statement is generated, never authored

Date: 2026-08-09

## Status

Accepted

## Context

Court Mode must emit a courtroom-safe formulation of a claim. The obvious design lets a
reviewer write that sentence and then validates it against the graph.

That validation cannot be done well. Natural language overstates in ways no matcher
catches — mood, implicature, a single load-bearing adverb — and the person who wrote the
sentence is the last one able to see it.

## Decision

Render the safe statement **from the graph**, in the 金剛經 three-moment form, from the
claim's Term nodes:

- 所謂 X — what the term is conventionally taken to mean (`as_used`)
- 即非 X — what the record is not; the label has no inherent nature (`not_essence`)
- 是名 X — so it is named X, under stated conditions only (`stipulated`)

Nobody writes the sentence, so nobody can overstate it. `freeze()` additionally refuses
while any gate blocks and refuses a claim defeated in the grounded extension, with no
override parameter.

## Consequences

- The packet is the conventional register (世俗諦); the graph with all its boundaries is
  the ultimate one (勝義諦). The translation can only lose strength, never gain it.
- Packet quality becomes a function of Term quality, which is where the 正名 gate already
  applies pressure.
- A packet that could be forced would be worth nothing in the room it is made for, so the
  absence of an override is deliberate and should not be added later.
