//! The deterministic lint pack — checks that need no model and no catalogue.
//!
//! These are cheap, run over the whole graph, and catch the defects that recur
//! whatever the domain: prose that overstates its evidence, references that go
//! nowhere, grades nobody stands behind, and corroboration counted as independence.

use crate::Violation;
use peira_core::{EdgeKind, Graph, Node, NodeId, NodeKind};

/// Prose asserting more than evidence can carry.
pub const FORBIDDEN_VERB: &str = "PEIR-LINT-FORBIDDEN-VERB";
/// An edge pointing at a node that is not in the vault.
pub const DANGLING_EDGE: &str = "PEIR-LINT-DANGLING-EDGE";
/// A claim with nothing supporting it.
pub const ORPHAN_CLAIM: &str = "PEIR-LINT-ORPHAN-CLAIM";
/// A grade nobody has put their name to.
pub const UNREVIEWED_GRADE: &str = "PEIR-LINT-UNREVIEWED-GRADE";
/// Privileged material in the open tier.
pub const PRIVILEGE_LEAK: &str = "PEIR-LINT-PRIVILEGE-LEAK";
/// Restatements counted as though they were independent lines.
pub const FALSE_INDEPENDENCE: &str = "PEIR-LINT-FALSE-INDEPENDENCE";

/// Overstatements, and what to say instead.
///
/// Taken from the expert-witness substitution table: the left column is what gets
/// written under time pressure, the right column is what the evidence actually
/// supports. A tribunal hears the difference even when the author did not.
const OVERSTATEMENTS: &[(&str, &str)] = &[
    ("proves", "establishes / provides evidence of"),
    ("proven", "evidenced"),
    ("confirms", "is consistent with"),
    ("confirmed", "consistent with"),
    ("demonstrates conclusively", "is strongly consistent with"),
    ("is consistent only with", "is strongly consistent with"),
    ("contradicted by", "is not consistent with"),
    ("conclusively", "(delete — say what the evidence shows)"),
    ("definitively", "(delete — say what the evidence shows)"),
    ("beyond doubt", "(delete — the tribunal decides doubt)"),
    ("undoubtedly", "(delete)"),
    ("clearly shows", "shows"),
];

/// Fields that must never appear in the open tier.
const PRIVILEGED_FIELDS: &[&str] = &["privilege", "client", "matter_id", "instructing_solicitor"];

fn violation(
    gate: &'static str,
    subject: &NodeId,
    detail: String,
    remedy: &'static str,
) -> Violation {
    Violation {
        gate,
        lens: "LINT",
        subject: subject.clone(),
        detail,
        remedy,
    }
}

/// Whether `haystack` contains `needle` as a whole-word phrase.
fn contains_phrase(haystack: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = haystack[from..].find(needle) {
        let start = from + rel;
        let end = start + needle.len();
        let before_ok = start == 0
            || !haystack[..start]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
        let after_ok = end == haystack.len()
            || !haystack[end..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric);
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Prose that asserts more than evidence carries.
fn forbidden_verbs(node: &Node) -> Vec<Violation> {
    let haystack = format!("{} {}", node.title, node.body).to_ascii_lowercase();
    OVERSTATEMENTS
        .iter()
        .filter(|(word, _)| contains_phrase(&haystack, word))
        .map(|(word, instead)| {
            violation(
                FORBIDDEN_VERB,
                &node.id,
                format!("says \"{word}\" — {}", node.title),
                // The remedy is a &'static str, so the substitution is carried in the
                // detail line rather than fabricated per-call.
                match *instead {
                    s if s.starts_with("(delete") => {
                        "delete the intensifier and state what the evidence shows"
                    }
                    _ => "replace with consistent-with language: an observation is never a verdict",
                },
            )
        })
        .collect()
}

/// References that go nowhere.
fn dangling_edges(graph: &Graph) -> Vec<Violation> {
    graph
        .dangling_edges()
        .into_iter()
        .map(|e| {
            let missing = if graph.node(&e.to).is_none() {
                &e.to
            } else {
                &e.from
            };
            violation(
                DANGLING_EDGE,
                &e.from,
                format!(
                    "`{}` edge points at `{missing}`, which is not in the vault",
                    e.kind
                ),
                "create the node, or correct the reference — a dangling link is a defect, \
never a silent no-op",
            )
        })
        .collect()
}

/// Claims with nothing behind them.
fn orphan_claims(graph: &Graph, node: &Node) -> Vec<Violation> {
    if node.kind != NodeKind::Claim {
        return Vec::new();
    }
    let supported = graph
        .edges_to(&node.id)
        .any(|e| e.kind == EdgeKind::Supports);
    if supported {
        return Vec::new();
    }
    vec![violation(
        ORPHAN_CLAIM,
        &node.id,
        format!("\"{}\" has no supporting evidence", node.title),
        "attach an observation, a run, or another claim — or record it as a hypothesis \
until something supports it",
    )]
}

/// Grades nobody stands behind.
fn unreviewed_grades(graph: &Graph) -> Vec<Violation> {
    graph
        .edges()
        .filter(|e| e.grade().is_none() && e.grade_proposed.is_some())
        .map(|e| {
            violation(
                UNREVIEWED_GRADE,
                &e.from,
                format!(
                    "edge {} → {} carries a proposed grade with no reviewer",
                    e.from, e.to
                ),
                "a reviewer must settle the grade; a proposal — whoever or whatever made \
it — asserts nothing",
            )
        })
        .collect()
}

/// Privileged material that has escaped into the open tier.
fn privilege_leak(node: &Node) -> Vec<Violation> {
    PRIVILEGED_FIELDS
        .iter()
        .filter(|f| node.fields.contains(f))
        .map(|f| {
            violation(
                PRIVILEGE_LEAK,
                &node.id,
                format!("open-tier node carries `{f}`"),
                "move the node to the governed tier — the membrane is one-way by design",
            )
        })
        .collect()
}

/// Restatements counted as though they were independent lines of evidence.
///
/// The manifesto's rule that independent tools are not automatically independent
/// evidence, checked structurally: if two supporters of a claim are linked by a
/// `duplicates` edge, the claim has one line of evidence written twice, and any
/// argument resting on their agreement is resting on nothing.
fn false_independence(graph: &Graph, node: &Node) -> Vec<Violation> {
    let supporters: Vec<&NodeId> = graph
        .edges_to(&node.id)
        .filter(|e| e.kind == EdgeKind::Supports)
        .map(|e| &e.from)
        .collect();

    if supporters.len() < 2 {
        return Vec::new();
    }

    let mut out = Vec::new();
    for (i, a) in supporters.iter().enumerate() {
        for b in supporters.iter().skip(i + 1) {
            let duplicated = graph.edges().any(|e| {
                e.kind == EdgeKind::Duplicates
                    && ((&e.from == *a && &e.to == *b) || (&e.from == *b && &e.to == *a))
            });
            if duplicated {
                out.push(violation(
                    FALSE_INDEPENDENCE,
                    &node.id,
                    format!(
                        "`{a}` and `{b}` both support \"{}\", but one duplicates the other",
                        node.title
                    ),
                    "count them as one line of evidence — agreement between restatements \
is not corroboration",
                ));
            }
        }
    }
    out
}

/// Run every lint over the graph.
#[must_use]
pub fn lint(graph: &Graph) -> Vec<Violation> {
    let mut out = dangling_edges(graph);
    out.extend(unreviewed_grades(graph));
    for node in graph.nodes() {
        out.extend(forbidden_verbs(node));
        out.extend(orphan_claims(graph, node));
        out.extend(privilege_leak(node));
        out.extend(false_independence(graph, node));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::{parse_node, Edge, Grade};

    fn node(src: &str) -> Node {
        parse_node(src).expect("fixture parses")
    }

    fn graph_of(nodes: Vec<Node>, edges: Vec<Edge>) -> Graph {
        let mut g = Graph::new();
        for n in nodes {
            g.insert_node(n);
        }
        for e in edges {
            g.insert_edge(e);
        }
        g
    }

    fn codes(v: &[Violation]) -> Vec<&str> {
        v.iter().map(|x| x.gate).collect()
    }

    #[test]
    fn catches_proves_and_offers_the_substitution() {
        let n = node("---\nid: c1\ntype: claim\ntitle: This entry proves execution\n---\n");
        let v = forbidden_verbs(&n);
        assert_eq!(codes(&v), vec![FORBIDDEN_VERB]);
        assert!(v[0].detail.contains("proves"), "{}", v[0].detail);
        assert!(v[0].remedy.contains("consistent-with"), "{}", v[0].remedy);
    }

    #[test]
    fn does_not_fire_on_a_word_that_merely_contains_a_forbidden_one() {
        // "approves" contains "proves"; "disproven" contains "proven".
        let n = node(
            "---\nid: c1\ntype: claim\ntitle: The reviewer approves the disproven theory\n---\n",
        );
        assert!(
            forbidden_verbs(&n).is_empty(),
            "substring matching would make this lint unusable"
        );
    }

    #[test]
    fn catches_consistent_only_with() {
        let n = node(
            "---\nid: c1\ntype: claim\ntitle: t\n---\n\nThe pattern is consistent only with deliberate staging.\n",
        );
        assert_eq!(codes(&forbidden_verbs(&n)), vec![FORBIDDEN_VERB]);
    }

    #[test]
    fn a_descriptive_claim_trips_nothing() {
        let n = node(
            "---\nid: c1\ntype: claim\ntitle: The hive recorded the path\n---\n\nIt is consistent with presence.\n",
        );
        assert!(forbidden_verbs(&n).is_empty());
    }

    #[test]
    fn a_dangling_edge_is_reported_with_the_missing_target() {
        let c = node("---\nid: c1\ntype: claim\ntitle: t\nsupports: [ghost]\n---\n");
        let g = graph_of(
            vec![c],
            vec![Edge::new(
                NodeId::new("c1"),
                NodeId::new("ghost"),
                EdgeKind::Supports,
            )],
        );
        let v = dangling_edges(&g);
        assert_eq!(codes(&v), vec![DANGLING_EDGE]);
        assert!(v[0].detail.contains("ghost"), "{}", v[0].detail);
    }

    #[test]
    fn an_unsupported_claim_is_an_orphan() {
        let c = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let g = graph_of(vec![c.clone()], vec![]);
        assert_eq!(codes(&orphan_claims(&g, &c)), vec![ORPHAN_CLAIM]);
    }

    #[test]
    fn a_supported_claim_is_not_an_orphan() {
        let c = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let o = node("---\nid: o1\ntype: observation\ntitle: o\n---\n");
        let g = graph_of(
            vec![c.clone(), o],
            vec![Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )],
        );
        assert!(orphan_claims(&g, &c).is_empty());
    }

    #[test]
    fn a_proposed_grade_with_no_reviewer_is_flagged() {
        let g = graph_of(
            vec![],
            vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                    .proposing(Grade::G3),
            ],
        );
        assert_eq!(codes(&unreviewed_grades(&g)), vec![UNREVIEWED_GRADE]);
    }

    #[test]
    fn a_settled_grade_is_not_flagged() {
        let g = graph_of(
            vec![],
            vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                    .graded_by(Grade::G3, "albert"),
            ],
        );
        assert!(unreviewed_grades(&g).is_empty());
    }

    #[test]
    fn privileged_fields_may_not_appear_in_the_open_tier() {
        let n = node("---\nid: c1\ntype: claim\ntitle: t\nclient: Northgate\n---\n");
        let v = privilege_leak(&n);
        assert_eq!(codes(&v), vec![PRIVILEGE_LEAK]);
        assert!(v[0].detail.contains("client"), "{}", v[0].detail);
    }

    #[test]
    fn two_supporters_that_duplicate_each_other_are_one_line_of_evidence() {
        // The independence trap: two parsers vendoring the same decoder agree, and
        // the agreement is counted as corroboration.
        let claim = node("---\nid: c1\ntype: claim\ntitle: the path was catalogued\n---\n");
        let parser_a = node("---\nid: p1\ntype: observation\ntitle: parser A output\n---\n");
        let parser_b = node("---\nid: p2\ntype: observation\ntitle: parser B output\n---\n");
        let graph = graph_of(
            vec![claim.clone(), parser_a, parser_b],
            vec![
                Edge::new(NodeId::new("p1"), NodeId::new("c1"), EdgeKind::Supports),
                Edge::new(NodeId::new("p2"), NodeId::new("c1"), EdgeKind::Supports),
                Edge::new(NodeId::new("p1"), NodeId::new("p2"), EdgeKind::Duplicates),
            ],
        );
        let found = false_independence(&graph, &claim);
        assert_eq!(codes(&found), vec![FALSE_INDEPENDENCE]);
        assert!(found[0].detail.contains("p1"), "{}", found[0].detail);
        assert!(found[0].detail.contains("p2"), "{}", found[0].detail);
    }

    #[test]
    fn genuinely_independent_supporters_are_not_flagged() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let parser_a = node("---\nid: p1\ntype: observation\ntitle: a\n---\n");
        let parser_b = node("---\nid: p2\ntype: observation\ntitle: b\n---\n");
        let graph = graph_of(
            vec![claim.clone(), parser_a, parser_b],
            vec![
                Edge::new(NodeId::new("p1"), NodeId::new("c1"), EdgeKind::Supports),
                Edge::new(NodeId::new("p2"), NodeId::new("c1"), EdgeKind::Supports),
            ],
        );
        assert!(false_independence(&graph, &claim).is_empty());
    }
}
