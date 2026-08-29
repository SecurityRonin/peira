//! peira as an MCP server — the gates, where the writing happens.
//!
//! # Why this exists
//!
//! peira's checks only bite if something runs them, and until now the only thing
//! that did was a CLI pointed at a vault. That is the wrong place: the overstatement
//! is written before the vault exists, in a draft, by whoever is tired and in a
//! hurry. These tools need no vault at all.
//!
//! # What it deliberately does not do
//!
//! No scores. peira mints no SCORE field — no confidence/severity/probability/weight —
//! and a response schema is exactly where a helpful-looking `confidence: 0.82` would
//! appear. A finding carries a gate code, a subject, what was found and what to do
//! instead — nothing that could be averaged. (It does mint non-score numbers where they
//! are structural, not evidential: a packet's format version, a line count in a verify
//! diff. Those are facts about the artifact, not gradings of a claim.)
//!
//! And an empty finding list is NOT an endorsement. Exactly two checks run without a
//! node — overstated verbs and ultimate-issue conclusions — because every other rule
//! peira enforces compares prose against DECLARED fields that bare text does not have.
//! So "nothing fired" means those two found nothing, never "this sentence is safe".
//! Every response says which two, because a caller that reads silence as approval has
//! been given a worse instrument than none.

use std::path::Path;

use peira_citation::{
    all_findings, refusal_for, violations_for, Packet, PacketError, Verification,
};
use peira_core::{Graph, NodeId};
use peira_lens::Violation;
use rmcp::schemars;
use serde::Serialize;

/// One thing found in the text, with the tradition that named it.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct Finding {
    /// The published gate code, e.g. `PEIR-LINT-FORBIDDEN-VERB`.
    pub code: &'static str,
    /// What was found, quoting the offending words.
    pub detail: String,
    /// The safe form, named rather than merely demanded.
    pub remedy: &'static str,
}

/// The result of scanning prose, with its own limits attached.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ProseReport {
    /// What the scan named.
    pub findings: Vec<Finding>,
    /// Stated on EVERY response, including the clean one. See the module note.
    pub scope: &'static str,
}

/// The scope note, carried on every report so silence cannot read as approval.
const SCOPE: &str = "TWO checks run here, and only two: overstated verbs (an observation \
stated as a verdict) and ultimate-issue conclusions (a verdict word said of a party). \
peira's other rules — quantifier scope, causal rung, warrant, falsifier, boundaries, \
diagnosticity — compare prose against a node's DECLARED fields, so they cannot run on bare \
text and are not run. An empty result means these two found nothing; it is not a finding \
that the text is sound, and it says nothing about whether the claim is supported.";

/// Run every prose check over arbitrary text. No vault, no graph, no model.
#[must_use]
pub fn check_prose(text: &str) -> ProseReport {
    // A synthetic subject: these checks report against a node id, and here there is
    // no node. Named so it cannot be mistaken for a real one in output.
    let subject = NodeId::new("(prose)");
    let findings = peira_lens::lints::prose_findings_in(text, &subject)
        .into_iter()
        .map(|v| Finding {
            code: v.gate,
            detail: v.detail,
            remedy: v.remedy,
        })
        .collect();
    ProseReport {
        findings,
        scope: SCOPE,
    }
}

/// One catalogue entry, as a caller needs it.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct LensEntry {
    /// Stable id, e.g. `TRAIRUPYA`.
    pub id: &'static str,
    /// Display name, with the original term.
    pub name: &'static str,
    /// Where it comes from.
    pub tradition: &'static str,
    /// The specific way of being wrong it names — the reason it earns a place.
    pub failure_mode: &'static str,
    /// What it does to a claim graph.
    pub operation: &'static str,
    /// `Enforced` or `Catalogued`. An enforced lens refuses; a catalogued one is a
    /// reading, and the distinction must reach the caller or it reads as one thing.
    pub phase: &'static str,
    /// The gate codes it owns. Empty for a catalogued lens — and empty for one
    /// enforced lens, which is why [`LensEntry::enforced_by`] exists.
    pub gates: Vec<&'static str>,
    /// WHERE THE REFUSAL LIVES. For most enforced lenses that is their own gates.
    /// DUNG is enforced by the grounded-extension computation and owns no gate code,
    /// so `{phase: "Enforced", gates: []}` would otherwise reach a caller with no way
    /// to tell an engine-level rule from a mislabelled entry.
    pub enforced_by: &'static str,
    /// A worked example of the failure.
    pub worked_example: &'static str,
    /// Authoritative references.
    pub sources: Vec<&'static str>,
}

fn entry(l: &'static peira_lens::Lens) -> LensEntry {
    LensEntry {
        id: l.id,
        name: l.name,
        tradition: l.tradition.as_str(),
        failure_mode: l.failure_mode,
        operation: l.operation,
        phase: match l.phase {
            peira_lens::Phase::Enforced => "Enforced",
            peira_lens::Phase::Catalogued => "Catalogued",
        },
        gates: l.gates.iter().map(|g| g.code).collect(),
        enforced_by: match (l.phase, l.gates.is_empty()) {
            (peira_lens::Phase::Enforced, false) => "its own gates",
            (peira_lens::Phase::Enforced, true) => {
                "the engine itself — this rule is \
computed, not gated, so it owns no gate code"
            }
            (peira_lens::Phase::Catalogued, _) => {
                "nothing — a human screen, named but \
not mechanised"
            }
        },
        worked_example: l.worked_example,
        sources: l.sources.to_vec(),
    }
}

/// The catalogue as it crosses MCP: the entries under a named field.
///
/// MCP's `structuredContent` must be a JSON OBJECT, so the entries cannot cross as a
/// bare array — a top-level array is rejected in transit and the caller sees nothing.
/// This wrapper is why `check_prose` was already correct: [`ProseReport`] is a struct,
/// so it serialised to an object, and the catalogue must do the same.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct LensCatalogue {
    /// The lenses, in catalogue order. One entry when an `id` selected it; none when the
    /// id matched nothing — an empty list, never a placeholder.
    pub lenses: Vec<LensEntry>,
}

/// The catalogue, or one entry of it.
///
/// Exposed so a caller can REASON WITH the framework rather than only be checked by
/// it: each entry names a failure mode and cites where it was identified.
#[must_use]
pub fn catalogue(id: Option<&str>) -> LensCatalogue {
    let lenses = match id {
        Some(want) => peira_lens::lens(want).map(entry).into_iter().collect(),
        None => peira_lens::CATALOG.iter().map(entry).collect(),
    };
    LensCatalogue { lenses }
}

// ── Tier 2 — vault-aware (read-only) ──────────────────────────────────────────
//
// These take a vault path and still never write it. peira's value is that it refuses;
// a server that can write is a server that can be talked into writing. See ADR-0006.

/// One vault finding, carrying the SUBJECT that a prose [`Finding`] does not need.
///
/// A whole-vault survey answers for many nodes at once, so the subject is not optional
/// here. `PEIR-GATE-UNASSESSED` arrives as an ordinary finding with its own code — that
/// is how "no verdict reached" survives the JSON boundary instead of collapsing into a
/// pass or a bare `{ok: false}`.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct NodeFinding {
    /// The stable gate code, e.g. `PEIR-WARRANT-MISSING` or `PEIR-GATE-UNASSESSED`.
    pub code: &'static str,
    /// The lens the gate belongs to.
    pub lens: &'static str,
    /// The node the finding is against.
    pub subject: String,
    /// What was found, verbatim.
    pub detail: String,
    /// What would resolve it.
    pub remedy: &'static str,
}

fn node_finding(v: &Violation) -> NodeFinding {
    NodeFinding {
        code: v.gate,
        lens: v.lens,
        subject: v.subject.to_string(),
        detail: v.detail.clone(),
        remedy: v.remedy,
    }
}

/// A node's derived standing — the same question `peira status` answers. Derived from
/// the graph, never a field set on a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Standing {
    /// Enforced gates pass and the claim stands: it is in the grounded extension, or it
    /// is reference material that does not compete. A reviewer must still sign — peira
    /// does not.
    ReviewReady,
    /// Gates pass, but the claim is DEFEATED in the grounded extension: an attack on it
    /// stands unanswered. It loses on the argument, not on the evidence.
    Contested,
    /// One or more enforced gates BLOCK. The evidence is not yet enough to stand.
    EvidencePending,
}

/// The standing of one node, by reusing the SAME refusal `freeze` and `peira status`
/// obey — so this can never disagree with the tool's own refusal.
///
/// [`refusal_for`] owns the precedence: blocking gates before grounding. The one thing it
/// does not distinguish is reference material, which cannot lose an argument it never
/// entered; that single guard is added here, matching `peira status`.
fn standing_of(graph: &Graph, id: &NodeId) -> Standing {
    match refusal_for(graph, id) {
        Some(PacketError::Blocked { .. }) => Standing::EvidencePending,
        Some(PacketError::Defeated(_)) if graph.is_argument_node(id) => Standing::Contested,
        // Freezable (`None`), reference material that loses no argument, and the
        // `NoSuchClaim` that `refusal_for` never returns (existence is guarded upstream).
        None | Some(_) => Standing::ReviewReady,
    }
}

const EXAMINE_SCOPE: &str = "gates + lints for this claim and everything it rests on, plus \
its derived standing. An empty findings list means clean; a PEIR-GATE-UNASSESSED finding \
means a gate could NOT reach a verdict, which is never a pass. No claim is authored or \
graded here — peira renders from the graph, it does not write it.";

const STATUS_SCOPE: &str = "the derived standing of this node — the same question `peira \
status` answers, computed from the graph and never set. review_ready means gates pass and \
the claim stands, but a human reviewer must still sign; peira does not.";

const GATES_SCOPE: &str = "every gate and lint finding across the whole vault. An empty \
list over a NON-EMPTY vault means nothing was found; an absent or empty vault is an error, \
not a clean result — silence from an empty vault would be a lie.";

fn no_such_node(id: &NodeId) -> String {
    format!("no node `{id}` in the vault")
}

/// One claim examined: its standing, and everything blocking it (the evidential closure).
///
/// # Errors
/// The node must exist in the vault.
pub fn examine(graph: &Graph, id: &NodeId) -> Result<ExamineReport, String> {
    if graph.node(id).is_none() {
        return Err(no_such_node(id));
    }
    Ok(ExamineReport {
        node: id.to_string(),
        standing: standing_of(graph, id),
        findings: violations_for(graph, id).iter().map(node_finding).collect(),
        scope: EXAMINE_SCOPE,
    })
}

/// A node's standing alone — the lighter "is this claim actually standing?" question.
///
/// # Errors
/// The node must exist in the vault.
pub fn status(graph: &Graph, id: &NodeId) -> Result<StatusReport, String> {
    if graph.node(id).is_none() {
        return Err(no_such_node(id));
    }
    Ok(StatusReport {
        node: id.to_string(),
        standing: standing_of(graph, id),
        scope: STATUS_SCOPE,
    })
}

/// Every gate and lint finding across the whole vault.
#[must_use]
pub fn gates(graph: &Graph) -> GatesReport {
    // The COMPLETE survey — the enforced GATE pack AND the lint pack. `all_findings` is
    // lints + sealed-prose ONLY; using it alone made this tool (the sole whole-vault
    // survey, advertised as "every gate and lint finding") return an empty "clean" over a
    // vault whose every claim was unfreezable, and silently drop PEIR-GATE-UNASSESSED —
    // the no-verdict state the per-node path works to preserve. `examine_graph` is the
    // gate pack; union it in, deduped by (gate, subject, detail) since a finding can in
    // principle arise in both.
    let mut violations = peira_lens::examine_graph(graph);
    for v in all_findings(graph) {
        if !violations
            .iter()
            .any(|f| f.gate == v.gate && f.subject == v.subject && f.detail == v.detail)
        {
            violations.push(v);
        }
    }
    GatesReport {
        findings: violations.iter().map(node_finding).collect(),
        scope: GATES_SCOPE,
    }
}

/// Load a vault for read-only examination.
///
/// # Errors
/// The path must load and hold at least one node. AN EMPTY VAULT IS NOT A CLEAN ONE: a
/// directory with no nodes would make every survey below report nothing, indistinguishable
/// from a vault whose every claim passed. Those must stay distinguishable, so an empty or
/// unreadable vault is an error, never an empty result.
pub fn load_vault(path: &Path) -> Result<Graph, String> {
    let graph = peira_core::load(path).map_err(|e| e.to_string())?;
    if graph.nodes().next().is_none() {
        return Err(format!(
            "vault `{}` holds no nodes — nothing was examined, which is not the same as \
nothing being wrong",
            path.display()
        ));
    }
    Ok(graph)
}

/// One claim's standing and everything blocking it.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ExamineReport {
    /// The node examined.
    pub node: String,
    /// Its derived standing.
    pub standing: Standing,
    /// Every gate and lint blocking it, across its evidential closure.
    pub findings: Vec<NodeFinding>,
    /// Carried on every report; see the module note.
    pub scope: &'static str,
}

/// A node's standing, without the findings list.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StatusReport {
    /// The node.
    pub node: String,
    /// Its derived standing.
    pub standing: Standing,
    /// Carried on every report; see the module note.
    pub scope: &'static str,
}

/// Whole-vault gate and lint findings.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GatesReport {
    /// Every finding across the vault, each naming its subject.
    pub findings: Vec<NodeFinding>,
    /// Carried on every report; see the module note.
    pub scope: &'static str,
}

// ── Tier 4 — freeze / verify (read-only; a refusal is a RESULT) ────────────────
//
// A refusal is not an error here: the reasons are the product, and an LLM handed a bare
// error reads it as "try again". A packet is returned, never written. See ADR-0006.

/// The outcome of a freeze attempt over a claim that exists.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum FreezeReport {
    /// The packet froze. `body` is the court artifact; `digest` is SHA-256 over it. The
    /// packet is returned, not written — saving it is the caller's decision.
    Frozen {
        /// The claim frozen.
        subject: String,
        /// The rendered packet document; line 1 names the subject.
        body: String,
        /// SHA-256 over `body`.
        digest: String,
    },
    /// Gates BLOCK: the claim is not yet freezable, and these findings stand in the way.
    /// The reasons are the product; a bare "failed" would throw them away.
    Blocked {
        /// The claim.
        node: String,
        /// Everything blocking it, each naming its subject.
        violations: Vec<NodeFinding>,
    },
    /// The claim is DEFEATED in the grounded extension — an attack on it stands
    /// unanswered. Nothing is wrong with a packet; the claim loses on the argument.
    Defeated {
        /// The claim.
        node: String,
    },
}

/// Freeze a citation packet for one claim, or report why it will not freeze.
///
/// # Errors
/// The subject must be a claim that exists. A missing node or a non-claim is a bad
/// request — distinct from a claim that exists but declines to freeze, which is a
/// [`FreezeReport`] result carrying its reasons.
pub fn freeze(graph: &Graph, id: &NodeId) -> Result<FreezeReport, String> {
    match peira_citation::freeze(graph, id) {
        Ok(p) => Ok(FreezeReport::Frozen {
            subject: p.subject.to_string(),
            body: p.body,
            digest: p.digest,
        }),
        Err(PacketError::Blocked { violations, .. }) => Ok(FreezeReport::Blocked {
            node: id.to_string(),
            violations: violations.iter().map(node_finding).collect(),
        }),
        Err(PacketError::Defeated(_)) => Ok(FreezeReport::Defeated {
            node: id.to_string(),
        }),
        Err(PacketError::NoSuchClaim(_)) => Err(no_such_node(id)),
        Err(PacketError::NotAClaim { kind, .. }) => Err(format!(
            "`{id}` is not a claim (kind: {kind}); only claims freeze"
        )),
        // PacketError is #[non_exhaustive]: a refusal shape this build does not know is
        // surfaced loudly, never mapped to a plausible-but-wrong outcome.
        Err(other) => Err(format!("cannot freeze `{id}`: {other}")),
    }
}

/// The outcome of verifying a stored packet against the vault as it stands now.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum VerifyReport {
    /// Re-derived byte-identically from the vault. The packet still stands.
    Verified {
        /// The claim.
        subject: String,
        /// SHA-256 the stored packet was checked at.
        digest: String,
    },
    /// The vault no longer renders this packet. NOT by itself an accusation: a vault that
    /// GREW (a corroborating observation added later) and one whose evidence was ALTERED
    /// produce the same verdict — the reader judges which.
    DigestMismatch {
        /// The claim.
        subject: String,
        /// True iff the SOLE difference is the format number, which proves an edit — no
        /// older renderer could emit a body byte-identical to a newer one's.
        format_line_only: bool,
        /// The first line at which stored and fresh diverge — evidence to act on.
        first_difference: Option<String>,
    },
    /// Written by a different renderer than this build, so no comparison is meaningful.
    FormatSuperseded {
        /// The claim.
        subject: String,
        /// The format the packet declares.
        stored_format: u32,
        /// The format this build renders.
        current_format: u32,
    },
    /// The claim no longer freezes — a gate now blocks it, or it has been defeated.
    /// Nothing is wrong with the packet; the claim stopped qualifying.
    NoLongerFreezable {
        /// The claim.
        subject: String,
        /// Why it no longer freezes.
        reason: String,
    },
}

/// Verify a stored packet document against the vault as it stands.
///
/// # Errors
/// The text must be a citation packet — its first line names the subject.
pub fn verify(graph: &Graph, packet: String) -> Result<VerifyReport, String> {
    let doc = Packet::from_document(packet)?;
    let subject = doc.subject.to_string();
    let report = match peira_citation::verify(graph, &doc) {
        Verification::Verified => VerifyReport::Verified {
            subject,
            digest: doc.digest,
        },
        Verification::DigestMismatch {
            format_line_only,
            first_difference,
            ..
        } => VerifyReport::DigestMismatch {
            subject,
            format_line_only,
            first_difference,
        },
        Verification::FormatSuperseded {
            stored, current, ..
        } => VerifyReport::FormatSuperseded {
            subject,
            stored_format: stored,
            current_format: current,
        },
        Verification::NoLongerFreezable(err) => VerifyReport::NoLongerFreezable {
            subject,
            reason: err.to_string(),
        },
        // Verification is #[non_exhaustive]: an outcome this build does not know must
        // NOT be mapped to a plausible one — a false "verified" is the worst case — so it
        // fails loud instead.
        _ => return Err(format!("unrecognised verification outcome for `{subject}`")),
    };
    Ok(report)
}

// ── Tier 3 — propose (no vault; STRUCTURE and CLASSIFICATION only) ─────────────
//
// `propose` reads prose the author already wrote and emits the skeleton of a claim node
// — because a claim's required fields are the enforced gates in reverse. It infers the
// CLASSIFICATION fields (quantifier, aspect, causal_rung) from the author's own words,
// marked `confirm`, and reuses the lint's verb knowledge so it cannot disagree with the
// gates. It authors NO evidence: there is no grade/by/via field anywhere. See ADR-0006.

/// One classification field read from the proposition's own words. Confirmable — the
/// author owns it; peira only names what the sentence already says.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Inferred {
    /// The field: `quantifier`, `aspect`, or `causal_rung`.
    pub field: &'static str,
    /// The inferred value, in the vocabulary the gate reads.
    pub value: &'static str,
    /// The words that triggered it — so the author can check the reading.
    pub inferred_from: String,
    /// Always true: an inference is a suggestion, never a decision.
    pub confirm: bool,
}

/// A field the author must supply, named with the gate that will demand it. Blank by
/// construction — there is no value here to fill in wrongly.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct Need {
    /// The claim field left blank.
    pub field: &'static str,
    /// The gate that will block until it is supplied.
    pub gate: &'static str,
}

/// A draft claim proposed from prose: STRUCTURE and CLASSIFICATION only, no evidence.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ProposeReport {
    /// The node type proposed. `claim` — the author may retype it.
    pub proposed_type: &'static str,
    /// The author's sentence, verbatim. peira never rewrites it.
    pub proposition: String,
    /// Classification fields inferred from the proposition, each marked `confirm`.
    pub inferred: Vec<Inferred>,
    /// Terms the author explicitly quoted — candidate Term nodes, still undefined.
    pub candidate_terms: Vec<String>,
    /// Every field left blank, with the gate that will demand it. Fill these to pass.
    pub needs: Vec<Need>,
    /// Overstatement in the proposition itself, caught by the same lint `check_prose`
    /// runs — so an over-claim is named at the point of authoring.
    pub prose_findings: Vec<Finding>,
    /// Carried on every report; see the module note.
    pub scope: &'static str,
}

const PROPOSE_SCOPE: &str = "STRUCTURE and CLASSIFICATION only. The proposition is the \
author's own words, unaltered; the inferred fields classify those words and are marked to \
confirm; the `needs` are blank for the author to fill. peira authors NO evidence here — \
there is no grade, by, or via, because those assert something was examined, and nothing \
was.";

/// Whether `haystack` contains `word` as a whole word (both already lowercased).
fn contains_word(haystack: &str, word: &str) -> bool {
    // Split on separators, but keep hyphen and apostrophe WORD-INTERNAL — else
    // "all-clear", "no-one" and "y'all" leak the determiners "all"/"no" and misfire the
    // quantifier. Both straight and curly apostrophes.
    haystack
        .split(|c: char| !c.is_alphanumeric() && !matches!(c, '-' | '\'' | '\u{2019}'))
        .any(|token| token == word)
}

fn infer_quantifier(proposition: &str) -> Inferred {
    // A universal determiner makes the claim range over a class — the BAIMA question.
    const UNIVERSAL: [&str; 5] = ["every", "all", "each", "any", "no"];
    let lower = proposition.to_lowercase();
    if let Some(w) = UNIVERSAL.iter().find(|w| contains_word(&lower, w)) {
        Inferred {
            field: "quantifier",
            value: "universal",
            inferred_from: format!("the determiner \"{w}\" — the claim ranges over a class"),
            confirm: true,
        }
    } else {
        Inferred {
            field: "quantifier",
            value: "singular",
            inferred_from: "no universal determiner present".to_owned(),
            confirm: true,
        }
    }
}

fn infer_aspect(proposition: &str) -> Inferred {
    // A copular "X is a …" reads as a claim about what something IS (substance); an action
    // predicate reads as what it DID (function). Crude on purpose — peira has no parser
    // for meaning, so this is confirmable, not decided.
    let lower = proposition.to_lowercase();
    // Substance = a predicate NOMINAL: "X is/are a/an <noun>". Bare " are " was the bug —
    // it fired on passive voice ("the logs are wiped"), the commonest forensic register,
    // which is a FUNCTION claim about what was done. Requiring the article keeps passive
    // voice out of substance; the residual copular cases ("it is a fact that …") are why
    // this stays confirmable.
    let substance = lower.contains(" is a ")
        || lower.contains(" is an ")
        || lower.contains(" are a ")
        || lower.contains(" are an ");
    if substance {
        Inferred {
            field: "aspect",
            value: "substance",
            inferred_from: "a predicate nominal \"is/are a/an …\" — a claim about what \
something IS (confirm: passive voice can resemble this)"
                .to_owned(),
            confirm: true,
        }
    } else {
        Inferred {
            field: "aspect",
            value: "function",
            inferred_from: "no predicate nominal — read as a claim about what something DID"
                .to_owned(),
            confirm: true,
        }
    }
}

/// A causal marker in the proposition — a claim of causation, which reads above the
/// association rung whether or not it uses a proving verb. Negation is not special-cased:
/// "X did not cause Y" asserts non-causation, which needs the same rung to establish.
fn causal_marker(proposition: &str) -> Option<&'static str> {
    const WORDS: [&str; 6] = [
        "caused",
        "causes",
        "causing",
        "prevented",
        "prevents",
        "preventing",
    ];
    const PHRASES: [&str; 6] = [
        "because",
        "due to",
        "leads to",
        "led to",
        "results in",
        "resulted in",
    ];
    let lower = proposition.to_lowercase();
    if let Some(w) = WORDS.iter().find(|w| contains_word(&lower, w)) {
        return Some(w);
    }
    PHRASES.iter().copied().find(|p| lower.contains(p))
}

fn infer_rung(proposition: &str, findings: &[Violation]) -> Inferred {
    // A proving verb (the forbidden-verb lint, reused rather than a parallel table) is
    // epistemic overreach → counterfactual. A CAUSAL marker with no proving verb is a
    // causation claim → intervention. Either reads above the association rung, so the
    // RUNG gate FIRES on the draft — surfacing the over-claim rather than hiding it, which
    // is the whole point. The earlier version knew only proving verbs, so genuine causal
    // claims ("the malware caused the outage") were floored at association and the gate
    // stayed silent — a false LOW hiding exactly what this exists to catch.
    if findings
        .iter()
        .any(|v| v.gate == peira_lens::lints::FORBIDDEN_VERB)
    {
        Inferred {
            field: "causal_rung",
            value: "counterfactual",
            inferred_from: "a proving verb the forbidden-verb lint flags — the sentence \
reads above the association rung, so the RUNG gate will demand a protocol"
                .to_owned(),
            confirm: true,
        }
    } else if let Some(marker) = causal_marker(proposition) {
        Inferred {
            field: "causal_rung",
            value: "intervention",
            inferred_from: format!(
                "a causal marker \"{marker}\" — the claim reads at the intervention rung, \
so the RUNG gate will demand a protocol"
            ),
            confirm: true,
        }
    } else {
        Inferred {
            field: "causal_rung",
            value: "association",
            inferred_from: "no proving or causal language present — the safe floor".to_owned(),
            confirm: true,
        }
    }
}

/// Terms the author explicitly put in double quotes — a strong, unambiguous signal, unlike
/// guessing noun phrases, which peira has no model to do. Curly and straight quotes both.
fn quoted_terms(proposition: &str) -> Vec<String> {
    fn flush(current: &mut String, terms: &mut Vec<String>) {
        let t = current.trim();
        if !t.is_empty() && !terms.iter().any(|e| e == t) {
            terms.push(t.to_owned());
        }
        current.clear();
    }
    let mut terms: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut inside = false;
    for ch in proposition.chars() {
        match ch {
            // Straight quote is non-directional: toggle open/close.
            '"' => {
                if inside {
                    flush(&mut current, &mut terms);
                }
                inside = !inside;
            }
            // Curly quotes are DIRECTIONAL. Treating all three glyphs as interchangeable
            // toggles let a stray closing quote — common in copy-pasted prose — open a
            // span, turning the following words into a fabricated term.
            '\u{201C}' => inside = true,
            '\u{201D}' if inside => {
                flush(&mut current, &mut terms);
                inside = false;
            }
            '\u{201D}' => {} // stray close outside a span: ignore, do not open one
            _ if inside => current.push(ch),
            _ => {}
        }
    }
    terms
}

/// Propose a claim skeleton from prose the author already wrote.
///
/// No vault, no graph, no model. Infers the classification fields from the words, leaves
/// every evidentiary field blank, and names each blank with the gate that will demand it.
#[must_use]
pub fn propose(prose: &str) -> ProposeReport {
    let proposition = prose.trim().to_owned();
    // The same lint `check_prose` runs, over the proposition — used both to surface the
    // over-claim to the author and to infer the causal rung from it.
    let subject = NodeId::new("(proposed)");
    let findings = peira_lens::lints::prose_findings_in(&proposition, &subject);

    let inferred = vec![
        infer_quantifier(&proposition),
        infer_aspect(&proposition),
        infer_rung(&proposition, &findings),
    ];

    // The blanks, each labelled with the gate that will demand it. These are the enforced
    // gates in reverse: filling them is what turns the draft into a claim that passes.
    let needs = vec![
        Need {
            field: "warrant",
            gate: peira_lens::gates::WARRANT_MISSING,
        },
        Need {
            field: "boundaries",
            gate: peira_lens::gates::BOUNDARIES_MISSING,
        },
        Need {
            field: "falsifier",
            gate: peira_lens::gates::FALSIFIER_MISSING,
        },
        Need {
            field: "uses_term",
            gate: peira_lens::gates::TERM_UNSTIPULATED,
        },
    ];

    ProposeReport {
        proposed_type: "claim",
        candidate_terms: quoted_terms(&proposition),
        prose_findings: findings
            .iter()
            .map(|v| Finding {
                code: v.gate,
                detail: v.detail.clone(),
                remedy: v.remedy,
            })
            .collect(),
        proposition,
        inferred,
        needs,
        scope: PROPOSE_SCOPE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sentence an expert must not write, caught with no vault in sight.
    ///
    /// This is the whole argument for the crate: the overstatement happens in a
    /// draft, long before anything is a node in a graph.
    #[test]
    fn an_overstatement_is_named_without_a_vault() {
        let r = check_prose(
            "The Amcache entry proves the respondent executed the binary, and the \
evidence demonstrates the respondent is guilty of unauthorised access.",
        );
        let codes: Vec<&str> = r.findings.iter().map(|f| f.code).collect();
        assert!(
            codes.contains(&"PEIR-LINT-FORBIDDEN-VERB"),
            "`proves` is an observation stated as a verdict: {codes:?}"
        );
        assert!(
            codes.contains(&"PEIR-LINT-LEGAL-CONCLUSION"),
            "`is guilty of` decides the tribunal's question: {codes:?}"
        );
        assert!(
            r.findings.iter().all(|f| !f.remedy.is_empty()),
            "every finding must name the safe form, not merely refuse"
        );
    }

    /// Silence is not endorsement, and the response has to say so itself.
    ///
    /// A caller that reads an empty list as "this is fine" has been handed a worse
    /// instrument than none. The scope note is therefore carried on the CLEAN
    /// response too — which is the one where it matters.
    #[test]
    fn a_clean_scan_still_states_what_it_did_not_check() {
        let r = check_prose("The register recorded the path at 09:14 UTC.");
        assert!(
            r.findings.is_empty(),
            "control: a bounded sentence must not fire: {:?}",
            r.findings
        );
        assert!(
            r.scope.contains("not a finding that the text is sound"),
            "an empty result must disclaim itself"
        );
    }

    /// THE LIMIT, pinned. Found by using the tool, not by reading the tests.
    ///
    /// The first tool description advertised four checks: overstated verbs,
    /// ultimate-issue conclusions, unbounded quantifiers, and hedges. Only the first
    /// two run without a node — the rest compare prose against DECLARED fields that
    /// bare text does not have. So peira's own MCP surface overstated its coverage,
    /// which is the failure peira exists to name.
    ///
    /// This asserts the boundary rather than the promise: a sentence that quantifies
    /// over an estate draws NOTHING here, and the scope note must say why.
    #[test]
    fn an_unbounded_quantifier_is_not_caught_without_a_node() {
        let r = check_prose("Every device in the estate shows the same registry artefact.");
        assert!(
            r.findings.is_empty(),
            "if this now fires, the prose scan has grown a check and the scope note \
and tool description are both stale: {:?}",
            r.findings
        );
        assert!(
            r.scope.contains("declared") || r.scope.contains("DECLARED"),
            "the scope note must say WHY the other rules cannot run here, or a caller \
reads two checks as all of peira"
        );
        assert!(
            r.scope.contains("TWO") || r.scope.contains("two"),
            "the scope note must state how many checks actually ran"
        );
    }

    /// No number reaches the caller that peira did not derive from the graph.
    #[test]
    fn nothing_in_the_response_can_be_averaged() {
        let r = check_prose("This proves execution.");
        let json = serde_json::to_string(&r).expect("serialises");
        for minted in ["confidence", "score", "severity", "probability", "weight"] {
            assert!(
                !json.contains(minted),
                "the response carries `{minted}` — peira mints no numbers"
            );
        }
    }

    /// The catalogue crosses MCP as `structuredContent`, which the spec requires to be
    /// a JSON OBJECT. A bare `Vec` serialises to a top-level array, and the client
    /// rejects the whole response in transit — the caller never sees a single lens.
    /// This is the exact shape [`ProseReport`] already had and the catalogue did not.
    #[test]
    fn the_catalogue_payload_is_a_json_object_not_a_bare_array() {
        let v = serde_json::to_value(catalogue(None)).expect("serialises");
        assert!(
            v.is_object(),
            "MCP structuredContent must be an object; a bare array is rejected before \
the caller sees it: {v}"
        );
        let lenses = v
            .get("lenses")
            .expect("the entries cross under a named field, not as the whole value");
        assert!(
            lenses.is_array(),
            "the entries live under `lenses` as an array"
        );
    }

    /// The catalogue reaches the caller whole, and says which entries REFUSE.
    #[test]
    fn the_catalogue_distinguishes_what_enforces_from_what_is_merely_read() {
        let all = catalogue(None).lenses;
        assert_eq!(all.len(), peira_lens::CATALOG.len(), "every entry, or none");

        let enforced: Vec<&LensEntry> = all.iter().filter(|e| e.phase == "Enforced").collect();
        // An enforced lens either owns gates or SAYS where its refusal lives.
        // DUNG owns none: grounded-extension standing is computed in the engine. A
        // caller must be able to tell that from a mislabelled entry, and silence
        // cannot.
        assert!(
            enforced
                .iter()
                .all(|e| !e.gates.is_empty() || e.enforced_by.contains("engine")),
            "an entry marked Enforced with no gate must name where it IS enforced, or \
it claims an examination nothing performs"
        );
        assert!(
            all.iter().all(|e| !e.enforced_by.is_empty()),
            "every entry states where its refusal lives, including `nothing`"
        );
        assert!(
            all.iter()
                .filter(|e| e.phase == "Catalogued")
                .all(|e| e.gates.is_empty()),
            "a catalogued entry owning a gate is enforced, and mislabelled"
        );
        assert!(
            all.iter()
                .all(|e| !e.failure_mode.is_empty() && !e.sources.is_empty()),
            "every lens names a failure and cites where it was identified"
        );
    }

    /// One entry by id, and an unknown id yields nothing rather than something.
    #[test]
    fn an_unknown_lens_id_returns_nothing_not_a_placeholder() {
        let one = catalogue(Some("TRAIRUPYA")).lenses;
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].id, "TRAIRUPYA");
        assert!(one[0].gates.contains(&"PEIR-HETU-UNDIAGNOSTIC"));

        assert!(
            catalogue(Some("NOT-A-LENS")).lenses.is_empty(),
            "an unknown id must return nothing — a placeholder entry would be a lens \
that does not exist, cited as though it does"
        );
    }

    // ── Tier 2 — vault-aware ──────────────────────────────────────────────────

    /// Load a real fixture vault from the repo's `tests/vaults`. These are the same
    /// corpora `docs/validation.md` exercises: real engine output over ground truth
    /// derived from documented construction (Tier 2).
    fn vault(name: &str) -> Graph {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/vaults")
            .join(name);
        load_vault(&root).expect("fixture vault loads")
    }

    /// A bounded claim clears the gates and stands.
    #[test]
    fn examine_a_bounded_claim_is_review_ready_and_clean() {
        let g = vault("bounded");
        let r = examine(&g, &NodeId::new("c-bounded")).expect("node exists");
        assert_eq!(r.standing, Standing::ReviewReady);
        assert!(
            r.findings.is_empty(),
            "a bounded claim blocks nothing: {:?}",
            r.findings
        );
    }

    /// THE constraint-#2 test, end to end: a gate that reached no verdict must arrive as
    /// its own finding, never collapsed into a pass. `c-overclaim` produces a real
    /// `PEIR-GATE-UNASSESSED` (via ZHENGMING), so this rides the whole path from graph to
    /// serialisable report.
    #[test]
    fn examine_an_overclaim_is_evidence_pending_and_the_no_verdict_code_survives() {
        let g = vault("overclaim");
        let r = examine(&g, &NodeId::new("c-overclaim")).expect("node exists");
        assert_eq!(r.standing, Standing::EvidencePending);
        let codes: Vec<&str> = r.findings.iter().map(|f| f.code).collect();
        assert!(
            codes.contains(&"PEIR-GATE-UNASSESSED"),
            "no-verdict must reach the caller as its own code, not a pass: {codes:?}"
        );
        assert!(
            r.findings.iter().all(|f| !f.subject.is_empty()),
            "every vault finding names its subject"
        );
    }

    /// `status` answers standing and carries no findings list — the schema is the
    /// assertion.
    #[test]
    fn status_answers_standing_only() {
        let g = vault("overclaim");
        let r = status(&g, &NodeId::new("c-overclaim")).expect("node exists");
        assert_eq!(r.standing, Standing::EvidencePending);
    }

    /// A whole-vault survey: clean over `bounded`, and it names the offending node over
    /// `overclaim`.
    #[test]
    fn gates_surveys_the_whole_vault() {
        assert!(
            gates(&vault("bounded")).findings.is_empty(),
            "the bounded vault is clean"
        );

        // F3.1: the survey must carry the ENFORCED GATE pack, not only lints. The old
        // assertion (any subject == c-overclaim) was satisfied by a masking FORBIDDEN-VERB
        // lint while every gate finding — including the PEIR-GATE-UNASSESSED no-verdict
        // state — was silently dropped. Assert the survey is a superset of the gate-pack
        // codes the per-node examine surfaces, so a lint can no longer mask their absence.
        let g = vault("overclaim");
        let survey: std::collections::BTreeSet<&str> =
            gates(&g).findings.iter().map(|f| f.code).collect();
        let per_node = examine(&g, &NodeId::new("c-overclaim")).unwrap();
        for gate_code in per_node
            .findings
            .iter()
            .map(|f| f.code)
            .filter(|c| !c.contains("LINT"))
        {
            assert!(
                survey.contains(gate_code),
                "the whole-vault survey dropped the enforced-gate finding {gate_code}"
            );
        }
        assert!(
            survey.contains("PEIR-GATE-UNASSESSED"),
            "the no-verdict state must survive the survey, not collapse into a clean"
        );
    }

    /// An unknown node is an error that names the id, never an empty report that reads
    /// as "clean".
    #[test]
    fn an_unknown_node_is_an_error_not_an_empty_report() {
        let g = vault("bounded");
        let err = examine(&g, &NodeId::new("no-such")).unwrap_err();
        assert!(
            err.contains("no-such"),
            "the error names the missing id: {err}"
        );
        assert!(status(&g, &NodeId::new("no-such")).is_err());
    }

    /// The whole acceptance argument rests on empty-clean being distinguishable from
    /// clean-clean. Both an absent path and an existing-but-empty directory must error.
    #[test]
    fn an_empty_or_absent_vault_is_an_error_not_a_clean_result() {
        let absent = Path::new(env!("CARGO_MANIFEST_DIR")).join("../no-such-vault-xyz");
        assert!(load_vault(&absent).is_err(), "an absent vault is an error");

        let empty = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/empty-vault-fixture");
        std::fs::create_dir_all(&empty).expect("mk empty dir");
        assert!(
            load_vault(&empty).is_err(),
            "an empty vault is an error, not a clean result"
        );
    }

    /// No mintable number reaches the caller from a vault report either.
    #[test]
    fn no_number_reaches_the_caller_from_a_vault_report() {
        let g = vault("overclaim");
        let ex = serde_json::to_string(&examine(&g, &NodeId::new("c-overclaim")).unwrap()).unwrap();
        let ga = serde_json::to_string(&gates(&g)).unwrap();
        for minted in ["confidence", "score", "severity", "probability", "weight"] {
            assert!(!ex.contains(minted), "examine carries `{minted}`");
            assert!(!ga.contains(minted), "gates carries `{minted}`");
        }
    }

    /// Standing crosses as a stable `snake_case` string, so a caller can match on it.
    #[test]
    fn standing_serialises_as_a_stable_string() {
        use serde_json::json;
        assert_eq!(
            serde_json::to_value(Standing::ReviewReady).unwrap(),
            json!("review_ready")
        );
        assert_eq!(
            serde_json::to_value(Standing::Contested).unwrap(),
            json!("contested")
        );
        assert_eq!(
            serde_json::to_value(Standing::EvidencePending).unwrap(),
            json!("evidence_pending")
        );
    }

    // ── Tier 4 — freeze / verify ──────────────────────────────────────────────

    /// True iff no field is KEYED with a minted-score name, anywhere in the tree. Checks
    /// keys, not string content, so a packet body legitimately saying "carries no weight"
    /// does not trip it — the discipline forbids peira MINTING a score field, not the
    /// author using the word.
    fn has_no_score_key(v: &serde_json::Value) -> bool {
        const MINTED: [&str; 5] = ["confidence", "score", "severity", "probability", "weight"];
        match v {
            serde_json::Value::Object(m) => m
                .iter()
                .all(|(k, val)| !MINTED.contains(&k.as_str()) && has_no_score_key(val)),
            serde_json::Value::Array(a) => a.iter().all(has_no_score_key),
            _ => true,
        }
    }

    fn frozen_body(g: &Graph, id: &str) -> String {
        match freeze(g, &NodeId::new(id)).expect("a claim") {
            FreezeReport::Frozen { body, .. } => body,
            other => panic!("expected Frozen for {id}: {other:?}"),
        }
    }

    /// A bounded claim freezes into a packet whose first line names its subject.
    #[test]
    fn a_bounded_claim_freezes() {
        let body = frozen_body(&vault("bounded"), "c-bounded");
        assert!(
            body.starts_with("# Citation packet — c-bounded"),
            "the packet must name its subject on line 1: {body:.60}"
        );
    }

    /// An over-claim is REFUSED as a result carrying its reasons — never an error string.
    /// This is constraint #4: flattening `Blocked` to "failed" loses the product.
    #[test]
    fn an_overclaim_freeze_is_blocked_as_a_result_with_its_reasons() {
        let r = freeze(&vault("overclaim"), &NodeId::new("c-overclaim"))
            .expect("a result, not an error");
        match r {
            FreezeReport::Blocked { violations, .. } => {
                assert!(!violations.is_empty(), "blocked must carry why");
            }
            other => panic!("an over-claim must be blocked, not {other:?}"),
        }
    }

    /// A missing node and a non-claim are bad requests, distinct from a claim that
    /// declines to freeze.
    #[test]
    fn freezing_a_non_claim_or_missing_node_is_an_error() {
        let g = vault("bounded");
        assert!(freeze(&g, &NodeId::new("no-such")).is_err(), "missing node");
        let not_a_claim = freeze(&g, &NodeId::new("o1")).unwrap_err();
        assert!(
            not_a_claim.contains("not a claim"),
            "a non-claim names what it is: {not_a_claim}"
        );
    }

    /// A freshly frozen packet verifies against the same vault; tampering with the body
    /// is caught as a digest mismatch.
    #[test]
    fn verify_confirms_an_untouched_packet_and_catches_a_tampered_one() {
        let g = vault("bounded");
        let body = frozen_body(&g, "c-bounded");

        let clean = verify(&g, body.clone()).expect("a packet");
        assert!(
            matches!(clean, VerifyReport::Verified { .. }),
            "an untouched packet verifies: {clean:?}"
        );

        // Append a line the vault never rendered, keeping line 1 intact so the subject
        // still parses. The re-derivation will not contain it, so digests diverge.
        let tampered = format!("{body}\nan extra line the vault never rendered\n");
        let dirty = verify(&g, tampered).expect("a packet");
        assert!(
            matches!(dirty, VerifyReport::DigestMismatch { .. }),
            "a tampered packet is a mismatch: {dirty:?}"
        );
    }

    /// Text that is not a packet is a bad request, not a verdict.
    #[test]
    fn verifying_a_non_packet_is_an_error() {
        assert!(verify(&vault("bounded"), "not a packet".to_owned()).is_err());
    }

    /// No score field is minted by any Tier-4 report — checked on the freeze (both
    /// outcomes) and verify shapes.
    #[test]
    fn no_score_field_is_minted_by_the_tier_4_reports() {
        let g = vault("bounded");
        let frozen = freeze(&g, &NodeId::new("c-bounded")).unwrap();
        let blocked = freeze(&vault("overclaim"), &NodeId::new("c-overclaim")).unwrap();
        let verified = verify(&g, frozen_body(&g, "c-bounded")).unwrap();
        for report in [
            serde_json::to_value(&frozen).unwrap(),
            serde_json::to_value(&blocked).unwrap(),
            serde_json::to_value(&verified).unwrap(),
        ] {
            assert!(
                has_no_score_key(&report),
                "a score field was minted: {report}"
            );
        }
    }

    /// freeze and verify serialise to tagged OBJECTS, never a bare value — the Tier-1
    /// bare-array bug must not recur at a new surface.
    #[test]
    fn tier_4_reports_are_tagged_objects() {
        let g = vault("bounded");
        let frozen = serde_json::to_value(freeze(&g, &NodeId::new("c-bounded")).unwrap()).unwrap();
        assert_eq!(frozen.get("outcome"), Some(&serde_json::json!("frozen")));
        let verified =
            serde_json::to_value(verify(&g, frozen_body(&g, "c-bounded")).unwrap()).unwrap();
        assert_eq!(
            verified.get("outcome"),
            Some(&serde_json::json!("verified"))
        );
    }

    /// The documented example vault (`examples/prefetch-vault`) must stay freezable — a
    /// gate change that breaks it should fail the build, not silently make its README
    /// false. This is the "deliverables are compiled from source" rule applied to a doc.
    #[test]
    fn the_documented_example_vaults_stay_freezable() {
        for (dir, claim) in [
            ("../examples/prefetch-vault", "c-prefetch"),
            ("../examples/self-examination-vault", "c-mints-bounded"),
        ] {
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
            let g = load_vault(&root).unwrap_or_else(|e| panic!("{dir} loads: {e}"));
            assert!(
                gates(&g).findings.is_empty(),
                "{dir} must be clean: {:?}",
                gates(&g).findings
            );
            let frozen = freeze(&g, &NodeId::new(claim)).expect("a claim");
            assert!(
                matches!(frozen, FreezeReport::Frozen { .. }),
                "{dir}/{claim} must freeze: {frozen:?}"
            );
        }
    }

    // ── Tier 3 — propose ──────────────────────────────────────────────────────

    fn inferred_value<'a>(r: &'a ProposeReport, field: &str) -> &'a str {
        r.inferred
            .iter()
            .find(|i| i.field == field)
            .unwrap_or_else(|| panic!("no inference for {field}"))
            .value
    }

    /// The proposition is the author's sentence, verbatim — peira never rewrites it.
    #[test]
    fn propose_takes_the_proposition_verbatim() {
        let r = propose("  The Amcache entry proves execution of the binary.  ");
        assert_eq!(r.proposed_type, "claim");
        assert_eq!(
            r.proposition,
            "The Amcache entry proves execution of the binary."
        );
    }

    /// The rung is inferred from the verb, reusing the lint — a proving verb reads above
    /// the association rung (so the draft's RUNG gate will fire), a descriptive one does
    /// not. The over-claim is also surfaced in `prose_findings`.
    #[test]
    fn propose_infers_the_rung_from_the_verb_reusing_the_lint() {
        let over = propose("The Amcache entry proves execution.");
        assert_eq!(inferred_value(&over, "causal_rung"), "counterfactual");
        assert!(
            over.prose_findings
                .iter()
                .any(|f| f.code == "PEIR-LINT-FORBIDDEN-VERB"),
            "the proving verb is surfaced to the author too"
        );

        let bounded = propose("The Amcache entry records catalogued presence.");
        assert_eq!(inferred_value(&bounded, "causal_rung"), "association");
    }

    /// The quantifier is inferred from determiners.
    #[test]
    fn propose_infers_the_quantifier_from_determiners() {
        assert_eq!(
            inferred_value(&propose("Every host was compromised."), "quantifier"),
            "universal"
        );
        assert_eq!(
            inferred_value(&propose("This host was compromised."), "quantifier"),
            "singular"
        );
    }

    /// Only terms the author explicitly quoted become candidates — no guessing.
    #[test]
    fn propose_extracts_only_quoted_candidate_terms() {
        let r = propose("The record establishes \"presence\", never \"execution\".");
        assert_eq!(r.candidate_terms, vec!["presence", "execution"]);
        let none = propose("The record establishes catalogued presence.");
        assert!(
            none.candidate_terms.is_empty(),
            "nothing quoted, nothing guessed"
        );
    }

    /// Every blank is named with the gate that will demand it.
    #[test]
    fn propose_names_each_blank_with_its_gate() {
        let r = propose("The record shows presence.");
        let by_field: std::collections::BTreeMap<_, _> =
            r.needs.iter().map(|n| (n.field, n.gate)).collect();
        assert_eq!(by_field.get("warrant"), Some(&"PEIR-WARRANT-MISSING"));
        assert_eq!(by_field.get("boundaries"), Some(&"PEIR-BOUNDARIES-MISSING"));
        assert_eq!(by_field.get("falsifier"), Some(&"PEIR-FALSIFIER-MISSING"));
    }

    /// No evidentiary KEY (grade/by/via) and no `grade=`/`by=`/`via=` value may appear
    /// anywhere. All three tokens — the guard's scope note promises all three, and an
    /// earlier version enforced only `grade=`/`via=`, letting `by=` (authorship
    /// attribution) through.
    fn authors_no_evidence(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Object(m) => m.iter().all(|(k, val)| {
                !["grade", "by", "via"].contains(&k.as_str()) && authors_no_evidence(val)
            }),
            serde_json::Value::Array(a) => a.iter().all(authors_no_evidence),
            serde_json::Value::String(s) => {
                !s.contains("grade=") && !s.contains("by=") && !s.contains("via=")
            }
            _ => true,
        }
    }

    /// THE Tier-3 guard: `propose` authors no evidence, on any input.
    #[test]
    fn propose_authors_no_evidence() {
        for prose in [
            "The Amcache entry proves execution of the binary.",
            "Every host was compromised, and each was reviewed.",
            "The record shows \"catalogued presence\".",
        ] {
            let v = serde_json::to_value(propose(prose)).unwrap();
            assert!(authors_no_evidence(&v), "propose authored evidence: {v}");
        }
    }

    /// The guard's NEGATIVE control: prove each of grade=/by=/via= actually turns it red,
    /// on the exact echo path (propose copies the proposition verbatim). Without this the
    /// string arm never fired in any test, so the missing `by=` clause shipped unnoticed —
    /// a control never seen to fail is not known to work.
    #[test]
    fn the_evidence_guard_goes_red_on_each_of_grade_by_via() {
        for token in ["grade=G4", "by=analyst", "via=perception"] {
            let payload = serde_json::json!({ "proposition": format!("x {token} y") });
            assert!(
                !authors_no_evidence(&payload),
                "the guard must reject a value containing `{token}`"
            );
        }
        assert!(
            authors_no_evidence(&serde_json::json!({ "proposition": "no evidence grammar here" })),
            "control: a clean value passes"
        );
    }

    /// And it mints no score.
    #[test]
    fn propose_mints_no_score() {
        let v = serde_json::to_value(propose("The Amcache entry proves execution.")).unwrap();
        assert!(has_no_score_key(&v), "a score field was minted: {v}");
    }

    /// F4.2: a causal claim must read ABOVE association so the RUNG gate fires — the
    /// earlier version knew only proving verbs and floored causal claims at association,
    /// hiding the over-claim it exists to surface.
    #[test]
    fn infer_rung_fires_on_causal_claims_not_only_proving_verbs() {
        for prose in [
            "The malware caused the outage.",
            "Because of the exploit, the server fell over.",
            "The patch prevented the breach.",
        ] {
            assert_eq!(
                inferred_value(&propose(prose), "causal_rung"),
                "intervention",
                "causal claim floored at association: {prose:?}"
            );
        }
        assert_eq!(
            inferred_value(&propose("The tool proves execution."), "causal_rung"),
            "counterfactual",
            "a proving verb still reads counterfactual"
        );
        assert_eq!(
            inferred_value(
                &propose("The record shows catalogued presence."),
                "causal_rung"
            ),
            "association",
            "a plain descriptive claim is still the safe floor"
        );
    }

    /// F4.1: passive voice (" are …") is the majority forensic register and is a FUNCTION
    /// claim — it must not be labelled substance.
    #[test]
    fn infer_aspect_reads_passive_voice_as_function() {
        for prose in [
            "The logs are wiped.",
            "Processes are terminated.",
            "The users are authenticated.",
        ] {
            assert_eq!(
                inferred_value(&propose(prose), "aspect"),
                "function",
                "passive voice mislabelled substance: {prose:?}"
            );
        }
        assert_eq!(
            inferred_value(&propose("The file is a copy of the original."), "aspect"),
            "substance",
            "a genuine predicate nominal is still substance"
        );
    }

    /// F4.4: hyphens and apostrophes are word-internal — "all-clear"/"no-one"/"y'all" must
    /// not leak the universal determiners.
    #[test]
    fn contains_word_keeps_hyphens_and_apostrophes_word_internal() {
        for prose in [
            "All-clear was declared.",
            "No-one was harmed.",
            "Y'all know the drill.",
        ] {
            assert_eq!(
                inferred_value(&propose(prose), "quantifier"),
                "singular",
                "a hyphen/apostrophe leaked a determiner: {prose:?}"
            );
        }
        assert_eq!(
            inferred_value(&propose("Every host was compromised."), "quantifier"),
            "universal",
            "a real universal still fires"
        );
    }

    /// F4.5: a stray CLOSING curly quote must not open a span and fabricate a term.
    #[test]
    fn quoted_terms_ignores_a_stray_closing_quote() {
        let stray = propose("close-first \u{201D} then \u{201C}");
        assert!(
            stray.candidate_terms.is_empty(),
            "a stray close-quote fabricated a term: {:?}",
            stray.candidate_terms
        );
        let paired = propose("the record establishes \u{201C}presence\u{201D}");
        assert_eq!(
            paired.candidate_terms,
            vec!["presence"],
            "a real curly pair extracts"
        );
    }
}
