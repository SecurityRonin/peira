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
    /// Adjacency indices into `edges`, by source and by target.
    ///
    /// `edges_from`/`edges_to` were full scans of the edge vector, and the promotion
    /// gates run a transitive walk that calls them once per node per gate — so a chain
    /// of 400 nodes took 17 seconds and the cost grew cubically. A forensic vault is
    /// not small, and a checker nobody can afford to run is a checker nobody runs.
    ///
    /// Held inside `Graph` and updated in `insert_edge`, so no caller can forget it and
    /// no caller changes.
    out: BTreeMap<NodeId, Vec<usize>>,
    inc: BTreeMap<NodeId, Vec<usize>>,
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
        let i = self.edges.len();
        self.out.entry(edge.from.clone()).or_default().push(i);
        self.inc.entry(edge.to.clone()).or_default().push(i);
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
        self.out
            .get(id)
            .into_iter()
            .flatten()
            .filter_map(move |i| self.edges.get(*i))
    }

    /// Edges whose target is `id`.
    pub fn edges_to<'a>(&'a self, id: &'a NodeId) -> impl Iterator<Item = &'a Edge> {
        self.inc
            .get(id)
            .into_iter()
            .flatten()
            .filter_map(move |i| self.edges.get(*i))
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
    /// Every node the record withdraws, resolved to a fixed point.
    ///
    /// A retraction is an AUTHORED ASSERTION, not a fact about the world, and it binds
    /// only while it itself stands: a retraction whose author has been retracted stops
    /// binding, and what it withdrew comes back. Otherwise the last writer wins.
    ///
    /// Public because two callers need the SAME answer. `attackers` uses it to decide
    /// who may argue, and the lint pack uses it to decide whether a packet's subject is
    /// withdrawn — and when those disagreed, `peira status` reported `review_ready` over
    /// a claim `peira packet` refused. One question, asked once.
    #[must_use]
    pub fn withdrawn(&self) -> BTreeSet<NodeId> {
        // GROUNDED over the retraction relation — the same skeptical machinery this file
        // already uses for attacks, and for the same reason.
        //
        // The previous version recomputed the set from scratch each pass, so as it grew
        // FEWER retractions stayed active and the set shrank: the opposite of monotone.
        // A retraction cycle oscillated forever, the loop exited on its bound, and the
        // answer was whichever phase the PARITY of the vault's total retraction count
        // landed on. Adding an unrelated bookkeeping note flipped a defeated claim to
        // standing. The comment claiming "a set that only grows, so it terminates" was
        // false, and it was the argument for correctness rather than a description of it.
        //
        // A retraction binds only if its author is DEFINITELY undefeated. Where
        // retractions dispute each other and none settles, none of them binds — a
        // contested withdrawal must not silently suppress an attack, which is this
        // project's rule about never making the graph quietly smaller.
        let retracts: Vec<(&NodeId, &NodeId)> = self
            .edges
            .iter()
            // `Sublates` too — its own docstring is "preserves the target while
            // SUPERSEDING it", which is the lifecycle claim `Supersedes` makes. It was
            // parsed, listed as a known kind, and read by nothing, so the identical
            // statement froze silently under one spelling and was refused under the
            // other. When you forbid a thing, sweep for the other grammars that
            // express it.
            .filter(|e| e.kind.supersedes_target())
            .map(|e| (&e.from, &e.to))
            .collect();
        if retracts.is_empty() {
            return BTreeSet::new();
        }

        let retractors_of = |x: &NodeId| -> Vec<&NodeId> {
            retracts
                .iter()
                .filter(|(_, to)| *to == x)
                .map(|(from, _)| *from)
                .collect()
        };

        // Least fixed point of "every retraction against x is itself retracted by
        // something settled". Monotone by construction: `settled` only ever grows, and
        // the sequence stabilises within one step per participant.
        let participants: BTreeSet<&NodeId> = retracts.iter().flat_map(|(f, t)| [*f, *t]).collect();
        let mut settled: BTreeSet<NodeId> = BTreeSet::new();
        loop {
            let next: BTreeSet<NodeId> = participants
                .iter()
                .filter(|x| {
                    retractors_of(x)
                        .iter()
                        .all(|r| retractors_of(r).iter().any(|rr| settled.contains(*rr)))
                })
                .map(|x| (*x).clone())
                .collect();
            if next == settled {
                break;
            }
            settled = next;
        }

        // Withdrawn: retracted by an author that is itself settled-undefeated.
        retracts
            .iter()
            .filter(|(from, _)| settled.contains(*from))
            .map(|(_, to)| (*to).clone())
            .collect()
    }

    /// Whether `id` names a node that can argue at all.
    ///
    /// A missing node is NOT an argument: a dangling attack edge must not silently
    /// confer standing on whatever it points at.
    #[must_use]
    pub fn is_argument_node(&self, id: &NodeId) -> bool {
        self.node(id).is_some_and(|n| n.kind.is_argument())
    }

    fn attackers(&self) -> BTreeMap<NodeId, Vec<NodeId>> {
        // A withdrawn claim is not a participant in the dispute. Letting it attack and
        // then be defeated would be wrong twice over: it never had standing to argue,
        // and in the other direction a withdrawn DEFENDER props up the claim it shields
        // — so a live claim would survive on the strength of an argument its own author
        // retracted. Removing it from the relation is the honest form.
        let withdrawn = self.withdrawn();

        let mut map: BTreeMap<NodeId, Vec<NodeId>> = BTreeMap::new();
        for edge in self.edges.iter().filter(|e| {
            e.kind.is_attack()
                && !withdrawn.contains(&e.from)
                // REFERENCE MATERIAL DOES NOT COMPETE. `is_argument` says so, and
                // `grounded_extension` already honours it when choosing candidates —
                // but the relation itself did not, so a term carrying `contradicts:`
                // defeated a claim no argument opposed. A rule enforced on one side of
                // a join is a rule the other side ignores.
                //
                // Silently dropping the edge would be the swallow this project forbids,
                // so `PEIR-LINT-NON-ARGUMENT-ATTACK` reports it against the node whose
                // author wrote it.
                && self.is_argument_node(&e.from)
                && self.is_argument_node(&e.to)
        }) {
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

    /// Superseding something is not retracting it, and the two obey different laws.
    ///
    /// A RETRACTION can be lifted: if the node that retracted X is itself retracted, the
    /// retraction does not bind and X stands again. That is a dialectical dispute about
    /// whether the withdrawal was right, and the fixed point models it correctly.
    ///
    /// A SUPERSESSION is a version chain, and lifting is nonsense there. v2 supersedes
    /// v1; v3 supersedes v2. v2 is now superseded — but v1 does not come back. It is
    /// superseded by a superseded version, which is still superseded. Grouping the two
    /// relations under one predicate applied retraction-lifting algebra to version
    /// chains and REVIVED the twice-superseded original, with the packet asserting "the
    /// withdrawal was lifted".
    ///
    /// A supersession stops binding only if its author was RETRACTED — withdrawn as
    /// wrong, rather than merely replaced.
    #[test]
    fn a_supersession_chain_does_not_revive_the_original() {
        use crate::{Edge, EdgeKind, NodeId};
        let mk = |id: &str| {
            crate::parse_node(&format!("---\nid: {id}\ntype: claim\ntitle: t\n---\n"))
                .expect("fixture parses")
        };
        let build = |kind: EdgeKind| {
            let mut g = Graph::new();
            for id in ["v1", "v2", "v3"] {
                g.insert_node(mk(id));
            }
            g.insert_edge(Edge::new(NodeId::new("v2"), NodeId::new("v1"), kind));
            g.insert_edge(Edge::new(NodeId::new("v3"), NodeId::new("v2"), kind));
            g.withdrawn()
        };

        for kind in [EdgeKind::Supersedes, EdgeKind::Sublates] {
            let w = build(kind);
            assert!(
                w.contains(&NodeId::new("v2")),
                "{kind:?}: v2 was replaced by v3 and is not current"
            );
            assert!(
                w.contains(&NodeId::new("v1")),
                "{kind:?}: v1 stays replaced — superseded by a superseded version is \
still superseded, and reviving it is how a retired conclusion returns"
            );
            assert!(
                !w.contains(&NodeId::new("v3")),
                "{kind:?}: v3 is the current version"
            );
        }

        // The lifting law still holds where it belongs: a RETRACTION whose author is
        // itself retracted does not bind.
        let mut g = Graph::new();
        for id in ["c1", "d1", "d2"] {
            g.insert_node(mk(id));
        }
        g.insert_edge(Edge::new(
            NodeId::new("d1"),
            NodeId::new("c1"),
            EdgeKind::Retracts,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("d2"),
            NodeId::new("d1"),
            EdgeKind::Retracts,
        ));
        let w = g.withdrawn();
        assert!(
            !w.contains(&NodeId::new("c1")),
            "control: a retracted retraction is lifted, and c1 stands"
        );

        // And a supersession made by a RETRACTED author does stop binding — the author
        // was withdrawn as wrong, not merely replaced.
        let mut g = Graph::new();
        for id in ["v1", "v2", "d1"] {
            g.insert_node(mk(id));
        }
        g.insert_edge(Edge::new(
            NodeId::new("v2"),
            NodeId::new("v1"),
            EdgeKind::Supersedes,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("d1"),
            NodeId::new("v2"),
            EdgeKind::Retracts,
        ));
        let w = g.withdrawn();
        assert!(
            !w.contains(&NodeId::new("v1")),
            "the version that replaced v1 was itself withdrawn as wrong, so v1 stands"
        );
    }

    /// Reference material does not compete.
    ///
    /// `NodeKind::is_argument` says so in as many words — "a `Term` or `Criterion` is
    /// reference material: it is *used by* arguments and never competes with them" —
    /// and `grounded_extension` honours it when choosing CANDIDATES. The attack
    /// relation did not, so a term carrying `contradicts:` defeated a claim that no
    /// argument opposed, and no gate or lint said a word about it.
    #[test]
    fn a_term_cannot_defeat_a_claim() {
        use crate::{Edge, EdgeKind, NodeId};
        let mk = |id: &str, kind: &str| {
            crate::parse_node(&format!("---\nid: {id}\ntype: {kind}\ntitle: t\n---\n"))
                .expect("fixture parses")
        };
        let build = |attacker_kind: &str| {
            let mut g = Graph::new();
            g.insert_node(mk("c1", "claim"));
            g.insert_node(mk("x1", attacker_kind));
            g.insert_edge(Edge::new(
                NodeId::new("x1"),
                NodeId::new("c1"),
                EdgeKind::Contradicts,
            ));
            g.is_grounded(&NodeId::new("c1"))
        };

        assert!(
            !build("claim"),
            "positive control: an unanswered claim-attack does defeat c1"
        );
        assert!(
            build("term"),
            "a term is reference material and cannot compete, so c1 stands"
        );
        assert!(build("criterion"), "nor can a criterion");
    }

    /// A retraction cycle must not make the answer depend on the rest of the vault.
    ///
    /// `withdrawn()` recomputed its set from scratch each pass, so it shrank as it grew
    /// and a cycle oscillated forever; the loop exited on its bound and returned
    /// whichever phase the PARITY of the vault's retraction count landed on. Adding an
    /// unrelated bookkeeping note flipped a defeated claim to standing.
    #[test]
    fn a_retraction_cycle_does_not_depend_on_unrelated_retractions() {
        use crate::{Edge, EdgeKind, NodeId};
        let mk = |id: &str| {
            crate::parse_node(&format!("---\nid: {id}\ntype: claim\ntitle: t\n---\n"))
                .expect("fixture parses")
        };
        let build = |extra: bool| {
            let mut g = Graph::new();
            for id in ["c1", "rival", "r1", "r2", "z", "d9"] {
                g.insert_node(mk(id));
            }
            let e = |f: &str, t: &str, k: EdgeKind| Edge::new(NodeId::new(f), NodeId::new(t), k);
            g.insert_edge(e("rival", "c1", EdgeKind::Contradicts));
            g.insert_edge(e("r1", "rival", EdgeKind::Retracts));
            g.insert_edge(e("r1", "r2", EdgeKind::Retracts));
            g.insert_edge(e("r2", "r1", EdgeKind::Retracts));
            if extra {
                // Entirely unrelated to the dispute.
                g.insert_edge(e("d9", "z", EdgeKind::Retracts));
            }
            g.grounded_extension().contains(&NodeId::new("c1"))
        };
        assert_eq!(
            build(false),
            build(true),
            "an unrelated retraction elsewhere in the vault changed whether this claim \
survives — the answer depended on a count, not on the dispute"
        );
        assert!(
            !build(false),
            "r1 and r2 retract each other, so neither settles and neither binds; the \
attack on c1 therefore stands. A contested withdrawal must not suppress an attack."
        );
    }

    /// A retraction is an authored assertion, not a fact about the world.
    ///
    /// Withdrawn claims were removed from the attack relation so a retracted attacker
    /// could not defeat a live claim. Right in principle, wrong in scope: the removal
    /// asked *is this withdrawn* and never *by whom, and does the withdrawal stand*. So
    /// an idle note by anyone at all flipped a defeated claim to `review_ready`, froze a
    /// packet, and the packet asserted "every attack on it is itself defeated" — while
    /// printing the withdrawn rival twice.
    ///
    /// A retraction that is ITSELF withdrawn restores what it withdrew. Anything else
    /// makes the last writer win.
    #[test]
    fn a_retraction_that_is_itself_retracted_restores_the_attack() {
        use crate::{Edge, EdgeKind, NodeId};
        let mk = |id: &str, kind: &str| {
            crate::parse_node(&format!("---\nid: {id}\ntype: {kind}\ntitle: t\n---\n"))
                .expect("fixture parses")
        };
        let mut g = Graph::new();
        for (id, k) in [
            ("c1", "claim"),
            ("rival", "claim"),
            ("d1", "dissent"),
            ("d2", "dissent"),
        ] {
            g.insert_node(mk(id, k));
        }
        let e = |f: &str, t: &str, k: EdgeKind| Edge::new(NodeId::new(f), NodeId::new(t), k);
        g.insert_edge(e("rival", "c1", EdgeKind::Contradicts));

        assert!(
            !g.grounded_extension().contains(&NodeId::new("c1")),
            "a live attack defeats it"
        );

        g.insert_edge(e("d1", "rival", EdgeKind::Retracts));
        assert!(
            g.grounded_extension().contains(&NodeId::new("c1")),
            "withdrawing the attacker lets the claim stand"
        );

        // The retraction is itself withdrawn: the attack is live again.
        g.insert_edge(e("d2", "d1", EdgeKind::Retracts));
        assert!(
            !g.grounded_extension().contains(&NodeId::new("c1")),
            "a withdrawn retraction cannot go on suppressing the attack — otherwise the \
last writer wins, and the graph gets quietly smaller with every note"
        );
    }
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
