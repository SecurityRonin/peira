//! `peira` — the deterministic checker.
//!
//! Note what is absent: there is **no subcommand that sets a claim's state**. State
//! is computed from gates, reviewer records and the grounded extension, and the
//! absence of a write path is the enforcement. A model driving this CLI can add
//! nodes and edges all day and never assert that anything is accepted.

use clap::{Parser, Subcommand};
use peira_core::{Graph, NodeId};
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
    peira_core::load(vault).map_err(|e| e.to_string())
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
    let mut found = examine_graph(&graph);
    if let Some(id) = node {
        let id = NodeId::new(id);
        if graph.node(&id).is_none() {
            return Err(format!("no node `{id}` in the vault"));
        }
        found.retain(|v| v.subject == id);
    }
    Ok(report(&found, "gates"))
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

    Ok(if blocking.is_empty() {
        exit::OK
    } else {
        exit::VIOLATIONS
    })
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
            stored,
            fresh,
            first_difference,
        } => {
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
            Ok(report(&lints::lint(&graph), "lint"))
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
