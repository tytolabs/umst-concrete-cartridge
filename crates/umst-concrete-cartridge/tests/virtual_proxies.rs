// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
// WS-PROXY: virtual stack + extrusion proxy scores.

use umst_concrete_cartridge::proxies::{virtual_extrusion, virtual_stack};

#[test]
fn virtual_stack_scores_finite_and_ordered() {
    let low = virtual_stack::virtual_stack_score(50.0);
    let mid = virtual_stack::virtual_stack_score_in_band(270.0);
    let high = virtual_stack::virtual_stack_score(800.0);
    assert!(low.is_finite() && mid.is_finite() && high.is_finite());
    assert_eq!(mid, 1.0, "in-band τ₀ should score 1.0");
    assert!(low < mid, "below-band τ₀ should score lower");
}

#[test]
fn virtual_extrusion_prefers_printable_window() {
    let in_band = virtual_extrusion::virtual_extrusion_score(270.0, 0.7);
    let too_low = virtual_extrusion::virtual_extrusion_score(50.0, 0.2);
    let too_high = virtual_extrusion::virtual_extrusion_score(2_000.0, 0.2);
    assert!(
        in_band > too_low && in_band > too_high,
        "in-band τ₀ should win"
    );
    assert!(in_band >= 0.5 && in_band <= 1.0);
}

#[test]
fn virtual_proxies_complete_under_one_second() {
    let start = std::time::Instant::now();
    for tau in (180..360).step_by(20) {
        let _ = virtual_stack::virtual_stack_score(tau as f32);
        let _ = virtual_extrusion::virtual_extrusion_score(tau as f32, 0.55);
    }
    assert!(start.elapsed().as_secs_f32() < 1.0);
}
