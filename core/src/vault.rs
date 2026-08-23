//! Loading a vault directory into a [`Graph`].
//!
//! Edges are declared in frontmatter as lists of target ids under the edge's own
//! name — `supports: [o1, o2]`, `judged_by: [60.01]` — and the loader turns them
//! into typed edges. Evidence edges may instead be written as mappings carrying
//! `to`, `grade`, `graded_by` and `pramana`.

use crate::{
    edge::{Edge, EdgeKind, Grade, Pramana},
    graph::Graph,
    node::{parse_node, NodeId, ParseError},
};
use std::{fmt, fs, io, path::Path, path::PathBuf};

/// Why a vault could not be loaded.
#[non_exhaustive]
#[derive(Debug)]
pub enum VaultError {
    /// The vault root does not exist or is not a directory. Carries the path.
    NotADirectory(PathBuf),
    /// Walking the tree failed.
    Io {
        /// Where.
        path: PathBuf,
        /// What.
        source: io::Error,
    },
    /// A document would not parse. Carries the path so the diagnostic is actionable.
    Parse {
        /// Which file.
        path: PathBuf,
        /// Why.
        source: ParseError,
    },
    /// Two documents claim the same id.
    DuplicateId {
        /// The contested id.
        id: NodeId,
        /// The file that claimed it first.
        first: PathBuf,
        /// The file that collided.
        second: PathBuf,
    },
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaultError::NotADirectory(p) => {
                write!(f, "vault root `{}` is not a directory", p.display())
            }
            VaultError::Io { path, source } => {
                write!(f, "reading `{}`: {source}", path.display())
            }
            VaultError::Parse { path, source } => {
                write!(f, "{}: {source}", path.display())
            }
            VaultError::DuplicateId { id, first, second } => write!(
                f,
                "id `{id}` is claimed by both `{}` and `{}`",
                first.display(),
                second.display()
            ),
        }
    }
}

impl std::error::Error for VaultError {}

/// Parse `id grade=G2 by=albert via=perception` into an edge.
///
/// The inline form keeps a graded edge on one line in the source note, which is
/// where the person writing it is actually looking.
fn edge_from_spec(from: &NodeId, spec: &str, kind: EdgeKind) -> Edge {
    let mut parts = spec.split_whitespace();
    let target = parts.next().unwrap_or("");
    let mut edge = Edge::new(from.clone(), NodeId::new(target), kind);

    let mut grade = None;
    let mut grader = None;
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        match key {
            "grade" => grade = Grade::from_str_opt(value),
            "proposed" => {
                if let Some(g) = Grade::from_str_opt(value) {
                    edge = edge.proposing(g);
                }
            }
            // AN EMPTY VALUE IS NOT A GRADER. `by=` with nothing after it built a grade
            // "settled" by the empty string: the ungraded-support and unreviewed-grade
            // lints both stayed silent, and the packet printed "Credited: , albert".
            // The invariant this breaks is the one the type exists for — a grade cannot
            // be built without its grader — and the emptiest possible input walked
            // straight through it.
            "by" if !value.trim().is_empty() => grader = Some(value.trim().to_owned()),
            "via" => {
                if let Some(p) = Pramana::from_str_opt(value) {
                    edge = edge.via(p);
                }
            }
            _ => {}
        }
    }

    // A grade without an attributed grader is not silently downgraded to an
    // ungraded edge, nor silently accepted: it becomes a PROPOSAL, which is what
    // an unattributed opinion actually is. The lint pack reports it.
    match (grade, grader) {
        (Some(g), Some(who)) => edge.graded_by(g, who),
        (Some(g), None) => edge.proposing(g),
        (None, _) => edge,
    }
}

/// Every `.md` file under `root`, depth-first, in sorted order.
fn markdown_files(root: &Path) -> Result<Vec<PathBuf>, VaultError> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| VaultError::Io {
            path: dir.clone(),
            source: e,
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| VaultError::Io {
                path: dir.clone(),
                source: e,
            })?;
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            // Skip dotted directories: .git, .obsidian and friends are not vault
            // content, and walking them turns a lint run into a repo scan.
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case("README.md"))
                || is_a_frozen_packet(&path)
            {
                // A FROZEN PACKET IS AN EXPORT, NOT A NODE. `peira init` scaffolds
                // `90-packets/` and its README says "frozen Court Mode exports … verify
                // against the vault" — and doing that made every subsequent command
                // refuse the whole vault, `verify` included. Following the tool's own
                // layout broke the tool.
                //
                // Recognised by CONTENT, not by directory: a packet is whatever begins
                // with the line `freeze` writes, so a vault that keeps exports elsewhere
                // works too, and a genuine node under `90-packets/` is still read.
                // README.md IS NOT A NODE. `peira init` scaffolds one in each area as
                // guidance, and the loader then refused the vault it had just created:
                // "document does not begin with a `---` frontmatter fence", exit 2. The
                // quickstart in the project's own README was broken end to end.
                //
                // Skipping by NAME rather than by malformedness — a file that is meant
                // to be a node and is malformed must still fail loudly, which is why
                // this is a named convention and not a silent tolerance of bad input.
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Load every markdown document under `root` into a graph.
/// Whether this file is a packet `freeze` wrote, rather than a node.
///
/// The first line of a rendered packet is `# Citation packet — <id>`, which `cmd_verify`
/// already relies on to find its subject. Reading only that line keeps the check cheap
/// on a large vault.
fn is_a_frozen_packet(path: &std::path::Path) -> bool {
    use std::io::BufRead as _;
    std::fs::File::open(path).ok().is_some_and(|f| {
        std::io::BufReader::new(f)
            .lines()
            .next()
            .and_then(Result::ok)
            .is_some_and(|l| l.starts_with("# Citation packet — "))
    })
}

pub fn load(root: &Path) -> Result<Graph, VaultError> {
    if !root.is_dir() {
        return Err(VaultError::NotADirectory(root.to_path_buf()));
    }

    let mut graph = Graph::new();
    let mut seen: Vec<(NodeId, PathBuf)> = Vec::new();

    for path in markdown_files(root)? {
        let text = fs::read_to_string(&path).map_err(|e| VaultError::Io {
            path: path.clone(),
            source: e,
        })?;
        let node = parse_node(&text).map_err(|source| VaultError::Parse {
            path: path.clone(),
            source,
        })?;

        if let Some((id, first)) = seen.iter().find(|(id, _)| *id == node.id) {
            return Err(VaultError::DuplicateId {
                id: id.clone(),
                first: first.clone(),
                second: path.clone(),
            });
        }
        seen.push((node.id.clone(), path.clone()));

        for kind in EdgeKind::ALL {
            for spec in node.field_list(kind.as_str()) {
                graph.insert_edge(edge_from_spec(&node.id, spec, *kind));
            }
        }
        graph.insert_node(node);
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("peira-vault-test-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, name: &str, body: &str) {
        if let Some(parent) = dir.join(name).parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(dir.join(name), body).unwrap();
    }

    /// A vault may name the detector an observation came off.
    ///
    /// peira models what was concluded and who graded it, never what produced the
    /// evidence — so a claim resting on a tool whose failure mode nobody established
    /// is indistinguishable from one resting on a validated instrument. Nothing about
    /// an instrument can be said until there is somewhere to say it.
    ///
    /// Asserted through `as_str` rather than the enum variants, so this compiles
    /// before they exist and fails because the vault refuses the document.
    #[test]
    fn an_observation_may_name_the_instrument_that_produced_it() {
        let dir = scratch("instrument");
        write(
            &dir,
            "70-inquiry/i1.md",
            "---\nid: i1\ntype: instrument\ntitle: Reactor, cluster attribution\n\
positive_control: fires on the known mixer deposit in the 2025 sample\n\
negative_control: silent on the exchange hot wallet in the same sample\n---\n",
        );
        write(
            &dir,
            "70-inquiry/o1.md",
            "---\nid: o1\ntype: observation\ntitle: Address clustered to entity E\n\
measured_by: [i1]\n---\n",
        );

        let graph = load(&dir).expect("a vault naming an instrument must load");

        let instrument = graph
            .node(&NodeId::new("i1"))
            .expect("the instrument is a node");
        assert_eq!(instrument.kind.as_str(), "instrument");
        assert_eq!(
            instrument.field("positive_control"),
            Some("fires on the known mixer deposit in the 2025 sample")
        );

        // Bound to a local: `edges_from` ties the iterator's lifetime to the id, so
        // a temporary would be dropped while still borrowed.
        let o1 = NodeId::new("o1");
        let measured: Vec<&Edge> = graph
            .edges_from(&o1)
            .filter(|e| e.kind.as_str() == "measured_by")
            .collect();
        assert_eq!(
            measured.len(),
            1,
            "`measured_by:` must build an edge from the observation to the instrument"
        );
        assert_eq!(measured[0].to, NodeId::new("i1"));
    }

    #[test]
    fn loads_nodes_and_builds_edges_from_frontmatter() {
        let dir = scratch("basic");
        write(
            &dir,
            "70-inquiry/c1.md",
            "---\nid: c1\ntype: claim\ntitle: a claim\nsupports: [o1]\n---\n",
        );
        write(
            &dir,
            "70-inquiry/o1.md",
            "---\nid: o1\ntype: observation\ntitle: an observation\n---\n",
        );

        let g = load(&dir).unwrap();
        assert_eq!(g.nodes().count(), 2);
        assert_eq!(g.edges().count(), 1);
        let e = g.edges().next().unwrap();
        assert_eq!(e.kind, EdgeKind::Supports);
        assert_eq!(e.from, NodeId::new("c1"));
        assert_eq!(e.to, NodeId::new("o1"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parses_an_inline_graded_edge_spec() {
        let e = edge_from_spec(
            &NodeId::new("o1"),
            "c1 grade=G2 by=albert via=perception",
            EdgeKind::Supports,
        );
        assert_eq!(e.to, NodeId::new("c1"));
        assert_eq!(e.grade(), Some(Grade::G2));
        assert_eq!(e.grader(), Some("albert"));
        assert_eq!(e.pramana, Some(Pramana::Perception));
    }

    #[test]
    fn a_grade_with_no_grader_degrades_to_a_proposal() {
        let e = edge_from_spec(&NodeId::new("o1"), "c1 grade=G4", EdgeKind::Supports);
        assert_eq!(e.grade(), None, "an unattributed grade settles nothing");
        assert_eq!(
            e.grade_proposed,
            Some(Grade::G4),
            "but it is preserved as what it actually is — a proposal"
        );
    }

    #[test]
    fn a_duplicate_id_is_an_error_naming_both_files() {
        let dir = scratch("dupe");
        write(&dir, "a.md", "---\nid: x\ntype: claim\ntitle: first\n---\n");
        write(
            &dir,
            "b.md",
            "---\nid: x\ntype: claim\ntitle: second\n---\n",
        );

        let err = load(&dir).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("a.md"), "{msg}");
        assert!(msg.contains("b.md"), "{msg}");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_parse_failure_names_the_file() {
        let dir = scratch("badparse");
        write(&dir, "bad.md", "no frontmatter here\n");

        let err = load(&dir).unwrap_err();
        assert!(err.to_string().contains("bad.md"), "{err}");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dot_directories_are_not_vault_content() {
        let dir = scratch("dotdirs");
        write(&dir, "good.md", "---\nid: g\ntype: claim\ntitle: t\n---\n");
        write(&dir, ".git/objects/nope.md", "not a node at all\n");

        let g = load(&dir).unwrap();
        assert_eq!(g.nodes().count(), 1);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_missing_root_is_reported_not_treated_as_empty() {
        let missing = std::env::temp_dir().join("peira-definitely-not-here");
        let _ = fs::remove_dir_all(&missing);
        let err = load(&missing).unwrap_err();
        assert!(matches!(err, VaultError::NotADirectory(_)));
    }

    #[test]
    fn every_vault_error_renders_and_names_its_subject() {
        let cases = [
            VaultError::NotADirectory(PathBuf::from("/nope")),
            VaultError::Io {
                path: PathBuf::from("/some/file.md"),
                source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
            },
            VaultError::Parse {
                path: PathBuf::from("/some/bad.md"),
                source: ParseError::MissingFrontmatter,
            },
            VaultError::DuplicateId {
                id: NodeId::new("x"),
                first: PathBuf::from("/a.md"),
                second: PathBuf::from("/b.md"),
            },
        ];
        for err in cases {
            let rendered = err.to_string();
            assert!(rendered.len() > 15, "too terse: {rendered}");
            let _: &dyn std::error::Error = &err;
        }
        assert!(VaultError::NotADirectory(PathBuf::from("/nope"))
            .to_string()
            .contains("/nope"));
        assert!(VaultError::Io {
            path: PathBuf::from("/some/file.md"),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        }
        .to_string()
        .contains("denied"));
    }

    #[test]
    fn an_inline_spec_may_propose_without_settling() {
        let e = edge_from_spec(&NodeId::new("o1"), "c1 proposed=G3", EdgeKind::Supports);
        assert_eq!(e.grade_proposed, Some(Grade::G3));
        assert_eq!(e.grade(), None);
    }

    #[test]
    fn unknown_and_malformed_spec_tokens_are_ignored_not_fatal() {
        // A note is hand-written. A typo in one token must not lose the edge.
        let e = edge_from_spec(
            &NodeId::new("o1"),
            "c1 colour=blue grade=NOTAGRADE via=telepathy bare",
            EdgeKind::Supports,
        );
        assert_eq!(e.to, NodeId::new("c1"), "the edge itself survives");
        assert_eq!(e.grade(), None, "an unparseable grade settles nothing");
        assert_eq!(
            e.pramana, None,
            "an unrecognised means of knowing is dropped"
        );
    }

    #[test]
    fn an_empty_spec_yields_an_edge_to_an_empty_id_which_lints_as_dangling() {
        let e = edge_from_spec(&NodeId::new("o1"), "", EdgeKind::Supports);
        assert_eq!(e.to, NodeId::new(""));

        // THE HALF AFTER "which". The name promised the edge lints as dangling and
        // nothing checked it, so a plausible hygiene filter — skip edges with an empty
        // id — would have silently swallowed the very edge this test exists to catch,
        // with the suite green and the name still making the promise.
        let mut g = crate::Graph::new();
        g.insert_node(
            crate::parse_node("---\nid: o1\ntype: observation\ntitle: t\n---\n")
                .expect("fixture parses"),
        );
        g.insert_edge(e);
        assert_eq!(
            g.dangling_edges().len(),
            1,
            "an edge to an empty id points at nothing, and must be reported as such"
        );
    }

    /// An empty `by=` is not a grader.
    ///
    /// `by=` with nothing after it built a grade "settled" by the empty string: both the
    /// ungraded-support and unreviewed-grade lints stayed silent, and the packet printed
    /// "Credited: , albert". The invariant this breaks is the one the type exists for.
    #[test]
    fn an_empty_grader_settles_nothing() {
        let e = edge_from_spec(
            &NodeId::new("o1"),
            "c1 grade=G2 by= via=perception",
            EdgeKind::Supports,
        );
        assert_eq!(e.grader(), None, "the empty string is not a reviewer");
        assert_eq!(
            e.grade(),
            None,
            "and a grade cannot be settled without one — the whole point of the pair"
        );

        let e = edge_from_spec(
            &NodeId::new("o1"),
            "c1 grade=G2 by=albert via=perception",
            EdgeKind::Supports,
        );
        assert_eq!(e.grader(), Some("albert"), "control");
    }

    #[test]
    fn a_grader_without_a_grade_settles_nothing() {
        let e = edge_from_spec(&NodeId::new("o1"), "c1 by=albert", EdgeKind::Supports);
        assert_eq!(e.grade(), None);
        assert_eq!(e.grader(), None);
    }

    #[test]
    fn nested_directories_are_walked() {
        let dir = scratch("nested");
        write(
            &dir,
            "a/b/c/deep.md",
            "---\nid: d\ntype: claim\ntitle: t\n---\n",
        );
        write(&dir, "top.md", "---\nid: t\ntype: claim\ntitle: t\n---\n");

        let g = load(&dir).unwrap();
        assert_eq!(g.nodes().count(), 2);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn non_markdown_files_are_left_alone() {
        let dir = scratch("nonmd");
        write(&dir, "keep.md", "---\nid: k\ntype: claim\ntitle: t\n---\n");
        write(&dir, "notes.txt", "not frontmatter at all");
        write(&dir, "data.json", "{}");

        let g = load(&dir).unwrap();
        assert_eq!(g.nodes().count(), 1);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_empty_vault_loads_as_an_empty_graph() {
        let dir = scratch("empty");
        let g = load(&dir).unwrap();
        assert_eq!(g.nodes().count(), 0);
        fs::remove_dir_all(&dir).unwrap();
    }
}
