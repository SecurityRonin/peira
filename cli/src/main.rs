//! `peira` — the deterministic checker.
//!
//! Note what is absent: there is **no subcommand that sets a claim's state**. State
//! is computed from gates, reviewer records and the grounded extension, and the
//! absence of a write path is the enforcement. A model driving this CLI can add
//! nodes and edges all day and never assert that anything is accepted.

use clap::{Parser, Subcommand};
use peira_core::{Graph, NodeId, NodeKind};
use peira_court::Verification;
use peira_lens::{examine_graph, lints, Violation, CATALOG};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

/// Exit codes, so CI and the three-way control can tell outcomes apart.
mod exit {
    /// Everything the command checked passed.
    pub const OK: u8 = 0;
    /// The command ran and found violations. Not an error — a verdict.
    pub const VIOLATIONS: u8 = 1;
    /// The command could not run at all.
    pub const ERROR: u8 = 2;
}

#[derive(Parser)]
#[command(
    name = "peira",
    about = "Examine a knowledge vault against classical critical-thinking gates.",
    long_about = "Examine a knowledge vault against classical critical-thinking gates.\n\n\
There is deliberately no command that sets a claim's status. Status is derived from \
gates, reviewer records and the grounded extension. The absence is the enforcement.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a vault: Lexicon, Inquiry, Examinations, Packets.
    Init {
        /// Where to create it.
        path: PathBuf,
    },
    /// Rebuild the derived index. Disposable — never authoritative, never committed.
    Index {
        /// Vault root.
        vault: PathBuf,
        /// Where to write the database.
        #[arg(long, default_value = "index.sqlite")]
        out: PathBuf,
    },
    /// Run the deterministic lint pack.
    Lint {
        /// Vault root.
        vault: PathBuf,
    },
    /// Run the enforced lens gates.
    Gates {
        /// Vault root.
        vault: PathBuf,
        /// Restrict to one node.
        #[arg(long)]
        node: Option<String>,
    },
    /// Show a node's derived standing. Derived — never set.
    Status {
        /// Vault root.
        vault: PathBuf,
        /// The node.
        id: String,
    },
    /// Print the grounded extension.
    Graph {
        /// Vault root.
        vault: PathBuf,
        /// Compute the grounded extension.
        #[arg(long)]
        grounded: bool,
    },
    /// Browse the lens catalogue.
    Lens {
        /// A lens id; omit to list all.
        id: Option<String>,
    },
    /// Freeze a Court Mode citation packet.
    Packet {
        /// Vault root.
        vault: PathBuf,
        /// The claim.
        id: String,
        /// Write the packet here instead of stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Re-derive a frozen packet and compare it against the vault.
    Verify {
        /// Vault root.
        vault: PathBuf,
        /// The frozen packet file.
        packet: PathBuf,
    },
}

fn load(vault: &Path) -> Result<Graph, String> {
    let graph = peira_core::load(vault).map_err(|e| e.to_string())?;
    // AN EMPTY VAULT IS NOT A CLEAN ONE. A directory holding no nodes reported
    // "✓ nothing to report", exit 0 — indistinguishable from a vault whose every claim
    // passed. That is "found nothing" wearing the face of "found nothing wrong", and
    // this project's whole acceptance argument rests on those being distinguishable:
    // control C exists to prove an absent vault exits 2 rather than looking clean.
    //
    // A path that exists but holds no readable node is the same category — the tool
    // could not look at anything — so it gets the same code.
    if graph.nodes().next().is_none() {
        return Err(format!(
            "vault root `{}` holds no nodes — nothing was examined, which is not the \
same as nothing being wrong",
            vault.display()
        ));
    }
    Ok(graph)
}

fn report(violations: &[Violation], what: &str) -> u8 {
    if violations.is_empty() {
        println!("✓ {what}: nothing to report.");
        return exit::OK;
    }
    println!("✗ {what}: {} finding(s).\n", violations.len());
    for v in violations {
        println!("  {} [{}]  {}", v.gate, v.lens, v.subject);
        println!("      {}", v.detail);
        println!("      → {}\n", v.remedy);
    }
    exit::VIOLATIONS
}

/// Attacks that stopped counting because someone withdrew them.
///
/// The status line said "every attack on it is itself defeated" while an idle note by
/// anyone at all could withdraw a rival — the attack was removed, not answered, and
/// the difference is the whole of what a reader needs. The packet discloses it; a
/// status line that disagreed with the packet would be the drift this file already
/// deleted once.
/// What `peira status` should exit with.
///
/// The defeat verdict was PRINTED and then dropped: status said "contested — defeated
/// in the grounded extension" and exited 0, while `packet` refused the same claim and
/// exited 1. A state the output names must reach the exit code, or every script reading
/// this command is told the claim is fine.
const fn status_exit(blocking_empty: bool, grounded: bool, is_argument: bool) -> u8 {
    if !blocking_empty {
        return exit::VIOLATIONS;
    }
    if is_argument && !grounded {
        return exit::VIOLATIONS;
    }
    exit::OK
}

/// Delegated, never re-derived. This was a direct-edge test while court used the
/// `Graph::withdrawn()` fixed point, so `status` and `packet` gave opposite accounts of
/// the same restored attack. Two implementations of one question is how a checker and
/// the thing it checks drift apart — the same seam `blocking_for` already closes.
fn withdrawn_attacks(graph: &Graph, id: &NodeId) -> usize {
    peira_court::withdrawn_attacks(graph, id).len()
}

/// Everything blocking one node — the same question `freeze` asks, asked once.
///
/// This filtered findings to the node's own id while court walked the evidential
/// closure, so `peira status` reported "all enforced gates pass" over a claim
/// `peira packet` refused. A status line that disagrees with the tool's own refusal
/// is worse than no status line.
fn blocking_for(graph: &Graph, id: &NodeId) -> Vec<Violation> {
    peira_court::violations_for(graph, id)
}

/// The vault skeleton. Areas `00-59` belong to OGS; peira claims `60-99`.
const AREAS: &[(&str, &str)] = &[
    (
        "60-lexicon",
        "Terms and Criteria — bounded reference objects, Johnny.Decimal addressed.\n\
Every load-bearing term gets all three moments: as_used, not_essence, stipulated.\n",
    ),
    (
        "70-inquiry",
        "Questions, Hypotheses, Claims, Observations, Protocols, Runs.\n\
UID-addressed and flat: a graph must not be foldered.\n",
    ),
    (
        "80-examinations",
        "Examination records and preserved Dissents. Rejection never deletes.\n",
    ),
    (
        "90-packets",
        "Frozen Court Mode exports. Write-once by convention; verify against the vault.\n",
    ),
];

fn cmd_init(path: &Path) -> Result<u8, String> {
    for (dir, readme) in AREAS {
        let full = path.join(dir);
        std::fs::create_dir_all(&full).map_err(|e| format!("{}: {e}", full.display()))?;
        let marker = full.join("README.md");
        if !marker.exists() {
            std::fs::write(&marker, format!("# {dir}\n\n{readme}"))
                .map_err(|e| format!("{}: {e}", marker.display()))?;
        }
    }
    println!("Scaffolded a vault at {}", path.display());
    for (dir, _) in AREAS {
        println!("  {dir}/");
    }
    Ok(exit::OK)
}

fn cmd_index(vault: &Path, out: &Path) -> Result<u8, String> {
    let graph = load(vault)?;
    peira_index::build(&graph, out).map_err(|e| format!("{}: {e}", out.display()))?;
    println!(
        "Indexed {} node(s) and {} edge(s) → {}",
        graph.nodes().count(),
        graph.edges().count(),
        out.display()
    );
    println!("  (derived and disposable — rebuilt from the markdown, never committed)");
    Ok(exit::OK)
}

fn cmd_gates(vault: &Path, node: Option<String>) -> Result<u8, String> {
    let graph = load(vault)?;
    // WITHOUT `--node` this is a vault-wide GATE survey, and that is what it says.
    // WITH `--node` the question changes to "what stands in the way of this node", and
    // that question already has an answer used by `status` and `freeze`. Running the
    // gate pack alone and scoping it answered a NARROWER question and disagreed with
    // the tool's own refusal: "nothing to report", exit 0, over a claim `peira packet`
    // refused, because the finding belonged to a lint family this command never ran.
    //
    // Third recurrence of this pair disagreeing. The first two were fixed by correcting
    // the SCOPE; the scope was never the whole of it — the finding SET was narrower too.
    if let Some(id) = node {
        let id = NodeId::new(id);
        if graph.node(&id).is_none() {
            return Err(format!("no node `{id}` in the vault"));
        }
        return Ok(report(&blocking_for(&graph, &id), "gates"));
    }
    Ok(report(&examine_graph(&graph), "gates"))
}

fn cmd_status(vault: &Path, id: &str) -> Result<u8, String> {
    let graph = load(vault)?;
    let id = NodeId::new(id);
    let node = graph
        .node(&id)
        .ok_or_else(|| format!("no node `{id}` in the vault"))?;

    let blocking = blocking_for(&graph, &id);
    let grounded = graph.is_grounded(&id);
    let is_arg = node.kind.is_argument();

    println!("{id}  ({})", node.kind);
    println!("  {}", node.title);
    println!();
    println!(
        "  grounded extension : {}",
        if grounded {
            if withdrawn_attacks(&graph, &id) > 0 {
                "IN — but some attacks were WITHDRAWN, not answered (see `peira packet`)"
            } else {
                "IN — every attack on it is itself defeated"
            }
        } else if is_arg {
            "OUT — an attack stands unanswered"
        } else {
            "n/a — reference material does not compete"
        }
    );
    println!(
        "  gates              : {}",
        if blocking.is_empty() {
            "all enforced gates pass".to_owned()
        } else {
            format!("{} blocking", blocking.len())
        }
    );
    println!(
        "  derived state      : {}",
        if blocking.is_empty() {
            if grounded || !is_arg {
                "review_ready — gates pass; a reviewer must still sign"
            } else {
                "contested — defeated in the grounded extension"
            }
        } else {
            "evidence_pending — gates block"
        }
    );
    println!("\n  (derived, not stored — there is no field to write it to)");

    Ok(status_exit(blocking.is_empty(), grounded, is_arg))
}

fn cmd_graph(vault: &Path, grounded: bool) -> Result<u8, String> {
    let graph = load(vault)?;
    if !grounded {
        println!(
            "{} node(s), {} edge(s)",
            graph.nodes().count(),
            graph.edges().count()
        );
        return Ok(exit::OK);
    }
    let extension = graph.grounded_extension();
    println!("Grounded extension ({} in):", extension.len());
    for id in &extension {
        if let Some(n) = graph.node(id) {
            println!("  IN   {id}  {}", n.title);
        }
    }
    for n in graph.nodes().filter(|n| n.kind.is_argument()) {
        if !extension.contains(&n.id) {
            println!("  out  {}  {}", n.id, n.title);
        }
    }
    Ok(exit::OK)
}

fn cmd_lens(id: Option<String>) -> Result<u8, String> {
    let Some(id) = id else {
        let enforced = peira_lens::enforced().count();
        println!("{} lenses ({enforced} enforced):\n", CATALOG.len());
        for l in CATALOG {
            println!("  {:<11} {:<11} {}", l.id, format!("{:?}", l.phase), l.name);
        }
        return Ok(exit::OK);
    };

    let l = peira_lens::lens(&id)
        .ok_or_else(|| format!("no lens `{id}`; run `peira lens` to list them"))?;
    println!("{}  ({})\n", l.name, l.tradition);
    println!("  failure mode : {}", l.failure_mode);
    println!("  operation    : {}", l.operation);
    println!("  phase        : {:?}", l.phase);
    if !l.gates.is_empty() {
        println!("  gates        :");
        for g in l.gates {
            println!("      {}", g.code);
        }
    }
    println!("\n  worked example:\n    {}", l.worked_example);
    println!("\n  sources:");
    for s in l.sources {
        println!("    {s}");
    }
    Ok(exit::OK)
}

fn cmd_packet(vault: &Path, id: &str, out: Option<PathBuf>) -> Result<u8, String> {
    let graph = load(vault)?;
    match peira_court::freeze(&graph, &NodeId::new(id)) {
        Ok(packet) => {
            if let Some(path) = out {
                std::fs::write(&path, &packet.body)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                println!("Froze {} → {}", packet.subject, path.display());
                println!("sha256 {}", packet.digest);
            } else {
                print!("{}", packet.body);
            }
            Ok(exit::OK)
        }
        Err(e) => {
            println!("{e}");
            Ok(exit::VIOLATIONS)
        }
    }
}

fn cmd_verify(vault: &Path, packet: &Path) -> Result<u8, String> {
    let graph = load(vault)?;
    let stored =
        std::fs::read_to_string(packet).map_err(|e| format!("{}: {e}", packet.display()))?;
    let subject = stored
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("# Citation packet — "))
        .ok_or_else(|| format!("{} does not look like a packet", packet.display()))?
        .trim();

    // The library owns the comparison, including reading the packet's declared format.
    // Re-implementing it here is how a checker and the thing it checks drift apart.
    let doc = peira_court::Packet::from_stored(NodeId::new(subject), stored);
    match peira_court::verify(&graph, &doc) {
        Verification::Verified => {
            println!(
                "✓ {} still matches the vault (sha256 {})",
                packet.display(),
                doc.digest
            );
            Ok(exit::OK)
        }
        // NOT by itself an accusation. A vault that grew — a corroborating observation
        // added months later — and a vault whose cited evidence was altered produce the
        // same verdict, and only one is misconduct. Report the difference, name where it
        // starts, and let the reader judge.
        Verification::DigestMismatch {
            format_line_only,
            stored,
            fresh,
            first_difference,
        } => {
            if format_line_only {
                // THE ONE CASE THE TOOL CAN ESTABLISH. `verify` proved the format number
                // is the sole difference: correcting it makes the stored body
                // byte-identical to what this build renders, and no older renderer could
                // produce a newer one's bytes. The generic exculpation below was printed
                // over this too — the proof was computed and dropped in transit, in the
                // fix written to close exactly that.
                println!(
                    "✗ {} was EDITED — its declared packet format was changed by hand",
                    packet.display()
                );
                println!("  packet sha256 {stored}");
                println!("  vault  sha256 {fresh}");
                if let Some(d) = first_difference {
                    println!("\n  first difference:\n  {d}");
                }
                println!(
                    "\n  Correcting that one number makes the stored body byte-identical\n  to what this build renders, so it was not written by an older\n  renderer — an older \
renderer cannot emit a newer one's bytes."
                );
                return Ok(exit::VIOLATIONS);
            }
            println!(
                "✗ {} no longer matches the vault — the record has changed since it froze",
                packet.display()
            );
            println!("  packet sha256 {stored}");
            println!("  vault  sha256 {fresh}");
            if let Some(d) = first_difference {
                println!("\n  first difference:\n  {d}");
            }
            println!(
                "\n  This says the record moved, not that anyone altered it. Adding \
evidence\n  moves it too. Compare the packet against the vault's history before \
drawing\n  any conclusion about why."
            );
            Ok(exit::VIOLATIONS)
        }
        // Exit 2, the same code an absent vault returns, because this is the same
        // category: not a verdict, an inability to reach one. Reporting it as a
        // mismatch would accuse the holder of a packet that is perfectly intact.
        Verification::FormatSuperseded {
            stored,
            current,
            body_matches: _,
        } => {
            println!(
                "? {} was written in packet format {stored}; this build renders {current}",
                packet.display()
            );
            println!("  no verdict — re-freeze the claim to compare against this format");
            println!(
                "\n  This build cannot re-derive it, so nothing here is a verdict about \
the\n  packet's integrity. A body that differs beyond the format line is\n  consistent BOTH with an older renderer and with alteration, and peira\n  cannot tell those apart — the information is not in the artifact.\n  Compare against the vault's \
history at the time it was frozen."
            );
            Ok(exit::ERROR)
        }
        // A verdict, but about the CLAIM rather than the packet: something now blocks
        // it. `e` names which gates, in full.
        Verification::NoLongerFreezable(e) => {
            println!(
                "✗ {} cannot be re-derived — the claim no longer qualifies",
                packet.display()
            );
            println!("  {e}");
            Ok(exit::VIOLATIONS)
        }
        // `Verification` is #[non_exhaustive], so a newer library can hand this build
        // a verdict it has never heard of. Show the value verbatim and refuse to reach
        // a conclusion: an unrecognised verdict rendered as a pass is the exact failure
        // this crate exists to prevent.
        other => {
            println!(
                "? {} — this build does not recognise the verdict {other:?}",
                packet.display()
            );
            println!("  no verdict — upgrade the CLI to match the library");
            Ok(exit::ERROR)
        }
    }
}

fn run() -> Result<u8, String> {
    match Cli::parse().command {
        Command::Init { path } => cmd_init(&path),
        Command::Index { vault, out } => cmd_index(&vault, &out),
        Command::Lint { vault } => {
            let graph = load(&vault)?;
            // AND what a packet would seal. The node-level pack deliberately skips a
            // term's moments — mention is not use — but a packet quotes them verbatim,
            // so `peira lint` reported nothing over a term `peira packet` refused. The
            // command an author runs to FIND problems was silent about the one they had.
            let mut found = lints::lint(&graph);
            for n in graph.nodes().filter(|n| n.kind == NodeKind::Claim) {
                for v in peira_court::sealed_prose_findings(&graph, &n.id) {
                    if !found
                        .iter()
                        // Dedup by SUBJECT too. Ignoring it suppressed the second
                        // claim sealing the same overstated word, while `status` and
                        // `packet` both blocked it — a finding hidden because another
                        // node happened to have the same problem first.
                        .any(|f| f.gate == v.gate && f.subject == v.subject && f.detail == v.detail)
                    {
                        found.push(v);
                    }
                }
            }
            Ok(report(&found, "lint"))
        }
        Command::Gates { vault, node } => cmd_gates(&vault, node),
        Command::Status { vault, id } => cmd_status(&vault, &id),
        Command::Graph { vault, grounded } => cmd_graph(&vault, grounded),
        Command::Lens { id } => cmd_lens(id),
        Command::Packet { vault, id, out } => cmd_packet(&vault, &id, out),
        Command::Verify { vault, packet } => cmd_verify(&vault, &packet),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(msg) => {
            eprintln!("peira: {msg}");
            ExitCode::from(exit::ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::{parse_node, Edge, EdgeKind, Node};

    fn node(src: &str) -> Node {
        parse_node(src).expect("fixture parses")
    }

    /// The exit code is asserted through the COMMAND, not the helper.
    ///
    /// `status_exit` had a test and it proved the pure function. Mutating the CALL SITE
    /// — passing `true` for `grounded` — reintroduced the documented defect (prints
    /// "contested — defeated", exits 0) with the whole suite green. Extracting a
    /// testable helper moves the untested part from the function into the call, and the
    /// test follows the function every time.
    #[test]
    fn cmd_status_exits_non_zero_on_a_defeated_claim() {
        let dir = std::env::temp_dir().join("peira-cmd-status-exit-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("70-inquiry")).expect("scratch vault");
        let write = |name: &str, body: &str| {
            std::fs::write(dir.join("70-inquiry").join(name), body).expect("write node");
        };
        // GROOMED so that gates PASS. An ungroomed fixture exits non-zero for its own
        // findings whatever the grounding is, and then `grounded` never reaches the
        // assertion — the first version of this test measured grooming and stayed green
        // under the mutation it exists to catch.
        write(
            "c1.md",
            "---\nid: c1\ntype: claim\ntitle: The register recorded the path\n\
warrant: The register records what it is given.\nquantifier: singular\naspect: function\n\
causal_rung: association\nno_terms_of_art: true\n\
boundaries:\n  - Windows 10 1809 and later\n\
corners:\n  - it holds\n  - it does not hold\n  - it holds in part\n  - the question does not arise\n\
falsifier:\n  - a register shown to record paths never supplied\n---\n",
        );
        write(
            "o1.md",
            "---\nid: o1\ntype: observation\ntitle: the register entry is present\n\
aspect: function\nsupports: [\"c1 grade=G2 by=a-reviewer via=perception\"]\n---\n",
        );

        assert_eq!(
            cmd_status(&dir, "c1").expect("status runs"),
            exit::OK,
            "control: with gates passing and nothing attacking it, status must be 0 — \
otherwise the assertion below never depends on the grounding"
        );

        // Now defeat it: a live rival nothing answers.
        write(
            "r1.md",
            "---\nid: r1\ntype: claim\ntitle: An inventory sweep produced the record\n\
warrant: Sweeps populate the same table.\nquantifier: singular\naspect: function\n\
causal_rung: association\nno_terms_of_art: true\n\
boundaries:\n  - Windows 10 1809 and later\n\
corners:\n  - it holds\n  - it does not hold\n  - it holds in part\n  - the question does not arise\n\
falsifier:\n  - a sweep shown never to write this table\n\
attacks: [\"c1\"]\n---\n",
        );
        write(
            "o2.md",
            "---\nid: o2\ntype: observation\ntitle: the sweep log\naspect: function\n\
supports: [\"r1 grade=G2 by=a-reviewer via=perception\"]\n---\n",
        );
        let defeated = cmd_status(&dir, "c1").expect("status runs");
        assert_eq!(
            defeated,
            exit::VIOLATIONS,
            "a claim `packet` refuses must not report success from `status`"
        );

        // And a node that never competes is not defeated by being outside the extension.
        write(
            "t1.md",
            "---\nid: t1\ntype: term\ntitle: a term\nas_used: a\nnot_essence: b\n\
stipulated: c\n---\n",
        );
        assert_eq!(
            cmd_status(&dir, "t1").expect("status runs"),
            exit::OK,
            "reference material does not compete, so being OUT is not a defeat"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// An empty vault is not a clean one.
    ///
    /// A directory holding no nodes reported "nothing to report", exit 0 —
    /// indistinguishable from a vault whose every claim passed. That is "found nothing"
    /// wearing the face of "found nothing wrong", and control C exists precisely to
    /// prove those are distinguishable.
    #[test]
    fn an_empty_vault_is_not_a_clean_one() {
        let dir = std::env::temp_dir().join("peira-empty-vault-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let err = load(&dir).expect_err("a vault with no nodes examined nothing");
        assert!(
            err.contains("no nodes"),
            "and it must say so rather than looking clean: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A state the output NAMES must reach the exit code.
    ///
    /// `status` printed "contested — defeated in the grounded extension" and returned
    /// 0, while `packet` refused the same claim and returned 1. Anything scripting this
    /// command was told the claim was fine.
    #[test]
    fn status_exits_non_zero_on_a_defeated_claim() {
        assert_eq!(
            status_exit(true, true, true),
            exit::OK,
            "clean and standing"
        );
        assert_eq!(
            status_exit(false, true, true),
            exit::VIOLATIONS,
            "gates block"
        );
        assert_eq!(
            status_exit(true, false, true),
            exit::VIOLATIONS,
            "gates pass, but the claim is defeated — `packet` refuses it, so status must not say 0"
        );
        assert_eq!(
            status_exit(true, false, false),
            exit::OK,
            "reference material does not compete, so being outside the extension is not a defeat"
        );
    }

    /// The display path had no tests, and it drifted.
    ///
    /// `Graph::withdrawn()` is a fixed point: a retraction that has itself been
    /// retracted does not bind, so the attack it named is LIVE again. Asking only
    /// "does an incoming retraction exist" counts it as withdrawn anyway — the
    /// pre-fix question, which court stopped asking and this file went on asking. So
    /// `peira status` reported an attack WITHDRAWN on the same vault whose packet
    /// reported it defeated on the merits.
    #[test]
    fn withdrawn_attacks_follows_the_fixed_point() {
        let mut g = Graph::new();
        g.insert_node(node(
            "---\nid: c1\ntype: claim\ntitle: A catalogue entry was recorded\n---\n",
        ));
        g.insert_node(node(
            "---\nid: a1\ntype: claim\ntitle: The installer wrote it\n---\n",
        ));
        g.insert_node(node(
            "---\nid: d1\ntype: dissent\ntitle: that attack is withdrawn\n---\n",
        ));
        g.insert_edge(Edge::new(
            NodeId::new("a1"),
            NodeId::new("c1"),
            EdgeKind::Attacks,
        ));
        g.insert_edge(Edge::new(
            NodeId::new("d1"),
            NodeId::new("a1"),
            EdgeKind::Retracts,
        ));
        assert_eq!(
            withdrawn_attacks(&g, &NodeId::new("c1")),
            1,
            "positive control: a binding retraction withdraws the attack"
        );

        g.insert_node(node(
            "---\nid: d2\ntype: dissent\ntitle: that withdrawal was itself withdrawn\n---\n",
        ));
        g.insert_edge(Edge::new(
            NodeId::new("d2"),
            NodeId::new("d1"),
            EdgeKind::Retracts,
        ));
        assert_eq!(
            withdrawn_attacks(&g, &NodeId::new("c1")),
            0,
            "the retraction was lifted, so nothing was withdrawn — status must not say otherwise"
        );
    }
}
