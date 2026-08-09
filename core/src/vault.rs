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

/// Edge-bearing frontmatter keys, and the edge each produces.
const EDGE_KEYS: &[EdgeKind] = &[
    EdgeKind::Supports,
    EdgeKind::Contradicts,
    EdgeKind::Limits,
    EdgeKind::Duplicates,
    EdgeKind::DependsOn,
    EdgeKind::Supersedes,
    EdgeKind::Retracts,
    EdgeKind::IsA,
    EdgeKind::HasA,
    EdgeKind::InstanceOf,
    EdgeKind::PartOf,
    EdgeKind::SubstanceOf,
    EdgeKind::FunctionOf,
    EdgeKind::Negates,
    EdgeKind::Sublates,
    EdgeKind::Attacks,
    EdgeKind::JudgedBy,
    EdgeKind::UsesTerm,
    EdgeKind::Examines,
];

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
            "by" => grader = Some(value.to_owned()),
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
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Load every markdown document under `root` into a graph.
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

        for kind in EDGE_KEYS {
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
        let dir = std::env::temp_dir().join(format!("elenchus-vault-test-{name}"));
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
        let missing = std::env::temp_dir().join("elenchus-definitely-not-here");
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
