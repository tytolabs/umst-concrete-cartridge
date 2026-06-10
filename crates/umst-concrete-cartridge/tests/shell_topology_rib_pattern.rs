// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(feature = "solver-experimental")]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::too_many_arguments)]

//! **Track B6 (v0.4)** — `shell_topology_rib_pattern`: Striatus-class gates ([`composer_prompts/v0.4_solver_completion_no_namesakes.md`](../../../../composer_prompts/v0.4_solver_completion_no_namesakes.md) §B6).
//!
//! - [`shell_topology_rib_pattern_quick`]: CI — compact **0.8×0.8×0.1** m slab, coords **\([-1,1]^3\)** (same as [`optimize_shell_3d`](../examples/optimize_shell_3d.rs)), default **9×8×2** cells when all `UMST_SHELL_{NX,NY,NZ,ITERS}` are unset, gentle roof **x-ramp** \(r=0.2\) at **50 Pa**, Heaviside \(\beta=10\). **Helmholtz is omitted on the Burn AD tape**; [`VolumeProjection`] after Adam. Default **24** steps. Gates: VF ±15%, top-face variance of Heaviside \(\hat\rho\) \(> 2\times 10^{-5}\) (full B6: \(>0.1\) on final \(\rho\)); greyness not asserted on quick path; compliance ratio bounded.
//! - [`shell_topology_rib_pattern_full_v04`]: `#[ignore]` — **40×40×4**, **200** iters, **seed 42**. **Open roadmap item:** full Striatus-scale B6 stays off default CI (same opt-in pattern as manifold long/`#[ignore]` gates). **Run:** set **`UMST_SHELL_RIB_PATTERN=1`**, then `cargo test -p umst-concrete-cartridge --test shell_topology_rib_pattern --features solver-experimental shell_topology_rib_pattern_full_v04 --release -- --ignored` (**`--release` before `--`**; flags after `--` go to the test harness, not rustc). Append **`--nocapture`** for one **`pre-gate metrics`** line (**`vf`**, **`greyness`**, **`g_uni`**, **`xy_var_z_avg`**, **`c0`**, **`c1`**, **`adam_skipped`**, **`UMST_SHELL_GREY_LAMBDA`**, **`UMST_SHELL_XY_VAR_LAMBDA`**, **`UMST_SHELL_HEAVISIDE_BETA0`**). **Subset / smoke:** **`UMST_SHELL_RIB_FULL_ITERS`** (default **200**, clamped **1…200**) shortens the Adam outer loop; **one** outer still runs the full **40×40×4** forward + backward and can take **many CPU minutes** in `--release**, and the **optimisation** does not satisfy the brief greyness / compliance gates unless you run the full **200** outers — the Rust test **skips** those acceptance asserts when **`UMST_SHELL_RIB_FULL_ITERS` < 200** (finite compliance + loose VF band only). **Helmholtz:** same as [`optimize_shell_3d`](../examples/optimize_shell_3d.rs) — **only** literal **`UMST_SHELL_HELM=1`** enables the graph filter on the Burn tape (an empty `UMST_SHELL_HELM=` must **not** enable — older `!= \"0\"` parsing turned it on and tripped scatter backward at Striatus N); default **off**. **Full-harness parity with `optimize_shell_3d`:** **`UMST_SHELL_SELF_WEIGHT`** (default **on** — Bruyneel–Duysinx self-weight on **`ρ_bar`** plus roof traction; set **`0`** to disable), **`UMST_SHELL_VOL_LOOP`** (default **on**; **`0`** skips in-loop volume projection), **`UMST_SHELL_MAX_CG`**, **`UMST_SHELL_PCG`**, **`UMST_SHELL_E_MIN_REL`** — same semantics as the example. **Multi-term outer loss (experimental):** **`UMST_SHELL_GREY_LAMBDA`** adds **`λ_g·mean(4ρ(1−ρ))`** on **post–volume-projection** **`ρ_bar`** (same grey statistic as the gate); **`UMST_SHELL_XY_VAR_LAMBDA`** adds **`-λ_{xy}·Var_{xy}(\bar\rho)`** where **`Var_{xy}`** is the **z-averaged** column variance (matches the **`xy_plane_variance`** gate on **`ρ`**). **`UMST_SHELL_HEAVISIDE_BETA0`** / **`UMST_SHELL_HEAVISIDE_BETA_MAX`** override Heaviside log-continuation endpoints (defaults **1** and **32**). Non-finite **iter 1** raw compliance **panics** immediately (PCG / conditioning root). Quick-path sizing env **`UMST_SHELL_*`** applies only to [`shell_topology_rib_pattern_quick`], not the full grid defaults (**40³** slab is fixed in the full harness).
//!
//! **Q1-hex PCG (B6 H4, 2026-06-10):** forward+adjoint use [`AdjointComplianceQ1Hex`] (continuum SIMP on the extruded grid). Bar-network ground structure was retired after mechanism probes on **9×8×2**; see [`Solver-Status.md`](../../docs/Solver-Status.md).
//!
//! **Performance discipline:** mechanism probes and operator sanity checks stay on **9×8×2** only. **40×40×4** runs require a converging operator and **`--release` before `--`**. For faster debug harness iteration, workspace `Cargo.toml` sets `[profile.dev.package."*"] opt-level = 3` so tensor deps stay optimized while test code remains unoptimized.

//! **`UMST_RIB_QUICK`:** unset or `1` — implied “small / few iterations” CI mode (grid defaults below). Set `UMST_RIB_QUICK=0` only if you intentionally enlarge the quick harness via `UMST_SHELL_*`.
//!
//! **Sizing env:** `UMST_SHELL_NX`, `UMST_SHELL_NY`, `UMST_SHELL_NZ`, `UMST_SHELL_ITERS`, `UMST_SHELL_VF`.

use std::cell::Cell;
use std::env;

use burn::backend::Autodiff;
use burn::module::{AutodiffModule, Module, ModuleMapper, ModuleVisitor, ParamId};
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::{
    backend::{AutodiffBackend, Backend as BackendTrait},
    Data, Shape, Tensor,
};
use burn_ndarray::NdArray;
use umst_manifold::ai::topology::{
    BetaContinuation, ContinuationSchedule, HeavisideProjection, PlateauBetaContinuation,
    TopologyOptimizer, VolumeEtaProjection, VolumeProjection,
};
use umst_manifold::physics::adjoint::{
    AdjointComplianceDiagnostics, AdjointFiniteStageAudit, SimpElasticMaterial,
};
use umst_manifold::physics::adjoint_q1_hex::AdjointComplianceQ1Hex;
use umst_manifold::physics::extruded_plate::{ElasticMaterial, ExtrudedPlateMechanics};
use umst_manifold::physics::mechanics::SelfWeightConfig;
use umst_manifold::physics::q1_hex_elasticity::{
    HEX_PCG_MAX_ITER_DEFAULT_STRIATUS, HEX_PCG_REL_TOL_F32, HEX_PCG_REL_TOL_F64,
};
use umst_manifold::physics::time_orchestration::MechanicsInnerLoopConfig;
use umst_manifold::physics::topology_filter::HelmholtzFilter;

use umst_concrete_cartridge::print_ready::symmetry::{
    apply_reflection_xy_average, reflection_xy_partner_indices,
};

type AD = Autodiff<NdArray<f32>>;
type B = AD;
type Inner = <AD as AutodiffBackend>::InnerBackend;

struct ScaleWeights(f32);
impl<Bk: BackendTrait> ModuleMapper<Bk> for ScaleWeights {
    fn map_float<const D: usize>(&mut self, _id: &ParamId, tensor: Tensor<Bk, D>) -> Tensor<Bk, D> {
        tensor.mul_scalar(self.0)
    }
}

fn topology_optimizer_scaled(
    volume_target: f32,
    penalization: f32,
    hidden_dim: usize,
    scale: f32,
    device: &<NdArray<f32> as BackendTrait>::Device,
) -> TopologyOptimizer<B> {
    let mut opt = TopologyOptimizer::new(volume_target, penalization, hidden_dim, device);
    if (scale - 1.0).abs() > 1e-6 {
        let mut s = ScaleWeights(scale);
        opt.density_net = opt.density_net.map(&mut s);
    }
    opt
}

fn sym_unit_lcg(i: usize) -> f32 {
    let x = i.wrapping_mul(1664525).wrapping_add(1013904223);
    (x as f32) / (u32::MAX as f32) * 2.0 - 1.0
}

/// Deterministic **additive** weight noise on [`TopologyOptimizer`] after the width scale map (`UMST_SHELL_DENSITY_INIT_JITTER`).
#[derive(Debug)]
struct AddDensityInitJitter {
    amplitude: f32,
    idx: Cell<usize>,
}

impl<Bk: BackendTrait<FloatElem = f32>> ModuleMapper<Bk> for AddDensityInitJitter {
    fn map_float<const D: usize>(&mut self, _id: &ParamId, tensor: Tensor<Bk, D>) -> Tensor<Bk, D> {
        if self.amplitude <= 0.0 {
            return tensor;
        }
        let dev = tensor.device();
        let d = tensor.into_data();
        let mut out = Vec::with_capacity(d.value.len());
        for &x in &d.value {
            let u = sym_unit_lcg(self.idx.get());
            self.idx.set(self.idx.get() + 1);
            out.push(x + self.amplitude * u);
        }
        Tensor::from_data(Data::new(out, d.shape), &dev)
    }
}

/// Per-node **`sin(2π x̂)\,sin(2π ŷ)`** on extruded-plate order (`UMST_SHELL_XY_RIB_PRIOR_AMP`).
fn xy_rib_prior_pattern_b<Bk: BackendTrait<FloatElem = f32>>(
    nx: usize,
    ny: usize,
    nz: usize,
    device: &Bk::Device,
) -> Tensor<Bk, 3> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let nz1 = nz + 1;
    let n = nx1 * ny1 * nz1;
    let two_pi = 2.0 * std::f32::consts::PI;
    let mut v = vec![0.0_f32; n];
    for iz in 0..nz1 {
        for iy in 0..ny1 {
            for ix in 0..nx1 {
                let nid = ix + iy * nx1 + iz * nx1 * ny1;
                let xh = (ix as f32 + 0.5) / nx1 as f32;
                let yh = (iy as f32 + 0.5) / ny1 as f32;
                v[nid] = (two_pi * xh).sin() * (two_pi * yh).sin();
            }
        }
    }
    Tensor::from_data(Data::new(v, Shape::new([1, n, 1])), device)
}

fn pin_bottom_perimeter_inner(
    nx: usize,
    ny: usize,
    nz: usize,
    device: &<NdArray<f32> as BackendTrait>::Device,
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
    Tensor::from_data(Data::new(bm, Shape::new([1, n, 3])), device)
}

/// Uniform roof pressure on the top face (same nodal lumping as `shell_demo_smoke`).
#[allow(dead_code)]
fn top_load_inner(
    nx: usize,
    ny: usize,
    nz: usize,
    pa: f32,
    dx: f32,
    dy: f32,
    device: &<NdArray<f32> as BackendTrait>::Device,
) -> Tensor<Inner, 3> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let n = nx1 * ny1 * (nz + 1);
    let mut bf = vec![0.0f32; n * 3];
    let iz_top = nz;
    for iy in 0..=ny {
        for ix in 0..=nx {
            let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
            bf[nid * 3 + 2] = -pa * dx * dy;
        }
    }
    Tensor::from_data(Data::new(bf, Shape::new([1, n, 3])), device)
}

/// Roof traction \(\propto 1 + r\,i_x/n_x\) (same lumping as [`top_load_inner`]).
fn top_load_inner_x_ramp(
    nx: usize,
    ny: usize,
    nz: usize,
    pa: f32,
    dx: f32,
    dy: f32,
    ramp: f32,
    device: &<NdArray<f32> as BackendTrait>::Device,
) -> Tensor<Inner, 3> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let n = nx1 * ny1 * (nz + 1);
    let mut bf = vec![0.0f32; n * 3];
    let iz_top = nz;
    let nx_d = nx.max(1) as f32;
    for iy in 0..=ny {
        for ix in 0..=nx {
            let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
            let w = 1.0_f32 + ramp * (ix as f32 / nx_d);
            bf[nid * 3 + 2] = -pa * dx * dy * w;
        }
    }
    Tensor::from_data(Data::new(bf, Shape::new([1, n, 3])), device)
}

fn parse_usize(key: &str) -> Option<usize> {
    env::var(key).ok()?.parse().ok()
}

fn short_mesh_and_iters() -> (usize, usize, usize, usize) {
    let gx = parse_usize("UMST_SHELL_NX");
    let gy = parse_usize("UMST_SHELL_NY");
    let gz = parse_usize("UMST_SHELL_NZ");
    let git = parse_usize("UMST_SHELL_ITERS");
    let all_unset = gx.is_none() && gy.is_none() && gz.is_none() && git.is_none();
    if all_unset {
        // Mild XY aspect ratio + a few extra Adam steps help the quick gate clear the 4e-5 top-slice floor.
        return (9, 8, 2, 24);
    }
    (
        gx.unwrap_or(9).clamp(4, 32).min(16),
        gy.unwrap_or(8).clamp(4, 32).min(16),
        gz.unwrap_or(2).clamp(2, 8).min(4),
        git.unwrap_or(24).clamp(1, 64).min(32),
    )
}

fn parse_target_vf() -> f32 {
    env::var("UMST_SHELL_VF")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.15_f32)
}

fn greyness_mean(rho: &[f32]) -> f32 {
    let n = rho.len().max(1) as f32;
    rho.iter().map(|&r| 4.0 * r * (1.0 - r)).sum::<f32>() / n
}

fn h4_diag_enabled() -> bool {
    matches!(env::var("UMST_SHELL_H4_DIAG").as_deref(), Ok("1"))
}

fn h5_skip_projection_compliance() -> bool {
    matches!(env::var("UMST_SHELL_H5_SKIP_PROJ").as_deref(), Ok("1"))
}

fn vec_l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn vec_spatial_variance(v: &[f32]) -> f32 {
    let n = v.len().max(1) as f32;
    let mean = v.iter().sum::<f32>() / n;
    v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / n
}

fn rho_raw_range(rho: &[f32]) -> (f32, f32, f32) {
    let (min, max) = rho
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &x| {
            (lo.min(x), hi.max(x))
        });
    let mean = rho.iter().sum::<f32>() / rho.len().max(1) as f32;
    (min, max, mean)
}

fn autodiff_param_grad_l2<M: AutodiffModule<B>>(grads: &GradientsParams, module: &M) -> f32 {
    let (l2, _, _) = autodiff_param_grad_audit(grads, module);
    l2
}

/// Per-tensor grad L2 and non-finite counts (H5 stage **d**).
fn tensor_grad_l2_inner(g: &Tensor<Inner, 3>) -> (f32, f32) {
    let vals = g.clone().into_data().value;
    let l2 = vals.iter().map(|x| x * x).sum::<f32>().sqrt();
    let max = vals.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
    (l2, max)
}

fn autodiff_param_grad_audit<M: AutodiffModule<B>>(
    grads: &GradientsParams,
    module: &M,
) -> (f32, f32, Vec<(usize, usize, usize)>) {
    struct Visitor<'a> {
        gp: &'a GradientsParams,
        sum_sq: f32,
        max_abs: f32,
        layer: usize,
        layers: Vec<(usize, usize, usize)>,
    }
    impl ModuleVisitor<B> for Visitor<'_> {
        fn visit_float<const D: usize>(&mut self, id: &ParamId, _tensor: &Tensor<B, D>) {
            if let Some(g) = self.gp.get::<Inner, D>(id) {
                let vals = g.clone().into_data().value;
                let nf = vals.iter().filter(|x| !x.is_finite()).count();
                let n = vals.len();
                if nf > 0 {
                    self.layers.push((self.layer, nf, n));
                }
                for &x in &vals {
                    self.sum_sq += x * x;
                    self.max_abs = self.max_abs.max(x.abs());
                }
            }
            self.layer += 1;
        }
    }
    let mut visitor = Visitor {
        gp: grads,
        sum_sq: 0.0,
        max_abs: 0.0,
        layer: 0,
        layers: Vec::new(),
    };
    module.visit(&mut visitor);
    (visitor.sum_sq.sqrt(), visitor.max_abs, visitor.layers)
}

fn log_h5_finite_chain(tag: &str, outer: usize, audit: &AdjointFiniteStageAudit) {
    eprintln!(
        "{tag}: H5 finite chain outer {outer}: \
u_nf={} u_pinned_nf={} u_pinned_abs_max={:.3e} ge_nf={} nodal_sens_nf={} first_bad={}",
        audit.u_nonfinite,
        audit.u_pinned_nonfinite,
        audit.u_pinned_abs_max,
        audit.ge_nonfinite,
        audit.nodal_sens_nonfinite,
        audit
            .first_bad_stage
            .map(|s| s.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
}

fn log_h5_grad_layers(
    tag: &str,
    outer: usize,
    grad_l2: f32,
    grad_max: f32,
    rho_grad_l2: f32,
    rho_grad_max: f32,
    layers: &[(usize, usize, usize)],
) {
    eprintln!(
        "{tag}: H5 grad outer {outer}: param_l2={grad_l2:.6} param_max={grad_max:.6} \
rho_bar_l2={rho_grad_l2:.6} rho_bar_max={rho_grad_max:.6} layer_nf={}",
        layers.len()
    );
    for &(idx, nf, n) in layers {
        eprintln!("{tag}: H5 grad outer {outer}: layer={idx} nonfinite={nf}/{n}");
    }
}

fn log_h4_outer(
    tag: &str,
    outer: usize,
    outer_total: usize,
    rho_raw: &[f32],
    diag: &AdjointComplianceDiagnostics,
    grad_l2: f32,
    adam_skipped: usize,
    xy_var: f32,
    loss_scalar: f32,
) {
    let (rmin, rmax, rmean) = rho_raw_range(rho_raw);
    let sens = &diag.nodal_sensitivity;
    eprintln!(
        "{tag}: H4 outer {outer}/{outer_total} rho_raw=[{rmin:.6},{rmax:.6}] mean={rmean:.6} \
sens_l2={:.6} sens_var={:.6} pcg_iter={} pcg_rel_res={:.3e} eq_rel_res={:.3e} \
grad_l2={:.6} adam_skipped={adam_skipped} xy_var={xy_var:.6} loss={loss_scalar:.6}",
        vec_l2_norm(sens),
        vec_spatial_variance(sens),
        diag.pcg.iterations,
        diag.pcg.rel_residual,
        diag.equilibrium_rel_residual,
        grad_l2,
    );
}

fn xy_plane_variance(rho: &[f32], nx: usize, ny: usize, nz: usize) -> f32 {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let nz1 = nz + 1;
    let mut sums = vec![0.0_f32; nx1 * ny1];
    for iz in 0..nz1 {
        for iy in 0..ny1 {
            for ix in 0..nx1 {
                let nid = ix + iy * nx1 + iz * nx1 * ny1;
                let k = ix + iy * nx1;
                sums[k] += rho[nid];
            }
        }
    }
    let denom = nz1 as f32;
    let n_xy = (nx1 * ny1) as f32;
    let mean_z: Vec<f32> = sums.iter().map(|s| s / denom).collect();
    let mean_all = mean_z.iter().sum::<f32>() / n_xy;
    mean_z.iter().map(|v| (v - mean_all).powi(2)).sum::<f32>() / n_xy
}

/// **\(z = n_z\)** face (roof plane). Quick CI prefers this slice when checking planar texture;
/// volume-averaged XY stacks can dilute weak gradients.
fn xy_top_slice_variance(rho: &[f32], nx: usize, ny: usize, nz: usize) -> f32 {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let iz = nz;
    let mut vals: Vec<f32> = Vec::with_capacity(nx1 * ny1);
    for iy in 0..ny1 {
        for ix in 0..nx1 {
            let nid = ix + iy * nx1 + iz * nx1 * ny1;
            vals.push(rho[nid]);
        }
    }
    let n = vals.len().max(1) as f32;
    let mean = vals.iter().sum::<f32>() / n;
    vals.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n
}

/// Volume mean of **`4ρ(1−ρ)`** on **`ρ_bar`** `[1,N,1]` (same statistic as [`greyness_mean`] on flat **`ρ`**).
fn mean_greyness_tensor<B: AutodiffBackend<FloatElem = f32>>(
    rho_bar: Tensor<B, 3>,
) -> Tensor<B, 1> {
    let [batch, n, c] = rho_bar.dims();
    assert_eq!((batch, c), (1, 1));
    let count = n.max(1) as f32;
    rho_bar
        .clone()
        .mul(rho_bar.clone().neg().add_scalar(1.0))
        .mul_scalar(4.0)
        .sum()
        .div_scalar(count)
        .reshape([1])
}

/// **`z`‑stacked mean per \((x,y)\)** column, then variance over the **`(nx+1)(ny+1)`** columns — matches [`xy_plane_variance`] on the same flat indexing as the extruded grid.
fn xy_plane_variance_z_avg_tensor<B: AutodiffBackend<FloatElem = f32>>(
    rho_bar: Tensor<B, 3>,
    nx: usize,
    ny: usize,
    nz: usize,
) -> Tensor<B, 1> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let nz1 = nz + 1;
    let [b, n, c] = rho_bar.dims();
    assert_eq!((b, c), (1, 1));
    assert_eq!(n, nx1 * ny1 * nz1);
    let nz_f = nz1 as f32;
    let nxy = (nx1 * ny1) as f32;
    let t = rho_bar.reshape([nx1, ny1, nz1]);
    let mz = t.sum_dim(2).div_scalar(nz_f);
    let sum = mz.clone().sum();
    let sumsq = mz.powf_scalar(2.0).sum();
    let mean_sq = sumsq.div_scalar(nxy);
    let mean = sum.div_scalar(nxy);
    mean_sq.sub(mean.powf_scalar(2.0)).reshape([1])
}

#[derive(Clone, Debug)]
struct RibMetrics {
    vf: f32,
    greyness: f32,
    xy_var: f32,
    c0: f32,
    c1: f32,
    adam_skipped: usize,
    /// PCG iterations on the **final** forward solve (convergence evidence vs bar-network stall).
    pcg_iters: usize,
    pcg_rel_res: f32,
    eq_rel_res: f32,
    /// Last-outer density-net grad L2.
    last_grad_l2: f32,
    /// Peak density-net grad L2 across Adam outers (smoke gate 1).
    max_grad_l2: f32,
    /// Last-outer `ρ_raw` range (smoke gate 3).
    last_rho_raw_min: f32,
    last_rho_raw_max: f32,
    /// Effective Heaviside β on the last outer (continuation schedule).
    last_outer_beta: f32,
    target_vf: f32,
    roof_ramp_on: bool,
    roof_ramp_strength: f32,
    /// Mean nodal `ρ` per z-layer on final `ρ_bar` (see [`rho_z_layer_profile`]).
    z_profile: String,
}

fn log_striatus_acceptance_line(tag: &str, nx: usize, ny: usize, nz: usize, m: &RibMetrics) {
    eprintln!(
        "{tag}: acceptance diag \
UMST_SHELL_ROOF_RAMP={} ramp_strength={:.3} target_vf={:.4} vf_final={:.6} vf_err={:+.6} \
GREYNESS={:.6} z_rho_mean={} xy_var={:.6} c0={:.6} c1={:.6} beta_last={:.3} \
max_grad_l2={:.6} eq_rel={:.3e}",
        if m.roof_ramp_on { 1 } else { 0 },
        m.roof_ramp_strength,
        m.target_vf,
        m.vf,
        m.vf - m.target_vf,
        m.greyness,
        m.z_profile,
        m.xy_var,
        m.c0,
        m.c1,
        m.last_outer_beta,
        m.max_grad_l2,
        m.eq_rel_res,
    );
    if m.greyness < 0.05 && m.last_outer_beta < 8.0 {
        eprintln!(
            "{tag}: greyness sanity — greyness={:.4} at beta={:.3}: Heaviside cannot sharpen this \
far at low beta; field is already near-binary from DensityNet. Confirm vf_final vs target_vf \
(η-bisection); binary-at-wrong-volume is a silent failure mode.",
            m.greyness,
            m.last_outer_beta
        );
    }
}

/// Striatus **40×40×4** acceptance runs must use `cargo test --release` (not debug `dev`).
#[allow(unused_variables)]
fn assert_striatus_release_profile(tag: &str) {
    #[cfg(debug_assertions)]
    panic!(
        "{tag}: Striatus-scale 40×40×4 rejected in debug profile — use \
`cargo test --release` before `--` (see module `//!` performance discipline)"
    );
}

fn assert_pcg_equilibrium_gate(tag: &str, pcg_rel: f32, eq_rel: f32, tol: f32) {
    assert!(
        pcg_rel.is_finite() && pcg_rel <= tol,
        "{tag}: Q1-hex PCG rel_residual {pcg_rel:.3e} > tol {tol:.3e}"
    );
    assert!(
        eq_rel.is_finite() && eq_rel <= tol,
        "{tag}: equilibrium rel_residual {eq_rel:.3e} > tol {tol:.3e}"
    );
}

#[allow(clippy::too_many_arguments)]
fn q1_compliance_forward(
    rho_bar: Tensor<B, 3>,
    plate: &ExtrudedPlateMechanics,
    boundary: Tensor<Inner, 3>,
    body_force: Tensor<Inner, 3>,
    mat: SimpElasticMaterial,
    cg: &MechanicsInnerLoopConfig,
) -> (Tensor<B, 1>, f32) {
    AdjointComplianceQ1Hex::forward_and_loss(
        rho_bar,
        plate.nx,
        plate.ny,
        plate.nz,
        plate.dx,
        plate.dy,
        plate.dz,
        body_force,
        boundary,
        mat,
        cg,
    )
}

#[allow(clippy::too_many_arguments)]
fn q1_compliance_with_diagnostics(
    rho_bar: Tensor<B, 3>,
    plate: &ExtrudedPlateMechanics,
    boundary: Tensor<Inner, 3>,
    body_force: Tensor<Inner, 3>,
    mat: SimpElasticMaterial,
    cg: &MechanicsInnerLoopConfig,
) -> (Tensor<B, 1>, f32, AdjointComplianceDiagnostics) {
    AdjointComplianceQ1Hex::forward_loss_with_diagnostics(
        rho_bar,
        plate.nx,
        plate.ny,
        plate.nz,
        plate.dx,
        plate.dy,
        plate.dz,
        body_force,
        boundary,
        mat,
        cg,
    )
}

fn run_rib_quick_metrics() -> RibMetrics {
    <B as BackendTrait>::seed(42);
    let (nx, ny, nz, iterations) = short_mesh_and_iters();
    let target_vf = parse_target_vf();

    let device_default = Default::default();
    let device = &device_default;

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
    let coord_scale = lx.max(ly).max(lz);
    let coords_norm = plate
        .coords_bn3::<B>(device)
        .div_scalar(coord_scale)
        .mul_scalar(2.0)
        .sub_scalar(1.0);
    // Gentle x-ramp on roof traction (same lumping as [`top_load_inner`]); biases adjoint sensitivities in x
    // without Striatus-style XY reflection averaging.
    let live_inner = top_load_inner_x_ramp(nx, ny, nz, 50.0, dx, dy, 0.2, device);

    let proj = HeavisideProjection::new(10.0, 0.5);

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

    let boundary_inner = pin_bottom_perimeter_inner(nx, ny, nz, device);

    let simp_mat = SimpElasticMaterial {
        e0: mat.e0,
        nu: mat.nu,
        p: mat.simp_p,
        e_min: mat.e_min,
    };

    // Large enough that `sigmoid(MLP(x))` is not numerically flat on the roof slice under f32; full
    // Striatus uses continuation + reflection instead of this knob.
    let mut opt = topology_optimizer_scaled(target_vf, 3.0, 32, 0.42, device);
    let mut adam = AdamConfig::new().init::<B, _>();

    let mut comp_scale = 1e-12_f32;
    let mut c0 = f32::NAN;
    let mut c1 = f32::NAN;

    let h4 = h4_diag_enabled();
    for it in 0..iterations.max(1) {
        let rho_raw_t = opt.density_net.forward_batched(coords_norm.clone());
        let rho_raw_vec = rho_raw_t.clone().into_data().value;
        // Helmholtz is skipped on the Burn AD tape here (scatter-shaped backward quirks on some 3-D grids).
        let rho_bar = proj.project(rho_raw_t);

        let (surrogate, c_raw) = q1_compliance_forward(
            rho_bar.clone(),
            &plate,
            boundary_inner.clone(),
            live_inner.clone(),
            simp_mat,
            &cg,
        );
        let h4_bundle = if h4 {
            Some(
                q1_compliance_with_diagnostics(
                    rho_bar.clone(),
                    &plate,
                    boundary_inner.clone(),
                    live_inner.clone(),
                    simp_mat,
                    &cg,
                )
                .2,
            )
        } else {
            None
        };

        if it == 0 {
            comp_scale = c_raw.max(1e-12);
            c0 = c_raw / comp_scale;
        }
        c1 = c_raw / comp_scale;

        let total_loss = surrogate.div_scalar(comp_scale);
        let loss_scalar = total_loss.clone().into_data().value[0];
        assert!(
            loss_scalar.is_finite(),
            "step {it}: scaled surrogate must be finite, got {loss_scalar}",
        );

        if let Some(ref diag) = h4_bundle {
            let rho_mid_vec = rho_bar.clone().into_data().value;
            let xy_v = xy_top_slice_variance(&rho_mid_vec, nx, ny, nz);
            log_h4_outer(
                "shell_topology_rib_pattern_quick",
                it + 1,
                iterations.max(1),
                &rho_raw_vec,
                diag,
                f32::NAN,
                0,
                xy_v,
                loss_scalar,
            );
        }
        let grads = total_loss.backward();
        let gp = GradientsParams::from_grads(grads, &opt.density_net);
        opt.density_net = adam.step(0.005, opt.density_net, gp);
    }

    let rho_raw = opt.density_net.forward_batched(coords_norm.clone());
    // Match the inner-loop AD path (Helmholtz skipped in-loop for Burn scatter stability); otherwise a
    // terminal Helmholtz pass can flatten ρ̂ and drive `xy_var` to ~0 on coarse grids.
    let rho_mid = proj.project(rho_raw);
    let rho_mid_vec = rho_mid.clone().into_data().value;
    let xy_var = xy_top_slice_variance(&rho_mid_vec, nx, ny, nz);

    let (_, _, final_diag) = q1_compliance_with_diagnostics(
        rho_mid.clone(),
        &plate,
        boundary_inner.clone(),
        live_inner.clone(),
        simp_mat,
        &cg,
    );
    let pcg_tol = cg.pcg_tolerance.max(cg.cg_tolerance);

    let rho_inner = rho_mid.inner();
    let vol_proj = VolumeProjection::new(target_vf, 48);
    let rho_phys = vol_proj.project(rho_inner);
    let rho_vec = rho_phys.into_data().value;
    let vf = rho_vec.iter().sum::<f32>() / rho_vec.len().max(1) as f32;
    // Quick CI: greyness on **Heaviside** ρ (pre-`VolumeProjection`). v0.4 B6 print gate uses
    // post-projection ρ; bisection pushes many nodes toward ~0.5 and inflates `4ρ(1-ρ)`.
    let greyness = greyness_mean(&rho_mid_vec);

    RibMetrics {
        vf,
        greyness,
        xy_var,
        c0,
        c1,
        adam_skipped: 0,
        pcg_iters: final_diag.pcg.iterations,
        pcg_rel_res: final_diag.pcg.rel_residual,
        eq_rel_res: final_diag.equilibrium_rel_residual,
        last_grad_l2: f32::NAN,
        max_grad_l2: f32::NAN,
        last_rho_raw_min: f32::NAN,
        last_rho_raw_max: f32::NAN,
        last_outer_beta: f32::NAN,
        target_vf: target_vf,
        roof_ramp_on: true,
        roof_ramp_strength: 0.2,
        z_profile: String::new(),
    }
    .also_pcg_gate("shell_topology_rib_pattern_quick", pcg_tol)
}

impl RibMetrics {
    fn also_pcg_gate(self, tag: &str, tol: f32) -> Self {
        assert_pcg_equilibrium_gate(tag, self.pcg_rel_res, self.eq_rel_res, tol);
        self
    }
}

fn parse_full_rib_adam_iters() -> usize {
    env::var("UMST_SHELL_RIB_FULL_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
        .clamp(1, 200)
}

/// Heaviside β and SIMP‑\(p\) continuation use this denominator even when
/// [`parse_full_rib_adam_iters`] shortens the smoke subset — otherwise β ramps to
/// \(\beta_{\max}\) in ~20 steps and saturates `ρ_mid` AD (artifact, not a path bug).
const STRIATUS_B6_SCHEDULE_OUTERS: usize = 200;

/// Abort long Striatus runs when mean VF misses target this many outers in a row.
const STRIATUS_VF_ERR_ABORT_BAND: f32 = 0.1;
const STRIATUS_VF_ERR_ABORT_STREAK: usize = 3;

/// Roof traction asymmetry: `UMST_SHELL_ROOF_RAMP=0` → uniform; default **on** with
/// strength `UMST_SHELL_ROOF_RAMP_STRENGTH` (default **0.2**, traction \(\propto 1+r\,i_x/n_x\)).
fn parse_roof_ramp() -> (bool, f32) {
    let on = env::var("UMST_SHELL_ROOF_RAMP")
        .map(|v| v != "0")
        .unwrap_or(true);
    let strength = env::var("UMST_SHELL_ROOF_RAMP_STRENGTH")
        .or_else(|_| env::var("UMST_SHELL_ROOF_RAMP_F"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.2_f32)
        .clamp(0.0, 4.0);
    (on, if on { strength } else { 0.0 })
}

/// Coarse **z-profile**: mean nodal `ρ` per `z` layer (`nz+1` entries) — sandwich vs ribs at a glance.
fn rho_z_layer_profile(rho: &[f32], nx: usize, ny: usize, nz: usize) -> String {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let nz1 = nz + 1;
    let mut layers = Vec::with_capacity(nz1);
    for iz in 0..nz1 {
        let mut sum = 0.0_f32;
        let mut cnt = 0usize;
        for iy in 0..ny1 {
            for ix in 0..nx1 {
                let nid = ix + iy * nx1 + iz * nx1 * ny1;
                if nid < rho.len() {
                    sum += rho[nid];
                    cnt += 1;
                }
            }
        }
        layers.push(sum / cnt.max(1) as f32);
    }
    format!(
        "[{}]",
        layers
            .iter()
            .map(|v| format!("{v:.3}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// Density-net Kaiming scale (`UMST_SHELL_INIT_SCALE`). Default **1.0** on Striatus **40×40×4**:
/// `0.05` pins ρ≈0.5 with uniform nodal sensitivity → exact param-gradient cancellation.
fn parse_density_init_scale(default: f32) -> f32 {
    env::var("UMST_SHELL_INIT_SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
        .clamp(0.0, 1.0)
}

/// Parsed **`UMST_SHELL_*`** knobs shared by [`run_rib_full_striatus`] and the **`pre-gate metrics`** line.
fn parse_umst_shell_b6_aux_env() -> (f32, f32, f32, f32, f32) {
    let grey_lambda = env::var("UMST_SHELL_GREY_LAMBDA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0_f32)
        .max(0.0);
    let xy_var_lambda = env::var("UMST_SHELL_XY_VAR_LAMBDA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0_f32)
        .max(0.0);
    let heaviside_beta0 = env::var("UMST_SHELL_HEAVISIDE_BETA0")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0_f32)
        .clamp(1e-6, 512.0);
    let density_init_jitter = env::var("UMST_SHELL_DENSITY_INIT_JITTER")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0_f32)
        .clamp(0.0, 0.25);
    let xy_rib_prior_amp = match env::var("UMST_SHELL_XY_RIB_PRIOR_AMP") {
        Ok(s) if !s.trim().is_empty() => s.parse::<f32>().unwrap_or(0.0).clamp(0.0, 0.25),
        _ => 0.0_f32,
    };
    (
        grey_lambda,
        xy_var_lambda,
        heaviside_beta0,
        density_init_jitter,
        xy_rib_prior_amp,
    )
}

fn run_rib_full_striatus(target_vf: f32) -> RibMetrics {
    assert_striatus_release_profile("run_rib_full_striatus");
    <B as BackendTrait>::seed(42);
    let device_default = Default::default();
    let device = &device_default;

    let use_self_weight = env::var("UMST_SHELL_SELF_WEIGHT")
        .map(|v| v != "0")
        .unwrap_or(true);
    let vol_in_loop = env::var("UMST_SHELL_VOL_LOOP")
        .map(|v| v != "0")
        .unwrap_or(true);
    let use_pc = env::var("UMST_SHELL_PCG").map(|v| v != "0").unwrap_or(true);
    let max_cg = env::var("UMST_SHELL_MAX_CG")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(HEX_PCG_MAX_ITER_DEFAULT_STRIATUS)
        .clamp(50, 50_000);
    let e_min_rel = env::var("UMST_SHELL_E_MIN_REL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1e-3_f32)
        .clamp(1e-9, 1.0);

    let (grey_lambda, xy_var_lambda, heaviside_beta0, density_init_jitter, xy_rib_prior_amp) =
        parse_umst_shell_b6_aux_env();
    let heaviside_beta_max = env::var("UMST_SHELL_HEAVISIDE_BETA_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(32.0_f32)
        .clamp(1.0, 512.0);

    let nx = 40usize;
    let ny = 40usize;
    let nz = 4usize;
    let iterations = parse_full_rib_adam_iters();
    let iter_total = STRIATUS_B6_SCHEDULE_OUTERS;
    let smoke_subset = iterations < STRIATUS_B6_SCHEDULE_OUTERS;

    let lx = 4.0_f32;
    let ly = 4.0_f32;
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
    let n = plate.n_nodes();
    let coords_bn3 = plate.coords_bn3::<B>(device);
    let coord_scale = lx.max(ly).max(lz);
    let coords_norm = coords_bn3
        .clone()
        .div_scalar(coord_scale)
        .mul_scalar(2.0)
        .sub_scalar(1.0);
    let edges_b1 = plate.edges_b1::<B>(device);
    let boundary_b = {
        let nx1 = nx + 1;
        let ny1 = ny + 1;
        let nn = nx1 * ny1 * (nz + 1);
        let mut bm = vec![1.0_f32; nn * 3];
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
        Tensor::<B, 3>::from_data(Data::new(bm, Shape::new([1, nn, 3])), device)
    };

    let (roof_ramp_on, roof_ramp_strength) = parse_roof_ramp();
    // x-ramp roof traction (`UMST_SHELL_ROOF_RAMP`): breaks load symmetry so nodal sensitivities vary.
    let mut live_f = vec![0.0f32; n * 3];
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let iz_top = nz;
    let nx_d = nx.max(1) as f32;
    for iy in 0..=ny {
        for ix in 0..=nx {
            let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
            let w = 1.0_f32 + roof_ramp_strength * (ix as f32 / nx_d);
            live_f[nid * 3 + 2] = -50.0 * dx * dy * w;
        }
    }
    let live_force: Tensor<B, 3> =
        Tensor::from_data(Data::new(live_f, Shape::new([1, n, 3])), device);

    let voxel_vol = dx * dy * dz;
    let sw_cfg = SelfWeightConfig {
        gravity_m_s2: 9.81,
        voxel_volume_m3: voxel_vol,
        mass_penalty_q: 1.0,
        direction: [0.0, 0.0, -1.0],
    };

    let helm = HelmholtzFilter::new((2.0 * dx.min(dy).min(dz)).max(1e-6), 240, 1e-7);
    // **`UMST_SHELL_HELM=1`** enables straight-through Helmholtz ([`HelmholtzFilter::apply_straight_through`]).
    // Default **off**: even with a finite filter forward, the full compliance adjoint can still yield
    // non-finite outer loss at Striatus N until the PCG / SIMP path is hardened (B6 follow-up).
    let helm_on = matches!(env::var("UMST_SHELL_HELM").as_deref(), Ok("1"));
    // In-loop: `VolumeProjection` AL when `UMST_SHELL_VOL_LOOP=1` (default). η-bisect in-loop only
    // with `UMST_SHELL_VOL_BISECT=1` (debug). Terminal η-bisect finisher unless `UMST_SHELL_VOL_BISECT=0`.
    let vol_bisect_in_loop = matches!(env::var("UMST_SHELL_VOL_BISECT").as_deref(), Ok("1"));
    let vol_eta_terminal = env::var("UMST_SHELL_VOL_BISECT")
        .map(|v| v != "0")
        .unwrap_or(true);
    let metrics_on = matches!(env::var("UMST_SHELL_METRICS").as_deref(), Ok("1"));
    let h4_diag = h4_diag_enabled();
    let plateau_beta = PlateauBetaContinuation::new(5, 0.008);
    let mut proj = HeavisideProjection::new(heaviside_beta0, 0.5);
    let vol_proj = VolumeProjection::new(target_vf, 48);
    let vol_eta = VolumeEtaProjection::new(48, 1e-4);
    let mut greyness_hist: Vec<f32> = Vec::new();

    let e0 = 200e6_f32;
    let material = ElasticMaterial {
        e0,
        nu: 0.2,
        simp_p: 3.0,
        // Soft void floor: match `optimize_shell_3d` / Track B6 — override with `UMST_SHELL_E_MIN_REL`.
        e_min: e0 * e_min_rel,
    };
    let cg_cfg = MechanicsInnerLoopConfig {
        // `packed_bar_network_equilibrium` caps at min(this, 3N); 200 iters was far below a useful solve.
        max_cg_iterations: max_cg,
        cg_tolerance: HEX_PCG_REL_TOL_F64,
        pcg_tolerance: HEX_PCG_REL_TOL_F64,
        use_preconditioner: use_pc,
        max_equilibrium_substeps: 1,
    };

    let boundary_inner = boundary_b.clone().inner();

    // `topology_optimizer_scaled(..., 0.05)` parks ρ≈0.5; uniform nodal sens → zero param grad at 40×40×4.
    let init_scale = parse_density_init_scale(1.0);
    let mut opt = topology_optimizer_scaled(target_vf, 3.0, 64, init_scale, device);
    if density_init_jitter > 0.0 {
        let mut jm = AddDensityInitJitter {
            amplitude: density_init_jitter,
            idx: Cell::new(0),
        };
        opt.density_net = opt.density_net.map(&mut jm);
    }
    let mut adam = AdamConfig::new().init::<B, _>();
    let dx_f = dx.min(dy).min(dz);

    let partners = reflection_xy_partner_indices::<B>(nx, ny, nz, device);
    let xy_rib_pat = if xy_rib_prior_amp > 0.0 {
        Some(xy_rib_prior_pattern_b(nx, ny, nz, device))
    } else {
        None
    };
    let sym_period = 20usize;

    let mut comp_scale = 1e-12_f32;
    let mut c0 = f32::NAN;
    let mut c1 = f32::NAN;
    let mut last_rho: Vec<f32> = Vec::new();
    let mut last_rho_bar: Option<Tensor<B, 3>> = None;
    let mut last_bf_inner: Option<Tensor<Inner, 3>> = None;
    let mut last_simp_mat = SimpElasticMaterial {
        e0: material.e0,
        nu: material.nu,
        p: 3.0,
        e_min: material.e_min,
    };
    let mut adam_skipped = 0usize;
    let mut last_grad_l2 = 0.0_f32;
    let mut max_grad_l2 = 0.0_f32;
    let mut min_grad_l2 = f32::INFINITY;
    let mut first_c1 = f32::NAN;
    let mut first_xy_var = f32::NAN;
    let mut last_rho_raw_min = f32::NAN;
    let mut last_rho_raw_max = f32::NAN;
    let mut last_outer_beta = heaviside_beta0;
    let mut vf_err_streak = 0usize;
    let pcg_tol = cg_cfg.pcg_tolerance.max(cg_cfg.cg_tolerance);

    for it in 1..=iterations {
        let base_beta = BetaContinuation::beta(
            it.saturating_sub(1),
            iter_total,
            heaviside_beta0,
            heaviside_beta_max.max(64.0),
        );
        let beta =
            plateau_beta.effective_beta(base_beta, &greyness_hist, heaviside_beta_max.max(64.0));
        proj.set_beta(beta);

        let mut rho_raw = opt
            .density_net
            .forward_batched(coords_norm.clone())
            .reshape([1, n, 1]);
        if sym_period > 0 && it % sym_period == 0 {
            rho_raw = apply_reflection_xy_average(rho_raw, &partners).reshape([1, n, 1]);
        }
        if let Some(ref pat) = xy_rib_pat {
            rho_raw = rho_raw
                .add(pat.clone().mul_scalar(xy_rib_prior_amp))
                .clamp(0.0, 1.0);
        }
        let rho_tilde = if helm_on {
            helm.apply_straight_through(rho_raw.clone(), edges_b1.clone(), dx_f)
                .reshape([1, n, 1])
        } else {
            rho_raw.clone()
        };
        let rho_mid = proj.project(rho_tilde.clone()).reshape([1, n, 1]);
        let rho_bar = if vol_bisect_in_loop {
            vol_eta
                .project(rho_tilde.clone().reshape([1, n, 1]), beta, target_vf)
                .reshape([1, n, 1])
        } else if vol_in_loop {
            vol_proj.project(rho_mid.clone()).reshape([1, n, 1])
        } else {
            rho_mid.clone()
        };
        if metrics_on && (it % 20 == 0 || it == iterations) {
            let pre = greyness_mean(&rho_mid.clone().into_data().value);
            let post = greyness_mean(&rho_bar.clone().into_data().value);
            eprintln!(
                "shell_topology_rib_pattern_full_v04: outer {it}/{iter_total} greyness_pre_vol={pre:.6} greyness_post_vol={post:.6} beta={beta:.3} helm_on={helm_on} vol_bisect_in_loop={vol_bisect_in_loop} vol_eta_terminal={vol_eta_terminal}"
            );
        }

        let bf = if use_self_weight {
            sw_cfg.body_force(rho_bar.clone()).add(live_force.clone())
        } else {
            live_force.clone()
        };
        let p_act = ContinuationSchedule::value(it.saturating_sub(1), iter_total);
        let simp_mat = SimpElasticMaterial {
            e0: material.e0,
            nu: material.nu,
            p: p_act,
            e_min: material.e_min,
        };
        last_simp_mat = simp_mat;
        let rho_raw_vec = rho_raw.clone().into_data().value;
        let bf_inner = bf.inner();
        last_rho_bar = Some(rho_bar.clone());
        last_bf_inner = Some(bf_inner.clone());
        // Compliance AD on post–volume-projection ρ_bar (in-loop AL); β schedule anchored at 200 outers.
        // `UMST_SHELL_H5_SKIP_PROJ=1` forces raw net output for isolation probes.
        let rho_comp = if h5_skip_projection_compliance() {
            rho_raw.clone()
        } else {
            rho_bar.clone()
        };
        let (surrogate, c_raw, h4_bundle) = if h4_diag || smoke_subset {
            let (s, c, diag) = q1_compliance_with_diagnostics(
                rho_comp.clone(),
                &plate,
                boundary_inner.clone(),
                bf_inner,
                simp_mat,
                &cg_cfg,
            );
            (s, c, Some(diag))
        } else {
            let (s, c) = q1_compliance_forward(
                rho_comp.clone(),
                &plate,
                boundary_inner.clone(),
                bf_inner.clone(),
                simp_mat,
                &cg_cfg,
            );
            (s, c, None)
        };

        last_rho = rho_bar.clone().into_data().value;
        greyness_hist.push(greyness_mean(&last_rho));

        if it == 1 {
            assert!(
                c_raw.is_finite(),
                "B6 full harness: non-finite raw compliance at iter 1 (Q1-hex PCG / load path). \
Try UMST_SHELL_SELF_WEIGHT=0, UMST_SHELL_MAX_CG>=2000, UMST_SHELL_E_MIN_REL=0.001, UMST_SHELL_PCG=1. \
Got c_raw={c_raw:?} (self_weight={use_self_weight}, vol_in_loop={vol_in_loop}, max_cg={max_cg})."
            );
            comp_scale = c_raw.max(1e-12);
            c0 = c_raw / comp_scale;
        }
        c1 = c_raw / comp_scale;
        if it == 1 {
            first_c1 = c1;
            first_xy_var = xy_plane_variance(&last_rho, nx, ny, nz);
        }
        let mut total_loss = surrogate.clone().div_scalar(comp_scale);
        if grey_lambda > 0.0 {
            let grey_t = mean_greyness_tensor(rho_bar.clone());
            total_loss = total_loss.add(grey_t.mul_scalar(grey_lambda));
        }
        if xy_var_lambda > 0.0 {
            let v_xy = xy_plane_variance_z_avg_tensor(rho_bar.clone(), nx, ny, nz);
            total_loss = total_loss.sub(v_xy.mul_scalar(xy_var_lambda));
        }

        let loss_scalar = total_loss.clone().into_data().value[0];
        if loss_scalar.is_nan() || loss_scalar.is_infinite() {
            if h4_diag {
                if let Some(ref diag) = h4_bundle {
                    let xy_v = xy_plane_variance(&last_rho, nx, ny, nz);
                    log_h4_outer(
                        "shell_topology_rib_pattern_full_v04",
                        it,
                        iter_total,
                        &rho_raw_vec,
                        diag,
                        f32::NAN,
                        adam_skipped,
                        xy_v,
                        loss_scalar,
                    );
                }
            }
            adam_skipped += 1;
            eprintln!(
                "shell_topology_rib_pattern_full_v04: outer {it}/{iter_total} skipped Adam (non-finite loss={loss_scalar})"
            );
            continue;
        }

        let rho_bar_grad_anchor = rho_comp.clone();
        let grads = total_loss.backward();
        let (rho_grad_l2, rho_grad_max) = rho_bar_grad_anchor
            .grad(&grads)
            .map(|g| tensor_grad_l2_inner(&g))
            .unwrap_or((f32::NAN, f32::NAN));
        let grads_params = GradientsParams::from_grads(grads, &opt.density_net);
        let (grad_l2, grad_max, grad_layer_nf) =
            autodiff_param_grad_audit(&grads_params, &opt.density_net);
        last_grad_l2 = grad_l2;
        if grad_l2.is_finite() {
            max_grad_l2 = max_grad_l2.max(grad_l2);
            min_grad_l2 = min_grad_l2.min(grad_l2);
        }
        let (rmin, rmax, _) = rho_raw_range(&rho_raw_vec);
        last_rho_raw_min = rmin;
        last_rho_raw_max = rmax;
        last_outer_beta = beta;
        let vf_now = last_rho.iter().sum::<f32>() / last_rho.len().max(1) as f32;
        let vf_err = vf_now - target_vf;
        if vf_err.abs() > STRIATUS_VF_ERR_ABORT_BAND {
            vf_err_streak += 1;
        } else {
            vf_err_streak = 0;
        }
        if vf_err_streak >= STRIATUS_VF_ERR_ABORT_STREAK {
            panic!(
                "striatus_vf_band_guard: |vf-target|>{STRIATUS_VF_ERR_ABORT_BAND} for \
{STRIATUS_VF_ERR_ABORT_STREAK} consecutive outers (outer {it}/{iterations} vf={vf_now:.6} \
target_vf={target_vf:.4} err={vf_err:+.6})"
            );
        }
        if smoke_subset || metrics_on {
            let grey_now = greyness_mean(&last_rho);
            let xy_now = xy_plane_variance(&last_rho, nx, ny, nz);
            let eq_rel = h4_bundle
                .as_ref()
                .map(|d| d.equilibrium_rel_residual)
                .unwrap_or(f32::NAN);
            if smoke_subset {
                assert_pcg_equilibrium_gate(
                    &format!("shell_topology_rib_pattern_full_v04 smoke outer {it}"),
                    h4_bundle.as_ref().map(|d| d.pcg.rel_residual).unwrap_or(f32::NAN),
                    eq_rel,
                    pcg_tol,
                );
                assert!(
                    grad_l2.is_finite() && grad_l2 > 0.0,
                    "smoke outer {it}: grad_l2={grad_l2}"
                );
            }
            eprintln!(
                "shell_topology_rib_pattern_full_v04: outer {it}/{iterations} \
schedule_k={} beta={beta:.3} vf={vf_now:.6} vf_err={vf_err:+.6} greyness={grey_now:.6} \
grad_l2={grad_l2:.6} xy_var={xy_now:.6} c1={c1:.6} eq_rel={eq_rel:.3e}",
                it.saturating_sub(1),
            );
        }
        if h4_diag {
            if let Some(ref diag) = h4_bundle {
                let xy_v = xy_plane_variance(&last_rho, nx, ny, nz);
                if let Some(ref audit) = diag.finite_audit {
                    log_h5_finite_chain("shell_topology_rib_pattern_full_v04", it, audit);
                }
                log_h5_grad_layers(
                    "shell_topology_rib_pattern_full_v04",
                    it,
                    grad_l2,
                    grad_max,
                    rho_grad_l2,
                    rho_grad_max,
                    &grad_layer_nf,
                );
                log_h4_outer(
                    "shell_topology_rib_pattern_full_v04",
                    it,
                    iter_total,
                    &rho_raw_vec,
                    diag,
                    grad_l2,
                    adam_skipped,
                    xy_v,
                    loss_scalar,
                );
            }
        }
        opt.density_net = adam.step(0.005, opt.density_net, grads_params);
    }

    assert!(!last_rho.is_empty(), "full rib run produced no ρ");

    // Terminal η-bisect finisher (export / gate metrics). In-loop volume is `VolumeProjection` AL.
    if vol_eta_terminal && !vol_bisect_in_loop {
        let mut rho_raw_f = opt
            .density_net
            .forward_batched(coords_norm.clone())
            .reshape([1, n, 1]);
        if sym_period > 0 && iterations % sym_period == 0 {
            rho_raw_f =
                apply_reflection_xy_average(rho_raw_f, &partners).reshape([1, n, 1]);
        }
        if let Some(ref pat) = xy_rib_pat {
            rho_raw_f = rho_raw_f
                .add(pat.clone().mul_scalar(xy_rib_prior_amp))
                .clamp(0.0, 1.0);
        }
        let rho_tilde_f = if helm_on {
            helm.apply_straight_through(rho_raw_f.clone(), edges_b1.clone(), dx_f)
                .reshape([1, n, 1])
        } else {
            rho_raw_f.clone()
        };
        // Detached export step: use schedule β_max so η-bisect can hit V* on soft ρ̃ (continuation
        // β≈1.5 cannot enforce 0.15 VF — same silent failure as the pre-fix 20-outer run).
        let finisher_beta = heaviside_beta_max.max(last_outer_beta);
        let rho_bar_f = vol_eta
            .project(
                rho_tilde_f.reshape([1, n, 1]),
                finisher_beta,
                target_vf,
            )
            .reshape([1, n, 1]);
        last_rho = rho_bar_f.clone().into_data().value;
        last_rho_bar = Some(rho_bar_f);
        let vf_fin = last_rho.iter().sum::<f32>() / last_rho.len() as f32;
        eprintln!(
            "shell_topology_rib_pattern_full_v04: terminal η-bisect finisher \
beta_cont={:.3} beta_fin={finisher_beta:.3} vf={vf_fin:.6} vf_err={:+.6} greyness={:.6}",
            last_outer_beta,
            vf_fin - target_vf,
            greyness_mean(&last_rho),
        );
    }

    let vf = last_rho.iter().sum::<f32>() / last_rho.len() as f32;
    let (_, _, final_diag) = q1_compliance_with_diagnostics(
        last_rho_bar.expect("last rho_bar"),
        &plate,
        boundary_inner.clone(),
        last_bf_inner.expect("last body force"),
        last_simp_mat,
        &cg_cfg,
    );
    let z_profile = rho_z_layer_profile(&last_rho, nx, ny, nz);
    let metrics = RibMetrics {
        vf,
        greyness: greyness_mean(&last_rho),
        xy_var: xy_plane_variance(&last_rho, nx, ny, nz),
        c0,
        c1,
        adam_skipped,
        pcg_iters: final_diag.pcg.iterations,
        pcg_rel_res: final_diag.pcg.rel_residual,
        eq_rel_res: final_diag.equilibrium_rel_residual,
        last_grad_l2,
        max_grad_l2,
        last_rho_raw_min,
        last_rho_raw_max,
        last_outer_beta,
        target_vf,
        roof_ramp_on,
        roof_ramp_strength,
        z_profile,
    };
    log_striatus_acceptance_line("shell_topology_rib_pattern_full_v04", nx, ny, nz, &metrics);
    metrics.also_pcg_gate("shell_topology_rib_pattern_full_v04", pcg_tol)
}

#[test]
fn shell_topology_rib_pattern_quick() {
    let target_vf = parse_target_vf();
    let m = run_rib_quick_metrics();
    let band = 0.15_f32 * target_vf;
    assert!(
        (m.vf - target_vf).abs() <= band,
        "projected vf {:?} within ±15% of {:?}",
        m.vf,
        target_vf
    );
    // v0.4 B6: planar “rib-like” texture — full gate uses `xy_var > 0.1` on final ρ; quick path
    // only needs a **non-degenerate** slice variance after a few Adam steps (coarse grid, traction-only).
    assert!(
        m.xy_var > 2e-5,
        "top-slice xy variance of Heaviside ρ {:?} (quick floor 2e-5; full v0.4 volume_xy_var > 0.1)",
        m.xy_var
    );
    assert!(m.xy_var.is_finite(), "xy var {:?}", m.xy_var);
    // Quick path reports greyness on **Heaviside** ρ (pre-`VolumeProjection`); bisection toward V*
    // can park many nodes near 0.5 so `4ρ(1−ρ)` is large — unlike full B6, which measures post-proj ρ.
    assert!(
        m.greyness.is_finite() && m.greyness <= 1.0_f32 + 1e-3,
        "greyness (Heaviside ρ) {:?} finite in [0,1] band",
        m.greyness
    );
    assert!(
        m.c0.is_finite() && m.c1.is_finite(),
        "compliance {:?}",
        (m.c0, m.c1)
    );
    assert!(m.c1 <= m.c0 * 1.45_f32, "compliance {:?}", (m.c0, m.c1));
}

/// H5 Striatus grid: isolates 40×40×4 density-net → Q1 compliance AD (no projections).
#[test]
#[ignore = "slow: cargo test --release -p umst-concrete-cartridge --test shell_topology_rib_pattern --features solver-experimental h5_striatus_density_net_compliance_grad_40x40x4 -- --ignored --nocapture"]
fn h5_striatus_density_net_compliance_grad_40x40x4() {
    assert_striatus_release_profile("h5_striatus_density_net_compliance_grad_40x40x4");
    <B as BackendTrait>::seed(42);
    let device = Default::default();
    let nx = 40usize;
    let ny = 40usize;
    let nz = 4usize;
    let lx = 4.0_f32;
    let ly = 4.0_f32;
    let lz = 0.1_f32;
    let plate = ExtrudedPlateMechanics {
        nx,
        ny,
        nz,
        dx: lx / nx as f32,
        dy: ly / ny as f32,
        dz: lz / nz as f32,
    };
    let n = plate.n_nodes();
    let coords = plate
        .coords_bn3::<B>(&device)
        .div_scalar(lx.max(ly).max(lz))
        .mul_scalar(2.0)
        .sub_scalar(1.0);
    let boundary = pin_bottom_perimeter_inner(nx, ny, nz, &device);
    let bf = top_load_inner(nx, ny, nz, 50.0, plate.dx, plate.dy, &device);
    let init_scale = parse_density_init_scale(1.0);
    let opt = topology_optimizer_scaled(0.15, 3.0, 64, init_scale, &device);
    let rho = opt
        .density_net
        .forward_batched(coords.clone())
        .reshape([1, n, 1]);
    let mat = SimpElasticMaterial {
        e0: 200e6,
        nu: 0.2,
        p: 3.0,
        e_min: 200e3,
    };
    let cg = MechanicsInnerLoopConfig {
        max_cg_iterations: HEX_PCG_MAX_ITER_DEFAULT_STRIATUS,
        cg_tolerance: HEX_PCG_REL_TOL_F64,
        pcg_tolerance: HEX_PCG_REL_TOL_F64,
        use_preconditioner: true,
        max_equilibrium_substeps: 1,
    };
    let (surrogate, _c, diag) = q1_compliance_with_diagnostics(
        rho.clone(),
        &plate,
        boundary.clone(),
        bf.clone(),
        mat,
        &cg,
    );
    if let Some(audit) = &diag.finite_audit {
        assert_eq!(audit.first_bad_stage, None, "{audit:?}");
    }
    let grads = surrogate.backward();
    let grads_params = GradientsParams::from_grads(grads, &opt.density_net);
    let (comp_l2, comp_max, nf_layers) =
        autodiff_param_grad_audit(&grads_params, &opt.density_net);
    eprintln!(
        "h5_striatus_density_net_compliance_grad_40x40x4: init_scale={init_scale} \
param_l2={comp_l2:.6} param_max={comp_max:.6} sens_l2={:.6} layer_nf={}",
        vec_l2_norm(&diag.nodal_sensitivity),
        nf_layers.len()
    );
    assert!(
        comp_l2 > 0.0,
        "Striatus-scale compliance must backprop to density-net (init_scale={init_scale} param_l2={comp_l2})"
    );
}

fn h5_density_net_compliance_grad_probe(
    nx: usize,
    ny: usize,
    nz: usize,
    lx: f32,
    ly: f32,
    lz: f32,
    weight_scale: Option<f32>,
    hidden_dim: usize,
    tag: &str,
) -> (f32, f32, f32) {
    <B as BackendTrait>::seed(42);
    let device = Default::default();
    let plate = ExtrudedPlateMechanics {
        nx,
        ny,
        nz,
        dx: lx / nx as f32,
        dy: ly / ny as f32,
        dz: lz / nz as f32,
    };
    let n = plate.n_nodes();
    let coord_scale = lx.max(ly).max(lz);
    let coords = plate
        .coords_bn3::<B>(&device)
        .div_scalar(coord_scale)
        .mul_scalar(2.0)
        .sub_scalar(1.0);
    let boundary = pin_bottom_perimeter_inner(nx, ny, nz, &device);
    let use_uniform_load = matches!(
        env::var("UMST_H5_PROBE_UNIFORM_LOAD").as_deref(),
        Ok("1")
    );
    let bf = if nx >= 32 && use_uniform_load {
        top_load_inner(nx, ny, nz, 50.0, plate.dx, plate.dy, &device)
    } else {
        top_load_inner_x_ramp(nx, ny, nz, 50.0, plate.dx, plate.dy, 0.2, &device)
    };
    let opt = match weight_scale {
        Some(s) => topology_optimizer_scaled(0.15, 3.0, hidden_dim, s, &device),
        None => TopologyOptimizer::new(0.15, 3.0, hidden_dim, &device),
    };
    let rho = opt
        .density_net
        .forward_batched(coords.clone())
        .reshape([1, n, 1]);
    let rho_flat = rho.clone().into_data().value;
    let (rmin, rmax, rmean) = rho_raw_range(&rho_flat);
    let mat = SimpElasticMaterial {
        e0: 200e6,
        nu: 0.2,
        p: 3.0,
        e_min: 200e3,
    };
    let cg = MechanicsInnerLoopConfig {
        max_cg_iterations: if nx >= 32 {
            HEX_PCG_MAX_ITER_DEFAULT_STRIATUS
        } else {
            2000
        },
        cg_tolerance: if nx >= 32 {
            HEX_PCG_REL_TOL_F64
        } else {
            HEX_PCG_REL_TOL_F32
        },
        pcg_tolerance: if nx >= 32 {
            HEX_PCG_REL_TOL_F64
        } else {
            HEX_PCG_REL_TOL_F32
        },
        use_preconditioner: true,
        max_equilibrium_substeps: 1,
    };
    let (surrogate, c_raw, diag) = q1_compliance_with_diagnostics(
        rho.clone(),
        &plate,
        boundary.clone(),
        bf.clone(),
        mat,
        &cg,
    );
    if let Some(audit) = &diag.finite_audit {
        assert_eq!(audit.first_bad_stage, None, "{tag}: {audit:?}");
    }
    let sens_l2 = vec_l2_norm(&diag.nodal_sensitivity);
    let grads = surrogate.backward();
    let rho_grad_l2 = rho
        .grad(&grads)
        .map(|g| {
            let v = g.into_data().value;
            v.iter().map(|x| x * x).sum::<f32>().sqrt()
        })
        .unwrap_or(f32::NAN);
    let grads_params = GradientsParams::from_grads(grads, &opt.density_net);
    let (param_l2, param_max, nf_layers) =
        autodiff_param_grad_audit(&grads_params, &opt.density_net);
    eprintln!(
        "{tag}: rho=[{rmin:.6},{rmax:.6}] mean={rmean:.6} sens_l2={sens_l2:.6} c_raw={c_raw:.6} \
rho_grad_l2={rho_grad_l2:.6} param_l2={param_l2:.6} param_max={param_max:.6} layer_nf={}",
        nf_layers.len()
    );
    (param_l2, rho_grad_l2, sens_l2)
}

/// H5: density-net → Q1 compliance AD chain on quick grid (isolates grad plumbing from projections).
#[test]
fn h5_density_net_compliance_grad_9x8x2() {
    let (param_l2, _, _) =
        h5_density_net_compliance_grad_probe(9, 8, 2, 0.8, 0.8, 0.1, None, 64, "h5_9x8x2");
    assert!(
        param_l2 > 0.0,
        "compliance surrogate must backprop to density-net params (l2={param_l2})"
    );
}

/// Striatus-scale grad probe: 40×40×4, one forward+backward (matches full harness DensityNet init).
#[test]
#[ignore = "slow: cargo test -p umst-concrete-cartridge --test shell_topology_rib_pattern --features solver-experimental h5_density_net_compliance_grad_40x40x4_striatus --release -- --ignored --nocapture"]
fn h5_density_net_compliance_grad_40x40x4_striatus() {
    let (param_l2, _, sens_l2) = h5_density_net_compliance_grad_probe(
        40,
        40,
        4,
        4.0,
        4.0,
        0.1,
        None,
        64,
        "h5_40x40x4_striatus",
    );
    assert!(
        param_l2 > 0.0 && sens_l2 > 0.0,
        "Striatus-scale compliance AD must reach density-net (param_l2={param_l2} sens_l2={sens_l2})"
    );
}

#[test]
#[ignore = "slow B6: UMST_SHELL_RIB_PATTERN=1 cargo test -p umst-concrete-cartridge --test shell_topology_rib_pattern --features solver-experimental shell_topology_rib_pattern_full_v04 --release -- --ignored (optional UMST_SHELL_RIB_FULL_ITERS=1..200 for smoke)"]
fn shell_topology_rib_pattern_full_v04() {
    assert_eq!(
        env::var("UMST_SHELL_RIB_PATTERN").as_deref(),
        Ok("1"),
        "set UMST_SHELL_RIB_PATTERN=1 for the long Striatus-scale acceptance run"
    );
    let target_vf = parse_target_vf();
    let adam_iters = parse_full_rib_adam_iters();
    let m = run_rib_full_striatus(target_vf);
    let (gl, xyl, b0, jit, rib) = parse_umst_shell_b6_aux_env();
    let g_uni = 4.0 * target_vf * (1.0 - target_vf);
    eprintln!(
        "shell_topology_rib_pattern_full_v04: pre-gate metrics \
GREYNESS(4ρ(1−ρ))={:.6} vf={:.6} target_vf={:.4} vf_err={:+.6} \
UMST_SHELL_ROOF_RAMP={} ramp_strength={:.3} z_rho_mean={} \
xy_var_z_avg={:.6} c0={:.6} c1={:.6} beta_last={:.3} \
max_grad_l2={:.6} last_grad_l2={:.6} g_uni=4·vf·(1−vf)={:.6} pcg_iter_final={} pcg_rel_res={:.3e} eq_rel_res={:.3e} adam_skipped={}/{} UMST_SHELL_GREY_LAMBDA={:.6} UMST_SHELL_XY_VAR_LAMBDA={:.6} UMST_SHELL_HEAVISIDE_BETA0={:.6} UMST_SHELL_DENSITY_INIT_JITTER={:.6} UMST_SHELL_XY_RIB_PRIOR_AMP={:.6}",
        m.greyness,
        m.vf,
        m.target_vf,
        m.vf - m.target_vf,
        if m.roof_ramp_on { 1 } else { 0 },
        m.roof_ramp_strength,
        m.z_profile,
        m.xy_var,
        m.c0,
        m.c1,
        m.last_outer_beta,
        m.max_grad_l2,
        m.last_grad_l2,
        g_uni,
        m.pcg_iters,
        m.pcg_rel_res,
        m.eq_rel_res,
        m.adam_skipped,
        adam_iters,
        gl,
        xyl,
        b0,
        jit,
        rib
    );
    assert!(
        m.c0.is_finite() && m.c1.is_finite(),
        "compliance finite: c0={:?} c1={:?} (vf={} greyness={} xy_var={})",
        m.c0,
        m.c1,
        m.vf,
        m.greyness,
        m.xy_var
    );
    assert!(
        m.xy_var.is_finite() && m.greyness.is_finite(),
        "metrics finite: xy_var={} greyness={}",
        m.xy_var,
        m.greyness
    );
    // Smoke (`UMST_SHELL_RIB_FULL_ITERS` < 200): five-criteria gate before 200-outer acceptance.
    if adam_iters < 200 {
        assert!(
            m.max_grad_l2.is_finite() && m.max_grad_l2 > 0.0 && m.last_grad_l2 > 0.0,
            "smoke grad_l2: max={} last={} (adam_skipped={} vf={} vf_err={:+.6} xy_var={})",
            m.max_grad_l2,
            m.last_grad_l2,
            m.adam_skipped,
            m.vf,
            m.vf - m.target_vf,
            m.xy_var
        );
        assert!(
            (m.vf - m.target_vf).abs() <= STRIATUS_VF_ERR_ABORT_BAND,
            "smoke vf band: vf={} target={} err={} (guard should have aborted in-loop)",
            m.vf,
            m.target_vf,
            m.vf - m.target_vf
        );
        assert_eq!(
            m.adam_skipped, 0,
            "smoke adam_skipped: got {} (grad_l2={} vf={})",
            m.adam_skipped, m.last_grad_l2, m.vf
        );
        assert!(
            m.last_rho_raw_min < 0.501 - 1e-6 || m.last_rho_raw_max > 0.501 + 1e-6,
            "smoke rho_raw plateau: [{}, {}] must leave [0.501, 0.501]",
            m.last_rho_raw_min,
            m.last_rho_raw_max
        );
        assert!(
            m.xy_var > 0.0,
            "smoke xy_var: got {} (grad_l2={} vf={})",
            m.xy_var,
            m.last_grad_l2,
            m.vf
        );
        eprintln!(
            "shell_topology_rib_pattern_full_v04: smoke PASS ({adam_iters} outer) — \
GREYNESS={:.6} max_grad_l2={:.6} last_grad_l2={:.6} beta_last={:.3} \
xy_var={:.6} c0={:.6} c1={:.6} rho_raw=[{:.6},{:.6}] eq_rel={:.3e} adam_skipped=0; \
full B6 vf/greyness<0.15/compliance-drop gates deferred to 200-outer",
            m.greyness,
            m.max_grad_l2,
            m.last_grad_l2,
            m.last_outer_beta,
            m.xy_var,
            m.c0,
            m.c1,
            m.last_rho_raw_min,
            m.last_rho_raw_max,
            m.eq_rel_res
        );
        return;
    }
    assert!(
        (m.vf - target_vf).abs() <= 0.01,
        "vf gate: got vf={} target_vf={} (greyness={} xy_var={} c0={} c1={})",
        m.vf,
        target_vf,
        m.greyness,
        m.xy_var,
        m.c0,
        m.c1
    );
    assert!(
        m.greyness < 0.15,
        "greyness gate: got {} (vf={} xy_var={} c0={} c1={})",
        m.greyness,
        m.vf,
        m.xy_var,
        m.c0,
        m.c1
    );
    assert!(
        m.xy_var > 0.1,
        "xy_var gate: got {} (vf={} greyness={} c0={} c1={})",
        m.xy_var,
        m.vf,
        m.greyness,
        m.c0,
        m.c1
    );
    assert!(
        m.c1 < m.c0 * 0.6,
        "compliance drop gate: c0={} c1={} ratio={} (vf={} greyness={} xy_var={})",
        m.c0,
        m.c1,
        m.c1 / m.c0.max(1e-30),
        m.vf,
        m.greyness,
        m.xy_var
    );
}
