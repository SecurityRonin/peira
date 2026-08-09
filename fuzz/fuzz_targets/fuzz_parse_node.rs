#![no_main]
//! `parse_node` reads one vault document — YAML frontmatter plus a markdown body —
//! from a file a human hand-edited and an adversary may have shaped. Parsing
//! arbitrary text must NEVER panic; every malformation must come back as a
//! `ParseError` naming what was wrong.
//!
//! This is also the guard on the load-bearing invariant: a document carrying a
//! `status` or `confidence` key must be REFUSED, never absorbed. A panic here
//! would be a denial-of-service on the checker; a silent acceptance would be worse.

use libfuzzer_sys::fuzz_target;
use peira_core::parse_node;

fuzz_target!(|data: &[u8]| {
    // Lossy rather than a `from_utf8` guard: rejecting invalid UTF-8 up front would
    // throw away almost every input the fuzzer generates, so the parser would
    // barely be reached. `read_to_string` is what the real loader uses, and lossy
    // conversion keeps valid UTF-8 byte-identical.
    let text = String::from_utf8_lossy(data);

    if let Ok(node) = parse_node(&text) {
        // A parse that SUCCEEDS must not have admitted derived state. Checking it
        // here rather than trusting the parser means the fuzzer is searching for a
        // way past the refusal, not merely for a crash.
        assert!(
            node.field("status").is_none(),
            "parse_node accepted a `status` field: derived state must be refused"
        );
        assert!(
            node.field("confidence").is_none(),
            "parse_node accepted a `confidence` field: derived state must be refused"
        );

        // Accessors over whatever survived parsing.
        let _ = node.field("warrant");
        let _ = node.field_list("boundaries");
        let _ = node.fields.keys().count();
    }
});
