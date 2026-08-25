---
id: c-prefetch
type: claim
title: >-
  A Prefetch file whose run count is at least one evidences that Windows started the named
  program at least once, within the bounds stated below
warrant: >-
  Windows writes a Prefetch (.pf) file and increments its run counter when the image loader
  prefetches an executable for a process it is starting. A non-zero counter therefore
  evidences a start event, and licenses a statement about program starts and nothing beyond.
aspect: function
quantifier: singular
causal_rung: association
uses_term: ["exec-01"]
boundaries:
  - Windows client SKUs with SysMain (Prefetch) enabled; where it is disabled, absence proves nothing
  - The .pf as acquired, not a carved or reconstructed fragment
  - Says nothing about which user started it, nor whether the process completed
falsifier:
  - A demonstrated write path that increments the counter without a process start — a
    prefetch warm-up, a security product, or a testing harness — would defeat the start reading
  - A build or configuration where the run counter is shown not to track starts
---

Start evidence at the association rung, with the alternative counter-increment mechanisms
left standing in the falsifier rather than argued away.
