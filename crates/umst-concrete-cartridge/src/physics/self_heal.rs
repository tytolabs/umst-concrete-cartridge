// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
use burn::tensor::{backend::Backend, Tensor};

use crate::chem_adapter::nano_healing_boost_per_dosage_f32;

/// Differentiable slice of the manifold state feeding the autogenous-healing head.
///
/// This is an internal cohesion type: the public API remains
/// [`SelfHealEngine::compute_healing_potential`], which delegates here.
#[derive(Clone)]
pub(crate) struct HealingObservableState<B: Backend> {
    pub(crate) degree_hydration: Tensor<B, 4>,
    pub(crate) internal_rh: Tensor<B, 4>,
    pub(crate) nano_dosage: Tensor<B, 4>,
}

/// Pure state transformer: observable field bundle → healing-potential field \([0,1]\).
///
/// No `&mut self`; all `burn` ops are functional on owned tensors.
pub(crate) fn transform_healing_observable_state<B: Backend>(
    state: HealingObservableState<B>,
) -> Tensor<B, 4> {
    let HealingObservableState {
        degree_hydration,
        internal_rh,
        nano_dosage,
    } = state;

    // 1. Unhydrated Cement Fraction
    let unhydrated_fraction = degree_hydration
        .mul_scalar(-1.0_f32)
        .add_scalar(1.0_f32)
        .clamp_min(0.0_f32);

    // 2. Moisture Availability (Healing requires water)
    // High internal RH (> 90%) dramatically accelerates healing.
    let moisture_factor = internal_rh
        .clone()
        .sub_scalar(0.8_f32)
        .clamp_min(0.0_f32)
        .mul_scalar(5.0_f32)
        .clamp_max(1.0_f32);

    // 3. Nucleation Seeding (Nano-silica provides sites for C-S-H precipitation)
    let nano_boost = nano_dosage
        .mul_scalar(nano_healing_boost_per_dosage_f32())
        .add_scalar(1.0_f32);

    // Healing potential metric (0.0 to 1.0)
    unhydrated_fraction
        .mul(moisture_factor)
        .mul(nano_boost)
        .clamp(0.0_f32, 1.0_f32)
}

/// Pure tensor implementation of the Autogenous Healing Engine.
/// Computes crack-closure potential from unhydrated cement particles and precipitation.
/// formal_anchor: empirical://datasets/dataset_selfheal.csv
/// formal_status: Empirical
/// formal_axioms: NONE
/// formal_dataset: "prototype_selfheal_boundary"
/// formal_citation: "Edvardsen (1999) ACI Mater. J. 96, 448"
/// formal_envelope: "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs"
pub struct SelfHealEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> SelfHealEngine<B> {
    /// Computes the healing potential and recovered fracture energy.
    /// Unhydrated cement and moisture presence allow microcracks to seal over time.
    /// formal_anchor: empirical://datasets/dataset_selfheal.csv
    /// formal_status: Empirical
    /// formal_axioms: NONE
    /// formal_dataset: "prototype_selfheal_boundary"
    /// formal_citation: "Edvardsen (1999) ACI Mater. J. 96, 448"
    /// formal_envelope: "Boundary profile (no [acceptance] strength gate); paired CSV still listed in dataset_metrics skip — healing kinetics exercised under tests/realism/adversarial_physics.rs"
    pub fn compute_healing_potential(
        degree_hydration: Tensor<B, 4>,
        internal_rh: Tensor<B, 4>,
        nano_dosage: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        transform_healing_observable_state(HealingObservableState {
            degree_hydration,
            internal_rh,
            nano_dosage,
        })
    }
}
