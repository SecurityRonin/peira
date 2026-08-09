//! The argumentation graph and its grounded extension.

use crate::{
    edge::Edge,
    node::{Node, NodeId},
};
use std::collections::{BTreeMap, BTreeSet};

/// A vault's nodes and edges.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    nodes: BTreeMap<NodeId, Node>,
    edges: Vec<Edge>,
}

impl Graph {
    /// An empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node, replacing any node with the same id.
    pub fn insert_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add an edge.
    pub fn insert_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Look up a node.
    #[must_use]
    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Every node, in id order.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Every edge, in insertion order.
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }

    /// Edges whose source is `id`.
    pub fn edges_from<'a>(&'a self, id: &'a NodeId) -> impl Iterator<Item = &'a Edge> {
        self.edges.iter().filter(move |e| &e.from == id)
    }

    /// Edges whose target is `id`.
    pub fn edges_to<'a>(&'a self, id: &'a NodeId) -> impl Iterator<Item = &'a Edge> {
        self.edges.iter().filter(move |e| &e.to == id)
    }

    /// Node ids that reference something absent from the graph, paired with the
    /// dangling target. A dangling reference is a defect, never a silent no-op.
    #[must_use]
    pub fn dangling_edges(&self) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| !self.nodes.contains_key(&e.from) || !self.nodes.contains_key(&e.to))
            .collect()
    }

    /// The attack relation, as `target -> attackers`.
    fn attackers(&self) -> BTreeMap<NodeId, Vec<NodeId>> {
        let mut map: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
        for edge in self.edges.iter().filter(|e| e.kind.is_attack()) {
            map.entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
        }
        map
    }

    /// Whether `arg` is defended by `set` — for every attacker of `arg`, some member
    /// of `set` attacks that attacker.
    fn is_acceptable(
        arg: &NodeId,
        set: &BTreeSet<NodeId>,
        attackers: &BTreeMap<NodeId, Vec<NodeId>>,
    ) -> bool {
        // Written as a match rather than `is_none_or`, which stabilised in 1.82 and
        // would break the 1.75 floor this workspace promises downstreams.
        match attackers.get(arg) {
            None => true,
            Some(attacks_on_arg) => attacks_on_arg.iter().all(|attacker| {
                attackers
                    .get(attacker)
                    .is_some_and(|counter| counter.iter().any(|c| set.contains(c)))
            }),
        }
    }

    /// The **grounded extension** — the least fixed point of the characteristic
    /// function, computed over the attack relation.
    ///
    /// Grounded semantics is chosen deliberately over preferred or stable: it is
    /// unique, polynomial, and the most *skeptical* of the standard semantics — an
    /// argument is IN only when every attack on it is itself defeated. Preferred and
    /// stable semantics are credulous and NP-hard, and a system whose output is meant
    /// to survive cross-examination has no business being credulous. Scepticism here
    /// is the feature.
    ///
    /// Only argument-bearing nodes participate; a `Term` or `Criterion` is reference
    /// material that arguments *use*, and it never competes with them.
    #[must_use]
    pub fn grounded_extension(&self) -> BTreeSet<NodeId> {
        let attackers = self.attackers();
        let arguments: Vec<&NodeId> = self
            .nodes
            .values()
            .filter(|n| n.kind.is_argument())
            .map(|n| &n.id)
            .collect();

        let mut current: BTreeSet<NodeId> = BTreeSet::new();
        // The characteristic function is monotone over a finite set, so it reaches a
        // fixed point within `arguments.len() + 1` steps. The bound is a backstop, not
        // the termination condition: an unbounded loop is a hang waiting to happen.
        for _ in 0..=arguments.len() {
            let next: BTreeSet<NodeId> = arguments
                .iter()
                .filter(|arg| Self::is_acceptable(arg, &current, &attackers))
                .map(|arg| (*arg).clone())
                .collect();
            if next == current {
                return current;
            }
            current = next;
        }
        // cov:unreachable: the characteristic function is monotone over a finite set,
        // so the sequence ∅ ⊆ F(∅) ⊆ F²(∅) ⊆ … strictly grows until it stabilises, and
        // must therefore reach its fixed point within `arguments.len() + 1` steps —
        // which returns above. This line exists so a future change that breaks
        // monotonicity degrades into a conservative answer instead of an infinite
        // loop. Kept deliberately: never delete a defensive guard to satisfy a
        // coverage gate.
        current
    }

    /// Whether `id` survives in the grounded extension.
    #[must_use]
    pub fn is_grounded(&self, id: &NodeId) -> bool {
        self.grounded_extension().contains(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        edge::EdgeKind,
        node::{Fields, NodeKind},
    };

    fn claim(id: &str) -> Node {
        Node {
            id: NodeId::new(id),
            kind: NodeKind::Claim,
            title: id.to_owned(),
            body: String::new(),
            fields: Fields::default(),
        }
    }

    fn term(id: &str) -> Node {
        Node {
            id: NodeId::new(id),
            kind: NodeKind::Term,
            title: id.to_owned(),
            body: String::new(),
            fields: Fields::default(),
        }
    }

    fn attacks(from: &str, to: &str) -> Edge {
        Edge::new(NodeId::new(from), NodeId::new(to), EdgeKind::Contradicts)
    }

    #[test]
    fn an_unattacked_claim_is_grounded() {
        let mut g = Graph::new();
        g.insert_node(claim("a"));
        assert!(g.is_grounded(&NodeId::new("a")));
    }

    #[test]
    fn a_claim_attacked_by_an_unattacked_claim_is_out() {
        let mut g = Graph::new();
        g.insert_node(claim("a"));
        g.insert_node(claim("b"));
        g.insert_edge(attacks("b", "a"));

        let ext = g.grounded_extension();
        assert!(ext.contains(&NodeId::new("b")), "b is unattacked");
        assert!(!ext.contains(&NodeId::new("a")), "a is defeated by b");
    }

    #[test]
    fn reinstatement_a_defended_claim_returns() {
        // c → b → a.  c is unattacked, so it defeats b, which reinstates a.
        // This is the case that separates a real fixed-point computation from a
        // one-pass "is anything pointing at me" check.
        let mut g = Graph::new();
        g.insert_node(claim("a"));
        g.insert_node(claim("b"));
        g.insert_node(claim("c"));
        g.insert_edge(attacks("b", "a"));
        g.insert_edge(attacks("c", "b"));

        let ext = g.grounded_extension();
        assert!(ext.contains(&NodeId::new("c")), "c is unattacked");
        assert!(!ext.contains(&NodeId::new("b")), "b is defeated by c");
        assert!(
            ext.contains(&NodeId::new("a")),
            "a is reinstated: its only attacker is itself defeated"
        );
    }

    #[test]
    fn a_mutual_attack_leaves_both_out_under_grounded_semantics() {
        // Grounded is sceptical: an unresolved stand-off yields no winner, where a
        // credulous semantics would pick one arbitrarily.
        let mut g = Graph::new();
        g.insert_node(claim("a"));
        g.insert_node(claim("b"));
        g.insert_edge(attacks("a", "b"));
        g.insert_edge(attacks("b", "a"));

        let ext = g.grounded_extension();
        assert!(!ext.contains(&NodeId::new("a")));
        assert!(!ext.contains(&NodeId::new("b")));
    }

    #[test]
    fn an_odd_cycle_terminates_and_yields_nothing() {
        // a → b → c → a. Famous non-terminating case for a naive implementation.
        let mut g = Graph::new();
        for id in ["a", "b", "c"] {
            g.insert_node(claim(id));
        }
        g.insert_edge(attacks("a", "b"));
        g.insert_edge(attacks("b", "c"));
        g.insert_edge(attacks("c", "a"));

        assert!(g.grounded_extension().is_empty());
    }

    #[test]
    fn reference_objects_do_not_compete_with_arguments() {
        let mut g = Graph::new();
        g.insert_node(claim("a"));
        g.insert_node(term("60.01"));

        let ext = g.grounded_extension();
        assert!(ext.contains(&NodeId::new("a")));
        assert!(
            !ext.contains(&NodeId::new("60.01")),
            "a term is used by arguments, it does not join them"
        );
    }

    #[test]
    fn retraction_does_not_act_as_an_attack() {
        let mut g = Graph::new();
        g.insert_node(claim("a"));
        g.insert_node(claim("b"));
        g.insert_edge(Edge::new(
            NodeId::new("b"),
            NodeId::new("a"),
            EdgeKind::Retracts,
        ));

        assert!(
            g.is_grounded(&NodeId::new("a")),
            "retraction is a lifecycle act on a claim, not a competing argument"
        );
    }

    #[test]
    fn dangling_edges_are_reported_not_ignored() {
        let mut g = Graph::new();
        g.insert_node(claim("a"));
        g.insert_edge(attacks("ghost", "a"));

        assert_eq!(g.dangling_edges().len(), 1);
    }
}
