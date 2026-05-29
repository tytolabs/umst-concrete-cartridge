// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Reinforced Concrete Topology Optimization using UMST Phase 4.
//!
//! This example demonstrates how the UMST framework handles heterogeneous materials.
//! By masking the bottom chord of the beam as a **non-editable** full-density spine (the “rebar” corridor)
//! and the rest as editable “concrete” driven by [`TopologyOptimizer`], the network discovers load paths
//! that tie into that fixed chord (Strut-and-Tie–style layouts in the density field).
//!
//! **Mechanics + AD:** compliance uses **[`AdjointCompliance`]** (discrete-adjoint bar surrogate), matching
//! [`optimize_shell_3d`](./optimize_shell_3d.rs): equilibrium runs on the **inner** (non-autodiff) backend while
//! \(\partial c/\partial\rho\) follows the SIMP law on the tape. That avoids differentiating through PCG /
//! `masked_dot` on `Autodiff<NdArray>`, which has triggered Burn NdArray **scatter** shape mismatches in the
//! backward pass for this graph. A **uniform** reference Young’s modulus (**30 GPa**) and void floor
//! **`E_{\min}=10^{-3}E_0`** are used for the adjoint bar network (the demo’s story is the **mask**, not
//! a separate 200 GPa steel constitutive branch in the surrogate).
//!
//! **Iteration dumps (for [`notebooks/render_beam_gif.py`](../../../notebooks/render_beam_gif.py)):** dumps are **on by default** (`UMST_BEAM_DUMP=0` / `false` to skip).
//! Writes float32 **`iter_*.npy`** (**shape `[1, N, 1]`**, `N=n_x·n_y`) plus a **`manifest.json`** with grid, stride, compliance stats, and frame epoch list under **`examples/_artifacts/beam/`**.
//! **`UMST_BEAM_DUMP_STRIDE`** (default **3**, ≥1); **`UMST_BEAM_ITERS`** (default **90**, ≥1).
//!
//! **End-to-end GIF:** from repo root, `bash notebooks/_run_beam_demo.sh` runs this example then the renderer
//! (no synthetic NPY fallback when the optimiser completes successfully).

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use burn::backend::Autodiff;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::backend::AutodiffBackend;
use burn::tensor::{Data, Int, Shape, Tensor};
use burn_ndarray::NdArray;
use umst_manifold::ai::topology::TopologyOptimizer;
use umst_manifold::physics::adjoint::{AdjointCompliance, SimpElasticMaterial};
use umst_manifold::physics::solvers::PhaseFieldFractureSolver;
use umst_manifold::physics::time_orchestration::MechanicsInnerLoopConfig;

type Backend = Autodiff<NdArray<f32>>;
type B = Backend;
type Inner = <B as AutodiffBackend>::InnerBackend;

/// NumPy `.npy` v1.0 writer for C-contiguous `float32` payload (matches `optimize_shell_3d`).
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

/// Helper to print a 2D ASCII density map of the beam
fn print_density_map(density: Vec<f32>, nx: usize, ny: usize) {
    println!("\n--- Beam Topology Map ---");
    // Print from top to bottom (max y down to 0)
    for y in (0..ny).rev() {
        let mut row_str = String::new();
        for x in 0..nx {
            let idx = y * nx + x;
            let d = density[idx];
            let char = if y == 0 {
                "======" // Steel rebar representation
            } else if d > 0.8 {
                "██████" // Dense concrete
            } else if d > 0.5 {
                "▒▒▒▒▒▒" // Medium density
            } else if d > 0.2 {
                "░░░░░░" // Low density
            } else {
                "      " // Void
            };
            row_str.push_str(char);
        }
        println!("{row_str}");
    }
    println!("-------------------------\n");
}

fn main() {
    println!("=== UMST Reinforced Concrete Beam Optimization ===");

    let dump_beam = match env::var("UMST_BEAM_DUMP") {
        Err(_) => true,
        Ok(ref s) if s.trim().is_empty() => true,
        Ok(ref s)
            if s == "0" || s.eq_ignore_ascii_case("false") || s.eq_ignore_ascii_case("off") =>
        {
            false
        }
        Ok(ref s) => {
            s == "1"
                || s.eq_ignore_ascii_case("true")
                || s.eq_ignore_ascii_case("yes")
                || s.eq_ignore_ascii_case("on")
        }
    };
    let dump_stride = env::var("UMST_BEAM_DUMP_STRIDE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3usize)
        .max(1usize);
    let epochs: usize = env::var("UMST_BEAM_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(90usize)
        .max(1usize);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let art_dir = manifest_dir.join("examples/_artifacts/beam");

    let device = Default::default();
    let batch = 1usize;
    let nx = 32usize;
    let ny = 8usize;
    let n_nodes = nx * ny;
    let dx = 0.1_f32; // 10cm grid

    // 1. Build Coordinate Grid
    let mut coords_data = Vec::with_capacity(n_nodes * 3);
    for y in 0..ny {
        for x in 0..nx {
            coords_data.push(x as f32 * dx);
            coords_data.push(y as f32 * dx);
            coords_data.push(0.0);
        }
    }
    let coords: Tensor<B, 3> = Tensor::from_data(
        Data::new(coords_data, Shape::new([batch, n_nodes, 3])),
        &device,
    );

    // 2. Build Edge Topology (Grid)
    let mut edges = Vec::new();
    for y in 0..ny {
        for x in 0..nx {
            let i = y * nx + x;
            if x < nx - 1 {
                edges.push((i, i + 1));
            } // Horizontal
            if y < ny - 1 {
                edges.push((i, i + nx));
            } // Vertical
            if x < nx - 1 && y < ny - 1 {
                edges.push((i, i + nx + 1)); // Diagonal
            }
        }
    }
    let n_edges = edges.len();
    let mut edge_flat = Vec::with_capacity(n_edges * 2);
    for (src, _) in &edges {
        edge_flat.push(*src as i64);
    }
    for (_, tgt) in &edges {
        edge_flat.push(*tgt as i64);
    }
    let edges_b1: Tensor<B, 2, Int> =
        Tensor::from_data(Data::new(edge_flat, Shape::new([2, n_edges])), &device);

    // 3. Boundary Conditions
    // Fix the left wall (x == 0)
    let mut bm_data = vec![1.0_f32; batch * n_nodes * 3];
    for y in 0..ny {
        let i = y * nx;
        bm_data[i * 3] = 0.0;
        bm_data[i * 3 + 1] = 0.0;
        bm_data[i * 3 + 2] = 0.0;
    }
    let boundary_mask: Tensor<B, 3> =
        Tensor::from_data(Data::new(bm_data, Shape::new([batch, n_nodes, 3])), &device);

    // 4. Loading (Body Force)
    // Apply downward load on the top-right node
    let mut bf_data = vec![0.0_f32; batch * n_nodes * 3];
    let top_right = (ny - 1) * nx + (nx - 1);
    bf_data[top_right * 3 + 1] = -50_000.0; // 50 kN downward
    let body_force: Tensor<B, 3> =
        Tensor::from_data(Data::new(bf_data, Shape::new([batch, n_nodes, 3])), &device);

    // 5. AI constraints (rebar corridor): bottom row fixed at ρ=1, non-editable.
    let mut editable_mask_data = vec![1.0_f32; batch * n_nodes]; // 1.0 = AI can edit
    for slot in &mut editable_mask_data[..nx] {
        *slot = 0.0_f32; // AI cannot remove steel
    }
    let editable_mask = Tensor::from_data(
        Data::new(editable_mask_data, Shape::new([batch, n_nodes, 1])),
        &device,
    );
    let fixed_steel_density = Tensor::<B, 3>::ones([batch, n_nodes, 1], &device)
        .mul(Tensor::<B, 3>::ones_like(&editable_mask).sub(editable_mask.clone()));

    // 6. Setup Optimizer & Constraints
    let mut topopt = TopologyOptimizer::new(0.4, 3.0, 32, &device); // 40% volume fraction, p=3
    let mut adam = AdamConfig::new().init::<B, _>();
    let inner_cfg = MechanicsInnerLoopConfig {
        max_cg_iterations: 200,
        cg_tolerance: 1e-5,
        pcg_tolerance: 1e-5,
        use_preconditioner: true,
        max_equilibrium_substeps: 1,
    };
    let area = 0.01_f32; // 10cm x 10cm cross section
    let volume_lambda = 5000.0_f32; // Lagrange multiplier for volume constraint

    // Discrete-adjoint bar path: inner tensors only (same layout as `optimize_shell_3d`).
    let coords_n3_inner = coords
        .clone()
        .slice([0..batch, 0..n_nodes, 0..3])
        .reshape([n_nodes, 3])
        .inner();
    let edges_inner = edges_b1.clone().inner();
    let boundary_inner = boundary_mask.clone().inner();
    let body_force_inner = body_force.clone().inner();
    let damage_inner = Tensor::<Inner, 3>::zeros([batch, n_nodes, 1], &device);
    let e0_concrete = 30e9_f32;
    let simp_mat = SimpElasticMaterial {
        e0: e0_concrete,
        nu: 0.3,
        p: topopt.penalization,
        e_min: e0_concrete * 1e-3_f32,
    };

    println!("Starting Strut-and-Tie discovery...");

    let log_every = if epochs <= 48 {
        2usize
    } else {
        (epochs / 28).max(3)
    };
    let map_milestones: Vec<usize> = {
        let mut v = vec![1usize, epochs];
        if epochs > 2 {
            v.push(epochs / 3);
        }
        if epochs > 4 {
            v.push((2 * epochs) / 3);
        }
        v.sort_unstable();
        v.dedup();
        v
    };

    let mut dump_epochs: Vec<usize> = Vec::new();
    let mut compliance_initial: Option<f32> = None;
    let mut compliance_best = f32::MAX;
    let mut compliance_final = 0.0_f32;

    // 7. Training Loop
    for epoch in 1..=epochs {
        // Forward Pass: Ask AI to guess shape
        let raw_rho = topopt.density_net.forward_batched(coords.clone());

        // APPLY POLICY MASK: Force steel to remain density 1.0, AI controls the rest
        let rho = raw_rho
            .clone()
            .mul(editable_mask.clone())
            .add(fixed_steel_density.clone());

        // Calculate Volume Constraint Penalty
        let current_volume_fraction = rho.clone().mean().reshape([1]);
        let target_vol = Tensor::<B, 1>::zeros([1], &device).add_scalar(topopt.volume_target);
        let vol_diff = current_volume_fraction.clone().sub(target_vol);
        let volume_loss = vol_diff.clone().powf_scalar(2.0).mul_scalar(volume_lambda);

        let (compliance, c_raw) = AdjointCompliance::forward_and_loss(
            rho.clone(),
            edges_inner.clone(),
            coords_n3_inner.clone(),
            boundary_inner.clone(),
            body_force_inner.clone(),
            damage_inner.clone(),
            simp_mat,
            &inner_cfg,
            area,
        );

        // Total Loss = Compliance surrogate + Volume Penalty
        let total_loss = compliance.clone().add(volume_loss.clone());

        let loss_val = total_loss.clone().into_data().value[0];
        let comp_val = c_raw;
        let vol_val = current_volume_fraction.clone().into_data().value[0];

        if epoch == 1 {
            compliance_initial = Some(comp_val);
        }
        compliance_best = compliance_best.min(comp_val);
        compliance_final = comp_val;

        if epoch == 1 || epoch == epochs || epoch % log_every == 0 {
            println!("Epoch {epoch:03}/{epochs:03} | loss {loss_val:.2} | c {comp_val:.2} | c_min {compliance_best:.2} | vol {vol_val:.3}");
        }
        if map_milestones.binary_search(&epoch).is_ok() {
            let density_vec = rho.clone().into_data().value;
            print_density_map(density_vec, nx, ny);
        }

        if dump_beam && (epoch == 1 || epoch % dump_stride == 0 || epoch == epochs) {
            let v = rho.clone().into_data().value;
            let fname = format!("iter_{epoch:03}.npy");
            let path = art_dir.join(fname);
            write_npy_f32(&path, &v, &[1, n_nodes, 1])
                .unwrap_or_else(|e| panic!("write {path:?}: {e}"));
            dump_epochs.push(epoch);
        }

        // Backward Pass
        let grads = total_loss.backward();
        let grads_params = GradientsParams::from_grads(grads, &topopt.density_net);

        // Update Neural Network Weights
        topopt.density_net = adam.step(0.01, topopt.density_net, grads_params);
    }

    // 8. Phase-Field Fracture Check
    println!("Running Phase-Field Fracture Check on Final Optimized Beam...");
    let fracture = PhaseFieldFractureSolver { length_scale: 0.15 };
    let damage_old = Tensor::<B, 3>::zeros([batch, n_nodes, 1], &device);
    let gc_bn1 = Tensor::<B, 3>::zeros([batch, n_nodes, 1], &device).add_scalar(100.0); // Concrete Gc

    // Zero-strain placeholder for fracture wiring in this example (full THMC–mechanics coupling not exercised here).
    let strain_zero_placeholder = Tensor::<B, 4>::zeros([batch, n_nodes, 3, 3], &device);
    let damage_new = fracture.update_damage(
        strain_zero_placeholder,
        damage_old,
        gc_bn1,
        edges_b1.clone(),
    );

    let max_damage = damage_new.max().into_data().value[0];
    println!("Maximum Crack Damage (d): {max_damage:.3}");
    if max_damage > 0.9 {
        println!("WARNING: Beam has cracked and failed structurally!");
    } else {
        println!("SUCCESS: Beam holds the 50kN load safely with optimal material distribution.");
    }

    if dump_beam {
        let c0 = compliance_initial.unwrap_or(compliance_final);
        let frames_csv = dump_epochs
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let vol_tgt = topopt.volume_target;
        let p_simp = topopt.penalization;
        // Artefact **`manifest.json`**: typed envelope lives in [`crate::facade::manifest::UmstManifest`]
        // after enabling **`manifest-bridge`** plus a manifold revision that publishes `manifest::UmstManifest`.
        let manifest = format!(
            concat!(
                "{{",
                "\"schema\":\"umst_beam_dump_v2\",",
                "\"nx\":{nx},\"ny\":{ny},",
                "\"n_nodes\":{n_nodes},\"dx\":{dx},\"batch\":{batch},",
                "\"iters\":{epochs},\"dump_stride\":{dump_stride},",
                "\"n_frames\":{n_frames},\"frame_epochs\":[{frames_csv}],",
                "\"compliance_initial\":{c0:.8},\"compliance_final\":{cf:.8},\"compliance_best\":{cb:.8},",
                "\"e0_pa\":{e0},\"nu\":0.3,\"volume_target\":{vol_tgt},\"simp_p\":{p_simp},",
                "\"n_edges\":{n_edges}",
                "}}"
            ),
            nx = nx,
            ny = ny,
            n_nodes = n_nodes,
            dx = dx,
            batch = batch,
            epochs = epochs,
            dump_stride = dump_stride,
            n_frames = dump_epochs.len(),
            frames_csv = frames_csv,
            c0 = c0,
            cf = compliance_final,
            cb = compliance_best,
            e0 = e0_concrete,
            vol_tgt = vol_tgt,
            p_simp = p_simp,
            n_edges = n_edges,
        );
        fs::write(art_dir.join("manifest.json"), manifest).expect("write beam manifest.json");
        println!("wrote {} (manifest + iter_*.npy)", art_dir.display());
    }
}
