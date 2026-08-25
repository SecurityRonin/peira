//! `peira-mcp` — the two checks that need no vault, over MCP stdio.
//!
//! Run it from an MCP client:
//!
//! ```jsonc
//! { "mcpServers": { "peira": { "command": "peira-mcp" } } }
//! ```
//!
//! READ-ONLY BY CONSTRUCTION. Neither tool opens a file, and there is no vault
//! argument to open one with. peira's value is that it refuses; a server that can
//! write is a server that can be talked into writing.

use std::path::Path;

use peira_core::NodeId;
use peira_mcp::{
    catalogue, check_prose, examine, freeze, gates, load_vault, propose, status, verify,
    ExamineReport, FreezeReport, GatesReport, LensCatalogue, ProposeReport, ProseReport,
    StatusReport, VerifyReport,
};
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt};
use serde::Deserialize;

/// Map a human-readable failure (bad vault path, unknown node) to an MCP error. A vault
/// that will not load or a node that does not exist is a bad request, not a finding —
/// distinct from a refusal-with-reasons, which is a Tier 4 concern.
fn bad_input(message: String) -> ErrorData {
    ErrorData::invalid_params(message, None)
}

#[derive(Deserialize, schemars::JsonSchema)]
struct CheckProseArgs {
    /// The prose to scan. A sentence, a paragraph, or a whole draft.
    text: String,
}

#[derive(Deserialize, schemars::JsonSchema, Default)]
struct LensArgs {
    /// A lens id such as `TRAIRUPYA`. Omit for the whole catalogue.
    id: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct VaultArgs {
    /// Path to the vault root directory. Read-only; never written.
    vault: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct ProposeArgs {
    /// The prose to turn into a claim skeleton — a sentence the author already wrote.
    prose: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct VaultNodeArgs {
    /// Path to the vault root directory. Read-only; never written.
    vault: String,
    /// The node id, e.g. `c-bounded`.
    node: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct VaultPacketArgs {
    /// Path to the vault root directory. Read-only; never written.
    vault: String,
    /// The stored packet document; its first line names the subject.
    packet: String,
}

#[derive(Debug, Clone, Default)]
struct Peira;

#[tool_router]
impl Peira {
    #[tool(
        name = "peira_check_prose",
        description = "Scan prose for the TWO failure modes peira can name without a \
vault: overstated verbs (an observation stated as a verdict) and ultimate-issue \
conclusions (a verdict word said of a party). Every other rule peira enforces — \
quantifier scope, causal rung, warrant, falsifier, boundaries — compares prose against a \
node's declared fields and cannot run on bare text. Returns each finding with the safe \
form named. An empty result means those two found nothing; it is NOT a finding that the \
text is sound."
    )]
    fn check_prose(Parameters(a): Parameters<CheckProseArgs>) -> Json<ProseReport> {
        Json(check_prose(&a.text))
    }

    #[tool(
        name = "peira_lens",
        description = "The catalogue of critical-thinking lenses peira mechanises — \
each naming a specific way of being wrong, where it was identified, and whether it \
REFUSES a claim or is only a reading. Omit `id` for all of them."
    )]
    fn lens(Parameters(a): Parameters<LensArgs>) -> Json<LensCatalogue> {
        Json(catalogue(a.id.as_deref()))
    }

    #[tool(
        name = "peira_examine",
        description = "Examine one claim in a vault: its derived standing \
(review_ready | contested | evidence_pending) and every gate and lint blocking it — the \
claim and everything it rests on. READ-ONLY; the vault is never written. A \
PEIR-GATE-UNASSESSED finding means a gate could NOT reach a verdict, which is never a \
pass. No claim is authored or graded."
    )]
    fn examine(Parameters(a): Parameters<VaultNodeArgs>) -> Result<Json<ExamineReport>, ErrorData> {
        let graph = load_vault(Path::new(&a.vault)).map_err(bad_input)?;
        examine(&graph, &NodeId::new(a.node))
            .map(Json)
            .map_err(bad_input)
    }

    #[tool(
        name = "peira_status",
        description = "The derived standing of one node — the same question `peira status` \
answers, computed from the graph and never set. review_ready means gates pass and the \
claim stands, but a human reviewer must still sign; peira does not. READ-ONLY."
    )]
    fn status(Parameters(a): Parameters<VaultNodeArgs>) -> Result<Json<StatusReport>, ErrorData> {
        let graph = load_vault(Path::new(&a.vault)).map_err(bad_input)?;
        status(&graph, &NodeId::new(a.node))
            .map(Json)
            .map_err(bad_input)
    }

    #[tool(
        name = "peira_gates",
        description = "Survey every gate and lint finding across a whole vault, each \
naming its subject. READ-ONLY. An empty list over a NON-EMPTY vault means nothing was \
found; an absent or empty vault is an error, not a clean result."
    )]
    fn gates(Parameters(a): Parameters<VaultArgs>) -> Result<Json<GatesReport>, ErrorData> {
        let graph = load_vault(Path::new(&a.vault)).map_err(bad_input)?;
        Ok(Json(gates(&graph)))
    }

    #[tool(
        name = "peira_freeze",
        description = "Freeze a Court-Mode citation packet for one claim, or report why \
it will NOT freeze. READ-ONLY — the packet is RETURNED, never written to the vault; saving \
it is the caller's decision. A refusal is a RESULT, not an error: outcome=blocked carries \
the gate findings in the way, outcome=defeated means the claim loses on the argument. A \
missing node or a non-claim is a bad request."
    )]
    fn freeze(Parameters(a): Parameters<VaultNodeArgs>) -> Result<Json<FreezeReport>, ErrorData> {
        let graph = load_vault(Path::new(&a.vault)).map_err(bad_input)?;
        freeze(&graph, &NodeId::new(a.node))
            .map(Json)
            .map_err(bad_input)
    }

    #[tool(
        name = "peira_verify",
        description = "Re-derive a stored packet from the vault as it stands and compare. \
READ-ONLY. Pass the packet document text; its first line names the subject. \
outcome=verified means byte-identical; digest_mismatch means the vault no longer renders \
it — NOT by itself an accusation, since a vault that GREW and one whose evidence was \
ALTERED look the same; no_longer_freezable means the claim stopped qualifying (a gate now \
blocks it, or it was defeated)."
    )]
    fn verify(Parameters(a): Parameters<VaultPacketArgs>) -> Result<Json<VerifyReport>, ErrorData> {
        let graph = load_vault(Path::new(&a.vault)).map_err(bad_input)?;
        verify(&graph, a.packet).map(Json).map_err(bad_input)
    }

    #[tool(
        name = "peira_propose",
        description = "Turn prose the author already wrote into a DRAFT claim skeleton to \
accept — no vault needed. It takes the proposition VERBATIM, infers the classification \
fields (quantifier, aspect, causal_rung) from the words and marks them to confirm, \
extracts only quoted candidate terms, and names every blank field with the gate that will \
demand it (fill the blanks → pass the gates). It authors NO evidence: there is no grade, \
by, or via, because those assert something was examined and nothing was. An over-claiming \
verb is surfaced in prose_findings."
    )]
    fn propose(Parameters(a): Parameters<ProposeArgs>) -> Json<ProposeReport> {
        Json(propose(&a.prose))
    }
}

// Written out rather than taking `#[tool_router(server_handler)]`'s generated
// handler, for one reason: the default `get_info` reports the SDK's name, so the
// server announced itself as "rmcp". A user browsing their installed servers would
// see the library, not the tool.
#[tool_handler]
impl ServerHandler for Peira {
    fn get_info(&self) -> ServerInfo {
        // Built by mutation: both structs are #[non_exhaustive], so a struct
        // expression will not compile even with `..default()`.
        let mut me = Implementation::default();
        me.name = "peira".into();
        me.version = env!("CARGO_PKG_VERSION").into();

        let mut info = ServerInfo::default();
        info.server_info = me;
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some(
            "peira names specific ways of being wrong, each drawn from a \
critical-thinking tradition that identified it. Call `peira_check_prose` on any draft \
before it reaches a reader — it needs no vault. An empty result is not an endorsement: \
the checks are narrow by design and say so. Call `peira_lens` to read the catalogue. \
With a vault, `peira_examine`, `peira_status` and `peira_gates` READ the graph and never \
write it: a claim's derived standing and what blocks it. `peira_freeze` renders a citation \
packet (returned, never written) or reports why it refuses; `peira_verify` checks a stored \
packet against the vault. `peira_propose` turns a draft sentence into a claim skeleton to \
accept — no vault, and no evidence authored."
                .into(),
        );
        info
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // stdout is the protocol channel. Anything written to it that is not JSON-RPC
    // corrupts the stream, so diagnostics go to stderr and nowhere else.
    let service = Peira.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
