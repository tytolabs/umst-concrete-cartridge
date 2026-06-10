# V1 — tyto_mortar prediction bundle (Track V)

**Profile:** [`profiles/tyto_mortar.v1.toml`](../profiles/tyto_mortar.v1.toml)  
**Manifold catalog digest:** `ef0ed071fc82bf8ebc8971aeee8d142b4b54e15583f0c575d942cb237474d1dc`

## Contents

| File | Role |
|------|------|
| `input_mix.json` | Mix spec wire input |
| `prediction.json` | **`tyto_mortar` profile** via `umst predict --profile tyto_mortar` |
| `prediction_default_profile.json` | Same mix with bundled `default` profile (sanity / contrast) |
| `gate_verdicts.json` | Admissibility witness record |
| `provenance.json` | Commands + sanity notes |

## Reproduce

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cd umst-concrete-cartridge
cargo build -p umst-cli
./target/debug/umst predict --profile tyto_mortar \
  --input calibration/v1_tyto_mortar_prediction_bundle/input_mix.json \
  --schema-version v2 > calibration/v1_tyto_mortar_prediction_bundle/prediction.json
```

## Sanity note (default profile)

`prediction_default_profile.json` used the generic OPC `default` profile and reported **~53.8 MPa @ 24h** for the same mix — **physically high for mortar**. With `--profile tyto_mortar`, the same input yields **~28 MPa** (Powers gel-space params from `tyto_mortar.v1.toml`). Open calibration review if default profile is used on mortar-like mixes without explicit profile selection.

## V3 pipeline rehearsal (not validation)

[`../v3_pipeline_rehearsal/`](../v3_pipeline_rehearsal/) — placeholder data only; **V3-gate: CLOSED**.
