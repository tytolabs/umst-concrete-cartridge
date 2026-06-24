// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Concrete-cartridge [`GateCartridge::transition_evidence`] — dissipation / strength / hydration SSOT.

use umst_manifold::gate::transition_proposal::ThermodynamicStateSnapshot;
use umst_manifold::runtime::gate::evidence::{explain_cd_transition_host, TransitionEvidence};
use umst_manifold::runtime::gate::GateCartridge;

use crate::material_transition::{CementMaterialParams, CEMENT_DEFAULT_S_INTRINSIC_MPA};

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
        let host =
            umst_manifold::gate::transition_proposal::transition_outcome(old, new, dt, tolerance);
        host.dissipation
    }
}

impl GateCartridge for ConcreteTransitionCartridge {
    fn transition_evidence(
        &self,
        old: &ThermodynamicStateSnapshot,
        new: &ThermodynamicStateSnapshot,
        dt: f64,
    ) -> TransitionEvidence {
        let explanation = explain_cd_transition_host(old, new, dt, 1e-6);
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
        assert!(host.energy_positive);

        let evidence = ConcreteTransitionCartridge.transition_evidence(&old, &new, 1.0);
        assert_eq!(evidence.catalog_id, CD_TRANSITION_CATALOG_ID);
        assert_eq!(evidence.admissibility, AdmissibilityToken::Admissible);
    }

    #[test]
    fn concrete_transition_cartridge_rejects_psi_spike() {
        let old = ConcreteTransitionCartridge::snapshot_from_mix(0.45, 0.3, 293.15);
        let mut new = old;
        new.free_energy = 1.0e6;
        let host = transition_outcome(&old, &new, 1.0, 1e-6);
        assert!(
            !host.energy_positive,
            "sanity: ψ spike should reject on host"
        );
        let evidence = ConcreteTransitionCartridge.transition_evidence(&old, &new, 1.0);
        assert_eq!(evidence.admissibility, AdmissibilityToken::Inadmissible);
        assert!(
            ConcreteTransitionCartridge::dissipation_joules(&old, &new, 1.0, 1e-6) < 0.0,
            "inadmissible mix transition must show negative dissipation on host path"
        );
    }
}
