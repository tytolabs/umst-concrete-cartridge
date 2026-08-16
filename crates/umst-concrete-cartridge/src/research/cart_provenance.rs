// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// AGAP-2033-SEC-CART-PROV — UCRS + optional trust provenance on constitutive admit ingest.
// Material cartridge memory path only (`contribution::accept`).

use super::types::ObservedAt;
#[cfg(feature = "ucrs-provenance")]
use super::wire_v2::observed_at_to_v2;

#[cfg(feature = "ucrs-provenance")]
use umst_trust::{default_cipher_suite, Trust, TrustAuthority, TrustScope};
#[cfg(feature = "ucrs-provenance")]
use umst_ucrs::shared_types::accept::{
    DurableAccept, DurableAcceptWire, TrustAttestedWarrant, TrustCipherSuite,
};
#[cfg(feature = "ucrs-provenance")]
use umst_ucrs::shared_types::observation::{ObservedAtV2Wire, UcrsObservedAt};

/// Constitutive admit stamp bundle returned from successful material ingest.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Gate-admitted row carries UCRS stamp; trust optional via env.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstitutiveAdmitStamp {
    /// Always true when accept succeeds — explicit wire leg for transcript / audit consumers.
    pub constitutive_admit: bool,
    /// Resolved catalog digest pinned on the memory row.
    pub catalog_hash: String,
    /// Optional `durable_accept.v0` when `ucrs-provenance` + trust env are configured.
    #[cfg(feature = "ucrs-provenance")]
    pub durable_accept: Option<DurableAcceptWire>,
}

impl ConstitutiveAdmitStamp {
    /// Build admit stamp after gate re-check and catalog pin resolution.
    /// formal_anchor: lean://umst-formal/Lean/Compat/Gate.lean#Admissible
    /// formal_status: Mechanised
    /// formal_anchor_rationale: UCRS observed_at stamped on every constitutive admit; trust when env set.
    #[must_use]
    pub fn from_admit(observed_at: &ObservedAt, catalog_hash: String) -> Self {
        #[cfg(feature = "ucrs-provenance")]
        {
            let durable_accept = stamp_durable_accept_if_configured(observed_at);
            return Self {
                constitutive_admit: true,
                catalog_hash,
                durable_accept,
            };
        }
        #[cfg(not(feature = "ucrs-provenance"))]
        {
            let _ = observed_at;
            Self {
                constitutive_admit: true,
                catalog_hash,
            }
        }
    }
}

#[cfg(feature = "ucrs-provenance")]
fn observed_at_to_ucrs(obs: &ObservedAt) -> UcrsObservedAt {
    let v2 = observed_at_to_v2(obs);
    let wire = ObservedAtV2Wire {
        schema_version: v2.schema_version,
        stamp_tier: v2.stamp_tier,
        ucrs_seq: v2.ucrs_seq,
        phase_entropy_bits_q: v2.phase_entropy_bits_q,
        phase_entropy_bits_scale: v2.phase_entropy_bits_scale,
        credit_head_bits_q: v2.credit_head_bits_q,
        credit_head_bits_scale: v2.credit_head_bits_scale,
        wall_ms: v2.wall_ms,
    };
    UcrsObservedAt::from_v2_wire(&wire)
}

#[cfg(feature = "ucrs-provenance")]
fn trust_cipher_suite_from_core(suite: &umst_trust::CipherSuite) -> TrustCipherSuite {
    TrustCipherSuite {
        kem: suite.kem.clone(),
        sig: suite.sig.clone(),
        hash: suite.hash.clone(),
    }
}

#[cfg(feature = "ucrs-provenance")]
fn trust_scope_from_env(raw: &str) -> TrustScope {
    match raw.trim().to_ascii_lowercase().as_str() {
        "ephemeral" => TrustScope::Ephemeral,
        "federated" => TrustScope::Federated,
        "highassurance" | "high_assurance" => TrustScope::HighAssurance,
        "hardwarerooted" | "hardware_rooted" => TrustScope::HardwareRooted,
        _ => TrustScope::Device,
    }
}

#[cfg(feature = "ucrs-provenance")]
fn trust_from_env() -> Option<Trust> {
    let root = std::env::var("UMST_TRUST_CHAIN_ROOT").ok()?;
    if root.trim().len() != 64 || !root.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let authority =
        std::env::var("UMST_TRUST_AUTHORITY").unwrap_or_else(|_| "cartridge-mcp".into());
    let scope_raw = std::env::var("UMST_TRUST_SCOPE").unwrap_or_else(|_| "device".into());
    Some(Trust {
        authority: TrustAuthority(authority),
        scope: trust_scope_from_env(&scope_raw),
        suite: default_cipher_suite(),
        chain: umst_trust::AttestationChain {
            chain_root_hex: root,
            expires_at_unix_ms: std::env::var("UMST_TRUST_EXPIRES_MS")
                .ok()
                .and_then(|s| s.parse().ok()),
        },
        revocation: umst_trust::MerkleRevocationTree::default(),
    })
}

#[cfg(feature = "ucrs-provenance")]
fn warrant_from_trust(trust: &Trust) -> TrustAttestedWarrant {
    TrustAttestedWarrant::from_trust_attested(
        trust.authority.0.clone(),
        format!("{:?}", trust.scope),
        trust.chain.chain_root_hex.clone(),
        trust.chain.expires_at_unix_ms,
        trust_cipher_suite_from_core(&trust.suite),
    )
}

#[cfg(feature = "ucrs-provenance")]
fn stamp_durable_accept_if_configured(observed_at: &ObservedAt) -> Option<DurableAcceptWire> {
    let trust = trust_from_env()?;
    let ucrs = observed_at_to_ucrs(observed_at);
    let warrant = warrant_from_trust(&trust);
    let accept = DurableAccept::bind(ucrs, warrant).ok()?;
    Some(accept.to_wire())
}

#[cfg(all(test, feature = "ucrs-provenance"))]
mod tests {
    use super::*;
    use crate::research::provenance::{observed_at_for_tick, WallClock};

    #[test]
    fn admit_stamp_without_trust_env_has_no_durable_wire() {
        std::env::remove_var("UMST_TRUST_CHAIN_ROOT");
        let obs = observed_at_for_tick(1, WallClock.epoch_ms());
        let stamp = ConstitutiveAdmitStamp::from_admit(&obs, "sha256:aa".repeat(32));
        assert!(stamp.constitutive_admit);
        assert!(stamp.durable_accept.is_none());
    }
}
