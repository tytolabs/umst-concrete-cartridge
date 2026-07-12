// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `umst-mcp-rmcp` — official `rmcp` stdio MCP binary (Stage S3, default off).
//!
//! GO-LIVE Step 2 (HELD): package `default-run = "umst-mcp-rmcp"` with `default = ["rmcp-wire"]`.
//! Hand-rolled `umst-mcp` binary remains available; soak decision is USER-gated.

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
