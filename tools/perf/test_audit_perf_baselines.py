from __future__ import annotations

import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from audit_perf_baselines import audit_baseline


def _metric_row(script: str, payload: bool = False) -> dict[str, object]:
    measured = {
        "top_total_time_us": 120,
        "top_layout_time_us": 20,
        "top_layout_engine_solve_time_us": 0,
    }
    thresholds = {
        "max_top_total_us": 180,
        "max_top_layout_us": 40,
        "max_top_layout_engine_solve_us": 0,
    }
    if payload:
        measured.update(
            {
                "renderer_instance_bytes": 2048,
                "renderer_encode_scene_text_ops": 32,
            }
        )
        thresholds.update(
            {
                "max_renderer_instance_bytes": 4096,
                "max_renderer_encode_scene_text_ops": 64,
            }
        )
    return {
        "script": script,
        "measured_p50": dict(measured),
        "measured_p90": dict(measured),
        "measured_p95": dict(measured),
        "measured_max": dict(measured),
        "threshold_seed": dict(measured),
        "thresholds": thresholds,
    }


def _baseline(threshold_surface: str, rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "kind": "perf_baseline",
        "schema_version": 1,
        "repeat": 7,
        "threshold_surface": threshold_surface,
        "rows": rows,
    }


class PerfBaselineAuditTests(unittest.TestCase):
    def test_ui_baseline_does_not_require_renderer_payload_metrics(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "ui-resize-probes.windows-rtx4090.v2.json"
            path.write_text(
                json.dumps(_baseline("ui", [_metric_row("resize.json")])),
                encoding="utf-8",
            )

            report = audit_baseline(path)

        self.assertEqual(report.missing_fields, ())
        self.assertEqual(report.missing_payload_fields, ())

    def test_payload_surface_requires_measured_seed_and_threshold_payload_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "editor-paint.windows-rtx4090.v1.json"
            row = _metric_row("editor.json", payload=True)
            del row["measured_p95"]["renderer_instance_bytes"]  # type: ignore[index]
            del row["threshold_seed"]["renderer_encode_scene_text_ops"]  # type: ignore[index]
            del row["thresholds"]["max_renderer_instance_bytes"]  # type: ignore[index]
            path.write_text(
                json.dumps(_baseline("ui-renderer-payload", [row])),
                encoding="utf-8",
            )

            report = audit_baseline(path)

        self.assertEqual(report.missing_fields, ())
        self.assertEqual(
            report.missing_payload_fields,
            (
                "editor.json.measured_p95.renderer_instance_bytes",
                "editor.json.threshold_seed.renderer_encode_scene_text_ops",
                "editor.json.thresholds.max_renderer_instance_bytes",
            ),
        )

    def test_payload_surface_accepts_complete_renderer_payload_contract(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "editor-paint.windows-rtx4090.v1.json"
            path.write_text(
                json.dumps(
                    _baseline("ui-renderer-payload", [_metric_row("editor.json", payload=True)])
                ),
                encoding="utf-8",
            )

            report = audit_baseline(path)

        self.assertEqual(report.missing_fields, ())
        self.assertEqual(report.missing_payload_fields, ())


if __name__ == "__main__":
    unittest.main()
