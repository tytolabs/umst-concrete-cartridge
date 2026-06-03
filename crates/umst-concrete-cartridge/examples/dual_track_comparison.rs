// SPDX-License-Identifier: MIT
// WS-COMPARE: Track A (coordinate descent + dual gate) vs Track B (experimental PPO stub).

use std::time::Instant;

use umst_concrete_cartridge::calibration::Profile;
use umst_concrete_cartridge::facade::{MixSpec, TemperatureK, WaterCementRatio};
use umst_concrete_cartridge::pipeline::{coordinate_descent_optimize, TrackAObjective};

fn s1_base_mix() -> MixSpec {
    MixSpec {
        w_c: WaterCementRatio::try_from(0.45).expect("w_c"),
        temperature_k: TemperatureK::try_from(298.15).expect("T"),
        superplasticiser_pct: 1.0,
        silica_fume_pct: 10.0,
        fly_ash_pct: 0.0,
        aggregate_volume_fraction: 0.35,
        target_age_hours: 1.0,
        profile_name: "tyto_mortar".into(),
    }
}

fn main() {
    let profile = Profile::load_bundled("tyto_mortar").expect("tyto_mortar profile");
    let base = s1_base_mix();

    println!("=== Track A: coordinate descent + dual gate (shippable) ===");
    let t0 = Instant::now();
    let (proposed_a, summary_a, gate_a) =
        coordinate_descent_optimize(&profile, &base, TrackAObjective::PrintableWindow, 20);
    let dt_a = t0.elapsed();
    println!(
        "  proposed w_c={:.3} sp={:.2}% τ₀={:.1} Pa extr={:.3} gate_pass={} ({:.1?})",
        proposed_a.w_c.value(),
        proposed_a.superplasticiser_pct,
        summary_a.rheology_yield_stress_pa,
        summary_a.printability_extrudability,
        gate_a.passes(),
        dt_a
    );

    println!("\n=== Track B: epistemic PPO (experimental — not in default build) ===");
    println!("  Track B requires `umst-manifold` with `--features epistemic-ppo`.");
    println!("  Status: experimental; promotion off-flag requires human sign-off.");
    println!("  Track A is the publishable best-working path for S1 mortar retune.");

    if !gate_a.passes() {
        eprintln!("warning: Track A did not reach dual-gate pass on this run");
        std::process::exit(1);
    }
}
