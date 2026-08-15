//! The peira lens catalogue.
//!
//! Each entry names a **failure mode** that a specific critical-thinking tradition
//! identified, and pairs it with a machine-checkable gate over the claim graph. The
//! wager of this crate is that these traditions are not decoration: each one names a
//! recurring way of being wrong, and a graph can mechanically refuse to promote a
//! claim that has not been examined for it.
//!
//! The catalogue is data, in the shape of `forensicnomicon`: `static`/`const` memory
//! with no runtime construction, so it is auditable as source and testable as a
//! table. The meta-tests at the bottom assert properties *of the knowledge* — every
//! lens cites a source, every gate code is unique, every MVP lens actually has a
//! gate — rather than of any code path.
//!
//! Only **domain-neutral** lenses live here. Domain packs (`peira-forensic`,
//! `peira-legal`) depend down onto this crate and add their own criteria; that is
//! the mechanism by which "universal" stays universal instead of quietly becoming a
//! forensics tool.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod gates;
pub mod lints;

use peira_core::{Graph, Node, NodeId, NodeKind};
use std::fmt;

/// The intellectual lineage a lens comes from.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tradition {
    /// Greek — Socratic elenchus, Aristotelian causes.
    Greek,
    /// Chinese — 名家, 宋明理學, 易/太極.
    Chinese,
    /// Indian — Nyāya, and the wider pramāṇa epistemology.
    Indian,
    /// Buddhist — Madhyamaka, 金剛經, 二諦.
    Buddhist,
    /// Jewish — Talmudic machloket and the preservation of minority opinion.
    Jewish,
    /// Modern analytic and scientific method — Toulmin, Popper, Pearl, Heuer.
    Modern,
    /// Formal — Dung argumentation, and computability of dialectical status.
    Formal,
}

impl Tradition {
    /// A short label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Tradition::Greek => "Greek",
            Tradition::Chinese => "Chinese",
            Tradition::Indian => "Indian",
            Tradition::Buddhist => "Buddhist",
            Tradition::Jewish => "Jewish",
            Tradition::Modern => "Modern",
            Tradition::Formal => "Formal",
        }
    }
}

impl fmt::Display for Tradition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether a lens is enforced today or catalogued for a later phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Enforced now: the lens has at least one working gate.
    Enforced,
    /// Named, sourced and specified, but not yet mechanised.
    Catalogued,
}

/// What a gate concluded about one node.
///
/// `Unassessed` exists because an empty result must never read as a pass. A gate
/// that could not run — the data it needs was never written, the subject is out of
/// scope for it — has *not* cleared the claim, and rendering that as success is the
/// single most common way a checking system lies to the person relying on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateResult {
    /// The gate ran and the node satisfies it.
    Pass,
    /// The gate ran and the node fails it.
    Block(Violation),
    /// The gate does not apply to this kind of node at all.
    NotApplicable,
    /// The gate could not reach a verdict. **Never a pass.**
    Unassessed {
        /// Why no verdict was reached, naming what was missing.
        why: String,
    },
}

/// A gate reached no verdict. **Never a pass**, and never silent.
///
/// Published like any other code, because a packet may cite it: a claim blocked for
/// want of a verdict is blocked for a different reason than one that failed a check,
/// and the reader is entitled to the difference.
pub const GATE_UNASSESSED: &str = "PEIR-GATE-UNASSESSED";

impl GateResult {
    /// Whether this result permits promotion. Only [`GateResult::Pass`] and
    /// [`GateResult::NotApplicable`] do.
    #[must_use]
    pub fn permits_promotion(&self) -> bool {
        matches!(self, GateResult::Pass | GateResult::NotApplicable)
    }

    /// The violation, if the gate blocked.
    #[must_use]
    pub fn violation(&self) -> Option<&Violation> {
        match self {
            GateResult::Block(v) => Some(v),
            _ => None,
        }
    }
}

/// A specific failure of a specific gate on a specific node.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The stable gate code, e.g. `PEIR-WARRANT-MISSING`. A published contract:
    /// packets cite these, so a shipped code never changes meaning.
    pub gate: &'static str,
    /// The lens the gate belongs to.
    pub lens: &'static str,
    /// The node that failed.
    pub subject: NodeId,
    /// What was actually found, shown verbatim.
    pub detail: String,
    /// What would resolve it.
    pub remedy: &'static str,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] {}: {} — {}",
            self.gate, self.lens, self.subject, self.detail, self.remedy
        )
    }
}

/// A gate predicate: pure, no I/O, over the graph and one node.
pub type GateFn = fn(&Graph, &Node) -> GateResult;

/// One machine-checkable obligation belonging to a lens.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct Gate {
    /// Stable published code.
    pub code: &'static str,
    /// The predicate.
    pub check: GateFn,
}

/// One examination, drawn from a tradition, reduced to a graph operation.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub struct Lens {
    /// Stable machine id, e.g. `LIJI`.
    pub id: &'static str,
    /// Display name, with the original term where there is one.
    pub name: &'static str,
    /// Where it comes from.
    pub tradition: Tradition,
    /// The failure mode it names — the reason it earns a place.
    pub failure_mode: &'static str,
    /// What it does to the graph.
    pub operation: &'static str,
    /// Node kinds it examines. Empty means all.
    pub applies_to: &'static [NodeKind],
    /// Its enforced obligations.
    pub gates: &'static [Gate],
    /// A worked example of the gate firing — required, so that every catalogue
    /// entry is falsifiable rather than merely plausible.
    pub worked_example: &'static str,
    /// Authoritative references.
    pub sources: &'static [&'static str],
    /// Enforced now, or catalogued for later.
    pub phase: Phase,
}

impl Lens {
    /// Run every gate this lens owns against a node.
    #[must_use]
    pub fn examine(&self, graph: &Graph, node: &Node) -> Vec<GateResult> {
        if !self.applies_to.is_empty() && !self.applies_to.contains(&node.kind) {
            return vec![GateResult::NotApplicable];
        }
        self.gates.iter().map(|g| (g.check)(graph, node)).collect()
    }
}

/// Look up a lens by id.
#[must_use]
pub fn lens(id: &str) -> Option<&'static Lens> {
    CATALOG.iter().find(|l| l.id == id)
}

/// Every lens that is enforced today.
pub fn enforced() -> impl Iterator<Item = &'static Lens> {
    CATALOG.iter().filter(|l| l.phase == Phase::Enforced)
}

/// Run every enforced gate over every node in the graph, returning the violations.
#[must_use]
pub fn examine_graph(graph: &Graph) -> Vec<Violation> {
    let mut out = Vec::new();
    for node in graph.nodes() {
        for lens in enforced() {
            for result in lens.examine(graph, node) {
                // Every result that does NOT permit promotion must reach the caller.
                // Filtering on `violation()` alone discarded `Unassessed`, and with it
                // the rule that a gate which could not run has cleared nothing — the
                // predicate encoding that rule then had no production caller at all.
                if result.permits_promotion() {
                    continue;
                }
                match &result {
                    GateResult::Block(v) => out.push(v.clone()),
                    GateResult::Unassessed { why } => out.push(Violation {
                        gate: GATE_UNASSESSED,
                        lens: lens.id,
                        subject: node.id.clone(),
                        detail: format!("[{}] reached no verdict: {why}", lens.id),
                        remedy: "supply what the gate needs, or say plainly that this \
claim was never examined for it — a gate that could not run has cleared nothing",
                    }),
                    // Unreachable: both permit promotion and were skipped above. A
                    // future variant lands here and is DROPPED, so the match is
                    // exhaustive on purpose rather than a catch-all.
                    GateResult::Pass | GateResult::NotApplicable => {}
                }
            }
        }
    }
    out
}

/// Node kinds that compete in the graph and can therefore be attacked.
///
/// Used for the ATTACK relation, and deliberately no longer for scoping promotion
/// gates. `Lens::examine` applies `applies_to` BEFORE the gate runs, so a static kind
/// list in front of `gates::under_promotion` made its non-argument arm unreachable:
/// a universal over-claim labelled `type: observation` drew zero findings and froze,
/// while the identical node labelled `type: hypothesis` drew six and was refused.
///
/// That is the first defect this project ever found, one layer up — a rule that is
/// correct, unit-tested, and never reached. The lenses whose gates decide their own
/// scope now declare `applies_to: &[]` and let the gate decide, so there is ONE kind
/// test rather than two disagreeing ones.
const ARGUMENTS: &[NodeKind] = &[NodeKind::Claim, NodeKind::Hypothesis];

// Promotion obligations attach to being LOAD-BEARING rather than to the node kind:
// see `gates::under_promotion`. A hypothesis nothing leans on is a candidate, and a
// checker that blocks you for thinking out loud is a checker you switch off.

/// The catalogue.
pub static CATALOG: &[Lens] = &[
    Lens {
        id: "LIJI",
        name: "立極 — Set the Pole",
        tradition: Tradition::Chinese,
        failure_mode: "judging something without ever declaring the standard judged against",
        operation: "an evaluative claim requires a `judged_by` edge to a Criterion",
        applies_to: &[],
        gates: &[Gate {
            code: gates::CRITERION_UNDECLARED,
            check: gates::criterion_declared,
        }],
        worked_example: "\"…sits in a directory commonly used to stage malware\" calls a path \
suspicious without ever stating the standard of suspicion, so the reader cannot tell whether \
%TEMP% qualifies by frequency, by policy, or by the author's intuition.",
        sources: &[
            "周敦頤《太極圖說》: 聖人定之以中正仁義而主靜，立人極焉",
            "https://ctext.org/wiki.pl?if=en&chapter=592302",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "ZHENGMING",
        name: "正名 / 所謂 X 即非 X 是名 X — Rectify the Name",
        tradition: Tradition::Buddhist,
        failure_mode: "reifying a label into a thing, so a word does argumentative work its \
definition never licensed",
        operation: "every key term resolves to a Term node carrying as_used, not_essence and \
stipulated",
        applies_to: &[],
        gates: &[Gate {
            code: gates::TERM_UNSTIPULATED,
            check: gates::key_terms_stipulated,
        }],
        worked_example: "\"execution\" is used as though self-evident. The Diamond Sutra form \
makes the three moments explicit: 所謂「執行」— what is called execution; 即非「執行」— the \
artifact is not execution; 是名「執行」— it is conventionally named execution, under stated bounds.",
        sources: &[
            "《金剛般若波羅蜜經》: 所謂佛法者，即非佛法，是名佛法",
            "https://ctext.org/analects/zi-lu (論語·子路: 必也正名乎)",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "TIYONG",
        name: "體用 — Substance and Function",
        tradition: Tradition::Chinese,
        failure_mode: "reporting what a thing DID as though it established what a thing IS",
        operation: "a substance claim needs at least one substance-aspect support; function \
evidence alone cannot license it",
        applies_to: ARGUMENTS,
        gates: &[Gate {
            code: gates::FUNCTION_AS_SUBSTANCE,
            check: gates::substance_not_from_function_alone,
        }],
        worked_example: "\"Amcache is an execution artifact\" (substance) resting only on \
\"Amcache recorded this path\" (function). The record is a thing the artifact did; the claim is \
about what it is.",
        sources: &[
            "程頤《易傳序》: 體用一源，顯微無間",
            "https://plato.stanford.edu/entries/neo-confucianism/",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "BAIMA",
        name: "白馬非馬 — The White Horse",
        tradition: Tradition::Chinese,
        failure_mode: "sliding between a type and its tokens, or between intension and extension",
        operation: "a claim quantifying over a class must declare that class's extension",
        applies_to: &[],
        gates: &[Gate {
            code: gates::CLASS_EXTENSION_UNDECLARED,
            check: gates::class_extension_declared,
        }],
        worked_example: "\"Amcache entries indicate execution\" quantifies over every entry, \
having examined one. Gongsun Long's point is that the class and the qualified subclass are not \
interchangeable, however natural the slide feels.",
        sources: &[
            "《公孫龍子·白馬論》: 白馬非馬",
            "https://plato.stanford.edu/entries/school-names/",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "CATUSKOTI",
        name: "四句 catuṣkoṭi — The Four Corners",
        tradition: Tradition::Buddhist,
        failure_mode: "collapsing a contested question into a binary before the other positions \
have been stated",
        operation: "a contested claim must address all four corners: A, ¬A, both, neither",
        applies_to: &[],
        gates: &[Gate {
            code: gates::CORNERS_UNADDRESSED,
            check: gates::four_corners_addressed,
        }],
        worked_example: "\"executed / not executed\" omits the corner that actually fits an \
Amcache record: catalogued without execution — neither cleanly one nor the other.",
        sources: &[
            "Nāgārjuna, Mūlamadhyamakakārikā 1.1, 18.8",
            "https://plato.stanford.edu/entries/nagarjuna/",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "TOULMIN",
        name: "Toulmin — Name the Warrant",
        tradition: Tradition::Modern,
        failure_mode: "the unstated warrant: grounds and claim are given, the rule connecting \
them never is",
        operation: "`warrant` is a required field, not an optional one",
        applies_to: &[],
        gates: &[Gate {
            code: gates::WARRANT_MISSING,
            check: gates::warrant_present,
        }],
        worked_example: "From \"the hive holds this path\" to \"the program ran\" there is an \
unwritten rule licensing the step. Written down, it is visibly false.",
        sources: &[
            "Stephen Toulmin, The Uses of Argument (1958), ch. 3",
            "https://plato.stanford.edu/entries/reasoning-defeasible/",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "PRAMANA",
        name: "प्रमाण pramāṇa — Type the Means of Knowing",
        tradition: Tradition::Indian,
        failure_mode: "testimony passed off as observation, and corroboration mistaken for \
independence",
        operation: "evidence is typed, and each type caps the grade an edge may carry",
        applies_to: &[],
        gates: &[Gate {
            code: gates::GRADE_EXCEEDS_PRAMANA,
            check: gates::grades_within_pramana_ceiling,
        }],
        worked_example: "Two parsers agreeing on a hive is śabda corroboration, not pratyakṣa. \
If they vendor the same decoding library they are not independent at all, and no count of them \
reaches G4.",
        sources: &[
            "Nyāya Sūtra 1.1.3 (pratyakṣa, anumāna, upamāna, śabda)",
            "https://plato.stanford.edu/entries/epistemology-india/",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "RUNG",
        name: "Causal Ladder — Earn the Rung",
        tradition: Tradition::Modern,
        failure_mode: "asserting intervention or counterfactual conclusions from observational \
data, and stating a conclusion with no boundary conditions",
        operation: "a claim above the association rung requires an executed protocol supporting \
it; every claim declares its boundaries",
        applies_to: &[],
        gates: &[
            Gate {
                code: gates::CAUSAL_RUNG_UNREACHED,
                check: gates::causal_rung_earned,
            },
            Gate {
                code: gates::BOUNDARIES_MISSING,
                check: gates::boundaries_declared,
            },
        ],
        worked_example: "\"This Amcache entry proves execution\" is a rung-3 assertion resting \
on rung-1 data, with no Windows build named. Nothing was intervened on; nothing was compared \
against a world where the file was only copied.",
        sources: &[
            "Judea Pearl, The Book of Why (2018), ch. 1 — the ladder of causation",
            "Austin Bradford Hill, Proc. R. Soc. Med. 58 (1965) 295",
        ],
        phase: Phase::Enforced,
    },
    // ── Catalogued, not yet mechanised ───────────────────────────────────────
    Lens {
        id: "ELENCHUS",
        name: "ἔλεγχος elenchus — Socratic Cross-Examination",
        tradition: Tradition::Greek,
        failure_mode: "premises that were never examined because nobody asked",
        operation: "six question families spawn child Question nodes; none may remain open at \
review",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "Clarification, assumption, evidence, viewpoint, implication, and the \
question about the question — each generating a child that must be answered or explicitly \
dismissed.",
        sources: &[
            "Plato, Meno 79e–86c; Gorgias 471d–479e",
            "https://plato.stanford.edu/entries/plato-ethics-shorter/",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "ACH",
        name: "Analysis of Competing Hypotheses",
        tradition: Tradition::Modern,
        failure_mode: "confirmation by consistency — collecting evidence that fits the favoured \
hypothesis without asking what it rules out",
        operation: "hypotheses are eliminated by inconsistency, weighted by diagnosticity",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "Evidence consistent with execution is usually also consistent with \
installation and with scanning, so it is not diagnostic and should carry no weight either way.",
        sources: &[
            "Richards J. Heuer Jr., Psychology of Intelligence Analysis (CIA CSI, 1999), ch. 8",
            "https://www.cia.gov/resources/csi/books-monographs/psychology-of-intelligence-analysis-2/",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "PANCAVAYAVA",
        name: "पञ्चावयव pañcāvayava — The Five-Membered Argument",
        tradition: Tradition::Indian,
        failure_mode: "a reason that looks valid but is unestablished, contradictory, \
inconclusive, counterbalanced, or already defeated",
        operation: "arguments take the five members, and reasons are screened against the \
hetvābhāsa taxonomy",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "pratijñā, hetu, udāharaṇa, upanaya, nigamana — with the reason tested \
for asiddha, viruddha, anaikāntika, satpratipakṣa and bādhita.",
        sources: &[
            "Nyāya Sūtra 1.1.32–1.1.39; 1.2.4–1.2.9",
            "https://plato.stanford.edu/entries/logic-india/",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "STEELMAN",
        name: "Rapoport's Rules — Steelman First",
        tradition: Tradition::Modern,
        failure_mode: "attacking a position its holder would not recognise",
        operation: "a `contradicts` edge requires a steelman the opponent would accept",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "State the opposing case so well that its holder thanks you, before \
saying a word against it.",
        sources: &[
            "Daniel Dennett, Intuition Pumps and Other Tools for Thinking (2013), ch. 3",
            "Anatol Rapoport, Fights, Games and Debates (1960)",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "DOUBLECRUX",
        name: "Double Crux",
        tradition: Tradition::Modern,
        failure_mode: "disagreement that circles because the load-bearing belief was never located",
        operation: "find the belief whose reversal would reverse the conclusion, for both parties",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "If neither side can name what would change their mind, the dispute is \
not yet about anything checkable.",
        sources: &["https://www.lesswrong.com/posts/exa5kmvopeRyfJgCy/double-crux-a-strategy-for-resolving-disagreement"],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "MACHLOKET",
        name: "מחלוקת machloket — Preserve the Minority",
        tradition: Tradition::Jewish,
        failure_mode: "deleting the losing argument, so the reasoning that rejected it becomes \
unreviewable",
        operation: "rejecting a claim requires recording the rejected position as a Dissent node",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "The Mishnah records minority opinions that were not adopted, precisely \
so a later court can see what was weighed. A knowledge base that deletes them keeps the verdict \
and loses the case.",
        sources: &[
            "Mishnah Eduyot 1:5–1:6",
            "https://www.sefaria.org/Mishnah_Eduyot.1.5",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "AUFHEBUNG",
        name: "Aufhebung — Synthesis That Preserves",
        tradition: Tradition::Modern,
        failure_mode: "a synthesis that quietly discards what it claimed to reconcile",
        operation: "a synthesis node must `sublates` both parents and state what it keeps from each",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "Aufheben carries all three senses at once — to cancel, to preserve, to \
lift up. A synthesis citing only one parent has merely picked a side.",
        sources: &[
            "Hegel, Wissenschaft der Logik, Bk I §§185–188",
            "https://plato.stanford.edu/entries/hegel-dialectics/",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "THESEUS",
        name: "Ship of Theseus — Amend or Supersede",
        tradition: Tradition::Greek,
        failure_mode: "silent identity drift: a claim's meaning changes across edits while its \
id, and everything citing it, stays put",
        operation: "editing an accepted claim's proposition forces an explicit amend-or-supersede \
decision",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "forensicnomicon's ChangeType already types this for artifacts, and its \
BehaviorChanged variant — semantics changed without format change — is exactly the drift that \
leaves every downstream citation pointing at something it no longer says.",
        sources: &[
            "Plutarch, Life of Theseus 23.1",
            "https://plato.stanford.edu/entries/identity-over-time/",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "CHESTERTON",
        name: "Chesterton's Fence",
        tradition: Tradition::Modern,
        failure_mode: "removing something without recovering why it was put there",
        operation: "a `supersedes` edge requires a recorded reason the prior existed",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "Before the fence comes down, the reason it went up must be stated — \
not because the reason is necessarily good, but because not knowing it is not an argument.",
        sources: &["G. K. Chesterton, The Thing (1929), ch. 4 \"The Drift from Domesticity\""],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "PREMORTEM",
        name: "Premortem / Inversion",
        tradition: Tradition::Modern,
        failure_mode: "no recorded falsifier, so nothing could ever count as being wrong",
        operation: "promotion requires at least one stated condition that would defeat the claim",
        applies_to: &[],
        gates: &[Gate {
            code: gates::FALSIFIER_MISSING,
            check: gates::falsifier_declared,
        }],
        worked_example: "Assume it is a year later and the conclusion collapsed; say what did it. \
A claim with no answer is not yet a claim.",
        sources: &[
            "Gary Klein, Harvard Business Review, September 2007, \"Performing a Project Premortem\"",
            "Karl Popper, Logik der Forschung (1934), §6",
        ],
        phase: Phase::Enforced,
    },
    Lens {
        id: "ERDI",
        name: "二諦 — The Two Truths, and Court Mode",
        tradition: Tradition::Buddhist,
        failure_mode: "a courtroom sentence that asserts more than the graph behind it supports",
        operation: "the exported safe statement must be entailed by the grounded extension",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "The conventional register (世俗諦) is what may be said aloud; the \
ultimate register (勝義諦) is the fully-bounded graph. Court Mode is the disciplined translation \
between them, and it may only ever lose strength, never gain it.",
        sources: &[
            "Nāgārjuna, Mūlamadhyamakakārikā 24.8–24.10",
            "https://plato.stanford.edu/entries/twotruths-india/",
        ],
        phase: Phase::Catalogued,
    },
    Lens {
        id: "DUNG",
        name: "Grounded Extension — Compute, Don't Assert",
        tradition: Tradition::Formal,
        failure_mode: "dialectical status asserted by whoever wrote last, rather than computed \
from the attack relation",
        operation: "claim standing is the least fixed point of the characteristic function",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "c defeats b, which reinstates a. No participant need agree; the result \
follows from the attack graph, and grounded semantics refuses to pick a winner in a stand-off.",
        sources: &[
            "P. M. Dung, Artificial Intelligence 77 (1995) 321–357",
            "https://doi.org/10.1016/0004-3702(94)00041-X",
        ],
        phase: Phase::Enforced,
    },
];

#[cfg(test)]
mod meta_tests {
    //! Tests over the *knowledge*, not over a code path — the payoff of holding the
    //! catalogue as data. Borrowed wholesale from forensicnomicon, which asserts the
    //! same class of property over its artifact tables.

    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_lens_cites_at_least_one_source() {
        for l in CATALOG {
            assert!(!l.sources.is_empty(), "{} cites nothing", l.id);
            for s in l.sources {
                assert!(!s.trim().is_empty(), "{} has a blank source", l.id);
            }
        }
    }

    #[test]
    fn every_lens_has_a_worked_example_of_the_gate_firing() {
        for l in CATALOG {
            assert!(
                l.worked_example.len() > 40,
                "{} has no substantive worked example — a catalogue entry must be \
falsifiable, not merely plausible",
                l.id
            );
        }
    }

    #[test]
    fn lens_ids_are_unique() {
        let ids: BTreeSet<_> = CATALOG.iter().map(|l| l.id).collect();
        assert_eq!(ids.len(), CATALOG.len(), "duplicate lens id");
    }

    #[test]
    fn gate_codes_are_unique_across_the_catalogue() {
        let mut seen = BTreeSet::new();
        for l in CATALOG {
            for g in l.gates {
                assert!(seen.insert(g.code), "duplicate gate code {}", g.code);
            }
        }
    }

    #[test]
    fn gate_codes_are_scheme_prefixed() {
        for l in CATALOG {
            for g in l.gates {
                assert!(
                    g.code.starts_with("PEIR-"),
                    "{} is not scheme-prefixed; codes are a published contract",
                    g.code
                );
            }
        }
    }

    #[test]
    fn an_enforced_lens_actually_enforces_something() {
        for l in CATALOG.iter().filter(|l| l.phase == Phase::Enforced) {
            assert!(
                !l.gates.is_empty() || l.id == "DUNG",
                "{} claims to be enforced but owns no gate",
                l.id
            );
        }
    }

    #[test]
    fn a_catalogued_lens_does_not_pretend_to_enforce() {
        for l in CATALOG.iter().filter(|l| l.phase == Phase::Catalogued) {
            assert!(
                l.gates.is_empty(),
                "{} is marked Catalogued but owns gates — say Enforced or drop them",
                l.id
            );
        }
    }

    #[test]
    fn every_lens_names_a_failure_mode_and_an_operation() {
        for l in CATALOG {
            assert!(!l.failure_mode.trim().is_empty(), "{}", l.id);
            assert!(!l.operation.trim().is_empty(), "{}", l.id);
        }
    }

    #[test]
    fn the_eight_mvp_lenses_are_all_enforced() {
        for id in [
            "LIJI",
            "ZHENGMING",
            "TIYONG",
            "BAIMA",
            "CATUSKOTI",
            "TOULMIN",
            "PRAMANA",
            "RUNG",
        ] {
            let l = lens(id).unwrap_or_else(|| panic!("{id} missing from the catalogue"));
            assert_eq!(l.phase, Phase::Enforced, "{id} must be enforced");
            assert!(!l.gates.is_empty(), "{id} must own a gate");
        }
    }

    #[test]
    fn lookup_finds_what_the_catalogue_holds() {
        assert!(lens("LIJI").is_some());
        assert!(lens("NOT-A-LENS").is_none());
    }

    #[test]
    fn every_tradition_renders_its_name() {
        for t in [
            Tradition::Greek,
            Tradition::Chinese,
            Tradition::Indian,
            Tradition::Buddhist,
            Tradition::Jewish,
            Tradition::Modern,
            Tradition::Formal,
        ] {
            assert!(!t.as_str().is_empty());
            assert_eq!(t.to_string(), t.as_str());
        }
    }

    #[test]
    fn the_catalogue_draws_on_more_than_one_tradition() {
        // A "universal" catalogue sourced from one lineage would be a claim this
        // project could not make honestly.
        let traditions: BTreeSet<_> = CATALOG.iter().map(|l| l.tradition.as_str()).collect();
        assert!(
            traditions.len() >= 5,
            "only {} traditions represented: {traditions:?}",
            traditions.len()
        );
    }

    #[test]
    fn a_lens_reports_not_applicable_outside_its_scope() {
        use peira_core::{parse_node, Graph};
        let term =
            parse_node("---\nid: 60.01\ntype: term\ntitle: presence\n---\n").expect("parses");
        let graph = Graph::new();
        let liji = lens("LIJI").expect("LIJI exists");
        assert_eq!(
            liji.examine(&graph, &term),
            vec![GateResult::NotApplicable],
            "立極 examines arguments, not reference material"
        );
    }

    #[test]
    fn a_violation_renders_gate_lens_subject_and_remedy() {
        let v = Violation {
            gate: "PEIR-TEST",
            lens: "TEST",
            subject: peira_core::NodeId::new("c1"),
            detail: "what was found".to_owned(),
            remedy: "what to do".to_owned().leak(),
        };
        let rendered = v.to_string();
        for part in ["PEIR-TEST", "TEST", "c1", "what was found", "what to do"] {
            assert!(rendered.contains(part), "missing {part} in {rendered}");
        }
    }

    /// A gate that reached no verdict must reach the decision point.
    ///
    /// `permits_promotion()` correctly returns false for `Unassessed`, and until this
    /// test existed it had no production caller: `examine_graph` kept only results
    /// carrying a `Violation`, so a no-verdict result vanished before `freeze`,
    /// `status` or `gates` could act on it. A packet then froze over three silent
    /// gates while asserting "All enforced gates pass".
    ///
    /// Asserted through the public aggregation, not the predicate, because the
    /// predicate was already right — it was the join that discarded it.
    /// The kind filter must not sit in front of the load-bearing test.
    ///
    /// `under_promotion` decides whether a node is examined — but `Lens::examine`
    /// applies `applies_to` FIRST, so its non-argument arm never ran on the shipping
    /// path. A universal over-claim labelled `type: observation` drew zero findings and
    /// froze; the identical node labelled `type: hypothesis` drew six and was refused.
    ///
    /// Asserted through `examine_graph` — the JOIN — and not by calling the gate
    /// directly. The round-3 test for this passed precisely because it called the gate
    /// directly, which is how the same defect survived being fixed once already.
    #[test]
    fn a_load_bearing_assertion_is_examined_whatever_kind_it_declares() {
        use peira_core::{parse_node, Edge, EdgeKind, Graph, NodeId};

        let counts = |kind: &str| {
            let mut g = Graph::new();
            g.insert_node(parse_node("---\nid: c1\ntype: claim\ntitle: t\n---\n").unwrap());
            g.insert_node(
                parse_node(&format!(
                    "---\nid: o9\ntype: {kind}\n\
title: Every entry on every version is written at execution time\n\
quantifier: universal\n---\n"
                ))
                .unwrap(),
            );
            g.insert_edge(Edge::new(
                NodeId::new("o9"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            ));
            examine_graph(&g)
                .into_iter()
                .filter(|v| v.subject.as_str() == "o9")
                .count()
        };

        let as_hypothesis = counts("hypothesis");
        assert!(
            as_hypothesis > 0,
            "a load-bearing assertion must be examined"
        );
        assert_eq!(
            counts("observation"),
            as_hypothesis,
            "the SAME node must be examined the same way whatever `type:` it declares — \
the kind is a self-declared string, and relabelling it must not strip the obligations"
        );
    }

    #[test]
    fn a_gate_that_reached_no_verdict_is_not_silently_dropped() {
        use peira_core::{parse_node, Graph};

        // Declares warrant, boundaries and falsifier — but no `uses_term`, no
        // `quantifier`, no `causal_rung`. Those three gates cannot reach a verdict,
        // which is precisely the cheapest evasion: write less.
        let mut g = Graph::new();
        g.insert_node(
            parse_node(
                "---\nid: c1\ntype: claim\ntitle: The suspect executed the payload\n\
warrant: The record establishes execution.\n\
boundaries:\n  - everywhere\nfalsifier:\n  - nothing known\n---\n",
            )
            .expect("fixture parses"),
        );

        let found = examine_graph(&g);
        let unassessed: Vec<&Violation> = found
            .iter()
            .filter(|v| v.gate == "PEIR-GATE-UNASSESSED")
            .collect();

        assert!(
            unassessed.len() >= 3,
            "ZHENGMING, BAIMA and RUNG each reach no verdict here; \
examine_graph reported {} no-verdict result(s) out of {} findings",
            unassessed.len(),
            found.len()
        );
        assert!(
            unassessed.iter().all(|v| !v.detail.is_empty()),
            "a no-verdict finding must carry the reason the gate could not run"
        );
    }

    #[test]
    fn examine_graph_returns_nothing_for_an_empty_graph() {
        // Vacuously clean, and it must not be confusable with "checked and passed" —
        // which is why the CLI reports counts rather than a bare tick for a vault it
        // could not find.
        assert!(examine_graph(&peira_core::Graph::new()).is_empty());
    }

    /// A claim must say what would make it wrong.
    ///
    /// Written against the public verdict rather than against a gate function, so it
    /// compiles before the gate exists and fails because the examination does not
    /// happen — not because a symbol is missing.
    ///
    /// The three cases are one contract: the demand is for a recorded defeater, and
    /// a defeater already recorded AS A NODE satisfies it. Requiring a duplicate
    /// string as well would be bookkeeping, not examination.
    #[test]
    fn a_claim_with_no_recorded_falsifier_is_blocked() {
        use peira_core::{parse_node, Edge, EdgeKind, Graph, NodeId};

        // Scoped to a subject, not counted graph-wide: an attacker node is itself a
        // claim, and is itself subject to this gate, so a graph-wide count answers a
        // different question than the one each case asks.
        let falsifier_findings = |g: &Graph, subject: &str| {
            examine_graph(g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-FALSIFIER-MISSING" && v.subject.as_str() == subject)
                .count()
        };

        // Everything else the enforced gates ask for is present, so the only thing
        // under test is the falsifier.
        let complete = "warrant: A catalogue entry evidences that the path was recorded.\n\
quantifier: singular\naspect: function\ncausal_rung: association\n\
boundaries:\n  - Windows 10 1809 and later\n";

        let mut bare = Graph::new();
        bare.insert_node(
            parse_node(&format!(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n{complete}---\n"
            ))
            .expect("fixture parses"),
        );
        assert_eq!(
            falsifier_findings(&bare, "c1"),
            1,
            "a claim stating no condition that would defeat it must be blocked"
        );

        let mut stated = Graph::new();
        stated.insert_node(
            parse_node(&format!(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n{complete}\
falsifier:\n  - the entry predates the file's creation timestamp\n---\n"
            ))
            .expect("fixture parses"),
        );
        assert_eq!(
            falsifier_findings(&stated, "c1"),
            0,
            "a stated falsifier satisfies the gate"
        );

        let mut attacked = Graph::new();
        attacked.insert_node(
            parse_node(&format!(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n{complete}---\n"
            ))
            .expect("fixture parses"),
        );
        attacked.insert_node(
            parse_node(
                "---\nid: c2\ntype: claim\ntitle: The entry predates the file\naspect: function\n---\n",
            )
            .expect("fixture parses"),
        );
        attacked.insert_edge(Edge::new(
            NodeId::new("c2"),
            NodeId::new("c1"),
            EdgeKind::Contradicts,
        ));
        assert_eq!(
            falsifier_findings(&attacked, "c1"),
            0,
            "a defeater recorded as a node satisfies the gate without a duplicate field"
        );
        assert_eq!(
            falsifier_findings(&attacked, "c2"),
            1,
            "the attacker is itself a claim and is itself held to the same demand"
        );
    }

    #[test]
    fn enforced_is_a_strict_subset_of_the_catalogue() {
        let n = enforced().count();
        assert!(n > 0, "nothing is enforced");
        assert!(n < CATALOG.len(), "everything claims to be enforced");
    }

    #[test]
    fn a_block_result_yields_its_violation_and_others_do_not() {
        let v = Violation {
            gate: "PEIR-TEST",
            lens: "TEST",
            subject: peira_core::NodeId::new("c1"),
            detail: String::new(),
            remedy: "",
        };
        assert!(GateResult::Block(v).violation().is_some());
        assert!(GateResult::Pass.violation().is_none());
        assert!(GateResult::NotApplicable.violation().is_none());
        assert!(GateResult::Unassessed { why: String::new() }
            .violation()
            .is_none());
    }

    #[test]
    fn unassessed_never_permits_promotion() {
        let u = GateResult::Unassessed {
            why: "no data".to_owned(),
        };
        assert!(
            !u.permits_promotion(),
            "an empty check means NOT RUN, never PASSED"
        );
        assert!(GateResult::Pass.permits_promotion());
        assert!(GateResult::NotApplicable.permits_promotion());
    }
}
