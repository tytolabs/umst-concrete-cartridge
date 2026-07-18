// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Fleet / parity IO boundary for Darwin operators and agents.
//!
//! [`crate::physics`] and cartridge facades stay pure; fixture reads, env lookups,
//! and delegation to fleet shell SSOT live here only (see `fp_photonics_cli_injection` pattern).

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

/// Parity digest + fixture path pins — SSOT in `umst-gate::admissibility_census` (≤1 writer).
pub use umst_manifold::gate::{
    gate_parity_fixture_path_from, GATE_PARITY_V0_FIXTURE_REL, GATE_PARITY_V0_SHA256,
    GATE_PARITY_V0_SHA256_PREFIX,
};

const FLEET_PICKER_REL: &str = "outputs/.tmp/fleet_pick_target_dir.sh";
const FLEET_SCRATCH_LOADER_REL: &str = "outputs/.tmp/fleet_load_scratch.sh";

/// Parity digest witness (stdout / JSON wire).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParityDigestReport {
    pub sha256: String,
    pub sha256_prefix: String,
    pub fixture_path: String,
    pub matches_locked: bool,
}

/// Darwin fleet scratch target resolution (stdout / JSON wire).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScratchTargetReport {
    pub card: String,
    pub umst_dynamic_target_dir: String,
    pub umst_target_ttl_mins: String,
    pub umst_target_pressure: String,
    pub umst_target_free_gi: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub umst_scratch_root: Option<String>,
}

/// How [`scratch_target_resolve`] formats subprocess output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScratchTargetMode {
    /// Print `UMST_DYNAMIC_TARGET_DIR` only (picker default).
    Path,
    /// `export VAR=…` lines for `eval`.
    Export,
    /// `VAR=value` lines (no `export` prefix).
    PrintEnv,
}

/// IO-boundary errors for ops helpers.
#[derive(Debug)]
pub enum OpsHostError {
    DarwinOnly,
    WorkspaceNotFound,
    FleetPickerMissing(PathBuf),
    FleetPickerFailed(String),
    FixtureRead(std::io::Error),
    FixtureMissing(PathBuf),
    Json(serde_json::Error),
}

impl fmt::Display for OpsHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DarwinOnly => write!(f, "scratch-target is Darwin-only (macOS host expected)"),
            Self::WorkspaceNotFound => {
                write!(f, "MaOS workspace root not found (set WS or UMST_WORKSPACE)")
            }
            Self::FleetPickerMissing(p) => write!(f, "fleet picker missing: {}", p.display()),
            Self::FleetPickerFailed(msg) => write!(f, "fleet_pick_target_dir failed: {msg}"),
            Self::FixtureRead(e) => write!(f, "read parity fixture: {e}"),
            Self::FixtureMissing(p) => write!(f, "parity fixture missing: {}", p.display()),
            Self::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl std::error::Error for OpsHostError {}

/// SHA-256 hex digest of `bytes` (pure).
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Default `gate_parity_v0.json` path from the `umst-cli` crate layout.
#[must_use]
pub fn default_parity_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../umst-mcp/tests/fixtures/gate_parity_v0.json")
}

/// Resolve MaOS-Workspace integration root (env or walk-up from crate dir).
pub fn resolve_workspace_root(explicit: Option<&Path>) -> Result<PathBuf, OpsHostError> {
    if let Some(root) = explicit {
        return Ok(root.to_path_buf());
    }
    for key in ["WS", "UMST_WORKSPACE"] {
        if let Ok(ws) = std::env::var(key) {
            let path = PathBuf::from(ws);
            if path.join(FLEET_PICKER_REL).is_file() {
                return Ok(path);
            }
        }
    }
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
        if dir.join(FLEET_PICKER_REL).is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(OpsHostError::WorkspaceNotFound)
}

/// Read fixture bytes and build a parity digest witness.
pub fn parity_digest_from_path(fixture: &Path) -> Result<ParityDigestReport, OpsHostError> {
    let bytes = std::fs::read(fixture).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            OpsHostError::FixtureMissing(fixture.to_path_buf())
        } else {
            OpsHostError::FixtureRead(e)
        }
    })?;
    let sha256 = sha256_hex(&bytes);
    let sha256_prefix = sha256.chars().take(16).collect();
    Ok(ParityDigestReport {
        matches_locked: sha256 == GATE_PARITY_V0_SHA256,
        fixture_path: fixture.display().to_string(),
        sha256,
        sha256_prefix,
    })
}

/// Resolve fixture path: explicit → workspace-relative → crate-default.
pub fn resolve_parity_fixture_path(workspace: Option<&Path>) -> Result<PathBuf, OpsHostError> {
    let default = default_parity_fixture_path();
    if default.is_file() {
        return Ok(default);
    }
    if let Some(ws) = workspace {
        let candidate = gate_parity_fixture_path_from(ws);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    if let Ok(ws) = resolve_workspace_root(workspace) {
        let candidate = gate_parity_fixture_path_from(&ws);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(OpsHostError::FixtureMissing(default))
}

/// Hash live `gate_parity_v0.json` and compare to the locked digest pin.
pub fn parity_digest_report(workspace: Option<&Path>) -> Result<ParityDigestReport, OpsHostError> {
    let fixture = resolve_parity_fixture_path(workspace)?;
    parity_digest_from_path(&fixture)
}

/// Human line matching `consumer_worktree_bootstrap.sh` preflight output.
#[must_use]
pub fn format_parity_digest_line(report: &ParityDigestReport) -> String {
    let status = if report.matches_locked { "OK" } else { "DRIFT" };
    format!(
        "parity digest: {}… {status}",
        &report.sha256_prefix
    )
}

/// Serialize parity digest witness as pretty JSON.
pub fn parity_digest_json(report: &ParityDigestReport) -> Result<String, OpsHostError> {
    serde_json::to_string_pretty(report).map_err(OpsHostError::Json)
}

fn ensure_darwin() -> Result<(), OpsHostError> {
    if std::env::consts::OS != "macos" {
        return Err(OpsHostError::DarwinOnly);
    }
    Ok(())
}

fn fleet_picker_path(workspace: &Path) -> PathBuf {
    workspace.join(FLEET_PICKER_REL)
}

/// Source fleet scratch env then delegate to `fleet_pick_target_dir.sh` (Darwin IO).
pub fn scratch_target_resolve(
    workspace: Option<&Path>,
    card: &str,
    mode: ScratchTargetMode,
) -> Result<ScratchTargetReport, OpsHostError> {
    ensure_darwin()?;
    let ws = resolve_workspace_root(workspace)?;
    let picker = fleet_picker_path(&ws);
    if !picker.is_file() {
        return Err(OpsHostError::FleetPickerMissing(picker));
    }

    let loader = ws.join(FLEET_SCRATCH_LOADER_REL);
    let mut cmd = if loader.is_file() {
        let mut bash = Command::new("bash");
        bash.arg("-c");
        bash.arg(format!(
            "source {loader} && bash {picker} {card} --print-env",
            loader = shell_quote(&loader.to_string_lossy()),
            picker = shell_quote(&picker.to_string_lossy()),
            card = shell_quote(card),
        ));
        bash
    } else {
        let mut bash = Command::new("bash");
        bash.arg(&picker).arg(card).arg("--print-env");
        bash
    };

    cmd.env("WS", &ws);
    let output = cmd
        .output()
        .map_err(|e| OpsHostError::FleetPickerFailed(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OpsHostError::FleetPickerFailed(stderr.trim().to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut fields = parse_print_env(&stdout);
    let report = ScratchTargetReport {
        card: card.to_string(),
        umst_dynamic_target_dir: fields
            .remove("UMST_DYNAMIC_TARGET_DIR")
            .unwrap_or_default(),
        umst_target_ttl_mins: fields.remove("UMST_TARGET_TTL_MINS").unwrap_or_default(),
        umst_target_pressure: fields.remove("UMST_TARGET_PRESSURE").unwrap_or_default(),
        umst_target_free_gi: fields.remove("UMST_TARGET_FREE_GI").unwrap_or_default(),
        umst_scratch_root: fields.remove("UMST_SCRATCH_ROOT"),
    };

    if report.umst_dynamic_target_dir.is_empty() {
        return Err(OpsHostError::FleetPickerFailed(
            "picker returned empty UMST_DYNAMIC_TARGET_DIR".to_string(),
        ));
    }

    // mode is used by formatters below; report is complete
    let _ = mode;
    Ok(report)
}

/// Format scratch target for stdout (path | export | print-env).
#[must_use]
pub fn format_scratch_target_output(report: &ScratchTargetReport, mode: ScratchTargetMode) -> String {
    match mode {
        ScratchTargetMode::Path => report.umst_dynamic_target_dir.clone(),
        ScratchTargetMode::Export => {
            let mut lines = vec![
                format!(
                    "export UMST_DYNAMIC_TARGET_DIR={}",
                    shell_quote(&report.umst_dynamic_target_dir)
                ),
                format!(
                    "export UMST_TARGET_TTL_MINS={}",
                    shell_quote(&report.umst_target_ttl_mins)
                ),
                format!(
                    "export UMST_TARGET_PRESSURE={}",
                    shell_quote(&report.umst_target_pressure)
                ),
                format!(
                    "export UMST_TARGET_FREE_GI={}",
                    shell_quote(&report.umst_target_free_gi)
                ),
            ];
            if let Some(root) = &report.umst_scratch_root {
                lines.push(format!("export UMST_SCRATCH_ROOT={}", shell_quote(root)));
            }
            lines.join("\n")
        }
        ScratchTargetMode::PrintEnv => {
            let mut lines = vec![
                format!("UMST_DYNAMIC_TARGET_DIR={}", report.umst_dynamic_target_dir),
                format!("UMST_TARGET_TTL_MINS={}", report.umst_target_ttl_mins),
                format!("UMST_TARGET_PRESSURE={}", report.umst_target_pressure),
                format!("UMST_TARGET_FREE_GI={}", report.umst_target_free_gi),
            ];
            if let Some(root) = &report.umst_scratch_root {
                lines.push(format!("UMST_SCRATCH_ROOT={root}"));
            }
            lines.join("\n")
        }
    }
}

/// Serialize scratch target witness as pretty JSON.
pub fn scratch_target_json(report: &ScratchTargetReport) -> Result<String, OpsHostError> {
    serde_json::to_string_pretty(report).map_err(OpsHostError::Json)
}

fn parse_print_env(stdout: &str) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            out.insert(key.to_string(), value.to_string());
        }
    }
    out
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._-:".contains(c))
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_locked_pin() {
        let fixture = default_parity_fixture_path();
        if !fixture.is_file() {
            return;
        }
        let bytes = std::fs::read(&fixture).expect("read fixture");
        let digest = sha256_hex(&bytes);
        assert_eq!(digest, GATE_PARITY_V0_SHA256);
    }

    #[test]
    fn parity_digest_report_matches_locked() {
        let report = parity_digest_report(None).expect("parity report");
        assert!(report.matches_locked);
        assert_eq!(report.sha256_prefix, GATE_PARITY_V0_SHA256_PREFIX);
    }

    #[test]
    fn format_parity_digest_line_ok_suffix() {
        let report = ParityDigestReport {
            sha256: GATE_PARITY_V0_SHA256.to_string(),
            sha256_prefix: GATE_PARITY_V0_SHA256_PREFIX.to_string(),
            fixture_path: "/tmp/gate_parity_v0.json".to_string(),
            matches_locked: true,
        };
        assert_eq!(
            format_parity_digest_line(&report),
            "parity digest: 149081fa81a6525f… OK"
        );
    }

    #[test]
    fn parse_print_env_roundtrip() {
        let stdout = "UMST_DYNAMIC_TARGET_DIR=/Volumes/Darwin/umst-scratch/umst-p-card\nUMST_TARGET_PRESSURE=scratch\n";
        let map = parse_print_env(stdout);
        assert_eq!(
            map.get("UMST_DYNAMIC_TARGET_DIR").map(String::as_str),
            Some("/Volumes/Darwin/umst-scratch/umst-p-card")
        );
        assert_eq!(map.get("UMST_TARGET_PRESSURE").map(String::as_str), Some("scratch"));
    }
}
