import io
import json
import sys
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

import diag_editor_paint_contract_validate as validate
import diag_editor_paint_contract_windows_handoff as handoff


class EditorPaintContractWindowsHandoffTests(unittest.TestCase):
    def test_plan_chains_validation_attribution_verify_and_closeout(self) -> None:
        plan = handoff.build_plan(
            python_bin="python",
            date_tag="unit-date",
            out_dir="target/fret-diag/handoff-unit-date",
            skip_build=False,
            skip_preflight=False,
        )

        self.assertEqual(
            [
                "build-fretboard-dev",
                "build-fret-ui-gallery",
                "preflight",
                "baseline-validation",
                "attribution-validation",
                "verify-artifacts",
                "closeout",
            ],
            [step["name"] for step in plan],
        )

        validation_dir = validate._default_out_dir("unit-date")
        attribution_dir = validate._default_out_dir("unit-date-attrib")

        build_gallery = plan[1]["cmd"]
        baseline = plan[3]["cmd"]
        attribution = plan[4]["cmd"]
        verify_cmd = plan[5]["cmd"]
        closeout_cmd = plan[6]["cmd"]

        self.assertEqual(
            [
                "cargo",
                "build",
                "-p",
                "fret-ui-gallery",
                "--release",
                "--features",
                handoff.GALLERY_FEATURES,
            ],
            build_gallery,
        )

        self.assertIn("tools/perf/diag_editor_paint_contract_validate.py", baseline)
        self.assertIn("--date-tag", baseline)
        self.assertIn("unit-date", baseline)
        self.assertIn("--skip-preflight", baseline)
        self.assertNotIn("--with-paint-perf", baseline)

        self.assertIn("unit-date-attrib", attribution)
        self.assertIn("--with-paint-perf", attribution)
        self.assertIn("--skip-preflight", attribution)

        self.assertIn(validation_dir, verify_cmd)
        self.assertIn(attribution_dir, verify_cmd)
        self.assertIn(validation_dir, closeout_cmd)
        self.assertIn(attribution_dir, closeout_cmd)

    def test_plan_can_skip_separate_preflight(self) -> None:
        plan = handoff.build_plan(
            python_bin="python",
            date_tag="unit-date",
            out_dir="target/fret-diag/handoff-unit-date",
            skip_build=False,
            skip_preflight=True,
        )

        self.assertEqual(
            [
                "build-fretboard-dev",
                "build-fret-ui-gallery",
                "baseline-validation",
                "attribution-validation",
                "verify-artifacts",
                "closeout",
            ],
            [step["name"] for step in plan],
        )

    def test_plan_can_skip_release_builds(self) -> None:
        plan = handoff.build_plan(
            python_bin="python",
            date_tag="unit-date",
            out_dir="target/fret-diag/handoff-unit-date",
            skip_build=True,
            skip_preflight=False,
        )

        self.assertEqual(
            ["preflight", "baseline-validation", "attribution-validation", "verify-artifacts", "closeout"],
            [step["name"] for step in plan],
        )

    def test_dry_run_writes_handoff_plan(self) -> None:
        with TemporaryDirectory() as td:
            out_dir = Path(td) / "handoff"
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_windows_handoff.py",
                    "--dry-run",
                    "--date-tag",
                    "unit-date",
                    "--out-dir",
                    str(out_dir),
                    "--python-bin",
                    "python",
                ],
            ):
                with redirect_stdout(io.StringIO()):
                    rc = handoff.main()

            summary = json.loads((out_dir / "handoff-plan.json").read_text(encoding="utf-8"))

        self.assertEqual(0, rc)
        self.assertTrue(summary["dry_run"])
        self.assertEqual("editor_paint_contract_windows_handoff_plan", summary["kind"])
        self.assertEqual("unit-date", summary["date_tag"])
        self.assertEqual(
            [
                "build-fretboard-dev",
                "build-fret-ui-gallery",
                "preflight",
                "baseline-validation",
                "attribution-validation",
                "verify-artifacts",
                "closeout",
            ],
            [step["name"] for step in summary["steps"]],
        )

    def test_non_dry_run_rejects_non_windows_host_by_default(self) -> None:
        stderr = io.StringIO()
        with patch.object(sys, "platform", "darwin"):
            with patch.object(
                sys,
                "argv",
                [
                    "diag_editor_paint_contract_windows_handoff.py",
                    "--date-tag",
                    "unit-date",
                ],
            ):
                with redirect_stderr(stderr):
                    rc = handoff.main()

        self.assertEqual(2, rc)
        self.assertIn("target Windows host", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
