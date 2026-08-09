# 1. Claim state is derived, and has no field to write it to

Date: 2026-08-09

## Status

Accepted

## Context

The Vibe Research model gives claims a lifecycle (`draft → evidence_pending →
review_ready → accepted | contested | rejected → superseded`) and its manifesto says AI
"may not manufacture evidence or elevate confidence". Stated as a rule, that is advice.
Anything holding a `status:` field can have it written by whoever edits the file last —
including a model asked to "update the status".

## Decision

**No node carries `status` or `confidence`.** The parser refuses any document containing
either, naming the offending key and the reason. State is computed on demand from gates,
reviewer records and the grounded extension.

The rule is general rather than per-kind. A per-kind exception would concede that the
invariant is about claims being special, when it is about no node storing what the engine
derives.

The same shape is applied to edges: a settled grade is stored inseparably from the
reviewer who set it, so an unattributed grade is a value that cannot be constructed
rather than a lint failure caught later. `grade_proposed` exists for anyone — including a
model — to suggest without asserting.

## Consequences

- The CLI has no state-setting subcommand. The absence is the enforcement.
- Round-tripping a vault through a tool that adds `status:` breaks loudly, which is the
  desired behaviour.
- Derived state must be recomputed rather than read, which is cheap at vault scale and
  becomes the job of the derived index if it ever is not.
