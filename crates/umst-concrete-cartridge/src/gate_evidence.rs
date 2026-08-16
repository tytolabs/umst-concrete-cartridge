// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Concrete-cartridge [`GateCartridge::transition_evidence`] — dissipation / strength / hydration SSOT.

use umst_manifold::gate::transition_proposal::{
    transition_outcome, ThermodynamicStateSnapshot, TRANSITION_TOLERANCE,
};
use umst_manifold::runtime::gate::evidence::{explain_cd_transition_host, TransitionEvidence};
use umst_manifold::runtime::gate::GateCartridge;

use crate::material_transition::{CementMaterialParams, CEMENT_DEFAULT_S_INTRINSIC_MPA};

/// Cartridge-enriched transition witness — core [`TransitionEvidence`] plus scalar telemetry.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Telemetry envelope around manifold [`TransitionEvidence`]; CD admissibility on `core` leg.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConcreteTransitionWitness {
    pub core: TransitionEvidence,
    pub strength_mpa: f64,
    pub hydration_alpha: f64,
    pub temperature_k: f64,
    pub dissipation_joules: f64,
}

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Zero-sized [`GateCartridge`] witness binding cement SSOT into manifold transition evidence.
/// Cartridge-backed transition witness — lifts mix-calibrated snapshots with cement SSOT constants.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConcreteTransitionCartridge;

impl ConcreteTransitionCartridge {
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Deterministic lift of mix scalars into [`ThermodynamicStateSnapshot`] via cartridge cement params.
    /// Lift admissible mix coordinates into a gate snapshot (hydration + strength scale).
    #[must_use]
    pub fn snapshot_from_mix(
        w_c: f64,
        alpha: f64,
        temperature_k: f64,
    ) -> ThermodynamicStateSnapshot {
        ThermodynamicStateSnapshot::from_mix_calibrated_with_params(
            w_c,
            alpha,
            temperature_k,
            CEMENT_DEFAULT_S_INTRINSIC_MPA,
            &CementMaterialParams,
        )
    }

    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Delegates dissipation to manifold `transition_outcome` host path for telemetry parity.
    /// Host dissipation scalar for telemetry (joules-equivalent, same contract as manifold gate).
    #[must_use]
    pub fn dissipation_joules(
        old: &ThermodynamicStateSnapshot,
        new: &ThermodynamicStateSnapshot,
        dt: f64,
        tolerance: f64,
    ) -> f64 {
        transition_outcome(old, new, dt, tolerance).dissipation
    }

    /// Enriched witness with cartridge scalar snapshots and dissipation metadata.
    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Composes manifold `transition_evidence` with cartridge dissipation telemetry.
    #[must_use]
    pub fn transition_witness(
        &self,
        old: &ThermodynamicStateSnapshot,
        new: &ThermodynamicStateSnapshot,
        dt: f64,
    ) -> ConcreteTransitionWitness {
        let core = self.transition_evidence(old, new, dt);
        ConcreteTransitionWitness {
            core,
            strength_mpa: new.strength,
            hydration_alpha: new.reaction_extent,
            temperature_k: new.temperature,
            dissipation_joules: Self::dissipation_joules(old, new, dt, TRANSITION_TOLERANCE),
        }
    }
}

impl GateCartridge for ConcreteTransitionCartridge {
    fn transition_evidence(
        &self,
        old: &ThermodynamicStateSnapshot,
        new: &ThermodynamicStateSnapshot,
        dt: f64,
    ) -> TransitionEvidence {
        let explanation = explain_cd_transition_host(old, new, dt, TRANSITION_TOLERANCE);
        TransitionEvidence::from_constraint_explanation(explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umst_manifold::gate::transition_proposal::transition_outcome;
    use umst_manifold::runtime::catalog::traceability::CD_TRANSITION_CATALOG_ID;
    use umst_manifold::runtime::gate::evidence::AdmissibilityToken;

    #[test]
    fn concrete_transition_cartridge_matches_cd_on_identity_mix() {
        use umst_manifold::runtime::gate::{CdTransitionCartridge, GateCartridge};

        let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
        let new = old;
        let concrete = ConcreteTransitionCartridge.transition_evidence(&old, &new, 1.0);
        let cd = CdTransitionCartridge.transition_evidence(&old, &new, 1.0);
        assert_eq!(concrete.catalog_id, cd.catalog_id);
        assert_eq!(concrete.admissibility, cd.admissibility);
    }

    #[test]
    fn concrete_transition_cartridge_admits_idle_mix() {
        let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
        let new = old;
        let host = transition_outcome(&old, &new, 1.0, 1e-6);
        assert!(host.is_energy_positive());

        let evidence = ConcreteTransitionCartridge.transition_evidence(&old, &new, 1.0);
        assert_eq!(evidence.catalog_id, CD_TRANSITION_CATALOG_ID);
        assert_eq!(evidence.admissibility, AdmissibilityToken::Admissible);
    }

    #[test]
    fn concrete_transition_cartridge_rejects_psi_spike() {
        let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
        let mut new = old;
        new.free_energy = 1.0e6;
        let host = transition_outcome(&old, &new, 1.0, TRANSITION_TOLERANCE);
        assert!(
            !host.is_energy_positive(),
            "sanity: ψ spike should reject on host"
        );
        let evidence = ConcreteTransitionCartridge.transition_evidence(&old, &new, 1.0);
        assert_eq!(evidence.admissibility, AdmissibilityToken::Inadmissible);
        assert!(
            ConcreteTransitionCartridge::dissipation_joules(&old, &new, 1.0, TRANSITION_TOLERANCE)
                < 0.0,
            "inadmissible mix transition must show negative dissipation on host path"
        );
    }

    #[test]
    fn concrete_transition_witness_carries_scalar_metadata() {
        let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
        let new = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.5, 298.0);
        let witness = ConcreteTransitionCartridge.transition_witness(&old, &new, 3600.0);
        assert!(witness.strength_mpa.is_finite());
        assert!((witness.hydration_alpha - 0.5).abs() < 1e-9);
        assert!((witness.temperature_k - 298.0).abs() < 1e-9);
        assert!(witness.dissipation_joules.is_finite());
        assert_eq!(witness.core.catalog_id, CD_TRANSITION_CATALOG_ID);
    }
}
