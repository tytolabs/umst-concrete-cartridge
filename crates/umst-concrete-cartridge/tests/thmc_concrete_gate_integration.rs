// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cartridge THMC + gate evidence integration — [`ConcreteTransitionCartridge`] on coupled step.
//!
//! **S4:** [`s4_prep`] module holds B3 compose-wire parity pins (enabled when stack green).

use burn::tensor::{Data, Int, Shape, Tensor};
use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::{
    gate_cartridge_witness, with_gate_cartridge, ConcreteCartridge, ConcreteTransitionCartridge,
    ConcreteTransitionWitness, CEMENT_DEFAULT_S_INTRINSIC_MPA,
};
use umst_manifold::core::tensors::UnifiedMaterialStateTensor;
use umst_manifold::core::umst_schema::UMST_SCALAR_CHANNEL_COUNT;
use umst_manifold::physics::solvers::{ThmcSolver, ThmcSolverStep, ThmcState};
use umst_manifold::runtime::catalog::traceability::CD_TRANSITION_CATALOG_ID;
use umst_manifold::runtime::gate::AdmissibilityToken;

type B = NdArray<f32>;

fn dev() -> NdArrayDevice {
    NdArrayDevice::default()
}

fn umst(n: usize) -> UnifiedMaterialStateTensor<B> {
    let dev = dev();
    let f = UMST_SCALAR_CHANNEL_COUNT;
    let coords: Tensor<B, 2, Int> =
        Tensor::from_data(Data::new(vec![0i64; n * 5], Shape::new([n, 5])), &dev);
    let edges_b1: Tensor<B, 2, Int> = Tensor::from_data(
        Data::new(vec![0i64, 1i64, 1i64, 0i64], Shape::new([2, 2])),
        &dev,
    );
    let faces_b2: Tensor<B, 2, Int> =
        Tensor::from_data(Data::new(vec![0i64, 0i64], Shape::new([2, 1])), &dev);
    UnifiedMaterialStateTensor {
        coords,
        edges_b1,
        faces_b2,
        scalar_features: Tensor::<B, 2>::zeros([n, f], &dev),
        vector_features: Tensor::<B, 3>::zeros([n, 1, 3], &dev),
        matrix_features: Tensor::<B, 4>::zeros([n, 1, 3, 3], &dev),
        resolution_mm: [1.0, 1.0, 1.0],
        node_positions: None,
        displacement_bc_mask: Tensor::<B, 3>::ones([n, 3, 1], &dev),
        policy_editable_mask: Tensor::<B, 2>::ones([n, 1], &dev),
        #[cfg(feature = "formal-witness")]
        catalog_schema_digest: None,
    }
}

fn mk_state(
    dev: &NdArrayDevice,
    n: usize,
    temp: f32,
    humidity: f32,
    alpha: f32,
    time: f32,
) -> ThmcState<B> {
    ThmcState::from_tensors(
        Tensor::full([1, n, 1], temp, dev),
        Tensor::full([1, n, 1], humidity, dev),
        Tensor::zeros([1, n, 3], dev),
        Tensor::full([1, n, 1], alpha, dev),
        Tensor::zeros([1, n, 1], dev),
        time,
    )
}

#[test]
fn with_gate_cartridge_sets_cement_intrinsic_strength() {
    let cartridge = ConcreteTransitionCartridge;
    let solver = with_gate_cartridge(ThmcSolver::default(), &cartridge);
    assert!(
        (solver.gate_intrinsic_strength_mpa - CEMENT_DEFAULT_S_INTRINSIC_MPA).abs() < 1e-9,
        "expected cement SSOT intrinsic strength"
    );
}

#[test]
fn concrete_transition_witness_enriched_scalars() {
    let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
    let new = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.5, 298.0);
    let witness: ConcreteTransitionWitness =
        ConcreteTransitionCartridge.transition_witness(&old, &new, 3600.0);
    assert_eq!(witness.core.catalog_id, CD_TRANSITION_CATALOG_ID);
    assert!(witness.dissipation_joules.is_finite());
    assert!((witness.hydration_alpha - 0.5).abs() < 1e-9);
}

#[test]
fn thmc_step_drains_cartridge_gate_evidence() {
    let n = 2usize;
    let mut manifold = umst(n);
    let dev = dev();
    let pre = mk_state(&dev, n, 293.0_f32, 0.5_f32, 0.42_f32, 0.0_f32);
    let cartridge = ConcreteTransitionCartridge;
    let mut solver = with_gate_cartridge(ThmcSolver::default(), &cartridge);
    let science = ConcreteCartridge::default();
    let post = solver
        .step(&science, pre.clone(), &mut manifold)
        .expect("thmc step");
    let drained = solver.drain_gate_evidence();
    assert!(!drained.is_empty(), "expected post-step gate evidence");
    assert_eq!(drained[0].transition.catalog_id, CD_TRANSITION_CATALOG_ID);

    let direct =
        ThmcSolverStep::attach_gate_evidence(&solver, &science, &pre, &post, &manifold, solver.dt)
            .expect("attach gate evidence");
    assert_eq!(
        direct.transition.admissibility,
        AdmissibilityToken::Admissible
    );
    let _gate = gate_cartridge_witness(&cartridge);
}

// ---------------------------------------------------------------------------
// S4 — B3 compose wire parity pins (enabled @ stack green · digest hold `d5608148…`).
// ---------------------------------------------------------------------------

mod s4_prep {
    use super::*;
    use umst_cartridge_concrete::{
        extract_matches_contribution_at_passive_pin, g0_probe_atom_state, gate_route_composed,
        hydration_alpha_parity_holds, try_b3_layer_contribution, MixScalars,
        B3_DISSIPATION_EXTRACT_PARITY_ABS_TOL, B3_PSI_EXTRACT_PARITY_ABS_TOL, D_CLOSURE_ABS_TOL,
        GATE_PARITY_SHA256, HYDRATION_ALPHA_PARITY_ABS_TOL, PSI_CLOSURE_ABS_TOL,
    };
    use umst_cartridge_continuum::ContinuumAtomRates;
    use umst_manifold::gate::transition_proposal::TRANSITION_TOLERANCE;
    use umst_manifold::runtime::gate::{CdTransitionCartridge, GateCartridge};

    /// Wave 3 parity digest prefix — immutable SSOT pin (`d5608148…`).
    const GATE_PARITY_DIGEST_PREFIX: &str = "d5608148";

    #[test]
    fn gate_parity_digest_locked() {
        assert!(
            GATE_PARITY_SHA256.starts_with(GATE_PARITY_DIGEST_PREFIX),
            "gate_parity_v0.json digest drift — hold d5608148… unless operator-approved fixture change"
        );
    }

    #[test]
    fn b3_hydration_alpha_parity_at_g0_passive_pin() {
        let mix = MixScalars::g0_pass_rational_default();
        assert!(
            hydration_alpha_parity_holds(&mix, HYDRATION_ALPHA_PARITY_ABS_TOL),
            "consumer mix oracle must match B3 lift before S4 wire"
        );
    }

    #[test]
    fn composed_b3_passive_dissipation_zero_digest_safe() {
        let mix = MixScalars::g0_pass_rational_default();
        let outcome = gate_route_composed(
            &mix,
            g0_probe_atom_state(),
            ContinuumAtomRates::PASSIVE,
            0.0,
            PSI_CLOSURE_ABS_TOL,
            D_CLOSURE_ABS_TOL,
        );
        assert_eq!(outcome.constitutive.dissipation_b3, 0.0);
        assert!(outcome.constitutive.psi_b3.is_finite());
        assert!(outcome.constitutive.psi_closure_holds(PSI_CLOSURE_ABS_TOL));
        assert!(outcome.route.admissible);
    }

    #[test]
    fn b3_extract_matches_contribution_at_passive_pin() {
        let mix = MixScalars::g0_pass_rational_default();
        assert!(extract_matches_contribution_at_passive_pin(
            &mix,
            B3_PSI_EXTRACT_PARITY_ABS_TOL,
            B3_DISSIPATION_EXTRACT_PARITY_ABS_TOL,
        ));
    }

    #[test]
    fn cartridge_matches_cd_on_identity_transition() {
        let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
        let new = old;
        let concrete = ConcreteTransitionCartridge.transition_evidence(&old, &new, 1.0);
        let cd = CdTransitionCartridge.transition_evidence(&old, &new, 1.0);
        assert_eq!(concrete.catalog_id, cd.catalog_id);
        assert_eq!(concrete.admissibility, cd.admissibility);
        assert_eq!(concrete.catalog_id, CD_TRANSITION_CATALOG_ID);
    }

    #[test]
    fn thmc_step_and_composed_b3_share_cd_catalog() {
        let n = 2usize;
        let mut manifold = umst(n);
        let dev = dev();
        let pre = mk_state(&dev, n, 293.0_f32, 0.5_f32, 0.42_f32, 0.0_f32);
        let cartridge = ConcreteTransitionCartridge;
        let mut solver = with_gate_cartridge(ThmcSolver::default(), &cartridge);
        let science = ConcreteCartridge::default();
        let post = solver
            .step(&science, pre.clone(), &mut manifold)
            .expect("thmc step");
        let drained = solver.drain_gate_evidence();
        assert!(!drained.is_empty());
        assert_eq!(drained[0].transition.catalog_id, CD_TRANSITION_CATALOG_ID);

        let mix = MixScalars::g0_pass_rational_default();
        let b3 = try_b3_layer_contribution(
            &mix,
            g0_probe_atom_state(),
            ContinuumAtomRates::PASSIVE,
            0.0,
        )
        .expect("B3 passive pin");
        assert_eq!(b3.dissipation_b3, 0.0);

        let direct = ThmcSolverStep::attach_gate_evidence(
            &solver, &science, &pre, &post, &manifold, solver.dt,
        )
        .expect("attach gate evidence");
        assert_eq!(direct.transition.catalog_id, CD_TRANSITION_CATALOG_ID);
        assert_eq!(
            direct.transition.admissibility,
            AdmissibilityToken::Admissible
        );
    }

    #[test]
    fn witness_dissipation_matches_host_on_calibrated_advance() {
        let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
        let new = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.5, 298.0);
        let witness = ConcreteTransitionCartridge.transition_witness(&old, &new, 3600.0);
        let host = ConcreteTransitionCartridge::dissipation_joules(
            &old,
            &new,
            3600.0,
            TRANSITION_TOLERANCE,
        );
        assert!(
            (witness.dissipation_joules - host).abs() < 1e-12,
            "witness telemetry must track host dissipation path"
        );
        assert_eq!(witness.core.catalog_id, CD_TRANSITION_CATALOG_ID);
    }
}
