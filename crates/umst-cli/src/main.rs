// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use umst_cli::cli::{self, MixSpec, PredictionWireVersion};
use umst_concrete_cartridge::calibration::Profile;

const MIX_SCHEMA_V1: &str = include_str!("../../../schema/mix.v1.json");
const RESULT_SCHEMA_V1: &str = include_str!("../../../schema/result.v1.json");
const RESULT_SCHEMA_V2: &str = include_str!("../../../schema/result.v2.json");

#[derive(Parser)]
#[command(
    name = "umst",
    version,
    about = "UMST concrete cartridge CLI (optional feature build)"
)]
struct CliRoot {
    #[command(flatten)]
    globals: Globals,
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
struct Globals {
    /// Calibration bundle id (`default`, `uci_d1`, `zenodo_ndt`, …) unless `--profile-file` is set.
    #[arg(long, default_value = "default")]
    profile: String,
    /// Override bundled profile with external TOML (wins over `--profile`).
    #[arg(long, value_name = "PATH")]
    profile_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Predict constitutive scalars from mix JSON (`--input` or stdin).
    Predict {
        #[arg(long, value_name = "FILE")]
        input: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = PredictionWireCli::V2)]
        schema_version: PredictionWireCli,
        /// Attach legacy homogeneous scalars under `homogeneous_compare` (tensor path remains default).
        #[arg(long)]
        compare_homogeneous: bool,
    },
    Optimize {
        #[arg(long)]
        target: String,
        #[arg(long, default_value_t = 24)]
        steps: usize,
        #[arg(long, value_name = "FILE")]
        input: Option<PathBuf>,
    },
    Schema {
        #[command(subcommand)]
        kind: SchemaCmd,
    },
    #[command(subcommand)]
    Profiles(ProfilesCmd),
    /// Emit certify JSON (formal-anchor chain) for bundled `NAME`.
    Certify { name: String },
}

#[derive(Subcommand)]
enum ProfilesCmd {
    /// List bundled profile ids with one-line descriptions.
    List,
    /// Emit full bundled TOML for `NAME` to stdout.
    Describe { name: String },
    /// Emit the `[regime]` block only as JSON.
    Regime { name: String },
}

#[derive(Subcommand)]
enum SchemaCmd {
    Mix,
    Result,
    /// JSON Schema draft 2020-12 for `result.v2` wire objects.
    ResultV2,
}

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
enum PredictionWireCli {
    V1,
    #[default]
    V2,
}

fn read_mix_value(input: Option<PathBuf>) -> Result<Value> {
    let mut buf = String::new();
    match input.as_ref() {
        Some(path) => {
            buf = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        }
        None => {
            io::stdin().read_to_string(&mut buf).context("read stdin")?;
        }
    }
    serde_json::from_str(buf.trim()).context("parse mix JSON")
}

fn load_profile(globals: &Globals) -> Result<Profile> {
    if let Some(path) = &globals.profile_file {
        let id = globals.profile.trim();
        return Profile::load_from_path(id, path).map_err(|e| anyhow::anyhow!("{e}"));
    }
    Profile::load_bundled(&globals.profile).map_err(|e| anyhow::anyhow!("{e}"))
}

fn regime_as_json(profile: &Profile) -> Value {
    let r = &profile.regime;
    json!({
        "w_c_min": r.w_c_min,
        "w_c_max": r.w_c_max,
        "temperature_k_min": r.temperature_k_min,
        "temperature_k_max": r.temperature_k_max,
        "age_hours_min": r.age_hours_min,
        "age_hours_max": r.age_hours_max,
        "fly_ash_pct_max": r.fly_ash_pct_max,
        "silica_fume_pct_max": r.silica_fume_pct_max,
        "slag_pct_max": r.slag_pct_max,
        "scm_sum_min_pct": r.scm_sum_min_pct,
        "silica_fume_pct_max_special": r.silica_fume_pct_max_special,
    })
}

fn print_prediction(
    profile: &Profile,
    spec: &MixSpec,
    wire: PredictionWireVersion,
    compare_homogeneous: bool,
) -> Result<()> {
    let bundle = cli::predict_with_options(
        profile,
        spec,
        cli::PredictOptions {
            compare_homogeneous,
        },
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    let out = cli::serialize_prediction(&bundle, wire).map_err(|e| anyhow::anyhow!("{e}"))?;
    let text = serde_json::to_string_pretty(&out).context("serialize prediction JSON")?;
    println!("{text}");
    Ok(())
}

fn run_optimize(profile: &Profile, base: MixSpec, target_raw: String, steps: usize) -> Result<()> {
    let (field, target_val) =
        cli::parse_optimize_target(&target_raw).map_err(|e| anyhow::anyhow!("{e}"))?;
    let tuned = cli::optimize_mix(profile, &base, field, target_val, steps)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let out = cli::serialize_mix_spec(&tuned).map_err(|e| anyhow::anyhow!("{e}"))?;
    let text = serde_json::to_string_pretty(&out).context("serialize mix JSON")?;
    println!("{text}");
    Ok(())
}

fn main() -> ExitCode {
    let root = CliRoot::parse();

    let result = match &root.command {
        Command::Schema { kind } => {
            let payload = match kind {
                SchemaCmd::Mix => MIX_SCHEMA_V1,
                SchemaCmd::Result => RESULT_SCHEMA_V1,
                SchemaCmd::ResultV2 => RESULT_SCHEMA_V2,
            };
            println!("{payload}");
            Ok(())
        }
        Command::Profiles(sub) => {
            let r = handle_profiles(sub);
            r.map_err(|e| e.to_string())
        }
        Command::Certify { name } => match Profile::load_bundled(name) {
            Ok(p) => {
                let j = cli::certify_profile_json(&p);
                println!("{}", serde_json::to_string_pretty(&j).unwrap_or_default());
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        },
        Command::Predict {
            input,
            schema_version,
            compare_homogeneous,
        } => {
            let globals = &root.globals;
            let profile_requested = globals.profile.trim();
            if globals.profile_file.is_none() && profile_requested.eq_ignore_ascii_case("default") {
                eprintln!("info: using default calibration profile 'default' (regime: generic OPC fallback)");
            }
            let profile = match load_profile(globals) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{e:#}");
                    return ExitCode::from(1);
                }
            };
            let v = match read_mix_value(input.clone()) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{e:#}");
                    return ExitCode::from(1);
                }
            };
            let spec = match MixSpec::try_from(v) {
                Ok(mut s) => {
                    if globals.profile_file.is_none() {
                        s.profile_name = globals.profile.clone();
                    } else {
                        s.profile_name = profile.bundle_id.clone();
                    }
                    s
                }
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            let wire = match schema_version {
                PredictionWireCli::V1 => PredictionWireVersion::V1,
                PredictionWireCli::V2 => PredictionWireVersion::V2,
            };
            print_prediction(&profile, &spec, wire, *compare_homogeneous).map_err(|e| e.to_string())
        }
        Command::Optimize {
            input,
            target,
            steps,
        } => {
            let globals = &root.globals;
            if globals.profile_file.is_none()
                && globals.profile.trim().eq_ignore_ascii_case("default")
            {
                eprintln!("info: using default calibration profile 'default' (regime: generic OPC fallback)");
            }
            let profile = match load_profile(globals) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{e:#}");
                    return ExitCode::from(1);
                }
            };
            let v = match read_mix_value(input.clone()) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{e:#}");
                    return ExitCode::from(1);
                }
            };
            let base = match MixSpec::try_from(v) {
                Ok(mut s) => {
                    if globals.profile_file.is_none() {
                        s.profile_name = globals.profile.clone();
                    } else {
                        s.profile_name = profile.bundle_id.clone();
                    }
                    s
                }
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            run_optimize(&profile, base, target.clone(), *steps).map_err(|e| e.to_string())
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
    }
}

fn handle_profiles(cmd: &ProfilesCmd) -> Result<()> {
    match cmd {
        ProfilesCmd::List => {
            for id in umst_concrete_cartridge::calibration::BUNDLED_PROFILE_IDS {
                let desc = umst_concrete_cartridge::calibration::profile_descriptions()
                    .get(id)
                    .copied()
                    .unwrap_or("no description");
                println!("{id}\t{desc}");
            }
            Ok(())
        }
        ProfilesCmd::Describe { name } => {
            let p = Profile::load_bundled(name).map_err(|e| anyhow::anyhow!("{e}"))?;
            let txt = match p.bundle_id.as_str() {
                "default" => include_str!("../../../calibration/profiles/default.v1.toml"),
                "uci_d1" => include_str!("../../../calibration/profiles/uci_d1.v1.toml"),
                "zenodo_ndt" => include_str!("../../../calibration/profiles/zenodo_ndt.v1.toml"),
                "zenodo_sonreb" => {
                    include_str!("../../../calibration/profiles/zenodo_sonreb.v1.toml")
                }
                "zenodo_rh" => include_str!("../../../calibration/profiles/zenodo_rh.v1.toml"),
                "uhpc" => include_str!("../../../calibration/profiles/uhpc.v1.toml"),
                "highscm" => include_str!("../../../calibration/profiles/highscm.v1.toml"),
                "selfheal" => include_str!("../../../calibration/profiles/selfheal.v1.toml"),
                _ => return Err(anyhow::anyhow!("unknown profile")),
            };
            print!("{txt}");
            Ok(())
        }
        ProfilesCmd::Regime { name } => {
            let p = Profile::load_bundled(name).map_err(|e| anyhow::anyhow!("{e}"))?;
            let j = regime_as_json(&p);
            println!("{}", serde_json::to_string_pretty(&j)?);
            Ok(())
        }
    }
}
