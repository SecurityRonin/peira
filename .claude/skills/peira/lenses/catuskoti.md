# 四句 catuṣkoṭi — The Four Corners

**Gate:** `PEIR-CORNERS-UNADDRESSED`
**Failure mode:** collapsing a contested question into a binary before the other
positions have been stated.

Nāgārjuna's tetralemma insists on four positions, not two: **A**, **not-A**, **both**,
**neither**. Most real disputes die in the third and fourth corners, which is precisely
why they get skipped.

## What fires this

A claim is contested — something attacks it, or it says `contested: true` — and it lists
fewer than four `corners:`.

## What to look for

Take the Amcache case:

| Corner | The position |
|---|---|
| **A** | the program executed |
| **¬A** | the program did not execute |
| **both** | it executed on one occasion and was merely catalogued on another; the record cannot separate them |
| **neither** | *catalogued without executing* — an install, an inventory pass, or a scan wrote the record |

The fourth corner is the correct answer, and a binary framing cannot even express it.
That is the entire value of the lens.

## What to write

```yaml
contested: true
corners:
  - "executed: a process was created from this image"
  - "not executed: the file was present and never ran"
  - "both: executed on one boot, inventoried on another; the record cannot separate them"
  - "neither: catalogued by an install or inventory pass, with no execution at all"
```

Ruling a corner out **counts as addressing it**, provided a reason is given. Silence
does not.

## Watch for

Padding to four with restatements. "Not executed" and "did not run" are one corner
written twice, and the count will be satisfied while the thinking is not.

Note that an unquoted `- neither: catalogued without running` is a YAML *mapping*, not a
string. The parser renders it rather than dropping it — but quote it, and the file says
what you meant.
