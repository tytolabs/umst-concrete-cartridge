// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Shared CLI logic (`predict`, wire JSON, certify) used by the `umst` binary and integration tests.

pub mod audit;
pub mod canonical;
pub mod cli;
pub mod ops_host;

#[cfg(feature = "agent-layer")]
pub mod memory_export;
#[cfg(feature = "agent-layer")]
pub mod promote;
#[cfg(feature = "agent-layer")]
pub mod propose_promotion;
