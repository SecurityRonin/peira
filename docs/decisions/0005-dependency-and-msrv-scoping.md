# 5. Dependency choices, and scoping a propagated MSRV

Date: 2026-08-09

## Status

Accepted

## Context

Three dependency decisions each had a trap.

**YAML.** `serde_yaml` is archived upstream and carries an unmaintained advisory;
forensicnomicon pins it at 0.9. Inheriting it into a new repo under a `cargo deny` gate
means inheriting a suppression.

**Hashing.** ADR-0010 makes preferring our own crates a hard rule, and `blazehash-core`
exposes exactly what packet digests need. It declares `rust-version = "1.88"`, while
`elenchus-core` and `elenchus-lens` promise 1.75 — a floor downstream consumers pin
against.

**MSRV enforcement.** A declared `rust-version` is a claim until something compiles
against it.

## Decision

- **`serde_yaml_ng`** for frontmatter. Maintained fork, drop-in serde integration, and
  an MSRV of 1.64 that does not drag our floor up. Rejected: `serde_yml` (self-describes
  as deprecated), `serde-saphyr` (1.0.1, wide dependency graph), `saphyr` (0.0.x, and its
  1.85 floor would raise our promise by ten releases). It is isolated behind `parse_node`
  so a later swap is a one-function change.
- **`blazehash-core`** for digests, and never a hand-rolled one. The 1.88 constraint is
  **scoped to `elenchus-court`**, where the dependency is actually used, instead of
  propagating up into the library crates.
- **MSRV is measured, not asserted — and the first measurement failed.** 1.75 was
  declared, and the msrv job disproved it immediately: `indexmap 2.14` (pulled by
  `serde_yaml_ng`) requires `edition2024`, which needs Cargo 1.85, so resolution fails
  before any of our code is reached. The declared floor is now **1.85, verified by
  `cargo +1.85 check`**.

  The distinction matters and is recorded rather than smoothed over: **1.85 is an
  inherited floor, not this code's own.** `elenchus-core` and `elenchus-lens` very likely
  compile lower — clippy's `incompatible_msrv` was already enforcing 1.75 against our
  source during development, and caught `Option::is_none_or` (stabilised 1.82), which was
  rewritten as a match rather than raising the floor. But our own floor is **unmeasured**,
  because nothing can compile it until the graph resolves. A number not observed is not a
  measurement, and must not be reported as one.

## Consequences

- The MSRV promise means something, because it fails when broken.
- A propagated constraint is fixed where it originates rather than surveyed where it
  lands. `blazehash-core`'s own 1.88 may itself be over-declared; that is a question for
  that repo, and not one to answer by raising our floor.
- `serde_yaml_ng` brings `unsafe-libyaml` (transpiled C) into the graph. Our crates
  `forbid(unsafe_code)`, but the dependency does not, and that is recorded here rather
  than left implicit.
