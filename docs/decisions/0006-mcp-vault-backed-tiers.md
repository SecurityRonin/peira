# 6. The MCP vault-backed tiers (2–4)

Date: 2026-08-25

## Status

Proposed.

The structure and the five constraints below are firm — they are the reasoning the issue
and every audit round already settled. Two questions are left open deliberately, named at
the end; this flips to Accepted when they are answered and the first tier lands.

## Context

The MCP server ships **Tier 1 only** — `peira_check_prose` and `peira_lens`, neither of
which opens a file. Those two are validated: tested against real prose, they bite on
genuine overstatement, scope the verification carve-out per clause, and stay silent on
careful writing. So `peira-mcp` is held at `release = false` for **scope, not
correctness** — the remaining barrier is that the server cannot yet see a vault.

Tiers 2–4 were designed and never built, and until now the design existed nowhere but a
conversation (issue #31). This ADR is that design, grounded against the library API as it
actually is, so the source of truth precedes the code rather than trailing it.

Every tool named below wraps a function that exists today:

| Operation | Function |
|---|---|
| Load a vault | `peira_core::load` — an empty or absent vault is a hard error, never empty-clean |
| Per-node findings | `peira_citation::violations_for` |
| Whole-vault findings | `peira_citation::all_findings` |
| Standing | `Graph::is_grounded`, `Graph::grounded_extension` |
| Freeze a packet | `peira_citation::freeze` → `Result<Packet, PacketError>` |
| Verify a packet | `peira_citation::verify` → `Verification` |
| Refusal payload | `PacketError::Blocked { id, violations }` |
| Verdict states | `GateResult` — `Pass`, `Block`, `NotApplicable`, `Unassessed { why }` |

## Decision

### The five constraints, and why each is not negotiable

1. **Read-only, always.** Tier 1 has no path argument, so there is nothing to write.
   Tiers 2–4 take a vault path and must still never write it — a packet is *returned*,
   never saved. peira's value is that it refuses; a server that can write is a server
   that can be talked into writing.

2. **`Unassessed` must survive the JSON boundary.** This is the defect shape that
   recurred through every audit round — a verdict computed and discarded in transit.
   `GateResult` has four states; `examine_graph` already converts `Unassessed` into a
   `Violation` carrying `PEIR-GATE-UNASSESSED` precisely so "no verdict" cannot be
   filtered out. The MCP layer inherits that for free **as long as it returns findings
   verbatim and never adds a top-level boolean.** Collapse to `{passed: bool}` and the
   defect reappears at the protocol layer, where no existing test would see it: an LLM
   reads `{ok: false}` as "try again", and `PEIR-GATE-UNASSESSED` means something else
   entirely.

3. **No scores.** A response schema is exactly where a helpful-looking `confidence: 0.82`
   appears. Tier 1 has a test that serialises the report and fails on
   `confidence`/`score`/`severity`/`probability`/`weight`; every schema here lifts that
   test verbatim.

4. **A refusal is a result, not an error.** `freeze` returning `Err(Blocked)` carries the
   violations — the reason it cannot freeze — and that reason is the product. Flattening
   it to "failed" throws away the one thing the caller needed.

5. **`propose` may not author claims.** peira's founding rule is that the safe statement
   is rendered from the graph, never written. So `propose` suggests STRUCTURE for prose a
   human already wrote — never new assertions — and every grade, `by` and `via` comes
   back blank for a human to fill.

### The shared response envelope

Every vault tool returns the same shape as Tier 1's `ProseReport`: a findings list plus a
scope note carried on **every** response, clean ones included, so silence cannot read as
approval. Per-node tools add a `standing` enum derived from the grounded extension. No
boolean, no number.

```jsonc
// peira_examine(vault, node)
{
  "node": "c-bounded",
  "standing": "grounded",          // grounded | contested | unsupported — DERIVED, never set
  "findings": [
    { "code": "PEIR-GATE-UNASSESSED", "subject": "c-bounded",
      "detail": "RUNG reached no verdict: no Windows build named", "remedy": "…" }
  ],
  "scope": "gates + lints + grounding for this node. Empty findings means clean; a
            PEIR-GATE-UNASSESSED finding means a gate could NOT reach a verdict — not a pass."
}
```

### Tier 2 — vault-aware (thin wrappers)

| Tool | Signature | Wraps |
|---|---|---|
| `peira_examine` | `(vault, node)` | `violations_for` + `is_grounded` |
| `peira_status` | `(vault, node)` | grounded-extension membership |
| `peira_gates` | `(vault)` | `all_findings` |

### Tier 4 — freeze / verify (refusal is a result)

Both outcomes are OK responses, modelled as a tagged union. The packet is returned, never
written to the vault.

```jsonc
// peira_freeze(vault, node)
{ "outcome": "frozen",  "packet": { /* the Court-Mode packet */ } }
{ "outcome": "blocked", "violations": [ /* why it cannot freeze */ ], "scope": "…" }
```

`peira_verify(vault, packet)` mirrors the `Verification` variants (matches / drifted /
`NoLongerFreezable`) as a tagged union.

### Tier 3 — `peira_propose(prose)` (the only net-new logic)

The adoption barrier is that nobody hand-writes
`supports: ["c1 grade=G2 by=albert via=perception"]`. `propose` reads prose the author
already wrote and emits the skeleton of a claim node — because **a claim's required fields
are the enforced gates in reverse** (`warrant`→TOULMIN, `aspect`→TIYONG,
`quantifier`→BAIMA, `causal_rung`→RUNG, `uses_term`→ZHENGMING, `boundaries`/`falsifier`→
RUNG/PREMORTEM). Filling the blanks it hands back is what turns the draft into a claim
that passes, and each blank is labelled with the gate that will demand it.

**Decision, 2026-08-25 — the extraction infers CLASSIFICATION, never EVIDENCE.** An earlier
draft of this section said `propose` "authors nothing" and inferred nothing. That is
refined here: it infers the three CLASSIFICATION fields — `quantifier`, `aspect`,
`causal_rung` — from the author's *own words*, transparently and marked *confirm*. It does
NOT touch the evidentiary fields (`grade`/`by`/`via`, `warrant` content, `boundaries`,
`falsifier`, term definitions), which stay blank in every case. The line the ADR draws is
between classifying the prose the author wrote (peira's core competency) and minting
evidence that does not exist (forbidden) — not between inferring and not inferring.

The inference **reuses the lint's verb knowledge, never a parallel table.** `causal_rung`
is inferred by running `prose_findings_in` on the proposition: a forbidden verb ("proves")
means the sentence reads at the counterfactual rung, so `propose` infers it there — which
makes the RUNG gate FIRE on the draft, surfacing the over-claim instead of hiding it behind
a blank. propose and the gates share one verb list by construction and cannot disagree.

```jsonc
{
  "proposed_type": "claim",
  "proposition": "<the author's sentence, verbatim>",
  "inferred": [                            // classification, from the author's own words
    { "field": "causal_rung", "value": "counterfactual",
      "inferred_from": "the verb \"proves\"", "confirm": true }
  ],
  "candidate_terms": ["execution"],        // crude (quoted) extraction — the caller refines
  "needs": [                               // every blank, with the gate that will demand it
    { "field": "warrant",  "gate": "PEIR-WARRANT-MISSING" },
    { "field": "boundaries", "gate": "PEIR-BOUNDARIES-MISSING" }
  ],
  "prose_findings": [ /* forbidden-verb / legal-conclusion findings on the proposition */ ],
  "scope": "STRUCTURE and CLASSIFICATION only; no grade/by/via/evidence is authored."
}
```

`propose` has **no grade/by/via field anywhere** — the evidentiary blanks are blank by
construction, not by being set to null. This is the one place an LLM could smuggle in a
fabricated grade, so it carries a mutation-proven adversarial test: any `grade`/`by`/`via`
key or `grade=`/`via=` value appearing in the output fails the build. The determinism line
holds too — peira does only crude, defensible extraction (verbatim title, determiner-scan
quantifier, quoted candidate terms); the LLM caller, which is good at language, refines the
rest.

### Build order

**Tier 2 → Tier 4 → Tier 3.** The first two are thin wrappers over verified functions and
could themselves justify lifting the release hold. Tier 3 carries the design risk and
lands last, on its own.

## Consequences

- The server becomes vault-aware without becoming able to write one. Read-only is a
  property of the surface, not a promise in the docs.
- The `Unassessed`-survives-transit and no-scores tests are the load-bearing checks; they
  are written before the schemas they guard, not after.
- Two decisions are left open and are the owner's to make:
  1. **Does Tier 3 gate the release, or do Tier 2 + Tier 4 alone?** 2 + 4 are low-risk
     wrappers and a defensible `v0.1.0` that is no longer "just two vault-free checks".
     Tier 3 is the adoption feature and the fabrication-risk surface.
  2. **Vault-path safety.** The tools read a vault path the caller names. Read-only covers
     writes; whether to refuse reads outside a configured root, or accept that the caller
     reads their own filesystem through their own client, is unresolved. The leaning is to
     accept — it is the caller's data — but it is a deliberate call, not a default.
