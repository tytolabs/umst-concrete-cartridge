// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! `umst-gate-ffi` — C-ABI over `gate_check_mix_result` (Layer 0 gate fiber).
//!
//! Chemistry scalars remain in `umst-concrete-ffi`. This crate does **not** replace them.

mod gate_bridge;

pub use gate_bridge::{sort_keys, verdict_code};

use gate_bridge::{cstr_to_str, gate_check_canonical_json};
use std::os::raw::{c_char, c_int};
use std::ptr;

/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: ABI version constant for gate FFI consumers.
pub const UMST_GATE_FFI_ABI_VERSION: u32 = 1;

/// Compact gate summary for C consumers.
/// formal_anchor: STRUCTURAL
/// formal_status: Structural
/// formal_anchor_rationale: `repr(C)` out-struct mirroring GateSummary scalars.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CGateSummary {
    /// 1 = admissible, 0 = reject.
    pub admissible: u8,
    /// 0 = PASS, 1 = REJECT, 2 = WARN.
    pub verdict: i32,
}

/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: C ABI version export; not a physics morphism.
#[must_use]
#[no_mangle]
pub extern "C" fn umst_gate_ffi_abi_version() -> u32 {
    UMST_GATE_FFI_ABI_VERSION
}

/// Run `gate_check_mix_result` and fill `out_summary` + optional JSON buffer.
///
/// `mix_json` / `profile_id` are NUL-terminated UTF-8.
/// `explain` non-zero → include explain block.
/// On success returns 0. Negative: -1 null, -2 parse, -3 profile/gate, -4 buffer too small.
///
/// # Safety
/// Pointers must be valid for their lengths; `out_json` may be null to skip JSON copy.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: C FFI entry; CD check delegated to `gate_check_mix_result`.
#[no_mangle]
pub unsafe extern "C" fn umst_gate_check(
    profile_id: *const c_char,
    mix_json: *const c_char,
    explain: c_int,
    ucrs_seq: u64,
    wall_ms: u64,
    out_summary: *mut CGateSummary,
    out_json: *mut c_char,
    out_json_len: usize,
) -> c_int {
    if out_summary.is_null() {
        return -1;
    }
    let profile = match cstr_to_str(profile_id) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let mix_s = match cstr_to_str(mix_json) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let mix: serde_json::Value = match serde_json::from_str(mix_s) {
        Ok(v) => v,
        Err(_) => return -2,
    };
    let (admissible, verdict, canon) =
        match gate_check_canonical_json(profile, &mix, explain != 0, ucrs_seq, wall_ms) {
            Ok(t) => t,
            Err(_) => return -3,
        };
    *out_summary = CGateSummary {
        admissible: u8::from(admissible),
        verdict,
    };
    if !out_json.is_null() && out_json_len > 0 {
        let bytes = canon.as_bytes();
        if bytes.len() + 1 > out_json_len {
            return -4;
        }
        ptr::copy_nonoverlapping(bytes.as_ptr(), out_json as *mut u8, bytes.len());
        *out_json.add(bytes.len()) = 0;
    }
    0
}
