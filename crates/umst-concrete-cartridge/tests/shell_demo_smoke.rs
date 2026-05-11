// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(feature = "solver-experimental")]

//! Smoke test for [`optimize_shell_3d`](../examples/optimize_shell_3d.rs): [`AdjointCompliance::forward_and_loss`],
//! Helmholtz + Heaviside, bottom-edge mask, one backward + Adam, deterministic \(\rho\) bytes.
//!
//! Coarse grid uses cell spacing \(\Delta x=\Delta y=0.1\) like the 40²×4 example. Body force is **traction-only**
//! (roof pressure as nodal loads): with self-weight included at this resolution, bar-network PCG can return
//! non-finite \(u\) in `f32`. [`VolumeProjection`] and continuations are covered in `optimize_shell_3d` and
//! `umst-manifold` tests.

use burn::backend::Autodiff;
use burn::module::{Module, ModuleMapper, ParamId};
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::{
    backend::{AutodiffBackend, Backend as BackendTrait},
    Data, Shape, Tensor,
};
use burn_ndarray::NdArray;

use umst_manifold::ai::topology::{HeavisideProjection, TopologyOptimizer};
use umst_manifold::physics::adjoint::{AdjointCompliance, SimpElasticMaterial};
use umst_manifold::physics::extruded_plate::{ElasticMaterial, ExtrudedPlateMechanics};
use umst_manifold::physics::time_orchestration::MechanicsInnerLoopConfig;
use umst_manifold::physics::topology_filter::HelmholtzFilter;

type AD = Autodiff<NdArray<f32>>;
type B = AD;
type Inner = <AD as AutodiffBackend>::InnerBackend;

struct ZeroWeights;

impl<Bk: BackendTrait> ModuleMapper<Bk> for ZeroWeights {
    fn map_float<const D: usize>(&mut self, _id: &ParamId, tensor: Tensor<Bk, D>) -> Tensor<Bk, D> {
        Tensor::zeros(tensor.shape(), &tensor.device())
    }
}

fn zeroed_optimizer(
    vt: f32,
    p: f32,
    hidden: usize,
    dev: &<NdArray<f32> as BackendTrait>::Device,
) -> TopologyOptimizer<B> {
    let mut o = TopologyOptimizer::new(vt, p, hidden, dev);
    let mut z = ZeroWeights;
    o.density_net = o.density_net.map(&mut z);
    o
}

fn pin_bottom_perimeter_inner(
    nx: usize,
    ny: usize,
    nz: usize,
    dev: &<NdArray<f32> as BackendTrait>::Device,
) -> Tensor<Inner, 3> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let n = nx1 * ny1 * (nz + 1);
    let mut bm = vec![1.0_f32; n * 3];
    let mut pin = |ix: usize, iy: usize| {
        let nid = ix + iy * nx1;
        bm[nid * 3] = 0.0;
        bm[nid * 3 + 1] = 0.0;
        bm[nid * 3 + 2] = 0.0;
    };
    for ix in 0..=nx {
        pin(ix, 0);
        pin(ix, ny);
    }
    for iy in 0..=ny {
        pin(0, iy);
        pin(nx, iy);
    }
    Tensor::from_data(Data::new(bm, Shape::new([1, n, 3])), dev)
}

fn top_load_inner(
    nx: usize,
    ny: usize,
    nz: usize,
    pa: f32,
    dx: f32,
    dy: f32,
    dev: &<NdArray<f32> as BackendTrait>::Device,
) -> Tensor<Inner, 3> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let n = nx1 * ny1 * (nz + 1);
    let mut bf = vec![0.0f32; n * 3];
    let iz = nz;
    for iy in 0..=ny {
        for ix in 0..=nx {
            let nid = ix + iy * nx1 + iz * nx1 * ny1;
            bf[nid * 3 + 2] = -pa * dx * dy;
        }
    }
    Tensor::from_data(Data::new(bf, Shape::new([1, n, 3])), dev)
}

fn run_one_adjoint_step() -> Vec<f32> {
    let device = Default::default();
    let nx = 8usize;
    let ny = 8usize;
    let nz = 2usize;
    let lx = 0.8_f32;
    let ly = 0.8_f32;
    let lz = 0.1_f32;
    let dx = lx / nx as f32;
    let dy = ly / ny as f32;
    let dz = lz / nz as f32;

    let plate = ExtrudedPlateMechanics {
        nx,
        ny,
        nz,
        dx,
        dy,
        dz,
    };
    let coords = plate.coords_bn3::<B>(&device);
    let edges = plate.edges_b1::<B>(&device);
    let live_inner = top_load_inner(nx, ny, nz, 50.0, dx, dy, &device);

    let helm = HelmholtzFilter::new((2.0 * dx.min(dy).min(dz)).max(1e-6), 240, 1e-7);
    let proj = HeavisideProjection::new(8.0, 0.5);
    let mat = ElasticMaterial {
        e0: 200e6,
        nu: 0.2,
        simp_p: 3.0,
        e_min: 1.0,
    };
    let cg = MechanicsInnerLoopConfig {
        max_cg_iterations: 200,
        cg_tolerance: 1e-4,
        pcg_tolerance: 1e-4,
        use_preconditioner: true,
        max_equilibrium_substeps: 1,
    };

    let n = plate.n_nodes();
    let edges_inner = plate.edges_b1::<Inner>(&device);
    let coords_n3 = plate
        .coords_bn3::<Inner>(&device)
        .reshape(Shape::new([n, 3]));
    let boundary_inner = pin_bottom_perimeter_inner(nx, ny, nz, &device);
    let damage_z = Tensor::<Inner, 3>::zeros([1, n, 1], &device);
    let cross_section_area = (dx * dy * dz).cbrt().powf(2.0);

    let mut opt = zeroed_optimizer(0.12, 3.0, 32, &device);
    let mut adam = AdamConfig::new().init::<B, _>();
    let dx_f = dx.min(dy).min(dz);

    let simp_mat = SimpElasticMaterial {
        e0: mat.e0,
        nu: mat.nu,
        p: mat.simp_p,
        e_min: mat.e_min,
    };

    let rho_raw = opt.density_net.forward_batched(coords.clone());
    let rho_t = helm.apply(rho_raw, edges.clone(), dx_f);
    let rho_bar = proj.project(rho_t);
    let (surrogate, c_raw) = AdjointCompliance::forward_and_loss(
        rho_bar.clone(),
        edges_inner,
        coords_n3,
        boundary_inner,
        live_inner,
        damage_z,
        simp_mat,
        &cg,
        cross_section_area,
    );
    let comp_scale = c_raw.max(1e-12);
    let total = surrogate.div_scalar(comp_scale);
    let loss = total.clone().into_data().value[0];
    assert!(
        loss.is_finite(),
        "scaled surrogate must be finite, got {loss} (c_raw={c_raw})"
    );

    let grads = total.backward();
    let gp = GradientsParams::from_grads(grads, &opt.density_net);
    opt.density_net = adam.step(0.005, opt.density_net, gp);

    let rho_raw = opt.density_net.forward_batched(coords);
    let rho_t = helm.apply(rho_raw, edges, dx_f);
    proj.project(rho_t).into_data().value
}

#[test]
fn shell_demo_smoke_runs_and_is_deterministic() {
    let a = run_one_adjoint_step();
    let b = run_one_adjoint_step();
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < 1e-20));
}
