# peira

[![CI](https://github.com/SecurityRonin/peira/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/peira/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/sponsor-h4x0r-ea4aaa?logo=githubsponsors)](https://github.com/sponsors/h4x0r)

**A knowledge system that refuses to promote a claim you have not examined — with gates drawn from Socratic elenchus, 金剛經, Nyāya, Madhyamaka and the causal ladder.**

Your notes already hold claims. Nothing in them stops you writing *"this proves
execution"* over evidence that shows presence. peira does — deterministically, with
no model in the loop, naming the tradition that identified the mistake.

```console
$ peira gates vault/
✗ gates: 7 finding(s).

  PEIR-FUNCTION-AS-SUBSTANCE [TIYONG]  c-overclaim
      substance claim "This Amcache entry proves execution of the suspicious binary"
      rests only on function evidence (o1, o2)
      → restate as a claim about what the thing did, or add evidence bearing on what it is

  PEIR-CAUSAL-RUNG-UNREACHED [RUNG]  c-overclaim
      claims the counterfactual rung but rests on observation alone —
      no executed protocol supports it
      → run a controlled protocol and cite the Run, or restate at the association rung
```

## Install

```bash
cargo install peira-cli
peira init vault/
```

## The idea

Every classical critical-thinking tool names a **specific way of being wrong**. Written
as a rule about a claim graph, each becomes machine-checkable:

| Move | The failure it names | The gate |
|---|---|---|
| 立極 establishing the pole | judging with no declared standard | evaluative claim needs a `judged_by` edge |
| 所謂 X 即非 X 是名 X | reifying a label into a thing | key terms carry all three moments |
| 體用 substance/function | what it *did*, stated as what it *is* | substance claims need substance evidence |
| 白馬非馬 | one token, quantified universally | class claims declare their extension |
| 四句 catuṣkoṭi | binarising a contested question | all four corners addressed |
| Toulmin | the unstated warrant | `warrant` is required |
| pramāṇa | testimony passed off as observation | grade ceilings by means of knowing |
| Pearl's ladder | rung-3 claims from rung-1 data | interventional claims need an executed protocol |

Twenty-one lenses are catalogued; eight are enforced today. `peira lens` lists them,
each with its sources and a worked example of its gate firing.

## The invariant

**A node has no `status` field and no `confidence` field.** Not "you shouldn't write
one" — there is nowhere to write it, and the parser refuses a document that tries,
naming the key. Claim state is derived from gates, reviewer records and the grounded
extension.

The same shape recurs. An edge's settled grade is stored inseparably from the reviewer
who set it, so an unattributed grade is a value that cannot be constructed. A model can
propose nodes, edges and grades indefinitely and never assert that anything is accepted.

## Court Mode

The safe statement is **generated from the graph, never authored** — because a sentence
a human wrote is a sentence no checker can reliably police:

```
所謂「execution」— what is called "execution": a user or service ran the program
即非「execution」— but the record is not the thing: Amcache does not observe process
                  creation; nothing distinguishes a launch from an install or a scan
是名「execution」— so it is named "execution" only as: a process was created from this
                  image, established only by evidence that observes process creation
```

`peira packet` refuses to freeze while any gate blocks, and refuses a claim defeated
in the grounded extension. There is no override flag.

## Evidence that the gates work

A green checker and a checker that passes everything look identical, so both controls
are run:

| Control | Setup | Result |
|---|---|---|
| **A** | `"This Amcache entry proves execution"` | **BLOCKS** — 7 gates + 1 lint, exit 1 |
| **B** | the bounded conclusion | **PASSES** — exit 0, packet freezes |
| **B′** | causal-rung gate neutered | that finding disappears, then returns on restore |
| **C** | vault absent | exit 2 in 2 ms — distinguishable from A |

Full transcript in [`docs/validation.md`](docs/validation.md).

## Documentation

- [Purpose and scope](docs/PRD.md)
- [Validation](docs/validation.md)
- [Architecture decisions](docs/decisions/)

---

[Privacy](docs/privacy.md) · [Terms](docs/terms.md) · © Security Ronin Ltd
