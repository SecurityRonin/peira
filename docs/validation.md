# Validation

**What is claimed:** the enforced gates discriminate between an over-stated claim and a
bounded one, and the discrimination is produced deterministically, with no model in the
loop.

**What is not claimed:** that the Amcache forensic conclusions in the corpus are true.
The corpus exercises the *engine*. Its ground truth is taken from a third-party README
and from the Vibe Research design document; peira did not establish it.

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
PEIR-CRITERION-UNDECLARED       [LIJI]       judged, with no standard declared
PEIR-FUNCTION-AS-SUBSTANCE      [TIYONG]     what it did, stated as what it is
PEIR-CLASS-EXTENSION-UNDECLARED [BAIMA]      one token, quantified universally
PEIR-CORNERS-UNADDRESSED        [CATUSKOTI]  contested, addresses 0 of 4 corners
PEIR-WARRANT-MISSING            [TOULMIN]    states no warrant
PEIR-CAUSAL-RUNG-UNREACHED      [RUNG]       counterfactual rung, observation only
PEIR-BOUNDARIES-MISSING         [RUNG]       no boundary conditions
PEIR-LINT-FORBIDDEN-VERB        [LINT]       says "proves"
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
baseline                 PEIR-CAUSAL-RUNG-UNREACHED findings: 1
gate neutered            PEIR-CAUSAL-RUNG-UNREACHED findings: 0   (6 findings total)
restored                 PEIR-CAUSAL-RUNG-UNREACHED findings: 1
```

### Control C — an absent vault is not an empty pass

```
peira: vault root `/nonexistent/vault` is not a directory
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
cargo test --workspace --no-fail-fast          # 141 tests, incl. the acceptance suite
cargo build -p peira-cli && tests/controls.sh target/debug/peira   # A, B and C
```

The three controls live in `tests/controls.sh` rather than inline in the CI job, so
the pre-commit hook and CI run the same bytes. Restated in two places they drift,
and a drifted control still reports green.

`--no-fail-fast` matters whenever test counts are compared across two trees: cargo
abandons remaining targets after a failure, so the totals would otherwise be counts of
different test sets.

## Coverage

**97.98% of lines**, over the workspace excluding the binary shell
(`src/main.rs`, `src/bin/`) — measured with `cargo llvm-cov --workspace`, never
`--lib`, which builds only each lib's own unit tests and so cannot see the
integration suite.

```
core/src/edge.rs     100.00%      lens/src/gates.rs     99.25%
core/src/graph.rs     99.44%      lens/src/lib.rs       98.93%
core/src/node.rs      98.57%      lens/src/lints.rs     98.50%
core/src/vault.rs     95.42%      court/src/lib.rs      95.73%
index/src/lib.rs      93.69%      TOTAL                 97.98%
```

**This is below the fleet's 100% standard, and the gap is stated rather than
rounded away.** The 51 uncovered lines are, in order of count: SQLite and I/O
error-propagation arms reached only by injecting filesystem or database failures;
and `panic!` arms inside test helpers, which by construction never execute while
the tests pass.

One line is deliberately unreachable and annotated rather than deleted:
`graph.rs` returns a conservative answer if the characteristic function's fixed
point is somehow not reached within `n+1` steps. Monotonicity over a finite set
makes that impossible today; the guard exists so a future change that breaks
monotonicity degrades instead of hanging. Never delete a defensive guard to
satisfy a coverage gate.

The CI job enforces `--fail-under-lines 97`, and **the threshold was verified to
fail**: at 99 it exits 1, at 97 it exits 0. A coverage job with no threshold
proves only that the tool ran.

## The supply-chain and robustness gates, each proven able to fail

A gate that has never gone red is not known to work. Every gate added for publication
was mutated until it failed, then restored and re-run — and each mutation was asserted
present in the file before the run, because a mutation that silently fails to apply
reports green while testing nothing.

| Gate | Mutation | Observed |
|---|---|---|
| `cargo vet` | removed the `serde_yaml_ng` exemption | exit 255, *"1 unvetted dependencies"* → restored, exit 0 |
| `tests/controls.sh` | `examine_graph` returns no violations | *"control A must exit 1; got 0"*, exit 1 → restored, exit 0 |
| `fuzz_parse_node` | `reject_derived_fields` neutered | panic at the assertion in ~62k runs (≈9 s) → restored, 1.06 M runs clean |
| gitleaks (`dir`) | planted a PAT in the working tree | exit 1 → clean tree, exit 0 |
| gitleaks (`git`) | planted a PAT in a commit | exit 1 → clean history, exit 0 |
| gitleaks (`stdin`) | planted a PAT in a commit *message* | exit 1, where `git` mode returns clean |

Two things that surfaced only because the controls were run:

**AWS's documented example key is allowlisted.** The first probe used
`AKIAIOSFODNN7EXAMPLE` and gitleaks returned exit 0 — reading as *"the scanner is
broken"* when the scanner was right. Probe a detector with a value it is meant to
catch, not with the one every vendor prints in its own documentation.

**`gitleaks git` and `gitleaks dir` are structurally blind to commit messages.** With
the secret removed from the file and left only in a message, both return clean and
only the `stdin` scan over `git log --format=%B` returns 1. A credential pasted into
a message is exactly the kind that reaches a public repo, so CI runs all three.

### Fuzzing

Two targets, seeded from the real vault fixtures rather than from nothing:

- **`fuzz_parse_node`** — one document. Beyond *does not panic*, it asserts the
  load-bearing invariant directly: a parse that SUCCEEDS must not have admitted a
  `status` or `confidence` field. The fuzzer is therefore searching for a way past
  the refusal, not merely for a crash.
- **`fuzz_vault`** — the whole pipeline over a directory of documents: edge
  construction, the gates, the lints, the grounded extension, and a `freeze`/`verify`
  round trip. It asserts that the grounded extension terminates and agrees with
  `is_grounded`, and that a packet frozen from an unchanged graph verifies against it
  — which is what would catch iteration order leaking into a digest.

Clean at 1,059,003 and 291,704 runs respectively. **That is a smoke run, not a
campaign**: 60 s per target is enough to gate a pull request and is not evidence of
absence of defects.

## Known limits

- The corpus is synthetic in its *structure*, though its epistemics are taken from real
  documents. It does not parse a real hive; `amcache-forensic` does that, and peira
  reasons over what such a parser reports.
- 13 of the 21 catalogued lenses are not yet mechanised. They are marked `Catalogued`,
  and a meta-test asserts a `Catalogued` lens owns no gates — so the catalogue cannot
  quietly imply enforcement it does not perform.
- The evaluative-term table behind 立極 is a heuristic and will both miss judgements and
  occasionally flag descriptions. `evaluative: true` is the explicit override.
