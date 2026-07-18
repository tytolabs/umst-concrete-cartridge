// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! # S6_RETIRE — Tier A.2 scalar `G_c` path (card `g_spawn_i_frac_s6_2054`)
//!
//! Profile-scaled fracture energy \(G_F\) / \(G_c\) [J/m²] shared by the physics pipeline fracture stage
//! and (under `solver-experimental`) the manifold phase-field fracture solver's \(G_c\) channel tensor.
//!
//! B2 scalar SSOT: `umst-cartridge-solid-inelastic::fracture_phase` (`g_spawn_i_fracture_1947`).
//! Consumer compose seam: `umst-cartridge-concrete/compose_b2_prep.rs` `try_fracture_phase_ledger` L167.
//! Inventory: `s6_fracture_tensor_inventory.rs`. **TODO-M3-007 OPEN** — purge BLOCKED; board
//! `outputs/.tmp/FRACTURE_S6_PURGE_2252.md` · sweep `outputs/.tmp/RESEARCH_TODO_NIGHT_2334.md`. **Delete this module** after orchestrator reroute
//! delegates `G_c` / `K_Ic` exclusively to B2 scalar.
//!
//! [`crate::physics::fracture::FractureEngine::compute_fracture_toughness`] maps \((E_{\mathrm{eff}}, G_F)\)
//! to a rank‑4 \(K_{Ic}\) proxy tensor used in [`crate::pipeline::PhysicsPipelineSummary`].

use crate::calibration::Profile;

/// Baseline fracture energy \(G_c\) [J/m²] for AT2 / cohesive models, anchored to bundled `uci_d1`
/// so [`Profile::load_bundled`] `"uci_d1"` reproduces the historical uniform placeholder magnitude.
pub(crate) const BASE_FRACTURE_ENERGY_GC_J_M2: f32 = 120.0;

/// [`PowersGelParameters::s_intrinsic`] for bundled profile `"uci_d1"` (MPa-scale gel intrinsic term).
/// Keeps `fracture_energy_gc_j_per_m2_from_profile` at [`BASE_FRACTURE_ENERGY_GC_J_M2`] for the default cartridge.
const UCI_D1_S_INTRINSIC_REF: f32 = 74.92;

/// Scalar fracture energy [J/m²] from calibration [`Profile`], linear in `powers.s_intrinsic`.
///
/// Other profiles scale relative to `uci_d1`. For a **spatial** \(G_c\) field, populate UMST
/// `scalar_features` column [`umst_manifold::core::SCALAR_FRACTURE_ENERGY_GC`]; otherwise this scalar
/// is broadcast to `[1, N, 1]` for the phase-field solver.
///
/// T2-S6 Tier A.2 allowlist [B2-SCALAR] `fracture_energy_gc_j_per_m2_from_profile` — purge BLOCKED until orchestrator reroute
/// B2 consumer SSOT: `umst-cartridge-solid-inelastic/src/fracture_phase.rs` `try_fracture_energy_gc_j_m2` L53
/// consumer compose seam: `umst-cartridge-concrete/src/compose_b2_prep.rs` `mechanics_extract_witness_at_mix` L167
#[must_use]
pub(crate) fn fracture_energy_gc_j_per_m2_from_profile(profile: &Profile) -> f32 {
    let s = profile.powers.s_intrinsic as f32;
    let s_clamped = s.max(1e-6);
    BASE_FRACTURE_ENERGY_GC_J_M2 * (s_clamped / UCI_D1_S_INTRINSIC_REF)
}
