# Example vault — peira examines its own audit

A vault holding one claim, its term, its instrument, and one observation — the **peira-examined
form of a conclusion that peira's own adversarial audit had over-stated**. Unlike
[`prefetch-vault`](../prefetch-vault) (a constructed teaching scenario), this one is a *real*
result: the claim, the over-statement, and every catch below actually happened.

> **Provenance.** Recorded 2026-08-27 from a live dogfooding session. An adversarial audit of
> peira's MCP surface concluded, in prose, that *"the product mints nothing on any route
> attacked."* peira was then run on that conclusion. What follows is what it found.

## The over-statement it started from

The audit's summary said, of peira's tools:

> the product mints nothing on **any** route attacked

A universal negative — the most dangerous claim shape, because it asserts the search was
*complete*. Run through peira, that sentence does not survive:

| lens | what it demanded |
|---|---|
| `peira_check_prose` | flagged `proves` / `proven` / `confirmed` elsewhere in the report — observations dressed as verdicts |
| **白馬非馬 (BAIMA)** | a `universal` quantifier with no declared **extension** — *declare what the class contains, or narrow to the case examined* |
| **Toulmin** | no `warrant` |
| **Pearl / Popper** | no `boundaries`, no `falsifier` — *as written, no observation could count against it* |
| **不可得因 (ANUPALABDHI)** | an **absence** claim must rest on an instrument with a recorded **positive control** — proof the search could have *found* a mint if one were present |
| declaration-contradiction lint | a first correction relabelled `quantifier: singular` while the sentence still said *"every"* — the class-extension gate switched off by the declaration, not the claim. peira caught the dodge. |

The last one is the point: peira contradicted the person operating it, mid-correction, while he
was trying to be honest — the one thing re-reading your own work cannot do.

## What the vault holds — the honest form

| File | Node | Role |
|---|---|---|
| `60-lexicon/mint.md` | `mint-term` | the load-bearing term: peira *minting* a value (computing it) vs *echoing* the caller's own input back |
| `70-inquiry/i-guard.md` | `i-guard` (instrument) | the anti-fabrication guard, with its **positive control** — it goes red on a planted `grade=`/`by=`/`via=` or a score field |
| `70-inquiry/o-agent2.md` | `o-agent2` (observation) | a bounded *measurement*: the guard flagged no peira-generated evidence field across the routes driven, `measured_by` the instrument |
| `70-inquiry/c-mints-bounded.md` | `c-mints-bounded` (claim) | the conclusion, bounded to the routes examined, with warrant, boundaries (incl. *elusion unmeasured*), and a falsifier |

The claim it freezes into is **materially weaker, and materially more true**, than the summary:
not *"peira mints nothing"* but *"every evidence token found in the **exercised** routes traced to
echoed input — search elusion unmeasured beyond the instrument's positive control."*

## Run it

```console
$ peira gates examples/self-examination-vault
✓ gates: nothing to report.

$ peira status examples/self-examination-vault c-mints-bounded
  grounded extension : IN — every attack on it is itself defeated
  gates              : all enforced gates pass
  derived state      : review_ready — gates pass; a reviewer must still sign

$ peira packet examples/self-examination-vault c-mints-bounded      # the honest form, frozen
# Citation packet — c-mints-bounded
…
```

## The lesson

Every mechanical catch above maps to a principle from the research-method discipline —
universal-negative-needs-bounding, absence-needs-a-positive-control, a-cap-evaded-by-mis-setting-its-field
— rediscovered by peira's lenses with no model in the loop. The tool bit on its own maker's
reasoning about itself, which is the strongest test it could be put to.
