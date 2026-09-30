# SPDX-License-Identifier: MIT

"""Parity: ``audit_rows([...])`` matches ``audit(csv_string)``."""

import csv
import io
import json
import unittest


class TestAuditRows(unittest.TestCase):
    """Compare JSON from audit_rows vs audit on the same tabular data."""

    def test_two_rows_match_roundtrip_csv(self) -> None:
        try:
            from umst_concrete_cartridge import audit, audit_rows
        except ImportError:
            self.skipTest("extension not installed")

        rows_list = [
            {
                "cement": 540.0,
                "slag": 0.0,
                "fly_ash": 0.0,
                "water": 162.0,
                "superplasticizer": 2.5,
                "coarse_agg": 1040.0,
                "fine_agg": 676.0,
                "age": 28.0,
                "strength": 79.98611076,
                "source": "D1",
                "temperature": 21.0,
                "humidity": 0.49,
            },
            {
                "cement": 332.5,
                "slag": 142.5,
                "fly_ash": 0.0,
                "water": 228.0,
                "superplasticizer": 0.0,
                "coarse_agg": 932.0,
                "fine_agg": 594.0,
                "age": 270.0,
                "strength": 40.269535256000005,
                "source": "D1",
                "temperature": 21.3,
                "humidity": 0.52,
            },
        ]
        buf = io.StringIO()
        keys = list(rows_list[0].keys())
        w = csv.DictWriter(buf, fieldnames=keys)
        w.writeheader()
        for r in rows_list:
            w.writerow(r)
        csv_text = buf.getvalue()

        a = json.dumps(audit("uci_d1", csv_text, None), sort_keys=True)
        b = json.dumps(audit_rows(rows_list, profile="uci_d1", limit=None), sort_keys=True)
        self.assertEqual(a, b)


if __name__ == "__main__":
    unittest.main()
