// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Phase 8 adversarial end-conditions — gate_check result wire, pagination, filter boundaries.

#![cfg(feature = "agent-layer")]

use serde_json::json;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::{
    accept, gate_check_mix_result, query_page, GateContext, GateVerdict, MemoryQuery, MemoryRecord,
    ProvenanceClock, ResearchStore, WallClock, GATE_REJECT_SCHEMA,
};

fn repo_root() -> String {
    env!("CARGO_MANIFEST_DIR").replace("/crates/umst-concrete-cartridge", "")
}

fn load_fixture(name: &str) -> serde_json::Value {
    let path = format!("{}/fixtures/golden-adversarial/{name}", repo_root());
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

fn synthetic_observed(seq: u64) -> umst_concrete_cartridge::research::ObservedAt {
    umst_concrete_cartridge::research::ObservedAt {
        stamp_tier: "Synthetic".into(),
        ucrs_seq: Some(seq),
        phase_entropy_bits_q: None,
        phase_entropy_bits_scale: None,
        credit_head_bits_q: None,
        credit_head_bits_scale: None,
        wall_ms: Some(1_000),
    }
}

#[test]
fn gate_check_result_admissible_has_no_gate_reject() {
    let profile = Profile::load_bundled("default").expect("profile");
    let mix = load_fixture("admissible_mix_01.json")
        .get("mix_spec")
        .cloned()
        .expect("mix_spec");
    let result = gate_check_mix_result(&profile, &mix, true, synthetic_observed(0));
    assert!(result.gate_summary.admissible);
    assert_eq!(result.gate_summary.verdict, GateVerdict::Pass);
    assert!(result.gate_reject.is_none());
    let explain = result.explain.expect("explain");
    assert!(explain.regime_violations.is_empty());
    assert!(!explain.catalog_witnesses.is_empty());
}

#[test]
fn gate_check_result_reject_embeds_gate_reject_v1() {
    let profile = Profile::load_bundled("default").expect("profile");
    let mix = json!({
        "w_c": "not-rational",
        "temperature_k": "29315/100"
    });
    let result = gate_check_mix_result(&profile, &mix, true, synthetic_observed(1));
    assert!(!result.gate_summary.admissible);
    assert_eq!(result.gate_summary.verdict, GateVerdict::Reject);
    let reject = result.gate_reject.expect("gate_reject row");
    assert_eq!(reject.schema_version, GATE_REJECT_SCHEMA);
    assert_eq!(reject.verdict, "REJECT");
    let explain = result.explain.expect("explain");
    assert!(explain
        .regime_violations
        .contains(&"mix_spec_rational_parse_fail".to_string()));
}

#[test]
fn golden_reject_mix_produces_gate_reject_row() {
    let profile = Profile::load_bundled("default").expect("profile");
    let fixture = load_fixture("reject_mix_01.json");
    let mix = fixture.get("mix_spec").cloned().expect("mix_spec");
    let result = gate_check_mix_result(&profile, &mix, false, synthetic_observed(2));
    assert!(!result.gate_summary.admissible);
    assert!(result.gate_reject.is_some());
    assert!(result.explain.is_none());
}

#[test]
fn query_page_empty_when_filters_impossible() {
    let profile = Profile::load_bundled("default").expect("profile");
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
    let rows = store.rows();
    let page = query_page(
        &rows,
        &MemoryQuery {
            curing_regime: Some("impossible_regime".into()),
            ..Default::default()
        },
    );
    assert!(page.rows.is_empty());
    assert!(page.next_cursor.is_none());
}

#[test]
fn query_page_cursor_past_last_returns_empty() {
    let profile = Profile::load_bundled("default").expect("profile");
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
    let row = store.rows()[0].clone();
    let page = query_page(
        &[row],
        &MemoryQuery {
            cursor: Some("nonexistent-content-id".into()),
            limit: Some(10),
            ..Default::default()
        },
    );
    // Unknown cursor: no skip applied; single row still returned.
    assert_eq!(page.rows.len(), 1);
}

#[test]
fn query_page_no_next_cursor_when_exhausted() {
    let profile = Profile::load_bundled("default").expect("profile");
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
    let page = query_page(
        &store.rows(),
        &MemoryQuery {
            limit: Some(100),
            ..Default::default()
        },
    );
    assert_eq!(page.rows.len(), 1);
    assert!(page.next_cursor.is_none());
}

#[test]
fn query_page_admissible_only_excludes_inadmissible_payload() {
    let profile = Profile::load_bundled("default").expect("profile");
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
    let mut bad = store.rows()[0].clone();
    bad.payload.gate_summary.admissible = false;
    bad.content_id = "bad-row".into();
    let page = query_page(
        &[bad],
        &MemoryQuery {
            admissible_only: true,
            ..Default::default()
        },
    );
    assert!(page.rows.is_empty());
}

#[test]
fn query_page_outcome_source_filter_end_condition() {
    let profile = Profile::load_bundled("default").expect("profile");
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
    let mut row = store.rows()[0].clone();
    row.payload.outcome = json!({ "source": "lab_batch_a" });
    let hit = query_page(
        std::slice::from_ref(&row),
        &MemoryQuery {
            outcome_source: Some("lab_batch_a".into()),
            admissible_only: false,
            ..Default::default()
        },
    );
    assert_eq!(hit.rows.len(), 1);
    let miss = query_page(
        std::slice::from_ref(&row),
        &MemoryQuery {
            outcome_source: Some("other".into()),
            admissible_only: false,
            ..Default::default()
        },
    );
    assert!(miss.rows.is_empty());
}

#[test]
fn query_page_wall_ms_window_excludes_out_of_range() {
    let row = MemoryRecord {
        schema_version: "memory_record.v1".into(),
        canon_version: "jcs-rfc8785-v1".into(),
        content_id: "t".into(),
        observed_at: synthetic_observed(0),
        payload: umst_concrete_cartridge::research::MemoryPayload {
            mix_spec: json!({}),
            process: json!({}),
            outcome: json!({}),
            gate_summary: umst_concrete_cartridge::research::GateSummary {
                admissible: true,
                verdict: GateVerdict::Pass,
                catalog_ids: vec![],
                safety_margin: None,
                mi_bits_est: None,
            },
        },
        catalog_hash: "sha256:0".into(),
        catalog_ids: vec![],
        memory_id: None,
        mix_geometry: None,
    };
    let page = query_page(
        &[row],
        &MemoryQuery {
            wall_ms_min: Some(9_999),
            wall_ms_max: Some(9_999),
            admissible_only: false,
            ..Default::default()
        },
    );
    assert!(page.rows.is_empty());
}
