# Grounded Extension — Compute, Don't Assert

<!-- Generated from peira_lens::CATALOG by cli/examples/gen_site.rs. Edit the lens in lens/src/lib.rs and regenerate. -->

**peira code:** `DUNG` · **Formal — argumentation theory** · **Enforced** — owns a gate

**Failure mode:** dialectical status asserted by whoever wrote last, rather than computed from the attack relation

**What it does:** claim standing is the least fixed point of the characteristic function

**Worked example:** c defeats b, which reinstates a. No participant need agree; the result follows from the attack graph, and grounded semantics refuses to pick a winner in a stand-off.

**In the tradition:** Phan Minh Dung's 1995 paper (Artificial Intelligence 77) founded abstract argumentation: arguments and an attack relation form a graph, and which arguments are acceptable is computed, not asserted — one stands if every attacker is itself defeated. The grounded extension is the least such set, and it declines to crown a winner in an unbroken stand-off. The lens uses it: a claim's standing is the fixed point of the attack graph, not whoever spoke last.

**Sources:**
- P. M. Dung, Artificial Intelligence 77 (1995) 321–357
- <https://doi.org/10.1016/0004-3702(94)00041-X>

Run `peira lens DUNG` for this entry in the tool.
