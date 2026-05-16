import io
import unittest
from tempfile import TemporaryDirectory
from pathlib import Path
import json
import sys
from contextlib import redirect_stderr, redirect_stdout
from unittest.mock import patch

import diag_editor_paint_contract_validate as validate


class EditorPaintContractValidateTests(unittest.TestCase):
    def test_default_plan_uses_windows_contract_surfaces(self) -> None:
        plan = validate.build_plan(
            python_bin="python",
            fretboard_bin="target/release/fretboard-dev.exe",
            launch_cmd=validate._default_launch_cmd(),
            out_dir="target/fret-diag/editor-paint-contract-validate-test",
            resize_attempts=3,
            resize_repeat=7,
            typical_repeat=15,
            complex_repeat=7,
            warmup_frames=5,
            skip_preflight=False,
            with_paint_perf=False,
        )

        self.assertEqual(
            ["preflight", "resize-jitter", "typical-autoscroll", "complex-wheel"],
            [step["name"] for step in plan],
        )

        joined = "\n".join(" ".join(step["cmd"]) for step in plan)
        self.assertIn(validate.RESIZE_BASELINE, joined)
        self.assertIn("--fretboard-bin target/release/fretboard-dev.exe", joined)
        self.assertIn(validate.TYPICAL_BASELINE, joined)
        self.assertIn(validate.COMPLEX_WHEEL_BASELINE, joined)
        self.assertIn("FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0", joined)
        self.assertIn("--launch-cmd cargo run -p fret-ui-gallery --release --features gallery-full", joined)
        self.assertIn("--launch -- cargo run -p fret-ui-gallery --release --features gallery-full", joined)
        self.assertNotIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[2]["cmd"]))
        self.assertNotIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[3]["cmd"]))

    def test_paint_perf_flag_only_marks_non_resize_diag_perf_steps(self) -> None:
        plan = validate.build_plan(
            python_bin="python",
            fretboard_bin="target/release/fretboard-dev.exe",
            launch_cmd=validate._default_launch_cmd(),
            out_dir="target/fret-diag/editor-paint-contract-validate-test",
            resize_attempts=3,
            resize_repeat=7,
            typical_repeat=15,
            complex_repeat=7,
            warmup_frames=5,
            skip_preflight=True,
            with_paint_perf=True,
        )

        self.assertEqual(["resize-jitter", "typical-autoscroll", "complex-wheel"], [step["name"] for step in plan])
        self.assertNotIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[0]["cmd"]))
        self.assertIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[1]["cmd"]))
        self.assertIn("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1", " ".join(plan[2]["cmd"]))

    def test_launch_cmd_from_args_defaults_to_inspectable_gallery_full(self) -> None:
        launch_cmd, launch_bin_path = validate._launch_cmd_from_args(
            Path("F:/repo"),
            type("Args", (), {"launch_cmd": "", "launch_bin": ""})(),
        )

        self.assertEqual(validate._default_launch_cmd(), launch_cmd)
        self.assertIsNone(launch_bin_path)

    def test_launch_cmd_from_args_preserves_legacy_launch_bin(self) -> None:
        launch_cmd, launch_bin_path = validate._launch_cmd_from_args(
            Path("F:/repo"),
            type("Args", (), {"launch_cmd": "", "launch_bin": "target/release/fret-ui-gallery.exe"})(),
        )

        self.assertEqual(["F:/repo/target/release/fret-ui-gallery.exe"], [token.replace("\\", "/") for token in launch_cmd])
        self.assertEqual("F:/repo/target/release/fret-ui-gallery.exe", str(launch_bin_path).replace("\\", "/"))

    def test_artifact_summary_prefers_threshold_bundle(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            (root / "check.perf_thresholds.json").write_text(
                json.dumps(
                    {
                        "failures": [],
                        "layout_perf_summary": {
                            "bundle_artifact": "target/fret-diag/run/worst/bundle.schema2.json"
                        },
                    }
                ),
                encoding="utf-8",
            )

            summary = validate.artifact_summary_for_step(root)

        self.assertEqual(0, summary["check_perf_thresholds_failures"])
        self.assertEqual("target/fret-diag/run/worst/bundle.schema2.json", summary["worst_bundle"])
        self.assertTrue(validate.thresholds_pass_for_artifacts(summary))

    def test_thresholds_pass_requires_empty_failures(self) -> None:
        self.assertFalse(validate.thresholds_pass_for_artifacts({"check_perf_thresholds_failures": None}))
        self.assertFalse(validate.thresholds_pass_for_artifacts({"check_perf_thresholds_failures": 1}))
        self.assertTrue(validate.thresholds_pass_for_artifacts({"check_perf_thresholds_failures": 0}))

    def test_artifact_summary_falls_back_to_regression_summary(self) -> None:
        with TemporaryDirectory() as td:
            root = Path(td)
            (root / "regression.summary.json").write_text(
                json.dumps(
                    {
                        "items": [
                            {
                                "evidence": {
                                    "bundle_artifact": "target/fret-diag/run/from-regression/bundle.schema2.json"
                                }
                            }
                        ]
                    }
                ),
                encoding="utf-8",
            )

            summary = validate.artifact_summary_for_step(root)

        self.assertIsNone(summary["check_perf_thresholds_failures"])
        self.assertEqual("target/fret-diag/run/from-regression/bundle.schema2.json", summary["worst_bundle"])

    def test_stats_coverage_tracks_closeout_field_groups(self) -> None:
        coverage = validate.stats_coverage_for_doc(
            {
                "p95": {
                    "paint_widget_time_us": 10,
                    "renderer_prepare_text_us": 11,
                    "renderer_encode_scene_us": 12,
                    "renderer_upload_us": 13,
                },
                "code_editor_paint_perf": {
                    "max": {"us_torture_overlay": 0},
                    "p95": {"us_total": 14},
                },
            }
        )

        self.assertEqual(
            {
                "paint_widget": True,
                "renderer_text_encode_upload": True,
                "code_editor_paint_perf": True,
                "code_editor_torture_overlay_zero": True,
            },
            coverage,
        )

    def test_stats_coverage_reports_missing_groups(self) -> None:
        coverage = validate.stats_coverage_for_doc({"p95": {"paint_widget_time_us": 10}})

        self.assertTrue(coverage["paint_widget"])
        self.assertFalse(coverage["renderer_text_encode_upload"])
        self.assertFalse(coverage["code_editor_paint_perf"])
        self.assertFalse(coverage["code_editor_torture_overlay_zero"])

    def test_stats_coverage_rejects_torture_overlay_work(self) -> None:
        coverage = validate.stats_coverage_for_doc(
            {
                "p95": {
                    "paint_widget_time_us": 10,
                    "renderer_prepare_text_us": 11,
                    "renderer_encode_scene_us": 12,
                    "renderer_upload_us": 13,
                },
                "code_editor_paint_perf": {
                    "max": {"us_torture_overlay": 1},
                    "p95": {"us_total": 14},
                },
            }
        )

        self.assertTrue(coverage["code_editor_paint_perf"])
        self.assertFalse(coverage["code_editor_torture_overlay_zero"])

    def test_dry_run_summary_records_date_tag(self) -> None:
        with TemporaryDirectory() as td:
            out_dir = Path(td) / "validation"
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_validate.py",
                    "--dry-run",
                    "--date-tag",
                    "unit-date",
                    "--out-dir",
                    str(out_dir),
                ],
            ):
                with redirect_stdout(io.StringIO()):
                    rc = validate.main()

            summary = json.loads((out_dir / "validation-plan.json").read_text(encoding="utf-8"))

        self.assertEqual(0, rc)
        self.assertEqual("unit-date", summary["date_tag"])
        self.assertEqual(validate._default_launch_cmd(), summary["launch_cmd"])
        self.assertIsNone(summary["launch_bin"])

    def test_non_dry_run_rejects_non_empty_out_dir_by_default(self) -> None:
        with TemporaryDirectory() as td:
            out_dir = Path(td) / "validation"
            out_dir.mkdir()
            (out_dir / "validation-plan.json").write_text("{}", encoding="utf-8")
            stderr = io.StringIO()
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_validate.py",
                    "--allow-non-windows",
                    "--out-dir",
                    str(out_dir),
                ],
            ):
                with patch.object(validate, "_validate_inputs", return_value=[]):
                    with redirect_stderr(stderr):
                        rc = validate.main()

        self.assertEqual(2, rc)
        self.assertIn("already exists and is not empty", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
