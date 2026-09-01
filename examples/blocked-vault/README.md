# Example vault — a claim peira **refuses to freeze**

A small, complete peira vault that does **not** freeze. One over-claimed claim, the term it
rests on, and one supporting observation. Where [`prefetch-vault`](../prefetch-vault) shows the
bounded claim that passes, this one shows the draft it started from — the *before* to that
*after*. It is here to be read and run: point `peira packet` at it and watch the refusal come
back as a **result**, not an error.

> **Provenance.** Constructed scenario, authored 2026-09-01 for documentation. It is
> *synthetic*: there is no real Prefetch file behind `o-prefetch`, and the `sealed://` pointer
> resolves to nothing. The over-statement is deliberate — the vault exists to be refused.

## What it holds

| File | Node | Role |
|---|---|---|
| `60-lexicon/exec-01.md` | `execution` (term) | the load-bearing term, with its three moments (`as_used` / `not_essence` / `stipulated`) |
| `70-inquiry/o-prefetch.md` | `o-prefetch` (observation) | the supporting observation (run count 3), graded `G2 via=perception` — *the same evidence that supports the bounded claim next door* |
| `70-inquiry/c-prefetch-draft.md` | `c-prefetch-draft` (claim) | the **over-claimed** draft — *“The Prefetch file proves that the user executed Notepad”* |

The evidence is sound. Everything that fails, fails in the **claim** built on top of it.

## The refusal

The enforced lens gates, verbatim — five of them, each naming the lens that raised it and the
one repair that answers it:

```console
$ peira gates examples/blocked-vault
✗ gates: 5 finding(s).

  PEIR-WARRANT-MISSING [TOULMIN]  c-prefetch-draft
      "The Prefetch file proves that the user executed Notepad" states no warrant
      → write the rule that licenses the step from grounds to claim — it is usually the part that turns out to be false

  PEIR-CAUSAL-RUNG-UNREACHED [RUNG]  c-prefetch-draft
      "The Prefetch file proves that the user executed Notepad" claims the counterfactual rung but rests on observation alone — no executed protocol supports it
      → run a controlled protocol and cite the Run, or restate the claim at the association rung

  PEIR-BOUNDARIES-MISSING [RUNG]  c-prefetch-draft
      "The Prefetch file proves that the user executed Notepad" declares no boundary conditions
      → name the versions, configurations or populations where the claim holds — and cite each, never a bare string

  PEIR-RIVALS-UNENUMERATED [ACH]  c-prefetch-draft
      "The Prefetch file proves that the user executed Notepad" explains counterfactually and the vault records nothing it was tested against — an explanation with no rival has been confirmed by consistency, not chosen
      → write down the competing explanation you rejected and attack this claim with it — the one that survives is the finding, and the ones that did not are the reason anyone should believe it

  PEIR-FALSIFIER-MISSING [PREMORTEM]  c-prefetch-draft
      "The Prefetch file proves that the user executed Notepad" records nothing that would defeat it — as written, no observation could count against it
      → state what would have to be observed for this to be wrong, as `falsifier:` or as a node that attacks it
```

`peira lint` adds the two the packet would carry into the artifact — both
`PEIR-LINT-FORBIDDEN-VERB`, on the word *“proves”* — and `peira packet` unions all seven and
refuses:

```console
$ peira packet examples/blocked-vault c-prefetch-draft
`c-prefetch-draft` cannot be frozen — 7 gate(s) block:
  …
```

It exits non-zero, but this is not a crash — it is the finding. Over the MCP surface the same
refusal is a structured **result**, not an error — `outcome: "blocked"`, carrying every
violation in a `violations` array:

```json
{ "node": "c-prefetch-draft", "outcome": "blocked",
  "violations": [ { "code": "PEIR-WARRANT-MISSING", "lens": "TOULMIN", "subject": "c-prefetch-draft", "detail": "…", "remedy": "…" } ] }
```

`peira_freeze` returns that; it does not raise. A bad request — a missing node, or a node that
is not a claim — *is* an error. A claim that simply has not earned promotion is a result. The
two are kept apart on purpose.

### Blocked is not defeated

```console
$ peira status examples/blocked-vault c-prefetch-draft
  grounded extension : IN — every attack on it is itself defeated
  gates              : 7 blocking
  derived state      : evidence_pending — gates block
```

Read the first two lines together. Nothing in the vault *argues against* the draft, so it sits
`IN` the grounded extension — it is not **defeated**. It is **blocked**: seven gates say it has
not been examined enough to freeze. `outcome=blocked` (gates in the way) and `outcome=defeated`
(an attack you could not answer) are different refusals, and this vault isolates the first.

## The fix

Each finding names one repair. Together they are the same few structural moves — drop the
verdict verb, step down to the rung the evidence actually reaches, and record the three things
a reader is entitled to (why it follows, where it holds, what would break it):

```diff
 ---
 id: c-prefetch-draft
 type: claim
-title: The Prefetch file proves that the user executed Notepad
+title: >-
+  A Prefetch file whose run count is at least one evidences that Windows started the named
+  program at least once, within the bounds stated below
+warrant: >-
+  Windows writes a Prefetch (.pf) file and increments its run counter when the image loader
+  prefetches an executable for a process it is starting. A non-zero counter therefore
+  evidences a start event, and licenses a statement about program starts and nothing beyond.
 aspect: function
 quantifier: singular
-causal_rung: counterfactual
+causal_rung: association
 uses_term: ["exec-01"]
+boundaries:
+  - Windows client SKUs with SysMain (Prefetch) enabled; where it is disabled, absence proves nothing
+  - The .pf as acquired, not a carved or reconstructed fragment
+  - Says nothing about which user started it, nor whether the process completed
+falsifier:
+  - A demonstrated write path that increments the counter without a process start — a
+    prefetch warm-up, a security product, or a testing harness — would defeat the start reading
+  - A build or configuration where the run counter is shown not to track starts
 ---
```

Notice what the repair does to the domain claim: *“proves that the user executed”* becomes
*“evidences that Windows started”*. The two over-reaches the gates could not see — attributing
the act to **a user**, and reading a **start** as an intentional **execution** — are answered
not by a gate but by the boundary line *“says nothing about which user started it”*. The gates
force the fields; writing them honestly forces the retreat.

Apply that diff and the claim carries the very fields of
[`prefetch-vault`](../prefetch-vault)'s `c-prefetch` — and freezes:

```console
$ peira gates <fixed>
✓ gates: nothing to report.
$ peira packet <fixed> …           # now it freezes
# Citation packet — c-prefetch
…
```

## The lesson

The gates cannot read the sentence's meaning — they cannot know that *“the user”* is an
attribution the artifact will not carry. What they can do is refuse to let the claim skip the
steps that would have surfaced it: a warrant that must be written down, a rung that must match
the evidence, boundaries that must name where it holds. The over-claim does not survive being
made to show its work. That is the whole mechanism — no model in the loop, only the refusal to
freeze what has not been examined.
