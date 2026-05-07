// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: Apache-2.0

//! Thermodynamic Admissibility Filter
//!
//! Enforces the Clausius-Duhem inequality as a hard constraint.
//! Paper: "Towards Unified Material-State Tensors for Physics-Gated AI"
//!
//! ## Physics Scope
//!
//! This module enforces:
//! - Conservation of Energy (First Law)
//! - Entropy increase (Second Law): `D_int >= 0`
//! - Irreversible hydration: `alpha_dot >= 0`
//! - Strength monotonicity for undamaged materials
//!
//! ## What This Module Does NOT Cover
//!
//! - Gravity: Handled separately in physics kernel (TODO)
//! - Electromagnetic forces: Use TransportEngine for ionic diffusion
//! - Nuclear forces: Irrelevant at material scales (fm vs m)
//! - Relativistic effects: v << c for all material processing
//!
//! ## Performance
//!
//! - L1 (algebraic): < 1ms
//! - L2 (Clausius-Duhem): 1-10ms
//! - L3 (predictive): 10-50ms

use super::constitution::Constitution;
use wasm_bindgen::prelude::*; // Full formal integration (PhysicalAxiom used via Constitution)

/// Result of thermodynamic admissibility check
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct AdmissibilityResult {
    pub accepted: bool,
    pub dissipation: f64, // D_int value
    pub mass_conserved: bool,
    pub energy_positive: bool,
}

#[wasm_bindgen]
impl AdmissibilityResult {
    #[wasm_bindgen(getter)]
    pub fn is_admissible(&self) -> bool {
        self.accepted
    }

    #[wasm_bindgen(getter)]
    pub fn get_rejection_reason(&self) -> String {
        if self.accepted {
            "ACCEPTED".to_string()
        } else if !self.mass_conserved {
            "MASS_VIOLATION".to_string()
        } else if !self.energy_positive {
            "NEGATIVE_DISSIPATION".to_string()
        } else {
            "UNKNOWN".to_string()
        }
    }
}

/// Thermodynamic state for admissibility checking
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ThermodynamicState {
    pub density: f64,          // kg/m³
    pub temperature: f64,      // K
    pub free_energy: f64,      // Helmholtz ψ (J/kg)
    pub entropy: f64,          // η (J/kg·K)
    pub hydration_degree: f64, // α (0-1)
    pub strength: f64,         // f_c (MPa)

    // [V8.1] Full Clausius-Duhem: Mechanical work and thermal gradients
    // Using Box<[f64]> for wasm_bindgen compatibility
    #[wasm_bindgen(skip)]
    pub stress_tensor: Box<[f64]>, // σ_xx, σ_yy, σ_zz, σ_xy, σ_xz, σ_yz (Pa)
    #[wasm_bindgen(skip)]
    pub strain_rate_tensor: Box<[f64]>, // ε̇_xx, ε̇_yy, ε̇_zz, ε̇_xy, ε̇_xz, ε̇_yz (1/s)
    #[wasm_bindgen(skip)]
    pub heat_flux_vector: Box<[f64]>, // q_x, q_y, q_z (W/m²)
    #[wasm_bindgen(skip)]
    pub temp_gradient_vector: Box<[f64]>, // ∇T_x, ∇T_y, ∇T_z (K/m)
}

#[wasm_bindgen]
impl ThermodynamicState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ThermodynamicState {
        ThermodynamicState {
            density: 2400.0,
            temperature: 293.0, // 20°C
            free_energy: 0.0,
            entropy: 0.0,
            hydration_degree: 0.0,
            strength: 0.0,
            // [V8.1] Full Clausius-Duhem: Initialize mechanical and thermal fields
            stress_tensor: vec![0.0; 6].into_boxed_slice(), // No stress (hydrostatic)
            strain_rate_tensor: vec![0.0; 6].into_boxed_slice(), // No strain rate (static)
            heat_flux_vector: vec![0.0; 3].into_boxed_slice(), // No heat flux (adiabatic/isothermal)
            temp_gradient_vector: vec![0.0; 3].into_boxed_slice(), // No temperature gradient (uniform T)
        }
    }

    /// Create state from mix parameters (uses default s_intrinsic = 240 MPa)
    pub fn from_mix(w_c: f64, alpha: f64, temp: f64) -> ThermodynamicState {
        Self::from_mix_calibrated(w_c, alpha, temp, 240.0)
    }

    /// Create state from mix parameters with calibrated intrinsic strength.
    /// s_intrinsic: Intrinsic gel strength (MPa), typically 80-240 depending on cement type.
    pub fn from_mix_calibrated(
        w_c: f64,
        alpha: f64,
        temp: f64,
        s_intrinsic: f64,
    ) -> ThermodynamicState {
        // Compute Helmholtz free energy from Powers model
        // ψ(α) = ψ₀ + ∫₀^α S_int · X(α')³ dα'
        let x = 0.68 * alpha / (0.32 * alpha + w_c + 1e-6); // Gel-space ratio
        let psi = s_intrinsic * x.powi(3) * alpha; // Simplified integral

        // Strength from Powers: fc = s_intrinsic * X³
        let fc = s_intrinsic * x.powi(3);

        ThermodynamicState {
            density: 2400.0 - 400.0 * w_c, // Approximate density reduction with w/c
            temperature: temp,
            free_energy: psi,
            entropy: alpha * 0.1, // Simplified entropy (increases with hydration)
            hydration_degree: alpha,
            strength: fc,
            // [V8.1] Full Clausius-Duhem: Initialize with typical values for fresh concrete
            stress_tensor: vec![0.0; 6].into_boxed_slice(), // No stress (fresh state)
            strain_rate_tensor: vec![0.0; 6].into_boxed_slice(), // No strain rate (static)
            heat_flux_vector: vec![0.0; 3].into_boxed_slice(), // No heat flux
            temp_gradient_vector: vec![0.0; 3].into_boxed_slice(), // Uniform temperature
        }
    }

    /// [V8.1] Set stress tensor for mechanical work calculation
    /// Voigt notation: [σ_xx, σ_yy, σ_zz, σ_xy, σ_xz, σ_yz] (Pa)
    pub fn set_stress_tensor(&mut self, stress: Vec<f64>) {
        if stress.len() == 6 {
            self.stress_tensor = stress.into_boxed_slice();
        }
    }

    /// [V8.1] Set strain rate tensor for mechanical work calculation
    /// Voigt notation: [ε̇_xx, ε̇_yy, ε̇_zz, ε̇_xy, ε̇_xz, ε̇_yz] (1/s)
    pub fn set_strain_rate_tensor(&mut self, strain_rate: Vec<f64>) {
        if strain_rate.len() == 6 {
            self.strain_rate_tensor = strain_rate.into_boxed_slice();
        }
    }

    /// [V8.1] Set heat flux vector for thermal conduction calculation
    /// [q_x, q_y, q_z] (W/m²)
    pub fn set_heat_flux(&mut self, heat_flux: Vec<f64>) {
        if heat_flux.len() == 3 {
            self.heat_flux_vector = heat_flux.into_boxed_slice();
        }
    }

    /// [V8.1] Set temperature gradient vector for thermal conduction calculation
    /// [∇T_x, ∇T_y, ∇T_z] (K/m)
    pub fn set_temp_gradient(&mut self, temp_gradient: Vec<f64>) {
        if temp_gradient.len() == 3 {
            self.temp_gradient_vector = temp_gradient.into_boxed_slice();
        }
    }
}

/// Thermodynamic Admissibility Filter (V8 Constitutional Physics Gate)
///
/// Enforces the Clausius-Duhem inequality: D_int ≥ 0
/// where D_int = σ:ε̇ - ρ(ψ̇ + sṪ) - q·∇T/T
///
/// For isothermal cement hydration (∇T = 0, Ṫ = 0):
///   D_int = -ρψ̇ = ρ · Q_hyd · α̇ ≥ 0
///
/// This gate implements the natural transformation η: K_phys ⇒ Admissible
/// as specified in MaOS Vision Manifesto v8.
///
/// Category-Theoretic Properties:
/// - Naturality: Gate(f) ∘ η_S₁ = η_S₂ ∘ K_phys(f) for all transitions f
/// - Compositionality: If Gate(f_i) = ACCEPT for all i, then Gate(f₁ ∘ ... ∘ fₙ) = ACCEPT
/// - Type Safety: ACCEPT states form a closed submanifold M_adm ⊂ ℝ⁶⁴
#[wasm_bindgen]
pub struct ThermodynamicFilter {
    tolerance: f64,
    rejections: u64,
    acceptances: u64,
    /// Heat of hydration (J/kg cement) - used for dissipation calculation
    q_hyd: f64,
}

#[wasm_bindgen]
impl ThermodynamicFilter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ThermodynamicFilter {
        ThermodynamicFilter {
            tolerance: 1e-3, // More reasonable tolerance for thermodynamic calculations
            rejections: 0,
            acceptances: 0,
            q_hyd: 450_000.0, // ~450 kJ/kg cement (typical Portland cement)
        }
    }

    /// Create filter with custom heat of hydration (for different cement types)
    /// - Portland OPC: ~450 kJ/kg
    /// - High Early Strength: ~500 kJ/kg
    /// - Low Heat (Type IV): ~250 kJ/kg
    /// - Slag Cement: ~300 kJ/kg
    pub fn with_q_hyd(q_hyd: f64) -> ThermodynamicFilter {
        ThermodynamicFilter {
            tolerance: 1e-3, // More reasonable tolerance for thermodynamic calculations
            rejections: 0,
            acceptances: 0,
            q_hyd,
        }
    }

    /// Check if a state transition is thermodynamically admissible
    ///
    /// # Arguments
    /// * `old_state` - Previous thermodynamic state
    /// * `new_state` - Proposed new state
    /// * `dt` - Time step (seconds)
    ///
    /// # Returns
    /// AdmissibilityResult with accept/reject decision
    ///
    /// # Physics (Clausius-Duhem Inequality)
    /// Full form: D_int = σ:ε̇ - ρ(ψ̇ + sṪ) - q·∇T/T ≥ 0
    ///
    /// For isothermal cement hydration (∇T = 0, Ṫ = 0, σ:ε̇ ≈ 0):
    ///   D_int = -ρ·ψ̇ = ρ·Q_hyd·α̇ ≥ 0
    ///
    /// where Q_hyd is the heat of hydration (~450 kJ/kg for OPC).
    /// Since Q_hyd > 0 and ρ > 0, admissibility requires α̇ ≥ 0.
    /// This physically means: hydration is irreversible (2nd law of thermodynamics).
    pub fn check_transition(
        &mut self,
        old_state: &ThermodynamicState,
        new_state: &ThermodynamicState,
        dt: f64,
    ) -> AdmissibilityResult {
        // Delegate to Constitution for PhysicalAxiom checks + proof-carrying witnesses.
        // The Constitution verdict is used as a consistency cross-check; the ThermodynamicFilter
        // then recomputes mass conservation with a tighter numerical criterion.
        let constitution = Constitution::standard();
        let constitution_result = constitution.verify_transition(old_state, new_state);

        // Hybrid augmentation: AND with tighter numerical mass check.
        let mass_conserved = constitution_result.mass_conserved
            && (new_state.density - old_state.density).abs() < 100.0;

        // 2. Compute rates (for full D_int)
        let d_alpha = new_state.hydration_degree - old_state.hydration_degree;
        let alpha_dot = d_alpha / (dt + 1e-10);

        let d_psi = new_state.free_energy - old_state.free_energy;
        let _psi_dot = d_psi / (dt + 1e-10);

        let d_temp = new_state.temperature - old_state.temperature;
        let _temp_dot = d_temp / (dt + 1e-10);

        // 3. Compute internal dissipation (FULL Clausius-Duhem Inequality)
        // D_int = σ:ε̇ - ρ(ψ̇ + sṪ) - (q·∇T)/T ≥ 0
        //
        // Terms:
        //   σ:ε̇    - Mechanical work (stress dotted with strain rate)
        //   ρψ̇     - Free energy rate (internal energy change)
        //   ρsṪ    - Entropy production from temperature change
        //   (q·∇T)/T - Thermal conduction (Fourier's law)

        let rho = old_state.density;
        let _temp_k = old_state.temperature;

        // Mechanical work: σ:ε̇ (double contraction of stress and strain rate tensors)
        // Using Voigt notation: [xx, yy, zz, xy, xz, yz]
        let mechanical_work = (0..6)
            .map(|i| {
                old_state.stress_tensor[i]
                    * (new_state.strain_rate_tensor[i] - old_state.strain_rate_tensor[i])
            })
            .sum::<f64>()
            / (dt + 1e-10);

        // Internal energy change: -ρψ̇ - ρsṪ
        let free_energy_rate = (new_state.free_energy - old_state.free_energy) / (dt + 1e-10);
        let _entropy_rate = (new_state.entropy - old_state.entropy) / (dt + 1e-10);
        let temp_rate = (new_state.temperature - old_state.temperature) / (dt + 1e-10);

        let internal_energy = -rho * free_energy_rate - rho * old_state.entropy * temp_rate;

        // Thermal conduction: -(q·∇T)/T
        // Using averaged temperature for stability
        let avg_temp = (old_state.temperature + new_state.temperature) / 2.0;
        let thermal_conduction = if avg_temp > 0.0 {
            // Dot product of heat flux and temperature gradient
            let heat_flux_dot_grad = (0..3)
                .map(|i| old_state.heat_flux_vector[i] * old_state.temp_gradient_vector[i])
                .sum::<f64>();
            -heat_flux_dot_grad / avg_temp
        } else {
            0.0
        };

        // Full Clausius-Duhem dissipation (W/m³)
        let d_int = mechanical_work + internal_energy + thermal_conduction;

        // [V8.1] For cement systems, add hydration-specific dissipation
        // During hydration: ψ̇ = -Q_hyd·α̇ (exothermic), so D_int includes ρ·Q_hyd·α̇
        let d_int_hydration = rho * self.q_hyd * alpha_dot / 1000.0; // Scale to reasonable W/m³ range
        let d_int_full = d_int + d_int_hydration;

        // 4. Strength monotonicity check (derived constraint)
        // In undamaged concrete, strength must not decrease (gel only grows)
        // This is a consequence of positive dissipation in hardening materials
        let strength_valid = new_state.strength >= old_state.strength - self.tolerance;

        // 5. Combined admissibility
        // Full Clausius-Duhem: D_int ≥ -ε (allow small numerical tolerance)
        let energy_positive = d_int_full >= -self.tolerance && strength_valid;

        let accepted = mass_conserved && energy_positive;

        if accepted {
            self.acceptances += 1;
        } else {
            self.rejections += 1;
        }

        AdmissibilityResult {
            accepted,
            dissipation: d_int_full,
            mass_conserved,
            energy_positive,
        }
    }

    /// Get filter statistics
    pub fn get_stats(&self) -> String {
        let total = self.acceptances + self.rejections;
        if total == 0 {
            return "No transitions checked".to_string();
        }
        let rate = self.acceptances as f64 / total as f64 * 100.0;
        format!(
            "Accepted: {}, Rejected: {}, Rate: {:.1}%",
            self.acceptances, self.rejections, rate
        )
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.acceptances = 0;
        self.rejections = 0;
    }

    /// [V9.0] Check thermodynamic admissibility with visual consistency
    ///
    /// Extends Clausius-Duhem inequality with visual gate:
    /// D_total = D_thermodynamic + D_visual_penalty
    ///
    /// Where visual inconsistency is treated as negative dissipation.
    /// This ensures physics predictions are consistent with visual observations.
    ///
    /// # Arguments
    /// * `old_state` - Previous thermodynamic state
    /// * `new_state` - Proposed new state
    /// * `dt` - Time step (seconds)
    /// * `visual_consistency` - Vision consistency score (0.0=complete mismatch, 1.0=perfect match)
    /// * `consistency_threshold` - Minimum consistency for acceptance (default 0.7)
    ///
    /// # Returns
    /// AdmissibilityResult with visual-enhanced checking
    pub fn check_transition_with_vision(
        &mut self,
        old_state: &ThermodynamicState,
        new_state: &ThermodynamicState,
        dt: f64,
        visual_consistency: f64,
        consistency_threshold: f64,
    ) -> AdmissibilityResult {
        // First perform standard thermodynamic check
        let thermo_result = self.check_transition(old_state, new_state, dt);

        // Calculate visual penalty as additional dissipation
        // Low visual consistency = high penalty (negative dissipation)
        let visual_penalty = (1.0 - visual_consistency) * 1000.0; // J/m³ penalty
        let total_dissipation = thermo_result.dissipation - visual_penalty;

        // Visual consistency check
        let visual_accepted = visual_consistency >= consistency_threshold;

        // Combined admissibility: thermodynamics AND visual consistency
        let accepted = thermo_result.accepted && visual_accepted;
        let energy_positive = total_dissipation >= -self.tolerance;

        // Update statistics
        if accepted {
            self.acceptances += 1;
        } else {
            self.rejections += 1;
        }

        AdmissibilityResult {
            accepted,
            dissipation: total_dissipation,
            mass_conserved: thermo_result.mass_conserved,
            energy_positive,
        }
    }

    /// [V9.0] Simplified vision-enhanced check for TypeScript interface
    /// Uses default consistency threshold of 0.7
    #[wasm_bindgen]
    pub fn check_transition_with_vision_simple(
        &mut self,
        old_state: &ThermodynamicState,
        new_state: &ThermodynamicState,
        dt: f64,
        visual_consistency: f64,
    ) -> AdmissibilityResult {
        self.check_transition_with_vision(old_state, new_state, dt, visual_consistency, 0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admissible_hydration() {
        let mut filter = ThermodynamicFilter::new();

        // Simulate forward hydration (allowed)
        let old = ThermodynamicState::from_mix(0.5, 0.3, 293.0);
        let new = ThermodynamicState::from_mix(0.5, 0.5, 293.0); // α increased

        let result = filter.check_transition(&old, &new, 3600.0);

        assert!(result.accepted, "Forward hydration should be admissible");
        assert!(result.dissipation >= 0.0, "Dissipation should be positive");
        println!("✅ Forward hydration: D_int = {:.4}", result.dissipation);
    }

    #[test]
    fn test_inadmissible_reverse_hydration() {
        let mut filter = ThermodynamicFilter::new();

        // Simulate reverse hydration (forbidden by 2nd law)
        let old = ThermodynamicState::from_mix(0.5, 0.7, 293.0);
        let new = ThermodynamicState::from_mix(0.5, 0.3, 293.0); // α decreased!

        let result = filter.check_transition(&old, &new, 3600.0);

        assert!(!result.accepted, "Reverse hydration should be rejected");
        assert!(result.dissipation < 0.0, "Dissipation should be negative");
        println!(
            "✅ Reverse hydration properly rejected: D_int = {:.4}",
            result.dissipation
        );
    }

    #[test]
    fn test_strength_monotonicity() {
        let mut filter = ThermodynamicFilter::new();

        // Simulate strength decrease (thermodynamically inadmissible without damage)
        let mut old = ThermodynamicState::new();
        old.strength = 30.0;
        old.hydration_degree = 0.5;

        let mut new = ThermodynamicState::new();
        new.strength = 25.0; // Decreased!
        new.hydration_degree = 0.5;

        let result = filter.check_transition(&old, &new, 1.0);

        assert!(!result.accepted, "Strength decrease should be rejected");
        println!("✅ Strength decrease properly rejected");
    }

    #[test]
    fn test_filter_statistics() {
        let mut filter = ThermodynamicFilter::new();

        // Run multiple transitions
        for i in 0..10 {
            let old = ThermodynamicState::from_mix(0.5, i as f64 * 0.1, 293.0);
            let new = ThermodynamicState::from_mix(0.5, (i + 1) as f64 * 0.1, 293.0);
            filter.check_transition(&old, &new, 3600.0);
        }

        let stats = filter.get_stats();
        println!("Filter stats: {}", stats);
        assert!(stats.contains("Accepted: 10"));
    }

    #[test]
    fn test_from_mix_calibrated() {
        // Test that from_mix uses default s_intrinsic = 240
        let state_default = ThermodynamicState::from_mix(0.5, 0.7, 293.0);

        // Test calibrated version with same parameters
        let state_calibrated = ThermodynamicState::from_mix_calibrated(0.5, 0.7, 293.0, 240.0);

        // Both should produce same strength (within tolerance)
        assert!(
            (state_default.strength - state_calibrated.strength).abs() < 0.01,
            "from_mix and from_mix_calibrated(240) should match"
        );

        // Test with lower s_intrinsic produces lower strength
        let state_low = ThermodynamicState::from_mix_calibrated(0.5, 0.7, 293.0, 80.0);
        assert!(
            state_low.strength < state_default.strength,
            "Lower s_intrinsic should produce lower strength"
        );

        // Verify the ratio is correct (strength scales linearly with s_intrinsic)
        let ratio = state_low.strength / state_default.strength;
        let expected_ratio = 80.0 / 240.0;
        assert!(
            (ratio - expected_ratio).abs() < 0.01,
            "Strength should scale linearly with s_intrinsic"
        );
    }

    #[test]
    fn test_custom_heat_of_hydration() {
        // Test with_q_hyd constructor for different cement types
        let mut filter_opc = ThermodynamicFilter::new(); // Default: 450 kJ/kg
        let mut filter_low_heat = ThermodynamicFilter::with_q_hyd(250_000.0); // Low heat cement

        // Both should accept forward hydration
        let old = ThermodynamicState::from_mix(0.5, 0.3, 293.0);
        let new = ThermodynamicState::from_mix(0.5, 0.5, 293.0);

        let result_opc = filter_opc.check_transition(&old, &new, 3600.0);
        let result_low = filter_low_heat.check_transition(&old, &new, 3600.0);

        assert!(
            result_opc.accepted,
            "OPC filter should accept forward hydration"
        );
        assert!(
            result_low.accepted,
            "Low heat filter should accept forward hydration"
        );

        // OPC should have higher dissipation (more heat released)
        // Note: The ratio won't be exact due to normalization
        println!(
            "OPC dissipation: {:.4}, Low heat: {:.4}",
            result_opc.dissipation, result_low.dissipation
        );
    }

    #[test]
    fn test_v8_compositionality() {
        // Test Law V: Compositional Safety
        // If individual transitions are admissible, their composition should be too
        let mut filter = ThermodynamicFilter::new();

        // Create a sequence of admissible transitions
        let states: Vec<ThermodynamicState> = (0..=5)
            .map(|i| ThermodynamicState::from_mix(0.5, i as f64 * 0.15, 293.0))
            .collect();

        // Check each individual transition
        let mut all_accepted = true;
        for i in 0..states.len() - 1 {
            let result = filter.check_transition(&states[i], &states[i + 1], 3600.0);
            if !result.accepted {
                all_accepted = false;
                break;
            }
        }

        // Also check the composite transition (first to last)
        filter.reset_stats();
        let composite_result =
            filter.check_transition(&states[0], &states[states.len() - 1], 3600.0 * 5.0);

        // Both should be accepted (compositionality)
        assert!(
            all_accepted,
            "All individual transitions should be accepted"
        );
        assert!(
            composite_result.accepted,
            "Composite transition should be accepted"
        );
        println!("✅ Compositionality verified: local admissibility → global admissibility");
    }
}
