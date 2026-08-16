// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Serializes `UMST_UCRS_WITNESS` mutations across parallel `#[test]` modules.

use std::sync::Mutex;

static WITNESS_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` while `UMST_UCRS_WITNESS` is set to `mode`, then remove the variable.
/// formal_anchor: NONE
/// formal_status: NONE
/// formal_anchor_rationale: Test-only env serialization; not production gate semantics.
pub fn with_witness_mode<F: FnOnce()>(mode: &str, f: F) {
    let _guard = WITNESS_ENV_LOCK.lock().expect("witness env lock");
    std::env::set_var("UMST_UCRS_WITNESS", mode);
    f();
    std::env::remove_var("UMST_UCRS_WITNESS");
}
