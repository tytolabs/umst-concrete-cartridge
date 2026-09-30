// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Shared CLI logic (`predict`, wire JSON, certify) used by the `umst` binary and integration tests.

pub mod audit;
pub mod canonical;
pub mod cli;

#[cfg(feature = "agent-layer")]
pub mod memory_export;
#[cfg(feature = "agent-layer")]
pub mod promote;
#[cfg(feature = "agent-layer")]
pub mod propose_promotion;
