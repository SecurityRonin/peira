# 正名 / 所謂 X 即非 X 是名 X — Rectify the Name

**Gate:** `PEIR-TERM-UNSTIPULATED`
**Failure mode:** reifying a label into a thing, so a word does argumentative work its
definition never licensed.

The 金剛經 formula is not mysticism; it is the most compact definitional discipline
anyone has written. 所謂佛法者，即非佛法，是名佛法 — three moments, in order:

1. **所謂 (as_used)** — what the word is taken to mean, conventionally, in the room.
2. **即非 (not_essence)** — what the word is *not*. The label has no inherent nature;
   the record is not the thing.
3. **是名 (stipulated)** — so it is *named* thus, under stated conditions, for this work.

The middle moment is the one people skip, and it is the one that catches over-claims.

## What fires this

A claim has `uses_term:` edges, and some Term is missing one of the three moments — or
points at a node that does not exist.

If a claim declares **no** key terms at all, the gate returns `Unassessed`, not `Pass`.
That is not a technicality: it means nobody has said which words are load-bearing, so
nothing has been checked.

## What to write

```markdown
---
id: "60.02"
type: term
title: execution
as_used: a user or service ran the program
not_essence: >-
  Amcache does not observe process creation; nothing in the record distinguishes a
  launch from an install, an inventory pass or a scan
stipulated: >-
  a process was created from this image, established only by evidence that observes
  process creation
---
```

Then `uses_term: ["60.02"]` on every claim that leans on the word.

## Watch for

- **A 即非 that only negates the opposite.** "Execution is not non-execution" is empty.
  The negation must say what the *evidence* fails to reach.
- **A 是名 wider than the 即非 allows.** If the second moment concedes the record cannot
  distinguish a launch from a scan, the third cannot then stipulate "the program ran".
  That inconsistency is exactly the over-claim, now visible in one file.
