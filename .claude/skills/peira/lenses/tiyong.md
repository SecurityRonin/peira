# 體用 — Substance and Function

**Gate:** `PEIR-FUNCTION-AS-SUBSTANCE`
**Failure mode:** reporting what a thing DID as though it established what a thing IS.

程頤: 體用一源，顯微無間 — substance and function share a source. Sharing a source is not
being identical. Function is evidence *about* substance; it is not substance.

## What fires this

A claim declares `aspect: substance` and every node supporting it declares
`aspect: function`.

## What to look for

The tell is a copula. "Amcache **is** an execution artifact" is a claim about the nature
of a data structure. What was actually observed is "Amcache **recorded** this path" — a
thing Windows did on one occasion.

Ask: if the artifact behaved differently tomorrow after a patch, would the claim be
false? If yes, it was a function claim wearing substance clothing.

## What to write

Almost always: **restate it as the function claim it actually is.**

- was: `Amcache is an execution artifact` (`aspect: substance`)
- now: `Amcache recorded this path and file identity under build X` (`aspect: function`)

If a substance claim is genuinely wanted, it needs evidence bearing on nature — vendor
documentation of the population mechanism, kernel source, a specification — carrying
`aspect: substance` itself.

## Watch for

Relabelling the supporting observation `aspect: substance` to clear the gate. That
converts a defect into a lie, and it is the single most tempting wrong move here.
