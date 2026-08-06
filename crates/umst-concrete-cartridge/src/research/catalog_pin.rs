// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// AGAP-2033-SEC-CART-PROV — catalog digest pin on material cartridge memory ingest.
// Languages remain `LanguageFunctor` in umst-semantics; this module is material-only.

use super::contribution::DEFAULT_CATALOG_HASH;
use thiserror::Error;

/// AGAP fleet card id.
pub const JOB_ID: &str = "SEC-CART-PROV";

/// Completion receipt cross-ref (AGAP-2127 re-verify deepens AGAP-2033 landing).
pub const RECEIPT_PATH: &str = "archived/residuals/misc-outputs-tmp/COMPLETION_AGAP_AGENT_SEC-CART-PROV_2127.md";

/// Catalog pin witness failure on constitutive admit ingest.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Pin mismatch blocks memory append; not a thermodynamic verdict.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CatalogPinError {
    #[error("catalog_hash does not match embedded manifest catalog lock digest")]
    Mismatch,
}

/// Whether `catalog_hash` is the agent CI placeholder (resolved at accept when manifest-bridge on).
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Placeholder detection; grounded hash applied on accept path.
#[must_use]
pub fn is_placeholder_catalog_hash(catalog_hash: &str) -> bool {
    catalog_hash == DEFAULT_CATALOG_HASH
}

/// Grounded catalog digest for memory ingest (`sha256:` + lock-bundle hex).
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Mirrors manifold `catalog_lock_bundle_sha256_hex` when manifest-bridge on.
#[must_use]
pub fn grounded_catalog_hash() -> String {
    #[cfg(feature = "manifest-bridge")]
    {
        return format!(
            "sha256:{}",
            umst_manifold::runtime::catalog::catalog_lock_bundle_sha256_hex()
        );
    }
    #[cfg(not(feature = "manifest-bridge"))]
    DEFAULT_CATALOG_HASH.to_string()
}

/// Pure pin witness — `catalog_hash` matches embedded lock digest or CI placeholder.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: MaOS `catalog_digest` ↔ `pin_witness_ok` parity (SEC-CATALOG-PIN consumer).
#[must_use]
pub fn catalog_pin_witness_ok(catalog_hash: &str) -> bool {
    if is_placeholder_catalog_hash(catalog_hash) {
        return true;
    }
    catalog_hash == grounded_catalog_hash()
}

/// Resolve submitted `catalog_hash` for constitutive admit ingest.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Placeholder → grounded pin; explicit hash must witness-ok.
pub fn resolve_catalog_hash_for_ingest(submitted: &str) -> Result<String, CatalogPinError> {
    if is_placeholder_catalog_hash(submitted) {
        return Ok(grounded_catalog_hash());
    }
    if catalog_pin_witness_ok(submitted) {
        return Ok(submitted.to_string());
    }
    Err(CatalogPinError::Mismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_resolves_to_grounded_or_default() {
        let resolved = resolve_catalog_hash_for_ingest(DEFAULT_CATALOG_HASH).expect("placeholder");
        assert!(resolved.starts_with("sha256:"));
        assert_eq!(resolved.len(), "sha256:".len() + 64);
    }

    #[test]
    fn grounded_hash_witnesses_self() {
        let grounded = grounded_catalog_hash();
        assert!(catalog_pin_witness_ok(&grounded));
        assert_eq!(
            resolve_catalog_hash_for_ingest(&grounded).expect("ok"),
            grounded
        );
    }

    #[test]
    fn wrong_hash_rejects() {
        let bad = "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert_eq!(
            resolve_catalog_hash_for_ingest(bad),
            Err(CatalogPinError::Mismatch)
        );
    }
}
