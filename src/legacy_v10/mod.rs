// SPDX-FileCopyrightText: 2018-2026 Studio TYTO
// SPDX-License-Identifier: LicenseRef-Proprietary
pub mod calibration; // Bayesian learning from experimental errors
pub mod chemo_water;
pub mod colloidal;
pub mod cost;
pub mod fracture;
pub mod itz;
pub mod maturity;
pub mod porosity;
pub mod printability; // [V8.1] 3D printing assessment (extrudability, buildability)
pub mod rheology;
// robotics moved to top-level robotics/ module
pub mod creep; // [V8.2] XMPS-inspired creep model
pub mod materials;
pub mod shrinkage; // [V8.2] Jensen-Hansen autogenous shrinkage
pub mod strength;
pub mod sustainability;
pub mod thermo;
pub mod thermodynamic_filter; // Constitutional Clausius-Duhem gate
pub mod transport; // [God-Grade] Material Profiles

// [V9.0] Advanced Material Physics Engines
pub mod constitution;
pub mod fiber; // Fiber reinforcement (Naaman/JSCE models)
pub mod freeze_thaw; // Freeze-thaw durability (Powers/Fagerlund)
pub mod nano; // Nanomaterial enhancement (nucleation, pozzolanic)
pub mod polymer; // Polymer modification (film formation/adhesion)
pub mod self_heal; // Self-healing concrete (autogenous/crystalline)
pub mod set_time; // Setting time prediction (Arrhenius/ACI) // First-class PhysicalAxiom trait + proof-carrying witnesses (full umst-formal integration, hybrid with TypedGate)
