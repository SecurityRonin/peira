# 白馬非馬 — The White Horse

**Gate:** `ELEN-CLASS-EXTENSION-UNDECLARED`
**Failure mode:** sliding between a type and its tokens, or between intension and
extension.

公孫龍's argument is usually taught as sophistry and is usually mis-taught. The point is
that 馬 (horse) and 白馬 (white horse) have different extensions, so a statement true of
one need not be true of the other — and natural language hides the slide.

## What fires this

A claim says `quantifier: universal` or `quantifier: class` and declares no
`extension:`. A claim with no `quantifier:` at all returns `Unassessed` — nobody said
whether one case or all cases was meant.

## What to look for

Count what was examined, then count what is being asserted.

- Examined: one `InventoryApplicationFile` record, on one host.
- Asserted: "Amcache entries indicate execution" — every entry, every host, every build.

That gap is the whole finding. It is rarely deliberate; the plural just slips in.

## What to write

One of three, in descending order of preference:

1. **Narrow the claim** to what was examined — `quantifier: singular`. Usually correct.
2. **Declare the extension** and accept the burden:
   ```yaml
   quantifier: class
   extension:
     - InventoryApplicationFile records on Windows 10 1809 through 22H2
     - excludes InventoryApplication and InventoryDriverBinary
   ```
3. **Split into two claims** — a singular one that is supported, and a class-level
   hypothesis that is not yet.

## Watch for

`is_a` and `has_a` doing each other's work. An Amcache record *has a* SHA-1 field; it
*is not a* file hash. Composition read as inheritance is the same error in a different
costume.
