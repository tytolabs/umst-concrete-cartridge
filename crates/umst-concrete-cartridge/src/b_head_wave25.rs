// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
//! B_head pool receipt pins — self-contained build @ census snapshot (audit only).

/// Steer receipt for wave-25 `LANE-POOL-B_head_build_failing-umst-concrete-cartridge`.
pub const B_HEAD_STEER_RECEIPT: &str = "STEER_20261001T0509";

/// Monotone steer-wave counter (audit only).
pub const B_HEAD_STEER_WAVE_SEQ: u32 = 25;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b_head_steer_wave_seq_twenty_five() {
        assert_eq!(B_HEAD_STEER_RECEIPT, "STEER_20261001T0509");
        assert_eq!(B_HEAD_STEER_WAVE_SEQ, 25);
    }
}
