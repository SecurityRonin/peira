//! The deterministic lint pack — checks that need no model and no catalogue.
//!
//! These are cheap, run over the whole graph, and catch the defects that recur
//! whatever the domain: prose that overstates its evidence, references that go
//! nowhere, grades nobody stands behind, and corroboration counted as independence.

use crate::Violation;
use peira_core::{EdgeKind, Graph, Node, NodeId, NodeKind};
use std::collections::BTreeSet;

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
/// A grade settled by the author of the very claim it grades.
pub const SELF_GRADED: &str = "PEIR-LINT-SELF-GRADED";
/// An onset that is only where somebody started looking.
pub const WINDOW_EDGE_AS_ONSET: &str = "PEIR-LINT-WINDOW-EDGE-AS-ONSET";
/// A claim whose support never reaches anything that touched the world.
pub const UNGROUNDED_CHAIN: &str = "PEIR-LINT-UNGROUNDED-CHAIN";

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

/// Claims resting on claims, all the way down.
///
/// [`orphan_claims`] checks depth 1 and accepts another claim as support. This walks
/// the support subtree and asks whether any path reaches something that touched the
/// world — an `Observation` or a `Run`. A claim that never does is standing on
/// narrative: its credibility is inherited from the story around it rather than from
/// anything observed.
///
/// `Term`, `Criterion` and `Protocol` do not ground anything. A stipulated meaning, a
/// declared standard and an unexecuted procedure are all reference material; only a
/// `Run` of that procedure, or an `Observation`, is contact with the world.
///
/// A claim with NO support is left to [`orphan_claims`] — reporting both would say the
/// same thing twice in different words.
///
/// The visited set is what makes a cycle terminate, and a cycle is itself the finding:
/// claims that support each other and nothing else are grounded in nothing.
fn ungrounded_chains(graph: &Graph, node: &Node) -> Vec<Violation> {
    if node.kind != NodeKind::Claim {
        return Vec::new();
    }

    let supporters = |id: &NodeId| -> Vec<NodeId> {
        graph
            .edges_to(id)
            .filter(|e| e.kind == EdgeKind::Supports)
            .map(|e| e.from.clone())
            .collect()
    };

    let direct = supporters(&node.id);
    if direct.is_empty() {
        return Vec::new();
    }

    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    let mut stack = direct;
    let mut depth = 0usize;
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        depth += 1;
        if let Some(n) = graph.node(&id) {
            if matches!(n.kind, NodeKind::Observation | NodeKind::Run) {
                return Vec::new();
            }
        }
        stack.extend(supporters(&id));
    }

    vec![violation(
        UNGROUNDED_CHAIN,
        &node.id,
        format!(
            "\"{}\" is supported through {depth} node(s), none of which is an observation or a run",
            node.title
        ),
        "attach evidence that touched the world — an observation, or a run of a \
protocol; inference resting on inference is a claim standing on its own narrative",
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

/// Findings signed off by their own author.
///
/// The sibling of [`unreviewed_grades`], one field further along: that one catches a
/// grade nobody stands behind, this one catches a grade the claimant stands behind
/// alone. Everything it needs is already in the graph — a settled grade is stored
/// inseparably from its grader, and `author:` is an ordinary frontmatter key.
///
/// Attributed to the claim rather than to the evidence: "the author of a finding" is
/// the author of the finding, and that is what a reader needs named.
///
/// A claim declaring no author is left alone. There is nothing to compare, and
/// guessing — from git blame, from the last editor — would put a name to a sign-off
/// nobody gave.
fn self_graded(graph: &Graph) -> Vec<Violation> {
    graph
        .edges()
        .filter_map(|e| {
            let grader = e.grader()?;
            let subject = graph.node(&e.to)?;
            let author = subject.field("author")?;
            if author != grader {
                return None;
            }
            Some(violation(
                SELF_GRADED,
                &e.to,
                format!(
                    "the grade on {} → {} was settled by `{grader}`, who authored \"{}\"",
                    e.from, e.to, subject.title
                ),
                "an independent reviewer must settle it — an author signing off their \
own finding is the one signature that carries no information",
            ))
        })
        .collect()
}

/// An onset read off the edge of the window somebody happened to query.
///
/// A windowed query returns a true fact *about the window*. Its edge is not the start
/// of a behaviour — but the earliest record in a collected range is exactly what gets
/// written up as "when it began", and the coincidence is invisible once the window is
/// no longer in front of you.
///
/// Distinct from the boundary gate, which asks *where* a claim holds. This asks
/// whether its ONSET is an artefact of where the looking started.
///
/// The comparison is string equality, never a date parse. Ordering dates would make
/// this lint's correctness depend on a date format the vault has never promised;
/// comparing opaque strings cannot be wrong about an order it never computes.
///
/// Conservative by construction: it fires only when EVERY supporter declaring a window
/// began at the asserted onset. One supporter that looked elsewhere means the onset
/// sits inside a window rather than on its edge. It will therefore miss cases — which
/// is the right way round, because a lint that cries wolf gets switched off, and a
/// lint that is switched off catches nothing.
fn window_edge_as_onset(graph: &Graph, node: &Node) -> Vec<Violation> {
    let Some(onset) = node.field("onset") else {
        return Vec::new();
    };
    let windows: Vec<&str> = graph
        .edges_to(&node.id)
        .filter(|e| e.kind == EdgeKind::Supports)
        .filter_map(|e| graph.node(&e.from))
        .filter_map(|n| n.field("window_from"))
        .collect();

    if windows.is_empty() || windows.iter().any(|w| *w != onset) {
        return Vec::new();
    }

    vec![violation(
        WINDOW_EDGE_AS_ONSET,
        &node.id,
        format!(
            "onset `{onset}` is also where every one of the {} supporting window(s) began looking",
            windows.len()
        ),
        "query the full history, then window it — and when an onset coincides with the \
edge of the search, widen before writing",
    )]
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
    out.extend(self_graded(graph));
    for node in graph.nodes() {
        out.extend(forbidden_verbs(node));
        out.extend(orphan_claims(graph, node));
        out.extend(privilege_leak(node));
        out.extend(false_independence(graph, node));
        out.extend(window_edge_as_onset(graph, node));
        out.extend(ungrounded_chains(graph, node));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::{parse_node, Edge, EdgeKind, Grade, NodeId};

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

    /// A retracted claim must not pass silently.
    ///
    /// The parser refuses `status: withdrawn` with a principled message; a
    /// `retracts:` edge says exactly that, is parsed, is recorded — and is read by
    /// nothing. A packet freezes for a claim the vault itself records as withdrawn,
    /// and never mentions the retraction. A refusal enforced at one entry point is a
    /// convention, not an impossibility.
    ///
    /// Deliberately NOT an attack edge: a retraction is a lifecycle fact, not a
    /// dialectical move, and making it one would let a claim defeat its own
    /// withdrawal by counter-argument.
    #[test]
    fn a_retracted_claim_is_flagged() {
        let retracted = |g: &Graph, subject: &str| {
            lint(g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-RETRACTED" && v.subject.as_str() == subject)
                .count()
        };
        let claim = |id: &str| node(&format!("---\nid: {id}\ntype: claim\ntitle: t\n---\n"));
        let obs = node("---\nid: o1\ntype: observation\ntitle: o\n---\n");
        let supports =
            |f: &str, t: &str| Edge::new(NodeId::new(f), NodeId::new(t), EdgeKind::Supports);

        let withdrawn = graph_of(
            vec![claim("c1"), obs.clone(), node("---\nid: d1\ntype: dissent\ntitle: withdrawn after the parser was found wrong\n---\n")],
            vec![
                supports("o1", "c1"),
                Edge::new(NodeId::new("d1"), NodeId::new("c1"), EdgeKind::Retracts),
            ],
        );
        assert_eq!(
            retracted(&withdrawn, "c1"),
            1,
            "a claim the vault records as retracted must be flagged"
        );

        let superseded = graph_of(
            vec![claim("c1"), claim("c2"), obs.clone()],
            vec![
                supports("o1", "c1"),
                Edge::new(NodeId::new("c2"), NodeId::new("c1"), EdgeKind::Supersedes),
            ],
        );
        assert_eq!(
            retracted(&superseded, "c1"),
            1,
            "a superseded claim is the same shape: the record says a newer version replaces it"
        );

        let live = graph_of(vec![claim("c1"), obs], vec![supports("o1", "c1")]);
        assert_eq!(
            retracted(&live, "c1"),
            0,
            "a claim nothing withdraws must not be flagged"
        );
    }

    /// A claim's support must reach the world, not just more claims.
    ///
    /// `orphan_claims` checks depth 1 and accepts another claim as support, so a tower
    /// of claims resting on claims — grounded in nothing that ever touched the world —
    /// passes every other check. That is a claim standing on narrative.
    ///
    /// A cycle is itself the finding, and must terminate rather than hang.
    #[test]
    fn a_claim_supported_only_by_other_claims_is_flagged() {
        let ungrounded = |g: &Graph, subject: &str| {
            lint(g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-UNGROUNDED-CHAIN" && v.subject.as_str() == subject)
                .count()
        };
        let claim = |id: &str| node(&format!("---\nid: {id}\ntype: claim\ntitle: t\n---\n"));
        let obs = |id: &str| {
            node(&format!(
                "---\nid: {id}\ntype: observation\ntitle: o\n---\n"
            ))
        };
        let supports = |from: &str, to: &str| {
            Edge::new(NodeId::new(from), NodeId::new(to), EdgeKind::Supports)
        };

        // c1 <- c2 <- c3, and c3 rests on nothing. Every link is inference.
        let tower = graph_of(
            vec![claim("c1"), claim("c2"), claim("c3")],
            vec![supports("c2", "c1"), supports("c3", "c2")],
        );
        assert_eq!(
            ungrounded(&tower, "c1"),
            1,
            "a claim whose whole support subtree is claims must be flagged"
        );

        // Grounded directly.
        let direct = graph_of(vec![claim("c1"), obs("o1")], vec![supports("o1", "c1")]);
        assert_eq!(ungrounded(&direct, "c1"), 0, "an observation grounds it");

        // Grounded through a chain — the walk must not stop at depth 1.
        let chain = graph_of(
            vec![claim("c1"), claim("c2"), obs("o1")],
            vec![supports("c2", "c1"), supports("o1", "c2")],
        );
        assert_eq!(
            ungrounded(&chain, "c1"),
            0,
            "reaching the world through an intermediate claim still grounds it"
        );

        // Mutual support, grounded in nothing. Must terminate AND flag.
        let cycle = graph_of(
            vec![claim("c1"), claim("c2")],
            vec![supports("c2", "c1"), supports("c1", "c2")],
        );
        assert_eq!(
            ungrounded(&cycle, "c1"),
            1,
            "claims supporting each other are grounded in nothing"
        );

        // No support at all is the orphan lint's business, not this one.
        let orphan = graph_of(vec![claim("c1")], vec![]);
        assert_eq!(
            ungrounded(&orphan, "c1"),
            0,
            "an unsupported claim is PEIR-LINT-ORPHAN-CLAIM, and must not be double-reported"
        );
    }

    /// A window's edge is not the start of a behaviour.
    ///
    /// The check is string equality between the claim's `onset:` and its supporters'
    /// `window_from:` — no date parsing, so no date-format assumption and no way for
    /// the lint to be wrong about ordering. Deliberately conservative: if any
    /// supporter looked at a different window it stays quiet, because the onset then
    /// sits inside a window rather than on its edge.
    #[test]
    fn an_onset_sitting_on_the_only_window_edge_is_flagged() {
        let findings = |claim: &str, windows: &[&str]| {
            let mut nodes = vec![node(&format!(
                "---\nid: c1\ntype: claim\ntitle: t\n{claim}---\n"
            ))];
            let mut edges = Vec::new();
            for (i, w) in windows.iter().enumerate() {
                nodes.push(node(&format!(
                    "---\nid: o{i}\ntype: observation\ntitle: o\n{w}---\n"
                )));
                edges.push(Edge::new(
                    NodeId::new(format!("o{i}")),
                    NodeId::new("c1"),
                    EdgeKind::Supports,
                ));
            }
            lint(&graph_of(nodes, edges))
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-WINDOW-EDGE-AS-ONSET")
                .count()
        };

        assert_eq!(
            findings(
                "onset: 2026-01-01\n",
                &["window_from: 2026-01-01\n", "window_from: 2026-01-01\n"]
            ),
            1,
            "every supporter began looking exactly when the behaviour supposedly began"
        );
        assert_eq!(
            findings(
                "onset: 2026-01-01\n",
                &["window_from: 2026-01-01\n", "window_from: 2025-06-01\n"]
            ),
            0,
            "a supporter looked at a different window, so the onset is not the edge"
        );
        assert_eq!(
            findings("", &["window_from: 2026-01-01\n"]),
            0,
            "no onset declared, nothing to compare"
        );
        assert_eq!(
            findings("onset: 2026-01-01\n", &["title_only: x\n"]),
            0,
            "no supporter declares a window, nothing to compare"
        );
    }

    /// The author of a finding must not issue its own sign-off.
    ///
    /// Written against the public `lint` output rather than a private predicate, so it
    /// compiles before the lint exists and fails because the check does not happen —
    /// not because a symbol is missing.
    ///
    /// Everything this needs is already in the graph: a settled grade is stored
    /// inseparably from its grader, and `author:` rides in `Fields` like any other
    /// frontmatter key. Nothing compares the two.
    #[test]
    fn a_grade_set_by_the_claims_own_author_is_flagged() {
        let self_graded = |author: &str, grader: &str| {
            let g = graph_of(
                vec![
                    node(&format!(
                        "---\nid: c1\ntype: claim\ntitle: t\nauthor: {author}\n---\n"
                    )),
                    node("---\nid: o1\ntype: observation\ntitle: o\n---\n"),
                ],
                vec![
                    Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                        .graded_by(Grade::G3, grader),
                ],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-SELF-GRADED")
                .count()
        };

        assert_eq!(
            self_graded("albert", "albert"),
            1,
            "a claim whose own author settled the grade supporting it must be flagged"
        );
        assert_eq!(
            self_graded("albert", "someone-else"),
            0,
            "an independent reviewer is the whole point — it must not be flagged"
        );

        // No `author:` declared: nothing to compare, and inventing an answer would be
        // worse than staying quiet. The claim is caught by other lints, not this one.
        let undeclared = graph_of(
            vec![
                node("---\nid: c1\ntype: claim\ntitle: t\n---\n"),
                node("---\nid: o1\ntype: observation\ntitle: o\n---\n"),
            ],
            vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                    .graded_by(Grade::G3, "albert"),
            ],
        );
        assert_eq!(
            lint(&undeclared)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-SELF-GRADED")
                .count(),
            0,
            "with no author declared there is nothing to compare"
        );
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
