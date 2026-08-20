# peira — purpose and scope

## Problem

Knowledge systems record conclusions and lose the reasoning. A note saying *"this proves
execution"* looks identical, in every vault ever built, to one saying *"this evidences
presence"* — same format, same links, same search behaviour. The difference only surfaces
under cross-examination, which is the worst possible moment.

Existing tools address adjacent problems. Perplexity shows where an answer looked. Cowork
shows what an agent did. Neither shows **why a claim may be stated, with what strength,
under which boundaries, and what would change it.**

## What peira is

A markdown-native knowledge vault plus a deterministic checker that refuses to promote a
claim which has not been examined for specific, named failure modes — each drawn from a
critical-thinking tradition that identified it.

## Who it is for

- **Expert witnesses and forensic examiners**, where an over-stated claim is not a bug
  report but a cross-examination.
- **Researchers** maintaining long-lived claims across supersession.
- **Anyone** who has written a confident sentence and later could not reconstruct why.

## Scope

**In:** the Open tier (areas `60-99`); typed nodes and edges; the lens catalogue and its
enforced gates; the deterministic lint pack; grounded-extension computation; Court Mode
packets with hash verification.

**Out, for now:** the Governed and Sealed tiers (WORM storage and RFC 3161 timestamping
are hooks, not implementations); the 10 catalogued-but-unenforced lenses; multi-user
review workflow; a web interface.

**Out, permanently:** any path by which a model, or a hurried human, can assert that a
claim is accepted. That is not a missing feature.

## Success

A claim reaching `review_ready` has demonstrably been examined against every enforced
lens, and a reviewer can see which ones, what they found, and what remains unassessed —
with `Unassessed` never rendering as a pass.
