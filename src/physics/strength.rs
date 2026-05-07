use burn::tensor::{backend::Backend, Tensor};

/// Pure tensor implementation of the Strength & Micromechanics Engine.
/// Upgraded to the absolute SOTA: Jennings CM-II (Colloidal Model of C-S-H)
/// coupled with Ulm & Constantinides (2004) nano-indentation continuum micromechanics.
pub struct StrengthEngine<B: Backend> {
    _backend: std::marker::PhantomData<B>,
}

impl<B: Backend> StrengthEngine<B> {
    /// Computes Compressive Strength using cutting-edge Continuum Micromechanics.
    /// Maps w/c and hydration into High-Density (HD) and Low-Density (LD) C-S-H fractions.
    ///
    /// # Arguments
    /// * `wc_ratio` - Water/Cement ratio tensor [Batch, Depth, Height, Width]
    /// * `degree_hydration` - Hydration degree tensor α (0.0 to 1.0)
    /// * `air_content` - Entrapped/entrained air volume fraction
    /// * `intrinsic_strength` - Intrinsic scaling factor for the specific cement chemistry (MPa)
    pub fn compute_strength_jennings(
        wc_ratio: Tensor<B, 4>,
        degree_hydration: Tensor<B, 4>,
        air_content: Tensor<B, 4>,
        intrinsic_strength: Tensor<B, 4>,
    ) -> (Tensor<B, 4>, Tensor<B, 4>, Tensor<B, 4>) {
        let safe_wc = wc_ratio.clone().clamp(0.20_f32, 0.80_f32);

        // 1. Volumes of Phases (Tennis & Jennings, 2000)
        // Normalized volume scaling
        let v_cement = safe_wc.clone().powf_scalar(-1.0_f32).mul_scalar(0.317_f32);
        let v_csh_total = degree_hydration.clone().mul(v_cement).mul_scalar(1.52_f32); // C-S-H forms ~1.52x cement volume

        // 2. High-Density (HD) vs Low-Density (LD) C-S-H partitioning
        // V_LD / V_total_CSH = 3.017 * (w/c) - 0.347 (simplified linear fit from T&J 2000)
        let ld_fraction = safe_wc
            .clone()
            .mul_scalar(3.017_f32)
            .sub_scalar(0.347_f32)
            .clamp(0.0_f32, 1.0_f32);
        let hd_fraction = ld_fraction.clone().mul_scalar(-1.0_f32).add_scalar(1.0_f32);

        let v_ld = v_csh_total.clone().mul(ld_fraction);
        let v_hd = v_csh_total.clone().mul(hd_fraction);

        // 3. Continuum Micromechanics (Ulm & Constantinides, 2004)
        // Nano-indentation proves universal intrinsic elastic moduli:
        // E_LD ≈ 21.7 GPa
        // E_HD ≈ 29.4 GPa
        let e_ld = 21.7_f32;
        let e_hd = 29.4_f32;

        // Effective Paste Modulus via rule of mixtures for C-S-H matrix (Voigt approx)
        let e_matrix = v_ld
            .clone()
            .mul_scalar(e_ld)
            .add(v_hd.clone().mul_scalar(e_hd));

        // 4. Porosity penalization (Capillary + Air)
        let porosity_capillary = safe_wc
            .clone()
            .sub(degree_hydration.clone().mul_scalar(0.36_f32))
            .clamp_min(0.0_f32);
        let total_porosity = porosity_capillary.add(air_content);

        // Modulus reduction due to porosity (Balshin model: E = E0 * (1-p)^3)
        let solid_fraction = total_porosity
            .clone()
            .mul_scalar(-1.0_f32)
            .add_scalar(1.0_f32)
            .clamp_min(0.01_f32);
        let e_eff = e_matrix.mul(solid_fraction.clone().powf_scalar(3.0_f32));

        // 5. Strength Scaling
        // Strength is proportional to the effective stiffness of the C-S-H gel network
        // We use the intrinsic strength anchor to scale the GPa modulus into MPa strength
        let compressive_strength = e_eff.mul(intrinsic_strength).mul_scalar(0.05_f32);

        (compressive_strength, v_hd, v_ld)
    }
}
