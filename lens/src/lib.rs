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

use peira_core::{EdgeKind, Graph, Node, NodeId, NodeKind};
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
    /// Buddhist — Madhyamaka, 金剛經, 二諦, and Dignāgan 因明/pramāṇavāda.
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
    /// Stable machine id, e.g. `CRITERION`.
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
    /// A short, source-checked note on the tradition the lens comes from and a famous
    /// usage — empty until written. Every claim here must trace to a cited source, not
    /// to recollection; an unsourced background paragraph is the overclaim this project
    /// exists to refuse. Rendered as an "In the tradition" section where non-empty.
    pub background: &'static str,
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

/// Whether anything in the graph leans on this node.
///
/// **The single definition of load-bearing.** Three functions used to answer this
/// question and none agreed with the others — `gates::under_promotion`,
/// `lints::ungraded_support` and the closure walk in `court` — and they had drifted
/// apart by accretion, each defect fixed in isolation. Every audit round found at least
/// one bug that was exactly that drift: a node in the closure but examined by nothing,
/// a supporter of a declared prerequisite examined by one walk and not the other,
/// `depends_on` followed in the wrong direction by one of them.
///
/// The rule, and the reason for each part:
///
/// - **A `Claim` always.** It asserts by existing; nothing has to lean on it.
/// - **Weight that REACHES a claim.** Following any outgoing support counted a
///   hypothesis supporting a bare sketch as load-bearing, so thinking out loud in two
///   steps was held to the promotion bar. The walk is transitive and stops at a claim.
/// - **An incoming `depends_on`.** `c1 --depends_on--> h1` says c1 cannot hold without
///   h1, so h1 carries c1's weight. This is the direction that was missed twice: the
///   edge that makes a node load-bearing can point either way.
///
/// Deliberately NOT a node-kind test. Whether a node carries weight is a property of
/// the edges around it, and a kind test has been removed from this codebase three times
/// and reintroduced twice. `court`'s closure asks a different question — *what does this
/// packet rest on and render* — and is named for it rather than sharing this one.
#[must_use]
pub fn carries_weight(graph: &Graph, node: &Node) -> bool {
    if node.kind == NodeKind::Claim {
        return true;
    }
    let mut seen: std::collections::BTreeSet<&NodeId> = std::collections::BTreeSet::new();
    let mut stack = vec![&node.id];
    while let Some(n) = stack.pop() {
        if !seen.insert(n) {
            continue;
        }
        for e in graph.edges_from(n).filter(|e| e.kind == EdgeKind::Supports) {
            if graph.node(&e.to).is_some_and(|d| d.kind == NodeKind::Claim) {
                return true;
            }
            stack.push(&e.to);
        }
        for e in graph.edges_to(n).filter(|e| e.kind == EdgeKind::DependsOn) {
            if graph
                .node(&e.from)
                .is_some_and(|d| d.kind == NodeKind::Claim)
            {
                return true;
            }
            stack.push(&e.from);
        }
    }
    false
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
        id: "CRITERION",
        name: "立極 — Set the Pole",
        tradition: Tradition::Chinese,
        failure_mode: "judging something without ever declaring the standard judged against",
        operation: "an evaluative claim requires a `judged_by` edge to a Criterion",
        applies_to: &[],
        gates: &[Gate {
            code: gates::CRITERION_UNDECLARED,
            check: gates::criterion_declared,
        }],
        worked_example: "\"A healthy breakfast\" names no standard — healthy for blood sugar, for \
weight, or for a day's hard labour? Until the pole is set, the word carries the judgement and the \
reader cannot tell what would count against it.",
        sources: &[
            "周敦頤《太極圖說》: 聖人定之以中正仁義而主靜，立人極焉",
            "https://ctext.org/wiki.pl?if=en&chapter=592302",
            "https://iep.utm.edu/neo-confucian-philosophy/",
        ],
        background: "立極 — to establish the pole — is from 周敦頤 (1017–1073) and his 太極圖說: 「聖人定之以中正仁義而主靜，立人極焉」, the sage fixing the human pole by a stated standard. The lens borrows the move, not the metaphysics: a judgement must set the pole it is measured against, or the word carries the verdict and nothing can check it.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "RECTIFY-NAME",
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
        worked_example: "Is a tomato a fruit? Botany says yes, the kitchen says no, and Nix v. \
Hedden (1893) had the US Supreme Court call it a vegetable for tariff law. 所謂「水果」— what is \
called fruit; 即非「水果」— no one thing answers to the bare word; 是名「水果」— it is fruit only \
under a stated sense.",
        sources: &[
            "《金剛般若波羅蜜經》: 所謂佛法者，即非佛法，是名佛法",
            "https://ctext.org/analects/zi-lu (論語·子路: 必也正名乎)",
            "陳那, 集量論 ch. 5 (遮詮) — a term excludes what \
it is not, which is the 即非 moment reached from Buddhist epistemology; `not_essence` is that \
obligation mechanised, so this earns no separate entry",
            "https://plato.stanford.edu/entries/confucius/",
        ],
        background: "正名, the rectification of names, is stated in the Analects (13.3): asked what he would do first to govern, 孔子 answers 「必也正名乎」— if names are not rectified, speech does not accord with reality and nothing is accomplished. The lens joins it to the 金剛經's 所謂 X・即非 X・是名 X and to 陳那's doctrine that a word means by excluding what it is not — three traditions on one discipline: fix the term before it does the arguing.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "SUBSTANCE-FUNCTION",
        name: "體用 — Substance and Function",
        tradition: Tradition::Chinese,
        failure_mode: "reporting what a thing DID as though it established what a thing IS",
        operation: "a substance claim needs at least one substance-aspect support; function \
evidence alone cannot license it",
        // `&[]`, not ARGUMENTS. `Lens::examine` applies this BEFORE the gate runs, so a
        // static kind list here is a kind test doing scoping — and SUBSTANCE-FUNCTION was the last
        // enforced lens still carrying one. Relabelling a load-bearing substance claim
        // `type: observation` stripped the 體用 obligation entirely and it froze. The
        // gate already scopes itself by `under_promotion`, which asks whether the node
        // carries weight rather than what it calls itself.
        applies_to: &[],
        gates: &[Gate {
            code: gates::FUNCTION_AS_SUBSTANCE,
            check: gates::substance_not_from_function_alone,
        }],
        worked_example: "A smoke alarm sounding is what the device did; \"there is a fire\" is a \
claim about what is — and burnt toast sets it off just as well. The alarm reports its function; \
that a fire exists is a substance claim the sound alone does not carry.",
        sources: &[
            "程頤《易傳序》: 體用一源，顯微無間",
            "https://plato.stanford.edu/entries/neo-confucianism/",
        ],
        background: "體用 (substance and function) has roots in early Confucian and Daoist texts, was first used systematically by 王弼 (226–249) in his commentary on the 道德經, and spread as a hermeneutic partly through Buddhism. 程頤's Neo-Confucian formula 「體用一源，顯微無間」— substance and function are one source, the manifest and the subtle without gap — is its classic statement. The lens takes only the distinction: what a thing is (體) is not settled by what it did (用).",
        phase: Phase::Enforced,
    },
    Lens {
        id: "WHITE-HORSE",
        name: "白馬非馬 — The White Horse",
        tradition: Tradition::Chinese,
        failure_mode: "sliding between a type and its tokens, or between intension and extension",
        operation: "a claim quantifying over a class must declare that class's extension",
        applies_to: &[],
        gates: &[Gate {
            code: gates::CLASS_EXTENSION_UNDECLARED,
            check: gates::class_extension_declared,
        }],
        worked_example: "One white horse was examined, and the claim is made of horses at large. \
公孫龍's 白馬非馬 turns on exactly this: \"white horse\" and \"horse\" have different extensions, so \
what holds of the qualified subclass need not hold of the class, however natural the plural feels.",
        sources: &[
            "《公孫龍子·白馬論》: 白馬非馬",
            "https://plato.stanford.edu/entries/school-names/",
        ],
        background: "公孫龍 (fl. 284–259 BCE), of the Warring-States 名家 (the School of Names), argued in the 白馬論 that a white horse is not a horse: \"horse\" names a shape and \"white horse\" a shape-with-colour, so the two pick out different things and cannot be swapped. Usually taught as sophistry, its point is the one the lens enforces — natural language slides between a class and a qualified subclass.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "FOUR-CORNERS",
        name: "四句 — The Four Corners",
        tradition: Tradition::Buddhist,
        failure_mode: "collapsing a contested question into a binary before the other positions \
have been stated",
        operation: "a contested claim must address all four corners: A, ¬A, both, neither",
        applies_to: &[],
        gates: &[Gate {
            code: gates::CORNERS_UNADDRESSED,
            check: gates::four_corners_addressed,
        }],
        worked_example: "\"Is light a wave or a particle?\" forces two corners, and each is wrong \
alone — the physics needed the third: it is both. 四句 keeps all four (is, is-not, both, neither) \
open until the question earns a collapse.",
        sources: &[
            "龍樹《中論》鳩摩羅什譯 (T30n1564), 觀因緣品第一・觀法品第十八",
            "https://plato.stanford.edu/entries/nagarjuna/ (龍樹, Stanford Encyclopedia of Philosophy)",
        ],
        background: "The four-cornered logic (四句; चतुष्कोटि) sets out four alternatives on a proposition: that it holds, that it fails to hold, that it does both, and that it does neither. It predates 中觀 — the early canon has the Buddha decline all four on the \"undeclared\" questions, as in the Aggi-Vacchagotta Sutta on the fate of an awakened one after death. 龍樹 (c. 150–250 CE) makes it a method in his root verses on the Middle Way; his use is not uniform, but the signature move denies all four, as in the opening verse refusing that anything arises from itself, from another, from both, or from neither.",
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
        worked_example: "Toulmin's own case: from \"Harry was born in Bermuda\" to \"Harry is a \
British subject\" the step rides on an unwritten warrant — the law that a Bermudan is a British \
subject. Written down, a warrant can be checked or shown false; left unwritten, it just works.",
        sources: &[
            "Stephen Toulmin, The Uses of Argument (1958), ch. 3",
            "https://plato.stanford.edu/entries/reasoning-defeasible/",
        ],
        background: "Stephen Toulmin, in The Uses of Argument (1958), held that real reasoning is not the syllogism but a structure of claim, grounds, and — crucially — a warrant: the inference-licence saying why the grounds bear on the claim, itself resting on backing and softened by qualifiers and rebuttals. His own case runs from \"Harry was born in Bermuda\" to \"Harry is a British subject\" via the warrant of Bermudan law. The lens makes the warrant a required field: the step from grounds to claim must be written, not assumed.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "MEANS-OF-KNOWING",
        name: "प्रमाण — Type the Means of Knowing",
        tradition: Tradition::Indian,
        failure_mode: "testimony passed off as observation, and corroboration mistaken for \
independence",
        operation: "evidence is typed, and each type caps the grade an edge may carry",
        applies_to: &[],
        gates: &[Gate {
            code: gates::GRADE_EXCEEDS_MEANS,
            check: gates::grades_within_means_ceiling,
        }],
        worked_example: "Two newspapers running the same wire story are testimony, not two eyes \
on the event. Sharing one source, they are a single line of evidence however many mastheads carry \
it, and no tally of them reaches direct knowledge.",
        sources: &[
            "《正理經》(喬答摩) 1.1.3 — perception, inference, comparison, testimony",
            "https://plato.stanford.edu/entries/epistemology-india/",
        ],
        background: "In classical Indian epistemology a प्रमाण is a means of valid knowledge, counted as four: perception, inference, comparison and testimony. The Buddhist logicians 陳那 and 法稱 (5th–7th c.) pared these to two — perception and inference — folding testimony into inference, and holding that no source certifies more than its own kind supports. The lens keeps that ceiling: testimony cannot be promoted to observation, and repetition of one source is not independence.",
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
        worked_example: "Ice-cream sales and drownings rise together, but \"ice cream causes \
drowning\" is a rung-3 claim resting on rung-1 correlation — the common cause is summer. Nothing \
was intervened on, and no world was compared where sales were forced up out of season.",
        sources: &[
            "Judea Pearl, The Book of Why (2018), ch. 1 — the ladder of causation",
            "Austin Bradford Hill, Proc. R. Soc. Med. 58 (1965) 295",
            "https://plato.stanford.edu/entries/causal-models/",
        ],
        background: "Judea Pearl's ladder of causation (The Book of Why, 2018) has three rungs — association (seeing), intervention (doing) and counterfactual (imagining) — and a claim on a higher rung cannot be earned from data on a lower one: correlation does not reach causation without an intervention or a model. Austin Bradford Hill's 1965 criteria were an earlier discipline for the same gap. The lens holds the ladder: a causal claim earns its rung or is restated at the rung its evidence supports.",
        phase: Phase::Enforced,
    },
    // ── Catalogued, not yet mechanised ───────────────────────────────────────
    Lens {
        id: "CROSS-EXAMINE",
        name: "ἔλεγχος — Socratic Cross-Examination",
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
        background: "ἔλεγχος — putting-to-the-test — is Socrates' method in Plato's early dialogues (Euthyphro, Laches, Charmides, Meno, Gorgias): from the interlocutor's own admissions he draws claims that jointly contradict the thesis under test, so the position falls under premises its holder granted, not under Socrates' own. The lens keeps the move — a claim is examined by the questions its own commitments must answer.",
        phase: Phase::Catalogued,
    },
    Lens {
        id: "ACH",
        name: "Analysis of Competing Hypotheses",
        tradition: Tradition::Modern,
        failure_mode: "confirmation by consistency — collecting evidence that fits the favoured \
hypothesis without asking what it rules out",
        operation: "an explanation above the association rung must name something it was \
tested against; diagnosticity is THREE-MARKS's half of the same rule",
        // No kind list: `examine` applies one BEFORE the gate, so it is an evasion by
        // relabelling. `rivals_enumerated` scopes itself through `under_promotion`.
        applies_to: &[],
        gates: &[Gate {
            code: gates::RIVALS_UNENUMERATED,
            check: gates::rivals_enumerated,
        }],
        worked_example: "Wet grass at dawn is consistent with overnight rain — and equally with \
the sprinkler timer. Evidence that fits every hypothesis discriminates none, so it should move the \
verdict toward neither; gathering more of it only feels like progress.",
        sources: &[
            "Richards J. Heuer Jr., Psychology of Intelligence Analysis (CIA CSI, 1999), ch. 8",
            "https://www.cia.gov/resources/csi/books-monographs/psychology-of-intelligence-analysis-2/",
        ],
        background: "Richards J. Heuer Jr. built ACH for CIA analysts (Psychology of Intelligence Analysis, 1999): list the hypotheses first, then score each piece of evidence by how well it discriminates among them — because the mind fixates on a favoured hypothesis and gathers what fits it. Evidence consistent with every hypothesis has no diagnostic value, however much accrues. The lens enforces it: a causal claim must name what it was tested against.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "FIVE-MEMBERS",
        name: "पञ्चावयव — The Five-Membered Argument",
        tradition: Tradition::Indian,
        failure_mode: "a reason that looks valid but is unestablished, contradictory, \
inconclusive, counterbalanced, or already defeated",
        operation: "arguments take the five members, and reasons are screened against the \
taxonomy of faulty reasons (似因)",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "The five members — thesis, reason, example, application, conclusion — \
with the reason screened for the classic faults: unestablished, contradictory, inconclusive, \
counterbalanced, already-defeated. 世親's 如實論 argues this five-member form carrying only a \
proto-因三相: the 古因明 stage, and the precursor to 陳那's reduction to three members \
(THREE-MARKS, SEMBLANCE), never its originator.",
        sources: &[
            "《正理經》(喬答摩) 1.1.32–1.1.39; 1.2.4–1.2.9",
            "世親《如實論・反質難品》真諦譯 c. 550, T32n1633 — the 古因明 five-member form",
            "https://plato.stanford.edu/entries/logic-india/",
        ],
        background: "The classical Indian demonstration has five members — the thesis, the reason, an example, the application and the conclusion — and screens the reason against the fallacy list. 世親's 如實論 carries this older five-member form (古因明) before 陳那 reduced it to three. The lens keeps the demand that an argument show all its members, not merely assert its conclusion.",
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
        background: "Anatol Rapoport's rules for criticism, popularised by Daniel Dennett (Intuition Pumps and Other Tools for Thinking, 2013), begin: state your opponent's position so clearly and fairly that they say \"thank you, I wish I'd put it that way\" — and only then criticise. The steelman is the straw man's opposite. The lens keeps it: an attack on a position must first restate it in a form its holder would accept.",
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
        background: "Double Crux, developed at CFAR and set out on LessWrong (2016), is a method for a disagreement that circles: each side finds the belief that, if it flipped, would flip their conclusion too — the shared crux — and the argument moves there rather than everywhere at once. If neither side can name what would change its mind, the dispute is not yet about anything checkable. The lens keeps the demand: locate the load-bearing belief before arguing.",
        phase: Phase::Catalogued,
    },
    Lens {
        id: "PRESERVE-MINORITY",
        name: "מחלוקת — Preserve the Minority",
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
        background: "Rabbinic tradition preserves the losing side of a dispute. The Mishnah (Eduyot 1:5–6) asks why it records minority opinions that were overruled, and answers: so a later court, finding the reasoning apt, may rely on them — a rejected view is kept, not erased. Pirkei Avot distinguishes a מחלוקת for the sake of heaven, which endures, from one that is not. The lens follows: rejecting a claim requires recording the rejected position, so the reasoning that dismissed it stays reviewable.",
        phase: Phase::Catalogued,
    },
    Lens {
        id: "SYNTHESIS",
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
        background: "Hegel's aufheben fuses three senses at once — to cancel, to preserve, and to lift up — so a sublation (Aufhebung) negates a stage's limitation while keeping its content and raising it into a new unity, not splitting the difference. (The tidy \"thesis–antithesis–synthesis\" triad is not Hegel's: it comes from Fichte, was codified by Chalybäus in 1843 after Hegel's death, and Hegel rejected such schematisation.) The lens keeps the demand: a synthesis must carry forward what it claims to reconcile from both parents, not quietly drop one.",
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
        worked_example: "Theseus's ship has every plank replaced yet keeps its name and its \
berth; whether it is still the same ship cannot be settled without a rule for identity across \
change. A claim edited past its evidence is that ship — the id and every citation stay put while \
the proposition drifts. When the IAU redefined \"planet\" in 2006 and Pluto fell out, every \
earlier \"nine planets\" silently came to mean something its readers never wrote.",
        sources: &[
            "Plutarch, Life of Theseus 23.1",
            "https://plato.stanford.edu/entries/identity-over-time/",
        ],
        background: "Plutarch (Life of Theseus 23.1) reports the ship the Athenians preserved by replacing each decayed plank, and the philosophers' question: is it the same ship? Hobbes sharpened it — gather the discarded planks, rebuild a second ship, and which is Theseus's? The puzzle is identity persisting under change while the name stays fixed. The lens keeps that edge: editing a claim's proposition while its id and every citation stay put is silent identity drift, and forces an amend-or-supersede choice.",
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
        background: "G. K. Chesterton's parable (The Thing, 1929, \"The Drift from Domesticity\"): if a fence stands across a road and a reformer cannot see why, the answer is not \"clear it away\" but \"go and find out why it was put there\" — only then may it come down. Not knowing the reason is not an argument against it. The lens keeps the rule: removing or superseding something requires first recovering the reason it existed.",
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
        worked_example: "Popper's mark of a real claim: \"all swans are white\" says something \
because a single black swan would refute it. Assume it is a year on and the conclusion collapsed — \
name what did it. A claim nothing could ever count against is a belief wearing a claim's clothes.",
        sources: &[
            "Gary Klein, Harvard Business Review, September 2007, \"Performing a Project Premortem\"",
            "Karl Popper, Logik der Forschung (1934), §6",
            "https://plato.stanford.edu/entries/popper/",
        ],
        background: "Two disciplines meet. Karl Popper (Logik der Forschung, 1934) made falsifiability the mark of an empirical claim: it must forbid something, so that some possible observation could refute it. Gary Klein's premortem (Harvard Business Review, 2007) turns the stance prospective — assume the plan has already failed and say what killed it. The lens requires the falsifier: a claim must state what would defeat it, or it is belief, not claim.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "THREE-MARKS",
        name: "因三相 — The Three Characteristics of a Valid Reason",
        tradition: Tradition::Buddhist,
        failure_mode: "a reason that also holds where the claim is false, so it proves nothing \
while looking like proof",
        operation: "support common to every side of a live contest carries no weight for any of \
them",
        applies_to: &[],
        gates: &[
            Gate {
                code: gates::REASON_UNDIAGNOSTIC,
                check: gates::reason_undiagnostic,
            },
            Gate {
                code: gates::CONTRARY_CASES_UNSURVEYED,
                check: gates::contrary_cases_surveyed,
            },
            Gate {
                code: gates::CONFIRMING_CASE_UNDECLARED,
                check: gates::confirming_case_declared,
            },
        ],
        worked_example: "The classic inference — \"the hill has fire, because it has smoke\" — \
holds because smoke is absent wherever fire is absent (no fire, no smoke). \"He has a fever, so he \
has the flu\" fails the same test: fever attends a hundred illnesses, present even where there is \
no flu (異品), so it discriminates nothing — 共不定, cell 1 of the 九句因.",
        sources: &[
            "商羯羅主《因明入正理論》玄奘譯 (647), T32n1630: 「因有三相…謂遍是宗法性、同品定有性、異品遍無性」",
            "陳那《因明正理門論本》玄奘譯, T32n1628",
            "法稱《正理滴論》II.5–7; tr. Th. Stcherbatsky, Buddhist Logic vol. II (1930)",
            "R. Hayes, Dignāga on the Interpretation of Signs (Kluwer 1988), ch. 4",
            "https://plato.stanford.edu/entries/logic-india/",
        ],
        background: "陳那's three marks of a valid reason (因三相) set what a reason must bear to prove anything: present in the case at issue, present in at least one similar case, and absent from every dissimilar case. The third is load-bearing — a reason that also occurs where the thesis is false proves nothing — which 陳那 mapped exhaustively in the 九句因 (the wheel of reasons). The lens enforces it: evidence common to a claim and its rival is not diagnostic.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "NON-PERCEPTION",
        name: "不可得因 — Non-Perception as a Reason",
        tradition: Tradition::Buddhist,
        failure_mode: "certifying an absence with a search never shown able to find the thing",
        operation: "an absence claim must rest on an instrument with a recorded positive control; \
naming no instrument reaches no verdict rather than passing",
        applies_to: &[],
        gates: &[Gate {
            code: gates::ABSENCE_UNCONTROLLED,
            check: gates::absence_is_controlled,
        }],
        worked_example: "You may say \"there is no elephant in the room\" — one would be seen. \
You may not say \"there are no bacteria\" from a look, because the eye cannot resolve them, so \
finding none reports the instrument, not the room. Non-perception establishes absence only of the \
perceptible; of the imperceptible it establishes nothing.",
        sources: &[
            "法稱《正理滴論》II.12–20 (the perceptibility restriction at II.13); tr. Th. \
Stcherbatsky, Buddhist Logic vol. II (1930)",
            "法稱《釋量論》為自比量品 (梵本 ed. Gnoli, Rome 1960; 法尊漢譯, 1980)",
            "呂澂《因明入正理論講解》中華書局 (1983) — the received Chinese terminology",
            "https://plato.stanford.edu/entries/dharmakiirti/ (法稱, Stanford Encyclopedia of Philosophy)",
        ],
        background: "法稱 (7th c.) admits non-perception (不可得) as a way to establish absence — but only under the perceivability condition: the thing, were it present, would have been perceived; the apparatus works; the conditions suffice. Of the imperceptible it proves nothing. The lens keeps the restriction: \"none found\" is evidence of absence only when the search could have found the thing.",
        phase: Phase::Enforced,
    },
    Lens {
        id: "SEMBLANCE",
        name: "似因・似宗 — The Semblance Taxonomies",
        tradition: Tradition::Buddhist,
        failure_mode: "a thesis or reason with the form of proof and not the force of it — \
unestablished, inconclusive, contradictory, or resting on terms the other party never granted",
        operation: "a human screen: each load-bearing inference is read against the fourteen \
似因 and the nine 似宗 before the packet leaves",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "能別不極成: one expert stipulates \"exfiltration\" as any copy to \
non-corporate storage, the other as transfer outside the tenant. Both proofs are internally \
valid, neither engages the other, and the tribunal receives two sound arguments about different \
words. peira has no party model, so 極成 is deliberately substituted by the 是名 disclosure — \
what cannot be verified (the opponent's assent) is replaced by what can be enforced (the \
admission that assent was never obtained).",
        sources: &[
            "商羯羅主《因明入正理論》T32n1630 — 似因十四過 (不成四・不定六・相違四), 似宗九過",
            "M. Tachikawa, \"A Sixth-Century Manual of Indian Logic\", J. Indian Philosophy 1 \
(1971) 111–145",
            "S. Katsura, \"The theory of anaikāntika in Buddhist logic\", in Studies in the \
Buddhist Epistemological Tradition (Vienna 1991)",
            "https://plato.stanford.edu/entries/logic-india/",
        ],
        background: "Indian logic catalogued the ways a proof can wear the form of validity without the force of it: the semblances of a reason (似因) and their counterpart for the thesis (似宗). 商羯羅主's 因明入正理論 (7th c., in 玄奘's translation) lists fourteen faulty reasons — unestablished, inconclusive, contradictory — beside nine faulty theses. The lens is the human screen: a reason that looks like proof is read against the taxonomy before it is trusted.",
        phase: Phase::Catalogued,
    },
    Lens {
        id: "TWO-TRUTHS",
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
            "龍樹《中論》鳩摩羅什譯 (T30n1564), 觀四諦品第二十四",
            "https://plato.stanford.edu/entries/twotruths-india/",
        ],
        background: "龍樹, in his root verses on the Middle Way (24.8–10): the Buddha's teaching rests on two truths — the conventional (世俗諦), what may be said in ordinary terms, and the ultimate (勝義諦), how things finally stand — and the ultimate is reached only by way of the conventional. peira's Court Mode is exactly that translation, one-directional: the statement said aloud may only lose strength against the fully-bounded graph, never gain it.",
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
        background: "Phan Minh Dung's 1995 paper (Artificial Intelligence 77) founded abstract argumentation: arguments and an attack relation form a graph, and which arguments are acceptable is computed, not asserted — one stands if every attacker is itself defeated. The grounded extension is the least such set, and it declines to crown a winner in an unbroken stand-off. The lens uses it: a claim's standing is the fixed point of the attack graph, not whoever spoke last.",
        phase: Phase::Enforced,
    },
    // ── The anti-summarization pass ───────────────────────────────────────────
    // Four lenses a distillation needs that the investigative set above does not
    // supply. Each is catalogued, not enforced, and deliberately: "what I smoothed
    // over" and "which error is costlier" are irreducibly judgement, and a gate that
    // pretended to check them would be the ceremony this project exists to refuse.
    // The full framework — the five questions these fan out from, and the crosswalk
    // to the lenses above — is in docs/method/anti-summarization.md.
    Lens {
        id: "KNOW-BY-DOING",
        name: "格物致知 · 知行合一 — Investigate Each Thing; Knowing Proven in Doing",
        tradition: Tradition::Chinese,
        failure_mode: "comprehension faked by compression — a distillation you can restate but \
could not apply to a case it has not already seen",
        operation: "a claim carried forward from a source survives an application test — predict \
with it, act on it, or generate a novel instance — not a paraphrase test",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "朱熹's 格物 is exhaustive, bottom-up investigation of 理 — the opposite \
of top-down summary. 王陽明 investigated bamboo for seven days, fell ill, and concluded 理 is \
confirmed in action, not external cataloguing: 知行合一. A summary you cannot act on you have not \
understood; you have only compressed.",
        sources: &[
            "朱熹《四書章句集注・大學章句》: 致知在格物",
            "王陽明《傳習錄》: 知行合一",
            "https://ctext.org/liji/da-xue",
            "https://plato.stanford.edu/entries/wang-yangming/",
        ],
        background: "格物致知 (\"investigate things, extend knowledge\") is from the 大學 (the Great Learning); 朱熹 (1130–1200) read it as exhausting the principle (理) in each thing. In 1492 王陽明 sat before bamboo for seven days to do exactly that, found no principle and fell ill — concluding that principle is not caught by treating the world as a detached object, which became his 知行合一, the unity of knowing and acting. The lens keeps the test: what you cannot act on, you have not yet understood.",
        phase: Phase::Catalogued,
    },
    Lens {
        id: "LACUNA",
        name: "闕文 · The Dog That Didn't Bark — What Is Conspicuously Absent",
        tradition: Tradition::Modern,
        failure_mode: "a distillation reports only what the source contains, so it is structurally \
blind to what a competent treatment of the topic would contain but this one omits — the missing \
counter-argument, dataset, caveat, or party",
        operation: "before a source is accepted as complete, enumerate what its topic demands and \
mark each item the source never addresses; the gap is a finding, not a silence",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "Holmes solves it by the dog that did NOT bark: the absence of the expected \
reaction is the evidence. 孔子 keeps the counterpart — 「吾猶及史之闕文也」, the honest historian \
leaves a blank where knowledge is missing rather than filling it. A summary that silently omits \
what it never saw does the opposite, and reads as complete.",
        sources: &[
            "Arthur Conan Doyle, \"The Adventure of Silver Blaze\" (1892)",
            "《論語・衛靈公》15.26: 吾猶及史之闕文也",
        ],
        background: "Two sources meet here. 孔子, in the Analects (15.26): 「吾猶及史之闕文也」— he could still recall when a scribe left a blank (闕文) rather than fill a gap he could not vouch for. And Conan Doyle's \"Silver Blaze\" (1892), where Holmes cracks the case by the dog that did not bark: the absence of the expected is the evidence. A distillation sees neither unbidden — it reports what is present, so an omission has to be hunted.",
        phase: Phase::Catalogued,
    },
    Lens {
        id: "IDOLA",
        name: "Idola Mentis — Bacon's Idols of the Mind",
        tradition: Tradition::Modern,
        failure_mode: "the distiller's own biases treated as if the mind were a clean mirror — what \
I wanted to find, and what I smoothed over for fluency, entering the summary unmarked",
        operation: "a distillation records, separately from the source's faults, the reader's own: \
the conclusion I was primed to reach, and the passage I made read cleaner than it was",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "Bacon's four idols — Tribus (human nature), Specus (the individual's \
cave), Fori (the words of the marketplace), Theatri (received systems). A clean chapter with \
nothing flagged is the mirror flattering the observer, not the source being sound: peira's own \
rule — a zero is a possible instrument failure — turned on the instrument that is the reader.",
        sources: &[
            "Francis Bacon, Novum Organum (1620), Bk I, Aphorisms 39–68",
            "https://plato.stanford.edu/entries/francis-bacon/",
        ],
        background: "Francis Bacon, Novum Organum (1620), named four \"idols\" that distort the mind before it reasons: idola tribus (of the tribe — human nature itself), idola specus (of the cave — the individual's bent), idola fori (of the marketplace — the loose words we argue in), and idola theatri (of the theatre — inherited systems taken on stage). The lens turns them on the distiller: a clean pass with nothing flagged is the mirror flattering the observer, not proof the source is sound.",
        phase: Phase::Catalogued,
    },
    Lens {
        id: "BLACKSTONE",
        name: "Blackstone's Ratio — The Asymmetry of Error",
        tradition: Tradition::Modern,
        failure_mode: "equal scrutiny spread across every uncertainty, when the cost of being \
wrong runs overwhelmingly in one direction",
        operation: "before closing, name which direction of error is costlier — false positive or \
false negative — and concentrate the remaining doubt on the claim guarding the irreversible or \
expensive decision",
        applies_to: ARGUMENTS,
        gates: &[],
        worked_example: "\"Better that ten guilty persons escape than that one innocent suffer\" \
is a declared preference for one error over its opposite, not indifference between them. A \
distillation that spends the same rigour on a load-bearing claim under an irreversible decision \
and on an incidental aside has mis-allocated its scepticism.",
        sources: &[
            "William Blackstone, Commentaries on the Laws of England, vol. IV (1769), ch. 27",
            "J. Neyman & E. S. Pearson, Phil. Trans. R. Soc. A 231 (1933) 289–337",
        ],
        background: "William Blackstone, Commentaries on the Laws of England (1769), Book IV ch. 27: \"better that ten guilty persons escape than that one innocent suffer\" — a stated preference for one direction of error over the other, not indifference between them. The Neyman–Pearson framework (1933) formalised the same asymmetry as Type I versus Type II error. The lens makes it a triage: name which way it is costlier to be wrong, and spend the scrutiny there.",
        phase: Phase::Catalogued,
    },
];

#[cfg(test)]
mod meta_tests {
    //! Tests over the *knowledge*, not over a code path — the payoff of holding the
    //! catalogue as data. Borrowed wholesale from forensicnomicon, which asserts the
    //! same class of property over its artifact tables.

    use super::*;
    use std::collections::BTreeSet;

    /// Both declared characteristics must survive the trip to the caller.
    ///
    /// Worth stating what the fixture vaults do NOT prove here: no edge in either of
    /// them declares `via=inference`, so `bounded` staying clean is the SCOPE control
    /// passing — perception and testimony owe these fields nothing — and is silent on
    /// whether the gates work. This is where that evidence lives.
    #[test]
    fn the_declared_characteristics_reach_the_caller() {
        use peira_core::{parse_node, Edge, EdgeKind, Means, NodeId};
        let mut g = Graph::new();
        for src in [
            "---\nid: c1\ntype: claim\ntitle: The binary was executed\n---\n",
            "---\nid: o1\ntype: observation\ntitle: Amcache entry\n---\n",
        ] {
            g.insert_node(parse_node(src).expect("fixture parses"));
        }
        g.insert_edge(
            Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                .via(Means::Inference),
        );

        // Both reach NO VERDICT, and Unassessed is never a pass — so what must arrive
        // is the unassessed report naming each gate, not a violation carrying its code.
        let unassessed: Vec<String> = examine_graph(&g).iter().map(|v| v.detail.clone()).collect();
        assert!(
            unassessed.iter().any(|d| d.contains("dissimilar cases")),
            "the vipakṣa survey demand was computed and then discarded in transit: {unassessed:?}"
        );
        assert!(
            unassessed
                .iter()
                .any(|d| d.contains("not a rule yet")),
            "the sapakṣa instance demand was computed and then discarded in transit: {unassessed:?}"
        );
    }

    /// The dṛśya restriction must survive the trip to the caller.
    #[test]
    fn an_uncontrolled_absence_reaches_the_caller() {
        use peira_core::{parse_node, Edge, EdgeKind, NodeId};
        let mut g = Graph::new();
        for src in [
            "---\nid: c1\ntype: claim\ntitle: No Prefetch file exists for the binary\n---\n",
            "---\nid: o1\ntype: observation\ntitle: Prefetch directory listing\n---\n",
            "---\nid: i1\ntype: instrument\ntitle: Prefetch enumerator\n---\n",
        ] {
            g.insert_node(parse_node(src).expect("fixture parses"));
        }
        g.insert_edge(Edge::new(
            NodeId::new("o1"),
            NodeId::new("c1"),
            EdgeKind::Supports,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("o1"),
            NodeId::new("i1"),
            EdgeKind::MeasuredBy,
        ));

        let found = examine_graph(&g);
        assert!(
            found.iter().any(|v| v.gate == gates::ABSENCE_UNCONTROLLED),
            "the dṛśya restriction was computed and then discarded in transit; \
violations reaching the caller were: {:?}",
            found.iter().map(|v| v.gate).collect::<Vec<_>>()
        );
    }

    /// ACH's demand must survive the trip to the caller.
    #[test]
    fn an_unrivalled_explanation_reaches_the_caller() {
        use peira_core::{parse_node, Edge, EdgeKind, NodeId};
        let mut g = Graph::new();
        for src in [
            "---\nid: c1\ntype: claim\ntitle: Running the binary wrote the entry\n\
causal_rung: counterfactual\n---\n",
            "---\nid: run1\ntype: run\ntitle: Controlled execution on a clean VM\n---\n",
        ] {
            g.insert_node(parse_node(src).expect("fixture parses"));
        }
        g.insert_edge(Edge::new(
            NodeId::new("run1"),
            NodeId::new("c1"),
            EdgeKind::Supports,
        ));

        let found = examine_graph(&g);
        assert!(
            found.iter().any(|v| v.gate == gates::RIVALS_UNENUMERATED),
            "ACH's demand was computed and then discarded in transit; violations \
reaching the caller were: {:?}",
            found.iter().map(|v| v.gate).collect::<Vec<_>>()
        );
    }

    /// H5 — the relabelling evasion is closed for EVERY gate-owning lens, not one.
    ///
    /// `Lens::examine` applies `applies_to` BEFORE the gate runs, so a static kind list
    /// on a gate-owning lens means relabelling a load-bearing claim `type: observation`
    /// strips the obligation entirely. That was found on SUBSTANCE-FUNCTION and fixed there, and
    /// the regression test was written for SUBSTANCE-FUNCTION alone — restoring `ARGUMENTS` on
    /// FOUR-CORNERS passed the whole suite. The rule is a property of the catalogue, so
    /// it is asserted over the catalogue.
    ///
    /// A gateless lens may carry a kind list: it scopes documentation, not enforcement.
    #[test]
    fn no_gate_owning_lens_scopes_itself_by_node_kind() {
        for l in CATALOG {
            if l.gates.is_empty() {
                continue;
            }
            assert!(
                l.applies_to.is_empty(),
                "{} owns gates and lists node kinds {:?}; `examine` applies that list \
before the gate, so relabelling the node evades every one of them. Scope inside the \
gate instead — `under_promotion` is the shared predicate.",
                l.id,
                l.applies_to
            );
        }
    }

    /// 九句因 — the wheel is the truth table inside the gate, not a lens of its own.
    ///
    /// Two of Dignāga's nine cells are valid (2 and 8), both characterised by absence
    /// from the dissimilar cases. This asserts what the STRUCTURAL gate can and cannot
    /// see, so the coverage limit is machine-checked rather than promised in prose:
    /// the graph records which supporters a rival shares, and nothing more. Cell 1
    /// (all shared) is visible and blocks. Cells 3/9 (some shared) deliberately pass —
    /// one discriminating line is what decides a contest. Cells 5 and 7 turn on how the
    /// reason behaves across similar and dissimilar CLASSES, which no attack edge
    /// records; those need `sapaksa:`/`vipaksa:`, and until those land this lens does
    /// not claim them.
    #[test]
    fn the_wheel_of_reasons_maps_onto_what_the_graph_can_see() {
        use peira_core::{parse_node, Edge, EdgeKind, NodeId};
        // (shared supporters, unshared supporters, cell, must block)
        let wheel = [
            (
                1_usize,
                0_usize,
                "1 共不定 — present in every dissimilar case",
                true,
            ),
            (0, 1, "2 正因 — absent from the dissimilar cases", false),
            (2, 0, "1 共不定, several lines, all of them shared", true),
            (
                1,
                1,
                "3/9 — partial sharing; one discriminating line decides",
                false,
            ),
            (
                2,
                1,
                "3/9 — mostly shared, still decided by the one line",
                false,
            ),
        ];
        for (shared, unshared, cell, must_block) in wheel {
            let mut g = Graph::new();
            for src in [
                "---\nid: c1\ntype: claim\ntitle: The user executed it\n---\n",
                "---\nid: r1\ntype: claim\ntitle: The appraiser catalogued it\n---\n",
            ] {
                g.insert_node(parse_node(src).expect("fixture parses"));
            }
            g.insert_edge(Edge::new(
                NodeId::new("r1"),
                NodeId::new("c1"),
                EdgeKind::Attacks,
            ));
            for i in 0..shared + unshared {
                let id = format!("o{i}");
                g.insert_node(
                    parse_node(&format!(
                        "---\nid: {id}\ntype: observation\ntitle: line {i}\n---\n"
                    ))
                    .expect("fixture parses"),
                );
                g.insert_edge(Edge::new(
                    NodeId::new(&id),
                    NodeId::new("c1"),
                    EdgeKind::Supports,
                ));
                if i < shared {
                    g.insert_edge(Edge::new(
                        NodeId::new(&id),
                        NodeId::new("r1"),
                        EdgeKind::Supports,
                    ));
                }
            }
            // Scoped to c1. The gate fires symmetrically by design — where r1's own
            // single line is shared, r1 is 共不定 too, and each author can fix the node
            // they own. Asserting over the whole graph would conflate the two verdicts.
            let subject = NodeId::new("c1");
            let blocked = examine_graph(&g)
                .iter()
                .any(|v| v.gate == gates::REASON_UNDIAGNOSTIC && v.subject == subject);
            assert_eq!(
                blocked, must_block,
                "cell {cell}: {shared} shared / {unshared} unshared supporters"
            );
        }
    }

    /// The 共不定 verdict must survive the trip to the caller.
    ///
    /// A gate is not enforced because its predicate is correct; it is enforced when the
    /// aggregator carries its verdict out. This crate has shipped a rule whose only
    /// callers were tests, so the assertion is made THROUGH `examine_graph` — unwiring
    /// the catalogue entry must turn this red.
    #[test]
    fn undiagnostic_support_reaches_the_caller() {
        use peira_core::{parse_node, Edge, EdgeKind, NodeId};
        let mut g = Graph::new();
        for src in [
            "---\nid: c1\ntype: claim\ntitle: The user executed coreupdater.exe\n---\n",
            "---\nid: r1\ntype: claim\ntitle: The appraiser catalogued it, unrun\n---\n",
            "---\nid: o1\ntype: observation\ntitle: InventoryApplicationFile entry\n---\n",
        ] {
            g.insert_node(parse_node(src).expect("fixture parses"));
        }
        g.insert_edge(Edge::new(
            NodeId::new("r1"),
            NodeId::new("c1"),
            EdgeKind::Attacks,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("o1"),
            NodeId::new("c1"),
            EdgeKind::Supports,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("o1"),
            NodeId::new("r1"),
            EdgeKind::Supports,
        ));

        let found = examine_graph(&g);
        assert!(
            found.iter().any(|v| v.gate == gates::REASON_UNDIAGNOSTIC),
            "共不定 was computed and then discarded in transit; violations reaching the \
caller were: {:?}",
            found.iter().map(|v| v.gate).collect::<Vec<_>>()
        );
    }

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
    fn every_lens_carries_a_tradition_note() {
        // The background note is the source-checked "In the tradition" section. Once
        // written for every lens, it stays written — a new lens must carry one too, so
        // the catalogue cannot ship a lens with no account of where it comes from.
        for l in CATALOG {
            assert!(
                l.background.trim().len() > 40,
                "{} has no substantive background note — add its 'In the tradition' text",
                l.id
            );
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
            "CRITERION",
            "RECTIFY-NAME",
            "SUBSTANCE-FUNCTION",
            "WHITE-HORSE",
            "FOUR-CORNERS",
            "TOULMIN",
            "MEANS-OF-KNOWING",
            "RUNG",
        ] {
            let l = lens(id).unwrap_or_else(|| panic!("{id} missing from the catalogue"));
            assert_eq!(l.phase, Phase::Enforced, "{id} must be enforced");
            assert!(!l.gates.is_empty(), "{id} must own a gate");
        }
    }

    #[test]
    fn lookup_finds_what_the_catalogue_holds() {
        assert!(lens("CRITERION").is_some());
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
        let liji = lens("CRITERION").expect("CRITERION exists");
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
    /// Load-bearing, asserted against an INDEPENDENT table rather than against itself.
    ///
    /// A first version of this test asserted "anything the promotion gates examine, the
    /// grading demand also considers". That is true BY CONSTRUCTION now that both share
    /// [`carries_weight`], so it could never fail — a tautology wearing a test's
    /// clothes. Mutating `carries_weight` to drop the `depends_on` direction left it
    /// green, which is how it was caught.
    ///
    /// So the expectations are written out independently of the implementation. Each row
    /// is a fact about the domain: what it means for something to be leaned on.
    #[test]
    fn load_bearing_matches_what_it_means_to_be_leaned_on() {
        use peira_core::{parse_node, Edge, EdgeKind, Graph, NodeId};

        // (wiring, node kind, expected: does anything lean on it?)
        let cases: &[(&str, &str, bool)] = &[
            ("isolated", "observation", false),
            ("isolated", "hypothesis", false),
            ("isolated", "claim", true), // a claim asserts by existing
            ("supports a claim", "observation", true),
            ("supports a claim", "hypothesis", true),
            ("supports a claim", "run", true),
            ("a claim depends on it", "observation", true), // the direction missed twice
            ("a claim depends on it", "hypothesis", true),
            ("a claim depends on it", "protocol", true),
            ("supports a bare sketch", "hypothesis", false), // thinking out loud
            ("supports a bare sketch", "observation", false),
            ("it depends on a claim", "hypothesis", false), // needs ≠ carries
        ];

        for (wiring, kind, expected) in cases {
            let mut g = Graph::new();
            g.insert_node(parse_node("---\nid: c1\ntype: claim\ntitle: t\n---\n").unwrap());
            g.insert_node(
                parse_node("---\nid: h0\ntype: hypothesis\ntitle: sketch\n---\n").unwrap(),
            );
            let n = parse_node(&format!("---\nid: x\ntype: {kind}\ntitle: t\n---\n")).unwrap();
            g.insert_node(n.clone());
            let e = |f: &str, t: &str, k: EdgeKind| Edge::new(NodeId::new(f), NodeId::new(t), k);
            match *wiring {
                "supports a claim" => g.insert_edge(e("x", "c1", EdgeKind::Supports)),
                "a claim depends on it" => g.insert_edge(e("c1", "x", EdgeKind::DependsOn)),
                "supports a bare sketch" => g.insert_edge(e("x", "h0", EdgeKind::Supports)),
                "it depends on a claim" => g.insert_edge(e("x", "c1", EdgeKind::DependsOn)),
                _ => {}
            }
            assert_eq!(
                carries_weight(&g, &n),
                *expected,
                "a {kind} where {wiring}: expected carries_weight = {expected}"
            );
        }
    }

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
            "RECTIFY-NAME, WHITE-HORSE and RUNG each reach no verdict here; \
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
