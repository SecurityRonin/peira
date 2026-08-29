---
id: c-mints-bounded
type: claim
title: >-
  Every evidence or score token found in the output of the exercised tool routes traced to
  echoed caller input, and none to peira-generated text
aspect: function
quantifier: universal
extension:
  - the output fields of check_prose, examine, status, gates, freeze, verify, propose and
    lens, driven with the adversarial inputs of this audit on this build
uses_term: ["mint-term"]
causal_rung: association
warrant: >-
  A value is minted only when peira computes it; echoing the caller's own input back is not
  minting, so an evidence token that traces to the input is not evidence peira authored.
boundaries:
  - The routes and adversarial-input kinds exercised on this build; not the unbounded input space
  - Elusion (false-negative rate) was not measured beyond the instrument's positive control
falsifier:
  - A peira-generated field carrying grade/by/via or a score that does NOT trace to caller input
  - A route not exercised here found to mint
---
