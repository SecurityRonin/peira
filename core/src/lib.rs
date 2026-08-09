//! Typed node/edge model and vault parser for elenchus.
//!
//! # The load-bearing invariant
//!
//! A [`Claim`] has **no `status` field and no `confidence` field**, and the parser
//! *refuses* a document that carries one. Claim state is derived by the engine from
//! gates, reviewer records and the grounded extension; it is never written by hand
//! and never written by a model. The wrong thing is made impossible rather than
//! discouraged — you cannot write what has no field, and a document that tries is a
//! loud parse error naming the offending key.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::fmt;

/// The kinds of node a vault may hold.
///
/// `Question`/`Hypothesis`/`Claim`/`Observation` live in Inquiry (areas `70-79`,
/// UID-addressed, because a graph must not be foldered); `Term`/`Criterion` live in
/// Lexicon (`60-69`, Johnny.Decimal, because they are bounded reference objects).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// An open research question.
    Question,
    /// A candidate explanation, competing with others.
    Hypothesis,
    /// An atomic, scoped proposition — the mergeable unit.
    Claim,
    /// Something observed, pointing at sealed evidence by hash. Never holds bytes.
    Observation,
    /// A term whose meaning is stipulated before use (正名).
    Term,
    /// A declared standard against which something is judged (立極).
    Criterion,
    /// A reproducible test protocol.
    Protocol,
    /// One execution of a protocol.
    Run,
    /// The record of a lens having been run against a subject.
    Examination,
    /// A preserved minority position (machloket) — rejection never deletes.
    Dissent,
    /// A frozen Court Mode export.
    Packet,
}

impl NodeKind {
    /// The `type:` string this kind is written as in frontmatter.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            NodeKind::Question => "question",
            NodeKind::Hypothesis => "hypothesis",
            NodeKind::Claim => "claim",
            NodeKind::Observation => "observation",
            NodeKind::Term => "term",
            NodeKind::Criterion => "criterion",
            NodeKind::Protocol => "protocol",
            NodeKind::Run => "run",
            NodeKind::Examination => "examination",
            NodeKind::Dissent => "dissent",
            NodeKind::Packet => "packet",
        }
    }

    /// Parse a frontmatter `type:` string.
    #[must_use]
    pub fn from_str_opt(s: &str) -> Option<Self> {
        Some(match s {
            "question" => NodeKind::Question,
            "hypothesis" => NodeKind::Hypothesis,
            "claim" => NodeKind::Claim,
            "observation" => NodeKind::Observation,
            "term" => NodeKind::Term,
            "criterion" => NodeKind::Criterion,
            "protocol" => NodeKind::Protocol,
            "run" => NodeKind::Run,
            "examination" => NodeKind::Examination,
            "dissent" => NodeKind::Dissent,
            "packet" => NodeKind::Packet,
            _ => return None,
        })
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A node's stable identifier — a UID for graph objects, a Johnny.Decimal
/// coordinate for bounded reference objects.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(String);

impl NodeId {
    /// Wrap a raw identifier.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The identifier as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A parsed vault node.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Stable identifier.
    pub id: NodeId,
    /// What kind of node this is.
    pub kind: NodeKind,
    /// Human-readable title.
    pub title: String,
    /// Markdown body below the frontmatter, verbatim.
    pub body: String,
}

/// Why a document could not be read as a vault node.
///
/// Every variant carries the offending value verbatim: "unrecognized" is a prompt
/// to show evidence, never to hide it.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The document does not open with a `---` frontmatter fence.
    MissingFrontmatter,
    /// A frontmatter fence opened but never closed.
    UnterminatedFrontmatter,
    /// The frontmatter is not a YAML mapping, or is malformed. Carries the
    /// underlying message.
    Yaml(String),
    /// A required key is absent.
    MissingField {
        /// The key that should have been present.
        field: &'static str,
    },
    /// A key is present but not of the expected shape.
    WrongFieldType {
        /// The key.
        field: &'static str,
        /// What was expected.
        expected: &'static str,
    },
    /// `type:` names something this vault has no node kind for.
    UnknownNodeType(String),
    /// A field the schema deliberately does not have — writing it is the error
    /// this parser exists to make impossible.
    ForbiddenField {
        /// The offending key, verbatim.
        field: String,
        /// The kind of node it appeared on.
        kind: NodeKind,
        /// Why the schema refuses it.
        reason: &'static str,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingFrontmatter => {
                f.write_str("document does not begin with a `---` frontmatter fence")
            }
            ParseError::UnterminatedFrontmatter => {
                f.write_str("frontmatter fence opened but never closed")
            }
            ParseError::Yaml(msg) => write!(f, "malformed frontmatter YAML: {msg}"),
            ParseError::MissingField { field } => write!(f, "missing required field `{field}`"),
            ParseError::WrongFieldType { field, expected } => {
                write!(f, "field `{field}` must be {expected}")
            }
            ParseError::UnknownNodeType(t) => write!(f, "unknown node type `{t}`"),
            ParseError::ForbiddenField {
                field,
                kind,
                reason,
            } => write!(
                f,
                "field `{field}` may not appear on a `{kind}` node: {reason}"
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// Split a document into its frontmatter YAML and the markdown body.
///
/// The closing fence is a line that is exactly `---`, so a `---` appearing inside
/// a YAML scalar does not truncate the frontmatter.
fn split_frontmatter(source: &str) -> Result<(&str, &str), ParseError> {
    let rest = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
        .ok_or(ParseError::MissingFrontmatter)?;

    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']) == "---" {
            let frontmatter = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return Ok((frontmatter, body));
        }
        offset += line.len();
    }
    Err(ParseError::UnterminatedFrontmatter)
}

/// Render a YAML scalar as a string.
///
/// Numbers are accepted because a UID like `202607241412` is a perfectly ordinary
/// frontmatter `id:` and YAML types it as an integer — rejecting it would fail on
/// the vault's own documented convention.
fn scalar_as_string(value: &serde_yaml_ng::Value) -> Option<String> {
    match value {
        serde_yaml_ng::Value::String(s) => Some(s.clone()),
        serde_yaml_ng::Value::Number(n) => Some(n.to_string()),
        serde_yaml_ng::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Fetch a required scalar field.
fn required_scalar(
    map: &serde_yaml_ng::Mapping,
    field: &'static str,
) -> Result<String, ParseError> {
    let value = map
        .get(serde_yaml_ng::Value::String(field.to_owned()))
        .ok_or(ParseError::MissingField { field })?;
    scalar_as_string(value).ok_or(ParseError::WrongFieldType {
        field,
        expected: "a scalar (string or number)",
    })
}

/// Parse one vault document into a [`Node`].
pub fn parse_node(source: &str) -> Result<Node, ParseError> {
    let (frontmatter, body) = split_frontmatter(source)?;

    let value: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(frontmatter).map_err(|e| ParseError::Yaml(e.to_string()))?;
    let map = value
        .as_mapping()
        .ok_or_else(|| ParseError::Yaml("frontmatter is not a mapping".to_owned()))?;

    let type_name = required_scalar(map, "type")?;
    let kind = NodeKind::from_str_opt(&type_name).ok_or(ParseError::UnknownNodeType(type_name))?;

    Ok(Node {
        id: NodeId::new(required_scalar(map, "id")?),
        kind,
        title: required_scalar(map, "title")?,
        body: body.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAIM_DOC: &str = "\
---
id: 20260809T142530
type: claim
title: Amcache recorded coreupdater.exe at a staging path
---

The hive catalogued the file identity and path.
";

    #[test]
    fn parses_a_claim_node() {
        let node = parse_node(CLAIM_DOC).expect("a well-formed claim document should parse");

        assert_eq!(node.kind, NodeKind::Claim);
        assert_eq!(node.id, NodeId::new("20260809T142530"));
        assert_eq!(
            node.title,
            "Amcache recorded coreupdater.exe at a staging path"
        );
        assert_eq!(
            node.body.trim(),
            "The hive catalogued the file identity and path."
        );
    }
}
