// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cementitious transition closure values (W9 Tier 2c SSOT).
//!
//! **FP contract:** all closures are `const`/`Copy` witnesses — no IO, no ambient state.
//! [`MaterialTransitionParams`] impls are natural transformations from cartridge objects
//! into kernel-admissible parameter records consumed by pure gate evaluators.

use umst_manifold::core::{MaterialTransitionParams, ReactionExtentKineticsSpec};

use crate::chem_adapter;

/// formal_anchor: literature://cement-hydration-enthalpy-order-of-magnitude
/// formal_status: Literature
/// formal_citation: "Representative cementitious hydration enthalpy scale (order 450 J/g binder mass basis)."
/// formal_form: "`CEMENT_REACTION_ENTHALPY_J_PER_KG` feeds Clausius–Duhem transition gate via [`MaterialTransitionParams`]."
/// formal_anchor_rationale: Cartridge SSOT for W9 Tier-2c; kernel must not duplicate this literal.
pub const CEMENT_REACTION_ENTHALPY_J_PER_KG: f64 =
    chem_adapter::cement_reaction_enthalpy_j_per_kg();

/// formal_anchor: literature://powers-intrinsic-strength-scale
/// formal_status: Literature
/// formal_citation: "Powers-style monotonic strength closure intrinsic scale (order 240 MPa)."
/// formal_form: "`CEMENT_DEFAULT_S_INTRINSIC_MPA` upper-bounds admissible strength jumps in transition gate."
/// formal_anchor_rationale: Cartridge SSOT for W9 Tier-2c; kernel must not duplicate this literal.
pub const CEMENT_DEFAULT_S_INTRINSIC_MPA: f64 = 240.0;

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Bundled THMC kinetics witness for parity harnesses; values mirror cartridge calibration lane.
#[must_use]
pub const fn cement_reaction_extent_kinetics_spec(
) -> umst_manifold::core::ReactionExtentKineticsSpec {
    chem_adapter::cement_reaction_extent_kinetics_spec()
}

impl MaterialTransitionParams for crate::calibration::Profile {
    fn reaction_enthalpy_j_per_kg(&self) -> f64 {
        CEMENT_REACTION_ENTHALPY_J_PER_KG
    }

    fn default_intrinsic_strength_mpa(&self) -> f64 {
        CEMENT_DEFAULT_S_INTRINSIC_MPA
    }

    fn reaction_extent_kinetics_spec(&self) -> ReactionExtentKineticsSpec {
        cement_reaction_extent_kinetics_spec()
    }
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Zero-sized cement closure witness for harnesses without a loaded [`Profile`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CementMaterialParams;

impl MaterialTransitionParams for CementMaterialParams {
    fn reaction_enthalpy_j_per_kg(&self) -> f64 {
        CEMENT_REACTION_ENTHALPY_J_PER_KG
    }

    fn default_intrinsic_strength_mpa(&self) -> f64 {
        CEMENT_DEFAULT_S_INTRINSIC_MPA
    }

    fn reaction_extent_kinetics_spec(&self) -> ReactionExtentKineticsSpec {
        cement_reaction_extent_kinetics_spec()
    }
}
