// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Agent contribution governance — scope tokens (pure validation against allowlist).

use thiserror::Error;

/// Scope token validation failures for `umst_contribute`.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Operator allowlist gate; not thermodynamic admissibility.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScopeError {
    #[error("scope_token required for umst_contribute")]
    Missing,
    #[error("scope_token not in allowlist")]
    Denied,
}

/// Pure: validate optional scope token against `UMST_AGENT_SCOPE_TOKENS` (comma-separated).
/// When `UMST_AGENT_SCOPE_ALLOW_ANY=1`, any non-empty token passes (dev only).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: YAML/env governance allowlist; physics on `gate_recheck` only.
pub fn validate_scope_token(token: Option<&str>) -> Result<(), ScopeError> {
    if std::env::var("UMST_AGENT_SCOPE_ALLOW_ANY")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"))
    {
        return Ok(());
    }

    let allowlist = std::env::var("UMST_AGENT_SCOPE_TOKENS").unwrap_or_default();
    if allowlist.is_empty() {
        // Production default: no token required until operator sets allowlist.
        return Ok(());
    }

    let token = token.filter(|t| !t.is_empty()).ok_or(ScopeError::Missing)?;
    let allowed = allowlist
        .split(',')
        .map(str::trim)
        .any(|entry| entry == token);
    if allowed {
        Ok(())
    } else {
        Err(ScopeError::Denied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allowlist_passes() {
        std::env::remove_var("UMST_AGENT_SCOPE_TOKENS");
        std::env::remove_var("UMST_AGENT_SCOPE_ALLOW_ANY");
        assert!(validate_scope_token(None).is_ok());
    }
}
