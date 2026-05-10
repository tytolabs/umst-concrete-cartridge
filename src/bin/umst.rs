// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar,
// Santosh Prabhu Shenbagamoorthy — Studio TYTO

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::Value;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use umst_concrete_cartridge::cli::{self, MixSpec};

const MIX_SCHEMA_V1: &str = include_str!("../../schema/mix.v1.json");
const RESULT_SCHEMA_V1: &str = include_str!("../../schema/result.v1.json");

#[derive(Parser)]
#[command(
    name = "umst",
    version,
    about = "UMST concrete cartridge CLI (optional feature build)"
)]
struct CliRoot {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Predict constitutive scalars from a mix JSON payload on stdin or via --input.
    Predict {
        #[arg(long, value_name = "FILE")]
        input: Option<PathBuf>,
    },
    /// Adjust water-cement ratio to approach a scalar target (bisection on w/c).
    Optimize {
        #[arg(long)]
        target: String,
        #[arg(long, default_value_t = 24)]
        steps: usize,
        #[arg(long, value_name = "FILE")]
        input: Option<PathBuf>,
    },
    /// Print an embedded JSON Schema (draft 2020-12) for tooling.
    Schema {
        #[arg(value_enum)]
        kind: SchemaKind,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SchemaKind {
    Mix,
    Result,
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

fn print_prediction(spec: &MixSpec) -> Result<()> {
    let pr = cli::predict(spec).map_err(|e| anyhow::anyhow!("{e}"))?;
    let out = cli::serialize_prediction(&pr).map_err(|e| anyhow::anyhow!("{e}"))?;
    let text = serde_json::to_string_pretty(&out).context("serialize prediction JSON")?;
    println!("{text}");
    Ok(())
}

fn run_optimize(base: MixSpec, target_raw: String, steps: usize) -> Result<()> {
    let (field, target_val) =
        cli::parse_optimize_target(&target_raw).map_err(|e| anyhow::anyhow!("{e}"))?;
    let tuned =
        cli::optimize_mix(&base, field, target_val, steps).map_err(|e| anyhow::anyhow!("{e}"))?;
    let out = cli::serialize_mix_spec(&tuned).map_err(|e| anyhow::anyhow!("{e}"))?;
    let text = serde_json::to_string_pretty(&out).context("serialize mix JSON")?;
    println!("{text}");
    Ok(())
}

fn main() -> ExitCode {
    let result = match CliRoot::parse().command {
        Command::Predict { input } => {
            let v = match read_mix_value(input) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{e:#}");
                    return ExitCode::from(1);
                }
            };
            let spec = match MixSpec::try_from(v) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            print_prediction(&spec).map_err(|e| e.to_string())
        }
        Command::Optimize {
            input,
            target,
            steps,
        } => {
            let v = match read_mix_value(input) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{e:#}");
                    return ExitCode::from(1);
                }
            };
            let base = match MixSpec::try_from(v) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    return ExitCode::from(1);
                }
            };
            run_optimize(base, target, steps).map_err(|e| e.to_string())
        }
        Command::Schema { kind } => {
            let payload = match kind {
                SchemaKind::Mix => MIX_SCHEMA_V1,
                SchemaKind::Result => RESULT_SCHEMA_V1,
            };
            println!("{payload}");
            Ok(())
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
