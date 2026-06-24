#!/usr/bin/env python3
"""Walk gate REJECT payloads with explain:true (rational parse + thermodynamic fail)."""

from __future__ import annotations

import json

from mcp_rpc import call_tool, initialize, parse_tool_text, repo_root, spawn_mcp


def assert_explain_shape(body: dict, *, expect_violation: str) -> None:
    explain = body.get("explain", {})
    violations = explain.get("regime_violations", [])
    remediation = explain.get("remediation", [])
    fields = explain.get("fields", [])
    assert violations, f"expected regime_violations, got {body}"
    assert expect_violation in violations, violations
    assert remediation, "expected remediation strings"
    assert len(remediation) >= len(violations), remediation
    assert fields, "expected explain.fields"
    gate_reject = body.get("gate_reject", {})
    assert gate_reject.get("schema_version") == "gate_reject.v1", gate_reject
    assert gate_reject.get("verdict") == "REJECT", gate_reject


def main() -> int:
    reject_fixture = repo_root() / "fixtures" / "golden-adversarial" / "reject_mix_01.json"
    reject_mix = json.loads(reject_fixture.read_text())["mix_spec"]

    proc = spawn_mcp()
    try:
        initialize(proc)

        parse_fail = call_tool(
            proc,
            50,
            "umst_gate_check",
            {
                "mix": {"w_c": "not-rational", "temperature_k": "29315/100"},
                "explain": True,
            },
        )
        assert parse_fail.get("isError") is True, parse_fail
        parse_body = parse_tool_text(parse_fail)
        assert_explain_shape(parse_body, expect_violation="mix_spec_rational_parse_fail")
        print("parse_fail violations:", parse_body["explain"]["regime_violations"])

        thermo_fail = call_tool(
            proc,
            51,
            "umst_gate_check",
            {"mix": reject_mix, "explain": True},
        )
        assert thermo_fail.get("isError") is True, thermo_fail
        thermo_body = parse_tool_text(thermo_fail)
        violations = thermo_body.get("explain", {}).get("regime_violations", [])
        assert violations, thermo_body
        print("thermo_fail violations:", violations)
        assert thermo_body.get("gate_summary", {}).get("admissible") is False

        print("05_explain_violations: ok")
        return 0
    finally:
        proc.terminate()
        proc.wait(timeout=120)


if __name__ == "__main__":
    raise SystemExit(main())
