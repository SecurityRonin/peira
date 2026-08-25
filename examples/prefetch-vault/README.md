# Example vault — Prefetch execution evidence

A small, complete peira vault that **freezes**: one bounded claim, the term it rests on, and
one supporting observation. Unlike the fixtures under `tests/vaults/`, this is here to be
*read and run* — a worked example of the authoring workflow, not a test oracle.

> **Provenance.** Constructed scenario, authored 2026-08-25 for documentation. It is
> *synthetic*: there is no real Prefetch file behind `o-prefetch`, and the `sealed://`
> pointer resolves to nothing. The forensic proposition is sound, but the artefact is
> illustrative — do not cite it as evidence of anything.

## What it holds

| File | Node | Role |
|---|---|---|
| `60-lexicon/exec-01.md` | `execution` (term) | the load-bearing term, with its three moments (`as_used` / `not_essence` / `stipulated`) |
| `70-inquiry/c-prefetch.md` | `c-prefetch` (claim) | a **bounded** claim: a non-zero Prefetch run count evidences a program *start*, at the association rung, with the rival counter-increment mechanisms left standing in the falsifier |
| `70-inquiry/o-prefetch.md` | `o-prefetch` (observation) | a supporting observation (run count 3), graded `G2 via=perception` |

The claim carries every field the enforced gates demand — `warrant`, `aspect`, `quantifier`,
`causal_rung`, `uses_term`, `boundaries`, `falsifier` — which is exactly why it freezes.

## Run it

```console
$ peira gates examples/prefetch-vault
✓ gates: nothing to report.

$ peira status examples/prefetch-vault c-prefetch
c-prefetch  (claim)
  A Prefetch file whose run count is at least one evidences that Windows started the named program at least once, within the bounds stated below

  grounded extension : IN — every attack on it is itself defeated
  gates              : all enforced gates pass
  derived state      : review_ready — gates pass; a reviewer must still sign

  (derived, not stored — there is no field to write it to)

$ peira packet examples/prefetch-vault c-prefetch      # freeze the citation packet
# Citation packet — c-prefetch
…
```

The same nodes over the MCP surface: `peira_gates`, `peira_examine`, `peira_status`,
`peira_freeze`, then `peira_verify` on the frozen packet.

## The workflow it demonstrates

1. **`peira_propose`** on a draft — *“The Prefetch file proves the user executed Notepad”* —
   infers `causal_rung: counterfactual` from the verb “proves”, flags it, and lists the blank
   fields with the gate each will demand.
2. **Author** the node, filling those blanks and choosing language that does not over-claim
   (“evidences … a start”, not “proves execution”).
3. **`peira_gates` / `peira_examine`** until nothing blocks.
4. **`peira_freeze`** renders the Court-Mode packet; **`peira_verify`** re-derives it against
   the vault and confirms it byte-for-byte.

Filling the blanks `propose` hands back *is* what turns the draft into a claim that passes —
the required fields are the enforced gates in reverse.
