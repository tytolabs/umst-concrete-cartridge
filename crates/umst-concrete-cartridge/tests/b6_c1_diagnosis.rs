// SPDX-License-Identifier: MIT
// B6 c1 gate post-processing — reference triplet + spatial breakdown (H-A).

#![cfg(feature = "solver-experimental")]

use std::env;
use std::fs;
use std::path::Path;

use umst_manifold::ai::topology::ContinuationSchedule;
use umst_manifold::physics::adjoint::SimpElasticMaterial;
use umst_manifold::physics::adjoint_q1_hex::AdjointComplianceQ1Hex;
use umst_manifold::physics::mechanics::SelfWeightConfig;
use umst_manifold::physics::time_orchestration::MechanicsInnerLoopConfig;
use umst_manifold::physics::q1_hex_elasticity::HEX_PCG_MAX_ITER_DEFAULT_STRIATUS;

const NX: usize = 40;
const NY: usize = 40;
const NZ: usize = 4;
const LX: f32 = 4.0;
const LY: f32 = 4.0;
const LZ: f32 = 0.1;
const TARGET_VF: f32 = 0.15;
const E0: f32 = 200e6;
const NU: f32 = 0.2;
const E_MIN_REL: f32 = 1e-3;
const VOID_RHO: f32 = 0.1;
const C0_UNIFORM_P1: f32 = 3.881671;
const GATE_RATIO: f32 = 0.6;

struct B6Mesh {
    dx: f32,
    dy: f32,
    dz: f32,
    n_nodes: usize,
    live_f: Vec<f32>,
    mask: Vec<f32>,
    sw: SelfWeightConfig,
    cg: MechanicsInnerLoopConfig,
}

impl B6Mesh {
    fn striatus_v04() -> Self {
        let dx = LX / NX as f32;
        let dy = LY / NY as f32;
        let dz = LZ / NZ as f32;
        let nx1 = NX + 1;
        let ny1 = NY + 1;
        let n_nodes = nx1 * ny1 * (NZ + 1);
        let mut live_f = vec![0.0_f32; n_nodes * 3];
        let mut mask = vec![1.0_f32; n_nodes * 3];
        let iz_top = NZ;
        let roof_ramp_strength = 0.2_f32;
        let nx_d = NX.max(1) as f32;
        for iy in 0..=NY {
            for ix in 0..=NX {
                let nid = ix + iy * nx1 + iz_top * nx1 * ny1;
                let w = 1.0_f32 + roof_ramp_strength * (ix as f32 / nx_d);
                live_f[nid * 3 + 2] = -50.0 * dx * dy * w;
            }
        }
        let pin = |ix: usize, iy: usize, bm: &mut [f32], nx1: usize| {
            let nid = ix + iy * nx1;
            bm[nid * 3] = 0.0;
            bm[nid * 3 + 1] = 0.0;
            bm[nid * 3 + 2] = 0.0;
        };
        for ix in 0..=NX {
            pin(ix, 0, &mut mask, nx1);
            pin(ix, NY, &mut mask, nx1);
        }
        for iy in 0..=NY {
            pin(0, iy, &mut mask, nx1);
            pin(NX, iy, &mut mask, nx1);
        }
        let sw = SelfWeightConfig {
            gravity_m_s2: 9.81,
            voxel_volume_m3: dx * dy * dz,
            mass_penalty_q: 1.0,
            direction: [0.0, 0.0, -1.0],
        };
        let cg = MechanicsInnerLoopConfig {
            max_cg_iterations: HEX_PCG_MAX_ITER_DEFAULT_STRIATUS,
            cg_tolerance: 1e-4,
            pcg_tolerance: 1e-4,
            use_preconditioner: true,
            max_equilibrium_substeps: 1,
        };
        Self {
            dx,
            dy,
            dz,
            n_nodes,
            live_f,
            mask,
            sw,
            cg,
        }
    }

    fn simp(p: f32) -> SimpElasticMaterial {
        SimpElasticMaterial {
            e0: E0,
            nu: NU,
            p,
            e_min: E0 * E_MIN_REL,
        }
    }

    fn compliance(&self, rho: &[f32], p: f32) -> f32 {
        let (audit, _) = AdjointComplianceQ1Hex::evaluate_compliance(
            rho,
            NX,
            NY,
            NZ,
            self.dx,
            self.dy,
            self.dz,
            &self.live_f,
            &self.mask,
            B6Mesh::simp(p),
            &self.cg,
            Some(self.sw),
        );
        audit.compliance
    }
}

fn node_id(ix: usize, iy: usize, iz: usize, nx1: usize, ny1: usize) -> usize {
    ix + iy * nx1 + iz * nx1 * ny1
}

fn uniform_rho(target_vf: f32, n_nodes: usize) -> Vec<f32> {
    vec![target_vf; n_nodes]
}

/// Hand rib: one solid column every `period` in x, ρ=1 on rib nodes else 0; scaled to hit vf.
fn hand_rib_rho(period: usize, n_nodes: usize) -> Vec<f32> {
    let nx1 = NX + 1;
    let ny1 = NY + 1;
    let nz1 = NZ + 1;
    let mut rho = vec![0.0_f32; n_nodes];
    for iz in 0..nz1 {
        for iy in 0..ny1 {
            for ix in 0..nx1 {
                let nid = node_id(ix, iy, iz, nx1, ny1);
                if ix % period.max(1) == 0 {
                    rho[nid] = 1.0;
                }
            }
        }
    }
    scale_to_vf(&mut rho, TARGET_VF);
    rho
}

/// Best-effort z-gradient from acceptance z-profile, uniform in xy per layer.
fn z_concentrated_rho(n_nodes: usize) -> Vec<f32> {
    let nx1 = NX + 1;
    let ny1 = NY + 1;
    let nz1 = NZ + 1;
    let layer_raw = [0.160_f32, 0.156, 0.151, 0.146, 0.142];
    let mean_raw: f32 = layer_raw.iter().sum::<f32>() / layer_raw.len() as f32;
    let scale = TARGET_VF / mean_raw.max(1e-6);
    let mut rho = vec![0.0_f32; n_nodes];
    for iz in 0..nz1 {
        let r = (layer_raw[iz] * scale).clamp(0.0, 1.0);
        for iy in 0..ny1 {
            for ix in 0..nx1 {
                let nid = node_id(ix, iy, iz, nx1, ny1);
                rho[nid] = r;
            }
        }
    }
    scale_to_vf(&mut rho, TARGET_VF);
    rho
}

fn scale_to_vf(rho: &mut [f32], target: f32) {
    let mean = rho.iter().sum::<f32>() / rho.len().max(1) as f32;
    if mean > 1e-12 {
        let s = target / mean;
        for r in rho.iter_mut() {
            *r = (*r * s).clamp(0.0, 1.0);
        }
    }
}

fn load_rho_bin(path: &Path) -> Vec<f32> {
    let bytes = fs::read(path).expect("read rho export");
    assert_eq!(bytes.len() % 4, 0, "rho bin must be f32 LE");
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn print_triplet(mesh: &B6Mesh, p_accept: f32) {
    let rho_uni = uniform_rho(TARGET_VF, mesh.n_nodes);
    let rho_rib = hand_rib_rho(7, mesh.n_nodes);
    let rho_z = z_concentrated_rho(mesh.n_nodes);

    let c_uni_p1 = mesh.compliance(&rho_uni, 1.0);
    let c_uni_p = mesh.compliance(&rho_uni, p_accept);
    let c_rib = mesh.compliance(&rho_rib, p_accept);
    let c_z = mesh.compliance(&rho_z, p_accept);

    let gate = GATE_RATIO * C0_UNIFORM_P1;
    eprintln!("=== B6 c1 reference triplet (vf={TARGET_VF}, p_accept={p_accept:.3}) ===");
    eprintln!("(i)   uniform @ p=1 Voigt:  c={c_uni_p1:.6}  (ledger c0_uniform_raw={C0_UNIFORM_P1:.6})");
    eprintln!("(i')  uniform @ p={p_accept:.1}: c={c_uni_p:.6}");
    eprintln!("(ii)  hand rib (period=7):   c={c_rib:.6}  ratio/c0={:.3}", c_rib / C0_UNIFORM_P1);
    eprintln!("(iii) z-concentrated:        c={c_z:.6}  ratio/c0={:.3}", c_z / C0_UNIFORM_P1);
    eprintln!("gate c1 < {gate:.6} (0.6 * c0_uniform_p1)");
    eprintln!(
        "pass gate? uni_p1={} rib={} z={}",
        c_uni_p1 < gate,
        c_rib < gate,
        c_z < gate
    );
}

fn spatial_breakdown(mesh: &B6Mesh, rho: &[f32], p_accept: f32, label: &str) {
    let (audit, u) = AdjointComplianceQ1Hex::evaluate_compliance(
        rho,
        NX,
        NY,
        NZ,
        mesh.dx,
        mesh.dy,
        mesh.dz,
        &mesh.live_f,
        &mesh.mask,
        B6Mesh::simp(p_accept),
        &mesh.cg,
        Some(mesh.sw),
    );
    let frac = AdjointComplianceQ1Hex::top_void_column_fractions(
        &audit,
        &u,
        rho,
        NX,
        NY,
        NZ,
        &mesh.live_f,
        &mesh.mask,
        VOID_RHO,
    );
    eprintln!("=== spatial breakdown: {label} ===");
    eprintln!("compliance total: {:.6}", audit.compliance);
    eprintln!("strain_energy total: {:.6}", audit.strain_energy_total);
    eprintln!(
        "top-void-column compliance fraction: {:.1}% (H-A threshold >50%)",
        frac.compliance_fraction * 100.0
    );
    eprintln!(
        "top-void-column strain_energy fraction: {:.1}%",
        frac.strain_energy_fraction * 100.0
    );
    eprintln!(
        "void columns (top ρ<{VOID_RHO}): {:.1}% of xy grid",
        frac.void_column_fraction_xy * 100.0
    );
    eprintln!(
        "c1 ratio vs c0_p1: {:.3}",
        audit.compliance / C0_UNIFORM_P1
    );
    if frac.compliance_fraction > 0.5 || frac.strain_energy_fraction > 0.5 {
        eprintln!("H-A: CONFIRMED mechanically (>50% top-void contribution)");
    } else {
        eprintln!("H-A: NOT confirmed at >50% threshold");
    }
}

#[test]
#[ignore = "B6 c1 reference triplet — release, ~minutes per layout"]
fn b6_c1_reference_triplet() {
    let mesh = B6Mesh::striatus_v04();
    let p_accept = ContinuationSchedule::value(199, 200);
    print_triplet(&mesh, p_accept);
}

#[test]
#[ignore = "B6 c1 spatial breakdown — needs UMST_SHELL_RHO_BIN export from 200-outer"]
fn b6_c1_accepted_export_spatial() {
    let path = env::var("UMST_SHELL_RHO_BIN").unwrap_or_else(|_| {
        "/tmp/b6_acceptance_rho.bin".to_string()
    });
    if !Path::new(&path).exists() {
        eprintln!("skip: rho export missing at {path} — re-run 200-outer with UMST_SHELL_EXPORT_RHO={path}");
        return;
    }
    let rho = load_rho_bin(Path::new(&path));
    let mesh = B6Mesh::striatus_v04();
    assert_eq!(rho.len(), mesh.n_nodes, "rho node count mismatch");
    let p_accept = ContinuationSchedule::value(199, 200);
    print_triplet(&mesh, p_accept);
    spatial_breakdown(&mesh, &rho, p_accept, "accepted export");
}
