// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//
// MaOS — Bayesian Calibrator: Learning from Experimental Errors (AP04)
//
// This file is part of MaOS, developed by Santhosh Shyamsundar & Santosh Prabhu Shenbagamoorthy.
// For licensing terms, see the LICENSE file in the project root.

//! Bayesian Calibrator: Learning from Experimental Errors (AP04)
//!
//! Implements Bayesian inference to update physics model parameters based on
//! the difference between predicted and actual experimental results.
//!
//! This enables the system to learn from mistakes and improve prediction accuracy
//! over successive experimental batches.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Internal struct for computation (not exposed to WASM)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationDataPoint {
    pub batch_id: String,
    pub level: String,
    pub predicted_f28_compressive: f32,
    pub predicted_slump_flow: f32,
    pub predicted_yield_stress: f32,
    pub actual_f28_compressive: Option<f32>,
    pub actual_slump_flow: Option<f32>,
    pub actual_yield_stress: Option<f32>,
    pub timestamp: f64,
    pub age_days: f32,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationParameters {
    pub yield_stress_correction: f32,
    pub slump_flow_correction: f32,
    pub strength_correction: f32,
    pub maturity_correction: f32,
    pub density_correction: f32,
    pub yield_stress_variance: f32,
    pub strength_variance: f32,
    pub slump_variance: f32,
}

#[wasm_bindgen]
pub struct BayesianCalibrator {
    calibration_data: Vec<CalibrationDataPoint>,
    parameters: CalibrationParameters,
    priors: CalibrationParameters,
}

#[wasm_bindgen]
impl BayesianCalibrator {
    /// Create a new BayesianCalibrator instance
    #[wasm_bindgen]
    pub fn create() -> BayesianCalibrator {
        let priors = CalibrationParameters {
            yield_stress_correction: 1.0,
            slump_flow_correction: 0.0,
            strength_correction: 1.0,
            maturity_correction: 1.0,
            density_correction: 1.0,
            yield_stress_variance: 25.0, // Pa²
            strength_variance: 4.0,      // MPa²
            slump_variance: 400.0,       // mm²
        };

        BayesianCalibrator {
            calibration_data: Vec::new(),
            parameters: priors.clone(),
            priors,
        }
    }

    /// Add a new calibration data point from experimental results
    #[wasm_bindgen]
    pub fn add_calibration_point(&mut self, data_point: JsValue) -> Result<(), JsValue> {
        let point: CalibrationDataPoint = serde_wasm_bindgen::from_value(data_point)?;

        self.calibration_data.push(point);

        // Keep only last 50 data points for computational efficiency
        if self.calibration_data.len() > 50 {
            self.calibration_data = self
                .calibration_data
                .split_off(self.calibration_data.len() - 50);
        }

        // Update parameters using Bayesian inference
        self.update_parameters();

        Ok(())
    }

    /// Apply calibrated corrections to physics predictions
    #[wasm_bindgen]
    pub fn apply_calibration(&self, physics_result: JsValue) -> Result<JsValue, JsValue> {
        let mut result: serde_json::Value = serde_wasm_bindgen::from_value(physics_result)?;

        // Apply rheology corrections
        if let Some(fresh) = result.get_mut("fresh") {
            if let Some(fresh_obj) = fresh.as_object_mut() {
                if let Some(yield_stress) = fresh_obj.get_mut("yieldStress") {
                    if let Some(ys_val) = yield_stress.as_f64() {
                        *yield_stress = serde_json::Value::from(
                            ys_val as f32 * self.parameters.yield_stress_correction,
                        );
                    }
                }
                if let Some(slump_flow) = fresh_obj.get_mut("slumpFlow") {
                    if let Some(sf_val) = slump_flow.as_f64() {
                        let corrected = sf_val as f32 + self.parameters.slump_flow_correction;
                        *slump_flow = serde_json::Value::from(corrected.max(200.0));
                        // Physical bounds
                    }
                }
            }
        }

        // Apply strength corrections
        if let Some(hardened) = result.get_mut("hardened") {
            if let Some(hardened_obj) = hardened.as_object_mut() {
                if let Some(f28) = hardened_obj.get_mut("f28_compressive") {
                    if let Some(f28_val) = f28.as_f64() {
                        *f28 = serde_json::Value::from(
                            f28_val as f32 * self.parameters.strength_correction,
                        );
                    }
                }
                if let Some(maturity) = hardened_obj.get_mut("maturityIndex") {
                    if let Some(maturity_val) = maturity.as_f64() {
                        *maturity = serde_json::Value::from(
                            maturity_val as f32 * self.parameters.maturity_correction,
                        );
                    }
                }
            }
        }

        // Apply packing corrections
        if let Some(packing) = result.get_mut("packing") {
            if let Some(packing_obj) = packing.as_object_mut() {
                if let Some(density) = packing_obj.get_mut("density") {
                    if let Some(density_val) = density.as_f64() {
                        *density = serde_json::Value::from(
                            density_val as f32 * self.parameters.density_correction,
                        );
                    }
                }
            }
        }

        Ok(serde_wasm_bindgen::to_value(&result)?)
    }

    /// Get current calibration parameters
    #[wasm_bindgen]
    pub fn get_parameters(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.parameters)?)
    }

    /// Get calibration data count
    #[wasm_bindgen]
    pub fn get_data_point_count(&self) -> usize {
        self.calibration_data.len()
    }

    /// Bayesian parameter update using experimental data
    fn update_parameters(&mut self) {
        if self.calibration_data.len() < 3 {
            // Need minimum data for meaningful updates
            return;
        }

        // Calculate residuals (prediction errors)
        let mut yield_stress_residuals = Vec::new();
        let mut strength_residuals = Vec::new();
        let mut slump_residuals = Vec::new();

        for point in &self.calibration_data {
            if let (Some(actual_ys), predicted_ys) =
                (point.actual_yield_stress, point.predicted_yield_stress)
            {
                yield_stress_residuals.push(actual_ys - predicted_ys);
            }

            if let (Some(actual_f28), predicted_f28) = (
                point.actual_f28_compressive,
                point.predicted_f28_compressive,
            ) {
                strength_residuals.push(actual_f28 - predicted_f28);
            }

            if let (Some(actual_slump), predicted_slump) =
                (point.actual_slump_flow, point.predicted_slump_flow)
            {
                slump_residuals.push(actual_slump - predicted_slump);
            }
        }

        // Bayesian updates for each parameter
        if !yield_stress_residuals.is_empty() {
            self.update_yield_stress_parameter(&yield_stress_residuals);
        }

        if !strength_residuals.is_empty() {
            self.update_strength_parameter(&strength_residuals);
        }

        if !slump_residuals.is_empty() {
            self.update_slump_parameter(&slump_residuals);
        }
    }

    fn update_yield_stress_parameter(&mut self, residuals: &[f32]) {
        let mean_residual: f32 = residuals.iter().sum::<f32>() / residuals.len() as f32;
        let learning_rate = 0.01; // Conservative learning rate

        // Update correction factor
        let correction = 1.0 + (mean_residual / 100.0) * learning_rate; // Normalize by typical yield stress
        self.parameters.yield_stress_correction *= correction;

        // Keep within reasonable bounds
        self.parameters.yield_stress_correction =
            self.parameters.yield_stress_correction.max(0.5).min(2.0);

        // Update variance estimate
        let variance: f32 = residuals.iter().map(|r| r * r).sum::<f32>() / residuals.len() as f32;
        self.parameters.yield_stress_variance = variance;
    }

    fn update_strength_parameter(&mut self, residuals: &[f32]) {
        let mean_residual: f32 = residuals.iter().sum::<f32>() / residuals.len() as f32;
        let learning_rate = 0.005; // More conservative for strength

        let correction = 1.0 + (mean_residual / 10.0) * learning_rate; // Normalize by typical strength
        self.parameters.strength_correction *= correction;

        self.parameters.strength_correction = self.parameters.strength_correction.max(0.7).min(1.5);

        let variance: f32 = residuals.iter().map(|r| r * r).sum::<f32>() / residuals.len() as f32;
        self.parameters.strength_variance = variance;
    }

    fn update_slump_parameter(&mut self, residuals: &[f32]) {
        let mean_residual: f32 = residuals.iter().sum::<f32>() / residuals.len() as f32;
        let learning_rate = 0.1; // More aggressive for slump (easier to adjust)

        // Direct additive correction
        self.parameters.slump_flow_correction += mean_residual * learning_rate;

        // Keep within reasonable bounds
        self.parameters.slump_flow_correction =
            self.parameters.slump_flow_correction.max(-50.0).min(50.0);

        let variance: f32 = residuals.iter().map(|r| r * r).sum::<f32>() / residuals.len() as f32;
        self.parameters.slump_variance = variance;
    }

    /// Calculate prediction confidence intervals
    #[wasm_bindgen]
    pub fn get_confidence_intervals(&self, prediction: JsValue) -> Result<JsValue, JsValue> {
        let pred: serde_json::Value = serde_wasm_bindgen::from_value(prediction)?;

        let ys_pred = pred["fresh"]["yieldStress"].as_f64().unwrap_or(0.0) as f32;
        let strength_pred = pred["hardened"]["f28_compressive"].as_f64().unwrap_or(0.0) as f32;
        let slump_pred = pred["fresh"]["slumpFlow"].as_f64().unwrap_or(0.0) as f32;

        let confidence_intervals = serde_json::json!({
            "yieldStress": {
                "lower": (ys_pred * (self.parameters.yield_stress_correction - 2.0 * self.parameters.yield_stress_variance.sqrt())) as f64,
                "upper": (ys_pred * (self.parameters.yield_stress_correction + 2.0 * self.parameters.yield_stress_variance.sqrt())) as f64
            },
            "strength": {
                "lower": (strength_pred * (self.parameters.strength_correction - 2.0 * self.parameters.strength_variance.sqrt())) as f64,
                "upper": (strength_pred * (self.parameters.strength_correction + 2.0 * self.parameters.strength_variance.sqrt())) as f64
            },
            "slump": {
                "lower": ((slump_pred + self.parameters.slump_flow_correction) - 2.0 * self.parameters.slump_variance.sqrt()).max(200.0) as f64,
                "upper": ((slump_pred + self.parameters.slump_flow_correction) + 2.0 * self.parameters.slump_variance.sqrt()) as f64
            }
        });

        Ok(serde_wasm_bindgen::to_value(&confidence_intervals)?)
    }

    /// Get prior calibration parameters
    #[wasm_bindgen]
    pub fn get_priors(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(&self.priors)?)
    }

    /// Reset to factory defaults
    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.parameters = CalibrationParameters {
            yield_stress_correction: 1.0,
            slump_flow_correction: 0.0,
            strength_correction: 1.0,
            maturity_correction: 1.0,
            density_correction: 1.0,
            yield_stress_variance: 25.0,
            strength_variance: 4.0,
            slump_variance: 400.0,
        };
        self.calibration_data.clear();
        Ok(())
    }

    /// Export calibration data for analysis
    #[wasm_bindgen]
    pub fn export_calibration_data(&self) -> Result<JsValue, JsValue> {
        let statistics = serde_json::json!({
            "totalDataPoints": self.calibration_data.len(),
            "yieldStressDataPoints": self.calibration_data.iter().filter(|d| d.actual_yield_stress.is_some()).count(),
            "strengthDataPoints": self.calibration_data.iter().filter(|d| d.actual_f28_compressive.is_some()).count(),
            "slumpDataPoints": self.calibration_data.iter().filter(|d| d.actual_slump_flow.is_some()).count(),
            "lastUpdate": js_sys::Date::now()
        });

        let export_data = serde_json::json!({
            "parameters": self.parameters,
            "dataPoints": self.calibration_data,
            "statistics": statistics
        });

        Ok(serde_wasm_bindgen::to_value(&export_data)?)
    }
}
