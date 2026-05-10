// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

use burn::tensor::{backend::Backend, Tensor};
use umst_manifold::core::tensors::MixTensor;
use umst_manifold::core::traits::{IScienceCartridge, PhysicalResult};

use crate::calibration::Profile;
use crate::pipeline::{physical_result_from_report, run_full_physics_pipeline};

/// The concrete domain [`IScienceCartridge`] implementation: bulk `MixTensor` → tensor physics → [`PhysicalResult`] summary.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Cartridge functor F: mix layout → constitutive summaries; topology pass remains separate DEC hook.
pub struct ConcreteCartridge<B: Backend> {
    /// Active calibration profile (`compute_all` hydration margin + intrinsic strength scale).
    pub profile: Profile,
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend<FloatElem = f32>> ConcreteCartridge<B> {
    /// Bundle [`Profile::load_bundled`] `uci_d1` for doctest / manifold smoke defaults.
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Deterministic bundled baseline when callers omit explicit calibration.
    pub fn new() -> Self {
        Self::with_profile(Profile::load_bundled("uci_d1").expect("bundled calibration `uci_d1`"))
    }

    /// Cartridge pinned to an explicit calibration bundle (CLI / MCP **must** match `predict` profile).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Avoids silently mixing heterogeneous tensor kinetics with unrelated gel-space coefficients.
    #[must_use]
    pub fn with_profile(profile: Profile) -> Self {
        Self {
            profile,
            _backend: std::marker::PhantomData,
        }
    }
}

impl<B: Backend<FloatElem = f32>> Default for ConcreteCartridge<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend<FloatElem = f32>> IScienceCartridge<B> for ConcreteCartridge<B> {
    fn compute_all(&self, mix: &MixTensor<B>) -> PhysicalResult<B> {
        let report = run_full_physics_pipeline::<B>(&self.profile, mix);
        let dev = mix.fractions.device();
        physical_result_from_report::<B>(&self.profile, &report, &dev)
    }

    /// The Multi-Agent Heterogeneous Topology Pass.
    /// Extracts variables from the Cellular Sheaf topology and processes them
    /// via the DEC (Discrete Exterior Calculus) engines.
    fn compute_topology(
        &self,
        manifold: &umst_manifold::core::tensors::UnifiedMaterialStateTensor<B>,
    ) -> PhysicalResult<B> {
        // 1. `scalar_features` is [N_nodes, F] on the manifold; index 3 = temperature, 4 = damage.
        let features = manifold.scalar_features.clone();
        let dev = features.device();
        let n_nodes = features.dims()[0];

        let temp_c = features.clone().slice([0..n_nodes, 3..4]).unsqueeze_dim(0);
        let damage = features.clone().slice([0..n_nodes, 4..5]).unsqueeze_dim(0);

        // 2. Thermodynamic heat flow on the 1-skeleton (graph Laplacian).
        let heat_flux_gradient =
            umst_manifold::physics::laplacian::TopologicalLaplacian::scalar_laplacian(
                temp_c,
                manifold.edges_b1.clone(),
                damage,
            );

        let dissipation = heat_flux_gradient.abs().squeeze(2);

        let free_energy = Tensor::<B, 2>::zeros([1, n_nodes], &dev).add_scalar(10.0_f32);
        let safety_margin = Tensor::<B, 2>::zeros([1, n_nodes], &dev).add_scalar(1.0_f32);
        let cost = Tensor::<B, 2>::zeros([1, n_nodes], &dev).add_scalar(0.01_f32);

        PhysicalResult {
            free_energy,
            dissipation,
            safety_margin,
            cost,
        }
    }
}
