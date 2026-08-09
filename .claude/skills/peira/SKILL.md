---
name: peira
description: Examine a claim in a peira vault through the classical critical-thinking lenses — 立極, 正名/所謂即非是名, 體用, 白馬非馬, 四句, Toulmin, pramāṇa, and the causal ladder — and write examination artifacts that propose nodes and edges. Use when a claim needs cross-examining before it is relied on, when `peira gates` blocks and the fix is not obvious, or when turning folklore into a scoped proposition. Triggers - "examine this claim", "cross-examine", "is this over-claimed", "run the lenses", "why is this blocked", "make this court-safe".
---

# peira — examine a claim

## What you may and may not do

You are the **doer**. The CLI is the **checker**. That division is not advice; it is
built into the data model, and you should not try to work around it.

**You may:**
- create and edit `question`, `hypothesis`, `claim`, `observation`, `term`,
  `criterion`, `protocol`, `run`, `examination` and `dissent` nodes
- add edges between them
- propose an evidence grade with `proposed=G2` on an edge
- write an `examination` node recording what you found

**You may not:**
- write `status:` or `confidence:` on any node. There is no such field. The parser
  refuses the document and names the key. Do not attempt it.
- write `grade=` without `by=`. An unattributed grade silently degrades to a
  proposal, and the lint pack reports it.
- declare that a claim is accepted, verified, or cleared. Only
  `peira status` says that, and it derives the answer.

If you catch yourself wanting to assert a verdict, that is the feeling the design
exists to produce. Write the evidence instead.

## Workflow

1. **Read the state first.** Never begin by proposing fixes.

   ```bash
   peira gates <vault> --node <id>
   peira status <vault> <id>
   ```

   Every blocking finding names its gate, what was actually found, and a remedy.
   Start from what the engine already told you.

2. **Pick the lenses that the blocks point to.** Each gate belongs to a lens, and
   each lens has a playbook in `lenses/`:

   | Gate | Lens | Playbook |
   |---|---|---|
   | `PEIR-CRITERION-UNDECLARED` | 立極 | [liji.md](lenses/liji.md) |
   | `PEIR-TERM-UNSTIPULATED` | 正名 | [zhengming.md](lenses/zhengming.md) |
   | `PEIR-FUNCTION-AS-SUBSTANCE` | 體用 | [tiyong.md](lenses/tiyong.md) |
   | `PEIR-CLASS-EXTENSION-UNDECLARED` | 白馬非馬 | [baima.md](lenses/baima.md) |
   | `PEIR-CORNERS-UNADDRESSED` | 四句 | [catuskoti.md](lenses/catuskoti.md) |
   | `PEIR-WARRANT-MISSING` | Toulmin | [toulmin.md](lenses/toulmin.md) |
   | `PEIR-GRADE-EXCEEDS-PRAMANA` | pramāṇa | [pramana.md](lenses/pramana.md) |
   | `PEIR-CAUSAL-RUNG-UNREACHED`, `PEIR-BOUNDARIES-MISSING` | Causal ladder | [rung.md](lenses/rung.md) |

   `peira lens <ID>` prints the failure mode, the operation and a worked example
   for any of them.

3. **Do the examination.** Follow the playbook. It will tell you what to look for
   and what to write.

4. **Write an examination node** into `80-examinations/`:

   ```markdown
   ---
   id: 20260809T160000
   type: examination
   title: 體用 examination of c-overclaim
   lens: TIYONG
   examines: [c-overclaim]
   examined_by: claude
   ---

   What the lens looked for, what was found, and what is proposed — with the
   proposed nodes and edges written out so a human can accept or reject them.
   ```

5. **Re-run the checker.** Do not report a fix as done on your own authority:

   ```bash
   peira gates <vault> --node <id>
   ```

## The move that resolves most blocks

Most over-claims are not fixed by adding evidence. They are fixed by **restating
the claim within what the evidence already carries**.

The acceptance corpus shows this exactly. The same observations, the same competing
hypothesis — but against *"this entry proves execution"* the alternative is a
`contradicts` edge, and against *"the record establishes catalogued presence, and
may contribute to an execution inference alongside independent evidence"* the very
same hypothesis becomes a `limits` edge. Nothing about the evidence changed. The
claim stopped reaching past it.

Reach for that before you reach for more evidence.

## Honest failure

If a claim cannot be rescued, say so and write a `dissent` node preserving it and
its best argument. Rejection never deletes — that is the machloket rule, and the
reasoning that rejected something is worth as much later as the conclusion.
