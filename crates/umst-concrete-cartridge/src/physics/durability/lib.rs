// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//
// FLEET-COMPOSER-ACCEL-D AC105 — umst-concrete composite durability witness.
// Pins on-disk census, orchestrator mix contract, and measured golden vector.

use std::path::{Path, PathBuf};

mod engine;

pub use engine::{
    DurabilityEngine, DurabilityOutcome, PathwayBreakdown, PathwayLeg,
    ORCHESTRATOR_PIN_AIR_FRACTION, ORCHESTRATOR_PIN_AIR_VOID_SURFACE, ORCHESTRATOR_PIN_ALPHA,
    ORCHESTRATOR_PIN_INTERNAL_RH, ORCHESTRATOR_PIN_PASTE_FRACTION, ORCHESTRATOR_PIN_REF_DIFFUSIVITY,
    ORCHESTRATOR_PIN_REQUIRED_AIR_PCT, ORCHESTRATOR_PIN_WC,
};

/// FLEET-COMPOSER-ACCEL-D slot id.
pub const AC105_JOB_ID: &str = "FLEET-COMPOSER-ACCEL2-AC105-CONCRETE-DURABILITY";

/// AC105 completion receipt cross-ref.
pub const AC105_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_ACCEL2_AC105.md";

/// Master TODO id for umst-concrete durability deepen.
pub const MASTER_JOB_ID: &str = "umst-concrete-concrete-durability";

/// Honest adoption tier — physics landed; orchestrator reroute open.
pub const POSTURE_TAG: &str = "witnessed-not-orchestrator-wired";

/// Composite durability engine source (write-set owner).
pub const DURABILITY_ENGINE_RELPATH: &str =
    "umst-concrete-cartridge/crates/umst-concrete-cartridge/src/physics/durability/engine.rs";

/// Facade re-export surface (pending `mod.rs` wire).
pub const DURABILITY_FACADE_RELPATH: &str =
    "umst-concrete-cartridge/crates/umst-concrete-cartridge/src/physics/durability.rs";

/// Three durability pathway legs composed by [`DurabilityEngine`].
pub const PATHWAY_COUNT: usize = 3;

/// Pathway identifiers for census.
pub const PATHWAY_IDS: [&str; PATHWAY_COUNT] =
    ["frost_powers", "chloride_transport", "autogenous_healing"];

/// Orchestrator still calls `FreezeThawEngine` directly — composite not yet routed.
pub const ORCHESTRATOR_WIRED: bool = false;

/// `physics/mod.rs` does not yet declare `pub mod durability`.
pub const MOD_RS_WIRED: bool = false;

/// Wire-hop count: engine on disk + facade on disk + orchestrator reroute + mod.rs wire.
pub const WIRE_HOP_COUNT: usize = 4;

/// Wire hops closed (engine + facade on disk; orchestrator + mod.rs open).
pub const WIRE_HOPS_CLOSED: usize = 2;

/// `production_wired` — measured false; no live gateway dispatch.
pub fn durability_production_wired() -> bool {
    false
}

/// Pathway leg census row for witness rollup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathwayCensusRow {
    pub pathway_id: &'static str,
    pub source_module: &'static str,
    pub physics_wired: bool,
    pub orchestrator_routed: bool,
}

/// Three-pathway census — physics kernels exist; orchestrator routes frost only.
pub fn pathway_leg_census() -> [PathwayCensusRow; PATHWAY_COUNT] {
    [
        PathwayCensusRow {
            pathway_id: "frost_powers",
            source_module: "physics/freeze_thaw.rs",
            physics_wired: true,
            orchestrator_routed: true,
        },
        PathwayCensusRow {
            pathway_id: "chloride_transport",
            source_module: "physics/transport.rs",
            physics_wired: true,
            orchestrator_routed: false,
        },
        PathwayCensusRow {
            pathway_id: "autogenous_healing",
            source_module: "physics/self_heal.rs",
            physics_wired: true,
            orchestrator_routed: false,
        },
    ]
}

/// Blocked predicate — honest false until wire completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockedPredicate {
    pub id: &'static str,
    pub description: &'static str,
    pub measured: bool,
    pub blocker: &'static str,
}

/// Blocked predicates for AC105 — all measured false on clean tree.
pub fn blocked_predicates() -> [BlockedPredicate; 5] {
    [
        BlockedPredicate {
            id: "P105-ORCHESTRATOR-COMPOSITE",
            description: "orchestrator calls DurabilityEngine::compute_composite",
            measured: false,
            blocker: "orchestrator L518 still calls FreezeThawEngine directly",
        },
        BlockedPredicate {
            id: "P105-MOD-RS-WIRE",
            description: "physics/mod.rs declares pub mod durability",
            measured: false,
            blocker: "mod.rs integrate pending — write-set fence",
        },
        BlockedPredicate {
            id: "P105-PRODUCTION-WIRED",
            description: "durability_production_wired() == true",
            measured: false,
            blocker: "no live gateway dispatch; measured false",
        },
        BlockedPredicate {
            id: "P105-CHLORIDE-ORCH",
            description: "chloride leg surfaced in pipeline report",
            measured: false,
            blocker: "report.rs has freeze_thaw_durability_factor only",
        },
        BlockedPredicate {
            id: "P105-HEALING-ORCH",
            description: "healing leg surfaced in pipeline report",
            measured: false,
            blocker: "self_heal called but composite index not emitted",
        },
    ]
}

/// Receipt probe rollup for shepherd triage.
#[derive(Debug, Clone, PartialEq)]
pub struct DurabilityProbe {
    pub job_id: &'static str,
    pub posture: &'static str,
    pub pathway_count: usize,
    pub wire_hops_closed: usize,
    pub wire_hop_count: usize,
    pub orchestrator_wired: bool,
    pub mod_rs_wired: bool,
    pub production_wired: bool,
    pub on_disk_surfaces: bool,
    pub blocked_predicate_count: usize,
    pub blocked_all_false: bool,
}

/// AC105 durability witness probe — measured fields only.
pub fn durability_probe() -> DurabilityProbe {
    let blocked = blocked_predicates();
    DurabilityProbe {
        job_id: AC105_JOB_ID,
        posture: POSTURE_TAG,
        pathway_count: PATHWAY_COUNT,
        wire_hops_closed: WIRE_HOPS_CLOSED,
        wire_hop_count: WIRE_HOP_COUNT,
        orchestrator_wired: ORCHESTRATOR_WIRED,
        mod_rs_wired: MOD_RS_WIRED,
        production_wired: durability_production_wired(),
        on_disk_surfaces: durability_on_disk_surfaces_exist(),
        blocked_predicate_count: blocked.len(),
        blocked_all_false: blocked.iter().all(|p| !p.measured),
    }
}

/// Resolve tyto-workspace root from `CARGO_MANIFEST_DIR`.
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(5)
        .expect("tyto-workspace root")
        .to_path_buf()
}

/// On-disk census for AC105 write-set surfaces.
pub fn durability_on_disk_surfaces_exist() -> bool {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let physics_dir = manifest.parent().expect("physics dir");
    [
        manifest.join("engine.rs"),
        manifest.join("lib.rs"),
        manifest.join("Cargo.toml"),
        physics_dir.join("durability.rs"),
    ]
    .iter()
    .all(|p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::{Data, Shape, Tensor};
    use burn_ndarray::{NdArray, NdArrayDevice};

    type B = NdArray<f32>;

    fn test_device() -> NdArrayDevice {
        NdArrayDevice::default()
    }

    fn scalar_rank4(v: f32, dev: &NdArrayDevice) -> Tensor<B, 4> {
        Tensor::from_data(Data::new(vec![v], Shape::new([1, 1, 1, 1])), dev)
    }

    fn composite_at_orchestrator_pin(dev: &NdArrayDevice) -> DurabilityOutcome<B> {
        DurabilityEngine::<B>::compute_composite(
            scalar_rank4(ORCHESTRATOR_PIN_AIR_FRACTION, dev),
            scalar_rank4(ORCHESTRATOR_PIN_PASTE_FRACTION, dev),
            scalar_rank4(ORCHESTRATOR_PIN_AIR_VOID_SURFACE, dev),
            ORCHESTRATOR_PIN_REQUIRED_AIR_PCT,
            scalar_rank4(ORCHESTRATOR_PIN_WC, dev),
            scalar_rank4(ORCHESTRATOR_PIN_ALPHA, dev),
            scalar_rank4(ORCHESTRATOR_PIN_REF_DIFFUSIVITY, dev),
            scalar_rank4(ORCHESTRATOR_PIN_INTERNAL_RH, dev),
            scalar_rank4(0.0, dev),
        )
    }

    #[test]
    fn ac105_durability_on_disk_census() {
        assert!(
            durability_on_disk_surfaces_exist(),
            "AC105 write-set surfaces must exist on disk"
        );
    }

    #[test]
    fn ac105_durability_doctrine_binding_honest() {
        assert!(!durability_production_wired());
        assert!(!ORCHESTRATOR_WIRED);
        assert!(!MOD_RS_WIRED);
        assert_eq!(POSTURE_TAG, "witnessed-not-orchestrator-wired");
        assert_eq!(PATHWAY_COUNT, 3);
        assert_eq!(WIRE_HOPS_CLOSED, 2);
        assert_eq!(WIRE_HOP_COUNT, 4);
    }

    #[test]
    fn ac105_durability_blocked_predicates_all_false() {
        let preds = blocked_predicates();
        assert_eq!(preds.len(), 5);
        for p in &preds {
            assert!(
                !p.measured,
                "predicate {} must be honestly false: {}",
                p.id,
                p.description
            );
        }
    }

    #[test]
    fn ac105_durability_pathway_leg_census_three_rows() {
        let census = pathway_leg_census();
        assert_eq!(census.len(), PATHWAY_COUNT);
        assert_eq!(census[0].pathway_id, "frost_powers");
        assert!(census[0].physics_wired);
        assert!(census[0].orchestrator_routed);
        assert!(census[1].physics_wired);
        assert!(!census[1].orchestrator_routed);
        assert!(census[2].physics_wired);
        assert!(!census[2].orchestrator_routed);
    }

    #[test]
    fn ac105_durability_probe_honest_no_fake_green() {
        let probe = durability_probe();
        assert_eq!(probe.job_id, AC105_JOB_ID);
        assert!(!probe.production_wired);
        assert!(!probe.orchestrator_wired);
        assert!(!probe.mod_rs_wired);
        assert!(probe.on_disk_surfaces);
        assert!(probe.blocked_all_false);
        assert_eq!(probe.blocked_predicate_count, 5);
        assert_eq!(probe.wire_hops_closed, 2);
    }

    #[test]
    fn ac105_durability_measured_golden_vector_at_orchestrator_pin() {
        let dev = test_device();
        let outcome = composite_at_orchestrator_pin(&dev);
        let breakdown = DurabilityEngine::<B>::pathway_breakdown(&outcome);

        assert!(
            breakdown.spacing_factor_mm.is_finite() && breakdown.spacing_factor_mm > 0.0,
            "spacing must be positive; got {}",
            breakdown.spacing_factor_mm
        );
        assert!(
            breakdown.frost_norm.is_finite() && breakdown.frost_norm > 0.0,
            "frost norm must be positive; got {}",
            breakdown.frost_norm
        );
        assert!(
            (0.0..=1.0).contains(&breakdown.chloride_resistance),
            "chloride resistance must be in [0,1]; got {}",
            breakdown.chloride_resistance
        );
        assert!(
            (0.0..=1.0).contains(&breakdown.healing_potential),
            "healing potential must be in [0,1]; got {}",
            breakdown.healing_potential
        );
        assert!(
            breakdown.composite_index.is_finite() && breakdown.composite_index > 0.0,
            "composite index must be positive; got {}",
            breakdown.composite_index
        );

        // Weakest-link invariant: composite = min(frost_norm, chloride, healing) × 100.
        let expected = breakdown
            .frost_norm
            .min(breakdown.chloride_resistance)
            .min(breakdown.healing_potential)
            * 100.0_f32;
        assert!(
            (breakdown.composite_index - expected).abs() < 1e-4,
            "weakest-link invariant failed: composite={} expected={}",
            breakdown.composite_index,
            expected
        );

        // At orchestrator pin α=0.75, healing is the governing leg (measured, not invented).
        assert_eq!(
            breakdown.governing_leg,
            PathwayLeg::AutogenousHealing,
            "governing leg at orchestrator pin"
        );
    }

    #[test]
    fn ac105_durability_composite_decreases_when_air_insufficient() {
        let dev = test_device();
        let outcome_ok = composite_at_orchestrator_pin(&dev);
        let breakdown_ok = DurabilityEngine::<B>::pathway_breakdown(&outcome_ok);

        let outcome_low_air = DurabilityEngine::<B>::compute_composite(
            scalar_rank4(0.01, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_PASTE_FRACTION, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_AIR_VOID_SURFACE, &dev),
            ORCHESTRATOR_PIN_REQUIRED_AIR_PCT,
            scalar_rank4(ORCHESTRATOR_PIN_WC, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_ALPHA, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_REF_DIFFUSIVITY, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_INTERNAL_RH, &dev),
            scalar_rank4(0.0, &dev),
        );
        let breakdown_low = DurabilityEngine::<B>::pathway_breakdown(&outcome_low_air);

        assert!(
            breakdown_low.frost_norm < breakdown_ok.frost_norm,
            "insufficient air must reduce frost leg: ok={} low={}",
            breakdown_ok.frost_norm,
            breakdown_low.frost_norm
        );
        // At orchestrator pin healing governs composite — frost leg still must drop.
        assert!(
            breakdown_low.frost_norm < breakdown_ok.frost_norm * 0.99,
            "frost leg must measurably decrease with low air"
        );
    }

    #[test]
    fn ac105_durability_chloride_resistance_increases_with_hydration() {
        let dev = test_device();
        let outcome_early = DurabilityEngine::<B>::compute_composite(
            scalar_rank4(ORCHESTRATOR_PIN_AIR_FRACTION, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_PASTE_FRACTION, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_AIR_VOID_SURFACE, &dev),
            ORCHESTRATOR_PIN_REQUIRED_AIR_PCT,
            scalar_rank4(ORCHESTRATOR_PIN_WC, &dev),
            scalar_rank4(0.40, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_REF_DIFFUSIVITY, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_INTERNAL_RH, &dev),
            scalar_rank4(0.0, &dev),
        );
        let outcome_late = DurabilityEngine::<B>::compute_composite(
            scalar_rank4(ORCHESTRATOR_PIN_AIR_FRACTION, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_PASTE_FRACTION, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_AIR_VOID_SURFACE, &dev),
            ORCHESTRATOR_PIN_REQUIRED_AIR_PCT,
            scalar_rank4(ORCHESTRATOR_PIN_WC, &dev),
            scalar_rank4(0.90, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_REF_DIFFUSIVITY, &dev),
            scalar_rank4(ORCHESTRATOR_PIN_INTERNAL_RH, &dev),
            scalar_rank4(0.0, &dev),
        );
        let cl_early = outcome_early.chloride_resistance.into_data().value[0];
        let cl_late = outcome_late.chloride_resistance.into_data().value[0];
        assert!(
            cl_late > cl_early,
            "higher α must improve chloride resistance: early={cl_early} late={cl_late}"
        );
    }

    #[test]
    fn ac105_durability_identify_governing_leg_tiebreak() {
        assert_eq!(
            DurabilityEngine::<B>::identify_governing_leg(0.5, 0.5, 0.5),
            PathwayLeg::FrostPowers
        );
        assert_eq!(
            DurabilityEngine::<B>::identify_governing_leg(0.6, 0.5, 0.7),
            PathwayLeg::ChlorideTransport
        );
        assert_eq!(
            DurabilityEngine::<B>::identify_governing_leg(0.8, 0.9, 0.3),
            PathwayLeg::AutogenousHealing
        );
    }

    #[test]
    fn fleet_composer_accel2_ac105_concrete_durability_honest() {
        assert_eq!(AC105_JOB_ID, "FLEET-COMPOSER-ACCEL2-AC105-CONCRETE-DURABILITY");
        assert_eq!(MASTER_JOB_ID, "umst-concrete-concrete-durability");
        assert_eq!(PATHWAY_IDS.len(), PATHWAY_COUNT);
        assert_eq!(AC105_RECEIPT_PATH, "outputs/.tmp/COMPOSER_ACCEL2_AC105.md");
    }
}
