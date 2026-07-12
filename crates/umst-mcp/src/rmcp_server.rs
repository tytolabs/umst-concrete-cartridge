// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Official `rmcp` MCP server (Stage S3) — stdio transport, golden-tool parity subset.
//!
//! ADDITIVE — compiled only with feature `rmcp-wire` (default off).

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, ContentBlock, ErrorData as McpError, Implementation,
        ListToolsResult, ServerCapabilities, ServerInfo, Tool,
    },
    service::{RequestContext, RoleServer},
};
use serde_json::{json, Map, Value};

use crate::handlers::{
    exec_umst_gate_check_pure, exec_umst_mi_estimate_pure, exec_umst_predict, exec_umst_profiles,
    ToolPayload, PARITY_TOOL_NAMES,
};

/// `rmcp` MCP server — exposes S0 golden parity tools via official SDK.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Wire adapter; physics on shared `handlers` / cartridge.
#[derive(Clone, Default)]
pub struct UmstRmcpServer {
    tools: Arc<Vec<Tool>>,
}

impl UmstRmcpServer {
    /// Construct server with parity tool manifest.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Composition root for `umst-mcp-rmcp` binary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: Arc::new(parity_rmcp_tools()),
        }
    }

    fn payload_to_call_result(payload: ToolPayload) -> CallToolResult {
        let block = ContentBlock::text(payload.text);
        if payload.is_error {
            CallToolResult::error(vec![block])
        } else {
            CallToolResult::success(vec![block])
        }
    }
}

impl ServerHandler for UmstRmcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_server_info(
            Implementation::new("umst-concrete-cartridge-rmcp", env!("CARGO_PKG_VERSION")),
        )
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let tools = self.tools.clone();
        async move {
            Ok(ListToolsResult {
                tools: (*tools).clone(),
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        let name = request.name.to_string();
        let args = json_object_to_value(request.arguments);
        async move {
            if !PARITY_TOOL_NAMES.contains(&name.as_str()) {
                return Err(McpError::method_not_found::<
                    rmcp::model::CallToolRequestMethod,
                >());
            }
            let payload = match name.as_str() {
                "umst_profiles" => exec_umst_profiles(),
                "umst_predict" => exec_umst_predict(&args),
                "umst_gate_check" => exec_umst_gate_check_pure(&args),
                "umst_mi_estimate" => exec_umst_mi_estimate_pure(&args),
                _ => unreachable!("checked above"),
            };
            Ok(Self::payload_to_call_result(payload))
        }
    }
}

fn json_object_to_value(obj: Option<Map<String, Value>>) -> Value {
    match obj {
        Some(map) => Value::Object(map),
        None => json!({}),
    }
}

/// Build `rmcp` `Tool` descriptors for S0 golden parity subset.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: MCP list_tools wire; schemas match hand-rolled subset.
#[must_use]
pub fn parity_rmcp_tools() -> Vec<Tool> {
    parity_tool_schema_values()
        .iter()
        .map(|schema| {
            let name = schema["name"].as_str().expect("tool name").to_string();
            let description = schema["description"].as_str().unwrap_or("").to_string();
            let input = schema["inputSchema"]
                .as_object()
                .expect("inputSchema object")
                .clone();
            Tool::new(Cow::Owned(name), Cow::Owned(description), Arc::new(input))
        })
        .collect()
}

fn parity_tool_schema_values() -> Vec<Value> {
    vec![
        json!({
            "name": "umst_profiles",
            "description": "List bundled calibration profile ids sorted lexicographically with descriptions.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "umst_predict",
            "description": "Constitutive prediction envelope result.v2 (read-only).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mix": { "type": "object" },
                    "profile": { "type": "string", "default": "default" },
                    "compare_homogeneous": { "type": "boolean", "default": false },
                    "schema_version": { "type": "string", "enum": ["v1", "v2"], "default": "v2" },
                    "canonical": { "type": "boolean", "default": false }
                },
                "required": ["mix"]
            }
        }),
        json!({
            "name": "umst_gate_check",
            "description": "Hard thermodynamic admissibility check (read-only).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mix": { "type": "object" },
                    "profile": { "type": "string", "default": "default" },
                    "explain": { "type": "boolean", "default": true }
                },
                "required": ["mix"]
            }
        }),
        json!({
            "name": "umst_mi_estimate",
            "description": "Advisory MI estimate (read-only).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mix": { "type": "object" }
                },
                "required": ["mix"]
            }
        }),
    ]
}
