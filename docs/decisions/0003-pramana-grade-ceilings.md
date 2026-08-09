# 3. Evidence grade is capped by its means of knowing

Date: 2026-08-09

## Status

Accepted

## Context

The Vibe manifesto states that "independent tools are not automatically independent
evidence". As prose this is agreed with and then forgotten. Two parsers agreeing on a
hive feels like corroboration, and if they vendor the same decoding library they are one
implementation reporting twice.

## Decision

Type every evidence edge by the Nyāya means of valid knowledge, and cap the grade it may
carry:

| pramāṇa | ceiling |
|---|---|
| pratyakṣa — perception, including an instrument reading | `G3` |
| anumāna — inference | `G2` |
| upamāna — comparison | `G1` |
| śabda — testimony, including another tool's report | `G1` |

**No single edge reaches `G4` under any pramāṇa**, because G4 requires multiple
materially independent convergent lines — a property of the graph, not of one piece of
evidence.

Separately, `ELEN-LINT-FALSE-INDEPENDENCE` reports two supporters of one claim that are
linked by a `duplicates` edge.

## Consequences

- A tool's output is testimony about the artifact, and cannot buy a grade that direct
  perception would earn.
- G4 can only be earned by the shape of the graph, never asserted on an edge.
- An edge with no declared pramāṇa is *unassessed*, which the gates report separately and
  which never counts as a pass.
