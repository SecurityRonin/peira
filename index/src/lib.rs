//! A derived index over a vault.
//!
//! # This index is never authoritative
//!
//! The markdown is the source of truth. This database is **rebuilt from scratch**
//! by `peira index`, is gitignored, and is dropped and recreated on every build
//! rather than updated incrementally.
//!
//! That is a deliberate constraint, not laziness. An incrementally-updated index
//! can drift from the files it describes, and once it can drift it becomes a second
//! source of truth that nobody reviewed. It also keeps every CI gate satisfiable
//! from committed bytes alone: delete the database and nothing is lost.
//!
//! What it buys is query. Once a vault outgrows what fits in a terminal, questions
//! like *which claims lack boundaries*, *which edges rest on testimony*, or *what is
//! out of the grounded extension and why* are SQL rather than a re-scan.

use peira_core::{Graph, NodeId};
use peira_lens::examine_graph;
use rusqlite::{params, Connection};
use std::path::Path;

/// Rebuild the index at `path` from `graph`, replacing whatever was there.
///
/// # Errors
/// Propagates any `SQLite` failure. A partially-built index is never left behind:
/// the whole build runs in one transaction.
pub fn build(graph: &Graph, path: &Path) -> rusqlite::Result<()> {
    let mut conn = Connection::open(path)?;
    let tx = conn.transaction()?;

    // Dropped rather than migrated. The index is derived, so a schema change is a
    // rebuild, and there is nothing here worth preserving across one.
    tx.execute_batch(
        "DROP TABLE IF EXISTS violations;
         DROP TABLE IF EXISTS grounded;
         DROP TABLE IF EXISTS edges;
         DROP TABLE IF EXISTS fields;
         DROP TABLE IF EXISTS nodes;

         CREATE TABLE nodes (
             id     TEXT PRIMARY KEY,
             kind   TEXT NOT NULL,
             title  TEXT NOT NULL,
             body   TEXT NOT NULL
         );
         CREATE TABLE fields (
             node_id  TEXT NOT NULL REFERENCES nodes(id),
             key      TEXT NOT NULL,
             value    TEXT NOT NULL,
             ordinal  INTEGER NOT NULL
         );
         CREATE TABLE edges (
             from_id    TEXT NOT NULL,
             to_id      TEXT NOT NULL,
             kind       TEXT NOT NULL,
             grade      TEXT,
             graded_by  TEXT,
             proposed   TEXT,
             pramana    TEXT
         );
         CREATE TABLE grounded (
             node_id TEXT PRIMARY KEY
         );
         CREATE TABLE violations (
             gate     TEXT NOT NULL,
             lens     TEXT NOT NULL,
             subject  TEXT NOT NULL,
             detail   TEXT NOT NULL,
             remedy   TEXT NOT NULL
         );
         CREATE INDEX idx_fields_key ON fields(key);
         CREATE INDEX idx_edges_to   ON edges(to_id);
         CREATE INDEX idx_edges_from ON edges(from_id);
         CREATE INDEX idx_viol_subj  ON violations(subject);",
    )?;

    write_nodes(&tx, graph)?;
    write_edges(&tx, graph)?;
    write_derived(&tx, graph)?;

    tx.commit()
}

/// Nodes and their flattened frontmatter.
fn write_nodes(tx: &rusqlite::Transaction<'_>, graph: &Graph) -> rusqlite::Result<()> {
    {
        let mut node_stmt =
            tx.prepare("INSERT INTO nodes (id, kind, title, body) VALUES (?1, ?2, ?3, ?4)")?;
        let mut field_stmt = tx
            .prepare("INSERT INTO fields (node_id, key, value, ordinal) VALUES (?1, ?2, ?3, ?4)")?;

        for node in graph.nodes() {
            node_stmt.execute(params![
                node.id.as_str(),
                node.kind.as_str(),
                node.title,
                node.body
            ])?;
            for key in node.fields.keys() {
                for (i, value) in node.field_list(key).iter().enumerate() {
                    field_stmt.execute(params![
                        node.id.as_str(),
                        key,
                        value,
                        i64::try_from(i).unwrap_or(i64::MAX)
                    ])?;
                }
            }
        }
    }
    Ok(())
}

/// Edges, with their grades and means of knowing.
fn write_edges(tx: &rusqlite::Transaction<'_>, graph: &Graph) -> rusqlite::Result<()> {
    {
        let mut edge_stmt = tx.prepare(
            "INSERT INTO edges (from_id, to_id, kind, grade, graded_by, proposed, pramana)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )?;
        for edge in graph.edges() {
            edge_stmt.execute(params![
                edge.from.as_str(),
                edge.to.as_str(),
                edge.kind.as_str(),
                edge.grade().map(peira_core::Grade::as_str),
                edge.grader(),
                edge.grade_proposed.map(peira_core::Grade::as_str),
                edge.pramana.map(peira_core::Pramana::as_str),
            ])?;
        }
    }
    Ok(())
}

/// What the engine computed: the grounded extension, and every violation.
fn write_derived(tx: &rusqlite::Transaction<'_>, graph: &Graph) -> rusqlite::Result<()> {
    {
        let mut grounded_stmt = tx.prepare("INSERT INTO grounded (node_id) VALUES (?1)")?;
        for id in graph.grounded_extension() {
            grounded_stmt.execute(params![id.as_str()])?;
        }

        let mut viol_stmt = tx.prepare(
            "INSERT INTO violations (gate, lens, subject, detail, remedy)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for v in examine_graph(graph)
            .into_iter()
            .chain(peira_citation::all_findings(graph))
        {
            viol_stmt.execute(params![
                v.gate,
                v.lens,
                v.subject.as_str(),
                v.detail,
                v.remedy
            ])?;
        }
    }
    Ok(())
}

/// Claims blocked by at least one gate, worst first.
///
/// # Errors
/// Propagates any `SQLite` failure.
pub fn blocked_claims(conn: &Connection) -> rusqlite::Result<Vec<(NodeId, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT v.subject, COUNT(*) AS n
           FROM violations v
           JOIN nodes n ON n.id = v.subject
          WHERE n.kind = 'claim'
       GROUP BY v.subject
       ORDER BY n DESC, v.subject",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((NodeId::new(row.get::<_, String>(0)?), row.get::<_, i64>(1)?))
    })?;
    rows.collect()
}

/// Claims that are out of the grounded extension.
///
/// # Errors
/// Propagates any `SQLite` failure.
pub fn defeated_claims(conn: &Connection) -> rusqlite::Result<Vec<NodeId>> {
    let mut stmt = conn.prepare(
        "SELECT n.id FROM nodes n
          WHERE n.kind IN ('claim', 'hypothesis')
            AND n.id NOT IN (SELECT node_id FROM grounded)
       ORDER BY n.id",
    )?;
    let rows = stmt.query_map([], |row| Ok(NodeId::new(row.get::<_, String>(0)?)))?;
    rows.collect()
}

/// Edges resting on testimony — the ones whose independence needs checking.
///
/// # Errors
/// Propagates any `SQLite` failure.
pub fn testimony_edges(conn: &Connection) -> rusqlite::Result<Vec<(NodeId, NodeId)>> {
    let mut stmt = conn.prepare(
        "SELECT from_id, to_id FROM edges
          WHERE pramana = 'testimony'
       ORDER BY from_id, to_id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((
            NodeId::new(row.get::<_, String>(0)?),
            NodeId::new(row.get::<_, String>(1)?),
        ))
    })?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use peira_core::load;
    use std::path::PathBuf;

    fn vault(name: &str) -> Graph {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tests")
            .join("vaults")
            .join(name);
        load(&root).unwrap_or_else(|e| panic!("loading {name}: {e}"))
    }

    /// Each test gets its own database file. Sharing one path across tests that
    /// cargo runs in parallel produces `database is locked`, which reads as a bug
    /// in the code under test rather than in the harness.
    fn built(vault_name: &str, tag: &str) -> Connection {
        let path = std::env::temp_dir().join(format!("peira-index-{vault_name}-{tag}.sqlite"));
        let _ = std::fs::remove_file(&path);
        let graph = vault(vault_name);
        build(&graph, &path).expect("index builds");
        Connection::open(&path).expect("opens")
    }

    #[test]
    fn indexes_the_bounded_vault_with_no_violations() {
        let conn = built("bounded", "clean");
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM violations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "the bounded vault must index clean");

        let nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nodes, 6);
    }

    #[test]
    fn the_overclaim_shows_up_as_the_most_blocked_claim() {
        let conn = built("overclaim", "blocked");
        let blocked = blocked_claims(&conn).unwrap();
        assert_eq!(
            blocked.first().map(|(id, _)| id.as_str()),
            Some("c-overclaim")
        );
        assert!(
            blocked[0].1 >= 5,
            "expected at least five findings, got {}",
            blocked[0].1
        );

        // ORDERING NEEDS SOMETHING TO ORDER. With one row, "most blocked" is satisfied
        // by any ORDER BY at all — inverting it to ASC left this green. Assert the
        // sequence is descending across every row, so the claim in the test's name is
        // the thing being measured.
        // ORDERING NEEDS SOMETHING TO ORDER. The overclaim vault holds exactly one
        // blocked claim, so "most blocked" was satisfied by any ORDER BY at all —
        // inverting it to ASC left this green. A second claim, deliberately groomed to
        // draw FEWER findings, makes the sequence the thing under test.
        let mut g = vault("overclaim");
        g.insert_node(
            peira_core::parse_node(
                "---\nid: c-lighter\ntype: claim\ntitle: A narrower reading of the record\nwarrant: The register records what it is given.\nquantifier: singular\naspect: function\ncausal_rung: association\nno_terms_of_art: true\n---\n",
            )
            .expect("fixture parses"),
        );
        let path = std::env::temp_dir().join(format!(
            "peira-index-ordering-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        build(&g, &path).expect("index builds");
        let conn = Connection::open(&path).expect("opens");
        let blocked = blocked_claims(&conn).unwrap();

        assert!(
            blocked.len() >= 2,
            "a one-row result cannot test an ordering: {blocked:?}"
        );
        assert!(
            blocked.windows(2).all(|w| w[0].1 >= w[1].1),
            "worst first, and it is not: {blocked:?}"
        );
    }

    #[test]
    fn grounded_membership_is_materialised() {
        let conn = built("bounded", "grounded");
        let defeated = defeated_claims(&conn).unwrap();
        assert!(
            !defeated.iter().any(|id| id.as_str() == "c-bounded"),
            "the bounded claim survives; got {defeated:?}"
        );

        // AND A POSITIVE. Asserting only that something is absent passes for a query
        // that returns nothing at all — `AND 1 = 0` in the selector left this green.
        // A negative assertion needs a positive beside it or it is measuring silence.
        let conn = built("overclaim", "grounded-positive");
        let defeated = defeated_claims(&conn).unwrap();
        assert!(
            !defeated.is_empty(),
            "the overclaim vault records a defeated claim; a selector matching nothing \
would satisfy the assertion above and this one catches it"
        );
    }

    #[test]
    fn testimony_edges_are_queryable() {
        let conn = built("bounded", "testimony");
        let edges = testimony_edges(&conn).unwrap();
        assert_eq!(
            edges,
            vec![(NodeId::new("o2"), NodeId::new("c-bounded"))],
            "the parser-output observation is testimony, and only it"
        );
    }

    /// H7 — the violations table is documented as "claims blocked by at least one
    /// gate", and the lint pack is half of what blocks. Dropping
    /// `.chain(lints::lint(graph))` from `build` lost every lint finding from the
    /// table with the whole suite green, because every index test was satisfied by
    /// gate findings alone.
    ///
    /// Asserted against the BUILT DATABASE, never by re-evaluating the same chain
    /// expression here — a test that reuses the implementation's own predicate cannot
    /// detect a defect in it, which is how the first version of this test was wrong.
    #[test]
    fn the_violations_table_carries_lint_findings_not_only_gate_findings() {
        let path = std::env::temp_dir().join(format!(
            "peira-index-lints-in-table-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let graph = vault("overclaim");
        build(&graph, &path).expect("index builds");
        let conn = Connection::open(&path).expect("opens");

        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM violations WHERE gate LIKE 'PEIR-LINT-%'",
                [],
                |r| r.get(0),
            )
            .expect("counts");
        assert!(
            n > 0,
            "the built index holds no lint findings — the lint pack is unwired from \
`build`, and the table documented as what blocks a claim is showing gates alone"
        );
        let _ = std::fs::remove_file(&path);
    }

    /// E3 — the index must ask the SAME question the CLI does about what blocks.
    ///
    /// The violations table is documented as "claims blocked by at least one gate".
    /// It was built from `examine_graph + lints::lint` while the CLI additionally
    /// consolidated what a packet would SEAL — the node-level pack skips a term's
    /// moments, and a packet quotes them verbatim. Two implementations of one
    /// question, and the derived index held the narrower one.
    ///
    /// Asserted as a count against `court::all_findings`, because the extra finding
    /// shares a gate and subject with one the node pack already raises and differs
    /// only in its detail — a set comparison on gate+subject would not have seen it.
    #[test]
    fn the_violations_table_asks_the_same_question_the_cli_does() {
        let path = std::env::temp_dir().join(format!(
            "peira-index-same-question-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let graph = vault("overclaim");
        build(&graph, &path).expect("index builds");
        let conn = Connection::open(&path).expect("opens");

        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM violations", [], |r| r.get(0))
            .expect("counts");
        let expected = examine_graph(&graph).len() + peira_citation::all_findings(&graph).len();
        assert_eq!(
            rows as usize, expected,
            "the index's violations table has drifted from what the CLI reports as \
blocking — it is re-deriving a narrower question"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rebuilding_over_an_existing_index_replaces_rather_than_accumulates() {
        let path = std::env::temp_dir().join(format!(
            "peira-index-test-rebuild-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let graph = vault("bounded");

        build(&graph, &path).unwrap();
        build(&graph, &path).unwrap();

        let conn = Connection::open(&path).unwrap();
        let nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nodes, 6, "a second build must replace, not double up");
    }
}
