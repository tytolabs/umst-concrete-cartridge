// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Serializes `UMST_UCRS_WITNESS` mutations across parallel `#[test]` modules.

use std::sync::Mutex;

static WITNESS_ENV_LOCK: Mutex<()> = Mutex::new(());

/// Run `f` while `UMST_UCRS_WITNESS` is set to `mode`, then remove the variable.
pub fn with_witness_mode<F: FnOnce()>(mode: &str, f: F) {
    let _guard = WITNESS_ENV_LOCK.lock().expect("witness env lock");
    std::env::set_var("UMST_UCRS_WITNESS", mode);
    f();
    std::env::remove_var("UMST_UCRS_WITNESS");
}
