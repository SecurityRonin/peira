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
/// A claim the record itself withdraws or replaces.
pub const RETRACTED: &str = "PEIR-LINT-RETRACTED";
/// Support nobody has weighed.
pub const UNGRADED_SUPPORT: &str = "PEIR-LINT-UNGRADED-SUPPORT";
/// A finding that decides the ultimate issue — the tribunal's question, not the expert's.
pub const LEGAL_CONCLUSION: &str = "PEIR-LINT-LEGAL-CONCLUSION";
/// A declared field the claim's own language contradicts.
pub const DECLARATION_CONTRADICTED: &str = "PEIR-LINT-DECLARATION-CONTRADICTED";

/// Scan arbitrary rendered text for overstatement.
///
/// The counterpart to [`forbidden_verbs`], which walks a node's fields. This takes
/// text that has ALREADY been rendered — a packet body — so nothing has to be
/// enumerated in advance. A field list has to be kept in step with a renderer by
/// hand, and the list in this file was already out of date with `freeze` on the day
/// it was written: `warrant`, `boundaries` and `falsifier` were rendered verbatim
/// and scanned by nothing.
/// Every prose check, run over already-rendered text.
///
/// Runs ALL of them. The first version ran only the forbidden-verb list, so a legal
/// conclusion sailed through the body scan that was supposed to be the structural
/// backstop — the same defect as the field list it replaced, one layer up: enumerating
/// one check where every check was needed.
#[must_use]
pub fn prose_findings_in(text: &str, subject: &NodeId) -> Vec<Violation> {
    let mut out = overstatements_in(text, subject);
    let haystack = text.to_ascii_lowercase();
    out.extend(
        ULTIMATE_ISSUES
            .iter()
            .filter(|w| contains_phrase(&haystack, w) && !clause_negated(&haystack, w))
            .filter(|w| predicated_of_a_party(&haystack, w))
            .map(|w| {
                violation(
                    LEGAL_CONCLUSION,
                    subject,
                    format!(
                        "the rendered packet says \"{w}\" — that is the tribunal's question, \
and it would be sealed into the artifact"
                    ),
                    "state what the evidence shows and hand the conclusion back",
                )
            }),
    );
    out
}

#[must_use]
pub fn overstatements_in(text: &str, subject: &NodeId) -> Vec<Violation> {
    let haystack = text.to_ascii_lowercase();
    OVERSTATEMENTS
        .iter()
        .filter(|(word, _)| contains_phrase(&haystack, word) && !is_negated(&haystack, word))
        .map(|(word, instead)| {
            violation(
                FORBIDDEN_VERB,
                subject,
                format!(
                    "the rendered packet says \"{word}\" — it would be sealed into the \
artifact exactly as written"
                ),
                if instead.starts_with("(delete") {
                    "delete the intensifier and state what the evidence shows"
                } else {
                    "replace with consistent-with language: an observation is never a verdict"
                },
            )
        })
        .collect()
}

/// Authored fields that Court Mode renders into a packet VERBATIM.
///
/// The safe statement is generated so that nobody can overstate it — but these three
/// are copied through unaltered, so an overstatement placed here reaches a tribunal
/// having passed every check. They are scanned exactly like a title or a body.
///
/// This list is NOT the guarantee, and an earlier version of this comment claimed it
/// was — while `warrant`, `boundaries` and `falsifier` were rendered verbatim and
/// reached by nothing. Keeping a list in step with a renderer by hand is the failure
/// mode, not the fix. The guarantee is [`prose_findings_in`], which `freeze` runs over
/// the FINISHED body: whatever is sealed has been scanned, by construction.
///
/// These three stay because catching an overstatement at the TERM gives the author a
/// finding on the node they can fix, rather than on the packet.
pub const RENDERED_FIELDS: &[&str] = &["as_used", "not_essence", "stipulated"];

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

/// Words that decide the ultimate issue rather than describe evidence.
///
/// Words that only decide the ultimate issue when a PARTY is their subject.
///
/// "An innocent explanation remains consistent with the record" preserves an
/// alternative — the discipline this tool teaches — and blocking it punishes exactly
/// the careful author it exists to serve. The word decides nothing unless someone is
/// the one being called it.
const NEEDS_A_PERSON: &[&str] = &["innocent", "guilty", "liable", "negligent"];

/// Subjects that make those words a verdict about somebody.
const PARTIES: &[&str] = &[
    "suspect",
    "defendant",
    "accused",
    "respondent",
    "claimant",
    "he",
    "she",
    "they",
    "party",
    "holder",
    "account",
    "employee",
    "director",
    "company",
    "user",
];

/// **A heuristic backstop, and NOT complete.** An earlier version of this comment said
/// this list "can be COMPLETE for its purpose, unlike overstatement" — that was false,
/// and an outside review falsified it in one line: *"The defendant murdered the
/// victim"* is as pure an ultimate-issue verdict as exists and contains no word below.
/// Criminal and civil verdict verbs are an open class, exactly like overstatement.
///
/// The list catches the phrasings a technical author reaches for without noticing they
/// have crossed from evidence into verdict. Layer-3 detection remains a HUMAN
/// obligation — `docs/method/expert-witness.md` says so, and this lint does not change
/// that.
///
/// T3 instrument: our own table, not a decode of anyone's spec.
const ULTIMATE_ISSUES: &[&str] = &[
    // Verdict verbs — the act, stated as found rather than as evidenced.
    "murdered",
    "defrauded",
    "embezzled",
    "assaulted",
    "forged",
    "laundered",
    "trafficked",
    "misappropriated",
    "conspired",
    "perjured",
    "guilty",
    "innocent",
    "liable",
    "not liable",
    "negligent",
    "fraudulent",
    "committed fraud",
    "is fraud",
    "stole",
    "theft by",
    "breached the contract",
    "in breach of contract",
    "unlawful",
    "criminally",
    "defamed",
    "infringed",
];

/// Whether the clause containing `needle` is itself negated.
///
/// A clause boundary is a comma, semicolon, or a coordinating conjunction. Looking only
/// at the text between the nearest boundary and the word asks the right question — does
/// the negator GOVERN this word — where scanning the whole sentence merely asked whether
/// one appears somewhere.
///
/// A heuristic over English, and it says so: it will miss subordinate structures a
/// grammar would catch. It is deliberately biased toward FIRING, because the check it
/// guards blocks a packet and the alternative is an unhedged verdict reaching a
/// tribunal.
fn clause_negated(haystack: &str, needle: &str) -> bool {
    // EVERY occurrence, not the first. Checking `find()` alone asked about whichever
    // instance happened to come first, so "the suspect is NOT guilty of tampering; the
    // suspect IS guilty of unauthorised access" was excused by its own opening clause.
    // One unhedged occurrence is a verdict however many hedged ones precede it.
    occurrences(haystack, needle).all(|at| clause_at_is_negated(haystack, at))
}

/// Byte offsets of every whole-word occurrence of `needle`.
fn occurrences<'a>(haystack: &'a str, needle: &'a str) -> impl Iterator<Item = usize> + 'a {
    haystack.match_indices(needle).filter_map(move |(at, _)| {
        let end = at + needle.len();
        let before_ok = at == 0
            || !haystack[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
        let after_ok = end == haystack.len()
            || !haystack[end..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric);
        (before_ok && after_ok).then_some(at)
    })
}

fn clause_at_is_negated(haystack: &str, at: usize) -> bool {
    let head = &haystack[..at];
    // A NEWLINE is a clause boundary, and the strongest one. Without it this reads a
    // whole rendered packet as ONE clause, so a negator lines away suppresses a verdict
    // printed under its own heading — which is exactly how a legal conclusion on a
    // limiter survived the body scan.
    let start = [
        "\n",
        ",",
        ";",
        ":",
        " and ",
        " but ",
        " however ",
        " while ",
        " whereas ",
    ]
    .iter()
    .filter_map(|b| head.rfind(b).map(|i| i + b.len()))
    .max()
    .unwrap_or(0);

    head[start..]
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .any(|w| NEGATORS.contains(&w))
}

/// Whether an ultimate-issue word is said OF SOMEBODY.
///
/// `innocent`, `guilty`, `liable`, `negligent` decide nothing when their subject is a
/// thing: "an innocent explanation", "a guilty plea was not entered", "the liable
/// portion of the balance". They decide everything when their subject is a party.
/// Words like `unlawful` or `infringed` need no such test — they are verdicts about
/// conduct however they are phrased.
///
/// A heuristic, and a deliberately conservative one: it looks for a party word in the
/// same clause. Missing a real conclusion costs less here than blocking the careful
/// sentence an expert is obliged to write.
fn predicated_of_a_party(haystack: &str, word: &str) -> bool {
    if !NEEDS_A_PERSON.contains(&word) {
        return true;
    }
    // ANY occurrence said of a party makes it a verdict — the same reason
    // `clause_negated` looks at all of them rather than the first.
    occurrences(haystack, word).any(|at| {
        let start = ["\n", ",", ";", ":", " and ", " but "]
            .iter()
            .filter_map(|b| haystack[..at].rfind(b).map(|i| i + b.len()))
            .max()
            .unwrap_or(0);
        haystack[start..at]
            .split(|c: char| !c.is_alphanumeric() && c != '\'')
            .any(|w| PARTIES.contains(&w))
    })
}

/// A declaration the claim's own words contradict.
///
/// The gates trust `quantifier:` and `causal_rung:` because an author is better placed
/// than a word list to say what a claim asserts. But trust is not the same as
/// unexamined: "deleting the file caused the loss of evidence on EVERY host" declared
/// `quantifier: singular` and `causal_rung: association`, and both gates switched
/// themselves off. That is not omission — it is an affirmative false declaration, and
/// the cheapest one available, because writing the field looks like diligence.
///
/// This does NOT decide what the claim really is; it reports the DISAGREEMENT between
/// what the author declared and how the author wrote. A heuristic, and it says so —
/// but a contradiction between two things the author supplied is a fact about the
/// document, not an inference about the world.
fn declaration_contradicted(node: &Node) -> Vec<Violation> {
    // STRONG universals are attributions wherever they appear — "every host was
    // compromised by the account holder" is a universal claim in a body as much as in a
    // title. WEAK ones are scanned in the title only: "this node holds the pointer,
    // never the bytes" is method description, and firing on it broke this repository's
    // own clean fixture.
    const STRONG: &[&str] = &["every", "all", "always", "each"];
    const WEAK: &[&str] = &["any", "never", "none"];
    // Strong causal markers only. "produced" and "made" were tried and removed: both
    // are routine descriptive verbs in forensic writing — "the tool produced output",
    // "installation produced the record" — and flagging them punished a legitimate
    // rival hypothesis in this repository's own fixture. A heuristic that fires on
    // ordinary professional prose is one people switch off.
    const INTERVENTIONAL: &[&str] = &[
        "caused",
        "causes",
        "causing",
        "resulted in",
        "led to",
        "because of",
        "triggered",
    ];

    let title = node.title.to_ascii_lowercase();
    let full = format!("{} {}", node.title, node.body).to_ascii_lowercase();
    let haystack = title.clone();
    let mut out = Vec::new();

    // Fires on ABSENCE as well as on a false declaration. Declaring `singular` on a
    // universal sentence was caught; declaring nothing at all was not, and that is the
    // cheaper move — a constraint that activates on a field's presence is evaded by its
    // absence. Silence where an assertion was made is not neutral.
    let quantifier = node.field("quantifier");
    if quantifier != Some("universal") && quantifier != Some("class") {
        let found = STRONG
            .iter()
            .find(|w| contains_phrase(&full, w))
            .or_else(|| WEAK.iter().find(|w| contains_phrase(&title, w)));
        if let Some(w) = found {
            out.push(violation(
                DECLARATION_CONTRADICTED,
                &node.id,
                match quantifier {
                    Some(q) => format!(
                        "declares `quantifier: {q}` but says \"{w}\" — 白馬非馬 was \
switched off by the declaration, not by the claim"
                    ),
                    None => format!(
                        "says \"{w}\" and declares no `quantifier:` — the extension gate \
never runs, so the widest word in the sentence is the one nothing examined"
                    ),
                },
                "declare the quantifier the sentence actually uses, or rewrite the \
sentence to the scope you can support",
            ));
        }
    }

    if node.field("causal_rung") == Some("association") {
        if let Some(w) = INTERVENTIONAL
            .iter()
            .find(|w| contains_phrase(&haystack, w))
        {
            out.push(violation(
                DECLARATION_CONTRADICTED,
                &node.id,
                format!(
                    "declares `causal_rung: association` but says \"{w}\" — that is a \
claim about doing, not about seeing"
                ),
                "raise the rung and satisfy it with an executed protocol, or state the \
association without the causal verb",
            ));
        }
    }
    out
}

/// The subject of a packet, withdrawn by its own record.
///
/// [`retracted`] reports only where a withdrawn node holds something ELSE up, so that
/// retained history stays quiet. That leaves the case where the retired claim IS the
/// packet, which `freeze` asks about directly.
#[must_use]
pub fn subject_withdrawn(graph: &Graph, id: &NodeId) -> Option<Violation> {
    // The FIXED POINT, not a direct-edge test. A retraction that has itself been
    // retracted no longer binds — `attackers` already knew that and this did not, so a
    // corrected lifecycle record was `review_ready` and impossible to freeze at once.
    if !graph.withdrawn().contains(id) {
        return None;
    }
    graph
        .edges_to(id)
        .find(|e| matches!(e.kind, EdgeKind::Retracts | EdgeKind::Supersedes))
        .map(|e| {
            violation(
                RETRACTED,
                id,
                format!(
                    "`{id}` is withdrawn by `{}`, in the record it is drawn from",
                    e.from
                ),
                "cite the superseding version, or the retraction itself",
            )
        })
}

/// A claim that decides the tribunal's question instead of describing evidence.
///
/// Layer 3 is never the expert's — see `docs/method/expert-witness.md`. The
/// forbidden-verb lint cannot reach this: *"the suspect is guilty of unauthorised
/// access"* contains no overstated verb, passes every check, and freezes into a court
/// packet under the tool's own authority.
///
/// Reports rather than blocks would be the softer choice, and is the wrong one here: a
/// packet is the artifact that reaches a tribunal, and this is the one sentence that
/// must never reach one.
fn legal_conclusions(node: &Node) -> Vec<Violation> {
    // MENTION IS NOT USE. A falsifier names a possibility to be tested — "evidence that
    // the transfer was fraudulent" is exactly what PEIR-FALSIFIER-MISSING demands — and
    // scanning it refused the packet for containing the word the tool required. Two of
    // its own rules in direct contradiction, and the one that lost was the discipline.
    //
    // Only the title and body are scanned: those assert. `falsifier:`, `boundaries:`
    // and the term moments describe, quote or delimit.
    let haystack = format!("{} {}", node.title, node.body).to_ascii_lowercase();
    // Negation is scoped to the CLAUSE the word sits in, not the whole sentence.
    //
    // "does a negator appear anywhere" was the wrong question, and appending any
    // negated clause switched this check off — including, in the worked case, a clause
    // that is ITSELF on the substitution table: whether a thing is "in dispute" is the
    // tribunal's call, so the phrase that evaded the check is one the check exists to
    // catch.
    //
    //   negator in the SAME clause  -> "the record is NOT evidence that X is LIABLE"
    //                                  a correct negative finding, and it passes
    //   negator in a LATER clause   -> "X is GUILTY, and this is NOT in dispute"
    //                                  the conclusion stands unhedged, and it fires
    ULTIMATE_ISSUES
        .iter()
        .filter(|w| contains_phrase(&haystack, w) && !clause_negated(&haystack, w))
        .filter(|w| predicated_of_a_party(&haystack, w))
        .map(|w| {
            violation(
                LEGAL_CONCLUSION,
                &node.id,
                format!("says \"{w}\" — that is the tribunal's question, not the evidence's"),
                "state what the evidence shows and hand the conclusion back: \
\"the Court may draw its own conclusions\"",
            )
        })
        .collect()
}

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

/// Words that negate the phrase following them.
///
/// Kept small and literal on purpose: this decides whether to SUPPRESS a finding, so a
/// generous list would silence real overstatements. Anything subtler than a negation
/// immediately before the verb is left to a human.
const NEGATORS: &[&str] = &[
    "not", "never", "no", "nothing", "cannot", "does", "doesn't", "don't", "isn't", "wasn't",
    "neither", "nor", "without",
];

/// Whether the occurrence of `needle` in `haystack` sits inside its own negation.
///
/// The 即非 moment's whole job is denial — "a catalogue entry does not PROVE
/// execution" is the correct form, and flagging it punishes exactly the careful author
/// the lint exists to serve. Looks back a few words only: "not" three words before
/// "proves" negates it; "not" two sentences earlier does not.
fn is_negated(haystack: &str, needle: &str) -> bool {
    let Some(at) = haystack.find(needle) else {
        return false;
    };
    haystack[..at]
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .rev()
        // Six words, not four: "is NOT evidence that the account holder is liable"
        // puts the negator six back, and flagging that sentence would block the exact
        // careful phrasing the expert-witness discipline asks for.
        .take(6)
        .any(|w| NEGATORS.contains(&w))
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
    // Every place an overstatement can reach a reader: the prose, and the authored
    // term fields Court Mode renders verbatim into the packet. Scanning title and
    // body alone left the ONE channel that reaches a tribunal unchecked.
    let mut haystacks = vec![("", format!("{} {}", node.title, node.body))];
    for f in RENDERED_FIELDS {
        if let Some(v) = node.field(f) {
            haystacks.push((*f, v.to_owned()));
        }
    }

    haystacks
        .iter()
        .flat_map(|(field, text)| {
            let haystack = text.to_ascii_lowercase();
            OVERSTATEMENTS
                .iter()
                .filter(move |(word, _)| {
                    contains_phrase(&haystack, word) && !is_negated(&haystack, word)
                })
                .map(move |(word, instead)| (*field, *word, *instead))
        })
        .map(|(field, word, instead)| {
            violation(
                FORBIDDEN_VERB,
                &node.id,
                if field.is_empty() {
                    format!("says \"{word}\" — {}", node.title)
                } else {
                    // Name the field: three are rendered, and a bare "says proves"
                    // sends the author hunting through the node.
                    format!(
                        "`{field}:` says \"{word}\" — and Court Mode renders that field \
verbatim into the packet"
                    )
                },
                // The remedy is a &'static str, so the substitution is carried in the
                // detail line rather than fabricated per-call.
                match instead {
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

/// Claims the vault's own record retires.
///
/// The parser refuses `status: withdrawn` with a principled message. A `retracts:`
/// edge says exactly that in the graph's own grammar — and was parsed, recorded and
/// read by nothing, so a packet froze for a claim the record had withdrawn without
/// ever mentioning it. **When you forbid something, sweep for the other grammars
/// that express it.**
///
/// `Supersedes` is included: "a newer version replaces this" is the same lifecycle
/// statement, and citing the retired version is the same error.
///
/// This is a lint rather than an attack edge on purpose. A retraction is not a
/// dialectical move that a counter-argument could defeat — modelling it as one would
/// let a claim win against its own withdrawal in the grounded extension.
fn retracted(graph: &Graph, node: &Node) -> Vec<Violation> {
    // A retired version EXISTING is correct — "retained, never deleted" is the design.
    // Reporting it forever trains a reader to skip the category, and a permanently red
    // check is worse than none. The finding is raised only where the retired node is
    // still LOAD-BEARING: something live still leans on it. Court Mode reaches this
    // through the evidential closure, so a packet resting on withdrawn work is refused
    // while a properly retained history stays quiet.
    // `is_attack()`, not the literal `Attacks` kind: a withdrawn node that CONTRADICTS
    // or NEGATES something is doing exactly as much work as one that attacks it, and
    // naming one of three grammars let the other two pass unreported. When you forbid a
    // thing, sweep for the other spellings of it.
    // OUTGOING only. A retired node keeps the evidence it once rested on — that is the
    // retention the design requires — and counting its own incoming support as "still
    // cited" made the comment below false: every properly retained history fired
    // forever. What matters is whether the withdrawn node is holding something ELSE up.
    let still_cited = graph.edges_from(&node.id).any(|e| {
        e.kind == EdgeKind::Supports || e.kind == EdgeKind::DependsOn || e.kind.is_attack()
    });
    if !still_cited {
        return Vec::new();
    }

    graph
        .edges_to(&node.id)
        .filter(|e| matches!(e.kind, EdgeKind::Retracts | EdgeKind::Supersedes))
        .map(|e| {
            let (verb, remedy) = if e.kind == EdgeKind::Retracts {
                (
                    "retracted",
                    "cite the retraction, or delete the claim — a packet frozen over a \
withdrawn claim is a conclusion the record itself retired",
                )
            } else {
                (
                    "superseded",
                    "cite the superseding version instead; the retired one is history, \
not a finding",
                )
            };
            violation(
                RETRACTED,
                &node.id,
                format!(
                    "\"{}\" is {verb} by `{}`, and the vault records it",
                    node.title, e.from
                ),
                remedy,
            )
        })
        .collect()
}

/// Support nobody has weighed.
///
/// The sibling of [`unreviewed_grades`] one step earlier: that catches a grade
/// PROPOSED and never settled, this catches an edge nobody graded at all. Until now
/// an ungraded, unattributed edge supported promotion exactly as effectively as
/// reviewed direct perception, so `Grade` and `Pramana` bound only authors who chose
/// to be bound — the apparatus was inert unless volunteered into.
///
/// Claims only, and support edges only. A hypothesis may rest on anything while it is
/// still a candidate; an observation is not graded, it is what does the grading.
fn ungraded_support(graph: &Graph, node: &Node) -> Vec<Violation> {
    // Claims, and hypotheses something LEANS ON — the same load-bearing rule the
    // promotion gates use. A node-kind test here let an ungraded inference edge hide
    // inside a chain: observation --graded--> h1 --UNGRADED--> h2 --graded--> claim
    // reported nothing, went review_ready, and froze. Every other check in this file
    // asks whether a node is carrying weight; this one asked what kind it was.
    let carries_weight = node.kind == NodeKind::Claim
        || (node.kind == NodeKind::Hypothesis
            && graph
                .edges_from(&node.id)
                .any(|e| e.kind == EdgeKind::Supports));
    if !carries_weight {
        return Vec::new();
    }
    graph
        .edges_to(&node.id)
        .filter(|e| e.kind == EdgeKind::Supports && e.grade().is_none())
        .map(|e| {
            violation(
                UNGRADED_SUPPORT,
                &node.id,
                format!(
                    "support edge {} → {} carries no settled grade, so nothing has weighed it",
                    e.from, e.to
                ),
                "grade the edge and attribute it — `grade=G2 by=<reviewer> via=<means>` \
— or record why it is cited ungraded",
            )
        })
        .collect()
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
            // Reported on the node CARRYING the edge, not on the claim it points at.
            // `by=` is an unauthenticated free string, so anyone able to write the vault
            // can attach `by=<author>` to a claim and make its own `peira status` report
            // it as defective — the finding landed on the victim, naming a file its
            // author cannot fix. It now names the file that can be.
            //
            // The underlying exposure is not fixable here and is stated rather than
            // papered over: peira cannot authenticate `by=`. In a shared vault the
            // question "who actually wrote this edge" is answered by the version
            // control history, not by the field.
            Some(violation(
                SELF_GRADED,
                &e.from,
                format!(
                    "the grade this node puts on {} → {} is settled by `{grader}`, who \
authored \"{}\" — `by=` is not authenticated, so check the history if you did not write it",
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
        out.extend(retracted(graph, node));
        out.extend(ungraded_support(graph, node));
        out.extend(legal_conclusions(node));
        out.extend(declaration_contradicted(node));
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

    /// The lint must reach every field the packet renders.
    ///
    /// `safe_statement` renders a Term's `as_used`, `not_essence` and `stipulated`
    /// verbatim into the court artifact, and `forbidden_verbs` scanned only title and
    /// body. So the one channel that reaches a tribunal was the one channel nobody
    /// checked: "proves" placed in a stipulation passed clean and printed unaltered.
    ///
    /// "Nobody writes the sentence, so nobody can overstate it" is false while an
    /// authored field is rendered verbatim. Either the lint reaches those fields or
    /// the claim is not true.
    #[test]
    fn an_overstatement_in_a_rendered_term_field_is_caught() {
        let term = node(
            "---\nid: t1\ntype: term\ntitle: execution\n\
as_used: the program ran\n\
not_essence: the record is not the running\n\
stipulated: the entry proves the suspect executed the binary\n---\n",
        );
        let g = graph_of(vec![term], vec![]);
        let found: Vec<Violation> = lint(&g)
            .into_iter()
            .filter(|v| v.gate == "PEIR-LINT-FORBIDDEN-VERB")
            .collect();
        assert_eq!(
            found.len(),
            1,
            "an overstatement in `stipulated:` reaches the packet verbatim and must be caught"
        );
        assert!(
            found[0].detail.contains("stipulated"),
            "the finding must name WHICH field carries it, since three are rendered: {}",
            found[0].detail
        );
    }

    /// Negation is scoped to a CLAUSE, not a sentence.
    ///
    /// Sentence-level negation was the right instinct for the wrong reason: it asked
    /// *does a negator appear* rather than *does the negator govern this word*. So
    /// appending any negated clause switched the check off — and the appended clause in
    /// the worked case is itself on the expert-witness substitution table as something
    /// to delete, because whether a thing is in dispute is the tribunal's call.
    #[test]
    fn a_negator_in_a_later_clause_does_not_excuse_the_conclusion() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-LEGAL-CONCLUSION")
                .count()
        };

        assert_eq!(
            fired("The suspect is guilty of unauthorised access, and this is not in dispute"),
            1,
            "the negator governs a later clause; the conclusion still stands unhedged"
        );
        assert_eq!(
            fired("The record is not evidence that the account holder is liable"),
            0,
            "here the negator governs the clause the word sits in — a correct negative finding"
        );
        assert_eq!(
            fired("No relationship was found, and the transfer cannot be attributed"),
            0,
            "an ordinary hedged negative must stay quiet"
        );
        assert_eq!(
            fired("The defendant is liable; nothing further was examined"),
            1,
            "a semicolon is a clause boundary too"
        );
    }

    /// An ultimate-issue word is a verdict only when a PARTY is its subject.
    ///
    /// "An innocent explanation remains consistent with the record" preserves an
    /// alternative — the discipline this tool teaches — and it was blocked. That is the
    /// class of careful-author punishment that gets a checker disabled.
    #[test]
    fn an_ultimate_issue_word_said_of_a_thing_is_not_a_verdict() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-LEGAL-CONCLUSION")
                .count()
        };
        assert_eq!(
            fired("An innocent explanation remains consistent with the Amcache record"),
            0,
            "preserving an alternative is the discipline, not a verdict"
        );
        assert_eq!(
            fired("The suspect is guilty of unauthorised access"),
            1,
            "said of a party, it decides the tribunal's question"
        );
    }

    /// A declared field the claim's own language contradicts.
    ///
    /// Writing `quantifier: singular` on a sentence saying "every host" switched 白馬非馬
    /// off by declaration. Not omission — an affirmative false declaration, and the
    /// cheapest one available, because writing the field looks like diligence.
    #[test]
    fn a_declaration_its_own_sentence_contradicts_is_reported() {
        let fired = |front: &str, title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: {title}\n{front}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-DECLARATION-CONTRADICTED")
                .count()
        };
        assert_eq!(
            fired(
                "quantifier: singular\ncausal_rung: association",
                "Deleting the file caused the loss of evidence on every host"
            ),
            2,
            "both declarations are contradicted by the sentence that carries them"
        );
        assert_eq!(
            fired(
                "quantifier: singular\ncausal_rung: association",
                "Installation or inventory produced the record without user execution"
            ),
            0,
            "\"produced\" is ordinary forensic description — a heuristic that fires on \
professional prose is one people switch off"
        );
    }

    /// A claim may not decide the tribunal's question.
    ///
    /// Layer 3 is never the expert's. The forbidden-verb lint cannot reach this —
    /// "the suspect is guilty of unauthorised access" contains no overstated verb, so
    /// it passed every check and froze into a court packet.
    ///
    /// The negative case is the one that keeps this honest: a careful negative finding
    /// uses the same words and must pass.
    #[test]
    fn a_claim_may_not_decide_the_ultimate_issue() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-LEGAL-CONCLUSION")
                .count()
        };
        assert_eq!(
            fired("The suspect is guilty of unauthorised access"),
            1,
            "a bald legal conclusion must be caught"
        );
        assert_eq!(
            fired("The record is not evidence that the account holder is liable"),
            0,
            "a careful negative finding uses the same words and must pass — blocking it \
would punish the discipline this lint teaches"
        );
        assert_eq!(
            fired("The record shows a process was created from this image"),
            0,
            "an ordinary evidential statement must pass"
        );
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

        // LOAD-BEARING: c1 still holds c2 up, so withdrawing it is a live problem.
        let withdrawn = graph_of(
            vec![claim("c1"), claim("c2"), obs.clone(), node("---\nid: d1\ntype: dissent\ntitle: withdrawn after the parser was found wrong\n---\n")],
            vec![
                supports("o1", "c1"),
                supports("c1", "c2"),
                Edge::new(NodeId::new("d1"), NodeId::new("c1"), EdgeKind::Retracts),
            ],
        );
        assert_eq!(
            retracted(&withdrawn, "c1"),
            1,
            "a withdrawn claim that still holds something up must be flagged"
        );

        // RETIRED: nothing leans on it. The evidence it once rested on is retention,
        // not citation, and reporting it forever trains a reader to skip the category.
        // `freeze` still refuses a packet whose SUBJECT is withdrawn.
        let retired = graph_of(
            vec![
                claim("c1"),
                obs.clone(),
                node("---\nid: d2\ntype: dissent\ntitle: retired\n---\n"),
            ],
            vec![
                supports("o1", "c1"),
                Edge::new(NodeId::new("d2"), NodeId::new("c1"), EdgeKind::Retracts),
            ],
        );
        assert_eq!(
            retracted(&retired, "c1"),
            0,
            "properly retained history stays quiet"
        );

        // Supersession is the same shape, and load-bearing the same way: c1 still holds
        // c3 up while the record says a newer version replaces c1.
        let superseded = graph_of(
            vec![claim("c1"), claim("c2"), claim("c3"), obs.clone()],
            vec![
                supports("o1", "c1"),
                supports("c1", "c3"),
                Edge::new(NodeId::new("c2"), NodeId::new("c1"), EdgeKind::Supersedes),
            ],
        );
        assert_eq!(
            retracted(&superseded, "c1"),
            1,
            "a superseded claim still holding something up is the same shape"
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
