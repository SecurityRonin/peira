# 立極 — Set the Pole

**Gate:** `PEIR-CRITERION-UNDECLARED`
**Failure mode:** judging something without ever declaring the standard judged against.

周敦頤's 太極圖說 has 聖人定之以中正仁義而主靜，立人極焉 — the sage *establishes the pole*
before anything can be measured against it. An evaluation with no declared pole is not a
weak evaluation; it is not an evaluation at all, because there is nothing for the reader
to disagree with.

## What fires this

A claim reads as a judgement rather than a description — either it says
`evaluative: true`, or its text carries a word from the evaluative table
(suspicious, malicious, anomalous, significant, benign, severe, excessive …) — and it
has no `judged_by:` edge to a Criterion.

## What to look for

Ask the question the gate is really asking: **suspicious compared to what?**

- By frequency? Then the criterion is a base rate, and the base rate needs a source.
- By policy? Then the criterion is the policy, cited by name and version.
- By the author's experience? That is legitimate — say so, and say whose experience,
  so the reader can weigh it.

If none of the three can be answered, the word is doing rhetorical work rather than
analytic work, and the honest fix is to delete it.

## What to write

Either a Criterion node in `60-lexicon/`:

```markdown
---
id: "60.10"
type: criterion
title: Staging-path standard
basis: >-
  Directories that malware commonly writes to before execution — %TEMP%,
  Downloads, $Recycle.Bin — per the cited catalogue.
sources:
  - https://attack.mitre.org/techniques/T1204/
---
```

and a `judged_by: ["60.10"]` on the claim — **or** rewrite the claim to describe rather
than judge, which is usually the better answer.

## Watch for

Naming a criterion that merely restates the judgement ("the criterion is that the path
is suspicious") clears the gate and fixes nothing. The criterion must be checkable by
someone who disagrees with you.
