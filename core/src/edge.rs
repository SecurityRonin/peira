//! Edges, evidence grades, and the means-of-knowledge typing that caps them.

use crate::node::NodeId;
use std::fmt;

/// How one node bears on another.
///
/// Four families. The epistemic ones come from the Vibe Research model; the
/// ontological ones exist so 白馬非馬 has something to check; 體用 gets its own pair
/// so a claim about what a thing *does* cannot be filed as a claim about what it
/// *is*; and the dialectical ones make Aufhebung structural — a synthesis has to
/// name the parents it preserves.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EdgeKind {
    // ── Epistemic ────────────────────────────────────────────────────────────
    /// Evidence for the target.
    Supports,
    /// Evidence against the target.
    Contradicts,
    /// Bounds the target's scope without refuting it.
    Limits,
    /// Restates the target — not independent evidence.
    Duplicates,
    /// The target must hold for the source to.
    DependsOn,
    /// A newer version replaces the target, which is retained, never deleted.
    Supersedes,
    /// Withdraws a previously accepted target.
    Retracts,

    // ── Ontological (白馬非馬, is-a / has-a) ──────────────────────────────────
    /// Subtype of.
    IsA,
    /// Has as a component or attribute.
    HasA,
    /// A token of the target type.
    InstanceOf,
    /// A part of the target whole.
    PartOf,

    // ── 體用 (substance / function) ──────────────────────────────────────────
    /// The source states what the target *is*.
    SubstanceOf,
    /// The source states what the target *does*.
    FunctionOf,

    // ── Dialectical ─────────────────────────────────────────────────────────
    /// The antithesis of the target.
    Negates,
    /// A synthesis that preserves the target while superseding it (Aufhebung).
    Sublates,

    // ── Structural ──────────────────────────────────────────────────────────
    /// An explicit attack, when `Contradicts` is too weak a word.
    Attacks,
    /// The standard by which an evaluative claim is judged (立極).
    JudgedBy,
    /// A term the source uses in a load-bearing way (正名).
    UsesTerm,
    /// An examination's subject.
    Examines,
    /// The source observation came off the target instrument.
    MeasuredBy,
}

impl EdgeKind {
    /// Every edge kind, and the ONE list of them.
    ///
    /// The vault loader used to carry its own copy as `EDGE_KEYS` — a third spelling
    /// of the edge grammar beside this enum and `from_str`. Deleting one entry there
    /// silently stopped loading that edge for every vault in the field, with the whole
    /// suite green, because nothing held the two lists together. `exhaustive` below is
    /// what holds them together now: adding a variant fails the build until it is
    /// added here too.
    pub const ALL: &'static [EdgeKind] = &[
        EdgeKind::Supports,
        EdgeKind::Contradicts,
        EdgeKind::Limits,
        EdgeKind::Duplicates,
        EdgeKind::DependsOn,
        EdgeKind::Supersedes,
        EdgeKind::Retracts,
        EdgeKind::IsA,
        EdgeKind::HasA,
        EdgeKind::InstanceOf,
        EdgeKind::PartOf,
        EdgeKind::SubstanceOf,
        EdgeKind::FunctionOf,
        EdgeKind::Negates,
        EdgeKind::Sublates,
        EdgeKind::Attacks,
        EdgeKind::JudgedBy,
        EdgeKind::UsesTerm,
        EdgeKind::Examines,
        EdgeKind::MeasuredBy,
    ];

    /// A compile-time proof that [`EdgeKind::ALL`] is complete.
    ///
    /// Not a test — a test can only run after someone remembers to write it. This is
    /// an exhaustive match, so a new variant is a BUILD failure rather than a silently
    /// unloadable edge.
    #[allow(dead_code)]
    const fn exhaustive(self) {
        match self {
            EdgeKind::Supports
            | EdgeKind::Contradicts
            | EdgeKind::Limits
            | EdgeKind::Duplicates
            | EdgeKind::DependsOn
            | EdgeKind::Supersedes
            | EdgeKind::Retracts
            | EdgeKind::IsA
            | EdgeKind::HasA
            | EdgeKind::InstanceOf
            | EdgeKind::PartOf
            | EdgeKind::SubstanceOf
            | EdgeKind::FunctionOf
            | EdgeKind::Negates
            | EdgeKind::Sublates
            | EdgeKind::Attacks
            | EdgeKind::JudgedBy
            | EdgeKind::UsesTerm
            | EdgeKind::Examines
            | EdgeKind::MeasuredBy => (),
        }
    }

    /// The frontmatter spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            EdgeKind::Supports => "supports",
            EdgeKind::Contradicts => "contradicts",
            EdgeKind::Limits => "limits",
            EdgeKind::Duplicates => "duplicates",
            EdgeKind::DependsOn => "depends_on",
            EdgeKind::Supersedes => "supersedes",
            EdgeKind::Retracts => "retracts",
            EdgeKind::IsA => "is_a",
            EdgeKind::HasA => "has_a",
            EdgeKind::InstanceOf => "instance_of",
            EdgeKind::PartOf => "part_of",
            EdgeKind::SubstanceOf => "substance_of",
            EdgeKind::FunctionOf => "function_of",
            EdgeKind::Negates => "negates",
            EdgeKind::Sublates => "sublates",
            EdgeKind::Attacks => "attacks",
            EdgeKind::JudgedBy => "judged_by",
            EdgeKind::UsesTerm => "uses_term",
            EdgeKind::Examines => "examines",
            EdgeKind::MeasuredBy => "measured_by",
        }
    }

    /// Parse a frontmatter edge name.
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        Some(match s {
            "supports" => EdgeKind::Supports,
            "contradicts" => EdgeKind::Contradicts,
            "limits" => EdgeKind::Limits,
            "duplicates" => EdgeKind::Duplicates,
            "depends_on" => EdgeKind::DependsOn,
            "supersedes" => EdgeKind::Supersedes,
            "retracts" => EdgeKind::Retracts,
            "is_a" => EdgeKind::IsA,
            "has_a" => EdgeKind::HasA,
            "instance_of" => EdgeKind::InstanceOf,
            "part_of" => EdgeKind::PartOf,
            "substance_of" => EdgeKind::SubstanceOf,
            "function_of" => EdgeKind::FunctionOf,
            "negates" => EdgeKind::Negates,
            "sublates" => EdgeKind::Sublates,
            "attacks" => EdgeKind::Attacks,
            "judged_by" => EdgeKind::JudgedBy,
            "uses_term" => EdgeKind::UsesTerm,
            "examines" => EdgeKind::Examines,
            "measured_by" => EdgeKind::MeasuredBy,
            _ => return None,
        })
    }

    /// Whether this edge constitutes an attack in the argumentation graph.
    ///
    /// `Retracts` deliberately does **not** attack: a retraction is a reviewed act
    /// on a claim's own lifecycle, not a competing argument, and treating it as an
    /// attack would let it be "defended against" by a third claim.
    #[must_use]
    pub fn is_attack(self) -> bool {
        matches!(
            self,
            EdgeKind::Contradicts | EdgeKind::Attacks | EdgeKind::Negates
        )
    }

    /// Whether this kind says the target has been replaced by something newer.
    ///
    /// `Retracts` withdraws it; `Supersedes` replaces it; `Sublates` — "preserves the
    /// target while superseding it" — replaces it while keeping its content in the
    /// synthesis. All three mean the target is no longer the current statement, and
    /// naming only two let the third seal a retired claim in silence.
    #[must_use]
    pub fn supersedes_target(self) -> bool {
        matches!(
            self,
            EdgeKind::Retracts | EdgeKind::Supersedes | EdgeKind::Sublates
        )
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The strength of one evidence-to-claim edge.
///
/// Distinct from claim confidence, source quality, claim state and investigative
/// severity — the Vibe model keeps five grades separate, and collapsing any two is
/// the mistake this type exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Grade {
    /// Unsupported assertion or unverified pointer.
    G0,
    /// A relevant pinned passage or observation.
    G1,
    /// Directly applicable evidence with method and provenance.
    G2,
    /// A reproducible test or independently verified observation.
    G3,
    /// Multiple materially independent convergent lines, boundaries addressed.
    G4,
}

impl Grade {
    /// The frontmatter spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Grade::G0 => "G0",
            Grade::G1 => "G1",
            Grade::G2 => "G2",
            Grade::G3 => "G3",
            Grade::G4 => "G4",
        }
    }

    /// Parse a grade.
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        Some(match s {
            "G0" => Grade::G0,
            "G1" => Grade::G1,
            "G2" => Grade::G2,
            "G3" => Grade::G3,
            "G4" => Grade::G4,
            _ => return None,
        })
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How a knowing was arrived at — the Nyāya means of knowledge (pramāṇa).
///
/// Typing evidence this way is what stops testimony being filed as observation.
/// It is also the mechanical form of the manifesto's rule that independent tools
/// are not automatically independent evidence: two parsers agreeing is *śabda*
/// corroboration, not perception, and it cannot buy a grade perception would earn.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pramana {
    /// Pratyakṣa — direct perception, including a reading taken off an instrument.
    Perception,
    /// Anumāna — inference from what was perceived.
    Inference,
    /// Upamāna — comparison or analogy.
    Comparison,
    /// Śabda — testimony: documentation, a write-up, another tool's report.
    Testimony,
}

impl Pramana {
    /// The frontmatter spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Pramana::Perception => "perception",
            Pramana::Inference => "inference",
            Pramana::Comparison => "comparison",
            Pramana::Testimony => "testimony",
        }
    }

    /// Parse a means of knowledge, accepting both the English and Sanskrit names.
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        Some(match s {
            "perception" | "pratyaksa" | "pratyakṣa" => Pramana::Perception,
            "inference" | "anumana" | "anumāna" => Pramana::Inference,
            "comparison" | "upamana" | "upamāna" => Pramana::Comparison,
            "testimony" | "sabda" | "śabda" => Pramana::Testimony,
            _ => return None,
        })
    }

    /// The highest grade an edge resting on this means of knowledge may carry.
    ///
    /// `G4` is reachable by no single edge at all: it requires multiple materially
    /// independent lines, which is a property of the graph rather than of any one
    /// piece of evidence. Encoding that here means a G4 can never be asserted, only
    /// earned.
    #[must_use]
    pub fn grade_ceiling(self) -> Grade {
        match self {
            Pramana::Perception => Grade::G3,
            Pramana::Inference => Grade::G2,
            Pramana::Comparison | Pramana::Testimony => Grade::G1,
        }
    }
}

impl fmt::Display for Pramana {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A directed edge between two nodes.
///
/// A settled `grade` is inseparable from the reviewer who set it: the two live in
/// one field, so an edge carrying a grade without an attributed grader is not a
/// lint failure to be caught later but a value that cannot be built. A model may
/// still contribute by way of `grade_proposed`, which asserts nothing.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Source node.
    pub from: NodeId,
    /// Target node.
    pub to: NodeId,
    /// The relation.
    pub kind: EdgeKind,
    /// A reviewed grade together with the identity that set it.
    graded: Option<(Grade, String)>,
    /// A grade anyone — including a model — may suggest. Carries no authority.
    pub grade_proposed: Option<Grade>,
    /// How the supporting knowing was arrived at, when this is an evidence edge.
    pub pramana: Option<Pramana>,
}

impl Edge {
    /// A plain, ungraded edge.
    #[must_use]
    pub fn new(from: NodeId, to: NodeId, kind: EdgeKind) -> Self {
        Self {
            from,
            to,
            kind,
            graded: None,
            grade_proposed: None,
            pramana: None,
        }
    }

    /// Suggest a grade without asserting it.
    #[must_use]
    pub fn proposing(mut self, grade: Grade) -> Self {
        self.grade_proposed = Some(grade);
        self
    }

    /// Record a reviewed grade. The grader's identity is required by construction.
    #[must_use]
    pub fn graded_by(mut self, grade: Grade, reviewer: impl Into<String>) -> Self {
        self.graded = Some((grade, reviewer.into()));
        self
    }

    /// Declare how the knowing was arrived at.
    #[must_use]
    pub fn via(mut self, pramana: Pramana) -> Self {
        self.pramana = Some(pramana);
        self
    }

    /// The settled grade, if a reviewer set one.
    #[must_use]
    pub fn grade(&self) -> Option<Grade> {
        self.graded.as_ref().map(|(g, _)| *g)
    }

    /// Who settled the grade.
    #[must_use]
    pub fn grader(&self) -> Option<&str> {
        self.graded.as_ref().map(|(_, who)| who.as_str())
    }

    /// Whether the settled grade exceeds what its means of knowledge allows.
    ///
    /// An ungraded edge, or one with no declared pramāṇa, is not over-graded — it
    /// is *unassessed*, which the gates report separately. Silence is never a pass.
    #[must_use]
    pub fn exceeds_pramana_ceiling(&self) -> bool {
        match (self.grade(), self.pramana) {
            (Some(grade), Some(pramana)) => grade > pramana.grade_ceiling(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod published_pramana_spellings {
    use super::*;

    /// H6 — `every_pramana_renders_its_name` compared a value to itself.
    ///
    /// These four tokens are what an author writes as `via=` and what a packet prints;
    /// pinning them to literals is the only thing that notices one changing.
    #[test]
    fn every_pramana_token_is_pinned_to_its_published_literal() {
        for (p, spelling) in [
            (Pramana::Perception, "perception"),
            (Pramana::Inference, "inference"),
            (Pramana::Comparison, "comparison"),
            (Pramana::Testimony, "testimony"),
        ] {
            assert_eq!(
                p.as_str(),
                spelling,
                "the published `via=` token changed — every vault in the field writes \
the old one"
            );
        }
    }
}

#[cfg(test)]
mod tests {

    /// The GRADE grammar, for the same reason.
    ///
    /// Grade tokens had no table at all, so a double-rename of "G0" round-tripped
    /// invisibly while every vault writing the old spelling silently settled nothing —
    /// the loader drops an unparsable grade rather than refusing it.
    #[test]
    fn the_published_grade_grammar() {
        use crate::Grade;
        const GRAMMAR: &[(Grade, &str)] = &[
            (Grade::G0, "G0"),
            (Grade::G1, "G1"),
            (Grade::G2, "G2"),
            (Grade::G3, "G3"),
            (Grade::G4, "G4"),
        ];
        for (grade, token) in GRAMMAR {
            assert_eq!(
                grade.as_str(),
                *token,
                "the token a vault must write changed"
            );
            assert_eq!(
                Grade::from_str_opt(token),
                Some(*grade),
                "a vault writing `{token}` must still load as {grade:?}"
            );
        }
    }

    /// The vault grammar, written down where a rename cannot follow it.
    ///
    /// `as_str` and `from_str` were checked only against each other, so renaming a
    /// token in BOTH match arms round-tripped perfectly while every vault using the old
    /// spelling silently lost its edges — the loader does not reject an unknown key, it
    /// drops it. A round trip proves two functions agree; it says nothing about what
    /// they agree ON.
    ///
    /// This table is the published grammar. Changing a token here is changing what
    /// every existing vault means, and that should take an argument, not a rename.
    #[test]
    fn the_published_edge_grammar() {
        use crate::EdgeKind;
        const GRAMMAR: &[(EdgeKind, &str)] = &[
            (EdgeKind::Supports, "supports"),
            (EdgeKind::Contradicts, "contradicts"),
            (EdgeKind::DependsOn, "depends_on"),
            (EdgeKind::Limits, "limits"),
            (EdgeKind::Retracts, "retracts"),
            (EdgeKind::Supersedes, "supersedes"),
            (EdgeKind::SubstanceOf, "substance_of"),
            (EdgeKind::FunctionOf, "function_of"),
            (EdgeKind::Negates, "negates"),
            (EdgeKind::Sublates, "sublates"),
            (EdgeKind::Attacks, "attacks"),
            (EdgeKind::JudgedBy, "judged_by"),
            (EdgeKind::UsesTerm, "uses_term"),
            (EdgeKind::MeasuredBy, "measured_by"),
            (EdgeKind::Duplicates, "duplicates"),
            (EdgeKind::IsA, "is_a"),
            (EdgeKind::HasA, "has_a"),
            (EdgeKind::InstanceOf, "instance_of"),
            (EdgeKind::PartOf, "part_of"),
            (EdgeKind::Examines, "examines"),
        ];
        for (kind, token) in GRAMMAR {
            assert_eq!(
                kind.as_str(),
                *token,
                "the token a vault must write for {kind:?} changed"
            );
            assert_eq!(
                EdgeKind::from_str_opt(token),
                Some(*kind),
                "a vault writing `{token}` must still load as {kind:?}"
            );
        }
    }
    use super::*;

    fn edge() -> Edge {
        Edge::new(NodeId::new("a"), NodeId::new("b"), EdgeKind::Supports)
    }

    #[test]
    fn a_proposed_grade_is_not_a_settled_grade() {
        let e = edge().proposing(Grade::G4);
        assert_eq!(e.grade_proposed, Some(Grade::G4));
        assert_eq!(e.grade(), None, "a proposal asserts nothing");
        assert_eq!(e.grader(), None);
    }

    #[test]
    fn a_settled_grade_always_carries_its_grader() {
        let e = edge().graded_by(Grade::G2, "albert");
        assert_eq!(e.grade(), Some(Grade::G2));
        assert_eq!(e.grader(), Some("albert"));
    }

    #[test]
    fn testimony_cannot_carry_a_perception_grade() {
        let e = edge()
            .via(Pramana::Testimony)
            .graded_by(Grade::G3, "albert");
        assert!(
            e.exceeds_pramana_ceiling(),
            "two tools agreeing is corroboration, not perception"
        );
    }

    #[test]
    fn perception_may_carry_g3() {
        let e = edge()
            .via(Pramana::Perception)
            .graded_by(Grade::G3, "albert");
        assert!(!e.exceeds_pramana_ceiling());
    }

    #[test]
    fn no_single_edge_may_reach_g4_whatever_its_pramana() {
        for pramana in [
            Pramana::Perception,
            Pramana::Inference,
            Pramana::Comparison,
            Pramana::Testimony,
        ] {
            let e = edge().via(pramana).graded_by(Grade::G4, "albert");
            assert!(
                e.exceeds_pramana_ceiling(),
                "G4 needs independent convergent lines, a graph property — {pramana} must not reach it"
            );
        }
    }

    #[test]
    fn an_ungraded_edge_is_unassessed_not_passing() {
        let e = edge().via(Pramana::Testimony);
        assert!(
            !e.exceeds_pramana_ceiling(),
            "unassessed is not a violation — but it is also not a pass, which the gates report"
        );
        assert_eq!(e.grade(), None);
    }

    #[test]
    fn retraction_is_not_an_attack() {
        assert!(!EdgeKind::Retracts.is_attack());
        assert!(EdgeKind::Contradicts.is_attack());
        assert!(EdgeKind::Attacks.is_attack());
        assert!(EdgeKind::Negates.is_attack());
    }

    #[test]
    fn edge_names_round_trip() {
        for kind in [
            EdgeKind::Supports,
            EdgeKind::Contradicts,
            EdgeKind::Limits,
            EdgeKind::Duplicates,
            EdgeKind::DependsOn,
            EdgeKind::Supersedes,
            EdgeKind::Retracts,
            EdgeKind::IsA,
            EdgeKind::HasA,
            EdgeKind::InstanceOf,
            EdgeKind::PartOf,
            EdgeKind::SubstanceOf,
            EdgeKind::FunctionOf,
            EdgeKind::Negates,
            EdgeKind::Sublates,
            EdgeKind::Attacks,
            EdgeKind::JudgedBy,
            EdgeKind::UsesTerm,
            EdgeKind::Examines,
            EdgeKind::MeasuredBy,
        ] {
            assert_eq!(EdgeKind::from_str_opt(kind.as_str()), Some(kind));
            assert_eq!(kind.to_string(), kind.as_str());
        }
        assert_eq!(EdgeKind::from_str_opt("nonesuch"), None);
    }

    #[test]
    fn every_grade_round_trips_and_renders() {
        for grade in [Grade::G0, Grade::G1, Grade::G2, Grade::G3, Grade::G4] {
            assert_eq!(Grade::from_str_opt(grade.as_str()), Some(grade));
            assert_eq!(grade.to_string(), grade.as_str());
        }
        assert_eq!(Grade::from_str_opt("G5"), None);
        assert_eq!(Grade::from_str_opt(""), None);
    }

    #[test]
    fn grades_order_from_weakest_to_strongest() {
        assert!(Grade::G0 < Grade::G1);
        assert!(Grade::G3 < Grade::G4);
    }

    #[test]
    fn pramana_accepts_english_and_sanskrit_including_diacritics() {
        for (input, expected) in [
            ("perception", Pramana::Perception),
            ("pratyaksa", Pramana::Perception),
            ("pratyakṣa", Pramana::Perception),
            ("inference", Pramana::Inference),
            ("anumana", Pramana::Inference),
            ("anumāna", Pramana::Inference),
            ("comparison", Pramana::Comparison),
            ("upamana", Pramana::Comparison),
            ("upamāna", Pramana::Comparison),
            ("testimony", Pramana::Testimony),
            ("sabda", Pramana::Testimony),
            ("śabda", Pramana::Testimony),
        ] {
            assert_eq!(
                Pramana::from_str_opt(input),
                Some(expected),
                "input {input}"
            );
        }
        assert_eq!(Pramana::from_str_opt("revelation"), None);
    }

    #[test]
    fn every_pramana_renders_its_name() {
        for p in [
            Pramana::Perception,
            Pramana::Inference,
            Pramana::Comparison,
            Pramana::Testimony,
        ] {
            assert_eq!(p.to_string(), p.as_str());
        }
    }

    #[test]
    fn the_ceiling_ordering_reflects_the_epistemology() {
        // Perception outranks inference, which outranks the two second-hand routes.
        assert!(Pramana::Perception.grade_ceiling() > Pramana::Inference.grade_ceiling());
        assert!(Pramana::Inference.grade_ceiling() > Pramana::Testimony.grade_ceiling());
        assert_eq!(
            Pramana::Comparison.grade_ceiling(),
            Pramana::Testimony.grade_ceiling()
        );
    }

    #[test]
    fn an_edge_with_a_grade_but_no_pramana_is_not_flagged_as_over_graded() {
        let e = edge().graded_by(Grade::G4, "albert");
        assert!(
            !e.exceeds_pramana_ceiling(),
            "with no declared means of knowing there is nothing to compare against; \
the gates report that separately as unassessed"
        );
    }
}
