// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! R1: loss_scalar == gate_scalar at anchor p_gate=3 (ComplianceFunctional SSOT).

#![cfg(feature = "solver-experimental")]

use umst_manifold::physics::adjoint::SimpElasticMaterial;
use umst_manifold::physics::adjoint_q1_hex::AdjointComplianceQ1Hex;
use umst_manifold::physics::compliance_functional::{
    ComplianceContext, ComplianceFunctional, ComplianceHostInput, CompliancePenalization,
    Q1HexBrickSpec, Q1HexComplianceFunctional,
};
use umst_manifold::physics::extruded_plate::ExtrudedPlateMechanics;
use umst_manifold::physics::time_orchestration::MechanicsInnerLoopConfig;

fn plate_bottom_uz_mask(nx: usize, ny: usize, nz: usize) -> Vec<f32> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let n = nx1 * ny1 * (nz + 1);
    let mut m = vec![1.0_f32; n * 3];
    for iz in 0..=nz {
        for iy in 0..=ny {
            for ix in 0..=nx {
                let nid = ix + iy * nx1 + iz * nx1 * ny1;
                if iz == 0 {
                    m[nid * 3 + 2] = 0.0;
                }
            }
        }
    }
    m
}

#[test]
fn r1_anchor_loss_gate_parity_at_p3() {
    let nx = 4_usize;
    let ny = 4_usize;
    let nz = 1_usize;
    let plate = ExtrudedPlateMechanics {
        nx,
        ny,
        nz,
        dx: 0.25,
        dy: 0.25,
        dz: 0.05,
    };
    let n_nodes = (nx + 1) * (ny + 1) * (nz + 1);
    let rho_flat = vec![0.5_f32; n_nodes];
    let mut bf = vec![0.0_f32; n_nodes * 3];
    let top_nid = nx / 2 + (ny / 2) * (nx + 1) + nz * (nx + 1) * (ny + 1);
    bf[top_nid * 3 + 2] = -1.0;
    let bm = plate_bottom_uz_mask(nx, ny, nz);
    let cg = MechanicsInnerLoopConfig::default();
    let ctx = ComplianceContext {
        material: SimpElasticMaterial {
            e0: 1.0,
            nu: 0.3,
            p: 1.0,
            e_min: 1e-9,
        },
        mesh: Q1HexBrickSpec {
            nx,
            ny,
            nz,
            dx: plate.dx,
            dy: plate.dy,
            dz: plate.dz,
        },
        cg: cg.clone(),
        self_weight: None,
    };
    let p_gate = 3.0_f32;
    let gate_fn = Q1HexComplianceFunctional
        .eval_inner(
            &ctx,
            ComplianceHostInput {
                rho_flat: &rho_flat,
                body_force: &bf,
                boundary_mask: &bm,
                penalization: CompliancePenalization::Gate(p_gate),
            },
        )
        .expect("gate eval");
    let legacy_gate = AdjointComplianceQ1Hex::raw_compliance_at_rho(
        &rho_flat,
        nx,
        ny,
        nz,
        plate.dx,
        plate.dy,
        plate.dz,
        &bf,
        &bm,
        SimpElasticMaterial {
            p: p_gate,
            ..ctx.material
        },
        &cg,
        None,
    );
    let loss_scalar = gate_fn.c_raw;
    let gate_scalar = legacy_gate;
    let eps = 1e-4_f32;
    assert!(
        (loss_scalar - gate_scalar).abs() < eps,
        "loss_scalar {loss_scalar} != gate_scalar {gate_scalar} at p_gate={p_gate}"
    );
    assert!((gate_fn.penalization_p - p_gate).abs() < 1e-6);
    eprintln!(
        "r1_anchor: p_gate={p_gate} loss_scalar={loss_scalar:.6} gate_scalar={gate_scalar:.6} eq_rel={:.3e}",
        gate_fn.eq_rel
    );
}
