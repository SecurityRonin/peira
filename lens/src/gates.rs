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
    // A node kind is a SELF-DECLARED STRING, so it cannot be the exemption. The `_ =>
    // false` arm here was a static scope hiding inside the load-bearing test that
    // replaced one: relabelling a universal over-claim `type: observation` stripped all
    // seven promotion obligations while it went on supporting a claim.
    //
    // An observation that RECORDS declares none of these fields and is examined by
    // nothing — the gates return NotApplicable of their own accord. One that ASSERTS,
    // in the shape of a claim, and is leaned on, answers like one.
    let leaned_on = || {
        graph
            .edges_from(&node.id)
            .any(|e| matches!(e.kind, EdgeKind::Supports | EdgeKind::DependsOn))
            || graph
                .edges_to(&node.id)
                .any(|e| e.kind == EdgeKind::DependsOn)
    };
    // Leaned on AND asserting. Leaned-on alone would demand a quantifier and a causal
    // rung from every observation that supports anything, which is ceremony, and
    // ceremony is routed around. A node that declares one of these fields has taken a
    // position in the shape of a claim; one that records has not.
    let asserts = || {
        ["quantifier", "causal_rung", "aspect", "warrant"]
            .iter()
            .any(|f| node.field(f).is_some())
    };
    match node.kind {
        // A claim asserts by existing.
        NodeKind::Claim => true,
        // A hypothesis is an ARGUMENT — it competes in the extension — so being leaned
        // on is enough. This is what stops an unexamined hypothesis laundering a
        // conclusion into a packet.
        NodeKind::Hypothesis => leaned_on(),
        // Everything else is evidence or reference. Being leaned on is not enough:
        // demanding a quantifier and a causal rung from every supporting observation is
        // ceremony, and ceremony is routed around. It must also have taken a position.
        _ => leaned_on() && asserts(),
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

/// A claim about what a thing *is* may not rest solely on what it *did*.
pub fn substance_not_from_function_alone(graph: &Graph, node: &Node) -> GateResult {
    if node.field("aspect") != Some("substance") {
        return GateResult::NotApplicable;
    }
    let support = supporters(graph, node);
    if support.is_empty() {
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
        .filter(|s| s.field("aspect").is_none())
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
        .all(|s| s.field("aspect") == Some("function"));

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
    if !matches!(quantifier, "universal" | "class") {
        return GateResult::NotApplicable;
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
    let attacked = graph.edges_to(&node.id).any(|e| e.kind.is_attack());
    let contested = attacked || node.field("contested") == Some("true");
    if !contested {
        return GateResult::NotApplicable;
    }
    let corners = node.field_list("corners");
    if corners.len() == 4 {
        GateResult::Pass
    } else {
        block(
            CORNERS_UNADDRESSED,
            "CATUSKOTI",
            node,
            format!(
                "contested claim \"{}\" addresses {} of 4 corners",
                node.title,
                corners.len()
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
    let incoming: Vec<_> = graph
        .edges_to(&node.id)
        .filter(|e| e.kind == EdgeKind::Supports)
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
    if graph.edges_to(&node.id).any(|e| e.kind.is_attack()) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::{parse_node, Edge, Grade, NodeId, Pramana};

    fn node(src: &str) -> Node {
        parse_node(src).expect("fixture parses")
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
