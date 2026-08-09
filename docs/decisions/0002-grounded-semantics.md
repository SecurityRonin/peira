# 2. Grounded semantics, not preferred or stable

Date: 2026-08-09

## Status

Accepted

## Context

Dung's abstract argumentation offers several semantics for deciding which arguments
survive an attack relation. Preferred and stable semantics are *credulous*: where
arguments deadlock, they select a maximal consistent set, effectively picking a winner.
Both are NP-hard, and stable extensions may not exist at all.

## Decision

Compute the **grounded extension** only — the least fixed point of the characteristic
function.

It is unique, polynomial, and the most *sceptical* of the standard semantics: an argument
is IN only when every attack on it is itself defeated. A mutual attack leaves both out.

## Consequences

- A stand-off yields no winner. For a system whose output is meant to survive
  cross-examination, that is the correct answer and not a limitation — being unable to
  say which of two competing claims wins **is** the finding.
- Reinstatement works: `c` defeats `b`, which returns `a` to the extension. This is what
  distinguishes a real fixed-point computation from a one-pass "is anything pointing at
  me" check, and it is tested directly.
- Retraction is deliberately excluded from the attack relation. A retraction is a
  lifecycle act on a claim's own history; treating it as an attack would let a third
  claim "defend" against it.
