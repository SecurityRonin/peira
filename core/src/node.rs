//! Vault node model and frontmatter parser.

use std::fmt;

/// The kinds of node a vault may hold.
///
/// `Question`/`Hypothesis`/`Claim`/`Observation` live in Inquiry (areas `70-79`,
/// UID-addressed, because a graph must not be foldered); `Term`/`Criterion` live in
/// Lexicon (`60-69`, Johnny.Decimal, because they are bounded reference objects).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    /// Whether this kind is an argument in the argumentation graph — something that
    /// can attack, be attacked, and therefore be in or out of the grounded extension.
    ///
    /// A `Term` or `Criterion` is reference material: it is *used by* arguments and
    /// never competes with them.
    #[must_use]
    pub fn is_argument(self) -> bool {
        matches!(
            self,
            NodeKind::Claim | NodeKind::Hypothesis | NodeKind::Observation | NodeKind::Dissent
        )
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
///
/// `fields` keeps the remaining frontmatter as scalars so lens gates can ask about
/// `warrant`, `causal_rung`, `boundaries` and the rest without this crate having to
/// know every lens's schema in advance.
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
    /// Remaining frontmatter keys, rendered as scalars or lists of scalars.
    pub fields: Fields,
}

impl Node {
    /// A scalar frontmatter field, if present and non-empty after trimming.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.scalar(name)
    }

    /// A list-valued frontmatter field. A lone scalar reads as a one-element list,
    /// because `boundaries: windows-10` and `boundaries: [windows-10]` mean the same
    /// thing to whoever wrote the note.
    #[must_use]
    pub fn field_list(&self, name: &str) -> Vec<&str> {
        self.fields.list(name)
    }
}

/// Frontmatter keys beyond the common ones, flattened to strings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fields {
    entries: Vec<(String, Vec<String>)>,
}

impl Fields {
    /// Insert a key with its rendered values.
    pub fn insert(&mut self, key: impl Into<String>, values: Vec<String>) {
        self.entries.push((key.into(), values));
    }

    /// The first value for `name`, if it is present and not blank.
    #[must_use]
    pub fn scalar(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == name)
            .and_then(|(_, v)| v.first())
            .map(String::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    /// All values for `name`, blanks removed.
    #[must_use]
    pub fn list(&self, name: &str) -> Vec<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| {
                v.iter()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether the key was written at all, even if blank. Distinguishes "absent"
    /// from "present but empty" — a blank `warrant:` is a different defect from a
    /// missing one, and the gates should be able to say which.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|(k, _)| k == name)
    }

    /// Every key present, in document order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(k, _)| k.as_str())
    }
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

/// Render a YAML value as a list of strings: a scalar becomes one element, a
/// sequence becomes its scalar members, anything else becomes empty.
fn values_of(value: &serde_yaml_ng::Value) -> Vec<String> {
    match value {
        serde_yaml_ng::Value::Sequence(items) => {
            items.iter().filter_map(scalar_as_string).collect()
        }
        other => scalar_as_string(other).into_iter().collect(),
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

/// Fields no node may carry, because the engine derives them.
///
/// This table is the whole doer/checker split, made structural. A model — or a
/// hurried human — can propose nodes and edges all day and still never assert that
/// a claim is accepted, because there is nowhere to write it. Keeping the rule
/// general rather than per-kind is deliberate: a special case here would be a
/// confession that the model is wrong.
const DERIVED_FIELDS: &[(&str, &str)] = &[
    (
        "status",
        "claim state is derived from gates, reviewer records and the grounded extension",
    ),
    (
        "confidence",
        "confidence is a reviewed assessment of the whole current graph, computed on demand",
    ),
];

/// Reject any frontmatter key the engine owns.
fn reject_derived_fields(map: &serde_yaml_ng::Mapping, kind: NodeKind) -> Result<(), ParseError> {
    for (field, reason) in DERIVED_FIELDS {
        if map.contains_key(serde_yaml_ng::Value::String((*field).to_owned())) {
            return Err(ParseError::ForbiddenField {
                field: (*field).to_owned(),
                kind,
                reason,
            });
        }
    }
    Ok(())
}

/// Keys consumed into [`Node`]'s own fields rather than into [`Fields`].
const COMMON_KEYS: &[&str] = &["id", "type", "title"];

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

    reject_derived_fields(map, kind)?;

    let mut fields = Fields::default();
    for (key, val) in map {
        let Some(key) = scalar_as_string(key) else {
            continue;
        };
        if COMMON_KEYS.contains(&key.as_str()) {
            continue;
        }
        fields.insert(key, values_of(val));
    }

    Ok(Node {
        id: NodeId::new(required_scalar(map, "id")?),
        kind,
        title: required_scalar(map, "title")?,
        body: body.to_owned(),
        fields,
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
warrant: A catalogued path is evidence of presence.
boundaries:
  - Windows 10 1809+
  - Amcache.hve InventoryApplicationFile
---

The hive catalogued the file identity and path.
";

    const CLAIM_WITH_STATUS: &str = "\
---
id: 20260809T142530
type: claim
title: Amcache recorded coreupdater.exe at a staging path
status: accepted
---

Body.
";

    const CLAIM_WITH_CONFIDENCE: &str = "\
---
id: 20260809T142530
type: claim
title: Amcache recorded coreupdater.exe at a staging path
confidence: 0.9
---

Body.
";

    #[test]
    fn refuses_a_node_carrying_a_hand_written_status() {
        match parse_node(CLAIM_WITH_STATUS) {
            Err(ParseError::ForbiddenField { field, kind, .. }) => {
                assert_eq!(field, "status");
                assert_eq!(kind, NodeKind::Claim);
            }
            other => panic!("a hand-written `status` must be refused, got {other:?}"),
        }
    }

    #[test]
    fn refuses_a_node_carrying_a_hand_written_confidence() {
        match parse_node(CLAIM_WITH_CONFIDENCE) {
            Err(ParseError::ForbiddenField { field, kind, .. }) => {
                assert_eq!(field, "confidence");
                assert_eq!(kind, NodeKind::Claim);
            }
            other => panic!("a hand-written `confidence` must be refused, got {other:?}"),
        }
    }

    #[test]
    fn forbidden_field_error_names_the_offending_key_verbatim() {
        let Err(err) = parse_node(CLAIM_WITH_STATUS) else {
            panic!("expected a refusal");
        };
        let rendered = err.to_string();
        assert!(
            rendered.contains("status"),
            "the error must show the offending key verbatim, got: {rendered}"
        );
    }

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

    #[test]
    fn keeps_extra_frontmatter_as_queryable_fields() {
        let node = parse_node(CLAIM_DOC).unwrap();

        assert_eq!(
            node.field("warrant"),
            Some("A catalogued path is evidence of presence.")
        );
        assert_eq!(
            node.field_list("boundaries"),
            vec!["Windows 10 1809+", "Amcache.hve InventoryApplicationFile"]
        );
        assert_eq!(node.field("nonesuch"), None);
    }

    #[test]
    fn a_lone_scalar_reads_as_a_one_element_list() {
        let doc = "\
---
id: 1
type: claim
title: t
boundaries: Windows 11 only
---
";
        let node = parse_node(doc).unwrap();
        assert_eq!(node.field_list("boundaries"), vec!["Windows 11 only"]);
    }

    #[test]
    fn distinguishes_a_blank_field_from_an_absent_one() {
        let doc = "\
---
id: 1
type: claim
title: t
warrant: \"\"
---
";
        let node = parse_node(doc).unwrap();
        assert!(node.fields.contains("warrant"), "written, though blank");
        assert_eq!(node.field("warrant"), None, "blank is not a value");
        assert!(!node.fields.contains("causal_rung"), "never written at all");
    }

    #[test]
    fn a_dashes_line_inside_a_scalar_does_not_truncate_frontmatter() {
        let doc = "\
---
id: 1
type: claim
title: \"a title with --- inside\"
warrant: still parsed
---

Body.
";
        let node = parse_node(doc).unwrap();
        assert_eq!(node.field("warrant"), Some("still parsed"));
    }

    #[test]
    fn reports_an_unknown_node_type_verbatim() {
        let doc = "---\nid: 1\ntype: assertion\ntitle: t\n---\n";
        match parse_node(doc) {
            Err(ParseError::UnknownNodeType(t)) => assert_eq!(t, "assertion"),
            other => panic!("expected UnknownNodeType, got {other:?}"),
        }
    }

    #[test]
    fn rejects_a_document_with_no_frontmatter() {
        assert_eq!(
            parse_node("just a body\n"),
            Err(ParseError::MissingFrontmatter)
        );
    }

    #[test]
    fn rejects_an_unterminated_frontmatter_fence() {
        assert_eq!(
            parse_node("---\nid: 1\ntype: claim\n"),
            Err(ParseError::UnterminatedFrontmatter)
        );
    }
}
