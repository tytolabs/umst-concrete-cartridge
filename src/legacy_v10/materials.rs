// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//
// MaOS — Material Agnostic Operating System
// God-Grade Material Definitions (Parity with TypeScript Material2)

use serde::{Deserialize, Serialize};

/// Rheology Profile: Fluid dynamics properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RheologyProfile {
    #[serde(rename = "yieldStress")]
    pub yield_stress: f32, // Pascals
    pub viscosity: f32,  // Pa*s
    pub thixotropy: f32, // dimensionless 0-1
    #[serde(rename = "slumpFlow")]
    pub slump_flow: Option<f32>, // mm
}

impl Default for RheologyProfile {
    fn default() -> Self {
        Self {
            yield_stress: 0.0,
            viscosity: 0.0,
            thixotropy: 0.0,
            slump_flow: None,
        }
    }
}

/// Ecology Profile: Environmental Impact
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcologyProfile {
    #[serde(rename = "embodiedCarbon")]
    pub embodied_carbon: f32, // kgCO2e/kg
    pub certification: String, // e.g. "ISO 14001"
}

impl Default for EcologyProfile {
    fn default() -> Self {
        Self {
            embodied_carbon: 0.0,
            certification: "None".to_string(),
        }
    }
}

/// Economy Profile: Cost and Supply Chain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomyProfile {
    #[serde(rename = "costPerKg")]
    pub cost_per_kg: f32, // USD
    pub supplier: String,
    #[serde(rename = "leadTimeDays")]
    pub lead_time_days: Option<u32>,
}

impl Default for EconomyProfile {
    fn default() -> Self {
        Self {
            cost_per_kg: 0.0,
            supplier: "Generic".to_string(),
            lead_time_days: None,
        }
    }
}

/// Provenance Profile: Traceability
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceProfile {
    pub origin: String, // e.g. "Chennai, IN"
    #[serde(rename = "batchId")]
    pub batch_id: String,
    #[serde(rename = "productionDate")]
    pub production_date: Option<String>,
}

impl Default for ProvenanceProfile {
    fn default() -> Self {
        Self {
            origin: "Unknown".to_string(),
            batch_id: "N/A".to_string(),
            production_date: None,
        }
    }
}
