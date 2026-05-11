from __future__ import annotations

import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from diag_perf_baseline_select import _threshold_loosening_report


def _baseline(rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "kind": "perf_baseline",
        "schema_version": 1,
        "rows": rows,
    }


def _row(script: str, thresholds: dict[str, object]) -> dict[str, object]:
    return {
        "script": script,
        "thresholds": thresholds,
    }


class ThresholdLooseningReportTests(unittest.TestCase):
    def test_reports_max_increase_and_min_decrease(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            old_path = root / "old.json"
            new_path = root / "new.json"
            old_path.write_text(
                json.dumps(
                    _baseline(
                        [
                            _row(
                                "script.json",
                                {
                                    "max_top_total_us": 100,
                                    "min_cache_hits": 5,
                                },
                            )
                        ]
                    )
                ),
                encoding="utf-8",
            )
            new_path.write_text(
                json.dumps(
                    _baseline(
                        [
                            _row(
                                "script.json",
                                {
                                    "max_top_total_us": 120,
                                    "min_cache_hits": 4,
                                },
                            )
                        ]
                    )
                ),
                encoding="utf-8",
            )

            report = _threshold_loosening_report(old_path, new_path)

        self.assertEqual(
            {(item["threshold"], item["reason"]) for item in report},
            {
                ("max_top_total_us", "max_increased"),
                ("min_cache_hits", "min_decreased"),
            },
        )

    def test_reports_threshold_and_row_removal(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            old_path = root / "old.json"
            new_path = root / "new.json"
            old_path.write_text(
                json.dumps(
                    _baseline(
                        [
                            _row("kept.json", {"max_top_total_us": 100}),
                            _row("removed.json", {"max_top_total_us": 100}),
                        ]
                    )
                ),
                encoding="utf-8",
            )
            new_path.write_text(
                json.dumps(_baseline([_row("kept.json", {})])),
                encoding="utf-8",
            )

            report = _threshold_loosening_report(old_path, new_path)

        self.assertEqual(
            {(item["threshold"], item["reason"]) for item in report},
            {
                ("max_top_total_us", "threshold_removed"),
                ("*", "row_removed"),
            },
        )

    def test_ignores_previously_non_gated_null_threshold(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            old_path = root / "old.json"
            new_path = root / "new.json"
            old_path.write_text(
                json.dumps(
                    _baseline([_row("script.json", {"max_renderer_encode_scene_us": None})])
                ),
                encoding="utf-8",
            )
            new_path.write_text(
                json.dumps(
                    _baseline([_row("script.json", {"max_renderer_encode_scene_us": 400})])
                ),
                encoding="utf-8",
            )

            report = _threshold_loosening_report(old_path, new_path)

        self.assertEqual(report, [])


if __name__ == "__main__":
    unittest.main()
