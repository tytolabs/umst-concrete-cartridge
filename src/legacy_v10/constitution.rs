// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary

//! Constitutional Physics Layer (Elevated v2 - Ported to MaOS-Core)
//! First-class PhysicalAxiom trait with proof-carrying witnesses and umst-formal traceability.
//! Hybrid with existing TypedGate/AdmissibilityProof for no-compromise integration.

use crate::science::thermodynamic_filter::{AdmissibilityResult, ThermodynamicState};
use serde::{Deserialize, Serialize};

/// A single constitutional axiom (one of the inviolable physical laws).
pub trait PhysicalAxiom {
    fn check(
        &self,
        old: &ThermodynamicState,
        new: &ThermodynamicState,
    ) -> Result<InvariantWitness, Violation>;
    fn formal_reference(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn affected_invariant(&self) -> &'static str;
}

/// Proof-carrying witness for a satisfied invariant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvariantWitness {
    MassConserved {
        delta_rho: f64,
        tolerance: f64,
    },
    HydrationIrreversible {
        delta_alpha: f64,
        tolerance: f64,
    },
    PositiveDissipation {
        d_int: f64,
        rho: f64,
        psi_dot: f64,
    },
    StrengthMonotonic {
        delta_fc: f64,
        tolerance: f64,
    },
    Custom {
        name: &'static str,
        metadata: serde_json::Value,
    },
}

/// Violation with proof attempt and formal reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub axiom: &'static str,
    pub witness_attempt: Option<InvariantWitness>,
    pub formal_ref: &'static str,
}

/// The Constitution - a composable set of physical axioms.
pub struct Constitution {
    axioms: Vec<Box<dyn PhysicalAxiom>>,
}

impl Constitution {
    pub fn new() -> Self {
        Constitution { axioms: vec![] }
    }

    pub fn add_axiom<A: PhysicalAxiom + 'static>(&mut self, axiom: A) {
        self.axioms.push(Box::new(axiom));
    }

    /// Register a boxed axiom (dyn-compatible); preferred entry point for Science Cartridges.
    pub fn register_axiom(&mut self, axiom: Box<dyn PhysicalAxiom>) {
        self.axioms.push(axiom);
    }

    pub fn verify_transition(
        &self,
        old: &ThermodynamicState,
        new: &ThermodynamicState,
    ) -> AdmissibilityResult {
        let mut violations = vec![];
        let mut witnesses = vec![];
        let mut mass_conserved = true;
        let mut energy_positive = true;
        let mut _hydration_irreversible = true;

        for axiom in &self.axioms {
            match axiom.check(old, new) {
                Ok(witness) => {
                    witnesses.push(witness);
                }
                Err(violation) => {
                    violations.push(violation);
                    match axiom.affected_invariant() {
                        "mass_conserved" => mass_conserved = false,
                        "hydration_irreversible" => _hydration_irreversible = false,
                        _ => energy_positive = false,
                    }
                }
            }
        }

        let accepted = violations.is_empty();
        let _cgs = if accepted { 9.5 } else { 3.0 };

        AdmissibilityResult {
            accepted,
            dissipation: 0.0,
            mass_conserved,
            energy_positive,
        }
    }

    pub fn standard() -> Self {
        let mut constitution = Constitution::new();
        constitution.add_axiom(MassConservationAxiom);
        constitution.add_axiom(HydrationIrreversibilityAxiom);
        constitution.add_axiom(ClausiusDuhemAxiom);
        constitution.add_axiom(StrengthMonotonicityAxiom);
        constitution
    }
}

// Basic axiom implementations (aligned with umst-formal)
pub struct MassConservationAxiom;
impl PhysicalAxiom for MassConservationAxiom {
    fn check(
        &self,
        old: &ThermodynamicState,
        new: &ThermodynamicState,
    ) -> Result<InvariantWitness, Violation> {
        let delta = (new.density - old.density).abs();
        let tolerance = 0.01 * old.density.max(1.0);
        if delta > tolerance {
            Err(Violation {
                axiom: "MassConservation",
                witness_attempt: Some(InvariantWitness::MassConserved {
                    delta_rho: delta,
                    tolerance,
                }),
                formal_ref: self.formal_reference(),
            })
        } else {
            Ok(InvariantWitness::MassConserved {
                delta_rho: delta,
                tolerance,
            })
        }
    }

    fn formal_reference(&self) -> &'static str {
        "docs/formal/Lean/Gate.lean: massConserved (embedded in independent MaOS-Core)"
    }

    fn description(&self) -> &'static str {
        "Mass conservation: |ρ_new - ρ_old| < δ"
    }

    fn affected_invariant(&self) -> &'static str {
        "mass_conserved"
    }
}

/// Full implementations for all 4 axioms (ported from prototype-2a, adapted for core independence and ThermodynamicState fields)
pub struct HydrationIrreversibilityAxiom;
impl PhysicalAxiom for HydrationIrreversibilityAxiom {
    fn check(
        &self,
        old: &ThermodynamicState,
        new: &ThermodynamicState,
    ) -> Result<InvariantWitness, Violation> {
        let delta = new.hydration_degree - old.hydration_degree;
        let tolerance = 1e-6;
        if delta < -tolerance {
            Err(Violation {
                axiom: "HydrationIrreversibility",
                witness_attempt: Some(InvariantWitness::HydrationIrreversible {
                    delta_alpha: delta,
                    tolerance,
                }),
                formal_ref: self.formal_reference(),
            })
        } else {
            Ok(InvariantWitness::HydrationIrreversible {
                delta_alpha: delta,
                tolerance,
            })
        }
    }

    fn formal_reference(&self) -> &'static str {
        "docs/formal/Agda/Gate.agda: forward-hydration-admissible (embedded in independent MaOS-Core)"
    }

    fn description(&self) -> &'static str {
        "Hydration irreversibility: α_new ≥ α_old"
    }

    fn affected_invariant(&self) -> &'static str {
        "hydration_irreversible"
    }
}

/// Clausius-Duhem Dissipation Axiom
pub struct ClausiusDuhemAxiom;
impl PhysicalAxiom for ClausiusDuhemAxiom {
    fn check(
        &self,
        old: &ThermodynamicState,
        new: &ThermodynamicState,
    ) -> Result<InvariantWitness, Violation> {
        let psi_dot = new.free_energy - old.free_energy;
        let rho = old.density.max(1.0);
        let d_int_approx = -rho * psi_dot;
        if d_int_approx < 0.0 {
            Err(Violation {
                axiom: "ClausiusDuhem",
                witness_attempt: Some(InvariantWitness::PositiveDissipation {
                    d_int: d_int_approx,
                    rho,
                    psi_dot,
                }),
                formal_ref: self.formal_reference(),
            })
        } else {
            Ok(InvariantWitness::PositiveDissipation {
                d_int: d_int_approx,
                rho,
                psi_dot,
            })
        }
    }

    fn formal_reference(&self) -> &'static str {
        "docs/formal/Coq/Gate.v: clausius_duhem_forward (embedded in independent MaOS-Core)"
    }

    fn description(&self) -> &'static str {
        "Clausius-Duhem dissipation: D_int ≥ 0"
    }

    fn affected_invariant(&self) -> &'static str {
        "energy_positive"
    }
}

/// Strength Monotonicity Axiom
pub struct StrengthMonotonicityAxiom;
impl PhysicalAxiom for StrengthMonotonicityAxiom {
    fn check(
        &self,
        old: &ThermodynamicState,
        new: &ThermodynamicState,
    ) -> Result<InvariantWitness, Violation> {
        let delta = new.strength - old.strength;
        let tolerance = 1e-6;
        if delta < -tolerance {
            Err(Violation {
                axiom: "StrengthMonotonicity",
                witness_attempt: Some(InvariantWitness::StrengthMonotonic {
                    delta_fc: delta,
                    tolerance,
                }),
                formal_ref: self.formal_reference(),
            })
        } else {
            Ok(InvariantWitness::StrengthMonotonic {
                delta_fc: delta,
                tolerance,
            })
        }
    }

    fn formal_reference(&self) -> &'static str {
        "docs/formal/Lean/Gate.lean: strengthMono (embedded in independent MaOS-Core)"
    }

    fn description(&self) -> &'static str {
        "Strength monotonicity: fc_new ≥ fc_old"
    }

    fn affected_invariant(&self) -> &'static str {
        "strength_monotonic"
    }
}
