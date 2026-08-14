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
use std::collections::BTreeSet;
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

/// What `verify` concluded about a stored packet.
///
/// A `bool` collapsed four different situations into `false`, and only one of them is
/// an accusation. "Your evidence was altered" and "a gate was added since this was
/// frozen" are not the same sentence to say to someone holding a packet.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verification {
    /// Re-derived byte-identically from the vault as it stands.
    Verified,
    /// The vault changed under the packet. This is the one that is an accusation.
    DigestMismatch {
        /// What the packet carries.
        stored: String,
        /// What the vault produces now.
        fresh: String,
    },
    /// Written by a different renderer, so no comparison against it is meaningful.
    FormatSuperseded {
        /// The format the packet declares.
        stored: u32,
        /// The format this build renders.
        current: u32,
    },
    /// The claim no longer freezes at all — a gate now blocks it, or it has since been
    /// defeated. Nothing is wrong with the packet; the claim stopped qualifying.
    NoLongerFreezable(PacketError),
}

impl Verification {
    /// Whether the packet still stands.
    #[must_use]
    pub fn is_verified(&self) -> bool {
        matches!(self, Verification::Verified)
    }
}

/// The packet body format this build renders, and the only one it can verify.
///
/// Declared INSIDE the hashed body. A version beside the digest rather than under it
/// is one an adversary rewrites at will, so the packet would assert a format without
/// that assertion being covered by the hash it is checked against.
///
/// Bumping this invalidates every packet frozen before the bump — deliberately, and
/// visibly, which is the whole point. A body change that did not bump it would be the
/// silent case this exists to remove.
pub const PACKET_FORMAT: u32 = 1;

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

impl Packet {
    /// A packet read back from storage, digested from the bytes exactly as stored.
    ///
    /// The counterpart to [`freeze`]: `freeze` renders and hashes, this takes bytes
    /// somebody kept and hashes them the same way, so [`verify`] compares like with
    /// like. Without it a caller outside this crate cannot present a stored packet at
    /// all — `Packet` is `#[non_exhaustive]` — and would have to re-implement the
    /// comparison, which is how a checker and the thing it checks drift apart.
    #[must_use]
    pub fn from_stored(subject: NodeId, body: String) -> Self {
        let digest = hash_bytes(Algorithm::Sha256, body.as_bytes());
        Self {
            subject,
            body,
            digest,
        }
    }
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
/// Render the packet body for a claim — the exact bytes `freeze` would seal.
///
/// Extracted so the same text can be SCANNED before anyone commits to sealing it.
/// While this lived inline in `freeze`, the overstatement check ran where only
/// `freeze` could see it, and `peira status` contradicted the packet command.
fn render_body(graph: &Graph, id: &NodeId) -> Option<String> {
    let claim = graph.node(id)?;

    // An attack REMOVED because it was withdrawn was not DEFEATED, and saying so would
    // be the packet's own overstatement. Disclose it: an idle note by anyone at all can
    // withdraw a rival, and a reader is entitled to know that is why nothing stands
    // against this claim.
    let withdrawn_attacks: Vec<&Node> = graph
        .edges_to(id)
        .filter(|e| e.kind.is_attack())
        .filter(|e| {
            graph
                .edges_to(&e.from)
                .any(|r| matches!(r.kind, EdgeKind::Retracts | EdgeKind::Supersedes))
        })
        .filter_map(|e| graph.node(&e.from))
        .collect();

    let standing = if withdrawn_attacks.is_empty() {
        "Survives in the grounded extension; every attack on it is itself defeated.".to_owned()
    } else {
        let names = withdrawn_attacks
            .iter()
            .map(|n| format!("[{}] {}", n.id, n.title))
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Survives in the grounded extension — but NOT because every attack was \
answered. {} attack(s) were WITHDRAWN by a retraction rather than defeated on the \
merits: {names}. Read the retraction before relying on this.",
            withdrawn_attacks.len()
        )
    };
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
         Packet format: {format}\n\
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
         {standing}\n\
         All enforced gates pass.\n",
        id = claim.id,
        format = PACKET_FORMAT,
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

    // Scan what will actually be SEALED, not a list of fields somebody remembered to
    // enumerate. The lint pack's rendered-field list was already out of step with this
    // renderer on the day it was written: `warrant`, `boundaries` and `falsifier` are
    // rendered verbatim and were scanned by nothing. Reading the finished body makes
    // "rendered but unscanned" impossible rather than merely enumerable.
    Some(body)
}

/// Everything standing in the way of freezing a packet for `id`.
///
/// Public because the CLI must ask the SAME question rather than re-deriving a
/// narrower one: `peira status` printed "all enforced gates pass" over claims court
/// refused, because it filtered findings to the claim's own id while this walks the
/// evidential closure. Two implementations of one question is how a checker and the
/// thing it checks drift apart.
#[must_use]
pub fn violations_for(graph: &Graph, id: &NodeId) -> Vec<Violation> {
    // The EVIDENTIAL CLOSURE: the claim, everything it rests on transitively, and
    // everything it renders. Filtering to the claim's own id made every check evadable
    // by one hop — put the overstatement, the ungraded G4 and the missing warrant on a
    // supporting claim, and a "clean" claim froze on top of it while the vault reported
    // seven findings nobody's packet had to answer for.
    //
    // A packet asserts that what it rests on was examined. That assertion is only true
    // if the examination follows the support.
    // Deliberately does NOT include rivals and limiters. Their TITLES are printed, and
    // the body scan answers for that text — but their own gate profile is their
    // author's business. Pulling them in blocks the victim of someone else's
    // frontmatter, which is the defect this project already fixed once in the pramāṇa
    // gate. What is rendered is scanned; what is rested on is examined.
    let mut closure: BTreeSet<NodeId> = BTreeSet::new();
    let mut stack = vec![id.clone()];
    while let Some(n) = stack.pop() {
        if !closure.insert(n.clone()) {
            continue;
        }
        // Backwards along support: what this rests on.
        for e in graph.edges_to(&n).filter(|e| e.kind == EdgeKind::Supports) {
            stack.push(e.from.clone());
        }
        // And FORWARDS along `depends_on`, which is documented as "the target must hold
        // for the source to" — a stronger relation than support, and one that had no
        // consumer outside `core`. Every walk here followed `Supports`, so a claim whose
        // own frontmatter said it could not hold without another froze cleanly while the
        // vault recorded that other as withdrawn. A declared prerequisite is
        // load-bearing by definition; nothing else in the file needed to change.
        for e in graph
            .edges_from(&n)
            .filter(|e| e.kind == EdgeKind::DependsOn)
        {
            stack.push(e.to.clone());
        }
        // Forwards to what a packet renders of it: the stipulated terms.
        for e in graph
            .edges_from(&n)
            .filter(|e| e.kind == EdgeKind::UsesTerm)
        {
            stack.push(e.to.clone());
        }
    }

    let mut found: Vec<Violation> = examine_graph(graph)
        .into_iter()
        .chain(lints::lint(graph))
        .filter(|v| closure.contains(&v.subject))
        .collect();

    // Scan what WOULD be sealed. This lived inside `freeze` and so was invisible to
    // `peira status`, which then printed "all enforced gates pass" over a claim the
    // packet command refused. A check only one caller can see is a check the other
    // caller contradicts.
    if let Some(body) = render_body(graph, id) {
        // Same exclusion as in `freeze`: the falsifier section discloses, it does not assert.
        let asserted: String = body
            .split("\n## ")
            .filter(|s| !s.starts_with("What would defeat this"))
            .collect::<Vec<_>>()
            .join("\n## ");
        found.extend(lints::prose_findings_in(&asserted, id));
    }
    found
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

    // The SUBJECT's own withdrawal. `retracted` reports only where a withdrawn node
    // holds something else up, so retained history stays quiet — which leaves the case
    // where the retired claim IS the packet.
    if let Some(v) = lints::subject_withdrawn(graph, id) {
        return Err(PacketError::Blocked {
            id: id.clone(),
            violations: vec![v],
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

    let body = render_body(graph, id).ok_or_else(|| PacketError::NoSuchClaim(id.clone()))?;
    // Scan what the packet ASSERTS, not what it discloses. The falsifier section exists
    // to name what would defeat the claim, so scanning it refuses a packet for making
    // the disclosure the tool demands — see `legal_conclusions`. Overstatement in a
    // falsifier is still caught by the node-level lint on title and body.
    let asserted: String = body
        .split("\n## ")
        .filter(|s| !s.starts_with("What would defeat this"))
        .collect::<Vec<_>>()
        .join("\n## ");
    let overstated = lints::prose_findings_in(&asserted, id);
    if !overstated.is_empty() {
        return Err(PacketError::Blocked {
            id: id.clone(),
            violations: overstated,
        });
    }
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
pub fn verify(graph: &Graph, packet: &Packet) -> Verification {
    // Format first, and deliberately before touching the graph. It is a property of
    // the stored artifact alone, and once it differs, a digest comparison is
    // guaranteed to differ too — reporting THAT would be true and useless, and would
    // read as an accusation.
    let stored = declared_format(&packet.body);
    if stored != PACKET_FORMAT {
        return Verification::FormatSuperseded {
            stored,
            current: PACKET_FORMAT,
        };
    }

    match freeze(graph, &packet.subject) {
        Ok(fresh) if fresh.digest == packet.digest => Verification::Verified,
        Ok(fresh) => Verification::DigestMismatch {
            stored: packet.digest.clone(),
            fresh: fresh.digest,
        },
        // Not tampering: the claim stopped qualifying. The packet is untouched and the
        // error says which gate or defeat is responsible, so it is carried rather than
        // flattened.
        Err(e) => Verification::NoLongerFreezable(e),
    }
}

/// The format a stored packet declares.
///
/// A body with no declaration was rendered before formats were declared at all, and
/// reads as 0 — which is never [`PACKET_FORMAT`], so it is reported as superseded
/// rather than compared. Scans lines rather than assuming a position, because the
/// stored body is untrusted input: it is whatever is on disk, not what we rendered.
fn declared_format(body: &str) -> u32 {
    body.lines()
        .find_map(|l| l.strip_prefix("Packet format: "))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::{parse_node, Edge, Grade, Pramana};

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
        // Graded and attributed: a packet may not rest on evidence nobody weighed,
        // and this fixture stands for a claim that clears every check.
        g.insert_edge(
            Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                .graded_by(Grade::G2, "a-reviewer")
                .via(Pramana::Perception),
        );
        g.insert_edge(Edge::new(
            NodeId::new("c1"),
            NodeId::new("60.01"),
            EdgeKind::UsesTerm,
        ));
        g
    }

    /// `verify` must say WHY, not just no.
    ///
    /// Four situations, and a `bool` renders them identically. Only one of them —
    /// `DigestMismatch` — is an accusation; reporting the others in the same breath
    /// tells whoever holds the packet that their evidence was altered when it was not.
    #[test]
    fn verify_says_why_it_failed() {
        let g = clean_graph();
        let packet = freeze(&g, &NodeId::new("c1")).expect("clean claim should freeze");
        assert_eq!(verify(&g, &packet), Verification::Verified);

        // Frozen by an older renderer: its body honestly declares format 0 and its
        // digest honestly covers that body. Nothing is wrong with it — it is simply
        // not comparable against what this build renders.
        let stale = Packet::from_stored(
            packet.subject.clone(),
            packet.body.replace("Packet format: 1", "Packet format: 0"),
        );
        assert_eq!(
            verify(&g, &stale),
            Verification::FormatSuperseded {
                stored: 0,
                current: PACKET_FORMAT
            },
            "a packet from an older renderer must not be reported as tampering"
        );

        // The claim stopped qualifying: the falsifier is gone, so a gate blocks and
        // the packet cannot be re-derived at all. Also not tampering.
        let mut regressed = clean_graph();
        regressed.insert_node(node(
            "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file at that path\n\
warrant: A catalogue entry evidences that the path was recorded.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
boundaries:\n  - Windows 10 1809 and later\n---\n",
        ));
        assert!(
            matches!(
                verify(&regressed, &packet),
                Verification::NoLongerFreezable(PacketError::Blocked { .. })
            ),
            "a claim a gate now blocks must not be reported as tampering, got {:?}",
            verify(&regressed, &packet)
        );

        // And the one that IS an accusation still reads as one.
        let mut tampered = clean_graph();
        tampered.insert_node(node(
            "---\nid: o1\ntype: observation\ntitle: SOMETHING ELSE ENTIRELY\n\
aspect: function\n---\n",
        ));
        assert!(
            matches!(
                verify(&tampered, &packet),
                Verification::DigestMismatch { .. }
            ),
            "a mutated cited node must still read as a mismatch"
        );
    }

    /// A declared prerequisite is load-bearing by definition.
    ///
    /// `DependsOn` is documented as "the target must hold for the source to" — a
    /// STRONGER relation than `Supports` — and had no consumer outside `core`. Every
    /// walk in this file followed `Supports`, so a claim whose own frontmatter says it
    /// cannot hold without c2 froze cleanly while the vault recorded c2 as withdrawn.
    #[test]
    fn a_packet_answers_for_the_prerequisites_it_declares() {
        let mut g = clean_graph();
        g.insert_node(node(
            "---\nid: c2\ntype: claim\ntitle: The prerequisite finding\n---\n",
        ));
        g.insert_node(node("---\nid: d1\ntype: dissent\ntitle: withdrawn\n---\n"));
        g.insert_edge(Edge::new(
            NodeId::new("c1"),
            NodeId::new("c2"),
            EdgeKind::DependsOn,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("d1"),
            NodeId::new("c2"),
            EdgeKind::Retracts,
        ));
        let err = freeze(&g, &NodeId::new("c1"))
            .expect_err("a claim may not freeze over a prerequisite the record withdraws");
        assert!(
            err.to_string().contains("c2"),
            "the refusal must name the prerequisite: {err}"
        );
    }

    /// The closure covers everything the packet RENDERS, not only what it rests on.
    ///
    /// `freeze` renders nodes reached by `Contradicts` and `Limits` under their own
    /// headings, and the closure followed only `Supports` and `UsesTerm` — so a legal
    /// conclusion on a limiter printed verbatim into a court artifact while `peira lint`
    /// reported it happily elsewhere.
    #[test]
    fn a_packet_answers_for_the_limiters_and_rivals_it_prints() {
        let mut g = clean_graph();
        g.insert_node(node(
            "---\nid: lim\ntype: claim\ntitle: The suspect is guilty of unauthorised access\n---\n",
        ));
        g.insert_edge(Edge::new(
            NodeId::new("lim"),
            NodeId::new("c1"),
            EdgeKind::Limits,
        ));
        let err = freeze(&g, &NodeId::new("c1"))
            .expect_err("a packet rendering a limiter that decides the ultimate issue must refuse");
        assert!(
            err.to_string().contains("guilty"),
            "the refusal must quote what would have been printed: {err}"
        );
    }

    /// A packet answers for the prose it renders.
    ///
    /// The lint reaching a term's `stipulated:` was necessary and not sufficient: the
    /// finding lands on the TERM, and `violations_for` filtered to the claim's own id,
    /// so the overstatement was reported and frozen into the packet anyway. Detecting
    /// is not withholding.
    #[test]
    fn a_packet_refuses_an_overstatement_in_the_prose_it_renders() {
        let mut g = clean_graph();
        g.insert_node(node(
            "---\nid: 60.01\ntype: term\ntitle: presence\n\
as_used: the file was on the system\n\
not_essence: a catalogue record is not the file\n\
stipulated: the entry proves the file was executed\n---\n",
        ));
        let err = freeze(&g, &NodeId::new("c1"))
            .expect_err("a packet rendering an overstated term must be refused");
        assert!(
            err.to_string().contains("proves"),
            "the refusal must quote the offending word: {err}"
        );
    }

    /// A packet must not rest on ungraded evidence.
    ///
    /// `Grade` and `Pramana` are stored inseparably from the grader, and the ceiling
    /// gate caps what a means of knowing can carry — but nothing required a support
    /// edge to be graded at all. An ungraded, unattributed edge supported promotion
    /// exactly as well as reviewed direct perception, so the whole grading apparatus
    /// was inert unless an author volunteered into it.
    ///
    /// Scoped to freezing on purpose. Ordinary vault work stays ungraded; the demand
    /// arrives when a claim is about to become a court artifact.
    #[test]
    fn a_packet_refuses_evidence_that_was_never_graded() {
        // Strip the grade from the otherwise-clean fixture: everything else about
        // this claim is in order, so the only thing under test is the missing grade.
        let mut g = Graph::new();
        for n in ["c1", "o1", "60.01"] {
            if let Some(node) = clean_graph().node(&NodeId::new(n)) {
                g.insert_node(node.clone());
            }
        }
        g.insert_edge(Edge::new(
            NodeId::new("c1"),
            NodeId::new("60.01"),
            EdgeKind::UsesTerm,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("o1"),
            NodeId::new("c1"),
            EdgeKind::Supports,
        ));
        let err = freeze(&g, &NodeId::new("c1"))
            .expect_err("a packet resting on an ungraded support edge must be refused");
        let rendered = err.to_string();
        assert!(
            rendered.contains("o1") && rendered.to_lowercase().contains("grade"),
            "the refusal must name the ungraded edge and say what is missing: {rendered}"
        );

        // Graded and attributed, within its ceiling: freezes.
        let g = clean_graph();
        assert!(
            freeze(&g, &NodeId::new("c1")).is_ok(),
            "a graded, attributed support edge within its ceiling must freeze"
        );
    }

    /// A packet must declare the format it was written in.
    ///
    /// Inside the hashed body, never beside it: a version a tamperer can edit without
    /// changing the digest proves nothing. Being inside means it costs a digest change
    /// to introduce, which is why it lands with one rather than on its own.
    ///
    /// It sits below the title because `peira verify` reads the subject from line 1.
    #[test]
    fn a_packet_declares_its_format_version() {
        let g = clean_graph();
        let p = freeze(&g, &NodeId::new("c1")).expect("clean claim should freeze");
        assert!(
            p.body.contains("Packet format: 1"),
            "no format declaration:\n{}",
            p.body
        );
        assert!(
            p.body.starts_with("# Citation packet — c1\n"),
            "the format line must not displace the title `peira verify` parses:\n{}",
            p.body
        );
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
        assert!(
            verify(&g, &packet).is_verified(),
            "unmutated vault must verify"
        );

        let mut tampered = clean_graph();
        tampered.insert_node(node(
            "---\nid: o1\ntype: observation\ntitle: SOMETHING ELSE ENTIRELY\naspect: function\n---\n",
        ));
        assert!(
            !verify(&tampered, &packet).is_verified(),
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
