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

use peira_mcp::{catalogue, check_prose, LensEntry, ProseReport};
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{schemars, tool, tool_handler, tool_router, ServerHandler, ServiceExt};
use serde::Deserialize;

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

#[derive(Debug, Clone, Default)]
struct Peira;

#[tool_router]
impl Peira {
    #[tool(
        name = "peira_check_prose",
        description = "Scan prose for the failure modes peira names: overstated verbs \
(an observation stated as a verdict), ultimate-issue conclusions that decide the \
tribunal's question, unbounded quantifiers, and hedges that do not reach the claim. \
Needs no vault. Returns each finding with the safe form named. An empty result means \
the scan found nothing it knows how to name — it is NOT a finding that the text is sound."
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
    fn lens(Parameters(a): Parameters<LensArgs>) -> Json<Vec<LensEntry>> {
        Json(catalogue(a.id.as_deref()))
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
the checks are narrow by design and say so. Call `peira_lens` to read the catalogue."
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
