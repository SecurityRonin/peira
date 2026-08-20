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
    /// The vault no longer renders this packet.
    ///
    /// **Not by itself an accusation**, and an earlier version of this comment said it
    /// was. A vault that GREW — a second corroborating observation added months later —
    /// and a vault whose cited evidence was ALTERED produce the same verdict, and only
    /// one of them is misconduct. The tool cannot tell them apart, so it reports the
    /// difference and names where it starts; the reader judges.
    DigestMismatch {
        /// What the packet carries.
        stored: String,
        /// What the vault produces now.
        fresh: String,
        /// The first line at which the two renderings diverge, if any — evidence a
        /// reader can act on rather than a bare verdict.
        first_difference: Option<String>,
    },
    /// Written by a different renderer, so no comparison against it is meaningful.
    ///
    /// **Reached only when the difference is more than the format line.** A packet whose
    /// format number is the SOLE difference from the current rendering has been edited —
    /// an older renderer could not have produced a body identical to a newer one's — and
    /// that is reported as [`Verification::DigestMismatch`] instead.
    FormatSuperseded {
        /// The format the packet declares.
        stored: u32,
        /// The format this build renders.
        current: u32,
        /// Whether the body matches the current rendering once the format line alone is
        /// normalised. Always false here; `true` is the edit case and does not reach
        /// this variant. Carried so a caller can state what is and is not known rather
        /// than inferring it.
        body_matches: bool,
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
/// visibly, which is the whole point.
///
/// **2** — the body gained a "Provenance of the grading" section disclosing that `by=`
/// attributions are self-declared. Without the bump every packet frozen under format 1
/// would report `DigestMismatch`, which is an accusation, when the truth is that the
/// renderer changed underneath them. A body change that did not bump it would be the
/// silent case this exists to remove.
///
/// **3** — stated falsifiers gained the `FALSIFIER_FRAME` prefix, so a defeat line
/// keeps its conditional sense once quoted away from the heading.
pub const PACKET_FORMAT: u32 = 3;

/// The prefix carried by every STATED falsifier in a packet's defeat section.
///
/// A heading frames the lines beneath it only while they stay beneath it. The one
/// place a falsifier is read is the place it has been lifted out of — a submission, a
/// letter, a slide — and there the heading is gone while the sentence remains.
///
/// Public because a reader parsing packets needs to know which prefix is the renderer's
/// and which words are the author's.
pub const FALSIFIER_FRAME: &str = "Would defeat this claim:";

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
        let as_used = quote_authored(term.field("as_used").unwrap_or("(not stated)"));
        let not_essence = quote_authored(term.field("not_essence").unwrap_or("(not stated)"));
        let stipulated = quote_authored(term.field("stipulated").unwrap_or("(not stated)"));
        let _ = write!(
            out,
            "所謂「{name}」— what is called \"{name}\": {as_used}\n\
             即非「{name}」— but the record is not the thing: {not_essence}\n\
             是名「{name}」— so it is named \"{name}\" only as: {stipulated}\n\n",
            name = quote_authored(&term.title)
        );
    }
    out
}

/// Nodes standing in a given relation to the claim.
fn related<'a>(graph: &'a Graph, claim: &Node, kind: EdgeKind) -> Vec<&'a Node> {
    // DEDUPLICATED. A vault may declare the same edge twice — `supports: ["c1 ...",
    // "c1 ..."]` — and the graph records both faithfully, which is correct. But a packet
    // that lists the same supporter twice reads as two independent lines of evidence,
    // and independence is the thing this whole tool exists to keep honest.
    let mut seen = BTreeSet::new();
    graph
        .edges_to(&claim.id)
        .filter(|e| e.kind == kind)
        .filter(|e| seen.insert(e.from.clone()))
        .filter_map(|e| graph.node(&e.from))
        .collect()
}

fn bullet_list(nodes: &[&Node], empty: &str) -> String {
    if nodes.is_empty() {
        return format!("  {empty}\n");
    }
    nodes.iter().fold(String::new(), |mut acc, n| {
        let _ = writeln!(acc, "  - [{}] {}", n.id, foreign_title(n));
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
        // The frame travels with the line. See `FALSIFIER_FRAME`: what a reader
        // finally reads is the sentence somebody lifted OUT of this section.
        let _ = writeln!(out, "  - {FALSIFIER_FRAME} {}", quote_authored(f));
    }
    for e in graph.edges_to(&claim.id).filter(|e| e.kind.is_attack()) {
        if let Some(n) = graph.node(&e.from) {
            // A rival's title is ANOTHER AUTHOR'S ASSERTION, rendered verbatim. Blocking
            // this claim over it would punish the victim for prose they cannot edit —
            // the defect this project fixed twice. But shipping a rival's verdict inside
            // your own packet unremarked is not acceptable either, so it is DISCLOSED:
            // the reader is told the phrase is quoted and flagged, and `peira lint`
            // reports it against the node whose author can fix it.
            // WITHHELD, not annotated. This used to quote a flagged rival with a note
            // saying it was not adopted — but the packet is what a tribunal reads, and
            // an ultimate-issue verdict printed inside it is in front of them however
            // it is captioned. `foreign_title` names the node and points at the lint.
            let _ = writeln!(
                out,
                "  - [{}] {} — on record as an attack",
                n.id,
                foreign_title(n)
            );
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
/// Authored text, prevented from impersonating packet structure.
///
/// The packet's sections are its own; an authored field that begins a line with `#`
/// manufactures one. That was not cosmetic: the body scan skips the falsifier section
/// by heading text, so injecting `## What would defeat this` into a `warrant:` moved a
/// legal conclusion into a region nothing reads, and it sealed.
///
/// Escaping the delimiter fixes the CLASS. Subtracting a section from rendered text
/// asks "which part of this is not an assertion", and the answer was authored by the
/// same person the check exists to constrain.
fn quote_authored(s: &str) -> String {
    s.lines()
        .map(|l| {
            if l.trim_start().starts_with('#') {
                format!("> {l}")
            } else {
                l.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Another author's title, rendered so it cannot punish the node quoting it.
///
/// A rival's, limiter's or contradictor's prose is written by someone the subject
/// cannot edit. Rendering it raw put it in the region `freeze` scans, so the subject
/// was refused for words that were never theirs — and the finding landed on the wrong
/// author, who could do nothing about it.
///
/// Disclosure over refusal, the same answer this file already reached twice: name the
/// node, withhold the flagged words, and point at the lint that reports them against
/// the author who can fix them. Withholding the words is not hiding the node.
///
/// ONE function, because this was fixed in `defeat_block`, fixed again in
/// `standing_line`, and arrived a third time through the `## Contradicting` section.
fn foreign_title(n: &Node) -> String {
    if lints::prose_findings_in(&n.title, &n.id).is_empty() {
        quote_authored(&n.title)
    } else {
        format!(
            "(title withheld — its own prose is flagged; see `peira lint {}`)",
            n.id
        )
    }
}

/// The attacks on `id` that were WITHDRAWN rather than answered.
///
/// Public because three places asked this and two asked it wrongly. `Graph::withdrawn()`
/// is a fixed point: a retraction that has itself been retracted does not bind, so the
/// attack it named is live again. A direct-edge test — "does an incoming retraction
/// exist" — counts it as withdrawn anyway, and that is what `peira status` did while
/// this file did the other thing, on the same vault, in the same run.
///
/// One implementation, so there is nothing left to disagree with.
#[must_use]
pub fn withdrawn_attacks<'g>(graph: &'g Graph, id: &NodeId) -> Vec<&'g Node> {
    let withdrawn = graph.withdrawn();
    graph
        .edges_to(id)
        .filter(|e| e.kind.is_attack())
        .filter(|e| withdrawn.contains(&e.from))
        .filter_map(|e| graph.node(&e.from))
        .collect()
}

/// The standing line, and what it must not claim.
///
/// An attack REMOVED because it was withdrawn was not DEFEATED, and saying so would be
/// the packet's own overstatement. Extracted from `render_body` when that function
/// outgrew its line budget — a coherent unit rather than an arbitrary cut, since every
/// line here answers one question: on what basis does this claim still stand?
fn standing_line(graph: &Graph, id: &NodeId) -> String {
    // An attack REMOVED because it was withdrawn was not DEFEATED, and saying so would
    // be the packet's own overstatement. Disclose it: an idle note by anyone at all can
    // withdraw a rival, and a reader is entitled to know that is why nothing stands
    // against this claim.
    // `Graph::withdrawn()`, not a direct-edge test. A retraction that has itself been
    // retracted does not bind, so the attacker is LIVE — and asking only "does an
    // incoming retraction exist" counted it as withdrawn anyway. That is the
    // non-monotone defect fixed in `core` and never propagated here; a fix that does not
    // reach its copies is barely a fix.
    let withdrawn = graph.withdrawn();
    let withdrawn_attacks = withdrawn_attacks(graph, id);

    // The SUBJECT's own lifecycle. A claim withdrawn and then restored froze into a
    // packet mentioning neither event, though both are material to anyone weighing it —
    // and the packet already discloses the symmetric fact about attacks. An attack
    // removed is not an attack answered; by the same argument, a claim restored is not a
    // claim never doubted.
    let lifted: Vec<String> = graph
        .edges_to(id)
        .filter(|r| r.kind.supersedes_target())
        .filter(|e| withdrawn.contains(&e.from))
        .map(|e| {
            let by = graph
                .edges_to(&e.from)
                .filter(|r| r.kind.supersedes_target())
                .map(|r| r.from.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}] (itself withdrawn by {by})", e.from)
        })
        .collect();

    let history = if lifted.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nThis claim was WITHDRAWN and later restored: {} no longer binds. It stands \
because the withdrawal was lifted, not because it was never doubted — read the record before \
relying on it.",
            lifted.join("; ")
        )
    };

    if withdrawn_attacks.is_empty() {
        return format!(
            "Survives in the grounded extension; every attack on it is itself defeated.{history}"
        );
    }
    {
        // A withdrawn rival's title is ANOTHER AUTHOR'S prose, quoted so the reader knows
        // what was withdrawn. Rendering it raw put it in the scanned region, so a
        // verdict-titled rival blocked the claim it had attacked — the victim punished
        // for words they cannot edit, and the exact mirror of the live-attacker case
        // already handled in `defeat_block`. Same content must not change outcome by
        // lifecycle state.
        let names = withdrawn_attacks
            .iter()
            .map(|n| format!("[{}] {}", n.id, foreign_title(n)))
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "Survives in the grounded extension — but NOT because every attack was \
answered. {} attack(s) were WITHDRAWN by a retraction rather than defeated on the \
merits: {names}. Read the retraction before relying on this.{history}",
            withdrawn_attacks.len()
        )
    }
}

fn render_body(graph: &Graph, id: &NodeId) -> Option<String> {
    let claim = graph.node(id)?;

    // WHO IS CREDITED, and the limit on that credit. `by=` is a free string: peira has
    // no way to check that the named reviewer settled the grade, because the gates are
    // pure functions of the graph and cannot consult the version control that would
    // answer it. The packet used to rest on graded evidence and say nothing about the
    // grading at all — asserting that gates passed while the review they relied on was
    // unverifiable.
    //
    // Disclosure, as everywhere else peira cannot establish something. Naming the
    // reviewers is what makes it actionable: a reader who knows the matter can tell
    // whether the person credited plausibly did the work.
    // BOTH DIRECTIONS, for the same reason the ceiling reads both: `X depends_on Y`
    // points FROM the dependant, so a prerequisite's grader sits on an OUTGOING edge.
    // Reading only incoming credited nobody for the strongest evidence relation there
    // is — the packet named its graders and silently omitted one class of them.
    let mut graders: Vec<String> = graph
        .edges_to(id)
        .filter(|e| e.kind == EdgeKind::Supports)
        .chain(
            graph
                .edges_from(id)
                .filter(|e| e.kind == EdgeKind::DependsOn),
        )
        .filter_map(|e| e.grader().map(std::string::ToString::to_string))
        .collect();
    graders.sort_unstable();
    graders.dedup();
    let provenance = if graders.is_empty() {
        String::new()
    } else {
        format!(
            "\n## Provenance of the grading\n\
             \n\
             Credited: {}\n\
             \n\
             These attributions are SELF-DECLARED and not authenticated. peira records who a\n\
             grade says it came from; it cannot establish that they settled it. Who wrote an\n\
             edge is a question for the version-control history of the vault.\n",
            graders.join(", ")
        )
    };

    let standing = standing_line(graph, id);

    let supports = related(graph, claim, EdgeKind::Supports);
    let contradicts = related(graph, claim, EdgeKind::Contradicts);
    let limits = related(graph, claim, EdgeKind::Limits);

    let defeat_block = defeat_block(graph, claim);

    let boundaries = claim.field_list("boundaries");
    let boundary_block = if boundaries.is_empty() {
        "  (none declared)\n".to_owned()
    } else {
        boundaries.iter().fold(String::new(), |mut acc, b| {
            let _ = writeln!(acc, "  - {}", quote_authored(b));
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
         All enforced gates pass.\n\
         {provenance}",
        id = claim.id,
        format = PACKET_FORMAT,
        statement = if safe_statement(graph, claim).is_empty() {
            String::new()
        } else {
            format!("{}\n", safe_statement(graph, claim).trim_end())
        },
        title = quote_authored(&claim.title),
        warrant = quote_authored(claim.field("warrant").unwrap_or("(none stated)")),
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

/// Every node whose examination this claim's packet asserts.
///
/// Public because the CLI scoped by the claim's OWN id — the narrowing this walk was
/// written to replace — so `peira gates --node X` answered "nothing to report" over a
/// claim `peira status X` reported blocking. Three callers, one question.
#[must_use]
pub fn evidential_closure(graph: &Graph, id: &NodeId) -> BTreeSet<NodeId> {
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
        // AND ALONG THE DEFENCE RELATION. `freeze` refuses a claim defeated in the
        // grounded extension, so grounding decides whether a packet exists at all —
        // yet the attack relation that decides grounding was never walked. Any live
        // rival could be neutralised by one unexamined line, and the packet then said
        // "every attack on it is itself defeated" on the strength of it.
        //
        // DEFENDERS ONLY, and the asymmetry is the whole point. A node that defeats an
        // attacker is holding this claim up, exactly as a supporter does. An ATTACKER
        // is opposition: pulling it in would refuse the subject for prose and
        // frontmatter another author wrote and they cannot edit — the defect fixed in
        // `foreign_title`. What holds this up is examined; what opposes it is not.
        //
        // Only defenders that are actually IN the extension count. One that is itself
        // defeated is doing no work, so demanding it be groomed would block a packet
        // over a node that changed nothing.
        for a in graph.edges_to(&n).filter(|e| e.kind.is_attack()) {
            for d in graph.edges_to(&a.from).filter(|e| e.kind.is_attack()) {
                if graph.is_grounded(&d.from) {
                    stack.push(d.from.clone());
                }
            }
        }
        // Forwards to what a packet renders of it: the stipulated terms.
        for e in graph
            .edges_from(&n)
            .filter(|e| e.kind == EdgeKind::UsesTerm)
        {
            stack.push(e.to.clone());
        }
    }

    // The SUBJECT's own withdrawal belongs HERE, not in `freeze`. While it sat there,
    // `peira status` — which calls this — reported `review_ready` over a claim the
    // packet command refused, which is precisely the drift this function was made
    // public to delete.
    closure
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
    let closure = evidential_closure(graph, id);
    let mut found: Vec<Violation> = lints::subject_withdrawn(graph, id)
        .into_iter()
        .chain(
            examine_graph(graph)
                .into_iter()
                .chain(lints::lint(graph))
                .filter(|v| closure.contains(&v.subject)),
        )
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
    // the disclosure the tool demands — see `legal_conclusions`.
    //
    // AN EARLIER VERSION OF THIS COMMENT CLAIMED A BACKSTOP THAT DOES NOT EXIST: it said
    // overstatement in a falsifier is "still caught by the node-level lint on title and
    // body". `falsifier:` is a frontmatter field, so it is neither. Nothing checks it,
    // and that is the accepted cost of not punishing the disclosure.
    //
    // WHAT THE COST IS NOW, exactly: an author can still seal any sentence here
    // unexamined. What changed is that the sentence can no longer be mistaken for a
    // finding by a reader who meets it alone — `FALSIFIER_FRAME` rides on the line
    // itself rather than on the heading above it. Unscanned, but not unframed.
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
        // NORMALISE THE FORMAT LINE AND RE-COMPARE. If the body becomes byte-identical
        // to the current rendering once only its format number is corrected, then the
        // format line is the sole difference — and a packet written by an older renderer
        // cannot be byte-identical to a newer one's output. That is an edit, provably,
        // and calling it "no verdict" is how the one accusatory outcome became the one
        // an adversary could opt out of.
        //
        // Where the body differs beyond that line, peira genuinely cannot tell staleness
        // from alteration: the information is not in the artifact. It says so.
        let normalised = packet.body.replace(
            &format!("Packet format: {stored}"),
            &format!("Packet format: {PACKET_FORMAT}"),
        );
        let body_matches = freeze(graph, &packet.subject).is_ok_and(|fresh| {
            hash_bytes(Algorithm::Sha256, normalised.as_bytes()) == fresh.digest
        });

        if !body_matches {
            return Verification::FormatSuperseded {
                stored,
                current: PACKET_FORMAT,
                body_matches,
            };
        }
        // Fall through: the format line was edited on an otherwise-current packet.
    }

    match freeze(graph, &packet.subject) {
        Ok(fresh) if fresh.digest == packet.digest => Verification::Verified,
        Ok(fresh) => {
            // Name where the two renderings diverge. A bare "does not match" is a
            // verdict; the first differing line is evidence, and it is usually enough
            // to tell a vault that grew from one whose cited evidence was altered.
            let first_difference = packet
                .body
                .lines()
                .zip(fresh.body.lines())
                .find(|(a, b)| a != b)
                .map(|(a, b)| format!("stored: {a}\n  fresh:  {b}"))
                .or_else(|| {
                    let (s, f) = (packet.body.lines().count(), fresh.body.lines().count());
                    (s != f)
                        .then(|| format!("the packet has {s} line(s); the vault now renders {f}"))
                });
            Verification::DigestMismatch {
                stored: packet.digest.clone(),
                fresh: fresh.digest,
                first_difference,
            }
        }
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

    /// A defender that has answered for itself.
    ///
    /// Bare one-line nodes used to be adequate here because nothing examined a
    /// defender — which is the defect `a_claim_answers_for_the_defender_it_stands_on`
    /// closes. A fixture that would not survive the rule it is exercising tests
    /// nothing, so this builds the grooming every such node now owes.
    fn groomed(id: &str, title: &str) -> String {
        format!(
            "---\nid: {id}\ntype: claim\ntitle: {title}\nwarrant: Stated so the defender answers for itself, as any claim must.\nquantifier: singular\naspect: function\ncausal_rung: association\nno_terms_of_art: true\nboundaries:\n  - Windows 10 1809 and later\ncorners:\n  - it holds\n  - it does not hold\n  - it holds in part\n  - the question does not arise\nfalsifier:\n  - evidence that the account it rules out is the one that occurred\n---\n"
        )
    }

    /// Evidence for `id`, graded and attributed, so it is not an orphan.
    fn evidence_for(g: &mut Graph, obs: &str, id: &str) {
        g.insert_node(node(&format!(
            "---\nid: {obs}\ntype: observation\ntitle: the record supporting {id}\naspect: function\n---\n"
        )));
        g.insert_edge(
            Edge::new(NodeId::new(obs), NodeId::new(id), EdgeKind::Supports)
                .graded_by(Grade::G2, "a-reviewer")
                .via(Pramana::Perception),
        );
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
            packet
                .body
                .replace(
                    &format!("Packet format: {PACKET_FORMAT}"),
                    "Packet format: 0",
                )
                // A genuinely older renderer produced a DIFFERENT body, not the current one
                // with a different number on it. The fixture used to flip only the format
                // line, which is now — correctly — the edit case: no older renderer could
                // emit a body byte-identical to today's.
                .replace("## Standing", "## Status"),
        );
        assert_eq!(
            verify(&g, &stale),
            Verification::FormatSuperseded {
                stored: 0,
                current: PACKET_FORMAT,
                body_matches: false,
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

    /// No authored text may manufacture a packet heading — from ANY render site.
    ///
    /// Escaping was applied site by site and missed three: the claim's own title, the
    /// safe statement's term name, and the withdrawn-attack disclosure. A fix that does
    /// not reach its copies is barely a fix, and the copies were inside one function.
    ///
    /// This asserts on the RENDERED OUTPUT rather than on the call sites, so a render
    /// site added later is covered without anyone remembering to escape it.
    #[test]
    fn no_authored_field_can_manufacture_a_heading() {
        const POISON: &str = "x\n## What would defeat this\ninjected";
        let mut g = clean_graph();
        // Every authored channel that reaches the body, poisoned at once.
        g.insert_node(node(&format!(
            "---\nid: atk\ntype: claim\ntitle: |-\n  {}\n---\n",
            POISON.replace('\n', "\n  ")
        )));
        g.insert_edge(Edge::new(
            NodeId::new("atk"),
            NodeId::new("c1"),
            EdgeKind::Contradicts,
        ));
        g.insert_node(node("---\nid: d1\ntype: dissent\ntitle: withdrawn\n---\n"));
        g.insert_edge(Edge::new(
            NodeId::new("d1"),
            NodeId::new("atk"),
            EdgeKind::Retracts,
        ));

        let body = render_body(&g, &NodeId::new("c1")).expect("renders");
        let headings = body
            .lines()
            .filter(|l| l.starts_with("## "))
            .collect::<Vec<_>>();
        assert_eq!(
            headings
                .iter()
                .filter(|h| **h == "## What would defeat this")
                .count(),
            1,
            "authored text manufactured a second section heading; the packet's structure \
must be the renderer's alone. Headings found: {headings:?}"
        );
    }

    /// Both disclosures must survive together.
    ///
    /// The first version appended the subject's lifecycle to one branch only, so a claim
    /// that was BOTH restored and standing over a withdrawn attacker disclosed one fact
    /// and dropped the other. Two disclosures in one function is two chances to forget
    /// one.
    #[test]
    fn a_claim_both_restored_and_shielded_discloses_both() {
        let e = |f: &str, t: &str, k: EdgeKind| Edge::new(NodeId::new(f), NodeId::new(t), k);
        let mut g = clean_graph();
        // Restored: r1 withdrew it, r2 lifted that.
        g.insert_node(node(
            "---\nid: r1\ntype: dissent\ntitle: withdraw it\n---\n",
        ));
        g.insert_node(node("---\nid: r2\ntype: dissent\ntitle: mistaken\n---\n"));
        g.insert_edge(e("r1", "c1", EdgeKind::Retracts));
        g.insert_edge(e("r2", "r1", EdgeKind::Retracts));
        // Shielded: a rival exists and was withdrawn.
        g.insert_node(node("---\nid: atk\ntype: claim\ntitle: a rival\n---\n"));
        g.insert_node(node(
            "---\nid: d1\ntype: dissent\ntitle: withdraw the rival\n---\n",
        ));
        g.insert_edge(e("atk", "c1", EdgeKind::Contradicts));
        g.insert_edge(e("d1", "atk", EdgeKind::Retracts));

        let line = standing_line(&g, &NodeId::new("c1"));
        assert!(
            line.contains("WITHDRAWN by a retraction"),
            "the withdrawn attack must be disclosed: {line}"
        );
        assert!(
            line.contains("later restored"),
            "the subject's own restoration must be disclosed in the SAME line: {line}"
        );
    }

    /// A format-line edit must not launder tampering into "no verdict".
    ///
    /// `verify` checked the declared format FIRST and returned before comparing digests,
    /// so editing one line — the format number — converted "this artifact no longer
    /// matches the record" into "this build cannot check this artifact". The one
    /// accusatory verdict was the one an adversary could opt out of. Five audit rounds
    /// raised it; every proposed fix was worse than the disclosure, because a genuinely
    /// old packet also mismatches and reporting THAT would be a false accusation.
    ///
    /// There is a discriminator nobody used: normalise the format line and re-compare.
    /// If a stored packet becomes byte-identical to the current rendering once only its
    /// format number is corrected, the format line is the SOLE difference — and a packet
    /// written by an older renderer cannot be byte-identical to a newer one's output.
    /// That is an edit, provably, and it is reported as one.
    #[test]
    fn a_format_line_edit_is_reported_as_an_edit_not_as_staleness() {
        let g = clean_graph();
        let fresh = freeze(&g, &NodeId::new("c1")).expect("freezes");

        // Flip only the format number on an otherwise-current packet.
        let tampered = Packet::from_stored(
            fresh.subject.clone(),
            fresh.body.replace(
                &format!("Packet format: {PACKET_FORMAT}"),
                "Packet format: 1",
            ),
        );
        assert!(
            matches!(verify(&g, &tampered), Verification::DigestMismatch { .. }),
            "the format line was the only thing changed on a current packet — an older \
renderer could not have produced this body, so it is an edit and must be named one"
        );

        // A packet that genuinely differs elsewhere as well: peira cannot tell staleness
        // from alteration, and must say so rather than pick one.
        let older = Packet::from_stored(
            fresh.subject.clone(),
            fresh
                .body
                .replace(
                    &format!("Packet format: {PACKET_FORMAT}"),
                    "Packet format: 1",
                )
                .replace("## Standing", "## Standing (old renderer)"),
        );
        match verify(&g, &older) {
            Verification::FormatSuperseded { body_matches, .. } => assert!(
                !body_matches,
                "the body differs beyond the format line, so this is the undecidable case"
            ),
            other => panic!("expected FormatSuperseded for a genuinely older body: {other:?}"),
        }
    }

    /// A packet must not present unverifiable review as though it were established.
    ///
    /// `by=` is a free string. Anyone able to write the vault can attribute a grade to
    /// anyone, and nothing in peira checks it — the gates are pure functions of the
    /// graph, so they cannot consult git, and the packet renders no grades at all.
    ///
    /// The consequence is not that a forged name reaches a tribunal; it is that the
    /// packet asserts every gate passed while the review those gates relied on is
    /// unverifiable, and says nothing about it. peira's answer everywhere else — a
    /// withdrawn attack, a rival's prose, a restored claim — is to DISCLOSE what it
    /// cannot establish.
    #[test]
    fn a_packet_discloses_that_its_grading_is_self_declared() {
        let g = clean_graph();
        let body = render_body(&g, &NodeId::new("c1")).expect("renders");
        assert!(
            body.to_lowercase().contains("self-declared")
                || body.to_lowercase().contains("not authenticated"),
            "the packet rests on graded evidence and must say the attribution is the \
author's own, not something peira verified:\n{body}"
        );
        // The reviewer named on the edge must appear, so the disclosure is actionable
        // rather than generic.
        assert!(
            body.contains("a-reviewer"),
            "naming who is credited is what makes the disclosure checkable by a reader"
        );
    }

    /// A claim restored after withdrawal must say so, and a restored ATTACK must count.
    ///
    /// Two halves of one omission. `standing_line` disclosed withdrawn attacks and said
    /// nothing about the subject's own lifecycle — so a claim that was withdrawn and then
    /// un-withdrawn froze into a packet mentioning neither event, though both are
    /// material to anyone weighing it.
    ///
    /// And its withdrawn-attack test was a DIRECT EDGE test rather than
    /// `Graph::withdrawn()`, so a retraction that had itself been retracted still counted
    /// the attacker as withdrawn — the non-monotone defect fixed in `core` and never
    /// propagated here. A fix that does not reach its copies is barely a fix.
    #[test]
    fn the_standing_line_discloses_the_subjects_own_lifecycle() {
        let e = |f: &str, t: &str, k: EdgeKind| Edge::new(NodeId::new(f), NodeId::new(t), k);

        // The subject was withdrawn, and the withdrawal was itself withdrawn.
        let mut g = clean_graph();
        g.insert_node(node(
            "---\nid: r1\ntype: dissent\ntitle: withdraw it\n---\n",
        ));
        g.insert_node(node(
            "---\nid: r2\ntype: dissent\ntitle: that was mistaken\n---\n",
        ));
        g.insert_edge(e("r1", "c1", EdgeKind::Retracts));
        g.insert_edge(e("r2", "r1", EdgeKind::Retracts));

        let body = render_body(&g, &NodeId::new("c1")).expect("renders");
        assert!(
            body.contains("r1"),
            "a claim restored after withdrawal must name the retraction that was lifted; \
the packet said nothing about either event:\n{body}"
        );

        // An attacker whose retraction was itself retracted is LIVE again, so it must
        // not appear as a withdrawn attack.
        let mut g2 = clean_graph();
        g2.insert_node(node("---\nid: atk\ntype: claim\ntitle: a rival\n---\n"));
        g2.insert_node(node(
            "---\nid: d1\ntype: dissent\ntitle: withdraw the rival\n---\n",
        ));
        g2.insert_node(node(
            "---\nid: d2\ntype: dissent\ntitle: no, it stands\n---\n",
        ));
        g2.insert_edge(e("atk", "c1", EdgeKind::Contradicts));
        g2.insert_edge(e("d1", "atk", EdgeKind::Retracts));
        g2.insert_edge(e("d2", "d1", EdgeKind::Retracts));

        let line = standing_line(&g2, &NodeId::new("c1"));
        assert!(
            !line.contains("WITHDRAWN by a retraction"),
            "d1 is itself retracted, so the attack is live and must not be reported as \
withdrawn — this used a direct-edge test instead of the fixed point: {line}"
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
        // GROOMED. The prerequisite used to be a bare `title:` line, so it drew eight
        // unrelated findings and `freeze` refused for those instead — the test named
        // withdrawal and measured grooming, and no withdrawal finding was raised at
        // all. It passed for four audit rounds while the behaviour it names did not
        // occur. The control below is the point: without the retraction this freezes.
        let groomed = "---\nid: c2\ntype: claim\ntitle: The prerequisite finding\n\
warrant: The register records the entry, and licenses nothing beyond that.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
no_terms_of_art: true\n\
boundaries:\n  - Windows 10 1809 and later\n\
falsifier:\n  - a register shown to record entries that were never written\n---\n";

        let mut clean = clean_graph();
        clean.insert_node(node(groomed));
        clean.insert_node(node(
            "---\nid: o2\ntype: observation\ntitle: the register entry is present\n\
aspect: function\n---\n",
        ));
        clean.insert_edge(
            Edge::new(NodeId::new("o2"), NodeId::new("c2"), EdgeKind::Supports)
                .graded_by(Grade::G2, "a-reviewer")
                .via(Pramana::Perception),
        );
        clean.insert_edge(
            Edge::new(NodeId::new("c1"), NodeId::new("c2"), EdgeKind::DependsOn)
                .graded_by(Grade::G2, "a-reviewer")
                .via(Pramana::Inference),
        );
        assert!(
            freeze(&clean, &NodeId::new("c1")).is_ok(),
            "control: with the prerequisite groomed and NOT withdrawn, c1 must freeze — \
otherwise the assertion below measures grooming, which is how this test passed before: {:?}",
            freeze(&clean, &NodeId::new("c1")).err()
        );

        let mut g = clean;
        g.insert_node(node("---\nid: d1\ntype: dissent\ntitle: withdrawn\n---\n"));
        g.insert_edge(Edge::new(
            NodeId::new("d1"),
            NodeId::new("c2"),
            EdgeKind::Retracts,
        ));
        let err = freeze(&g, &NodeId::new("c1"))
            .expect_err("a claim may not freeze over a prerequisite the record withdraws");
        let msg = err.to_string();
        assert!(
            msg.contains("c2"),
            "the refusal must name the prerequisite: {msg}"
        );
        assert!(
            msg.contains("RETRACTED"),
            "and it must refuse for the WITHDRAWAL, not for some unrelated finding: {msg}"
        );
    }

    /// A claim standing on a defender answers for that defender.
    ///
    /// `freeze` refuses a claim defeated in the grounded extension, so grounding is a
    /// real barrier — but the closure that gets EXAMINED followed support, while
    /// grounding is decided by the ATTACK relation the closure never visited. So any
    /// live rival could be neutralised by one unexamined line: the node that decides
    /// whether the packet freezes was held to no standard at all.
    ///
    /// DEFENDERS, not attackers. A node that defeats an attacker is holding this claim
    /// up, exactly as a supporter does, and is the claim's own author's business.
    /// An attacker is opposition — pulling it in would block the victim for someone
    /// else's frontmatter, which is the defect fixed one test above this one.
    #[test]
    fn a_claim_answers_for_the_defender_it_stands_on() {
        let corners = concat!(
            "corners:\n",
            "  - it was catalogued\n",
            "  - it was not catalogued\n",
            "  - catalogued under one mechanism and not another\n",
            "  - the question does not arise\n"
        );
        let build = |defender: Option<&str>| {
            let mut g = clean_graph();
            g.insert_node(node(&format!(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file at that path\n\
warrant: A catalogue entry evidences that the path was recorded.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
boundaries:\n  - Windows 10 1809 and later\n{corners}falsifier:\n\
  - an entry shown to be written without the path ever being present\n---\n"
            )));
            // A groomed rival that genuinely defeats c1.
            g.insert_node(node(&format!(
                "---\nid: rival\ntype: claim\ntitle: An inventory sweep produced the record\n\
warrant: Sweeps populate the same table without observing the path.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
no_terms_of_art: true\nboundaries:\n  - Windows 10 1809 and later\n{corners}\
falsifier:\n  - a sweep shown never to write this table\n---\n"
            )));
            g.insert_edge(Edge::new(
                NodeId::new("rival"),
                NodeId::new("c1"),
                EdgeKind::Attacks,
            ));
            if let Some(src) = defender {
                g.insert_node(node(src));
                g.insert_edge(Edge::new(
                    NodeId::new("def"),
                    NodeId::new("rival"),
                    EdgeKind::Attacks,
                ));
                // A defender rests on evidence like anything else. Given to BOTH arms,
                // so the bare/groomed difference is the grooming and nothing else.
                g.insert_node(node(
                    "---\nid: o3\ntype: observation\ntitle: the sweep log postdates the recorded write\naspect: function\n---\n",
                ));
                g.insert_edge(
                    Edge::new(NodeId::new("o3"), NodeId::new("def"), EdgeKind::Supports)
                        .graded_by(Grade::G2, "a-reviewer")
                        .via(Pramana::Perception),
                );
            }
            freeze(&g, &NodeId::new("c1"))
        };

        assert!(
            build(None).is_err(),
            "control: with the rival unanswered, c1 is defeated and must not freeze"
        );

        let bare = build(Some(
            "---\nid: def\ntype: claim\ntitle: The sweep account does not fit this host timeline\n---\n",
        ));
        assert!(
            bare.is_err(),
            "a ONE-LINE node decided that this packet freezes; it must answer for itself"
        );

        let groomed = build(Some(&format!(
            "---\nid: def\ntype: claim\ntitle: The sweep account does not fit this host timeline\n\
warrant: The sweep ran after the recorded write, so it cannot have produced it.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
no_terms_of_art: true\nboundaries:\n  - Windows 10 1809 and later\n{corners}\
falsifier:\n  - a sweep timestamp preceding the recorded write\n---\n"
        )));
        assert!(
            groomed.is_ok(),
            "and a defender that HAS answered for itself must let the packet through: {:?}",
            groomed.as_ref().err()
        );
    }

    /// Another author's words must not decide this packet's fate — by any spelling.
    ///
    /// A rival's title is an assertion its own author wrote and the subject cannot
    /// edit. `defeat_block` settled this: quote it, flag it, say plainly it is not
    /// adopted, and report the finding against the node whose author can fix it.
    /// `standing_line` settled it again for withdrawn attackers.
    ///
    /// The `## Contradicting` section was the third door. Spelled `attacks:` the rival
    /// was disclosed and the packet froze; spelled `contradicts:` — the same words, the
    /// same node — the victim was blocked for saying "guilty". Same content must not
    /// change outcome by edge spelling, which is this project's rule about synonyms.
    #[test]
    fn a_rival_cannot_block_the_packet_it_is_quoted_in() {
        let build = |kind: EdgeKind| {
            let mut g = clean_graph();
            g.insert_node(node(
                "---\nid: rival\ntype: claim\ntitle: The suspect is guilty of unauthorised access\n\
corners:\n  - it was catalogued\n  - it was not catalogued\n  - both mechanisms ran\n  - the question does not arise\n---\n",
            ));
            g.insert_edge(Edge::new(NodeId::new("rival"), NodeId::new("c1"), kind));
            // The rival must itself be answered, or c1 leaves the grounded extension and
            // freeze refuses for that instead — which would make this test measure
            // standing rather than prose.
            g.insert_node(node(&groomed(
                "rk",
                "The rival account does not fit the host timeline",
            )));
            evidence_for(&mut g, "o-rk", "rk");
            g.insert_edge(Edge::new(
                NodeId::new("rk"),
                NodeId::new("rival"),
                EdgeKind::Attacks,
            ));
            // c1 becomes contested by either spelling, so answer 四句 in both arms —
            // otherwise the arms differ for a reason that has nothing to do with prose.
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file at that path\n\
warrant: A catalogue entry evidences that the path was recorded.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
boundaries:\n  - Windows 10 1809 and later\n\
falsifier:\n  - an entry shown to be written without the path ever being present\n\
corners:\n  - it was catalogued\n  - it was not catalogued\n  - catalogued under one mechanism and not another\n  - the question does not arise\n---\n",
            ));
            freeze(&g, &NodeId::new("c1"))
        };

        let via_attacks = build(EdgeKind::Attacks);
        let via_contradicts = build(EdgeKind::Contradicts);
        let via_limits = build(EdgeKind::Limits);

        assert!(
            via_attacks.is_ok(),
            "control: the disclosed-rival path already freezes — {:?}",
            via_attacks.as_ref().err()
        );
        assert!(
            via_contradicts.is_ok(),
            "the same words under a different edge kind must not block the victim: {:?}",
            via_contradicts.as_ref().err()
        );
        assert!(
            via_limits.is_ok(),
            "nor under a third: {:?}",
            via_limits.as_ref().err()
        );

        for (what, p) in [
            ("attacks", via_attacks.unwrap()),
            ("contradicts", via_contradicts.unwrap()),
            ("limits", via_limits.unwrap()),
        ] {
            assert!(
                !p.body.contains("guilty"),
                "{what}: the flagged verdict must not be printed verbatim into a court artifact"
            );
            assert!(
                p.body.contains("rival"),
                "{what}: but the rival must still be NAMED — withholding the words is not hiding the node"
            );
        }
    }

    /// The closure covers everything the packet RENDERS, not only what it rests on.
    ///
    /// `freeze` renders nodes reached by `Contradicts` and `Limits` under their own
    /// headings, and the closure followed only `Supports` and `UsesTerm` — so a legal
    /// conclusion on a limiter printed verbatim into a court artifact while `peira lint`
    /// reported it happily elsewhere.
    ///
    /// THE REMEDY CHANGED, and the concern did not. This test used to require a
    /// REFUSAL, which punished the subject for prose another author wrote and they
    /// could not edit — the same defect, in the other direction. Withholding the
    /// flagged words satisfies the original worry more exactly than refusing did: the
    /// conclusion still never reaches the artifact, and the packet still freezes.
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
        let p = freeze(&g, &NodeId::new("c1"))
            .expect("the subject is not answerable for a limiter's prose");
        assert!(
            !p.body.contains("guilty"),
            "the ultimate issue must not print verbatim into a court artifact:\n{}",
            p.body
        );
        assert!(
            p.body.contains("lim") && p.body.contains("peira lint lim"),
            "and the reader must be told which node was withheld, and where to read it:\n{}",
            p.body
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
            p.body.contains(&format!("Packet format: {PACKET_FORMAT}")),
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
        g.insert_node(node(&groomed(
            "c3",
            "The write path requires the file on the volume",
        )));
        evidence_for(&mut g, "o-c3", "c3");
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
        // The other half of the framing property, so that disjunct is not merely
        // asserted in `a_defeat_line_carries_its_frame_out_of_the_section` but reached:
        // a rival's title is quoted here, and quoting it without saying so adopts it.
        assert!(
            p.body.contains(
                "Catalogued without the path ever being present — on record as an attack"
            ),
            "the attacker's title is quoted without the note that frames it:\n{}",
            p.body
        );
    }

    /// Every line of the defeat section must carry its own frame.
    ///
    /// The section heading is the only thing telling a reader that these are
    /// conditions which WOULD defeat the claim rather than things asserted about the
    /// world — and a heading does not travel. Lifted into a skeleton argument or an
    /// email, `- the transfer was fraudulent` is indistinguishable from a finding.
    ///
    /// The attack bullets already solved this, one loop below in the same function:
    /// each ends "— on record as an attack", so the disclosure survives quotation.
    /// The stated falsifiers were the lines that did not.
    #[test]
    fn a_defeat_line_carries_its_frame_out_of_the_section() {
        let g = clean_graph();
        let p = freeze(&g, &NodeId::new("c1")).expect("clean claim should freeze");
        let section = p
            .body
            .split("\n## ")
            .find(|s| s.starts_with("What would defeat this"))
            .expect("the defeat section renders");
        let bullets: Vec<&str> = section
            .lines()
            .filter(|l| l.trim_start().starts_with("- "))
            .collect();
        assert!(!bullets.is_empty(), "no bullets to check:\n{section}");
        for line in bullets {
            assert!(
                line.contains(FALSIFIER_FRAME) || line.contains("on record as an attack"),
                "this line reads as an assertion once quoted away from the heading:\n{line}"
            );
        }
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
