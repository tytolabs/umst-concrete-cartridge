<!-- SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->

# Formal anchors (`src/`)

This inventory lists every **`pub`** function, struct, enum, and trait across `src/**/*.rs`, with the `formal_anchor:` line documented on that symbol.

Regression: `cargo test --test formal_anchors`.

| Symbol | Kind | Location | formal_anchor |
|--------|------|-----------|---------------|
| `bool_and` | `fn` | `src/burn_compat.rs` | NONE |
| `ModelKind` | `enum` | `src/calibration.rs` | lean://umst-formal/Lean/Constitutional.lean#kleisliComposeWellTypedN |
| `PowersGelParameters` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/Powers.lean#PowersState |
| `FormalBlock` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/Gate.lean#Admissible |
| `ProvenanceFormal` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy |
| `CalibrationMeta` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/Powers.lean#S_intrinsic |
| `CalibrationProvenance` | `struct` | `src/calibration.rs` | NONE |
| `RegimeBounds` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration |
| `CalibrationModelSection` | `struct` | `src/calibration.rs` | NONE |
| `AcceptanceBlock` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/OrderStatisticsBand.lean#p25_p75_admissibility |
| `ContractBlock` | `struct` | `src/calibration.rs` | NONE |
| `Profile` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/Naturality.lean#gateMaterialAgnostic |
| `RegimeViolation` | `struct` | `src/calibration.rs` | lean://umst-formal/Lean/Gate.lean#Admissible |
| `CalibrationError` | `enum` | `src/calibration.rs` | NONE |
| `load_bundled` | `fn` | `src/calibration.rs` | lean://umst-formal/Lean/Constitutional.lean#kleisliCompose |
| `load_from_path` | `fn` | `src/calibration.rs` | lean://umst-formal/Lean/LandauerLaw.lean#ErasureProcess |
| `regime_check_scalars` | `fn` | `src/calibration.rs` | lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration |
| `any_bundled_profile_covers_scalars` | `fn` | `src/calibration.rs` | lean://umst-formal/Lean/Naturality.lean#naturalitySquare |
| `profile_descriptions` | `fn` | `src/calibration.rs` | lean://umst-formal/Lean/Constitutional.lean#KleisliArrow |
| `PredictionWireVersion` | `enum` | `src/cli/mod.rs` | lean://umst-formal/Lean/Naturality.lean#gateMaterialAgnostic |
| `WaterCementRatio` | `struct` | `src/cli/mod.rs` | NONE |
| `value` | `fn` | `src/cli/mod.rs` | NONE |
| `TemperatureK` | `struct` | `src/cli/mod.rs` | NONE |
| `value` | `fn` | `src/cli/mod.rs` | NONE |
| `MixSpec` | `struct` | `src/cli/mod.rs` | lean://umst-formal/Lean/Gate.lean#Admissible |
| `MixSpecError` | `enum` | `src/cli/mod.rs` | NONE |
| `CliError` | `enum` | `src/cli/mod.rs` | lean://umst-formal/Lean/Naturality.lean#naturalitySquare |
| `PredictBundle` | `struct` | `src/cli/mod.rs` | lean://umst-formal/Lean/Naturality.lean#gateMaterialAgnostic |
| `predict` | `fn` | `src/cli/mod.rs` | lean://umst-formal/Lean/Powers.lean#powers_monotone |
| `serialize_prediction` | `fn` | `src/cli/mod.rs` | lean://umst-formal/Lean/MeasurementCost.lean#zero_info_zero_energy |
| `CertifyChain` | `struct` | `src/cli/mod.rs` | lean://umst-formal/Lean/Constitutional.lean#kleisliCompose |
| `certify_profile_json` | `fn` | `src/cli/mod.rs` | lean://umst-formal/Lean/Constitutional.lean#kleisliComposeWellTypedN |
| `serialize_mix_spec` | `fn` | `src/cli/mod.rs` | NONE |
| `OptimizeField` | `enum` | `src/cli/mod.rs` | NONE |
| `parse_optimize_target` | `fn` | `src/cli/mod.rs` | lean://umst-formal/Lean/OrderStatisticsBand.lean#order_statistic_concentration |
| `optimize_mix` | `fn` | `src/cli/mod.rs` | lean://umst-formal/Lean/Powers.lean#powers_monotone |
| `ConcreteCartridge` | `struct` | `src/core/implementation.rs` | NONE |
| `new` | `fn` | `src/core/implementation.rs` | NONE |
| `MixRow` | `struct` | `src/homogeneous.rs` | lean://umst-formal/Lean/Powers.lean#PowersState |
| `HomogeneousError` | `enum` | `src/homogeneous.rs` | NONE |
| `ultimate_doh` | `fn` | `src/homogeneous.rs` | NONE |
| `mix_hydration_state` | `fn` | `src/homogeneous.rs` | lean://umst-formal/Lean/Powers.lean#powers_monotone |
| `powers_compressive_strength_mpa` | `fn` | `src/homogeneous.rs` | lean://umst-formal/Lean/Powers.lean#powers_monotone |
| `compressive_strength_mpa` | `fn` | `src/homogeneous.rs` | lean://umst-formal/Lean/Powers.lean#PowersState |
| `degree_of_hydration_alpha` | `fn` | `src/homogeneous.rs` | lean://umst-formal/Lean/Powers.lean#powers_monotone |
| `capillary_porosity` | `fn` | `src/homogeneous.rs` | lean://umst-formal/Lean/Gate.lean#Admissible |
| `yield_stress_pa` | `fn` | `src/homogeneous.rs` | NONE |
| `embodied_co2_kg_per_m3` | `fn` | `src/homogeneous.rs` | NONE |
| `safety_margin` | `fn` | `src/homogeneous.rs` | lean://umst-formal/Lean/Gate.lean#Admissible |
| `constituent_masses_kg_m3` | `fn` | `src/homogeneous.rs` | NONE |
| `mix_row_from_scalar_spec` | `fn` | `src/homogeneous.rs` | lean://umst-formal/Lean/Naturality.lean#gateMaterialAgnostic |
| `ChemoWaterEngine` | `struct` | `src/physics/chemo_water.rs` | NONE |
| `compute_moisture_transport` | `fn` | `src/physics/chemo_water.rs` | NONE |
| `ColloidalEngine` | `struct` | `src/physics/colloidal.rs` | NONE |
| `compute_dlvo_potential` | `fn` | `src/physics/colloidal.rs` | NONE |
| `compute_flocculation_multiplier` | `fn` | `src/physics/colloidal.rs` | NONE |
| `compute_cost` | `fn` | `src/physics/cost.rs` | NONE |
| `CreepEngine` | `struct` | `src/physics/creep.rs` | NONE |
| `compute_compliance` | `fn` | `src/physics/creep.rs` | NONE |
| `FiberEngine` | `struct` | `src/physics/fiber.rs` | NONE |
| `compute_micromechanics` | `fn` | `src/physics/fiber.rs` | NONE |
| `FractureEngine` | `struct` | `src/physics/fracture.rs` | NONE |
| `compute_effective_modulus_mt` | `fn` | `src/physics/fracture.rs` | NONE |
| `compute_fracture_toughness` | `fn` | `src/physics/fracture.rs` | NONE |
| `FreezeThawEngine` | `struct` | `src/physics/freeze_thaw.rs` | NONE |
| `compute_durability` | `fn` | `src/physics/freeze_thaw.rs` | NONE |
| `compute_hydration_degree` | `fn` | `src/physics/hydration.rs` | NONE |
| `compute_itz_thickness_microns` | `fn` | `src/physics/itz.rs` | NONE |
| `compute_itz_porosity` | `fn` | `src/physics/itz.rs` | NONE |
| `compute_itz_percolation_factor` | `fn` | `src/physics/itz.rs` | NONE |
| `NanoEngine` | `struct` | `src/physics/nano.rs` | NONE |
| `compute_enhancements` | `fn` | `src/physics/nano.rs` | NONE |
| `compute_packing_density` | `fn` | `src/physics/packing.rs` | NONE |
| `PolymerEngine` | `struct` | `src/physics/polymer.rs` | NONE |
| `compute_modifiers` | `fn` | `src/physics/polymer.rs` | NONE |
| `compute_capillary_porosity` | `fn` | `src/physics/porosity.rs` | NONE |
| `PrintabilityEngine` | `struct` | `src/physics/printability.rs` | NONE |
| `compute_buildability` | `fn` | `src/physics/printability.rs` | NONE |
| `compute_extrudability` | `fn` | `src/physics/printability.rs` | NONE |
| `RheologyEngine` | `struct` | `src/physics/rheology.rs` | NONE |
| `compute_chateau_ovarlez` | `fn` | `src/physics/rheology.rs` | NONE |
| `compute_yield_stress_yodel` | `fn` | `src/physics/rheology.rs` | NONE |
| `SelfHealEngine` | `struct` | `src/physics/self_heal.rs` | NONE |
| `compute_healing_potential` | `fn` | `src/physics/self_heal.rs` | NONE |
| `SetTimeEngine` | `struct` | `src/physics/set_time.rs` | NONE |
| `compute_setting_time` | `fn` | `src/physics/set_time.rs` | NONE |
| `ShrinkageEngine` | `struct` | `src/physics/shrinkage.rs` | NONE |
| `compute_autogenous_shrinkage` | `fn` | `src/physics/shrinkage.rs` | NONE |
| `StrengthEngine` | `struct` | `src/physics/strength.rs` | NONE |
| `compute_strength_jennings` | `fn` | `src/physics/strength.rs` | NONE |
| `SustainabilityEngine` | `struct` | `src/physics/sustainability.rs` | NONE |
| `compute_embodied_carbon` | `fn` | `src/physics/sustainability.rs` | NONE |
| `compute_financial_cost` | `fn` | `src/physics/sustainability.rs` | NONE |
| `ThermoEngine` | `struct` | `src/physics/thermo.rs` | NONE |
| `compute_heat_rate` | `fn` | `src/physics/thermo.rs` | NONE |
| `TransportEngine` | `struct` | `src/physics/transport.rs` | NONE |
| `compute_capillary_porosity` | `fn` | `src/physics/transport.rs` | NONE |
| `compute_chloride_diffusivity` | `fn` | `src/physics/transport.rs` | NONE |

*Total documented public items: **98**.*
