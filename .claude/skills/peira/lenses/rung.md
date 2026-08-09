# Causal Ladder — Earn the Rung

**Gates:** `PEIR-CAUSAL-RUNG-UNREACHED`, `PEIR-BOUNDARIES-MISSING`
**Failure mode:** asserting interventional or counterfactual conclusions from
observational data — and stating a conclusion with no conditions under which it would
change.

Pearl's ladder has three rungs, and evidence from one cannot license a claim on another:

| Rung | | What it takes |
|---|---|---|
| 1 | **association** — seeing | observation |
| 2 | **intervention** — doing | a controlled protocol, actually executed |
| 3 | **counterfactual** — imagining | rung-2 evidence plus a model of the alternative |

## What fires this

`causal_rung: intervention` or `counterfactual` with no supporting node of kind `run` —
that is, nothing was ever executed. And separately, any claim with no `boundaries:`.

A claim that declares no rung at all returns `Unassessed`. Not a pass.

## What to look for

"Proves execution" is a rung-3 assertion: it says that had the file only been copied,
the record would be absent. Nothing in the vault compares those two worlds. The data is
rung 1.

The test matrix the Vibe doc specifies is what rung 2 actually costs: clean baseline,
copy without launch, install, inventory pass, antimalware scan, explicit execution,
deletion after introduction — each on a pinned build, with negative controls and
repeated runs. *"The operator did not double-click it"* is not a control.

## What to write

Either restate at the rung the evidence reaches:

```yaml
causal_rung: association
```

or add the protocol and its execution:

```markdown
---
id: r1
type: run
title: Controlled launch on 22H2, snapshot-reverted
protocol: p-amcache-matrix
environment: "Windows 11 22H2 build 22621.3007, VM snapshot S1"
supports: ["c-execution grade=G3 by=albert via=perception"]
---
```

## Boundaries

Every claim declares them, and each should carry its citation rather than being a bare
string — `version_history.rs` in forensicnomicon is the model: a change is recorded with
both the version it took effect in *and* a reference.

```yaml
boundaries:
  - Windows 10 1809 and later, where InventoryApplicationFile is populated
  - Amcache.hve as acquired; not a reconstructed or merged hive
  - Says nothing about builds where the population mechanism differs
```

The last one matters most. A boundary that names what the claim does **not** cover is
worth more than three that restate what it does.
