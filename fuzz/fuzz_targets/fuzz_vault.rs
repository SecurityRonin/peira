#![no_main]
//! The whole pipeline over an adversarial vault: `vault::load` builds a graph from
//! a directory of documents (including every edge inferred from frontmatter), the
//! lens gates and lints examine it, the grounded extension decides what survives
//! attack, and Court Mode freezes a packet.
//!
//! `fuzz_parse_node` covers one document in isolation; this covers what only shows
//! up between documents — an edge pointing at nothing, a cycle in the attack graph,
//! a claim that attacks itself. Two properties beyond "does not panic":
//!
//! * **The grounded extension terminates.** It is a fixpoint over an attack graph
//!   an adversary shaped; a cycle that fails to converge hangs the checker, and a
//!   hang here surfaces as a libFuzzer timeout rather than as a stuck CI job.
//! * **`freeze` is deterministic.** `verify` re-derives the packet and compares
//!   digests, so `freeze` followed by `verify` on an UNCHANGED graph must hold. If
//!   iteration order ever leaked into the digest, this is what catches it — and a
//!   packet whose digest is not reproducible is worthless in the room it is made for.

use libfuzzer_sys::fuzz_target;
use peira_core::{vault, NodeId};
use std::fs;
use std::path::PathBuf;

/// Documents are separated by a byte that cannot occur in YAML frontmatter, so the
/// fuzzer can discover the multi-document shape by inserting one byte.
const DOC_SEPARATOR: char = '\u{1}';

/// Caps, so one pathological input cannot make a single iteration unboundedly slow
/// and starve the campaign. They bound the HARNESS, never the code under test.
const MAX_DOCS: usize = 16;
const MAX_PACKETS: usize = 8;

fn scratch() -> PathBuf {
    // One directory reused across iterations: libFuzzer runs in-process and
    // single-threaded, and creating a fresh temp dir per iteration would make the
    // filesystem, not the parser, the thing being measured.
    std::env::temp_dir().join("peira-fuzz-vault")
}

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let dir = scratch();

    // Wipe first: stale documents from the previous iteration would make the input
    // under test not the input the fuzzer thinks it is, and any crash unreproducible.
    let _ = fs::remove_dir_all(&dir);
    if fs::create_dir_all(&dir).is_err() {
        return;
    }

    for (i, doc) in text.split(DOC_SEPARATOR).take(MAX_DOCS).enumerate() {
        if fs::write(dir.join(format!("{i}.md")), doc).is_err() {
            return;
        }
    }

    let Ok(graph) = vault::load(&dir) else {
        return;
    };

    let _ = peira_lens::examine_graph(&graph);
    let _ = peira_lens::lints::lint(&graph);
    let _ = graph.dangling_edges();

    // Terminates, and agrees with itself: `is_grounded` must not disagree with the
    // set `grounded_extension` computed.
    let grounded = graph.grounded_extension();
    for id in &grounded {
        assert!(
            graph.is_grounded(id),
            "grounded_extension contains {id:?} but is_grounded says otherwise"
        );
    }

    let ids: Vec<NodeId> = graph.nodes().map(|n| n.id.clone()).collect();
    for id in ids.iter().take(MAX_PACKETS) {
        if let Ok(packet) = peira_citation::freeze(&graph, id) {
            assert!(
                peira_citation::verify(&graph, &packet).is_verified(),
                "a packet frozen from an unchanged graph failed to verify against it"
            );
        }
    }
});
