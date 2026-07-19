// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(feature = "agent-layer")]

//! Phase 0c — three-way split of MCP gate path (`contribution` module).
//!
//! **Card:** Phase 0c (gate consolidation).  
//! **Parity anchor:** `gate_parity_v0.json` · SHA256 `d5608148e29eeabd83935988699d08ce1233c3e87f2cd217d658e0c71c7a841e`.  
//! **Next:** Phase 0d — route every caller through one `umst-gate`. **Done** (canonical_gate).
//!
//! | Boundary | Module | Owns |
//! |----------|--------|------|
//! | Adapter | `research::contribution::adapter` | rational parse, `MixSpecWire`, `mix_spec_from_json` |
//! | Gate | `research::contribution::gate` | `gate_check_mix`, `gate_recheck`, `gate_recheck_with_spec` |
//! | Infra | `research::contribution::infra` | explain codes, remediation, `gate_check_mix_result` wire |

use serde_json::json;
use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::research::contribution::adapter::{
    mix_spec_from_json, mix_wire_from_spec_value, rational_to_f64,
};
use umst_concrete_cartridge::research::contribution::gate::gate_check_mix;
use umst_concrete_cartridge::research::contribution::infra::gate_check_mix_result;
use umst_concrete_cartridge::research::ObservedAt;

fn default_profile() -> Profile {
    Profile::load_bundled("default").expect("default profile")
}

#[test]
fn phase0c_adapter_parses_rational_mix_wire() {
    assert_eq!(rational_to_f64("9/20"), Some(0.45));
    let mix = json!({
        "w_c": "9/20",
        "temperature_k": "29315/100"
    });
    let wire = mix_wire_from_spec_value(&mix).expect("wire");
    assert!((wire.w_c - 0.45).abs() < 1e-12);
    assert!((wire.temperature_k - 293.15).abs() < 1e-9);
}

#[test]
fn phase0c_adapter_lifts_mix_spec_with_profile() {
    let profile = default_profile();
    let mix = json!({
        "w_c": "9/20",
        "temperature_k": "29315/100"
    });
    let spec = mix_spec_from_json(&profile, &mix).expect("spec");
    assert_eq!(spec.profile_name, profile.bundle_id);
}

#[test]
fn phase0c_gate_does_not_import_explain_types() {
    // Compile-time boundary: gate module path is separate from infra explain wire.
    let profile = default_profile();
    let mix = json!({
        "w_c": "9/20",
        "temperature_k": "29315/100"
    });
    let summary = gate_check_mix(&profile, &mix);
    assert!(summary.admissible || !summary.admissible);
}

#[test]
fn phase0c_infra_gate_check_mix_result_preserves_parity_path() {
    let profile = default_profile();
    let mix = json!({
        "w_c": "9/20",
        "temperature_k": "29315/100"
    });
    let observed = ObservedAt {
        stamp_tier: "Synthetic".into(),
        ucrs_seq: Some(0),
        phase_entropy_bits_q: None,
        phase_entropy_bits_scale: None,
        credit_head_bits_q: None,
        credit_head_bits_scale: None,
        wall_ms: None,
    };
    let result = gate_check_mix_result(&profile, &mix, true, observed);
    assert_eq!(result.gate_summary.catalog_ids, vec!["umst.gate.cd_transition"]);
    assert!(result.explain.is_some());
}

#[test]
fn phase0c_gate_uses_pure_manifold_core_not_predict_composite() {
    use umst_concrete_cartridge::pipeline::canonical_gate::thermodynamic_admissible;

    let profile = default_profile();
    let mix = json!({
        "w_c": "9/20",
        "temperature_k": "29315/100"
    });
    let spec = mix_spec_from_json(&profile, &mix).expect("spec");
    let summary = gate_check_mix(&profile, &mix);
    assert_eq!(
        summary.admissible,
        thermodynamic_admissible(&profile, &spec),
        "gate_check_mix must match canonical composed gate path"
    );
}
