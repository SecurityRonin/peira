---
title: peira
description: A knowledge system that refuses to promote a claim you have not examined.
nav_order: 1
---

# peira

**A knowledge system that refuses to promote a claim you have not examined** — with gates
drawn from Socratic ἔλεγχος, the 金剛經, classical Indian logic, 中觀 and the causal ladder.

> Care is not a control. A control is something that can go red, and it must be shown to do so.

An examiner who is being careful and an examiner who is being fooled produce the same
subjective experience. The difference is external: a check that fails on a known-bad input,
run before the conclusion is trusted. peira is that check, made deterministic — no model in
the loop, naming the tradition that identified each mistake.

## Two ways in

<div class="cards">
  <h3><a href="{{ '/doors/' | relative_url }}">→ The seven doors</a></h3>
  <p>The <strong>anti-summarization pass</strong>: seven probing questions for distilling any source without letting compression smooth away its contradictions, unsupported claims, and omissions. Each door leads to the lenses behind it.</p>

  <h3><a href="{{ '/lenses/' | relative_url }}">→ The lens catalogue</a></h3>
  <p>Twenty-seven <strong>named ways of being wrong</strong>, each from a critical-thinking tradition. Thirteen are enforced as deterministic gates; the rest are catalogued readings. Every one carries a worked example and authoritative sources.</p>

  <h3><a href="{{ '/method/' | relative_url }}">→ The method</a></h3>
  <p>The reasoning peira mechanises, written out so the tool stands alone: the six structures of investigative error, the claim-grading standard, and reporting to a tribunal.</p>
</div>

## What a gate looks like

```console
$ peira gates vault/
✗ gates: 8 finding(s).

  PEIR-FUNCTION-AS-SUBSTANCE [TIYONG]  c-overclaim
      substance claim "This Amcache entry proves execution of the suspicious binary"
      rests only on function evidence (o1, o2)
      → restate as a claim about what the thing did, or add evidence bearing on what it is
```

The safe statement is **generated from the graph, never authored**, because a sentence a
human wrote is a sentence no checker can reliably police. A node has no `status` field and no
`confidence` field — there is nowhere to write one, and the parser refuses a document that
tries. Claim standing is *derived*, never asserted.

## Install

```bash
cargo install --git https://github.com/SecurityRonin/peira peira-cli
peira init vault/
peira gates vault/
```

`peira method anti-summarization` prints the anti-summarization framework from the binary,
version-stamped; `peira lens <ID>` shows any lens in full.
