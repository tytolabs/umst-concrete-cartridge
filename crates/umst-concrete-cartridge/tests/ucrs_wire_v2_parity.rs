// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! U5 concrete consumer wire parity — `shared_types::observation` fixture roundtrip (CELL_UCRS_READY_U5_CONCRETE).

use umst_ucrs::shared_types::observation::{ObservedAtV2Wire, UcrsObservedAt, WIRE_SCALE};

const WIRE_V2_FIXTURE: &str =
    include_str!("../../../../umst-ucrs/fixtures/wire_v2_observed_at.json");

#[test]
fn wire_v2_fixture_roundtrip_via_shared_types() {
    let wire: ObservedAtV2Wire = serde_json::from_str(WIRE_V2_FIXTURE).expect(
        "wire_v2_observed_at.json must parse ObservedAtV2Wire via shared_types::observation",
    );
    assert_eq!(wire.schema_version, "observed_at.v2");
    assert_eq!(wire.phase_entropy_bits_scale, Some(WIRE_SCALE));

    let obs = UcrsObservedAt::from_v2_wire(&wire);
    let back = obs.to_v2_wire();
    let json = serde_json::to_string(&back).expect("v2 roundtrip serializes");
    let reparsed: ObservedAtV2Wire =
        serde_json::from_str(&json).expect("v2 roundtrip deserializes");
    assert_eq!(back, reparsed);
    assert_eq!(obs.ucrs_seq, wire.ucrs_seq);
}

#[test]
fn research_wire_v2_maps_shared_types_stamp() {
    use umst_concrete_cartridge::research::wire_v2::{ucrs_observed_at_to_v2, OBSERVED_AT_V2_SCHEMA};

    let u = UcrsObservedAt::synthetic(3, 0.25);
    let v2 = ucrs_observed_at_to_v2(&u);
    assert_eq!(v2.schema_version, OBSERVED_AT_V2_SCHEMA);
    assert_eq!(v2.ucrs_seq, Some(3));
    assert_eq!(v2.stamp_tier, "Synthetic");
}
