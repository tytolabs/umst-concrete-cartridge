// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! S5 path-dep bridge — monolith seam into `umst-cartridge-concrete`.
//!
//! The monolith crate always path-deps the consumer cartridge (T2-S5). Production
//! routing is unconditional post-S6 batch B1; this module is the single import surface
//! for composed constitutive + gate helpers. MCP harness bytes and fixture routing
//! remain frozen — see `umst-mcp/tests/gate_parity.rs`.

pub use umst_cartridge_concrete::api_consumer_history_prep::{
    g0_consumer_history_binding, try_consumer_gate_route_with_history,
};
pub use umst_cartridge_concrete::compose::gate_route_composed_with_history;
pub use umst_cartridge_concrete::{
    dissipation_modulus_eta, dissipation_modulus_eta_from_profile,
    dissipation_modulus_eta_from_s_intrinsic, g0_probe_atom_state, gate_route_composed,
    regime_hyperbox_admits_w_c, regime_hyperbox_admits_w_c_for_max, ComposedGateOutcome,
    MixScalars, D_CLOSURE_ABS_TOL, GATE_W_C_REGIME_HYPERBOX_MAX, PSI_CLOSURE_ABS_TOL,
};
pub use umst_cartridge_continuum::{
    ContinuumAtomRates, ContinuumAtomStateWithHistory, ContinuumPhysicsError,
};

#[cfg(test)]
mod witness {
    use super::{
        dissipation_modulus_eta, g0_probe_atom_state, gate_route_composed, ContinuumAtomRates,
        MixScalars, D_CLOSURE_ABS_TOL, PSI_CLOSURE_ABS_TOL,
    };

    /// G0 pass mix — mirrors `gate_parity_v0.json` first admissible row.
    fn g0_pass_mix() -> MixScalars {
        MixScalars {
            key: Some("g0_pass".into()),
            w_c: 0.45,
            temperature_k: 293.15,
            fly_ash_pct: 0.0,
            silica_fume_pct: 0.0,
            superplasticiser_pct: 0.0,
            target_age_hours: 28.0 * 24.0,
            s_intrinsic_mpa: None,
            w_c_max: None,
            total_binder_kg_m3: None,
            aggregate_volume_fraction: None,
            strength_model: None,
            jennings_exponent: None,
        }
    }

    #[test]
    fn bridge_dissipation_modulus_matches_consumer_ssot() {
        assert_eq!(
            dissipation_modulus_eta(),
            umst_cartridge_concrete::dissipation_modulus_eta()
        );
    }

    #[test]
    fn bridge_gate_route_matches_consumer_ssot_at_g0_pass() {
        let mix = g0_pass_mix();
        let via_bridge = gate_route_composed(
            &mix,
            g0_probe_atom_state(),
            ContinuumAtomRates::PASSIVE,
            0.0,
            PSI_CLOSURE_ABS_TOL,
            D_CLOSURE_ABS_TOL,
        );
        let via_consumer = umst_cartridge_concrete::gate_route_composed(
            &mix,
            umst_cartridge_concrete::g0_probe_atom_state(),
            umst_cartridge_continuum::ContinuumAtomRates::PASSIVE,
            0.0,
            umst_cartridge_concrete::PSI_CLOSURE_ABS_TOL,
            umst_cartridge_concrete::D_CLOSURE_ABS_TOL,
        );
        assert_eq!(via_bridge.route.admissible, via_consumer.route.admissible);
        assert!(
            (via_bridge.constitutive.psi_total() - via_consumer.constitutive.psi_total()).abs()
                < 1e-9
        );
    }
}
