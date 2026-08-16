// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use umst_concrete_cartridge::calibration::{Profile, BUNDLED_PROFILE_IDS};

fn lean_path_from_uri(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("lean://umst-formal/Lean/")?;
    let stem = rest.split('#').next()?;
    let root = std::env::var("UMST_FORMAL_ROOT").ok()?;
    Some(Path::new(&root).join("Lean").join(stem))
}

#[test]
fn all_profiles_parse_and_bounds_positive() -> Result<(), Box<dyn Error>> {
    for id in BUNDLED_PROFILE_IDS {
        let p = Profile::load_bundled(id)?;
        assert!(
            p.regime.w_c_min < p.regime.w_c_max && p.regime.w_c_min >= 0.10,
            "{id} w/c bounds",
        );
        assert!(
            p.regime.temperature_k_min < p.regime.temperature_k_max,
            "{id} temperature bounds",
        );
        assert!(p.powers.s_intrinsic > 0.0, "{id} s_intrinsic");
        if p.contract.verification_status == "Contract" {
            assert!(
                p.acceptance.strength_mae_max.is_some(),
                "{id} Contract profile missing strength_mae_max",
            );
        }
    }
    Ok(())
}

#[test]
fn formal_uris_point_at_existing_lean_when_root_set() -> Result<(), Box<dyn Error>> {
    let root = match std::env::var("UMST_FORMAL_ROOT") {
        Ok(r) => r,
        Err(_) => {
            eprintln!("SKIP: UMST_FORMAL_ROOT unset; Lean existence check not run");
            return Ok(());
        }
    };
    assert!(
        Path::new(&root).join("Lean").is_dir(),
        "UMST_FORMAL_ROOT={root} missing Lean/"
    );
    for id in BUNDLED_PROFILE_IDS {
        let p = Profile::load_bundled(id)?;
        let pf = p.provenance.formal.as_ref().expect("formal block");
        assert!(
            pf.anchor.starts_with("lean://"),
            "{id} provenance.formal.anchor"
        );
        let path =
            lean_path_from_uri(&pf.anchor).unwrap_or_else(|| panic!("bad URI {}", pf.anchor));
        assert!(
            path.exists(),
            "{id} anchor file missing: {}",
            path.display()
        );
        if let Some(acc) = &p.acceptance.formal_anchor {
            let ap = lean_path_from_uri(acc).unwrap_or_else(|| panic!("bad URI {acc}"));
            assert!(
                ap.exists(),
                "{} acceptance anchor missing {}",
                id,
                ap.display()
            );
        }
    }
    Ok(())
}

#[test]
fn lemma_naturality_square_exists_when_formal_root_set() -> Result<(), Box<dyn Error>> {
    let root = match std::env::var("UMST_FORMAL_ROOT") {
        Ok(r) => r,
        Err(_) => {
            eprintln!("SKIP: UMST_FORMAL_ROOT unset; Naturality lemma check not run");
            return Ok(());
        }
    };
    let lean = Path::new(&root).join("Lean/Naturality.lean");
    assert!(lean.is_file(), "missing {}", lean.display());
    let body = fs::read_to_string(&lean)?;
    assert!(
        body.contains("naturalitySquare"),
        "expected `naturalitySquare` marker in {}",
        lean.display()
    );
    Ok(())
}
