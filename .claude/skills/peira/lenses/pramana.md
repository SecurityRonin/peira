# प्रमाण pramāṇa — Type the Means of Knowing

**Gate:** `PEIR-GRADE-EXCEEDS-PRAMANA`
**Failure mode:** testimony passed off as observation, and corroboration mistaken for
independence.

Nyāya Sūtra 1.1.3 names four means of valid knowledge, and peira caps the grade an
edge may carry by which one it rests on:

| pramāṇa | | ceiling |
|---|---|---|
| **pratyakṣa** | perception, including a reading off an instrument | `G3` |
| **anumāna** | inference from what was perceived | `G2` |
| **upamāna** | comparison, analogy | `G1` |
| **śabda** | testimony: documentation, a write-up, another tool's report | `G1` |

**No single edge reaches `G4` under any pramāṇa.** G4 requires multiple materially
independent convergent lines, which is a property of the graph, not of one piece of
evidence — so it can only ever be earned, never asserted.

## What to look for

The hard case, and the one the manifesto singles out: **two parsers agreeing is not two
observations.**

- Each parser's output is *śabda* — the tool's report of what it read.
- If both vendor the same decoding library, they are not independent even as testimony.
  They are one implementation reporting twice.
- Independence is a claim about **lineage**, not about vendor names. Two tools with a
  shared dependency are one line of evidence for everything that dependency touches.

Reading the hive yourself in a hex editor is *pratyakṣa*. Reading a tool's summary of it
is *śabda* about the tool, which is in turn testimony about the hive.

## What to write

Declare it on the edge:

```yaml
supports: ["c-bounded grade=G2 by=albert via=perception"]
```

and on the observation node, `pramana: testimony` where that is what it is.

If two supporters restate one another, record it — `duplicates:` — and the
`PEIR-LINT-FALSE-INDEPENDENCE` lint will stop them being counted twice.

## Watch for

Grading an edge `G3` because the *conclusion* feels solid. The grade describes how the
knowing was arrived at, not how convinced you are.
