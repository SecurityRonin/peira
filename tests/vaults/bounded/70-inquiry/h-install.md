---
id: h-install
type: hypothesis
title: Installation or inventory produced the record without user execution
warrant: Windows populates the same table during install and inventory passes.
quantifier: singular
aspect: function
causal_rung: association
boundaries:
  - Windows 10 1809 and later
falsifier:
  - >-
    A prefetch or SRUM record placing the image in execution within the same
    window would leave this mechanism unable to account for the record on its own
limits: [c-bounded]
---

A competing mechanism for the same observation. Against the bounded claim this
is a LIMIT rather than an attack: the claim already declines to say the program
ran, so the alternative bounds its scope instead of contradicting it.

That change of edge is the whole lesson. Restating a claim within what the
evidence carries turns an attack into a limit.
