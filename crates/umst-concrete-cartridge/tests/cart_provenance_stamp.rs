// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(all(feature = "agent-layer", feature = "ucrs-provenance"))]

use serde_json::Value;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{
    accept, catalog_pin_witness_ok, grounded_catalog_hash, is_placeholder_catalog_hash,
    resolve_catalog_hash_for_ingest, AcceptError, GateContext, ProvenanceClock, ResearchStore,
    WallClock, CART_PROV_JOB_ID,
};

fn load_fixture(name: &str) -> Value {
    let path = format!(
        "{}/fixtures/golden-adversarial/{name}",
        env!("CARGO_MANIFEST_DIR").replace("/crates/umst-concrete-cartridge", "")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

#[test]
fn sec_cart_prov_job_id_is_fleet_slice() {
    assert_eq!(CART_PROV_JOB_ID, "SEC-CART-PROV");
}

#[test]
fn placeholder_catalog_resolves_to_grounded_pin() {
    let v = load_fixture("admissible_mix_01.json");
    let placeholder = v
        .get("catalog_hash")
        .and_then(|x| x.as_str())
        .expect("fixture catalog_hash");
    assert!(is_placeholder_catalog_hash(placeholder));
    let grounded = resolve_catalog_hash_for_ingest(placeholder).expect("resolve");
    assert_eq!(grounded, grounded_catalog_hash());
    assert!(catalog_pin_witness_ok(&grounded));
}

#[test]
fn constitutive_admit_stamps_ucrs_and_catalog_on_accept() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let store = ResearchStore::default();
    let clock = ProvenanceClock::default();
    let ctx = GateContext { profile: &profile };

    let (store, _clock, result) =
        accept(store, clock, WallClock, &ctx, &v).expect("accept admissible");

    assert!(result.constitutive_admit);
    assert!(catalog_pin_witness_ok(&result.catalog_hash));
    assert!(result.observed_at.ucrs_seq.is_some());
    assert_eq!(store.rows()[0].catalog_hash, result.catalog_hash);
}

#[test]
fn wrong_catalog_hash_rejects_before_memory_append() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let mut v = load_fixture("admissible_mix_01.json");
    v["catalog_hash"] = serde_json::json!(
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    let ctx = GateContext { profile: &profile };
    let err = accept(
        ResearchStore::default(),
        ProvenanceClock::default(),
        WallClock,
        &ctx,
        &v,
    )
    .unwrap_err();
    assert!(matches!(err, AcceptError::CatalogPin(_)));
}

#[test]
fn trust_env_emits_durable_accept_on_admit() {
    std::env::set_var(
        "UMST_TRUST_CHAIN_ROOT",
        "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    );
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let ctx = GateContext { profile: &profile };
    let (_, _, result) = accept(
        ResearchStore::default(),
        ProvenanceClock::default(),
        WallClock,
        &ctx,
        &v,
    )
    .expect("accept");
    let wire = result
        .durable_accept
        .as_ref()
        .expect("durable_accept when trust env configured");
    assert_eq!(
        wire.get("schema_version").and_then(|x| x.as_str()),
        Some("durable_accept.v0")
    );
    std::env::remove_var("UMST_TRUST_CHAIN_ROOT");
}
