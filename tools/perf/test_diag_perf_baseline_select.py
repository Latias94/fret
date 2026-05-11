from __future__ import annotations

import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from diag_perf_baseline_select import _clamp_threshold_loosening, _threshold_loosening_report


def _baseline(rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "kind": "perf_baseline",
        "schema_version": 1,
        "rows": rows,
    }


def _row(
    script: str,
    thresholds: dict[str, object],
    measured_max: dict[str, object] | None = None,
) -> dict[str, object]:
    row = {
        "script": script,
        "thresholds": thresholds,
    }
    if measured_max is not None:
        row["measured_max"] = measured_max
    return row


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

    def test_clamps_looser_max_threshold_when_measurement_fits_old_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            old_path = root / "old.json"
            new_path = root / "new.json"
            old_path.write_text(
                json.dumps(_baseline([_row("script.json", {"max_top_total_us": 100})])),
                encoding="utf-8",
            )
            new_path.write_text(
                json.dumps(
                    _baseline(
                        [
                            _row(
                                "script.json",
                                {"max_top_total_us": 120},
                                {"top_total_time_us": 80},
                            )
                        ]
                    )
                ),
                encoding="utf-8",
            )

            clamps = _clamp_threshold_loosening(
                old_path=old_path,
                new_path=new_path,
                source_baseline="old.json",
            )
            report = _threshold_loosening_report(old_path, new_path)
            new_doc = json.loads(new_path.read_text(encoding="utf-8"))

        self.assertEqual(len(clamps), 1)
        self.assertEqual(new_doc["rows"][0]["thresholds"]["max_top_total_us"], 100)
        self.assertEqual(new_doc["threshold_clamp_policy"]["mode"], "no_threshold_loosening")
        self.assertEqual(report, [])

    def test_does_not_clamp_max_threshold_when_measurement_exceeds_old_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            old_path = root / "old.json"
            new_path = root / "new.json"
            old_path.write_text(
                json.dumps(_baseline([_row("script.json", {"max_top_total_us": 100})])),
                encoding="utf-8",
            )
            new_path.write_text(
                json.dumps(
                    _baseline(
                        [
                            _row(
                                "script.json",
                                {"max_top_total_us": 120},
                                {"top_total_time_us": 110},
                            )
                        ]
                    )
                ),
                encoding="utf-8",
            )

            clamps = _clamp_threshold_loosening(
                old_path=old_path,
                new_path=new_path,
                source_baseline="old.json",
            )
            report = _threshold_loosening_report(old_path, new_path)

        self.assertEqual(clamps, [])
        self.assertEqual(
            {(item["threshold"], item["reason"]) for item in report},
            {("max_top_total_us", "max_increased")},
        )


if __name__ == "__main__":
    unittest.main()
