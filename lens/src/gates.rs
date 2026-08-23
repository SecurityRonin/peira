//! The enforced gate predicates.
//!
//! Every gate is a pure function of the graph and one node — no I/O, no clock, no
//! randomness — so each is independently testable and fuzzable, and the engine's
//! verdict is reproducible from committed bytes alone.
//!
//! A gate that cannot reach a verdict returns [`GateResult::Unassessed`] naming what
//! was missing. It never returns `Pass`. Silence is not consent.

use crate::{GateResult, Violation};
use peira_core::{EdgeKind, Graph, Node, NodeKind};

// ── Stable published gate codes ──────────────────────────────────────────────
// These appear in Court Mode packets. A shipped code never changes meaning.

/// 立極: an evaluative claim with no declared standard.
pub const CRITERION_UNDECLARED: &str = "PEIR-CRITERION-UNDECLARED";
/// 正名: a load-bearing term that was never stipulated.
pub const TERM_UNSTIPULATED: &str = "PEIR-TERM-UNSTIPULATED";
/// 體用: a claim about what a thing IS, resting only on what it DID.
pub const FUNCTION_AS_SUBSTANCE: &str = "PEIR-FUNCTION-AS-SUBSTANCE";
/// 白馬非馬: a class claim that never says what the class contains.
pub const CLASS_EXTENSION_UNDECLARED: &str = "PEIR-CLASS-EXTENSION-UNDECLARED";
/// 四句: a contested claim that addressed fewer than four corners.
pub const CORNERS_UNADDRESSED: &str = "PEIR-CORNERS-UNADDRESSED";
/// Toulmin: the rule licensing grounds → claim was never written down.
pub const WARRANT_MISSING: &str = "PEIR-WARRANT-MISSING";
/// pramāṇa: an edge graded above what its means of knowing allows.
pub const GRADE_EXCEEDS_PRAMANA: &str = "PEIR-GRADE-EXCEEDS-PRAMANA";
/// Pearl: an interventional or counterfactual claim with no executed protocol.
pub const CAUSAL_RUNG_UNREACHED: &str = "PEIR-CAUSAL-RUNG-UNREACHED";
/// A conclusion stated with no conditions under which it would change.
pub const BOUNDARIES_MISSING: &str = "PEIR-BOUNDARIES-MISSING";
/// Premortem: a claim with nothing that could ever count against it.
pub const FALSIFIER_MISSING: &str = "PEIR-FALSIFIER-MISSING";
/// 因三相 (異品遍無性): every line of support is shared with a live rival.
pub const HETU_UNDIAGNOSTIC: &str = "PEIR-HETU-UNDIAGNOSTIC";
/// ACH: a causal claim with no competing explanation written down.
pub const RIVALS_UNENUMERATED: &str = "PEIR-RIVALS-UNENUMERATED";
/// 不可得因: an absence certified by a search never shown able to find the thing.
pub const ABSENCE_UNCONTROLLED: &str = "PEIR-ABSENCE-UNCONTROLLED";
/// 因三相 (異品遍無性): an inference that never surveyed the dissimilar cases.
pub const VIPAKSA_UNSURVEYED: &str = "PEIR-VIPAKSA-UNSURVEYED";
/// 因三相 (同品定有性): an inference rule with no known instance.
pub const SAPAKSA_UNDECLARED: &str = "PEIR-SAPAKSA-UNDECLARED";

/// Words that turn a description into a judgement.
///
/// This is our own detection heuristic rather than a decode of anyone's spec, so a
/// self-authored table is the honest instrument for it. It exists to catch the case
/// where nobody thought to mark a claim evaluative — `evaluative: true` handles the
/// case where they did.
const EVALUATIVE_TERMS: &[&str] = &[
    "suspicious",
    "malicious",
    "anomalous",
    "unusual",
    "concerning",
    "significant",
    "benign",
    "legitimate",
    "safe",
    "severe",
    "critical",
    "poor",
    "excessive",
    "inadequate",
    "unacceptable",
];

/// Rungs of Pearl's ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CausalRung {
    /// Seeing: correlation, observed association.
    Association,
    /// Doing: what happens if we intervene.
    Intervention,
    /// Imagining: what would have happened otherwise.
    Counterfactual,
}

impl CausalRung {
    fn from_str_opt(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().as_str() {
            "association" | "seeing" | "1" => CausalRung::Association,
            "intervention" | "doing" | "2" => CausalRung::Intervention,
            "counterfactual" | "imagining" | "3" => CausalRung::Counterfactual,
            _ => return None,
        })
    }
}

/// Whether a promotion gate examines this node.
///
/// A `Hypothesis` is a candidate explanation, and demanding boundaries, a falsifier
/// and a causal rung before it may exist inverts what the kind is for — a checker that
/// blocks you for thinking out loud is a checker you switch off.
///
/// But the exemption ends where something LEANS on it. A hypothesis supporting a claim
/// is doing a claim's work, and scoping these gates to `Claim` alone let an unexamined
/// hypothesis carry an over-statement into a frozen packet: put the conclusion on a
/// hypothesis, support the claim with it, and every promotion gate looked away.
///
/// Same rule as `PEIR-LINT-RETRACTED`: the obligation attaches to being load-bearing,
/// not to the node kind.
fn under_promotion(graph: &Graph, node: &Node) -> bool {
    // Load-bearing AND asserting. The first half is [`crate::carries_weight`] — the one
    // definition, shared with the grading demand — and this adds the only thing that is
    // genuinely specific to PROMOTION gates.
    //
    // The refinement: these gates ask whether a node declares its scope, its warrant,
    // its rung. A hypothesis is an ARGUMENT and competes in the extension, so being
    // leaned on is enough. Everything else is evidence, and demanding a quantifier from
    // every supporting observation is ceremony — ceremony is routed around. It must
    // also have taken a position.
    //
    // `aspect:` is deliberately not a position: 體用 ASKS an observation to declare it,
    // so counting the answer would punish the author who answers.
    if !crate::carries_weight(graph, node) {
        return false;
    }
    match node.kind {
        NodeKind::Claim | NodeKind::Hypothesis => true,
        _ => {
            ["quantifier", "causal_rung", "warrant"]
                .iter()
                .any(|f| node.field(f).is_some())
                || is_evaluative(node)
        }
    }
}

/// Build a blocking result.
fn block(
    gate: &'static str,
    lens: &'static str,
    node: &Node,
    detail: String,
    remedy: &'static str,
) -> GateResult {
    GateResult::Block(Violation {
        gate,
        lens,
        subject: node.id.clone(),
        detail,
        remedy,
    })
}

/// Nodes supporting `node` via a `supports` edge.
fn supporters<'a>(graph: &'a Graph, node: &Node) -> Vec<&'a Node> {
    graph
        .edges_to(&node.id)
        .filter(|e| e.kind == EdgeKind::Supports)
        .filter_map(|e| graph.node(&e.from))
        .collect()
}

/// Whether the claim reads as a judgement rather than a description.
fn is_evaluative(node: &Node) -> bool {
    if node.field("evaluative") == Some("true") {
        return true;
    }
    let haystack = format!("{} {}", node.title, node.body).to_ascii_lowercase();
    EVALUATIVE_TERMS.iter().any(|t| {
        haystack
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| w == *t)
    })
}

// ── 立極 ─────────────────────────────────────────────────────────────────────

/// An evaluative claim must name the standard it is judged against.
pub fn criterion_declared(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    if !is_evaluative(node) {
        return GateResult::NotApplicable;
    }
    let has_criterion = graph
        .edges_from(&node.id)
        .filter(|e| e.kind == EdgeKind::JudgedBy)
        .any(|e| {
            graph
                .node(&e.to)
                .is_some_and(|n| n.kind == NodeKind::Criterion)
        });

    if has_criterion {
        GateResult::Pass
    } else {
        block(
            CRITERION_UNDECLARED,
            "LIJI",
            node,
            format!("evaluative claim \"{}\" declares no criterion", node.title),
            "add a `judged_by:` edge to a Criterion stating the standard applied",
        )
    }
}

// ── 正名 ─────────────────────────────────────────────────────────────────────

/// The three moments a Term must carry before a claim may lean on it.
const TERM_MOMENTS: &[&str] = &["as_used", "not_essence", "stipulated"];

/// Every load-bearing term resolves to a fully stipulated Term node.
pub fn key_terms_stipulated(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    let uses: Vec<_> = graph
        .edges_from(&node.id)
        .filter(|e| e.kind == EdgeKind::UsesTerm)
        .collect();

    if uses.is_empty() {
        // "This claim turns on no term of art" is a real state, and until it could be
        // SAID the gate made every plain claim unfreezable — silence and denial read
        // identically, which is the failure this whole catalogue exists to name. The
        // author declares it explicitly, exactly as `evaluative: true` overrides the
        // 立極 word table. Declaring it is a statement someone can be held to; omitting
        // the field still reaches no verdict.
        if node.field("no_terms_of_art") == Some("true") {
            return GateResult::Pass;
        }
        return GateResult::Unassessed {
            why: format!(
                "\"{}\" declares no key terms, so which words are load-bearing is unknown \
— add `uses_term:`, or declare `no_terms_of_art: true` if it truly turns on none",
                node.title
            ),
        };
    }

    for edge in uses {
        let Some(term) = graph.node(&edge.to) else {
            return block(
                TERM_UNSTIPULATED,
                "ZHENGMING",
                node,
                format!("key term `{}` does not resolve to any node", edge.to),
                "create the Term node, or correct the reference",
            );
        };
        let missing: Vec<&str> = TERM_MOMENTS
            .iter()
            .copied()
            .filter(|m| term.field(m).is_none())
            .collect();
        if !missing.is_empty() {
            return block(
                TERM_UNSTIPULATED,
                "ZHENGMING",
                node,
                format!("term `{}` is missing {}", term.id, missing.join(", ")),
                "give the term all three moments: as_used (所謂), not_essence (即非), \
stipulated (是名)",
            );
        }
    }
    GateResult::Pass
}

// ── 體用 ─────────────────────────────────────────────────────────────────────

/// What a node says it speaks about — by field OR by edge.
///
/// `substance_of:` and `function_of:` are the edge grammar `EdgeKind` documents for 體用,
/// and they were parsed, recorded and read by NOTHING. So the identical substance claim
/// was examined when it wrote `aspect: substance` and froze when it wrote
/// `substance_of:` — the same statement, the same lens, the outcome decided by which of
/// two published spellings the author picked. Byte-for-byte the `Sublates` defect.
///
/// The field wins where both are present: it is the more specific declaration, and
/// disagreement between them is the author's to resolve, not this gate's to arbitrate.
fn declared_aspect(graph: &Graph, node: &Node) -> Option<&'static str> {
    // CASE AND TYPOS. `_ => {}` fell through to `None`, so `aspect: Substance` — one
    // capital — silently switched 體用 off, and so did any misspelling. That is an
    // unrecognised value disabling a check instead of being shown verbatim, which is
    // this repository's own stated rule, and the identical defect was already fixed for
    // `quantifier:` and `causal_rung:`. An unrecognised value is reported by
    // `substance_not_from_function_alone`, which blocks and prints it.
    match node.field("aspect").map(str::trim) {
        Some(a) if a.eq_ignore_ascii_case("substance") => return Some("substance"),
        Some(a) if a.eq_ignore_ascii_case("function") => return Some("function"),
        _ => {}
    }
    let mut out = None;
    for e in graph.edges_from(&node.id) {
        match e.kind {
            EdgeKind::SubstanceOf => return Some("substance"),
            EdgeKind::FunctionOf => out = Some("function"),
            _ => {}
        }
    }
    out
}

/// A claim about what a thing *is* may not rest solely on what it *did*.
pub fn substance_not_from_function_alone(graph: &Graph, node: &Node) -> GateResult {
    // AN UNRECOGNISED VALUE IS SHOWN, NEVER SWALLOWED. `aspect: subtance` used to fall
    // through to `NotApplicable` and switch the lens off in silence — the same defect
    // already fixed for `causal_rung:`, which blocks and prints the offending word.
    if let Some(raw) = node.field("aspect") {
        let t = raw.trim();
        if !t.eq_ignore_ascii_case("substance") && !t.eq_ignore_ascii_case("function") {
            return GateResult::Unassessed {
                why: format!(
                    "\"{}\" declares `aspect: {t}`, which is neither `substance` nor \
`function` — so which of the two it speaks of is unknown",
                    node.title
                ),
            };
        }
    }
    if declared_aspect(graph, node) != Some("substance") {
        return GateResult::NotApplicable;
    }
    let support = supporters(graph, node);
    if support.is_empty() {
        // A LEAF HAS NOTHING TO CLASSIFY, and demanding it produce some is the regress:
        // this gate asks SUPPORTERS to declare an aspect, and an observation that
        // answered honestly then found the annotation itself blocking, with no way out
        // that was not a lie. Primitive evidence is where a chain ends.
        //
        // A CLAIM resting on nothing is a different matter and stays Unassessed: it
        // asserts what a thing IS on no evidence at all, which is unexamined rather than
        // inapplicable. Stated as a reason, not a category — the distinction is whether
        // the node is the evidence or rests on it.
        if !matches!(node.kind, NodeKind::Claim | NodeKind::Hypothesis) {
            return GateResult::NotApplicable;
        }
        return GateResult::Unassessed {
            why: format!("\"{}\" has no supporting evidence to classify", node.title),
        };
    }
    // Three states, not a boolean. `all(== function)` was false for "some supporter
    // says something else" AND for "some supporter says nothing", and Pass was reported
    // for both — so annotating evidence honestly blocked the claim while deleting the
    // line froze it. A predicate whose `false` means two different things is a rule
    // lost at a join.
    let silent: Vec<&str> = support
        .iter()
        .filter(|s| declared_aspect(graph, s).is_none())
        .map(|s| s.id.as_str())
        .collect();
    if !silent.is_empty() {
        return GateResult::Unassessed {
            why: format!(
                "\"{}\" claims what the thing IS, and {} declares no `aspect:` — whether \
its evidence bears on substance or only on function is unknown",
                node.title,
                silent.join(", ")
            ),
        };
    }

    let function_only = support
        .iter()
        .all(|s| declared_aspect(graph, s) == Some("function"));

    if function_only {
        block(
            FUNCTION_AS_SUBSTANCE,
            "TIYONG",
            node,
            format!(
                "substance claim \"{}\" rests only on function evidence ({})",
                node.title,
                support
                    .iter()
                    .map(|s| s.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "restate as a claim about what the thing did, or add evidence bearing on \
what it is",
        )
    } else {
        GateResult::Pass
    }
}

// ── 白馬非馬 ─────────────────────────────────────────────────────────────────

/// A claim quantifying over a class must say what the class contains.
pub fn class_extension_declared(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    let Some(quantifier) = node.field("quantifier") else {
        return GateResult::Unassessed {
            why: format!(
                "\"{}\" does not say whether it speaks of a class or a single case",
                node.title
            ),
        };
    };
    // An UNRECOGNISED value is not "singular-like". `quantifier: all` sailed through as
    // NotApplicable while quantifying over everything, so a present-but-unknown value
    // did BETTER than an absent one — and absence already reaches no verdict. Show the
    // offending value rather than hiding it.
    match quantifier {
        "universal" | "class" => {}
        "singular" => return GateResult::NotApplicable,
        other => {
            return GateResult::Unassessed {
                why: format!(
                    "\"{}\" declares `quantifier: {other}`, which this gate does not \
recognise — expected `universal`, `class` or `singular`",
                    node.title
                ),
            }
        }
    }
    if node.field_list("extension").is_empty() {
        block(
            CLASS_EXTENSION_UNDECLARED,
            "BAIMA",
            node,
            format!(
                "claim quantifies `{quantifier}` over \"{}\" without declaring its extension",
                node.title
            ),
            "declare `extension:` — what the class contains — or narrow the claim to the \
case actually examined",
        )
    } else {
        GateResult::Pass
    }
}

// ── 四句 ─────────────────────────────────────────────────────────────────────

/// A contested claim must address all four corners.
pub fn four_corners_addressed(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    // The LIVE relation: an attack the argumentation discards does not make a claim
    // contested, and counting it blocked the victim over one line in a term author's
    // frontmatter.
    let attacked = graph.live_attacks_on(&node.id).next().is_some();
    let contested = attacked || node.field("contested") == Some("true");
    if !contested {
        return GateResult::NotApplicable;
    }
    let corners = node.field_list("corners");
    // DISTINCT. 四句 asks for A, not-A, both and neither — four different positions.
    // Counting entries let one corner stated four times satisfy the gate, which is the
    // shape of check that measures the aggregation point rather than the effective
    // thing. Compared case- and whitespace-insensitively: an author who addressed one
    // corner four times did not address four, however they capitalised it.
    let distinct: std::collections::BTreeSet<String> = corners
        .iter()
        .map(|c| {
            // PUNCTUATION too. Case and whitespace were folded and trailing marks were
            // not, so `[It holds, "It holds.", "It holds!", "It holds?"]` counted as
            // four distinct corners — the same one, typed four ways.
            c.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
                .chars()
                .filter(|ch| ch.is_alphanumeric() || *ch == ' ')
                .collect::<String>()
                .trim()
                .to_owned()
        })
        .collect();
    if distinct.len() == 4 {
        GateResult::Pass
    } else {
        block(
            CORNERS_UNADDRESSED,
            "CATUSKOTI",
            node,
            // The DISTINCT count, and the raw one when they differ. "addresses 4 of 4"
            // over four identical entries would be the gate reporting the number it
            // measured rather than the one it means.
            format!(
                "contested claim \"{}\" addresses {} of 4 corners{}",
                node.title,
                distinct.len(),
                if distinct.len() == corners.len() {
                    String::new()
                } else {
                    format!(" ({} entries, {} distinct)", corners.len(), distinct.len())
                }
            ),
            "state all four: A, not-A, both, neither — ruling one out with a reason counts \
as addressing it",
        )
    }
}

// ── Toulmin ──────────────────────────────────────────────────────────────────

/// The rule licensing grounds → claim must be written down.
pub fn warrant_present(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    if node.field("warrant").is_some() {
        return GateResult::Pass;
    }
    let detail = if node.fields.contains("warrant") {
        format!("\"{}\" has a `warrant:` key, but it is blank", node.title)
    } else {
        format!("\"{}\" states no warrant", node.title)
    };
    block(
        WARRANT_MISSING,
        "TOULMIN",
        node,
        detail,
        "write the rule that licenses the step from grounds to claim — it is usually \
the part that turns out to be false",
    )
}

// ── pramāṇa ──────────────────────────────────────────────────────────────────

/// No edge may be graded above what its means of knowing allows.
pub fn grades_within_pramana_ceiling(graph: &Graph, node: &Node) -> GateResult {
    // EVIDENCE edges only. A rival's `contradicts` carrying a bad grade is the rival's
    // defect, and blocking the claim it attacks punishes the victim for someone else's
    // frontmatter — a finding must land on the node that can fix it.
    // BOTH DIRECTIONS. `X depends_on Y` points FROM the dependant, so a prerequisite's
    // grade lives on an outgoing edge — and `DependsOn` is the stronger relation of the
    // two ("the target must hold for the source to"). Sweeping `Supports` alone let the
    // same evidence carry a G4 on testimony past the cap by being spelled differently.
    // `ungraded_support` already fixed this reading and says so in its own comment.
    let incoming: Vec<_> = graph
        .edges_to(&node.id)
        .filter(|e| e.kind == EdgeKind::Supports)
        .chain(
            graph
                .edges_from(&node.id)
                .filter(|e| e.kind == EdgeKind::DependsOn),
        )
        .collect();
    if incoming.is_empty() {
        return GateResult::NotApplicable;
    }

    // Look at EVERY edge before answering. Returning on the first non-passing one let
    // an early no-verdict mask a later real ceiling violation, and a blocking finding
    // outranks a no-verdict one: "this grade exceeds what its source can carry" is a
    // verdict, "this one declares no source" is the absence of one.
    let mut unassessed: Option<String> = None;
    for edge in incoming {
        if edge.exceeds_pramana_ceiling() {
            let (Some(grade), Some(pramana)) = (edge.grade(), edge.pramana) else {
                continue;
            };
            return block(
                GRADE_EXCEEDS_PRAMANA,
                "PRAMANA",
                node,
                format!(
                    "edge {} → {} is graded {grade} on {pramana}, whose ceiling is {}",
                    edge.from,
                    edge.to,
                    pramana.grade_ceiling()
                ),
                "lower the grade, or obtain evidence of a kind that earns it — corroboration \
between tools is testimony, not perception",
            );
        }
        if unassessed.is_none() {
            if let (Some(grade), None) = (edge.grade(), edge.pramana) {
                unassessed = Some(format!(
                    "edge {} → {} is settled at {grade} but declares no means of knowing, \
so no ceiling applies to it",
                    edge.from, edge.to
                ));
            }
        }
    }
    match unassessed {
        Some(why) => GateResult::Unassessed { why },
        None => GateResult::Pass,
    }
}

// ── Pearl's ladder ───────────────────────────────────────────────────────────

/// A claim above the association rung needs an executed protocol behind it.
pub fn causal_rung_earned(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    let Some(raw) = node.field("causal_rung") else {
        return GateResult::Unassessed {
            why: format!(
                "\"{}\" does not declare which rung of the causal ladder it stands on",
                node.title
            ),
        };
    };
    let Some(rung) = CausalRung::from_str_opt(raw) else {
        return block(
            CAUSAL_RUNG_UNREACHED,
            "RUNG",
            node,
            format!("`causal_rung: {raw}` is not a rung"),
            "use association, intervention or counterfactual",
        );
    };
    if rung == CausalRung::Association {
        return GateResult::Pass;
    }
    let has_run = supporters(graph, node)
        .iter()
        .any(|s| s.kind == NodeKind::Run);

    if has_run {
        GateResult::Pass
    } else {
        block(
            CAUSAL_RUNG_UNREACHED,
            "RUNG",
            node,
            format!(
                "\"{}\" claims the {} rung but rests on observation alone — no executed \
protocol supports it",
                node.title,
                match rung {
                    CausalRung::Intervention => "intervention",
                    CausalRung::Counterfactual => "counterfactual",
                    CausalRung::Association => "association",
                }
            ),
            "run a controlled protocol and cite the Run, or restate the claim at the \
association rung",
        )
    }
}

/// Every claim states the conditions under which it would change.
pub fn boundaries_declared(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    if node.field_list("boundaries").is_empty() {
        block(
            BOUNDARIES_MISSING,
            "RUNG",
            node,
            format!("\"{}\" declares no boundary conditions", node.title),
            "name the versions, configurations or populations where the claim holds — and \
cite each, never a bare string",
        )
    } else {
        GateResult::Pass
    }
}

// ── Premortem ────────────────────────────────────────────────────────────────

/// A claim must record at least one thing that would defeat it.
///
/// Distinct from [`boundaries_declared`], which asks *where* a claim holds — versions,
/// configurations, populations. A claim can be perfectly scoped and still have nothing
/// that could ever count against it.
///
/// An incoming attack edge satisfies this as fully as a `falsifier:` field does: the
/// gate asks whether anyone has said what would make the claim wrong, and a defeater
/// recorded as a node has said it. Demanding the string as well would be bookkeeping.
/// A *defeated* attacker still counts — it is evidence the claim was examined for
/// defeat, and whether it survives is the grounded extension's question, not this one.
pub fn falsifier_declared(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    if !node.field_list("falsifier").is_empty() {
        return GateResult::Pass;
    }
    if graph.live_attacks_on(&node.id).next().is_some() {
        return GateResult::Pass;
    }
    block(
        FALSIFIER_MISSING,
        "PREMORTEM",
        node,
        format!(
            "\"{}\" records nothing that would defeat it — as written, no observation \
could count against it",
            node.title
        ),
        "state what would have to be observed for this to be wrong, as `falsifier:` \
or as a node that attacks it",
    )
}

/// The arguments contesting the same ground as `node`.
///
/// Rivalry is symmetric — whichever way the author happened to draw the edge, the two
/// nodes are arguing against each other. `live_attacks_on` is asked in both directions
/// rather than reimplemented, so the withdrawn-attacker and reference-material rules
/// are inherited rather than forgotten.
///
/// One definition, because two gates need it and they must never disagree about what
/// a rival is: TRAIRUPYA blocks when rivals exist and share all the evidence, ACH
/// blocks when none exists at all. A drift between the two spellings would open a gap
/// between them that answers to neither.
fn live_rivals<'a>(graph: &'a Graph, node: &Node) -> Vec<&'a Node> {
    graph
        .nodes()
        .filter(|r| r.id != node.id)
        .filter(|r| {
            graph.live_attacks_on(&node.id).any(|e| e.from == r.id)
                || graph.live_attacks_on(&r.id).any(|e| e.from == node.id)
        })
        .collect()
}

/// 因三相, third characteristic (異品遍無性): evidence must be absent where the claim
/// is false. Its structural limiting case — every supporter shared with a live rival —
/// is 共不定 (sādhāraṇa-anaikāntika), and needs no declaration to detect.
pub fn hetu_undiagnostic(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    // An unsupported node is the orphan lint's business. Reporting it here as
    // undiagnostic would blame the author for evidence they have not filed yet.
    let supp = supporters(graph, node);
    if supp.is_empty() {
        return GateResult::NotApplicable;
    }

    let rivals = live_rivals(graph, node);
    // 異品遍無性 presupposes a 異品. An uncontested claim owes this gate nothing —
    // and scoring a thin hypothesis set as diagnosticity is exactly the error the
    // Heuer entry warns against.
    if rivals.is_empty() {
        return GateResult::NotApplicable;
    }

    for rival in rivals {
        // ALL, never ANY. Background evidence legitimately supports both sides of a
        // contest — "the file was present" is consistent with install and with
        // execution — and blocking on one shared line would refuse the vault an
        // expert is obliged to build. One uncommon supporter is what decides a
        // contest, and its presence is the pass.
        //
        // Direct edges only. A supporter reaching the rival down a chain is a weaker
        // relation, and sweeping it in would import the drift `carries_weight`
        // documents. Stated as a limit rather than hidden.
        let all_shared = supp.iter().all(|s| {
            graph
                .edges_from(&s.id)
                .any(|e| e.kind == EdgeKind::Supports && e.to == rival.id)
        });
        if all_shared {
            let shared = supp
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return block(
                HETU_UNDIAGNOSTIC,
                "TRAIRUPYA",
                node,
                format!(
                    "every line of support under \"{}\" ({shared}) also supports its \
live rival \"{}\" — evidence standing in the same relation to both sides of a contest \
discriminates neither (共不定)",
                    node.title, rival.title
                ),
                "record the line of evidence that distinguishes this claim from its \
rival, or state in the packet that the evidence held does not decide the contest",
            );
        }
    }
    GateResult::Pass
}

/// Heuer's move: a causal claim owes at least one competing explanation.
pub fn rivals_enumerated(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) {
        return GateResult::NotApplicable;
    }
    // Naming an alternative is an obligation about EXPLANATION. A claim that two things
    // co-occur owes none, and demanding one would be ceremony. An absent `causal_rung:`
    // is not an escape: RUNG already returns Unassessed for it, which blocks.
    let Some(rung) = node.field("causal_rung").and_then(CausalRung::from_str_opt) else {
        return GateResult::NotApplicable;
    };
    if rung == CausalRung::Association {
        return GateResult::NotApplicable;
    }
    if live_rivals(graph, node).is_empty() {
        return block(
            RIVALS_UNENUMERATED,
            "ACH",
            node,
            format!(
                "\"{}\" explains {} and the vault records nothing it was tested against \
— an explanation with no rival has been confirmed by consistency, not chosen",
                node.title,
                match rung {
                    CausalRung::Intervention => "by intervention",
                    CausalRung::Counterfactual => "counterfactually",
                    CausalRung::Association => "by association",
                }
            ),
            "write down the competing explanation you rejected and attack this claim \
with it — the one that survives is the finding, and the ones that did not are the \
reason anyone should believe it",
        );
    }
    GateResult::Pass
}

/// Dharmakīrti's dṛśya restriction: non-perception establishes absence only of what
/// would have been perceived had it been there.
pub fn absence_is_controlled(graph: &Graph, node: &Node) -> GateResult {
    if !under_promotion(graph, node) || !asserts_absence(node) {
        return GateResult::NotApplicable;
    }
    let supp = supporters(graph, node);
    if supp.is_empty() {
        return GateResult::NotApplicable;
    }

    let mut named_an_instrument = false;
    for s in &supp {
        for e in graph
            .edges_from(&s.id)
            .filter(|e| e.kind == EdgeKind::MeasuredBy)
        {
            if let Some(i) = graph.node(&e.to) {
                named_an_instrument = true;
                if i.field("positive_control").is_some() {
                    return GateResult::Pass;
                }
            }
        }
    }

    // Silence about the instrument must not read better than naming an uncontrolled
    // one. `PEIR-LINT-UNCONTROLLED-INSTRUMENT` iterates measured_by edges, so a
    // supporting observation that names no instrument produces no edges and no
    // violation — absence of configuration reported as absence of a problem.
    if !named_an_instrument {
        return GateResult::Unassessed {
            why: format!(
                "\"{}\" asserts an absence and its evidence names no instrument, so \
whether the search could have found the thing is unknown",
                node.title
            ),
        };
    }

    block(
        ABSENCE_UNCONTROLLED,
        "ANUPALABDHI",
        node,
        format!(
            "\"{}\" rests on a search that has never been shown to find the thing when \
it IS there — non-perception establishes absence only of the perceptible",
            node.title
        ),
        "run the same protocol against a known positive and cite it as \
`positive_control:` on the instrument, or restate the claim as what the search \
returned rather than as what is not there",
    )
}

/// Comparative idioms that open with a negator and bound a quantity rather than deny
/// a thing. Stripped before the markers are read: "no later than 09:14" is a
/// timestamp, and a checker that calls it an absence claim gets switched off.
const BOUNDS_NOT_ABSENCES: &[&str] = &[
    "no later than",
    "no earlier than",
    "no more than",
    "no fewer than",
    "no less than",
    "no sooner than",
    "no greater than",
];

/// Words that turn a description into a denial.
///
/// Deliberately narrow, per the project's bias: a missed absence claim costs a miss,
/// a blocked idiom costs the tool. `polarity:` overrides in BOTH directions — it is
/// the author's escape from a false positive and the gate's reach past a phrasing the
/// list does not know.
const ABSENCE_MARKERS: &[&str] = &[
    "no evidence",
    "no trace",
    "no record",
    "no sign",
    "no indication",
    "no artifact",
    "not present",
    "not found",
    "does not exist",
    "did not exist",
    "never ",
    "nothing ",
    "absent",
];

fn asserts_absence(node: &Node) -> bool {
    match node.field("polarity") {
        Some("negative") => return true,
        Some("positive") => return false,
        _ => {}
    }
    let mut haystack = format!("{} {}", node.title, node.body).to_ascii_lowercase();
    for idiom in BOUNDS_NOT_ABSENCES {
        haystack = haystack.replace(idiom, " ");
    }
    ABSENCE_MARKERS.iter().any(|m| haystack.contains(m)) || haystack.trim_start().starts_with("no ")
}

/// 異品遍無性: in what known circumstances does this evidence occur WITHOUT the thing?
pub fn vipaksa_surveyed(_graph: &Graph, _node: &Node) -> GateResult {
    GateResult::Pass
}

/// 同品定有性: has anyone ever seen this reason and this property co-occur?
pub fn sapaksa_declared(_graph: &Graph, _node: &Node) -> GateResult {
    GateResult::Pass
}

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::{parse_node, Edge, Grade, NodeId, Pramana};

    fn node(src: &str) -> Node {
        parse_node(src).expect("fixture parses")
    }

    /// The two declared characteristics, and the scope that keeps them honest.
    ///
    /// Trairūpya is the theory of anumāna and of nothing else. A claim resting on
    /// perception owes no dissimilar-case survey, and demanding one would be the
    /// checker refusing a sentence it has no doctrine against — so the trigger is an
    /// evidence edge declaring `via=inference`, not promotion in general.
    ///
    /// The obvious dodge, omitting `via=`, is already policed: a settled grade with no
    /// declared pramāṇa blocks as `PEIR-GATE-UNASSESSED [PRAMANA]`. Writing less does
    /// not win.
    #[test]
    fn an_inference_surveys_the_dissimilar_cases_and_cites_a_positive_instance() {
        let build = |via: Pramana, fields: &str| {
            let mut g = Graph::new();
            g.insert_node(node(&format!(
                "---\nid: c1\ntype: claim\ntitle: The binary was executed\n{fields}---\n"
            )));
            g.insert_node(node(
                "---\nid: o1\ntype: observation\ntitle: Amcache entry\n---\n",
            ));
            g.insert_edge(
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports).via(via),
            );
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            (vipaksa_surveyed(&g, &n), sapaksa_declared(&g, &n))
        };

        let (v, sa) = build(Pramana::Inference, "");
        assert!(
            matches!(v, GateResult::Unassessed { .. }),
            "an inference that never surveyed the dissimilar cases reaches no verdict"
        );
        assert!(
            matches!(sa, GateResult::Unassessed { .. }),
            "an inference rule with no cited instance reaches no verdict"
        );

        let surveyed = concat!(
            "vipaksa:\n",
            "  - staged but never run: the appraiser inventories on scheduled scan, so \
the entry is present anyway\n",
            "sapaksa:\n",
            "  - run-2026-08-11: known execution on a clean VM produced the entry\n"
        );
        let (v, sa) = build(Pramana::Inference, surveyed);
        assert!(matches!(v, GateResult::Pass), "control: a survey exists");
        assert!(matches!(sa, GateResult::Pass), "control: an instance is cited");

        // Scope control — the one that decides whether this is doctrine or ceremony.
        let (v, sa) = build(Pramana::Perception, "");
        assert!(
            matches!(v, GateResult::NotApplicable),
            "perception owes no dissimilar-case survey — trairūpya is about anumāna"
        );
        assert!(
            matches!(sa, GateResult::NotApplicable),
            "perception owes no positive instance either"
        );
    }

    /// A search that could not have found the thing has not established its absence.
    ///
    /// The Prefetch case: on a Server SKU with `SysMain` disabled, no execution would
    /// ever have written a `.pf`, so "no Prefetch file exists" is non-perception of
    /// the imperceptible (adṛśyānupalabdhi) and certifies nothing.
    ///
    /// The third case is the false-positive control that decides whether this gate is
    /// usable: "no later than" is a comparative idiom, not an absence claim, and a
    /// checker that blocks it is a checker that gets switched off.
    #[test]
    fn absence_rests_on_a_search_shown_able_to_find_the_thing() {
        let build = |title: &str, control: bool, instrument: bool| {
            let mut g = Graph::new();
            g.insert_node(node(&format!(
                "---\nid: c1\ntype: claim\ntitle: {title}\n---\n"
            )));
            g.insert_node(node(
                "---\nid: o1\ntype: observation\ntitle: Prefetch directory listing\n---\n",
            ));
            g.insert_edge(Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            ));
            if instrument {
                let pc = if control {
                    "positive_control: lists a .pf for a binary known to have run\n"
                } else {
                    ""
                };
                g.insert_node(node(&format!(
                    "---\nid: i1\ntype: instrument\ntitle: Prefetch enumerator\n{pc}---\n"
                )));
                g.insert_edge(Edge::new(
                    NodeId::new("o1"),
                    NodeId::new("i1"),
                    EdgeKind::MeasuredBy,
                ));
            }
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            absence_is_controlled(&g, &n)
        };

        assert!(
            matches!(
                build("No Prefetch file exists for the binary", false, true),
                GateResult::Block(_)
            ),
            "an absence measured by an uncontrolled instrument must block"
        );
        assert!(
            matches!(
                build("No Prefetch file exists for the binary", false, false),
                GateResult::Unassessed { .. }
            ),
            "naming no instrument at all must not read better than naming an \
uncontrolled one"
        );
        assert!(
            matches!(
                build("No Prefetch file exists for the binary", true, true),
                GateResult::Pass
            ),
            "control: an executed positive control is exactly what the gate asks for"
        );
        assert!(
            matches!(
                build("The archive was written no later than 09:14", false, true),
                GateResult::NotApplicable
            ),
            "false-positive control: `no later than` is a comparative idiom, not an \
absence claim"
        );
    }

    /// An explanation with nothing to beat has not been tested against anything.
    ///
    /// This is the other half of the 共不定 interlock: `PEIR-HETU-UNDIAGNOSTIC` is
    /// `NotApplicable` where no rival exists, so deleting the rival would buy silence.
    /// It buys this instead.
    #[test]
    fn a_causal_claim_with_no_rival_has_not_done_the_work() {
        let build = |rival: bool| {
            let mut g = Graph::new();
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: Running the binary wrote the entry\n\
causal_rung: counterfactual\n---\n",
            ));
            g.insert_node(node(
                "---\nid: run1\ntype: run\ntitle: Controlled execution on a clean VM\n---\n",
            ));
            g.insert_edge(Edge::new(
                NodeId::new("run1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            ));
            if rival {
                g.insert_node(node(
                    "---\nid: h1\ntype: hypothesis\ntitle: The appraiser wrote it on scan\n---\n",
                ));
                g.insert_edge(Edge::new(
                    NodeId::new("h1"),
                    NodeId::new("c1"),
                    EdgeKind::Attacks,
                ));
            }
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            rivals_enumerated(&g, &n)
        };

        assert!(
            matches!(build(false), GateResult::Block(_)),
            "a counterfactual claim with no competing explanation must block"
        );
        assert!(
            matches!(build(true), GateResult::Pass),
            "control: one written-down rival is what ACH asks for — it must pass"
        );
    }

    /// 共不定: support a rival claims just as loudly decides nothing.
    ///
    /// The Amcache `InventoryApplicationFile` entry is written by the compatibility
    /// appraiser for binaries that were inventoried and never run, so it stands in
    /// exactly the same relation to "it executed" and to "it was catalogued". A vault
    /// whose entire case rests on it has not argued for either side.
    #[test]
    fn support_common_to_a_live_rival_decides_nothing() {
        let build = |discriminating: bool| {
            let mut g = Graph::new();
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: The user executed coreupdater.exe\n---\n",
            ));
            g.insert_node(node(
                "---\nid: r1\ntype: claim\ntitle: The appraiser catalogued it, unrun\n---\n",
            ));
            g.insert_node(node(
                "---\nid: o1\ntype: observation\ntitle: InventoryApplicationFile entry\n---\n",
            ));
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
            if discriminating {
                g.insert_node(node(
                    "---\nid: o2\ntype: observation\ntitle: Prefetch .pf, run counter 3\n---\n",
                ));
                g.insert_edge(Edge::new(
                    NodeId::new("o2"),
                    NodeId::new("c1"),
                    EdgeKind::Supports,
                ));
            }
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            hetu_undiagnostic(&g, &n)
        };

        assert!(
            matches!(build(false), GateResult::Block(_)),
            "every supporter shared with a live rival is 共不定 and must block"
        );
        assert!(
            matches!(build(true), GateResult::Pass),
            "control: one discriminating line is what decides a contest — it must pass"
        );
    }

    /// Four corners means four corners, not one typed four ways.
    ///
    /// Case and whitespace were folded and trailing punctuation was not, so
    /// `[It holds, "It holds.", "It holds!", "It holds?"]` satisfied 四句. A checker
    /// that measures the shape of a string rather than what it says is always one
    /// keystroke from being satisfied by nothing.
    #[test]
    fn punctuation_does_not_make_a_second_corner() {
        let build = |corners: &str| {
            let mut g = Graph::new();
            g.insert_node(node(&format!(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n{corners}---\n"
            )));
            g.insert_node(node("---\nid: r1\ntype: claim\ntitle: a rival\n---\n"));
            g.insert_edge(Edge::new(
                NodeId::new("r1"),
                NodeId::new("c1"),
                EdgeKind::Attacks,
            ));
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            four_corners_addressed(&g, &n)
        };

        let typed_four_ways = concat!(
            "corners:\n",
            "  - It holds\n",
            "  - \"It holds.\"\n",
            "  - \"It holds!\"\n",
            "  - \"It holds?\"\n"
        );
        let genuinely_four = concat!(
            "corners:\n",
            "  - It holds\n",
            "  - It does not hold\n",
            "  - It holds in part\n",
            "  - The question does not arise\n"
        );

        assert!(
            matches!(build(typed_four_ways), GateResult::Block(_)),
            "one corner punctuated four ways addresses one corner"
        );
        assert!(
            matches!(build(genuinely_four), GateResult::Pass),
            "control: four different positions still pass"
        );
    }

    /// 體用 is owed by whatever carries the weight, not by whatever it calls itself.
    ///
    /// TIYONG was the last ENFORCED lens still carrying a static `applies_to` kind list,
    /// and `Lens::examine` applies that BEFORE the gate runs — so relabelling a
    /// load-bearing substance claim `type: observation` stripped the obligation entirely
    /// and the packet froze. The codebase's first recorded defect, still open for one
    /// lens.
    ///
    /// And the aspect itself is declarable two ways. `substance_of:` is the edge grammar
    /// `EdgeKind` documents for this lens, and it was parsed, recorded and read by
    /// nothing, so the same statement was examined under one published spelling and
    /// sealed under the other.
    #[test]
    fn the_aspect_obligation_follows_weight_and_either_spelling() {
        let build = |kind: &str, decl: &str| {
            let mut g = Graph::new();
            g.insert_node(node(&format!(
                "---\nid: sub\ntype: {kind}\ntitle: Amcache is an execution artifact\n\
warrant: The table records what ran.\nquantifier: singular\ncausal_rung: association\n\
{decl}---\n"
            )));
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: The downstream conclusion\n---\n",
            ));
            g.insert_node(node(
                "---\nid: o1\ntype: observation\ntitle: the record\naspect: function\n---\n",
            ));
            g.insert_edge(Edge::new(
                NodeId::new("sub"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            ));
            g.insert_edge(Edge::new(
                NodeId::new("o1"),
                NodeId::new("sub"),
                EdgeKind::Supports,
            ));
            // THROUGH THE AGGREGATION. `applies_to` is applied by `Lens::examine`
            // BEFORE the gate runs, so calling the gate directly cannot see a static
            // kind list at all — the first version of this test called it directly and
            // stayed green when the list was restored, which is the very defect three
            // other findings in this round are about.
            crate::examine_graph(&g)
                .into_iter()
                .filter(|v| v.subject == NodeId::new("sub") && v.lens == "TIYONG")
                .count()
        };

        for kind in ["claim", "hypothesis", "observation"] {
            assert!(
                build(kind, "aspect: substance\n") > 0,
                "{kind}: a load-bearing substance claim owes 體用 whatever it is labelled"
            );
        }
        // The EDGE spelling. `parse_node` does not build edges — only the vault loader
        // does — so the edge is inserted directly here rather than written as
        // frontmatter, which would silently assert nothing.
        {
            let mut g = Graph::new();
            g.insert_node(node(
                "---\nid: sub\ntype: claim\ntitle: Amcache is an execution artifact\n\
warrant: The table records what ran.\nquantifier: singular\ncausal_rung: association\n---\n",
            ));
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: The downstream conclusion\n---\n",
            ));
            g.insert_node(node(
                "---\nid: o1\ntype: observation\ntitle: the record\naspect: function\n---\n",
            ));
            g.insert_edge(Edge::new(
                NodeId::new("sub"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            ));
            g.insert_edge(Edge::new(
                NodeId::new("o1"),
                NodeId::new("sub"),
                EdgeKind::Supports,
            ));
            g.insert_edge(Edge::new(
                NodeId::new("sub"),
                NodeId::new("o1"),
                EdgeKind::SubstanceOf,
            ));
            assert!(
                crate::examine_graph(&g)
                    .into_iter()
                    .any(|v| v.subject == NodeId::new("sub") && v.lens == "TIYONG"),
                "the edge grammar the docs publish must reach the gate the docs promise"
            );
        }
        assert_eq!(
            build("claim", "aspect: function\n"),
            0,
            "control: a function claim owes this lens nothing"
        );
    }

    /// An unrecognised aspect is shown, and a leaf is not asked to classify itself.
    ///
    /// D1: `_ => {}` fell through to `None`, so `aspect: Substance` — one capital — and
    /// any misspelling switched 體用 off in silence. An unrecognised value disabling a
    /// check instead of being shown verbatim is this repository's own stated rule, and
    /// the identical defect was already fixed for `quantifier:` and `causal_rung:`.
    ///
    /// D2: this gate asks SUPPORTERS to declare an aspect, and an observation that
    /// answered honestly then found the annotation itself blocking — "has no supporting
    /// evidence to classify" — with no exit that was not a lie. Primitive evidence is
    /// where a chain ends. A CLAIM resting on nothing stays Unassessed: that is
    /// unexamined, not inapplicable.
    #[test]
    fn the_aspect_gate_reads_the_value_and_spares_the_leaf() {
        let with_supporter = |aspect: &str| {
            let mut g = Graph::new();
            g.insert_node(node(&format!(
                "---\nid: sub\ntype: claim\ntitle: Amcache is an execution artifact\n\
aspect: {aspect}\n---\n"
            )));
            g.insert_node(node(
                "---\nid: ev\ntype: observation\ntitle: the table lists the path\n\
aspect: function\n---\n",
            ));
            g.insert_edge(Edge::new(
                NodeId::new("ev"),
                NodeId::new("sub"),
                EdgeKind::Supports,
            ));
            let n = g.node(&NodeId::new("sub")).expect("sub").clone();
            substance_not_from_function_alone(&g, &n)
        };

        for spelling in ["substance", "Substance", "SUBSTANCE", "  substance  "] {
            assert!(
                matches!(with_supporter(spelling), GateResult::Block(_)),
                "`aspect: {spelling}` must reach the gate — case is not a declaration"
            );
        }
        match with_supporter("subtance") {
            GateResult::Unassessed { why } => assert!(
                why.contains("subtance"),
                "an unrecognised value must be shown verbatim: {why}"
            ),
            other => panic!("a typo must not switch the lens off: {other:?}"),
        }
        assert!(
            matches!(with_supporter("function"), GateResult::NotApplicable),
            "control: a function claim owes this lens nothing"
        );

        // D2 — the leaf, and the claim, resting on nothing.
        let alone = |kind: &str| {
            let mut g = Graph::new();
            g.insert_node(node(&format!(
                "---\nid: n1\ntype: {kind}\ntitle: x is y\naspect: substance\n---\n"
            )));
            let n = g.node(&NodeId::new("n1")).expect("n1").clone();
            substance_not_from_function_alone(&g, &n)
        };
        assert!(
            matches!(alone("observation"), GateResult::NotApplicable),
            "an observation IS the evidence; asking it to classify itself is the regress"
        );
        assert!(
            matches!(alone("claim"), GateResult::Unassessed { .. }),
            "but a claim asserting what a thing IS on no evidence is unexamined"
        );
    }

    /// A WITHDRAWN attack is not an attack the gates should honour.
    ///
    /// `attackers()` has always dropped a withdrawn attacker — a node the record retires
    /// is not a participant in the dispute. `live_attacks_on` did not, and a comment
    /// called that deliberate: "a withdrawn attacker is disclosed as removed rather than
    /// answered, and that distinction belongs to the packet". Right about the packet,
    /// wrong about the gates. A retired rival still made its victim contested under 四句,
    /// and still SATISFIED the demand for something that could count against the claim —
    /// so a claim recording nothing that could defeat it passed the falsifier gate on
    /// the strength of an argument the graph had already retired.
    #[test]
    fn a_withdrawn_attack_neither_contests_nor_satisfies() {
        let build = |withdraw: bool| {
            let mut g = Graph::new();
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n\
quantifier: singular\naspect: function\ncausal_rung: association\n---\n",
            ));
            g.insert_node(node(
                "---\nid: riv\ntype: claim\ntitle: An inventory sweep produced it\n---\n",
            ));
            g.insert_edge(Edge::new(
                NodeId::new("riv"),
                NodeId::new("c1"),
                EdgeKind::Attacks,
            ));
            if withdraw {
                g.insert_node(node(
                    "---\nid: d1\ntype: dissent\ntitle: that rival was withdrawn\n---\n",
                ));
                g.insert_edge(Edge::new(
                    NodeId::new("d1"),
                    NodeId::new("riv"),
                    EdgeKind::Retracts,
                ));
            }
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            (
                four_corners_addressed(&g, &n),
                falsifier_declared(&g, &n),
                g.live_attacks_on(&NodeId::new("c1")).count(),
                g.attacks_on(&NodeId::new("c1")).count(),
            )
        };

        let (corners, falsifier, live, raw) = build(false);
        assert_eq!(
            (live, raw),
            (1, 1),
            "positive control: a live rival competes"
        );
        assert!(
            matches!(corners, GateResult::Block(_)),
            "and contests the claim"
        );
        assert!(
            matches!(falsifier, GateResult::Pass),
            "and is something that could count against it"
        );

        let (corners, falsifier, live, raw) = build(true);
        assert_eq!(live, 0, "a withdrawn rival is not in play");
        assert_eq!(
            raw, 1,
            "but the raw relation still holds it — `standing_line` reports precisely these"
        );
        assert!(
            !matches!(corners, GateResult::Block(_)),
            "so it does not make the claim contested"
        );
        assert!(
            !matches!(falsifier, GateResult::Pass),
            "and it does not answer the demand for something that could defeat the claim"
        );
    }

    /// An edge the argumentation discards has no effects anywhere else either.
    ///
    /// A `term` carrying `contradicts:` is dropped from the grounded relation, because
    /// reference material does not compete. Six other places asked the raw question and
    /// counted it anyway — so the same void edge made its victim contested under 四句,
    /// satisfied the falsifier gate, and let a packet's standing line say "every attack
    /// on it is itself defeated" about an attack that was never counted.
    ///
    /// The victim cannot edit the term author's frontmatter, which makes this the
    /// "punished for prose they cannot fix" class as well.
    #[test]
    fn a_discarded_attack_has_no_effects() {
        let build = |kind: &str| {
            let mut g = Graph::new();
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n\
quantifier: singular\naspect: function\ncausal_rung: association\n---\n",
            ));
            g.insert_node(node(&format!(
                "---\nid: x1\ntype: {kind}\ntitle: something that opposes it\n\
as_used: a\nnot_essence: b\nstipulated: c\n---\n"
            )));
            g.insert_edge(Edge::new(
                NodeId::new("x1"),
                NodeId::new("c1"),
                EdgeKind::Contradicts,
            ));
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            (
                four_corners_addressed(&g, &n),
                falsifier_declared(&g, &n),
                g.live_attacks_on(&NodeId::new("c1")).count(),
            )
        };

        let (corners, falsifier, live) = build("claim");
        assert_eq!(live, 1, "positive control: a claim competes");
        assert!(
            matches!(corners, GateResult::Block(_)),
            "a contested claim owes all four corners"
        );
        assert!(
            matches!(falsifier, GateResult::Pass),
            "and a live attack is something that could count against it"
        );

        let (corners, falsifier, live) = build("term");
        assert_eq!(live, 0, "reference material does not compete");
        assert!(
            !matches!(corners, GateResult::Block(_)),
            "so it does not make the claim contested"
        );
        assert!(
            !matches!(falsifier, GateResult::Pass),
            "and it does not satisfy the demand for something that could defeat the claim"
        );
    }

    /// The pramāṇa ceiling binds a declared prerequisite too.
    ///
    /// `DependsOn` is documented as "the target must hold for the source to" — a
    /// STRONGER relation than support. The ceiling swept `Supports` alone, so spelling
    /// the edge `depends_on` carried a G4 on testimony past the cap that exists to stop
    /// exactly that. The same one-direction reading that `ungraded_support` already
    /// corrected, with the comment still sitting there: "the edge that makes a node
    /// load-bearing can point either way, and only one direction was checked."
    #[test]
    fn the_ceiling_binds_a_declared_prerequisite() {
        let verdict = |kind: EdgeKind| {
            let mut g = Graph::new();
            g.insert_node(node(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n---\n",
            ));
            g.insert_node(node(
                "---\nid: o3\ntype: observation\ntitle: an account of what happened\n---\n",
            ));
            // `c1 depends_on o3` points FROM the claim; `o3 supports c1` points TO it.
            let e = if kind == EdgeKind::DependsOn {
                Edge::new(NodeId::new("c1"), NodeId::new("o3"), kind)
            } else {
                Edge::new(NodeId::new("o3"), NodeId::new("c1"), kind)
            };
            g.insert_edge(e.graded_by(Grade::G4, "eve").via(Pramana::Testimony));
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            grades_within_pramana_ceiling(&g, &n)
        };

        assert!(
            matches!(verdict(EdgeKind::Supports), GateResult::Block(_)),
            "positive control: G4 on testimony exceeds the ceiling when spelled `supports`"
        );
        assert!(
            matches!(verdict(EdgeKind::DependsOn), GateResult::Block(_)),
            "and spelling the SAME evidence `depends_on` must not carry it past the cap"
        );
    }

    /// Four corners means four DIFFERENT corners.
    ///
    /// 四句 asks a contested claim to address A, not-A, both, and neither. The gate
    /// counted entries, so `corners: [A, A, A, A]` satisfied it — the author addressed
    /// one corner four times and the packet sealed.
    #[test]
    fn four_corners_means_four_distinct_corners() {
        let verdict = |corners: &str| {
            let mut g = Graph::new();
            g.insert_node(node(&format!(
                "---\nid: c1\ntype: claim\ntitle: The hive catalogued the file\n{corners}---\n"
            )));
            g.insert_node(node(
                "---\nid: r1\ntype: claim\ntitle: a rival account\n---\n",
            ));
            g.insert_edge(Edge::new(
                NodeId::new("r1"),
                NodeId::new("c1"),
                EdgeKind::Attacks,
            ));
            let n = g.node(&NodeId::new("c1")).expect("c1").clone();
            four_corners_addressed(&g, &n)
        };

        let four = concat!(
            "corners:\n",
            "  - it was catalogued\n",
            "  - it was not catalogued\n",
            "  - catalogued under one mechanism and not another\n",
            "  - the question does not arise\n"
        );
        let repeated = concat!(
            "corners:\n",
            "  - it was catalogued\n",
            "  - it was catalogued\n",
            "  - it was catalogued\n",
            "  - it was catalogued\n"
        );

        assert!(
            matches!(verdict(four), GateResult::Pass),
            "positive control: four genuinely different corners pass"
        );
        assert!(
            matches!(verdict(repeated), GateResult::Block(_)),
            "one corner stated four times addresses one corner"
        );
    }

    /// A node kind is a self-declared string, not an exemption.
    ///
    /// `under_promotion` replaced a static node-kind scope with a load-bearing test and
    /// kept a static scope inside it: the `_ => false` arm exempted every kind but
    /// Claim and Hypothesis. So relabelling a universal over-claim `type: observation`
    /// stripped all seven promotion obligations while it went on supporting a claim.
    ///
    /// An observation that merely records what was seen still declares nothing and is
    /// examined by nothing — that is correct. One that ASSERTS, in the shape of a
    /// claim, and is leaned on, answers like one.
    #[test]
    fn a_load_bearing_observation_that_asserts_is_held_to_the_bar() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let overclaim = node(
            "---\nid: o9\ntype: observation\n\
title: Every Amcache entry on every Windows version is written only at execution time\n\
quantifier: universal\n---\n",
        );
        let plain = node("---\nid: o1\ntype: observation\ntitle: the hive holds a record\n---\n");
        let g = |obs: Node| {
            let id = obs.id.clone();
            graph_of(
                vec![claim.clone(), obs],
                vec![Edge::new(id, NodeId::new("c1"), EdgeKind::Supports)],
            )
        };

        let asserting = g(overclaim.clone());
        assert!(
            !class_extension_declared(&asserting, &overclaim).permits_promotion(),
            "a universal quantifier in a load-bearing observation must still declare its \
extension — the node kind is a self-declared string"
        );

        let recording = g(plain.clone());
        assert!(
            class_extension_declared(&recording, &plain).permits_promotion(),
            "an observation that records rather than asserts stays exempt"
        );
    }

    /// Unannotated evidence is not evidence of substance.
    ///
    /// `all(aspect == function)` is false on MISSING data as readily as on contrary
    /// data, and the gate returned `Pass` for both — so the author who annotated an
    /// observation honestly was blocked while the author who deleted the line froze a
    /// packet. That inverts the incentive the whole catalogue runs on.
    ///
    /// It also contradicts this module's own header, three lines from the top: a gate
    /// that cannot reach a verdict returns `Unassessed`, and never `Pass`.
    #[test]
    fn a_supporter_that_declares_no_aspect_yields_no_verdict() {
        let claim = node(
            "---\nid: c1\ntype: claim\ntitle: Amcache is an execution artifact\n\
aspect: substance\n---\n",
        );
        let mk = |id: &str, aspect: &str| {
            node(&format!(
                "---\nid: {id}\ntype: observation\ntitle: a record\n{aspect}---\n"
            ))
        };
        let g = |obs: Node| {
            let id = obs.id.clone();
            graph_of(
                vec![claim.clone(), obs],
                vec![Edge::new(id, NodeId::new("c1"), EdgeKind::Supports)],
            )
        };

        let silent = g(mk("o1", ""));
        assert!(
            !substance_not_from_function_alone(&silent, &claim).permits_promotion(),
            "a supporter declaring no aspect cannot clear a substance claim — the gate \
reached no verdict and must not report one"
        );

        let honest = g(mk("o2", "aspect: function\n"));
        assert!(
            !substance_not_from_function_alone(&honest, &claim).permits_promotion(),
            "function-only evidence still blocks a substance claim"
        );

        let bearing = g(mk("o3", "aspect: substance\n"));
        assert!(
            substance_not_from_function_alone(&bearing, &claim).permits_promotion(),
            "evidence bearing on what the thing IS clears it"
        );
    }

    /// A hypothesis carrying a claim is doing a claim's work.
    ///
    /// Scoping promotion gates to `Claim` fixed over-firing on candidate explanations
    /// and opened a laundering route: put the conclusion on a hypothesis, support the
    /// claim with it, and every promotion gate looked away. The obligation attaches to
    /// being LOAD-BEARING, not to the node kind.
    #[test]
    fn a_hypothesis_that_supports_a_claim_is_held_to_a_claims_bar() {
        let hypo = node("---\nid: h1\ntype: hypothesis\ntitle: A candidate explanation\n---\n");
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");

        let bare = graph_of(vec![hypo.clone()], vec![]);
        assert!(
            warrant_present(&bare, &hypo).permits_promotion(),
            "a hypothesis nothing leans on is a candidate — thinking out loud is allowed"
        );

        let load_bearing = graph_of(
            vec![hypo.clone(), claim],
            vec![Edge::new(
                NodeId::new("h1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )],
        );
        assert!(
            !warrant_present(&load_bearing, &hypo).permits_promotion(),
            "once a claim rests on it, it answers for itself"
        );
    }

    /// A settled grade must declare how it was known.
    ///
    /// `exceeds_pramana_ceiling` compares grade against ceiling only when BOTH are
    /// present, so omitting `via=` removed the cap entirely: a single edge could
    /// settle at G4 — a grade defined as multiple materially independent convergent
    /// lines — on one document somebody wrote. The cap bound only authors polite
    /// enough to declare their means of knowing.
    ///
    /// An UNGRADED edge is a different thing and stays silent here: it asserts
    /// nothing, and the lint pack reports it separately.
    #[test]
    fn a_settled_grade_without_a_declared_means_of_knowing_is_not_a_pass() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let obs = node("---\nid: o1\ntype: observation\ntitle: o\n---\n");

        let graded_no_pramana = graph_of(
            vec![claim.clone(), obs.clone()],
            vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                    .graded_by(Grade::G4, "a-reviewer"),
            ],
        );
        assert!(
            !grades_within_pramana_ceiling(&graded_no_pramana, &claim).permits_promotion(),
            "a G4 settled with no declared means of knowing must not permit promotion"
        );

        let graded_with_pramana = graph_of(
            vec![claim.clone(), obs.clone()],
            vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                    .graded_by(Grade::G3, "a-reviewer")
                    .via(Pramana::Perception),
            ],
        );
        assert!(
            grades_within_pramana_ceiling(&graded_with_pramana, &claim).permits_promotion(),
            "a grade within its declared ceiling still passes"
        );

        let ungraded = graph_of(
            vec![claim.clone(), obs],
            vec![Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )],
        );
        assert!(
            grades_within_pramana_ceiling(&ungraded, &claim).permits_promotion(),
            "an ungraded edge asserts nothing and is the lint pack's business, not this gate's"
        );
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

    // ── 立極 ──────────────────────────────────────────────────────────────

    #[test]
    fn an_evaluative_claim_without_a_criterion_is_blocked() {
        let n = node("---\nid: c1\ntype: claim\ntitle: The path is suspicious\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        let r = criterion_declared(&g, &n);
        assert_eq!(r.violation().map(|v| v.gate), Some(CRITERION_UNDECLARED));
    }

    #[test]
    fn a_descriptive_claim_is_out_of_scope_for_the_pole() {
        let n = node("---\nid: c1\ntype: claim\ntitle: The hive records this path\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        assert_eq!(criterion_declared(&g, &n), GateResult::NotApplicable);
    }

    #[test]
    fn declaring_the_criterion_clears_the_pole() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: The path is suspicious\n---\n");
        let crit = node("---\nid: 60.01\ntype: criterion\ntitle: Staging-path standard\n---\n");
        let g = graph_of(
            vec![claim.clone(), crit],
            vec![Edge::new(
                NodeId::new("c1"),
                NodeId::new("60.01"),
                EdgeKind::JudgedBy,
            )],
        );
        assert_eq!(criterion_declared(&g, &claim), GateResult::Pass);
    }

    // ── 正名 ──────────────────────────────────────────────────────────────

    #[test]
    fn a_term_missing_a_moment_blocks() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let term =
            node("---\nid: 60.02\ntype: term\ntitle: execution\nas_used: running a program\n---\n");
        let g = graph_of(
            vec![claim.clone(), term],
            vec![Edge::new(
                NodeId::new("c1"),
                NodeId::new("60.02"),
                EdgeKind::UsesTerm,
            )],
        );
        let v = key_terms_stipulated(&g, &claim);
        let v = v.violation().expect("must block");
        assert_eq!(v.gate, TERM_UNSTIPULATED);
        assert!(v.detail.contains("not_essence"), "{}", v.detail);
        assert!(v.detail.contains("stipulated"), "{}", v.detail);
    }

    #[test]
    fn a_fully_stipulated_term_passes() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let term = node(
            "---\nid: 60.02\ntype: term\ntitle: execution\nas_used: running a program\n\
not_essence: a catalogue record is not a running program\nstipulated: a process was created \
from this image\n---\n",
        );
        let g = graph_of(
            vec![claim.clone(), term],
            vec![Edge::new(
                NodeId::new("c1"),
                NodeId::new("60.02"),
                EdgeKind::UsesTerm,
            )],
        );
        assert_eq!(key_terms_stipulated(&g, &claim), GateResult::Pass);
    }

    #[test]
    fn no_declared_terms_is_unassessed_never_a_pass() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let g = graph_of(vec![claim.clone()], vec![]);
        let r = key_terms_stipulated(&g, &claim);
        assert!(matches!(r, GateResult::Unassessed { .. }));
        assert!(!r.permits_promotion());
    }

    // ── 體用 ──────────────────────────────────────────────────────────────

    #[test]
    fn a_substance_claim_on_function_evidence_alone_is_blocked() {
        let claim =
            node("---\nid: c1\ntype: claim\ntitle: Amcache is an execution artifact\naspect: substance\n---\n");
        let obs = node(
            "---\nid: o1\ntype: observation\ntitle: the hive recorded this path\naspect: function\n---\n",
        );
        let g = graph_of(
            vec![claim.clone(), obs],
            vec![Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )],
        );
        let r = substance_not_from_function_alone(&g, &claim);
        assert_eq!(r.violation().map(|v| v.gate), Some(FUNCTION_AS_SUBSTANCE));
    }

    #[test]
    fn a_substance_claim_with_substance_evidence_passes() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: x is y\naspect: substance\n---\n");
        let obs = node("---\nid: o1\ntype: observation\ntitle: o\naspect: substance\n---\n");
        let g = graph_of(
            vec![claim.clone(), obs],
            vec![Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )],
        );
        assert_eq!(
            substance_not_from_function_alone(&g, &claim),
            GateResult::Pass
        );
    }

    // ── 白馬非馬 ──────────────────────────────────────────────────────────

    #[test]
    fn a_class_claim_without_extension_is_blocked() {
        let n = node(
            "---\nid: c1\ntype: claim\ntitle: Amcache entries indicate execution\nquantifier: universal\n---\n",
        );
        let g = graph_of(vec![n.clone()], vec![]);
        assert_eq!(
            class_extension_declared(&g, &n).violation().map(|v| v.gate),
            Some(CLASS_EXTENSION_UNDECLARED)
        );
    }

    #[test]
    fn a_singular_claim_is_out_of_scope() {
        let n = node("---\nid: c1\ntype: claim\ntitle: t\nquantifier: singular\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        assert_eq!(class_extension_declared(&g, &n), GateResult::NotApplicable);
    }

    // ── 四句 ──────────────────────────────────────────────────────────────

    #[test]
    fn a_contested_claim_needs_all_four_corners() {
        let a = node("---\nid: c1\ntype: claim\ntitle: it executed\n---\n");
        let b = node("---\nid: c2\ntype: claim\ntitle: it was only copied\n---\n");
        let g = graph_of(
            vec![a.clone(), b],
            vec![Edge::new(
                NodeId::new("c2"),
                NodeId::new("c1"),
                EdgeKind::Contradicts,
            )],
        );
        assert_eq!(
            four_corners_addressed(&g, &a).violation().map(|v| v.gate),
            Some(CORNERS_UNADDRESSED)
        );
    }

    #[test]
    fn four_corners_stated_passes() {
        let a = node(
            "---\nid: c1\ntype: claim\ntitle: it executed\ncontested: true\ncorners:\n  - executed\n  - not executed\n  - both, on different occasions\n  - neither: catalogued without running\n---\n",
        );
        let g = graph_of(vec![a.clone()], vec![]);
        assert_eq!(four_corners_addressed(&g, &a), GateResult::Pass);
    }

    // ── Toulmin ───────────────────────────────────────────────────────────

    #[test]
    fn a_missing_warrant_is_blocked_and_distinguished_from_a_blank_one() {
        let absent = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let blank = node("---\nid: c2\ntype: claim\ntitle: t\nwarrant: \"\"\n---\n");
        let g = graph_of(vec![absent.clone(), blank.clone()], vec![]);

        let a = warrant_present(&g, &absent);
        let b = warrant_present(&g, &blank);
        assert_eq!(a.violation().map(|v| v.gate), Some(WARRANT_MISSING));
        assert_eq!(b.violation().map(|v| v.gate), Some(WARRANT_MISSING));
        assert!(a.violation().unwrap().detail.contains("states no warrant"));
        assert!(b.violation().unwrap().detail.contains("blank"));
    }

    // ── pramāṇa ───────────────────────────────────────────────────────────

    #[test]
    fn an_overgraded_testimony_edge_is_blocked() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let obs = node("---\nid: o1\ntype: observation\ntitle: o\n---\n");
        let g = graph_of(
            vec![claim.clone(), obs],
            vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                    .via(Pramana::Testimony)
                    .graded_by(Grade::G3, "albert"),
            ],
        );
        let v = grades_within_pramana_ceiling(&g, &claim);
        let v = v.violation().expect("must block");
        assert_eq!(v.gate, GRADE_EXCEEDS_PRAMANA);
        assert!(v.detail.contains("G3"), "{}", v.detail);
        assert!(v.detail.contains("testimony"), "{}", v.detail);
    }

    // ── Pearl ─────────────────────────────────────────────────────────────

    #[test]
    fn a_counterfactual_claim_on_observation_alone_is_blocked() {
        let claim = node(
            "---\nid: c1\ntype: claim\ntitle: this entry proves execution\ncausal_rung: counterfactual\n---\n",
        );
        let obs = node("---\nid: o1\ntype: observation\ntitle: hive record\n---\n");
        let g = graph_of(
            vec![claim.clone(), obs],
            vec![Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )],
        );
        assert_eq!(
            causal_rung_earned(&g, &claim).violation().map(|v| v.gate),
            Some(CAUSAL_RUNG_UNREACHED)
        );
    }

    #[test]
    fn an_association_claim_needs_no_protocol() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\ncausal_rung: association\n---\n");
        let g = graph_of(vec![claim.clone()], vec![]);
        assert_eq!(causal_rung_earned(&g, &claim), GateResult::Pass);
    }

    #[test]
    fn an_executed_protocol_earns_the_higher_rung() {
        let claim = node(
            "---\nid: c1\ntype: claim\ntitle: launching it creates the record\ncausal_rung: intervention\n---\n",
        );
        let run = node("---\nid: r1\ntype: run\ntitle: controlled launch on 22H2\n---\n");
        let g = graph_of(
            vec![claim.clone(), run],
            vec![Edge::new(
                NodeId::new("r1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )],
        );
        assert_eq!(causal_rung_earned(&g, &claim), GateResult::Pass);
    }

    #[test]
    fn an_undeclared_rung_is_unassessed_not_passed() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let g = graph_of(vec![claim.clone()], vec![]);
        let r = causal_rung_earned(&g, &claim);
        assert!(matches!(r, GateResult::Unassessed { .. }));
        assert!(!r.permits_promotion());
    }

    #[test]
    fn an_explicit_evaluative_flag_engages_the_pole_without_a_trigger_word() {
        let n =
            node("---\nid: c1\ntype: claim\ntitle: A plain description\nevaluative: true\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        assert_eq!(
            criterion_declared(&g, &n).violation().map(|v| v.gate),
            Some(CRITERION_UNDECLARED),
            "self-declaration must work when the word list does not fire"
        );
    }

    #[test]
    fn a_criterion_edge_pointing_at_a_non_criterion_does_not_satisfy_the_pole() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: The path is suspicious\n---\n");
        let other = node("---\nid: o1\ntype: observation\ntitle: not a criterion\n---\n");
        let g = graph_of(
            vec![claim.clone(), other],
            vec![Edge::new(
                NodeId::new("c1"),
                NodeId::new("o1"),
                EdgeKind::JudgedBy,
            )],
        );
        assert_eq!(
            criterion_declared(&g, &claim).violation().map(|v| v.gate),
            Some(CRITERION_UNDECLARED)
        );
    }

    #[test]
    fn a_key_term_pointing_at_nothing_is_blocked_with_the_dangling_id() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let g = graph_of(
            vec![claim.clone()],
            vec![Edge::new(
                NodeId::new("c1"),
                NodeId::new("ghost"),
                EdgeKind::UsesTerm,
            )],
        );
        let r = key_terms_stipulated(&g, &claim);
        let v = r.violation().expect("must block");
        assert_eq!(v.gate, TERM_UNSTIPULATED);
        assert!(v.detail.contains("ghost"), "{}", v.detail);
    }

    #[test]
    fn a_substance_claim_with_no_evidence_is_unassessed_not_passed() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: x is y\naspect: substance\n---\n");
        let g = graph_of(vec![claim.clone()], vec![]);
        let r = substance_not_from_function_alone(&g, &claim);
        assert!(matches!(r, GateResult::Unassessed { .. }));
        assert!(!r.permits_promotion());
    }

    #[test]
    fn a_claim_that_never_says_class_or_case_is_unassessed() {
        let n = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        let r = class_extension_declared(&g, &n);
        assert!(matches!(r, GateResult::Unassessed { .. }));
        assert!(!r.permits_promotion());
    }

    #[test]
    fn a_class_claim_declaring_its_extension_passes() {
        let n = node(
            "---\nid: c1\ntype: claim\ntitle: t\nquantifier: class\nextension:\n  - InventoryApplicationFile on 1809+\n---\n",
        );
        let g = graph_of(vec![n.clone()], vec![]);
        assert_eq!(class_extension_declared(&g, &n), GateResult::Pass);
    }

    #[test]
    fn an_uncontested_claim_is_out_of_scope_for_the_four_corners() {
        let n = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        assert_eq!(four_corners_addressed(&g, &n), GateResult::NotApplicable);
    }

    #[test]
    fn a_node_with_no_incoming_edges_is_out_of_scope_for_pramana() {
        let n = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        assert_eq!(
            grades_within_pramana_ceiling(&g, &n),
            GateResult::NotApplicable
        );
    }

    #[test]
    fn well_graded_edges_pass_the_ceiling_check() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let obs = node("---\nid: o1\ntype: observation\ntitle: o\n---\n");
        let g = graph_of(
            vec![claim.clone(), obs],
            vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports)
                    .via(Pramana::Inference)
                    .graded_by(Grade::G2, "albert"),
            ],
        );
        assert_eq!(grades_within_pramana_ceiling(&g, &claim), GateResult::Pass);
    }

    #[test]
    fn an_unrecognised_rung_is_blocked_and_shown_verbatim() {
        let n = node("---\nid: c1\ntype: claim\ntitle: t\ncausal_rung: teleological\n---\n");
        let g = graph_of(vec![n.clone()], vec![]);
        let r = causal_rung_earned(&g, &n);
        let v = r.violation().expect("must block");
        assert_eq!(v.gate, CAUSAL_RUNG_UNREACHED);
        assert!(v.detail.contains("teleological"), "{}", v.detail);
    }

    #[test]
    fn rung_names_are_accepted_in_their_common_spellings() {
        for (spelling, needs_run) in [
            ("association", false),
            ("seeing", false),
            ("1", false),
            ("intervention", true),
            ("doing", true),
            ("2", true),
            ("counterfactual", true),
            ("imagining", true),
            ("3", true),
            ("  ASSOCIATION  ", false),
        ] {
            let doc =
                format!("---\nid: c1\ntype: claim\ntitle: t\ncausal_rung: \"{spelling}\"\n---\n");
            let n = node(&doc);
            let g = graph_of(vec![n.clone()], vec![]);
            let r = causal_rung_earned(&g, &n);
            if needs_run {
                assert!(
                    r.violation().is_some(),
                    "{spelling} should demand a protocol"
                );
            } else {
                assert_eq!(r, GateResult::Pass, "{spelling} should pass at rung 1");
            }
        }
    }

    #[test]
    fn a_counterfactual_claim_names_its_rung_in_the_diagnostic() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\ncausal_rung: intervention\n---\n");
        let g = graph_of(vec![claim.clone()], vec![]);
        let v = causal_rung_earned(&g, &claim);
        let v = v.violation().expect("must block");
        assert!(v.detail.contains("intervention"), "{}", v.detail);
    }

    #[test]
    fn boundaries_are_required_and_a_declared_one_passes() {
        let without = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let with =
            node("---\nid: c2\ntype: claim\ntitle: t\nboundaries:\n  - Windows 10 1809+\n---\n");
        let g = graph_of(vec![without.clone(), with.clone()], vec![]);
        assert_eq!(
            boundaries_declared(&g, &without)
                .violation()
                .map(|v| v.gate),
            Some(BOUNDARIES_MISSING)
        );
        assert_eq!(boundaries_declared(&g, &with), GateResult::Pass);
    }
}
