//! The deterministic lint pack — checks that need no model and no catalogue.
//!
//! These are cheap, run over the whole graph, and catch the defects that recur
//! whatever the domain: prose that overstates its evidence, references that go
//! nowhere, grades nobody stands behind, and corroboration counted as independence.

use crate::{carries_weight, Violation};
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
/// An instrument nobody has shown to work.
pub const UNCONTROLLED_INSTRUMENT: &str = "PEIR-LINT-UNCONTROLLED-INSTRUMENT";

/// An attack edge whose author or target cannot argue.
pub const NON_ARGUMENT_ATTACK: &str = "PEIR-LINT-NON-ARGUMENT-ATTACK";
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
        .filter(|(word, _)| {
            contains_phrase(&haystack, word) && !every_occurrence_is_excused(&haystack, word)
        })
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

/// Deliberately only the unambiguous ones. "user", "account", "holder", "company" and
/// "party" were on this list and collided with ordinary technical language — "user
/// hive" made "Entries in the user hive are liable to be overwritten" read as a verdict
/// Parties a verdict can be about.
///
/// The list is COMPLETE — nothing is omitted for collision. It was cut back twice and
/// both cuts were wrong: removing `employee`, `company`, `holder` and `party` let "The
/// employee is guilty of data theft" freeze, and removing `user` let "The user is
/// guilty of unauthorised access" freeze.
///
/// The collision that prompted the cutting — "user hive" reading as a verdict about a
/// person — is handled in [`clause_has_party`], where a party word followed by a
/// technical noun is recognised as naming a thing. Fixing a false positive by deleting
/// the rule is how a checker stops checking.
const PARTIES: &[&str] = &[
    "suspect",
    "defendant",
    "accused",
    "respondent",
    "claimant",
    "employee",
    "company",
    "user",
    "director",
    "holder",
    "party",
    "he",
    "she",
    "they",
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
    occurrences(haystack, needle).all(|at| clause_at_is_negated(haystack, at, needle.len()))
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

/// Frames that make what follows SOMEBODY ELSE'S assertion.
///
/// Reciting the allegation from the instructions is a CPR 35 duty; quoting the
/// contention being rebutted is how a rebuttal is written. Both were read as the
/// author's own verdict, so the tool refused the two sentences its own discipline
/// requires. What follows an attribution is reported, not asserted.
///
/// A closed list, and it must stay closed: the frames are formulaic, while the
/// assertions they introduce are not. An unrecognised frame merely means the sentence
/// is judged as the author's own, which is the safe direction.
const ATTRIBUTIONS: &[&str] = &[
    "i am instructed that",
    "i have been instructed that",
    "i am asked to assume that",
    "i am told that",
    "it is the claimant's case that",
    "it is the respondent's case that",
    "it is the defendant's case that",
    "it is alleged that",
    "it is averred that",
    "it is contended that",
    "the claimant alleges that",
    "the claimant contends that",
    "the respondent alleges that",
    "the respondent contends that",
    "the opposing report asserts that",
    "the opposing report contends that",
    "the opposing expert asserts that",
    "the particulars of claim allege that",
    "the pleading alleges that",
];

/// Openers that refuse to conclude, and govern the whole sentence after them.
///
/// "It has not been possible, on the material provided, to say that the entry confirms
/// execution" is a refusal, and the clause carrying the verb holds no negator — the
/// negation sits in the matrix two clauses back. Clause scope is right for ordinary
/// prose and wrong for this shape, which is exactly the phrasing the discipline asks an
/// expert to use when the evidence will not carry the point.
/// Handing the question back, which is the remedy this tool RECOMMENDS.
///
/// "Whether the respondent is liable is a matter for the court" is the sentence an
/// expert is supposed to write when the ultimate issue arises — and it was refused as a
/// legal conclusion, blocking the packet. A checker that rejects its own prescribed
/// remedy is one an expert switches off at the first encounter.
///
/// These forms ASSERT NOTHING about the issue; they decline it explicitly. Closed and
/// formulaic, like the other frames.
const HAND_BACKS: &[&str] = &[
    "is a matter for the court",
    "is a matter for the tribunal",
    "is a question for the court",
    "is a question for the tribunal",
    "is for the court to decide",
    "is for the tribunal to decide",
    "the court may draw its own conclusions",
    "the tribunal may draw its own conclusions",
    "i express no view on",
    "i express no opinion on",
];

const HEDGE_OPENERS: &[&str] = &[
    "it has not been possible",
    "it was not possible",
    "it is not possible",
    "it cannot be said",
    "it could not be said",
    "it could not be established",
    "it cannot be established",
    "it cannot be determined",
    "it could not be determined",
    "no view is expressed",
    "no opinion is expressed",
];

/// The sentence containing `at`, and whether it is spoken by somebody else or refused.
///
/// Both questions are asked of the SENTENCE rather than the clause, because both frames
/// govern everything after them: an attribution hands the whole statement to another
/// speaker, and a hedge opener declines the whole statement.
fn sentence_is_reported_or_refused(haystack: &str, at: usize) -> bool {
    // THE FRAME GOVERNS ITS OWN CLAUSE, and no further. Reaching to the next full stop
    // let the author resume their own voice after a semicolon or a conjunction and keep
    // the exemption:
    //
    //   "It is alleged that …; my own analysis shows the respondent forged the entry"
    //   "It is not possible to say precisely when, but the respondent forged the ledger"
    //
    // Both are the author asserting, in a sentence that opens by quoting or declining.
    // Looking only inside the current clause makes the reach exactly as long as the
    // grammar allows.
    //
    // A hedge opener is matched by CONTAINS rather than `starts_with` for the same
    // reason in the other direction: "In my opinion, it cannot be said that …" is the
    // same refusal, and requiring the sentence to begin with the formula refused it.
    let region = &haystack[clause_start(haystack, at)..at];
    // A hand-back is read over the WHOLE clause, not just the part before the word: the
    // formula sits AFTER the issue it declines — "whether the respondent is liable is a
    // matter for the court" — so looking only backwards from `liable` would miss it.
    // `clause_end`, not a second boundary set written by hand. This scan stopped only at
    // `. ! ? ; \n` while `clause_start` also cuts at ` and `, ` but ` and an em dash, so a
    // verdict laundered by appending the tool's own remedy: "The respondent is guilty of
    // fraud and whether costs follow is a matter for the court" sealed. One boundary set,
    // both directions.
    let whole_clause = &haystack[clause_start(haystack, at)..clause_end(haystack, at)];
    ATTRIBUTIONS.iter().any(|a| region.contains(a))
        || HEDGE_OPENERS.iter().any(|h| region.contains(h))
        || HAND_BACKS.iter().any(|b| whole_clause.contains(b))
}

/// Where the clause containing byte offset `at` begins.
///
/// ONE definition, because two existed and disagreed in both directions.
/// `clause_negated` treated `.`/`!`/`?` as boundaries and `clause_has_party` did not, so
/// a party named in a previous sentence leaked forward; and both treated a bare COMMA as
/// a boundary, which severs a parenthetical from what it modifies:
///
///   "The defendant is, on this evidence, guilty of fraud."   party lost, verdict sealed
///   "There is no evidence, in the material examined, that …" negator lost, denial refused
///
/// A comma is not a clause boundary in English — it is punctuation inside one. The
/// boundaries are the sentence terminators, the semicolon, and the coordinating
/// conjunctions that genuinely start a new claim.
/// The one boundary set, shared by both directions.
const CLAUSE_BOUNDS: &[&str] = &[
    "\n",
    ".",
    "!",
    "?",
    ";",
    // An em dash starts a new clause as surely as a semicolon does.
    "—",
    " and ",
    " but ",
    " however ",
    " while ",
    " whereas ",
];

/// Where the clause containing byte offset `at` ENDS.
///
/// The twin of `clause_start`, and it exists because a second forward scan was written
/// beside it with a SMALLER boundary set — stopping only at `. ! ? ; \n` while
/// `clause_start` also cuts at ` and `, ` but ` and an em dash. A verdict then laundered
/// by appending the tool's own remedy: "The respondent is guilty of fraud and whether
/// costs follow is a matter for the court" sealed.
///
/// One definition per direction, sharing one boundary set.
fn clause_end(haystack: &str, at: usize) -> usize {
    CLAUSE_BOUNDS
        .iter()
        .filter_map(|b| haystack[at..].find(b).map(|i| at + i))
        .min()
        .unwrap_or(haystack.len())
}

fn clause_start(haystack: &str, at: usize) -> usize {
    CLAUSE_BOUNDS
        .iter()
        .filter_map(|b| haystack[..at].rfind(b).map(|i| i + b.len()))
        .max()
        .unwrap_or(0)
}

/// Whether `confirms` here is a cryptographic operation rather than an opinion.
///
/// "Hash verification confirmed the image digest" reports what a tool DID. The word is
/// on the substitution table because "the evidence confirms execution" is an
/// overstatement of belief — but a checksum matching is a fact about bytes, and every
/// acquisition note in forensics is written this way. Refusing it makes the tool
/// unusable on exactly the material it exists for.
///
/// Deliberately narrow: only where an integrity operation is the subject.
/// Whether EVERY occurrence of `needle` is excused — negated, or reporting what a
/// verification tool did.
///
/// One per-occurrence rule instead of two document-wide ones. The verification carve-out
/// was applied to the WHOLE haystack, so a single "digest" anywhere in a node — or
/// anywhere in a rendered packet body — switched the overstatement scan off entirely.
/// Every acquisition note in forensics contains that word, so the check was off by
/// default on exactly the material it exists for.
fn every_occurrence_is_excused(haystack: &str, needle: &str) -> bool {
    occurrences(haystack, needle).all(|at| {
        clause_at_is_negated(haystack, at, needle.len())
            || is_a_verification_operation(
                &haystack[clause_start(haystack, at)..clause_end(haystack, at)],
            )
    })
}

fn is_a_verification_operation(clause: &str) -> bool {
    const OPERATIONS: &[&str] = &[
        "hash verification",
        "hash check",
        "checksum",
        "digest",
        "signature verification",
        "integrity check",
        "integrity verification",
        "write blocker",
    ];
    OPERATIONS.iter().any(|o| clause.contains(o))
}

fn clause_at_is_negated(haystack: &str, at: usize, len: usize) -> bool {
    const OBJECT_NEGATORS: &[&str] = &["nothing", "no", "none", "neither"];
    // Somebody else's words, or a refusal to conclude. Both govern the whole sentence,
    // so both are asked of the sentence rather than the clause.
    if sentence_is_reported_or_refused(haystack, at) {
        return true;
    }
    // A negator can FOLLOW the word it governs, and only in the object position.
    // "proves nothing about execution" is a denial; refusing it punishes the exact
    // sentence the discipline asks for. Deliberately just the next word — scanning the
    // rest of the clause would excuse "proves that the file was not present", where the
    // negator governs the object clause and something is still claimed to be proved.
    // Only across SPACES. Any punctuation between is a clause boundary, and "The
    // defendant is liable; nothing further was examined" is a verdict followed by an
    // unrelated remark, not a denial.
    let next_word = haystack[at + len..]
        .trim_start_matches([' ', '\t'])
        .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '\u{2019}')
        .next()
        .unwrap_or_default();
    if OBJECT_NEGATORS.contains(&next_word) {
        return true;
    }

    let head = &haystack[..at];
    // A NEWLINE is a clause boundary, and the strongest one. Without it this reads a
    // whole rendered packet as ONE clause, so a negator lines away suppresses a verdict
    // printed under its own heading — which is exactly how a legal conclusion on a
    // limiter survived the body scan.
    // A FULL STOP IS A CLAUSE BOUNDARY, and its absence here was a defect of the same
    // family as the first-occurrence one: "It could not be established that X. The entry
    // proves execution" read the negator from the PREVIOUS SENTENCE and fell silent. A
    // sentence that has ended cannot govern the next one.
    let start = clause_start(haystack, at);

    head[start..]
        .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '\u{2019}')
        .any(|w| NEGATORS.contains(&w))
}

/// Whether the clause ending at `at` names a party.
///
/// `user` is on the party list and collides with technical names — "user hive", "user
/// profile". The collision is handled HERE rather than by deleting the word: a party
/// word immediately followed by a technical noun is naming a thing, not a person.
fn clause_has_party(haystack: &str, at: usize) -> bool {
    const TECHNICAL: &[&str] = &[
        "hive",
        "profile",
        "account",
        "key",
        "path",
        "registry",
        "folder",
        "directory",
        "file",
        "session",
        "space",
        "agent",
        "mode",
        "id",
        "name",
        "activity",
        "software",
        "process",
        "data",
        "input",
        "context",
        "token",
        "credential",
        "identifier",
    ];
    let start = clause_start(haystack, at);
    // WHOLE WORDS. This walked every alphanumeric index and took the word starting
    // there, so "the" contained "he" — a pronoun on the party list — and any sentence
    // with an article plus an ultimate-issue word read as a verdict about a person.
    // "The record admits an innocent explanation" is the example this file uses to
    // explain the rule, and it was refused by it.
    //
    // A hyphen ends a word here: "third-party" names software, not a party, and
    // "remote-access" is one compound noun rather than two words.
    let clause = &haystack[start..at];
    clause
        .split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '\u{2019}')
        .filter(|w| !w.is_empty())
        .filter(|w| PARTIES.contains(w))
        .any(|w| {
            // A party word immediately followed by a technical noun is naming a thing:
            // "user hive", "user profile", "user activity".
            let Some(at_word) = clause.rfind(w) else {
                return true;
            };
            let rest = clause[at_word + w.len()..].trim_start();
            let next = rest
                .split(|c: char| !c.is_alphanumeric())
                .find(|t| !t.is_empty())
                .unwrap_or_default();
            // And a HYPHEN binds tighter than a space: "third-party software" is a
            // compound naming a thing, whatever follows it.
            let hyphenated =
                clause[..at_word].ends_with('-') || clause[at_word + w.len()..].starts_with('-');
            !hyphenated && !TECHNICAL.contains(&next)
        })
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
    // "liable TO BE overwritten" is the passive, prone-to sense — ordinary English about
    // how evidence degrades. "liable TO PAY damages" is the legal one, and exempting
    // every "liable to " let it through. Only the passive construction is innocent, and
    // only where THIS occurrence sits: scanning the whole node let one unrelated
    // "liable to be" suppress every verdict in it.
    // PER OCCURRENCE, which is what the comment always claimed and the code did not do:
    // one "liable to be overwritten" suppressed every later "liable" in the same node,
    // including a real verdict. An occurrence is exempt only where IT is the passive,
    // prone-to construction; any other occurrence is judged on its own.
    if word == "liable" {
        // "liable TO <verb>" is the prone-to sense — ordinary English about how evidence
        // behaves ("liable to change at shutdown", "liable to be overwritten"). Matching
        // the literal "liable to be " alone caught the passive and refused the active
        // form of the same meaning, and a pronoun subject made it worse: "they are
        // liable to change", said of registry values, read as a verdict about people.
        //
        // The LEGAL sense is the closed set — "liable to pay", "liable to indemnify" —
        // while the ordinary verbs it must not swallow are an open class. Denying the
        // small closed set is the right way round; listing the open one is not.
        const OBLIGATION: &[&str] = &[
            "pay",
            "repay",
            "refund",
            "compensate",
            "indemnify",
            "reimburse",
            "contribute",
            "account",
        ];
        return occurrences(haystack, word).any(|at| {
            let prone = haystack[at..]
                .strip_prefix("liable to ")
                .is_some_and(|rest| {
                    let verb = rest
                        .split(|c: char| !c.is_alphanumeric())
                        .find(|t| !t.is_empty())
                        .unwrap_or_default();
                    !OBLIGATION.contains(&verb)
                });
            !prone && clause_has_party(haystack, at)
        });
    }
    // ANY occurrence said of a party makes it a verdict — the same reason
    // `clause_negated` looks at all of them rather than the first.
    occurrences(haystack, word).any(|at| clause_has_party(haystack, at))
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
/// Does the prose quantify over the world, and did the author say so?
///
/// Extracted from [`declaration_contradicted`] when that function outgrew its line
/// budget — split on the seam it already had, since the quantifier and causal halves
/// share nothing but the node.
const SCOPES: &[&str] = &[
    "report",
    "analysis",
    "document",
    "examination",
    "appendix",
    "annex",
    "section",
    "exhibit",
    "table",
    "schedule",
    "chapter",
    "paragraph",
    "figure",
];
// PHRASES, not bare words. `contains("below")` matched "Every host fell below the
// patch level" and switched the quantifier gate off — the same substring defect fixed
// in `clause_has_party`, left standing one function away. A word that means "elsewhere
// in this document" only means it in these forms.
const METADISCOURSE: &[&str] = &[
    "herein",
    "described below",
    "set out below",
    "listed below",
    "shown below",
    "stated below",
    "described above",
    "set out above",
    "stated above",
    "shown above",
    "as above",
];
// ...or an INSTRUCTION. "Always image the disk before every acquisition" is a
// procedure, and being universal is what makes it one rather than a suggestion.
// An instruction says what to DO; a claim says what IS.
//
// Detected in the TEXT, not by node kind. A kind test has been removed from this
// codebase three times, and skipping `Protocol` would let an author relabel a
// universal claim and escape — a protocol's title renders into any packet it
// supports. Same category-not-special-case shape as the metadiscourse carve-out
// above.
//
// Fails SAFE: an instruction the list misses is merely flagged, and the author
// sees why. A claim wrongly excused would not be.
const DEONTIC: &[&str] = &[
    " must ",
    " must not",
    " should ",
    " shall ",
    " ought to ",
    " is required to ",
];
const IMPERATIVE_OPENERS: &[&str] = &[
    "always ",
    "never ",
    "do not ",
    "verify ",
    "record ",
    "image ",
    "capture ",
    "document ",
    "acquire ",
    "photograph ",
    "seal ",
    "hash ",
    "ensure ",
    "confirm ",
    "label ",
    "store ",
    "avoid ",
    "check ",
];

/// A sentence that is not a claim about the world.
///
/// Three ways a universal can be found and ONE gate that exempts it — an instruction
/// ("Verify the hash on all acquired images"), a scope note ("All timestamps in this
/// appendix are UTC"), or plain metadiscourse. One function, because the third entry
/// point kept an inline copy that never learned about scope notes — directly under a
/// comment saying a rule with three entry points needs one gate.
fn is_exempt_from_quantifier(s: &str) -> bool {
    const BOUNDED: &[&str] = &[
        "none of the",
        "each of the",
        "all of the",
        "any of the",
        "every one of the",
    ];
    const PRONOMINAL: &[&str] = &[
        "none was",
        "none were",
        "none is",
        "none are",
        "none of them",
        "each was",
        "each were",
        "all was",
        "all were",
    ];
    // A quantifier that names its own domain has DECLARED its extension in the sentence:
    // "None of the recovered entries postdates the acquisition", "Each of the four hives
    // was examined". That is the bounded, scoped writing the discipline asks for, and
    // refusing it punishes the author for being specific.
    //
    // And a PRONOMINAL use quantifies nothing: in "the entry would have been expected;
    // none was present", `none` refers back to an antecedent — it is not a claim about
    // a class. The tell is that a verb follows rather than a noun.
    let t = s.trim_start();
    if BOUNDED.iter().any(|b| t.contains(b)) || PRONOMINAL.iter().any(|b| t.contains(b)) {
        return true;
    }
    IMPERATIVE_OPENERS.iter().any(|o| t.starts_with(o))
        || DEONTIC.iter().any(|d| t.contains(d))
        || SCOPES.iter().any(|w| {
            t.contains(&format!("in this {w}")) || t.contains(&format!("throughout this {w}"))
        })
        || METADISCOURSE.iter().any(|m| t.contains(m))
}

fn contradicted_quantifier(node: &Node) -> Vec<Violation> {
    // STRONG universals are attributions wherever they appear — "every host was
    // compromised by the account holder" is a universal claim in a body as much as in a
    // title. WEAK ones are scanned in the title only: "this node holds the pointer,
    // never the bytes" is method description, and firing on it broke this repository's
    // own clean fixture.
    // Body-scanned: assertive enough that appearing in prose is a claim about the
    // world. "all" and "each" were here and are not — "All timestamps in this report
    // are UTC" is a scope note, and refusing it punishes a careful author.
    const STRONG: &[&str] = &["every", "always"];
    const WEAK: &[&str] = &["all", "each", "any", "never", "none"];

    let title = node.title.to_ascii_lowercase();
    let full = format!("{} {}", node.title, node.body).to_ascii_lowercase();
    let mut out = Vec::new();

    // Fires on ABSENCE as well as on a false declaration. Declaring `singular` on a
    // universal sentence was caught; declaring nothing at all was not, and that is the
    // cheaper move — a constraint that activates on a field's presence is evaded by its
    // absence. Silence where an assertion was made is not neutral.
    let quantifier = node.field("quantifier");
    if quantifier != Some("universal") && quantifier != Some("class") {
        // A WEAK word is ordinary prose mid-sentence ("all of the entries were read")
        // and a universal claim when it OPENS one ("All systems in the estate show this
        // pattern"). Scanning weak words in the title only let the second escape by
        // moving one line down; scanning them everywhere fired on the first.
        // ...unless the sentence is about the DOCUMENT rather than the world. "All
        // timestamps in this report are UTC" is metadiscourse — a scope note telling the
        // reader how to read what follows — and blocking it punishes a careful author.
        // "All systems in the estate show this pattern" is a claim. The distinction is a
        // category, not a special case: statements about the artefact are not statements
        // about the subject matter.
        // METADISCOURSE is a CATEGORY — a note about where a statement binds — and it
        // was implemented as four phrasings. "All timestamps in this report are UTC" is
        // the carve-out's own motivating example; binding the same note to an appendix
        // or a table was refused, though it is the narrower and more careful claim.
        let in_a_claiming_sentence = |w: &str, hay: &str| {
            contains_phrase(hay, w)
                && hay
                    .split(['.', '!', '?', '\n'])
                    .filter(|s| contains_phrase(s, w))
                    .any(|s| !is_exempt_from_quantifier(s))
        };
        let found = STRONG
            .iter()
            .find(|w| in_a_claiming_sentence(w, &full))
            .or_else(|| WEAK.iter().find(|w| in_a_claiming_sentence(w, &title)))
            .or_else(|| {
                WEAK.iter().find(|w| {
                    full.split(['.', '!', '?', '\n']).any(|s| {
                        let s = s.trim_start();
                        s.starts_with(*w) && !is_exempt_from_quantifier(s)
                    })
                })
            });
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

    // Absence as well as false declaration, exactly as for `quantifier:`. "Deleting the
    // key CAUSED the artefact to disappear" with no `causal_rung:` drew nothing — the
    // rung gate never runs, so the strongest word in the sentence went unexamined.
    out
}

/// Does the prose claim about DOING, while the declared rung says only seeing?
fn contradicted_rung(node: &Node) -> Vec<Violation> {
    // Strong causal markers only. "produced" and "made" were tried and removed: both
    // are routine descriptive verbs in forensic writing — "the tool produced output",
    // "installation produced the record" — and flagging them punished a legitimate
    // rival hypothesis in this repository's own fixture. A heuristic that fires on
    // ordinary professional prose is one people switch off.
    // Two rungs above association, and the list had grammar for only one of them.
    // INTERVENTIONAL is the language of doing. COUNTERFACTUAL is the language of what
    // would have happened otherwise — rung three, the highest on the ladder, and it had
    // no marker at all, so a claim written in pure counterfactual grammar could declare
    // `association` and switch the ladder gate off unexamined.
    //
    // Both lists are FORMULAIC constructions rather than content words. "establishes"
    // is deliberately absent: it is the substitution table's own recommended replacement
    // for "proves", and flagging it would refuse the phrasing the discipline asks for.
    const INTERVENTIONAL: &[&str] = &[
        "caused",
        "causes",
        "causing",
        "resulted in",
        "led to",
        "because of",
    ];
    const COUNTERFACTUAL: &[&str] = &[
        "would not have",
        "had it not been",
        "but for",
        "could not have occurred without",
        "could not have happened without",
        "in the absence of which",
    ];
    let full = format!("{} {}", node.title, node.body).to_ascii_lowercase();
    let mut out = Vec::new();
    let rung = node.field("causal_rung");
    if rung.is_none() || rung == Some("association") {
        if let Some(w) = INTERVENTIONAL
            .iter()
            .chain(COUNTERFACTUAL.iter())
            // FULL text. "caused", "resulted in", "led to" are unambiguous claims about
            // doing wherever they sit — unlike "all", which is ordinary prose. Scanning
            // the title alone let the sentence move one line down and escape.
            .find(|w| contains_phrase(&full, w))
        {
            out.push(violation(
                DECLARATION_CONTRADICTED,
                &node.id,
                match rung {
                    Some(r) => format!(
                        "declares `causal_rung: {r}` but says \"{w}\" — that is a claim \
about doing, not about seeing"
                    ),
                    None => format!(
                        "says \"{w}\" and declares no `causal_rung:` — the ladder gate \
never runs, so a claim about DOING is examined as though it were about seeing"
                    ),
                },
                "raise the rung and satisfy it with an executed protocol, or state the \
association without the causal verb",
            ));
        }
    }
    out
}

fn declaration_contradicted(node: &Node) -> Vec<Violation> {
    let mut out = contradicted_quantifier(node);
    out.extend(contradicted_rung(node));
    out
}

/// Evidence resting on an instrument nobody has shown to work.
///
/// From `docs/method/source-register.md`: the failure a register exists for is a source
/// that answers SUCCESSFULLY with the wrong thing — a 200 carrying an empty body, a
/// filter matching a field that does not exist. The control that catches it is a
/// positive one: a query whose answer you already know. Until an instrument has fired
/// on a known positive, a null from it is an UNMEASURED result wearing a measurement's
/// clothes, and "no evidence of X" built on it is confident precisely because the search
/// found nothing.
///
/// Reported on the OBSERVATION, not the instrument: the observation is what cites it,
/// and its author is who can add the control or stop relying on the reading.
///
/// Silent where no `measured_by:` edge exists at all. Instrument provenance is not yet
/// required — demanding it everywhere would be ceremony, and this fires only for authors
/// who recorded an instrument and left it uncontrolled.
fn uncontrolled_instrument(graph: &Graph, node: &Node) -> Vec<Violation> {
    graph
        .edges_from(&node.id)
        .filter(|e| e.kind == EdgeKind::MeasuredBy)
        .filter_map(|e| graph.node(&e.to).map(|i| (e, i)))
        .filter(|(_, i)| i.field("positive_control").is_none())
        .map(|(_, i)| {
            violation(
                UNCONTROLLED_INSTRUMENT,
                &node.id,
                format!(
                    "\"{}\" was measured by `{}` ({}), which declares no `positive_control:` \
— nothing has shown this instrument fires when it should",
                    node.title, i.id, i.title
                ),
                "record a `positive_control:` on the instrument — a query whose answer is \
already known — or treat its nulls as unmeasured rather than as zero",
            )
        })
        .collect()
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
        .find(|e| e.kind.supersedes_target())
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
/// `does` is DELIBERATELY absent. It is an auxiliary, not a negator: every negating
/// form of it — "does not", "doesn't" — is already carried by `not` and `doesn't`, so
/// the bare word only ever read an emphatic affirmative as a denial. "The metadata does
/// show the respondent is liable" is as flat a verdict as the sentence without it.
const NEGATORS: &[&str] = &[
    "not",
    "never",
    "no",
    "nothing",
    "cannot",
    "doesn't",
    "don't",
    "isn't",
    "wasn't",
    "neither",
    "nor",
    "without",
    // CURLY APOSTROPHES. This project's own house style is curly quotes in prose, so the
    // spelling an author is told to use was the one that stopped negating: `wasn’t` was
    // refused where `wasn't` passed. A checker that refuses the house style is one the
    // house switches off.
    "doesn\u{2019}t",
    "don\u{2019}t",
    "isn\u{2019}t",
    "wasn\u{2019}t",
];

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
                    contains_phrase(&haystack, word)
                        && !every_occurrence_is_excused(&haystack, word)
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
/// An attack edge that the grounded extension will not honour.
///
/// A `term` or `criterion` is reference material — `is_argument` says so — so an attack
/// edge touching one is discarded from the relation. Discarding it SILENTLY is the
/// swallow this project forbids: the author wrote a move they believe is in play, and
/// the vault would go on quietly ignoring it. Say what was dropped, and why.
fn non_argument_attacks(graph: &Graph) -> Vec<Violation> {
    graph
        .edges()
        .filter(|e| e.kind.is_attack())
        .filter(|e| !graph.is_argument_node(&e.from) || !graph.is_argument_node(&e.to))
        .filter(|e| graph.node(&e.from).is_some() && graph.node(&e.to).is_some())
        .map(|e| {
            let (which, id) = if graph.is_argument_node(&e.from) {
                ("target", &e.to)
            } else {
                ("source", &e.from)
            };
            let kind = graph
                .node(id)
                .map_or_else(|| "?".to_owned(), |n| n.kind.to_string());
            violation(
                NON_ARGUMENT_ATTACK,
                &e.from,
                format!(
                    "`{}` attacks `{}`, but its {which} `{id}` is a `{kind}` — reference \
material is used by arguments and does not compete with them, so this edge is \
DISCARDED from the grounded extension",
                    e.from, e.to
                ),
                "attack from a claim, hypothesis, observation or dissent — or record the \
disagreement as a claim that cites the term, rather than as the term itself",
            )
        })
        .collect()
}

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
    // A KIND TEST DOING SCOPING, which this codebase has removed three times and had
    // grow back a fourth. Round 6 closed "an unexamined node manufactures standing" by
    // examining defenders — and one relabel evaded it, because the lint that makes a
    // bare defender FAIL keyed on `type: claim`. Spelled `type: hypothesis`, the same
    // vacuous rebuttal defeated a live rival and answered to nothing.
    //
    // A hypothesis nothing leans on is a legitimate candidate with no support — that is
    // what a hypothesis IS. A hypothesis being USED, as a weapon or as a support, is
    // being asserted, and an assertion resting on nothing is the thing this lint exists
    // to name. Whether a node carries weight is a property of the edges, not the kind.
    // A SMALLER KIND TEST IS STILL A KIND TEST. The previous version replaced
    // `node.kind != Claim` with `matches!(node.kind, Claim | Hypothesis)` — in the commit
    // whose message says weight is a property of the edges — so the same vacuous
    // rebuttal spelled `type: dissent` or `type: observation` restored a defeated claim
    // and drew nothing. `is_argument` admits four kinds; this admitted two.
    //
    // The question is USE, not kind: a node that attacks something, or that something
    // leans on, is being asserted. An OBSERVATION is the one exception, and for a
    // reason rather than by category — it is primitive evidence, the leaf a chain ends
    // at, so demanding it rest on something else is the regress D2 is about. But an
    // observation WIELDED as a weapon is making an argument, and answers for it.
    let wields_an_attack = graph
        .edges_from(&node.id)
        .any(|e| e.kind.is_attack() && graph.is_argument_node(&e.to));
    // OBSERVATIONS AND DISSENTS ARE LEAVES. Primitive evidence is where a chain ENDS;
    // demanding it rest on something else is the infinite regress. So merely carrying
    // weight is not enough for them — an observation supporting a claim is doing exactly
    // its job. Wielding an ATTACK is different: that is making an argument, and an
    // argument resting on nothing is what this lint names.
    //
    // A hypothesis sits between: something leaning on it makes it inferential rather
    // than primitive, so `carries_weight` counts there as round 6 established.
    let wielded = match node.kind {
        NodeKind::Claim => true,
        NodeKind::Hypothesis => wields_an_attack || carries_weight(graph, node),
        _ => wields_an_attack,
    };
    if !wielded {
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
    // DIRECTION. `X depends_on Y` is an edge from X to Y, so the OUTGOING direction
    // asks what this node leans on — the opposite question. What makes it load-bearing
    // is what leans on IT, and reading the wrong way meant a fully-groomed withdrawn
    // prerequisite reported nothing at all while a claim declaring it could not hold
    // without that prerequisite froze cleanly.
    // A DISCARDED attack holds nothing up. Counting it made withdrawing a REDUNDANT
    // defender — a node whose attack the argumentation never honoured — newly block the
    // packet it was not holding up, which punishes exactly the machloket retention this
    // project asks for: keep the rejected alternative on the record, and be refused for
    // having kept it.
    let holds_something_up = graph
        .edges_from(&node.id)
        .any(|e| e.kind == EdgeKind::Supports)
        || graph
            .edges_from(&node.id)
            .any(|e| e.kind.is_attack() && graph.is_argument_node(&e.to))
        || graph
            .edges_to(&node.id)
            .any(|e| e.kind == EdgeKind::DependsOn);
    if !holds_something_up {
        return Vec::new();
    }

    // `Graph::withdrawn()`, not a direct-edge test — the same fixed point court reads.
    // A retraction that has itself been retracted does not bind, and asking merely
    // whether an incoming retraction EXISTS blocked a live supporter forever with a
    // message the vault's own semantics call false.
    let withdrawn = graph.withdrawn();
    if !withdrawn.contains(&node.id) {
        return Vec::new();
    }

    graph
        .edges_to(&node.id)
        .filter(|e| e.kind.supersedes_target())
        .filter(|e| !withdrawn.contains(&e.from))
        .map(|e| {
            // Name the RIGHT verb. A sublation preserves what it replaces, so telling
            // an author their claim was "superseded" would misdescribe the record they
            // are being sent to read.
            let (verb, remedy) = match e.kind {
                EdgeKind::Retracts => (
                    "retracted",
                    "cite the retraction, or delete the claim — a packet frozen over a \
withdrawn claim is a conclusion the record itself retired",
                ),
                EdgeKind::Sublates => (
                    "sublated",
                    "cite the synthesis instead; it preserves this claim's content and \
replaces it as the current statement",
                ),
                _ => (
                    "superseded",
                    "cite the superseding version instead; the retired one is history, \
not a finding",
                ),
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
/// Claims always; anything else when something leans on it. Support AND `depends_on`
/// edges, because a declared prerequisite is the stronger relation of the two.
///
/// An earlier version of this comment said "claims only, and support edges only", and
/// it was false in three ways at once by the time anyone read it — the kind test had
/// gone, `depends_on` had been added in both directions, and each change left the
/// sentence behind. A comment describing scope is the first thing a scope change
/// invalidates.
fn ungraded_support(graph: &Graph, node: &Node) -> Vec<Violation> {
    // Claims, and hypotheses something LEANS ON — the same load-bearing rule the
    // promotion gates use. A node-kind test here let an ungraded inference edge hide
    // inside a chain: observation --graded--> h1 --UNGRADED--> h2 --graded--> claim
    // reported nothing, went review_ready, and froze. Every other check in this file
    // asks whether a node is carrying weight; this one asked what kind it was.
    // Incoming DependsOn counts too: `c1 --depends_on--> h1` means c1 cannot hold
    // without h1, so h1 is carrying c1's weight even though nothing "supports" anything
    // from h1. The edge that makes a node load-bearing can point either way, and only
    // one direction was checked.
    // The one definition, shared with the promotion gates. Grading needs no refinement
    // on top of it: whether an edge was graded is a property of the EDGE, so a claim
    // declaring `depends_on: [o3]` is exactly as ungraded whether o3 is an observation,
    // a run or a protocol.
    let carries_weight = crate::carries_weight(graph, node);
    if !carries_weight {
        return Vec::new();
    }
    graph
        .edges_to(&node.id)
        // DependsOn as well as Supports. `depends_on` is documented as "the target must
        // hold for the source to" — a STRONGER relation than support — so an ungraded
        // prerequisite is evidence nobody weighed, by the same argument and more so.
        .filter(|e| {
            matches!(e.kind, EdgeKind::Supports | EdgeKind::DependsOn) && e.grade().is_none()
        })
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

            // SHARED INSTRUMENT. Two readings from one tool are one line of evidence
            // however they are labelled, and G4 is defined as multiple materially
            // INDEPENDENT convergent lines. `duplicates:` catches only the author who
            // declares the overlap; this catches the overlap itself.
            // PRODUCING instruments only. A tool that MEASURED the substantive fact is
            // a shared line of evidence; one that merely verified or handled the
            // artifact is not. Two observations from different sources, each hashed by
            // the same verification tool, remain two lines — and refusing them punishes
            // an author for recording provenance, which is the discipline this whole
            // project asks for.
            //
            // Declared on the instrument, and ABSENCE FAILS SAFE: an instrument that
            // says nothing is treated as producing, so the finding still fires. Writing
            // `role: verifying` is an accountable act on the record, the same shape as
            // `no_terms_of_art:` — a claim the author makes and can be held to, rather
            // than a silence that buys an exemption.
            // THE LINEAGE, not the identity. Two supporters measured by different tools
            // that share an upstream are one line of evidence for anything the shared
            // layer gets wrong — two checkers vendoring the same dependency are not
            // independent for that layer, however differently they read.
            //
            // The walk starts at the supporter, follows `measured_by` to its
            // instruments, and then follows `measured_by` and `depends_on` BETWEEN
            // instruments. Restricted to instrument nodes on purpose: `depends_on` from
            // a claim is a prerequisite, an entirely different relation, and following
            // it here would make every claim sharing a premise "dependent".
            let instruments_of = |id: &NodeId| -> BTreeSet<NodeId> {
                let mut seen: BTreeSet<NodeId> = BTreeSet::new();
                let mut stack: Vec<NodeId> = graph
                    .edges_from(id)
                    .filter(|e| e.kind == EdgeKind::MeasuredBy)
                    .map(|e| e.to.clone())
                    .collect();
                while let Some(n) = stack.pop() {
                    let Some(node) = graph.node(&n) else { continue };
                    if node.kind != NodeKind::Instrument {
                        continue;
                    }
                    // A VERIFYING tool did not produce the finding, so it is not a shared
                    // line — and neither is anything it alone depends on.
                    if node
                        .field("role")
                        .is_some_and(|r| r.trim().eq_ignore_ascii_case("verifying"))
                    {
                        continue;
                    }
                    if !seen.insert(n.clone()) {
                        continue;
                    }
                    for e in graph
                        .edges_from(&n)
                        .filter(|e| matches!(e.kind, EdgeKind::MeasuredBy | EdgeKind::DependsOn))
                    {
                        stack.push(e.to.clone());
                    }
                }
                seen
            };
            let shared: Vec<NodeId> = instruments_of(a)
                .intersection(&instruments_of(b))
                .cloned()
                .collect();

            if !shared.is_empty() {
                let names = shared
                    .iter()
                    .map(|i| {
                        graph
                            .node(i)
                            .map_or_else(|| i.to_string(), |n| format!("{i} ({})", n.title))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push(violation(
                    FALSE_INDEPENDENCE,
                    &node.id,
                    format!(
                        "`{a}` and `{b}` both support \"{}\", and both were measured by {names} \
— one instrument is one line of evidence, not two",
                        node.title
                    ),
                    "cite them as a single line, or corroborate with evidence from an \
instrument that could fail differently — convergence between two runs of one tool is \
repetition, not independence",
                ));
            }

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
/// The lint pack, as DATA rather than a hand-written call list.
///
/// Fifteen `out.extend(...)` lines used to sit in `lint()`, and deleting any ONE of them
/// left the whole suite green — rustc would warn the function was now unused, and
/// nothing else would. A confidentiality control (`privilege_leak`) could be unwired in
/// a one-line diff that reads like tidying.
///
/// A table cannot be unwired without removing a row that NAMES the code it produces, and
/// `every_published_lint_is_wired_in` pins the rows against the published constants. The
/// wiring is the thing that kept being lost, so the wiring is what gets written down.
type GraphLint = fn(&Graph) -> Vec<Violation>;
type NodeLint = fn(&Graph, &Node) -> Vec<Violation>;

const GRAPH_LINTS: &[(&str, GraphLint)] = &[
    (DANGLING_EDGE, dangling_edges),
    (NON_ARGUMENT_ATTACK, non_argument_attacks),
    (UNREVIEWED_GRADE, unreviewed_grades),
    (SELF_GRADED, self_graded),
];

const NODE_LINTS: &[(&str, NodeLint)] = &[
    (FORBIDDEN_VERB, |_, n| forbidden_verbs(n)),
    (ORPHAN_CLAIM, orphan_claims),
    (PRIVILEGE_LEAK, |_, n| privilege_leak(n)),
    (FALSE_INDEPENDENCE, false_independence),
    (WINDOW_EDGE_AS_ONSET, window_edge_as_onset),
    (UNGROUNDED_CHAIN, ungrounded_chains),
    (RETRACTED, retracted),
    (UNGRADED_SUPPORT, ungraded_support),
    (UNCONTROLLED_INSTRUMENT, uncontrolled_instrument),
    (LEGAL_CONCLUSION, |_, n| legal_conclusions(n)),
    (DECLARATION_CONTRADICTED, |_, n| declaration_contradicted(n)),
];

pub fn lint(graph: &Graph) -> Vec<Violation> {
    let mut out = Vec::new();
    for (_, f) in GRAPH_LINTS {
        out.extend(f(graph));
    }
    for node in graph.nodes() {
        for (_, f) in NODE_LINTS {
            out.extend(f(graph, node));
        }
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
    /// A discarded edge is reported, never swallowed.
    ///
    /// The grounded extension drops an attack whose source or target is reference
    /// material, because reference material does not compete. The author wrote a move
    /// they believe is in play; leaving it silently ignored is the swallow this project
    /// forbids, and it hides the reason their claim did not fall.
    #[test]
    fn an_attack_from_reference_material_is_reported() {
        let g = graph_of(
            vec![
                node("---\nid: c1\ntype: claim\ntitle: A catalogue entry was recorded\n---\n"),
                node(
                    "---\nid: 60.01\ntype: term\ntitle: presence\nas_used: on the system\n\
not_essence: a record is not the file\nstipulated: the OS recorded this path\n---\n",
                ),
            ],
            vec![Edge::new(
                NodeId::new("60.01"),
                NodeId::new("c1"),
                EdgeKind::Contradicts,
            )],
        );
        let found: Vec<Violation> = lint(&g)
            .into_iter()
            .filter(|v| v.gate == NON_ARGUMENT_ATTACK)
            .collect();
        assert_eq!(found.len(), 1, "the discarded edge must be named");
        assert!(
            found[0].detail.contains("term") && found[0].detail.contains("DISCARDED"),
            "and it must say what was dropped and why: {}",
            found[0].detail
        );
    }

    /// Eight sentences a careful expert writes, and the two that must still fire.
    ///
    /// Every row is a finding from audit round 7. The four "clean" groups are prose the
    /// discipline asks for; the "fires" rows are the controls that prove the carve-outs
    /// did not simply switch the checks off.
    #[test]
    fn the_prose_an_expert_is_obliged_to_write() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| {
                    v.gate == LEGAL_CONCLUSION
                        || v.gate == FORBIDDEN_VERB
                        || v.gate == DECLARATION_CONTRADICTED
                })
                .count()
        };

        for clean in [
            // N8 — the remedy this tool itself recommends
            "Whether the respondent is liable is a matter for the court",
            "I express no view on whether the respondent is liable",
            // N11 — a scope condition is not causal grammar
            "The record is admissible only if the hive was acquired intact",
            // N12/N13 — bounded and pronominal, not universal
            "None of the recovered entries postdates the acquisition",
            "Each of the four hives was examined",
            "The entry would have been expected; none was present",
            // N14 — an integrity operation, not an opinion
            "Hash verification confirmed the image digest",
            // N10 — a genuine scope note
            "All timestamps described below are stated in UTC",
        ] {
            assert_eq!(
                fired(clean),
                0,
                "refused prose an expert is obliged to write: {clean}"
            );
        }

        for verdict in [
            // N10 — "below" as a comparative must not buy an exemption
            "Every host fell below the patch level",
            // N9 — an imperative plus `!` must not hide the universal after it
            "Always verify the hash first! Every host on the estate ran the installer",
            "The respondent is liable for the loss",
            "The evidence confirms the program was executed",
            "The respondent is liable; that is a matter for the court",
        ] {
            assert!(fired(verdict) > 0, "let a real finding through: {verdict}");
        }
    }

    /// An exemption is per OCCURRENCE and per CLAUSE, never per document.
    ///
    /// `is_a_verification_operation` was handed the whole haystack — its parameter is
    /// named `clause` — so one "digest" anywhere in a node, or anywhere in a rendered
    /// packet body, switched the entire overstatement scan off. Every acquisition note
    /// in forensics contains that word and this repository's own fixture renders
    /// "SHA-256 digest H" into packets, so the check was off by default on exactly the
    /// material it exists for.
    ///
    /// And the hand-back scan used a SMALLER forward boundary set than `clause_start`
    /// uses backwards, so a verdict laundered by appending the tool's own remedy.
    /// `CLAUSE_BOUNDS` is one set now, read in both directions.
    #[test]
    fn an_exemption_does_not_reach_beyond_its_clause() {
        let fired = |title: &str, body: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n\n{body}\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == FORBIDDEN_VERB || v.gate == LEGAL_CONCLUSION)
                .count()
        };

        // A1 — a verification word elsewhere must not excuse the verdict.
        assert_eq!(
            fired("The Amcache entry proves execution of the binary", ""),
            1,
            "positive control"
        );
        assert_eq!(
            fired(
                "The Amcache entry proves execution of the binary",
                "The acquired image has SHA-256 digest H."
            ),
            1,
            "an unrelated acquisition sentence must not disarm the scan"
        );
        assert_eq!(
            fired("Hash verification confirmed the image digest", ""),
            0,
            "and a genuine verification report is still not an opinion"
        );

        // A2 — the forward boundary set must match the backward one.
        assert_eq!(
            fired(
                "The respondent is guilty of fraud and whether costs follow is a matter \
for the court",
                ""
            ),
            1,
            "a hand-back in a LATER clause does not cover the verdict in this one"
        );
        assert_eq!(
            fired(
                "Whether the respondent is liable is a matter for the court",
                ""
            ),
            0,
            "control: the genuine hand-back still passes"
        );
    }

    /// A frame governs its own clause, and no further.
    ///
    /// An attribution or a hedge opener reaching to the next full stop let the author
    /// resume their own voice after a semicolon or a conjunction and keep the exemption
    /// — every carve-out added to stop a false positive is, by construction, a hole
    /// somebody can write through, and this was the hole.
    #[test]
    fn a_frame_does_not_cover_the_authors_resumed_voice() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == LEGAL_CONCLUSION || v.gate == FORBIDDEN_VERB)
                .count()
        };

        assert_eq!(
            fired("It is alleged that the respondent forged the entry"),
            0,
            "control: reciting the allegation is a duty, not a finding"
        );
        assert_eq!(
            fired(
                "It is alleged that entries were altered; my own analysis shows the \
respondent forged them"
            ),
            1,
            "after the semicolon the author is speaking, and the frame does not reach"
        );
        assert_eq!(
            fired("It is not possible to say when, but the respondent forged the ledger entry"),
            1,
            "nor past a `but` — the hedge declines, then the sentence asserts anyway"
        );
        assert_eq!(
            fired("In my opinion, it cannot be said that the entry confirms execution"),
            0,
            "and a hedge need not OPEN the sentence to govern its clause"
        );
        assert_eq!(
            fired("I have been instructed that the respondent forged the entry"),
            0,
            "the commonest CPR 35 recital form"
        );
    }

    /// The house style must not be the spelling that breaks the checker.
    ///
    /// Curly quotes are this project's own written convention, and `wasn’t` stopped
    /// negating while `wasn't` passed — so the careful author using the prescribed
    /// punctuation was refused and the one ignoring it was not.
    #[test]
    fn a_curly_apostrophe_still_negates() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == LEGAL_CONCLUSION)
                .count()
        };
        assert_eq!(
            fired("The record wasn't evidence that the respondent is liable"),
            0,
            "control: the ASCII spelling always worked"
        );
        assert_eq!(
            fired("The record wasn\u{2019}t evidence that the respondent is liable"),
            0,
            "and the curly one is the same sentence"
        );
        assert_eq!(
            fired("The record is evidence that the respondent is liable"),
            1,
            "control: without the negator it is a verdict either way"
        );
    }

    /// An em dash starts a new clause.
    #[test]
    fn an_em_dash_is_a_clause_boundary() {
        let g = graph_of(
            vec![node(
                "---\nid: n1\ntype: observation\ntitle: The record proves no innocent \
explanation exists — the respondent forged the entries\n---\n",
            )],
            vec![],
        );
        assert!(
            lint(&g).iter().any(|v| v.gate == LEGAL_CONCLUSION),
            "the negator belongs to the clause before the dash, not the verdict after it"
        );
    }

    /// Withdrawing a node that was holding nothing up must not block anything.
    ///
    /// A defender whose attack the argumentation DISCARDS is doing no work. Counting
    /// that discarded edge as "holds something up" meant retiring the node newly blocked
    /// a packet it had never held up — punishing exactly the machloket retention this
    /// project asks for: keep the rejected alternative on the record, and be refused for
    /// having kept it.
    #[test]
    fn withdrawing_a_redundant_node_blocks_nothing() {
        let build = |attacked_kind: &str| {
            let g = graph_of(
                vec![
                    node("---\nid: c1\ntype: claim\ntitle: The subject\n---\n"),
                    node("---\nid: o1\ntype: observation\ntitle: a record\n---\n"),
                    node(&format!(
                        "---\nid: t1\ntype: {attacked_kind}\ntitle: the thing attacked\n\
as_used: a\nnot_essence: b\nstipulated: c\n---\n"
                    )),
                    node("---\nid: n1\ntype: dissent\ntitle: a retired objection\n---\n"),
                    node("---\nid: d1\ntype: dissent\ntitle: it was withdrawn\n---\n"),
                ],
                vec![
                    Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports),
                    Edge::new(NodeId::new("n1"), NodeId::new("t1"), EdgeKind::Attacks),
                    Edge::new(NodeId::new("d1"), NodeId::new("n1"), EdgeKind::Retracts),
                ],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == RETRACTED && v.subject == NodeId::new("n1"))
                .count()
        };

        assert_eq!(
            build("claim"),
            1,
            "positive control: a withdrawn node whose attack COUNTS is reported"
        );
        assert_eq!(
            build("term"),
            0,
            "its attack on reference material was never honoured, so retiring it \
holds nothing up and blocks nothing"
        );
    }

    /// Every published lint code is wired into the pack.
    ///
    /// The pack used to be fifteen `out.extend(...)` lines, and deleting any ONE left
    /// the whole suite green. Four of them were demonstrated in audit round 8:
    /// `unreviewed_grades`, `privilege_leak`, `dangling_edges` and the sealed-prose loop
    /// could each be unwired in a one-line diff that reads like tidying — rustc would
    /// warn the function was unused and nothing else would notice. A confidentiality
    /// control among them.
    ///
    /// Fixing the four instances was not the fix. This pins the WIRING against the
    /// published constants: a code the crate publishes and the pack does not run is a
    /// rule the vault believes it has.
    #[test]
    fn every_published_lint_is_wired_in() {
        // The published surface, written down independently of the tables — a list
        // derived FROM the tables could not detect a row being deleted from them.
        const PUBLISHED: &[&str] = &[
            FORBIDDEN_VERB,
            DANGLING_EDGE,
            ORPHAN_CLAIM,
            UNREVIEWED_GRADE,
            PRIVILEGE_LEAK,
            FALSE_INDEPENDENCE,
            SELF_GRADED,
            WINDOW_EDGE_AS_ONSET,
            UNGROUNDED_CHAIN,
            RETRACTED,
            UNCONTROLLED_INSTRUMENT,
            NON_ARGUMENT_ATTACK,
            UNGRADED_SUPPORT,
            LEGAL_CONCLUSION,
            DECLARATION_CONTRADICTED,
        ];

        let mut wired: Vec<&str> = GRAPH_LINTS.iter().map(|(c, _)| *c).collect();
        wired.extend(NODE_LINTS.iter().map(|(c, _)| *c));

        for code in PUBLISHED {
            assert!(
                wired.contains(code),
                "`{code}` is published by this crate and run by nothing — a rule the \
vault believes it has"
            );
        }
        assert_eq!(
            wired.len(),
            PUBLISHED.len(),
            "the pack runs {} lints and {} codes are published; a row without a constant \
is as wrong as a constant without a row",
            wired.len(),
            PUBLISHED.len()
        );
    }

    /// A smaller kind test is still a kind test.
    ///
    /// The previous fix replaced `node.kind != Claim` with
    /// `matches!(node.kind, Claim | Hypothesis)` — in the commit whose own message says
    /// weight is a property of the edges — so the same vacuous rebuttal spelled
    /// `type: dissent` or `type: observation` restored a defeated claim and drew nothing.
    /// `is_argument` admits four kinds; that admitted two.
    ///
    /// The exemption that remains is stated as a REASON, not a category: an observation
    /// or dissent is primitive evidence, the leaf a chain ends at, so merely being leaned
    /// on cannot oblige it to rest on something else — that is the regress. Wielding an
    /// ATTACK is different: it is making an argument.
    #[test]
    fn a_bare_attacker_answers_for_itself_whatever_it_is_labelled() {
        let orphans = |kind: &str, attacks: bool| {
            let mut nodes = vec![
                node("---\nid: c1\ntype: claim\ntitle: The subject\n---\n"),
                node("---\nid: o1\ntype: observation\ntitle: a record\n---\n"),
                node(&format!(
                    "---\nid: x1\ntype: {kind}\ntitle: an account that does not fit\n---\n"
                )),
            ];
            nodes.push(node("---\nid: pad\ntype: observation\ntitle: pad\n---\n"));
            let mut edges = vec![Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )];
            if attacks {
                edges.push(Edge::new(
                    NodeId::new("x1"),
                    NodeId::new("c1"),
                    EdgeKind::Attacks,
                ));
            } else {
                edges.push(Edge::new(
                    NodeId::new("x1"),
                    NodeId::new("c1"),
                    EdgeKind::Supports,
                ));
            }
            lint(&graph_of(nodes, edges))
                .into_iter()
                .filter(|v| v.gate == ORPHAN_CLAIM && v.subject == NodeId::new("x1"))
                .count()
        };

        for kind in ["claim", "hypothesis", "dissent", "observation"] {
            assert_eq!(
                orphans(kind, true),
                1,
                "{kind}: a bare node wielding an attack is making an argument on nothing"
            );
        }
        assert_eq!(
            orphans("observation", false),
            0,
            "an observation SUPPORTING a claim is a leaf doing its job, not an orphan"
        );
        assert_eq!(
            orphans("dissent", false),
            0,
            "and so is a dissent that attacks nothing"
        );
    }

    /// A hypothesis being WIELDED is being asserted.
    ///
    /// Round 6 closed "an unexamined node manufactures standing" by examining defenders.
    /// One relabel evaded it: the lint that makes a bare defender FAIL keyed on
    /// `type: claim`, so the same vacuous rebuttal spelled `type: hypothesis` defeated a
    /// live rival and answered to nothing. A kind test doing scoping — removed three
    /// times from this codebase and grown back a fourth.
    ///
    /// The distinction that matters is use, not kind: a hypothesis nothing leans on is a
    /// legitimate candidate with no support, which is what a hypothesis IS.
    #[test]
    fn a_hypothesis_used_as_a_weapon_answers_for_itself() {
        let build = |kind: &str, attacks: bool| {
            let mut nodes = vec![
                node("---\nid: c1\ntype: claim\ntitle: The rival account\n---\n"),
                node("---\nid: o1\ntype: observation\ntitle: a record\n---\n"),
                node(&format!(
                    "---\nid: h1\ntype: {kind}\ntitle: An account that does not fit\n---\n"
                )),
            ];
            nodes.push(node("---\nid: pad\ntype: observation\ntitle: pad\n---\n"));
            let mut edges = vec![Edge::new(
                NodeId::new("o1"),
                NodeId::new("c1"),
                EdgeKind::Supports,
            )];
            if attacks {
                edges.push(Edge::new(
                    NodeId::new("h1"),
                    NodeId::new("c1"),
                    EdgeKind::Attacks,
                ));
            }
            lint(&graph_of(nodes, edges))
                .into_iter()
                .filter(|v| v.gate == ORPHAN_CLAIM && v.subject == NodeId::new("h1"))
                .count()
        };

        assert_eq!(
            build("hypothesis", false),
            0,
            "a hypothesis nothing leans on is a candidate, not an orphan"
        );
        assert_eq!(
            build("hypothesis", true),
            1,
            "one used to defeat something is being asserted, and rests on nothing"
        );
        assert_eq!(
            build("claim", true),
            1,
            "control: the spelling that always fired still fires"
        );
    }

    /// One definition of where a clause begins, and a comma is not it.
    ///
    /// Two boundary lists existed and disagreed in BOTH directions: `clause_negated`
    /// treated `.`/`!`/`?` as boundaries and `clause_has_party` did not, so a party
    /// named in a previous sentence leaked forward — and both cut at a bare COMMA,
    /// which severs a parenthetical from what it modifies. A comma is punctuation
    /// inside a clause, not the start of one.
    #[test]
    fn a_parenthetical_does_not_sever_a_clause() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == LEGAL_CONCLUSION)
                .count()
        };

        assert_eq!(
            fired("The defendant is guilty of fraud"),
            1,
            "positive control"
        );
        assert_eq!(
            fired("The defendant is, on this evidence, guilty of fraud"),
            1,
            "an aside between commas must not hide the subject the verdict is about"
        );
        assert_eq!(
            fired("There is no evidence, in the material examined, that the respondent is liable"),
            0,
            "and the same aside must not sever the negator from what it governs"
        );
        assert_eq!(
            fired("The respondent was interviewed. The record admits an innocent explanation"),
            0,
            "a party named in the PREVIOUS sentence does not make this one a verdict"
        );
    }

    /// A party word must be a WORD, not a substring.
    ///
    /// `clause_has_party` walked every alphanumeric index and took the word starting
    /// there, so "the" contained "he" — a pronoun on the party list. Any sentence
    /// carrying an article and an ultimate-issue word read as a verdict about a person,
    /// including this file's own documented example of what must never fire: "an
    /// innocent explanation" is a thing, and the comment beside `NEEDS_A_PERSON` says so.
    #[test]
    fn a_party_word_must_be_a_whole_word() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == LEGAL_CONCLUSION)
                .count()
        };

        assert_eq!(
            fired("The respondent is liable for the loss"),
            1,
            "positive control: a verdict about a party still fires"
        );
        assert_eq!(
            fired("The record admits an innocent explanation"),
            0,
            "`the` is not `he` — and this is the file's own example of a thing, not a person"
        );
        assert_eq!(
            fired("The user activity admits an innocent explanation"),
            0,
            "nor does an activity become a person"
        );
        assert_eq!(
            fired("Third-party remote-access software admits an innocent explanation"),
            0,
            "a hyphenated compound naming software is not a party"
        );
    }

    /// Attributed speech is not the author's assertion.
    ///
    /// Reciting the allegation from the instructions is a CPR 35 duty, and quoting the
    /// contention being rebutted is how a rebuttal is written. Both were read as the
    /// expert's own verdict — the tool refusing the two sentences its own discipline
    /// requires.
    #[test]
    fn attributed_speech_is_not_the_authors_verdict() {
        let fired = |body: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: A record was examined\n---\n\n{body}\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == LEGAL_CONCLUSION || v.gate == FORBIDDEN_VERB)
                .count()
        };

        assert_eq!(
            fired("The respondent forged the entry."),
            1,
            "positive control: an unattributed verdict is the author's own"
        );
        assert_eq!(
            fired("I am instructed that the respondent forged the entry."),
            0,
            "reciting the instructions is a duty, not a finding"
        );
        assert_eq!(
            fired("The opposing report asserts that the entry proves execution."),
            0,
            "quoting the contention being rebutted is how a rebuttal is written"
        );
        assert_eq!(
            fired("It is alleged that the respondent forged the entry."),
            0,
            "an allegation recited is not an allegation adopted"
        );
    }

    /// A hedge may be longer than the clause its verb sits in.
    ///
    /// "It has not been possible, on the material provided, to say that the entry
    /// confirms execution" is a refusal to conclude, and the clause carrying "confirms"
    /// holds no negator — the negation is in the matrix two clauses back.
    #[test]
    fn a_sentence_level_hedge_negates_what_follows_it() {
        let fired = |body: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: A record was examined\n---\n\n{body}\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == FORBIDDEN_VERB)
                .count()
        };

        assert_eq!(
            fired("The entry confirms execution."),
            1,
            "positive control"
        );
        assert_eq!(
            fired(
                "It has not been possible, on the material provided, to say that the entry \
confirms execution."
            ),
            0,
            "a refusal to conclude, and the negation sits two clauses from the verb"
        );
        assert_eq!(
            fired("It could not be established that the entry confirms execution."),
            0,
            "the same shape, and the phrasing the discipline asks for"
        );
    }

    /// A packet may not freeze over a claim the record withdraws — and nothing tested it.
    ///
    /// `subject_withdrawn` has one production caller, in court, and no test anywhere.
    /// Neutering it with `if true ||` left the entire suite green while a packet froze
    /// over a claim the vault says was retracted. A rule with no failing test is a rule
    /// you believe you have.
    #[test]
    fn a_withdrawn_subject_is_reported() {
        let build = |lift: bool| {
            let mut nodes = vec![
                node("---\nid: c1\ntype: claim\ntitle: The original finding\n---\n"),
                node("---\nid: d1\ntype: dissent\ntitle: it is withdrawn\n---\n"),
            ];
            let mut edges = vec![Edge::new(
                NodeId::new("d1"),
                NodeId::new("c1"),
                EdgeKind::Retracts,
            )];
            if lift {
                nodes.push(node(
                    "---\nid: d2\ntype: dissent\ntitle: that withdrawal is itself withdrawn\n---\n",
                ));
                edges.push(Edge::new(
                    NodeId::new("d2"),
                    NodeId::new("d1"),
                    EdgeKind::Retracts,
                ));
            }
            let g = graph_of(nodes, edges);
            subject_withdrawn(&g, &NodeId::new("c1")).is_some()
        };

        assert!(
            build(false),
            "the record withdraws c1, and a packet must not freeze"
        );
        assert!(
            !build(true),
            "the withdrawal was lifted, so c1 stands — the fixed point, not a direct edge"
        );
    }

    /// The ladder has three rungs and the backstop had grammar for two.
    ///
    /// `contradicted_rung` catches a claim that declares `association` and writes the
    /// language of DOING. It had no marker for the rung above that — the language of
    /// what would have happened otherwise — so a claim written in pure counterfactual
    /// grammar declared `association` and switched the gate off unexamined.
    ///
    /// "establishes" stays absent on purpose: it is the substitution table's own
    /// recommended replacement for "proves", and flagging it would refuse the phrasing
    /// the discipline asks for.
    #[test]
    fn counterfactual_grammar_contradicts_a_declared_association() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: {title}\ncausal_rung: association\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == DECLARATION_CONTRADICTED)
                .count()
        };

        assert_eq!(
            fired("Deleting the file caused the loss of evidence"),
            1,
            "positive control: the language of doing"
        );
        assert_eq!(
            fired("The record would not have been written but for the program running"),
            1,
            "the language of what would have happened otherwise is a rung higher still"
        );
        assert_eq!(
            fired("The entry establishes that the path was catalogued"),
            0,
            "`establishes` is the discipline's own sanctioned word and must stay unflagged"
        );
        assert_eq!(
            fired("The record is consistent with the path having been catalogued"),
            0,
            "and ordinary association prose passes"
        );
    }

    /// An oracle is independent only if its LINEAGE is.
    ///
    /// Two supporters measured by the same tool are one line of evidence, and that is
    /// caught. Two supporters measured by DIFFERENT tools that share an upstream are
    /// also one line — two checkers vendoring the same dependency are not independent
    /// for the shared layer, however differently they read. The blind spot is inherited
    /// with the ancestry.
    ///
    /// This is the half of the two-method rule the instrument register was built for and
    /// never reached: it compared instruments by identity, so a shared PARENT was
    /// invisible.
    #[test]
    fn independence_is_a_property_of_the_lineage() {
        let build = |shared_parent: bool| {
            let mut nodes = vec![
                node("---\nid: c1\ntype: claim\ntitle: The path was recorded\n---\n"),
                node("---\nid: o1\ntype: observation\ntitle: the hive entry\n---\n"),
                node("---\nid: o2\ntype: observation\ntitle: the MFT record\n---\n"),
                node("---\nid: t1\ntype: instrument\ntitle: hive parser\n---\n"),
                node("---\nid: t2\ntype: instrument\ntitle: MFT parser\n---\n"),
                node(
                    "---\nid: lib\ntype: instrument\ntitle: the shared decoder both vendor\n---\n",
                ),
            ];
            nodes.push(node("---\nid: pad\ntype: observation\ntitle: pad\n---\n"));
            let mut edges = vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports),
                Edge::new(NodeId::new("o2"), NodeId::new("c1"), EdgeKind::Supports),
                Edge::new(NodeId::new("o1"), NodeId::new("t1"), EdgeKind::MeasuredBy),
                Edge::new(NodeId::new("o2"), NodeId::new("t2"), EdgeKind::MeasuredBy),
            ];
            if shared_parent {
                edges.push(Edge::new(
                    NodeId::new("t1"),
                    NodeId::new("lib"),
                    EdgeKind::DependsOn,
                ));
                edges.push(Edge::new(
                    NodeId::new("t2"),
                    NodeId::new("lib"),
                    EdgeKind::DependsOn,
                ));
            }
            lint(&graph_of(nodes, edges))
                .into_iter()
                .filter(|v| v.gate == FALSE_INDEPENDENCE)
                .collect::<Vec<_>>()
        };

        assert!(
            build(false).is_empty(),
            "control: two genuinely separate tools are two lines of evidence"
        );
        let shared = build(true);
        assert_eq!(
            shared.len(),
            1,
            "two parsers vendoring one decoder are not independent for the shared layer"
        );
        assert!(
            shared[0].detail.contains("lib"),
            "and the finding must NAME the shared ancestor, or it cannot be checked: {}",
            shared[0].detail
        );
    }

    /// Recording provenance must not cost an author their independence.
    ///
    /// Two observations from different sources, each verified with the same hash tool,
    /// are still two lines of evidence — the tool did not produce either finding. The
    /// lint read every `measured_by:` alike, so documenting the verification collapsed
    /// them into one and the careful author was worse off than the silent one.
    ///
    /// The exemption is DECLARED and fails safe on absence: an instrument that says
    /// nothing is treated as producing.
    #[test]
    fn a_verifying_instrument_does_not_collapse_two_lines() {
        let build = |role: &str| {
            let g = graph_of(
                vec![
                    node("---\nid: c1\ntype: claim\ntitle: The path was recorded\n---\n"),
                    node("---\nid: o1\ntype: observation\ntitle: the hive entry\n---\n"),
                    node("---\nid: o2\ntype: observation\ntitle: the MFT record\n---\n"),
                    node(&format!(
                        "---\nid: t1\ntype: instrument\ntitle: a hash verification tool\n{role}---\n"
                    )),
                ],
                vec![
                    Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports),
                    Edge::new(NodeId::new("o2"), NodeId::new("c1"), EdgeKind::Supports),
                    Edge::new(NodeId::new("o1"), NodeId::new("t1"), EdgeKind::MeasuredBy),
                    Edge::new(NodeId::new("o2"), NodeId::new("t1"), EdgeKind::MeasuredBy),
                ],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == FALSE_INDEPENDENCE)
                .count()
        };

        assert_eq!(
            build(""),
            1,
            "absence fails safe: an instrument that declares no role is a producing one"
        );
        assert_eq!(
            build("role: producing\n"),
            1,
            "and saying so plainly changes nothing"
        );
        assert_eq!(
            build("role: verifying\n"),
            0,
            "a tool that checked the artifact did not produce either finding"
        );
    }

    /// `sublates` is a spelling of "supersedes", and it was read by nothing.
    ///
    /// Its own docstring is "preserves the target while SUPERSEDING it". `Supersedes`
    /// drives the withdrawal fixed point, the standing line and this lint; `Sublates`
    /// was parsed, listed as a known kind, and consumed nowhere — so the identical
    /// statement sealed in silence under one spelling and was refused under the other.
    #[test]
    fn sublation_retires_a_claim_like_supersession() {
        let build = |kind: EdgeKind| {
            let g = graph_of(
                vec![
                    node("---\nid: c1\ntype: claim\ntitle: The original finding\n---\n"),
                    node("---\nid: c2\ntype: claim\ntitle: The synthesis\n---\n"),
                    node("---\nid: o1\ntype: observation\ntitle: a record\n---\n"),
                ],
                vec![
                    Edge::new(NodeId::new("c1"), NodeId::new("o1"), EdgeKind::Supports),
                    Edge::new(NodeId::new("c2"), NodeId::new("c1"), kind),
                ],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == RETRACTED)
                .collect::<Vec<_>>()
        };

        let superseded = build(EdgeKind::Supersedes);
        assert_eq!(superseded.len(), 1, "positive control");

        let sublated = build(EdgeKind::Sublates);
        assert_eq!(
            sublated.len(),
            1,
            "the same lifecycle claim, spelled the other way"
        );
        assert!(
            sublated[0].detail.contains("sublated"),
            "and named for what it is — a sublation preserves what it replaces, so \
calling it superseded misdescribes the record: {}",
            sublated[0].detail
        );
    }

    /// "liable to change" is how evidence behaves, not what a party owes.
    ///
    /// The exemption matched the literal "liable to be " and so refused the active form
    /// of the same meaning. A pronoun subject made it worse: "they are liable to
    /// change", said of registry values, read as a verdict about people. The LEGAL
    /// sense is a closed set; the ordinary verbs are an open class, so the closed one
    /// is what gets listed.
    #[test]
    fn liable_to_change_is_volatility_not_a_verdict() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == LEGAL_CONCLUSION)
                .count()
        };

        assert_eq!(
            fired("The respondent is liable for the loss"),
            1,
            "positive control: the bare legal sense"
        );
        assert_eq!(
            fired("The respondent is liable to pay damages"),
            1,
            "and the obligation sense, which is what the closed list keeps"
        );
        assert_eq!(
            fired("Registry values are cached; they are liable to change at shutdown"),
            0,
            "said of registry values, however the pronoun reads"
        );
        assert_eq!(
            fired("These entries are liable to be overwritten"),
            0,
            "the passive form that always worked"
        );
    }

    /// A sentence that has ended cannot govern the next one.
    ///
    /// Found by probing the carve-outs added for the false positives above, not by
    /// either audit lineage: the clause boundaries listed commas, semicolons and
    /// conjunctions and no FULL STOP, so a negator in the previous sentence silenced a
    /// verdict in this one. Same family as the first-occurrence defect — a negation
    /// reaching further than it governs.
    #[test]
    fn a_negator_does_not_reach_across_a_full_stop() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: n1\ntype: observation\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == FORBIDDEN_VERB)
                .count()
        };

        assert_eq!(
            fired("The record is not conclusive. The entry proves execution"),
            1,
            "the hedge belongs to the previous sentence; this one is a flat assertion"
        );
        assert_eq!(
            fired("It could not be established that X. The entry proves execution"),
            1,
            "and a recognised hedge OPENER governs its own sentence, not the one after it"
        );
        assert_eq!(
            fired("The entry does not prove execution"),
            0,
            "control: negation inside the clause still works"
        );
    }

    /// Metadiscourse is a category, not four phrasings.
    ///
    /// The carve-out exists so "All timestamps in this report are UTC" is read as a
    /// scope note rather than a universal claim about the world. It listed whole-document
    /// words only, so binding the same note to an appendix or a table refused it.
    #[test]
    fn a_scope_note_may_name_the_part_it_binds() {
        let fired = |title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: {title}\nquantifier: singular\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-DECLARATION-CONTRADICTED")
                .count()
        };

        assert_eq!(
            fired("All hosts on the estate ran the installer"),
            1,
            "positive control: a real universal against a singular declaration"
        );
        assert_eq!(
            fired("All timestamps in this report are stated in UTC"),
            0,
            "the carve-out's own motivating example"
        );
        assert_eq!(
            fired("All timestamps in this appendix are stated in UTC"),
            0,
            "and the same note bound to a part of the document"
        );
        assert_eq!(
            fired("All figures in this table are stated to the cent"),
            0,
            "or to a table"
        );
    }

    /// A retraction that has itself been retracted does not bind.
    ///
    /// `Graph::withdrawn()` is a fixed point precisely because retractions can be
    /// lifted. The lint asked the pre-fix question — "does an incoming retraction
    /// exist" — so a supporter whose withdrawal was lifted blocked its claim forever,
    /// with a message the vault's own semantics call false. Third copy of one fix.
    #[test]
    fn a_lifted_retraction_does_not_block() {
        let build = |lift: bool| {
            let mut nodes = vec![
                node("---\nid: c1\ntype: claim\ntitle: A catalogue entry was recorded\n---\n"),
                node("---\nid: o1\ntype: observation\ntitle: the entry is present\n---\n"),
                node("---\nid: d1\ntype: dissent\ntitle: the observation was withdrawn\n---\n"),
            ];
            let mut edges = vec![
                Edge::new(NodeId::new("o1"), NodeId::new("c1"), EdgeKind::Supports),
                Edge::new(NodeId::new("d1"), NodeId::new("o1"), EdgeKind::Retracts),
            ];
            if lift {
                nodes.push(node(
                    "---\nid: d2\ntype: dissent\ntitle: that withdrawal was itself withdrawn\n---\n",
                ));
                edges.push(Edge::new(
                    NodeId::new("d2"),
                    NodeId::new("d1"),
                    EdgeKind::Retracts,
                ));
            }
            lint(&graph_of(nodes, edges))
                .into_iter()
                .filter(|v| v.gate == RETRACTED)
                .count()
        };

        assert_eq!(
            build(false),
            1,
            "positive control: a binding retraction is reported"
        );
        assert_eq!(
            build(true),
            0,
            "d1 is itself withdrawn, so it does not bind and o1 is live"
        );
    }

    /// A prerequisite is load-bearing through its INCOMING edges.
    ///
    /// `X depends_on Y` is an edge from X to Y, so what makes Y load-bearing is that
    /// something depends ON it. `still_cited` read the outgoing direction — Y's own
    /// dependencies — which is the question of what Y leans on, not what leans on Y. A
    /// fully-groomed withdrawn prerequisite therefore reported nothing at all.
    #[test]
    fn a_withdrawn_prerequisite_is_load_bearing() {
        let g = graph_of(
            vec![
                node("---\nid: c1\ntype: claim\ntitle: A catalogue entry was recorded\n---\n"),
                node("---\nid: p1\ntype: claim\ntitle: The prerequisite finding\n---\n"),
                node("---\nid: d1\ntype: dissent\ntitle: the prerequisite was withdrawn\n---\n"),
            ],
            vec![
                Edge::new(NodeId::new("c1"), NodeId::new("p1"), EdgeKind::DependsOn),
                Edge::new(NodeId::new("d1"), NodeId::new("p1"), EdgeKind::Retracts),
            ],
        );
        assert_eq!(
            lint(&g).into_iter().filter(|v| v.gate == RETRACTED).count(),
            1,
            "c1 declares it cannot hold without p1, and the record withdraws p1"
        );
    }

    /// One negation rule, not two — and the verb lint had the losing copy.
    ///
    /// `clause_negated` checks EVERY occurrence within its own clause. `is_negated`
    /// checked whichever occurrence `find()` reached first, with a six-word lookback
    /// that walked straight through commas and conjunctions. The forbidden-verb lint
    /// used the second, so one hedged mention licensed every later unhedged one — an
    /// author writes "this does not conclusively record the path", then says what they
    /// meant, and the packet seals the verdict.
    ///
    /// Asserted through `lint`, never through the predicate: the last time a check like
    /// this was verified by calling the function directly, the join was what was broken.
    #[test]
    fn an_earlier_hedge_does_not_excuse_a_later_verdict() {
        let fired = |body: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: A catalogue entry was recorded\n---\n\n{body}\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == FORBIDDEN_VERB)
                .count()
        };

        assert_eq!(
            fired("The entry conclusively shows the program ran."),
            1,
            "positive control: an unhedged intensifier must fire on its own"
        );
        assert_eq!(
            fired(
                "The hive does not conclusively record the path.\n                 The entry conclusively shows the program ran."
            ),
            1,
            "an earlier NEGATED occurrence must not excuse the later bare one"
        );
        assert_eq!(
            fired("This does not prove execution, but the entry proves that the program ran."),
            1,
            "a six-word lookback reached past the comma and the `but` into the previous clause"
        );
        assert_eq!(
            fired("The hive does not conclusively record the path."),
            0,
            "a genuinely hedged sentence stays permitted — the point of the check"
        );
    }

    /// A bare auxiliary is not a negator.
    ///
    /// `does` sat in NEGATORS beside `not`, `never` and `cannot`. Every NEGATING form of
    /// it — "does not", "doesn't" — was already covered by those entries, so the bare
    /// word contributed nothing except reading a plain affirmative as a denial. "The
    /// metadata does establish that the entry was forged" is as flat a verdict as the
    /// same sentence without the auxiliary.
    #[test]
    fn a_bare_auxiliary_does_not_negate() {
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
            fired("The metadata shows the respondent is liable for the loss"),
            1,
            "positive control"
        );
        assert_eq!(
            fired("The metadata does show the respondent is liable for the loss"),
            1,
            "`does` is an emphatic auxiliary here, and the verdict is unqualified"
        );
        assert_eq!(
            fired("The metadata does not show the respondent is liable for the loss"),
            0,
            "the negating form still reads as negated — `not` carries it, as it always did"
        );
    }

    /// A negator can follow the word it governs.
    ///
    /// "proves nothing about execution" is a DENIAL, and refusing it punishes the exact
    /// sentence the discipline asks an expert to write. Only the object position is
    /// read — the word immediately after the verb — because scanning the whole clause
    /// would excuse "proves that the file was not present", where the negator governs
    /// something else entirely.
    #[test]
    fn a_negator_in_the_object_position_negates_the_verb() {
        let fired = |body: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: c1\ntype: claim\ntitle: A catalogue entry was recorded\n---\n\n{body}\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == FORBIDDEN_VERB)
                .count()
        };

        assert_eq!(
            fired("The entry proves nothing about execution."),
            0,
            "a denial, and the careful form of it"
        );
        assert_eq!(
            fired("The entry proves that the file was not present on the volume."),
            1,
            "the negator governs the object clause, not the verb — still a claim to have proved something"
        );
    }

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

    /// An instruction is not a universal claim about the world.
    ///
    /// "Always image the disk before every acquisition" is a **procedure**, and being
    /// universal is what makes it one rather than a suggestion. The declaration lint
    /// read `always` and `every` as unbacked quantification and demanded a
    /// `quantifier:`, which a protocol has no business declaring.
    ///
    /// Deliberately NOT solved by skipping `Protocol` nodes: a kind test has been
    /// removed from this codebase three times, and an author could relabel a universal
    /// claim `type: protocol` and escape — a protocol's title renders into any packet it
    /// supports. The distinction is in the TEXT. An instruction says what to DO; a claim
    /// says what IS.
    #[test]
    fn an_instruction_is_not_a_universal_claim() {
        let fired = |kind: &str, title: &str| {
            let g = graph_of(
                vec![node(&format!(
                    "---\nid: x\ntype: {kind}\ntitle: {title}\n---\n"
                ))],
                vec![],
            );
            lint(&g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-DECLARATION-CONTRADICTED")
                .count()
        };

        for t in [
            "Always image the disk before every acquisition",
            "Every exhibit must be photographed before it is opened",
            "Verify the hash on all acquired images",
            "Never power on the original device",
        ] {
            assert_eq!(
                fired("protocol", t),
                0,
                "an instruction is not a claim: {t}"
            );
        }

        assert_eq!(
            fired(
                "protocol",
                "Every Amcache entry is written at execution time"
            ),
            1,
            "a claim about the world does not stop being one because the node says \
`type: protocol` — the kind is a self-declared string"
        );
        assert_eq!(
            fired("claim", "Every Amcache entry is written at execution time"),
            1,
            "and it still fires on a claim"
        );
    }

    /// An instrument with no recorded positive control cannot certify a zero.
    ///
    /// From `docs/method/source-register.md`: a source that answers successfully with
    /// the WRONG thing is the failure mode a register exists for, and the control that
    /// catches it is a positive one — a query whose answer you already know. Until an
    /// instrument has fired on a known positive, a null from it is an UNMEASURED result
    /// wearing a measurement's clothes.
    ///
    /// The discipline was documented, the graph could express it, and nothing read it.
    #[test]
    fn an_instrument_with_no_positive_control_cannot_certify_a_null() {
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let e = |f: &str, t: &str, k: EdgeKind| Edge::new(NodeId::new(f), NodeId::new(t), k);
        let fired = |g: &Graph| {
            lint(g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-UNCONTROLLED-INSTRUMENT")
                .count()
        };

        let uncontrolled = graph_of(
            vec![
                claim.clone(),
                node("---\nid: i1\ntype: instrument\ntitle: a feed nobody has tested\n---\n"),
                node("---\nid: o1\ntype: observation\ntitle: the search returned nothing\n---\n"),
            ],
            vec![
                e("o1", "c1", EdgeKind::Supports),
                e("o1", "i1", EdgeKind::MeasuredBy),
            ],
        );
        assert_eq!(
            fired(&uncontrolled),
            1,
            "an instrument that has never been shown to fire on a known positive cannot \
support a claim — its null is unmeasured, not zero"
        );

        let controlled = graph_of(
            vec![
                claim,
                node(
                    "---\nid: i1\ntype: instrument\ntitle: a feed with a control\n\
positive_control: returns a nonzero price for a major listed asset at a known past date\n---\n",
                ),
                node("---\nid: o1\ntype: observation\ntitle: the search returned nothing\n---\n"),
            ],
            vec![
                e("o1", "c1", EdgeKind::Supports),
                e("o1", "i1", EdgeKind::MeasuredBy),
            ],
        );
        assert_eq!(
            fired(&controlled),
            0,
            "a declared positive control is what the register asks for; demanding more \
would punish the author who kept one"
        );
    }

    /// Two observations off the same instrument are one line, not two.
    ///
    /// `false_independence` fired only where an author explicitly wrote `duplicates:` —
    /// the honest-author-only shape this project keeps finding. But independence is what
    /// the whole grading system rests on: G4 means *multiple materially independent
    /// convergent lines*, and two readings from one tool are one line however they are
    /// labelled.
    ///
    /// `instrument` nodes and `measured_by:` edges have existed since the schema work
    /// and had **zero consumers outside `core`** — the same state `retracts:` was in
    /// before it turned out to be the structural synonym for a refused field.
    #[test]
    fn two_observations_off_one_instrument_are_not_independent() {
        let instrument =
            node("---\nid: i1\ntype: instrument\ntitle: the parser build under test\n---\n");
        let claim = node("---\nid: c1\ntype: claim\ntitle: t\n---\n");
        let obs = |id: &str| {
            node(&format!(
                "---\nid: {id}\ntype: observation\ntitle: r\n---\n"
            ))
        };
        let e = |f: &str, t: &str, k: EdgeKind| Edge::new(NodeId::new(f), NodeId::new(t), k);

        let shared = graph_of(
            vec![claim.clone(), instrument.clone(), obs("o1"), obs("o2")],
            vec![
                e("o1", "c1", EdgeKind::Supports),
                e("o2", "c1", EdgeKind::Supports),
                e("o1", "i1", EdgeKind::MeasuredBy),
                e("o2", "i1", EdgeKind::MeasuredBy),
            ],
        );
        let fired = |g: &Graph| {
            lint(g)
                .into_iter()
                .filter(|v| v.gate == "PEIR-LINT-FALSE-INDEPENDENCE")
                .count()
        };
        assert_eq!(
            fired(&shared),
            1,
            "two supporters measured by the same instrument are one line of evidence"
        );

        // Different instruments: genuinely two lines, and must stay quiet.
        let distinct = graph_of(
            vec![
                claim,
                instrument,
                node("---\nid: i2\ntype: instrument\ntitle: an independent tool\n---\n"),
                obs("o1"),
                obs("o2"),
            ],
            vec![
                e("o1", "c1", EdgeKind::Supports),
                e("o2", "c1", EdgeKind::Supports),
                e("o1", "i1", EdgeKind::MeasuredBy),
                e("o2", "i2", EdgeKind::MeasuredBy),
            ],
        );
        assert_eq!(
            fired(&distinct),
            0,
            "different instruments are different lines — flagging them would punish the \
author who recorded provenance at all"
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
