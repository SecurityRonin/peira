//! Typed node/edge model, vault parser and argumentation graph for elenchus.
//!
//! # The load-bearing invariant
//!
//! A node has **no `status` field and no `confidence` field**, and the parser
//! *refuses* a document that carries one. Claim state is derived by the engine from
//! gates, reviewer records and the grounded extension; it is never written by hand
//! and never written by a model. The wrong thing is made impossible rather than
//! discouraged — you cannot write what has no field, and a document that tries is a
//! loud parse error naming the offending key.
//!
//! The same shape recurs throughout: an [`edge::Edge`]'s settled grade is stored
//! inseparably from the reviewer who set it, so an unattributed grade is not a lint
//! failure caught later but a value that cannot be constructed.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod edge;
pub mod graph;
pub mod node;
pub mod vault;

pub use edge::{Edge, EdgeKind, Grade, Pramana};
pub use graph::Graph;
pub use node::{parse_node, Fields, Node, NodeId, NodeKind, ParseError};
pub use vault::{load, VaultError};
