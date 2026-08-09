//! `elenchus` — the deterministic checker.
//!
//! Note what is absent: there is **no subcommand that sets a claim's state**. State
//! is computed from gates, reviewer records and the grounded extension, and the
//! absence of a write path is the enforcement. A model driving this CLI can add
//! nodes and edges all day and never assert that anything is accepted.

use clap::{Parser, Subcommand};
use elenchus_core::{Graph, NodeId};
use elenchus_lens::{examine_graph, lints, Violation, CATALOG};
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
    name = "elenchus",
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
    elenchus_core::load(vault).map_err(|e| e.to_string())
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

/// Everything blocking one node, gates and lints together.
fn blocking_for(graph: &Graph, id: &NodeId) -> Vec<Violation> {
    examine_graph(graph)
        .into_iter()
        .chain(lints::lint(graph))
        .filter(|v| &v.subject == id)
        .collect()
}

/// The vault skeleton. Areas `00-59` belong to OGS; elenchus claims `60-99`.
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
            "IN — every attack on it is itself defeated"
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
        let enforced = elenchus_lens::enforced().count();
        println!("{} lenses ({enforced} enforced):\n", CATALOG.len());
        for l in CATALOG {
            println!("  {:<11} {:<11} {}", l.id, format!("{:?}", l.phase), l.name);
        }
        return Ok(exit::OK);
    };

    let l = elenchus_lens::lens(&id)
        .ok_or_else(|| format!("no lens `{id}`; run `elenchus lens` to list them"))?;
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
    match elenchus_court::freeze(&graph, &NodeId::new(id)) {
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

    let fresh = elenchus_court::freeze(&graph, &NodeId::new(subject)).map_err(|e| e.to_string())?;
    if fresh.body == stored {
        println!(
            "✓ {} still matches the vault (sha256 {})",
            packet.display(),
            fresh.digest
        );
        Ok(exit::OK)
    } else {
        println!(
            "✗ {} does NOT match the vault — the graph changed under a frozen packet",
            packet.display()
        );
        Ok(exit::VIOLATIONS)
    }
}

fn run() -> Result<u8, String> {
    match Cli::parse().command {
        Command::Init { path } => cmd_init(&path),
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
            eprintln!("elenchus: {msg}");
            ExitCode::from(exit::ERROR)
        }
    }
}
