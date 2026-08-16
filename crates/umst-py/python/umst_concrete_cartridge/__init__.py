# SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
# SPDX-License-Identifier: MIT
"""Python bindings: `predict`, `audit`, `audit_rows`, `certify`, `audit_dataframe`, schemas, `canonical_json`.

`predict(spec, *, profile="default", schema_version="v2")` matches the v0.2
surface: mix/spec dict first, keyword-only profile and schema version.
Canonical JSON bytes match `umst predict` piped through `umst-canonical`.
"""

from __future__ import annotations

from typing import Any, Optional

from ._umst_concrete_cartridge import (
    audit,
    audit_rows,
    bundled_profile_ids,
    canonical_json,
    certify,
    predict,
    schema,
)


def audit_dataframe(
    df: Any,
    *,
    profile: str = "default",
    observed_column: str = "strength",
    limit: Optional[int] = None,
) -> Any:
    """Run :func:`audit` over a pandas ``DataFrame``.

    Returns a ``DataFrame`` mirroring ``audit.v1`` ``rows`` with the nested ``input``
    map flattened as ``input_*`` columns. ``summary`` is stored in ``.attrs[\"summary\"]``.

    Requires pandas (install via ``pip install umst-concrete-cartridge[notebook]``).
    """
    try:
        import pandas as pd
    except ImportError as exc:
        raise ImportError(
            "audit_dataframe requires pandas (pip install umst-concrete-cartridge[notebook])"
        ) from exc

    work = df
    cols = list(work.columns)
    if observed_column != "strength" and observed_column in cols:
        work = work.rename(columns={observed_column: "strength"})

    csv_text = work.to_csv(index=False)
    raw = audit(profile, csv_text, limit)
    rows = raw.get("rows") or []
    flat: list[dict[str, Any]] = []
    for row in rows:
        out: dict[str, Any] = {}
        for k, v in row.items():
            if k == "input" and isinstance(v, dict):
                for ik, iv in v.items():
                    out[f"input_{ik}"] = iv
            else:
                out[k] = v
        flat.append(out)
    out_df = pd.DataFrame.from_records(flat)
    out_df.attrs["summary"] = raw.get("summary", {})
    return out_df


__all__ = [
    "audit",
    "audit_rows",
    "audit_dataframe",
    "bundled_profile_ids",
    "canonical_json",
    "certify",
    "predict",
    "schema",
]
