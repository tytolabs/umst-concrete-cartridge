// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Integration-style anchor: [`StrengthEngine::compute_strength_jennings`] with **`s_intrinsic`** from
//! [`calibration/profiles/uci_d1.v1.toml`](../../calibration/profiles/uci_d1.v1.toml) should land near
//! the UCI D1 compressive-strength cluster (~37 MPa) for a representative **w/c–α** tuple.

use burn::tensor::{Data, Shape, Tensor};
use burn_ndarray::{NdArray, NdArrayDevice};
use umst_concrete_cartridge::mix_layout::collapsed_rank4_from_rank2_scalar;
use umst_concrete_cartridge::physics::strength::StrengthEngine;

type B = NdArray<f32>;

#[test]
fn strength_engine_uci_d1_intrinsic_scales_to_literature_cluster_mpa() {
    let dev = NdArrayDevice::default();
    let t01 = |v: f32| Tensor::<B, 2>::from_data(Data::new(vec![v], Shape::new([1, 1])), &dev);

    // `s_intrinsic` from `calibration/profiles/uci_d1.v1.toml` ([parameters.powers_gel_space]).
    let s_intrinsic = 74.92_f32;
    // Representative lab mix in the D1 **w/c** band (28 d class); **α** chosen so f_c ≈ 37 MPa at this intrinsic.
    let wc = 0.38_f32;
    let alpha = 0.61_f32;
    let air = 0.02_f32;

    let wc4 = collapsed_rank4_from_rank2_scalar(t01(wc), &dev);
    let a4 = collapsed_rank4_from_rank2_scalar(t01(alpha), &dev);
    let air4 = collapsed_rank4_from_rank2_scalar(t01(air), &dev);
    let int4 = collapsed_rank4_from_rank2_scalar(t01(s_intrinsic), &dev);

    let (fc, _, _) = StrengthEngine::<B>::compute_strength_jennings(wc4, a4, air4, int4);
    let fc_mpa = fc.into_data().value[0];

    let target = 37.0_f32;
    let lo = target * 0.85;
    let hi = target * 1.15;
    assert!(
        fc_mpa >= lo && fc_mpa <= hi,
        "expected f_c in [{lo:.1},{hi:.1}] MPa (≈37±15% UCI D1 cluster), got {fc_mpa:.2} (wc={wc}, α={alpha}, s_int={s_intrinsic})"
    );
}
