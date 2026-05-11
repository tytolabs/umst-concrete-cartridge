// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Striatus-class concrete shell topology optimisation (extruded plate + SIMP).
//!
//! Optimisation loss uses **discrete-adjoint compliance** ([`AdjointCompliance`]): equilibrium PCG runs on
//! the inner (non-AD) backend while sensitivities follow the SIMP bar-network surrogate. Pseudo-density
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
//! Optional per-iteration dumps: set `UMST_SHELL_DUMP_ITER=1` (writes large gitignored `iter_*.npy`).
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
//! formal_anchor: Literature
//! formal_citation: Sigmund & Maute 2013, Struct. Multidisc. Optim. 48:1031-1055
//! formal_form: Neural-SIMP topology optimisation on an extruded \(40^2\times4\) hex slab under self-weight + roof pressure

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

fn build_top_pressure_load(nx: usize, ny: usize, nz: usize, pa: f32, dx: f32, dy: f32) -> Vec<f32> {
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
    let live_flat = build_top_pressure_load(nx, ny, nz, pressure_pa, dx, dy);
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
        cg_tolerance: 1e-5,
        pcg_tolerance: 1e-5,
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
        .unwrap_or(1usize)
        .max(1usize);

    let sym_on = env::var("UMST_SHELL_SYMMETRY")
        .map(|v| v != "0")
        .unwrap_or(true);
    let sym_period = env::var("UMST_SHELL_SYMM_PERIOD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20usize);
    let partners = reflection_xy_partner_indices::<B>(nx, ny, nz, &device);

    let helm_on = env::var("UMST_SHELL_HELM")
        .map(|v| v != "0")
        .unwrap_or(false);
    let vol_in_loop = env::var("UMST_SHELL_VOL_LOOP")
        .map(|v| v != "0")
        .unwrap_or(true);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let art_dir = manifest_dir.join("examples/_artifacts/shell");
    let _ = fs::create_dir_all(&art_dir);

    let manifest = format!(
        r#"{{"nx":{nx},"ny":{ny},"nz":{nz},"lx":{lx},"ly":{ly},"lz":{lz},"dx":{dx},"dy":{dy},"dz":{dz}}}"#
    );
    fs::write(art_dir.join("manifest.json"), manifest).expect("write manifest.json");

    println!(
        "optimize_shell_3d: grid {}×{}×{} cells, {} nodes, {} iterations, self_weight={}, vol_in_loop={}",
        nx, ny, nz, n, iterations, use_self_weight, vol_in_loop
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
        let (surrogate, c_raw) = AdjointCompliance::forward_and_loss(
            rho_bar.clone(),
            edges_inner.clone(),
            coords_n3_inner.clone(),
            boundary_inner.clone(),
            bf.inner(),
            damage_z.clone(),
            simp_mat,
            &cg_cfg,
            cross_section_area,
        );

        if it == 1 {
            comp_scale = c_raw.max(1e-12);
        }
        let compliance = surrogate.div_scalar(comp_scale);
        let total_loss = compliance.clone();

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

    let rho_final = opt
        .density_net
        .forward_batched(coords_norm.clone())
        .clamp(1e-6, 1.0 - 1e-6);
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
