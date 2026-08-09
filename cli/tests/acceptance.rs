//! The acceptance test, from the Vibe Research design doc:
//!
//! > Given the claim "this Amcache entry proves execution", can the system
//! > immediately show the precise observation, alternative mechanisms, version
//! > boundaries, supporting and contradicting evidence, exact passages,
//! > reproducible tests, reviewer state and courtroom-safe formulation?
//!
//! Restated as an assertion: the engine must **refuse to promote** the over-claim,
//! name the lenses that blocked it, and emit the bounded formulation — with no
//! model in the loop.
//!
//! # Why two vaults rather than one
//!
//! A green result on the bounded vault alone cannot be distinguished from a checker
//! that passes everything. The over-claim vault is the positive control: it proves
//! the gates discriminate. Both are needed, and neither is sufficient.
//!
//! The third control — neutering a gate to prove it was load-bearing — mutates the
//! source and so is run by hand; `docs/validation.md` records it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use peira_core::{load, Graph, NodeId};
use peira_lens::{examine_graph, lints};
use std::path::PathBuf;

fn vault(name: &str) -> Graph {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("vaults")
        .join(name);
    load(&root).unwrap_or_else(|e| panic!("loading {name}: {e}"))
}

fn codes(graph: &Graph) -> Vec<&'static str> {
    examine_graph(graph)
        .into_iter()
        .chain(lints::lint(graph))
        .map(|v| v.gate)
        .collect()
}

// ── Control A: the over-claim must be refused ────────────────────────────────

#[test]
fn control_a_the_overclaim_is_blocked_by_at_least_five_lenses() {
    let graph = vault("overclaim");
    let found = codes(&graph);

    let expected = [
        "PEIR-CRITERION-UNDECLARED",  // 立極 — judged with no declared standard
        "PEIR-FUNCTION-AS-SUBSTANCE", // 體用 — what it did, stated as what it is
        "PEIR-CLASS-EXTENSION-UNDECLARED", // 白馬非馬 — one token, quantified universally
        "PEIR-CORNERS-UNADDRESSED",   // 四句 — contested, binarised
        "PEIR-WARRANT-MISSING",       // Toulmin — the unwritten rule
        "PEIR-CAUSAL-RUNG-UNREACHED", // Pearl — rung 3 from rung 1 data
        "PEIR-BOUNDARIES-MISSING",    // no conditions under which it would change
        "PEIR-LINT-FORBIDDEN-VERB",   // "proves"
    ];

    for code in expected {
        assert!(
            found.contains(&code),
            "expected {code} to fire on the over-claim; got {found:?}"
        );
    }
    assert!(
        found.len() >= 5,
        "the acceptance bar is five independent lenses; got {}",
        found.len()
    );
}

#[test]
fn control_a_the_overclaim_cannot_be_frozen_into_a_packet() {
    let graph = vault("overclaim");
    let err = peira_court::freeze(&graph, &NodeId::new("c-overclaim"))
        .expect_err("a packet must not be frozen over an unexamined claim");
    let rendered = err.to_string();
    assert!(rendered.contains("cannot be frozen"), "{rendered}");
    assert!(rendered.contains("PEIR-"), "it must name what blocked");
}

// ── Control B: the bounded conclusion must pass ──────────────────────────────

#[test]
fn control_b_the_bounded_conclusion_passes_every_gate() {
    let graph = vault("bounded");
    let found = codes(&graph);
    assert!(
        found.is_empty(),
        "the bounded conclusion must clear every gate; got {found:?}"
    );
}

#[test]
fn control_b_the_bounded_claim_survives_in_the_grounded_extension() {
    let graph = vault("bounded");
    assert!(graph.is_grounded(&NodeId::new("c-bounded")));
}

#[test]
fn control_b_freezes_a_packet_carrying_the_three_moments() {
    let graph = vault("bounded");
    let packet = peira_court::freeze(&graph, &NodeId::new("c-bounded"))
        .expect("the bounded claim must freeze");

    // The safe statement is generated, not authored — so these are structural.
    assert!(packet.body.contains("所謂"), "{}", packet.body);
    assert!(packet.body.contains("即非"), "{}", packet.body);
    assert!(packet.body.contains("是名"), "{}", packet.body);

    // The alternative mechanism appears, and appears as a LIMIT rather than an
    // attack: restating a claim within what the evidence carries is what turns a
    // contradiction into a boundary.
    assert!(packet.body.contains("## Limits"));
    assert!(packet.body.contains("h-install"), "{}", packet.body);

    // Boundaries are present and specific.
    assert!(packet.body.contains("Windows 10 1809"), "{}", packet.body);

    assert_eq!(packet.digest.len(), 64, "sha256 hex digest");
}

// ── The discrimination the two controls exist to establish ───────────────────

#[test]
fn the_gates_discriminate_rather_than_blocking_everything() {
    let over = codes(&vault("overclaim"));
    let bounded = codes(&vault("bounded"));
    assert!(
        !over.is_empty() && bounded.is_empty(),
        "a checker that blocks everything, or passes everything, has told us nothing: \
over-claim={over:?} bounded={bounded:?}"
    );
}

// ── Control C: an absent vault is an error, not an empty pass ────────────────

#[test]
fn control_c_an_absent_vault_errors_rather_than_reporting_success() {
    let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("definitely-not-a-vault");
    assert!(
        load(&missing).is_err(),
        "a missing vault must fail loudly — zero findings over zero nodes is the \
classic false green"
    );
}

// ── Tamper detection ─────────────────────────────────────────────────────────

#[test]
fn a_frozen_packet_stops_verifying_when_its_evidence_changes() {
    let graph = vault("bounded");
    let packet = peira_court::freeze(&graph, &NodeId::new("c-bounded")).unwrap();
    assert!(peira_court::verify(&graph, &packet));

    // Re-load and mutate an observation the packet cites.
    let mut tampered = vault("bounded");
    let doctored = peira_core::parse_node(
        "---\nid: o1\ntype: observation\ntitle: SOMETHING ELSE ENTIRELY\naspect: function\n\
pramana: perception\nsupports: [\"c-bounded grade=G2 by=albert via=perception\"]\n---\n",
    )
    .unwrap();
    tampered.insert_node(doctored);

    assert!(
        !peira_court::verify(&tampered, &packet),
        "changing cited evidence under a frozen packet must break it"
    );
}
