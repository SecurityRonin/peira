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
`peira-core` and `peira-lens` promise 1.75 — a floor downstream consumers pin
against.

**MSRV enforcement.** A declared `rust-version` is a claim until something compiles
against it.

## Decision

- **`serde_yaml_ng`** for frontmatter. Maintained fork, drop-in serde integration, and
  an MSRV of 1.64 that does not drag our floor up. Rejected: `serde_yml` (self-describes
  as deprecated), `serde-saphyr` (1.0.1, wide dependency graph), `saphyr` (0.0.x, and its
  1.85 floor would raise our promise by ten releases). It is isolated behind `parse_node`
  so a later swap is a one-function change.
- **`blazehash-core`** for digests, and never a hand-rolled one. The 1.88 constraint
  entered at `peira-citation`, where the dependency is actually used.

  **[SUPERSEDED 2026-08-24 — the scoping no longer holds.]** `peira-index` gained a
  `peira-citation` dependency when `all_findings` became one implementation shared with
  the CLI, so 1.88 now reaches `peira-index` too. It went on declaring 1.85 — a floor its
  own graph could not satisfy, which is worse than an overstated one because it fails for
  CONSUMERS rather than for us. It survived because the msrv job checked two of the five
  crates. `peira-index` now declares **1.88**, and the job checks every crate that
  declares a floor, with a negative control asserting 1.85 is REFUSED for it.
- **MSRV is measured, not asserted — and the first measurement failed.** 1.75 was
  declared, and the msrv job disproved it immediately: `indexmap 2.14` (pulled by
  `serde_yaml_ng`) requires `edition2024`, which needs Cargo 1.85, so resolution fails
  before any of our code is reached. The declared floor is now **1.85, verified by
  `cargo +1.85 check`**.

  The distinction matters and is recorded rather than smoothed over: **1.85 is an
  inherited floor, not this code's own.** `peira-core` and `peira-lens` very likely
  compile lower — clippy's `incompatible_msrv` was already enforcing 1.75 against our
  source during development, and caught `Option::is_none_or` (stabilised 1.82), which was
  rewritten as a match rather than raising the floor.

  **[MEASURED 2026-08-24 — previously recorded as unmeasured.]** Pinning `indexmap` back
  to 2.11.4 (the last edition-2021 release; `serde_yaml_ng` requires only `^2.2.1`, so
  2.14 was never forced) lets `peira-core` and `peira-lens` compile on **1.80**. Our own
  floor is therefore at most 1.80, and the declared 1.85 is entirely inherited.

  The declaration stays at 1.85 anyway, deliberately. A consumer resolving fresh gets
  `indexmap 2.14` and needs 1.85, so declaring lower without capping `indexmap` in the
  manifest would publish a promise false for everyone downstream. Capping a transitive
  dependency to protect a floor is a freshness liability, and it was weighed and declined
  — but the option is recorded here so the next reader does not have to re-derive it.

  Two probes below 1.80 returned `failed to parse lock file ... version 4`. That is a
  LOCKFILE FORMAT error, not a language one, and reading it as an MSRV result would have
  invented a floor that does not exist.

## Consequences

- The MSRV promise means something, because it fails when broken.
- A propagated constraint is fixed where it originates rather than surveyed where it
  lands. `blazehash-core`'s own 1.88 may itself be over-declared; that is a question for
  that repo, and not one to answer by raising our floor.
- `serde_yaml_ng` brings `unsafe-libyaml` (transpiled C) into the graph. Our crates
  `forbid(unsafe_code)`, but the dependency does not, and that is recorded here rather
  than left implicit.
