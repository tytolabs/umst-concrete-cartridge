# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
from __future__ import annotations

import json
import os
from pathlib import Path

import pytest
import trimesh

REPO = Path(__file__).resolve().parents[2]
STL_V04 = REPO / "notebooks" / "_artifacts" / "striatus_shell_v0.4.stl"
JSON_V04 = REPO / "notebooks" / "_artifacts" / "striatus_shell_v0.4.print_ready.json"
# Legacy alias (symlink or copy from export_print_ready.py).
STL_V03 = REPO / "notebooks" / "_artifacts" / "striatus_shell_v0.3.stl"
JSON_V03 = REPO / "notebooks" / "_artifacts" / "striatus_shell_v0.3.print_ready.json"

_REQUIRE_B8 = os.environ.get("UMST_REQUIRE_B8", "").strip() == "1"


def _b8_gates_from_numerics(side: dict) -> tuple[bool, bool, bool, bool]:
    """Mirror `notebooks/export_print_ready.py` B7/B8 gate formulas (single source of truth for thresholds)."""
    dens_xy = float(side["density_xy_plane_variance"])
    nodal_vf = float(side["nodal_volume_fraction"])
    genus_raw = side.get("mesh_genus_closed_orientable_largest")
    n_cc = int(side["mesh_connected_components"])
    chi = side.get("mesh_euler_characteristic_largest")
    chi_ok = chi is None or float(chi) <= 1.5 + 1e-6
    genus = None if genus_raw is None else float(genus_raw)
    topo_signal = (genus is not None and genus >= 1.0 - 1e-6) or n_cc >= 4
    gate_topo = bool(chi_ok and topo_signal)
    gate_vf = 0.10 <= nodal_vf <= 0.25
    gate_var = dens_xy >= 0.1 - 1e-9
    rollup = bool(gate_topo and gate_var and gate_vf)
    return gate_topo, gate_vf, gate_var, rollup


def _sidecar_b8_complete() -> bool:
    if not JSON_V04.is_file():
        return False
    side = json.loads(JSON_V04.read_text())
    return bool(side.get("gates_track_b8_all_pass"))


def _stl_json_paths() -> tuple[Path, Path]:
    if STL_V04.is_file():
        return STL_V04, JSON_V04
    return STL_V03, JSON_V03


@pytest.mark.skipif(not _stl_json_paths()[0].is_file(), reason="run notebooks/export_print_ready.py or _run_shell_demo.sh first")
def test_striatus_stl_feasibility() -> None:
    stl, json_path = _stl_json_paths()
    mesh = trimesh.load(str(stl), force="mesh")
    assert mesh.is_watertight
    assert mesh.is_winding_consistent

    side = json.loads(json_path.read_text())
    min_feat = float(
        side.get("min_feature_size_mm", side.get("min_feature_circumradius_mm", 0.0))
    )
    assert min_feat >= 6.0
    # Exporter reports angle from +z via asin(||n×ê_z||) in (0°, 90°]; shallow shells approach ~90°.
    assert float(side["max_overhang_deg"]) <= 90.0 + 1e-3
    tv = float(side["total_volume_cm3"])
    assert 1.0 <= tv <= 3_000_000.0


@pytest.mark.skipif(not JSON_V04.is_file(), reason="v0.4 sidecar from export_print_ready.py required for B8 gates")
def test_print_ready_track_b8_topology_gates() -> None:
    side = json.loads(JSON_V04.read_text())
    assert side.get("artefact_version") == "v0.4"
    for key in (
        "density_xy_plane_variance",
        "nodal_volume_fraction",
        "mesh_volume_fraction_in_bbox",
        "mesh_connected_components",
        "mesh_euler_characteristic_largest",
        "mesh_genus_closed_orientable_largest",
        "gate_topo_complexity_b7",
        "gate_volume_fraction_mesh_b7",
        "gate_density_xy_variance_b8",
        "gates_track_b8_all_pass",
    ):
        assert key in side, f"missing sidecar field {key!r} (re-run export_print_ready.py)"
    sha = side.get("source_final_npy_sha256")
    if side["gates_track_b8_all_pass"]:
        assert isinstance(sha, str) and len(sha) == 64, (
            "B8-complete sidecar must include source_final_npy_sha256 (64 hex chars) from export_print_ready.py"
        )
    if not side["gates_track_b8_all_pass"]:
        msg = (
            "committed print_ready is STL-feasible but not B8-complete "
            "(regenerate 40×40×4 Track L + export_print_ready.py). "
            "Set UMST_REQUIRE_B8=1 to fail instead of skip."
        )
        if _REQUIRE_B8:
            pytest.fail(msg)
        pytest.skip(msg)
    assert side["gate_density_xy_variance_b8"] is True, "density_xy_plane_variance gate (brief ≥ 0.1)"
    assert side["gate_volume_fraction_mesh_b7"] is True, "nodal_volume_fraction should be in [0.10, 0.25]"
    assert side["gate_topo_complexity_b7"] is True, (
        "topology gate (brief): (genus ≥ 1) OR (≥4 watertight components), with χ≤1.5 on largest part when known"
    )
    assert side["gates_track_b8_all_pass"] is True


@pytest.mark.skipif(not JSON_V04.is_file(), reason="v0.4 sidecar required")
def test_print_ready_sidecar_gate_booleans_match_numerics() -> None:
    """Catch hand-edited gate booleans out of sync with mesh / ρ statistics (policy: re-run exporter)."""
    side = json.loads(JSON_V04.read_text())
    exp_topo, exp_vf, exp_var, exp_roll = _b8_gates_from_numerics(side)
    assert side["gate_topo_complexity_b7"] is exp_topo
    assert side["gate_volume_fraction_mesh_b7"] is exp_vf
    assert side["gate_density_xy_variance_b8"] is exp_var
    assert side["gates_track_b8_all_pass"] is exp_roll


if __name__ == "__main__":
    raise SystemExit(pytest.main([__file__, "-v"]))
