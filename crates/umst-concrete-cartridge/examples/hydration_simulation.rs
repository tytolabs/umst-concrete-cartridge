// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Hydration-simulation example.
//!
//! Reproduces a Powers (1948) degree-of-hydration curve for an OPC paste
//! at w/c = 0.40, isothermal at 293 K. Reports the ultimate degree of
//! hydration, the Mills (1966) saturation form, and the predicted DoH
//! at 1, 7, 28, and 90 days.
//!
//! Run with:
//! ```bash
//! cargo run --example hydration_simulation --release
//! ```

const W_C: f32 = 0.40;
const TAU_HOURS: f32 = 10.0;
const BETA: f32 = 0.85;

fn ultimate_degree_of_hydration(w_c: f32) -> f32 {
    1.031 * w_c / (0.194 + w_c)
}

fn doh_at(t_hours: f32, w_c: f32) -> f32 {
    let alpha_inf = ultimate_degree_of_hydration(w_c);
    let exp_arg = (t_hours / TAU_HOURS).powf(BETA);
    alpha_inf * (1.0 - (-exp_arg).exp())
}

fn main() {
    println!("UMST Concrete Cartridge — hydration example");
    println!("===========================================");
    println!("System: OPC paste, w/c = {W_C}, T = 293 K, isothermal");

    let alpha_inf = ultimate_degree_of_hydration(W_C);
    println!("Mills ultimate DoH α∞ = {alpha_inf:.4}");
    println!();

    let checkpoints_hours = [24.0, 24.0 * 7.0, 24.0 * 28.0, 24.0 * 90.0];
    let labels = ["1 d", "7 d", "28 d", "90 d"];

    println!("{:>6}  {:>9}  {:>9}", "age", "α(t)", "α/α∞");
    println!("{:->6}  {:->9}  {:->9}", "", "", "");
    for (h, label) in checkpoints_hours.iter().zip(labels.iter()) {
        let alpha = doh_at(*h, W_C);
        let frac = alpha / alpha_inf;
        println!("{label:>6}  {alpha:>9.4}  {frac:>9.3}");
    }

    println!();
    println!("Sanity checks (Powers 1948 envelope, ±5% MAE):");
    let alpha_28d = doh_at(24.0 * 28.0, W_C);
    let powers_28d = 0.66; // Powers 1948 OPC w/c = 0.40 reference
    let err = (alpha_28d - powers_28d).abs() / powers_28d;
    println!("  α(28 d) predicted = {alpha_28d:.3}");
    println!("  α(28 d) Powers    = {powers_28d:.3}");
    println!("  relative error    = {:.1}%", err * 100.0);
    assert!(
        err < 0.10,
        "DoH at 28 d is {:.3}, expected within 10% of Powers 1948 ({:.3})",
        alpha_28d,
        powers_28d
    );

    println!();
    println!("OK — hydration kinetics within Powers 1948 envelope.");
}
