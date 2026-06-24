// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cartridge THMC + gate evidence integration — [`ConcreteTransitionCartridge`] on coupled step.

use burn::tensor::{Data, Int, Shape, Tensor};
use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::{
    gate_cartridge_witness, with_gate_cartridge, ConcreteCartridge, ConcreteTransitionCartridge,
    ConcreteTransitionWitness, CEMENT_DEFAULT_S_INTRINSIC_MPA,
};
use umst_manifold::core::tensors::UnifiedMaterialStateTensor;
use umst_manifold::core::umst_schema::UMST_SCALAR_CHANNEL_COUNT;
use umst_manifold::physics::solvers::{
    ChemicalPlan, HydrologicPlan, MechanicalPlan, ThermalPlan, ThmcSolver, ThmcSolverStep,
    ThmcState,
};
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
    ThmcState {
        thermal: ThermalPlan {
            temperature: Tensor::full([1, n, 1], temp, dev),
        },
        hydro: HydrologicPlan {
            humidity: Tensor::full([1, n, 1], humidity, dev),
        },
        mechanical: MechanicalPlan {
            displacement: Tensor::zeros([1, n, 3], dev),
        },
        chemical: ChemicalPlan {
            reaction_extent: Tensor::full([1, n, 1], alpha, dev),
        },
        damage: Tensor::zeros([1, n, 1], dev),
        time,
    }
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
    let manifold = umst(n);
    let dev = dev();
    let pre = mk_state(&dev, n, 293.0_f32, 0.5_f32, 0.42_f32, 0.0_f32);
    let cartridge = ConcreteTransitionCartridge;
    let mut solver = with_gate_cartridge(ThmcSolver::default(), &cartridge);
    let science = ConcreteCartridge::default();
    let post = solver
        .step(&science, pre.clone(), &manifold)
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
