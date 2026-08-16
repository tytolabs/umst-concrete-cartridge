// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! stdin: one JSON value -> stdout: **[`umst_cli::canonical::canonical_json_bytes`]** (no trailing newline).

use std::io::{self, Read, Write};

use serde_json::Value;
use umst_cli::canonical::{canonical_json_bytes, CanonicalJsonError};

fn main() -> Result<(), String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(buf.trim()).map_err(|e| e.to_string())?;
    let out = canonical_json_bytes(&v).map_err(|e: CanonicalJsonError| e.to_string())?;
    io::stdout().write_all(&out).map_err(|e| e.to_string())?;
    Ok(())
}
