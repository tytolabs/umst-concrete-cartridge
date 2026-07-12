// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `umst-mcp-rmcp` — official `rmcp` stdio MCP binary (Stage S3, default off).
//!
//! Hand-rolled `umst-mcp` remains the default binary (`default-run = "umst-mcp"`).
//! GO-LIVE Step 3 enables `agent-layer` by default; `rmcp-wire` stays opt-in.

use rmcp::{transport::stdio, ServiceExt};
use umst_mcp::rmcp_server::UmstRmcpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("UMST MCP server (rmcp stdio, parity tools).");

    let server = UmstRmcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
