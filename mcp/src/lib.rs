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
//! No scores. peira mints no numbers, and a response schema is exactly where a
//! helpful-looking `confidence: 0.82` would appear. A finding carries a gate code, a
//! subject, what was found and what to do instead — nothing that could be averaged.
//!
//! And an empty finding list is NOT an endorsement. The prose checks are narrow
//! heuristics by design, so "nothing fired" means the scan found nothing it knows
//! how to name, never "this sentence is safe". Every response says so, because a
//! caller that reads silence as approval has been given a worse instrument than none.

use peira_core::NodeId;
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
const SCOPE: &str = "These are narrow prose heuristics: overstated verbs, ultimate-issue \
conclusions, quantifiers, hedge frames. An empty result means the scan found nothing it \
knows how to name — it is not a finding that the text is sound, and it says nothing about \
whether the underlying claim is supported.";

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

/// The catalogue, or one entry of it.
///
/// Exposed so a caller can REASON WITH the framework rather than only be checked by
/// it: each entry names a failure mode and cites where it was identified.
#[must_use]
pub fn catalogue(id: Option<&str>) -> Vec<LensEntry> {
    match id {
        Some(want) => peira_lens::lens(want).map(entry).into_iter().collect(),
        None => peira_lens::CATALOG.iter().map(entry).collect(),
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

    /// The catalogue reaches the caller whole, and says which entries REFUSE.
    #[test]
    fn the_catalogue_distinguishes_what_enforces_from_what_is_merely_read() {
        let all = catalogue(None);
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
        let one = catalogue(Some("TRAIRUPYA"));
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].id, "TRAIRUPYA");
        assert!(one[0].gates.contains(&"PEIR-HETU-UNDIAGNOSTIC"));

        assert!(
            catalogue(Some("NOT-A-LENS")).is_empty(),
            "an unknown id must return nothing — a placeholder entry would be a lens \
that does not exist, cited as though it does"
        );
    }
}
