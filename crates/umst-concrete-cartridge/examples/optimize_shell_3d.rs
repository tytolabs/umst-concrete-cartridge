// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Striatus-class concrete shell topology optimisation (extruded plate + SIMP).
//!
//! Optimisation loss uses **discrete-adjoint compliance**: default **[`AdjointCompliance`]** (bar-network
//! surrogate); set **`UMST_SHELL_ADJOINT_KIND=q1_hex`** for
//! [`AdjointComplianceQ1Hex`](umst_manifold::physics::adjoint_q1_hex::AdjointComplianceQ1Hex)
//! (Q1-hex continuum surrogate on the same extruded brick; enabled with **`solver-experimental`**).
//! Equilibrium runs on the inner (non-AD) backend while sensitivities follow the SIMP law. Pseudo-density
//! pipeline: optional **Helmholtz** (`UMST_SHELL_HELM=1`; default **off** — 240 Richardson steps on the Burn
//! Autodiff tape can destabilise `f32` on large 3-D slabs; manifold `HelmholtzFilter` no longer forces a 400-iter
//! floor over the constructor count) → Heaviside (\(\beta\) via [`BetaContinuation`]) → optional in-loop
//! [`VolumeProjection`] (`UMST_SHELL_VOL_LOOP=0` skips in-loop projection; default **on** so mean VF tracks
//! `UMST_SHELL_VF` and bar stiffness stays well-posed; terminal export still runs projection on the inner backend).
//! SIMP exponent \(p\) follows [`ContinuationSchedule`]. Final `final_sigma.npy` still comes from a plain
//! [`ExtrudedPlateMechanics::solve_equilibrium`] pass (no AD on that path).
//! Writes `examples/_artifacts/shell/final.npy` (`float32`, shape `[1, N, 1]`) — same tensor layout
//! consumed by the Python render/export pipeline (`render_shell_gif.py`, `export_print_ready.py`).
//! Also writes `final_sigma.npy` (`float32`, shape `[1, N, 6]`, Voigt order
//! \([\sigma_{xx},\sigma_{yy},\sigma_{zz},\sigma_{xy},\sigma_{yz},\sigma_{xz}]\)) for
//! `overlay_final_isostatics.py` when present.
//! Optional per-iteration dumps: set **`UMST_SHELL_DUMP_ITER=1`** (writes large gitignored **`iter_*.npy`**).
//! When dumps are on, **`UMST_SHELL_DUMP_STRIDE`** defaults to **10** (first, every **N**th, and last outer —
//! same default as **`notebooks/_run_shell_demo.sh`**) so a **40×40×4** / **200**-outer run does not write **200** full fields.
//! **Self-weight:** default **off** (roof traction only — matches `shell_demo_smoke` and avoids **non-finite**
//! bar PCG in `f32` on large grids). Set **`UMST_SHELL_SELF_WEIGHT=1`** for gravity body force.
//! **40×40×4 Striatus grid:** bar-network PCG in `packed_bar_network_equilibrium` runs up to
//! **`min(max_cg_iterations, 3N)`** iterations and **early-exits** when the projected residual
//! \(\|P(f-Ku)\|_2 \le \max(\texttt{pcg\_tolerance},\texttt{cg\_tolerance})\,\|Pf\|_2\) (see manifold
//! `mechanics.rs`). Match **Track B6** full harness (`shell_topology_rib_pattern` `#!`): **`E_min = 10⁻³·E₀`**
//! and **`max_cg_iterations = 2000`** (defaults here); a too-small CG budget with **`E_min = 1`** Pa vs
//! **`E₀ = 200`** GPa caused **non-finite** compliance at iter 1. Override PCG budget with **`UMST_SHELL_MAX_CG`**;
//! void floor with **`UMST_SHELL_E_MIN_REL`** (fraction of `e0`).
//!
//! **Bounded grid caps (env):** **`UMST_SHELL_NX` / `NY` / `NZ`** clamp to **`[6, 40]`** / **`[6, 40]`** / **`[2, 8]`**
//! cells; **`UMST_SHELL_ITERS`** to **`[1, 500]`**. Track L / B8 proof on the **40×40×4** lattice still expects
//! **`UMST_SHELL_ITERS=200`** (see `notebooks/_run_shell_demo.sh`). **Overnight Rust-only (explicit 40×40×4, log):** from **`umst-concrete-cartridge/`**, after removing stale **`iter_*.npy`**: `UMST_SHELL_NX=40 UMST_SHELL_NY=40 UMST_SHELL_NZ=4 UMST_SHELL_ITERS=200 UMST_SHELL_DUMP_ITER=0 cargo run --release -p umst-concrete-cartridge --example optimize_shell_3d --features 'solver-experimental render' 2>&1 | tee shell_opt_40cube4_i200.log` — **`bash notebooks/_run_shell_demo.sh`** re-runs the Rust binary then GIF/STL/JSON; to post-process an existing **`final.npy`** only, invoke **`render_shell_gif.py`** … **`export_print_ready.py`** in order instead. For a **fast wall-clock** check that
//! **`export_print_ready.py`** accepts the field (**ρ span ≥ 1e⁻³**), use a smaller grid, e.g.
//! **`UMST_SHELL_NX=16 UMST_SHELL_NY=16 UMST_SHELL_NZ=4 UMST_SHELL_ITERS=40`** (~minutes on a laptop CPU in
//! `--release`); that does **not** by itself satisfy Ring‑1 B8 topology gates on the full Striatus lattice.
//! **Track B8 rollup:** `export_print_ready.py` writes **`gates_track_b8_all_pass`** into **`notebooks/_artifacts/striatus_shell_v0.4.print_ready.json`** (see **`docs/Solver-Status.md`** — P0 / Topology shell).
//! Optional outer-loss weights **`UMST_SHELL_GREY_LAMBDA`** / **`UMST_SHELL_XY_VAR_LAMBDA`**: both auxiliary
//! terms are evaluated on **`ρ_mid`** (post-Heaviside, **pre–volume projection**) so gradients are not
//! blocked by the bisection in [`VolumeProjection`]; B8 **`density_xy_plane_variance`** is still measured
//! on the exported **post–projection** lattice in Python.
//! **`UMST_SHELL_XY_RIB_PRIOR_AMP`** / **`UMST_SHELL_DENSITY_INIT_JITTER`**: same semantics as the rib harness
//! (`tests/shell_topology_rib_pattern.rs`). On the **40×40×4** Striatus lattice, **`UMST_SHELL_XY_RIB_PRIOR_AMP`**
//! defaults to **0.12** when unset (override with **`0`** to disable); smaller grids default **0** unless set.
//!
//! **Final artefact vs last training step (XY symmetry):** when **`UMST_SHELL_SYMMETRY`** is on (default), the optimiser
//! averages **ρ** over the four XY mirror partners every **`UMST_SHELL_SYMM_PERIOD`** outers (default **20**).
//! **`final.npy`** then applies the **same XY reflection average once** after the terminal forward (before
//! Helmholtz / projection / stress export). GIF frames built from **`iter_*.npy`** show **in-loop** **ρ** (mirrored
//! only on sym-period steps), so the last animation frame can differ slightly from **`final.npy`** until you
//! rely on **`final.npy`** / **`export_print_ready.py`** for Track L — use **`manifest.json`** (`symmetry_xy`,
//! `sym_period`, …) to record what ran. **`UMST_SHELL_SYMMETRY=0`** disables both in-loop mirroring and the terminal average.
//!
//! formal_anchor: Literature
//! formal_citation: Sigmund & Maute 2013, Struct. Multidisc. Optim. 48:1031-1055
//! formal_form: Neural-SIMP topology optimisation on an extruded \(40^2\times4\) hex slab under self-weight + roof pressure

use std::cell::Cell;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use burn::backend::Autodiff;
use burn::module::{Module, ModuleMapper, ParamId};
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::{
    backend::{AutodiffBackend, Backend as BackendTrait},
    Data, Shape, Tensor,
};
use burn_ndarray::NdArray;
use umst_manifold::ai::topology::{
    BetaContinuation, ContinuationSchedule, HeavisideProjection, TopologyOptimizer,
    VolumeProjection,
};
use umst_manifold::physics::adjoint::{AdjointCompliance, SimpElasticMaterial};
use umst_manifold::physics::adjoint_q1_hex::AdjointComplianceQ1Hex;
use umst_manifold::physics::extruded_plate::{ElasticMaterial, ExtrudedPlateMechanics};
use umst_manifold::physics::mechanics::SelfWeightConfig;
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

/// Construct a `TopologyOptimizer` with weights scaled down by `scale` from Burn's default Kaiming init.
/// `scale = 0.0` yields identically-zero weights (dead-ReLU; useful only for byte-identical reproducibility);
/// small non-zero values (e.g. 0.05) preserve symmetry breaking while bounding pre-sigmoid logits.
fn topology_optimizer_scaled(
    volume_target: f32,
    penalization: f32,
    hidden_dim: usize,
    scale: f32,
    device: &<NdArray<f32> as BackendTrait>::Device,
) -> TopologyOptimizer<B> {
    let mut opt = TopologyOptimizer::new(volume_target, penalization, hidden_dim, device);
    let mut s = ScaleWeights(scale);
    opt.density_net = opt.density_net.map(&mut s);
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

/// Volume mean of **`4ρ(1−ρ)`** on **`ρ`** `[1,N,1]` (caller passes **`ρ_mid`** for differentiable greyness).
fn mean_greyness_tensor<Bk: AutodiffBackend<FloatElem = f32>>(
    rho_bar: Tensor<Bk, 3>,
) -> Tensor<Bk, 1> {
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

/// Z-stacked mean per `(x,y)` column, then variance over `(nx+1)(ny+1)` columns — same statistic as
/// [`export_print_ready::density_xy_plane_variance`] / `shell_topology_rib_pattern::xy_plane_variance`.
fn xy_plane_variance_z_avg_tensor<Bk: AutodiffBackend<FloatElem = f32>>(
    rho_bar: Tensor<Bk, 3>,
    nx: usize,
    ny: usize,
    nz: usize,
) -> Tensor<Bk, 1> {
    let nx1 = nx + 1;
    let ny1 = ny + 1;
    let nz1 = nz + 1;
    let [b, n, c] = rho_bar.dims();
    assert_eq!((b, c), (1, 1));
    assert_eq!(n, nx1 * ny1 * nz1);
    let nz_f = nz1 as f32;
    let nxy = (nx1 * ny1) as f32;
    let t = rho_bar.reshape([nz1, nx1 * ny1]);
    let mz = t.sum_dim(0).div_scalar(nz_f).reshape([nx1 * ny1]);
    let sum = mz.clone().sum();
    let sumsq = mz.powf_scalar(2.0).sum();
    let mean_sq = sumsq.div_scalar(nxy);
    let mean = sum.div_scalar(nxy);
    mean_sq.sub(mean.powf_scalar(2.0)).reshape([1])
}

fn pin_four_corners_bm(
    nx: usize,
    ny: usize,
    nz: usize,
    device: &<NdArray<f32> as BackendTrait>::Device,
) -> Tensor<B, 3> {
    pin_bottom_perimeter_bm(nx, ny, nz, device)
}

/// Pin every bottom-edge node (`z = 0` perimeter) in all three directions — Striatus-style continuous edge support.
fn pin_bottom_perimeter_bm(
    nx: usize,
    ny: usize,
    nz: usize,
    device: &<NdArray<f32> as BackendTrait>::Device,
) -> Tensor<B, 3> {
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

/// Roof nodal traction with optional **x** ramp \(w = 1 + r\,i_x/n_x\) (same lumping as `shell_topology_rib_pattern`).
/// Uniform roof pressure: **`UMST_SHELL_ROOF_RAMP=0`**. Default is **on** — **`UMST_SHELL_ROOF_RAMP_F`** (default **0.2**)
/// to bias sensitivities in **x** when short runs on small slabs would otherwise stay nearly symmetric / flat in XY.
fn build_top_pressure_load_ramp(
    nx: usize,
    ny: usize,
    nz: usize,
    pa: f32,
    dx: f32,
    dy: f32,
    ramp: f32,
) -> Vec<f32> {
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
    bf
}

/// NumPy `.npy` v1.0 writer for C-contiguous `float32` payload.
fn write_npy_f32(path: &std::path::Path, data: &[f32], shape: &[usize]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let shape_lit = shape
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let dict = format!("{{'descr': '<f4', 'fortran_order': False, 'shape': ({shape_lit}), }}");
    let mut pad = dict.into_bytes();
    while (pad.len() + 10) % 64 != 0 {
        pad.push(b' ');
    }
    pad.push(b'\n');
    let header_len = pad.len();
    if header_len > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "npy header too large",
        ));
    }
    let mut out = Vec::with_capacity(10 + header_len + data.len() * 4);
    out.extend_from_slice(b"\x93NUMPY");
    out.extend_from_slice(&[1, 0]);
    out.extend_from_slice(&(header_len as u16).to_le_bytes());
    out.extend_from_slice(&pad);
    for v in data {
        out.extend_from_slice(&v.to_le_bytes());
    }
    fs::write(path, out)
}

fn main() {
    let device = Default::default();
    // Match `shell_topology_rib_pattern_full_v04` for reproducible first-iter numerics (unseeded MLP + AD
    // Helmholtz can hit NaNs on large slabs).
    <B as BackendTrait>::seed(42);
    let nx = env::var("UMST_SHELL_NX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40usize)
        .clamp(6, 40);
    let ny = env::var("UMST_SHELL_NY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40usize)
        .clamp(6, 40);
    let nz = env::var("UMST_SHELL_NZ")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4usize)
        .clamp(2, 8);

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
    let coords_bn3 = plate.coords_bn3::<B>(&device);
    let coord_scale = lx.max(ly).max(lz);
    let coords_norm = coords_bn3
        .clone()
        .div_scalar(coord_scale)
        .mul_scalar(2.0)
        .sub_scalar(1.0);
    let edges_b1 = plate.edges_b1::<B>(&device);

    let boundary_mask = pin_four_corners_bm(nx, ny, nz, &device);

    let pressure_pa = 50.0_f32;
    // Match module docs + `shell_topology_rib_pattern`: ramp on unless explicitly disabled (`=0`).
    let roof_ramp_on = env::var("UMST_SHELL_ROOF_RAMP")
        .map(|v| v != "0")
        .unwrap_or(true);
    let roof_ramp_f = env::var("UMST_SHELL_ROOF_RAMP_F")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.2_f32);
    let ramp = if roof_ramp_on { roof_ramp_f } else { 0.0_f32 };
    let live_flat = build_top_pressure_load_ramp(nx, ny, nz, pressure_pa, dx, dy, ramp);
    let live_force = Tensor::from_data(Data::new(live_flat, Shape::new([1, n, 3])), &device);

    let voxel_vol = dx * dy * dz;
    let sw_cfg = SelfWeightConfig {
        gravity_m_s2: 9.81,
        voxel_volume_m3: voxel_vol,
        mass_penalty_q: 1.0,
        direction: [0.0, 0.0, -1.0],
    };
    let use_self_weight = env::var("UMST_SHELL_SELF_WEIGHT")
        .map(|v| v != "0")
        .unwrap_or(false);

    let helm = HelmholtzFilter::new((2.0 * dx.min(dy).min(dz)).max(1e-6), 240, 1e-7);
    let mut proj = HeavisideProjection::new(1.0, 0.5);
    let target_vf = env::var("UMST_SHELL_VF")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.15_f32);
    let vol_proj = VolumeProjection::new(target_vf, 48);

    let e0 = 200e6_f32;
    let e_min_rel = env::var("UMST_SHELL_E_MIN_REL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1e-3_f32)
        .clamp(1e-9, 1.0);
    let material = ElasticMaterial {
        e0,
        nu: 0.2,
        simp_p: 3.0,
        e_min: e0 * e_min_rel,
    };
    let use_pc = env::var("UMST_SHELL_PCG").map(|v| v != "0").unwrap_or(true);
    let max_cg = env::var("UMST_SHELL_MAX_CG")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000usize)
        .clamp(50, 50_000);
    let cg_cfg = MechanicsInnerLoopConfig {
        max_cg_iterations: max_cg,
        // Match `shell_topology_rib_pattern` full harness (`run_rib_full_striatus`) — slightly tighter than
        // 1e-5 to reduce spurious non-finite adjoint compliance on stiff 40×40×4 slabs.
        cg_tolerance: 1e-6,
        pcg_tolerance: 1e-6,
        use_preconditioner: use_pc,
        max_equilibrium_substeps: 1,
    };

    let edges_inner = plate.edges_b1::<Inner>(&device);
    let coords_n3_inner = plate
        .coords_bn3::<Inner>(&device)
        .reshape(Shape::new([n, 3]));
    let boundary_inner = boundary_mask.clone().inner();
    let damage_z = Tensor::<Inner, 3>::zeros([1, n, 1], &device);
    let cross_section_area = voxel_vol.cbrt().powf(2.0);

    let init_scale = env::var("UMST_SHELL_INIT_SCALE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.05_f32);
    let mut opt = topology_optimizer_scaled(target_vf, 3.0, 64, init_scale, &device);
    let density_init_jitter = env::var("UMST_SHELL_DENSITY_INIT_JITTER")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0_f32)
        .clamp(0.0, 0.25);
    if density_init_jitter > 0.0 {
        let mut jm = AddDensityInitJitter {
            amplitude: density_init_jitter,
            idx: Cell::new(0),
        };
        opt.density_net = opt.density_net.map(&mut jm);
    }
    let mut adam = AdamConfig::new().init::<B, _>();

    let dx_f = dx.min(dy).min(dz);

    let iterations = env::var("UMST_SHELL_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200usize)
        .clamp(1, 500);

    let dump_iter = env::var("UMST_SHELL_DUMP_ITER")
        .map(|v| v == "1")
        .unwrap_or(false);
    let dump_stride = env::var("UMST_SHELL_DUMP_STRIDE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10usize)
        .max(1usize);

    let sym_on = env::var("UMST_SHELL_SYMMETRY")
        .map(|v| v != "0")
        .unwrap_or(true);
    let sym_period = env::var("UMST_SHELL_SYMM_PERIOD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20usize);
    // Optional outer-loss terms (same semantics as `run_rib_full_striatus`): post–volume-projection ρ.
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
    let xy_rib_prior_amp = match env::var("UMST_SHELL_XY_RIB_PRIOR_AMP") {
        Ok(s) if !s.trim().is_empty() => s.parse::<f32>().unwrap_or(0.0).clamp(0.0, 0.25),
        _ => {
            if nx == 40 && ny == 40 && nz == 4 {
                0.12_f32
            } else {
                0.0_f32
            }
        }
    };
    let partners = reflection_xy_partner_indices::<B>(nx, ny, nz, &device);
    let xy_rib_pat = if xy_rib_prior_amp > 0.0 {
        Some(xy_rib_prior_pattern_b(nx, ny, nz, &device))
    } else {
        None
    };

    // Only `UMST_SHELL_HELM=1` enables Helmholtz; `UMST_SHELL_HELM=` yields `Ok("")`, which must not enable.
    let helm_on = matches!(env::var("UMST_SHELL_HELM").as_deref(), Ok("1"));
    let vol_in_loop = env::var("UMST_SHELL_VOL_LOOP")
        .map(|v| v != "0")
        .unwrap_or(true);
    let use_q1_hex_adjoint = env::var("UMST_SHELL_ADJOINT_KIND")
        .map(|s| s.eq_ignore_ascii_case("q1_hex"))
        .unwrap_or(false);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let art_dir = manifest_dir.join("examples/_artifacts/shell");
    let _ = fs::create_dir_all(&art_dir);

    println!(
        "optimize_shell_3d: grid {}×{}×{} cells, {} nodes, {} iterations, adjoint={}, self_weight={}, vol_in_loop={}, roof_x_ramp={}, grey_λ={}, xy_var_λ={}, xy_rib_prior_amp={}, density_init_jitter={}",
        nx,
        ny,
        nz,
        n,
        iterations,
        if use_q1_hex_adjoint { "q1_hex" } else { "bar" },
        use_self_weight,
        vol_in_loop,
        roof_ramp_on,
        grey_lambda,
        xy_var_lambda,
        xy_rib_prior_amp,
        density_init_jitter
    );

    let mut last_loss = f32::NAN;
    let mut comp_scale: f32 = 0.0;
    let iter_total = iterations.max(1);

    for it in 1..=iterations {
        let beta = BetaContinuation::beta(it.saturating_sub(1), iter_total, 1.0, 32.0);
        proj.set_beta(beta);

        let mut rho_raw = opt.density_net.forward_batched(coords_norm.clone());
        if sym_on && sym_period > 0 && it % sym_period == 0 {
            rho_raw = apply_reflection_xy_average(rho_raw, &partners);
        }
        if let Some(pat) = &xy_rib_pat {
            rho_raw = rho_raw
                .add(pat.clone().mul_scalar(xy_rib_prior_amp))
                .clamp(0.0, 1.0);
        }
        // `UMST_SHELL_HELM=1` enables graph filter on the AD tape (can diverge in `f32` at Striatus scale).
        let rho_tilde = if helm_on {
            helm.apply(rho_raw.clone(), edges_b1.clone(), dx_f)
        } else {
            rho_raw.clone()
        };
        let rho_mid = proj.project(rho_tilde);
        let rho_bar = if vol_in_loop {
            vol_proj.project(rho_mid.clone())
        } else {
            rho_mid.clone()
        };

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
        let (surrogate, c_raw) = if use_q1_hex_adjoint {
            AdjointComplianceQ1Hex::forward_and_loss(
                rho_bar.clone(),
                nx,
                ny,
                nz,
                dx,
                dy,
                dz,
                bf.inner(),
                boundary_inner.clone(),
                simp_mat,
                &cg_cfg,
                if use_self_weight { Some(sw_cfg) } else { None },
            )
        } else {
            AdjointCompliance::forward_and_loss(
                rho_bar.clone(),
                edges_inner.clone(),
                coords_n3_inner.clone(),
                boundary_inner.clone(),
                bf.inner(),
                damage_z.clone(),
                simp_mat,
                &cg_cfg,
                cross_section_area,
            )
        };

        if it == 1 {
            comp_scale = c_raw.max(1e-12);
        }
        let compliance = surrogate.div_scalar(comp_scale);
        let mut total_loss = compliance.clone();
        if grey_lambda > 0.0 {
            let grey_t = mean_greyness_tensor(rho_mid.clone());
            total_loss = total_loss.add(grey_t.mul_scalar(grey_lambda));
        }
        if xy_var_lambda > 0.0 {
            // Use `rho_mid` (post-Heaviside, pre–volume projection): variance on `rho_bar` collapses
            // toward ~0 when the projector pins mean VF, starving the rib term of gradient on 40×40×4.
            let v_xy = xy_plane_variance_z_avg_tensor(rho_mid.clone(), nx, ny, nz);
            total_loss = total_loss.sub(v_xy.mul_scalar(xy_var_lambda));
        }

        let vf = rho_bar.clone().mean().into_scalar();

        let loss_scalar = total_loss.clone().into_data().value[0];
        let comp_scalar = c_raw / comp_scale;
        if it > 1
            && loss_scalar.is_finite()
            && last_loss.is_finite()
            && loss_scalar > last_loss + 1e-3
        {
            println!("warn: loss rose at iter {it}: {loss_scalar} (prev {last_loss})");
        }
        if loss_scalar.is_finite() {
            last_loss = loss_scalar;
        }

        if dump_iter && (it == 1 || it % dump_stride == 0 || it == iterations) {
            let fname = format!("iter_{it:03}.npy");
            let path = art_dir.join(fname);
            let v = rho_bar.clone().into_data().value;
            write_npy_f32(&path, &v, &[1, n, 1]).expect("write iter npy");
        }

        if it == 1 || it % 20 == 0 || it == iterations {
            let beta = proj.beta();
            println!(
                "iter {it:03} | loss={loss_scalar:.6} | compliance={comp_scalar:.6} | vf={vf:.4} | beta={beta:.2}",
            );
        }

        // Guard: skip backward if loss is NaN (avoids autodiff crash on degenerate graphs).
        if loss_scalar.is_nan() || loss_scalar.is_infinite() {
            println!("warn: skipping backward at iter {it} (loss={loss_scalar})");
            continue;
        }

        let grads = total_loss.backward();
        let grads_params = GradientsParams::from_grads(grads, &opt.density_net);
        opt.density_net = adam.step(0.005, opt.density_net, grads_params);
    }

    let mut rho_final = opt
        .density_net
        .forward_batched(coords_norm.clone())
        .clamp(1e-6, 1.0 - 1e-6)
        .reshape([1, n, 1]);
    if sym_on {
        rho_final = apply_reflection_xy_average(rho_final, &partners);
    }
    if let Some(pat) = &xy_rib_pat {
        rho_final = rho_final
            .add(pat.clone().mul_scalar(xy_rib_prior_amp))
            .clamp(0.0, 1.0);
    }
    let rho_tilde = if helm_on {
        helm.apply(rho_final.clone(), edges_b1.clone(), dx_f)
    } else {
        rho_final
    };
    let rho_mid = proj.project(rho_tilde);
    // Volume projection on inner only: no AD backward on bisection (export-only when `vol_in_loop` is false).
    let rho_out_inner = vol_proj.project(rho_mid.inner());
    let bytes = rho_out_inner.clone().into_data().value;
    let final_path = art_dir.join("final.npy");
    write_npy_f32(&final_path, &bytes, &[1, n, 1]).expect("write final.npy");

    let body_force_inner = if use_self_weight {
        sw_cfg
            .body_force(rho_out_inner.clone())
            .add(live_force.clone().inner())
    } else {
        live_force.clone().inner()
    };

    let (_u_post, sigma_post) = plate.solve_equilibrium(
        rho_out_inner,
        body_force_inner,
        boundary_mask.clone().inner(),
        material,
        &cg_cfg,
    );
    let sig_bytes = sigma_post.into_data().value;
    let sigma_path = art_dir.join("final_sigma.npy");
    write_npy_f32(&sigma_path, &sig_bytes, &[1, n, 6]).expect("write final_sigma.npy");

    let sym_json = if sym_on { "true" } else { "false" };
    let roof_json = if roof_ramp_on { "true" } else { "false" };
    let sw_json = if use_self_weight { "true" } else { "false" };
    let vol_json = if vol_in_loop { "true" } else { "false" };
    let dump_json = if dump_iter { "true" } else { "false" };
    // Track-L sidecar JSON: hydrate with [`crate::facade::manifest::UmstManifest`] when serde typing is wired
    // (`cargo --features manifest-bridge` once `tytolabs/umst-manifold` exports `umst_manifold::manifest::UmstManifest`).
    let manifest = format!(
        r#"{{"nx":{nx},"ny":{ny},"nz":{nz},"lx":{lx},"ly":{ly},"lz":{lz},"dx":{dx},"dy":{dy},"dz":{dz},"burn_seed":42,"iters":{iterations},"symmetry_xy":{sym_json},"sym_period":{sym_period},"roof_x_ramp":{roof_json},"self_weight":{sw_json},"vol_in_loop":{vol_json},"dump_iter":{dump_json},"dump_stride":{dump_stride},"density_init_jitter":{density_init_jitter},"xy_rib_prior_amp":{xy_rib_prior_amp}}}"#
    );
    fs::write(art_dir.join("manifest.json"), manifest).expect("write manifest.json");

    println!(
        "wrote {} (float32 [1, {}, 1]) — Python contract for Striatus pipeline",
        final_path.display(),
        n
    );
    println!(
        "wrote {} (float32 [1, {}, 6]) — Voigt stress for isostatic overlay",
        sigma_path.display(),
        n
    );
}
