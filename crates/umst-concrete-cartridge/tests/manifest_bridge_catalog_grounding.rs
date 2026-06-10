// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! With **`manifest-bridge`**, default manifest gate `catalog_id`s must trace to the manifold
//! embedded catalog lock digest (git-pinned `umst-manifold`; no workspace `[patch]` required).

use std::path::PathBuf;

use umst_manifold::gate::GateEvaluator;
use umst_manifold::manifest::UmstManifest;
use umst_manifold::runtime::catalog::{
    bundled_catalog_lock_json, catalog_lock_bundle_sha256_hex, witness_catalog_quickcheck_ok,
    WitnessCatalog,
};

/// Pinned in `umst-manifold/artifacts/catalog.lock.json` (119-module unified export).
const EXPECTED_UPSTREAM_CATALOG_DIGEST_HEX: &str =
    "ef0ed071fc82bf8ebc8971aeee8d142b4b54e15583f0c575d942cb237474d1dc";
/// SHA-256 of verbatim `umst-manifold/artifacts/catalog.lock.json` (v2 dual-pin lock file).
const EXPECTED_CATALOG_LOCK_BUNDLE_SHA256_HEX: &str =
    "a8ffb055ad917a43a5dd6a59f7e3829782d773855fb972d4d01eb231a0f7dfd3";

/// Lean modules cited by cartridge mechanised `formal_anchor` blocks (FORMAL_GROUNDING_AUDIT).
const MECHANISED_LEAN_MODULE_BASENAMES: &[&str] = &[
    "Gate",
    "Powers",
    "RegimeSoundness",
    "OrderStatisticsBand",
    "JenningsGelSpace",
    "Helmholtz",
    "MeasurementCost",
];

/// Optional monorepo sibling for doc-file cross-checks (not required in GHA).
fn sibling_umst_manifold_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../umst-manifold");
    root.join("Cargo.toml").is_file().then_some(root)
}

fn default_manifest_gate_catalog_ids(manifest: &UmstManifest) -> Vec<&'static str> {
    vec![manifest.default_transition_gate.catalog_id()]
}

#[test]
fn manifest_default_gate_catalog_ids_resolve_embedded_catalog_digest() {
    let manifest = UmstManifest::default();
    let catalog_ids = default_manifest_gate_catalog_ids(&manifest);

    assert!(
        !catalog_ids.is_empty(),
        "expected at least one default gate catalog_id from UmstManifest::default()"
    );
    assert_eq!(
        catalog_ids[0], "umst.gate.cd_transition",
        "predict/manifest-bridge path must use Clausius–Duhem transition SSOT"
    );

    let lock_bundle_hex = catalog_lock_bundle_sha256_hex();
    assert_eq!(
        UmstManifest::compiled_catalog_lock_bundle_sha256_hex(),
        lock_bundle_hex,
        "manifest compiled_catalog_lock_bundle_sha256_hex must match runtime catalog embed"
    );
    assert_eq!(lock_bundle_hex.len(), 64);

    let lock_json = bundled_catalog_lock_json();
    assert!(
        lock_json.contains("\"upstream_catalog_digest_hex\"")
            || lock_json.contains("\"composed_catalog_digest_hex\""),
        "pinned catalog.lock.json must carry composed or upstream Lean export digest"
    );
    assert!(
        lock_json.contains("\"fiber_pins\""),
        "v2 dual-pin lock must list per-fiber digests"
    );
    assert!(
        lock_json.contains(&format!(
            "\"upstream_catalog_digest_hex\": \"{EXPECTED_UPSTREAM_CATALOG_DIGEST_HEX}\""
        )),
        "119-export upstream digest must match catalog.lock.json pin"
    );
    assert!(
        lock_json.contains("\"module_count\": 119"),
        "catalog.lock.json must pin the 119-module umst-formal-double-slit export"
    );
    assert_eq!(
        lock_bundle_hex, EXPECTED_CATALOG_LOCK_BUNDLE_SHA256_HEX,
        "runtime catalog_lock_bundle_sha256_hex must match documented R0 pin"
    );

    assert!(
        witness_catalog_quickcheck_ok(),
        "embedded witness catalog quickcheck must pass"
    );
    let witness = WitnessCatalog::from_embedded().expect("embedded witness catalog parses");
    assert_eq!(witness.version, 1);

    // Optional monorepo-only cross-checks (skipped in clean-clone CI).
    let Some(manifold_root) = sibling_umst_manifold_root() else {
        eprintln!(
            "manifest_bridge_catalog_grounding: git-dep core checks OK; skip sibling doc/Lean checks"
        );
        return;
    };

    let formal_root = manifold_root
        .parent()
        .map(|p| p.join("umst-formal-double-slit"));
    if let Some(formal_root) = formal_root.filter(|p| p.join("artifacts/catalog.json").is_file()) {
        let catalog_json = std::fs::read_to_string(formal_root.join("artifacts/catalog.json"))
            .expect("catalog.json");
        for module in MECHANISED_LEAN_MODULE_BASENAMES {
            assert!(
                catalog_json.contains(&format!("\"path\": \"Lean/{module}.lean\""))
                    || catalog_json.contains(&format!("\"module\": \"{module}\""))
                    || catalog_json.contains(module),
                "mechanised Lean module `{module}` must appear in {}",
                formal_root.join("artifacts/catalog.json").display()
            );
        }
    } else {
        eprintln!(
            "manifest_bridge_catalog_grounding: skip Lean module basename check — umst-formal-double-slit not sibling to manifold"
        );
    }

    let claims_path = manifold_root.join("docs/claims-vs-proofs.md");
    let claims = std::fs::read_to_string(&claims_path).unwrap_or_else(|e| {
        panic!(
            "claims-vs-proofs required for catalog_id grounding at {}: {e}",
            claims_path.display()
        )
    });
    for id in catalog_ids {
        assert!(
            claims.contains(id),
            "catalog_id `{id}` must appear in {}",
            claims_path.display()
        );
    }

    let gate_spec = manifold_root.join("docs/GateUnificationSpec.md");
    if gate_spec.is_file() {
        let spec = std::fs::read_to_string(&gate_spec).expect("read GateUnificationSpec");
        for id in default_manifest_gate_catalog_ids(&manifest) {
            assert!(
                spec.contains(id),
                "catalog_id `{id}` must appear in {}",
                gate_spec.display()
            );
        }
    }
}
