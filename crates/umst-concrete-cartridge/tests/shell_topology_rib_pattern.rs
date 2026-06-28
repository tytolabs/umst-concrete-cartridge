// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#![cfg(feature = "solver-experimental")]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::too_many_arguments)]

//! **Track B6 (v0.4)** — `shell_topology_rib_pattern`: Striatus-class gates ([`composer_prompts/v0.4_solver_completion_no_namesakes.md`](../../../../composer_prompts/v0.4_solver_completion_no_namesakes.md) §B6).
//!
//! - [`shell_topology_rib_pattern_quick`]: CI — compact **0.8×0.8×0.1** m slab, coords **\([-1,1]^3\)** (same as [`optimize_shell_3d`](../examples/optimize_shell_3d.rs)), default **9×8×2** cells when all `UMST_SHELL_{NX,NY,NZ,ITERS}` are unset, gentle roof **x-ramp** \(r=0.2\) at **50 Pa**, Heaviside \(\beta=10\). **Helmholtz is omitted on the Burn AD tape**; [`VolumeProjection`] after Adam. Default **24** steps. Gates: VF ±15%, top-face variance of Heaviside \(\hat\rho\) \(> 2\times 10^{-5}\) (full B6: \(>0.1\) on final \(\rho\)); greyness not asserted on quick path; compliance ratio bounded.
//! - [`shell_topology_rib_pattern_full_v04`]: `#[ignore]` — **40×40×4**, **200** iters, **seed 42**.
//!   **Absorbing-step invariant:** no **periodic mutation** of ρ (or schedule-driving state) without a **subsequent
//!   Adam step** to absorb it — any new mechanism wired into the outer loop inherits this rule. Instances today:
//!   (i) XY reflection (`sym_period`) never on outer `N`; (ii) [`outer_schedule_k`] **frozen** on the final outer;
//!   (iii) acceptance gates read **post-finisher export ρ** (terminal b-bisect @ β_max) with one fresh equilibrium
//!   solve — not in-loop `ρ_mid` of outer `N`. Mid-run **greyness jump** tripwire: adjacent-outer ratio ≥ **10×**
//!   logs `WARN greyness_jump` (known corruption signature when sym fires without an absorbing step). **Open roadmap item:** full Striatus-scale B6 stays off default CI (same opt-in pattern as manifold long/`#[ignore]` gates). **Run:** set **`UMST_SHELL_RIB_PATTERN=1`**, then `cargo test -p umst-concrete-cartridge --test shell_topology_rib_pattern --features solver-experimental shell_topology_rib_pattern_full_v04 --release -- --ignored` (**`--release` before `--`**; flags after `--` go to the test harness, not rustc). Append **`--nocapture`** for one **`pre-gate metrics`** line (**`vf`**, **`greyness`**, **`g_uni`**, **`xy_var_z_avg`**, **`c0`**, **`c1`**, **`adam_skipped`**, **`UMST_SHELL_GREY_LAMBDA`**, **`UMST_SHELL_XY_VAR_LAMBDA`**, **`UMST_SHELL_HEAVISIDE_BETA0`**). **Subset / smoke:** **`UMST_SHELL_RIB_FULL_ITERS`** (default **200**, clamped **1…200**) shortens the Adam outer loop; **one** outer still runs the full **40×40×4** forward + backward and can take **many CPU minutes** in `--release**, and the **optimisation** does not satisfy the brief greyness / compliance gates unless you run the full **200** outers — the Rust test **skips** those acceptance asserts when **`UMST_SHELL_RIB_FULL_ITERS` < 200** (finite compliance + loose VF band only). **Helmholtz:** same as [`optimize_shell_3d`](../examples/optimize_shell_3d.rs) — **only** literal **`UMST_SHELL_HELM=1`** enables the graph filter on the Burn tape (an empty `UMST_SHELL_HELM=` must **not** enable — older `!= \"0\"` parsing turned it on and tripped scatter backward at Striatus N); default **off**. **Full-harness parity with `optimize_shell_3d`:** **`UMST_SHELL_SELF_WEIGHT`** (default **on** — Bruyneel–Duysinx self-weight on **`ρ_bar`** plus roof traction; set **`0`** to disable), **`UMST_SHELL_VOL_LOOP`** (default **on**; **`0`** skips in-loop volume projection), **`UMST_SHELL_MAX_CG`**, **`UMST_SHELL_PCG`**, **`UMST_SHELL_E_MIN_REL`** — same semantics as the example. **Multi-term outer loss (experimental):** **`UMST_SHELL_GREY_LAMBDA`** adds **`λ_g·mean(4ρ(1−ρ))`** on **post–volume-projection** **`ρ_bar`** (same grey statistic as the gate); **`UMST_SHELL_XY_VAR_LAMBDA`** adds **`-λ_{xy}·Var_{xy}(\bar\rho)`** where **`Var_{xy}`** is the **z-averaged** column variance (matches the **`xy_plane_variance`** gate on **`ρ`**). **`UMST_SHELL_HEAVISIDE_BETA0`** / **`UMST_SHELL_HEAVISIDE_BETA_MAX`** override Heaviside log-continuation endpoints (defaults **1** and **32**). Non-finite **iter 1** raw compliance **panics** immediately (PCG / conditioning root). Quick-path sizing env **`UMST_SHELL_*`** applies only to [`shell_topology_rib_pattern_quick`], not the full grid defaults (**40³** slab is fixed in the full harness).
//!
//! **Q1-hex PCG (B6 H4, 2026-06-10):** forward+adjoint use [`AdjointComplianceQ1Hex`] (continuum SIMP on the extruded grid). Bar-network ground structure was retired after mechanism probes on **9×8×2**; see [`Solver-Status.md`](../../docs/Solver-Status.md).
//!
//! **Performance discipline:** mechanism probes and operator sanity checks stay on **9×8×2** only. **40×40×4** runs require a converging operator and **`--release` before `--`**. For faster debug harness iteration, workspace `Cargo.toml` sets `[profile.dev.package."*"] opt-level = 3` so tensor deps stay optimized while test code remains unoptimized.

//! **`UMST_RIB_QUICK`:** unset or `1` — implied “small / few iterations” CI mode (grid defaults below). Set `UMST_RIB_QUICK=0` only if you intentionally enlarge the quick harness via `UMST_SHELL_*`.
//!
//! **Sizing env:** `UMST_SHELL_NX`, `UMST_SHELL_NY`, `UMST_SHELL_NZ`, `UMST_SHELL_ITERS`, `UMST_SHELL_VF`.

use std::cell::Cell;
use std::env;
use std::time::Instant;

use burn::backend::Autodiff;
use burn::module::{AutodiffModule, Module, ModuleMapper, ModuleVisitor, ParamId};
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::{
    activation::sigmoid,
    backend::{AutodiffBackend, Backend as BackendTrait},
    Data, Int, Shape, Tensor,
};
use burn_ndarray::NdArray;
use umst_manifold::ai::topology::{
    logit_offset_vf_from_slice, BetaContinuation, ContinuationSchedule, HeavisideProjection,
    PlateauBetaContinuation, TopologyOptimizer, VolumeEtaProjection, VolumeLogitOffsetProjection,
    VolumeProjection,
};
use umst_manifold::physics::adjoint::{
    AdjointComplianceDiagnostics, AdjointFiniteStageAudit, HexPreconditionerKind,
    SimpElasticMaterial,
};
use umst_manifold::physics::adjoint_q1_hex::{
    AdjointComplianceQ1Hex, Q1HexSolveOptions, Q1HexTopVoidColumnFractions,
};
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

/// Optimizer sensitivity uses schedule `p_act`; §9 gate uses fixed `p_gate` (R1a).
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
enum CompliancePenalization {
    Schedule { outer: usize, total: usize },
    Gate(f32),
    Fixed(f32),
}

impl CompliancePenalization {
    fn resolve_p(&self, schedule: impl Fn(usize, usize) -> f32) -> f32 {
        match self {
            Self::Schedule { outer, total } => schedule(*outer, *total),
            Self::Gate(p) | Self::Fixed(p) => *p,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum MetricProvenance {
    FastOuter,
    FullSolve,
    F64Verify,
}

impl MetricProvenance {
    fn as_str(self) -> &'static str {
        match self {
            Self::FastOuter => "fast_outer",
            Self::FullSolve => "full_solve",
            Self::F64Verify => "f64_verify",
        }
    }
}

/// §9 gate exponent — default **3.0** (`UMST_SHELL_GATE_P`).
fn parse_shell_gate_p() -> f32 {
    env::var("UMST_SHELL_GATE_P")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|p| p.is_finite() && *p > 0.0)
        .unwrap_or(3.0)
}

fn continuation_schedule_p(outer: usize, total: usize) -> f32 {
    if let Ok(s) = env::var("UMST_SHELL_FIXED_P_ACT") {
        if let Ok(p) = s.parse::<f32>() {
            if p.is_finite() && p > 0.0 {
                return p;
            }
        }
    }
    ContinuationSchedule::value(outer, total)
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

fn thesis_reconfig_enabled() -> bool {
    matches!(
        env::var("UMST_SHELL_THESIS_RECONFIG").as_deref(),
        Ok("1") | Ok("true")
    )
}

/// Constants from `scripts/b6_harness_setup.sh` (source before harness runs).
fn b6_env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn b6_env_f32(name: &str, default: f32) -> f32 {
    env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn parse_target_vf() -> f32 {
    let default = if thesis_reconfig_enabled() {
        b6_env_f32("B6_VF_TARGET", 0.30_f32)
    } else {
        0.15_f32
    };
    env::var("UMST_SHELL_VF")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// Striatus full harness slab: (nx, ny, nz, lx, ly, lz).
fn striatus_slab_geometry() -> (usize, usize, usize, f32, f32, f32) {
    if thesis_reconfig_enabled() {
        let nx = b6_env_usize("B6_NX", 40);
        let ny = b6_env_usize("B6_NY", 40);
        let nz = b6_env_usize("B6_NZ", 8);
        let lz = b6_env_f32("B6_SLAB_THICKNESS_M", 0.3_f32);
        (nx, ny, nz, 4.0_f32, 4.0_f32, lz)
    } else {
        (40, 40, 4, 4.0_f32, 4.0_f32, 0.1_f32)
    }
}

/// Uniform interior ρ scaled to `target_vf` with non-design top skin ρ=1 (c0 reference field).
fn thesis_uniform_rho_at_vf(nx: usize, ny: usize, nz: usize, target_vf: f32) -> Vec<f32> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let nz1 = nz + 1;
    let n = nx1 * ny1 * nz1;
    let iz_top = nz;
    let n_skin = nx1 * ny1;
    let n_interior = n.saturating_sub(n_skin);
    let rho_int = if n_interior > 0 {
        ((target_vf * n as f32) - n_skin as f32) / n_interior as f32
    } else {
        target_vf
    }
    .clamp(0.0, 1.0);
    let mut rho = vec![rho_int; n];
    for iz in 0..nz1 {
        if iz == iz_top {
            for iy in 0..ny1 {
                for ix in 0..nx1 {
                    let nid = ix + iy * nx1 + iz * nx1 * ny1;
                    rho[nid] = 1.0;
                }
            }
        }
    }
    rho
}

/// Roof traction on the **fixed solid top skin** only (thesis load model — no void-column roof lumping).
fn thesis_roof_live_force_vec(
    nx: usize,
    ny: usize,
    nz: usize,
    dx: f32,
    dy: f32,
    roof_ramp_strength: f32,
    n_dof: usize,
) -> Vec<f32> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let iz_top = nz;
    let nx_d = nx.max(1) as f32;
    let mut live_f = vec![0.0_f32; n_dof];
    for iy in 0..=ny {
        for ix in 0..=nx {
            let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
            let w = 1.0_f32 + roof_ramp_strength * (ix as f32 / nx_d);
            live_f[nid * 3 + 2] = -50.0 * dx * dy * w;
        }
    }
    live_f
}

/// Fix top z-layer ρ=1 (non-design solid skin); ribs optimize below.
fn apply_non_design_skin(rho: &mut [f32], nx: usize, ny: usize, nz: usize) {
    if !thesis_reconfig_enabled() {
        return;
    }
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let iz_top = nz;
    for ix in 0..=nx {
        for iy in 0..=ny {
            let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
            rho[nid] = 1.0;
        }
    }
}

/// B6 thesis `policy_editable_mask`: 1.0 = design DOF (ribs), 0.0 = fixed solid top skin.
fn policy_editable_mask_vec(nx: usize, ny: usize, nz: usize) -> Vec<f32> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let n = nx1 * ny1 * (nz + 1);
    let mut mask = vec![1.0_f32; n];
    if !thesis_reconfig_enabled() {
        return mask;
    }
    let iz_top = nz;
    for ix in 0..=nx {
        for iy in 0..=ny {
            let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
            mask[nid] = 0.0;
        }
    }
    mask
}

/// ρ ← mask·ρ + (1−mask)·1 on fixed skin nodes (straight-through on interior).
fn apply_policy_editable_mask<Bk: BackendTrait<FloatElem = f32>>(
    rho: Tensor<Bk, 3>,
    mask: &Tensor<Bk, 3>,
) -> Tensor<Bk, 3> {
    if !thesis_reconfig_enabled() {
        return rho;
    }
    let fixed = Tensor::<Bk, 3>::ones_like(&rho);
    rho.mul(mask.clone()).add(fixed.sub(mask.clone()))
}

/// Top-layer void-column threshold for **H-c1-A** (matches `b6_c1_diagnosis`).
const THESIS_VOID_RHO_THRESHOLD: f32 = 0.1;

/// **H-c1-A** spatial audit: fraction of compliance work on top-void columns.
#[allow(clippy::too_many_arguments)]
fn h_c1_a_top_void_fractions(
    rho: &[f32],
    plate: &ExtrudedPlateMechanics,
    live_f: &[f32],
    m_flat: &[f32],
    mat: SimpElasticMaterial,
    cg: &MechanicsInnerLoopConfig,
    sw: Option<SelfWeightConfig>,
) -> Q1HexTopVoidColumnFractions {
    let (audit, u) = AdjointComplianceQ1Hex::evaluate_compliance(
        rho, plate.nx, plate.ny, plate.nz, plate.dx, plate.dy, plate.dz, live_f, m_flat, mat, cg,
        sw,
    );
    AdjointComplianceQ1Hex::top_void_column_fractions(
        &audit,
        &u,
        rho,
        plate.nx,
        plate.ny,
        plate.nz,
        live_f,
        m_flat,
        THESIS_VOID_RHO_THRESHOLD,
    )
}

fn log_h_c1_a_verdict(tag: &str, frac: &Q1HexTopVoidColumnFractions) {
    let comp = frac.compliance_fraction * 100.0;
    let se = frac.strain_energy_fraction * 100.0;
    eprintln!(
        "{tag}: H-c1-A audit top_void_column compliance={comp:.1}% strain_energy={se:.1}% \
void_xy={:.1}% (gate <50%)",
        frac.void_column_fraction_xy * 100.0
    );
    if comp >= 50.0 || se >= 50.0 {
        panic!(
            "{tag}: H-c1-A STOP — top-void-column share >=50% (compliance={comp:.1}%, SE={se:.1}%); \
load model still wrong — no 200-outer"
        );
    }
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

#[allow(dead_code)]
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

fn format_rho_bar_grad_component(v: f32) -> String {
    if v.is_nan() {
        "n/a (non-leaf)".to_string()
    } else {
        format!("{v:.6}")
    }
}

fn log_h5_grad_layers(
    tag: &str,
    outer: usize,
    grad_l2: f32,
    grad_max: f32,
    rho_bar_grad: Option<(f32, f32)>,
    layers: &[(usize, usize, usize)],
) {
    let (rho_l2_s, rho_max_s) = match rho_bar_grad {
        Some((l2, mx)) => (
            format_rho_bar_grad_component(l2),
            format_rho_bar_grad_component(mx),
        ),
        None => ("n/a (non-leaf)".to_string(), "n/a (non-leaf)".to_string()),
    };
    eprintln!(
        "{tag}: H5 grad outer {outer}: param_l2={grad_l2:.6} param_max={grad_max:.6} \
rho_bar_l2={rho_l2_s} rho_bar_max={rho_max_s} layer_nf={}",
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
    /// In-loop / schedule-p compliance on acceptance ρ (`p_act`).
    c1_running: f32,
    /// Fixed gate-p compliance on acceptance ρ (`p_gate`; §9 row).
    c1_fixed_p3: f32,
    /// Schedule final `p` on acceptance ρ (audit; not §9 gate).
    p_act: f32,
    /// §9 gate exponent (default 3.0).
    p_gate: f32,
    /// Legacy alias: same as `c1_fixed_p3` for greyness/VF context lines.
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
    /// In-loop VF on `ρ_mid` (last outer, before terminal b-bisect export).
    vf_loop: f32,
    /// Post–b-bisect export VF when terminal finisher runs (`vf_loop` otherwise).
    vf_export: f32,
    /// Greyness on `ρ_mid` at outer 1 (A′: must fall vs final).
    greyness_outer1: f32,
    /// Peak normalized compliance across outers (A′: final must sit below peak).
    c1_peak: f32,
    /// Peak fixed-p (=3) compliance across outers (smoke gate semantics; D2 Tier 4b).
    c1_peak_fixed_p3: f32,
    /// Last-outer in-loop fixed-p (=3) compliance (before terminal finisher).
    c1_fixed_p3_loop_last: f32,
    /// Min `xy_var` on `ρ_mid` for outers ≥ 18 (A′: must stay > 0 past old collapse point).
    min_xy_var_from_outer_18: f32,
    /// Min `xy_var` on `ρ_mid` for outers ≥ 50 (60-outer schedule-regime smoke).
    min_xy_var_from_outer_50: f32,
    /// Count of β steps taken with AL×β handshake.
    beta_step_count: usize,
    /// Greyness on `ρ_mid` recorded immediately after each β step.
    greyness_at_beta_steps: Vec<f32>,
    /// Peak `|vf_err|` across outers (logit-offset smoke: must stay within band every outer).
    max_vf_err_abs: f32,
    /// Peak `|vf_err|` when detached b-bisection ran (logit-offset is feasible by construction).
    max_vf_err_when_feasible: f32,
    /// Compliance baseline: uniform ρ = target_vf @ SIMP **p = 1** (Voigt; [`b6-c0-uniform-at-target-vf`]).
    c0_uniform: f32,
    /// Raw `c0_uniform` compliance before normalization (§9 row 7).
    c0_uniform_raw: f32,
    /// Matched-p baseline: uniform ρ @ schedule final `p` (§9 gate pairing — B0a).
    c0_uniform_p_final_raw: f32,
    /// `c0_uniform_p_final_raw / c0_uniform_raw` (normalized matched-p gate reference).
    c0_uniform_p_final: f32,
    /// H-c1-A: top-void-column compliance fraction on acceptance ρ (thesis re-config).
    h_c1_a_comp_frac: f32,
    /// H-c1-A: top-void-column strain-energy fraction on acceptance ρ.
    h_c1_a_se_frac: f32,
    /// Greyness on in-loop `ρ_mid` at the last outer (may differ from acceptance when sym boundary fires).
    greyness_loop_last: f32,
    /// Wall-clock for the last completed Adam outer (ms).
    last_outer_wall_ms: f64,
    /// Cumulative wall-clock from first outer through last (s).
    total_wall_s: f64,
    /// Deterministic seed passed to the Burn backend (`run_rib_full_striatus` fixes **42**).
    seed: u64,
    /// Active Cargo feature set for this binary (e.g. `solver-experimental+blas-accelerate`).
    active_backend_features: String,
}

/// Active Cargo feature set for BLAS / solver lane (logged on pre-gate metrics; no numeric effect).
fn active_backend_feature_set() -> String {
    let feats = Vec::from(["solver-experimental".to_string()]);
    #[cfg(feature = "render")]
    let feats = {
        let mut feats = feats;
        feats.push("render".to_string());
        feats
    };
    #[cfg(feature = "blas-accelerate")]
    let feats = {
        let mut feats = feats;
        feats.push("blas-accelerate".to_string());
        feats
    };
    #[cfg(feature = "mac-fast")]
    let feats = {
        let mut feats = feats;
        feats.push("mac-fast".to_string());
        feats
    };
    feats.join("+")
}

/// Continuation schedule index for outer `it` — instance of the **absorbing-step invariant**:
/// schedule index must not advance on the final outer (no Adam step after to absorb it).
fn outer_schedule_k(it: usize, iterations: usize) -> usize {
    if it >= iterations {
        iterations.saturating_sub(2)
    } else {
        it.saturating_sub(1)
    }
}

/// Warn when greyness jumps sharply between adjacent outers (periodic mutation without absorb).
fn warn_greyness_jump_if_needed(
    tag: &str,
    outer: usize,
    outer_total: usize,
    prev: f32,
    now: f32,
    sym_apply: bool,
) {
    if !prev.is_finite() || !now.is_finite() {
        return;
    }
    let eps = 1e-12_f32;
    let ratio = (now + eps) / (prev + eps);
    let jump = ratio.max(1.0 / ratio);
    if jump >= STRIATUS_GREYNESS_JUMP_WARN_RATIO {
        eprintln!(
            "{tag}: WARN greyness_jump outer {outer}/{outer_total} prev={prev:.6} now={now:.6} \
ratio={jump:.1}x sym_apply={} (corruption signature: periodic ρ mutation without subsequent Adam step?)",
            u8::from(sym_apply),
        );
    }
}

fn log_striatus_acceptance_line(tag: &str, _nx: usize, _ny: usize, _nz: usize, m: &RibMetrics) {
    eprintln!(
        "{tag}: acceptance diag stage=acceptance \
p_act={:.3} p_gate={:.3} c_running={:.6} c_fixed_p3={:.6} provenance={} \
UMST_SHELL_ROOF_RAMP={} ramp_strength={:.3} target_vf={:.4} vf_final={:.6} vf_err={:+.6} \
GREYNESS={:.6} z_rho_mean={} xy_var={:.6} c0={:.6} c1={:.6} beta_last={:.3} \
max_grad_l2={:.6} max_vf_err_abs={:.6} c0_uniform={:.6} c0_uniform_raw={:.6} \
h_c1_a_comp={:.1}% greyness_loop_last={:.6} eq_rel={:.3e}",
        m.p_act,
        m.p_gate,
        m.c1_running,
        m.c1_fixed_p3,
        MetricProvenance::FullSolve.as_str(),
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
        m.max_vf_err_abs,
        m.c0_uniform,
        m.c0_uniform_raw,
        m.h_c1_a_comp_frac * 100.0,
        m.greyness_loop_last,
        m.eq_rel_res,
    );
    if m.greyness < 0.05 && m.last_outer_beta < 8.0 {
        eprintln!(
            "{tag}: greyness sanity — greyness={:.4} at beta={:.3}: Heaviside cannot sharpen this \
far at low beta; field is already near-binary from DensityNet. Confirm vf_final vs target_vf \
(η-bisection); binary-at-wrong-volume is a silent failure mode.",
            m.greyness, m.last_outer_beta
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
    self_weight: Option<SelfWeightConfig>,
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
        self_weight,
    )
}

/// Env-selected solve options for the shell forward solve.
///
/// `UMST_SHELL_PRECOND` ∈ {`bj`, `cache`, unset/`jacobi`} selects the PCG
/// preconditioner lever. **`mg` is rejected at Striatus scale** — thin-slab
/// anisotropic grids (40×40×4) do not converge with the current geometric
/// V-cycle (`rel_residual≈1`); use Jacobi until semicoarsening lands (D3).
fn shell_solve_options() -> Q1HexSolveOptions {
    let mut opts = Q1HexSolveOptions::default();
    match env::var("UMST_SHELL_PRECOND").as_deref() {
        Ok("mg") => {
            eprintln!(
                "WARN shell_solve_options: UMST_SHELL_PRECOND=mg ignored — \
MG V-cycle fails on anisotropic Striatus slab; using Jacobi (D3)"
            );
        }
        Ok("bj") => {
            opts.precond_kind = Some(HexPreconditionerKind::BlockJacobiNodal3x3);
            opts.use_operator_cache = true;
        }
        Ok("cache") => {
            opts.use_operator_cache = true;
        }
        _ => {}
    }
    opts
}

fn d1_three_point_enabled() -> bool {
    matches!(env::var("UMST_SHELL_D1").as_deref(), Ok("1"))
}

/// Hold Heaviside β fixed (D2 trial when continuation is the compliance culprit).
fn parse_fixed_heaviside_beta() -> Option<f32> {
    env::var("UMST_SHELL_FIXED_BETA")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|b: &f32| b.is_finite() && *b > 0.0)
}

/// Fixed-p=3 compliance at a detached density (no AD tape) — D1 three-point diagnostic.
#[allow(clippy::too_many_arguments)]
fn rib_c1_fixed_p3_at_rho_inner(
    rho_inner: Tensor<Inner, 3>,
    plate: &ExtrudedPlateMechanics,
    boundary_inner: &Tensor<Inner, 3>,
    live_force: &Tensor<B, 3>,
    material: &ElasticMaterial,
    cg: &MechanicsInnerLoopConfig,
    use_self_weight: bool,
    sw_cfg: SelfWeightConfig,
    comp_scale: f32,
    device: &<B as BackendTrait>::Device,
) -> f32 {
    let rho_flat = rho_inner.into_data().value;
    let bf = if use_self_weight {
        let rho_b = Tensor::<B, 3>::from_data(
            Data::new(rho_flat.clone(), Shape::new([1, rho_flat.len(), 1])),
            device,
        );
        sw_cfg.body_force(rho_b).add(live_force.clone()).inner()
    } else {
        live_force.clone().inner()
    };
    let simp = SimpElasticMaterial {
        e0: material.e0,
        nu: material.nu,
        p: material.simp_p,
        e_min: material.e_min,
    };
    let c = AdjointComplianceQ1Hex::raw_compliance_at_rho(
        &rho_flat,
        plate.nx,
        plate.ny,
        plate.nz,
        plate.dx,
        plate.dy,
        plate.dz,
        &bf.into_data().value,
        &boundary_inner.clone().into_data().value,
        simp,
        cg,
        if use_self_weight { Some(sw_cfg) } else { None },
    );
    c / comp_scale
}

#[allow(clippy::too_many_arguments)]
fn q1_compliance_with_diagnostics(
    rho_bar: Tensor<B, 3>,
    plate: &ExtrudedPlateMechanics,
    boundary: Tensor<Inner, 3>,
    body_force: Tensor<Inner, 3>,
    mat: SimpElasticMaterial,
    cg: &MechanicsInnerLoopConfig,
    self_weight: Option<SelfWeightConfig>,
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
        self_weight,
        &shell_solve_options(),
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
            None,
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
                    None,
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
        None,
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
        c1_running: c1,
        c1_fixed_p3: c1,
        p_act: f32::NAN,
        p_gate: parse_shell_gate_p(),
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
        target_vf,
        roof_ramp_on: true,
        roof_ramp_strength: 0.2,
        z_profile: String::new(),
        vf_loop: vf,
        vf_export: vf,
        greyness_outer1: greyness,
        c1_peak: c1,
        c1_peak_fixed_p3: c1,
        c1_fixed_p3_loop_last: c1,
        min_xy_var_from_outer_18: xy_var,
        min_xy_var_from_outer_50: f32::NAN,
        beta_step_count: 0,
        greyness_at_beta_steps: Vec::new(),
        max_vf_err_abs: f32::NAN,
        max_vf_err_when_feasible: f32::NAN,
        c0_uniform: f32::NAN,
        c0_uniform_raw: f32::NAN,
        c0_uniform_p_final_raw: f32::NAN,
        c0_uniform_p_final: f32::NAN,
        h_c1_a_comp_frac: f32::NAN,
        h_c1_a_se_frac: f32::NAN,
        greyness_loop_last: greyness,
        last_outer_wall_ms: 0.0,
        total_wall_s: 0.0,
        seed: 42,
        active_backend_features: active_backend_feature_set(),
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

/// Logit-offset tripwire: bisection failure or skipped-b pathology (band = gate / 100).
const STRIATUS_VF_ERR_ABORT_BAND: f32 = 0.02;
/// Terminal b-bisect must predict this VF residual before export (honest failure, not silent 0.15→0.97).
const STRIATUS_B_FINISHER_VF_TOL: f32 = 0.02;
const STRIATUS_B_BISECT_MAX_ITERS: usize = 48;
const STRIATUS_HEAVISIDE_ETA: f32 = 0.5;
/// SIMP `p` for [`b6-c0-uniform-at-target-vf`] — **Voigt bound** (linear rule of mixtures).
/// Strictest honest smeared reference; schedule-final `p` (e.g. 3) crushes uniform stiffness and trivializes the gate.
const STRIATUS_C0_UNIFORM_SIMP_P: f32 = 1.0;
/// Adjacent-outer greyness ratio above this logs `WARN greyness_jump` (sym-without-absorb signature).
const STRIATUS_GREYNESS_JUMP_WARN_RATIO: f32 = 10.0;

fn parse_b_bisect_tol() -> f32 {
    env::var("UMST_SHELL_B_BISECT_TOL")
        .ok()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            env::var("UMST_SHELL_ETA_BISECT_TOL")
                .ok()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(1e-3_f32)
        .max(1e-8)
}

/// Skip in-loop b-bisect for the first N outers (synthetic guard pathology only).
fn parse_skip_b_bisect_outers() -> usize {
    env::var("UMST_SHELL_SKIP_B_BISECT_OUTERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            env::var("UMST_SHELL_SKIP_ETA_BISECT_OUTERS")
                .ok()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(0)
        .clamp(0, 200)
}

/// In-loop volume enforcement mode (D2).
/// - `logit` (default): uniform logit shift `b` (Hoyer et al. 2019) — can undo compliance descent.
/// - `eta`: OC-style η-bisection on fixed `ρ̃` via [`VolumeEtaProjection`] — preserves spatial layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VolProjMode {
    LogitB,
    /// OC η-bisection on `ρ̃` (preserves layout; VF only when `ρ̃` spread allows).
    EtaOc,
    /// OC λ-shift on `ρ̃` pre-Heaviside (classic mean-preserving rescale).
    LambdaOc,
}

fn parse_vol_proj_mode() -> VolProjMode {
    match env::var("UMST_SHELL_VOL_MODE")
        .ok()
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("eta") | Some("oc") | Some("eta_oc") => VolProjMode::EtaOc,
        Some("lambda") | Some("lambda_oc") => VolProjMode::LambdaOc,
        _ => VolProjMode::LogitB,
    }
}

/// Terminal VF export mode (B0). Default **`oc`** when [`VolProjMode::EtaOc`] in-loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VolExportMode {
    /// η@β_fin + OC λ [`VolumeProjection`] (layout-preserving; no logit-`b` fallback).
    Oc,
    /// Legacy logit-`b` finisher @ β_max (regression only).
    Logit,
}

fn parse_vol_export_mode(vol_mode: VolProjMode) -> VolExportMode {
    match env::var("UMST_SHELL_EXPORT_VOL")
        .ok()
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("oc") | Some("eta_oc") | Some("lambda_oc") => VolExportMode::Oc,
        Some("logit") => VolExportMode::Logit,
        _ if matches!(vol_mode, VolProjMode::EtaOc) => VolExportMode::Oc,
        _ => VolExportMode::Logit,
    }
}

fn export_vol_label(mode: VolExportMode) -> &'static str {
    match mode {
        VolExportMode::Oc => "oc",
        VolExportMode::Logit => "logit",
    }
}

/// Last N outers force β ramp toward `UMST_SHELL_HEAVISIDE_BETA_MAX` while η stays in-loop (B0).
fn parse_binarize_outers() -> usize {
    env::var("UMST_SHELL_BINARIZE_OUTERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        .clamp(0, 200)
}

/// Per-outer capped logit-`b` nudge after η when VF floor binds at low β (D2 Tier 4c). `0` disables.
fn parse_eta_micro_b_max() -> f32 {
    env::var("UMST_SHELL_ETA_MICRO_B_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.5_f32)
        .clamp(0.0, 2.0)
}

fn d1_culprit_label(d_ab: f32, d_bc: f32) -> &'static str {
    if d_ab <= 0.0 && d_bc <= 0.0 {
        return "none(Adam-descent-ok)";
    }
    if d_ab >= d_bc {
        "volume-projection"
    } else {
        "heaviside/filter/sym"
    }
}

/// η / λ in-loop preserves compliance descent; terminal export uses OC or logit-`b` (B0 / D2).
fn hybrid_terminal_export(vol_mode: VolProjMode) -> bool {
    matches!(vol_mode, VolProjMode::EtaOc | VolProjMode::LambdaOc)
}

fn vol_mode_label(vol_mode: VolProjMode) -> &'static str {
    match vol_mode {
        VolProjMode::LogitB => "logit",
        VolProjMode::EtaOc => "eta",
        VolProjMode::LambdaOc => "lambda",
    }
}

/// Apply sym / rib / Helmholtz on a detached density field (bisection inner loop; plain `helm.apply`).
fn apply_rho_raw_pipeline_detached<Bk: BackendTrait<FloatElem = f32>>(
    rho_raw: Tensor<Bk, 3>,
    sym_apply: bool,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
) -> Tensor<Bk, 3> {
    let [_, n, _] = rho_raw.dims();
    let mut rho = rho_raw.reshape([1, n, 1]);
    if sym_apply {
        rho = apply_reflection_xy_average(rho, partners).reshape([1, n, 1]);
    }
    if let Some(pat) = xy_rib_pat {
        rho = rho
            .add(pat.clone().mul_scalar(xy_rib_prior_amp))
            .clamp(0.0, 1.0);
    }
    if helm_on {
        helm.apply(rho.clone(), edges_b1.clone(), dx_f)
            .reshape([1, n, 1])
    } else {
        rho
    }
}

/// Taped sym / rib / Helmholtz (straight-through filter on the AD path).
fn apply_rho_raw_pipeline_taped(
    rho_raw: Tensor<B, 3>,
    sym_apply: bool,
    partners: &Tensor<B, 3, Int>,
    xy_rib_pat: Option<&Tensor<B, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<B, 2, Int>,
    dx_f: f32,
) -> Tensor<B, 3> {
    let [_, n, _] = rho_raw.dims();
    let mut rho = rho_raw.reshape([1, n, 1]);
    if sym_apply {
        rho = apply_reflection_xy_average(rho, partners).reshape([1, n, 1]);
    }
    if let Some(pat) = xy_rib_pat {
        rho = rho
            .add(pat.clone().mul_scalar(xy_rib_prior_amp))
            .clamp(0.0, 1.0);
    }
    if helm_on {
        helm.apply_straight_through(rho.clone(), edges_b1.clone(), dx_f)
            .reshape([1, n, 1])
    } else {
        rho
    }
}

/// Projected VF at a fixed logit offset (detached pipeline).
fn projected_vf_at_b_detached<Bk: BackendTrait<FloatElem = f32>>(
    logits_det: &Tensor<Bk, 3>,
    b: f32,
    beta: f32,
    sym_apply: bool,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
    policy_mask: Option<&Tensor<Bk, 3>>,
) -> f32 {
    let rho = sigmoid(logits_det.clone().add_scalar(b));
    let rho_tilde = apply_rho_raw_pipeline_detached(
        rho,
        sym_apply,
        partners,
        xy_rib_pat,
        xy_rib_prior_amp,
        helm_on,
        helm,
        edges_b1,
        dx_f,
    );
    let rho_tilde = match policy_mask {
        Some(m) => apply_policy_editable_mask(rho_tilde, m),
        None => rho_tilde,
    };
    let rho_mid = HeavisideProjection::new(beta, STRIATUS_HEAVISIDE_ETA).project(rho_tilde);
    let n = rho_mid.dims()[1].max(1) as f32;
    rho_mid.into_data().value.iter().sum::<f32>() / n
}

/// Post-Adam settled greyness on `ρ_mid` (sym off — B2 absorbing-step metric read).
fn greyness_at_vol_absorbed<Bk: BackendTrait<FloatElem = f32>>(
    logits_det: &Tensor<Bk, 3>,
    b: f32,
    beta: f32,
    skip_vol: bool,
    vol_mode: VolProjMode,
    vol_eta: &VolumeEtaProjection,
    target_vf: f32,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
    policy_mask: Option<&Tensor<Bk, 3>>,
    nx: usize,
    ny: usize,
    nz: usize,
) -> f32 {
    let rho_raw = match vol_mode {
        VolProjMode::LogitB if !skip_vol => sigmoid(logits_det.clone().add_scalar(b)),
        _ => sigmoid(logits_det.clone()),
    };
    let rho_tilde = apply_rho_raw_pipeline_detached(
        rho_raw,
        false,
        partners,
        xy_rib_pat,
        xy_rib_prior_amp,
        helm_on,
        helm,
        edges_b1,
        dx_f,
    );
    let mut rho_tilde = match policy_mask {
        Some(m) => apply_policy_editable_mask(rho_tilde, m),
        None => rho_tilde,
    };
    if matches!(vol_mode, VolProjMode::LambdaOc) && !skip_vol {
        let vol_lambda = VolumeProjection::new(target_vf, STRIATUS_B_BISECT_MAX_ITERS);
        rho_tilde = vol_lambda.project(rho_tilde);
    }
    let mut rho_mid = match (vol_mode, skip_vol) {
        (VolProjMode::EtaOc, false) => {
            vol_eta
                .project(rho_tilde, beta, target_vf)
                .into_data()
                .value
        }
        _ => {
            HeavisideProjection::new(beta, STRIATUS_HEAVISIDE_ETA)
                .project(rho_tilde)
                .into_data()
                .value
        }
    };
    apply_non_design_skin(&mut rho_mid, nx, ny, nz);
    greyness_mean(&rho_mid)
}

/// Detached b-bisect on logits with the full sym / rib / helm pipeline (η fixed at 0.5).
fn bisect_logit_offset_b_detached<Bk: BackendTrait<FloatElem = f32>>(
    logits_det: &Tensor<Bk, 3>,
    beta: f32,
    target_vf: f32,
    tol: f32,
    max_iters: usize,
    sym_apply: bool,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
    policy_mask: Option<&Tensor<Bk, 3>>,
) -> f32 {
    let logits_flat = logits_det.clone().into_data().value;
    let eval_vf = |b: f32| {
        projected_vf_at_b_detached(
            logits_det,
            b,
            beta,
            sym_apply,
            partners,
            xy_rib_pat,
            xy_rib_prior_amp,
            helm_on,
            helm,
            edges_b1,
            dx_f,
            policy_mask,
        )
    };
    let mut width = 8.0_f32;
    let (mut lo, mut hi) = loop {
        let vf_lo = eval_vf(-width);
        let vf_hi = eval_vf(width);
        if vf_lo <= target_vf + tol && vf_hi >= target_vf - tol {
            break (-width, width);
        }
        if width > 1_000_000.0 {
            let vf_slice =
                logit_offset_vf_from_slice(&logits_flat, 0.0, beta, STRIATUS_HEAVISIDE_ETA);
            panic!(
                "logit_offset_bisect: bracket failed — vf@b=-{width}={vf_lo:.6} vf@b=+{width}={vf_hi:.6} \
target={target_vf:.6} beta={beta:.3} identity_vf@b=0={vf_slice:.6}"
            );
        }
        width *= 2.0;
    };
    for _ in 0..max_iters.max(1) {
        let mid = 0.5 * (lo + hi);
        let vf = eval_vf(mid);
        if vf > target_vf + tol {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Logit-`b` bisection bracketed to `[b_lo, b_hi]` (η hybrid micro-`b` per outer).
fn bisect_logit_offset_b_bounded_detached<Bk: BackendTrait<FloatElem = f32>>(
    logits_det: &Tensor<Bk, 3>,
    beta: f32,
    target_vf: f32,
    tol: f32,
    max_iters: usize,
    sym_apply: bool,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
    policy_mask: Option<&Tensor<Bk, 3>>,
    b_lo: f32,
    b_hi: f32,
) -> f32 {
    let mut lo = b_lo.min(b_hi);
    let mut hi = b_lo.max(b_hi);
    let eval_vf = |b: f32| {
        projected_vf_at_b_detached(
            logits_det,
            b,
            beta,
            sym_apply,
            partners,
            xy_rib_pat,
            xy_rib_prior_amp,
            helm_on,
            helm,
            edges_b1,
            dx_f,
            policy_mask,
        )
    };
    for _ in 0..max_iters.max(1) {
        let mid = 0.5 * (lo + hi);
        let vf = eval_vf(mid);
        if vf > target_vf + tol {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Predict VF after b-bisect on detached logits (same bisection as in-loop path).
fn b_finisher_predicted_vf<Bk: BackendTrait<FloatElem = f32>>(
    logits_det: &Tensor<Bk, 3>,
    beta: f32,
    target_vf: f32,
    max_iters: usize,
    tol: f32,
    sym_apply: bool,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
    policy_mask: Option<&Tensor<Bk, 3>>,
) -> (f32, f32) {
    let b = bisect_logit_offset_b_detached(
        logits_det,
        beta,
        target_vf,
        tol,
        max_iters,
        sym_apply,
        partners,
        xy_rib_pat,
        xy_rib_prior_amp,
        helm_on,
        helm,
        edges_b1,
        dx_f,
        policy_mask,
    );
    let vf = projected_vf_at_b_detached(
        logits_det,
        b,
        beta,
        sym_apply,
        partners,
        xy_rib_pat,
        xy_rib_prior_amp,
        helm_on,
        helm,
        edges_b1,
        dx_f,
        policy_mask,
    );
    (vf, b)
}

/// Predict VF after η-bisection on detached `ρ̃` (OC volume path).
fn eta_finisher_predicted_vf<Bk: BackendTrait<FloatElem = f32>>(
    logits_det: &Tensor<Bk, 3>,
    beta: f32,
    target_vf: f32,
    vol_eta: &VolumeEtaProjection,
    sym_apply: bool,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
    policy_mask: Option<&Tensor<Bk, 3>>,
) -> f32 {
    let rho_raw = sigmoid(logits_det.clone());
    let rho_tilde = apply_rho_raw_pipeline_detached(
        rho_raw,
        sym_apply,
        partners,
        xy_rib_pat,
        xy_rib_prior_amp,
        helm_on,
        helm,
        edges_b1,
        dx_f,
    );
    let rho_tilde = match policy_mask {
        Some(m) => apply_policy_editable_mask(rho_tilde, m),
        None => rho_tilde,
    };
    let rho_mid = vol_eta.project(rho_tilde, beta, target_vf);
    let n = rho_mid.dims()[1].max(1) as f32;
    rho_mid.into_data().value.iter().sum::<f32>() / n
}

/// Predict VF after λ-shift on `ρ̃` then Heaviside (OC lambda path).
fn lambda_finisher_predicted_vf<Bk: BackendTrait<FloatElem = f32>>(
    logits_det: &Tensor<Bk, 3>,
    beta: f32,
    vol_lambda: &VolumeProjection,
    sym_apply: bool,
    partners: &Tensor<Bk, 3, Int>,
    xy_rib_pat: Option<&Tensor<Bk, 3>>,
    xy_rib_prior_amp: f32,
    helm_on: bool,
    helm: &HelmholtzFilter,
    edges_b1: &Tensor<Bk, 2, Int>,
    dx_f: f32,
    policy_mask: Option<&Tensor<Bk, 3>>,
) -> f32 {
    let rho_raw = sigmoid(logits_det.clone());
    let rho_tilde = apply_rho_raw_pipeline_detached(
        rho_raw,
        sym_apply,
        partners,
        xy_rib_pat,
        xy_rib_prior_amp,
        helm_on,
        helm,
        edges_b1,
        dx_f,
    );
    let rho_tilde = match policy_mask {
        Some(m) => apply_policy_editable_mask(rho_tilde, m),
        None => rho_tilde,
    };
    let rho_tilde = vol_lambda.project(rho_tilde);
    let rho_mid = HeavisideProjection::new(beta, STRIATUS_HEAVISIDE_ETA).project(rho_tilde);
    let n = rho_mid.dims()[1].max(1) as f32;
    rho_mid.into_data().value.iter().sum::<f32>() / n
}

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

    let (nx, ny, nz, lx, ly, lz) = striatus_slab_geometry();
    let iterations = parse_full_rib_adam_iters();
    let iter_total = STRIATUS_B6_SCHEDULE_OUTERS;
    let smoke_subset = iterations < STRIATUS_B6_SCHEDULE_OUTERS;
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
    let policy_mask_vec = policy_editable_mask_vec(nx, ny, nz);
    let policy_mask: Tensor<B, 3> = {
        Tensor::from_data(
            Data::new(policy_mask_vec.clone(), Shape::new([1, n, 1])),
            device,
        )
    };
    let policy_mask_inner = policy_mask.clone().inner();
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
    // Roof traction: thesis re-config loads the fixed solid top skin only (`scripts/b6_harness_setup.sh`).
    let live_f = if thesis_reconfig_enabled() {
        thesis_roof_live_force_vec(nx, ny, nz, dx, dy, roof_ramp_strength, n * 3)
    } else {
        let mut f = vec![0.0f32; n * 3];
        let nx1 = nx + 1;
        let ny1 = ny + 1;
        let iz_top = nz;
        let nx_d = nx.max(1) as f32;
        for iy in 0..=ny {
            for ix in 0..=nx {
                let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
                let w = 1.0_f32 + roof_ramp_strength * (ix as f32 / nx_d);
                f[nid * 3 + 2] = -50.0 * dx * dy * w;
            }
        }
        f
    };
    let live_force: Tensor<B, 3> =
        Tensor::from_data(Data::new(live_f.clone(), Shape::new([1, n, 3])), device);

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
    // In-loop: logit-offset b-bisection (Hoyer et al. 2019) on detached logits, taped apply.
    // `UMST_SHELL_VOL_LOOP=0` disables b-bisect (pathology harness only). Terminal finisher @ β_max unless `UMST_SHELL_VOL_BISECT=0`.
    let vol_b_on = env::var("UMST_SHELL_VOL_LOOP")
        .map(|v| v != "0")
        .unwrap_or(true);
    let vol_b_terminal = env::var("UMST_SHELL_VOL_BISECT")
        .map(|v| v != "0")
        .unwrap_or(true);
    let b_bisect_tol = parse_b_bisect_tol();
    let skip_b_bisect_outers = parse_skip_b_bisect_outers();
    let vol_proj_mode = parse_vol_proj_mode();
    let vol_export_mode = parse_vol_export_mode(vol_proj_mode);
    let binarize_outers = parse_binarize_outers();
    let metrics_on = matches!(env::var("UMST_SHELL_METRICS").as_deref(), Ok("1"));
    let h4_diag = h4_diag_enabled();
    let plateau_beta = PlateauBetaContinuation::new(5, 0.008);
    let vol_logit = VolumeLogitOffsetProjection::new(STRIATUS_B_BISECT_MAX_ITERS, b_bisect_tol);
    let vol_eta = VolumeEtaProjection::new(STRIATUS_B_BISECT_MAX_ITERS, b_bisect_tol);
    let vol_lambda = VolumeProjection::new(target_vf, STRIATUS_B_BISECT_MAX_ITERS);
    let eta_micro_b_cap = parse_eta_micro_b_max();
    let mut eta_micro_b_acc = 0.0_f32;
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

    // `b6-c0-uniform-at-target-vf`: compliance(uniform ρ @ SIMP p = 1) — Voigt-bound baseline.
    // Thesis re-config: skin ρ=1 + uniform interior scaled to target_vf (`scripts/b6_harness_setup.sh`).
    let rho_uniform_vec = if thesis_reconfig_enabled() {
        thesis_uniform_rho_at_vf(nx, ny, nz, target_vf)
    } else {
        vec![target_vf; n]
    };
    let rho_uniform = Tensor::<B, 3>::from_data(
        Data::new(rho_uniform_vec.clone(), Shape::new([1, n, 1])),
        device,
    );
    let bf_uniform = if use_self_weight {
        sw_cfg
            .body_force(rho_uniform.clone())
            .add(live_force.clone())
    } else {
        live_force.clone()
    };
    let simp_uniform_gate = SimpElasticMaterial {
        e0: material.e0,
        nu: material.nu,
        p: STRIATUS_C0_UNIFORM_SIMP_P,
        e_min: material.e_min,
    };
    let boundary_mask_flat = boundary_inner.clone().into_data().value;
    let bf_uniform_inner = bf_uniform.inner();
    let (_, c0_uniform_raw, _) = q1_compliance_with_diagnostics(
        rho_uniform.clone(),
        &plate,
        boundary_inner.clone(),
        bf_uniform_inner.clone(),
        simp_uniform_gate,
        &cg_cfg,
        if use_self_weight { Some(sw_cfg) } else { None },
    );
    if thesis_reconfig_enabled() {
        let frac_c0 = h_c1_a_top_void_fractions(
            &rho_uniform_vec,
            &plate,
            &live_f,
            &boundary_mask_flat,
            simp_uniform_gate,
            &cg_cfg,
            if use_self_weight { Some(sw_cfg) } else { None },
        );
        log_h_c1_a_verdict("shell_topology_rib_pattern_full_v04@c0_uniform", &frac_c0);
    }
    // Audit only: full-schedule final `p` on uniform ρ (e.g. p=3 @ 200 outers — ~34× compliance vs Voigt p=1; not a gate).
    let p_schedule_final = ContinuationSchedule::value(iter_total.saturating_sub(1), iter_total);
    let p_gate = parse_shell_gate_p();
    let simp_uniform_p_final = SimpElasticMaterial {
        e0: material.e0,
        nu: material.nu,
        p: p_gate,
        e_min: material.e_min,
    };
    let (_, c0_uniform_p_final_raw, _) = q1_compliance_with_diagnostics(
        rho_uniform,
        &plate,
        boundary_inner.clone(),
        bf_uniform_inner,
        simp_uniform_p_final,
        &cg_cfg,
        if use_self_weight { Some(sw_cfg) } else { None },
    );
    eprintln!(
        "shell_topology_rib_pattern_full_v04: c0_uniform_at_target_vf \
target_vf={target_vf:.4} gate_p={STRIATUS_C0_UNIFORM_SIMP_P} c0_uniform_raw={c0_uniform_raw:.6} \
audit_p_final={p_schedule_final:.3} p_gate={p_gate:.3} c0_uniform_p_final_raw={c0_uniform_p_final_raw:.6} \
gate_threshold_matched_p={:.6} (0.6·c0_p_final @ p_gate; §9 pairing) eq_rel baseline provenance={}",
        0.6_f32 * c0_uniform_p_final_raw,
        MetricProvenance::FullSolve.as_str(),
    );

    let comp_scale = c0_uniform_raw.max(1e-12);
    let c0 = c0_uniform_raw / comp_scale;
    let c0_uniform_p_final = c0_uniform_p_final_raw / comp_scale;
    #[allow(unused_assignments)]
    let mut c1 = f32::NAN;
    let mut last_rho: Vec<f32> = Vec::new();
    let mut _last_rho_bar: Option<Tensor<B, 3>> = None;
    let mut adam_skipped = 0usize;
    let mut last_grad_l2 = 0.0_f32;
    let mut max_grad_l2 = 0.0_f32;
    let mut min_grad_l2 = f32::INFINITY;
    let mut _first_c1 = f32::NAN;
    let mut _first_xy_var = f32::NAN;
    let mut c1_peak = f32::NEG_INFINITY;
    let mut c1_peak_fixed_p3 = f32::NEG_INFINITY;
    let mut c1_fixed_p3_loop_last = f32::NAN;
    let mut last_rho_raw_min = f32::NAN;
    let mut last_rho_raw_max = f32::NAN;
    let mut last_outer_beta = heaviside_beta0;
    let mut prev_vf_mid = f32::NAN;
    let mut greyness_outer1 = f32::NAN;
    let mut min_xy_var_from_outer_18 = f32::INFINITY;
    let mut min_xy_var_from_outer_50 = f32::INFINITY;
    let mut beta_step_count = 0usize;
    let mut greyness_at_beta_steps: Vec<f32> = Vec::new();
    let mut max_vf_err_abs = 0.0_f32;
    let mut max_vf_err_when_feasible = 0.0_f32;
    let mut _last_b = 0.0_f32;
    let mut prev_greyness_outer = f32::NAN;
    let pcg_tol = cg_cfg.pcg_tolerance.max(cg_cfg.cg_tolerance);
    let beta_max_sched = heaviside_beta_max.max(64.0);
    const RIB_SEED: u64 = 42;
    let t_run_start = Instant::now();
    let mut last_outer_wall_ms = 0.0_f64;

    for it in 1..=iterations {
        let t_outer_start = Instant::now();
        let sched_k = outer_schedule_k(it, iterations);
        let base_beta =
            BetaContinuation::beta(sched_k, iter_total, heaviside_beta0, beta_max_sched);
        let schedule_beta = if binarize_outers > 0 && it + binarize_outers > iterations {
            let start = iterations.saturating_sub(binarize_outers).max(1);
            let denom = (iterations.saturating_sub(start)).max(1) as f32;
            let t = (it.saturating_sub(start) as f32 / denom).clamp(0.0, 1.0);
            let log_b0 = heaviside_beta0.max(1e-6).ln();
            let log_bm = beta_max_sched.max(heaviside_beta0).ln();
            (log_b0 + t * (log_bm - log_b0)).exp()
        } else {
            base_beta
        };
        let beta = match (parse_fixed_heaviside_beta(), vol_export_mode) {
            (Some(fixed), VolExportMode::Logit) => fixed,
            _ => plateau_beta.effective_beta(
                schedule_beta,
                &greyness_hist,
                beta_max_sched,
                last_outer_beta,
            ),
        };
        assert!(
            it == 1 || beta + 1e-6 >= last_outer_beta,
            "striatus_beta_monotone: outer {it} beta={beta:.6} < prev={last_outer_beta:.6}"
        );
        let beta_stepped = it > 1 && beta > last_outer_beta * (1.0 + 1e-6);

        let logits = opt
            .density_net
            .forward_logits_batched(coords_norm.clone())
            .reshape([1, n, 1]);
        // XY reflection is a training regularizer only — never on the final outer (no Adam step after).
        let sym_apply = sym_period > 0 && it % sym_period == 0 && it < iterations;
        let skip_vol = !vol_b_on || it <= skip_b_bisect_outers;
        let partners_inner = partners.clone().inner();
        let edges_inner = edges_b1.clone().inner();
        let xy_rib_inner = xy_rib_pat.as_ref().map(|p| p.clone().inner());
        let (b_star, vf_pred, b_bisect_ok) = match (vol_proj_mode, skip_vol) {
            (VolProjMode::LogitB, true) => (0.0_f32, f32::NAN, false),
            (VolProjMode::LogitB, false) => {
                let logits_det = logits.clone().detach().inner();
                let b = bisect_logit_offset_b_detached(
                    &logits_det,
                    beta,
                    target_vf,
                    b_bisect_tol,
                    STRIATUS_B_BISECT_MAX_ITERS,
                    sym_apply,
                    &partners_inner,
                    xy_rib_inner.as_ref(),
                    xy_rib_prior_amp,
                    helm_on,
                    &helm,
                    &edges_inner,
                    dx_f,
                    Some(&policy_mask_inner),
                );
                let vf = projected_vf_at_b_detached(
                    &logits_det,
                    b,
                    beta,
                    sym_apply,
                    &partners_inner,
                    xy_rib_inner.as_ref(),
                    xy_rib_prior_amp,
                    helm_on,
                    &helm,
                    &edges_inner,
                    dx_f,
                    Some(&policy_mask_inner),
                );
                (b, vf, (vf - target_vf).abs() <= STRIATUS_VF_ERR_ABORT_BAND)
            }
            (VolProjMode::EtaOc, true) => (0.0_f32, f32::NAN, false),
            (VolProjMode::EtaOc, false) => {
                let logits_det = logits.clone().detach().inner();
                let vf = eta_finisher_predicted_vf(
                    &logits_det,
                    beta,
                    target_vf,
                    &vol_eta,
                    sym_apply,
                    &partners_inner,
                    xy_rib_inner.as_ref(),
                    xy_rib_prior_amp,
                    helm_on,
                    &helm,
                    &edges_inner,
                    dx_f,
                    Some(&policy_mask_inner),
                );
                (
                    0.0_f32,
                    vf,
                    (vf - target_vf).abs() <= STRIATUS_VF_ERR_ABORT_BAND,
                )
            }
            (VolProjMode::LambdaOc, true) => (0.0_f32, f32::NAN, false),
            (VolProjMode::LambdaOc, false) => {
                let logits_det = logits.clone().detach().inner();
                let vf = lambda_finisher_predicted_vf(
                    &logits_det,
                    beta,
                    &vol_lambda,
                    sym_apply,
                    &partners_inner,
                    xy_rib_inner.as_ref(),
                    xy_rib_prior_amp,
                    helm_on,
                    &helm,
                    &edges_inner,
                    dx_f,
                    Some(&policy_mask_inner),
                );
                (
                    0.0_f32,
                    vf,
                    (vf - target_vf).abs() <= STRIATUS_VF_ERR_ABORT_BAND,
                )
            }
        };
        _last_b = b_star;
        let rho_raw = match (vol_proj_mode, skip_vol) {
            (VolProjMode::LogitB, false) => vol_logit.apply_shift(logits.clone(), b_star),
            (VolProjMode::EtaOc, false) if eta_micro_b_acc.abs() > 1e-12 => {
                sigmoid(logits.clone().add_scalar(eta_micro_b_acc))
            }
            _ => sigmoid(logits.clone()),
        };
        let rho_tilde = apply_rho_raw_pipeline_taped(
            rho_raw.clone(),
            sym_apply,
            &partners,
            xy_rib_pat.as_ref(),
            xy_rib_prior_amp,
            helm_on,
            &helm,
            &edges_b1,
            dx_f,
        );
        let mut rho_tilde = apply_policy_editable_mask(rho_tilde, &policy_mask);
        if matches!(vol_proj_mode, VolProjMode::LambdaOc) && !skip_vol {
            rho_tilde = vol_lambda.project(rho_tilde);
        }
        let mut rho_mid = match (vol_proj_mode, skip_vol) {
            (VolProjMode::EtaOc, false) => vol_eta
                .project_with_mask(
                    rho_tilde.clone(),
                    beta,
                    target_vf,
                    Some(policy_mask_vec.as_slice()),
                )
                .reshape([1, n, 1]),
            _ => HeavisideProjection::new(beta, STRIATUS_HEAVISIDE_ETA)
                .project(rho_tilde.clone())
                .reshape([1, n, 1]),
        };
        if matches!(vol_proj_mode, VolProjMode::EtaOc) && !skip_vol && eta_micro_b_cap > 0.0 {
            let vf_after_eta = rho_mid.clone().into_data().value.iter().sum::<f32>() / n as f32;
            if (vf_after_eta - target_vf).abs() > b_bisect_tol {
                let logits_det = logits.clone().detach().inner();
                let b_lo = eta_micro_b_acc - eta_micro_b_cap;
                let b_hi = eta_micro_b_acc + eta_micro_b_cap;
                let b_micro = bisect_logit_offset_b_bounded_detached(
                    &logits_det,
                    beta,
                    target_vf,
                    b_bisect_tol,
                    STRIATUS_B_BISECT_MAX_ITERS,
                    sym_apply,
                    &partners_inner,
                    xy_rib_inner.as_ref(),
                    xy_rib_prior_amp,
                    helm_on,
                    &helm,
                    &edges_inner,
                    dx_f,
                    Some(&policy_mask_inner),
                    b_lo,
                    b_hi,
                );
                eta_micro_b_acc = b_micro;
                let rho_raw_mb = sigmoid(logits.clone().add_scalar(eta_micro_b_acc));
                let rho_tilde_mb = apply_rho_raw_pipeline_taped(
                    rho_raw_mb,
                    sym_apply,
                    &partners,
                    xy_rib_pat.as_ref(),
                    xy_rib_prior_amp,
                    helm_on,
                    &helm,
                    &edges_b1,
                    dx_f,
                );
                let rho_tilde_mb = apply_policy_editable_mask(rho_tilde_mb, &policy_mask);
                rho_mid = vol_eta
                    .project_with_mask(
                        rho_tilde_mb,
                        beta,
                        target_vf,
                        Some(policy_mask_vec.as_slice()),
                    )
                    .reshape([1, n, 1]);
            }
        }
        let rho_mech = rho_mid.clone();

        if d1_three_point_enabled() {
            let rho_a_inner = sigmoid(logits.clone().detach()).inner();
            let c_a = rib_c1_fixed_p3_at_rho_inner(
                rho_a_inner,
                &plate,
                &boundary_inner,
                &live_force,
                &material,
                &cg_cfg,
                use_self_weight,
                sw_cfg,
                comp_scale,
                device,
            );
            let rho_b_tensor = match vol_proj_mode {
                VolProjMode::LogitB => rho_raw.clone().detach().inner(),
                VolProjMode::EtaOc | VolProjMode::LambdaOc => rho_tilde.clone().detach().inner(),
            };
            let c_b = rib_c1_fixed_p3_at_rho_inner(
                rho_b_tensor,
                &plate,
                &boundary_inner,
                &live_force,
                &material,
                &cg_cfg,
                use_self_weight,
                sw_cfg,
                comp_scale,
                device,
            );
            let c_c = rib_c1_fixed_p3_at_rho_inner(
                rho_mid.clone().detach().inner(),
                &plate,
                &boundary_inner,
                &live_force,
                &material,
                &cg_cfg,
                use_self_weight,
                sw_cfg,
                comp_scale,
                device,
            );
            let d_ab = c_b - c_a;
            let d_bc = c_c - c_b;
            let culprit = d1_culprit_label(d_ab, d_bc);
            eprintln!(
                "shell_topology_rib_pattern_full_v04: D1 three-point outer {it}/{iter_total} \
beta={beta:.3} sym={} vol_mode={} | (a)post-Adam={c_a:.6} (b)post-vol={c_b:.6} (c)post-Heaviside={c_c:.6} \
Δab={d_ab:+.6} Δbc={d_bc:+.6} Δac={:+.6} culprit={culprit}",
                u8::from(sym_apply),
                vol_mode_label(vol_proj_mode),
                c_c - c_a,
            );
        }

        if metrics_on && (it % 20 == 0 || it == iterations) {
            let grey_mid = greyness_mean(&rho_mid.clone().into_data().value);
            let vf_mid_m = rho_mid.clone().into_data().value.iter().sum::<f32>() / n as f32;
            eprintln!(
                "shell_topology_rib_pattern_full_v04: outer {it}/{iter_total} greyness_rho_mid={grey_mid:.6} \
beta={beta:.3} b={b_star:.6} vf_mid={vf_mid_m:.6} vf_err={:+.6} skip_vol={skip_vol} b_bisect_ok={b_bisect_ok} helm_on={helm_on} \
vol_b_on={vol_b_on} vol_b_terminal={vol_b_terminal}",
                vf_mid_m - target_vf,
            );
        }

        let bf = if use_self_weight {
            sw_cfg.body_force(rho_mech.clone()).add(live_force.clone())
        } else {
            live_force.clone()
        };
        let p_act = continuation_schedule_p(sched_k, iter_total);
        let simp_mat = SimpElasticMaterial {
            e0: material.e0,
            nu: material.nu,
            p: p_act,
            e_min: material.e_min,
        };
        let rho_raw_vec = rho_raw.clone().into_data().value;
        let bf_inner = bf.inner();
        _last_rho_bar = Some(rho_mech.clone());
        // Compliance AD on post-Heaviside ρ_mid (A′). `UMST_SHELL_H5_SKIP_PROJ=1` → raw net output.
        let rho_comp = if h5_skip_projection_compliance() {
            rho_raw.clone()
        } else {
            rho_mid.clone()
        };
        let sw_adj = if use_self_weight { Some(sw_cfg) } else { None };
        let (surrogate, c_raw, h4_bundle) = if h4_diag || smoke_subset || metrics_on {
            let (s, c, diag) = q1_compliance_with_diagnostics(
                rho_comp.clone(),
                &plate,
                boundary_inner.clone(),
                bf_inner.clone(),
                simp_mat,
                &cg_cfg,
                sw_adj,
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
                sw_adj,
            );
            (s, c, None)
        };

        last_rho = rho_mid.clone().into_data().value;
        apply_non_design_skin(&mut last_rho, nx, ny, nz);
        let grey_now = greyness_mean(&last_rho);
        greyness_hist.push(grey_now);

        if it == 1 {
            assert!(
                c_raw.is_finite(),
                "B6 full harness: non-finite raw compliance at iter 1 (Q1-hex PCG / load path). \
Try UMST_SHELL_SELF_WEIGHT=0, UMST_SHELL_MAX_CG>=2000, UMST_SHELL_E_MIN_REL=0.001, UMST_SHELL_PCG=1. \
Got c_raw={c_raw:?} (self_weight={use_self_weight}, vol_b_on={vol_b_on}, max_cg={max_cg})."
            );
        }
        c1 = c_raw / comp_scale;
        if c1.is_finite() {
            c1_peak = c1_peak.max(c1);
        }
        if it == 1 {
            _first_c1 = c1;
            _first_xy_var = xy_plane_variance(&last_rho, nx, ny, nz);
        }
        let surrogate_scalar = surrogate.clone().into_data().value[0];
        let comp_loss_scaled = surrogate_scalar / comp_scale;
        // D1: fixed-p (=3) c1 beside running-p c1 (schedule `p_act`).
        let c1_fixed_p3 = if metrics_on || smoke_subset {
            let simp_mat_p3 = SimpElasticMaterial {
                e0: material.e0,
                nu: material.nu,
                p: material.simp_p,
                e_min: material.e_min,
            };
            let rho_flat_diag = rho_comp.clone().inner().into_data().value;
            let f_flat_diag = bf_inner.clone().into_data().value;
            let m_flat_diag = boundary_inner.clone().into_data().value;
            let c_audit_p3 = AdjointComplianceQ1Hex::raw_compliance_at_rho(
                &rho_flat_diag,
                nx,
                ny,
                nz,
                dx,
                dy,
                dz,
                &f_flat_diag,
                &m_flat_diag,
                simp_mat_p3,
                &cg_cfg,
                sw_adj,
            );
            c_audit_p3 / comp_scale
        } else {
            f32::NAN
        };
        if c1_fixed_p3.is_finite() {
            c1_peak_fixed_p3 = c1_peak_fixed_p3.max(c1_fixed_p3);
            c1_fixed_p3_loop_last = c1_fixed_p3;
        }
        if it == 1 && (metrics_on || smoke_subset) {
            // D3: surrogate compliance vs reported c1 — same ρ, e_cell(ρ^p·E0+E_min), and u.
            let c_raw_audit = AdjointComplianceQ1Hex::raw_compliance_at_rho(
                &rho_comp.clone().inner().into_data().value,
                nx,
                ny,
                nz,
                dx,
                dy,
                dz,
                &bf_inner.clone().into_data().value,
                &boundary_inner.clone().into_data().value,
                simp_mat,
                &cg_cfg,
                sw_adj,
            );
            eprintln!(
                "shell_topology_rib_pattern_full_v04: c1_diag_d3 outer1 \
p_act={p_act:.3} c_raw_fwd={c_raw:.9} c_raw_audit={c_raw_audit:.9} c_raw_delta={:.3e} \
surrogate={surrogate_scalar:.9} comp_loss_scaled={comp_loss_scaled:.9} c1_running={c1:.9} \
same_e_cell_u=true",
                (c_raw_audit - c_raw).abs(),
            );
        }
        let mut total_loss = surrogate.clone().div_scalar(comp_scale);
        if grey_lambda > 0.0 {
            let grey_t = mean_greyness_tensor(rho_mid.clone());
            total_loss = total_loss.add(grey_t.mul_scalar(grey_lambda));
        }
        if xy_var_lambda > 0.0 {
            let v_xy = xy_plane_variance_z_avg_tensor(rho_mid.clone(), nx, ny, nz);
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
            last_outer_wall_ms = t_outer_start.elapsed().as_secs_f64() * 1000.0;
            if metrics_on || smoke_subset {
                eprintln!(
                    "shell_topology_rib_pattern_full_v04: outer {it}/{iterations} wall_ms={last_outer_wall_ms:.1} \
total_s={:.3} seed={RIB_SEED} backend_features={}",
                    t_run_start.elapsed().as_secs_f64(),
                    active_backend_feature_set(),
                );
            }
            continue;
        }

        let rho_bar_grad_anchor = rho_comp.clone();
        let grads = total_loss.backward();
        let rho_bar_grad = rho_bar_grad_anchor
            .grad(&grads)
            .map(|g| tensor_grad_l2_inner(&g));
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
        let vf_err_abs = vf_err.abs();
        max_vf_err_abs = max_vf_err_abs.max(vf_err_abs);
        if b_bisect_ok {
            max_vf_err_when_feasible = max_vf_err_when_feasible.max(vf_err_abs);
        }
        let vf_drift = if prev_vf_mid.is_finite() {
            vf_now - prev_vf_mid
        } else {
            f32::NAN
        };
        prev_vf_mid = vf_now;

        if beta_stepped {
            beta_step_count += 1;
            greyness_at_beta_steps.push(greyness_mean(&last_rho));
        }

        // Logit-offset tripwire: skipped-b pathology, or feasible bisection that tape/guard violates.
        if skip_vol && vf_err_abs > STRIATUS_VF_ERR_ABORT_BAND {
            panic!(
                "striatus_vf_bisect_guard: |vf-target|>{STRIATUS_VF_ERR_ABORT_BAND} with b-bisect skipped \
(outer {it}/{iterations} vf_mid={vf_now:.6} err={vf_err:+.6} b={b_star:.6} beta={beta:.3})"
            );
        }
        if b_bisect_ok && vf_err_abs > STRIATUS_VF_ERR_ABORT_BAND {
            panic!(
                "striatus_vf_bisect_guard: b-bisect ok (vf_pred={vf_pred:.6}) but |vf_mid-target|>{STRIATUS_VF_ERR_ABORT_BAND} \
(outer {it}/{iterations} vf_mid={vf_now:.6} err={vf_err:+.6} b={b_star:.6} beta={beta:.3} vf_drift={vf_drift:+.6})"
            );
        }
        if smoke_subset || metrics_on {
            let xy_now = xy_plane_variance(&last_rho, nx, ny, nz);
            if it >= 18 {
                min_xy_var_from_outer_18 = min_xy_var_from_outer_18.min(xy_now);
            }
            if it >= 50 {
                min_xy_var_from_outer_50 = min_xy_var_from_outer_50.min(xy_now);
            }
            let eq_rel = h4_bundle
                .as_ref()
                .map(|d| d.equilibrium_rel_residual)
                .unwrap_or(f32::NAN);
            if smoke_subset {
                assert_pcg_equilibrium_gate(
                    &format!("shell_topology_rib_pattern_full_v04 smoke outer {it}"),
                    h4_bundle
                        .as_ref()
                        .map(|d| d.pcg.rel_residual)
                        .unwrap_or(f32::NAN),
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
schedule_k={} beta={beta:.3} b={b_star:.6} vf_mid={vf_now:.6} vf_err={vf_err:+.6} vf_drift={vf_drift:+.6} \
greyness={grey_now:.6} grad_l2={grad_l2:.6} xy_var={xy_now:.6} c1={c1:.6} eq_rel={eq_rel:.3e} \
vf_pred={vf_pred:.6} b_bisect_ok={} beta_stepped={} skip_b={}",
                sched_k,
                u8::from(b_bisect_ok),
                u8::from(beta_stepped),
                u8::from(skip_vol),
            );
            // D1/D2: fixed-p c1 + optimizer loss beside running-p c1.
            eprintln!(
                "shell_topology_rib_pattern_full_v04: c1_diag outer {it}/{iterations} \
p_act={p_act:.3} c1_running={c1:.6} c1_fixed_p3={c1_fixed_p3:.6} \
comp_loss_scaled={comp_loss_scaled:.6} total_loss={loss_scalar:.6}",
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
                    rho_bar_grad,
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
        let logits_absorbed = opt
            .density_net
            .forward_logits_batched(coords_norm.clone())
            .reshape([1, n, 1]);
        let grey_absorbed = greyness_at_vol_absorbed(
            &logits_absorbed.clone().detach().inner(),
            b_star,
            beta,
            skip_vol,
            vol_proj_mode,
            &vol_eta,
            target_vf,
            &partners.clone().inner(),
            xy_rib_pat.as_ref().map(|p| p.clone().inner()).as_ref(),
            xy_rib_prior_amp,
            helm_on,
            &helm,
            &edges_b1.clone().inner(),
            dx_f,
            Some(&policy_mask_inner),
            nx,
            ny,
            nz,
        );
        warn_greyness_jump_if_needed(
            "shell_topology_rib_pattern_full_v04",
            it,
            iter_total,
            prev_greyness_outer,
            grey_absorbed,
            false,
        );
        prev_greyness_outer = grey_absorbed;
        if it == 1 {
            greyness_outer1 = grey_absorbed;
        }
        last_outer_wall_ms = t_outer_start.elapsed().as_secs_f64() * 1000.0;
        if metrics_on || smoke_subset {
            eprintln!(
                "shell_topology_rib_pattern_full_v04: outer {it}/{iterations} wall_ms={last_outer_wall_ms:.1} \
total_s={:.3} seed={RIB_SEED} backend_features={}",
                t_run_start.elapsed().as_secs_f64(),
                active_backend_feature_set(),
            );
        }
    }

    let total_wall_s = t_run_start.elapsed().as_secs_f64();
    assert!(!last_rho.is_empty(), "full rib run produced no ρ");
    let vf_loop = last_rho.iter().sum::<f32>() / last_rho.len() as f32;
    let greyness_loop_last = greyness_mean(&last_rho);
    let mut vf_export = vf_loop;
    let mut rho_acceptance = last_rho.clone();

    // Terminal volume finisher at β_max. Full 200-outer always; smoke subsets too when hybrid
    // (η/λ in-loop + terminal export) so VF gates see `vf_export` not drifting in-loop VF.
    let run_terminal_finisher =
        vol_b_terminal && (!smoke_subset || hybrid_terminal_export(vol_proj_mode));
    if run_terminal_finisher {
        let logits_f = opt
            .density_net
            .forward_logits_batched(coords_norm.clone())
            .reshape([1, n, 1]);
        // Acceptance export: no XY reflection (training-only; reflection on a near-binary field injects grey).
        let sym_apply_f = false;
        let finisher_beta = heaviside_beta_max.max(last_outer_beta);
        let logits_det_f = logits_f.clone().detach().inner();
        let partners_inner = partners.clone().inner();
        let edges_inner = edges_b1.clone().inner();
        let xy_rib_inner = xy_rib_pat.as_ref().map(|p| p.clone().inner());
        // B0: η@β_fin + OC λ export (default for η-hybrid). D2 regression: logit-`b` when EXPORT_VOL=logit.
        let (vf_pred, b_star, rho_bar_f, finisher_eta, finisher_oc) = {
            let rho_raw_eta =
                if matches!(vol_proj_mode, VolProjMode::EtaOc) && eta_micro_b_acc.abs() > 1e-12 {
                    sigmoid(logits_f.clone().add_scalar(eta_micro_b_acc))
                } else {
                    sigmoid(logits_f.clone())
                };
            let rho_tilde_eta = apply_rho_raw_pipeline_taped(
                rho_raw_eta,
                sym_apply_f,
                &partners,
                xy_rib_pat.as_ref(),
                xy_rib_prior_amp,
                helm_on,
                &helm,
                &edges_b1,
                dx_f,
            );
            let rho_tilde_eta = apply_policy_editable_mask(rho_tilde_eta, &policy_mask);
            if matches!(vol_export_mode, VolExportMode::Oc)
                && matches!(vol_proj_mode, VolProjMode::EtaOc | VolProjMode::LambdaOc)
            {
                let rho_pre = if matches!(vol_proj_mode, VolProjMode::LambdaOc) {
                    vol_lambda.project(rho_tilde_eta.clone())
                } else {
                    vol_eta.project_with_mask(
                        rho_tilde_eta.clone(),
                        finisher_beta,
                        target_vf,
                        Some(policy_mask_vec.as_slice()),
                    )
                };
                let rho_oc = vol_lambda.project(rho_pre.reshape([1, n, 1]));
                let mut rho_oc_vec = rho_oc.clone().into_data().value;
                apply_non_design_skin(&mut rho_oc_vec, nx, ny, nz);
                let vf_oc = rho_oc_vec.iter().sum::<f32>() / rho_oc_vec.len() as f32;
                let rho_bar =
                    Tensor::<B, 3>::from_data(Data::new(rho_oc_vec, Shape::new([1, n, 1])), device)
                        .reshape([1, n, 1]);
                (
                    vf_oc,
                    eta_micro_b_acc,
                    rho_bar,
                    matches!(vol_proj_mode, VolProjMode::EtaOc),
                    true,
                )
            } else {
                let mut eta_fin_ok = false;
                let mut vf_eta_try = f32::NAN;
                let b_eta_try = eta_micro_b_acc;
                let mut rho_eta_bar = rho_tilde_eta.clone().reshape([1, n, 1]);
                if matches!(vol_proj_mode, VolProjMode::EtaOc)
                    && hybrid_terminal_export(vol_proj_mode)
                {
                    let rho_eta_fin = vol_eta.project_with_mask(
                        rho_tilde_eta.clone(),
                        finisher_beta,
                        target_vf,
                        Some(policy_mask_vec.as_slice()),
                    );
                    let mut rho_eta_vec = rho_eta_fin.clone().into_data().value;
                    apply_non_design_skin(&mut rho_eta_vec, nx, ny, nz);
                    vf_eta_try = rho_eta_vec.iter().sum::<f32>() / rho_eta_vec.len() as f32;
                    if (vf_eta_try - target_vf).abs() <= STRIATUS_VF_ERR_ABORT_BAND {
                        rho_eta_bar = Tensor::<B, 3>::from_data(
                            Data::new(rho_eta_vec, Shape::new([1, n, 1])),
                            device,
                        )
                        .reshape([1, n, 1]);
                        eta_fin_ok = true;
                    }
                }
                if eta_fin_ok {
                    (vf_eta_try, b_eta_try, rho_eta_bar, true, false)
                } else {
                    let (vf_pred, b_star) = b_finisher_predicted_vf(
                        &logits_det_f,
                        finisher_beta,
                        target_vf,
                        vol_logit.max_bisection,
                        vol_logit.tol,
                        sym_apply_f,
                        &partners_inner,
                        xy_rib_inner.as_ref(),
                        xy_rib_prior_amp,
                        helm_on,
                        &helm,
                        &edges_inner,
                        dx_f,
                        Some(&policy_mask_inner),
                    );
                    let rho_raw_f = vol_logit.apply_shift(logits_f.clone(), b_star);
                    let rho_tilde_f = apply_rho_raw_pipeline_taped(
                        rho_raw_f,
                        sym_apply_f,
                        &partners,
                        xy_rib_pat.as_ref(),
                        xy_rib_prior_amp,
                        helm_on,
                        &helm,
                        &edges_b1,
                        dx_f,
                    );
                    let rho_tilde_f = apply_policy_editable_mask(rho_tilde_f, &policy_mask);
                    let rho_bar_f = HeavisideProjection::new(finisher_beta, STRIATUS_HEAVISIDE_ETA)
                        .project(rho_tilde_f.reshape([1, n, 1]))
                        .reshape([1, n, 1]);
                    (vf_pred, b_star, rho_bar_f, false, false)
                }
            }
        };
        let grey_tilde = greyness_mean(&rho_bar_f.clone().into_data().value);
        let vf_pred_err = vf_pred - target_vf;
        if vf_pred_err.abs() > STRIATUS_B_FINISHER_VF_TOL {
            panic!(
                "striatus_vol_finisher_unreachable: volume projection cannot reach target_vf on current \
logits. vol_mode={} export={} beta_fin={finisher_beta:.3} b*={b_star:.6} vf_predicted={vf_pred:.6} \
vf_err={vf_pred_err:+.6} tol={STRIATUS_B_FINISHER_VF_TOL} greyness_tilde={grey_tilde:.6} \
vf_loop={vf_loop:.6} finisher_oc={} — not exporting a bogus field",
                vol_mode_label(vol_proj_mode),
                export_vol_label(vol_export_mode),
                u8::from(finisher_oc),
            );
        }
        let mut rho_export = rho_bar_f.clone().into_data().value;
        apply_non_design_skin(&mut rho_export, nx, ny, nz);
        rho_acceptance = rho_export.clone();
        vf_export = rho_export.iter().sum::<f32>() / rho_export.len() as f32;
        let vf_export_err = vf_export - target_vf;
        if vf_export_err.abs() > STRIATUS_VF_ERR_ABORT_BAND {
            panic!(
                "striatus_terminal_vf_guard: |vf_export-target|>{STRIATUS_VF_ERR_ABORT_BAND} \
after b-bisect (vf_loop={vf_loop:.6} vf_export={vf_export:.6} err={vf_export_err:+.6} \
greyness_export={:.6})",
                greyness_mean(&rho_export),
            );
        }
        eprintln!(
            "shell_topology_rib_pattern_full_v04: terminal finisher vol_mode={} export={} hybrid={} finisher_eta={} finisher_oc={} \
beta_cont={:.3} beta_fin={finisher_beta:.3} b={b_star:.6} vf_loop={vf_loop:.6} vf_export={vf_export:.6} \
vf_export_err={vf_export_err:+.6} greyness_export={:.6}",
            vol_mode_label(vol_proj_mode),
            export_vol_label(vol_export_mode),
            u8::from(hybrid_terminal_export(vol_proj_mode)),
            u8::from(finisher_eta),
            u8::from(finisher_oc),
            last_outer_beta,
            greyness_mean(&rho_export),
        );
    }

    // Acceptance gates: post-finisher export ρ (one equilibrium solve on exactly that field).
    if let Ok(export_path) = env::var("UMST_SHELL_EXPORT_RHO") {
        let bytes: Vec<u8> = rho_acceptance
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        if let Err(e) = std::fs::write(&export_path, bytes) {
            eprintln!(
                "shell_topology_rib_pattern_full_v04: WARN failed to export rho to {export_path}: {e}"
            );
        } else {
            eprintln!(
                "shell_topology_rib_pattern_full_v04: exported acceptance rho ({} nodes) -> {export_path}",
                rho_acceptance.len()
            );
        }
    }
    let rho_acceptance_t = Tensor::<B, 3>::from_data(
        Data::new(rho_acceptance.clone(), Shape::new([1, n, 1])),
        device,
    );
    let p_act = CompliancePenalization::Schedule {
        outer: iterations.saturating_sub(1),
        total: iter_total,
    }
    .resolve_p(continuation_schedule_p);
    let p_gate_accept =
        CompliancePenalization::Gate(parse_shell_gate_p()).resolve_p(continuation_schedule_p);
    let simp_accept = SimpElasticMaterial {
        e0: material.e0,
        nu: material.nu,
        p: p_act,
        e_min: material.e_min,
    };
    let simp_gate = SimpElasticMaterial {
        e0: material.e0,
        nu: material.nu,
        p: p_gate_accept,
        e_min: material.e_min,
    };
    let bf_accept = if use_self_weight {
        sw_cfg
            .body_force(rho_acceptance_t.clone())
            .add(live_force.clone())
    } else {
        live_force.clone()
    };
    let bf_accept_inner = bf_accept.inner();
    let (_, c1_accept_raw, final_diag) = q1_compliance_with_diagnostics(
        rho_acceptance_t.clone(),
        &plate,
        boundary_inner.clone(),
        bf_accept_inner.clone(),
        simp_accept,
        &cg_cfg,
        if use_self_weight { Some(sw_cfg) } else { None },
    );
    let (_, c1_gate_raw, _) = q1_compliance_with_diagnostics(
        rho_acceptance_t,
        &plate,
        boundary_inner.clone(),
        bf_accept_inner,
        simp_gate,
        &cg_cfg,
        if use_self_weight { Some(sw_cfg) } else { None },
    );
    let c1_running_accept = c1_accept_raw / comp_scale;
    let c1_gate_norm = c1_gate_raw / comp_scale;
    c1 = c1_gate_norm;
    let _ = (c1_running_accept, c1);
    let (h_c1_a_comp_frac, h_c1_a_se_frac) = if thesis_reconfig_enabled() {
        let frac = h_c1_a_top_void_fractions(
            &rho_acceptance,
            &plate,
            &live_f,
            &boundary_mask_flat,
            simp_accept,
            &cg_cfg,
            if use_self_weight { Some(sw_cfg) } else { None },
        );
        log_h_c1_a_verdict("shell_topology_rib_pattern_full_v04@acceptance", &frac);
        (frac.compliance_fraction, frac.strain_energy_fraction)
    } else {
        (f32::NAN, f32::NAN)
    };
    let vf = if run_terminal_finisher {
        vf_export
    } else {
        vf_loop
    };
    let z_profile = rho_z_layer_profile(&rho_acceptance, nx, ny, nz);
    let min_xy_18 = if min_xy_var_from_outer_18.is_finite() {
        min_xy_var_from_outer_18
    } else {
        f32::NAN
    };
    let metrics = RibMetrics {
        vf,
        greyness: greyness_mean(&rho_acceptance),
        xy_var: xy_plane_variance(&rho_acceptance, nx, ny, nz),
        c0,
        c1_running: c1_running_accept,
        c1_fixed_p3: c1_gate_norm,
        p_act,
        p_gate: p_gate_accept,
        c1: c1_gate_norm,
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
        vf_loop,
        vf_export,
        greyness_outer1,
        c1_peak,
        c1_peak_fixed_p3,
        c1_fixed_p3_loop_last,
        min_xy_var_from_outer_18: min_xy_18,
        min_xy_var_from_outer_50: if min_xy_var_from_outer_50.is_finite() {
            min_xy_var_from_outer_50
        } else {
            f32::NAN
        },
        beta_step_count,
        greyness_at_beta_steps,
        max_vf_err_abs,
        max_vf_err_when_feasible,
        c0_uniform: c0,
        c0_uniform_raw,
        c0_uniform_p_final_raw,
        c0_uniform_p_final,
        h_c1_a_comp_frac,
        h_c1_a_se_frac,
        greyness_loop_last,
        last_outer_wall_ms,
        total_wall_s,
        seed: RIB_SEED,
        active_backend_features: active_backend_feature_set(),
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
        None,
    );
    if let Some(audit) = &diag.finite_audit {
        assert_eq!(audit.first_bad_stage, None, "{audit:?}");
    }
    let grads = surrogate.backward();
    let grads_params = GradientsParams::from_grads(grads, &opt.density_net);
    let (comp_l2, comp_max, nf_layers) = autodiff_param_grad_audit(&grads_params, &opt.density_net);
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
    let use_uniform_load = matches!(env::var("UMST_H5_PROBE_UNIFORM_LOAD").as_deref(), Ok("1"));
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
        None,
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
max_grad_l2={:.6} last_grad_l2={:.6} g_uni=4·vf·(1−vf)={:.6} pcg_iter_final={} pcg_rel_res={:.3e} eq_rel_res={:.3e} adam_skipped={}/{} UMST_SHELL_GREY_LAMBDA={:.6} UMST_SHELL_XY_VAR_LAMBDA={:.6} UMST_SHELL_HEAVISIDE_BETA0={:.6} UMST_SHELL_DENSITY_INIT_JITTER={:.6} UMST_SHELL_XY_RIB_PRIOR_AMP={:.6} \
last_outer_wall_ms={:.1} total_wall_s={:.3} seed={} backend_features={}",
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
        rib,
        m.last_outer_wall_ms,
        m.total_wall_s,
        m.seed,
        m.active_backend_features,
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
    // Smoke (`UMST_SHELL_RIB_FULL_ITERS` < 200): logit-offset health before 200-outer acceptance.
    // Pass = vf within ±0.02 every outer, greyness↓, xy_var alive @18+, grad_l2 bounded.
    if adam_iters < 200 {
        assert!(
            m.max_grad_l2.is_finite() && m.max_grad_l2 > 0.0 && m.last_grad_l2 > 0.0,
            "smoke grad_l2: max={} last={} (adam_skipped={} vf_loop={} xy_var={})",
            m.max_grad_l2,
            m.last_grad_l2,
            m.adam_skipped,
            m.vf_loop,
            m.xy_var
        );
        assert!(
            m.max_vf_err_when_feasible.is_finite()
                && (m.max_vf_err_when_feasible <= STRIATUS_VF_ERR_ABORT_BAND
                    || m.max_vf_err_when_feasible == 0.0),
            "smoke vf when b-bisect ok: max|vf_err|={} band={}",
            m.max_vf_err_when_feasible,
            STRIATUS_VF_ERR_ABORT_BAND
        );
        assert!(
            (m.vf - m.target_vf).abs() <= STRIATUS_VF_ERR_ABORT_BAND,
            "smoke vf (export when hybrid finisher ran): vf={} vf_loop={} vf_export={} target={} err={}",
            m.vf,
            m.vf_loop,
            m.vf_export,
            m.target_vf,
            m.vf - m.target_vf
        );
        assert_eq!(
            m.adam_skipped, 0,
            "smoke adam_skipped: got {} (grad_l2={} vf_loop={})",
            m.adam_skipped, m.last_grad_l2, m.vf_loop
        );
        assert!(
            m.last_rho_raw_min < 0.501 - 1e-6 || m.last_rho_raw_max > 0.501 + 1e-6,
            "smoke rho_raw plateau: [{}, {}] must leave [0.501, 0.501]",
            m.last_rho_raw_min,
            m.last_rho_raw_max
        );
        if matches!(
            parse_vol_export_mode(parse_vol_proj_mode()),
            VolExportMode::Oc
        ) && m.last_outer_beta
            >= parse_umst_shell_b6_aux_env().2.max(
                env::var("UMST_SHELL_HEAVISIDE_BETA_MAX")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(32.0_f32),
            ) * 0.95
        {
            assert!(
                m.greyness < 0.15,
                "B0 smoke oc export greyness gate: got {} (vf={} beta_last={})",
                m.greyness,
                m.vf,
                m.last_outer_beta
            );
        }
        assert!(
            m.greyness < m.greyness_outer1 - 1e-4,
            "smoke A′ greyness falling: outer1={} final={}",
            m.greyness_outer1,
            m.greyness
        );
        if adam_iters >= 18 {
            assert!(
                m.min_xy_var_from_outer_18.is_finite() && m.min_xy_var_from_outer_18 > 1e-6,
                "smoke A′ xy_var past outer 18: min_xy_var={} final_xy_var={}",
                m.min_xy_var_from_outer_18,
                m.xy_var
            );
            // Legacy Striatus: in-loop fixed-p3 c1 should trend down before acceptance finisher reshapes ρ.
            if !thesis_reconfig_enabled() {
                assert!(
                    m.c1_peak_fixed_p3.is_finite()
                        && m.c1_fixed_p3_loop_last < m.c1_peak_fixed_p3 - 1e-3,
                    "smoke A′ c1_fixed_p3 trending down from peak: loop_last={} peak={} acceptance_c1={}",
                    m.c1_fixed_p3_loop_last,
                    m.c1_peak_fixed_p3,
                    m.c1
                );
            }
        } else {
            assert!(
                m.xy_var.is_finite(),
                "smoke short-run xy_var finite: {}",
                m.xy_var
            );
        }
        if thesis_reconfig_enabled() {
            assert!(
                m.h_c1_a_comp_frac.is_finite() && m.h_c1_a_comp_frac < 0.5,
                "thesis smoke H-c1-A: void_column_compliance={:.1}%",
                m.h_c1_a_comp_frac * 100.0
            );
            assert!(
                m.c0_uniform_raw.is_finite() && m.c0_uniform_raw > 0.0,
                "thesis smoke c0_uniform_raw recorded: {}",
                m.c0_uniform_raw
            );
        }
        eprintln!(
            "shell_topology_rib_pattern_full_v04: smoke logit-offset PASS ({adam_iters} outer) — \
GREYNESS={:.6} (outer1={:.6}) max_grad_l2={:.6} vf_loop={:.6} vf_export={:.6} \
min_xy_var@18+={:.6} xy_var={:.6} c0={:.6} c1={:.6} c1_peak={:.6} c1_peak_fixed_p3={:.6} c1_loop_last={:.6} beta_last={:.3} beta_steps={} eq_rel={:.3e}",
            m.greyness,
            m.greyness_outer1,
            m.max_grad_l2,
            m.vf_loop,
            m.vf_export,
            m.min_xy_var_from_outer_18,
            m.xy_var,
            m.c0,
            m.c1,
            m.c1_peak,
            m.c1_peak_fixed_p3,
            m.c1_fixed_p3_loop_last,
            m.last_outer_beta,
            m.beta_step_count,
            m.eq_rel_res
        );
        if adam_iters == 60 && h4_diag_enabled() {
            assert!(
                m.beta_step_count >= 3,
                "60-outer schedule-regime: need ≥3 β steps (got {})",
                m.beta_step_count
            );
            assert!(
                m.max_grad_l2.is_finite() && m.max_grad_l2 < 500.0,
                "60-outer schedule-regime: grad_l2 bounded (<500); max_grad_l2={}",
                m.max_grad_l2
            );
            assert!(
                m.min_xy_var_from_outer_18.is_finite() && m.min_xy_var_from_outer_18 > 1e-6,
                "60-outer schedule-regime: xy_var alive past outer 18: min={}",
                m.min_xy_var_from_outer_18
            );
            assert!(
                m.min_xy_var_from_outer_50.is_finite() && m.min_xy_var_from_outer_50 > 1e-6,
                "60-outer schedule-regime: xy_var alive past outer 50: min={}",
                m.min_xy_var_from_outer_50
            );
            if m.greyness_at_beta_steps.len() >= 2 {
                let g0 = m.greyness_at_beta_steps[0];
                let g_last = *m.greyness_at_beta_steps.last().unwrap();
                assert!(
                    g_last < g0 - 1e-4,
                    "60-outer schedule-regime: greyness trending down across β steps: first={g0:.6} last={g_last:.6} trail={:?}",
                    m.greyness_at_beta_steps
                );
            }
            eprintln!(
                "shell_topology_rib_pattern_full_v04: 60-outer schedule-regime smoke PASS — \
beta_steps={} greyness@β_steps={:?} min_xy_var@18+={:.6} min_xy_var@50+={:.6} max_grad_l2={:.6}",
                m.beta_step_count,
                m.greyness_at_beta_steps,
                m.min_xy_var_from_outer_18,
                m.min_xy_var_from_outer_50,
                m.max_grad_l2
            );
        }
        return;
    }
    assert!(
        (m.vf_export - target_vf).abs() <= 0.01,
        "vf export gate: vf_export={} vf_loop={} target_vf={} (greyness={} xy_var={} c0={} c1={})",
        m.vf_export,
        m.vf_loop,
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
        m.c1 < m.c0_uniform_p_final * 0.6,
        "compliance drop gate (matched-p c0: uniform ρ @ p_final): c0_p_final={} c1={} ratio={} \
(p=1 audit c0_uniform={}; vf={} greyness={} xy_var={})",
        m.c0_uniform_p_final,
        m.c1,
        m.c1 / m.c0_uniform_p_final.max(1e-30),
        m.c0_uniform,
        m.vf,
        m.greyness,
        m.xy_var
    );
}

/// Prove `striatus_vf_bisect_guard` fires when in-loop b-bisect is skipped (bisection-failure tripwire).
#[test]
#[ignore = "synthetic pathology: UMST_SHELL_RIB_PATTERN=1 UMST_SHELL_SKIP_B_BISECT_OUTERS=60 UMST_SHELL_RIB_FULL_ITERS=25 UMST_SHELL_SELF_WEIGHT=1 cargo test ... shell_topology_rib_pattern_vf_guard_synthetic_pathology --release -- --ignored --nocapture"]
#[should_panic(expected = "striatus_vf_bisect_guard")]
fn shell_topology_rib_pattern_vf_guard_synthetic_pathology() {
    assert_eq!(
        env::var("UMST_SHELL_RIB_PATTERN").as_deref(),
        Ok("1"),
        "set UMST_SHELL_RIB_PATTERN=1"
    );
    env::set_var("UMST_SHELL_SKIP_B_BISECT_OUTERS", "60");
    env::set_var("UMST_SHELL_RIB_FULL_ITERS", "25");
    env::set_var("UMST_SHELL_SELF_WEIGHT", "1");
    let _ = run_rib_full_striatus(parse_target_vf());
}

/// Thesis re-config: 0.3 m / nz=8 / vf≈0.30 / non-design solid skin (see `outputs/.plans/b6-thesis-reconfig.md`).
#[test]
#[ignore = "thesis re-config: UMST_SHELL_THESIS_RECONFIG=1 UMST_SHELL_RIB_PATTERN=1 UMST_SHELL_VF=0.30 cargo test ... shell_topology_rib_pattern_thesis_reconfig --release -- --ignored --nocapture"]
fn shell_topology_rib_pattern_thesis_reconfig() {
    assert_eq!(
        env::var("UMST_SHELL_RIB_PATTERN").as_deref(),
        Ok("1"),
        "set UMST_SHELL_RIB_PATTERN=1"
    );
    env::set_var("UMST_SHELL_THESIS_RECONFIG", "1");
    if env::var("UMST_SHELL_VF").is_err() {
        env::set_var("UMST_SHELL_VF", "0.30");
    }
    let target_vf = parse_target_vf();
    let adam_iters = parse_full_rib_adam_iters();
    let m = run_rib_full_striatus(target_vf);
    let (_gl, _xyl, _b0, _jit, _rib) = parse_umst_shell_b6_aux_env();
    let g_uni = 4.0 * target_vf * (1.0 - target_vf);
    eprintln!(
        "shell_topology_rib_pattern_thesis_reconfig: pre-gate metrics \
GREYNESS(4ρ(1−ρ))={:.6} vf={:.6} target_vf={:.4} vf_err={:+.6} \
xy_var_z_avg={:.6} c0={:.6} c1={:.6} beta_last={:.3} \
max_grad_l2={:.6} last_grad_l2={:.6} g_uni=4·vf·(1−vf)={:.6} pcg_iter_final={} pcg_rel_res={:.3e} eq_rel_res={:.3e} adam_skipped={}/{} \
last_outer_wall_ms={:.1} total_wall_s={:.3} seed={} backend_features={} \
h_c1_a_comp_frac={:.4} c0_uniform_p_final_raw={:.6} h_c1_a_se_frac={:.6}",
        m.greyness,
        m.vf,
        m.target_vf,
        m.vf - m.target_vf,
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
        m.last_outer_wall_ms,
        m.total_wall_s,
        m.seed,
        m.active_backend_features,
        m.h_c1_a_comp_frac,
        m.c0_uniform_p_final_raw,
        m.h_c1_a_se_frac,
    );
    eprintln!(
        "thesis_reconfig: nz=8 lz=0.3 vf_target={target_vf:.2} vf_final={:.4} c0={:.4} c1={:.4} c0_uniform_p_final_raw={:.6} h_c1_a_se_frac={:.6}",
        m.vf, m.c0_uniform_raw, m.c1, m.c0_uniform_p_final_raw, m.h_c1_a_se_frac
    );
}
