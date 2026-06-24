// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

#[cfg(feature = "solver-experimental")]
use burn::tensor::Int;
use burn::tensor::{backend::Backend, Tensor};
use umst_manifold::core::apply_physics_to_umst;
use umst_manifold::core::tensors::StatePoint;
use umst_manifold::core::tensors::UnifiedMaterialStateTensor;
use umst_manifold::core::traits::{IScienceCartridge, PhysicalResult};
#[cfg(feature = "solver-experimental")]
use umst_manifold::core::SCALAR_HUMIDITY;
use umst_manifold::core::{SCALAR_DAMAGE, SCALAR_INTERNAL_VARIABLE_0, SCALAR_TEMPERATURE};
#[cfg(feature = "solver-experimental")]
use umst_manifold::core::{SCALAR_FRACTURE_ENERGY_GC, VECTOR_MECHANICAL_DISPLACEMENT};

#[cfg(feature = "solver-experimental")]
use umst_manifold::physics::solvers::{
    ChemicalPlan, HydrologicPlan, MechanicalPlan, PhaseFieldFractureSolver, ThermalPlan,
    ThmcSolver, ThmcState,
};

use crate::calibration::Profile;
use crate::pipeline::{physical_result_from_report, run_full_physics_pipeline};

/// Placeholder AT2 length scale \(l\) (m) for topology-phase-field until calibration overrides.
#[cfg(feature = "solver-experimental")]
const DEFAULT_PHASE_FIELD_LENGTH_SCALE: f32 = 1e-3;

/// Tensorized [`crate::homogeneous::safety_margin`] for sparse topology layouts `[1, N]`.
fn topology_nodal_safety_margin<B: Backend<FloatElem = f32>>(
    profile: &Profile,
    w_c_eff: f32,
    alpha_bn: Tensor<B, 2>,
) -> Tensor<B, 2> {
    let alpha_inf = crate::homogeneous::ultimate_doh(profile, w_c_eff);
    let denom = alpha_inf.max(1e-6);
    let mills = alpha_bn
        .clone()
        .neg()
        .add_scalar(alpha_inf)
        .div_scalar(denom)
        .clamp(0.0, 1.0);
    let porosity = Tensor::<B, 2>::ones_like(&alpha_bn)
        .mul_scalar(w_c_eff)
        .sub(alpha_bn.mul_scalar(0.36))
        .div_scalar(w_c_eff + 0.32)
        .clamp(0.0, 1.0);
    mills.add(porosity).mul_scalar(0.5).clamp(0.0, 1.0)
}

/// L¹ norm (sum of absolute values) as a scalar — cheap populated-check for `[1,·]` solver tensors.
fn tensor_l1<B: Backend<FloatElem = f32>>(t: Tensor<B, 2>) -> f32 {
    t.abs().sum().into_scalar()
}

/// Broadcast a single `[1,1]` row from [`physical_result_from_report`] to nodal `[1,N]` layout.
fn broadcast_scalar_to_nodes<B: Backend<FloatElem = f32>>(
    scalar_11: Tensor<B, 2>,
    template_1n: &Tensor<B, 2>,
) -> Tensor<B, 2> {
    let v = scalar_11.into_scalar();
    Tensor::<B, 2>::ones_like(template_1n).mul_scalar(v)
}

/// Map UMST bulk tensors into phase-field fracture solver arguments.
///
/// **Scalar convention:** [`SCALAR_TEMPERATURE`], [`SCALAR_DAMAGE`] from [`umst_manifold::core`].
/// **Strain:** `matrix_features[:, 0, :, :]` is treated as ε `[N, 3, 3]`; if `F_matrices == 0`, strain is zero.
/// **\(G_c\):** if `scalar_features` includes column [`SCALAR_FRACTURE_ENERGY_GC`], per-node values
/// [J/m²]; otherwise a uniform scalar from
/// [`crate::physics::fracture_material::fracture_energy_gc_j_per_m2_from_profile`] (broadcast).
///
/// Returns `(strain [1,N,3,3], damage [1,N,1], gc [1,N,1])`.
#[cfg(feature = "solver-experimental")]
fn phase_field_inputs_from_umst<B: Backend<FloatElem = f32>>(
    manifold: &UnifiedMaterialStateTensor<B>,
    profile: &Profile,
) -> (Tensor<B, 4>, Tensor<B, 3>, Tensor<B, 3>) {
    let dev = manifold.scalar_features.device();
    let n_nodes = manifold.scalar_features.dims()[0];
    let n_mat = manifold.matrix_features.dims()[1];

    let strain_n33 = if n_mat > 0 {
        manifold
            .matrix_features
            .clone()
            .slice([0..n_nodes, 0..1, 0..3, 0..3])
            .reshape([n_nodes, 3, 3])
    } else {
        Tensor::<B, 3>::zeros([n_nodes, 3, 3], &dev)
    };
    let strain = strain_n33.unsqueeze_dim::<4>(0);

    let features = manifold.scalar_features.clone();
    let nf = features.dims()[1];
    let damage = features
        .clone()
        .slice([0..n_nodes, SCALAR_DAMAGE..SCALAR_DAMAGE + 1])
        .unsqueeze_dim::<3>(0);

    let gc_scalar =
        crate::physics::fracture_material::fracture_energy_gc_j_per_m2_from_profile(profile);
    let gc = if nf > SCALAR_FRACTURE_ENERGY_GC {
        features
            .clone()
            .slice([
                0..n_nodes,
                SCALAR_FRACTURE_ENERGY_GC..SCALAR_FRACTURE_ENERGY_GC + 1,
            ])
            .unsqueeze_dim::<3>(0)
    } else {
        Tensor::<B, 3>::zeros([1, n_nodes, 1], &dev).add_scalar(gc_scalar)
    };

    (strain, damage, gc)
}

/// Build a minimal [`ThmcState`] aligned with `manifold` node count so graph operators are safe to call.
///
/// **Integration note:** maps scalar temperature / humidity / hydration columns when present.
/// **Displacement:** [`VECTOR_MECHANICAL_DISPLACEMENT`] in `vector_features` when `F_vectors > 0`;
/// otherwise zeros (initial guess for the mechanical block).
#[cfg(feature = "solver-experimental")]
fn thmc_state_from_umst<B: Backend<FloatElem = f32>>(
    manifold: &UnifiedMaterialStateTensor<B>,
    damage_bn1: Tensor<B, 3>,
) -> ThmcState<B> {
    let dev = manifold.scalar_features.device();
    let n = manifold.scalar_features.dims()[0];
    let nf = manifold.scalar_features.dims()[1];
    let f = manifold.scalar_features.clone();

    let temperature = if nf > SCALAR_TEMPERATURE {
        f.clone()
            .slice([0..n, SCALAR_TEMPERATURE..SCALAR_TEMPERATURE + 1])
            .unsqueeze_dim::<3>(0)
    } else {
        Tensor::<B, 3>::zeros([1, n, 1], &dev)
    };

    let humidity = if nf > SCALAR_HUMIDITY {
        f.clone()
            .slice([0..n, SCALAR_HUMIDITY..SCALAR_HUMIDITY + 1])
            .unsqueeze_dim::<3>(0)
    } else {
        Tensor::<B, 3>::zeros([1, n, 1], &dev)
    };

    let hydration_alpha = if nf > SCALAR_INTERNAL_VARIABLE_0 {
        f.slice([
            0..n,
            SCALAR_INTERNAL_VARIABLE_0..SCALAR_INTERNAL_VARIABLE_0 + 1,
        ])
        .unsqueeze_dim::<3>(0)
    } else {
        Tensor::<B, 3>::zeros([1, n, 1], &dev).add_scalar(0.01_f32)
    };

    let f_vec = manifold.vector_features.dims()[1];
    let displacement = if f_vec > VECTOR_MECHANICAL_DISPLACEMENT {
        manifold
            .vector_features
            .clone()
            .slice([
                0..n,
                VECTOR_MECHANICAL_DISPLACEMENT..VECTOR_MECHANICAL_DISPLACEMENT + 1,
                0..3,
            ])
            .squeeze::<2>(1)
            .unsqueeze_dim::<3>(0)
    } else {
        Tensor::<B, 3>::zeros([1, n, 3], &dev)
    };

    ThmcState {
        thermal: ThermalPlan { temperature },
        hydro: HydrologicPlan { humidity },
        mechanical: MechanicalPlan { displacement },
        chemical: ChemicalPlan {
            reaction_extent: hydration_alpha,
        },
        damage: damage_bn1,
        time: 0.0_f32,
    }
}

/// The concrete domain [`IScienceCartridge`] implementation: bulk `MixTensor` → tensor physics → [`PhysicalResult`] summary.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Cartridge functor F: mix layout → constitutive summaries; topology pass remains separate DEC hook.
pub struct ConcreteCartridge<B: Backend> {
    /// Active calibration profile (`compute_all` hydration margin + intrinsic strength scale).
    pub profile: Profile,
    /// When set, [`IScienceCartridge::compute_topology`] uses this recipe instead of regime midpoint.
    pub topology_nominal: Option<crate::pipeline::TopologyNominalMix>,
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
            topology_nominal: None,
            _backend: std::marker::PhantomData,
        }
    }

    /// Pin topology headline mix to an explicit recipe ([`TopologyNominalMix`] / [`MixSpec`] conversion).
    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Avoids silent regime-midpoint surrogate when a design is known.
    #[must_use]
    pub fn with_topology_nominal(mut self, nominal: crate::pipeline::TopologyNominalMix) -> Self {
        self.topology_nominal = Some(nominal);
        self
    }

    /// formal_anchor: NONE
    /// formal_status: NONE
    /// formal_anchor_rationale: Mutable UMST merge path for topology tensors; proof witnesses remain caller-owned.
    ///
    /// Merges [`PhysicalResult::damage`] and, when set, [`PhysicalResult::temperature_delta`] into
    /// `umst` via [`UnifiedMaterialStateTensor::project_scalar_channel`]. Does **not** return a
    /// [`VerifiedUMST`](umst_manifold::core::tensors::VerifiedUMST); [`IScienceCartridge::compute_topology`] still yields tensors only.
    pub fn apply_topology_result_to_umst(
        &self,
        result: &PhysicalResult<B>,
        umst: &mut UnifiedMaterialStateTensor<B>,
    ) -> Result<(), String> {
        let _ = &self.profile;
        apply_physics_to_umst(result, umst)
    }
}

impl<B: Backend<FloatElem = f32>> Default for ConcreteCartridge<B> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend<FloatElem = f32>> IScienceCartridge<B> for ConcreteCartridge<B> {
    fn compute_all(&self, mix: &StatePoint<B>) -> PhysicalResult<B> {
        let report = run_full_physics_pipeline::<B>(&self.profile, mix);
        let dev = mix.fractions.device();
        physical_result_from_report::<B>(&self.profile, &report, &dev)
    }

    /// The Multi-Agent Heterogeneous Topology Pass.
    /// Extracts variables from the Cellular Sheaf topology and processes them
    /// via the DEC (Discrete Exterior Calculus) engines.
    ///
    /// With `--features solver-experimental` on **this** crate and matching manifold support, runs an
    /// AT2 damage update then one [`ThmcSolver::step`] (orchestration stub on the manifold side)
    /// before filling [`PhysicalResult::damage`]. Default builds keep the previous heat Laplacian-only path.
    ///
    /// **Merge precedence (topology vs nominal-mix solver / pipeline):** One [`crate::pipeline::run_full_physics_pipeline`]
    /// pass yields [`crate::pipeline::physical_result_from_report`] (`solver_pr`). Nodal graph surrogates are merged so
    /// populated solver channels are not blindly overwritten: **dissipation** keeps Laplacian heat flux when it is
    /// active; otherwise falls back to solver hydration α. **free_energy** uses solver headline \(f_c'\) / \(\tau\) scalars
    /// when those channels are non-zero, else headline scalars from the same report. **cost** uses solver GWP when
    /// populated and always adds the nodal dissipation surcharge. **safety_margin** uses the elementwise minimum of
    /// broadcast homogeneous solver margin and local topology margin when the solver margin is populated (conservative).
    fn compute_topology(&self, manifold: &UnifiedMaterialStateTensor<B>) -> PhysicalResult<B> {
        // 1. `scalar_features` is [N_nodes, F]; see `umst_manifold::core` scalar indices.
        let features = manifold.scalar_features.clone();
        let dev = features.device();
        let n_nodes = features.dims()[0];

        let pipeline_report = crate::pipeline::physical_summary::topology_pipeline_report::<B>(
            &self.profile,
            &dev,
            self.topology_nominal,
        );
        let solver_pr =
            crate::pipeline::physical_result_from_report(&self.profile, &pipeline_report, &dev);
        let (fc_mpa, tau_pa, gwp, w_c_eff, alpha_ref, k_ic) =
            crate::pipeline::physical_summary::topology_pipeline_headlines_from_report(
                &pipeline_report,
            );

        let temp_c = features
            .clone()
            .slice([0..n_nodes, SCALAR_TEMPERATURE..SCALAR_TEMPERATURE + 1])
            .unsqueeze_dim::<3>(0);
        let damage = features
            .clone()
            .slice([0..n_nodes, SCALAR_DAMAGE..SCALAR_DAMAGE + 1])
            .unsqueeze_dim::<3>(0);

        // 2. Thermodynamic heat operator on the 1-skeleton (graph Laplacian).
        let heat_flux_gradient =
            umst_manifold::physics::laplacian::TopologicalLaplacian::scalar_laplacian(
                temp_c.clone(),
                manifold.edges_b1.clone(),
                damage.clone(),
            );

        let heat_dissipation = heat_flux_gradient.clone().abs().squeeze::<2>(2);

        // Explicit-Euler increment surrogate (same `dt` as the bundled [`ThmcSolver`] below) for optional UMST write-back.
        let temperature_delta = Some(
            heat_flux_gradient
                .clone()
                .mul_scalar(1.0_f32)
                .squeeze::<2>(2),
        );

        #[cfg(feature = "solver-experimental")]
        let _ = alpha_ref;

        #[cfg(not(feature = "solver-experimental"))]
        let manifold_alpha_bn = {
            let nf = features.dims()[1];
            if nf > SCALAR_INTERNAL_VARIABLE_0 {
                features
                    .clone()
                    .slice([
                        0..n_nodes,
                        SCALAR_INTERNAL_VARIABLE_0..SCALAR_INTERNAL_VARIABLE_0 + 1,
                    ])
                    .unsqueeze_dim::<3>(0)
                    .squeeze::<2>(2)
            } else {
                Tensor::<B, 2>::zeros([1, n_nodes], &dev).add_scalar(alpha_ref)
            }
        };

        let (damage_out, alpha_gate_bn) = {
            #[cfg(feature = "solver-experimental")]
            {
                let (strain, damage_bn1, gc_bn1) =
                    phase_field_inputs_from_umst(manifold, &self.profile);
                let fracture = PhaseFieldFractureSolver {
                    length_scale: DEFAULT_PHASE_FIELD_LENGTH_SCALE,
                };
                let edges: Tensor<B, 2, Int> = manifold.edges_b1.clone();
                let damage_pf = fracture.update_damage(strain, damage_bn1, gc_bn1, edges);

                let mut thmc = ThmcSolver {
                    dt: 1.0_f32,
                    max_newton: 1_usize,
                    tol: 1e-2_f32,
                    drying_last_node_evaporation_k: 0.0_f32,
                    drying_ambient_h: 0.5_f32,
                    ..Default::default()
                };
                let state0 = thmc_state_from_umst(manifold, damage_pf);
                let state1 = thmc
                    .step(self, state0, manifold)
                    .expect("THMC step must not fail in experimental mode");

                let alpha_exp = state1.chemical.reaction_extent.clone().squeeze::<2>(2);
                (state1.damage.squeeze::<2>(2), alpha_exp)
            }
            #[cfg(not(feature = "solver-experimental"))]
            {
                (damage.squeeze::<2>(2), manifold_alpha_bn)
            }
        };

        const SOLVER_FIELD_EPS: f32 = 1e-12_f32;

        let dissipation = {
            let l1_heat = tensor_l1(heat_dissipation.clone());
            let l1_sol = tensor_l1(solver_pr.dissipation.clone());
            if l1_heat < 1e-18_f32 && l1_sol > SOLVER_FIELD_EPS {
                broadcast_scalar_to_nodes(solver_pr.dissipation.clone(), &heat_dissipation)
            } else {
                heat_dissipation
            }
        };

        let ones_n = Tensor::<B, 2>::ones_like(&damage_out);

        let fc_use = {
            let t = solver_pr.free_energy.clone().slice([0..1, 0..1]);
            if tensor_l1(t.clone()) > SOLVER_FIELD_EPS {
                t.into_scalar()
            } else {
                fc_mpa
            }
        };
        let tau_use = {
            let t = solver_pr.free_energy.clone().slice([0..1, 1..2]);
            if tensor_l1(t.clone()) > SOLVER_FIELD_EPS {
                t.into_scalar()
            } else {
                tau_pa
            }
        };

        let free_energy = ones_n
            .clone()
            .mul_scalar(fc_use)
            .sub(damage_out.clone().mul_scalar(fc_use))
            .add(dissipation.clone().mul_scalar(tau_use * 1e-6_f32))
            .add(
                ones_n
                    .clone()
                    .sub(damage_out.clone())
                    .mul_scalar(k_ic * 1e-3_f32),
            );

        let topo_safety_margin =
            topology_nodal_safety_margin(&self.profile, w_c_eff, alpha_gate_bn);
        let safety_margin = {
            let sm = solver_pr.safety_margin.clone();
            if tensor_l1(sm.clone()) > SOLVER_FIELD_EPS {
                let sol_n = broadcast_scalar_to_nodes(sm.clone(), &topo_safety_margin);
                sol_n.clone().add(
                    topo_safety_margin
                        .clone()
                        .sub(sol_n)
                        .clamp(f32::NEG_INFINITY, 0.0_f32),
                )
            } else {
                topo_safety_margin
            }
        };

        let cost = {
            let inc = dissipation.clone().mul_scalar(0.01_f32);
            let gwpt = solver_pr.cost.clone();
            if tensor_l1(gwpt.clone()) > SOLVER_FIELD_EPS {
                Tensor::<B, 2>::ones_like(&damage_out)
                    .mul_scalar(gwpt.into_scalar())
                    .add(inc)
            } else {
                ones_n.mul_scalar(gwp).add(inc)
            }
        };

        PhysicalResult {
            free_energy,
            dissipation,
            safety_margin,
            cost,
            damage: damage_out,
            temperature_delta,
        }
    }
}
