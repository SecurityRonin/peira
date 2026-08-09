//! Court Mode — frozen, hash-verified citation packets.
//!
//! # The safe statement is generated, never authored
//!
//! The obvious design is to let someone write the courtroom sentence and then check
//! it against the graph. That check is impossible to do well: natural language
//! overstates in ways no matcher catches, and the reviewer who wrote the sentence is
//! the last person able to see it.
//!
//! So peira inverts it. The safe statement is **rendered from the graph** using
//! the 金剛經 three-moment form — 所謂 X (what is called X), 即非 X (X is not the
//! thing itself), 是名 X (it is named X under these conditions). Nobody writes the
//! sentence, so nobody can overstate it. This is 二諦 as a build step: the packet is
//! the conventional register (世俗諦), the graph with all its boundaries is the
//! ultimate one (勝義諦), and the translation only ever loses strength.
//!
//! A packet also **refuses to freeze while any gate blocks**. There is no override.

use blazehash_core::algorithm::{hash_bytes, Algorithm};
use peira_core::{EdgeKind, Graph, Node, NodeId, NodeKind};
use peira_lens::{examine_graph, lints, Violation};
use std::fmt::{self, Write as _};

/// Why a packet could not be frozen.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    /// No such claim in the vault.
    NoSuchClaim(NodeId),
    /// The subject is not a claim.
    NotAClaim {
        /// The node asked for.
        id: NodeId,
        /// What it actually is.
        kind: NodeKind,
    },
    /// Gates block. A packet may not be frozen over an unexamined claim, and there
    /// is deliberately no override.
    Blocked {
        /// The claim.
        id: NodeId,
        /// Everything standing in the way, in full.
        violations: Vec<Violation>,
    },
    /// The claim does not survive in the grounded extension.
    Defeated(NodeId),
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PacketError::NoSuchClaim(id) => write!(f, "no claim `{id}` in the vault"),
            PacketError::NotAClaim { id, kind } => {
                write!(f, "`{id}` is a {kind}, not a claim")
            }
            PacketError::Blocked { id, violations } => {
                writeln!(
                    f,
                    "`{id}` cannot be frozen — {} gate(s) block:",
                    violations.len()
                )?;
                for v in violations {
                    writeln!(f, "  {v}")?;
                }
                Ok(())
            }
            PacketError::Defeated(id) => write!(
                f,
                "`{id}` is defeated in the grounded extension — an attack on it stands unanswered"
            ),
        }
    }
}

impl std::error::Error for PacketError {}

/// A frozen citation packet.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    /// The claim this packet is about.
    pub subject: NodeId,
    /// The rendered document.
    pub body: String,
    /// SHA-256 over `body`.
    pub digest: String,
}

/// Render the three-moment safe statement for a claim.
///
/// Falls back gracefully when a claim declares no key term: the moments still hold,
/// they are simply stated of the claim itself rather than of a named term.
fn safe_statement(graph: &Graph, claim: &Node) -> String {
    let terms: Vec<&Node> = graph
        .edges_from(&claim.id)
        .filter(|e| e.kind == EdgeKind::UsesTerm)
        .filter_map(|e| graph.node(&e.to))
        .collect();

    let mut out = String::new();
    for term in terms {
        let as_used = term.field("as_used").unwrap_or("(not stated)");
        let not_essence = term.field("not_essence").unwrap_or("(not stated)");
        let stipulated = term.field("stipulated").unwrap_or("(not stated)");
        let _ = write!(
            out,
            "所謂「{name}」— what is called \"{name}\": {as_used}\n\
             即非「{name}」— but the record is not the thing: {not_essence}\n\
             是名「{name}」— so it is named \"{name}\" only as: {stipulated}\n\n",
            name = term.title
        );
    }
    out
}

/// Nodes standing in a given relation to the claim.
fn related<'a>(graph: &'a Graph, claim: &Node, kind: EdgeKind) -> Vec<&'a Node> {
    graph
        .edges_to(&claim.id)
        .filter(|e| e.kind == kind)
        .filter_map(|e| graph.node(&e.from))
        .collect()
}

fn bullet_list(nodes: &[&Node], empty: &str) -> String {
    if nodes.is_empty() {
        return format!("  {empty}\n");
    }
    nodes.iter().fold(String::new(), |mut acc, n| {
        let _ = writeln!(acc, "  - [{}] {}", n.id, n.title);
        acc
    })
}

/// What would defeat the claim, from BOTH things the falsifier gate accepts.
///
/// Stated `falsifier:` entries, and defeaters recorded as attacking nodes. A node
/// satisfying the gate has no string of its own, so it is named here rather than
/// leaving a heading over an empty list — which would read as *"nothing would"*, the
/// opposite of what the graph says.
fn defeat_block(graph: &Graph, claim: &Node) -> String {
    let mut out = String::new();
    for f in claim.field_list("falsifier") {
        let _ = writeln!(out, "  - {f}");
    }
    for e in graph.edges_to(&claim.id).filter(|e| e.kind.is_attack()) {
        if let Some(n) = graph.node(&e.from) {
            let _ = writeln!(out, "  - [{}] {} — on record as an attack", n.id, n.title);
        }
    }
    if out.is_empty() {
        // cov:unreachable: PEIR-FALSIFIER-MISSING blocks any claim with neither a
        // `falsifier:` field nor an incoming attack, and freeze refuses while any
        // gate blocks. Kept so that decoupling the two degrades into a visible
        // "(none recorded)" instead of a heading over silence.
        out.push_str("  (none recorded)\n");
    }
    out
}

/// Every violation bearing on one claim.
fn violations_for(graph: &Graph, id: &NodeId) -> Vec<Violation> {
    examine_graph(graph)
        .into_iter()
        .chain(lints::lint(graph))
        .filter(|v| &v.subject == id)
        .collect()
}

/// Freeze a packet for `id`.
///
/// # Errors
/// Refuses when the claim is missing, is not a claim, is defeated in the grounded
/// extension, or has any blocking gate. There is no override parameter, because a
/// packet that could be forced would be worth nothing in the room it is made for.
pub fn freeze(graph: &Graph, id: &NodeId) -> Result<Packet, PacketError> {
    let claim = graph
        .node(id)
        .ok_or_else(|| PacketError::NoSuchClaim(id.clone()))?;
    if claim.kind != NodeKind::Claim {
        return Err(PacketError::NotAClaim {
            id: id.clone(),
            kind: claim.kind,
        });
    }

    let violations = violations_for(graph, id);
    if !violations.is_empty() {
        return Err(PacketError::Blocked {
            id: id.clone(),
            violations,
        });
    }
    if !graph.is_grounded(id) {
        return Err(PacketError::Defeated(id.clone()));
    }

    let supports = related(graph, claim, EdgeKind::Supports);
    let contradicts = related(graph, claim, EdgeKind::Contradicts);
    let limits = related(graph, claim, EdgeKind::Limits);

    let defeat_block = defeat_block(graph, claim);

    let boundaries = claim.field_list("boundaries");
    let boundary_block = if boundaries.is_empty() {
        "  (none declared)\n".to_owned()
    } else {
        boundaries.iter().fold(String::new(), |mut acc, b| {
            let _ = writeln!(acc, "  - {b}");
            acc
        })
    };

    let body = format!(
        "# Citation packet — {id}\n\
         \n\
         ## Safe statement (世俗諦 — the conventional register)\n\
         \n\
         {statement}\
         ## Claim\n\
         \n\
         {title}\n\
         \n\
         ## Warrant\n\
         \n\
         {warrant}\n\
         \n\
         ## Supporting\n\
         \n\
         {supporting}\
         \n\
         ## Contradicting\n\
         \n\
         {contradicting}\
         \n\
         ## Limits\n\
         \n\
         {limiting}\
         \n\
         ## Boundary conditions\n\
         \n\
         {boundary_block}\
         \n\
         ## What would defeat this\n\
         \n\
         {defeat_block}\
         \n\
         ## Standing\n\
         \n\
         Survives in the grounded extension; every attack on it is itself defeated.\n\
         All enforced gates pass.\n",
        id = claim.id,
        statement = if safe_statement(graph, claim).is_empty() {
            String::new()
        } else {
            format!("{}\n", safe_statement(graph, claim).trim_end())
        },
        title = claim.title,
        warrant = claim.field("warrant").unwrap_or("(none stated)"),
        supporting = bullet_list(&supports, "(nothing)"),
        contradicting = bullet_list(&contradicts, "(nothing on record)"),
        limiting = bullet_list(&limits, "(none recorded)"),
    );

    let digest = hash_bytes(Algorithm::Sha256, body.as_bytes());
    Ok(Packet {
        subject: id.clone(),
        body,
        digest,
    })
}

/// Whether a frozen packet still matches what the vault would produce now.
///
/// Re-derives the packet and compares digests, so a mutated source node shows up as
/// a mismatch rather than as silence.
#[must_use]
pub fn verify(graph: &Graph, packet: &Packet) -> bool {
    freeze(graph, &packet.subject).is_ok_and(|fresh| fresh.digest == packet.digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::{parse_node, Edge};

    fn node(src: &str) -> Node {
        parse_node(src).expect("fixture parses")
    }

    /// A claim that clears every enforced gate.
    fn clean_graph() -> Graph {
        let mut g = Graph::new();
        g.insert_node(node(
            "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file at that path\n\
warrant: A catalogue entry evidences that the path was recorded.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
boundaries:\n  - Windows 10 1809 and later\n\
falsifier:\n  - an entry shown to be written without the path ever being present\n---\n",
        ));
        g.insert_node(node(
            "---\nid: o1\ntype: observation\ntitle: InventoryApplicationFile entry present\n\
aspect: function\n---\n",
        ));
        g.insert_node(node(
            "---\nid: 60.01\ntype: term\ntitle: presence\nas_used: the file was on the system\n\
not_essence: a catalogue record is not the file, and not its running\n\
stipulated: the OS recorded this path in Amcache\n---\n",
        ));
        g.insert_edge(Edge::new(
            NodeId::new("o1"),
            NodeId::new("c1"),
            EdgeKind::Supports,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("c1"),
            NodeId::new("60.01"),
            EdgeKind::UsesTerm,
        ));
        g
    }

    /// A packet must disclose what would defeat the claim.
    ///
    /// The gates refuse to promote a claim that records nothing which could count
    /// against it, so every packet that freezes at all has an answer. Demanding it
    /// and then withholding it from the one artifact made for people outside the
    /// vault would be the exact asymmetry this project exists to remove.
    ///
    /// Both ways of satisfying the gate must render. A falsifier recorded as an
    /// attacking NODE has no string to print, and an empty section under a heading
    /// reads as "nothing would" — the opposite of what the graph says.
    #[test]
    fn a_packet_discloses_what_would_defeat_the_claim() {
        let g = clean_graph();
        let p = freeze(&g, &NodeId::new("c1")).expect("clean claim should freeze");
        assert!(
            p.body.contains("## What would defeat this"),
            "no defeat section:\n{}",
            p.body
        );
        assert!(
            p.body
                .contains("an entry shown to be written without the path ever being present"),
            "the stated falsifier is not disclosed:\n{}",
            p.body
        );

        // Satisfied by an attack edge rather than a field. Two consequences of
        // adding the edge, both of them the gates working rather than obstacles:
        // the attacker must itself be defeated or c1 leaves the grounded extension
        // and freeze refuses; and c1 becomes CONTESTED, so 四句 now demands all four
        // corners of it.
        // Built from clean_graph so o1 and the term survive — the orphan-claim lint
        // blocks a claim with no supporting evidence, and `insert_node` replaces by
        // id, so only c1 itself is swapped: no `falsifier:` field, four corners.
        let mut g = clean_graph();
        g.insert_node(node(
            "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file at that path\n\
warrant: A catalogue entry evidences that the path was recorded.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
boundaries:\n  - Windows 10 1809 and later\n\
corners:\n  - it was catalogued\n  - it was not catalogued\n  - catalogued under one mechanism and not another\n  - the question does not arise\n---\n",
        ));
        g.insert_node(node(
            "---\nid: c2\ntype: claim\ntitle: Catalogued without the path ever being present\n\
aspect: function\n---\n",
        ));
        g.insert_node(node(
            "---\nid: c3\ntype: claim\ntitle: The write path requires the file on the volume\n\
aspect: function\n---\n",
        ));
        g.insert_edge(Edge::new(
            NodeId::new("c2"),
            NodeId::new("c1"),
            EdgeKind::Attacks,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("c3"),
            NodeId::new("c2"),
            EdgeKind::Attacks,
        ));
        let p = freeze(&g, &NodeId::new("c1")).expect("c1 survives, its attacker being defeated");
        assert!(
            p.body
                .contains("Catalogued without the path ever being present"),
            "an attack edge satisfied the gate but the attacker is not named:\n{}",
            p.body
        );
    }

    #[test]
    fn freezes_a_clean_claim_and_renders_the_three_moments() {
        let g = clean_graph();
        let p = freeze(&g, &NodeId::new("c1")).expect("clean claim should freeze");
        assert!(p.body.contains("所謂"), "{}", p.body);
        assert!(p.body.contains("即非"), "{}", p.body);
        assert!(p.body.contains("是名"), "{}", p.body);
        assert!(p.body.contains("Windows 10 1809 and later"));
        assert_eq!(p.digest.len(), 64, "sha256 hex");
    }

    #[test]
    fn refuses_to_freeze_while_a_gate_blocks() {
        let mut g = clean_graph();
        // Remove the warrant by replacing the claim.
        g.insert_node(node(
            "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file at that path\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
boundaries:\n  - Windows 10 1809 and later\n---\n",
        ));
        let err = freeze(&g, &NodeId::new("c1")).unwrap_err();
        match err {
            PacketError::Blocked { violations, .. } => {
                assert!(violations.iter().any(|v| v.gate.contains("WARRANT")));
            }
            other => panic!("expected Blocked, got {other:?}"),
        }
    }

    #[test]
    fn refuses_to_freeze_a_defeated_claim() {
        let mut g = clean_graph();
        g.insert_node(node(
            "---\nid: c2\ntype: claim\ntitle: The entry was written by the installer\n\
warrant: Installers populate the same table.\nquantifier: singular\naspect: function\n\
causal_rung: association\nboundaries:\n  - Windows 10 1809 and later\n---\n",
        ));
        g.insert_node(node(
            "---\nid: o2\ntype: observation\ntitle: installer log entry\naspect: function\n---\n",
        ));
        g.insert_edge(Edge::new(
            NodeId::new("o2"),
            NodeId::new("c2"),
            EdgeKind::Supports,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("c2"),
            NodeId::new("c1"),
            EdgeKind::Contradicts,
        ));
        // Being attacked makes c1 contested, so 四句 fires first. Answer it, so the
        // test reaches the condition it is actually about.
        g.insert_node(node(
            "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file at that path\n\
warrant: A catalogue entry evidences that the path was recorded.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
corners:\n  - catalogued\n  - not catalogued\n  - both, across boots\n  - \"neither: the table was rebuilt\"\n\
boundaries:\n  - Windows 10 1809 and later\n---\n",
        ));

        // c1 is now attacked by an undefeated c2, so it is out of the extension.
        let err = freeze(&g, &NodeId::new("c1")).unwrap_err();
        assert!(matches!(err, PacketError::Defeated(_)), "{err:?}");
    }

    #[test]
    fn verify_goes_red_when_a_source_node_is_mutated() {
        let g = clean_graph();
        let packet = freeze(&g, &NodeId::new("c1")).unwrap();
        assert!(verify(&g, &packet), "unmutated vault must verify");

        let mut tampered = clean_graph();
        tampered.insert_node(node(
            "---\nid: o1\ntype: observation\ntitle: SOMETHING ELSE ENTIRELY\naspect: function\n---\n",
        ));
        assert!(
            !verify(&tampered, &packet),
            "a mutated source node must break the packet's digest"
        );
    }

    #[test]
    fn refuses_a_subject_that_is_not_a_claim() {
        let g = clean_graph();
        let err = freeze(&g, &NodeId::new("o1")).unwrap_err();
        assert!(matches!(err, PacketError::NotAClaim { .. }), "{err:?}");
    }

    #[test]
    fn refuses_a_missing_claim_rather_than_returning_an_empty_packet() {
        let g = clean_graph();
        let err = freeze(&g, &NodeId::new("nope")).unwrap_err();
        assert!(matches!(err, PacketError::NoSuchClaim(_)), "{err:?}");
    }
}
