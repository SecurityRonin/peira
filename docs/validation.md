# Validation

**What is claimed:** the enforced gates discriminate between an over-stated claim and a
bounded one, and the discrimination is produced deterministically, with no model in the
loop.

**What is not claimed:** that the Amcache forensic conclusions in the corpus are true.
The corpus exercises the *engine*. Its ground truth is taken from a third-party README
and from the Vibe Research design document; elenchus did not establish it.

## Evidence tier

**Tier 2.** We authored the corpus, and the ground truth it encodes is derived from
documented construction (the amcache-forensic README's own statement that *"Amcache is
evidence of presence, not proof of execution"*, and the bounded conclusion set out in the
Vibe Research core document). Real engine output, checkable ground truth — but we chose
the scenario, so it is not Tier 1.

The gate predicates themselves are our own detection heuristics. Tier 3 is legitimate
there: a heuristic's oracle is the judgement it encodes, and no independent tool computes
立極 or 四句 compliance to compare against.

## The three-way control

A green result over a clean vault cannot be distinguished from a checker that passes
everything. **B is the control that matters**, and B′ is what proves any individual gate
is load-bearing.

| Control | Setup | Required | Observed |
|---|---|---|---|
| **A** | over-claim vault | BLOCKS, non-zero exit, ≥5 lenses | 7 gates + 1 lint, exit 1 |
| **B** | bounded vault | PASSES, exit 0 | exit 0, packet froze |
| **B′** | rung gate neutered | that finding disappears | 1 → 0 → 1 across mutate/build/restore |
| **C** | vault absent | distinguishable from A, ~0 s | exit 2, 2 ms |

### Control A — the over-claim blocks

Claim: `This Amcache entry proves execution of the suspicious binary`

```
ELEN-CRITERION-UNDECLARED       [LIJI]       judged, with no standard declared
ELEN-FUNCTION-AS-SUBSTANCE      [TIYONG]     what it did, stated as what it is
ELEN-CLASS-EXTENSION-UNDECLARED [BAIMA]      one token, quantified universally
ELEN-CORNERS-UNADDRESSED        [CATUSKOTI]  contested, addresses 0 of 4 corners
ELEN-WARRANT-MISSING            [TOULMIN]    states no warrant
ELEN-CAUSAL-RUNG-UNREACHED      [RUNG]       counterfactual rung, observation only
ELEN-BOUNDARIES-MISSING         [RUNG]       no boundary conditions
ELEN-LINT-FORBIDDEN-VERB        [LINT]       says "proves"
exit=1
```

### Control B — the bounded conclusion passes

```
✓ gates: nothing to report.     exit=0
✓ lint: nothing to report.      exit=0
grounded extension : IN
derived state      : review_ready — gates pass; a reviewer must still sign
```

### Control B′ — the gate is load-bearing

`causal_rung_earned` was replaced with an unconditional `Pass`, the workspace rebuilt,
and the over-claim re-examined. The mutation was asserted present in source before the
run and asserted absent after restore, because a mutation that silently fails to apply
reports green while testing nothing.

```
baseline                 ELEN-CAUSAL-RUNG-UNREACHED findings: 1
gate neutered            ELEN-CAUSAL-RUNG-UNREACHED findings: 0   (6 findings total)
restored                 ELEN-CAUSAL-RUNG-UNREACHED findings: 1
```

### Control C — an absent vault is not an empty pass

```
elenchus: vault root `/nonexistent/vault` is not a directory
exit=2   elapsed=2ms
```

Exit 2 is distinct from control A's exit 1, so *"found nothing"* can never be read as
*"could not look"*. The near-zero duration is the fingerprint of work not done, which is
what makes C meaningful rather than decorative.

## Tamper detection

```
freeze                   sha256 df224b0a79a533331ecc76fc66c3b2274d27eb26736df9ebb7a6c067efd9e9b2
verify (clean)           ✓ exit=0
mutate cited observation (asserted present in the file)
verify (tampered)        ✗ exit=1
verify (restored)        ✓ exit=0
```

## Reproducing

```bash
cargo test --workspace --no-fail-fast          # 91 tests, incl. the acceptance suite
cargo run -p elenchus-cli -- gates tests/vaults/overclaim   # exit 1
cargo run -p elenchus-cli -- gates tests/vaults/bounded     # exit 0
```

`--no-fail-fast` matters whenever test counts are compared across two trees: cargo
abandons remaining targets after a failure, so the totals would otherwise be counts of
different test sets.

## Known limits

- The corpus is synthetic in its *structure*, though its epistemics are taken from real
  documents. It does not parse a real hive; `amcache-forensic` does that, and elenchus
  reasons over what such a parser reports.
- 13 of the 21 catalogued lenses are not yet mechanised. They are marked `Catalogued`,
  and a meta-test asserts a `Catalogued` lens owns no gates — so the catalogue cannot
  quietly imply enforcement it does not perform.
- The evaluative-term table behind 立極 is a heuristic and will both miss judgements and
  occasionally flag descriptions. `evaluative: true` is the explicit override.
