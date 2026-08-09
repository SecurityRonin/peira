//! The elenchus lens catalogue.
//!
//! A zero-dependency knowledge leaf, in the shape of `forensicnomicon`: every lens
//! compiles to `static`/`const` memory, so a malformed catalogue is a compile error
//! rather than a runtime surprise, matches are exhaustive at compile time, and the
//! catalogue is auditable as source.
//!
//! This crate holds only **domain-neutral** examinations. Domain packs
//! (`elenchus-forensic`, `elenchus-legal`) depend down onto it and add their own
//! criteria — the mechanism by which "universal" stays universal.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
