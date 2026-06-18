// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(feature = "agent-layer")]

use serde_json::Value;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{
    accept, gate_check_mix, query, validate_for_accept, ContributeError, GateContext, MemoryQuery,
    ProvenanceClock, ResearchStore, WallClock,
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
fn rejects_inadmissible_contribution() {
    let v = load_fixture("reject_mix_01.json");
    assert!(validate_for_accept(&v).is_err());
}

#[test]
fn accepts_admissible_fixture_schema() {
    let v = load_fixture("admissible_mix_01.json");
    assert!(validate_for_accept(&v).is_ok());
}

#[test]
fn functional_accept_appends_row() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let store = ResearchStore::default();
    let clock = ProvenanceClock::default();
    let ctx = GateContext { profile: &profile };

    let (store, _clock, result) =
        accept(store, clock, WallClock, &ctx, &v).expect("accept admissible");
    assert!(!result.memory_id.is_empty());
    assert_eq!(store.rows().len(), 1);
}

#[test]
fn query_filter_by_regime() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let store = ResearchStore::default();
    let ctx = GateContext { profile: &profile };
    let (store, _, _) = accept(store, ProvenanceClock::default(), WallClock, &ctx, &v).unwrap();

    let hits = query(
        &store,
        &MemoryQuery {
            admissible_only: true,
            curing_regime: Some("standard_20C_water".into()),
            limit: Some(10),
            ..Default::default()
        },
    );
    assert_eq!(hits.len(), 1);

    let miss = query(
        &store,
        &MemoryQuery {
            admissible_only: true,
            curing_regime: Some("nonexistent".into()),
            limit: None,
            ..Default::default()
        },
    );
    assert!(miss.is_empty());
}

#[test]
fn gate_check_mix_pure() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let mix = v.get("mix_spec").cloned().expect("mix_spec");
    let summary = gate_check_mix(&profile, &mix);
    assert!(summary.admissible);
    assert_eq!(summary.catalog_ids.len(), 1);
}

#[test]
fn duplicate_content_rejected() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let ctx = GateContext { profile: &profile };
    let (store, clock, _) = accept(
        ResearchStore::default(),
        ProvenanceClock::default(),
        WallClock,
        &ctx,
        &v,
    )
    .unwrap();
    let err = accept(store, clock, WallClock, &ctx, &v).unwrap_err();
    assert!(matches!(err, ContributeError::Store(_)));
}

#[test]
fn mix_geometry_on_accepted_row() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let ctx = GateContext { profile: &profile };
    let (store, _, _) = accept(
        ResearchStore::default(),
        ProvenanceClock::default(),
        WallClock,
        &ctx,
        &v,
    )
    .unwrap();
    let row = &store.rows()[0];
    assert!(row.mix_geometry.is_some());
}

#[test]
fn near_mix_l1_query_sorts_by_distance() {
    let profile = Profile::load_bundled("default").expect("default profile");
    let v = load_fixture("admissible_mix_01.json");
    let ctx = GateContext { profile: &profile };
    let (store, _, _) = accept(
        ResearchStore::default(),
        ProvenanceClock::default(),
        WallClock,
        &ctx,
        &v,
    )
    .unwrap();
    let anchor = v.get("mix_spec").cloned().unwrap();
    let hits = query(
        &store,
        &MemoryQuery {
            admissible_only: true,
            near_mix_spec: Some(anchor),
            max_mix_l1: Some(1.0),
            limit: Some(5),
            ..Default::default()
        },
    );
    assert_eq!(hits.len(), 1);
}

#[test]
fn gate_reject_row_not_in_admissible_memory() {
    use umst_concrete_cartridge::research::ObservedAt;
    use umst_concrete_cartridge::research::{
        append_gate_reject_jsonl, build_gate_reject, GateVerdict,
    };

    let dir = std::env::temp_dir().join("umst_reject_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("gate_reject.jcs.jsonl");
    let mix = serde_json::json!({ "w_c": "3/4", "temperature_k": "29315/100" });
    let row = build_gate_reject(
        &mix,
        GateVerdict::Reject,
        vec!["umst.gate.cd_transition".into()],
        None,
        ObservedAt {
            stamp_tier: "Synthetic".into(),
            ucrs_seq: Some(1),
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(0),
        },
        None,
    );
    append_gate_reject_jsonl(&row, Some(&path)).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("gate_reject.v1"));
    let _ = std::fs::remove_dir_all(&dir);
}
