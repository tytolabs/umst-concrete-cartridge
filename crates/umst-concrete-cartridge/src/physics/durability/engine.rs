// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
//
// FLEET-COMPOSER-ACCEL-D AC105 — composite concrete durability facade.
// Composes freeze–thaw (Powers), chloride transport (Tang–Nilsson), and autogenous healing
// (Edvardsen) into a weakest-link durability index. Orchestrator still calls `freeze_thaw`
// directly until `mod.rs` wires this module.

use burn::tensor::{backend::Backend, Tensor};

use umst_concrete_cartridge::physics::{
    freeze_thaw::FreezeThawEngine, self_heal::SelfHealEngine, transport::TransportEngine,
};

/// Logical AND for boolean tensors (mirrors `crate::burn_compat::bool_and`).
#[inline]
fn bool_and<B: Backend, const D: usize>(
    a: Tensor<B, D, burn::tensor::Bool>,
    b: Tensor<B, D, burn::tensor::Bool>,
) -> Tensor<B, D, burn::tensor::Bool> {
    a.float().mul(b.float()).greater_elem(0.5_f32)
}

/// Orchestrator mix pin — air volume fraction on bulk freeze–thaw path.
/// Class: **Primitive-fact** (routing contract from `pipeline/orchestrator.rs` L527).
pub const ORCHESTRATOR_PIN_AIR_FRACTION: f32 = 0.04;

/// Orchestrator mix pin — paste volume fraction at agg_vf = 0.35.
/// Class: **Primitive-fact** (`(1.0 - agg_vf).clamp(0.15, 0.65)`).
pub const ORCHESTRATOR_PIN_PASTE_FRACTION: f32 = 0.65;

/// Orchestrator mix pin — air-void specific surface (mm⁻¹).
/// Class: **Primitive-fact** (orchestrator L529).
pub const ORCHESTRATOR_PIN_AIR_VOID_SURFACE: f32 = 35.0;

/// ASTM C666 severe exposure target air content (%).
/// Class: **Primitive-fact** (orchestrator L530).
pub const ORCHESTRATOR_PIN_REQUIRED_AIR_PCT: f32 = 6.0;

/// Orchestrator bulk-path w/c for chloride transport leg.
/// Class: **Primitive-fact** (aligned with strength/creep pins).
pub const ORCHESTRATOR_PIN_WC: f32 = 0.45;

/// Orchestrator hydration degree for transport + healing legs.
/// Class: **Primitive-fact** (aligned with strength pin).
pub const ORCHESTRATOR_PIN_ALPHA: f32 = 0.75;

/// Internal RH for self-heal leg on bulk path.
/// Class: **Primitive-fact** (orchestrator L548).
pub const ORCHESTRATOR_PIN_INTERNAL_RH: f32 = 0.92;

/// Reference chloride diffusivity (m²/s) — Life-365 nominal at 28 d.
/// Class: **Primitive-fact** (transport closure input, not fitted).
pub const ORCHESTRATOR_PIN_REF_DIFFUSIVITY: f32 = 1.0e-12;

/// Identifies which pathway leg governs the weakest-link composite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathwayLeg {
    /// Powers spacing + air-content frost durability.
    FrostPowers,
    /// Tang–Nilsson chloride ingress resistance.
    ChlorideTransport,
    /// Edvardsen autogenous healing potential.
    AutogenousHealing,
}

/// Scalar breakdown of the three pathway legs at a single mix point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathwayBreakdown {
    /// Spacing factor (mm) from Powers (1949).
    pub spacing_factor_mm: f32,
    /// Frost durability factor (0–100 scale).
    pub frost_norm: f32,
    /// Chloride ingress resistance (0–1).
    pub chloride_resistance: f32,
    /// Autogenous healing potential (0–1).
    pub healing_potential: f32,
    /// Weakest-link composite index (0–100).
    pub composite_index: f32,
    /// Pathway leg that governs the composite (tie → frost < chloride < healing order).
    pub governing_leg: PathwayLeg,
}

/// Composite durability outcome — three pathway tensors plus weakest-link index.
#[derive(Debug, Clone)]
pub struct DurabilityOutcome<B: Backend> {
    /// Powers spacing factor (mm).
    pub spacing_factor_mm: Tensor<B, 4>,
    /// Freeze–thaw durability factor (0–100 scale).
    pub frost_durability_factor: Tensor<B, 4>,
    /// Chloride ingress resistance (0–1, higher is better).
    pub chloride_resistance: Tensor<B, 4>,
    /// Autogenous healing potential (0–1).
    pub healing_potential: Tensor<B, 4>,
    /// Weakest-link composite index (0–100).
    pub composite_index: Tensor<B, 4>,
}

/// Pure tensor composite durability engine — §E Constitutive-Equations.md.
/// formal_anchor: empirical://datasets/dataset_d1.csv
/// formal_status: Empirical
/// formal_axioms: NONE
/// formal_dataset: "uci_concrete_yeh_1998"
/// formal_citation: "Powers (1949) frost · Tang & Nilsson (1992) chloride · Edvardsen (1999) healing"
/// formal_envelope: "Composite weakest-link index exercised under durability witness @ AC105"
pub struct DurabilityEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend<FloatElem = f32>> DurabilityEngine<B> {
    /// Spacing effectiveness from Powers spacing factor (ASTM C666: L ≤ 0.20 mm ideal).
    fn spacing_effectiveness(spacing_factor: Tensor<B, 4>) -> Tensor<B, 4> {
        let good_spacing = spacing_factor.clone().lower_equal_elem(0.2_f32);
        let mid_spacing = bool_and(
            spacing_factor.clone().greater_elem(0.2_f32),
            spacing_factor.clone().lower_equal_elem(0.4_f32),
        );
        let eff_mid = spacing_factor
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.2_f32)
            .mul_scalar(-0.5_f32)
            .add_scalar(0.9_f32);
        let eff_poor = spacing_factor
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(0.4_f32)
            .clamp_max(0.5_f32)
            .mul_scalar(-0.6_f32)
            .add_scalar(0.7_f32)
            .clamp_min(0.0_f32);
        spacing_factor
            .clone()
            .zeros_like()
            .mask_fill(good_spacing, 1.0_f32)
            .add(
                spacing_factor
                    .clone()
                    .zeros_like()
                    .mask_fill(mid_spacing, 1.0_f32)
                    .mul(eff_mid),
            )
            .add(
                spacing_factor
                    .clone()
                    .zeros_like()
                    .mask_fill(spacing_factor.greater_elem(0.4_f32), 1.0_f32)
                    .mul(eff_poor),
            )
    }

    /// Frost durability leg — spacing from `FreezeThawEngine`; air effectiveness with
    /// corrected `mask_fill` semantics (upstream `freeze_thaw.rs` zeros inadequate-air path).
    fn frost_durability_pathway(
        air_fraction: Tensor<B, 4>,
        paste_fraction: Tensor<B, 4>,
        air_void_specific_surface: Tensor<B, 4>,
        required_air_pct: f32,
    ) -> (Tensor<B, 4>, Tensor<B, 4>) {
        let (spacing_factor_mm, _) = FreezeThawEngine::<B>::compute_durability(
            air_fraction.clone(),
            paste_fraction,
            air_void_specific_surface,
            required_air_pct,
        );

        let air_content_pct = air_fraction.mul_scalar(100.0_f32);
        let adequate_air = air_content_pct.clone().greater_equal_elem(required_air_pct);
        let air_effectiveness = air_content_pct
            .div_scalar(required_air_pct)
            .sqrt()
            .clamp_max(1.0_f32);
        // Where adequate: 1.0; else partial credit from √(air/required).
        let final_air_eff = air_effectiveness.mask_fill(adequate_air, 1.0_f32);
        let spacing_eff = Self::spacing_effectiveness(spacing_factor_mm.clone());
        let frost_durability_factor = final_air_eff.mul(spacing_eff).mul_scalar(100.0_f32);

        (spacing_factor_mm, frost_durability_factor)
    }

    /// Computes capillary porosity → chloride diffusivity → ingress resistance \([0,1]\).
    fn chloride_resistance(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        ref_diffusivity: Tensor<B, 4>,
    ) -> Tensor<B, 4> {
        let phi_c = TransportEngine::<B>::compute_capillary_porosity(wc_ratio, degree_hydration);
        let ref_safe = ref_diffusivity.clone().clamp_min(1.0e-20_f32);
        let diffusivity =
            TransportEngine::<B>::compute_chloride_diffusivity(phi_c, ref_diffusivity);
        // Normalize against reference: D/D_ref → resistance = 1 / (1 + D/D_ref).
        let ratio = diffusivity.div(ref_safe);
        ratio.add_scalar(1.0_f32).recip()
    }

    /// Identifies the governing pathway leg from scalar leg values (0–1 normalized).
    pub fn identify_governing_leg(
        frost_norm: f32,
        chloride_resistance: f32,
        healing_potential: f32,
    ) -> PathwayLeg {
        let min_val = frost_norm.min(chloride_resistance).min(healing_potential);
        // Tie-break: frost < chloride < healing (most conservative for exposure).
        if (frost_norm - min_val).abs() < 1e-6 {
            PathwayLeg::FrostPowers
        } else if (chloride_resistance - min_val).abs() < 1e-6 {
            PathwayLeg::ChlorideTransport
        } else {
            PathwayLeg::AutogenousHealing
        }
    }

    /// Scalar pathway breakdown from a computed outcome (for witness / probe).
    pub fn pathway_breakdown(outcome: &DurabilityOutcome<B>) -> PathwayBreakdown {
        let spacing = outcome.spacing_factor_mm.clone().into_data().value[0];
        let frost_raw = outcome.frost_durability_factor.clone().into_data().value[0];
        let frost_norm = (frost_raw / 100.0_f32).clamp(0.0_f32, 1.0_f32);
        let chloride = outcome.chloride_resistance.clone().into_data().value[0];
        let healing = outcome.healing_potential.clone().into_data().value[0];
        let composite = outcome.composite_index.clone().into_data().value[0];
        PathwayBreakdown {
            spacing_factor_mm: spacing,
            frost_norm,
            chloride_resistance: chloride,
            healing_potential: healing,
            composite_index: composite,
            governing_leg: Self::identify_governing_leg(frost_norm, chloride, healing),
        }
    }

    /// Composite weakest-link durability index across frost, chloride, and healing pathways.
    ///
    /// # Arguments
    /// * `air_fraction` — entrained air volume fraction (0.0–0.15)
    /// * `paste_fraction` — cement paste volume fraction (0.15–0.65)
    /// * `air_void_specific_surface` — air-void surface (mm⁻¹, typically 25–45)
    /// * `required_air_pct` — target air content for exposure class (%)
    /// * `wc_ratio` — water/cement ratio
    /// * `degree_hydration` — hydration degree α (0–1)
    /// * `ref_diffusivity` — reference chloride diffusivity (m²/s)
    /// * `internal_rh` — internal relative humidity (0–1)
    /// * `nano_dosage` — nano-silica dosage (kg/m³ proxy)
    pub fn compute_composite(
        air_fraction: Tensor<B, 4>,
        paste_fraction: Tensor<B, 4>,
        air_void_specific_surface: Tensor<B, 4>,
        required_air_pct: f32,
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        ref_diffusivity: Tensor<B, 4>,
        internal_rh: Tensor<B, 4>,
        nano_dosage: Tensor<B, 4>,
    ) -> DurabilityOutcome<B> {
        let (spacing_factor_mm, frost_durability_factor) = Self::frost_durability_pathway(
            air_fraction,
            paste_fraction,
            air_void_specific_surface,
            required_air_pct,
        );

        let chloride_resistance = Self::chloride_resistance(
            wc_ratio,
            degree_hydration.clone(),
            ref_diffusivity,
        );

        let healing_potential = SelfHealEngine::<B>::compute_healing_potential(
            degree_hydration,
            internal_rh,
            nano_dosage,
        );

        // Weakest-link: min(frost/100, chloride_resistance, healing) × 100.
        let frost_norm = frost_durability_factor
            .clone()
            .div_scalar(100.0_f32)
            .clamp(0.0_f32, 1.0_f32);
        let pathway_min = frost_norm
            .min_pair(chloride_resistance.clone())
            .min_pair(healing_potential.clone());
        let composite_index = pathway_min.mul_scalar(100.0_f32);

        DurabilityOutcome {
            spacing_factor_mm,
            frost_durability_factor,
            chloride_resistance,
            healing_potential,
            composite_index,
        }
    }
}
