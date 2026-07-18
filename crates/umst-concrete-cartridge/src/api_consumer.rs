// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! M1 — concrete cartridge implements [`umst_cartridge_api::UMSTCartridge`] at scalar parity.
//!
//! With `b1-delegate` (S2 production wire), constitutive evaluation routes through
//! `umst-cartridge-concrete::gate_route_composed` instead of monolith homogeneous closures.

use std::sync::OnceLock;

use umst_cartridge_api::{
    CartridgeId, ClausiusDuhemWitness, ConstitutiveResponse,
    MassConservationWitness, PhysicalAxiom, Rates, ScalarAlgebra, State, StateSchema, StateVar,
    StateVarKind, TensorAlgebra, UMSTCartridge,
};

use crate::calibration::Profile;
use crate::homogeneous::{self as homog, MixRow};

#[cfg(feature = "b1-delegate")]
use crate::api_consumer_compose::{gate_route_via_compose, scalar_fields_from_composed};

/// Stable registry id for the cementitious consumer cartridge.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Registry string for MCP `@context` and cartridge catalog.
pub const CONCRETE_CARTRIDGE_ID: &str = "umst-cartridge-concrete";

/// Index of ψ (J/m³) in [`State::values`] for [`ConcreteApiCartridge`].
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Layout index into precomputed scalar state vector.
pub const IDX_PSI_J_PER_M3: usize = 0;
/// Index of bulk density ρ (kg/m³) in [`State::values`].
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Layout index into precomputed scalar state vector.
pub const IDX_DENSITY_KG_M3: usize = 1;
/// Index of dissipation modulus η (convex φ = η·α̇²) in [`State::values`].
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: Layout index into precomputed scalar state vector.
pub const IDX_DISSIPATION_MODULUS: usize = 2;

static MASS_AXIOM: MassConservationWitness = MassConservationWitness::parity_default();
static CD_AXIOM: ClausiusDuhemWitness = ClausiusDuhemWitness {
    tolerance: 1e-9,
};

/// formal_anchor: lean://umst-formal/Lean/Concrete/Powers.lean#PowersState
/// catalog_id: thermodynamic_mix
/// formal_status: Mechanised
/// formal_axioms: physicalSecondLaw
/// formal_anchor_rationale: M1 consumer implementing semver-locked [`UMSTCartridge`] via B1 composed delegate when `b1-delegate` is enabled.
#[derive(Debug, Clone)]
pub struct ConcreteApiCartridge {
    /// Active calibration profile (matches MCP / CLI `predict` profile).
    pub profile: Profile,
}

impl ConcreteApiCartridge {
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Deterministic bundled baseline for tests and smoke defaults.
    #[must_use]
    pub fn new() -> Result<Self, crate::calibration::CalibrationError> {
        Ok(Self::with_profile(Profile::load_bundled("uci_d1")?))
    }

    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Caller-owned calibration bundle; avoids silent profile mixing.
    #[must_use]
    pub fn with_profile(profile: Profile) -> Self {
        Self { profile }
    }

    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Lifts [`MixRow`] into scalar [`State`] slots for [`UMSTCartridge`] evaluation.
    #[must_use]
    pub fn scalar_state_from_mix_row(
        &self,
        row: &MixRow,
    ) -> Result<(StateSchema, Vec<f64>), homog::HomogeneousError> {
        let fields = scalar_state_fields(&self.profile, row)?;
        Ok((concrete_state_schema(), fields))
    }

    /// S2 composed delegate seam — routes through `gate_route_composed`.
    #[cfg(feature = "b1-delegate")]
    #[must_use]
    pub fn gate_route_via_compose(
        &self,
        row: &MixRow,
        alpha_dot: f64,
    ) -> umst_cartridge_concrete::ComposedGateOutcome {
        gate_route_via_compose(&self.profile, row, alpha_dot)
    }

    /// formal_anchor: STRUCTURAL
    /// formal_status: Structural
    /// formal_anchor_rationale: Adapter seam toward Core `gate<R>` (`ConstitutiveResponse` bundle).
    #[must_use]
    pub fn constitutive_response_from_mix_row(
        &self,
        row: &MixRow,
        alpha_dot: f64,
    ) -> Result<ConstitutiveResponse<ScalarAlgebra>, homog::HomogeneousError> {
        #[cfg(feature = "b1-delegate")]
        {
            let outcome = gate_route_via_compose(&self.profile, row, alpha_dot);
            return Ok(ConstitutiveResponse::passive(
                outcome.constitutive.psi_total(),
                outcome.constitutive.dissipation_total(),
                alpha_dot,
            ));
        }
        #[cfg(not(feature = "b1-delegate"))]
        {
            use umst_cartridge_api::constitutive_response;
            let (schema, values) = self.scalar_state_from_mix_row(row)?;
            let state = State {
                values: &values,
                schema: &schema,
            };
            let rates = Rates::<ScalarAlgebra> {
                internal: alpha_dot,
                species_source: None,
                values: None,
            };
            Ok(constitutive_response(self, &state, &rates))
        }
    }
}

impl UMSTCartridge for ConcreteApiCartridge {
    fn id(&self) -> CartridgeId {
        CartridgeId::new(CONCRETE_CARTRIDGE_ID)
    }

    fn state_schema(&self) -> &StateSchema {
        static SCHEMA: OnceLock<StateSchema> = OnceLock::new();
        SCHEMA.get_or_init(concrete_state_schema)
    }

    fn free_energy<A: TensorAlgebra>(&self, state: &State<'_, A>) -> A::Field {
        state
            .values
            .get(IDX_PSI_J_PER_M3)
            .cloned()
            .unwrap_or_else(A::zero)
    }

    fn dissipation_potential<A: TensorAlgebra>(
        &self,
        state: &State<'_, A>,
        rates: &Rates<'_, A>,
    ) -> A::Field {
        let rate = rates.internal.clone();
        let rate_sq = A::mul(rate.clone(), rate);
        let eta = state
            .values
            .get(IDX_DISSIPATION_MODULUS)
            .cloned()
            .unwrap_or_else(A::zero);
        A::mul(eta, rate_sq)
    }

    fn physical_axioms(&self) -> &[&dyn PhysicalAxiom] {
        static AXIOMS: OnceLock<Vec<&'static dyn PhysicalAxiom>> = OnceLock::new();
        AXIOMS
            .get_or_init(|| vec![&MASS_AXIOM, &CD_AXIOM])
            .as_slice()
    }
}

#[must_use]
fn concrete_state_schema() -> StateSchema {
    StateSchema {
        vars: vec![
            StateVar {
                name: "psi_j_per_m3",
                kind: StateVarKind::Scalar,
                unit: "J/m^3",
            },
            StateVar {
                name: "density_kg_m3",
                kind: StateVarKind::Scalar,
                unit: "kg/m^3",
            },
            StateVar {
                name: "dissipation_modulus",
                kind: StateVarKind::Scalar,
                unit: "J*s/m^3",
            },
        ],
    }
}

fn scalar_state_fields(
    profile: &Profile,
    row: &MixRow,
) -> Result<Vec<f64>, homog::HomogeneousError> {
    #[cfg(feature = "b1-delegate")]
    {
        let (psi_j_per_m3, density, eta) = scalar_fields_from_composed(profile, row, 0.0);
        return Ok(vec![psi_j_per_m3, density, eta]);
    }
    #[cfg(not(feature = "b1-delegate"))]
    {
        let (w_c_eff, alpha, _temp_c) = homog::mix_hydration_state(profile, row)?;
        let fc_mpa = homog::powers_compressive_strength_mpa(profile, row, alpha, w_c_eff)?;
        let psi_j_per_m3 = -f64::from(fc_mpa) * 1e6;
        let density = concrete_bulk_density_kg_m3(row);
        let eta = dissipation_modulus_from_profile(profile);
        Ok(vec![psi_j_per_m3, density, eta])
    }
}

#[must_use]
fn concrete_bulk_density_kg_m3(row: &MixRow) -> f64 {
    let binder = f64::from(row.cement_kg_m3 + row.slag_kg_m3 + row.fly_ash_kg_m3);
    let water = f64::from(row.water_kg_m3);
    let sp = f64::from(row.superplasticizer_kg_m3);
    binder + water + sp
}

#[must_use]
fn dissipation_modulus_from_profile(profile: &Profile) -> f64 {
    // Cartridge SSOT — mirrors `material_transition::CEMENT_REACTION_ENTHALPY_J_PER_KG` (450 J/kg).
    const CEMENT_REACTION_ENTHALPY_J_PER_KG: f64 = 450.0;
    let enthalpy = CEMENT_REACTION_ENTHALPY_J_PER_KG;
    let s_scale = f64::from(profile.powers.s_intrinsic);
    (enthalpy * s_scale * 1e6).max(1.0)
}
