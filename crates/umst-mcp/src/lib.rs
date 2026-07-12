// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `umst-mcp` library surface — pure helpers referenced by agent docs and tests.

pub mod handlers;
pub mod parity;
pub mod soft_gate;

#[cfg(feature = "agent-layer")]
pub mod agent_layer;

#[cfg(feature = "rmcp-wire")]
pub mod rmcp_server;

#[cfg(feature = "tool-manifest")]
pub mod manifest;
